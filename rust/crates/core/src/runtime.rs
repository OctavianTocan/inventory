//! What the JS backend took from its runtime's globals: background tasks, timers, logging, environment variables and
//! randomness. The host installs a [`Runtime`] once at start.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{OnceLock, RwLock};

/// A future the runtime runs in the background.
pub type Task = Pin<Box<dyn Future<Output = ()>>>;

/// The host's scheduler.
pub trait Runtime: Sync {
    /// Run `task` in the background on the current thread, as a promise nobody awaits.
    ///
    /// # Arguments
    /// * `task` - Work to run to completion.
    fn spawn(&self, task: Task);

    /// A future that completes after `ms` milliseconds (`setTimeout`).
    ///
    /// # Arguments
    /// * `ms` - Delay in milliseconds.
    fn sleep(&self, ms: u64) -> Task;

    /// Write one line to the server log (`console.log`).
    ///
    /// # Arguments
    /// * `line` - The line, without its newline.
    fn log(&self, line: &str);
}

static RUNTIME: OnceLock<&'static dyn Runtime> = OnceLock::new();
static ENV: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

/// Install the host's runtime; later calls are ignored.
///
/// # Arguments
/// * `runtime` - Scheduler and log for the life of the process or isolate.
pub fn install(runtime: &'static dyn Runtime) {
    let _ = RUNTIME.set(runtime);
}

fn runtime() -> &'static dyn Runtime {
    *RUNTIME.get().expect("the host installs a runtime before serving")
}

/// Run `task` in the background.
///
/// # Arguments
/// * `task` - Work to run to completion.
pub fn spawn(task: impl Future<Output = ()> + 'static) {
    runtime().spawn(Box::pin(task));
}

/// Wait `ms` milliseconds.
///
/// # Arguments
/// * `ms` - Delay in milliseconds.
pub async fn sleep(ms: u64) {
    runtime().sleep(ms).await;
}

/// Run `work` until it finishes or `ms` milliseconds pass, as an `AbortSignal.timeout(ms)` bounds a fetch.
///
/// # Arguments
/// * `ms` - Time limit in milliseconds.
/// * `work` - The work to bound.
///
/// # Returns
/// `None` when the time ran out first.
pub async fn timeout<T>(ms: u64, work: impl Future<Output = T>) -> Option<T> {
    use futures_util::future::{Either, select};
    match select(std::pin::pin!(work), std::pin::pin!(sleep(ms))).await {
        Either::Left((value, _)) => Some(value),
        Either::Right(_) => None,
    }
}

/// Write one line to the server log; without a runtime (in tests) the line goes to standard error.
///
/// # Arguments
/// * `line` - The line, without its newline.
pub fn log(line: &str) {
    match RUNTIME.get() {
        Some(runtime) => runtime.log(line),
        None => eprintln!("{line}"),
    }
}

/// Replace the environment variables the code reads (`process.env`, or the Worker's vars and secrets).
///
/// # Arguments
/// * `vars` - Every variable by name.
pub fn set_env(vars: HashMap<String, String>) {
    let lock = ENV.get_or_init(Default::default);
    *lock.write().expect("environment lock") = vars;
}

/// An environment variable, as `process.env[name]` reads it; `None` when unset.
///
/// # Arguments
/// * `name` - Variable name.
pub fn env(name: &str) -> Option<String> {
    ENV.get()?.read().expect("environment lock").get(name).cloned()
}

/// `crypto.randomBytes(count)`.
///
/// # Arguments
/// * `count` - Number of bytes.
///
/// # Panics
/// If the platform has no random source, as `randomBytes` throws.
pub fn random_bytes(count: usize) -> Vec<u8> {
    let mut bytes = vec![0; count];
    getrandom::fill(&mut bytes).expect("the platform has a random source");
    bytes
}

/// `crypto.randomUUID()`: a version 4 UUID in lower-case hex.
pub fn random_uuid() -> String {
    let mut bytes = random_bytes(16);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = mm_js::base64::hex(&bytes);
    format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
}

/// `Date.now()`: milliseconds since the epoch.
pub fn now_ms() -> f64 {
    mm_js::date::now()
}
