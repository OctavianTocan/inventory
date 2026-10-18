//! Native HTTP tests compare normalized responses with frozen pre-switch expectations.
// Each test binary uses only part of it.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use mm_js::{Value, json, obj};
use sha2::{Digest, Sha256};

#[allow(unused_imports)]
pub use mm_test_support::golden::{Batch, TestGuard, recording};

/// The repository root: `MYMIND_ROOT`, or the checkout this crate sits in.
pub fn repo() -> PathBuf {
    std::env::var_os("MYMIND_ROOT")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."), PathBuf::from)
        .canonicalize()
        .unwrap()
}

/// Build an isolated server root with the recorded assets and no environment files.
///
/// # Returns
/// A process-specific temporary directory, reused by this test binary.
///
/// # Panics
/// If a required immutable asset cannot be linked.
pub fn server_root() -> PathBuf {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let parent = Path::new(env!("CARGO_TARGET_TMPDIR"));
        std::fs::create_dir_all(parent).unwrap();
        let mut counter = 0u64;
        let root = loop {
            let candidate = parent.join(format!("http-root-{}-{counter}", std::process::id()));
            match std::fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => counter += 1,
                Err(error) => panic!("cannot create isolated server root: {error}"),
            }
        };
        for name in ["capture", "mirror", "replay"] {
            std::os::unix::fs::symlink(repo().join(name), root.join(name)).unwrap();
        }
        assert!(!root.join(".env").exists() && !root.join(".dev.vars").exists());
        root
    })
    .clone()
}

/// Hash opaque bytes without retaining private response or library contents.
///
/// # Arguments
/// * `bytes` - Exact bytes to fingerprint.
///
/// # Returns
/// Lower-case SHA-256 hexadecimal text.
pub fn digest(bytes: &[u8]) -> String {
    mm_js::base64::hex(&Sha256::digest(bytes))
}

/// Fingerprint an immutable fixture file before a server can mutate its copy.
///
/// # Arguments
/// * `file` - Existing frozen fixture.
///
/// # Returns
/// The file's byte length and SHA-256, without its path.
///
/// # Panics
/// If the fixture cannot be read.
pub fn fixture_identity(file: &Path) -> Value {
    let bytes = std::fs::read(file).unwrap();
    obj! { "length" => bytes.len(), "sha256" => digest(&bytes) }.into()
}

// Replace assigned paths so scratch directories never enter lookup identity or expected metadata.
fn normalized(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    for (path, placeholder) in [(server_root(), "<root>"), (repo(), "<repo>")] {
        bytes = replace(&bytes, path.to_string_lossy().as_bytes(), placeholder.as_bytes());
    }
    bytes
}

/// Encode complete ordered scenarios into request identities, including fixture state and trailing observations.
///
/// # Arguments
/// * `requests` - Ordered requests, including duplicate cases.
/// * `fixture` - Fingerprint of the initial database or synthetic fixture recipe.
/// * `observations` - Names of additional observations, such as aggregate application logs.
///
/// # Returns
/// One opaque identity per request followed by one for each additional observation.
pub fn request_cases(requests: &[Sent], fixture: Value, observations: &[&str]) -> Vec<Vec<u8>> {
    let requests: Vec<Vec<u8>> = requests.iter().map(Sent::identity).collect();
    let mut scenario = json::to_string(&fixture).into_bytes();
    for request in &requests {
        scenario.extend_from_slice(&(request.len() as u64).to_be_bytes());
        scenario.extend_from_slice(request);
    }
    let scenario = digest(&scenario);
    requests
        .into_iter()
        .enumerate()
        .map(|(index, request)| [scenario.as_bytes(), b"\0request\0", &(index as u64).to_be_bytes(), &request].concat())
        .chain(
            observations
                .iter()
                .map(|name| [scenario.as_bytes(), b"\0observation\0", name.as_bytes()].concat()),
        )
        .collect()
}

