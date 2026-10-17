//! String operations that count in UTF-16 code units or use JS's whitespace set, as JS strings do.
//!
//! A cut through a surrogate pair, which JS would keep as a lone surrogate, becomes U+FFFD.

/// Whether JS's `\s` and `trim` treat the character as whitespace.
pub fn is_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{9}' | '\u{a}' | '\u{b}' | '\u{c}' | '\u{d}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}'
            | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

/// `text.trim()`.
pub fn trim(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

/// `text.trimStart()`.
pub fn trim_start(text: &str) -> &str {
    text.trim_start_matches(is_whitespace)
}

/// `text.trimEnd()`.
pub fn trim_end(text: &str) -> &str {
    text.trim_end_matches(is_whitespace)
}

/// `text.length`: UTF-16 code units.
pub fn len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn from_units(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

fn relative(index: i64, length: usize) -> usize {
    if index < 0 { (length as i64 + index).max(0) as usize } else { (index as usize).min(length) }
}

/// `text.slice(start, end)`; `None` for `end` means to the end. Negative indexes count from the end.
pub fn slice(text: &str, start: i64, end: Option<i64>) -> String {
    if text.is_ascii() {
        let length = text.len();
        let from = relative(start, length);
        let to = end.map_or(length, |end| relative(end, length));
        return if from < to { text[from..to].to_owned() } else { String::new() };
    }
    let units = units(text);
    let from = relative(start, units.len());
    let to = end.map_or(units.len(), |end| relative(end, units.len()));
    if from < to { from_units(&units[from..to]) } else { String::new() }
}

/// `text.substring(start, end)`: negative indexes are 0 and the bounds swap when reversed.
pub fn substring(text: &str, start: i64, end: Option<i64>) -> String {
    let length = len(text) as i64;
    let a = start.clamp(0, length);
    let b = end.unwrap_or(length).clamp(0, length);
    let (from, to) = if a <= b { (a, b) } else { (b, a) };
    slice(text, from, Some(to))
}

/// `text.charCodeAt(index)`, or `None` past the end (where JS gives NaN).
pub fn char_code_at(text: &str, index: usize) -> Option<u16> {
    text.encode_utf16().nth(index)
}

/// `text.codePointAt(index)` at a UTF-16 index.
pub fn code_point_at(text: &str, index: usize) -> Option<u32> {
    let units = units(text);
    let first = *units.get(index)?;
    if (0xd800..0xdc00).contains(&first)
        && let Some(&second) = units.get(index + 1)
        && (0xdc00..0xe000).contains(&second)
    {
        return Some(0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00));
    }
    Some(u32::from(first))
}

fn pad(text: &str, target: usize, fill: &str) -> String {
    let length = len(text);
    if target <= length || fill.is_empty() {
        return String::new();
    }
    let fill_units = units(fill);
    let needed = target - length;
    let padding: Vec<u16> = fill_units.iter().copied().cycle().take(needed).collect();
    from_units(&padding)
}

/// `text.padStart(target, fill)`.
pub fn pad_start(text: &str, target: usize, fill: &str) -> String {
    format!("{}{text}", pad(text, target, fill))
}

/// `text.padEnd(target, fill)`.
pub fn pad_end(text: &str, target: usize, fill: &str) -> String {
    format!("{text}{}", pad(text, target, fill))
}

/// `text.indexOf(search, from)` in UTF-16 units.
pub fn index_of(text: &str, search: &str, from: usize) -> Option<usize> {
    let haystack = units(text);
    let needle = units(search);
    if needle.is_empty() {
        return Some(from.min(haystack.len()));
    }
    (from..=haystack.len().checked_sub(needle.len())?).find(|&at| haystack[at..at + needle.len()] == needle[..])
}

/// `new TextDecoder().decode(bytes)`: UTF-8 with replacement characters, a leading byte order mark dropped.
pub fn decode_utf8(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    String::from_utf8_lossy(bytes).into_owned()
}

/// `text.toLowerCase()`.
pub fn to_lower(text: &str) -> String {
    text.to_lowercase()
}

/// `text.toUpperCase()`.
pub fn to_upper(text: &str) -> String {
    text.to_uppercase()
}

/// How `a < b` orders two strings: by UTF-16 code units, which puts supplementary characters before U+E000..U+FFFF
/// where Rust's byte order puts them after.
pub fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}
