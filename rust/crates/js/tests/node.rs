//! Compare mm-js with frozen pre-switch golden expectations for every original oracle case.

use std::path::{Path, PathBuf};

use mm_js::{Object, Value, arr, base64, date, inspect, json, number, text, uri};
use mm_test_support::golden::{Batch, TestGuard};
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use sha2::{Digest, Sha256};

// Reject the retired recording adapter without starting another process.
fn oracle(request: &Value) -> Vec<Value> {
    let _ = request;
    panic!("Legacy recording requires the recorded pre-switch revision")
}

struct Random(u64);

impl Random {
    // Advance the original fixed-seed generator.
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    // Select a value below the exclusive bound.
    fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit
    }
}

// Keep number inputs lossless, including negative zero and non-finite values.
fn bits(number: f64) -> String {
    format!("{:016x}", number.to_bits())
}

// Preserve the sign of zero in the operations where Node exposes it.
fn show(number: f64) -> String {
    if number == 0.0 && number.is_sign_negative() { "-0".into() } else { number::number_to_string(number) }
}

// Reconstruct the original edge cases and generated number corpus.
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

// Include the Date.parse forms mm-js reads and the original legacy-form cases.
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
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
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
    guard.finish();
}

#[test]
fn text_matches_node() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
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
    guard.finish();
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

// Reconstruct the original fixed-seed inspect corpus.
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
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
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
    guard.finish();
}

#[test]
fn json_errors_match_node() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let cases: Vec<Value> = BAD_JSON.iter().map(|&text| arr!["parseError", text]).collect();
    let ours: Vec<Value> = BAD_JSON.iter().map(|&text| json::parse(text).err().map(|error| format!("SyntaxError: {error}")).into()).collect();
    compare("json errors", &cases, &ours);
    guard.finish();
}

// Preserve the original JSON corpus and add every frozen hosted response-header JSON file.
fn json_files() -> Vec<(String, PathBuf)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().expect("repository root resolves");
    let mut files: Vec<PathBuf> = [
        "capture/cards.json",
        "capture/details.json",
        "capture/meta.json",
        "capture/responses.json",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    assert!(
        files.iter().all(|file| file.is_file()),
        "required capture JSON files are present"
    );
    for directory in ["capture/responses", "replay/bodies"] {
        let entries = std::fs::read_dir(root.join(directory))
            .expect("required recorded JSON directory is present");
        for entry in entries {
            let entry = entry.expect("recorded JSON directory entry is readable");
            let path = entry.path();
            let metadata = entry
                .metadata()
                .expect("recorded JSON metadata is readable");
            if metadata.is_file() && metadata.len() < 50_000_000 {
                let bytes = std::fs::read(&path).expect("recorded JSON input is readable");
                let head = bytes.first().copied();
                if matches!(head, Some(b'{' | b'[' | b'"')) {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    let mut named: Vec<(String, PathBuf)> = files.into_iter().map(|file| {
        (file.strip_prefix(&root).unwrap().to_string_lossy().into_owned(), file)
    }).collect();
    let bucket = PathBuf::from(std::env::var_os("MYMIND_TEST_HOSTED_BUCKET")
        .expect("MYMIND_TEST_HOSTED_BUCKET must name the frozen hosted fixture"))
        .canonicalize().expect("MYMIND_TEST_HOSTED_BUCKET fixture is present");
    let headers = bucket.join(".headers");
    assert!(headers.is_dir(), "frozen hosted response-header directory is present");
    let mut header_files = Vec::new();
    header_json_files(&headers, &mut header_files);
    header_files.sort();
    assert!(!header_files.is_empty(), "frozen hosted response-header JSON corpus is present");
    named.extend(header_files.into_iter().map(|file| {
        (format!("<fixture:hosted-bucket>/{}", file.strip_prefix(&bucket).unwrap().to_string_lossy()), file)
    }));
    named
}

// Recursively enumerate frozen header JSON files without silently losing unreadable inputs.
fn header_json_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("frozen header directory is readable") {
        let entry = entry.expect("frozen header directory entry is readable");
        let path = entry.path();
        let kind = entry.file_type().expect("frozen header input type is readable");
        assert!(!kind.is_symlink(), "frozen header inputs must not be symbolic links");
        if kind.is_dir() {
            header_json_files(&path, files);
        } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "json") {
            files.push(path);
        }
    }
}

// Hash the exact text bytes used by the original oracle.
fn sha(text: &str) -> String {
    base64::hex(&Sha256::digest(text.as_bytes()))
}

#[test]
fn json_round_trips_match_node_on_the_recorded_library() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let files = json_files();
    assert!(files.len() > 50, "the recorded library is present");
    let contents: Vec<Vec<u8>> = files
        .iter()
        .map(|(_, file)| std::fs::read(file).unwrap())
        .collect();
    let requests: Vec<Vec<u8>> = files
        .iter()
        .zip(&contents)
        .map(|((name, _), bytes)| {
            json::to_string(&arr![
                "file",
                name.as_str(),
                bytes.len() as f64,
                base64::hex(&Sha256::digest(bytes))
            ])
            .into_bytes()
        })
        .collect();
    let mut batch = Batch::new(
        env!("CARGO_MANIFEST_DIR"),
        "node",
        "recorded-json",
        &requests,
    );
    let request = mm_js::obj! { "files" => Value::Array(files.iter().map(|(_, file)| file.to_string_lossy().into_owned().into()).collect()) };
    let theirs = batch.recording().then(|| oracle(&Value::Object(request)));
    if let Some(expected) = &theirs {
        assert_eq!(
            expected.len(),
            files.len(),
            "recorded JSON oracle answer count"
        );
    }
    for (index, bytes) in contents.iter().enumerate() {
        let text = String::from_utf8_lossy(bytes);
        let ours = match json::parse(&text) {
            Ok(Value::Null) | Err(_) => Value::Null,
            Ok(value) => arr![
                sha(&json::to_string(&value)),
                sha(&json::stringify_indented(&value, "  ").unwrap().unwrap())
            ],
        };
        let actual = json::to_string(&ours);
        let expected = theirs
            .as_ref()
            .map(|answers| json::to_string(&answers[index]));
        batch.check(
            index,
            actual.as_bytes(),
            expected.as_deref().map(str::as_bytes),
        );
    }
    batch.finish();
    eprintln!("json: {} files compared", files.len());
    guard.finish();
}

#[test]
fn json_round_trips_match_node_on_the_library_database() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let database = fixture_database();
    let texts = database_json(&database);
    assert!(texts.len() > 1000, "the library database holds JSON");
    let requests: Vec<Vec<u8>> = texts
        .iter()
        .map(|text| json::to_string(&arr!["database-json", text.as_str()]).into_bytes())
        .collect();
    let mut batch = Batch::new(
        env!("CARGO_MANIFEST_DIR"),
        "node",
        "database-json",
        &requests,
    );
    let theirs = batch
        .recording()
        .then(|| oracle(&mm_js::obj! { "db" => database.to_string_lossy().into_owned() }.into()));
    if let Some(expected) = &theirs {
        assert_eq!(
            expected.len(),
            texts.len(),
            "database JSON oracle answer count"
        );
    }
    for (index, text) in texts.iter().enumerate() {
        let value = json::parse(text).expect("fixture JSON is valid");
        let actual = json::to_string(&arr![
            text.as_str(),
            sha(&json::to_string(&value)),
            sha(&json::stringify_indented(&value, "  ").unwrap().unwrap())
        ]);
        let expected = theirs
            .as_ref()
            .map(|answers| json::to_string(&answers[index]));
        batch.check(
            index,
            actual.as_bytes(),
            expected.as_deref().map(str::as_bytes),
        );
    }
    batch.finish();
    eprintln!("database json: {} values compared", texts.len());
    guard.finish();
}

