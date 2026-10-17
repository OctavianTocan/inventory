use mm_js::{Object, Value};

use crate::timestamp::{EXT_TIMESTAMP, timestamp_from_date};

/// The encoder options the backend passes; every other option keeps the JS codec's default.
#[derive(Clone, Copy, Debug, Default)]
pub struct EncodeOptions {
    /// `useBigInt64`: write a `BigInt` as int64 or uint64, and integers past 32 bits as float64.
    pub use_big_int64: bool,
}

/// What the JS encoder throws for a value it cannot write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeError(pub String);

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EncodeError {}

const MAX_DEPTH: usize = 100;

/// `encode(value, options)`.
///
/// # Errors
/// Nesting deeper than 100, a `BigInt` without `use_big_int64`, or a length past 32 bits.
pub fn encode(value: &Value, options: EncodeOptions) -> Result<Vec<u8>, EncodeError> {
    let mut out = Vec::new();
    encode_into(&mut out, value, options)?;
    Ok(out)
}

/// [`encode`] appended to `out`, which is left as it was on an error.
///
/// # Errors
/// As [`encode`].
pub fn encode_into(out: &mut Vec<u8>, value: &Value, options: EncodeOptions) -> Result<(), EncodeError> {
    let start = out.len();
    let result = Encoder { out, options }.value(value, 0);
    if result.is_err() {
        out.truncate(start);
    }
    result
}

struct Encoder<'a> {
    out: &'a mut Vec<u8>,
    options: EncodeOptions,
}

fn too_large(what: &str, size: usize) -> EncodeError {
    EncodeError(format!("Too large {what}: {size}"))
}

impl Encoder<'_> {
    fn value(&mut self, value: &Value, depth: usize) -> Result<(), EncodeError> {
        if depth > MAX_DEPTH {
            return Err(EncodeError(format!("Too deep objects in depth {depth}")));
        }
        match value {
            Value::Undefined | Value::Null => self.out.push(0xc0),
            Value::Bool(flag) => self.out.push(if *flag { 0xc3 } else { 0xc2 }),
            Value::Number(number) => self.number(*number),
            Value::String(text) => self.string(text)?,
            Value::BigInt(big) if self.options.use_big_int64 => {
                if *big >= 0 {
                    self.out.push(0xcf);
                    self.out.extend_from_slice(&(*big as u64).to_be_bytes());
                } else {
                    self.out.push(0xd3);
                    self.out.extend_from_slice(&(*big as i64).to_be_bytes());
                }
            }
            Value::BigInt(_) => return Err(EncodeError("Unrecognized object: [object BigInt]".into())),
            Value::Date(ms) => self.extension(EXT_TIMESTAMP, &timestamp_from_date(*ms))?,
            Value::Ext(kind, data) => self.extension(*kind, data)?,
            Value::Array(items) => {
                self.header(items.len(), 0x90, 16, 0xdc, 0xdd).map_err(|size| too_large("array", size))?;
                for item in items {
                    self.value(item, depth + 1)?;
                }
            }
            Value::Bytes(bytes) => {
                let size = bytes.len();
                match size {
                    0..0x100 => self.out.extend_from_slice(&[0xc4, size as u8]),
                    0x100..0x10000 => self.with_u16(0xc5, size),
                    _ if (size as u64) < 1 << 32 => self.with_u32(0xc6, size),
                    _ => return Err(too_large("binary", size)),
                }
                self.out.extend_from_slice(bytes);
            }
            Value::Object(object) => self.map(object, depth)?,
        }
        Ok(())
    }

    fn with_u16(&mut self, byte: u8, size: usize) {
        self.out.push(byte);
        self.out.extend_from_slice(&(size as u16).to_be_bytes());
    }

    fn with_u32(&mut self, byte: u8, size: usize) {
        self.out.push(byte);
        self.out.extend_from_slice(&(size as u32).to_be_bytes());
    }

    fn header(&mut self, size: usize, fix: u8, fix_limit: usize, b16: u8, b32: u8) -> Result<(), usize> {
        if size < fix_limit {
            self.out.push(fix + size as u8);
        } else if size < 0x10000 {
            self.with_u16(b16, size);
        } else if (size as u64) < 1 << 32 {
            self.with_u32(b32, size);
        } else {
            return Err(size);
        }
        Ok(())
    }

    fn number(&mut self, number: f64) {
        if mm_js::number::is_safe_integer(number) {
            let big = self.options.use_big_int64;
            if number >= 0.0 {
                let value = number as u64;
                match value {
                    0..0x80 => return self.out.push(value as u8),
                    0x80..0x100 => return self.out.extend_from_slice(&[0xcc, value as u8]),
                    0x100..0x10000 => return self.with_u16(0xcd, value as usize),
                    0x10000..0x1_0000_0000 => return self.with_u32(0xce, value as usize),
                    _ if !big => {
                        self.out.push(0xcf);
                        return self.out.extend_from_slice(&value.to_be_bytes());
                    }
                    _ => {}
                }
            } else {
                let value = number as i64;
                match value {
                    -0x20..0 => return self.out.push(value as i8 as u8),
                    -0x80..-0x20 => return self.out.extend_from_slice(&[0xd0, value as i8 as u8]),
                    -0x8000..-0x80 => {
                        self.out.push(0xd1);
                        return self.out.extend_from_slice(&(value as i16).to_be_bytes());
                    }
                    -0x8000_0000..-0x8000 => {
                        self.out.push(0xd2);
                        return self.out.extend_from_slice(&(value as i32).to_be_bytes());
                    }
                    _ if !big => {
                        self.out.push(0xd3);
                        return self.out.extend_from_slice(&value.to_be_bytes());
                    }
                    _ => {}
                }
            }
        }
        self.out.push(0xcb);
        self.out.extend_from_slice(&number.to_be_bytes());
    }

    fn string(&mut self, text: &str) -> Result<(), EncodeError> {
        let size = text.len();
        match size {
            0..32 => self.out.push(0xa0 + size as u8),
            32..0x100 => self.out.extend_from_slice(&[0xd9, size as u8]),
            0x100..0x10000 => self.with_u16(0xda, size),
            _ if (size as u64) < 1 << 32 => self.with_u32(0xdb, size),
            _ => return Err(too_large("string", size)),
        }
        self.out.extend_from_slice(text.as_bytes());
        Ok(())
    }

    fn extension(&mut self, kind: i8, data: &[u8]) -> Result<(), EncodeError> {
        let size = data.len();
        match size {
            1 => self.out.push(0xd4),
            2 => self.out.push(0xd5),
            4 => self.out.push(0xd6),
            8 => self.out.push(0xd7),
            16 => self.out.push(0xd8),
            0..0x100 => self.out.extend_from_slice(&[0xc7, size as u8]),
            0x100..0x10000 => self.with_u16(0xc8, size),
            _ if (size as u64) < 1 << 32 => self.with_u32(0xc9, size),
            _ => return Err(EncodeError(format!("Too large extension object: {size}"))),
        }
        self.out.push(kind as u8);
        self.out.extend_from_slice(data);
        Ok(())
    }

    fn map(&mut self, object: &Object, depth: usize) -> Result<(), EncodeError> {
        self.header(object.len(), 0x80, 16, 0xde, 0xdf).map_err(|size| too_large("map object", size))?;
        for (key, value) in object.iter() {
            self.string(&key)?;
            self.value(value, depth + 1)?;
        }
        Ok(())
    }
}
