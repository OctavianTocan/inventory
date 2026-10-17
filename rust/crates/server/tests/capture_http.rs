//! The native capture transport against `createCaptureTransport` of `lib/capture-http.mjs`: both fetch the same
//! fixture URLs from a local server and must give the same bytes, headers, redirects and failure classes.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::cell::Cell;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::time::Duration;

use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder};
use mm_core::host::ResourceOptions;
use mm_core::{Error, Value, obj};
use mm_js::base64;
use mymind::capture_http::CaptureTransport;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/capture-http.mjs");
const KNOWN: [&str; 6] = ["Page unavailable", "Too many capture redirects", "Capture resource too large", "Unsupported capture encoding", "Invalid capture redirect", "Unsupported capture destination"];

fn page() -> Vec<u8> {
    (0..3000).map(|index| b"<p>hello mymind</p>\n"[index % 20]).collect()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn reply(stream: &mut TcpStream, status: &str, headers: &[(&str, String)], body: &[u8]) {
    let mut head = format!("HTTP/1.1 {status}\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("Connection: close\r\n\r\n");
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn serve(mut stream: TcpStream, port: u16, brotli: &[u8]) {
    let mut request = Vec::new();
    let mut buffer = [0u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => return,
            Ok(read) => request.extend_from_slice(&buffer[..read]),
        }
    }
    let text = String::from_utf8_lossy(&request).into_owned();
    let path = text.split(' ').nth(1).unwrap_or("/").to_owned();
    let page = page();
    let length = |bytes: &[u8]| ("Content-Length", bytes.len().to_string());
    let html = ("Content-Type", "text/html; charset=utf-8".to_owned());
    let location = |target: String| [("Location", target), ("Content-Length", "0".to_owned())];
    match path.as_str() {
        "/plain" => reply(&mut stream, "200 OK", &[html, length(&page), ("X-Alpha", "1".into()), ("Set-Cookie", "a=b".into()), ("X-Dup", "one".into()), ("X-Dup", "two".into())], &page),
        "/gzip" => {
            let body = gzip(&page);
            reply(&mut stream, "200 OK", &[html, length(&body), ("Content-Encoding", "gzip".into())], &body);
        }
        "/deflate" => {
            let body = deflate(&page);
            reply(&mut stream, "200 OK", &[html, length(&body), ("Content-Encoding", "DEFLATE".into())], &body);
        }
        "/br" => reply(&mut stream, "200 OK", &[html, length(brotli), ("Content-Encoding", "br".into())], brotli),
        "/identity" => reply(&mut stream, "200 OK", &[html, length(&page), ("Content-Encoding", "identity".into())], &page),
        "/zstd" => reply(&mut stream, "200 OK", &[html, length(&page), ("Content-Encoding", "zstd".into())], &page),
        "/corrupt" => reply(&mut stream, "200 OK", &[html, length(&page), ("Content-Encoding", "gzip".into())], &page),
        "/bomb" => {
            let body = gzip(&vec![0u8; 400_000]);
            reply(&mut stream, "200 OK", &[length(&body), ("Content-Encoding", "gzip".into())], &body);
        }
        "/empty" => reply(&mut stream, "200 OK", &[("Content-Length", "0".to_owned())], b""),
        "/typeless" => reply(&mut stream, "200 OK", &[length(&page)], &page),
        "/image" => reply(&mut stream, "200 OK", &[("content-type", "IMAGE/PNG".to_owned()), length(&page)], &page),
        "/chunked" => {
            let mut body = Vec::new();
            for part in page.chunks(700) {
                body.extend_from_slice(format!("{:x}\r\n", part.len()).as_bytes());
                body.extend_from_slice(part);
                body.extend_from_slice(b"\r\n");
            }
            body.extend_from_slice(b"0\r\n\r\n");
            reply(&mut stream, "200 OK", &[html, ("Transfer-Encoding", "chunked".into())], &body);
        }
        "/redirect-plain" => reply(&mut stream, "302 Found", &location("/plain".into()), b""),
        "/redirect-relative" => reply(&mut stream, "301 Moved", &location("plain".into()), b""),
        "/redirect-absolute" => reply(&mut stream, "307 Temporary", &location(format!("http://127.0.0.1:{port}/gzip")), b""),
        "/redirect-chain" => reply(&mut stream, "308 Permanent", &location("/redirect-plain".into()), b""),
        "/redirect-loop" => reply(&mut stream, "302 Found", &location("/redirect-loop".into()), b""),
        "/redirect-none" => reply(&mut stream, "302 Found", &[("Content-Length", "0".to_owned())], b""),
        "/redirect-blocked" => reply(&mut stream, "302 Found", &location("http://10.0.0.1/secret".into()), b""),
        "/redirect-external" => reply(&mut stream, "302 Found", &location("https://example.com/".into()), b""),
        "/redirect-credentials" => reply(&mut stream, "302 Found", &location(format!("http://user:pw@127.0.0.1:{port}/plain")), b""),
        "/redirect-scheme" => reply(&mut stream, "302 Found", &location("ftp://127.0.0.1/".into()), b""),
        "/redirect-303" => reply(&mut stream, "303 See Other", &location("/image".into()), b""),
        "/missing" => reply(&mut stream, "404 Not Found", &[length(b"no")], b"no"),
        "/broken" => reply(&mut stream, "500 Internal Server Error", &[length(b"no")], b"no"),
        "/informational" => reply(&mut stream, "204 No Content", &[], b""),
        "/slow" => {
            reply(&mut stream, "200 OK", &[("Content-Length", "10".to_owned())], b"abc");
            std::thread::sleep(Duration::from_secs(3));
        }
        _ => reply(&mut stream, "404 Not Found", &[length(b"")], b""),
    }
}

fn start(brotli: Vec<u8>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let brotli = brotli.clone();
            std::thread::spawn(move || serve(stream, port, &brotli));
        }
    });
    port
}

