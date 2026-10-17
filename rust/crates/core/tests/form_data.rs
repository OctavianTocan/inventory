//! `mm_core::form_data` against `Response.prototype.formData()` in Node 24.12: well-formed forms, hand-written edge
//! cases and many random mutations of them, under many content types.

mod common;

use mm_core::form_data::{self, FormValue};
use mm_core::{Value, obj};

const DRIVER: &str = "rust/crates/core/tests/drivers/form-data.mjs";

fn parsed(content_type: &str, body: &[u8]) -> Result<Value, form_data::FormError> {
    let form = form_data::parse(content_type, body)?;
    Ok(Value::Array(
        form.into_iter()
            .map(|(name, value)| match value {
                FormValue::Text(text) => obj! { "name" => name, "text" => text }.into(),
                FormValue::File(file) => obj! {
                    "name" => name,
                    "file" => Value::from(obj! { "name" => file.name, "type" => file.content_type, "bytes" => Value::Bytes(file.bytes) }),
                }
                .into(),
            })
            .collect(),
    ))
}

fn part(headers: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!("--B\r\n{headers}\r\n\r\n").into_bytes();
    out.extend_from_slice(body);
    out.extend_from_slice(b"\r\n");
    out
}

fn seeds() -> Vec<Vec<u8>> {
    let png = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0xff, b'-', b'-', b'B'];
    let mut seeds = Vec::new();
    let forms: Vec<Vec<Vec<u8>>> = vec![
        vec![
            part("Content-Disposition: form-data; name=\"metadata\"", br#"{"title":"x","spaceId":"s"}"#),
            part("Content-Disposition: form-data; name=\"blob\"; filename=\"a.png\"\r\nContent-Type: image/png", &png),
        ],
        vec![part("content-disposition: form-data; name=\"blob\"; filename=\"\"\r\ncontent-type: IMAGE/JPEG ", b"abc")],
        vec![part("Content-Disposition: form-data; name=\"blob\"; filename*=utf-8''caf%C3%A9.png", b"x")],
        vec![part("Content-Disposition: form-data; name=\"blob\"; filename*=UTF-8''%E9.png", b"x")],
        vec![part("Content-Disposition: form-data; name=\"blob\"; filename=\"a%22b%0Ac%0dd\"\r\nContent-Type: image/p\u{e9}ng", b"x")],
        vec![part("Content-Disposition: form-data; name=\"\u{feff}na%0Ame\"", "\u{feff}\u{feff}text\r\nline".as_bytes())],
        vec![part("Content-Disposition: form-data; name=\"blob\"; filename=\"b.txt\"\r\nContent-Transfer-Encoding: base64", b"aGVs bG8=\r\nd29y")],
        vec![part("Content-Disposition: form-data; name=\"blob\"; filename=\"b.txt\"\r\nContent-Transfer-Encoding: base64\r\nContent-Type: text/plain\t", "YW\u{141}J-_\u{13d}j".as_bytes())],
        vec![part("X-Other: 1\r\nContent-Disposition: form-data; name=\"a\"\r\nContent-Disposition: form-data; name=\"b\"; filename=\"c\"", b"")],
        vec![part("Content-Disposition: form-data; name=\"a\"x\nContent-Type: t/s", b"v")],
        vec![part("Content-Disposition: form-data; name=\"a\"; filename=  \"q\"", b"v")],
        vec![part("Content-Disposition: form-data; name=\"a\"; filename=x", b"v")],
        vec![part("Content-Disposition: form-data; name=\"a\"; other=\"1\"", b"v")],
        vec![part("Content-Disposition: form-data; name=\"metadata\"; filename=\"m.json\"\r\nContent-Type: application/json", "\u{feff}{\"a\":1}".as_bytes())],
        vec![],
    ];
    for parts in forms {
        let mut body = b"\r\n".to_vec();
        for part in &parts {
            body.extend_from_slice(part);
        }
        body.extend_from_slice(b"--B--\r\n\r\n");
        seeds.push(body);
    }
    seeds.push(b"--B--".to_vec());
    seeds.push(b"--B--\r\n".to_vec());
    seeds.push(b"--B\r\n\r\n--B--".to_vec());
    seeds.push(b"\r\n\r\n".to_vec());
    seeds.push(Vec::new());
    seeds.push(b"metadata=%7B%22a%22%3A1%7D&blob=x&&=&a&b=c=d+e%zz%E9%41".to_vec());
    seeds.push("?metadata=%22%E9\u{122}%22&metadata=2".as_bytes().to_vec());
    seeds.push("metadata=\u{e9}%E9&x=%ED%A0%80".as_bytes().to_vec());
    seeds
}

