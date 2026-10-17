//! The JavaScript semantics the backend's answers depend on, so a Rust port answers byte for byte what the JS
//! backend did: values with JS key order, `JSON`, number formatting, UTF-16 string operations, URI escapes, dates,
//! base64, `localeCompare` and `URL`.

pub mod base64;
mod collate;
pub mod date;
pub mod inspect;
pub mod json;
pub mod number;
pub mod text;
pub mod uri;
pub mod url;
mod value;

pub use collate::locale_compare;
pub use number::number_to_string;
pub use value::{Object, UNDEFINED, Value, array_index};
