//! Frozen Node HTTP request semantics checked against the live native adapter, including known parser differences.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::time::Duration;

use async_trait::async_trait;
use common::{Batch, Server, TestGuard, recording};
use mm_core::http::{Headers, Request, Response, search};
use mm_core::{Value, obj};
use mm_js::json;
use mymind::http::{Handler, serve};

struct Echo;

#[async_trait(?Send)]
impl Handler for Echo {
    async fn handle(&self, mut request: Request) -> Response {
        let body = request.take_body().bytes().await.unwrap();
        let headers: mm_core::Object = request.headers.iter().map(|(name, value)| (name.to_owned(), Value::from(value))).collect();
        let answer = obj! {
            "method" => request.method.as_str(),
            "url" => request.url.as_str(),
            "pathname" => request.pathname(),
            "search" => search(&request.url),
            "headers" => headers,
            "remote" => request.remote_address.as_str(),
            "body" => body.iter().map(|&byte| char::from(byte)).collect::<String>(),
        };
        Response::new(200, Headers::from([("content-type", "application/json")]), json::to_string(&answer.into()))
    }
}

fn rust_server() -> u16 {
    let (port_tx, port_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        mymind::runtime::run(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            port_tx.send(listener.local_addr().unwrap().port()).unwrap();
            serve(listener, Rc::new(Echo)).await;
        });
    });
    let port = port_rx.recv().unwrap();
    mm_test_support::golden::register_loopback_port(port);
    port
}

// Status and decoded body of the response to one raw request on its own connection.
fn exchange(port: u16, raw: &[u8]) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    stream.write_all(raw).unwrap();
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    let text = String::from_utf8_lossy(&response).into_owned();
    let status = text.get(9..12).and_then(|code| code.parse().ok()).unwrap_or(0);
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let body = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        let mut out = String::new();
        let mut rest = body;
        while let Some((size, after)) = rest.split_once("\r\n") {
            let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
            if size == 0 {
                break;
            }
            out.push_str(&after[..size]);
            rest = &after[size + 2..];
        }
        out
    } else {
        body.to_owned()
    };
    (status, body)
}