// Check every whole answer and reject truncated recording results.
fn compare(label: &str, cases: &[Value], ours: &[Value]) {
    assert_eq!(cases.len(), ours.len(), "Rust answer count");
    let requests: Vec<Vec<u8>> = cases
        .iter()
        .map(|case| json::to_string(case).into_bytes())
        .collect();
    let mut batch = Batch::new(env!("CARGO_MANIFEST_DIR"), "node", label, &requests);
    let theirs = batch
        .recording()
        .then(|| oracle(&mm_js::obj! { "cases" => Value::Array(cases.to_vec()) }.into()));
    if let Some(expected) = &theirs {
        assert_eq!(expected.len(), cases.len(), "oracle answer count");
    }
    for (index, answer) in ours.iter().enumerate() {
        let actual = json::to_string(answer);
        let expected = theirs
            .as_ref()
            .map(|answers| json::to_string(&answers[index]));
        batch.check(
            index,
            actual.as_bytes(),
            expected.as_deref().map(str::as_bytes),
        );
    }
    batch.finish();
    eprintln!("{label}: {} cases compared", cases.len());
}

// Require the immutable fixture selected by the test environment, never a mutable data database.
fn fixture_database() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("MYMIND_TEST_DB")
            .expect("MYMIND_TEST_DB must name the frozen library fixture"),
    );
    assert!(path.is_file(), "MYMIND_TEST_DB fixture is missing");
    let canonical = path
        .canonicalize()
        .expect("MYMIND_TEST_DB fixture path resolves");
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    if let Ok(data) = data.canonicalize() {
        assert!(
            !canonical.starts_with(data),
            "MYMIND_TEST_DB must be an immutable fixture outside data/"
        );
    }
    canonical
}

// Read the original oracle's JSON cells in SQLite table, row and column order without exposing their contents.
fn database_json(path: &Path) -> Vec<String> {
    let database = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("library fixture opens read-only");
    let tables: Vec<String> = database
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%'",
        )
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let mut texts = Vec::new();
    for table in tables {
        let mut statement = database
            .prepare(&format!("SELECT * FROM \"{}\"", table.replace('"', "\"\"")))
            .unwrap();
        let count = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for column in 0..count {
                let ValueRef::Text(bytes) = row.get_ref(column).unwrap() else {
                    continue;
                };
                if !matches!(bytes.first(), Some(b'[' | b'{')) {
                    continue;
                }
                let text = std::str::from_utf8(bytes).expect("fixture text is UTF-8");
                if json::parse(text).is_ok_and(|value| !matches!(value, Value::Null)) {
                    texts.push(text.to_owned());
                }
            }
        }
    }
    texts
}
