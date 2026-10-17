//! Base64 as Node's `Buffer` reads and writes it.

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn encode_with(bytes: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let joined = u32::from(chunk[0]) << 16 | u32::from(*chunk.get(1).unwrap_or(&0)) << 8 | u32::from(*chunk.get(2).unwrap_or(&0));
        for index in 0..=chunk.len() {
            out.push(alphabet[(joined >> (18 - 6 * index) & 63) as usize] as char);
        }
        if pad {
            for _ in chunk.len()..3 {
                out.push('=');
            }
        }
    }
    out
}

/// `buffer.toString('base64')`: the standard alphabet, padded.
pub fn encode(bytes: &[u8]) -> String {
    encode_with(bytes, STANDARD, true)
}

/// `buffer.toString('base64url')`: the URL alphabet, unpadded.
pub fn encode_url(bytes: &[u8]) -> String {
    encode_with(bytes, URL, false)
}

fn sextet(byte: u8) -> Option<u32> {
    Some(match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' | b'-' => 62,
        b'/' | b'_' => 63,
        _ => return None,
    } as u32)
}

/// `Buffer.from(text, 'base64')` and `Buffer.from(text, 'base64url')`, which read the same way: either alphabet,
/// padding optional, other characters skipped, and decoding stops at the first `=`. Like Node, a character outside
/// Latin-1 counts as the low byte of each of its UTF-16 code units.
pub fn decode(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut joined = 0_u32;
    let mut count = 0;
    for byte in text.encode_utf16().map(|unit| unit as u8) {
        if byte == b'=' {
            break;
        }
        let Some(value) = sextet(byte) else { continue };
        joined = joined << 6 | value;
        count += 1;
        if count == 4 {
            out.extend_from_slice(&[(joined >> 16) as u8, (joined >> 8) as u8, joined as u8]);
            joined = 0;
            count = 0;
        }
    }
    match count {
        2 => out.push((joined >> 4) as u8),
        3 => out.extend_from_slice(&[(joined >> 10) as u8, (joined >> 2) as u8]),
        _ => {}
    }
    out
}

/// `buffer.toString('hex')`.
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[usize::from(byte >> 4)] as char);
        out.push(DIGITS[usize::from(byte & 15)] as char);
    }
    out
}

/// `Buffer.from(text, 'hex')`: pairs of hex digits up to the first pair that is not one.
pub fn from_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map_while(|pair| Some(((pair[0] as char).to_digit(16)? << 4 | (pair[1] as char).to_digit(16)?) as u8))
        .collect()
}
