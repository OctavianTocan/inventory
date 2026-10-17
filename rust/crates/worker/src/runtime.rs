//! The isolate's scheduler, log and configuration for mm-core.

use std::cell::Cell;
use std::collections::HashMap;
use std::time::Duration;

use js_sys::Object;
use mm_core::runtime::{self, Runtime, Task};
use wasm_bindgen::{JsCast, JsValue};

struct IsolateRuntime;

impl Runtime for IsolateRuntime {
    fn spawn(&self, task: Task) {
        wasm_bindgen_futures::spawn_local(task);
    }

    fn sleep(&self, ms: u64) -> Task {
        Box::pin(async move {
            worker::Delay::from(Duration::from_millis(ms)).await;
        })
    }

    fn log(&self, line: &str) {
        web_sys::console::log_1(&line.into());
    }
}

static RUNTIME: IsolateRuntime = IsolateRuntime;

thread_local! {
    static CONFIGURED: Cell<bool> = const { Cell::new(false) };
}

/// Install the isolate's runtime and feed mm-core the Worker's variables and secrets, once per isolate.
///
/// # Arguments
/// * `env` - The Worker's `env`; its string properties become mm-core's environment.
pub fn configure(env: &worker::Env) {
    if CONFIGURED.replace(true) {
        return;
    }
    runtime::install(&RUNTIME);
    runtime::set_env(strings(env.as_ref()));
}

/// The string properties of the Worker's `env`: its variables and secrets, not its bindings.
pub fn strings(env: &JsValue) -> HashMap<String, String> {
    let Some(object) = env.dyn_ref::<Object>() else { return HashMap::new() };
    Object::entries(object).iter().filter_map(|entry| {
        let pair = js_sys::Array::from(&entry);
        Some((pair.get(0).as_string()?, pair.get(1).as_string()?))
    }).collect()
}
