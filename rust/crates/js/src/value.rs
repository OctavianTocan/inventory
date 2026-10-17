use std::collections::BTreeMap;
use std::fmt;

use indexmap::IndexMap;

/// A JavaScript value as the backend's JS code held it: what `JSON.parse` and the MessagePack decoder produce, and what
/// `JSON.stringify` and the encoder accept.
#[derive(Clone, Debug, Default)]
pub enum Value {
    /// `undefined`: dropped from objects by `JSON.stringify`, `null` inside arrays.
    #[default]
    Undefined,
    Null,
    Bool(bool),
    /// Every JS number, integers included.
    Number(f64),
    /// A `BigInt`; `JSON.stringify` refuses it.
    BigInt(i128),
    String(String),
    Array(Vec<Value>),
    Object(Object),
    /// A `Uint8Array`.
    Bytes(Vec<u8>),
    /// A `Date`, as milliseconds since the epoch; NaN for an invalid date.
    Date(f64),
    /// A MessagePack `ExtData`.
    Ext(i8, Vec<u8>),
}

/// `undefined`, for lookups that find nothing.
pub static UNDEFINED: Value = Value::Undefined;

/// An object's own properties in JS order: array-index keys ascending, then the others in insertion order.
#[derive(Clone, Default, PartialEq)]
pub struct Object {
    indexed: BTreeMap<u32, Value>,
    named: IndexMap<String, Value>,
}

/// The array index a property key names, if it is one (`"0"` to `"4294967294"`, no leading zeros).
pub fn array_index(key: &str) -> Option<u32> {
    let bytes = key.as_bytes();
    if bytes.is_empty() || bytes.len() > 10 || !bytes.iter().all(u8::is_ascii_digit) || (bytes[0] == b'0' && bytes.len() > 1) {
        return None;
    }
    let value: u64 = key.parse().ok()?;
    (value < u32::MAX as u64).then_some(value as u32)
}

impl Object {
    /// An empty object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of own properties.
    pub fn len(&self) -> usize {
        self.indexed.len() + self.named.len()
    }

    /// Whether the object has no properties.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The property's value, if the object has it.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match array_index(key) {
            Some(index) => self.indexed.get(&index),
            None => self.named.get(key),
        }
    }

    /// The property's value for changing in place, if the object has it.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match array_index(key) {
            Some(index) => self.indexed.get_mut(&index),
            None => self.named.get_mut(key),
        }
    }

    /// Whether the object has the property (`key in object` for own properties).
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Assign a property as `object[key] = value` does: an existing key keeps its place.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        let key = key.into();
        let value = value.into();
        match array_index(&key) {
            Some(index) => {
                self.indexed.insert(index, value);
            }
            None => {
                self.named.insert(key, value);
            }
        }
    }

    /// Delete a property as `delete object[key]` does, returning its value.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        match array_index(key) {
            Some(index) => self.indexed.remove(&index),
            None => self.named.shift_remove(key),
        }
    }

    /// Copy every property of `other` in, as `{...this, ...other}` does.
    pub fn extend(&mut self, other: &Object) {
        for (key, value) in other.iter() {
            self.insert(key.as_ref(), value.clone());
        }
    }

    /// Own properties in JS order.
    pub fn iter(&self) -> impl Iterator<Item = (std::borrow::Cow<'_, str>, &Value)> {
        self.indexed
            .iter()
            .map(|(index, value)| (std::borrow::Cow::Owned(index.to_string()), value))
            .chain(self.named.iter().map(|(key, value)| (std::borrow::Cow::Borrowed(key.as_str()), value)))
    }

    /// Own property names in JS order (`Object.keys` order).
    pub fn keys(&self) -> impl Iterator<Item = std::borrow::Cow<'_, str>> {
        self.iter().map(|(key, _)| key)
    }

    /// Own property values in JS order.
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.indexed.values().chain(self.named.values())
    }

    /// Own properties in JS order, for changing values in place.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (String, &mut Value)> {
        self.indexed
            .iter_mut()
            .map(|(index, value)| (index.to_string(), value))
            .chain(self.named.iter_mut().map(|(key, value)| (key.clone(), value)))
    }

    /// Keep only the properties `keep` accepts, in order.
    pub fn retain(&mut self, mut keep: impl FnMut(&str, &Value) -> bool) {
        self.indexed.retain(|index, value| keep(&index.to_string(), value));
        self.named.retain(|key, value| keep(key, value));
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Object {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut object = Object::new();
        for (key, value) in iter {
            object.insert(key, value);
        }
        object
    }
}

impl IntoIterator for Object {
    type Item = (String, Value);
    type IntoIter = Box<dyn Iterator<Item = (String, Value)>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.indexed.into_iter().map(|(index, value)| (index.to_string(), value)).chain(self.named))
    }
}

impl PartialEq for Value {
    /// Structural equality, with numbers compared as `===` compares them (NaN unequal, 0 equal to -0).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a.to_bits() == b.to_bits() || a == b,
            (Value::Ext(a, x), Value::Ext(b, y)) => a == b && x == y,
            _ => false,
        }
    }
}

impl Value {
    /// The property `key` as `value?.[key]` reads it: undefined when missing or when this is not an object.
    pub fn get(&self, key: &str) -> &Value {
        match self {
            Value::Object(object) => object.get(key).unwrap_or(&UNDEFINED),
            Value::Array(items) => match array_index(key) {
                Some(index) => items.get(index as usize).unwrap_or(&UNDEFINED),
                None => &UNDEFINED,
            },
            _ => &UNDEFINED,
        }
    }

