//! Small bridges between JavaScript values and mm-core's types.

use js_sys::{Array, Function, Reflect};
use mm_core::Error;
use wasm_bindgen::{JsCast, JsValue};

/// The error mm-core reports for a thrown JavaScript value: its `message`, as the JS Worker's `error.message`.
pub fn js_error(value: JsValue) -> Error {
    let message = Reflect::get(&value, &"message".into()).ok().and_then(|message| message.as_string());
    Error::internal(message.or_else(|| value.as_string()).unwrap_or_else(|| format!("{value:?}")))
}

/// A property of a JavaScript object.
pub fn get(target: &JsValue, key: &str) -> Result<JsValue, Error> {
    Reflect::get(target, &key.into()).map_err(js_error)
}

/// Call `target.method(...args)`.
pub fn call(target: &JsValue, method: &str, args: &[JsValue]) -> Result<JsValue, Error> {
    let function: Function = get(target, method)?.dyn_into().map_err(|_| Error::internal(format!("{method} is not a function")))?;
    let list = Array::new();
    for arg in args {
        list.push(arg);
    }
    function.apply(target, &list).map_err(js_error)
}

/// The error a `worker` call failed with, as mm-core reports it.
pub fn worker_error(error: worker::Error) -> Error {
    Error::internal(error.to_string())
}
