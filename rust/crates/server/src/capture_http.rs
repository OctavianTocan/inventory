//! The native transport behind link capture (`createCaptureTransport` of `lib/capture-http.mjs`): GET only, every DNS
//! answer and redirect checked against the destination policy, the connection pinned to a checked address, and the
//! body bounded on the wire and after decompression.

use std::cell::{Cell, RefCell};
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use mm_core::capture_html::source_url;
use mm_core::capture_http::{capture_destination, forbidden, public_address};
use mm_core::host::{FetchedResource, Resource, ResourceOptions};
use mm_core::http::Headers;
use mm_core::{Error, Result, Value};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio_rustls::TlsConnector;
use url::{Host, Url};

const USER_AGENT: &str = "mymind-replica/1.0 (personal link capture)";
const ATTEMPT_TIMEOUT: Duration = Duration::from_millis(250);
const SINGLE_VALUE: [&str; 18] = [
    "age", "authorization", "content-length", "content-type", "etag", "expires", "from", "host", "if-modified-since",
    "if-unmodified-since", "last-modified", "location", "max-forwards", "proxy-authorization", "referer", "retry-after",
    "server", "user-agent",
];

/// The shared byte budget of one capture (`budget.remaining`); wire and decoded bytes are charged against it.
pub type Budget = Rc<Cell<i64>>;

/// The capture transport; implements [`Resource`].
pub struct CaptureTransport {
    timeout: Duration,
    tls: Arc<rustls::ClientConfig>,
}

impl Default for CaptureTransport {
    fn default() -> Self {
        Self::new()
    }
}

fn too_large() -> Error {
    Error::internal("Capture resource too large")
}

struct Collected {
    bytes: Vec<u8>,
    wire: u64,
    decoded: u64,
    limit: u64,
    budget: Option<Budget>,
    overflow: bool,
}

impl Collected {
    fn charge(&mut self, count: u64) -> bool {
        if let Some(budget) = &self.budget {
            budget.set(budget.get() - count as i64);
        }
        let over = self.wire > self.limit || self.decoded > self.limit || self.budget.as_ref().is_some_and(|budget| budget.get() < 0);
        self.overflow |= over;
        !over
    }
}

struct Sink(Rc<RefCell<Collected>>);

impl Write for Sink {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let mut state = self.0.borrow_mut();
        if state.overflow {
            return Err(std::io::Error::other("Capture resource too large"));
        }
        state.decoded += data.len() as u64;
        if !state.charge(data.len() as u64) {
            return Err(std::io::Error::other("Capture resource too large"));
        }
        state.bytes.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

enum Decoder {
    Identity(Sink),
    Gzip(flate2::write::MultiGzDecoder<Sink>),
    Zlib(flate2::write::ZlibDecoder<Sink>),
    Brotli(Box<brotli_decompressor::DecompressorWriter<Sink>>),
}

impl Decoder {
    fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        match self {
            Decoder::Identity(sink) => sink.write_all(data),
            Decoder::Gzip(decoder) => decoder.write_all(data),
            Decoder::Zlib(decoder) => decoder.write_all(data),
            Decoder::Brotli(decoder) => decoder.write_all(data),
        }
    }

    fn finish(self) -> std::io::Result<()> {
        match self {
            Decoder::Identity(_) => Ok(()),
            Decoder::Gzip(decoder) => decoder.finish().map(|_| ()),
            Decoder::Zlib(decoder) => decoder.finish().map(|_| ()),
            Decoder::Brotli(mut decoder) => decoder.flush().and_then(|()| decoder.close()),
        }
    }
}

trait Io: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> Io for T {}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| char::from(*byte)).collect()
}

// Node's merge of repeated response headers: the first wins for single-value names, cookies join with `; `.
fn node_headers(map: &hyper::HeaderMap) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (name, value) in map {
        let (name, value) = (name.as_str().to_owned(), latin1(value.as_bytes()));
        match out.iter_mut().find(|(existing, _)| *existing == name) {
            None => out.push((name, value)),
            Some(_) if SINGLE_VALUE.contains(&name.as_str()) || name == "set-cookie" => {}
            Some((_, existing)) => {
                existing.push_str(if name == "cookie" { "; " } else { ", " });
                existing.push_str(&value);
            }
        }
    }
    out
}

// Node's `autoSelectFamily` connect: each address once, the two families interleaved starting with the first answer's,
// and every attempt but the last abandoned after `ATTEMPT_TIMEOUT`.
fn attempt_order(answers: Vec<SocketAddr>) -> Vec<SocketAddr> {
    let first_v6 = answers.first().is_some_and(SocketAddr::is_ipv6);
    let (mut first, mut second): (Vec<SocketAddr>, Vec<SocketAddr>) = (Vec::new(), Vec::new());
    for answer in answers {
        let family = if answer.is_ipv6() == first_v6 { &mut first } else { &mut second };
        if !family.iter().any(|seen| seen.ip() == answer.ip()) {
            family.push(answer);
        }
    }
    let mut order = Vec::with_capacity(first.len() + second.len());
    for index in 0..first.len().max(second.len()) {
        order.extend(first.get(index));
        order.extend(second.get(index));
    }
    order
}

