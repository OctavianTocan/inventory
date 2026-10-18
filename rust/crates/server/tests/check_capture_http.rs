//! Independent executable assertions from `tools/check-capture-http.mjs`; Stage A goldens stay unchanged.

use std::cell::Cell;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder};
use mm_core::capture_http::{capture_destination, is_blocked, public_address};
use mm_core::capture_link::{CaptureOptions, capture_link};
use mm_core::host::{
    AnalysisImage, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, ResourceOptions,
    Screenshot,
};
use mm_core::http::Response;
use mm_core::{Result, Value, obj};
use mm_js::base64;
use mymind::capture_http::{CaptureTransport, Connector};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

// These are the original synthetic fixtures compressed with Node's default Brotli options.
const BR_BOMB: &str = "W///j38CIB4LBHLvHwA=";
const BR_HELLO: &str = "CwKAaGVsbG8D";

struct Fixture {
    port: u16,
    requests: Arc<Mutex<Vec<String>>>,
    chunks: Arc<std::sync::atomic::AtomicUsize>,
}

// Encode a synthetic response in the original compression formats without a Node runtime dependency.
fn compressed(encoding: &str, bytes: &[u8]) -> Vec<u8> {
    match encoding {
        "gzip" => {
            let mut out = GzEncoder::new(Vec::new(), Compression::default());
            out.write_all(bytes).unwrap();
            out.finish().unwrap()
        }
        "deflate" => {
            let mut out = ZlibEncoder::new(Vec::new(), Compression::default());
            out.write_all(bytes).unwrap();
            out.finish().unwrap()
        }
        "br" if bytes == b"hello" => base64::decode(BR_HELLO),
        "br" => {
            assert_eq!(bytes, vec![0u8; 1024 * 1024]);
            base64::decode(BR_BOMB)
        }
        _ => unreachable!(),
    }
}

// Write one closed raw HTTP response to the synthetic loopback request.
fn reply(stream: &mut impl Write, status: u16, headers: &[(&str, &str)], body: &[u8]) {
    let mut head = format!("HTTP/1.1 {status} Fixture\r\nconnection: close\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if !headers.iter().any(|(name, _)| *name == "content-length") {
        head.push_str(&format!("content-length: {}\r\n", body.len()));
    }
    head.push_str("\r\n");
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

// Answer redirects, policy-sensitive HTML, streaming limits and decoded-body fixtures on loopback only.
fn handle(
    mut stream: impl Read + Write,
    requests: Arc<Mutex<Vec<String>>>,
    chunks: Arc<std::sync::atomic::AtomicUsize>,
) {
    let mut buffer = [0u8; 8192];
    let mut request = Vec::new();
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let Ok(n) = stream.read(&mut buffer) else {
            return;
        };
        if n == 0 {
            return;
        }
        request.extend_from_slice(&buffer[..n]);
    }
    let path = String::from_utf8_lossy(&request)
        .split(' ')
        .nth(1)
        .unwrap_or("/")
        .to_owned();
    requests
        .lock()
        .unwrap()
        .push(String::from_utf8(request).unwrap());
    if path == "/wire" {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n",
        );
        for _ in 0..100 {
            if stream
                .write_all(&[b"2000\r\n".as_slice(), vec![0u8; 8192].as_slice(), b"\r\n"].concat())
                .is_err()
            {
                return;
            }
            chunks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(2));
        }
        let _ = stream.write_all(b"0\r\n\r\n");
        return;
    }
    if let Some(target) = path.strip_prefix("/redirect/") {
        let target = match target {
            "private" => "http://10.0.0.1/",
            "numeric" => "http://134744072/",
            "ipv6" => "http://[fd00::1]/",
            "account" => "https://access.mymind.com/",
            "again" => "/fixture",
            _ => unreachable!(),
        };
        reply(&mut stream, 302, &[("location", target)], b"");
        return;
    }
    if path == "/constructor" {
        reply(
            &mut stream,
            200,
            &[("content-encoding", "constructor")],
            b"fixture",
        );
        return;
    }
    if let Some((encoding, variant)) = path.strip_prefix('/').and_then(|path| path.split_once('-'))
        && ["gzip", "deflate", "br"].contains(&encoding)
    {
        let bytes = if variant == "bomb" {
            vec![0u8; 1024 * 1024]
        } else {
            b"hello".to_vec()
        };
        let body = compressed(encoding, &bytes);
        reply(
            &mut stream,
            200,
            &[
                ("content-encoding", encoding),
                ("content-length", &body.len().to_string()),
            ],
            &body,
        );
        return;
    }
    let html = match path.as_str() {
        "/og-private" => "<meta property=\"og:image\" content=\"http://10.0.0.1/image.png\">",
        "/og-numeric" => "<meta property=\"og:image\" content=\"http://134744072/image.png\">",
        "/og-relative-numeric" => "<meta property=\"og:image\" content=\"//134744072/image.png\">",
        "/jsonld" => {
            "<script type=\"application/ld+json\">{\"@type\":\"Article\",\"image\":\"http://[::ffff:127.0.0.1]/image.png\"}</script>"
        }
        "/oembed" => "<link type=\"application/json+oembed\" href=\"http://100.64.0.1/oembed\">",
        "/thumbnail" => "<link type=\"application/json+oembed\" href=\"/embed\">",
        "/embed" => "{\"thumbnail_url\":\"http://[fe80::1]/image.png\"}",
        "/budget" => "first resource",
        _ => "fixture",
    };
    reply(
        &mut stream,
        200,
        &[("content-type", "text/html")],
        html.as_bytes(),
    );
}

