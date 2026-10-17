//! Compare Rust with frozen pre-switch golden expectations, preserving JSON order and error classes.
//! Other crates include this file with `#[path]`; Node runs only in explicit recording mode.
#![allow(dead_code)]

use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use mm_core::{Error, Object, Value, obj};
use mm_js::{base64, json};
use mm_test_support::golden::Batch;

/// Resolve the repository root without reading credential files.
///
/// # Returns
/// `MYMIND_ROOT` when set, otherwise the checkout containing this crate.
pub fn root() -> PathBuf {
    std::env::var_os("MYMIND_ROOT").map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."), PathBuf::from)
}

/// Finish a test's full batch inventory after its comparisons and assertions succeed.
pub struct GoldenGuard(Option<mm_test_support::golden::TestGuard>);

impl Drop for GoldenGuard {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.0.take().expect("active golden inventory").finish();
        }
    }
}

/// Open an inventory guard before any oracle comparisons in the current test.
///
/// # Returns
/// A guard that rejects missing or extra batches when the test exits successfully.
///
/// # Panics
/// On a missing replay inventory or an incomplete test at normal scope exit.
pub fn test_guard() -> GoldenGuard {
    GoldenGuard(Some(mm_test_support::golden::TestGuard::new(env!("CARGO_MANIFEST_DIR"), &target())))
}

/// What a JS call threw.
#[derive(Clone, Debug, PartialEq)]
pub struct JsError {
    /// Constructor name: `Error`, `TypeError`, `RangeError`, `SyntaxError`, `URIError`.
    pub name: String,
    /// Error message used only for status-error equivalence.
    pub message: String,
    /// Explicit HTTP status, when the original error carried one.
    pub status: Option<u16>,
    /// Application error code, when the original error carried one.
    pub code: Option<String>,
}

/// A deferred golden comparison, with the legacy answer available only while recording.
pub struct Answer {
    batch: Rc<RefCell<Pending>>,
    index: usize,
    oracle: Option<Result<Value, JsError>>,
}

struct Pending {
    batch: Option<Batch>,
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.batch.take().expect("unfinished golden batch").finish();
        }
    }
}

impl std::fmt::Debug for Answer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Answer")
            .field("index", &self.index)
            .field("recording", &self.oracle.is_some())
            .finish()
    }
}

impl Answer {
    /// Return the legacy answer only during explicit recording.
    ///
    /// # Returns
    /// The tagged-result value or legacy error in recording mode; `None` in replay.
    pub fn oracle(&self) -> Option<&Result<Value, JsError>> {
        self.oracle.as_ref()
    }

    /// Check canonical bytes against the frozen digest or recording-only oracle bytes.
    ///
    /// # Arguments
    /// * `actual` - Complete canonical Rust observation.
    /// * `oracle` - Matching legacy observation during recording; absent during replay.
    ///
    /// # Panics
    /// On a changed observation, missing oracle, or repeated case check.
    pub fn check(&self, actual: &[u8], oracle: Option<&[u8]>) {
        self.batch
            .borrow_mut()
            .batch
            .as_mut()
            .expect("active golden batch")
            .check(self.index, actual, oracle);
    }
}

/// Report whether the explicit legacy recording flag is enabled.
///
/// # Returns
/// True only when `MYMIND_RECORD_GOLDEN` is exactly `1`.
pub fn recording() -> bool {
    mm_test_support::golden::recording()
}

// Use a checkout-relative driver identity; Batch supplies test names and occurrence indexes.
fn batch_namespace(module: &str) -> String {
    let path = PathBuf::from(module);
    let root = root().canonicalize().expect("repository root exists");
    path.strip_prefix(&root)
        .map_or_else(|_| module.to_owned(), |relative| relative.to_string_lossy().replace('\\', "/"))
}

/// Identify this integration-test target.
///
/// # Returns
/// The executable name excluding Cargo's fingerprint suffix.
pub fn target() -> String {
    let executable = std::env::current_exe().expect("test executable");
    let name = executable.file_stem().unwrap().to_string_lossy();
    name.rsplit_once('-').map_or_else(|| name.to_string(), |(target, _)| target.to_owned())
}

/// Tag values JSON cannot hold using the legacy oracle representation.
///
/// # Arguments
/// * `value` - Value whose bytes and property order must be preserved.
///
/// # Returns
/// A recursively tagged JSON-compatible value.
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

/// Decode a tagged legacy answer.
///
/// # Arguments
/// * `value` - A JSON-compatible value using the oracle tags.
///
/// # Returns
/// The corresponding value, including bytes, BigInts, dates and undefined.
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