impl CaptureTransport {
    /// A transport with the 15 second per-request timeout and the bundled web roots.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(15))
    }

    /// A transport with a custom timeout for each request, its body included.
    pub fn with_timeout(timeout: Duration) -> Self {
        let roots = rustls::RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("tls versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
        Self { timeout, tls: Arc::new(config) }
    }

    async fn addresses(&self, target: &Url, offline: bool) -> Result<Vec<SocketAddr>> {
        let port = target.port_or_known_default().unwrap_or(80);
        let answers: Vec<SocketAddr> = match target.host() {
            Some(Host::Ipv4(ip)) => vec![SocketAddr::new(IpAddr::V4(ip), port)],
            Some(Host::Ipv6(ip)) => vec![SocketAddr::new(IpAddr::V6(ip), port)],
            Some(Host::Domain(name)) => {
                tokio::net::lookup_host((name, port)).await.map_err(|error| Error::internal(error.to_string()))?.collect()
            }
            None => return Err(forbidden()),
        };
        let allowed = |ip: IpAddr| if offline { ip.is_loopback() && !matches!(ip, IpAddr::V4(v4) if v4.octets() != [127, 0, 0, 1]) } else { public_address(&ip.to_string()) };
        if answers.is_empty() || answers.iter().any(|answer| !allowed(answer.ip())) {
            return Err(forbidden());
        }
        Ok(answers)
    }

    async fn connect(&self, target: &Url, offline: bool) -> Result<Box<dyn Io>> {
        let mut last = Error::internal("connect failed");
        let addresses = attempt_order(self.addresses(target, offline).await?);
        let count = addresses.len();
        for (index, address) in addresses.into_iter().enumerate() {
            let attempt = TcpStream::connect(address);
            let connected = if index + 1 < count {
                match tokio::time::timeout(ATTEMPT_TIMEOUT, attempt).await {
                    Ok(connected) => connected,
                    Err(_) => {
                        last = Error::internal("connect ETIMEDOUT");
                        continue;
                    }
                }
            } else {
                attempt.await
            };
            match connected {
                Ok(stream) => {
                    // As Node's sockets: with Nagle on, the request after the TLS handshake waits ~40 ms for an ACK.
                    let _ = stream.set_nodelay(true);
                    if target.scheme() != "https" {
                        return Ok(Box::new(stream));
                    }
                    let name = match target.host() {
                        Some(Host::Domain(name)) => ServerName::try_from(name.to_owned()).map_err(|error| Error::internal(error.to_string()))?,
                        _ => ServerName::IpAddress(address.ip().into()),
                    };
                    let tls = TlsConnector::from(self.tls.clone()).connect(name, stream).await.map_err(|error| Error::internal(error.to_string()))?;
                    return Ok(Box::new(tls));
                }
                Err(error) => last = Error::internal(error.to_string()),
            }
        }
        Err(last)
    }

    async fn hop(&self, target: &Url, offline: bool) -> Result<hyper::Response<hyper::body::Incoming>> {
        let stream = self.connect(target, offline).await?;
        let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await.map_err(|error| Error::internal(error.to_string()))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let mut path = target.path().to_owned();
        if let Some(query) = target.query() {
            path.push('?');
            path.push_str(query);
        }
        let authority = match target.port() {
            Some(port) => format!("{}:{port}", target.host_str().unwrap_or("")),
            None => target.host_str().unwrap_or("").to_owned(),
        };
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .header("host", authority)
            .header("accept-encoding", "gzip, deflate, br")
            .header("user-agent", USER_AGENT)
            .header("connection", "close")
            .body(Empty::<bytes::Bytes>::new())
            .map_err(|error| Error::internal(error.to_string()))?;
        sender.send_request(request).await.map_err(|error| Error::internal(error.to_string()))
    }

    async fn read_body(&self, response: hyper::Response<hyper::body::Incoming>, encoding: &str, limit: u64, budget: Option<Budget>, deadline: Instant) -> Result<Vec<u8>> {
        let state = Rc::new(RefCell::new(Collected { bytes: Vec::new(), wire: 0, decoded: 0, limit, budget, overflow: false }));
        let sink = || Sink(state.clone());
        let mut decoder = match encoding {
            "gzip" => Decoder::Gzip(flate2::write::MultiGzDecoder::new(sink())),
            "deflate" => Decoder::Zlib(flate2::write::ZlibDecoder::new(sink())),
            "br" => Decoder::Brotli(Box::new(brotli_decompressor::DecompressorWriter::new(sink(), 32 * 1024))),
            _ => Decoder::Identity(sink()),
        };
        let mut body = response.into_body();
        let reading = async {
            while let Some(frame) = body.frame().await {
                let frame = frame.map_err(|error| Error::internal(error.to_string()))?;
                let Some(data) = frame.data_ref() else { continue };
                {
                    let mut collected = state.borrow_mut();
                    collected.wire += data.len() as u64;
                    if !collected.charge(data.len() as u64) {
                        return Err(too_large());
                    }
                }
                decoder.write(data).map_err(|error| if state.borrow().overflow { too_large() } else { Error::internal(error.to_string()) })?;
            }
            decoder.finish().map_err(|error| if state.borrow().overflow { too_large() } else { Error::internal(error.to_string()) })
        };
        tokio::time::timeout_at(deadline, reading).await.map_err(|_| Error::internal("Capture request timed out"))??;
        Ok(std::mem::take(&mut state.borrow_mut().bytes))
    }

    /// `fetchResource`: fetch a public URL with the destination policy applied at every hop.
    ///
    /// # Arguments
    /// * `value` - Absolute HTTP(S) URL.
    /// * `options` - Loopback-only mode and the per-resource byte limit.
    /// * `budget` - Optional shared remaining-byte budget charged for wire and decoded bytes.
    ///
    /// # Errors
    /// A `CAPTURE_BLOCKED` error for rejected destinations, otherwise an error whose message names the failure class.
    pub async fn fetch_resource(&self, value: &str, options: ResourceOptions, budget: Option<Budget>) -> Result<FetchedResource> {
        let limit = options.limit_bytes();
        let mut url = capture_destination(value, options.offline)?;
        for _ in 0..=5 {
            if budget.as_ref().is_some_and(|budget| budget.get() <= 0) {
                return Err(too_large());
            }
            let target = Url::parse(&url).map_err(|_| forbidden())?;
            // One deadline per hop, as `AbortSignal.timeout` on the request also ends a body still arriving.
            let deadline = Instant::now() + self.timeout;
            let response = tokio::time::timeout_at(deadline, self.hop(&target, options.offline)).await.map_err(|_| Error::internal("Capture request timed out"))??;
            let status = response.status().as_u16();
            let mut headers = node_headers(response.headers());
            let header = |headers: &[(String, String)], name: &str| headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
            if matches!(status, 301 | 302 | 303 | 307 | 308) {
                let Some(location) = header(&headers, "location").filter(|location| !location.is_empty()) else {
                    return Err(Error::internal("Invalid capture redirect"));
                };
                let next = source_url(&Value::from(location.as_str()), &url).ok_or_else(forbidden)?;
                url = capture_destination(&next, options.offline)?;
                continue;
            }
            if !(200..300).contains(&status) {
                return Err(Error::internal("Page unavailable"));
            }
            let encoding = header(&headers, "content-encoding").unwrap_or_else(|| "identity".into()).to_lowercase();
            if !matches!(encoding.as_str(), "gzip" | "deflate" | "br" | "identity") {
                return Err(Error::internal("Unsupported capture encoding"));
            }
            let bytes = self.read_body(response, &encoding, limit, budget.clone(), deadline).await?;
            headers.retain(|(name, _)| !matches!(name.as_str(), "content-length" | "content-encoding" | "transfer-encoding" | "set-cookie"));
            let mime = header(&headers, "content-type").unwrap_or_default();
            return Ok(FetchedResource { bytes, url, mime, status, headers: Headers::from_pairs(headers) });
        }
        Err(Error::internal("Too many capture redirects"))
    }
}

#[async_trait(?Send)]
impl Resource for CaptureTransport {
    async fn fetch(&self, url: &str, options: ResourceOptions) -> Result<FetchedResource> {
        self.fetch_resource(url, options, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempts_interleave_families_like_node() {
        let address = |text: &str| text.parse::<SocketAddr>().unwrap();
        let answers = ["1.1.1.1:443", "1.0.0.1:443", "1.1.1.1:443", "[2606::1]:443", "[2606::2]:443", "8.8.8.8:443"].map(address);
        let order: Vec<String> = attempt_order(answers.to_vec()).iter().map(ToString::to_string).collect();
        assert_eq!(order, ["1.1.1.1:443", "[2606::1]:443", "1.0.0.1:443", "[2606::2]:443", "8.8.8.8:443"]);
        let order: Vec<String> = attempt_order(vec![address("[2606::1]:80"), address("1.1.1.1:80")]).iter().map(ToString::to_string).collect();
        assert_eq!(order, ["[2606::1]:80", "1.1.1.1:80"]);
    }
}
