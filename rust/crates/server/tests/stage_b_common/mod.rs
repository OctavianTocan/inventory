//! Isolated native HTTP fixtures for the assertion ports, with synthetic environment and loopback transport.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mm_core::sql::{Row, Sql};
use mm_core::store::Store;
use mm_js::{Value, json, obj};
use mm_sqlite::Sqlite;

#[path = "../common/mod.rs"]
mod http;

pub use http::{Answer, Client, Sent, Server};

/// Allocate scratch only on the disk-backed Cargo target directory.
///
/// # Arguments
/// * `name` - Synthetic scenario label used in the unique directory name.
///
/// # Returns
/// A new directory below CARGO_TARGET_TMPDIR.
pub fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "stage-b-{name}-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        mm_core::runtime::random_uuid()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Link read-only recordings into a fresh root without environment files.
///
/// # Arguments
/// * `name` - Synthetic scenario label for the fresh scratch directory.
///
/// # Returns
/// The isolated root with capture, mirror and replay symlinks.
pub fn root(name: &str) -> PathBuf {
    let root = scratch(name);
    std::os::unix::fs::symlink(http::repo().join("mirror"), root.join("mirror")).unwrap();
    assert!(!root.join(".env").exists() && !root.join(".dev.vars").exists());
    root
}

/// Reconstruct the original HTTP check library from the committed recordings with the Rust seeder.
///
/// # Arguments
/// * `root` - Isolated scratch root with linked immutable recording inputs.
///
/// # Returns
/// A fresh writable library built with the original verify-database seed recipe.
pub fn library(root: &Path) -> PathBuf {
    let target = root.join("library.db");
    let seeded = command(root)
        .arg("seed-db")
        .arg("--db")
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        seeded.status.success(),
        "Rust recording fixture seeding succeeds"
    );
    target
}

/// Construct an empty library with a synthetic user.
///
/// # Arguments
/// * `path` - New SQLite file below the scratch root.
pub fn synthetic(path: &Path) {
    let db = std::rc::Rc::new(Sqlite::open(path, false).unwrap());
    let _store = Store::new(db.clone()).unwrap();
    db.execute(
        "INSERT INTO users VALUES (?, ?, ?, 0, ?)",
        &[
            "search-user".into(),
            "search-account".into(),
            "fixture0000".into(),
            r#"{"objectLimit":10000}"#.into(),
        ],
    )
    .unwrap();
}

/// Configure a native subprocess without inherited credentials or environment files.
///
/// # Arguments
/// * `root` - Isolated scratch root and process working directory.
///
/// # Returns
/// The Rust binary command with an explicitly absent environment file.
pub fn command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
    command
        .current_dir(root)
        .env_clear()
        .env("MYMIND_ROOT", root)
        .env("MYMIND_ENV_FILE", root.join("absent-env"))
        .env("STORE_LOAD_ENV", "0")
        .env("OFFLINE", "1")
        .env("PORT", "0")
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"));
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
}

/// Start only the Rust server against the requested scratch library.
///
/// # Arguments
/// * `root` - Isolated scratch root with linked recordings.
/// * `db` - Scratch library file to serve.
/// * `readonly` - Whether to acknowledge mutations without writing.
/// * `env` - Additional synthetic process variables.
///
/// # Returns
/// The ready native process, killed when dropped.
pub fn start(root: &Path, db: &Path, readonly: bool, env: &[(&str, &str)]) -> Server {
    let mut command = command(root);
    command
        .arg("serve")
        .env("STORE_DB", db)
        .env("STORE_READONLY", if readonly { "1" } else { "0" });
    for (name, value) in env {
        command.env(name, value);
    }
    Server::start(command, "mymind replica on http://localhost:")
}

/// Decode JSON, HTML or concatenated MessagePack values as the original checker does.
///
/// # Arguments
/// * `answer` - The complete HTTP response to decode.
///
/// # Returns
/// The JSON value, HTML string or array of decoded stream packets.
pub fn value(answer: &Answer) -> Value {
    match header(answer, "content-type")
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap()
    {
        "application/msgpack" => Value::Array(
            mm_msgpack::decode_multi(
                &answer.body,
                mm_msgpack::DecodeOptions {
                    use_big_int64: true,
                },
            )
            .unwrap(),
        ),
        "application/json" | "application/prose+json" | "application/problem+json" => {
            json::parse(std::str::from_utf8(&answer.body).unwrap()).unwrap()
        }
        _ => Value::String(String::from_utf8(answer.body.clone()).unwrap()),
    }
}