struct Case {
    url: String,
    limit: Option<u64>,
    budget: bool,
}

fn rust_answer(result: Result<mm_core::host::FetchedResource, Error>, remaining: i64) -> Value {
    let mut answer: Value = match result {
        Ok(found) => {
            let mut headers: Vec<(String, String)> = found.headers.iter().map(|(name, value)| (name.to_owned(), value.to_owned())).collect();
            headers.sort();
            obj! { "ok" => obj! {
                "bytes" => base64::encode(&found.bytes), "url" => found.url, "mime" => found.mime, "status" => f64::from(found.status),
                "headers" => Value::Object(headers.into_iter().map(|(name, value)| (name, Value::from(value))).collect()),
            } }
            .into()
        }
        Err(error) => {
            let (status, code, message) = match &error {
                Error::Status { status, message, code } => (Some(*status), *code, message.clone()),
                Error::Invalid(message) | Error::Internal(message) => (None, None, message.clone()),
            };
            let message = if KNOWN.contains(&message.as_str()) { message } else { "other".into() };
            obj! { "error" => obj! { "status" => status.map_or(Value::Null, |status| f64::from(status).into()), "code" => code.map_or(Value::Null, Value::from), "message" => message } }.into()
        }
    };
    answer.set("remaining", Value::from(remaining as f64));
    answer
}

#[test]
fn transport_matches_node() {
    let encoded = common::call(DRIVER, "brotli", vec![base64::encode(&page()).into()]).unwrap();
    let brotli = base64::decode(encoded.as_str().unwrap());
    let port = start(brotli);
    let base = format!("http://127.0.0.1:{port}");
    let paths = [
        "/plain", "/gzip", "/deflate", "/br", "/identity", "/zstd", "/corrupt", "/bomb", "/empty", "/typeless", "/image", "/chunked", "/redirect-plain",
        "/redirect-relative", "/redirect-absolute", "/redirect-chain", "/redirect-loop", "/redirect-none", "/redirect-blocked", "/redirect-external",
        "/redirect-credentials", "/redirect-scheme", "/redirect-303", "/missing", "/broken", "/informational", "/nothing-here",
    ];
    let mut cases: Vec<Case> = Vec::new();
    for path in paths {
        cases.push(Case { url: format!("{base}{path}"), limit: None, budget: false });
    }
    for path in ["/plain", "/gzip", "/br", "/bomb", "/chunked", "/redirect-chain", "/empty"] {
        for limit in [100, 2999, 3000] {
            cases.push(Case { url: format!("{base}{path}"), limit: Some(limit), budget: false });
        }
    }
    for path in ["/plain", "/gzip", "/plain", "/br", "/chunked", "/plain", "/empty", "/plain", "/redirect-chain"] {
        cases.push(Case { url: format!("{base}{path}"), limit: None, budget: true });
    }
    for url in [
        "http://10.0.0.1/", "http://localhost.:1/", "https://example.com/", "ftp://127.0.0.1/", "http://127.0.0.1:1/", "http://[::1]:1/", "http://127.1/",
        "http://user:pw@127.0.0.1/", "not a url",
    ] {
        cases.push(Case { url: url.to_owned(), limit: None, budget: false });
    }
    cases.push(Case { url: format!("http://localhost:{port}/plain"), limit: None, budget: false });
    let budget_total = 7000;
    let driver_cases = Value::Array(
        cases.iter().map(|case| obj! { "url" => case.url.as_str(), "limit" => case.limit.map_or(Value::Null, |limit| (limit as f64).into()), "budget" => case.budget }.into()).collect(),
    );
    let js = common::call(DRIVER, "run", vec![driver_cases, Value::from(f64::from(budget_total))]).unwrap();
    let Value::Array(js) = js else { panic!("driver answered no array") };
    let transport = CaptureTransport::new();
    let rust: Vec<Value> = mymind::runtime::run(async {
        let budget = Rc::new(Cell::new(i64::from(budget_total)));
        let mut out = Vec::new();
        for case in &cases {
            let options = ResourceOptions { offline: true, limit: case.limit };
            let result = transport.fetch_resource(&case.url, options, case.budget.then(|| budget.clone())).await;
            out.push(rust_answer(result, budget.get()));
        }
        out
    });
    let mut oks = 0;
    for ((case, rust), js) in cases.iter().zip(&rust).zip(&js) {
        let context = format!("{} limit={:?} budget={}", case.url, case.limit, case.budget);
        let (rust, js) = (common::text(rust), common::text(js));
        assert_eq!(rust.replace(&port.to_string(), "PORT"), js.replace(&port.to_string(), "PORT"), "{context}");
        oks += usize::from(rust.starts_with("{\"ok\""));
    }
    assert!(oks > 15, "{oks} fetches succeeded");
    println!("transport cases: {} ({oks} successful)", cases.len());
}

#[test]
fn slow_body_times_out() {
    let port = start(Vec::new());
    let transport = CaptureTransport::with_timeout(Duration::from_millis(400));
    let result = mymind::runtime::run(async { transport.fetch_resource(&format!("http://127.0.0.1:{port}/slow"), ResourceOptions { offline: true, limit: None }, None).await });
    assert!(matches!(result, Err(Error::Internal(_))), "{result:?}");
}
