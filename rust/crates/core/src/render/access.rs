//! JS property access and coercion over card JSON, shared by the render ports.

use mm_js::Value;

use crate::{Error, Result};

/// `String(value ?? '')`: the text an `escapeHtml` argument becomes.
pub(crate) fn text(value: &Value) -> String {
    if value.is_nullish() { String::new() } else { value.to_js_string() }
}

/// `value.key`, which throws a `TypeError` on `null` and `undefined`.
pub(crate) fn member<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    if value.is_nullish() {
        return Err(Error::invalid(format!("Cannot read properties of {} (reading '{key}')", value.to_js_string())));
    }
    Ok(value.get(key))
}

/// `value?.key`.
pub(crate) fn optional<'a>(value: &'a Value, key: &str) -> &'a Value {
    if value.is_nullish() { &mm_js::UNDEFINED } else { value.get(key) }
}

/// The text of the member `Object.prototype` gives every plain object under `key` (`{}[key]`), if it has one.
pub(crate) fn prototype_member(key: &str) -> Option<String> {
    match key {
        "__proto__" => Some("[object Object]".into()),
        "constructor" => Some("function Object() { [native code] }".into()),
        "__defineGetter__" | "__defineSetter__" | "__lookupGetter__" | "__lookupSetter__" | "hasOwnProperty" | "isPrototypeOf"
        | "propertyIsEnumerable" | "toLocaleString" | "toString" | "valueOf" => Some(format!("function {key}() {{ [native code] }}")),
        _ => None,
    }
}

/// `Number.prototype` ToInt32 as `value & mask` applies it.
pub(crate) fn to_int32(number: f64) -> i32 {
    if !number.is_finite() {
        return 0;
    }
    (number.trunc().rem_euclid(4_294_967_296.0) as u64 as u32) as i32
}

/// `Math.min` over two numbers: NaN wins.
pub(crate) fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}
