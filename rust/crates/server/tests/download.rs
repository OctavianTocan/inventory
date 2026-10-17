//! `mymind download-server` against `node deploy/download-server.mjs`: both serve the same fixture downloads to the
//! same sign-in database (`tests/drivers/download-fixture.mjs`), with a manifest and without one.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{Client, Sent, Server, repo};
use mm_js::json;

const READY: &str = "downloads for signed-in users on http://127.0.0.1:";

struct Fixture {
    dir: PathBuf,
    sessions: mm_js::Value,
}

fn fixture() -> Fixture {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("download-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let root = repo();
    let output = Command::new("node").arg(root.join("rust/crates/server/tests/drivers/download-fixture.mjs")).arg(&dir).env("MYMIND_ROOT", &root).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    Fixture { dir, sessions: json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() }
}

// A port nothing listens on now; download-server.mjs prints the port it was given, so port 0 cannot be read back.
fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port()
}

fn start(mut command: Command, fixture: &Fixture, downloads: &str) -> Server {
    let root = repo();
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
    let mut command = Command::new("node");
    command.arg("--no-warnings").arg("deploy/download-server.mjs");
    start(command, fixture, downloads)
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
    let (reference, ported) = (node(fixture, downloads), rust(fixture, downloads));
    let (mut expected, mut actual) = (Client::new(reference.port), Client::new(ported.port));
    let (mut differences, mut served) = (Vec::new(), 0);
    for sent in &requests {
        let (want, got) = (expected.send(sent), actual.send(sent));
        served += usize::from(want.status.ends_with("200 OK") && want.body.len() == 300_000);
        if want != got {
            differences.push(format!("{} {} {:?}\n  node: {want:?}\n  rust: {got:?}", sent.method, sent.target, sent.headers));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let (want, got) = (reference.logged(&["refused "]), ported.logged(&["refused "]));
    if want != got {
        differences.push(format!("logs differ\n  node: {want:?}\n  rust: {got:?}"));
    }
    assert!(downloads == "empty" || served >= 5, "the package was served {served} times");
    let shown: Vec<&str> = differences.iter().take(30).map(String::as_str).collect();
    assert!(differences.is_empty(), "{downloads}: {} of {} requests differ:\n{}", differences.len(), requests.len(), shown.join("\n"));
}

#[test]
fn downloads_as_download_server_mjs_serves_them() {
    let fixture = fixture();
    compare(&fixture, "downloads");
    compare(&fixture, "empty");
}
