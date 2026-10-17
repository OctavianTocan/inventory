//! Frozen library HTTP/log scenarios for native and hosted modes, plus a real mutation and hub delivery.

mod common;
#[path = "../../core/tests/common/mod.rs"]
mod fixtures;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use common::{
    Answer, Batch, Client, Sent, Server, TestGuard, answer_bytes, fixture_identity, log_bytes, recording, replica_requests, request_cases, server_root,
};
use mm_core::sql::Sql;
use mm_js::{Value, json, obj};
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

impl Mode {
    // Separate golden namespaces retain all four original library modes.
    fn namespace(self) -> &'static str {
        match self {
            Self::Library { readonly: false } => "native-writable",
            Self::Library { readonly: true } => "native-readonly",
            Self::Hosted { readonly: false } => "hosted-writable",
            Self::Hosted { readonly: true } => "hosted-readonly",
        }
    }
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
            let status = Command::new("cp")
                .arg("-a")
                .arg("--reflink=auto")
                .arg(&bucket)
                .arg(directory.join("bucket"))
                .status()
                .unwrap();
            assert!(status.success());
            let vars = vec![
                ("HOSTED", "1".to_owned()),
                ("HOSTED_DIR", directory.to_string_lossy().into_owned()),
                ("HOSTED_LIBRARY", "test".to_owned()),
            ];
            (file, vars)
        }
    };
    if let Mode::Library { readonly: true } | Mode::Hosted { readonly: true } = mode {
        vars.push(("STORE_READONLY", "1".to_owned()));
    }
    (file, vars)
}

