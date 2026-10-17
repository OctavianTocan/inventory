//! Differential tests: every case runs through mm-js and through Node (`tests/oracle.mjs`), and the answers must match.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use mm_js::{Object, Value, arr, base64, date, inspect, json, number, text, uri};
use sha2::{Digest, Sha256};

fn oracle(request: &Value) -> Vec<Value> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut child = Command::new("node")
        .arg(manifest.join("tests/oracle.mjs"))
        .env("TZ", "UTC")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs");
    child.stdin.take().unwrap().write_all(json::to_string(request).as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "oracle failed");
    match json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() {
        Value::Array(items) => items,
        other => panic!("oracle answered {other:?}"),
    }
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }
}

fn bits(number: f64) -> String {
    format!("{:016x}", number.to_bits())
}

fn show(number: f64) -> String {
    if number == 0.0 && number.is_sign_negative() { "-0".into() } else { number::number_to_string(number) }
}

fn interesting_numbers(random: &mut Random) -> Vec<f64> {
    let mut numbers = vec![
        0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 1.5, 2.5, -2.5, 0.1, 0.2, 0.3, 1e21, 1e-7, 1.5e-7, 123456789012345680000.0, 1e20,
        f64::MAX, f64::MIN_POSITIVE, 5e-324, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 9007199254740993.0, 1.005, 2.675,
        1.45, 8.345, 0.000001, 0.0000001, 4.35, 1e100, 123.456, -0.0000001, 0.49999999999999994, 1700000000000.123,
    ];
    for _ in 0..3000 {
        numbers.push(f64::from_bits(random.next()));
    }
    for _ in 0..3000 {
        let scale = 10_f64.powi(random.below(12) as i32 - 4);
        numbers.push((random.below(2_000_000) as f64 - 1_000_000.0) / 1000.0 * scale);
    }
    for _ in 0..1000 {
        numbers.push((random.below(1_000_000) as f64) / 8.0 - 60000.0);
    }
    numbers
}

const TEXTS: &[&str] = &[
    "", " ", "42", " 42 ", "\u{feff}7\u{feff}", "\u{85}1", "\u{a0}1\u{3000}", "0x1f", "0X1F", "0o17", "0b101", "-0x10", "1e3", "1e",
    "1.", ".5", "-.5", "+.5", ".", "Infinity", "-Infinity", "+Infinity", "infinity", "1_000", "12px", "  -12.5e-3abc", "0.1.2",
    "08", "1e1000", "-1e-1000", "abc", "١٢", "9007199254740993", "123456789012345678901234567890", "\t\n\r\u{b}\u{c}9", "0x",
    "%", "%2", "%zz", "%41%42", "%C3%A9", "%E2%82%AC", "%F0%9F%98%80", "%C3", "%C0%80", "%ED%A0%80", "%2F%3F%23%26", "a+b%20c",
    "héllo wörld", "😀 emoji", "a\u{0}b", "ﬁ", "ß", "İstanbul", "ΣΑΣ", "Ǆ", "e\u{301}", "2026-10-17", "2026-10-17T10:20:30Z",
    "2026-10-17T10:20:30.123+02:00", "2026-10-17T10:20", "2026-10-17 10:20:30", "+002026-10-17T00:00:00Z", "-000001-01-01T00:00:00Z",
    "2026-02-30", "2026-13-01", "2024-02-29T24:00:00Z", "2026-10-17T10:20:30.1234567Z", "Sat, 17 Oct 2026 10:20:30 GMT",
    "Oct 17 2026", "October 17, 2026 10:20:30", "17 Oct 2026", "2026/10/17", "10/17/2026", "Sat Oct 17 2026 10:20:30 GMT+0200",
    "2026", "2026-10", "Tue, 01 Jan 2030 00:00:00 +0000", "SGVsbG8=", "SGVsbG8", "SGVs bG8=\n", "SGVsbG8==x", "-_-_", "+/+/",
    "a", "ab", "abc", "abcd=efgh", "eyJhbGciOiJSUzI1NiJ9", "!!!",
];

