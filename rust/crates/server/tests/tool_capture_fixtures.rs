//! `mymind capture-fixtures` against `tools/capture-fixtures.mjs`: every route in every control state, the
//! recorded requests, the synthetic media and the subcommand binary.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use mm_js::{Value, json, obj};
use mymind::tool_capture_fixtures::{CASES, EmbeddingReply, Fixtures, fixture_media};
use sha2::{Digest, Sha256};

fn work(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tool-capture-fixtures-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn js_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    std::fs::create_dir_all(&root).unwrap();
    tools_common::prepare_root(&root);
    root
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len() / 2).map(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap()).collect()
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

// Preserve decoded pixel bytes and container format without storing another media copy.
fn image_summary(bytes: &[u8]) -> String {
    let image = mymind::vips::decode(bytes).unwrap();
    let info = mymind::vips::metadata(bytes, 0).unwrap();
    format!("{:?} {}x{} decoded={}x{} bands={} pixels={} sha256={}", info.format, info.width, info.height, image.width, image.height, image.bands, image.pixels.len(), sha(&image.pixels))
}

// Cover every synthetic media type using the same byte and pixel contracts as the original oracle.
fn media_summary(pdf: &[u8], mp4: &[u8], png: &[u8], webp: &[u8]) -> String {
    format!("pdf {} mp4 {} png {} webp {}", tools_common::summary(pdf), sha(mp4), image_summary(png), image_summary(webp))
}

struct Driver {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    hello: Value,
}

