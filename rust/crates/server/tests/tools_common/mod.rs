//! Frozen operational-tool expectations, isolated scratch roots and loopback fake providers.
#![allow(dead_code)]

use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sha2::{Digest, Sha256};

struct Comparison {
    request: Vec<u8>,
    actual: Vec<u8>,
    oracle: Option<Vec<u8>>,
}

struct Session {
    target: String,
    namespace: String,
    identity: String,
    cases: Vec<Comparison>,
    inventory: mm_test_support::golden::TestGuard,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

/// Completes one ordered scenario only after its executable assertions have succeeded.
pub struct Golden;

/// Start a scenario whose identity includes its complete test recipe and immutable database inputs.
///
/// # Arguments
/// * `source` - The integration test source path from `file!()`.
/// * `recipe` - The complete source text defining the ordered operations and synthetic fixtures.
///
/// # Returns
/// A guard that verifies the complete ordered case set and finishes recording on successful scope exit.
pub fn golden(source: &str, recipe: &str) -> Golden {
    let target = Path::new(source).file_stem().unwrap().to_string_lossy().into_owned();
    let namespace = std::thread::current().name().expect("named integration test").to_owned();
    let identity = format!("recipe={}\n{}", mm_test_support::golden::source_identity(env!("CARGO_MANIFEST_DIR"), &target, "recipe", &format!("{:x}", Sha256::digest(recipe.as_bytes()))), fixture_identity());
    SESSION.with(|slot| {
        assert!(slot.borrow().is_none(), "nested golden scenario");
        let inventory = mm_test_support::golden::TestGuard::new(env!("CARGO_MANIFEST_DIR"), &target);
        *slot.borrow_mut() = Some(Session { target, namespace, identity, cases: Vec::new(), inventory });
    });
    Golden
}

// Hash the immutable input databases once per integration-test process.
fn fixture_identity() -> &'static str {
    static IDENTITY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    IDENTITY.get_or_init(|| {
        ["MYMIND_TEST_DB", "MYMIND_TEST_HOSTED_DB"].iter().map(|key| {
            let file = std::env::var_os(key).expect("required immutable database fixture path");
            let file = PathBuf::from(file);
            assert!(!matches!(file.file_name().and_then(|name| name.to_str()), Some(".env" | ".dev.vars")), "secret filenames cannot be test fixtures");
            let bytes = std::fs::read(file).expect("required immutable database fixture");
            format!("{key}={}:{:x}", bytes.len(), Sha256::digest(bytes))
        }).collect::<Vec<_>>().join("\n")
    })
}

impl Drop for Golden {
    fn drop(&mut self) {
        let session = SESSION.with(|slot| slot.borrow_mut().take()).expect("active golden scenario");
        if std::thread::panicking() {
            return;
        }
        assert!(!session.cases.is_empty(), "golden scenario must compare at least one case");
        let requests: Vec<Vec<u8>> = session.cases.iter().map(|case| case.request.clone()).collect();
        let mut batch = mm_test_support::golden::Batch::new(env!("CARGO_MANIFEST_DIR"), &session.target, &session.namespace, &requests);
        for (index, case) in session.cases.iter().enumerate() {
            batch.check(index, &case.actual, case.oracle.as_deref());
        }
        batch.finish();
        session.inventory.finish();
    }
}

/// Reject legacy recording after the switch.
///
/// # Returns
/// False; panics when recording is requested in this checkout.
pub fn recording() -> bool {
    mm_test_support::golden::recording()
}

/// Represent a potentially large output by its exact length and digest above 1024 bytes.
///
/// # Arguments
/// * `bytes` - The already normalized, credential-free output.
///
/// # Returns
/// A printable exact short output or a SHA-256 and byte length for larger outputs.
pub fn summary(bytes: &[u8]) -> String {
    if bytes.len() > 1024 {
        format!("bytes={} sha256={:x}", bytes.len(), Sha256::digest(bytes))
    } else if let Ok(text) = std::str::from_utf8(bytes) {
        format!("bytes={} utf8={text:?}", bytes.len())
    } else {
        format!("bytes={} hex={}", bytes.len(), bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
    }
}

/// Queue canonical Rust and optional live oracle bytes for comparison at successful scenario exit.
///
/// # Arguments
/// * `request` - Stable normalized case inputs, excluding ephemeral ports, paths and credentials.
/// * `actual` - The complete canonical Rust output or side-effect manifest.
/// * `oracle` - The JavaScript expectation, supplied only in recording mode.
pub fn check(request: &str, actual: &[u8], oracle: Option<&[u8]>) {
    assert_eq!(oracle.is_some(), recording(), "oracle only exists during explicit recording");
    SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let session = slot.as_mut().expect("start a golden scenario before comparing");
        let request = format!("{}\nstep={}\n{request}", session.identity, session.cases.len()).into_bytes();
        session.cases.push(Comparison { request, actual: actual.to_vec(), oracle: oracle.map(<[u8]>::to_vec) });
    });
}

/// What a finished process left behind.
#[derive(Clone, Debug)]
pub struct Out {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

fn repo() -> PathBuf {
    std::env::var_os("MYMIND_ROOT").map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().expect("repository root exists"), PathBuf::from)
}

/// Create a native scratch root containing only immutable recording links.
///
/// `label` identifies the scenario. Returns a fresh target-local scratch directory.
pub fn tmproot(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tools-{}-{}-{label}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    prepare_root(&root);
    root
}

/// Link immutable recording inputs without adding backend or environment files.
///
/// # Arguments
/// * `root` - An existing empty temporary directory, never the repository root.
pub fn prepare_root(root: &Path) {
    for name in ["mirror", "capture", "replay"] {
        let from = repo().join(name);
        assert!(from.is_dir(), "required immutable recording directory {name} is missing");
        std::os::unix::fs::symlink(from, root.join(name)).unwrap();
    }
}

