use std::fmt::Write;

use crate::{Object, Value, number_to_string};

/// Why `JSON.stringify` refused a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringifyError(pub &'static str);

/// `JSON.stringify(value)`: `None` where JS returns `undefined` (for `undefined` itself).
///
/// # Errors
/// A `BigInt` anywhere in the value, as JS throws a `TypeError`.
pub fn stringify(value: &Value) -> Result<Option<String>, StringifyError> {
    stringify_indented(value, "")
}

/// `JSON.stringify(value, null, indent)` with `indent` as the gap string (`"  "` for 2).
///
/// # Errors
/// A `BigInt` anywhere in the value.
pub fn stringify_indented(value: &Value, indent: &str) -> Result<Option<String>, StringifyError> {
    let mut out = String::new();
    let written = write_value(&mut out, value, indent, "")?;
    Ok(written.then_some(out))
}

/// `JSON.stringify(value)` for values the caller knows hold no `BigInt` and are not `undefined`.
///
/// # Panics
/// If the value holds a `BigInt` or is `undefined`.
pub fn to_string(value: &Value) -> String {
    stringify(value).expect("value holds no BigInt").expect("value is not undefined")
}

/// `JSON.stringify(text)`: the string as a quoted JSON literal.
pub fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    write_quoted(&mut out, text);
    out
}

fn write_value(out: &mut String, value: &Value, indent: &str, current: &str) -> Result<bool, StringifyError> {
    match value {
        Value::Undefined => return Ok(false),
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => {
            if number.is_finite() {
                out.push_str(&number_to_string(*number));
            } else {
                out.push_str("null");
            }
        }
        Value::BigInt(_) => return Err(StringifyError("Do not know how to serialize a BigInt")),
        Value::String(text) => write_quoted(out, text),
        Value::Date(ms) => match crate::date::iso_from_ms(*ms) {
            Some(iso) => write_quoted(out, &iso),
            None => out.push_str("null"),
        },
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(true);
            }
            let inner = format!("{current}{indent}");
            out.push('[');
            for (position, item) in items.iter().enumerate() {
                if position > 0 {
                    out.push(',');
                }
                if !indent.is_empty() {
                    out.push('\n');
                    out.push_str(&inner);
                }
                if !write_value(out, item, indent, &inner)? {
                    out.push_str("null");
                }
            }
            if !indent.is_empty() {
                out.push('\n');
                out.push_str(current);
            }
            out.push(']');
        }
        Value::Object(object) => write_object(out, object.iter(), indent, current)?,
        Value::Bytes(bytes) => write_object(out, bytes_entries(bytes), indent, current)?,
        Value::Ext(kind, data) => {
            let entries = [
                ("type".into(), Value::Number(f64::from(*kind))),
                ("data".into(), Value::Bytes(data.clone())),
            ];
            write_object(out, entries.into_iter(), indent, current)?;
        }
    }
    Ok(true)
}