impl Driver {
    fn start(root: &Path, media: &Path) -> Self {
        assert!(tools_common::recording(), "legacy fixture driver requires recording mode");
        let mut child = Command::new("node")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/drivers/tool-capture-fixtures.mjs"))
            .arg(root)
            .arg(media)
            .current_dir(root)
            .env_clear()
            .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
            .env("PATH", std::env::var_os("PATH").unwrap())
            .env("HOME", std::env::var_os("HOME").unwrap())
            .env("NODE_NO_WARNINGS", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let hello = json::parse(&line).unwrap_or_else(|_| panic!("driver said {line:?}"));
        Self { child, input, output, hello }
    }

    fn send(&mut self, command: &str) {
        writeln!(self.input, "{command}").unwrap();
    }

    fn requests(&mut self) -> String {
        self.send(r#"{"requests":true}"#);
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        json::to_string(json::parse(&line).unwrap().get("requests"))
    }

    fn base(&self) -> String {
        let base = self.hello.get("base").to_js_string();
        register_base(&base);
        base
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
struct Reply {
    status: u16,
    content_type: String,
    location: String,
    body: Vec<u8>,
}

// Keep the route metadata plus normalized text, exact binary or decoded image expectations.
fn reply_summary(reply: &Reply, base: &str) -> String {
    let body = if reply.content_type.starts_with("image/") {
        image_summary(&reply.body)
    } else if reply.content_type == "application/pdf" {
        format!("bytes={} sha256={}", reply.body.len(), sha(&reply.body))
    } else {
        tools_common::summary(String::from_utf8_lossy(&reply.body).replace(base, "<base>").as_bytes())
    };
    format!("status={} content-type={:?} location={:?} body={body}", reply.status, reply.content_type, reply.location)
}

// Register only dynamically assigned loopback ports, retaining literal URL regression cases.
fn register_base(base: &str) {
    let port = base.rsplit(':').next().unwrap().parse().unwrap();
    mm_test_support::golden::register_loopback_port(port);
}

fn fetch(base: &str, method: &str, path: &str, body: Option<&str>) -> Reply {
    let mut stream = TcpStream::connect(base.trim_start_matches("http://")).unwrap();
    let payload = body.unwrap_or("");
    let length = if body.is_some() { format!("Content-Length: {}\r\n", payload.len()) } else { String::new() };
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: fixture\r\nConnection: close\r\n{length}\r\n{payload}").unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let split = raw.windows(4).position(|window| window == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut rest = raw[split + 4..].to_vec();
    let mut lines = head.lines();
    let status = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
    let (mut content_type, mut location, mut chunked) = (String::new(), String::new(), false);
    for line in lines {
        let (name, value) = line.split_once(':').unwrap();
        match name.to_ascii_lowercase().as_str() {
            "content-type" => content_type = value.trim().to_owned(),
            "location" => location = value.trim().to_owned(),
            "transfer-encoding" => chunked = value.contains("chunked"),
            _ => {}
        }
    }
    if chunked {
        let (mut decoded, mut cursor) = (Vec::new(), 0);
        loop {
            let end = rest[cursor..].windows(2).position(|window| window == b"\r\n").unwrap() + cursor;
            let size = usize::from_str_radix(std::str::from_utf8(&rest[cursor..end]).unwrap().trim(), 16).unwrap();
            if size == 0 {
                break;
            }
            decoded.extend_from_slice(&rest[end + 2..end + 2 + size]);
            cursor = end + 2 + size + 2;
        }
        rest = decoded;
    }
    Reply { status, content_type, location, body: rest }
}

fn same_reply(label: &str, js: &Reply, js_base: &str, rust: &Reply, rust_base: &str) {
    assert_eq!(js.status, rust.status, "{label}: status");
    assert_eq!(js.content_type, rust.content_type, "{label}: content-type");
    assert_eq!(js.location, rust.location, "{label}: location");
    if js.content_type.starts_with("image/") {
        let (a, b) = (mymind::vips::decode(&js.body).unwrap(), mymind::vips::decode(&rust.body).unwrap());
        assert_eq!(a, b, "{label}: decoded pixels");
        let (a, b) = (mymind::vips::metadata(&js.body, 0).unwrap(), mymind::vips::metadata(&rust.body, 0).unwrap());
        assert_eq!((a.format, a.width, a.height), (b.format, b.width, b.height), "{label}: image format and size");
    } else {
        let normal = |reply: &Reply, base: &str| String::from_utf8_lossy(&reply.body).replace(base, "<base>");
        assert_eq!(normal(js, js_base), normal(rust, rust_base), "{label}: body");
        if js.content_type == "application/pdf" {
            assert_eq!(js.body, rust.body, "{label}: pdf bytes");
        }
    }
}

fn requests_json(fixtures: &Fixtures) -> String {
    let list = fixtures.requests().into_iter().map(|request| obj! { "path" => request.path, "body" => request.body, "method" => request.method }.into()).collect();
    json::to_string(&Value::Array(list))
}

type State = (&'static str, &'static str, Box<dyn Fn(&Fixtures)>);

fn probes() -> Vec<(&'static str, String, Option<&'static str>)> {
    let mut list: Vec<(&'static str, String, Option<&'static str>)> = Vec::new();
    for path in [
        "/cover.webp", "/cover.png", "/cover.png?x=1", "/doc.pdf", "/failed.apk", "/blocked", "/redirect-account", "/redirect-external", "/avatar.png",
        "/x-embed", "/embed", "/XQuality", "/ArticleQuality", "/Gallery", "/Twitter", "/NoImage", "/BrokenImage", "/Product", "/JsonLd", "/nothing-here", "/",
    ] {
        list.push(("GET", path.to_owned(), None));
    }
    for case in CASES {
        list.push(("GET", format!("/{}", case[0]), None));
    }
    list.push(("POST", "/v1beta/models/gemini-embedding-2:embedContent".into(), Some(r#"{"content":{"parts":[{"text":"hello"}]}}"#)));
    list.push(("POST", "/v1beta/models/gemini-embedding-2:embedContent".into(), None));
    list.push(("POST", "/v1beta/models/gemini-embedding-2:batchEmbedContents".into(), Some(r#"{"requests":[{"a":1},{"a":2},{"a":3}]}"#)));
    list.push(("POST", "/v1beta/models/gemini-embedding-2:batchEmbedContents".into(), Some("{}")));
    list.push(("POST", "/chat/completions".into(), Some(r#"{"messages":[]}"#)));
    list.push(("POST", "/v1/systemone".into(), Some(r#"{"questions":{"a":{"instructions":"is it tag \"Design\" ?"},"b":{"instructions":"tag \"cats\""},"c":{"instructions":"no tag"},"d":{"instructions":"tag \"IMAGE\""}}}"#)));
    list.push(("POST", "/v1/systemone".into(), Some(r#"{"questions":[{"instructions":"tag \"image\""},{}]}"#)));
    list.push(("POST", "/v1/systemone".into(), Some("{}")));
    list.push(("POST", "/v1/systemone".into(), Some(r#"{"questions":{"a":null}}"#)));
    list.push(("POST", "/v1/systemone".into(), None));
    list.push(("POST", "/cover.png".into(), Some("{not json")));
    list.push(("POST", "/cover.png".into(), Some(r#"{"fine":true}"#)));
    list.push(("PUT", "/Gallery".into(), Some("[1,2]")));
    list
}

#[test]
fn every_route_in_every_control_state_matches() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_capture_fixtures.rs")));
    let directory = work("routes");
    let root = js_root(&directory);
    std::fs::create_dir_all(directory.join("js-media")).unwrap();
    std::fs::create_dir_all(directory.join("rust-media")).unwrap();
    let mut driver = tools_common::recording().then(|| Driver::start(&root, &directory.join("js-media")));
    let fixtures = Fixtures::start(fixture_media(&directory.join("rust-media")).unwrap()).unwrap();
    let rust_base = fixtures.base.clone();
    register_base(&rust_base);
    let js_base = driver.as_ref().map(Driver::base);
    let states: [State; 7] = [
        ("default", "{}", Box::new(|_| {})),
        ("ai failure", r#"{"control":{"aiFailure":true}}"#, Box::new(|f| f.control(|c| c.ai_failure = true))),
        ("ai redirect", r#"{"control":{"aiFailure":false,"aiRedirect":true}}"#, Box::new(|f| f.control(|c| { c.ai_failure = false; c.ai_redirect = true; }))),
        ("custom tags", r#"{"control":{"aiRedirect":false,"tags":["alpha","Design"]}}"#, Box::new(|f| f.control(|c| { c.ai_redirect = false; c.tags = Some(json::parse(r#"["alpha","Design"]"#).unwrap()); }))),
        ("embedding hook", r#"{"control":{"tags":null},"embedding":true}"#, Box::new(|f| f.control(|c| {
            c.tags = None;
            c.embedding = Some(std::sync::Arc::new(|body: &Value| EmbeddingReply { body: obj! { "echo" => body.clone() }.into(), status: Some(202) }));
        }))),
        ("embedding removed", r#"{"embedding":false}"#, Box::new(|f| f.control(|c| c.embedding = None))),
        ("delays", r#"{"control":{"imageDelay":150,"chatDelay":150}}"#, Box::new(|f| f.control(|c| { c.image_delay = 150; c.chat_delay = 150; }))),
    ];
    for (name, command, apply) in &states {
        if let Some(driver) = &mut driver { driver.send(command); }
        apply(&fixtures);
        std::thread::sleep(Duration::from_millis(50));
        for (method, path, body) in probes() {
            let label = format!("{name}: {method} {path} {body:?}");
            let js = js_base.as_ref().map(|base| {
                let started = Instant::now();
                let reply = fetch(base, method, &path, body);
                (reply, started.elapsed())
            });
            let started = Instant::now();
            let rust = fetch(&rust_base, method, &path, body);
            let rust_time = started.elapsed();
            let actual = reply_summary(&rust, &rust_base);
            let oracle = js.as_ref().map(|(reply, _)| reply_summary(reply, js_base.as_ref().unwrap()));
            tools_common::check(&format!("{command}: {label}"), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
            if let Some((js, _)) = &js { same_reply(&label, js, js_base.as_ref().unwrap(), &rust, &rust_base); }
            if *name == "delays" && (path == "/cover.webp" || path == "/chat/completions") {
                assert!(rust_time >= Duration::from_millis(140), "{label}: Rust delay");
                if let Some((_, elapsed)) = js { assert!(elapsed >= Duration::from_millis(140), "{label}: legacy delay"); }
            }
        }
        let actual = requests_json(&fixtures);
        let oracle = driver.as_mut().map(Driver::requests);
        tools_common::check(&format!("{name}: cumulative recorded fixture requests"), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    }
    fixtures.close();
}

#[test]
fn media_matches_the_js_fixtures() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_capture_fixtures.rs")));
    let directory = work("media");
    let root = js_root(&directory);
    std::fs::create_dir_all(directory.join("js-media")).unwrap();
    std::fs::create_dir_all(directory.join("rust-media")).unwrap();
    let driver = tools_common::recording().then(|| Driver::start(&root, &directory.join("js-media")));
    let media = fixture_media(&directory.join("rust-media")).unwrap();
    let actual = media_summary(&media.pdf, &media.mp4, &media.png, &media.webp);
    let oracle = driver.as_ref().map(|driver| {
        let pdf = hex(&driver.hello.get("pdf").to_js_string());
        let png = hex(&driver.hello.get("png").to_js_string());
        let webp = hex(&driver.hello.get("webp").to_js_string());
        format!("pdf {} mp4 {} png {} webp {}", tools_common::summary(&pdf), driver.hello.get("mp4").to_js_string(), image_summary(&png), image_summary(&webp))
    });
    tools_common::check("synthetic PDF, MP4, PNG and WebP", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    if let Some(driver) = &driver {
        assert_eq!(media.pdf, hex(&driver.hello.get("pdf").to_js_string()), "pdf bytes");
        assert_eq!(sha(&media.mp4), driver.hello.get("mp4").to_js_string(), "mp4 bytes");
        let png = hex(&driver.hello.get("png").to_js_string());
        let webp = hex(&driver.hello.get("webp").to_js_string());
        for (name, ours, theirs) in [("png", &media.png, &png), ("webp", &media.webp, &webp)] {
            let (a, b) = (mymind::vips::decode(theirs).unwrap(), mymind::vips::decode(ours).unwrap());
            assert_eq!(a, b, "{name} pixels");
            let (a, b) = (mymind::vips::metadata(theirs, 0).unwrap(), mymind::vips::metadata(ours, 0).unwrap());
            assert_eq!((a.format, a.width, a.height), (b.format, b.width, b.height), "{name} format and size");
        }
    }
    for bytes in [&media.png, &media.webp] {
        let info = mymind::vips::metadata(bytes, 0).unwrap();
        assert_eq!((info.width, info.height), (640, 480));
    }
    let pixels = mymind::vips::decode(&media.png).unwrap().pixels;
    assert_eq!(&pixels[..3], &[0xcc, 0x44, 0x22]);
}

#[test]
fn the_subcommand_serves_the_fixtures() {
    let directory = work("binary");
    let mut child = Command::new(env!("CARGO_BIN_EXE_mymind"))
        .arg("capture-fixtures")
        .env_clear()
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("MYMIND_ROOT", &directory)
        .env("MYMIND_ENV_FILE", "fixture-vars")
        .env("STORE_LOAD_ENV", "0")
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let base = line.trim_end().strip_prefix("capture fixtures on ").unwrap_or_else(|| panic!("said {line:?}")).to_owned();
    let blocked = fetch(&base, "GET", "/blocked", None);
    let page = fetch(&base, "GET", "/Product", None);
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!((blocked.status, blocked.content_type.as_str(), blocked.body.as_slice()), (403, "text/plain", b"blocked".as_slice()));
    assert_eq!(page.status, 200);
    assert!(String::from_utf8_lossy(&page.body).contains("\"@type\":\"Product\""));
}