/// Encode status, ordered normalized headers and a body's length and hash, without its content.
///
/// # Arguments
/// * `answer` - Response after the test's clock and fixture-copy normalization.
///
/// # Returns
/// Canonical JSON bytes suitable for golden comparison.
pub fn answer_bytes(answer: &Answer) -> Vec<u8> {
    let headers: Vec<Value> = answer
        .headers
        .iter()
        .map(|(name, value)| Value::Array(vec![name.clone().into(), value.clone().into()]))
        .collect();
    let body = normalized(&answer.body);
    normalized(
        json::to_string(
            &obj! {
                "status" => answer.status.as_str(),
                "headers" => Value::Array(headers),
                "body" => obj! { "length" => body.len(), "sha256" => digest(&body) },
            }
            .into(),
        )
        .as_bytes(),
    )
}

/// Preserve the ordered application-log transcript without storing its potentially private text.
///
/// # Arguments
/// * `lines` - Already filtered and normalized application logs in arrival order.
///
/// # Returns
/// Canonical JSON with a length and SHA-256 for every line, preserving count and order.
pub fn log_bytes(lines: &[String]) -> Vec<u8> {
    let lines = lines
        .iter()
        .map(|line| {
            let bytes = normalized(line.as_bytes());
            Value::from(obj! { "length" => bytes.len(), "sha256" => digest(&bytes) })
        })
        .collect();
    json::to_string(&Value::Array(lines)).into_bytes()
}

/// A server process, killed when dropped, with everything it printed.
pub struct Server {
    child: Child,
    pub port: u16,
    log: Arc<Mutex<Vec<String>>>,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Server {
    /// Start `command`, its environment already set, and wait for the stdout line that starts with `ready` and
    /// names the port after it.
    pub fn start(mut command: Command, ready: &'static str) -> Self {
        let mut child = command.env("TMPDIR", env!("CARGO_TARGET_TMPDIR")).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let (port_tx, port_rx) = mpsc::channel();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let lines = Arc::clone(&log);
        std::thread::spawn(move || {
            for line in stdout.lines().map_while(Result::ok) {
                if let Some(port) = line.strip_prefix(ready).and_then(|rest| rest.split('/').next()) {
                    let _ = port_tx.send(port.parse::<u16>().unwrap());
                }
                lines.lock().unwrap().push(line);
            }
        });
        let stderr = BufReader::new(child.stderr.take().unwrap());
        let lines = Arc::clone(&log);
        std::thread::spawn(move || {
            for line in stderr.lines().map_while(Result::ok) {
                lines.lock().unwrap().push(line);
            }
        });
        let port = port_rx.recv_timeout(Duration::from_secs(120)).expect("the server starts");
        mm_test_support::golden::register_loopback_port(port);
        Self { child, port, log }
    }

    /// The lines the server printed so far that start with one of `prefixes`.
    pub fn logged(&self, prefixes: &[&str]) -> Vec<String> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|line| prefixes.iter().any(|prefix| line.starts_with(prefix)))
            .cloned()
            .collect()
    }
}

/// A request to send.
pub struct Sent {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Sent {
    /// A request without a body.
    pub fn new(method: &str, target: &str, headers: &[(&str, &str)]) -> Self {
        let headers = headers.iter().map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect();
        Self {
            method: method.to_owned(),
            target: target.to_owned(),
            headers,
            body: String::new(),
        }
    }

    /// Encode every request field in order, normalizing only assigned scratch paths.
    ///
    /// # Returns
    /// Canonical JSON request bytes; repeated header order and exact body bytes are preserved.
    pub fn identity(&self) -> Vec<u8> {
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| Value::Array(vec![name.clone().into(), value.clone().into()]))
            .collect();
        normalized(
            json::to_string(
                &obj! {
                    "method" => self.method.as_str(), "target" => self.target.as_str(),
                    "headers" => Value::Array(headers), "body" => self.body.as_str(),
                }
                .into(),
            )
            .as_bytes(),
        )
    }
}

