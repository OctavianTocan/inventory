//! `mm_core::site` against `lib/site.mjs`.

mod common;

use mm_core::http::Headers;
use mm_core::site::{LOCAL_PREFIXES, localize, localize_value, replay_key_for, send_range};
use mm_core::{Error, Object, Value, arr, obj};

const DRIVER: &str = "rust/crates/core/tests/drivers/site.mjs";

fn check(name: &str, cases: Vec<(Vec<Value>, Result<Value, Error>)>) {
    let calls: Vec<(&str, Vec<Value>)> = cases.iter().map(|(args, _)| (name, args.clone())).collect();
    for (answer, (args, rust)) in common::calls(DRIVER, &calls).iter().zip(&cases) {
        common::assert_same(rust, answer, &format!("{name}({})", common::text(&Value::Array(args.clone()))));
    }
}

#[test]
fn localize_matches_node() {
    let _goldens = common::test_guard();
    let alphabet = [
        "https://", "access.mymind.com/", "static.accelerator.net/", "media.mymind.host/", "carbon-media.accelerator.net/",
        "https://access.mymind.com/", "https://media.mymind.host/", "http://", "x", "/", "é", "😀", "\"", "\n",
    ];
    let mut random = common::Random(0x517e_10ca);
    let cases = (0..5_000)
        .map(|_| {
            let length = random.below(10) as usize;
            let text = random.text(&alphabet, length);
            let rust = Ok(Value::from(localize(&text)));
            (vec![Value::from(text)], rust)
        })
        .collect();
    check("localize", cases);
}

fn random_value(random: &mut common::Random, depth: u32) -> Value {
    let strings = ["https://access.mymind.com/cards", "https://static.accelerator.net/a.js", "plain", "", "😀https://media.mymind.host/x"];
    match random.below(if depth > 2 { 6 } else { 8 }) {
        0 => Value::Null,
        1 => Value::Bool(random.below(2) == 0),
        2 => Value::Number(random.below(1000) as f64 / 7.0),
        3 => Value::from(strings[random.below(strings.len() as u64) as usize]),
        4 => Value::Bytes(b"https://access.mymind.com/".to_vec()),
        5 => Value::Date(random.below(2_000_000_000_000) as f64),
        6 => Value::Array((0..random.below(4)).map(|_| random_value(random, depth + 1)).collect()),
        _ => {
            let keys = ["a", "2", "https://access.mymind.com/", "b", "0", "__proto__x"];
            let mut object = Object::default();
            for _ in 0..random.below(4) {
                object.insert(keys[random.below(keys.len() as u64) as usize].to_owned(), random_value(random, depth + 1));
            }
            Value::Object(object)
        }
    }
}

#[test]
fn localize_value_matches_node() {
    let _goldens = common::test_guard();
    let mut random = common::Random(0x10ca_1173);
    let cases = (0..2_000)
        .map(|_| {
            let value = random_value(&mut random, 0);
            let rust = Ok(localize_value(&value));
            (vec![value], rust)
        })
        .collect();
    check("localizeValue", cases);
}

#[test]
fn send_range_matches_node() {
    let _goldens = common::test_guard();
    let ranges = [
        None, Some("bytes=0-"), Some("bytes=-5"), Some("bytes=3-2"), Some("bytes=0-0"), Some("bytes=a-b"), Some("bytes=1-2,3-4"),
        Some("bytes= 1-2"), Some("bytes=99-"), Some("bytes=-0"), Some("bytes=00-01"), Some("bytes=1-999"), Some("bytes=-"),
        Some("bytes=9007199254740993-"), Some("bytes=2-4"), Some("BYTES=1-2"), Some(""), Some("bytes=1-2-3"),
    ];
    let header_sets: [Value; 4] = [
        obj! {}.into(),
        obj! { "content-type" => "video/mp4" }.into(),
        obj! { "cache-control" => "max-age=60", "content-range" => "old", "x-a" => "1" }.into(),
        obj! { "accept-ranges" => "none", "content-type" => "text/plain" }.into(),
    ];
    let mut cases = Vec::new();
    for range in ranges {
        for status in [200u16, 404, 206] {
            for size in [0usize, 1, 10] {
                for headers in &header_sets {
                    let body: Vec<u8> = (0..size as u8).collect();
                    let Value::Object(pairs) = headers else { unreachable!() };
                    let headers_out = Headers::from_pairs(pairs.iter().map(|(name, value)| (name.into_owned(), value.as_str().unwrap().to_owned())));
                    let request = range.map_or_else(Headers::new, |range| Headers::from([("range", range)]));
                    let response = send_range(&request, status, body.clone(), &headers_out);
                    let mm_core::http::Body::Bytes(sent) = response.body else { panic!("no whole body") };
                    let rust = obj! {
                        "status" => response.status,
                        "headers" => Value::Array(response.headers.iter().map(|(name, value)| arr![name, value]).collect()),
                        "body" => Value::Bytes(sent),
                    };
                    cases.push((vec![range.map_or(Value::Null, Value::from), f64::from(status).into(), Value::Bytes(body), headers.clone()], Ok(rust.into())));
                }
            }
        }
    }
    check("sendRange", cases);
}

// The local request target that asks for a recorded live URL.
fn local_target(live: &str) -> String {
    for (local, origin) in LOCAL_PREFIXES {
        if let Some(rest) = live.strip_prefix(origin) {
            return format!("{local}{rest}");
        }
    }
    live.strip_prefix("https://access.mymind.com").map_or_else(|| format!("/{live}"), str::to_owned)
}

#[test]
fn replay_key_for_matches_node() {
    let _goldens = common::test_guard();
    let index = std::fs::read_to_string(common::root().join("replay/index.json")).unwrap();
    let Value::Object(index) = mm_js::json::parse(&index).unwrap() else { panic!("index is no object") };
    let mut targets: Vec<(String, String, Option<&str>)> = Vec::new();
    for (key, _) in index.iter() {
        let key = key.rsplit('|').next().unwrap();
        let Some((kind, live)) = key.split_once(' ') else { continue };
        let (method, dest) = if kind == "DOC" { ("GET", Some("document")) } else { (kind, None) };
        targets.push((method.to_owned(), local_target(live), dest));
    }
    let mut random = common::Random(0x4e_9a1);
    let parts = ["/", "_static/", "_mediahost/", "_carbon/", "media/", "a", "?q=1", "#h", "%20", "é", "?", "/../", ";", "cards"];
    for _ in 0..3_000 {
        let length = random.below(6) as usize;
        let target = format!("/{}", random.text(&parts, length));
        let method = ["GET", "POST", "HEAD"][random.below(3) as usize];
        let dest = [None, Some("document"), Some("iframe")][random.below(3) as usize];
        targets.push((method.to_owned(), target, dest));
    }
    let cases = targets
        .into_iter()
        .map(|(method, target, dest)| {
            let headers = dest.map_or_else(Headers::new, |dest| Headers::from([("sec-fetch-dest", dest)]));
            let rust = Ok(mm_js::url::parse(&target, Some("http://localhost:4321")).map_or(Value::Null, |url| replay_key_for(&method, &headers, &url).into()));
            (vec![Value::from(method), dest.map_or(Value::Null, Value::from), Value::from(target)], rust)
        })
        .collect::<Vec<_>>();
    assert!(cases.len() > 3_000);
    check("replayKeyFor", cases);
}
