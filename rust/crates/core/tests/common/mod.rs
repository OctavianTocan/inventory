//! Differential testing against the JS backend: call an exported function of a JS module (`tests/oracle.mjs`) and
//! compare its answer with the Rust port's, as JSON text so key order and number formatting count. Other crates' tests
//! include this file with `#[path]`.
#![allow(dead_code)]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use mm_core::{Error, Object, Value, obj};
use mm_js::{base64, json};

/// The repository root: `MYMIND_ROOT`, or the checkout this crate sits in.
pub fn root() -> PathBuf {
    std::env::var_os("MYMIND_ROOT").map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."), PathBuf::from)
}

/// What a JS call threw.
#[derive(Clone, Debug, PartialEq)]
pub struct JsError {
    /// Constructor name: `Error`, `TypeError`, `RangeError`, `SyntaxError`, `URIError`.
    pub name: String,
    pub message: String,
    pub status: Option<u16>,
    pub code: Option<String>,
}

/// A JS call's result.
pub type Answer = Result<Value, JsError>;

/// Values JSON cannot hold, tagged as the oracle tags them.
pub fn tag(value: &Value) -> Value {
    match value {
        Value::Undefined => obj! { "$undefined" => true }.into(),
        Value::BigInt(number) => obj! { "$bigint" => number.to_string() }.into(),
        Value::Bytes(bytes) => obj! { "$bytes" => base64::encode(bytes) }.into(),
        Value::Date(ms) => obj! { "$date" => *ms }.into(),
        Value::Array(items) => Value::Array(items.iter().map(tag).collect()),
        Value::Object(object) => Value::Object(object.iter().map(|(key, item)| (key.into_owned(), tag(item))).collect()),
        other => other.clone(),
    }
}

/// The value a tagged answer stands for.
pub fn untag(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(untag).collect()),
        Value::Object(object) => {
            if object.get("$undefined").is_some() {
                return Value::Undefined;
            }
            if let Some(Value::String(digits)) = object.get("$bigint") {
                return Value::BigInt(digits.parse().expect("bigint digits"));
            }
            if let Some(Value::String(text)) = object.get("$bytes") {
                return Value::Bytes(base64::decode(text));
            }
            if let Some(Value::Number(ms)) = object.get("$date") {
                return Value::Date(*ms);
            }
            Value::Object(object.into_iter().map(|(key, item)| (key, untag(item))).collect::<Object>())
        }
        other => other,
    }
}

/// Call `module`'s exports once per entry of `calls` (function name and arguments) in one Node process.
///
/// # Panics
/// When Node cannot run the oracle.
pub fn calls(module: &str, calls: &[(&str, Vec<Value>)]) -> Vec<Answer> {
    let request = obj! {
        "module" => module,
        "calls" => calls.iter().map(|(name, args)| Value::from(obj! { "fn" => *name, "args" => Value::Array(args.iter().map(tag).collect()) })).collect::<Vec<_>>(),
    };
    let mut child = Command::new("node")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/tests/oracle.mjs"))
        .env("TZ", "UTC")
        .env("MYMIND_ROOT", root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs");
    child.stdin.take().unwrap().write_all(json::to_string(&request.into()).as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "oracle failed for {module}");
    let Value::Array(answers) = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() else { panic!("oracle answered no array") };
    answers
        .into_iter()
        .map(|answer| match answer.get("error") {
            Value::Object(error) => Err(JsError {
                name: error.get("name").and_then(Value::as_str).unwrap_or("Error").to_owned(),
                message: error.get("message").and_then(Value::as_str).unwrap_or("").to_owned(),
                status: error.get("status").and_then(Value::as_f64).map(|status| status as u16),
                code: error.get("code").and_then(Value::as_str).map(str::to_owned),
            }),
            _ => Ok(untag(answer.get("ok").clone())),
        })
        .collect()
}

/// Call one export once.
pub fn call(module: &str, name: &str, args: Vec<Value>) -> Answer {
    calls(module, &[(name, args)]).remove(0)
}

/// JSON text of a value with every JS-only value tagged, for comparisons where key order counts.
pub fn text(value: &Value) -> String {
    json::to_string(&tag(value))
}

/// Whether a Rust error is the one JS threw: the same status and message for status errors, a `TypeError` or
/// `RangeError` for invalid data, and any other error for internal failures.
pub fn same_error(rust: &Error, js: &JsError) -> bool {
    match rust {
        Error::Status { status, message, code } => js.status == Some(*status) && js.message == *message && js.code.as_deref() == *code,
        Error::Invalid(_) => js.status.is_none() && (js.name == "TypeError" || js.name == "RangeError"),
        Error::Internal(_) => js.status.is_none() && js.name != "TypeError" && js.name != "RangeError",
    }
}

/// Assert that the Rust result matches the JS answer, showing `context` on a mismatch.
///
/// # Panics
/// On any difference.
pub fn assert_same(rust: &Result<Value, Error>, js: &Answer, context: &str) {
    match (rust, js) {
        (Ok(rust), Ok(js)) => assert_eq!(text(rust), text(js), "{context}"),
        (Err(rust), Err(js)) => assert!(same_error(rust, js), "{context}: Rust {rust:?}, JS {js:?}"),
        (rust, js) => panic!("{context}: Rust {rust:?}, JS {js:?}"),
    }
}

/// A small deterministic random source for generated cases.
pub struct Random(pub u64);

impl Random {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }

    /// A string of `length` characters drawn from `alphabet`.
    pub fn text(&mut self, alphabet: &[&str], length: usize) -> String {
        (0..length).map(|_| alphabet[self.below(alphabet.len() as u64) as usize]).collect()
    }
}

/// A fixture database named by an environment variable (`rust/.cargo/config.toml` points them at `target/fixtures/`,
/// which `rust/fixtures.py` fills).
///
/// # Panics
/// When the variable is unset or the file is missing.
pub fn fixture(var: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(var).unwrap_or_else(|| panic!("{var} is not set")));
    assert!(path.exists(), "{} is missing; run rust/fixtures.py", path.display());
    path
}

/// The owner library fixture (`MYMIND_TEST_DB`).
pub fn test_db() -> PathBuf {
    fixture("MYMIND_TEST_DB")
}

/// The hosted test library fixture with its recordings (`MYMIND_TEST_HOSTED_DB`).
pub fn hosted_db() -> PathBuf {
    fixture("MYMIND_TEST_HOSTED_DB")
}

/// A copy of `path` in a new directory below the target's temporary directory, for tests that write. Copies that
/// test processes no longer running left behind are removed first.
pub fn temp_copy(path: &std::path::Path) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static SWEEP: std::sync::Once = std::sync::Once::new();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("fixture-copies");
    SWEEP.call_once(|| sweep(&base));
    let directory = base.join(format!("{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::create_dir_all(&directory).unwrap();
    let target = directory.join(path.file_name().unwrap());
    std::fs::copy(path, &target).unwrap();
    target
}

// Remove the copy directories of test processes that have exited (on systems with `/proc`).
fn sweep(base: &std::path::Path) {
    let (Ok(entries), true) = (std::fs::read_dir(base), std::path::Path::new("/proc/self").exists()) else { return };
    for entry in entries.flatten() {
        let pid = entry.file_name().to_str().and_then(|name| name.split('-').next()?.parse::<u32>().ok());
        if pid.is_some_and(|pid| pid != std::process::id() && !std::path::Path::new(&format!("/proc/{pid}")).exists()) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}