/// An answer as the tests compare it.
#[derive(PartialEq, Eq)]
pub struct Answer {
    pub status: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl std::fmt::Debug for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = String::from_utf8_lossy(&self.body);
        let shown: String = body.chars().take(300).collect();
        write!(f, "{} {:?} ({} bytes) {shown:?}", self.status, self.headers, self.body.len())
    }
}

fn replace(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut last = 0;
    for at in memchr::memmem::find_iter(bytes, from) {
        out.extend_from_slice(&bytes[last..at]);
        out.extend_from_slice(to);
        last = at + from.len();
    }
    out.extend_from_slice(&bytes[last..]);
    out
}

/// A keep-alive HTTP/1.1 client of one server.
pub struct Client {
    port: u16,
    stream: Option<BufReader<TcpStream>>,
}

impl Client {
    /// A client that ignores `content-length` outside `HEAD` answers: Node chunks a body after `writeHead` where
    /// hyper sends its length, and the bodies are compared anyway.
    pub fn new(port: u16) -> Self {
        Self { port, stream: None }
    }

    fn raw(&self, sent: &Sent) -> Vec<u8> {
        let mut head = format!("{} {} HTTP/1.1\r\n", sent.method, sent.target);
        if !sent.headers.iter().any(|(name, _)| name == "host") {
            head.push_str(&format!("host: localhost:{}\r\n", self.port));
        }
        for (name, value) in &sent.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if !sent.body.is_empty() || matches!(sent.method.as_str(), "POST" | "PUT" | "PATCH") {
            head.push_str(&format!("content-length: {}\r\n", sent.body.len()));
        }
        head.push_str("\r\n");
        let mut raw = head.into_bytes();
        raw.extend_from_slice(sent.body.as_bytes());
        raw
    }

    /// Send `sent` on the kept-alive connection, opening a new one when the server closed it.
    pub fn send(&mut self, sent: &Sent) -> Answer {
        for _ in 0..2 {
            if self.stream.is_none() {
                let Ok(stream) = TcpStream::connect(("127.0.0.1", self.port)) else {
                    return Answer {
                        status: "connection refused".to_owned(),
                        headers: Vec::new(),
                        body: Vec::new(),
                    };
                };
                stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
                self.stream = Some(BufReader::new(stream));
            }
            let raw = self.raw(sent);
            let reader = self.stream.as_mut().unwrap();
            if reader.get_mut().write_all(&raw).is_ok()
                && let Some(answer) = read_answer(reader, sent.method == "HEAD")
            {
                if answer.1 {
                    self.stream = None;
                }
                return self.normalized(answer.0, sent.method == "HEAD");
            }
            self.stream = None;
        }
        Answer {
            status: "no answer".to_owned(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn normalized(&self, answer: Answer, head: bool) -> Answer {
        let hop = ["date", "connection", "keep-alive", "transfer-encoding"];
        let port = format!(":{}", self.port);
        let headers = answer
            .headers
            .into_iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value.replace(&port, ":PORT")))
            .filter(|(name, _)| !hop.contains(&name.as_str()) && (head || name != "content-length"))
            .collect();
        Answer {
            status: answer.status,
            headers,
            body: replace(&answer.body, port.as_bytes(), b":PORT"),
        }
    }
}

// One response from a kept-alive connection, and whether the server closes the connection after it.
fn read_answer(reader: &mut BufReader<TcpStream>, head: bool) -> Option<(Answer, bool)> {
    let mut line = String::new();
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }
    let status = line.trim_end().to_owned();
    let mut headers = Vec::new();
    loop {
        let mut line = Vec::new();
        reader.read_until(b'\n', &mut line).ok()?;
        let line: String = line.iter().map(|&byte| char::from(byte)).collect();
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.to_owned(), value.trim().to_owned()));
    }
    let header = |wanted: &str| {
        headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(wanted))
            .map(|(_, value)| value.to_ascii_lowercase())
    };
    let code: u16 = status.get(9..12)?.parse().ok()?;
    let closes = header("connection").is_some_and(|value| value.contains("close"));
    let mut body = Vec::new();
    if head || code == 204 || code == 304 || code < 200 {
    } else if header("transfer-encoding").is_some_and(|value| value.contains("chunked")) {
        loop {
            let mut size = String::new();
            reader.read_line(&mut size).ok()?;
            let size = usize::from_str_radix(size.trim().split(';').next()?, 16).ok()?;
            if size == 0 {
                let mut trailer = String::new();
                while reader.read_line(&mut trailer).ok()? > 2 {
                    trailer.clear();
                }
                break;
            }
            let start = body.len();
            body.resize(start + size + 2, 0);
            reader.read_exact(&mut body[start..]).ok()?;
            body.truncate(start + size);
        }
    } else if let Some(length) = header("content-length") {
        body.resize(length.parse().ok()?, 0);
        reader.read_exact(&mut body).ok()?;
    } else {
        reader.read_to_end(&mut body).ok()?;
        return Some((Answer { status, headers, body }, true));
    }
    Some((Answer { status, headers, body }, closes))
}