/// Prepare one comparison per export call, running Node only when explicitly recording.
///
/// # Arguments
/// * `module` - Checkout-relative module path or absolute legacy driver path.
/// * `calls` - Complete ordered function-name and argument pairs.
///
/// # Returns
/// One deferred comparison per input, in exactly the same order.
///
/// # Panics
/// On a missing or incompatible golden, or an invalid legacy oracle result.
pub fn calls(module: &str, calls: &[(&str, Vec<Value>)]) -> Vec<Answer> {
    let requests: Vec<Vec<u8>> = calls
        .iter()
        .map(|(name, args)| {
            text(
                &obj! {
                    "module" => batch_namespace(module),
                    "fn" => *name,
                    "args" => Value::Array(args.iter().map(tag).collect()),
                }
                .into(),
            )
            .into_bytes()
        })
        .collect();
    let batch = Batch::new(env!("CARGO_MANIFEST_DIR"), &target(), &batch_namespace(module), &requests);
    let oracle = if batch.recording() { Some(legacy_calls(module, calls)) } else { None };
    if let Some(answers) = &oracle {
        assert_eq!(answers.len(), calls.len(), "oracle must answer every requested case");
    }
    let batch = Rc::new(RefCell::new(Pending { batch: Some(batch) }));
    (0..calls.len())
        .map(|index| Answer {
            batch: batch.clone(),
            index,
            oracle: oracle.as_ref().map(|answers| answers[index].clone()),
        })
        .collect()
}