/// Read one response header without changing its value.
///
/// # Arguments
/// * `answer` - The complete response containing normalized header names.
/// * `name` - Lower-case header name to look up.
///
/// # Returns
/// The first matching value, or None when absent.
pub fn header<'a>(answer: &'a Answer, name: &str) -> Option<&'a str> {
    answer
        .headers
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

/// Check the numeric HTTP status without logging fixture bodies.
///
/// # Arguments
/// * `answer` - The complete response whose status is checked.
/// * `expected` - Expected HTTP status code.
pub fn status(answer: &Answer, expected: u16) {
    assert_eq!(
        answer
            .status
            .get(9..12)
            .and_then(|code| code.parse::<u16>().ok()),
        Some(expected),
        "HTTP status"
    );
}

/// Send an explicit JSON request with independently selectable negotiation.
///
/// # Arguments
/// * `client` - Loopback HTTP client to send through.
/// * `method` - HTTP request method.
/// * `path` - Local absolute path and optional query.
/// * `body` - Undefined for no body, a string for exact bytes, otherwise JSON.
/// * `accept` - Desired response content type.
/// * `content_type` - Request body content type.
///
/// # Returns
/// The exact response with transport headers normalized by the HTTP test client.
pub fn request(
    client: &mut Client,
    method: &str,
    path: &str,
    body: &Value,
    accept: &str,
    content_type: &str,
) -> Answer {
    let mut sent = Sent::new(
        method,
        path,
        &[("accept", accept), ("content-type", content_type)],
    );
    if !matches!(body, Value::Undefined) {
        sent.body = if let Value::String(text) = body {
            text.clone()
        } else {
            json::to_string(body)
        };
    }
    client.send(&sent)
}

/// Read a successful JSON endpoint.
///
/// # Arguments
/// * `client` - Loopback HTTP client to send through.
/// * `path` - Local endpoint path and optional query.
///
/// # Returns
/// The decoded response after requiring status 200.
pub fn get(client: &mut Client, path: &str) -> Value {
    let answer = request(
        client,
        "GET",
        path,
        &Value::Undefined,
        "application/json",
        "application/json",
    );
    status(&answer, 200);
    value(&answer)
}

/// Require a successful mutation and the quota response header.
///
/// # Arguments
/// * `client` - Loopback HTTP client to send through.
/// * `method` - Mutation HTTP method.
/// * `path` - Local mutation path.
/// * `body` - JSON input, exact string bytes or Undefined for no body.
/// * `expected` - Expected successful status code.
///
/// # Returns
/// The decoded response after verifying the status and quota header.
pub fn write(client: &mut Client, method: &str, path: &str, body: Value, expected: u16) -> Value {
    let answer = request(
        client,
        method,
        path,
        &body,
        "application/json",
        "application/json",
    );
    status(&answer, expected);
    assert!(
        header(&answer, "x-object-quota").is_some(),
        "write quota header"
    );
    value(&answer)
}

/// Build the tiny editor document used in the original assertion suites.
///
/// # Arguments
/// * `text` - Visible paragraph text.
///
/// # Returns
/// A document with one paragraph and one text node.
pub fn doc(text: &str) -> Value {
    obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => text })] })] }.into()
}

/// Compare values without disclosing complete recorded card bodies on failure.
///
/// # Arguments
/// * `actual` - Observed value.
/// * `expected` - Required value.
/// * `context` - Bounded public assertion label.
pub fn same(actual: &Value, expected: &Value, context: &str) {
    assert!(
        actual == expected,
        "{context}: {}",
        difference(actual, expected, "$")
    );
}

// Name only the first differing structural path, never its recorded field values.
fn difference(actual: &Value, expected: &Value, path: &str) -> String {
    match (actual, expected) {
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return format!(
                    "{path}: array lengths {} versus {}",
                    left.len(),
                    right.len()
                );
            }
            left.iter()
                .zip(right)
                .enumerate()
                .find(|(_, (left, right))| left != right)
                .map_or_else(
                    || format!("{path}: array differs"),
                    |(index, (left, right))| difference(left, right, &format!("{path}[{index}]")),
                )
        }
        (Value::Object(left), Value::Object(right)) => {
            if left.len() != right.len() {
                return format!(
                    "{path}: object field counts {} versus {}",
                    left.len(),
                    right.len()
                );
            }
            left.iter()
                .find(|(key, left)| right.get(key).is_none_or(|right| *left != right))
                .map_or_else(
                    || format!("{path}: object differs"),
                    |(key, left)| {
                        difference(
                            left,
                            right.get(&key).unwrap_or(&Value::Undefined),
                            &format!("{path}.{key}"),
                        )
                    },
                )
        }
        _ => format!("{path}: scalar values or types differ"),
    }
}

