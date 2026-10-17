//! Frozen opaque-byte expectations replayed without executable JavaScript oracles.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use flate2::{Compression, GzBuilder, read::GzDecoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const VERSION: u32 = 1;
const SHARD_LIMIT: usize = 10_000_000;
const TOTAL_LIMIT: u64 = 50_000_000;
const INLINE_LIMIT: usize = 1024;
const REGENERATE: &str = "To regenerate JS expectations, check out the last pre-switch revision with lib/ and run this target with MYMIND_RECORD_GOLDEN=1.";
static STORAGE: Mutex<()> = Mutex::new(());
static OCCURRENCES: OnceLock<Mutex<BTreeMap<String, usize>>> = OnceLock::new();
static SEEN: OnceLock<Mutex<BTreeMap<String, BTreeSet<String>>>> = OnceLock::new();
static FIXTURES: OnceLock<Mutex<BTreeMap<PathBuf, FixtureSnapshot>>> = OnceLock::new();
static PORTS: OnceLock<Mutex<BTreeSet<u16>>> = OnceLock::new();

/// Refuse legacy recording in the Rust-only checkout.
///
/// Returns false without launching programs or accessing files. Panics when
/// `MYMIND_RECORD_GOLDEN=1`; recording requires the recorded pre-switch revision.
pub fn recording() -> bool {
    assert!(std::env::var("MYMIND_RECORD_GOLDEN").as_deref() != Ok("1"), "Legacy recording is unavailable after the Rust switch. Check out the recorded pre-switch revision to record JS expectations.");
    false
}

/// Report unequal recording answers with the same case identity as a golden batch.
///
/// `crate_dir` is the caller's `CARGO_MANIFEST_DIR`; `target` names its integration
/// test; `namespace` identifies the operation or scenario; `request` contains its
/// exact canonical request bytes; `actual` is the unequal Rust answer; `expected`
/// is the JS answer. Call only after finding a recording mismatch, before any
/// legacy assertion can print the full answers. Normalization matches `Batch::new`.
/// Never returns: panics with the target, test, namespace, normalized case ID,
/// lengths, SHA-256 digests, first differing offset, bounded actual differing bytes
/// and regeneration instructions. Does not write files, launch programs or read
/// secrets, fixtures or stored golden answers.
pub fn recording_mismatch(
    crate_dir: &str,
    target: &str,
    namespace: &str,
    request: &[u8],
    actual: &[u8],
    expected: &[u8],
) -> ! {
    let crate_dir = Path::new(crate_dir)
        .canonicalize()
        .expect("crate directory exists");
    let rust_dir = find_rust_dir(&crate_dir);
    let id = case_id(namespace, &normalize(request, &rust_dir));
    let label = format!("{target} test {} namespace {namespace}", test_name());
    mismatch(
        &label,
        &id,
        &expected_bytes(expected, false),
        actual,
        Some(expected),
    );
}

/// Register a port allocated to a test's loopback fake server.
///
/// `port` is the actual nonzero local listener port. Returns nothing. Requests
/// normalize only registered ports, preserving literal port numbers in URL tests.
/// Register listeners in deterministic scenario order before opening their batch.
pub fn register_loopback_port(port: u16) {
    assert_ne!(port, 0, "register the assigned loopback port after binding");
    PORTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(port);
}

/// A fixed ordered request corpus and its opaque canonical answers.
///
/// Case IDs hash a version tag, the caller's namespace and exact normalized request bytes.
/// Requests and digest-only answer text are never serialized. Call `check` exactly once
/// for each input, then `finish`; recording writes only after every Rust answer equals JS.
#[must_use = "every golden batch must check every case and call finish"]
pub struct Batch {
    crate_dir: PathBuf,
    rust_dir: PathBuf,
    path: PathBuf,
    target: String,
    key: String,
    record: BatchRecord,
    ids: Vec<String>,
    checked: BTreeSet<usize>,
    expected: Option<BatchRecord>,
    fixtures: FixtureSnapshot,
    source_sha256: String,
    recording: bool,
    inline_synthetic: bool,
    finished: bool,
    failed: bool,
}