// Invoke the pre-switch oracle in an environment containing no inherited credentials.
fn legacy_calls(module: &str, calls: &[(&str, Vec<Value>)]) -> Vec<Result<Value, JsError>> {
    assert!(recording(), "legacy oracle requires MYMIND_RECORD_GOLDEN=1");
    let request = obj! {
        "module" => module,
        "calls" => calls.iter().map(|(name, args)| Value::from(obj! { "fn" => *name, "args" => Value::Array(args.iter().map(tag).collect()) })).collect::<Vec<_>>(),
    };
    let mut child = Command::new("node")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/tests/oracle.mjs"))
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
        .env("TZ", "UTC")
        .env("MYMIND_ROOT", root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs in recording mode");
    let mut stdin = child.stdin.take().unwrap();
    let payload = json::to_string(&request.into());
    let writer = std::thread::spawn(move || stdin.write_all(payload.as_bytes()).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success(), "oracle failed for {module}");
    let Value::Array(answers) = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() else {
        panic!("oracle answered no array")
    };
    answers.into_iter().map(decode_answer).collect()
}

// Decode one tagged legacy success or error without changing property order.
fn decode_answer(answer: Value) -> Result<Value, JsError> {
    match answer.get("error") {
        Value::Object(error) => Err(JsError {
            name: error.get("name").and_then(Value::as_str).unwrap_or("Error").to_owned(),
            message: error.get("message").and_then(Value::as_str).unwrap_or("").to_owned(),
            status: error.get("status").and_then(Value::as_f64).map(|status| status as u16),
            code: error.get("code").and_then(Value::as_str).map(str::to_owned),
        }),
        _ => Ok(untag(answer.get("ok").clone())),
    }
}

/// Retrieve a small synthetic scrypt factory value used to rebuild compatibility rows.
///
/// # Arguments
/// * `module` - The synthetic auth-store driver path.
/// * `name` - Must be `hashes`, the driver's Node built-in scrypt adapter.
/// * `args` - Fixed synthetic password and salt pairs; never account data.
///
/// # Returns
/// The frozen Node scrypt hashes and a deferred expectation. Validate the rebuilt rows,
/// then check the hashes with `assert_value` before dropping the expectation.
///
/// # Panics
/// On another factory, a missing inline expectation, or an incompatible golden.
pub fn synthetic_call(module: &str, name: &str, args: Vec<Value>) -> (Value, Answer) {
    assert!(
        module.ends_with("/tests/drivers/auth_store.mjs") && name == "hashes",
        "inline fixtures are restricted to synthetic scrypt outputs"
    );
    let requests = vec![text(&obj! { "module" => batch_namespace(module), "fn" => name, "args" => args.clone() }.into()).into_bytes()];
    let mut batch = Batch::new(env!("CARGO_MANIFEST_DIR"), &target(), &batch_namespace(module), &requests);
    batch.allow_inline_synthetic();
    let value = if batch.recording() {
        legacy_calls(module, &[(name, args)]).remove(0).expect("synthetic factory succeeds")
    } else {
        let bytes = batch.inline_answer(0).expect("synthetic factory inline expectation");
        decode_answer(json::parse(std::str::from_utf8(bytes).expect("synthetic UTF-8")).expect("synthetic JSON")).expect("synthetic success")
    };
    let oracle = batch.recording().then(|| Ok(value.clone()));
    let answer = Answer { batch: Rc::new(RefCell::new(Pending { batch: Some(batch) })), index: 0, oracle };
    (value, answer)
}

/// Prepare one deferred export comparison.
///
/// # Arguments
/// * `module` - Checkout-relative module path or absolute legacy driver path.
/// * `name` - Export name.
/// * `args` - Complete arguments, including stateful scenario inputs.
///
/// # Returns
/// A comparison checked later with the complete Rust observation.
pub fn call(module: &str, name: &str, args: Vec<Value>) -> Answer {
    calls(module, &[(name, args)]).remove(0)
}

/// Serialize a comparison value with every JS-only value tagged.
///
/// # Arguments
/// * `value` - Value to serialize without reordering properties.
///
/// # Returns
/// Canonical comparison text preserving JavaScript number formatting.
pub fn text(value: &Value) -> String {
    json::to_string(&tag(value))
}

/// Whether a Rust error is the one JS threw: the same status and message for status errors, a `TypeError` or
/// `RangeError` for invalid data, and any other error for internal failures.
///
/// # Arguments
/// * `rust` - Error returned by the Rust implementation.
/// * `js` - Error thrown by the legacy oracle.
///
/// # Returns
/// Whether both errors belong to the existing differential equivalence class.
pub fn same_error(rust: &Error, js: &JsError) -> bool {
    match rust {
        Error::Status { status, message, code } => js.status == Some(*status) && js.message == *message && js.code.as_deref() == *code,
        Error::Invalid(_) => js.status.is_none() && (js.name == "TypeError" || js.name == "RangeError"),
        Error::Internal(_) => js.status.is_none() && js.name != "TypeError" && js.name != "RangeError",
    }
}

/// Assert that the Rust result matches its frozen expectation.
///
/// # Arguments
/// * `rust` - Complete Rust success value or error.
/// * `js` - Deferred expectation for the same request.
/// * `_context` - Caller label; failures identify the target, test and frozen case.
///
/// # Panics
/// On any difference.
pub fn assert_same(rust: &Result<Value, Error>, js: &Answer, _context: &str) {
    let expected = js.oracle().map(|answer| match answer {
        Ok(value) => success(value),
        Err(error) => js_error_class(error),
    });
    let actual = match rust {
        Ok(value) => success(value),
        Err(error) => error_class(error),
    };
    js.check(actual.as_bytes(), expected.as_deref().map(str::as_bytes));
}

/// Compare an entire successful value, preserving object order and tagged values.
///
/// # Arguments
/// * `rust` - Complete Rust success value.
/// * `js` - Deferred expectation for the same request.
/// * `context` - Caller label forwarded to the complete comparison.
///
/// # Panics
/// On a different value, an oracle error, or a missing expectation.
pub fn assert_value(rust: &Value, js: &Answer, context: &str) {
    assert_value_map(rust, js, context, Clone::clone);
}

/// Normalize both sides before checking the complete value.
///
/// # Arguments
/// * `rust` - Complete Rust success value.
/// * `js` - Deferred expectation for the same request.
/// * `_context` - Caller label; failures identify the target, test and frozen case.
/// * `map` - The original comparison's deterministic normalization, applied to both sides.
///
/// # Panics
/// On a different normalized value, an oracle error, or a missing expectation.
pub fn assert_value_map(rust: &Value, js: &Answer, _context: &str, map: impl Fn(&Value) -> Value) {
    let actual = success(&map(rust));
    let expected = js.oracle().map(|answer| match answer {
        Ok(value) => success(&map(value)),
        Err(error) => js_error_class(error),
    });
    js.check(actual.as_bytes(), expected.as_deref().map(str::as_bytes));
}

/// Preserve a comparison whose original contract accepts any error from either implementation.
///
/// # Arguments
/// * `rust` - Complete Rust success value or error.
/// * `js` - Deferred expectation for the same request.
/// * `_context` - Caller label; failures identify the target, test and frozen case.
///
/// # Panics
/// When successes differ or only one side fails.
pub fn assert_throws(rust: &Result<Value, Error>, js: &Answer, _context: &str) {
    let actual = match rust {
        Ok(value) => success(value),
        Err(_) => "{\"throws\":true}".to_owned(),
    };
    let expected = js.oracle().map(|answer| match answer {
        Ok(value) => success(value),
        Err(_) => "{\"throws\":true}".to_owned(),
    });
    js.check(actual.as_bytes(), expected.as_deref().map(str::as_bytes));
}

// Encode a successful comparison with an unambiguous result envelope.
fn success(value: &Value) -> String {
    text(&obj! { "ok" => value.clone() }.into())
}

/// Encode a Rust error using the existing `same_error` equivalence classes.
///
/// # Arguments
/// * `error` - Rust error to classify.
///
/// # Returns
/// Exact status, message and code for status errors; otherwise the accepted error class.
pub fn error_class(error: &Error) -> String {
    match error {
        Error::Status { status, message, code } => {
            text(&obj! { "status" => *status, "message" => message.as_str(), "code" => code.map_or(Value::Null, Value::from) }.into())
        }
        Error::Invalid(_) => "{\"errorClass\":\"invalid\"}".to_owned(),
        Error::Internal(_) => "{\"errorClass\":\"internal\"}".to_owned(),
    }
}

/// Encode a legacy error using the same status and class semantics as Rust errors.
///
/// # Arguments
/// * `error` - Legacy oracle error to classify.
///
/// # Returns
/// The same canonical text that a matching Rust error produces.
pub fn js_error_class(error: &JsError) -> String {
    if let Some(status) = error.status {
        text(&obj! { "status" => status, "message" => error.message.as_str(), "code" => error.code.as_deref().map_or(Value::Null, Value::from) }.into())
    } else if matches!(error.name.as_str(), "TypeError" | "RangeError") {
        "{\"errorClass\":\"invalid\"}".to_owned()
    } else {
        "{\"errorClass\":\"internal\"}".to_owned()
    }
}

/// A small deterministic random source for generated cases.
pub struct Random(pub u64);

impl Random {
    /// Advance the fixed xorshift sequence.
    ///
    /// # Returns
    /// The next deterministic 64-bit word.
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Draw one bounded deterministic integer.
    ///
    /// # Arguments
    /// * `limit` - Nonzero exclusive upper bound.
    ///
    /// # Returns
    /// The next word modulo `limit`.
    pub fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }

    /// Draw a deterministic string from an alphabet.
    ///
    /// # Arguments
    /// * `alphabet` - Nonempty array of character strings.
    /// * `length` - Number of selections to append.
    ///
    /// # Returns
    /// The concatenation of the selected alphabet entries.
    pub fn text(&mut self, alphabet: &[&str], length: usize) -> String {
        (0..length).map(|_| alphabet[self.below(alphabet.len() as u64) as usize]).collect()
    }
}

