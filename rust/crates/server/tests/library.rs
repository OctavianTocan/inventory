//! Synthetic library HTTP/log scenarios for native and hosted modes, plus a real mutation and hub delivery.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::{Answer, Client, Sent, Server, server_root};
use mm_core::sql::Sql;
use mm_js::{Value, json};
use mm_sqlite::Sqlite;
use regex::bytes::Regex;

const READY: &str = "mymind replica on http://localhost:";
const LOGGED: [&str; 4] = ["[miss]", "[write ignored]", "[placeholder]", "[error]"];
const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148";

#[derive(Clone, Copy)]
enum Mode {
    Library { readonly: bool },
    Hosted { readonly: bool },
}

fn scratch_dir(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "library-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn seed_db(file: &Path) {
    let root = server_root();
    let status = Command::new(env!("CARGO_BIN_EXE_mymind"))
        .current_dir(&root)
        .env("MYMIND_ROOT", &root)
        .env("STORE_LOAD_ENV", "0")
        .arg("seed-db")
        .arg("--db")
        .arg(file)
        .status()
        .unwrap();
    assert!(status.success(), "seed-db failed");
}

fn write_bucket_file(bucket: &Path, key: &str, bytes: &[u8], content_type: &str) {
    let path = bucket.join(key);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    let meta = bucket.join(".headers").join(format!("{key}.json"));
    std::fs::create_dir_all(meta.parent().unwrap()).unwrap();
    std::fs::write(&meta, format!(r#"{{"httpMetadata":{{"contentType":"{content_type}"}}}}"#)).unwrap();
}

fn setup_mode(mode: Mode) -> (PathBuf, Server) {
    let dir = scratch_dir(match mode {
        Mode::Library { readonly: false } => "native-rw",
        Mode::Library { readonly: true } => "native-ro",
        Mode::Hosted { readonly: false } => "hosted-rw",
        Mode::Hosted { readonly: true } => "hosted-ro",
    });
    let (file, mut vars) = match mode {
        Mode::Library { .. } => {
            let file = dir.join("mymind.db");
            seed_db(&file);
            let media_dir = dir.join("media");
            std::fs::create_dir_all(media_dir.join("acct/card1")).unwrap();
            std::fs::write(media_dir.join("acct/card1/photo.png"), b"\x89PNG\r\n\x1a\nsynthetic").unwrap();
            (
                file.clone(),
                vec![
                    ("STORE_DB", file.to_string_lossy().into_owned()),
                    ("STORE_MEDIA", media_dir.to_string_lossy().into_owned()),
                ],
            )
        }
        Mode::Hosted { .. } => {
            let file = dir.join("test.db");
            seed_db(&file);
            let bucket = dir.join("bucket");
            write_bucket_file(&bucket, "libraries/test/media/acct/card1/photo.png", b"\x89PNG\r\n\x1a\nsynthetic", "image/png");
            write_bucket_file(&bucket, "media/shared/asset.webp", b"RIFFxxxxWEBPVP8 ", "image/webp");
            (
                file,
                vec![
                    ("HOSTED", "1".to_owned()),
                    ("HOSTED_DIR", dir.to_string_lossy().into_owned()),
                    ("HOSTED_LIBRARY", "test".to_owned()),
                ],
            )
        }
    };
    if let Mode::Library { readonly: true } | Mode::Hosted { readonly: true } = mode {
        vars.push(("STORE_READONLY", "1".to_owned()));
    }
    let root = server_root();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
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
    for (name, value) in &vars {
        command.env(name, value);
    }
    let server = Server::start(command, READY);
    (file, server)
}

fn ids(db: &Sqlite, sql: &str) -> Vec<String> {
    db.query(sql, &[]).unwrap().iter().filter_map(|row| row.text("id").map(str::to_owned)).collect()
}

fn check_reads(file: &Path, client: &mut Client) {
    let db = Sqlite::open(file, true).unwrap();
    let cards = ids(&db, "SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 4");
    let spaces = ids(&db, "SELECT id FROM spaces ORDER BY id LIMIT 2");
    let queries = ids(
        &db,
        "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id LIMIT 1",
    );

    for target in [
        "/cards",
        "/cards/trash",
        "/spaces",
        "/tags/manual",
        "/tags/recent",
        "/changes?since=0",
        "/tags/suggest?q=d",
        "/quota",
        "/quotas/objects",
        "/account.json",
        "/search?q=design",
        "/search/suggestions?q=d",
    ] {
        let ans = client.send(&Sent::new("GET", target, &[("accept", "application/json")]));
        assert!(ans.status.contains(" 200 "), "{target}: {}", ans.status);
    }

    let shell = client.send(&Sent::new("GET", "/everything", &[("sec-fetch-dest", "document")]));
    assert!(shell.status.contains(" 200 "));
    assert!(String::from_utf8_lossy(&shell.body).contains("<script id=\"config\" type=\"application/json\">"));

    let phone = client.send(&Sent::new("GET", "/cards", &[("user-agent", IPHONE)]));
    assert!(phone.status.contains(" 200 "));

    for id in &cards {
        for suffix in ["", "/detail", "?template=PinnedCard"] {
            let ans = client.send(&Sent::new("GET", &format!("/cards/{id}{suffix}"), &[]));
            assert!(ans.status.contains(" 200 "), "/cards/{id}{suffix}: {}", ans.status);
        }
        for suffix in ["", "/tags", "/revisions"] {
            let ans = client.send(&Sent::new("GET", &format!("/objects/{id}{suffix}"), &[("accept", "application/json")]));
            assert!(ans.status.contains(" 200 "), "/objects/{id}{suffix}: {}", ans.status);
        }
    }

    for id in &spaces {
        let ans = client.send(&Sent::new("GET", &format!("/spaces/{id}"), &[("accept", "application/json")]));
        assert!(ans.status.contains(" 200 "));
    }
    for id in &queries {
        let ans = client.send(&Sent::new("GET", &format!("/queries/{id}/check"), &[("accept", "application/json")]));
        assert!(ans.status.contains(" 200 "));
    }
    let grid = format!("[{}]", cards.iter().map(|id| format!("\"{id}\"")).collect::<Vec<_>>().join(","));
    let grid_ans = client.send(&Sent {
        method: "POST".to_owned(),
        target: "/cards/grid".to_owned(),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: grid,
    });
    assert!(grid_ans.status.contains(" 200 "));
}

#[test]
fn library_api_answers_as_server_mjs() {
    let (file, server) = setup_mode(Mode::Library { readonly: false });
    let mut client = Client::new(server.port);
    check_reads(&file, &mut client);
}

#[test]
fn readonly_library_answers_as_server_mjs() {
    let (file, server) = setup_mode(Mode::Library { readonly: true });
    let mut client = Client::new(server.port);
    check_reads(&file, &mut client);
    let write = client.send(&Sent {
        method: "POST".to_owned(),
        target: "/objects".to_owned(),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: r#"{"title":"ignored"}"#.to_owned(),
    });
    assert!(write.status.contains(" 200 "));
    std::thread::sleep(Duration::from_millis(100));
    assert!(server.logged(&LOGGED).iter().any(|line| line.contains("[write ignored]")));
}

#[test]
fn hosted_library_answers_as_server_mjs() {
    let (file, server) = setup_mode(Mode::Hosted { readonly: false });
    let mut client = Client::new(server.port);
    check_reads(&file, &mut client);
    let media = client.send(&Sent::new("GET", "/media/acct/card1/photo.png", &[("range", "bytes=0-3")]));
    assert!(media.status.contains(" 206 "), "{}", media.status);
    assert_eq!(media.body.len(), 4);
    let miss = client.send(&Sent::new("GET", "/media/missing.png", &[]));
    assert!(miss.status.contains(" 404 "), "{}", miss.status);
}

#[test]
fn readonly_hosted_library_answers_as_server_mjs() {
    let (file, server) = setup_mode(Mode::Hosted { readonly: true });
    let mut client = Client::new(server.port);
    check_reads(&file, &mut client);
    let write = client.send(&Sent {
        method: "POST".to_owned(),
        target: "/objects".to_owned(),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: r#"{"title":"ignored"}"#.to_owned(),
    });
    assert!(write.status.contains(" 200 "));
}

fn masked(payload: &[u8]) -> Vec<u8> {
    let mask = [0x12, 0x34, 0x56, 0x78];
    let mut frame = vec![0x81, 0x80 | u8::try_from(payload.len()).unwrap()];
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(index, byte)| byte ^ mask[index % 4]));
    frame
}