#[test]
fn requests_read_as_node_reads_them() {
    let inventory = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "http");
    let node: Option<Server> = recording().then(|| { panic!("Legacy recording requires the recorded pre-switch revision") });
    let rust_port = rust_server();

    let close = "Connection: close\r\n";
    let mut requests: Vec<Vec<u8>> = [
        format!("GET /everything HTTP/1.1\r\nHost: localhost:4321\r\n{close}\r\n"),
        format!("GET /a?b=1&c=%20d HTTP/1.1\r\nHost: localhost\r\nX-A: 1\r\nx-a: 2\r\nCookie: a=1\r\nCookie: b=2\r\nUser-Agent: one\r\nUser-Agent: two\r\nAccept: x\r\nAccept: y\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: localhost\r\nSec-Fetch-Dest: document\r\nX-Spaces:   padded value   \r\nX-Empty:\r\n{close}\r\n"),
        format!("GET http://other.example:9/x?y HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        "GET /old HTTP/1.0\r\n\r\n".to_owned(),
        format!("GET //evil.example/path HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /a|b^c[d]{{e}}\"f HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /q?a=`b`&c={{d}}|^[e]'f HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /%zz/%2F/..%2F/./a/../b HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /a?q=%E9&r=caf%C3%A9&s=a+b HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /x? HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: [::1]:4321\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: Example.COM\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: a:b:c\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: localhost:99999\r\n{close}\r\n"),
        format!("POST /cards/grid HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 11\r\n{close}\r\n[\"a\",\"b\"]\r\n"),
        format!("POST /upload HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n{close}\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n"),
        format!("PUT /x HTTP/1.1\r\nHost: localhost\r\nContent-Type: a\r\nContent-Type: b\r\nAuthorization: one\r\nAuthorization: two\r\nReferer: r1\r\nReferer: r2\r\n{close}\r\n"),
        format!("DELETE /cards/x HTTP/1.1\r\nHost: localhost\r\nCF-Connecting-IP: 203.0.113.9\r\nX-Forwarded-For: 1.1.1.1\r\nX-Forwarded-For: 2.2.2.2\r\n{close}\r\n"),
        format!("GET / HTTP/1.1\r\nHost: localhost\r\nRange: bytes=0-1023\r\nIf-None-Match: \"a\"\r\nIf-None-Match: \"b\"\r\n{close}\r\n"),
        format!("OPTIONS * HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
        format!("GET /\\back\\slash HTTP/1.1\r\nHost: localhost\r\n{close}\r\n"),
    ]
    .map(String::into_bytes)
    .into();
    let mut latin = b"GET /latin HTTP/1.1\r\nHost: localhost\r\nX-Name: caf\xe9\r\n".to_vec();
    latin.extend_from_slice(close.as_bytes());
    latin.extend_from_slice(b"\r\n");
    requests.push(latin);

    // Known differences, none of which a browser or `fetch` produces: they percent-encode backticks and angle
    // brackets in paths and raw non-ASCII bytes, and never send a fragment.
    let known: [(&[u8], u16, u16, &str); 3] = [
        (b"GET /a`b HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", 200, 400, ""),
        (
            b"GET /raw\xc3\xa9 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            400,
            200,
            "http://localhost/raw%C3%A9",
        ),
        (
            b"GET /a#frag HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            200,
            200,
            "http://localhost/a\"",
        ),
    ];
    assert_eq!(requests.len(), 23);
    let mut identities = requests.clone();
    identities.extend(known.iter().map(|(raw, _, _, _)| raw.to_vec()));
    let scenario: Vec<u8> = identities
        .iter()
        .flat_map(|raw| [(raw.len() as u64).to_be_bytes().to_vec(), raw.clone()].concat())
        .collect();
    let scenario = common::digest(&scenario);
    for identity in &mut identities {
        *identity = [scenario.as_bytes(), b"\0", identity].concat();
    }
    let mut golden = Batch::new(env!("CARGO_MANIFEST_DIR"), "http", "request-semantics", &identities);
    golden.allow_inline_synthetic();
    let encode = |status: u16, body: &str| json::to_string(&obj! { "status" => status, "body" => body }.into()).into_bytes();
    for (index, raw) in requests.iter().enumerate() {
        let want = node.as_ref().map(|server| exchange(server.port, raw));
        let (rust_status, rust_body) = exchange(rust_port, raw);
        let frozen = golden
            .inline_answer(index)
            .map(|bytes| json::parse(std::str::from_utf8(bytes).unwrap()).unwrap());
        let oracle = want.as_ref().map(|(status, body)| encode(*status, body));
        let (expected_status, expected_body) = match &want {
            Some((status, body)) => (*status, body.as_str()),
            None => {
                let answer = frozen.as_ref().expect("synthetic HTTP expectation must be inline");
                (answer.get("status").as_f64().unwrap() as u16, answer.get("body").as_str().unwrap())
            }
        };
        // Retain the original allowance for parser refusals with an empty error page on either side.
        assert!(
            expected_status == rust_status && (expected_body == rust_body || expected_status >= 400 && (expected_body.is_empty() || rust_body.is_empty())),
            "HTTP case {index}: request parser semantics changed"
        );
        let actual = if expected_status >= 400 && (expected_body.is_empty() || rust_body.is_empty()) {
            encode(rust_status, expected_body)
        } else {
            encode(rust_status, &rust_body)
        };
        golden.check(index, &actual, oracle.as_deref());
    }
    for (offset, (raw, node_expected, rust_expected, rust_url)) in known.into_iter().enumerate() {
        let node_status = node.as_ref().map(|server| exchange(server.port, raw).0);
        let (rust_status, rust_body) = exchange(rust_port, raw);
        if let Some(status) = node_status {
            assert_eq!(status, node_expected);
        }
        assert_eq!(rust_status, rust_expected, "{}", String::from_utf8_lossy(raw));
        assert!(rust_body.contains(rust_url), "{rust_body}");
        let contract = |node_status, rust_status, url_matches| {
            json::to_string(
                &obj! {
                    "node_status" => node_status, "rust_status" => rust_status, "url_matches" => url_matches,
                }
                .into(),
            )
            .into_bytes()
        };
        let actual = contract(node_expected, rust_status, rust_body.contains(rust_url));
        let oracle = node_status.map(|status| contract(status, rust_expected, true));
        golden.check(requests.len() + offset, &actual, oracle.as_deref());
    }
    golden.finish();
    inventory.finish();
}
