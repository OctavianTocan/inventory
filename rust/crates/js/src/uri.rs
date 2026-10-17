/// The `URIError` JS throws for a malformed escape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriError;

impl std::fmt::Display for UriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("URI malformed")
    }
}

impl std::error::Error for UriError {}

const COMPONENT_SAFE: &[u8] = b"-_.!~*'()";
const URI_SAFE: &[u8] = b"-_.!~*'();/?:@&=+$,#";

fn encode(text: &str, safe: &[u8]) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || safe.contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `encodeURIComponent(text)`.
pub fn encode_uri_component(text: &str) -> String {
    encode(text, COMPONENT_SAFE)
}

/// `encodeURI(text)`.
pub fn encode_uri(text: &str) -> String {
    encode(text, URI_SAFE)
}

fn hex(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|digit| digit as u8)
}

fn decode(text: &str, reserved: &[u8]) -> Result<String, UriError> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    let escaped = |at: usize| -> Result<u8, UriError> {
        if bytes.get(at) != Some(&b'%') {
            return Err(UriError);
        }
        match (bytes.get(at + 1).copied().and_then(hex), bytes.get(at + 2).copied().and_then(hex)) {
            (Some(high), Some(low)) => Ok(high << 4 | low),
            _ => Err(UriError),
        }
    };
    while at < bytes.len() {
        if bytes[at] != b'%' {
            out.push(bytes[at]);
            at += 1;
            continue;
        }
        let first = escaped(at)?;
        if first < 0x80 {
            if reserved.contains(&first) {
                out.extend_from_slice(&bytes[at..at + 3]);
            } else {
                out.push(first);
            }
            at += 3;
            continue;
        }
        let length = match first {
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            _ => return Err(UriError),
        };
        let mut sequence = vec![first];
        for index in 1..length {
            let next = escaped(at + 3 * index)?;
            if next & 0xc0 != 0x80 {
                return Err(UriError);
            }
            sequence.push(next);
        }
        // Rust's UTF-8 check refuses overlong forms and surrogates, as the JS decoder does.
        std::str::from_utf8(&sequence).map_err(|_| UriError)?;
        out.extend_from_slice(&sequence);
        at += 3 * length;
    }
    String::from_utf8(out).map_err(|_| UriError)
}

/// `decodeURIComponent(text)`.
///
/// # Errors
/// A `%` not followed by two hex digits, or escapes that are not UTF-8.
pub fn decode_uri_component(text: &str) -> Result<String, UriError> {
    decode(text, b"")
}

/// `decodeURI(text)`: like [`decode_uri_component`] but escapes of `;/?:@&=+$,#` stay escaped.
///
/// # Errors
/// A `%` not followed by two hex digits, or escapes that are not UTF-8.
pub fn decode_uri(text: &str) -> Result<String, UriError> {
    decode(text, b";/?:@&=+$,#")
}