/// A complete-test inventory check for tests creating more than one batch.
///
/// Construct at the beginning of a test and call `finish` after all its batches finish.
/// The guard checks that no expected batch was removed, including batches never opened.
#[must_use = "the complete-test inventory guard must call finish"]
pub struct TestGuard {
    path: PathBuf,
    test: String,
    expected: BTreeSet<String>,
    recording: bool,
    finished: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileDigest {
    len: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureManifest {
    version: u32,
    files: BTreeMap<String, FileDigest>,
}

#[derive(Clone)]
struct FixtureSnapshot {
    path: PathBuf,
    manifest: Arc<FixtureManifest>,
    base: PathBuf,
    stamps: Arc<Mutex<BTreeMap<String, FileStamp>>>,
    sha256: String,
}

#[derive(Clone, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<std::time::SystemTime>,
    #[cfg(unix)]
    identity: (u64, u64, i64, i64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    len: usize,
    sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BatchRecord {
    test: String,
    namespace: String,
    occurrence: usize,
    count: usize,
    requests_sha256: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    synthetic: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    inputs: BTreeMap<String, FileDigest>,
    cases: BTreeMap<String, Expected>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Target {
    version: u32,
    normalization_version: u32,
    fixtures_sha256: String,
    source_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recorded_from_revision: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    oracle_sources: BTreeMap<String, FileDigest>,
    batches: BTreeMap<String, BatchRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    shards: Vec<Shard>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Shard {
    file: String,
    len: u64,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShardCase {
    batch: String,
    id: String,
    expected: Expected,
}

impl Batch {
    /// Open a batch without changing its corpus or invoking an oracle.
    ///
    /// `crate_dir` is the caller's `CARGO_MANIFEST_DIR`; `target` names its integration
    /// test file without `.rs`; `namespace` identifies the operation or scenario;
    /// `requests` contains every exact canonical request, in execution order.
    /// Returns an assertion batch. Panics on missing, extra, reordered or changed cases,
    /// source changes, or changed frozen database/bucket files in ordinary mode.
    pub fn new(crate_dir: &str, target: &str, namespace: &str, requests: &[Vec<u8>]) -> Self {
        Self::open(
            Path::new(crate_dir),
            target,
            namespace,
            requests,
            recording(),
            None,
        )
    }

    /// Whether this batch requires live legacy oracle answers.
    ///
    /// Returns the recording mode captured when the batch was opened.
    pub fn recording(&self) -> bool {
        self.recording
    }

    /// Permit small full answers from an explicitly synthetic factory.
    ///
    /// Call before `check` only when every answer in this batch is synthetic and
    /// contains no database-derived content or credentials. Answers over 1024 bytes
    /// remain digest-only. Returns nothing; panics if any case was already checked.
    pub fn allow_inline_synthetic(&mut self) {
        assert!(
            self.checked.is_empty(),
            "{}: inline policy must precede checks",
            self.label()
        );
        self.inline_synthetic = true;
        self.record.synthetic = true;
    }

    /// Read an explicitly synthetic, lossless UTF-8 answer, when one was recorded.
    ///
    /// `index` is an input index from `new`. Returns its full bytes only for a
    /// synthetic inline record of at most 1024 bytes; returns `None` otherwise.
    /// Panics if the index is outside the frozen corpus.
    pub fn inline_answer(&self, index: usize) -> Option<&[u8]> {
        let id = self
            .ids
            .get(index)
            .unwrap_or_else(|| panic!("{}: extra case index {index}", self.label()));
        self.expected
            .as_ref()?
            .cases
            .get(id)?
            .inline
            .as_deref()
            .map(str::as_bytes)
    }

    /// Freeze an extra generated input without storing its contents.
    ///
    /// `path` names an immutable local fixture file. Returns nothing. Its normalized
    /// path, byte length and SHA-256 must equal the recorded input in ordinary mode.
    /// Register the complete input set before `finish`; missing registrations fail.
    /// Panics on missing files, secret filenames, or a changed file fingerprint.
    pub fn register_input(&mut self, path: &Path) {
        reject_secret(path);
        let key = String::from_utf8(normalize(path.to_string_lossy().as_bytes(), &self.rust_dir))
            .expect("normalized path UTF-8");
        let actual = fingerprint(path);
        if let Some(expected) = self.expected.as_ref() {
            let old = expected.inputs.get(&key).unwrap_or_else(|| {
                panic!("{}: extra frozen input {key}. {REGENERATE}", self.label())
            });
            assert_file(&key, old, &actual);
        }
        assert!(
            self.record.inputs.insert(key.clone(), actual).is_none(),
            "{}: input {key} registered twice",
            self.label()
        );
    }

    /// Compare one exact canonical Rust answer before recording its JS expectation.
    ///
    /// `index` selects a request passed to `new`; `actual` is the Rust answer;
    /// `oracle` must contain the JS answer in recording mode and must be `None` in
    /// ordinary mode. Returns nothing. Panics on any byte mismatch, missing oracle,
    /// repeated/extra index, or changed expected length/digest. No files are written.
    pub fn check(&mut self, index: usize, actual: &[u8], oracle: Option<&[u8]>) {
        self.failed = true;
        let id = self
            .ids
            .get(index)
            .unwrap_or_else(|| {
                panic!(
                    "{}: extra case index {index}; frozen count {}. {REGENERATE}",
                    self.label(),
                    self.ids.len()
                )
            })
            .clone();
        assert!(
            self.checked.insert(index),
            "{} case {id}: case index {index} checked twice. {REGENERATE}",
            self.label()
        );
        if self.recording {
            let oracle = oracle.unwrap_or_else(|| {
                panic!(
                    "{} case {id}: recording requires a JS answer. {REGENERATE}",
                    self.label()
                )
            });
            if actual != oracle {
                mismatch(
                    &self.label(),
                    &id,
                    &expected_bytes(oracle, false),
                    actual,
                    Some(oracle),
                );
            }
            let expected = expected_bytes(oracle, self.inline_synthetic);
            if let Some(previous) = self.record.cases.insert(id.clone(), expected.clone()) {
                assert!(
                    same_expected(&previous, &expected),
                    "{} case {id}: duplicate request ID produced different answers; include the entire ordered scenario and initial fixture identity in the request. {REGENERATE}",
                    self.label()
                );
            }
        } else {
            assert!(
                oracle.is_none(),
                "{} case {id}: ordinary golden replay must not invoke JS",
                self.label()
            );
            let expected = self
                .expected
                .as_ref()
                .and_then(|record| record.cases.get(&id))
                .unwrap_or_else(|| {
                    panic!(
                        "{} case {id}: missing expectation. {REGENERATE}",
                        self.label()
                    )
                });
            if expected.len != actual.len() || expected.sha256 != digest(actual) {
                mismatch(
                    &self.label(),
                    &id,
                    expected,
                    actual,
                    expected.inline.as_deref().map(str::as_bytes),
                );
            }
        }
        self.failed = false;
    }

    /// Finish every comparison, then atomically persist a successful recording.
    ///
    /// Consumes the batch and returns nothing. Panics if a case or registered input
    /// is missing, fixture files changed, or the total golden budget exceeds 50 MB.
    /// Ordinary mode never writes, creates directories or launches a program.
    pub fn finish(mut self) {
        self.failed = true;
        assert_eq!(
            self.checked.len(),
            self.ids.len(),
            "{}: missing comparisons; checked {}, expected {}. {REGENERATE}",
            self.label(),
            self.checked.len(),
            self.ids.len()
        );
        if let Some(expected) = &self.expected {
            assert_eq!(
                self.record.inputs.keys().collect::<Vec<_>>(),
                expected.inputs.keys().collect::<Vec<_>>(),
                "{}: missing frozen input registrations. {REGENERATE}",
                self.label()
            );
        }
        refresh_fixtures(&self.fixtures, !self.fixtures.path.exists());
        if self.recording {
            let _lock = STORAGE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target = load_target(&self.path).unwrap_or_default();
            if target.source_sha256 != self.source_sha256 {
                target.batches.clear();
            }
            target.version = VERSION;
            target.normalization_version = VERSION;
            target.fixtures_sha256 = self.fixtures.sha256.clone();
            target.source_sha256 = self.source_sha256.clone();
            target.recorded_from_revision = target
                .recorded_from_revision
                .or_else(|| revision(&self.rust_dir));
            target.oracle_sources = oracle_sources(&self.crate_dir, &self.rust_dir);
            target.batches.insert(self.key.clone(), self.record.clone());
            save_target(&self.rust_dir, &self.path, target, &self.fixtures);
        }
        SEEN.get_or_init(Default::default)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(session_key(&self.path, &self.record.test))
            .or_default()
            .insert(self.key.clone());
        self.finished = true;
    }

    // Open a batch with an optional isolated fixture snapshot for support tests.
    fn open(
        crate_dir: &Path,
        target: &str,
        namespace: &str,
        requests: &[Vec<u8>],
        record_mode: bool,
        fixture_override: Option<FixtureSnapshot>,
    ) -> Self {
        assert!(
            !target.is_empty()
                && target
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
            "invalid golden target filename"
        );
        assert!(!namespace.is_empty(), "golden namespace must not be empty");
        let crate_dir = crate_dir.canonicalize().expect("crate directory exists");
        let rust_dir = find_rust_dir(&crate_dir);
        let path = crate_dir
            .join("tests/golden")
            .join(format!("{target}.json"));
        let test = test_name();
        let counter_key = format!("{}\0{test}\0{namespace}", path.display());
        let occurrence = {
            let mut counters = OCCURRENCES
                .get_or_init(Default::default)
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let next = counters.entry(counter_key).or_default();
            let occurrence = *next;
            *next += 1;
            occurrence
        };
        let key = format!("{test}::{namespace}::{occurrence:06}");
        let mut port_aliases = BTreeMap::new();
        let mut temporary_aliases = BTreeMap::new();
        let normalized = requests
            .iter()
            .map(|request| {
                normalize_with_aliases(
                    request,
                    &rust_dir,
                    &mut port_aliases,
                    &mut temporary_aliases,
                )
            })
            .collect::<Vec<_>>();
        let ids = normalized
            .iter()
            .map(|request| case_id(namespace, request))
            .collect::<Vec<_>>();
        let mut stream = Sha256::new();
        stream.update(b"mymind-request-stream-v1\0");
        for request in &normalized {
            stream.update((request.len() as u64).to_be_bytes());
            stream.update(request);
        }
        let requests_sha256 = format!("{:x}", stream.finalize());
        let source_sha256 = source_digest(&crate_dir, target);
        let fixtures = fixture_override.unwrap_or_else(|| {
            verify_fixture_environment(&rust_dir);
            fixture_snapshot(&rust_dir, record_mode)
        });
        refresh_fixtures(&fixtures, false);
        let loaded = if record_mode {
            None
        } else {
            load_target(&path)
        };
        let expected = if record_mode {
            None
        } else {
            let old = loaded.unwrap_or_else(|| {
                panic!(
                    "{target} test {test}: missing golden {}. {REGENERATE}",
                    path.display()
                )
            });
            assert_eq!(
                old.version, VERSION,
                "{target}: incompatible golden format. {REGENERATE}"
            );
            assert_eq!(
                old.normalization_version, VERSION,
                "{target}: incompatible normalization. {REGENERATE}"
            );
            assert_eq!(
                old.fixtures_sha256, fixtures.sha256,
                "{target}: frozen fixture manifest changed. {REGENERATE}"
            );
            assert_eq!(
                old.source_sha256, source_sha256,
                "{target}: test source changed; case counts or batch inventory may have changed. {REGENERATE}"
            );
            let expected = old.batches.get(&key).unwrap_or_else(|| panic!("{target} test {test} namespace {namespace}: missing batch {key}. {REGENERATE}")).clone();
            assert_eq!(
                expected.count,
                requests.len(),
                "{target} batch {key}: changed case count; expected {}, actual {}. {REGENERATE}",
                expected.count,
                requests.len()
            );
            assert_eq!(
                expected.requests_sha256,
                requests_sha256,
                "{target} batch {key}: changed/reordered request corpus; expected count {}, actual count {}. {REGENERATE}",
                expected.count,
                requests.len()
            );
            assert_eq!(
                expected.cases.keys().cloned().collect::<BTreeSet<_>>(),
                ids.iter().cloned().collect::<BTreeSet<_>>(),
                "{target} batch {key}: missing/extra case IDs. {REGENERATE}"
            );
            Some(expected)
        };
        let record = BatchRecord {
            test,
            namespace: namespace.to_owned(),
            occurrence,
            count: requests.len(),
            requests_sha256,
            ..Default::default()
        };
        Self {
            crate_dir,
            rust_dir,
            path,
            target: target.to_owned(),
            key,
            record,
            ids,
            checked: BTreeSet::new(),
            expected,
            fixtures,
            source_sha256,
            recording: record_mode,
            inline_synthetic: false,
            finished: false,
            failed: false,
        }
    }

    // Identify a failing target, test and namespace without printing private requests.
    fn label(&self) -> String {
        format!(
            "{} test {} namespace {}",
            self.target, self.record.test, self.record.namespace
        )
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        if !self.finished && !self.failed && !std::thread::panicking() {
            panic!(
                "{}: batch dropped without finish; checked {}, expected {}. {REGENERATE}",
                self.label(),
                self.checked.len(),
                self.ids.len()
            );
        }
    }
}

impl TestGuard {
    /// Open the current test's complete batch inventory.
    ///
    /// `crate_dir` is the caller's `CARGO_MANIFEST_DIR`; `target` names the integration
    /// target. Returns a guard whose `finish` rejects unused or extra stored batches.
    /// Validates the frozen fixtures before the caller can read their rows or bytes.
    /// Panics on a changed fixture or when an ordinary replay has no golden file.
    pub fn new(crate_dir: &str, target: &str) -> Self {
        Self::open(Path::new(crate_dir), target, recording(), true)
    }

    // Validate fixtures before opening an inventory, isolating environment checks in unit tests.
    fn open(crate_dir: &Path, target: &str, record_mode: bool, verify_environment: bool) -> Self {
        let crate_dir = crate_dir.canonicalize().expect("crate directory exists");
        let rust_dir = find_rust_dir(&crate_dir);
        if verify_environment {
            verify_fixture_environment(&rust_dir);
        }
        refresh_fixtures(&fixture_snapshot(&rust_dir, record_mode), true);
        let path = crate_dir
            .join("tests/golden")
            .join(format!("{target}.json"));
        let test = test_name();
        let expected = (if record_mode {
            None
        } else {
            load_target(&path)
        })
        .map(|target| {
            target
                .batches
                .into_iter()
                .filter_map(|(key, record)| (record.test == test).then_some(key))
                .collect()
        })
        .unwrap_or_else(|| {
            assert!(
                record_mode,
                "{target} test {test}: missing golden inventory. {REGENERATE}"
            );
            BTreeSet::new()
        });
        Self {
            path,
            test,
            expected,
            recording: record_mode,
            finished: false,
        }
    }

    /// Require the exact complete batch set for this test.
    ///
    /// Consumes the guard and returns nothing. Panics on an extra or missing batch.
    /// Recording rejects stale batches for this test that were not exercised.
    pub fn finish(mut self) {
        let rust_dir = find_rust_dir(&self.path);
        refresh_fixtures(&fixture_snapshot(&rust_dir, self.recording), true);
        let seen = SEEN
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&session_key(&self.path, &self.test))
            .cloned()
            .unwrap_or_default();
        if self.recording {
            let _lock = STORAGE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let target = load_target(&self.path)
                .expect("a successful recording batch precedes guard finish");
            let stored = target
                .batches
                .iter()
                .filter(|(_, record)| record.test == self.test)
                .map(|(key, _)| key.clone())
                .collect::<BTreeSet<_>>();
            assert_eq!(
                stored, seen,
                "test {}: missing/extra recorded batches; remove stale target goldens before recording a changed test. {REGENERATE}",
                self.test
            );
        } else {
            assert_eq!(
                seen, self.expected,
                "test {}: missing/extra batches in full-test inventory. {REGENERATE}",
                self.test
            );
        }
        self.finished = true;
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        if !self.finished && !std::thread::panicking() {
            panic!(
                "test {}: complete-test inventory guard dropped without finish. {REGENERATE}",
                self.test
            );
        }
    }
}

// Return the current Rust test's stable name, never the executable's hashed filename.
fn test_name() -> String {
    std::thread::current()
        .name()
        .unwrap_or("unnamed-test")
        .to_owned()
}

// Key a per-test inventory without exposing request bytes.
fn session_key(path: &Path, test: &str) -> String {
    format!("{}\0{test}", path.display())
}

// Hash exact bytes as lowercase SHA-256 hexadecimal.
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Frame namespace and request lengths so different concatenations cannot collide.
fn case_id(namespace: &str, request: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"mymind-golden-case-v1\0");
    hash.update((namespace.len() as u64).to_be_bytes());
    hash.update(namespace.as_bytes());
    hash.update((request.len() as u64).to_be_bytes());
    hash.update(request);
    format!("{:x}", hash.finalize())
}

// Locate the workspace from a caller crate without reading any configuration secrets.
fn find_rust_dir(crate_dir: &Path) -> PathBuf {
    crate_dir
        .ancestors()
        .find(|path| path.join("PORTING.md").is_file() && path.join("Cargo.toml").is_file())
        .unwrap_or_else(|| panic!("{} is outside the Rust workspace", crate_dir.display()))
        .to_owned()
}

// Fingerprint test source so static test/batch removals cannot silently pass.
fn source_digest(crate_dir: &Path, target: &str) -> String {
    let path = crate_dir.join("tests").join(format!("{target}.rs"));
    let mut sources = BTreeMap::from([(format!("{target}.rs"), fingerprint(&path))]);
    for name in ["common", "tools_common"] {
        let directory = crate_dir.join("tests").join(name);
        if directory.exists() {
            collect_files(&crate_dir.join("tests"), &directory, &mut sources);
        }
    }
    source_identity(crate_dir.to_str().unwrap(), target, "source", &digest(&encode(&sources)))
}

/// Preserve a frozen source identity only for an explicitly reviewed exact source transition.
///
/// `crate_dir` is the caller's crate directory; `target` is its integration target;
/// `kind` is `source` or `recipe`; `actual` is the current source SHA-256.
/// Returns the frozen identity for an exact approved transition, otherwise `actual`.
/// Later source edits still fail the existing golden source guard.
pub fn source_identity(crate_dir: &str, target: &str, kind: &str, actual: &str) -> String {
    let transitions: serde_json::Value = serde_json::from_str(include_str!("../source-transitions.json")).expect("reviewed source transition metadata");
    let name = Path::new(crate_dir).file_name().unwrap().to_str().unwrap();
    let transition = &transitions[format!("{name}/{target}")][kind];
    if transition["after"].as_str() == Some(actual) {
        transition["before"].as_str().expect("frozen source identity").to_owned()
    } else {
        actual.to_owned()
    }
}

// Represent an answer without copying private content into the golden corpus.
fn expected_bytes(bytes: &[u8], inline_synthetic: bool) -> Expected {
    Expected {
        len: bytes.len(),
        sha256: digest(bytes),
        inline: (inline_synthetic && bytes.len() <= INLINE_LIMIT)
            .then(|| std::str::from_utf8(bytes).ok().map(str::to_owned))
            .flatten(),
    }
}

// Check repeated request IDs have exactly one answer contract.
fn same_expected(left: &Expected, right: &Expected) -> bool {
    left.len == right.len && left.sha256 == right.sha256 && left.inline == right.inline
}

// Report bounded actual bytes honestly when expected text is intentionally absent.
fn mismatch(
    label: &str,
    id: &str,
    expected: &Expected,
    actual: &[u8],
    full_expected: Option<&[u8]>,
) -> ! {
    let detail = if let Some(bytes) = full_expected {
        let offset = bytes
            .iter()
            .zip(actual)
            .position(|(left, right)| left != right)
            .unwrap_or(bytes.len().min(actual.len()));
        let prefix = &actual[offset.min(actual.len())..actual.len().min(offset + 16)];
        format!("first difference at byte {offset}; actual differing prefix bytes {prefix:02x?}")
    } else {
        let prefix = &actual[..actual.len().min(16)];
        format!(
            "actual answer first bytes {prefix:02x?}; expected text omitted, so the first differing offset cannot be located"
        )
    };
    panic!(
        "{label} case {id}: expected length {} sha256 {}, actual length {} sha256 {}; {detail}. {REGENERATE}",
        expected.len,
        expected.sha256,
        actual.len(),
        digest(actual)
    );
}

// Reject credential-bearing fixture names before any read.
fn reject_secret(path: &Path) {
    assert!(
        !path
            .components()
            .any(|component| matches!(component.as_os_str().to_str(), Some(".env" | ".dev.vars"))),
        "secret files cannot be golden inputs"
    );
}

/// Read an immutable input only after rejecting secret names and symlink targets.
///
/// `path` must name a regular local fixture file. Returns its complete bytes.
/// Panics before opening a secret filename, a symlink, or a path resolving through
/// a secret-named component; also panics when the input cannot be read.
pub fn read_input(path: &Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    input_file(path)
        .read_to_end(&mut bytes)
        .expect("read frozen input");
    bytes
}

// Validate the filename, type and resolved path before opening any fixture bytes.
fn input_file(path: &Path) -> fs::File {
    reject_secret(path);
    let metadata = fs::symlink_metadata(path).unwrap_or_else(|error| {
        panic!(
            "frozen input {} is missing or unreadable: {error}. {REGENERATE}",
            path.display()
        )
    });
    assert!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "frozen input {} must be a regular file, never a symlink",
        path.display()
    );
    let resolved = path.canonicalize().expect("resolve frozen input");
    reject_secret(&resolved);
    fs::File::open(&resolved).unwrap_or_else(|error| {
        panic!(
            "frozen input {} is missing or unreadable: {error}. {REGENERATE}",
            path.display()
        )
    })
}

// Stream a validated file into its length/digest descriptor.
fn fingerprint(path: &Path) -> FileDigest {
    let mut file = input_file(path);
    let mut hash = Sha256::new();
    let mut len = 0;
    let mut block = [0; 128 * 1024];
    loop {
        let count = file.read(&mut block).expect("read frozen input");
        if count == 0 {
            break;
        }
        len += count as u64;
        hash.update(&block[..count]);
    }
    FileDigest {
        len,
        sha256: format!("{:x}", hash.finalize()),
    }
}

// Fail a changed file with useful fingerprints and no file contents.
fn assert_file(name: &str, expected: &FileDigest, actual: &FileDigest) {
    assert!(
        expected.len == actual.len && expected.sha256 == actual.sha256,
        "frozen input {name} changed: expected length {} sha256 {}, actual length {} sha256 {}. Goldens were recorded against these exact fixture bytes. Restore the frozen fixture; do not copy mutable data/ over it. {REGENERATE}",
        expected.len,
        expected.sha256,
        actual.len,
        actual.sha256
    );
}

// Enumerate immutable regular files, rejecting symlinks rather than following outside fixtures.
fn collect_files(base: &Path, current: &Path, output: &mut BTreeMap<String, FileDigest>) {
    reject_secret(current);
    let metadata = fs::symlink_metadata(current).unwrap_or_else(|error| {
        panic!(
            "frozen input {} is missing: {error}. {REGENERATE}",
            current.display()
        )
    });
    assert!(
        !metadata.file_type().is_symlink(),
        "frozen input {} must not be a symlink",
        current.display()
    );
    if metadata.is_dir() {
        let mut paths = fs::read_dir(current)
            .expect("read fixture directory")
            .map(|entry| entry.expect("fixture entry").path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_files(base, &path, output);
        }
    } else if metadata.is_file() {
        output.insert(
            current
                .strip_prefix(base)
                .expect("fixture below base")
                .to_string_lossy()
                .replace('\\', "/"),
            fingerprint(current),
        );
    } else {
        panic!("frozen input {} is not a regular file", current.display());
    }
}

// Hash the baseline database/bucket set once per process, excluding later generated media inputs.
fn fixture_snapshot(rust_dir: &Path, record_mode: bool) -> FixtureSnapshot {
    let mut cache = FIXTURES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(snapshot) = cache.get(rust_dir) {
        return snapshot.clone();
    }
    let base = rust_dir.join("test-fixtures");
    let mut files = BTreeMap::new();
    for name in ["mymind.db", "hosted-test.db", "hosted-bucket"] {
        collect_files(&base, &base.join(name), &mut files);
    }
    let manifest = FixtureManifest {
        version: VERSION,
        files,
    };
    let path = rust_dir.join("crates/core/tests/golden/fixtures-manifest.json");
    if path.exists() {
        let expected: FixtureManifest =
            decode(&fs::read(&path).expect("read fixture manifest"), &path);
        assert_eq!(
            expected.version, VERSION,
            "incompatible frozen fixture manifest. {REGENERATE}"
        );
        assert_eq!(
            expected.files.keys().collect::<Vec<_>>(),
            manifest.files.keys().collect::<Vec<_>>(),
            "frozen fixture files are missing or extra. Restore rust/test-fixtures, do not regenerate it from data/. {REGENERATE}"
        );
        for (name, actual) in &manifest.files {
            assert_file(name, &expected.files[name], actual);
        }
    } else {
        assert!(
            record_mode,
            "missing frozen fixture manifest {}. {REGENERATE}",
            path.display()
        );
    }
    let sha256 = digest(&encode(&manifest));
    let mut stamps = BTreeMap::new();
    for name in manifest.files.keys() {
        stamps.insert(name.clone(), file_stamp(&base.join(name)));
    }
    let snapshot = FixtureSnapshot {
        path,
        manifest: Arc::new(manifest),
        base,
        stamps: Arc::new(Mutex::new(stamps)),
        sha256,
    };
    cache.insert(rust_dir.to_owned(), snapshot.clone());
    snapshot
}

// Capture cheap metadata including Unix change time so content edits cannot hide behind reset mtimes.
fn file_stamp(path: &Path) -> FileStamp {
    let metadata = fs::symlink_metadata(path).unwrap_or_else(|error| {
        panic!(
            "frozen input {} is missing: {error}. {REGENERATE}",
            path.display()
        )
    });
    assert!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "frozen input {} is no longer a regular file",
        path.display()
    );
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        (
            metadata.dev(),
            metadata.ino(),
            metadata.ctime(),
            metadata.ctime_nsec(),
        )
    };
    FileStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        identity,
    }
}