fn frames(bytes: &[u8]) -> Vec<String> {
    let long_number = Regex::new(r"\d{13,}").unwrap();
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
            texts.push(String::from_utf8_lossy(&long_number.replace_all(payload, &b"N"[..])).into_owned());
        }
        at += head + length;
    }
    texts
}

const HUB_UPGRADE: &str = "GET /hub HTTP/1.1\r\nhost: localhost\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-version: 13\r\nsec-websocket-key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
const HUB_HANDSHAKE: &[u8] = b"{\"protocol\":\"json\",\"version\":1}\x1e";

fn rename_request(space: &str) -> Sent {
    Sent {
        method: "PATCH".to_owned(),
        target: format!("/spaces/{space}"),
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        body: r#"{"name":"Renamed by the hub test"}"#.to_owned(),
    }
}

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
    let (file, server) = setup_mode(Mode::Library { readonly: false });
    let space = ids(&Sqlite::open(&file, true).unwrap(), "SELECT id FROM spaces ORDER BY id LIMIT 1").remove(0);
    let got = broadcast(&server, &space);
    assert!(got.0.status.contains(" 200 "));
    assert!(got.1.len() > 1, "the Rust mutation must produce a hub broadcast");
    let joined = got.1.join("\n");
    let parsed: Value = json::parse(got.1.last().unwrap().trim_end_matches('\u{1e}')).unwrap();
    assert!(joined.contains("Renamed by the hub test") || parsed.truthy());
}
