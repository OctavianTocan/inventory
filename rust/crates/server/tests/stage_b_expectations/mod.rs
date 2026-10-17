//! Independent expectations recorded once from the retained JS implementation; provenance is in the fixture.
#![allow(dead_code)]

use mm_js::{Value, json, obj};
use regex::Regex;

// Load immutable synthetic JS expectations without invoking an oracle.
fn fixture() -> Value {
    json::parse(include_str!("../fixtures/stage_b_expectations.json")).unwrap()
}

// Collect generated identity values in model traversal order, preserving identity reuse.
fn identities(value: &Value, ids: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter() {
                if key == "id"
                    && let Some(id) = value.as_str()
                    && !ids.iter().any(|item| item == id)
                {
                    ids.push(id.to_owned());
                }
                identities(value, ids);
            }
        }
        Value::Array(values) => {
            for value in values {
                identities(value, ids);
            }
        }
        _ => {}
    }
}

// Normalize only synthetic generated ids and ISO wall-clock fields, identically to the one-time JS recording.
fn normalized(input: &Value, html: &str) -> (String, String) {
    let mut ids = Vec::new();
    identities(input, &mut ids);
    let mut input = json::to_string(input);
    let mut html = html.to_owned();
    for (index, id) in ids.iter().enumerate() {
        let label = format!("__id{index}__");
        input = input.replace(id, &label);
        html = html.replace(id, &label);
    }
    let iso = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z").unwrap();
    (
        iso.replace_all(&input, "__ISO__").into_owned(),
        iso.replace_all(&html, "__ISO__").into_owned(),
    )
}

/// Compare persisted mutation HTML with complete frozen JS markup for the exact normalized model.
///
/// # Arguments
/// * `data` - Persisted synthetic card data.
/// * `extras` - Persisted synthetic rendering metadata.
/// * `html` - Persisted rendered HTML to verify.
///
/// # Panics
/// If the model has no frozen case or its complete markup differs from JS.
pub fn assert_html(data: &Value, extras: &Value, html: &str) {
    let (input, actual) = normalized(
        &obj! { "data" => data.clone(), "extras" => extras.clone() }.into(),
        html,
    );
    let fixture = fixture();
    let expected = fixture
        .get("cards")
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry.get("input").as_str() == Some(input.as_str()))
        .unwrap_or_else(|| panic!("missing independent persisted-render expectation: {input}"));
    assert_eq!(
        actual,
        expected.get("html").as_str().unwrap(),
        "persisted HTML equals frozen JS render"
    );
}

/// Parse current persisted ISO timestamps using the system UTC calendar, independently of backend conversions.
///
/// # Arguments
/// * `iso` - UTC ISO timestamp emitted in persisted synthetic models.
///
/// # Returns
/// Milliseconds since the Unix epoch, with sub-millisecond precision discarded as JS Date.parse does.
///
/// # Panics
/// On a timestamp outside the fixture's ISO shape or unrepresentable calendar fields.
pub fn milliseconds(iso: &str) -> f64 {
    let field = |start: usize, end: usize| iso[start..end].parse::<i32>().unwrap();
    assert_eq!(&iso[10..11], "T");
    assert!(iso.ends_with('Z'));
    // timegm initializes the UTC calendar fields rather than using the backend's ISO/nanosecond helpers.
    let mut utc: libc::tm = unsafe { std::mem::zeroed() };
    utc.tm_year = field(0, 4) - 1900;
    utc.tm_mon = field(5, 7) - 1;
    utc.tm_mday = field(8, 10);
    utc.tm_hour = field(11, 13);
    utc.tm_min = field(14, 16);
    utc.tm_sec = field(17, 19);
    let seconds = unsafe { libc::timegm(&mut utc) };
    let fraction = iso[19..]
        .strip_prefix('.')
        .unwrap_or("")
        .trim_end_matches('Z');
    let ms = format!("{fraction:0<3}")[..3].parse::<i64>().unwrap();
    (seconds * 1000 + ms) as f64
}

/// Return the literal backfill prompt produced by JS analysisItem/documentText for this suite's synthetic note.
///
/// # Returns
/// The complete immutable hosted embedding prompt.
pub fn backfill_prompt() -> String {
    fixture()
        .get("backfill")
        .get("prompt")
        .as_str()
        .unwrap()
        .to_owned()
}

// Calibrate the independent system conversion against exact JS Date.parse answers, including leap days and fractions.
#[test]
fn system_timestamp_conversion_matches_frozen_js() {
    for entry in fixture().get("timestamps").as_array().unwrap() {
        assert_eq!(
            milliseconds(entry.get("iso").as_str().unwrap()),
            entry.get("ms").as_f64().unwrap()
        );
    }
}
