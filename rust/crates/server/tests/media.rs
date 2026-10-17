//! `LocalMedia`, `LocalProcessor` and `mymind container` against `lib/local-media.mjs`, `lib/media-processor.mjs` and
//! `worker/container/server.mjs` (driver: `tests/drivers/media.mjs`), over generated fixtures and real images of the
//! recorded library. Both link the libvips build sharp ships, so every answer, rendition and image is byte-identical.

mod common;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use common::{Server, repo};
use mm_core::host::{Media, MediaInput, MediaProcessor, MediaResult, MediaUser, RenditionSource};
use mm_core::http::Body;
use mm_core::{Error, arr, obj};
use mm_js::{Value, base64, json};
use mymind::local_media::{LocalMedia, analysis_image};
use mymind::media_processor::LocalProcessor;

struct Fixtures {
    dir: PathBuf,
    names: Vec<String>,
}

fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("media-test")
}

fn driver() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/drivers/media.mjs")
}

fn fixtures() -> &'static Fixtures {
    static FIXTURES: OnceLock<Fixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| {
        let dir = scratch().join("fixtures");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let output = Command::new("node").arg(driver()).arg("fixtures").arg(&dir).env("MYMIND_ROOT", repo()).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let names = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        let names = names.as_array().unwrap().iter().map(|name| name.as_str().unwrap().to_owned()).collect();
        Fixtures { dir, names }
    })
}

fn fresh(name: &str) -> PathBuf {
    let dir = scratch().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_js(root: &Path, ops: Vec<Value>) -> Vec<Value> {
    let job = scratch().join(format!("job-{}.json", root.file_name().unwrap().to_string_lossy()));
    let job_value: Value = obj! { "root" => root.to_string_lossy().into_owned(), "ops" => Value::Array(ops) }.into();
    std::fs::write(&job, json::to_string(&job_value)).unwrap();
    let output = Command::new("node").arg(driver()).arg("run").arg(&job).env("MYMIND_ROOT", repo()).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap().as_array().unwrap().clone()
}

fn seed_op(fixture: &Fixtures, name: &str) -> Value {
    obj! { "op" => "seed", "from" => fixture.dir.join(name).to_string_lossy().into_owned(), "to" => format!("acct/card/{name}") }.into()
}

fn seed_rust(fixture: &Fixtures, root: &Path, name: &str) {
    std::fs::create_dir_all(root.join("acct/card")).unwrap();
    std::fs::copy(fixture.dir.join(name), root.join("acct/card").join(name)).unwrap();
}

fn error_value(error: &Error) -> Value {
    match error {
        Error::Status { status, message, .. } => obj! { "error" => obj! { "status" => *status, "message" => message.as_str() } }.into(),
        _ => obj! { "error" => obj! { "status" => Value::Null, "message" => "unexpected" } }.into(),
    }
}

fn user() -> MediaUser {
    MediaUser { account_uid: "acct".to_owned(), id_prefix: "card_".to_owned() }
}

fn kind(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap() {
        "pdf" => "pdf",
        "mp4" | "webm" => "video",
        _ => "image",
    }
}

fn same_bytes(first: &[u8], second: &[u8], label: &str, problems: &mut Vec<String>) {
    if first != second {
        problems.push(format!("{label}: the bytes differ"));
    }
}

fn probe(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,codec_name,pix_fmt", "-of", "csv=p=0"])
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn report(name: &str, compared: usize, problems: &[String]) {
    println!("{name}: {compared} comparisons");
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
}

fn normalized(value: &Value) -> String {
    let text = json::to_string(value);
    match value.get("data").get("blob").get("id").as_str() {
        Some(id) => text.replace(id, "BLOBID"),
        None => text,
    }
}

#[test]
fn prepare_matches() {
    let fixture = fixtures();
    let js_root = fresh("prepare-js");
    let rs_root = fresh("prepare-rs");
    let cases: Vec<(usize, &String, bool)> = fixture.names.iter().enumerate().map(|(index, name)| (index, name, index % 4 == 1)).collect();
    let ops = cases
        .iter()
        .map(|(index, name, bytes)| {
            let user: Value = obj! { "accountUid" => "acct", "idPrefix" => "card_" }.into();
            obj! {
                "op" => "prepare", "file" => fixture.dir.join(name).to_string_lossy().into_owned(), "bytes" => *bytes,
                "name" => name.as_str(), "user" => user, "id" => format!("card_{index}")
            }
            .into()
        })
        .collect();
    let expected = run_js(&js_root, ops);
    let mut problems = Vec::new();
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for ((index, name, bytes), expected) in cases.iter().zip(&expected) {
            let path = fixture.dir.join(name);
            let input = if *bytes { MediaInput::Bytes(std::fs::read(&path).unwrap()) } else { MediaInput::File(path.to_string_lossy().into_owned()) };
            let got = match media.prepare(input, "application/octet-stream", name, &user(), &format!("card_{index}")).await {
                Ok(made) => made.to_value(),
                Err(error) => error_value(&error),
            };
            if normalized(expected) != normalized(&got) {
                problems.push(format!("{name}: JS {} Rust {}", normalized(expected), normalized(&got)));
            }
        }
    });
    let stored: Vec<_> = expected.iter().filter(|value| value.get("data").is_object()).collect();
    println!("prepared {} of {} fixtures", stored.len(), cases.len());
    assert!(cases.len() >= 60 && stored.len() >= 40);
    report("prepare", cases.len(), &problems);
}

