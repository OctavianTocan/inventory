/// `String(number)`: the shortest text that reads back as the same number, in JS's notation (`1e+21`, `1.5e-7`, `0`
/// for -0, `NaN`, `Infinity`).
pub fn number_to_string(number: f64) -> String {
    if number.is_nan() {
        return "NaN".into();
    }
    if number.is_infinite() {
        return if number > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if number == 0.0 {
        return "0".into();
    }
    ryu_js::Buffer::new().format_finite(number).to_owned()
}

/// `Number(text)`: decimal, `0x`/`0o`/`0b` and `Infinity` forms after trimming JS whitespace; NaN for anything else.
pub fn string_to_number(text: &str) -> f64 {
    let text = crate::text::trim(text);
    if text.is_empty() {
        return 0.0;
    }
    for (prefix, radix) in [("0x", 16), ("0X", 16), ("0o", 8), ("0O", 8), ("0b", 2), ("0B", 2)] {
        if let Some(digits) = text.strip_prefix(prefix) {
            return integer_digits(digits, radix).unwrap_or(f64::NAN);
        }
    }
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    if unsigned == "Infinity" {
        return if text.starts_with('-') { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    if decimal_literal_length(unsigned) == Some(unsigned.len()) {
        text.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}

fn integer_digits(digits: &str, radix: u32) -> Option<f64> {
    if digits.is_empty() {
        return None;
    }
    let values: Option<Vec<u32>> = digits.chars().map(|character| character.to_digit(radix)).collect();
    let values = values?;
    Some(if radix.is_power_of_two() { power_of_two_value(&values, radix) } else { generic_value(&values, radix) })
}

/// The correctly rounded value of digits in a power-of-two radix; digits past 128 bits only break rounding ties.
fn power_of_two_value(values: &[u32], radix: u32) -> f64 {
    let bits = radix.trailing_zeros();
    let mut mantissa = 0_u128;
    let mut dropped = 0_i32;
    let mut sticky = false;
    for &value in values {
        if dropped > 0 || mantissa.leading_zeros() < bits {
            dropped += bits as i32;
            sticky |= value != 0;
        } else {
            mantissa = mantissa << bits | u128::from(value);
        }
    }
    if sticky {
        mantissa |= 1;
    }
    mantissa as f64 * 2_f64.powi(dropped)
}

/// V8's value for other radixes: 32-bit chunks folded into a double, which rounds as V8 does past 2^53.
fn generic_value(values: &[u32], radix: u32) -> f64 {
    const MAXIMUM_MULTIPLIER: u32 = u32::MAX / 36;
    let mut result = 0.0_f64;
    let mut at = 0;
    while at < values.len() {
        let (mut part, mut multiplier) = (0_u32, 1_u32);
        while at < values.len() {
            let next = multiplier * radix;
            if next > MAXIMUM_MULTIPLIER {
                break;
            }
            part = part * radix + values[at];
            multiplier = next;
            at += 1;
        }
        result = result * f64::from(multiplier) + f64::from(part);
    }
    result
}

/// Length of the longest prefix of `text` that is a JS `StrUnsignedDecimalLiteral` without `Infinity`.
fn decimal_literal_length(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut at = 0;
    let integer_start = at;
    while at < bytes.len() && bytes[at].is_ascii_digit() {
        at += 1;
    }
    let mut digits = at > integer_start;
    if at < bytes.len() && bytes[at] == b'.' {
        let fraction_start = at + 1;
        let mut end = fraction_start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if digits || end > fraction_start {
            digits = true;
            at = end;
        }
    }
    if !digits {
        return None;
    }
    if at < bytes.len() && (bytes[at] == b'e' || bytes[at] == b'E') {
        let mut end = at + 1;
        if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
            end += 1;
        }
        let exponent_start = end;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > exponent_start {
            at = end;
        }
    }
    Some(at)
}

/// `parseFloat(text)`: the longest decimal prefix after leading whitespace, or NaN.
pub fn parse_float(text: &str) -> f64 {
    let text = crate::text::trim_start(text);
    let (sign, unsigned) = match text.as_bytes().first() {
        Some(b'-') => (-1.0, &text[1..]),
        Some(b'+') => (1.0, &text[1..]),
        _ => (1.0, text),
    };
    if unsigned.starts_with("Infinity") {
        return sign * f64::INFINITY;
    }
    match decimal_literal_length(unsigned) {
        Some(length) => sign * unsigned[..length].parse::<f64>().unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}

/// `parseInt(text, radix)` for radix 2 to 36, or 0 for "decide from the text" (`0x` means 16, else 10).
pub fn parse_int(text: &str, radix: u32) -> f64 {
    let text = crate::text::trim_start(text);
    let (sign, mut digits) = match text.as_bytes().first() {
        Some(b'-') => (-1.0, &text[1..]),
        Some(b'+') => (1.0, &text[1..]),
        _ => (1.0, text),
    };
    let mut radix = radix;
    if (radix == 0 || radix == 16) && (digits.starts_with("0x") || digits.starts_with("0X")) {
        digits = &digits[2..];
        radix = 16;
    }
    if radix == 0 {
        radix = 10;
    }
    if !(2..=36).contains(&radix) {
        return f64::NAN;
    }
    let end = digits.find(|character: char| character.to_digit(radix).is_none()).unwrap_or(digits.len());
    if end == 0 {
        return f64::NAN;
    }
    let value = if radix == 10 { digits[..end].parse::<f64>().unwrap_or(f64::NAN) } else { integer_digits(&digits[..end], radix).unwrap_or(f64::NAN) };
    sign * value
}

/// `Math.round(number)`: halves round toward +Infinity.
pub fn round(number: f64) -> f64 {
    if !number.is_finite() || number.fract() == 0.0 {
        return number;
    }
    let floor = number.floor();
    let rounded = if number - floor >= 0.5 { floor + 1.0 } else { floor };
    if rounded == 0.0 && number < 0.0 { -0.0 } else { rounded }
}

/// `number.toFixed(digits)` for `digits` up to 100: the exact binary value rounded half up, `-` for any negative
/// number, and `String(number)` from 1e21 up.
pub fn to_fixed(number: f64, digits: usize) -> String {
    if number.is_nan() {
        return "NaN".into();
    }
    if number.abs() >= 1e21 {
        return number_to_string(number);
    }
    let exact = format!("{:.1100}", number.abs());
    let dot = exact.find('.').expect("fixed notation has a point");
    let keep = if digits == 0 { dot } else { dot + 1 + digits };
    let mut kept: Vec<u8> = exact[..keep].bytes().collect();
    if exact.as_bytes()[dot + 1 + digits] >= b'5' {
        let mut index = kept.len();
        loop {
            if index == 0 {
                kept.insert(0, b'1');
                break;
            }
            index -= 1;
            match kept[index] {
                b'.' => {}
                b'9' => kept[index] = b'0',
                digit => {
                    kept[index] = digit + 1;
                    break;
                }
            }
        }
    }
    let text = String::from_utf8(kept).expect("ASCII digits");
    if number < 0.0 { format!("-{text}") } else { text }
}

/// `number.toString(radix)` for safe integers.
pub fn integer_to_radix(number: i64, radix: u32) -> String {
    if number == 0 {
        return "0".into();
    }
    let mut value = number.unsigned_abs();
    let mut digits = Vec::new();
    while value > 0 {
        digits.push(std::char::from_digit((value % u64::from(radix)) as u32, radix).expect("digit below radix"));
        value /= u64::from(radix);
    }
    if number < 0 {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// `Number.isSafeInteger(number)`.
pub fn is_safe_integer(number: f64) -> bool {
    number.is_finite() && number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0
}
