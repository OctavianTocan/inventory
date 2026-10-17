//! Port of `lib/render/encoding.mjs`: the HTML encoding of mymind's server templates.

use std::fmt::Write;

use crate::capture_entities::numeric_entity;
use crate::{Error, Result};

/// Encode text as the server's default Razor HTML encoder does (`escapeHtml`).
///
/// # Arguments
/// * `value` - Text or an attribute value; callers pass `""` for a nullish value.
///
/// # Returns
/// HTML with `&<>"'` and `+` escaped and every code point outside printable ASCII as an upper-case hex entity.
pub fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            '+' | '\0'..='\x1f' | '\x7f'.. => {
                let _ = write!(out, "&#x{:X};", u32::from(character));
            }
            _ => out.push(character),
        }
    }
    out
}

/// Encode text inside the server's ProseMirror serializer (`escapeProse`).
///
/// # Arguments
/// * `value` - Plain text; callers pass `""` for a nullish value.
///
/// # Returns
/// Text safe in an HTML text node or a quoted attribute.
pub fn escape_prose(value: &str) -> String {
    escape_html(value)
}

/// Decode the named and numeric entities the recorded templates emit (`unescapeHtml`).
///
/// # Arguments
/// * `value` - Entity-encoded text.
///
/// # Returns
/// Decoded text; unknown named entities stay, and a surrogate code point becomes U+FFFD.
///
/// # Errors
/// A numeric entity past U+10FFFF, where `String.fromCodePoint` throws a `RangeError`.
pub fn unescape_html(value: &str) -> Result<String> {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        if let Some(number) = after.strip_prefix('#') {
            if let Some((code, length)) = numeric_entity(number) {
                if code > f64::from(0x10ffff) {
                    return Err(Error::invalid(format!("Invalid code point {code}")));
                }
                out.push(char::from_u32(code as u32).unwrap_or('\u{fffd}'));
                rest = &number[length + 1..];
                continue;
            }
        } else if let Some((name, text)) = ["amp", "lt", "gt", "quot", "apos", "nbsp"]
            .iter()
            .zip(["&", "<", ">", "\"", "'", "\u{a0}"])
            .find(|(name, _)| after.len() > name.len() && after.as_bytes()[..name.len()].eq_ignore_ascii_case(name.as_bytes()) && after.as_bytes()[name.len()] == b';')
        {
            out.push_str(text);
            rest = &after[name.len() + 1..];
            continue;
        }
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    Ok(out)
}