fn suffixes(name: &str) -> Vec<&'static str> {
    let mut list = vec![";800x600-t.webp"];
    match kind(name) {
        "pdf" => list.extend([";background(fff)/page(1)/642x840-t.webp", ";background(fff)/page(2)/642x840-t.webp", ";background(fff)/page(9)/642x840-t.webp", ";300x213.jpeg"]),
        "video" => list.extend([";960x540.mp4", ";600x338.mp4", ";320x180/hevc.mp4", ";800x450-t.webp", ";300x213.jpeg"]),
        _ if name.starts_with("real-") || name == "plain.jpg" || name.starts_with("exif-") || name.ends_with(".png") || name.ends_with(".webp") => list.extend([
            ";800x350-c.webp", ";300x213.webp", ";800x800.jpeg", ";800x1000-t.webp", ";640x480-t.png", ";1200x900-t.webp", ";background(fff)/page(1)/642x840-t.webp",
            ";original.jpg",
        ]),
        _ => {}
    }
    list
}

fn check_result(label: &str, expected: &Value, got: &Result<Option<MediaResult>, Error>, js_root: &Path, problems: &mut Vec<String>) -> bool {
    let got_value = match got {
        Ok(None) => Value::Null,
        Ok(Some(found)) => obj! { "type" => found.content_type.as_str() }.into(),
        Err(error) => error_value(error),
    };
    let expected_value = if expected.get("type").is_string() { obj! { "type" => expected.get("type").clone() }.into() } else { expected.clone() };
    if json::to_string(&got_value) != json::to_string(&expected_value) {
        problems.push(format!("{label}: JS {} Rust {}", json::to_string(&expected_value), json::to_string(&got_value)));
        return false;
    }
    let Ok(Some(found)) = got else { return false };
    let theirs = js_root.join(expected.get("file").as_str().unwrap());
    let (ours, theirs_bytes) = (std::fs::read(&found.key).unwrap(), std::fs::read(&theirs).unwrap());
    if found.content_type == "video/mp4" && label.contains(';') {
        let (a, b) = (probe(&theirs), probe(Path::new(&found.key)));
        if a != b {
            problems.push(format!("{label}: ffprobe {a:?} against {b:?}"));
        }
    } else if ours != theirs_bytes {
        problems.push(format!("{label}: the stored bytes differ"));
    }
    true
}

#[test]
fn renditions_match() {
    let fixture = fixtures();
    let js_root = fresh("rendition-js");
    let rs_root = fresh("rendition-rs");
    let mut ops: Vec<Value> = fixture.names.iter().map(|name| seed_op(fixture, name)).collect();
    let mut cases = Vec::new();
    for name in &fixture.names {
        seed_rust(fixture, &rs_root, name);
        for suffix in suffixes(name) {
            let relative = format!("acct/card/{name}{suffix}");
            ops.push(obj! { "op" => "response", "relative" => relative.as_str() }.into());
            cases.push(relative);
        }
    }
    let results = run_js(&js_root, ops);
    let results = &results[fixture.names.len()..];
    let mut problems = Vec::new();
    let mut produced = 0;
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for (relative, expected) in cases.iter().zip(results) {
            let got = media.response(relative).await;
            if check_result(relative, expected, &got, &js_root, &mut problems) {
                produced += 1;
            }
        }
    });
    println!("{produced} of {} renditions were made by both", cases.len());
    assert!(produced >= 300);
    report("renditions", cases.len(), &problems);
}

async fn drain(body: Body) -> Vec<u8> {
    body.bytes().await.unwrap()
}