fn bytes_entries(bytes: &[u8]) -> impl Iterator<Item = (std::borrow::Cow<'static, str>, Value)> + '_ {
    bytes.iter().enumerate().map(|(index, byte)| (std::borrow::Cow::Owned(index.to_string()), Value::Number(f64::from(*byte))))
}

fn write_object<'a, V: std::borrow::Borrow<Value>>(
    out: &mut String,
    entries: impl Iterator<Item = (std::borrow::Cow<'a, str>, V)>,
    indent: &str,
    current: &str,
) -> Result<(), StringifyError> {
    let inner = format!("{current}{indent}");
    let start = out.len();
    out.push('{');
    let mut any = false;
    for (key, value) in entries {
        let value = value.borrow();
        if value.is_undefined() {
            continue;
        }
        if any {
            out.push(',');
        }
        if !indent.is_empty() {
            out.push('\n');
            out.push_str(&inner);
        }
        write_quoted(out, &key);
        out.push(':');
        if !indent.is_empty() {
            out.push(' ');
        }
        write_value(out, value, indent, &inner)?;
        any = true;
    }
    if !any {
        out.truncate(start);
        out.push_str("{}");
        return Ok(());
    }
    if !indent.is_empty() {
        out.push('\n');
        out.push_str(current);
    }
    out.push('}');
    Ok(())
}

fn write_quoted(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Why `JSON.parse` refused its input, in V8's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset of the first character the parser could not accept.
    pub position: usize,
    message: String,
}

impl ParseError {
    /// The `SyntaxError`'s message, as `error.message` reads it.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

/// `JSON.parse(text)`. Lone surrogates in `\u` escapes, which a Rust string cannot hold, become U+FFFD.
///
/// # Errors
/// Text that is not JSON, at the position and with the message V8 gives.
pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut parser = Parser { bytes: text.as_bytes(), text, at: 0 };
    parser.skip_space();
    let value = parser.value()?;
    parser.skip_space();
    if parser.at != parser.bytes.len() {
        return parser.fail(Problem::Trailing);
    }
    Ok(value)
}

#[derive(Clone, Copy)]
enum Problem {
    Token,
    PropertyName,
    QuotedName,
    Colon,
    CommaBrace,
    CommaBracket,
    Trailing,
    Unterminated,
    Control,
    Escape,
    Unicode,
    Minus,
    Fraction,
    Exponent,
    Number,
}

const CONTEXT: usize = 10;

// V8's `JsonParser::ReportUnexpectedToken` message for the UTF-16 unit at `position`.
fn unexpected_token(text: &str, position: usize) -> String {
    if matches!(text, "NaN" | "Infinity" | "undefined" | "[object Object]") {
        return format!("\"{text}\" is not valid JSON");
    }
    let units: Vec<u16> = text.encode_utf16().collect();
    let token = String::from_utf16_lossy(&units[position..=position]);
    let piece = |start: usize, end: usize| String::from_utf16_lossy(&units[start..end.min(units.len())]);
    if units.len() <= CONTEXT * 2 {
        format!("Unexpected token '{token}', \"{text}\" is not valid JSON")
    } else if position < CONTEXT {
        format!("Unexpected token '{token}', \"{}\"... is not valid JSON", piece(0, position + CONTEXT))
    } else if position < units.len() - CONTEXT {
        format!("Unexpected token '{token}', ...\"{}\"... is not valid JSON", piece(position - CONTEXT, position + CONTEXT))
    } else {
        format!("Unexpected token '{token}', ...\"{}\" is not valid JSON", piece(position - CONTEXT, units.len()))
    }
}

// `what` and ` at position N (line L column C)`, counted in UTF-16 units with `\r\n`, `\r` and `\n` ending lines.
fn located(what: &str, units: &[u16], position: usize) -> String {
    let (mut line, mut start) = (1, 0);
    for index in 0..position {
        let unit = units[index];
        if unit == u16::from(b'\r') && units.get(index + 1) == Some(&u16::from(b'\n')) {
            continue;
        }
        if unit == u16::from(b'\n') || unit == u16::from(b'\r') {
            line += 1;
            start = index + 1;
        }
    }
    format!("{what} at position {position} (line {line} column {})", position - start + 1)
}

struct Parser<'a> {
    bytes: &'a [u8],
    text: &'a str,
    at: usize,
}