/// The `Date.parse` forms mm-js reads; V8's looser legacy forms (a bare `42` is the year 2042) give NaN instead.
const DATES: &[&str] = &[
    "2026-10-17", "2026-10-17T10:20:30Z", "2026-10-17T10:20:30.123+02:00", "2026-10-17T10:20", "2026-10-17 10:20:30",
    "+002026-10-17T00:00:00Z", "-000001-01-01T00:00:00Z", "2026-02-30", "2026-02-31T00:00:00Z", "2026-04-31", "2026-02-32",
    "2026-13-01", "2024-02-29T24:00:00Z", "2024-02-29T24:00:01Z", "2026-10-17T10:20:30.1234567Z", "2026-10-17T10:20:30.5Z",
    "Sat, 17 Oct 2026 10:20:30 GMT", "Oct 17 2026", "October 17, 2026 10:20:30", "17 Oct 2026", "2026/10/17", "10/17/2026",
    "Sat Oct 17 2026 10:20:30 GMT+0200", "2026", "2026-10", "Tue, 01 Jan 2030 00:00:00 +0000", "1970-01-01T00:00:00.000Z",
    "abc", "", "2026-10-17T25:00:00Z", "2026-1-17", "Sat, 17 Oct 2026 10:20:30 UTC", "17 October 2026 10:20 GMT-0130",
];

#[test]
fn numbers_match_node() {
    let mut random = Random(0x9e3779b97f4a7c15);
    let numbers = interesting_numbers(&mut random);
    let mut cases = Vec::new();
    let mut ours = Vec::new();
    for &value in &numbers {
        cases.push(arr!["num", bits(value)]);
        ours.push(Value::from(number::number_to_string(value)));
        cases.push(arr!["round", bits(value)]);
        ours.push(Value::from(show(number::round(value))));
        cases.push(arr!["iso", bits(value.trunc() % 1e16)]);
        ours.push(Value::from(date::iso_from_ms(value.trunc() % 1e16)));
        if value.is_finite() && value.abs() < 1e25 {
            let digits = random.below(8);
            cases.push(arr!["fixed", bits(value), digits as f64]);
            ours.push(Value::from(number::to_fixed(value, digits as usize)));
        }
    }
    compare("numbers", &cases, &ours);
}