impl Fixture {
    // Start an isolated fixture listener and discard it when the test process exits.
    fn start() -> Self {
        Self::start_with_tls(None)
    }

    // Start a loopback-only HTTP or HTTPS listener with the same fixture responses.
    fn start_with_tls(tls: Option<Arc<rustls::ServerConfig>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let chunks = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (seen, sent) = (requests.clone(), chunks.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (seen, sent) = (seen.clone(), sent.clone());
                let tls = tls.clone();
                std::thread::spawn(move || {
                    if let Some(tls) = tls {
                        let connection = rustls::ServerConnection::new(tls).unwrap();
                        handle(rustls::StreamOwned::new(connection, stream), seen, sent);
                    } else {
                        handle(stream, seen, sent);
                    }
                });
            }
        });
        Self {
            port,
            requests,
            chunks,
        }
    }

    // Generate target-local TLS material for public.invalid and retain normal client certificate verification.
    fn https() -> (Self, Arc<rustls::ClientConfig>) {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("check-capture-http-tls-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let key_pem = directory.join("key.pem");
        let key_der = directory.join("key.der");
        let cert_der = directory.join("cert.der");
        let generated = Command::new("openssl")
            .args([
                "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
            ])
            .args([
                "-subj",
                "/CN=public.invalid",
                "-addext",
                "subjectAltName=DNS:public.invalid",
            ])
            .args([
                "-addext",
                "basicConstraints=critical,CA:FALSE",
                "-outform",
                "DER",
            ])
            .arg("-keyout")
            .arg(&key_pem)
            .arg("-out")
            .arg(&cert_der)
            .env("OPENSSL_CONF", "/dev/null")
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "generate synthetic loopback TLS certificate"
        );
        let converted = Command::new("openssl")
            .args(["pkcs8", "-topk8", "-nocrypt", "-outform", "DER", "-in"])
            .arg(&key_pem)
            .arg("-out")
            .arg(&key_der)
            .env("OPENSSL_CONF", "/dev/null")
            .output()
            .unwrap();
        assert!(
            converted.status.success(),
            "encode synthetic loopback TLS key"
        );
        let certificate = CertificateDer::from(std::fs::read(cert_der).unwrap());
        let key = PrivateKeyDer::try_from(std::fs::read(key_der).unwrap()).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let server = rustls::ServerConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.clone()], key)
            .unwrap();
        let mut roots = rustls::RootCertStore::empty();
        roots.add(certificate).unwrap();
        let client = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
        (
            Self::start_with_tls(Some(Arc::new(server))),
            Arc::new(client),
        )
    }

    // Build a real loopback URL for one fixture path.
    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    // Count HTTP requests accepted by the fixture rather than attempted or rejected connections.
    fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

struct LoopbackConnector {
    local: SocketAddr,
    selected: Rc<std::cell::RefCell<Vec<SocketAddr>>>,
}

#[async_trait(?Send)]
impl Connector for LoopbackConnector {
    async fn connect(&self, address: SocketAddr) -> std::io::Result<tokio::net::TcpStream> {
        self.selected.borrow_mut().push(address);
        assert_eq!(
            address,
            "8.8.8.8:443".parse::<SocketAddr>().unwrap(),
            "only the checked public destination reaches the connector"
        );
        assert!(self.local.ip().is_loopback());
        tokio::net::TcpStream::connect(self.local).await
    }
}