#[test]
fn send_matches() {
    let fixture = fixtures();
    let js_root = fresh("send-js");
    let rs_root = fresh("send-rs");
    let files = ["plain.jpg", "empty.bin", "clip.mp4"];
    let ranges: [Option<&str>; 14] = [
        None, Some("bytes=0-0"), Some("bytes=5-"), Some("bytes=-10"), Some("bytes=0-999999999"), Some("bytes=999999999-"), Some("bytes=abc"),
        Some("bytes=0-1,3-4"), Some("bytes=-0"), Some("bytes=5-2"), Some("items=0-1"), Some("bytes=-999999999"), Some("bytes=1-"), Some("bytes=-"),
    ];
    let mut ops: Vec<Value> = files.iter().map(|name| seed_op(fixture, name)).collect();
    let mut cases = Vec::new();
    for name in files {
        seed_rust(fixture, &rs_root, name);
        for range in ranges {
            for method in ["GET", "HEAD"] {
                let relative = format!("acct/card/{name}");
                ops.push(obj! { "op" => "send", "relative" => relative.as_str(), "method" => method, "range" => range.map_or(Value::Null, Value::from) }.into());
                cases.push((relative, method, range));
            }
        }
    }
    let results = run_js(&js_root, ops);
    let mut problems = Vec::new();
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for ((relative, method, range), expected) in cases.iter().zip(&results[files.len()..]) {
            let label = format!("{relative} {method} {range:?}");
            let found = media.response(relative).await.unwrap().unwrap();
            let sent = media.send(&found, method, *range).await.unwrap();
            let headers: Vec<Value> = sent.headers.iter().map(|(name, value)| arr![name, value]).collect();
            let body = base64::encode(&drain(sent.body).await);
            let got: Value = obj! { "status" => sent.status, "headers" => Value::Array(headers), "body" => body }.into();
            if json::to_string(&got) != json::to_string(expected) {
                problems.push(format!("{label}: JS {} Rust {}", json::to_string(expected), json::to_string(&got)));
            }
        }
    });
    report("send", cases.len(), &problems);
}

#[test]
fn errors_and_images_match() {
    let fixture = fixtures();
    let js_root = fresh("errors-js");
    let rs_root = fresh("errors-rs");
    let names = ["plain.jpg", "garbage.bin", "pages.pdf", "exif-6.jpg", "alpha.png", "truncated.jpg", "image.tiff", "image.avif", "image.svg", "huge.png"];
    let mut ops: Vec<Value> = names.iter().map(|name| seed_op(fixture, name)).collect();
    let relatives = [
        "acct/card/missing.jpg", "../escape.jpg", "acct/../../x.jpg", "nodir/x.jpg;800x600-t.webp", "acct/card/plain.jpg;bogus", "acct/card/plain.jpg;0x0.webp",
        "acct/card/plain.jpg;5000x10.webp", "acct/card/plain.jpg;original.jpg", "acct/card/garbage.bin;800x600-t.webp", "acct/card/plain;800x600-t.webp",
        "acct/card/exif-6;300x213.webp", "acct/card/pages.pdf;background(fff)/page(9)/642x840-t.webp", "acct/card/plain.jpg;800x600-t.gif",
        "acct/card/truncated.jpg;800x600-t.webp", "acct/card/image.tiff;300x200-t.webp", "acct/card/image.avif;300x200-t.jpeg",
        "acct/card/image.svg;300x200-t.webp", "acct/card/huge.png;300x200-t.webp", "acct/card/alpha.png;300x200-t.jpeg",
    ];
    for relative in relatives {
        ops.push(obj! { "op" => "response", "relative" => relative }.into());
    }
    let out = |name: &str| fresh(&format!("errors-out-{name}"));
    let images: Vec<(&str, Value)> = vec![
        ("path", obj! { "path" => "acct/card/plain.jpg" }.into()),
        ("src", obj! { "src" => "/media/acct/card/exif-6.jpg;800x1000-t.webp" }.into()),
        ("pdf", obj! { "path" => "acct/card/pages.pdf" }.into()),
        ("missing", obj! { "path" => "acct/card/nothing.jpg" }.into()),
        ("falsy", Value::Null),
        ("garbage", obj! { "path" => "acct/card/garbage.bin" }.into()),
    ];
    let image_dirs: Vec<PathBuf> = images.iter().map(|(name, _)| out(name)).collect();
    for ((_, value), dir) in images.iter().zip(&image_dirs) {
        ops.push(obj! { "op" => "image", "value" => value.clone(), "out" => dir.join("js.png").to_string_lossy().into_owned() }.into());
    }
    let results = run_js(&js_root, ops);
    let results = &results[names.len()..];
    let mut problems = Vec::new();
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for name in names {
            seed_rust(fixture, &rs_root, name);
        }
        for (relative, expected) in relatives.iter().zip(results) {
            let got = media.response(relative).await;
            check_result(relative, expected, &got, &js_root, &mut problems);
        }
        let concurrent = futures_util::future::join_all((0..8).map(|_| media.response("acct/card/alpha.png;777x333-t.webp"))).await;
        let keys: Vec<_> = concurrent.iter().map(|made| made.as_ref().unwrap().as_ref().unwrap().key.clone()).collect();
        if keys.iter().any(|key| key != &keys[0]) || !Path::new(&keys[0]).is_file() {
            problems.push("concurrent requests disagree".to_owned());
        }
        for (((label, value), dir), expected) in images.iter().zip(&image_dirs).zip(&results[relatives.len()..]) {
            let got = media.image(value).await;
            let got_value = match &got {
                Ok(None) => Value::Null,
                Ok(Some(made)) => obj! { "mime" => made.mime.as_str() }.into(),
                Err(_) => obj! { "error" => true }.into(),
            };
            let expected_value = if expected.get("error").is_object() { obj! { "error" => true }.into() } else { expected.clone() };
            if json::to_string(&got_value) != json::to_string(&expected_value) {
                problems.push(format!("image {label}: JS {} Rust {}", json::to_string(&expected_value), json::to_string(&got_value)));
            } else if let Ok(Some(made)) = &got {
                same_bytes(&std::fs::read(dir.join("js.png")).unwrap(), &made.data, &format!("image {label}"), &mut problems);
            }
        }
    });
    report("errors and images", relatives.len() + images.len(), &problems);
}