// Enumerate file identities cheaply, leaving unchanged content hashes cached.
fn collect_stamps(base: &Path, current: &Path, stamps: &mut BTreeMap<String, FileStamp>) {
    let metadata = fs::symlink_metadata(current).unwrap_or_else(|error| {
        panic!(
            "frozen input {} is missing: {error}. {REGENERATE}",
            current.display()
        )
    });
    assert!(
        !metadata.file_type().is_symlink(),
        "frozen input {} must not be a symlink",
        current.display()
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(current).expect("read frozen directory") {
            collect_stamps(base, &entry.expect("frozen entry").path(), stamps);
        }
    } else {
        stamps.insert(
            current
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            file_stamp(current),
        );
    }
}

// Rehash changed files only; inspect databases per batch and the complete bucket at test completion.
fn refresh_fixtures(snapshot: &FixtureSnapshot, all: bool) {
    let mut actual = BTreeMap::new();
    for name in ["mymind.db", "hosted-test.db"] {
        actual.insert(name.to_owned(), file_stamp(&snapshot.base.join(name)));
    }
    if all {
        collect_stamps(
            &snapshot.base,
            &snapshot.base.join("hosted-bucket"),
            &mut actual,
        );
        assert_eq!(
            actual.keys().collect::<Vec<_>>(),
            snapshot.manifest.files.keys().collect::<Vec<_>>(),
            "frozen fixture files are missing or extra. {REGENERATE}"
        );
    }
    let mut cached = snapshot
        .stamps
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for (name, stamp) in actual {
        if cached.get(&name) != Some(&stamp) {
            assert_file(
                &name,
                &snapshot.manifest.files[&name],
                &fingerprint(&snapshot.base.join(&name)),
            );
            cached.insert(name, stamp);
        }
    }
}

