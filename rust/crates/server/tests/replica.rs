//! The replay replica against `node server.mjs`: both serve the repository's recordings offline, without a library
//! database or sign-in, and answer the same requests (`tests/drivers/replica-requests.mjs`) and `/hub` upgrades.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use common::{Client, Server, replica_requests, repo};

const READY: &str = "mymind replica on http://localhost:";

fn start(mut command: Command) -> Server {
    let root = repo();
    let store = Path::new(env!("CARGO_TARGET_TMPDIR")).join("replica-test-no-library.db");
    assert!(!store.exists());
    command.current_dir(&root).env_clear().env("MYMIND_ROOT", &root).env("STORE_DB", &store).env("STORE_LOAD_ENV", "0").env("OFFLINE", "1").env("PORT", "0");
    for name in ["PATH", "HOME"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    Server::start(command, READY)
}

// What both servers log per request: misses, ignored writes, placeholders and the first line of each error.
const LOGGED: [&str; 4] = ["[miss]", "[write ignored]", "[placeholder]", "[error]"];

fn node() -> Server {
    let mut command = Command::new("node");
    command.arg("--no-warnings").arg("server.mjs");
    start(command)
}

fn rust() -> Server {
    start(Command::new(env!("CARGO_BIN_EXE_mymind")))
}

#[test]
fn answers_as_server_mjs_answers() {
    let requests = replica_requests();
    let (reference, ported) = (node(), rust());
    let (mut expected, mut actual) = (Client::new(reference.port), Client::new(ported.port));
    let mut differences = Vec::new();
    for sent in &requests {
        let (want, got) = (expected.send(sent), actual.send(sent));
        if want != got {
            differences.push(format!("{} {} {:?}\n  node: {want:?}\n  rust: {got:?}", sent.method, sent.target, sent.headers));
        }
    }
    std::thread::sleep(Duration::from_millis(500));
    let (want, got) = (reference.logged(&LOGGED), ported.logged(&LOGGED));
    for (index, (want, got)) in want.iter().zip(&got).enumerate() {
        if want != got {
            differences.push(format!("log line {index}\n  node: {want}\n  rust: {got}"));
            break;
        }
    }
    if want.len() != got.len() {
        differences.push(format!("{} log lines from node, {} from rust", want.len(), got.len()));
    }
    let shown: Vec<&String> = differences.iter().take(40).collect();
    assert!(differences.is_empty(), "{} of {} requests differ:\n{}", differences.len(), requests.len(), shown.iter().map(|text| text.as_str()).collect::<Vec<_>>().join("\n"));
}

fn masked(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mask = [0x12, 0x34, 0x56, 0x78];
    let mut frame = vec![0x80 | opcode, 0x80 | u8::try_from(payload.len()).unwrap()];
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(index, byte)| byte ^ mask[index % 4]));
    frame
}

// Everything the server sends on one connection until it closes it or goes quiet.
fn conversation(port: u16, raw: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    stream.write_all(raw).unwrap();
    let mut received = Vec::new();
    let _ = stream.read_to_end(&mut received);
    received
}

#[test]
fn upgrades_as_server_mjs_upgrades() {
    let (reference, ported) = (node(), rust());
    let mut frames = masked(1, b"{\"protocol\":\"json\",\"version\":1}\x1e");
    frames.extend(masked(1, b"{\"type\":6}\x1e"));
    frames.extend(masked(9, b"hi"));
    frames.extend(masked(8, &[0x03, 0xe8]));
    let upgrade = |target: &str, extra: &str| {
        format!("GET {target} HTTP/1.1\r\nhost: localhost\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-version: 13\r\n{extra}\r\n")
    };
    let key = "sec-websocket-key: dGhlIHNhbXBsZSBub25jZQ==\r\n";
    let mut cases: Vec<(String, Vec<u8>)> = vec![
        ("hub with frames".to_owned(), [upgrade("/hub", key).into_bytes(), frames.clone()].concat()),
        ("hub, frames later".to_owned(), upgrade("/hub?x=1", key).into_bytes()),
        ("unmasked frame".to_owned(), [upgrade("/hub", key).into_bytes(), vec![0x81, 0x01, b'x']].concat()),
        ("bad record".to_owned(), [upgrade("/hub", key).into_bytes(), masked(1, b"nope\x1e")].concat()),
        ("no key".to_owned(), upgrade("/hub", "").into_bytes()),
        ("other path".to_owned(), upgrade("/cards", key).into_bytes()),
        ("tunnelled".to_owned(), upgrade("/hub", &format!("{key}cf-connecting-ip: 203.0.113.9\r\n")).into_bytes()),
    ];
    cases.push(("keep-alive, Upgrade".to_owned(), upgrade("/hub", key).replace("connection: Upgrade", "connection: keep-alive, Upgrade").into_bytes()));
    let normalize = |bytes: Vec<u8>| -> Vec<u8> {
        let text: String = bytes.iter().map(|&byte| char::from(byte)).collect();
        let Some((head, rest)) = text.split_once("\r\n\r\n") else { return bytes };
        let mut lines: Vec<String> = head.split("\r\n").map(|line| line.to_ascii_lowercase()).filter(|line| !line.starts_with("date:")).collect();
        lines[1..].sort();
        format!("{}\r\n\r\n{rest}", lines.join("\r\n")).chars().map(|ch| ch as u8).collect()
    };
    for (name, raw) in cases {
        let want = normalize(conversation(reference.port, &raw));
        let got = normalize(conversation(ported.port, &raw));
        assert_eq!(String::from_utf8_lossy(&want), String::from_utf8_lossy(&got), "{name}");
    }
}
