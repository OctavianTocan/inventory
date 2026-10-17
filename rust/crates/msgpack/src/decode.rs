use mm_js::{Object, Value};

use crate::timestamp::{EXT_TIMESTAMP, date_from_timestamp};

/// The decoder options the backend passes; every other option keeps the JS codec's default.
#[derive(Clone, Copy, Debug, Default)]
pub struct DecodeOptions {
    /// `useBigInt64`: read int64 and uint64 as `BigInt` instead of a possibly rounded number.
    pub use_big_int64: bool,
}

/// Why the JS decoder threw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The input ended inside a value.
    Insufficient,
    /// Malformed input, with the JS codec's message.
    Invalid(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insufficient => f.write_str("Insufficient data"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DecodeError {}

/// `decode(bytes, options)`: exactly one value.
///
/// # Errors
/// Malformed or truncated input, or bytes left after the value.
pub fn decode(bytes: &[u8], options: DecodeOptions) -> Result<Value, DecodeError> {
    let mut decoder = Decoder { bytes, at: 0, options };
    let value = decoder.value()?;
    if decoder.at < bytes.len() {
        return Err(DecodeError::Invalid(format!(
            "Extra {} of {} byte(s) found at buffer[{}]",
            bytes.len() - decoder.at,
            bytes.len(),
            decoder.at
        )));
    }
    Ok(value)
}

/// `[...decodeMulti(bytes, options)]`: every value in a stream of concatenated values.
///
/// # Errors
/// The first malformed or truncated value; the JS spread throws without keeping the values before it.
pub fn decode_multi(bytes: &[u8], options: DecodeOptions) -> Result<Vec<Value>, DecodeError> {
    let mut decoder = Decoder { bytes, at: 0, options };
    let mut values = Vec::new();
    while decoder.at < bytes.len() {
        values.push(decoder.value()?);
    }
    Ok(values)
}

enum Frame {
    Array(Vec<Value>, usize),
    Key(Object, usize),
    Entry(Object, usize, String),
}

struct Decoder<'a> {
    bytes: &'a [u8],
    at: usize,
    options: DecodeOptions,
}

fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Undefined => "undefined",
        Value::Bool(_) => "boolean",
        Value::BigInt(_) => "bigint",
        _ => "object",
    }
}

