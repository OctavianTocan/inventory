//! The native server's view of requests against Node's: the same raw requests go to a Node server and to the hyper
//! adapter, each answering with the method, URL, headers and body it saw.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use async_trait::async_trait;
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
    port_rx.recv().unwrap()
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
    let mut node = Command::new("node")
        .arg(common::root().join("rust/crates/server/tests/drivers/echo-server.mjs"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs");
    let mut line = String::new();
    BufReader::new(node.stdout.take().unwrap()).read_line(&mut line).unwrap();
    let node_port: u16 = line.trim().parse().unwrap();
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

    let mut differences = Vec::new();
    for raw in &requests {
        let (node_status, node_body) = exchange(node_port, raw);
        let (rust_status, rust_body) = exchange(rust_port, raw);
        // Requests the HTTP parser itself refuses get each server's own error page.
        let same = node_status == rust_status && (node_body == rust_body || node_status >= 400 && (node_body.is_empty() || rust_body.is_empty()));
        if !same {
            differences.push(format!("{}\n  node {node_status} {node_body}\n  rust {rust_status} {rust_body}", String::from_utf8_lossy(raw).lines().next().unwrap_or("")));
        }
    }
    // Known differences, none of which a browser or `fetch` produces: they percent-encode backticks and angle
    // brackets in paths and raw non-ASCII bytes, and never send a fragment.
    let known: [(&[u8], u16, u16, &str); 3] = [
        (b"GET /a`b HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", 200, 400, ""),
        (b"GET /raw\xc3\xa9 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", 400, 200, "http://localhost/raw%C3%A9"),
        (b"GET /a#frag HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", 200, 200, "http://localhost/a\""),
    ];
    for (raw, node_expected, rust_expected, rust_url) in known {
        let (node_status, _) = exchange(node_port, raw);
        let (rust_status, rust_body) = exchange(rust_port, raw);
        assert_eq!((node_status, rust_status), (node_expected, rust_expected), "{}", String::from_utf8_lossy(raw));
        assert!(rust_body.contains(rust_url), "{rust_body}");
    }
    node.kill().unwrap();
    node.wait().unwrap();
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}