#[test]
fn text_matches_node() {
    let mut cases = Vec::new();
    let mut ours = Vec::new();
    let mut push = |case: Value, answer: Value| {
        cases.push(case);
        ours.push(answer);
    };
    for &input in TEXTS {
        push(arr!["Number", input], show(number::string_to_number(input)).into());
        push(arr!["parseFloat", input], show(number::parse_float(input)).into());
        for radix in [0, 10, 16, 2, 36] {
            push(arr!["parseInt", input, radix as f64], show(number::parse_int(input, radix)).into());
        }
        push(arr!["trim", input], text::trim(input).into());
        push(arr!["len", input], (text::len(input) as f64).into());
        for (start, end) in [(0, None), (1, Some(3)), (-2, None), (2, Some(-1)), (3, Some(1)), (-100, Some(100))] {
            push(arr!["slice", input, start as f64, end.map(|end: i64| end as f64)], text::slice(input, start, end).into());
            push(arr!["substring", input, start as f64, end.map(|end: i64| end as f64)], text::substring(input, start, end).into());
        }
        push(arr!["padStart", input, 6.0, "ab"], text::pad_start(input, 6, "ab").into());
        push(arr!["indexOf", input, "0", 1.0], text::index_of(input, "0", 1).map_or(-1.0, |at| at as f64).into());
        push(arr!["lower", input], text::to_lower(input).into());
        push(arr!["upper", input], text::to_upper(input).into());
        push(arr!["encodeURIComponent", input], uri::encode_uri_component(input).into());
        push(arr!["encodeURI", input], uri::encode_uri(input).into());
        push(arr!["decodeURIComponent", input], uri::decode_uri_component(input).ok().into());
        push(arr!["decodeURI", input], uri::decode_uri(input).ok().into());
        push(arr!["b64", input], base64::hex(&base64::decode(input)).into());
        push(arr!["b64url", input], base64::hex(&base64::decode(input)).into());
        let hex = base64::hex(input.as_bytes());
        push(arr!["b64encode", hex.clone()], base64::encode(input.as_bytes()).into());
        push(arr!["b64urlencode", hex], base64::encode_url(input.as_bytes()).into());
    }
    for &input in DATES {
        push(arr!["dateParse", input], show(date::parse(input)).into());
    }
    let mut sorted: Vec<&str> = TEXTS.to_vec();
    sorted.extend(["Zebra", "apple", "Apple", "äpple", "a1", "a10", "a2", "_x", "-x", "x-y", "xy", "00000002UnE0O4FRWlrffG", "00000002UnE0O4FRwlrffG"]);
    let expected = Value::Array(sorted.iter().map(|&item| item.into()).collect());
    sorted.sort_by(|a, b| mm_js::locale_compare(a, b));
    push(arr!["sort", expected], Value::Array(sorted.iter().map(|&item| item.into()).collect()));
    compare("text", &cases, &ours);
}

const BAD_JSON: &[&str] = &[
    "", " ", "[", "{", "{\"a\"", "{\"a\":", "{\"a\":1", "{\"a\":1,", "[1", "[1,", "[1,]", "{\"a\":1,}", "{,}", "{1:2}", "{'a':1}",
    "{\"a\" 1}", "{\"a\":1 \"b\":2}", "[1 2]", "not json", "é", "tru", "nul", "fals", "n", "falsey", "true false", "trux", "nulx",
    "[tru]", "tRue", "{\"a\":tru}", "\"abc", "\"a\nb\"", "\"\\x\"", "\"\\u12G4\"", "\"\\u12\"", "\"\\u00\"", "\"\\u", "\"\\",
    "\"\\uD83D\\u12\"", "-", "-a", "1.", "1.e5", "1e", "1e+", "1.5e", "01", "00", "-01", "[01]", "[-01]", "1 2", "{} x", "1e5x",
    "true1", "-0x1", "[0x1]", "undefined", "NaN", "[object Object]", "Infinity", "-Infinity", " undefined", "undefined ",
    "\"\u{1}\"", "\"\t\"", "éééééééééééééééééééééé", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "aaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaa",
    "[1,2,3,4,5,6,7,8,9,10,11,12,x]", "[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,x,17,18,19,20,21,22,23]", "[1,2,3,4,5,6,7,8,9,10,11,trux]",
    "{\"aaaaaaaaaaaaaaaaaaaaa\":1,}", "[\n1,\n2\nx]", "[\r\n1,\r2\rx]", "[\n\u{2028}\n1 2]", "\"\u{2028}abc", "\"é\u{1F600}abc",
    "{\"a\":1}}", "]", "}", ",", ":", "+1", ".5", "[-]", "[1.]", "[1e]", "{\"a\":1, \"b\"}", "{\"a\":1, \"b\":}", "[,1]", "[1,,2]",
    "\u{a0}[]", "\u{feff}[]", "[]\u{a0}", "\"abc\\\"", "{\"a\":1,\"b\"", "{\"a\":1,\"b\":", "{\"a\"}", "[1}", "{\"a\":1]", "[{]", "[[]",
    "[1x]", "{\"a\":-}", "{\"a\":\"\\", "{\"a\":\"\\x\"}", "\n\nx", "[1,2,3,4,5,6,7,8,9,10]x", "[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,tru",
    "[1.5.2]", "{\"a\":[1,{\"b\":tru}]}", "[\"é\", x]", "1", "[]", "{\"a\":\"\\uD800\"}",
];