impl Decoder<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let slice = self.bytes.get(self.at..self.at + N).ok_or(DecodeError::Insufficient)?;
        self.at += N;
        Ok(slice.try_into().expect("N bytes"))
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take::<1>()?[0])
    }

    fn u16(&mut self) -> Result<usize, DecodeError> {
        Ok(usize::from(u16::from_be_bytes(self.take()?)))
    }

    fn u32(&mut self) -> Result<usize, DecodeError> {
        Ok(u32::from_be_bytes(self.take()?) as usize)
    }

    fn slice(&mut self, size: usize) -> Result<&[u8], DecodeError> {
        let end = self.at.checked_add(size).ok_or(DecodeError::Insufficient)?;
        let slice = self.bytes.get(self.at..end).ok_or(DecodeError::Insufficient)?;
        self.at = end;
        Ok(slice)
    }

    fn string(&mut self, size: usize) -> Result<String, DecodeError> {
        if self.bytes.len() < self.at + size {
            return Err(DecodeError::Insufficient);
        }
        let text = utf8(self.bytes, self.at, size);
        self.at += size;
        Ok(text)
    }

    fn extension(&mut self, size: usize) -> Result<Value, DecodeError> {
        let kind = self.u8()? as i8;
        let data = self.slice(size)?;
        if kind == EXT_TIMESTAMP {
            return date_from_timestamp(data).map(Value::Date).map_err(DecodeError::Invalid);
        }
        Ok(Value::Ext(kind, data.to_vec()))
    }

    fn int64(&mut self, signed: bool) -> Result<Value, DecodeError> {
        let bytes: [u8; 8] = self.take()?;
        if self.options.use_big_int64 {
            return Ok(Value::BigInt(if signed { i128::from(i64::from_be_bytes(bytes)) } else { i128::from(u64::from_be_bytes(bytes)) }));
        }
        let low = f64::from(u32::from_be_bytes(bytes[4..].try_into().expect("four bytes")));
        let high = if signed {
            f64::from(i32::from_be_bytes(bytes[..4].try_into().expect("four bytes")))
        } else {
            f64::from(u32::from_be_bytes(bytes[..4].try_into().expect("four bytes")))
        };
        Ok(Value::Number(high * 4_294_967_296.0 + low))
    }

    fn value(&mut self) -> Result<Value, DecodeError> {
        let mut stack: Vec<Frame> = Vec::new();
        loop {
            let head = self.u8()?;
            let mut value = match head {
                0x00..=0x7f => Value::Number(f64::from(head)),
                0x80..=0x8f | 0xde | 0xdf => {
                    let size = match head {
                        0xde => self.u16()?,
                        0xdf => self.u32()?,
                        _ => usize::from(head - 0x80),
                    };
                    if size > 0 {
                        stack.push(Frame::Key(Object::new(), size));
                        continue;
                    }
                    Value::Object(Object::new())
                }
                0x90..=0x9f | 0xdc | 0xdd => {
                    let size = match head {
                        0xdc => self.u16()?,
                        0xdd => self.u32()?,
                        _ => usize::from(head - 0x90),
                    };
                    if size > 0 {
                        stack.push(Frame::Array(Vec::with_capacity(size.min(4096)), size));
                        continue;
                    }
                    Value::Array(Vec::new())
                }
                0xa0..=0xbf => Value::String(self.string(usize::from(head - 0xa0))?),
                0xd9 => {
                    let size = usize::from(self.u8()?);
                    Value::String(self.string(size)?)
                }
                0xda => {
                    let size = self.u16()?;
                    Value::String(self.string(size)?)
                }
                0xdb => {
                    let size = self.u32()?;
                    Value::String(self.string(size)?)
                }
                0xc0 => Value::Null,
                0xc2 => Value::Bool(false),
                0xc3 => Value::Bool(true),
                0xca => Value::Number(f64::from(f32::from_be_bytes(self.take()?))),
                0xcb => Value::Number(f64::from_be_bytes(self.take()?)),
                0xcc => Value::Number(f64::from(self.u8()?)),
                0xcd => Value::Number(self.u16()? as f64),
                0xce => Value::Number(self.u32()? as f64),
                0xcf => self.int64(false)?,
                0xd0 => Value::Number(f64::from(self.u8()? as i8)),
                0xd1 => Value::Number(f64::from(i16::from_be_bytes(self.take()?))),
                0xd2 => Value::Number(f64::from(i32::from_be_bytes(self.take()?))),
                0xd3 => self.int64(true)?,
                0xe0..=0xff => Value::Number(f64::from(head as i8)),
                0xc4..=0xc6 => {
                    let size = match head {
                        0xc4 => usize::from(self.u8()?),
                        0xc5 => self.u16()?,
                        _ => self.u32()?,
                    };
                    Value::Bytes(self.slice(size)?.to_vec())
                }
                0xd4 => self.extension(1)?,
                0xd5 => self.extension(2)?,
                0xd6 => self.extension(4)?,
                0xd7 => self.extension(8)?,
                0xd8 => self.extension(16)?,
                0xc7..=0xc9 => {
                    let size = match head {
                        0xc7 => usize::from(self.u8()?),
                        0xc8 => self.u16()?,
                        _ => self.u32()?,
                    };
                    self.extension(size)?
                }
                0xc1 => return Err(DecodeError::Invalid(format!("Unrecognized type byte: {}", pretty_byte(head)))),
            };
            loop {
                match stack.pop() {
                    None => return Ok(value),
                    Some(Frame::Array(mut items, size)) => {
                        items.push(value);
                        if items.len() < size {
                            stack.push(Frame::Array(items, size));
                            break;
                        }
                        value = Value::Array(items);
                    }
                    Some(Frame::Key(object, remaining)) => {
                        let key = match value {
                            Value::String(key) if key == "__proto__" => {
                                return Err(DecodeError::Invalid("The key __proto__ is not allowed".into()));
                            }
                            Value::String(key) => key,
                            Value::Number(number) => mm_js::number_to_string(number),
                            other => {
                                return Err(DecodeError::Invalid(format!(
                                    "The type of key must be string or number but {}",
                                    type_of(&other)
                                )));
                            }
                        };
                        stack.push(Frame::Entry(object, remaining, key));
                        break;
                    }
                    Some(Frame::Entry(mut object, remaining, key)) => {
                        object.insert(key, value);
                        if remaining > 1 {
                            stack.push(Frame::Key(object, remaining - 1));
                            break;
                        }
                        value = Value::Object(object);
                    }
                }
            }
        }
    }
}

fn pretty_byte(byte: u8) -> String {
    format!("0x{byte:02x}")
}

/// The JS codec's string decoding: its own lenient decoder up to 200 bytes, `TextDecoder` above.
fn utf8(bytes: &[u8], start: usize, size: usize) -> String {
    let slice = &bytes[start..start + size];
    if size > 200 {
        return mm_js::text::decode_utf8(slice);
    }
    if slice.is_ascii() {
        return String::from_utf8(slice.to_vec()).expect("ASCII");
    }
    // Truncated sequences read past the string into the bytes after it, as the JS decoder does.
    let byte = |at: usize| u32::from(*bytes.get(at).unwrap_or(&0));
    let end = start + size;
    let mut at = start;
    let mut units: Vec<u16> = Vec::with_capacity(size);
    while at < end {
        let first = byte(at);
        at += 1;
        if first & 0x80 == 0 {
            units.push(first as u16);
        } else if first & 0xe0 == 0xc0 {
            let second = byte(at) & 0x3f;
            at += 1;
            units.push(((first & 0x1f) << 6 | second) as u16);
        } else if first & 0xf0 == 0xe0 {
            let (second, third) = (byte(at) & 0x3f, byte(at + 1) & 0x3f);
            at += 2;
            units.push(((first & 0x1f) << 12 | second << 6 | third) as u16);
        } else if first & 0xf8 == 0xf0 {
            let (second, third, fourth) = (byte(at) & 0x3f, byte(at + 1) & 0x3f, byte(at + 2) & 0x3f);
            at += 3;
            let mut unit = (first & 0x07) << 18 | second << 12 | third << 6 | fourth;
            if unit > 0xffff {
                unit -= 0x10000;
                units.push((unit >> 10 & 0x3ff | 0xd800) as u16);
                unit = 0xdc00 | (unit & 0x3ff);
            }
            units.push(unit as u16);
        } else {
            units.push(first as u16);
        }
    }
    String::from_utf16_lossy(&units)
}
