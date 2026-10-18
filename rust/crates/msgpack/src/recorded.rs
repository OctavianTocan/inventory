use mm_js::Value;

/// The replica's own encoder for recorded cards (the historical JS `encodeMsgpack`), appended to `out`: fixints for
/// 0 to 127 and float64 for every other number, str8 only for 32 to 255 bytes, and anything that is not JSON
/// written as the map of its own enumerable properties, as `Object.entries` sees it.
pub fn encode_recorded(out: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Undefined | Value::Null => out.push(0xc0),
        Value::Bool(flag) => out.push(if *flag { 0xc3 } else { 0xc2 }),
        Value::Number(number) => {
            if number.fract() == 0.0 && *number >= 0.0 && *number < 128.0 {
                out.push(*number as u8);
            } else {
                out.push(0xcb);
                out.extend_from_slice(&number.to_be_bytes());
            }
        }
        Value::String(text) => string(out, text),
        Value::Array(items) => {
            header(out, 0x90, 0xdc, 0xdd, items.len(), 15);
            for item in items {
                encode_recorded(out, item);
            }
        }
        Value::Object(object) => {
            header(out, 0x80, 0xde, 0xdf, object.len(), 15);
            for (key, item) in object.iter() {
                string(out, &key);
                encode_recorded(out, item);
            }
        }
        Value::Bytes(bytes) => {
            header(out, 0x80, 0xde, 0xdf, bytes.len(), 15);
            for (index, byte) in bytes.iter().enumerate() {
                string(out, &index.to_string());
                encode_recorded(out, &Value::Number(f64::from(*byte)));
            }
        }
        Value::Ext(kind, data) => {
            out.push(0x82);
            string(out, "type");
            encode_recorded(out, &Value::Number(f64::from(*kind)));
            string(out, "data");
            encode_recorded(out, &Value::Bytes(data.clone()));
        }
        Value::BigInt(_) | Value::Date(_) => out.push(0x80),
    }
}

fn header(out: &mut Vec<u8>, fix: u8, b16: u8, b32: u8, size: usize, fix_max: usize) {
    if size <= fix_max {
        out.push(fix | size as u8);
    } else if size < 0x10000 {
        out.push(b16);
        out.extend_from_slice(&(size as u16).to_be_bytes());
    } else {
        out.push(b32);
        out.extend_from_slice(&(size as u32).to_be_bytes());
    }
}

fn string(out: &mut Vec<u8>, text: &str) {
    let size = text.len();
    if (32..0x100).contains(&size) {
        out.extend_from_slice(&[0xd9, size as u8]);
    } else {
        header(out, 0xa0, 0xda, 0xdb, size, 31);
    }
    out.extend_from_slice(text.as_bytes());
}
