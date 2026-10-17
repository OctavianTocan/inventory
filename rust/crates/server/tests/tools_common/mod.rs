//! Shared parts of the `tool_*` differential tests: a temporary project root for a JS tool and one for the native
//! subcommand, runners for both, and a fake HTTP server on 127.0.0.1.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What a finished process left behind.
#[derive(Debug)]
pub struct Out {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

fn repo() -> PathBuf {
    PathBuf::from(std::env::var_os("MYMIND_ROOT").expect("MYMIND_ROOT is set"))
}

/// A new project root below the target's temporary directory: a copy of the repository's `tools/` and links to the
/// libraries, recordings and `node_modules`. No `.env`, `data/` or `worker/` of the repository gets in.
pub fn tmproot(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tools-{}-{}-{label}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let status = Command::new("cp").arg("-r").arg(repo().join("tools")).arg(root.join("tools")).status().unwrap();
    assert!(status.success());
    for name in ["lib", "mirror", "capture", "replay", "node_modules", "package.json"] {
        let from = repo().join(name);
        if from.exists() {
            std::os::unix::fs::symlink(from, root.join(name)).unwrap();
        }
    }
    root
}

fn collect(mut command: Command, env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    command.env_clear();
    for name in ["PATH", "HOME"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command.envs(env.iter().copied());
    command.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    if let Some(text) = stdin {
        child.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    Out { status: output.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&output.stdout).into_owned(), stderr: String::from_utf8_lossy(&output.stderr).into_owned() }
}

/// Run `node <root>/tools/<tool>.mjs` in `root` with only the given variables (plus PATH and HOME).
pub fn run_js(root: &Path, tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    run_js_flags(root, &[], tool, args, env, stdin)
}

/// `run_js` with extra flags for node itself, such as `--env-file-if-exists=.env`.
pub fn run_js_flags(root: &Path, flags: &[&str], tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    let mut command = Command::new("node");
    command.args(flags).arg(root.join("tools").join(format!("{tool}.mjs"))).args(args).current_dir(root);
    // Node's "SQLite is experimental" notice is the runtime's, not the tool's.
    let env: Vec<(&str, &str)> = env.iter().copied().chain([("NODE_NO_WARNINGS", "1")]).collect();
    collect(command, &env, stdin)
}

/// Run the native subcommand `tool` with `MYMIND_ROOT=<root>` and the same variables.
pub fn run_rust(root: &Path, tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
    command.arg(tool).args(args).current_dir(root);
    let root = root.to_str().unwrap();
    collect(command, &env.iter().copied().chain([("MYMIND_ROOT", root)]).collect::<Vec<_>>(), stdin)
}

/// Two fresh roots, run both sides through `each`, which receives the root, a runner for that side and the side's
/// name, and returns whatever it wants to compare.
pub fn both<T>(label: &str, mut each: impl FnMut(&Path, &dyn Fn(&[&str], &[(&str, &str)], Option<&str>) -> Out) -> T) -> (T, T) {
    let js_root = tmproot(&format!("{label}-js"));
    let rust_root = tmproot(&format!("{label}-rust"));
    let tool = label.to_owned();
    let js = each(&js_root, &|args, env, stdin| run_js(&js_root, &tool, args, env, stdin));
    let rust = each(&rust_root, &|args, env, stdin| run_rust(&rust_root, &tool, args, env, stdin));
    (js, rust)
}

/// One request a fake server received.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Req {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Req {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }
}

/// What a fake server answers.
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn json(status: u16, body: &str) -> Resp {
        Resp { status, headers: vec![("content-type".into(), "application/json".into())], body: body.as_bytes().to_vec() }
    }
}

