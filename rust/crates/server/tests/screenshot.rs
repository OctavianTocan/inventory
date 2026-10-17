//! The Chromium screenshot host against a loopback fixture server: a rendered page, blocked resources and no
//! browser left behind.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use mm_core::capture_http::is_blocked;
use mm_core::host::Screenshot;
use mm_js::base64;
use mymind::capture_http::CaptureTransport;
use mymind::screenshot::ChromiumScreenshot;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
const PAGE: &str = "<html><body style=\"margin:0;background:rgb(255,0,0)\"><img src=\"/pic.png\"><script>fetch('/api', {method: 'POST'})</script></body></html>";
const REMOTE: &str = "<html><body><img src=\"https://example.com/x.png\"></body></html>";

async fn serve(listener: TcpListener, seen: Rc<RefCell<Vec<String>>>) {
    loop {
        let Ok((mut socket, _)) = listener.accept().await else { return };
        let seen = seen.clone();
        tokio::task::spawn_local(async move {
            let mut head = Vec::new();
            let mut chunk = [0u8; 4096];
            while !head.windows(4).any(|window| window == b"\r\n\r\n") {
                match socket.read(&mut chunk).await {
                    Ok(0) | Err(_) => return,
                    Ok(count) => head.extend_from_slice(&chunk[..count]),
                }
            }
            let text = String::from_utf8_lossy(&head).to_string();
            let mut words = text.lines().next().unwrap_or("").split(' ');
            let (method, path) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
            seen.borrow_mut().push(format!("{method} {path}"));
            let (kind, body) = match path {
                "/page" => ("text/html", PAGE.as_bytes().to_vec()),
                "/remote" => ("text/html", REMOTE.as_bytes().to_vec()),
                "/pic.png" => ("image/png", base64::decode(PNG)),
                _ => ("text/plain", b"nope".to_vec()),
            };
            let reply = format!("HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", body.len());
            let _ = socket.write_all(reply.as_bytes()).await;
            let _ = socket.write_all(&body).await;
            let _ = socket.shutdown().await;
        });
    }
}

fn children() -> Vec<String> {
    let mut found = Vec::new();
    for task in std::fs::read_dir("/proc/self/task").unwrap().flatten() {
        if let Ok(text) = std::fs::read_to_string(task.path().join("children")) {
            found.extend(text.split_whitespace().map(str::to_string));
        }
    }
    found
}

#[test]
fn screenshots_through_the_capture_transport() {
    let executable = PathBuf::from(std::env::var("VERIFIER_CHROMIUM").unwrap_or_else(|_| "/usr/lib/chromium/chromium".into()));
    if !executable.exists() {
        println!("skipped: no Chromium at {}", executable.display());
        return;
    }
    mymind::runtime::run(async {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Rc::new(RefCell::new(Vec::new()));
        tokio::task::spawn_local(serve(listener, seen.clone()));
        let host = ChromiumScreenshot::new(executable, Rc::new(CaptureTransport::new()));

        let png = host.capture(&format!("http://127.0.0.1:{port}/page"), true).await.unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let dimension = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        assert_eq!((dimension(16), dimension(20)), (1400, 940));
        let log = seen.borrow().clone();
        assert!(log.contains(&"GET /page".to_string()), "{log:?}");
        assert!(log.contains(&"GET /pic.png".to_string()), "{log:?}");
        assert!(!log.iter().any(|line| line.starts_with("POST")), "{log:?}");

        let error = host.capture("https://example.com/", true).await.unwrap_err();
        assert!(is_blocked(&error), "{error:?}");

        let error = host.capture(&format!("http://127.0.0.1:{port}/remote"), true).await.unwrap_err();
        assert!(is_blocked(&error), "{error:?}");

        for _ in 0..50 {
            if children().is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(children(), Vec::<String>::new());
    });
}