/// Load the immutable pre-switch request inventory without executing an enumerator.
///
/// # Returns
/// Every replayed and recorded request and the original route edge cases in their frozen order.
///
/// # Panics
/// If the inventory is missing, its enumeration inputs changed, or legacy recording is requested.
pub fn replica_requests() -> Vec<Sent> {
    let root = repo();
    let frozen = root.join("rust/test-fixtures/http/replica-requests.json");
    let inputs = enumeration_inputs();
    let manifest = if recording() { panic!("Legacy recording requires the recorded pre-switch revision") } else {
        let bytes = mm_test_support::golden::read_input(&frozen);
        json::parse(std::str::from_utf8(&bytes).unwrap()).unwrap()
    };
    assert_eq!(manifest.get("version").as_f64(), Some(1.0), "unsupported HTTP request inventory");
    assert_eq!(
        json::to_string(manifest.get("inputs")),
        json::to_string(&inputs),
        "HTTP enumeration corpus changed; use the frozen recordings"
    );
    let list = manifest.get("requests").clone();
    let Value::Array(list) = list else { panic!("requests are a list") };
    let text = |value: &Value| value.as_str().unwrap_or_default().to_owned();
    list.iter()
        .map(|request| {
            let headers = match request.get("headers") {
                Value::Array(pairs) => pairs.iter().map(|pair| (text(pair.get("0")), text(pair.get("1")))).collect(),
                _ => Vec::new(),
            };
            Sent {
                method: text(request.get("method")),
                target: text(request.get("target")),
                headers,
                body: text(request.get("body")),
            }
        })
        .collect()
}

// Fingerprint all enumeration source documents and ordered directory entries, without copying file contents.
fn enumeration_inputs() -> Value {
    let root = repo();
    let mut inputs: Vec<Value> = ["replay/index.json", "capture/responses.json", "capture/cards.json", "capture/details.json"]
        .into_iter()
        .map(|path| obj! { "path" => path, "file" => fixture_identity(&root.join(path)) }.into())
        .collect();
    for directory in ["mirror/static", "capture/media", "capture/mediahost"] {
        let mut paths = Vec::new();
        enumerate(&root.join(directory), &root, &mut paths);
        paths.sort();
        let names = json::to_string(&Value::Array(paths.into_iter().map(Value::from).collect()));
        inputs.push(obj! { "directory" => directory, "entries_sha256" => digest(names.as_bytes()) }.into());
    }
    Value::Array(inputs)
}

// Enumerate required files without silently accepting missing directories or unreadable entries.
fn enumerate(directory: &Path, root: &Path, paths: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            enumerate(&entry.path(), root, paths);
        } else if entry.file_type().unwrap().is_file() {
            paths.push(entry.path().strip_prefix(root).unwrap().to_string_lossy().into_owned());
        }
    }
}