/// A fake HTTP server on 127.0.0.1 that serves one connection at a time on its own thread.
pub struct Fake {
    pub addr: String,
    log: Arc<Mutex<Vec<Req>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

type Handler = Box<dyn FnMut(&Req) -> Resp>;

impl Fake {
    /// Start a server whose handler `make` builds inside the server thread (so it may hold non-`Send` state).
    pub fn start<F: FnOnce() -> Handler + Send + 'static>(make: F) -> Fake {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let log = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (thread_log, thread_stop) = (log.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            let mut handler = make();
            for stream in listener.incoming() {
                if thread_stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(20)));
                if let Some(request) = read_request(&mut stream) {
                    thread_log.lock().unwrap().push(request.clone());
                    let response = handler(&request);
                    let _ = write_response(&mut stream, &request, &response);
                }
            }
        });
        Fake { addr, log, stop, thread: Some(thread) }
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn log(&self) -> Vec<Req> {
        self.log.lock().unwrap().clone()
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(&self.addr);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Option<Req> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        if let Some(at) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break at;
        }
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..read]);
    };
    let head = String::from_utf8_lossy(&buffer[..head_end]).into_owned();
    let mut rest = buffer[head_end + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers: Vec<(String, String)> = lines.filter_map(|line| line.split_once(':').map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))).collect();
    let find = |name: &str| headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
    let body = if find("transfer-encoding").is_some_and(|value| value.contains("chunked")) {
        let mut body = Vec::new();
        loop {
            while !rest.windows(2).any(|window| window == b"\r\n") {
                let read = stream.read(&mut chunk).ok()?;
                if read == 0 {
                    return None;
                }
                rest.extend_from_slice(&chunk[..read]);
            }
            let line_end = rest.windows(2).position(|window| window == b"\r\n")?;
            let size = usize::from_str_radix(String::from_utf8_lossy(&rest[..line_end]).split(';').next()?.trim(), 16).ok()?;
            rest.drain(..line_end + 2);
            while rest.len() < size + 2 {
                let read = stream.read(&mut chunk).ok()?;
                if read == 0 {
                    return None;
                }
                rest.extend_from_slice(&chunk[..read]);
            }
            body.extend_from_slice(&rest[..size]);
            rest.drain(..size + 2);
            if size == 0 {
                break body;
            }
        }
    } else {
        let length: usize = find("content-length").and_then(|value| value.parse().ok()).unwrap_or(0);
        while rest.len() < length {
            let read = stream.read(&mut chunk).ok()?;
            if read == 0 {
                return None;
            }
            rest.extend_from_slice(&chunk[..read]);
        }
        rest.truncate(length);
        rest
    };
    let (path, query) = target.split_once('?').map_or((target.clone(), String::new()), |(path, query)| (path.to_owned(), query.to_owned()));
    Some(Req { method, path, query, headers, body })
}

fn write_response(stream: &mut TcpStream, request: &Req, response: &Resp) -> std::io::Result<()> {
    let mut head = format!("HTTP/1.1 {} X\r\nconnection: close\r\ncontent-length: {}\r\n", response.status, response.body.len());
    for (name, value) in &response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    if request.method != "HEAD" {
        stream.write_all(&response.body)?;
    }
    stream.flush()
}

/// Run a driver script with node in `root`, with `MYMIND_ROOT=<root>` and the given variables.
pub fn run_node(root: &Path, script: &str, args: &[&str], env: &[(&str, &str)]) -> Out {
    let mut command = Command::new("node");
    command.arg(script).args(args).current_dir(root);
    let root = root.to_str().unwrap();
    collect(command, &env.iter().copied().chain([("NODE_NO_WARNINGS", "1"), ("MYMIND_ROOT", root)]).collect::<Vec<_>>(), None)
}

fn seed_of(text: &str) -> usize {
    text.bytes().fold(7usize, |sum, byte| (sum * 31 + byte as usize) % 9973)
}

fn embedding_json(seed: usize) -> String {
    let values: Vec<String> = (0..768).map(|i| format!("{}", ((i * 7 + seed * 3) % 13) as f64 / 5.0 - 1.1)).collect();
    format!("[{}]", values.join(","))
}

fn text_of(value: &mm_js::Value) -> String {
    value.get("content").get("parts").get("0").get("text").as_str().unwrap_or("").to_owned()
}

