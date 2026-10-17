//! The native server with a library against `node server.mjs`: the library API over `STORE_DB` (writable, then
//! read-only) and a hosted library (`HOSTED=1`), each side on its own fixture copy. Both answer the replica's requests
//! and the API's own alike, log alike, and send a write's broadcast to a `/hub` client alike.

mod common;
#[path = "../../core/tests/common/mod.rs"]
mod fixtures;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use common::{Answer, Client, Sent, Server, replica_requests, repo};
use mm_core::sql::Sql;
use mm_sqlite::Sqlite;
use regex::bytes::Regex;

const READY: &str = "mymind replica on http://localhost:";
const LOGGED: [&str; 4] = ["[miss]", "[write ignored]", "[placeholder]", "[error]"];
const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148";
static ELAPSED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""elapsed":\d+"#).unwrap());
static LONG_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{13,}").unwrap());
static ISO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z").unwrap());

#[derive(Clone, Copy)]
enum Mode {
    Library { readonly: bool },
    Hosted { readonly: bool },
}

// A fresh copy of the fixture this mode serves, and the variables that point a server at it.
fn library(mode: Mode) -> (PathBuf, Vec<(&'static str, String)>) {
    let (file, mut vars) = match mode {
        Mode::Library { .. } => {
            let copy = fixtures::temp_copy(&fixtures::test_db());
            let vars = vec![("STORE_DB", copy.to_string_lossy().into_owned())];
            (copy, vars)
        }
        Mode::Hosted { .. } => {
            let copy = fixtures::temp_copy(&fixtures::hosted_db());
            let directory = copy.parent().unwrap().to_path_buf();
            let file = directory.join("test.db");
            std::fs::rename(&copy, &file).unwrap();
            let bucket = fixtures::fixture("MYMIND_TEST_HOSTED_BUCKET");
            let status = Command::new("cp").arg("-a").arg("--reflink=auto").arg(&bucket).arg(directory.join("bucket")).status().unwrap();
            assert!(status.success());
            let vars = vec![("HOSTED", "1".to_owned()), ("HOSTED_DIR", directory.to_string_lossy().into_owned()), ("HOSTED_LIBRARY", "test".to_owned())];
            (file, vars)
        }
    };
    if let Mode::Library { readonly: true } | Mode::Hosted { readonly: true } = mode {
        vars.push(("STORE_READONLY", "1".to_owned()));
    }
    (file, vars)
}

fn start(mut command: Command, vars: &[(&'static str, String)]) -> Server {
    let root = repo();
    command.current_dir(&root).env_clear().env("MYMIND_ROOT", &root).env("STORE_LOAD_ENV", "0").env("OFFLINE", "1").env("PORT", "0");
    for name in ["PATH", "HOME"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    for (name, value) in vars {
        command.env(name, value);
    }
    Server::start(command, READY)
}

// Both servers on their own copies, the database the requests' ids come from, and each server's copy directory.
fn servers(mode: Mode) -> (Server, Server, PathBuf, [PathBuf; 2]) {
    let (node_file, node_vars) = library(mode);
    let (file, rust_vars) = library(mode);
    let mut node = Command::new("node");
    node.arg("--no-warnings").arg("server.mjs");
    let copies = [node_file.parent().unwrap().to_path_buf(), file.parent().unwrap().to_path_buf()];
    (start(node, &node_vars), start(Command::new(env!("CARGO_BIN_EXE_mymind")), &rust_vars), file, copies)
}

// The answer with the server's own copy directory (which `/__replay/reload` names) written as `<copy>`.
fn uncopied(mut answer: Answer, copy: &Path) -> Answer {
    let pattern = regex::bytes::Regex::new(&regex::escape(&copy.to_string_lossy())).unwrap();
    answer.body = pattern.replace_all(&answer.body, &b"<copy>"[..]).into_owned();
    answer
}

fn ids(db: &Sqlite, sql: &str) -> Vec<String> {
    db.query(sql, &[]).unwrap().iter().filter_map(|row| row.text("id").map(str::to_owned)).collect()
}

// The library API's own routes over ids of the served library.
fn api_requests(file: &Path) -> Vec<Sent> {
    let db = Sqlite::open(file, true).unwrap();
    let cards = ids(&db, "SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 4");
    let spaces = ids(&db, "SELECT id FROM spaces ORDER BY id LIMIT 2");
    let queries = ids(&db, "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id LIMIT 1");
    let tags = ids(&db, "SELECT DISTINCT name AS id FROM annotations WHERE name NOT GLOB '*[#&+%?]*' ORDER BY name LIMIT 2");
    let mut targets: Vec<String> = ["/cards", "/cards/trash", "/spaces", "/spaces/nope", "/tags/manual", "/tags/recent", "/changes?since=0", "/changes?since=99999999999999999999"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    targets.extend(["/tags/suggest?q=a", "/tags/suggest", "/quota", "/quotas/objects", "/account.json", "/search?q=design", "/search?q=", "/search/suggestions?q=d"].map(str::to_owned));
    targets.extend(["/embeddings/search?q=chair", "/embeddings/related/nope", "/queries/nope/check", "/achievements/first", "/shares/nope", "/objects/nope", "/cards/nope/detail"].map(str::to_owned));
    for id in &cards {
        for suffix in ["", "/detail", "?template=PinnedCard", "?template=Card"] {
            targets.push(format!("/cards/{id}{suffix}"));
        }
        for suffix in ["", "/tags", "/content", "/revisions"] {
            targets.push(format!("/objects/{id}{suffix}"));
        }
        targets.push(format!("/embeddings/related/{id}"));
    }
    targets.extend(spaces.iter().map(|id| format!("/spaces/{id}")));
    targets.extend(queries.iter().map(|id| format!("/queries/{id}/check")));
    targets.extend(tags.iter().map(|name| format!("/tags/detail?name={}", mm_js::uri::encode_uri_component(name))));
    let mut sent = Vec::new();
    for target in targets {
        sent.push(Sent::new("GET", &target, &[("accept", "application/json")]));
        sent.push(Sent::new("GET", &target, &[("user-agent", IPHONE)]));
    }
    let grid = format!("[{}]", cards.iter().map(|id| format!("\"{id}\"")).collect::<Vec<_>>().join(","));
    sent.push(Sent { method: "POST".to_owned(), target: "/cards/grid".to_owned(), headers: vec![("content-type".to_owned(), "application/json".to_owned())], body: grid });
    for agent in ["Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36", IPHONE] {
        sent.push(Sent::new("GET", "/everything", &[("sec-fetch-dest", "document"), ("user-agent", agent)]));
    }
    sent
}

// Recorded objects of the hosted test library, with ranges and missing names.
fn hosted_requests() -> Vec<Sent> {
    let media = fixtures::fixture("MYMIND_TEST_HOSTED_BUCKET").join("libraries/test/capture");
    let mut files = Vec::new();
    for folder in ["media", "mediahost", "carbon"] {
        let mut found: Vec<PathBuf> = walk(&media.join(folder));
        found.sort();
        files.extend(found.into_iter().take(4).map(|path| (folder, path.strip_prefix(media.join(folder)).unwrap().to_string_lossy().into_owned())));
    }
    let mut sent = Vec::new();
    for (folder, file) in &files {
        let prefix = if *folder == "media" { "/media" } else { &format!("/_{folder}") };
        sent.push(Sent::new("GET", &format!("{prefix}/{file}"), &[]));
        sent.push(Sent::new("GET", &format!("{prefix}/{file}"), &[("range", "bytes=0-9")]));
        sent.push(Sent::new("HEAD", &format!("{prefix}/{file}"), &[]));
    }
    for target in ["/media/missing.png", "/media/missing.mp4", "/media/%2e%2e/x", "/_mediahost/missing.webp", "/_static/134/missing.js", "/downloads/manifest.json"] {
        sent.push(Sent::new("GET", target, &[]));
    }
    sent
}

fn walk(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else { return Vec::new() };
    entries.flatten().flat_map(|entry| if entry.path().is_dir() { walk(&entry.path()) } else { vec![entry.path()] }).collect()
}

fn now_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() * 1000.0
}

fn zeroed(digits: &[u8]) -> Vec<u8> {
    digits.iter().map(|&byte| if byte.is_ascii_digit() { b'0' } else { byte }).collect()
}

// The answer with the times the two servers' clocks wrote during the run (from `since`, in ms) zeroed digit by
// digit, so lengths and msgpack prefixes stay as they were and recorded times still compare exactly.
fn untimed(mut answer: Answer, since: f64) -> Answer {
    let since_iso = mm_js::date::iso_from_ms(since.floor()).unwrap()[..19].to_owned();
    let recent = |digits: &[u8]| digits.get(..13).and_then(|ms| std::str::from_utf8(ms).ok()?.parse::<f64>().ok()).is_some_and(|ms| ms >= since);
    let body = ELAPSED.replace_all(&answer.body, &b"\"elapsed\":0"[..]);
    let body = ISO.replace_all(&body, |found: &regex::bytes::Captures| if found[0][..19] >= *since_iso.as_bytes() { zeroed(&found[0]) } else { found[0].to_vec() });
    answer.body = LONG_NUMBER.replace_all(&body, |found: &regex::bytes::Captures| if recent(&found[0]) { zeroed(&found[0]) } else { found[0].to_vec() }).into_owned();
    for (_, value) in &mut answer.headers {
        if value.len() >= 13 && value.bytes().all(|byte| byte.is_ascii_digit()) && recent(value.as_bytes()) {
            *value = "0".repeat(value.len());
        }
    }
    answer
}

fn compare(mode: Mode, requests: impl Fn(&Path) -> Vec<Sent>) {
    let since = now_ms() - 1000.0;
    let (reference, ported, file, [node_copy, rust_copy]) = servers(mode);
    let mut requests = requests(&file);
    if let Ok(only) = std::env::var("LIBRARY_TEST_ONLY") {
        requests.retain(|sent| sent.target.contains(&only));
    }
    let (mut expected, mut actual) = (Client::new(reference.port), Client::new(ported.port));
    let mut differences = Vec::new();
    eprintln!("{} requests", requests.len());
    for (index, sent) in requests.iter().enumerate() {
        let started = Instant::now();
        let want = expected.send(sent);
        let sent_at = started.elapsed();
        let got = actual.send(sent);
        let got_at = started.elapsed();
        let (want, got) = (uncopied(untimed(want, since), &node_copy), uncopied(untimed(got, since), &rust_copy));
        if got_at > Duration::from_secs(1) || index % 1000 == 0 {
            eprintln!("{index} {} {} node {sent_at:?} rust {:?} total {:?} ({} bytes)", sent.method, sent.target, got_at - sent_at, started.elapsed(), want.body.len());
        }
        if want != got {
            let at = want.body.iter().zip(&got.body).position(|(a, b)| a != b).unwrap_or(want.body.len().min(got.body.len()));
            let window = |body: &[u8]| String::from_utf8_lossy(&body[at.saturating_sub(80)..(at + 120).min(body.len())]).into_owned();
            differences.push(format!(
                "{} {} {:?}\n  node: {want:?}\n  rust: {got:?}\n  body differs at {at}:\n    node …{:?}\n    rust …{:?}",
                sent.method, sent.target, sent.headers, window(&want.body), window(&got.body)
            ));
        }
    }
    std::thread::sleep(Duration::from_millis(500));
    let (want, got) = (reference.logged(&LOGGED), ported.logged(&LOGGED));
    if let Some(index) = (0..want.len().min(got.len())).find(|&index| want[index] != got[index]) {
        differences.push(format!("log line {index}\n  node: {}\n  rust: {}", want[index], got[index]));
    }
    if want.len() != got.len() {
        differences.push(format!("{} log lines from node, {} from rust", want.len(), got.len()));
    }
    if let Some(file) = std::env::var_os("LIBRARY_TEST_REPORT") {
        std::fs::write(file, differences.join("\n")).unwrap();
    }
    let shown: Vec<&str> = differences.iter().take(40).map(String::as_str).collect();
    assert!(differences.is_empty(), "{} of {} requests differ:\n{}", differences.len(), requests.len(), shown.join("\n"));
}

// The replica's reads only, since a write's answer carries the server's clock and random ids.
fn reads(file: &Path) -> Vec<Sent> {
    let mut sent: Vec<Sent> = replica_requests().into_iter().filter(|sent| matches!(sent.method.as_str(), "GET" | "HEAD")).collect();
    sent.extend(api_requests(file));
    sent
}

fn everything(file: &Path) -> Vec<Sent> {
    let mut sent = replica_requests();
    sent.extend(api_requests(file));
    sent
}

#[test]
fn library_api_answers_as_server_mjs() {
    compare(Mode::Library { readonly: false }, reads);
}

#[test]
fn readonly_library_answers_as_server_mjs() {
    compare(Mode::Library { readonly: true }, everything);
}

#[test]
fn hosted_library_answers_as_server_mjs() {
    compare(Mode::Hosted { readonly: false }, |file| {
        let mut sent = reads(file);
        sent.extend(hosted_requests());
        sent
    });
}

#[test]
fn readonly_hosted_library_answers_as_server_mjs() {
    compare(Mode::Hosted { readonly: true }, |file| {
        let mut sent = everything(file);
        sent.extend(hosted_requests());
        sent
    });
}

fn masked(payload: &[u8]) -> Vec<u8> {
    let mask = [0x12, 0x34, 0x56, 0x78];
    let mut frame = vec![0x81, 0x80 | u8::try_from(payload.len()).unwrap()];
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(index, byte)| byte ^ mask[index % 4]));
    frame
}

// The text frames a server sends, from raw bytes after the upgrade answer.
fn frames(bytes: &[u8]) -> Vec<String> {
    let Some(start) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else { return Vec::new() };
    let (mut at, mut texts) = (start + 4, Vec::new());
    while at + 2 <= bytes.len() {
        let (opcode, mut length, mut head) = (bytes[at] & 0x0f, usize::from(bytes[at + 1] & 0x7f), 2);
        if length == 126 {
            length = usize::from(u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]));
            head = 4;
        } else if length == 127 {
            length = usize::try_from(u64::from_be_bytes(bytes[at + 2..at + 10].try_into().unwrap())).unwrap();
            head = 10;
        }
        let Some(payload) = bytes.get(at + head..at + head + length) else { break };
        if opcode == 1 {
            texts.push(String::from_utf8_lossy(&LONG_NUMBER.replace_all(payload, &b"N"[..])).into_owned());
        }
        at += head + length;
    }
    texts
}