fn start(mut command: Command, vars: &[(&'static str, String)]) -> Server {
    let root = server_root();
    command
        .current_dir(&root)
        .env_clear()
        .env("MYMIND_ROOT", &root)
        .env("STORE_LOAD_ENV", "0")
        .env("OFFLINE", "1")
        .env("PORT", "0");
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

struct Servers {
    reference: Option<Server>,
    ported: Server,
    file: PathBuf,
    node_copy: Option<PathBuf>,
    rust_copy: PathBuf,
    fixture: Value,
}

// Each server gets its own copy, with the immutable initial database included in scenario identity.
fn servers(mode: Mode) -> Servers {
    let reference = recording().then(|| {
        let (node_file, node_vars) = library(mode);
        let mut node = Command::new("node");
        node.arg("--no-warnings").arg("server.mjs");
        (start(node, &node_vars), node_file.parent().unwrap().to_path_buf())
    });
    let (file, rust_vars) = library(mode);
    let fixture = fixture_identity(&file);
    let rust_copy = file.parent().unwrap().to_path_buf();
    let ported = start(Command::new(env!("CARGO_BIN_EXE_mymind")), &rust_vars);
    let (reference, node_copy) = reference.map_or((None, None), |(server, copy)| (Some(server), Some(copy)));
    Servers {
        reference,
        ported,
        file,
        node_copy,
        rust_copy,
        fixture,
    }
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
    let queries = ids(
        &db,
        "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id LIMIT 1",
    );
    let tags = ids(
        &db,
        "SELECT DISTINCT name AS id FROM annotations WHERE name NOT GLOB '*[#&+%?]*' ORDER BY name LIMIT 2",
    );
    let mut targets: Vec<String> = [
        "/cards",
        "/cards/trash",
        "/spaces",
        "/spaces/nope",
        "/tags/manual",
        "/tags/recent",
        "/changes?since=0",
        "/changes?since=99999999999999999999",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    targets.extend(
        [
            "/tags/suggest?q=a",
            "/tags/suggest",
            "/quota",
            "/quotas/objects",
            "/account.json",
            "/search?q=design",
            "/search?q=",
            "/search/suggestions?q=d",
        ]
        .map(str::to_owned),
    );
    targets.extend(
        [
            "/embeddings/search?q=chair",
            "/embeddings/related/nope",
            "/queries/nope/check",
            "/achievements/first",
            "/shares/nope",
            "/objects/nope",
            "/cards/nope/detail",
        ]
        .map(str::to_owned),
    );
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
    sent.push(Sent {
        method: "POST".to_owned(),
        target: "/cards/grid".to_owned(),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: grid,
    });
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
        files.extend(
            found
                .into_iter()
                .take(4)
                .map(|path| (folder, path.strip_prefix(media.join(folder)).unwrap().to_string_lossy().into_owned())),
        );
    }
    let mut sent = Vec::new();
    for (folder, file) in &files {
        let prefix = if *folder == "media" { "/media" } else { &format!("/_{folder}") };
        sent.push(Sent::new("GET", &format!("{prefix}/{file}"), &[]));
        sent.push(Sent::new("GET", &format!("{prefix}/{file}"), &[("range", "bytes=0-9")]));
        sent.push(Sent::new("HEAD", &format!("{prefix}/{file}"), &[]));
    }
    for target in [
        "/media/missing.png",
        "/media/missing.mp4",
        "/media/%2e%2e/x",
        "/_mediahost/missing.webp",
        "/_static/134/missing.js",
        "/downloads/manifest.json",
    ] {
        sent.push(Sent::new("GET", target, &[]));
    }
    sent
}

fn walk(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else { return Vec::new() };
    entries
        .flatten()
        .flat_map(|entry| if entry.path().is_dir() { walk(&entry.path()) } else { vec![entry.path()] })
        .collect()
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
    let recent = |digits: &[u8]| {
        digits
            .get(..13)
            .and_then(|ms| std::str::from_utf8(ms).ok()?.parse::<f64>().ok())
            .is_some_and(|ms| ms >= since)
    };
    let body = ELAPSED.replace_all(&answer.body, &b"\"elapsed\":0"[..]);
    let body = ISO.replace_all(&body, |found: &regex::bytes::Captures| {
        if found[0][..19] >= *since_iso.as_bytes() {
            zeroed(&found[0])
        } else {
            found[0].to_vec()
        }
    });
    answer.body = LONG_NUMBER
        .replace_all(
            &body,
            |found: &regex::bytes::Captures| if recent(&found[0]) { zeroed(&found[0]) } else { found[0].to_vec() },
        )
        .into_owned();
    for (_, value) in &mut answer.headers {
        if value.len() >= 13 && value.bytes().all(|byte| byte.is_ascii_digit()) && recent(value.as_bytes()) {
            *value = "0".repeat(value.len());
        }
    }
    answer
}

fn compare(mode: Mode, requests: impl Fn(&Path) -> Vec<Sent>) {
    let inventory = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "library");
    let since = now_ms() - 1000.0;
    let servers = servers(mode);
    let mut requests = requests(&servers.file);
    if let Ok(only) = std::env::var("LIBRARY_TEST_ONLY") {
        assert!(!recording(), "LIBRARY_TEST_ONLY cannot truncate a golden recording");
        requests.retain(|sent| sent.target.contains(&only));
    }
    assert!(!requests.is_empty(), "the library request scenario is empty");
    let identities = request_cases(&requests, servers.fixture.clone(), &["application-logs"]);
    let mut golden = Batch::new(env!("CARGO_MANIFEST_DIR"), "library", mode.namespace(), &identities);
    golden.register_input(&common::repo().join("rust/test-fixtures/http/replica-requests.json"));
    let mut expected = servers.reference.as_ref().map(|server| Client::new(server.port));
    let mut actual = Client::new(servers.ported.port);
    eprintln!("{} requests", requests.len());
    for (index, sent) in requests.iter().enumerate() {
        let started = Instant::now();
        let want = expected
            .as_mut()
            .map(|client| answer_bytes(&uncopied(untimed(client.send(sent), since), servers.node_copy.as_ref().unwrap())));
        let sent_at = started.elapsed();
        let got = actual.send(sent);
        let got_at = started.elapsed();
        let got = uncopied(untimed(got, since), &servers.rust_copy);
        if got_at > Duration::from_secs(1) || index % 1000 == 0 {
            eprintln!("{index}: oracle {sent_at:?}, Rust {:?}, {} response bytes", got_at - sent_at, got.body.len());
        }
        golden.check(index, &answer_bytes(&got), want.as_deref());
    }
    std::thread::sleep(Duration::from_millis(500));
    let normalized_logs = |server: &Server, copy: &Path| {
        log_bytes(
            &server
                .logged(&LOGGED)
                .into_iter()
                .map(|line| line.replace(copy.to_string_lossy().as_ref(), "<copy>"))
                .collect::<Vec<_>>(),
        )
    };
    let want = servers
        .reference
        .as_ref()
        .map(|server| normalized_logs(server, servers.node_copy.as_ref().unwrap()));
    golden.check(requests.len(), &normalized_logs(&servers.ported, &servers.rust_copy), want.as_deref());
    golden.finish();
    inventory.finish();
}

// The replica's reads only, since a write's answer carries the server's clock and random ids.
fn reads(file: &Path) -> Vec<Sent> {
    let mut sent: Vec<Sent> = replica_requests()
        .into_iter()
        .filter(|sent| matches!(sent.method.as_str(), "GET" | "HEAD"))
        .collect();
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
    let Some(start) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Vec::new();
    };
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

const HUB_UPGRADE: &str = "GET /hub HTTP/1.1\r\nhost: localhost\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-version: 13\r\nsec-websocket-key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
const HUB_HANDSHAKE: &[u8] = b"{\"protocol\":\"json\",\"version\":1}\x1e";

// The exact mutation is shared by the scenario identity and the executable HTTP request.
fn rename_request(space: &str) -> Sent {
    Sent {
        method: "PATCH".to_owned(),
        target: format!("/spaces/{space}"),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: r#"{"name":"Renamed by the hub test"}"#.to_owned(),
    }
}

// Join `/hub`, rename a space through the API, and collect what the hub sent.
fn broadcast(server: &Server, space: &str) -> (Answer, Vec<String>) {
    let mut hub = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    hub.write_all(HUB_UPGRADE.as_bytes()).unwrap();
    hub.write_all(&masked(HUB_HANDSHAKE)).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let answer = Client::new(server.port).send(&rename_request(space));
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
    let inventory = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "library");
    let servers = servers(Mode::Library { readonly: false });
    let space = ids(&Sqlite::open(&servers.file, true).unwrap(), "SELECT id FROM spaces ORDER BY id LIMIT 1").remove(0);
    let mut identities = request_cases(&[rename_request(&space)], servers.fixture, &[]);
    identities[0] = [HUB_UPGRADE.as_bytes(), &masked(HUB_HANDSHAKE), &identities[0]].concat();
    let mut golden = Batch::new(env!("CARGO_MANIFEST_DIR"), "library", "write-hub-broadcast", &identities);
    let want = servers.reference.as_ref().map(|server| broadcast(server, &space));
    let got = broadcast(&servers.ported, &space);
    assert!(got.1.len() > 1, "the Rust mutation must produce a hub broadcast");
    if let Some(want) = &want {
        assert!(want.1.len() > 1, "the oracle mutation must produce a hub broadcast");
    }
    let encode = |answer: &(Answer, Vec<String>)| {
        json::to_string(
            &obj! {
                "status" => answer.0.status.as_str(), "frames" => Value::Array(answer.1.iter().cloned().map(Value::from).collect()),
            }
            .into(),
        )
        .into_bytes()
    };
    golden.check(0, &encode(&got), want.as_ref().map(encode).as_deref());
    golden.finish();
    inventory.finish();
}