/// Read a required string field from a synthetic response.
///
/// # Arguments
/// * `value` - Synthetic response object containing an id field.
///
/// # Returns
/// An owned copy of the id.
pub fn id(value: &Value) -> String {
    value.get("id").as_str().expect("id").to_owned()
}

/// Test an array's membership by id.
///
/// # Arguments
/// * `value` - Response array of objects with id fields.
/// * `id` - Object identifier to find.
///
/// # Returns
/// Whether any element has the required id.
pub fn contains(value: &Value, id: &str) -> bool {
    value
        .as_array()
        .expect("array")
        .iter()
        .any(|item| item.get("id").as_str() == Some(id))
}

/// Snapshot every table and typed value in deterministic column order.
///
/// # Arguments
/// * `file` - Scratch database whose complete stored state is observed.
///
/// # Returns
/// Ordered table names and typed rows, including internal and search-index tables.
pub fn snapshot(file: &Path) -> Vec<(String, Vec<Row>)> {
    let db = Sqlite::open(file, true).unwrap();
    db.query(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
        &[],
    )
    .unwrap()
    .iter()
    .map(|table| {
        let name = table.text("name").unwrap();
        let quoted = format!("\"{}\"", name.replace('"', "\"\""));
        let columns = db
            .query(&format!("PRAGMA table_info({quoted})"), &[])
            .unwrap()
            .iter()
            .map(|column| format!("\"{}\"", column.text("name").unwrap().replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",");
        (
            name.to_owned(),
            db.query(&format!("SELECT * FROM {quoted} ORDER BY {columns}"), &[])
                .unwrap(),
        )
    })
    .collect()
}

// Read a complete server-to-client WebSocket frame.
fn frame(reader: &mut BufReader<TcpStream>) -> std::io::Result<Vec<u8>> {
    let mut first = [0; 2];
    reader.read_exact(&mut first)?;
    let length = match first[1] & 127 {
        126 => {
            let mut bytes = [0; 2];
            reader.read_exact(&mut bytes)?;
            u16::from_be_bytes(bytes) as usize
        }
        127 => {
            let mut bytes = [0; 8];
            reader.read_exact(&mut bytes)?;
            u64::from_be_bytes(bytes) as usize
        }
        length => length as usize,
    };
    assert!(length < 16 * 1024 * 1024, "bounded hub frame");
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// A live SignalR socket with an ordered collector for committed mutation events.
pub struct Hub {
    stream: TcpStream,
    messages: Arc<Mutex<Vec<Value>>>,
}

impl Hub {
    /// Upgrade a loopback socket and complete the SignalR JSON handshake before mutations begin.
    ///
    /// # Arguments
    /// * `port` - Port of the already running native server.
    ///
    /// # Returns
    /// The connected socket and its background message collector.
    pub fn connect(port: u16) -> Self {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        write!(stream, "GET /hub HTTP/1.1\r\nhost: localhost:{port}\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-version: 13\r\nsec-websocket-key: bG9jYWwtdGVzdC1zb2NrZXQ=\r\n\r\n").unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("101"), "hub upgrade");
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
        }
        let payload = b"{\"protocol\":\"json\",\"version\":1}\x1e";
        stream
            .write_all(&[&[0x81, 0x80 | payload.len() as u8, 0, 0, 0, 0][..], payload].concat())
            .unwrap();
        assert_eq!(frame(&mut reader).unwrap(), b"{}\x1e", "SignalR handshake");
        let messages = Arc::new(Mutex::new(Vec::new()));
        let copy = messages.clone();
        std::thread::spawn(move || {
            while let Ok(bytes) = frame(&mut reader) {
                for text in bytes
                    .split(|byte| *byte == 0x1e)
                    .filter(|part| !part.is_empty())
                {
                    if let Ok(text) = std::str::from_utf8(text)
                        && let Ok(value) = json::parse(text)
                    {
                        copy.lock().unwrap().push(value);
                    }
                }
            }
        });
        Self { stream, messages }
    }
    /// Snapshot delivered messages, allowing a bounded wait for an expected event.
    ///
    /// # Arguments
    /// * `target` - Optional event target to wait up to two seconds for.
    ///
    /// # Returns
    /// All delivered SignalR messages in arrival order.
    pub fn messages(&self, target: Option<&str>) -> Vec<Value> {
        let start = Instant::now();
        loop {
            let messages = self.messages.lock().unwrap().clone();
            if target.is_none_or(|target| {
                messages
                    .iter()
                    .any(|message| message.get("target").as_str() == Some(target))
            }) || start.elapsed() > Duration::from_secs(2)
            {
                return messages;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Hub {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

/// A loopback hosted API script and its complete synthetic request transcript.
pub struct FakeApi {
    /// Origin of the local fake provider.
    pub base: String,
    calls: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl FakeApi {
    /// Serve a scripted hosted API exclusively on loopback, retaining complete synthetic request bodies.
    ///
    /// # Arguments
    /// * `handler` - Synthetic request number and parsed body to an HTTP status and JSON response.
    ///
    /// # Returns
    /// A loopback origin, request collector and server stopped when dropped.
    pub fn start(handler: impl Fn(usize, &Value) -> (u16, Value) + Send + 'static) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let calls = Arc::new(Mutex::new(Vec::new()));
        let copy = calls.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(
                    line.starts_with("POST /"),
                    "fake API accepts only local POST requests"
                );
                let mut length = 0;
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((name, value)) = line.split_once(':')
                        && name.eq_ignore_ascii_case("content-length")
                    {
                        length = value.trim().parse().unwrap();
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                let body = json::parse(std::str::from_utf8(&body).unwrap()).unwrap();
                let index = {
                    let mut calls = copy.lock().unwrap();
                    calls.push(body.clone());
                    calls.len()
                };
                let (status, body) = handler(index, &body);
                let body = json::to_string(&body);
                write!(stream, "HTTP/1.1 {status} Fixture\r\ncontent-type: application/json\r\ncontent-length: {}\r\nretry-after: 0\r\nconnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        Self {
            base,
            calls,
            stop,
            thread: Some(thread),
        }
    }
    /// Return every synthetic provider request in arrival order.
    ///
    /// # Returns
    /// A complete snapshot of parsed request bodies.
    pub fn calls(&self) -> Vec<Value> {
        self.calls.lock().unwrap().clone()
    }
}
impl Drop for FakeApi {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

/// Install a syscall-level fixture destination guard on an HTTP-check subprocess.
///
/// # Arguments
/// * `command` - Child command configured without inherited credentials.
/// * `root` - Target-local directory for the compiled guard.
/// * `base` - The only permitted loopback fixture origin.
///
/// # Panics
/// If the fixture origin is not local or its C guard fails to compile.
pub fn fixture_connections(command: &mut Command, root: &Path, base: &str) {
    let base = url::Url::parse(base).unwrap();
    assert_eq!(base.host_str(), Some("127.0.0.1"));
    let library = root.join("fixture-connect.so");
    if !library.exists() {
        let output = Command::new("cc")
            .args(["-shared", "-fPIC", "-O2", "-Wall", "-Wextra", "-Werror"])
            .arg("-o")
            .arg(&library)
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/stage_b_common/fixture_connect.c"),
            )
            .arg("-ldl")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fixture connection guard: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    command.env("LD_PRELOAD", library).env(
        "MYMIND_FIXTURE_CONNECT_PORT",
        base.port_or_known_default().unwrap().to_string(),
    );
}

/// Run the native embedding command with a synthetic hosted API endpoint.
///
/// # Arguments
/// * `root` - Isolated root with no environment files.
/// * `db` - Scratch library to estimate or update.
/// * `args` - Additional explicit CLI flags.
/// * `base` - Loopback fake provider origin.
///
/// # Returns
/// Captured exit status, standard output and standard error.
pub fn backfill(root: &Path, db: &Path, args: &[&str], base: &str) -> Output {
    let mut child = command(root);
    fixture_connections(&mut child, root, base);
    let output = child
        .arg("embed-library")
        .arg("--db")
        .arg(db)
        .args(args)
        .env("GEMINI_API_KEY", "synthetic-test-key")
        .env("GEMINI_BASE_URL", base)
        .output()
        .unwrap();
    for bytes in [&output.stdout, &output.stderr] {
        assert!(
            !bytes
                .windows(b"synthetic-test-key".len())
                .any(|window| window == b"synthetic-test-key"),
            "provider secret is absent from command logs"
        );
    }
    output
}
