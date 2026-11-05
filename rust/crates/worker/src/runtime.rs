//! The isolate's scheduler, log and configuration for mm-core.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Duration;

use js_sys::Object;
use mm_core::runtime::{self, Runtime, Task};
use wasm_bindgen::{JsCast, JsValue};

struct IsolateRuntime;

// Secrets the Durable Object must see even when `Object.entries` skips a non-enumerable binding.
const KNOWN: &[&str] = &[
    "GEMINI_API_KEY",
    "AI_CHAT_API_KEY",
    "AI_CHAT_MODEL",
    "TYPESAFE_API_KEY",
    "GEMINI_BASE_URL",
    "AI_CHAT_BASE_URL",
    "TYPESAFE_BASE_URL",
    "OWNER_EMAIL",
    "FEEDBACK_OPEN",
    "ALLOWED_GOOGLE_DOMAIN",
    "AUTH",
    "PUBLIC_URL",
    "ADMIN_TOKEN",
    "OFFLINE",
    "STORE_READONLY",
    "GOOGLE_CLIENT_ID",
    "GOOGLE_CLIENT_SECRET",
];

thread_local! {
    // Owned JS state for `waitUntil`. `DurableObjectState` is not `Clone`; the `JsValue` is.
    static KEEPALIVE: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

impl Runtime for IsolateRuntime {
    fn spawn(&self, task: Task) {
        wasm_bindgen_futures::spawn_local(task);
    }

    fn defer(&self, task: Task) {
        // Copy the handle out before `waitUntil`, which polls the task and must not re-enter this borrow.
        let raw = KEEPALIVE.with(|slot| slot.borrow().clone());
        if let Some(raw) = raw {
            let state = worker::worker_sys::DurableObjectState::unchecked_from_js(raw);
            worker::State::from(state).wait_until(task);
        } else {
            self.spawn(task);
        }
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
/// * `env` - The Worker's `env`; its string properties and known secret bindings become mm-core's environment.
pub fn configure(env: &worker::Env) {
    if CONFIGURED.replace(true) {
        return;
    }
    runtime::install(&RUNTIME);
    runtime::set_env(bindings(env));
}

/// Keep `task` alive after a Durable Object response. Without this, `spawn_local` is cancelled when `fetch` returns
/// and a save's summary never lands.
///
/// # Arguments
/// * `state` - The object's state. `waitUntil` holds the task.
pub fn bind_keepalive(state: &worker::State) {
    let raw = <worker::worker_sys::DurableObjectState as AsRef<JsValue>>::as_ref(state.as_raw()).clone();
    KEEPALIVE.with(|slot| *slot.borrow_mut() = Some(raw));
}

/// String variables plus known secrets read by name, so a non-enumerable secret still configures AI and sign-in.
///
/// # Arguments
/// * `env` - The Worker's `env`.
pub fn bindings(env: &worker::Env) -> HashMap<String, String> {
    let mut vars = strings(env.as_ref());
    for name in KNOWN {
        if vars.get(*name).is_some_and(|value| !value.is_empty()) {
            continue;
        }
        let value = env.var(name).or_else(|_| env.secret(name)).ok().map(|binding| binding.to_string());
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            vars.insert((*name).to_owned(), value);
        }
    }
    vars
}

/// The string properties of the Worker's `env`: its variables and secrets, not its bindings.
pub fn strings(env: &JsValue) -> HashMap<String, String> {
    let Some(object) = env.dyn_ref::<Object>() else { return HashMap::new() };
    Object::entries(object).iter().filter_map(|entry| {
        let pair = js_sys::Array::from(&entry);
        Some((pair.get(0).as_string()?, pair.get(1).as_string()?))
    }).collect()
}
