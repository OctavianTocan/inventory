//! Port of the policy half of `lib/capture-http.mjs`: which destinations link capture may contact.

use std::sync::LazyLock;

use mm_js::{Value, text};
use regex::Regex;

use crate::capture_html::source_url;
use crate::{Error, Result};

/// The `code` of the error a rejected destination raises.
pub const CAPTURE_BLOCKED: &str = "CAPTURE_BLOCKED";

static RAW_AUTHORITY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i-u:https?)://([^/?#]+)").expect("pattern"));
static PORT_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":[0-9]+$").expect("pattern"));

const BLOCKED_V4: [([u8; 4], u32); 14] = [
    ([0, 0, 0, 0], 8),
    ([10, 0, 0, 0], 8),
    ([100, 64, 0, 0], 10),
    ([127, 0, 0, 0], 8),
    ([169, 254, 0, 0], 16),
    ([172, 16, 0, 0], 12),
    ([192, 0, 0, 0], 24),
    ([192, 0, 2, 0], 24),
    ([192, 88, 99, 0], 24),
    ([192, 168, 0, 0], 16),
    ([198, 18, 0, 0], 15),
    ([198, 51, 100, 0], 24),
    ([203, 0, 113, 0], 24),
    ([224, 0, 0, 0], 3),
];

/// The error a rejected destination raises (`forbidden`); it never names the destination.
pub fn forbidden() -> Error {
    Error::status(400, "Unsupported capture destination").with_code(CAPTURE_BLOCKED)
}

/// Whether an error is a `CAPTURE_BLOCKED` rejection (`error.code === 'CAPTURE_BLOCKED'`).
///
/// # Arguments
/// * `error` - Any error from a capture operation.
pub fn is_blocked(error: &Error) -> bool {
    matches!(error, Error::Status { code: Some(CAPTURE_BLOCKED), .. })
}

/// A dotted-decimal IPv4 literal as Node's `isIPv4` accepts it: four segments of 0 to 255 without leading zeros.
fn parse_v4(text: &str) -> Option<[u8; 4]> {
    let mut out = [0u8; 4];
    let mut parts = text.split('.');
    for slot in &mut out {
        let part = parts.next()?;
        if part.is_empty() || part.len() > 3 || !part.bytes().all(|byte| byte.is_ascii_digit()) || (part.len() > 1 && part.starts_with('0')) {
            return None;
        }
        *slot = u8::try_from(part.parse::<u16>().ok()?).ok()?;
    }
    parts.next().is_none().then_some(out)
}

/// The eight groups of an IPv6 literal as Node's `isIPv6` accepts it, with an optional `%zone` suffix.
fn parse_v6(text: &str) -> Option<[u16; 8]> {
    let (address, zone) = match text.split_once('%') {
        Some((address, zone)) => (address, Some(zone)),
        None => (text, None),
    };
    if zone.is_some_and(|zone| zone.is_empty() || !zone.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b':'))) {
        return None;
    }
    let (head, tail) = match address.split_once("::") {
        Some((head, tail)) => (head, Some(tail)),
        None => (address, None),
    };
    if tail.is_some_and(|tail| tail.contains("::")) {
        return None;
    }
    let groups = |part: &str, last: bool| -> Option<Vec<u16>> {
        if part.is_empty() {
            return Some(Vec::new());
        }
        let pieces: Vec<&str> = part.split(':').collect();
        let mut out = Vec::new();
        for (index, piece) in pieces.iter().enumerate() {
            if last && index + 1 == pieces.len() && piece.contains('.') {
                let v4 = parse_v4(piece)?;
                out.push(u16::from(v4[0]) << 8 | u16::from(v4[1]));
                out.push(u16::from(v4[2]) << 8 | u16::from(v4[3]));
            } else {
                if piece.is_empty() || piece.len() > 4 || !piece.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return None;
                }
                out.push(u16::from_str_radix(piece, 16).ok()?);
            }
        }
        Some(out)
    };
    let mut result = [0u16; 8];
    match tail {
        None => {
            let all = groups(head, true)?;
            if all.len() != 8 {
                return None;
            }
            result.copy_from_slice(&all);
        }
        Some(tail) => {
            let front = groups(head, false)?;
            let back = groups(tail, true)?;
            if front.len() + back.len() > 7 {
                return None;
            }
            result[..front.len()].copy_from_slice(&front);
            result[8 - back.len()..].copy_from_slice(&back);
        }
    }
    Some(result)
}

/// `isIP(text)`: 4, 6, or 0.
fn ip_family(text: &str) -> u8 {
    if parse_v4(text).is_some() {
        4
    } else if parse_v6(text).is_some() {
        6
    } else {
        0
    }
}

/// Test a resolved IP against public unicast ranges, rejecting mapped/compatible and special IPv6 (`publicAddress`).
///
/// # Arguments
/// * `address` - DNS result or literal IP, without URL brackets.
///
/// # Returns
/// True only for a public unicast address.
pub fn public_address(address: &str) -> bool {
    if let Some(v4) = parse_v4(address) {
        let value = u32::from_be_bytes(v4);
        return !BLOCKED_V4.iter().any(|(base, prefix)| {
            let mask = u32::MAX << (32 - prefix);
            value & mask == u32::from_be_bytes(*base) & mask
        });
    }
    let Some(groups) = parse_v6(address) else { return false };
    let (first, second) = (groups[0], groups[1]);
    let global = first & 0xe000 == 0x2000;
    let special = (first == 0x2001 && (second & 0xfe00 == 0 || second == 0x0db8)) || first == 0x2002 || (first == 0x3fff && second & 0xf000 == 0);
    global && !special
}

/// Validate URL syntax and literal destinations before any request or redirect (`captureDestination`).
///
/// # Arguments
/// * `value` - Absolute HTTP(S) URL without credentials or live account origin.
/// * `offline` - Allow only loopback fixture hosts and addresses.
///
/// # Returns
/// The normalized URL.
///
/// # Errors
/// A `CAPTURE_BLOCKED` status 400 error for rejected targets.
pub fn capture_destination(value: &str, offline: bool) -> Result<String> {
    let safe = source_url(&Value::from(value), value).ok_or_else(forbidden)?;
    let url = mm_js::url::parse(&safe, None).ok_or_else(|| Error::invalid("Invalid URL"))?;
    let hostname = url.host_str().unwrap_or("");
    let host = hostname.strip_prefix('[').unwrap_or(hostname);
    let host = host.strip_suffix(']').unwrap_or(host);
    if offline {
        if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
            return Err(forbidden());
        }
    } else {
        if text::to_lower(host.strip_suffix('.').unwrap_or(host)) == "localhost" || host.ends_with(".localhost") {
            return Err(forbidden());
        }
        let raw_host = RAW_AUTHORITY.captures(value).map(|captures| text::to_lower(&PORT_SUFFIX.replace(&captures[1], "")));
        let family = ip_family(host);
        if family == 4 && raw_host.as_deref() != Some(host) {
            return Err(forbidden());
        }
        if family != 0 && !public_address(host) {
            return Err(forbidden());
        }
    }
    Ok(url.as_str().to_owned())
}