fn mutate(random: &mut common::Random, seed: &[u8]) -> Vec<u8> {
    let pieces: [&[u8]; 24] = [
        b"\r", b"\n", b"\r\n", b"-", b"--", b"--B", b"\"", b";", b" ", b"\t", b":", b"=", b"*", b"%", b"%0a", b"%22", b"'", b"\xc3\xa9", b"\xef\xbb\xbf",
        b"\xe9", b"utf-8''", b"filename", b"name=\"", b"B",
    ];
    let mut body = seed.to_vec();
    for _ in 0..=random.below(2) / 2 {
        let at = if body.is_empty() { 0 } else { random.below(body.len() as u64 + 1) as usize };
        match random.below(3) {
            0 => {
                let piece = pieces[random.below(pieces.len() as u64) as usize];
                body.splice(at..at, piece.iter().copied());
            }
            1 if at < body.len() => {
                let end = (at + 1 + random.below(4) as usize).min(body.len());
                body.drain(at..end);
            }
            _ if at < body.len() => body[at] = pieces[random.below(pieces.len() as u64) as usize][0],
            _ => {}
        }
    }
    body
}

#[test]
fn forms_parse_as_node_parses_them() {
    let types = [
        "multipart/form-data; boundary=B",
        "multipart/form-data;boundary=\"B\"",
        "Multipart/Form-Data; BOUNDARY=B; boundary=other",
        "multipart/form-data; charset=utf-8; boundary=B",
        "multipart/form-data; boundary=\"\\B\"",
        "multipart/form-data; boundary=\"B",
        "multipart/form-data; boundary=",
        "multipart/form-data; boundary= B",
        "multipart/form-data ; boundary=B",
        "multipart/form-data",
        "multipart/form-datax; boundary=B",
        "multipart/form-data; boundary=B, text/plain",
        "multipart/form-data; boundary=B, */*",
        "multipart/form-data; boundary=\"B,C\", x",
        "multipart/form-data, application/x-www-form-urlencoded",
        "multipart/form-data; boundary=\u{e9}",
        "multipart/form-data; boundary=\u{263a}",
        "multipart/form-data; =B; boundary=B",
        "multipart/form-data;;; boundary=B;",
        "\tmultipart/form-data; boundary=B \t",
        "application/x-www-form-urlencoded",
        "text/plain",
    ];
    let mut random = common::Random(0x666f_726d);
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();
    for seed in seeds() {
        for content_type in types {
            cases.push((content_type.to_owned(), seed.clone()));
        }
        for _ in 0..400 {
            let content_type = if random.below(4) == 0 { types[random.below(types.len() as u64) as usize] } else { types[0] };
            cases.push((content_type.to_owned(), mutate(&mut random, &seed)));
        }
    }
    let calls: Vec<(&str, Vec<Value>)> = cases.iter().map(|(content_type, body)| ("formData", vec![content_type.as_str().into(), Value::Bytes(body.clone())])).collect();
    let answers = common::calls(DRIVER, &calls);
    let (mut differences, mut parsed_forms) = (Vec::new(), 0);
    for ((content_type, body), js) in cases.iter().zip(&answers) {
        let rust = parsed(content_type, body);
        let same = match (&rust, js) {
            (Ok(rust), Ok(js)) => common::text(rust) == common::text(js),
            (Err(_), Err(_)) => true,
            _ => false,
        };
        parsed_forms += usize::from(js.is_ok());
        if !same && differences.len() < 20 {
            differences.push(format!("{content_type:?} {:?}\n  rust: {rust:?}\n  node: {js:?}", String::from_utf8_lossy(body)));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
    assert!(parsed_forms > cases.len() / 8, "only {parsed_forms} of {} cases parsed", cases.len());
}
