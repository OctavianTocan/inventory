//! Compare mm-msgpack with frozen pre-switch goldens on the recorded library and generated inputs.

use std::path::{Path, PathBuf};

use mm_js::{Object, Value, arr, base64, json};
use mm_msgpack::{DecodeOptions, EncodeOptions, decode_multi, encode, encode_recorded};
use mm_test_support::golden::{Batch, TestGuard};
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use sha2::{Digest, Sha256};

// Locate the committed input corpus independently of the current directory.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

// Reject the retired recording adapter without starting another process.
fn oracle(request: Value) -> Vec<Value> {
    let _ = request;
    panic!("Legacy recording requires the recorded pre-switch revision")
}

// Hash exact codec output bytes.
fn sha(bytes: &[u8]) -> String {
    base64::hex(&Sha256::digest(bytes))
}

// Decode and re-encode all values, preserving the original oracle's null-on-error contract.
fn round_trip(bytes: &[u8], big: bool) -> Option<Vec<u8>> {
    let values = decode_multi(bytes, DecodeOptions { use_big_int64: big }).ok()?;
    let mut out = Vec::new();
    for value in &values {
        mm_msgpack::encode_into(&mut out, value, EncodeOptions { use_big_int64: big }).ok()?;
    }
    Some(out)
}

// Check every whole answer and reject truncated recording results.
fn compare(label: &str, requests: &[Vec<u8>], answers: &[Value], request: impl FnOnce() -> Value) {
    assert_eq!(requests.len(), answers.len(), "Rust answer count");
    let mut batch = Batch::new(env!("CARGO_MANIFEST_DIR"), "node", label, requests);
    let theirs = batch.recording().then(|| oracle(request()));
    if let Some(expected) = &theirs {
        assert_eq!(expected.len(), answers.len(), "oracle answer count");
    }
    for (index, answer) in answers.iter().enumerate() {
        let actual = json::to_string(answer);
        let expected = theirs.as_ref().map(|items| json::to_string(&items[index]));
        batch.check(
            index,
            actual.as_bytes(),
            expected.as_deref().map(str::as_bytes),
        );
    }
    batch.finish();
    eprintln!("{label}: {} compared", answers.len());
}

#[test]
fn recorded_msgpack_bodies_round_trip_like_the_js_codec() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let index =
        json::parse(&std::fs::read_to_string(root().join("replay/index.json")).unwrap()).unwrap();
    let mut files: Vec<PathBuf> = index
        .as_object()
        .unwrap()
        .values()
        .filter(|entry| {
            entry
                .get("headers")
                .get("content-type")
                .as_str()
                .is_some_and(|kind| kind.contains("msgpack"))
        })
        .map(|entry| {
            root()
                .join("replay/bodies")
                .join(entry.get("file").as_str().unwrap())
        })
        .collect();
    for entry in std::fs::read_dir(root().join("capture/responses")).unwrap() {
        let entry = entry.expect("recorded body directory entry is readable");
        files.push(entry.path());
    }
    files.sort();
    files.dedup();
    assert!(files.len() > 30, "the recorded library is present");
    let mut requests = Vec::new();
    let mut answers = Vec::new();
    let mut decoded = 0;
    for file in &files {
        let bytes = std::fs::read(file).expect("required recorded MessagePack body is present");
        requests.push(
            json::to_string(&arr![
                "file",
                file.strip_prefix(root())
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
                bytes.len() as f64,
                sha(&bytes)
            ])
            .into_bytes(),
        );
        let ours = arr![
            round_trip(&bytes, false).map(|out| sha(&out)),
            round_trip(&bytes, true).map(|out| sha(&out))
        ];
        if !ours.at(0).is_nullish() {
            decoded += 1;
        }
        answers.push(ours);
    }
    eprintln!("bodies that decode: {decoded}");
    assert!(decoded > 30);
    compare("recorded-bodies", &requests, &answers, || mm_js::obj! { "files" => Value::Array(files.iter().map(|file| file.to_string_lossy().into_owned().into()).collect()) }.into());
    guard.finish();
}

