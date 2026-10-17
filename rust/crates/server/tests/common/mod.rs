//! What the differential tests of the native servers share: a server process, a keep-alive HTTP client and the
//! answers both compare.
// Each test binary uses only part of it.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use mm_js::{Value, json};

/// The repository root: `MYMIND_ROOT`, or the checkout this crate sits in.
pub fn repo() -> PathBuf {
    std::env::var_os("MYMIND_ROOT").map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."), PathBuf::from).canonicalize().unwrap()
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
        let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
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
        Self { child, port, log }
    }

    /// The lines the server printed so far that start with one of `prefixes`.
    pub fn logged(&self, prefixes: &[&str]) -> Vec<String> {
        self.log.lock().unwrap().iter().filter(|line| prefixes.iter().any(|prefix| line.starts_with(prefix))).cloned().collect()
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
        Self { method: method.to_owned(), target: target.to_owned(), headers, body: String::new() }
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
                    return Answer { status: "connection refused".to_owned(), headers: Vec::new(), body: Vec::new() };
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
        Answer { status: "no answer".to_owned(), headers: Vec::new(), body: Vec::new() }
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
        Answer { status: answer.status, headers, body: replace(&answer.body, port.as_bytes(), b":PORT") }
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
    let header = |wanted: &str| headers.iter().find(|(name, _)| name.eq_ignore_ascii_case(wanted)).map(|(_, value)| value.to_ascii_lowercase());
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

/// The requests of `tests/drivers/replica-requests.mjs`: every replayed and recorded response and each route's edge
/// cases.
pub fn replica_requests() -> Vec<Sent> {
    let root = repo();
    let output = Command::new("node").arg(root.join("rust/crates/server/tests/drivers/replica-requests.mjs")).env("MYMIND_ROOT", &root).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let list = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let Value::Array(list) = list else { panic!("requests are a list") };
    let text = |value: &Value| value.as_str().unwrap_or_default().to_owned();
    list.iter()
        .map(|request| {
            let headers = match request.get("headers") {
                Value::Array(pairs) => pairs.iter().map(|pair| (text(pair.get("0")), text(pair.get("1")))).collect(),
                _ => Vec::new(),
            };
            Sent { method: text(request.get("method")), target: text(request.get("target")), headers, body: text(request.get("body")) }
        })
        .collect()
}