#[test]
fn analysis_matches() {
    let fixture = fixtures();
    let js_root = fresh("analysis-js");
    let out = fresh("analysis-out");
    let ops = fixture
        .names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            obj! { "op" => "analysis", "file" => fixture.dir.join(name).to_string_lossy().into_owned(), "out" => out.join(format!("{index}.png")).to_string_lossy().into_owned() }.into()
        })
        .collect();
    let results = run_js(&js_root, ops);
    let mut problems = Vec::new();
    let mut converted = 0;
    mymind::runtime::run(async {
        for (index, (name, expected)) in fixture.names.iter().zip(&results).enumerate() {
            let got = analysis_image(std::fs::read(fixture.dir.join(name)).unwrap()).await;
            match (expected.get("error").is_object(), got) {
                (true, Err(_)) => {}
                (false, Ok(made)) => {
                    converted += 1;
                    if made.mime != "image/png" {
                        problems.push(format!("{name}: mime {}", made.mime));
                    }
                    same_bytes(&std::fs::read(out.join(format!("{index}.png"))).unwrap(), &made.data, &format!("analysis {name}"), &mut problems);
                }
                (js_failed, got) => problems.push(format!("{name}: JS failed {js_failed}, Rust {:?}", got.map(|made| made.mime))),
            }
        }
    });
    println!("{converted} fixtures converted by both");
    assert!(converted >= 40);
    report("analysis", fixture.names.len(), &problems);
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn post(port: u16, method: &str, target: &str, headers: &[(&str, String)], body: &[u8]) -> Reply {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(120))).unwrap();
    let mut head = format!("{method} {target} HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\ncontent-length: {}\r\n", body.len());
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(body).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let split = raw.windows(4).position(|window| window == b"\r\n\r\n").unwrap();
    let text = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = text.lines();
    let status = lines.next().unwrap()[9..12].parse().unwrap();
    let headers: Vec<(String, String)> =
        lines.filter_map(|line| line.split_once(':')).map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned())).collect();
    let mut body = raw[split + 4..].to_vec();
    if headers.iter().any(|(name, value)| name == "transfer-encoding" && value.contains("chunked")) {
        let (mut out, mut rest) = (Vec::new(), &body[..]);
        loop {
            let end = rest.windows(2).position(|window| window == b"\r\n").unwrap();
            let size = usize::from_str_radix(std::str::from_utf8(&rest[..end]).unwrap().split(';').next().unwrap().trim(), 16).unwrap();
            if size == 0 {
                break;
            }
            out.extend_from_slice(&rest[end + 2..end + 2 + size]);
            rest = &rest[end + 2 + size + 2..];
        }
        body = out;
    }
    let headers = headers.into_iter().filter(|(name, _)| !["date", "connection", "keep-alive", "transfer-encoding", "content-length"].contains(&name.as_str())).collect();
    Reply { status, headers, body }
}