const KEYS: &[&str] = &["a", "media", "replayed", "static", "_x", "x1", "1x", "0", "with space", "dash-ed", "it's", "quote\"d", "é", "", "database"];
const WORDS: &[&str] = &[
    "", "a", "it's", "say \"hi\"", "it's \"both\"", "back`tick's \"x\"", "${x}'\"", "tab\there", "line\nbreak", "\u{1}\u{1f}\u{7f}\u{9f}\u{a0}",
    "back\\slash", "/home/octaviantocan/Projects/20-29_Personal_Products_and_Platforms/data/hosted/test.db", "é😀",
    "first line of a long text that keeps going\nsecond line of the long text that keeps going on\nthird",
];

fn random_value(random: &mut Random, depth: usize) -> Value {
    match random.below(if depth > 5 { 4 } else { 8 }) {
        0 => Value::Null,
        1 => Value::Bool(random.below(2) == 1),
        2 => [0.0, -0.0, 1.0, 42.0, 1234567.0, -3.5, 1e21, 0.1].map(Value::Number)[random.below(8) as usize].clone(),
        3 => WORDS[random.below(WORDS.len() as u64) as usize].into(),
        4 | 5 => {
            let mut object = Object::new();
            for _ in 0..random.below(9) {
                object.insert(KEYS[random.below(KEYS.len() as u64) as usize], random_value(random, depth + 1));
            }
            Value::Object(object)
        }
        6 => Value::Array((0..random.below(30)).map(|_| Value::Number((random.below(2000) as f64) - 50.0)).collect()),
        _ => Value::Array((0..random.below(12)).map(|_| random_value(random, depth + 1)).collect()),
    }
}

#[test]
fn inspect_matches_node() {
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let mut values: Vec<Value> = (0..3000).map(|_| random_value(&mut random, 0)).collect();
    let counts = |names: &[&str]| -> Value {
        let mut object = Object::new();
        for (at, name) in names.iter().enumerate() {
            object.insert(*name, (at * 1237) as f64);
        }
        Value::Object(object)
    };
    let mut library = counts(&["library", "database", "cards", "recorded", "replay", "skipped", "details", "responses"]);
    if let Value::Object(object) = &mut library {
        object.insert("library", "test");
        object.insert("database", "/home/octaviantocan/Projects/mymind/data/hosted/test.db");
        object.insert("captured", counts(&["media", "mediahost", "carbon", "replayed"]));
    }
    values.extend([counts(&["static", "replayed"]), library, Value::Array((0..150).map(|at| Value::Number(at as f64)).collect())]);
    values.push(Value::Array((0..120).map(|at| Value::from(format!("item {at}"))).collect()));
    values.push("x".repeat(10_050).into());
    let cases: Vec<Value> = values.iter().map(|value| arr!["inspect", json::to_string(value)]).collect();
    let ours: Vec<Value> = values.iter().map(|value| inspect::inspect(&json::parse(&json::to_string(value)).unwrap()).into()).collect();
    compare("inspect", &cases, &ours);
    let shown = |value: &Value| value.as_str().unwrap_or_default().to_owned();
    let (multiline, grouped) = (ours.iter().filter(|value| shown(value).contains('\n')).count(), ours.iter().filter(|value| shown(value).contains(",  ")).count());
    eprintln!("inspect: {multiline} multi-line, {grouped} grouped");
    assert!(multiline > 500 && grouped > 100, "the cases cover line breaking and grouping");
}

#[test]
fn json_errors_match_node() {
    let cases: Vec<Value> = BAD_JSON.iter().map(|&text| arr!["parseError", text]).collect();
    let ours: Vec<Value> = BAD_JSON.iter().map(|&text| json::parse(text).err().map(|error| format!("SyntaxError: {error}")).into()).collect();
    compare("json errors", &cases, &ours);
}