#[test]
fn public_https_pins_checked_dns_answers_without_relookup() {
    mymind::runtime::run(async {
        let (fixture, tls) = Fixture::https();
        let selected = Rc::new(std::cell::RefCell::new(Vec::new()));
        let connector = Rc::new(LoopbackConnector {
            local: SocketAddr::from(([127, 0, 0, 1], fixture.port)),
            selected: selected.clone(),
        });
        let options = ResourceOptions {
            offline: false,
            limit: None,
        };
        for answers in [
            vec![],
            vec!["169.254.169.254"],
            vec!["8.8.8.8", "::ffff:127.0.0.1"],
        ] {
            let transport = CaptureTransport::with_resolver(
                Duration::from_secs(2),
                Rc::new(move |_, port| {
                    assert_eq!(port, 443);
                    Ok(answers
                        .iter()
                        .map(|ip| SocketAddr::new(ip.parse().unwrap(), port))
                        .collect())
                }),
            )
            .with_connector(connector.clone(), tls.clone());
            let error = transport
                .fetch_resource("https://public.invalid/", options, None)
                .await
                .unwrap_err();
            assert!(
                is_blocked(&error),
                "check-capture-http.mjs:58,61 reject every unsafe DNS answer set"
            );
            assert!(
                selected.borrow().is_empty(),
                "check-capture-http.mjs:59,62 reject before connecting"
            );
        }
        let looked = Rc::new(Cell::new(0));
        let calls = looked.clone();
        let transport = CaptureTransport::with_resolver(
            Duration::from_secs(2),
            Rc::new(move |host, port| {
                assert_eq!((host, port), ("public.invalid", 443));
                calls.set(calls.get() + 1);
                assert_eq!(
                    calls.get(),
                    1,
                    "the selected socket must not trigger a second DNS lookup"
                );
                Ok(vec![SocketAddr::from(([8, 8, 8, 8], port))])
            }),
        )
        .with_connector(connector.clone(), tls.clone());
        let fetched = transport
            .fetch_resource("https://public.invalid/", options, None)
            .await
            .unwrap();
        assert_eq!(
            fetched.bytes, b"fixture",
            "check-capture-http.mjs:67 exact public-host HTTPS fixture"
        );
        assert_eq!(fetched.url, "https://public.invalid/");
        assert_eq!(
            &*selected.borrow(),
            &["8.8.8.8:443".parse::<SocketAddr>().unwrap()],
            "check-capture-http.mjs:68 checked destination is the connection address"
        );
        assert_eq!(
            looked.get(),
            1,
            "the connection uses the first checked answer without resolving again"
        );
        assert!(fixture.requests.lock().unwrap()[0].contains("\r\nhost: public.invalid\r\n"));
        let looked = Rc::new(Cell::new(0));
        let calls = looked.clone();
        let transport = CaptureTransport::with_resolver(
            Duration::from_secs(2),
            Rc::new(move |host, port| {
                assert_eq!((host, port), ("public.invalid", 443));
                let ip = if calls.get() == 0 {
                    [8, 8, 8, 8]
                } else {
                    [127, 0, 0, 1]
                };
                calls.set(calls.get() + 1);
                Ok(vec![SocketAddr::from((ip, port))])
            }),
        )
        .with_connector(connector.clone(), tls.clone());
        let before = selected.borrow().len();
        let error = transport
            .fetch_resource("https://public.invalid/redirect/again", options, None)
            .await
            .unwrap_err();
        assert!(
            is_blocked(&error),
            "check-capture-http.mjs:64 public redirect rebinding"
        );
        assert_eq!(
            selected.borrow().len(),
            before + 1,
            "check-capture-http.mjs:65 exactly one permitted connection"
        );
        assert_eq!(
            looked.get(),
            2,
            "each redirect has one fresh complete DNS lookup"
        );
        for path in ["private", "numeric", "ipv6", "account"] {
            let transport = CaptureTransport::with_resolver(
                Duration::from_secs(2),
                Rc::new(|_, port| Ok(vec![SocketAddr::from(([8, 8, 8, 8], port))])),
            )
            .with_connector(connector.clone(), tls.clone());
            let before = selected.borrow().len();
            let error = transport
                .fetch_resource(
                    &format!("https://public.invalid/redirect/{path}"),
                    options,
                    None,
                )
                .await
                .unwrap_err();
            assert!(is_blocked(&error), "check-capture-http.mjs:71 {path}");
            assert_eq!(
                selected.borrow().len(),
                before + 1,
                "check-capture-http.mjs:72 {path}"
            );
        }
        assert_eq!(
            fixture.count(),
            6,
            "only the public HTTPS fixture and permitted first redirect hops were served"
        );
    });
}