fn spawn_container(command: Command) -> Server {
    Server::start(command, "media container on :")
}

fn container_command(program: &str, arguments: &[&Path]) -> Command {
    let mut command = Command::new(program);
    let free = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    command.args(arguments).current_dir(repo()).env_clear().env("PORT", free.to_string()).env("TMPDIR", scratch());
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
}

fn media_args(value: Value) -> (&'static str, String) {
    ("x-media-args", base64::encode_url(json::to_string(&value).as_bytes()))
}

fn resource_server(image: Vec<u8>) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (mut stream, image) = (stream, image.clone());
            std::thread::spawn(move || {
                let mut request = [0u8; 2048];
                let read = stream.read(&mut request).unwrap_or(0);
                let first = String::from_utf8_lossy(&request[..read]).lines().next().unwrap_or("").to_owned();
                let (status, kind, body) = if first.contains("/image.png") { ("200 OK", "image/png", image) } else { ("404 Not Found", "text/plain", b"nope".to_vec()) };
                let head = format!("HTTP/1.1 {status}\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nx-fixture: yes\r\nconnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            });
        }
    });
    port
}

#[test]
fn container_matches() {
    let fixture = fixtures();
    let root = repo();
    let js = spawn_container(container_command("node", &[Path::new("--no-warnings"), &root.join("worker/container/server.mjs")]));
    let rust = spawn_container(container_command(env!("CARGO_BIN_EXE_mymind"), &[Path::new("container")]));
    let image_port = resource_server(std::fs::read(fixture.dir.join("alpha.png")).unwrap());
    let problems = std::cell::RefCell::new(Vec::<String>::new());
    let compared = std::cell::Cell::new(0);
    let both = |label: &str, method: &str, target: &str, headers: &[(&str, String)], body: &[u8], pixels: bool| -> (Reply, Reply) {
        let (a, b) = (post(js.port, method, target, headers, body), post(rust.port, method, target, headers, body));
        compared.set(compared.get() + 1);
        if a.status != b.status || a.headers != b.headers {
            problems.borrow_mut().push(format!("{label}: JS {} {:?} Rust {} {:?}", a.status, a.headers, b.status, b.headers));
        } else if pixels && a.status == 200 {
            same_bytes(&a.body, &b.body, label, &mut problems.borrow_mut());
        } else if a.body != b.body && !pixels {
            let (x, y) = (String::from_utf8_lossy(&a.body).into_owned(), String::from_utf8_lossy(&b.body).into_owned());
            let id = |text: &str| json::parse(text).ok().and_then(|value| value.get("data").get("blob").get("id").as_str().map(str::to_owned));
            let clean = |text: &str| id(text).map_or_else(|| text.to_owned(), |id| text.replace(&id, "BLOBID"));
            if clean(&x) != clean(&y) {
                problems.borrow_mut().push(format!("{label}: JS {x:.400} Rust {y:.400}"));
            }
        }
        (a, b)
    };
    both("health", "GET", "/health", &[], b"", false);
    both("health post", "POST", "/health", &[], b"", false);
    both("get prepare", "GET", "/prepare", &[], b"", false);
    both("unknown", "POST", "/nope", &[], b"", false);
    both("inherited", "POST", "/constructor", &[], b"", false);
    for name in ["plain.jpg", "exif-6.jpg", "alpha.png", "pages.pdf", "clip.mp4", "image.svg", "garbage.bin", "huge.png", "real-00.jpg", "anim.gif", "truncated.jpg", "audio-only.mp4"] {
        let args = obj! { "type" => "application/octet-stream", "name" => name, "user" => obj! { "accountUid" => "acct", "idPrefix" => "card_" }, "id" => "card_9" };
        both(&format!("prepare {name}"), "POST", "/prepare", &[media_args(args.into())], &std::fs::read(fixture.dir.join(name)).unwrap(), false);
    }
    for name in ["plain.jpg", "exif-6.jpg", "pages.pdf", "clip.mp4", "real-01.jpg"] {
        let body = std::fs::read(fixture.dir.join(name)).unwrap();
        let suffix = match kind(name) {
            "pdf" => ";background(fff)/page(1)/642x840-t.webp",
            "video" => ";320x180.mp4",
            _ => ";300x213.webp",
        };
        let relative = format!("acct/card/{name}{suffix}");
        let args = || media_args(obj! { "name" => format!("acct/card/{name}"), "relative" => relative.as_str() }.into());
        let (a, b) = both(&format!("rendition {name}"), "POST", "/rendition", &[args()], &body, !suffix.ends_with("mp4"));
        if suffix.ends_with("mp4") {
            let (x, y) = (scratch().join("container-js.mp4"), scratch().join("container-rs.mp4"));
            std::fs::write(&x, &a.body).unwrap();
            std::fs::write(&y, &b.body).unwrap();
            if probe(&x) != probe(&y) {
                problems.borrow_mut().push(format!("rendition {name}: ffprobe {:?} against {:?}", probe(&x), probe(&y)));
            }
        }
    }
    let missing = media_args(obj! { "name" => "acct/card/other.jpg", "relative" => "acct/card/plain.jpg;300x213.webp" }.into());
    both("rendition without original", "POST", "/rendition", &[missing], &std::fs::read(fixture.dir.join("plain.jpg")).unwrap(), false);
    let escape = media_args(obj! { "name" => "../x.jpg", "relative" => "acct/card/plain.jpg;300x213.webp" }.into());
    both("rendition outside", "POST", "/rendition", &[escape], b"x", false);
    let bad = media_args(obj! { "name" => "acct/card/a.jpg", "relative" => "acct/card/a.jpg;bogus" }.into());
    both("rendition bad suffix", "POST", "/rendition", &[bad], &std::fs::read(fixture.dir.join("plain.jpg")).unwrap(), false);
    for name in ["plain.jpg", "exif-3.jpg", "alpha.png", "anim.gif", "garbage.bin"] {
        both(&format!("image {name}"), "POST", "/image", &[], &std::fs::read(fixture.dir.join(name)).unwrap(), true);
    }
    both("bad args", "POST", "/image", &[("x-media-args", "!!!".to_owned())], b"", false);
    both("bad json", "POST", "/image", &[("x-media-args", base64::encode_url(b"{nope"))], b"", false);
    for (label, path, offline, limit) in [("png", "/image.png", true, Value::Null), ("missing", "/missing", true, Value::Null), ("online", "/image.png", false, Value::Null), ("limit", "/image.png", true, Value::from(10))] {
        let mut args = obj! { "url" => format!("http://127.0.0.1:{image_port}{path}"), "offline" => offline };
        if !limit.is_nullish() {
            args.insert("limit", limit);
        }
        let (a, b) = both(&format!("resource {label}"), "POST", "/resource", &[media_args(args.into())], b"", false);
        let meta = |reply: &Reply| reply.headers.iter().find(|(name, _)| name == "x-resource").map(|(_, value)| value.clone());
        if meta(&a) != meta(&b) && meta(&a).is_some() {
            let decode = |value: Option<String>| String::from_utf8_lossy(&base64::decode(&value.unwrap_or_default())).into_owned();
            problems.borrow_mut().push(format!("resource {label}: x-resource JS {} Rust {}", decode(meta(&a)), decode(meta(&b))));
        }
    }
    both("screenshot of a private address", "POST", "/screenshot", &[media_args(obj! { "url" => "http://127.0.0.1:1/" }.into())], b"", false);
    report("container", compared.get(), &problems.borrow());
}

#[test]
fn processor_matches_library_functions() {
    let fixture = fixtures();
    let directory = fresh("processor");
    let processor = LocalProcessor::new(Some(directory.clone()));
    mymind::runtime::run(async {
        let body = Body::Bytes(std::fs::read(fixture.dir.join("plain.jpg")).unwrap());
        let made = processor.prepare(body, "image/jpeg", "plain.jpg", &user(), "card_1").await.unwrap();
        assert_eq!(made.data.get("type").as_str(), Some("Image"));
        let original = RenditionSource { body: Body::Bytes(std::fs::read(fixture.dir.join("plain.jpg")).unwrap()), size: 0, name: "acct/card/plain.jpg".to_owned() };
        assert!(processor.rendition(original, "acct/card/plain.jpg;300x213.webp").await.unwrap().is_some());
        let outside = RenditionSource { body: Body::Empty, size: 0, name: "../plain.jpg".to_owned() };
        let failure = processor.rendition(outside, "acct/card/plain.jpg;300x213.webp").await.err().unwrap();
        assert_eq!(failure, Error::status(400, "Invalid media path"));
    });
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0, "the temporary directories are removed");
}