// Join `/hub`, rename a space through the API, and collect what the hub sent.
fn broadcast(server: &Server, space: &str) -> (Answer, Vec<String>) {
    let mut hub = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    let upgrade = "GET /hub HTTP/1.1\r\nhost: localhost\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-version: 13\r\nsec-websocket-key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
    hub.write_all(upgrade.as_bytes()).unwrap();
    hub.write_all(&masked(b"{\"protocol\":\"json\",\"version\":1}\x1e")).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let body = r#"{"name":"Renamed by the hub test"}"#.to_owned();
    let sent = Sent { method: "PATCH".to_owned(), target: format!("/spaces/{space}"), headers: vec![("content-type".to_owned(), "application/json".to_owned())], body };
    let answer = Client::new(server.port).send(&sent);
    let mut received = Vec::new();
    hub.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut buffer = [0; 64 * 1024];
    while Instant::now() < deadline {
        match hub.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => received.extend_from_slice(&buffer[..count]),
            Err(_) => {}
        }
    }
    (answer, frames(&received))
}

#[test]
fn writes_reach_the_hub_as_in_server_mjs() {
    let (reference, ported, file, _) = servers(Mode::Library { readonly: false });
    let space = ids(&Sqlite::open(&file, true).unwrap(), "SELECT id FROM spaces ORDER BY id LIMIT 1").remove(0);
    let (want, got) = (broadcast(&reference, &space), broadcast(&ported, &space));
    assert_eq!(want.0.status, got.0.status);
    assert!(want.1.len() > 1, "node broadcast nothing: {:?}", want.1);
    assert_eq!(want.1, got.1);
}