impl Parser<'_> {
    fn fail<T>(&self, problem: Problem) -> Result<T, ParseError> {
        let position = self.text[..self.at].encode_utf16().count();
        let what = match problem {
            Problem::Token if self.at >= self.bytes.len() => return self.error("Unexpected end of JSON input".to_owned()),
            Problem::Token => return self.error(unexpected_token(self.text, position)),
            Problem::PropertyName => "Expected property name or '}' in JSON",
            Problem::QuotedName => "Expected double-quoted property name in JSON",
            Problem::Colon => "Expected ':' after property name in JSON",
            Problem::CommaBrace => "Expected ',' or '}' after property value in JSON",
            Problem::CommaBracket => "Expected ',' or ']' after array element in JSON",
            Problem::Trailing => "Unexpected non-whitespace character after JSON",
            Problem::Unterminated => "Unterminated string in JSON",
            Problem::Control => "Bad control character in string literal in JSON",
            Problem::Escape => "Bad escaped character in JSON",
            Problem::Unicode => "Bad Unicode escape in JSON",
            Problem::Minus => "No number after minus sign in JSON",
            Problem::Fraction => "Unterminated fractional number in JSON",
            Problem::Exponent => "Exponent part is missing a number in JSON",
            Problem::Number => "Unexpected number in JSON",
        };
        let units: Vec<u16> = self.text.encode_utf16().collect();
        self.error(located(what, &units, position))
    }

    fn error<T>(&self, message: String) -> Result<T, ParseError> {
        Err(ParseError { position: self.at, message })
    }

    fn skip_space(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.bytes.get(self.at) {
            self.at += 1;
        }
    }

    fn literal(&mut self, word: &str, value: Value) -> Result<Value, ParseError> {
        for &expected in word.as_bytes() {
            if self.bytes.get(self.at) != Some(&expected) {
                return self.fail(Problem::Token);
            }
            self.at += 1;
        }
        Ok(value)
    }

    fn value(&mut self) -> Result<Value, ParseError> {
        match self.bytes.get(self.at) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => self.fail(Problem::Token),
        }
    }

    fn object(&mut self) -> Result<Value, ParseError> {
        self.at += 1;
        let mut object = Object::new();
        self.skip_space();
        match self.bytes.get(self.at) {
            Some(b'}') => {
                self.at += 1;
                return Ok(Value::Object(object));
            }
            Some(b'"') => {}
            _ => return self.fail(Problem::PropertyName),
        }
        loop {
            let key = self.string()?;
            self.skip_space();
            if self.bytes.get(self.at) != Some(&b':') {
                return self.fail(Problem::Colon);
            }
            self.at += 1;
            self.skip_space();
            let value = self.value()?;
            object.insert(key, value);
            self.skip_space();
            match self.bytes.get(self.at) {
                Some(b',') => {
                    self.at += 1;
                    self.skip_space();
                    if self.bytes.get(self.at) != Some(&b'"') {
                        return self.fail(Problem::QuotedName);
                    }
                }
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Value::Object(object));
                }
                _ => return self.fail(Problem::CommaBrace),
            }
        }
    }

    fn array(&mut self) -> Result<Value, ParseError> {
        self.at += 1;
        let mut items = Vec::new();
        self.skip_space();
        if self.bytes.get(self.at) == Some(&b']') {
            self.at += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_space();
            items.push(self.value()?);
            self.skip_space();
            match self.bytes.get(self.at) {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Value::Array(items));
                }
                _ => return self.fail(Problem::CommaBracket),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let mut unit = 0;
        for _ in 0..4 {
            match self.bytes.get(self.at).and_then(|&byte| char::from(byte).to_digit(16)) {
                Some(digit) => unit = unit * 16 + digit,
                None => return self.fail(Problem::Unicode),
            }
            self.at += 1;
        }
        Ok(unit)
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.at += 1;
        let mut out = String::new();
        loop {
            let start = self.at;
            while let Some(&byte) = self.bytes.get(self.at) {
                if byte == b'"' || byte == b'\\' || byte < 0x20 {
                    break;
                }
                self.at += 1;
            }
            out.push_str(&self.text[start..self.at]);
            match self.bytes.get(self.at) {
                Some(b'"') => {
                    self.at += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.at += 1;
                    let Some(&escape) = self.bytes.get(self.at) else { return self.fail(Problem::Token) };
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            self.at += 1;
                            let unit = self.hex4()?;
                            if (0xd800..0xdc00).contains(&unit) && self.text[self.at..].starts_with("\\u") {
                                let save = self.at;
                                self.at += 2;
                                let low = self.hex4()?;
                                if (0xdc00..0xe000).contains(&low) {
                                    let code = 0x10000 + ((unit - 0xd800) << 10) + (low - 0xdc00);
                                    out.push(char::from_u32(code).expect("valid surrogate pair"));
                                    continue;
                                }
                                self.at = save;
                            }
                            out.push(char::from_u32(unit).unwrap_or('\u{fffd}'));
                            continue;
                        }
                        _ => return self.fail(Problem::Escape),
                    }
                    self.at += 1;
                }
                Some(_) => return self.fail(Problem::Control),
                None => return self.fail(Problem::Unterminated),
            }
        }
    }

    fn number(&mut self) -> Result<Value, ParseError> {
        let start = self.at;
        if self.bytes[self.at] == b'-' {
            self.at += 1;
        }
        match self.bytes.get(self.at) {
            Some(b'0') => {
                self.at += 1;
                if let Some(b'0'..=b'9') = self.bytes.get(self.at) {
                    return self.fail(Problem::Number);
                }
            }
            Some(b'1'..=b'9') => {
                while let Some(b'0'..=b'9') = self.bytes.get(self.at) {
                    self.at += 1;
                }
            }
            _ => return self.fail(Problem::Minus),
        }
        if self.bytes.get(self.at) == Some(&b'.') {
            self.at += 1;
            if !matches!(self.bytes.get(self.at), Some(b'0'..=b'9')) {
                return self.fail(Problem::Fraction);
            }
            while let Some(b'0'..=b'9') = self.bytes.get(self.at) {
                self.at += 1;
            }
        }
        if let Some(b'e' | b'E') = self.bytes.get(self.at) {
            self.at += 1;
            if let Some(b'+' | b'-') = self.bytes.get(self.at) {
                self.at += 1;
            }
            if !matches!(self.bytes.get(self.at), Some(b'0'..=b'9')) {
                return self.fail(Problem::Exponent);
            }
            while let Some(b'0'..=b'9') = self.bytes.get(self.at) {
                self.at += 1;
            }
        }
        Ok(Value::Number(self.text[start..self.at].parse().expect("JSON number grammar")))
    }
}