// Reject fixture overrides that would test a different database than the frozen manifest.
fn verify_fixture_environment(rust_dir: &Path) {
    for (name, file) in [
        ("MYMIND_TEST_DB", "mymind.db"),
        ("MYMIND_TEST_HOSTED_DB", "hosted-test.db"),
        ("MYMIND_TEST_HOSTED_BUCKET", "hosted-bucket"),
    ] {
        if let Some(actual) = std::env::var_os(name) {
            let actual = PathBuf::from(actual)
                .canonicalize()
                .unwrap_or_else(|error| {
                    panic!("{name} frozen fixture is missing: {error}. {REGENERATE}")
                });
            let expected = rust_dir
                .join("test-fixtures")
                .join(file)
                .canonicalize()
                .expect("frozen fixture exists");
            assert_eq!(
                actual, expected,
                "{name} must select the immutable rust/test-fixtures/{file}; an override would bypass the frozen fixture fingerprint. {REGENERATE}"
            );
        }
    }
}

// Replace known paths byte-for-byte and normalize numeric ports on explicit loopback hosts.
fn normalize(request: &[u8], rust_dir: &Path) -> Vec<u8> {
    normalize_with_aliases(
        request,
        rust_dir,
        &mut BTreeMap::new(),
        &mut BTreeMap::new(),
    )
}