#[test]
fn library_json_encodes_like_both_js_encoders() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let mut texts: Vec<String> = Vec::new();
    for file in [
        "capture/cards.json",
        "capture/details.json",
        "capture/meta.json",
    ] {
        let text = std::fs::read_to_string(root().join(file)).unwrap();
        if let Value::Array(items) = json::parse(&text).unwrap() {
            texts.extend(items.iter().map(json::to_string));
        }
        texts.push(text);
    }
    let captured_count = texts.len();
    let database = fixture_database();
    let database_texts = database_json(&database);
    assert!(database_texts.len() > 1000, "the library database holds JSON");
    texts.extend(database_texts);
    assert!(
        texts.len() > 5000,
        "the original JSON corpus remains covered"
    );
    let requests: Vec<Vec<u8>> = texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            json::to_string(&arr![
                if index < captured_count {
                    "capture-json"
                } else {
                    "database-json"
                },
                text.as_str()
            ])
            .into_bytes()
        })
        .collect();
    let mut batch = Batch::new(
        env!("CARGO_MANIFEST_DIR"),
        "node",
        "library-json",
        &requests,
    );
    let theirs = batch.recording().then(|| {
        let answers = oracle(mm_js::obj! { "json" => Value::Array(texts[..captured_count].iter().map(|text| text.as_str().into()).collect()) }.into());
        assert_eq!(answers.len(), captured_count, "capture JSON oracle answer count");
        let mut rows = Vec::with_capacity(texts.len());
        for (index, answer) in answers.into_iter().enumerate() {
            let mut row = vec![Value::from(&texts[index])];
            row.extend(answer.as_array().expect("JSON oracle answer is an array").iter().cloned());
            rows.push(Value::Array(row));
        }
        let database_rows = oracle(mm_js::obj! { "db" => database.to_string_lossy().into_owned() }.into());
        assert_eq!(database_rows.len(), texts.len() - captured_count, "database JSON oracle answer count");
        rows.extend(database_rows);
        rows
    });
    for (index, text) in texts.iter().enumerate() {
        let value = json::parse(text).unwrap();
        let mut recorded = Vec::new();
        encode_recorded(&mut recorded, &value);
        let ours = arr![
            text.as_str(),
            sha(&encode(
                &value,
                EncodeOptions {
                    use_big_int64: true
                }
            )
            .unwrap()),
            sha(&encode(&value, EncodeOptions::default()).unwrap()),
            sha(&recorded),
        ];
        let actual = json::to_string(&ours);
        let expected = theirs.as_ref().map(|rows| json::to_string(&rows[index]));
        batch.check(
            index,
            actual.as_bytes(),
            expected.as_deref().map(str::as_bytes),
        );
    }
    batch.finish();
    eprintln!("json values: {} compared", texts.len());
    guard.finish();
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

const NUMBERS: &[f64] = &[
    0.0, -0.0, 1.0, 127.0, 128.0, 255.0, 256.0, 65535.0, 65536.0, 4294967295.0, 4294967296.0, 9007199254740991.0,
    9007199254740992.0, -1.0, -32.0, -33.0, -128.0, -129.0, -32768.0, -32769.0, -2147483648.0, -2147483649.0,
    -9007199254740991.0, 0.5, -1.5, 1e300, f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 1.0000001, 3.4e38,
];

const STRINGS: &[&str] = &["", "a", "id", "__proto__", "constructor", "0", "12", "-1", "1.5", "4294967295", "héllo", "😀", "\u{feff}x"];

// Reconstruct the original fixed-seed generated value corpus.
fn random_value(random: &mut Random, depth: u32) -> Value {
    match random.below(if depth > 3 { 9 } else { 12 }) {
        0 => Value::Null,
        1 => Value::Bool(random.below(2) == 0),
        2 => Value::Number(NUMBERS[random.below(NUMBERS.len() as u64) as usize]),
        3 => Value::Number((random.next() as i64 >> random.below(64)) as f64),
        4 => Value::Number(f64::from_bits(random.next())),
        5 => {
            let length = [0, 5, 31, 32, 200, 201, 255, 256, 70000][random.below(9) as usize];
            let unit = ["a", "é", "€", "😀", "\u{feff}"][random.below(5) as usize];
            Value::String(unit.repeat(length / unit.len().max(1) + usize::from(random.below(2) == 0)))
        }
        6 => Value::String(STRINGS[random.below(STRINGS.len() as u64) as usize].into()),
        7 => Value::Bytes((0..[0, 1, 255, 256, 70000][random.below(5) as usize]).map(|_| random.next() as u8).collect()),
        8 => {
            let ms = [0.0, -1.0, 1.0, 999.0, -1001.0, 1_792_093_660_123.0, 17_179_869_184_000.0, 4_294_967_296_000.0, -62_135_596_800_000.0, f64::NAN];
            Value::Date(ms[random.below(ms.len() as u64) as usize])
        }
        9 => Value::Array((0..[0, 1, 15, 16, 17][random.below(5) as usize]).map(|_| random_value(random, depth + 1)).collect()),
        10 => {
            let mut object = Object::new();
            for _ in 0..[0, 1, 15, 16, 20][random.below(5) as usize] {
                let key = if random.below(3) == 0 { random.below(40).to_string() } else { format!("k{}", random.below(1000)) };
                object.insert(key, random_value(random, depth + 1));
            }
            Value::Object(object)
        }
        _ => Value::Ext([-2, 0, 5, 127][random.below(4) as usize], (0..[0, 1, 2, 3, 4, 8, 16, 17, 300][random.below(9) as usize]).map(|_| random.next() as u8).collect()),
    }
}

