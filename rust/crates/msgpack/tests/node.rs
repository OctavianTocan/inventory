//! Differential tests: mm-msgpack against the mirrored JS codec (`tests/oracle.mjs`) on the recorded library and on
//! generated and corrupted input.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use mm_js::{Object, Value, arr, base64, json};
use mm_msgpack::{DecodeOptions, EncodeOptions, decode_multi, encode, encode_recorded};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn oracle(request: Value) -> Vec<Value> {
    let mut child = Command::new("node")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/oracle.mjs"))
        .env("TZ", "UTC")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs");
    child.stdin.take().unwrap().write_all(json::to_string(&request).as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "oracle failed");
    match json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() {
        Value::Array(items) => items,
        other => panic!("oracle answered {other:?}"),
    }
}

fn sha(bytes: &[u8]) -> String {
    base64::hex(&Sha256::digest(bytes))
}

fn round_trip(bytes: &[u8], big: bool) -> Option<Vec<u8>> {
    let values = decode_multi(bytes, DecodeOptions { use_big_int64: big }).ok()?;
    let mut out = Vec::new();
    for value in &values {
        mm_msgpack::encode_into(&mut out, value, EncodeOptions { use_big_int64: big }).ok()?;
    }
    Some(out)
}

fn report(label: &str, total: usize, mismatches: &[String]) {
    eprintln!("{label}: {total} compared");
    assert!(mismatches.is_empty(), "{} of {total} {label} differ:\n{}", mismatches.len(), mismatches[..mismatches.len().min(20)].join("\n"));
}

#[test]
fn recorded_msgpack_bodies_round_trip_like_the_js_codec() {
    let index = json::parse(&std::fs::read_to_string(root().join("replay/index.json")).unwrap()).unwrap();
    let mut files: Vec<PathBuf> = index
        .as_object()
        .unwrap()
        .values()
        .filter(|entry| entry.get("headers").get("content-type").as_str().is_some_and(|kind| kind.contains("msgpack")))
        .map(|entry| root().join("replay/bodies").join(entry.get("file").as_str().unwrap()))
        .collect();
    for entry in std::fs::read_dir(root().join("capture/responses")).unwrap().flatten() {
        files.push(entry.path());
    }
    files.sort();
    files.dedup();
    files.retain(|file| file.exists());
    assert!(files.len() > 30, "the recorded library is present");
    let theirs = oracle(mm_js::obj! { "files" => Value::Array(files.iter().map(|file| file.to_string_lossy().into_owned().into()).collect()) }.into());
    let mut mismatches = Vec::new();
    let mut decoded = 0;
    for (file, expected) in files.iter().zip(&theirs) {
        let bytes = std::fs::read(file).unwrap();
        let ours = arr![round_trip(&bytes, false).map(|out| sha(&out)), round_trip(&bytes, true).map(|out| sha(&out))];
        if !ours.at(0).is_nullish() {
            decoded += 1;
        }
        if &ours != expected {
            mismatches.push(format!("{} ours {} node {}", file.display(), json::to_string(&ours), json::to_string(expected)));
        }
    }
    eprintln!("bodies that decode: {decoded}");
    assert!(decoded > 30);
    report("recorded bodies", files.len(), &mismatches);
}