// Preserve listener and copy relationships across every request in the ordered batch.
fn normalize_with_aliases(
    request: &[u8],
    rust_dir: &Path,
    port_aliases: &mut BTreeMap<u16, usize>,
    temporary_aliases: &mut BTreeMap<Vec<u8>, usize>,
) -> Vec<u8> {
    let mut replacements = Vec::<(Vec<u8>, Vec<u8>)>::new();
    for (variable, placeholder) in [
        ("MYMIND_TEST_DB", "<fixture:owner-db>"),
        ("MYMIND_TEST_HOSTED_DB", "<fixture:hosted-db>"),
        ("MYMIND_TEST_HOSTED_BUCKET", "<fixture:hosted-bucket>"),
    ] {
        if let Some(path) = std::env::var_os(variable) {
            let path = PathBuf::from(path);
            if let Some(text) = path.to_str() {
                replacements.push((text.as_bytes().to_vec(), placeholder.as_bytes().to_vec()));
            }
        }
    }
    replacements.push((
        rust_dir
            .join("test-fixtures")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
        b"<fixtures>".to_vec(),
    ));
    replacements.push((
        rust_dir
            .join("target/tmp")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
        b"<temporary>".to_vec(),
    ));
    if let Some(root) = rust_dir.parent() {
        replacements.push((
            root.to_string_lossy().as_bytes().to_vec(),
            b"<repository>".to_vec(),
        ));
        for entry in fs::read_dir(rust_dir.join("crates"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let crate_path = entry.path();
            if crate_path.is_dir() {
                replacements.push((
                    crate_path
                        .join("../../..")
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                    b"<repository>".to_vec(),
                ));
                replacements.push((
                    crate_path
                        .join("../core/tests")
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                    b"<repository>/rust/crates/core/tests".to_vec(),
                ));
            }
        }
    }
    replacements.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));
    let mut bytes = request.to_vec();
    for (from, to) in replacements {
        bytes = replace_bytes(&bytes, &from, &to);
    }
    let ports = PORTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let host = [b"127.0.0.1:".as_slice(), b"localhost:", b"[::1]:"]
            .into_iter()
            .find(|host| bytes[at..].starts_with(host));
        if let Some(host) = host {
            normalized.extend_from_slice(host);
            at += host.len();
            let end = at
                + bytes[at..]
                    .iter()
                    .take_while(|byte| byte.is_ascii_digit())
                    .count();
            let port = std::str::from_utf8(&bytes[at..end])
                .ok()
                .and_then(|text| text.parse::<u16>().ok());
            if let Some(port) = port.filter(|port| ports.contains(port)) {
                let next = port_aliases.len();
                let index = port_aliases.entry(port).or_insert(next);
                normalized.extend_from_slice(format!("<loopback-port:{index}>").as_bytes());
                at = end;
            }
        } else {
            normalized.push(bytes[at]);
            at += 1;
        }
    }
    bytes = normalized;
    normalize_temp(&bytes, temporary_aliases)
}

// Remove process IDs and allocation counters from known temporary directory layouts.
fn normalize_temp(bytes: &[u8], aliases: &mut BTreeMap<Vec<u8>, usize>) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let prefix = if bytes[at..].starts_with(b"<temporary>/fixture-copies/") {
            Some(b"<temporary>/fixture-copies/".as_slice())
        } else if bytes[at..].starts_with(b"<temporary>/tools-") {
            Some(b"<temporary>/tools-".as_slice())
        } else if bytes[at..].starts_with(b"<temporary>/auth-rust-") {
            Some(b"<temporary>/auth-rust-".as_slice())
        } else {
            None
        };
        if let Some(prefix) = prefix {
            output.extend_from_slice(prefix);
            at += prefix.len();
            let end = at
                + bytes[at..]
                    .iter()
                    .take_while(|byte| byte.is_ascii_digit() || **byte == b'-')
                    .count();
            if end > at {
                let mut identity = prefix.to_vec();
                identity.extend_from_slice(&bytes[at..end]);
                let next = aliases.len();
                let index = aliases.entry(identity).or_insert(next);
                output.extend_from_slice(format!("<instance:{index}>").as_bytes());
                at = end;
            }
        } else {
            output.push(bytes[at]);
            at += 1;
        }
    }
    let pid = std::process::id().to_string();
    let mut normalized = Vec::with_capacity(output.len());
    let mut at = 0;
    while at < output.len() {
        if output[at..].starts_with(b"<temporary>/") {
            let end = at
                + output[at..]
                    .iter()
                    .position(|byte| matches!(byte, b'"' | b'\'' | b' ' | b'\n'))
                    .unwrap_or(output.len() - at);
            let path = &output[at..end];
            let mut position = 0;
            while position < path.len() {
                let before = position == 0 || matches!(path[position - 1], b'-' | b'/');
                let after = position + pid.len();
                if before
                    && path[position..].starts_with(pid.as_bytes())
                    && (after == path.len() || matches!(path[after], b'-' | b'/'))
                {
                    normalized.extend_from_slice(b"<pid>");
                    position = after;
                } else {
                    normalized.push(path[position]);
                    position += 1;
                }
            }
            at = end;
        } else {
            normalized.push(output[at]);
            at += 1;
        }
    }
    normalized
}

// Replace literal nonempty byte strings without interpreting or reserializing requests.
fn replace_bytes(input: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    if from.is_empty() {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(input.len());
    let mut at = 0;
    while at < input.len() {
        if input[at..].starts_with(from) {
            output.extend_from_slice(to);
            at += from.len();
        } else {
            output.push(input[at]);
            at += 1;
        }
    }
    output
}

// Serialize deterministic metadata with one final newline.
fn encode(value: &impl Serialize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize golden metadata");
    bytes.push(b'\n');
    bytes
}

// Deserialize a versioned metadata file with its filename in errors.
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8], path: &Path) -> T {
    serde_json::from_slice(bytes)
        .unwrap_or_else(|error| panic!("invalid golden {}: {error}. {REGENERATE}", path.display()))
}

// Resolve exactly one plain or gzip representation of a logical golden filename.
fn stored_path(path: &Path) -> Option<PathBuf> {
    let compressed = PathBuf::from(format!("{}.gz", path.display()));
    assert!(
        !(path.exists() && compressed.exists()),
        "duplicate plain and gzip goldens for {}. {REGENERATE}",
        path.display()
    );
    if path.exists() {
        Some(path.to_owned())
    } else if compressed.exists() {
        Some(compressed)
    } else {
        None
    }
}

// Inflate a declared gzip file while keeping ordinary JSON/JSONL opaque.
fn inflate(path: &Path, bytes: &[u8]) -> Vec<u8> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("gz") {
        return bytes.to_vec();
    }
    let mut plain = Vec::new();
    GzDecoder::new(bytes)
        .read_to_end(&mut plain)
        .unwrap_or_else(|error| {
            panic!(
                "invalid gzip golden {}: {error}. {REGENERATE}",
                path.display()
            )
        });
    plain
}

// Compress with a deterministic header only when at least one quarter is saved.
fn stored_bytes(path: &Path, bytes: Vec<u8>) -> (PathBuf, Vec<u8>) {
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    encoder.write_all(&bytes).expect("compress golden");
    let compressed = encoder.finish().expect("finish golden compression");
    if compressed.len() * 4 <= bytes.len() * 3 {
        (PathBuf::from(format!("{}.gz", path.display())), compressed)
    } else {
        (path.to_owned(), bytes)
    }
}

// Read all declared shards and reject undeclared ones without touching output files.
fn load_target(path: &Path) -> Option<Target> {
    let stored = stored_path(path)?;
    let mut target: Target = decode(
        &inflate(&stored, &fs::read(&stored).expect("read golden")),
        &stored,
    );
    let mut declared = BTreeSet::new();
    for shard in &target.shards {
        assert!(
            Path::new(&shard.file)
                .file_name()
                .and_then(|name| name.to_str())
                == Some(shard.file.as_str()),
            "invalid shard filename"
        );
        let shard_path = path.parent().unwrap().join(&shard.file);
        let bytes = fs::read(&shard_path).unwrap_or_else(|error| {
            panic!(
                "missing golden shard {}: {error}. {REGENERATE}",
                shard_path.display()
            )
        });
        assert!(
            bytes.len() as u64 == shard.len && digest(&bytes) == shard.sha256,
            "changed golden shard {}. {REGENERATE}",
            shard_path.display()
        );
        declared.insert(shard.file.clone());
        let plain = inflate(&shard_path, &bytes);
        for line in plain
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let case: ShardCase = decode(line, &shard_path);
            let record = target
                .batches
                .get_mut(&case.batch)
                .expect("declared shard batch");
            assert!(
                record.cases.insert(case.id, case.expected).is_none(),
                "duplicate sharded golden case"
            );
        }
    }
    assert_eq!(
        shard_files(path),
        declared,
        "missing or extra target shards {}. {REGENERATE}",
        path.display()
    );
    for record in target.batches.values() {
        assert!(
            record.cases.len() <= record.count,
            "golden case count exceeds frozen batch count"
        );
        for expected in record.cases.values() {
            if let Some(text) = &expected.inline {
                assert!(
                    record.synthetic
                        && text.len() <= INLINE_LIMIT
                        && expected.len == text.len()
                        && expected.sha256 == digest(text.as_bytes()),
                    "invalid inline golden: only explicitly synthetic answers <=1024 bytes may be stored"
                );
            }
        }
    }
    Some(target)
}