    /// The element at `index` as `value?.[index]` reads it.
    pub fn at(&self, index: usize) -> &Value {
        match self {
            Value::Array(items) => items.get(index).unwrap_or(&UNDEFINED),
            Value::Object(object) => object.get(&index.to_string()).unwrap_or(&UNDEFINED),
            _ => &UNDEFINED,
        }
    }

    /// A property of an object for changing in place.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(object) => object.get_mut(key),
            _ => None,
        }
    }

    /// Assign a property, as `value[key] = item` does on an object; does nothing on other values.
    pub fn set(&mut self, key: impl Into<String>, item: impl Into<Value>) {
        if let Value::Object(object) = self {
            object.insert(key, item);
        }
    }

    /// Delete a property, as `delete value[key]` does.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        match self {
            Value::Object(object) => object.remove(key),
            _ => None,
        }
    }

    /// The text of a string value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(text),
            _ => None,
        }
    }

    /// The number of a number value.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(*number),
            _ => None,
        }
    }

    /// The boolean of a boolean value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// The items of an array value.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The items of an array value, for changing in place.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The properties of an object value.
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(object) => Some(object),
            _ => None,
        }
    }

    /// The properties of an object value, for changing in place.
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Value::Object(object) => Some(object),
            _ => None,
        }
    }

    /// Whether this is `undefined`.
    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    /// Whether this is `null` or `undefined` (what `??` and `?.` test).
    pub fn is_nullish(&self) -> bool {
        matches!(self, Value::Undefined | Value::Null)
    }

    /// Whether this is a plain object (not an array, null, bytes or a date).
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Whether this is an array (`Array.isArray`).
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Whether this is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Whether this is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// JS truthiness: false for undefined, null, false, 0, -0, NaN, `""` and `0n`.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Bool(value) => *value,
            Value::Number(number) => *number != 0.0 && !number.is_nan(),
            Value::BigInt(number) => *number != 0,
            Value::String(text) => !text.is_empty(),
            _ => true,
        }
    }

    /// `typeof value`.
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::BigInt(_) => "bigint",
            Value::String(_) => "string",
            _ => "object",
        }
    }

    /// `this ?? fallback`.
    pub fn or<'a>(&'a self, fallback: &'a Value) -> &'a Value {
        if self.is_nullish() { fallback } else { self }
    }

    /// `String(value)` for primitives, arrays and objects; dates print as their ISO text, which the backend never
    /// relies on.
    pub fn to_js_string(&self) -> String {
        match self {
            Value::Undefined => "undefined".into(),
            Value::Null => "null".into(),
            Value::Bool(value) => value.to_string(),
            Value::Number(number) => crate::number_to_string(*number),
            Value::BigInt(number) => number.to_string(),
            Value::String(text) => text.clone(),
            Value::Array(items) => items
                .iter()
                .map(|item| if item.is_nullish() { String::new() } else { item.to_js_string() })
                .collect::<Vec<_>>()
                .join(","),
            Value::Bytes(bytes) => bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(","),
            Value::Date(ms) => crate::date::iso_from_ms(*ms).unwrap_or_else(|| "Invalid Date".into()),
            Value::Object(_) | Value::Ext(..) => "[object Object]".into(),
        }
    }

    /// `Number(value)` for the values the backend converts.
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Bool(value) => f64::from(u8::from(*value)),
            Value::Number(number) => *number,
            Value::BigInt(number) => *number as f64,
            Value::String(text) => crate::number::string_to_number(text),
            Value::Date(ms) => *ms,
            Value::Array(items) => match items.as_slice() {
                [] => 0.0,
                [only] => only.to_number(),
                _ => f64::NAN,
            },
            _ => f64::NAN,
        }
    }
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Value::String(text.to_owned())
    }
}

impl From<String> for Value {
    fn from(text: String) -> Self {
        Value::String(text)
    }
}

impl From<&String> for Value {
    fn from(text: &String) -> Self {
        Value::String(text.clone())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(number: f64) -> Self {
        Value::Number(number)
    }
}

macro_rules! from_integer {
    ($($kind:ty),*) => {$(
        impl From<$kind> for Value {
            fn from(number: $kind) -> Self {
                Value::Number(number as f64)
            }
        }
    )*};
}
from_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<Vec<Value>> for Value {
    fn from(items: Vec<Value>) -> Self {
        Value::Array(items)
    }
}

impl From<Object> for Value {
    fn from(object: Object) -> Self {
        Value::Object(object)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    /// `None` becomes `null`.
    fn from(value: Option<T>) -> Self {
        value.map_or(Value::Null, Into::into)
    }
}

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        value.clone()
    }
}

/// An object literal with properties in the order written: `obj! { "id" => id, "name" => name }`.
#[macro_export]
macro_rules! obj {
    () => { $crate::Object::new() };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut object = $crate::Object::new();
        $( object.insert($key, $crate::Value::from($value)); )+
        object
    }};
}

/// An array literal: `arr![a, b, c]`.
#[macro_export]
macro_rules! arr {
    ($($item:expr),* $(,)?) => { $crate::Value::Array(vec![$($crate::Value::from($item)),*]) };
}