/// Resolve an immutable fixture named by an environment variable.
///
/// # Arguments
/// * `var` - Fixture path variable, never a credential variable.
///
/// # Returns
/// The existing frozen fixture path.
///
/// # Panics
/// When the variable is unset or the file is missing.
pub fn fixture(var: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(var).unwrap_or_else(|| panic!("{var} is not set")));
    assert!(path.exists(), "{} is missing; run rust/fixtures.py", path.display());
    path
}

/// Resolve the owner library fixture (`MYMIND_TEST_DB`).
///
/// # Returns
/// The existing immutable database path.
pub fn test_db() -> PathBuf {
    fixture("MYMIND_TEST_DB")
}

/// Resolve the hosted test library fixture (`MYMIND_TEST_HOSTED_DB`).
///
/// # Returns
/// The existing immutable hosted database path.
pub fn hosted_db() -> PathBuf {
    fixture("MYMIND_TEST_HOSTED_DB")
}

/// A copy of `path` in a new directory below the target's temporary directory, for tests that write. Copies that
/// test processes no longer running left behind are removed first.
///
/// # Arguments
/// * `path` - Immutable fixture to copy.
///
/// # Returns
/// A fresh writable copy beneath `CARGO_TARGET_TMPDIR`.
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
    let (Ok(entries), true) = (std::fs::read_dir(base), std::path::Path::new("/proc/self").exists()) else {
        return;
    };
    for entry in entries.flatten() {
        let pid = entry.file_name().to_str().and_then(|name| name.split('-').next()?.parse::<u32>().ok());
        if pid.is_some_and(|pid| pid != std::process::id() && !std::path::Path::new(&format!("/proc/{pid}")).exists()) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}