// Apply the original fixed-seed mutation to a generated byte input.
fn corrupt(random: &mut Random, bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    match random.below(4) {
        0 if !out.is_empty() => {
            let at = random.below(out.len() as u64) as usize;
            out[at] = random.next() as u8;
        }
        1 if !out.is_empty() => out.truncate(random.below(out.len() as u64) as usize),
        2 => out.extend((0..random.below(6)).map(|_| random.next() as u8)),
        _ => out = (0..random.below(40)).map(|_| [0xa3, 0xa5, 0xc3, 0xe2, 0x82, 0xf0, 0x9f, 0x80, 0x41, 0xd9, 0x81, 0xd6, 0xff, 0xc0, 0xc1][random.below(15) as usize]).collect(),
    }
    out
}

#[test]
fn generated_and_corrupted_input_matches_the_js_codec() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let mut random = Random(0x243f6a8885a308d3);
    let mut inputs: Vec<Vec<u8>> = Vec::new();
    for _ in 0..4000 {
        let value = random_value(&mut random, 0);
        let big = random.below(2) == 0;
        let Ok(bytes) = encode(&value, EncodeOptions { use_big_int64: big }) else {
            continue;
        };
        if random.below(3) == 0 {
            inputs.push(corrupt(&mut random, &bytes));
        }
        inputs.push(bytes);
    }
    for _ in 0..1500 {
        inputs.push(corrupt(&mut random, &[]));
    }
    for big in [
        0_i128,
        1,
        i128::from(u64::MAX),
        -1,
        i128::from(i64::MIN),
        1 << 53,
        (1 << 53) + 1,
    ] {
        inputs.push(
            encode(
                &Value::BigInt(big),
                EncodeOptions {
                    use_big_int64: true,
                },
            )
            .unwrap(),
        );
    }
    let answers: Vec<Value> = inputs
        .iter()
        .map(|bytes| {
            arr![
                round_trip(bytes, false).map(|out| sha(&out)),
                round_trip(bytes, true).map(|out| sha(&out))
            ]
        })
        .collect();
    compare("generated-inputs", &inputs, &answers, || mm_js::obj! { "hex" => Value::Array(inputs.iter().map(|bytes| base64::hex(bytes).into()).collect()) }.into());
    guard.finish();
}

#[test]
fn timestamps_match_the_js_codec() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "node");
    let seconds = [
        0.0,
        1.0,
        4294967295.0,
        4294967296.0,
        17179869183.0,
        17179869184.0,
        -1.0,
        -62135596800.0,
        1792093660.0,
        1e15,
    ];
    let nanos = [
        0.0,
        1.0,
        999_999_999.0,
        123_456_789.0,
        1_073_741_823.0,
        1_073_741_824.0,
    ];
    let pairs: Vec<(f64, f64)> = seconds
        .iter()
        .flat_map(|&sec| nanos.iter().map(move |&nsec| (sec, nsec)))
        .collect();
    let request = mm_js::obj! { "timestamps" => Value::Array(pairs.iter().map(|&(sec, nsec)| arr![sec, nsec]).collect()) };
    let requests: Vec<Vec<u8>> = pairs
        .iter()
        .map(|&(sec, nsec)| json::to_string(&arr![sec, nsec]).into_bytes())
        .collect();
    let answers: Vec<Value> = pairs
        .iter()
        .map(|&(sec, nsec)| base64::hex(&mm_msgpack::timestamp(sec, nsec)).into())
        .collect();
    compare("timestamps", &requests, &answers, || request.into());
    let dates = [
        0.0,
        1.0,
        -1.0,
        999.0,
        -999.0,
        -1000.0,
        -1001.0,
        1_792_093_660_123.0,
        8.64e15,
        -8.64e15,
        17_179_869_184_000.0,
    ];
    let request =
        mm_js::obj! { "dates" => Value::Array(dates.iter().map(|&ms| ms.into()).collect()) };
    let requests: Vec<Vec<u8>> = dates
        .iter()
        .map(|&ms| json::to_string(&Value::Number(ms)).into_bytes())
        .collect();
    let answers: Vec<Value> = dates
        .iter()
        .map(|&ms| base64::hex(&encode(&Value::Date(ms), EncodeOptions::default()).unwrap()).into())
        .collect();
    compare("dates", &requests, &answers, || request.into());
    guard.finish();
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
    if let Ok(data) = root().join("data").canonicalize() {
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