fn json_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files: Vec<PathBuf> = ["capture/cards.json", "capture/details.json", "capture/meta.json", "capture/responses.json"]
        .iter()
        .map(|file| root.join(file))
        .collect();
    for directory in ["capture/responses", "replay/bodies"] {
        let Ok(entries) = std::fs::read_dir(root.join(directory)) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.metadata().is_ok_and(|meta| meta.is_file() && meta.len() < 50_000_000) {
                let head = std::fs::read(&path).map(|bytes| bytes.first().copied()).unwrap_or(None);
                if matches!(head, Some(b'{' | b'[' | b'"')) {
                    files.push(path);
                }
            }
        }
    }
    files.retain(|file| file.exists());
    files.sort();
    files
}

fn sha(text: &str) -> String {
    base64::hex(&Sha256::digest(text.as_bytes()))
}

#[test]
fn json_round_trips_match_node_on_the_recorded_library() {
    let files = json_files();
    assert!(files.len() > 50, "the recorded library is present");
    let request = mm_js::obj! { "files" => Value::Array(files.iter().map(|file| file.to_string_lossy().into_owned().into()).collect()) };
    let theirs = oracle(&Value::Object(request));
    let mut mismatches = Vec::new();
    for (file, expected) in files.iter().zip(&theirs) {
        let text = String::from_utf8_lossy(&std::fs::read(file).unwrap()).into_owned();
        let ours = match json::parse(&text) {
            Ok(Value::Null) | Err(_) => Value::Null,
            Ok(value) => arr![sha(&json::to_string(&value)), sha(&json::stringify_indented(&value, "  ").unwrap().unwrap())],
        };
        if &ours != expected {
            mismatches.push(file.display().to_string());
        }
    }
    eprintln!("json: {} files compared", files.len());
    assert!(mismatches.is_empty(), "{} of {} files differ, first: {:?}", mismatches.len(), files.len(), &mismatches[..mismatches.len().min(5)]);
}

#[test]
fn json_round_trips_match_node_on_the_library_database() {
    let database = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/mymind.db");
    if !database.exists() {
        eprintln!("no local library database, skipped");
        return;
    }
    let rows = oracle(&mm_js::obj! { "db" => database.to_string_lossy().into_owned() }.into());
    let mut mismatches = Vec::new();
    for row in &rows {
        let text = row.at(0).as_str().unwrap();
        let value = json::parse(text).expect("Node parsed it");
        let ours = arr![sha(&json::to_string(&value)), sha(&json::stringify_indented(&value, "  ").unwrap().unwrap())];
        if ours.at(0) != row.at(1) || ours.at(1) != row.at(2) {
            mismatches.push(text.chars().take(120).collect::<String>());
        }
    }
    eprintln!("database json: {} values compared", rows.len());
    assert!(rows.len() > 1000, "the library database holds JSON");
    assert!(mismatches.is_empty(), "{} of {} values differ, first: {:?}", mismatches.len(), rows.len(), &mismatches[..mismatches.len().min(5)]);
}

fn compare(label: &str, cases: &[Value], ours: &[Value]) {
    let theirs = oracle(&mm_js::obj! { "cases" => Value::Array(cases.to_vec()) }.into());
    let mismatches: Vec<String> = cases
        .iter()
        .zip(ours.iter().zip(&theirs))
        .filter(|(_, (ours, theirs))| ours != theirs)
        .map(|(case, (ours, theirs))| format!("{} -> ours {} node {}", json::to_string(case), json::to_string(ours), json::to_string(theirs)))
        .collect();
    eprintln!("{label}: {} cases compared", cases.len());
    assert!(mismatches.is_empty(), "{} of {} {label} cases differ:\n{}", mismatches.len(), cases.len(), mismatches[..mismatches.len().min(30)].join("\n"));
}
