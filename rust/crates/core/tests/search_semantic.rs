//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::collections::HashMap;

use common::{Random, assert_same, calls};
use mm_core::runtime;
use mm_core::search_semantic::{decode_vector, embeddings_enabled};
use mm_core::Value;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:search-semantic";

fn vector_bytes(random: &mut Random, kind: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    for index in 0..768u64 {
        let value = match (kind, index) {
            (3, 100) => f32::NAN,
            (4, 5) => f32::INFINITY,
            (5, 9) => f32::NEG_INFINITY,
            _ => (random.below(2000) as f32 - 1000.0) / 7.0,
        };
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    match kind {
        1 => bytes.truncate(3071),
        2 => bytes.push(0),
        _ => {}
    }
    bytes
}

#[test]
fn helpers_match_node() {
    let _goldens = common::test_guard();
    let keys = [None, Some(""), Some("k")];
    let bases = [None, Some("http://127.0.0.1:1/v1"), Some("http://localhost/x"), Some("http://[::1]:5/x"), Some("https://example.com"), Some("not a url"), Some("")];
    let mut cases = Vec::new();
    for key in keys {
        for base in bases {
            for offline in [false, true] {
                cases.push((key, base, offline));
            }
        }
    }
    let args: Vec<(&str, Vec<Value>)> = cases.iter().map(|(key, base, offline)| ("enabled", vec![key.map_or(Value::Null, Value::from), base.map_or(Value::Null, Value::from), (*offline).into()])).collect();
    let answers = calls(DRIVER, &args);
    for ((key, base, offline), js) in cases.iter().zip(&answers) {
        let mut vars = HashMap::new();
        if let Some(key) = key {
            vars.insert("GEMINI_API_KEY".to_owned(), (*key).to_owned());
        }
        if let Some(base) = base {
            vars.insert("GEMINI_BASE_URL".to_owned(), (*base).to_owned());
        }
        runtime::set_env(vars);
        assert_same(&Ok(embeddings_enabled(*offline).into()), js, &format!("enabled({key:?}, {base:?}, {offline})"));
    }
    let mut random = Random(7);
    let inputs: Vec<Vec<u8>> = (0..6).map(|kind| vector_bytes(&mut random, kind)).chain([Vec::new()]).collect();
    let args: Vec<(&str, Vec<Value>)> = inputs.iter().map(|bytes| ("decode", vec![Value::Bytes(bytes.clone())])).collect();
    for (bytes, js) in inputs.iter().zip(calls(DRIVER, &args)) {
        let rust = decode_vector(bytes).map_or(Value::Null, |vector| Value::Array(vector.iter().map(|value| if value.is_nan() { "NaN".into() } else { Value::Number(f64::from(*value)) }).collect()));
        assert_same(&Ok(rust), &js, &format!("decode({} bytes)", bytes.len()));
    }
}
