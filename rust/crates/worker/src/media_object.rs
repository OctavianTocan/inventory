//! The `Media` Durable Object and its container (`worker/media-object.mjs`).

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker::{State, durable_object};

use crate::util::{call, get, js_error};

const PORT: u16 = 8080;
const START_TIMEOUT_MS: f64 = 60_000.0;
const IDLE_MS: f64 = 3.0 * 60_000.0;

/// The Rust media container (`rust/container/`): image and video renditions, PDF previews, page screenshots and validated
/// fetches. One instance serves every library.
#[durable_object]
pub struct Media {
    state: State,
    started: Rc<Cell<bool>>,
}

impl Media {
    fn container(&self) -> Result<JsValue, worker::Error> {
        let container = get(self.state.as_raw().as_ref(), "container").map_err(|error| worker::Error::RustError(error.to_string()))?;
        if container.is_undefined() || container.is_null() {
            return Err(worker::Error::RustError("This Durable Object has no container".into()));
        }
        Ok(container)
    }

    async fn ready(&self) -> Result<JsValue, worker::Error> {
        let container = self.container()?;
        let fail = |error: JsValue| worker::Error::RustError(js_error(error).to_string());
        let running = get(&container, "running").ok().and_then(|value| value.as_bool()).unwrap_or(false);
        if !running {
            self.started.set(false);
            let options = Object::new();
            Reflect::set(&options, &"enableInternet".into(), &JsValue::TRUE).map_err(fail)?;
            call(&container, "start", &[options.into()]).map_err(|error| worker::Error::RustError(error.to_string()))?;
            let monitor = call(&container, "monitor", &[]).map_err(|error| worker::Error::RustError(error.to_string()))?;
            let started = self.started.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(error) = JsFuture::from(js_sys::Promise::from(monitor)).await {
                    web_sys::console::error_2(&"[media] container stopped".into(), &error);
                }
                started.set(false);
            });
        }
        if !self.started.get() {
            self.answering(&container).await?;
            self.started.set(true);
        }
        Ok(container)
    }

    async fn answering(&self, container: &JsValue) -> Result<(), worker::Error> {
        let deadline = js_sys::Date::now() + START_TIMEOUT_MS;
        let port = call(container, "getTcpPort", &[f64::from(PORT).into()]).map_err(|error| worker::Error::RustError(error.to_string()))?;
        loop {
            let attempt = call(&port, "fetch", &["http://container/health".into()]).map_err(|error| worker::Error::RustError(error.to_string()))?;
            match JsFuture::from(js_sys::Promise::from(attempt)).await {
                Ok(response) if response.dyn_ref::<web_sys::Response>().is_some_and(web_sys::Response::ok) => break,
                Err(error) if js_sys::Date::now() > deadline => return Err(worker::Error::RustError(js_error(error).to_string())),
                _ => {}
            }
            if js_sys::Date::now() > deadline {
                return Err(worker::Error::RustError("The media container did not start".into()));
            }
            worker::Delay::from(Duration::from_millis(250)).await;
        }
        let idle = call(container, "setInactivityTimeout", &[IDLE_MS.into()]);
        if let Ok(promise) = idle
            && let Err(error) = JsFuture::from(js_sys::Promise::from(promise)).await
        {
            web_sys::console::warn_2(&"[media] inactivity timeout not set".into(), &error);
        }
        Ok(())
    }
}

impl worker::DurableObject for Media {
    fn new(state: State, env: worker::Env) -> Self {
        crate::runtime::configure(&env);
        Self { state, started: Rc::new(Cell::new(false)) }
    }

    async fn fetch(&self, req: worker::Request) -> worker::Result<worker::Response> {
        let container = self.ready().await?;
        let port = call(&container, "getTcpPort", &[f64::from(PORT).into()]).map_err(|error| worker::Error::RustError(error.to_string()))?;
        let incoming: &JsValue = req.inner().as_ref();
        let answer = call(&port, "fetch", std::slice::from_ref(incoming)).map_err(|error| worker::Error::RustError(error.to_string()))?;
        let response = JsFuture::from(js_sys::Promise::from(answer)).await.map_err(|error| worker::Error::RustError(js_error(error).to_string()))?;
        Ok(response.unchecked_into::<web_sys::Response>().into())
    }
}