#[test]
fn library_json_encodes_like_both_js_encoders() {
    let mut texts: Vec<String> = Vec::new();
    for file in ["capture/cards.json", "capture/details.json", "capture/meta.json"] {
        let text = std::fs::read_to_string(root().join(file)).unwrap();
        if let Value::Array(items) = json::parse(&text).unwrap() {
            texts.extend(items.iter().map(json::to_string));
        }
        texts.push(text);
    }
    let mut request = Object::new();
    request.insert("json", Value::Array(texts.iter().map(|text| text.as_str().into()).collect()));
    let mut rows: Vec<Value> = oracle(request.into())
        .into_iter()
        .zip(&texts)
        .map(|(answer, text)| {
            let mut row = vec![Value::from(text)];
            row.extend(answer.as_array().unwrap().iter().cloned());
            Value::Array(row)
        })
        .collect();
    let database = root().join("data/mymind.db");
    if database.exists() {
        rows.extend(oracle(mm_js::obj! { "db" => database.to_string_lossy().into_owned() }.into()));
    }
    let mut mismatches = Vec::new();
    for row in &rows {
        let value = json::parse(row.at(0).as_str().unwrap()).unwrap();
        let mut recorded = Vec::new();
        encode_recorded(&mut recorded, &value);
        let ours = [
            sha(&encode(&value, EncodeOptions { use_big_int64: true }).unwrap()),
            sha(&encode(&value, EncodeOptions::default()).unwrap()),
            sha(&recorded),
        ];
        for (index, label) in ["useBigInt64", "default", "recorded"].iter().enumerate() {
            if row.at(index + 1).as_str() != Some(ours[index].as_str()) {
                mismatches.push(format!("{label}: {}", row.at(0).as_str().unwrap().chars().take(100).collect::<String>()));
            }
        }
    }
    assert!(rows.len() > 5000);
    report("json values", rows.len(), &mismatches);
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

const NUMBERS: &[f64] = &[
    0.0, -0.0, 1.0, 127.0, 128.0, 255.0, 256.0, 65535.0, 65536.0, 4294967295.0, 4294967296.0, 9007199254740991.0,
    9007199254740992.0, -1.0, -32.0, -33.0, -128.0, -129.0, -32768.0, -32769.0, -2147483648.0, -2147483649.0,
    -9007199254740991.0, 0.5, -1.5, 1e300, f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 1.0000001, 3.4e38,
];

const STRINGS: &[&str] = &["", "a", "id", "__proto__", "constructor", "0", "12", "-1", "1.5", "4294967295", "héllo", "😀", "\u{feff}x"];

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
    let mut random = Random(0x243f6a8885a308d3);
    let mut inputs: Vec<Vec<u8>> = Vec::new();
    for _ in 0..4000 {
        let value = random_value(&mut random, 0);
        let big = random.below(2) == 0;
        let Ok(bytes) = encode(&value, EncodeOptions { use_big_int64: big }) else { continue };
        if random.below(3) == 0 {
            inputs.push(corrupt(&mut random, &bytes));
        }
        inputs.push(bytes);
    }
    for _ in 0..1500 {
        inputs.push(corrupt(&mut random, &[]));
    }
    for big in [0_i128, 1, i128::from(u64::MAX), -1, i128::from(i64::MIN), 1 << 53, (1 << 53) + 1] {
        inputs.push(encode(&Value::BigInt(big), EncodeOptions { use_big_int64: true }).unwrap());
    }
    let theirs = oracle(mm_js::obj! { "hex" => Value::Array(inputs.iter().map(|bytes| base64::hex(bytes).into()).collect()) }.into());
    let mut mismatches = Vec::new();
    for (bytes, expected) in inputs.iter().zip(&theirs) {
        let ours = arr![round_trip(bytes, false).map(|out| sha(&out)), round_trip(bytes, true).map(|out| sha(&out))];
        if &ours != expected {
            mismatches.push(format!("{} -> ours {} node {}", base64::hex(bytes).chars().take(200).collect::<String>(), json::to_string(&ours), json::to_string(expected)));
        }
    }
    report("generated inputs", inputs.len(), &mismatches);
}

#[test]
fn timestamps_match_the_js_codec() {
    let seconds = [0.0, 1.0, 4294967295.0, 4294967296.0, 17179869183.0, 17179869184.0, -1.0, -62135596800.0, 1792093660.0, 1e15];
    let nanos = [0.0, 1.0, 999_999_999.0, 123_456_789.0, 1_073_741_823.0, 1_073_741_824.0];
    let pairs: Vec<(f64, f64)> = seconds.iter().flat_map(|&sec| nanos.iter().map(move |&nsec| (sec, nsec))).collect();
    let request = mm_js::obj! { "timestamps" => Value::Array(pairs.iter().map(|&(sec, nsec)| arr![sec, nsec]).collect()) };
    let mut mismatches = Vec::new();
    for ((sec, nsec), expected) in pairs.iter().zip(oracle(request.into())) {
        let ours = base64::hex(&mm_msgpack::timestamp(*sec, *nsec));
        if Some(ours.as_str()) != expected.as_str() {
            mismatches.push(format!("{sec} {nsec}: ours {ours} node {}", json::to_string(&expected)));
        }
    }
    let dates = [0.0, 1.0, -1.0, 999.0, -999.0, -1000.0, -1001.0, 1_792_093_660_123.0, 8.64e15, -8.64e15, 17_179_869_184_000.0];
    let request = mm_js::obj! { "dates" => Value::Array(dates.iter().map(|&ms| ms.into()).collect()) };
    for (ms, expected) in dates.iter().zip(oracle(request.into())) {
        let ours = base64::hex(&encode(&Value::Date(*ms), EncodeOptions::default()).unwrap());
        if Some(ours.as_str()) != expected.as_str() {
            mismatches.push(format!("date {ms}: ours {ours} node {}", json::to_string(&expected)));
        }
    }
    report("timestamps", pairs.len() + dates.len(), &mismatches);
}