/// A fake of the hosted AI APIs: Gemini embeddings (`:embedContent`, `:batchEmbedContents`), the chat endpoint and
/// TypeSafe Jev, with answers that depend only on the request text. `usage` adds `usageMetadata` to batch answers.
pub fn ai_fake(usage: bool) -> Fake {
    Fake::start(move || {
        Box::new(move |request: &Req| {
            let body = mm_js::json::parse(&String::from_utf8_lossy(&request.body)).unwrap_or(mm_js::Value::Null);
            if request.path.ends_with(":batchEmbedContents") {
                let requests = body.get("requests").as_array().cloned().unwrap_or_default();
                let items: Vec<String> = requests.iter().map(|item| format!("{{\"values\":{}}}", embedding_json(seed_of(&text_of(item))))).collect();
                let meta = if usage { format!(",\"usageMetadata\":{{\"totalTokenCount\":{}}}", requests.len() * 11) } else { String::new() };
                return Resp::json(200, &format!("{{\"embeddings\":[{}]{meta}}}", items.join(",")));
            }
            if request.path.ends_with(":embedContent") {
                return Resp::json(200, &format!("{{\"embedding\":{{\"values\":{}}}}}", embedding_json(seed_of(&text_of(&body)))));
            }
            if request.path.ends_with("/chat/completions") {
                let first = body.get("messages").get("1").get("content").get("0").get("text").as_str().unwrap_or("").lines().next().unwrap_or("").to_owned();
                let seed = seed_of(&first);
                let names = ["Design", "Typography", "Photography", "Recipes", "Travel Notes", "Machine Learning"];
                let tags: Vec<String> = (0..4).map(|i| format!("\"{}\"", names[(seed + i * 2) % names.len()])).collect();
                let content = format!("{{\"title\":\"Fake title number {seed} for a card\",\"summary\":\"A summary of {} that has several words.\",\"tags\":[{}]}}", first.replace('"', ""), tags.join(","));
                let envelope = mm_js::json::to_string(&mm_js::obj! { "choices" => mm_js::Value::Array(vec![mm_js::obj! { "message" => mm_js::obj! { "content" => content } }.into()]) }.into());
                return Resp::json(200, &envelope);
            }
            if request.path.ends_with("/v1/systemone") {
                let keys: Vec<String> = body.get("questions").as_object().map(|object| object.keys().map(|key| key.into_owned()).collect()).unwrap_or_default();
                let answers: Vec<String> = keys.iter().enumerate().map(|(i, key)| format!("\"{key}\":{{\"noul\":{}}}", [0.9, 0.75, 0.3, 0.65, 0.85][i % 5])).collect();
                return Resp::json(200, &format!("{{\"answers\":{{{}}}}}", answers.join(",")));
            }
            Resp::json(404, "{\"error\":{\"message\":\"unknown route\"}}")
        })
    })
}

/// Each request as one comparable line: method, path, the headers that matter, and the body with random Jev image
/// names replaced.
pub fn request_lines(fake: &Fake) -> Vec<String> {
    let uuid = regex::Regex::new(r"mymind-image-[0-9a-f-]{36}").unwrap();
    fake.log()
        .iter()
        .map(|request| {
            let headers: Vec<String> = ["x-goog-api-key", "authorization", "content-type", "typesafe-danger-zone-images"].iter().map(|name| format!("{name}={}", request.header(name).unwrap_or("-"))).collect();
            format!("{} {}?{} {} {}", request.method, request.path, request.query, headers.join(" "), uuid.replace_all(&String::from_utf8_lossy(&request.body), "mymind-image-<id>"))
        })
        .collect()
}

/// Every table of a database as `name rows digest`, with each stored blob, integer and real counted (the order of
/// the rows does not matter).
pub fn db_state(path: &Path) -> Vec<String> {
    use mm_core::sql::Sql;
    let db = mm_sqlite::Sqlite::open(path, true).unwrap();
    let tables = db.query("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' ORDER BY name", &[]).unwrap();
    tables
        .iter()
        .map(|row| {
            let name = row.text("name").unwrap().to_owned();
            let columns: Vec<String> = db.query(&format!("SELECT name FROM pragma_table_info('{name}')"), &[]).unwrap().iter().map(|column| column.text("name").unwrap().to_owned()).collect();
            let digest = mm_core::library_rows::table_digest(&db, &name, &columns).unwrap();
            format!("{name} {} {}", digest.rows, digest.sha256)
        })
        .collect()
}

/// Assert two request logs are equal, reporting only where the first difference is (bodies can be large).
pub fn assert_requests(name: &str, rust: &[String], js: &[String]) {
    if rust == js {
        return;
    }
    let clip = |line: &str, at: usize| line.chars().skip(at.saturating_sub(60)).take(200).collect::<String>();
    for (index, (a, b)) in rust.iter().zip(js).enumerate() {
        if a != b {
            let at = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
            panic!("{name}: request {index} differs at character {at}\n rust: ...{}\n   js: ...{}", clip(a, at), clip(b, at));
        }
    }
    panic!("{name}: rust sent {} requests, js sent {}", rust.len(), js.len());
}

/// The URL of a loopback port nothing listens on (not port 9, which fetch refuses as a blocked port).
pub fn closed_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://{}", listener.local_addr().unwrap())
}
