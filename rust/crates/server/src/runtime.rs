//! The library code's runtime on Tokio: background tasks on the current `LocalSet`, timers, and the log on standard
//! output as `console.log` wrote it.

use std::io::Write;
use std::time::Duration;

use mm_core::runtime::{Runtime, Task};

/// Tokio's current-thread scheduler as the library code's [`Runtime`].
pub struct Tokio;

impl Runtime for Tokio {
    fn spawn(&self, task: Task) {
        tokio::task::spawn_local(task);
    }

    fn sleep(&self, ms: u64) -> Task {
        Box::pin(tokio::time::sleep(Duration::from_millis(ms)))
    }

    fn log(&self, line: &str) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{line}");
    }
}

/// Install [`Tokio`] as the runtime and run `main` on a current-thread runtime inside a `LocalSet`.
///
/// # Arguments
/// * `main` - The program.
///
/// # Returns
/// What `main` returns.
///
/// # Panics
/// When the runtime cannot start.
pub fn run<F: Future>(main: F) -> F::Output {
    mm_core::runtime::install(&Tokio);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("the Tokio runtime starts");
    tokio::task::LocalSet::new().block_on(&runtime, main)
}