struct Unexpected;
#[async_trait(?Send)]
impl Screenshot for Unexpected {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        panic!("unexpected screenshot for blocked fixture")
    }
}
#[async_trait(?Send)]
impl Media for Unexpected {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        panic!("unexpected cover")
    }
    async fn prepare(
        &self,
        _input: MediaInput,
        _mime: &str,
        _name: &str,
        _user: &MediaUser,
        _id: &str,
    ) -> Result<PreparedMedia> {
        panic!("blocked image must never reach media preparation")
    }
    async fn send(
        &self,
        _result: &MediaResult,
        _method: &str,
        _range: Option<&str>,
    ) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, _relative: &str) -> Result<Option<MediaResult>> {
        unreachable!()
    }
    async fn image(&self, _source: &Value) -> Result<Option<AnalysisImage>> {
        unreachable!()
    }
}

#[test]
fn original_ssrf_dns_streaming_and_shared_budget_assertions() {
    mymind::runtime::run(async {
        let fixture = Fixture::start();
        let transport = CaptureTransport::new();
        let denied = [
            "0.0.0.0",
            "10.1.2.3",
            "100.64.0.1",
            "100.127.255.254",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "192.168.1.1",
            "192.0.2.1",
            "198.18.0.1",
            "224.0.0.1",
            "239.1.1.1",
            "240.0.0.1",
            "255.255.255.255",
            "[::]",
            "[::1]",
            "[fc00::1]",
            "[fe80::1]",
            "[ff02::1]",
            "[::ffff:127.0.0.1]",
            "[::127.0.0.1]",
            "[::ffff:8.8.8.8]",
            "[2001:db8::1]",
            "2130706433",
            "0x7f000001",
            "127.1",
            "134744072",
            "0x08080808",
            "010.010.010.010",
            "localhost",
        ];
        for host in denied {
            let result = transport
                .fetch_resource(
                    &format!("http://{host}/"),
                    ResourceOptions {
                        offline: false,
                        limit: None,
                    },
                    None,
                )
                .await;
            assert!(
                result.as_ref().is_err_and(is_blocked),
                "check-capture-http.mjs:49 {host}"
            );
        }
        assert_eq!(fixture.count(), 0, "check-capture-http.mjs:50");
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            assert!(public_address(ip), "check-capture-http.mjs:51 {ip}");
        }
        for host in ["127.0.0.1", "localhost", "[::1]"] {
            assert!(
                capture_destination(&format!("http://{host}/"), true).is_ok(),
                "check-capture-http.mjs:52"
            );
        }
        assert!(
            transport
                .fetch_resource(
                    "http://8.8.8.8/",
                    ResourceOptions {
                        offline: true,
                        limit: None
                    },
                    None
                )
                .await
                .is_err(),
            "check-capture-http.mjs:53"
        );
        let user = MediaUser {
            account_uid: "fixture".into(),
            id_prefix: "00000000000".into(),
        };
        for path in [
            "/redirect/private",
            "/og-private",
            "/og-numeric",
            "/og-relative-numeric",
            "/jsonld",
            "/oembed",
            "/thumbnail",
        ] {
            let patch = std::cell::RefCell::new(Value::Undefined);
            let card: Value = obj! { "source" => obj! { "url" => fixture.url(path) }, "type" => "WebPage", "title" => "Fixture" }.into();
            capture_link(
                &|| card.clone(),
                &CaptureOptions {
                    id: "fixture",
                    user: &user,
                    media: &Unexpected,
                    update: &|value| {
                        *patch.borrow_mut() = value;
                        Ok(())
                    },
                    offline: true,
                },
                &transport,
                &Unexpected,
            )
            .await
            .unwrap();
            assert_eq!(
                patch.borrow().get("data").get("type").as_str(),
                Some("Placeholder"),
                "check-capture-http.mjs:92 {path}"
            );
            assert_eq!(
                patch.borrow().get("extras").get("captureFailed").as_bool(),
                Some(true),
                "check-capture-http.mjs:93 {path}"
            );
        }
        let patch = std::cell::RefCell::new(Value::Undefined);
        let card: Value = obj! { "source" => obj! { "url" => "http://127.0.0.1/" }, "type" => "WebPage", "title" => "Fixture" }.into();
        capture_link(
            &|| card.clone(),
            &CaptureOptions {
                id: "fixture",
                user: &user,
                media: &Unexpected,
                update: &|value| {
                    *patch.borrow_mut() = value;
                    Ok(())
                },
                offline: false,
            },
            &transport,
            &Unexpected,
        )
        .await
        .unwrap();
        assert_eq!(
            patch.borrow().get("data").get("type").as_str(),
            Some("Placeholder"),
            "check-capture-http.mjs:92 saved private URL"
        );
        assert_eq!(
            patch.borrow().get("extras").get("captureFailed").as_bool(),
            Some(true),
            "check-capture-http.mjs:93 saved private URL"
        );
        let error = transport
            .fetch_resource(
                &fixture.url("/wire"),
                ResourceOptions {
                    offline: true,
                    limit: Some(16000),
                },
                None,
            )
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("too large"),
            "check-capture-http.mjs:99"
        );
        assert!(
            fixture.chunks.load(std::sync::atomic::Ordering::SeqCst) < 100,
            "check-capture-http.mjs:100 stop streaming source early"
        );
        let error = transport
            .fetch_resource(
                &fixture.url("/constructor"),
                ResourceOptions {
                    offline: true,
                    limit: None,
                },
                None,
            )
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("Unsupported capture encoding"),
            "check-capture-http.mjs:101"
        );
        for encoding in ["gzip", "deflate", "br"] {
            let error = transport
                .fetch_resource(
                    &fixture.url(&format!("/{encoding}-bomb")),
                    ResourceOptions {
                        offline: true,
                        limit: Some(32000),
                    },
                    None,
                )
                .await
                .unwrap_err();
            assert!(
                error.to_string().contains("too large"),
                "check-capture-http.mjs:105 {encoding}"
            );
            let result = transport
                .fetch_resource(
                    &fixture.url(&format!("/{encoding}-hello")),
                    ResourceOptions {
                        offline: true,
                        limit: None,
                    },
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                result.bytes, b"hello",
                "check-capture-http.mjs:108 {encoding}"
            );
            assert!(
                !result.headers.has("content-encoding") && !result.headers.has("content-length"),
                "check-capture-http.mjs:109 {encoding}"
            );
        }
        let budget = Rc::new(Cell::new(40));
        let options = ResourceOptions {
            offline: true,
            limit: None,
        };
        let first = transport
            .fetch_resource(&fixture.url("/budget"), options, Some(budget.clone()))
            .await
            .unwrap();
        assert_eq!(
            first.bytes, b"first resource",
            "check-capture-http.mjs:119 fulfilled body"
        );
        let second = transport
            .fetch_resource(&fixture.url("/budget"), options, Some(budget.clone()))
            .await;
        assert!(
            second.is_err() && budget.get() < 0,
            "check-capture-http.mjs:122 exactly first resource succeeds and second exhausts budget"
        );
        let before = fixture.count();
        assert!(
            transport
                .fetch_resource(&fixture.url("/budget"), options, Some(budget))
                .await
                .is_err()
        );
        assert_eq!(
            fixture.count(),
            before,
            "check-capture-http.mjs:125 no connection after exhaustion"
        );
        let blocked = transport
            .fetch_resource(
                "http://127.0.0.1/private",
                ResourceOptions {
                    offline: false,
                    limit: None,
                },
                None,
            )
            .await
            .unwrap_err();
        assert!(
            is_blocked(&blocked),
            "check-capture-http.mjs:127 forbidden subresource is blocked"
        );
        for remaining in [0, -100] {
            let error = transport
                .fetch_resource(
                    &fixture.url("/budget"),
                    options,
                    Some(Rc::new(Cell::new(remaining))),
                )
                .await
                .unwrap_err();
            assert!(
                error.to_string().contains("too large"),
                "check-capture-http.mjs:128 remaining={remaining}"
            );
        }
        assert_eq!(fixture.count(), before, "check-capture-http.mjs:129");
    });
}