// List only this target's numbered JSONL shards.
fn shard_files(path: &Path) -> BTreeSet<String> {
    let prefix = format!("{}.part-", path.file_stem().unwrap().to_string_lossy());
    fs::read_dir(path.parent().unwrap())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            name.starts_with(&prefix) && (name.ends_with(".jsonl") || name.ends_with(".jsonl.gz"))
        })
        .collect()
}

// Atomically save sorted batches, sharding only above the 10 MB on-disk threshold.
fn save_target(rust_dir: &Path, path: &Path, mut target: Target, fixtures: &FixtureSnapshot) {
    save_target_with_limit(rust_dir, path, &mut target, fixtures, SHARD_LIMIT);
}

// Apply the same storage algorithm with a smaller boundary in isolated unit tests.
fn save_target_with_limit(
    rust_dir: &Path,
    path: &Path,
    target: &mut Target,
    fixtures: &FixtureSnapshot,
    shard_limit: usize,
) {
    target.shards.clear();
    let (full_path, full_bytes) = stored_bytes(
        path,
        serde_json::to_vec(&target).expect("serialize compact golden"),
    );
    let mut files = BTreeMap::<PathBuf, Vec<u8>>::new();
    let actual_target_path;
    if full_bytes.len() <= shard_limit {
        actual_target_path = full_path.clone();
        files.insert(full_path, full_bytes);
    } else {
        let mut chunks = Vec::new();
        let mut chunk = Vec::new();
        for (batch, record) in &mut target.batches {
            for (id, expected) in std::mem::take(&mut record.cases) {
                let mut line = serde_json::to_vec(&ShardCase {
                    batch: batch.clone(),
                    id,
                    expected,
                })
                .unwrap();
                line.push(b'\n');
                assert!(
                    line.len() <= shard_limit,
                    "one golden record exceeds the per-file budget; no files were written"
                );
                if chunk.len() + line.len() > shard_limit && !chunk.is_empty() {
                    chunks.push(std::mem::take(&mut chunk));
                }
                chunk.extend(line);
            }
        }
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        for (index, bytes) in chunks.into_iter().enumerate() {
            let logical = PathBuf::from(format!(
                "{}.part-{index:04}.jsonl",
                path.file_stem().unwrap().to_string_lossy()
            ));
            let (stored, bytes) = stored_bytes(&logical, bytes);
            let file = stored.to_string_lossy().into_owned();
            assert!(
                bytes.len() <= shard_limit,
                "compressed shard exceeds per-file budget"
            );
            target.shards.push(Shard {
                file: file.clone(),
                len: bytes.len() as u64,
                sha256: digest(&bytes),
            });
            files.insert(path.parent().unwrap().join(file), bytes);
        }
        let (header, bytes) = stored_bytes(
            path,
            serde_json::to_vec(&target).expect("serialize golden index"),
        );
        assert!(
            bytes.len() <= shard_limit,
            "golden target metadata exceeds the per-file budget; no files were written"
        );
        actual_target_path = header.clone();
        files.insert(header, bytes);
    }
    if !fixtures.path.exists() {
        files.insert(fixtures.path.clone(), encode(fixtures.manifest.as_ref()));
    }
    let old_shards = shard_files(path);
    let old_target = stored_path(path);
    let mut total = golden_size(rust_dir);
    for (file, bytes) in &files {
        total = total.saturating_sub(fs::metadata(file).map_or(0, |metadata| metadata.len()))
            + bytes.len() as u64;
    }
    for shard in &old_shards {
        let file = path.parent().unwrap().join(shard);
        if !files.contains_key(&file) {
            total = total.saturating_sub(fs::metadata(file).map_or(0, |metadata| metadata.len()));
        }
    }
    if let Some(old) = &old_target
        && !files.contains_key(old)
    {
        total = total.saturating_sub(fs::metadata(old).map_or(0, |metadata| metadata.len()));
    }
    assert!(
        total <= TOTAL_LIMIT,
        "golden corpus would use {total} bytes, above the 50 MB budget; no files were written"
    );
    for (file, bytes) in &files {
        if file != &actual_target_path {
            atomic_write(file, bytes);
        }
    }
    atomic_write(&actual_target_path, &files[&actual_target_path]);
    if let Some(old) = old_target
        && old != actual_target_path
    {
        fs::remove_file(old).expect("remove retired golden encoding");
    }
    for shard in old_shards {
        let file = path.parent().unwrap().join(shard);
        if !files.contains_key(&file) {
            fs::remove_file(file).expect("remove retired golden shard");
        }
    }
}

// Sum only versioned golden directories, excluding all library and media contents.
fn golden_size(rust_dir: &Path) -> u64 {
    fn size(path: &Path) -> u64 {
        fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| {
                let metadata = entry.metadata().expect("golden file metadata");
                if metadata.is_dir() {
                    size(&entry.path())
                } else {
                    metadata.len()
                }
            })
            .sum()
    }
    fs::read_dir(rust_dir.join("crates"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| size(&entry.path().join("tests/golden")))
        .sum()
}

// Replace a single file through a same-directory temporary file.
fn atomic_write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).expect("create golden directory");
    let temporary = path.with_extension(format!("json.pending-{}", std::process::id()));
    let mut file = fs::File::create(&temporary).expect("create temporary golden");
    file.write_all(bytes).expect("write golden");
    file.sync_all().expect("sync golden");
    fs::rename(temporary, path).expect("replace golden atomically");
}

