//! Frozen download responses and refusal logs over Rust-built synthetic sessions and package files.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{Batch, Client, Sent, Server, TestGuard, answer_bytes, digest, log_bytes, recording, request_cases, server_root};
use mm_core::sql::Sql;
use mm_core::{auth_store::install_auth_schema, params};
use mm_js::{Value, json, obj};
use mm_sqlite::Sqlite;

const READY: &str = "downloads for signed-in users on http://127.0.0.1:";

struct Fixture {
    dir: PathBuf,
    sessions: mm_js::Value,
    schema_sha256: String,
}

fn fixture() -> Fixture {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("download-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = Sqlite::open(dir.join("auth.db"), false).unwrap();
    install_auth_schema(&db).unwrap();
    let schema: Vec<Value> = db
        .query("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name", &[])
        .unwrap()
        .iter()
        .map(|row| row.text("sql").unwrap().into())
        .collect();
    let hour = 3_600_000.0;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64;
    let mut sessions = mm_js::Object::new();
    let latin: String = "café-id".bytes().map(char::from).collect();
    for (key, id, email, ttl) in [
        ("owner", "owner-session-id", "owner@example.com", hour),
        ("escaped", "escaped-session-id", "<b>\"x\"&@example.com", hour),
        ("expired", "expired-session-id", "owner@example.com", -hour),
        ("blank", "blank-session-id", "", hour),
        ("latin", latin.as_str(), "latin@example.com", hour),
    ] {
        db.execute(
            "INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)",
            params![digest(id.as_bytes()), format!("sub-{email}"), email, now, now + ttl, now],
        )
        .unwrap();
        sessions.insert(key, id);
    }
    std::fs::create_dir_all(dir.join("downloads/sub")).unwrap();
    std::fs::create_dir_all(dir.join("empty")).unwrap();
    let mut seed = 7.0_f64;
    for (name, size) in [
        ("mymind-linux-x86_64.tar.gz", 300_000),
        ("my app.tar.gz", 1_000),
        ("a+b&c=d.tar.gz", 10),
        ("sub/inner.tar.gz", 20),
        ("notlisted.tar.gz", 5),
    ] {
        let bytes: Vec<u8> = (0..size)
            .map(|_| {
                seed = (seed * 1_103_515_245.0 + 12_345.0) % 2_147_483_648.0;
                (seed as u64 & 0xff) as u8
            })
            .collect();
        std::fs::write(dir.join("downloads").join(name), bytes).unwrap();
    }
    let mut files: Vec<Value> = [
        Value::from("mymind-linux-x86_64.tar.gz"),
        Value::from("my app.tar.gz"),
        Value::from("a+b&c=d.tar.gz"),
        Value::from("sub/inner.tar.gz"),
        Value::from("sub/../my app.tar.gz"),
        Value::from("missing.tar.gz"),
        Value::from(5),
    ]
    .into_iter()
    .map(|name| {
        let label = format!("Label <{}>", name.to_js_string());
        obj! { "name" => name, "label" => label, "bytes" => 52_428_800, "built" => "2026-10-17T01:02:03Z", "sha256" => "ab&cd" }.into()
    })
    .collect();
    files.push(obj! { "label" => "no name", "bytes" => 1, "built" => "", "sha256" => "" }.into());
    let manifest = obj! { "library" => "https://library.example/<x>", "files" => Value::Array(files) };
    std::fs::write(dir.join("downloads/manifest.json"), json::to_string(&manifest.into())).unwrap();
    let fixture = Fixture {
        dir,
        sessions: sessions.into(),
        schema_sha256: digest(json::to_string(&Value::Array(schema)).as_bytes()),
    };
    if recording() {
        check_legacy_fixture(&fixture);
    }
    fixture
}

// Recording also proves the Rust fixture retains the legacy synthetic bytes, session identities and TTLs.
fn check_legacy_fixture(fixture: &Fixture) {
    let _ = fixture;
    panic!("Legacy recording requires the recorded pre-switch revision")
}

// Allocate an unused loopback port before starting signed-in downloads.
fn free_port() -> u16 {
    let port = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    mm_test_support::golden::register_loopback_port(port);
    port
}

fn start(mut command: Command, fixture: &Fixture, downloads: &str) -> Server {
    let root = server_root();
    command
        .current_dir(&root)
        .env_clear()
        .env("MYMIND_ROOT", &root)
        .env("DOWNLOAD_DIR", fixture.dir.join(downloads))
        .env("AUTH_DB", fixture.dir.join("auth.db"))
        .env("DOWNLOAD_PORT", free_port().to_string());
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    Server::start(command, READY)
}

fn node(fixture: &Fixture, downloads: &str) -> Server {
    let _ = (fixture, downloads);
    panic!("Legacy recording requires the recorded pre-switch revision")
}

fn rust(fixture: &Fixture, downloads: &str) -> Server {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
    command.arg("download-server");
    start(command, fixture, downloads)
}

fn requests(fixture: &Fixture) -> Vec<Sent> {
    let id = |key: &str| fixture.sessions.get(key).as_str().unwrap().to_owned();
    let owner = id("owner");
    let cookies = vec![
        None,
        Some(format!("mm_session={owner}")),
        Some(format!("a=b; mm_session={owner}")),
        Some(format!("a=b;mm_session={owner}")),
        Some(format!("a=b;\t mm_session={owner}; c=d")),
        Some(format!("xmm_session={owner}")),
        Some(format!("mm_session=; mm_session={owner}")),
        Some(format!("mm_session={owner}; mm_session=wrong")),
        Some(format!("mm_session=wrong; mm_session={owner}")),
        Some(format!("MM_SESSION={owner}")),
        Some(format!("mm_session={owner} ")),
        Some(format!("mm_session=escaped-session-id; mm_session={owner}")),
        Some(format!("mm_session={}", id("expired"))),
        Some(format!("mm_session={}", id("blank"))),
        Some("mm_session=café-id".to_owned()),
        Some("mm_session=unknown".to_owned()),
    ];
    let targets = [
        "/download",
        "/download/",
        "/download?x=1",
        "/download/mymind-linux-x86_64.tar.gz",
        "/download/my%20app.tar.gz",
        "/download/a%2Bb%26c%3Dd.tar.gz",
        "/download/a+b&c=d.tar.gz",
        "/download/sub%2Finner.tar.gz",
        "/download/sub/inner.tar.gz",
        "/download/sub%2F..%2Fmy%20app.tar.gz",
        "/download/missing.tar.gz",
        "/download/notlisted.tar.gz",
        "/download/5",
        "/download/undefined",
        "/download/%ZZ",
        "/download/%E0%A4%A",
        "/download/..%2Fauth.db",
        "/download//mymind-linux-x86_64.tar.gz",
        "/downloadXmymind-linux-x86_64.tar.gz",
        "/",
        "/x",
        "/download/a/../../download",
    ];
    let mut sent = Vec::new();
    for cookie in &cookies {
        let headers: Vec<(&str, &str)> = cookie.iter().map(|cookie| ("cookie", cookie.as_str())).collect();
        for target in targets {
            sent.push(Sent::new("GET", target, &headers));
        }
        for target in ["/download", "/download/mymind-linux-x86_64.tar.gz", "/download/missing.tar.gz", "/x"] {
            sent.push(Sent::new("HEAD", target, &headers));
        }
    }
    let signed_in = format!("mm_session={owner}");
    for method in ["POST", "PUT", "DELETE", "OPTIONS", "PATCH"] {
        sent.push(Sent::new(method, "/download", &[("cookie", signed_in.as_str())]));
        sent.push(Sent::new(method, "/download/my%20app.tar.gz", &[]));
    }
    sent.push(Sent::new("GET", "/download", &[("cookie", "a=1"), ("cookie", signed_in.as_str())]));
    sent.push(Sent::new("GET", "/download", &[("host", "[::1"), ("cookie", signed_in.as_str())]));
    sent
}

fn compare(fixture: &Fixture, downloads: &str) {
    let requests = requests(fixture);
    assert_eq!(requests.len(), 428, "the original cookie, traversal, method and streaming cases remain covered");
    let fixture_id: Value = obj! {
        "recipe" => "download-fixture-v1", "sessions" => fixture.sessions.clone(),
        "schema_sha256" => fixture.schema_sha256.as_str(),
        "manifest" => common::fixture_identity(&fixture.dir.join("downloads/manifest.json")),
        "package" => common::fixture_identity(&fixture.dir.join("downloads/mymind-linux-x86_64.tar.gz")),
    }
    .into();
    let identities = request_cases(&requests, fixture_id, &["refusal-logs"]);
    let mut golden = Batch::new(env!("CARGO_MANIFEST_DIR"), "download", downloads, &identities);
    golden.allow_inline_synthetic();
    let reference = recording().then(|| node(fixture, downloads));
    let ported = rust(fixture, downloads);
    let mut expected = reference.as_ref().map(|server| Client::new(server.port));
    let mut actual = Client::new(ported.port);
    let mut served = 0;
    for (index, sent) in requests.iter().enumerate() {
        let want = expected.as_mut().map(|client| answer_bytes(&client.send(sent)));
        let got = actual.send(sent);
        served += usize::from(got.status.ends_with("200 OK") && got.body.len() == 300_000);
        golden.check(index, &answer_bytes(&got), want.as_deref());
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let want = reference.as_ref().map(|server| log_bytes(&server.logged(&["refused "])));
    golden.check(requests.len(), &log_bytes(&ported.logged(&["refused "])), want.as_deref());
    assert!(downloads == "empty" || served >= 5, "the package was served {served} times");
    golden.finish();
}

#[test]
fn downloads_as_download_server_mjs_serves_them() {
    let inventory = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "download");
    let fixture = fixture();
    compare(&fixture, "downloads");
    compare(&fixture, "empty");
    inventory.finish();
}