fn collect(mut command: Command, env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    command.env_clear();
    command.env("TMPDIR", env!("CARGO_TARGET_TMPDIR"));
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
    Out { status: output.status.code().unwrap_or(-1), stdout: legacy_help(&String::from_utf8_lossy(&output.stdout)), stderr: legacy_help(&String::from_utf8_lossy(&output.stderr)) }
}

/// Reject the retired operational oracle adapter.
///
/// `root`, `tool`, `args`, `env` and `stdin` retain the historical comparison signature.
/// Never returns; legacy recording requires the recorded pre-switch revision.
pub fn run_js(root: &Path, tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    run_js_flags(root, &[], tool, args, env, stdin)
}

/// Reject the retired operational oracle adapter with historical runtime flags.
///
/// `root`, `flags`, `tool`, `args`, `env` and `stdin` are unused historical inputs.
/// Never returns; legacy recording requires the recorded pre-switch revision.
pub fn run_js_flags(root: &Path, flags: &[&str], tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    let _ = (root, flags, tool, args, env, stdin);
    panic!("Legacy recording requires the recorded pre-switch revision")
}

/// Run the native subcommand `tool` with `MYMIND_ROOT=<root>` and the same variables.
pub fn run_rust(root: &Path, tool: &str, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Out {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
    command.arg(tool).args(args).current_dir(root);
    let root = root.to_str().unwrap();
    collect(command, &env.iter().copied().chain([("MYMIND_ROOT", root), ("MYMIND_ENV_FILE", "fixture-vars"), ("STORE_LOAD_ENV", "0")]).collect::<Vec<_>>(), stdin)
}

/// Two fresh roots, run both sides through `each`, which receives the root, a runner for that side and the side's
/// name, and returns whatever it wants to compare.
pub fn both<T: Clone + std::fmt::Debug>(label: &str, mut each: impl FnMut(&Path, &dyn Fn(&[&str], &[(&str, &str)], Option<&str>) -> Out) -> T) -> (T, T) {
    let rust_root = tmproot(&format!("{label}-rust"));
    let tool = label.to_owned();
    let js = recording().then(|| {
        let js_root = tmproot(&format!("{label}-js"));
        each(&js_root, &|args, env, stdin| run_js(&js_root, &tool, args, env, stdin))
    });
    let rust = each(&rust_root, &|args, env, stdin| run_rust(&rust_root, &tool, args, env, stdin));
    let actual = format!("{rust:?}");
    let oracle = js.as_ref().map(|out| format!("{out:?}"));
    check(label, actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    (rust.clone(), rust)
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
        mm_test_support::golden::register_loopback_port(listener.local_addr().unwrap().port());
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

/// Reject the retired driver adapter without starting another process.
///
/// `root`, `script`, `args` and `env` are unused historical inputs. Never returns;
/// legacy recording requires the recorded pre-switch revision.
pub fn run_node(root: &Path, script: &str, args: &[&str], env: &[(&str, &str)]) -> Out {
    let _ = (root, script, args, env);
    panic!("Legacy recording requires the recorded pre-switch revision")
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
            format!("{name} columns={columns:?} rows={} sha256={}", digest.rows, digest.sha256)
        })
        .collect()
}

/// Summarize typed rows without storing any row payload or card HTML in a golden.
///
/// # Arguments
/// * `tables` - Ordered tables and their original typed, ordered row encodings.
///
/// # Returns
/// Ordered table identities, row counts, byte lengths and SHA-256 digests.
pub fn row_manifest(tables: &[(String, Vec<String>)]) -> String {
    tables.iter().map(|(name, rows)| {
        let bytes = format!("{rows:?}");
        format!("{name:?} rows={} bytes={} sha256={:x}", rows.len(), bytes.len(), Sha256::digest(bytes.as_bytes()))
    }).collect::<Vec<_>>().join("\n")
}

/// Preserve the ordered column identities that give each typed table row its meaning.
///
/// # Arguments
/// * `path` - A read-only scratch database produced by the tested command.
///
/// # Returns
/// Table names and their ordered column names, excluding FTS shadow tables.
pub fn table_columns(path: &Path) -> Vec<String> {
    use mm_core::sql::Sql;
    let db = mm_sqlite::Sqlite::open(path, true).unwrap();
    db.query("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%' ORDER BY name", &[]).unwrap().iter().map(|row| {
        let name = row.text("name").unwrap();
        let quoted = name.replace('\'', "''");
        let columns: Vec<String> = db.query(&format!("SELECT name FROM pragma_table_info('{quoted}') ORDER BY cid"), &[]).unwrap().iter().map(|row| row.text("name").unwrap().to_owned()).collect();
        format!("{name} {columns:?}")
    }).collect()
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
    mm_test_support::golden::register_loopback_port(listener.local_addr().unwrap().port());
    format!("http://{}", listener.local_addr().unwrap())
}

/// Normalize the intentional executable prefix change in frozen CLI comparisons.
///
/// `text` is captured CLI output. Returns the same text with only usage prefixes
/// mapped to the historical spelling stored in the pre-switch golden answers.
pub fn legacy_help(text: &str) -> String {
    let mut text = text.to_owned();
    for tool in ["seed-db", "import-hosted", "auth-user", "upload-hosted", "embed-library", "try-ai"] {
        for prefix in ["Usage: ", "usage: "] {
            text = text.replace(&format!("{prefix}mymind {tool}"), &format!("{prefix}node tools/{tool}.mjs"));
        }
    }
    text
}