// Read only Git's revision through a read-only local command during recording.
fn revision(rust_dir: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(rust_dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

// Synthetic support tests can record without legacy application provenance.
fn oracle_sources(crate_dir: &Path, rust_dir: &Path) -> BTreeMap<String, FileDigest> {
    let _ = (crate_dir, rust_dir);
    BTreeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Harness {
        root: PathBuf,
        crate_dir: PathBuf,
        fixtures: FixtureSnapshot,
    }

    impl Harness {
        // Build an isolated synthetic checkout on disk, never modifying real fixtures.
        fn new() -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../target/tmp/mm-test-support")
                .join(format!(
                    "{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(root.join("crates/sample/tests")).unwrap();
            fs::write(root.join("PORTING.md"), "synthetic support test workspace").unwrap();
            fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
            let root = root.canonicalize().unwrap();
            let crate_dir = root.join("crates/sample");
            fs::write(
                crate_dir.join("tests/cases.rs"),
                "fn frozen_case_recipe() {}\n",
            )
            .unwrap();
            fs::create_dir_all(root.join("test-fixtures/hosted-bucket")).unwrap();
            let source = root.join("synthetic-source.db");
            fs::write(&source, b"immutable synthetic database source").unwrap();
            for name in ["mymind.db", "hosted-test.db"] {
                fs::copy(&source, root.join("test-fixtures").join(name)).unwrap();
            }
            fs::write(
                root.join("test-fixtures/hosted-bucket/object"),
                b"synthetic object",
            )
            .unwrap();
            let fixtures = fixture_snapshot(&root, true);
            Self {
                root,
                crate_dir,
                fixtures,
            }
        }

        // Reset invocation counters to model a new Rust test binary for replay.
        fn restart(&self) {
            let path = self.crate_dir.join("tests/golden/cases.json");
            let prefix = format!("{}\0", path.display());
            OCCURRENCES
                .get_or_init(Default::default)
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retain(|key, _| !key.starts_with(&prefix));
            SEEN.get_or_init(Default::default)
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&session_key(&path, &test_name()));
        }

        // Open an isolated batch without consulting process environment fixture overrides.
        fn batch(&self, namespace: &str, requests: &[Vec<u8>], record_mode: bool) -> Batch {
            Batch::open(
                &self.crate_dir,
                "cases",
                namespace,
                requests,
                record_mode,
                Some(self.fixtures.clone()),
            )
        }

        // Record a complete synthetic batch through the same comparison and writer path.
        fn record(&self, requests: &[Vec<u8>], answers: &[&[u8]]) {
            assert_eq!(requests.len(), answers.len());
            let mut batch = self.batch("operation", requests, true);
            for (index, answer) in answers.iter().enumerate() {
                batch.check(index, answer, Some(answer));
            }
            batch.finish();
            self.restart();
        }

        // Find the one target manifest used by this synthetic checkout.
        fn golden(&self) -> PathBuf {
            self.crate_dir.join("tests/golden/cases.json")
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    // Extract panic messages without depending on Rust's panic formatting wrapper.
    fn panic_text(action: impl FnOnce()) -> String {
        let panic =
            catch_unwind(AssertUnwindSafe(action)).expect_err("expected golden assertion failure");
        panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
            .unwrap_or_default()
    }

    #[test]
    fn replay_is_read_only_and_stores_no_private_request_or_answer_text() {
        let harness = Harness::new();
        let requests = vec![b"synthetic private request sentinel".to_vec()];
        let answer = b"synthetic private answer sentinel";
        harness.record(&requests, &[answer]);
        let stored = stored_path(&harness.golden()).unwrap();
        let before = fs::read(&stored).unwrap();
        let modified = fs::metadata(&stored).unwrap().modified().unwrap();
        let text = String::from_utf8(inflate(&stored, &before)).unwrap();
        assert!(
            !text.contains("private request sentinel") && !text.contains("private answer sentinel")
        );
        let mut replay = harness.batch("operation", &requests, false);
        assert!(replay.inline_answer(0).is_none());
        replay.check(0, answer, None);
        replay.finish();
        assert_eq!(fs::read(&stored).unwrap(), before);
        assert_eq!(fs::metadata(&stored).unwrap().modified().unwrap(), modified);
    }

    #[test]
    fn recording_compares_before_any_golden_or_fixture_manifest_write() {
        let harness = Harness::new();
        let message = panic_text(|| {
            let mut batch = harness.batch("operation", &[b"request".to_vec()], true);
            batch.check(0, b"different", Some(b"expected"));
        });
        assert!(
            message.contains("expected length 8 sha256")
                && message.contains("actual length 9 sha256")
        );
        assert!(
            message.contains("first difference at byte 0")
                && message.contains("MYMIND_RECORD_GOLDEN=1")
        );
        assert!(stored_path(&harness.golden()).is_none());
        assert!(!harness.fixtures.path.exists());
    }

    #[test]
    fn digest_only_mismatch_names_case_and_explains_missing_difference_offset() {
        let harness = Harness::new();
        let requests = vec![b"request".to_vec()];
        harness.record(&requests, &[b"expected"]);
        let message = panic_text(|| {
            let mut batch = harness.batch("operation", &requests, false);
            batch.check(0, b"changed", None);
        });
        assert!(message.contains("cases test") && message.contains(" case "));
        assert!(
            message.contains("expected length 8 sha256")
                && message.contains("actual length 7 sha256")
        );
        assert!(
            message.contains("actual answer first bytes")
                && message.contains("first differing offset cannot be located")
        );
    }

    #[test]
    fn recording_mismatch_uses_normalized_batch_identity_and_bounded_diagnostics() {
        let harness = Harness::new();
        let namespace = "synthetic recording mismatch";
        let request = format!(
            "{}/../../.. {}/target/tmp/fixture-copies/123-4/mymind.db",
            harness.crate_dir.display(),
            harness.root.display()
        )
        .into_bytes();
        let mut batch = harness.batch(namespace, std::slice::from_ref(&request), true);
        let id = batch.ids[0].clone();
        assert_ne!(id, case_id(namespace, &request));
        batch.finished = true;
        drop(batch);
        let actual = b"prefix!actual";
        let expected = b"prefix?expected";
        let message = panic_text(|| {
            recording_mismatch(
                harness.crate_dir.to_str().unwrap(),
                "cases",
                namespace,
                &request,
                actual,
                expected,
            );
        });
        assert!(message.contains(&format!(
            "cases test {} namespace {namespace} case {id}",
            test_name()
        )));
        assert!(message.contains(&format!(
            "expected length {} sha256 {}",
            expected.len(),
            digest(expected)
        )));
        assert!(message.contains(&format!(
            "actual length {} sha256 {}",
            actual.len(),
            digest(actual)
        )));
        assert!(message.contains("first difference at byte 6"));
        assert!(message.contains(&format!(
            "actual differing prefix bytes {:02x?}",
            &actual[6..]
        )));
        assert!(message.contains(REGENERATE));
        assert!(stored_path(&harness.golden()).is_none() && !harness.fixtures.path.exists());
    }

    #[test]
    fn missing_extra_reordered_and_changed_request_cases_fail() {
        let harness = Harness::new();
        let requests = vec![b"one".to_vec(), b"two".to_vec()];
        harness.record(&requests, &[b"a", b"b"]);
        assert!(
            panic_text(|| {
                let _ = harness.batch("operation", &requests[..1], false);
            })
            .contains("changed case count")
        );
        harness.restart();
        assert!(
            panic_text(|| {
                let _ = harness.batch(
                    "operation",
                    &[requests[1].clone(), requests[0].clone()],
                    false,
                );
            })
            .contains("changed/reordered request corpus")
        );
        harness.restart();
        assert!(
            panic_text(|| {
                let _ = harness.batch(
                    "operation",
                    &[requests[0].clone(), b"three".to_vec()],
                    false,
                );
            })
            .contains("changed/reordered request corpus")
        );
        harness.restart();
        assert!(
            panic_text(|| {
                let mut batch = harness.batch("operation", &requests, false);
                batch.check(0, b"a", None);
                batch.finish();
            })
            .contains("missing comparisons")
        );
        harness.restart();
        assert!(
            panic_text(|| {
                let mut batch = harness.batch("operation", &requests, false);
                batch.check(2, b"extra", None);
            })
            .contains("extra case index 2")
        );
        harness.restart();
        assert!(
            panic_text(|| {
                let mut batch = harness.batch("operation", &requests, false);
                batch.check(0, b"a", None);
                batch.check(0, b"a", None);
            })
            .contains("checked twice")
        );
    }

    #[test]
    fn recording_inventory_rejects_a_batch_lost_before_finish() {
        let harness = Harness::new();
        let mut batch = harness.batch("operation", &[b"request".to_vec()], true);
        batch.check(0, b"answer", Some(b"answer"));
        batch.finish();
        let guard = TestGuard {
            path: harness.golden(),
            test: test_name(),
            expected: BTreeSet::new(),
            recording: true,
            finished: false,
        };
        let mut target = load_target(&harness.golden()).unwrap();
        target.batches.clear();
        save_target(&harness.root, &harness.golden(), target, &harness.fixtures);
        assert!(panic_text(|| guard.finish()).contains("missing/extra recorded batches"));
    }

    #[test]
    fn changed_source_and_orphan_batch_inventory_fail() {
        let harness = Harness::new();
        let requests = vec![b"request".to_vec()];
        harness.record(&requests, &[b"answer"]);
        let target = load_target(&harness.golden()).unwrap();
        let guard = TestGuard {
            path: harness.golden(),
            test: test_name(),
            expected: target.batches.keys().cloned().collect(),
            recording: false,
            finished: false,
        };
        assert!(panic_text(|| guard.finish()).contains("missing/extra batches"));
        fs::write(
            harness.crate_dir.join("tests/cases.rs"),
            "fn removed_original_case() {}\n",
        )
        .unwrap();
        assert!(
            panic_text(|| {
                let _ = harness.batch("operation", &requests, false);
            })
            .contains("test source changed")
        );
    }

    #[test]
    fn synthetic_inline_is_opt_in_utf8_and_limited_to_1024_bytes() {
        let harness = Harness::new();
        let requests = vec![b"small".to_vec(), b"large".to_vec(), b"binary".to_vec()];
        let large = vec![b'x'; INLINE_LIMIT + 1];
        let mut batch = harness.batch("operation", &requests, true);
        batch.allow_inline_synthetic();
        batch.check(0, b"small synthetic", Some(b"small synthetic"));
        batch.check(1, &large, Some(&large));
        batch.check(2, &[0xff], Some(&[0xff]));
        batch.finish();
        harness.restart();
        let mut replay = harness.batch("operation", &requests, false);
        assert_eq!(replay.inline_answer(0), Some(b"small synthetic".as_slice()));
        assert!(replay.inline_answer(1).is_none() && replay.inline_answer(2).is_none());
        replay.check(0, b"small synthetic", None);
        replay.check(1, &large, None);
        replay.check(2, &[0xff], None);
        replay.finish();
    }

    #[test]
    fn corrupting_a_fixture_copy_fails_with_file_fingerprints() {
        let harness = Harness::new();
        harness.record(&[b"request".to_vec()], &[b"answer"]);
        let original = fs::read(harness.root.join("synthetic-source.db")).unwrap();
        let mut corrupted = original.clone();
        corrupted[0] ^= 1;
        fs::write(harness.root.join("test-fixtures/hosted-test.db"), corrupted).unwrap();
        let message = panic_text(|| {
            refresh_fixtures(&harness.fixtures, false);
        });
        assert!(
            message.contains("hosted-test.db changed")
                && message.contains("expected length")
                && message.contains("actual length")
        );
        assert_eq!(
            fs::read(harness.root.join("synthetic-source.db")).unwrap(),
            original
        );
    }

    #[test]
    fn guard_rejects_a_corrupt_fixture_before_the_caller_reads_it() {
        let harness = Harness::new();
        harness.record(&[b"request".to_vec()], &[b"answer"]);
        let copied_database = harness.root.join("test-fixtures/mymind.db");
        let length = fs::metadata(&copied_database).unwrap().len() as usize;
        fs::write(&copied_database, vec![b'X'; length]).unwrap();
        let consumed = std::cell::Cell::new(false);
        let message = panic_text(|| {
            let guard = TestGuard::open(&harness.crate_dir, "cases", false, false);
            consumed.set(true);
            guard.finish();
        });
        assert!(!consumed.get());
        assert!(
            message.contains("mymind.db changed")
                && message.contains("expected length")
                && message.contains("actual length")
                && message.matches("sha256").count() == 2
        );
    }

    #[test]
    fn generated_input_registration_rejects_changed_and_missing_inputs() {
        let harness = Harness::new();
        let generated = harness.root.join("test-fixtures/generated");
        fs::write(&generated, b"generated bytes").unwrap();
        let requests = vec![b"request".to_vec()];
        let mut record = harness.batch("operation", &requests, true);
        record.register_input(&generated);
        record.check(0, b"answer", Some(b"answer"));
        record.finish();
        harness.restart();
        assert!(
            panic_text(|| {
                let mut batch = harness.batch("operation", &requests, false);
                batch.check(0, b"answer", None);
                batch.finish();
            })
            .contains("missing frozen input registrations")
        );
        harness.restart();
        fs::write(&generated, b"changed bytes").unwrap();
        assert!(
            panic_text(|| {
                let mut batch = harness.batch("operation", &requests, false);
                batch.register_input(&generated);
            })
            .contains("frozen input <fixtures>/generated changed")
        );
    }

    #[test]
    fn normalization_preserves_binary_literal_ports_and_copy_alias_relationships() {
        let harness = Harness::new();
        let literal = b"\xffhttp://localhost:7/path\0";
        assert_eq!(normalize(literal, &harness.root), literal);
        register_loopback_port(54321);
        register_loopback_port(54322);
        let path = harness.crate_dir.join("../../..").join("capture/body");
        let text = format!("{} http://localhost:54321/path", path.display());
        assert_eq!(
            String::from_utf8(normalize(text.as_bytes(), &harness.root)).unwrap(),
            "<repository>/capture/body http://localhost:<loopback-port:0>/path"
        );
        let first = format!(
            "{}/target/tmp/fixture-copies/123-4/mymind.db",
            harness.root.display()
        );
        let second = format!(
            "{}/target/tmp/fixture-copies/987-6/mymind.db",
            harness.root.display()
        );
        assert_eq!(
            normalize(first.as_bytes(), &harness.root),
            normalize(second.as_bytes(), &harness.root)
        );
        assert_ne!(
            normalize(format!("{first} {first}").as_bytes(), &harness.root),
            normalize(format!("{first} {second}").as_bytes(), &harness.root)
        );
        assert_ne!(
            normalize(
                b"http://localhost:54321 http://localhost:54321",
                &harness.root
            ),
            normalize(
                b"http://localhost:54321 http://localhost:54322",
                &harness.root
            )
        );
        assert_eq!(
            normalize(
                b"http://localhost:54321 http://localhost:54322",
                &harness.root
            ),
            normalize(
                b"http://localhost:54322 http://localhost:54321",
                &harness.root
            )
        );
        let mut ports = BTreeMap::new();
        let mut copies = BTreeMap::new();
        let one = normalize_with_aliases(first.as_bytes(), &harness.root, &mut ports, &mut copies);
        let same = normalize_with_aliases(first.as_bytes(), &harness.root, &mut ports, &mut copies);
        let other =
            normalize_with_aliases(second.as_bytes(), &harness.root, &mut ports, &mut copies);
        assert_eq!(one, same);
        assert_ne!(one, other);
        let one = normalize_with_aliases(
            b"http://localhost:54321",
            &harness.root,
            &mut ports,
            &mut copies,
        );
        let same = normalize_with_aliases(
            b"http://localhost:54321",
            &harness.root,
            &mut ports,
            &mut copies,
        );
        let other = normalize_with_aliases(
            b"http://localhost:54322",
            &harness.root,
            &mut ports,
            &mut copies,
        );
        assert_eq!(one, same);
        assert_ne!(one, other);
        assert_ne!(case_id("ab", b"c"), case_id("a", b"bc"));
    }

    #[test]
    fn golden_budget_is_checked_before_writing_any_output() {
        let harness = Harness::new();
        let other = harness.root.join("crates/other/tests/golden/existing.json");
        fs::create_dir_all(other.parent().unwrap()).unwrap();
        fs::File::create(&other)
            .unwrap()
            .set_len(TOTAL_LIMIT)
            .unwrap();
        let message = panic_text(|| {
            let mut batch = harness.batch("operation", &[b"request".to_vec()], true);
            batch.check(0, b"answer", Some(b"answer"));
            batch.finish();
        });
        assert!(message.contains("50 MB budget; no files were written"));
        assert!(stored_path(&harness.golden()).is_none() && !harness.fixtures.path.exists());
    }

    #[test]
    fn gzip_is_deterministic_and_requires_a_quarter_savings() {
        let (path, bytes) = stored_bytes(Path::new("target.json"), vec![b'x'; 10_000]);
        assert_eq!(path, Path::new("target.json.gz"));
        assert!(bytes.len() * 4 <= 10_000 * 3);
        assert_eq!(
            stored_bytes(Path::new("target.json"), vec![b'x'; 10_000]).1,
            bytes
        );
        assert_eq!(inflate(&path, &bytes), vec![b'x'; 10_000]);
        assert_eq!(
            stored_bytes(Path::new("tiny.json"), b"{}".to_vec()).0,
            Path::new("tiny.json")
        );
    }

    #[cfg(unix)]
    #[test]
    fn input_symlinks_are_rejected_before_opening_a_secret_target() {
        let harness = Harness::new();
        let alias = harness.root.join("innocent-input");
        std::os::unix::fs::symlink(harness.root.join(".env"), &alias).unwrap();
        assert!(
            panic_text(|| {
                let _ = fingerprint(&alias);
            })
            .contains("never a symlink")
        );
        assert!(
            panic_text(|| {
                let _ = read_input(&alias);
            })
            .contains("never a symlink")
        );
        for name in [".env", ".dev.vars", ".env/child"] {
            assert!(
                panic_text(|| {
                    let _ = read_input(&harness.root.join(name));
                })
                .contains("secret files cannot be golden inputs")
            );
        }
    }

    #[test]
    fn large_targets_shard_in_sorted_order_and_extra_shards_fail() {
        let harness = Harness::new();
        let cases = (0..60_000)
            .map(|index| {
                (
                    digest(format!("synthetic case {index}").as_bytes()),
                    expected_bytes(b"answer", false),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut target = Target {
            version: VERSION,
            normalization_version: VERSION,
            fixtures_sha256: harness.fixtures.sha256.clone(),
            source_sha256: source_digest(&harness.crate_dir, "cases"),
            batches: BTreeMap::from([(
                "large".into(),
                BatchRecord {
                    count: cases.len(),
                    cases,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(encode(&target).len() > SHARD_LIMIT);
        save_target_with_limit(
            &harness.root,
            &harness.golden(),
            &mut target,
            &harness.fixtures,
            250_000,
        );
        let replay = load_target(&harness.golden()).unwrap();
        assert!(!replay.shards.is_empty());
        assert_eq!(replay.batches["large"].cases.len(), 60_000);
        assert!(
            replay
                .shards
                .iter()
                .all(|shard| shard.len <= SHARD_LIMIT as u64)
        );
        let first_path = harness
            .golden()
            .parent()
            .unwrap()
            .join(&replay.shards[0].file);
        let first = inflate(&first_path, &fs::read(&first_path).unwrap());
        let ids = first
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| decode::<ShardCase>(line, &harness.golden()).id)
            .collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        fs::write(
            harness
                .golden()
                .parent()
                .unwrap()
                .join("cases.part-9999.jsonl"),
            b"extra\n",
        )
        .unwrap();
        assert!(
            panic_text(|| {
                let _ = load_target(&harness.golden());
            })
            .contains("missing or extra target shards")
        );
    }
}
