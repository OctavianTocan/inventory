//! `Date` as the backend used it: milliseconds since the epoch, ISO text, and `Date.parse`.
//!
//! Local time is UTC, as on Workers. `Date.parse` reads ISO text and the common legacy forms (`Oct 17 2026`,
//! `Sat, 17 Oct 2026 10:00:00 GMT`, `2026/10/17`); other legacy forms V8 accepts give NaN.

use std::sync::atomic::{AtomicI64, Ordering};

const MS_PER_DAY: i64 = 86_400_000;
const MAX_MS: f64 = 8.64e15;

static NOW_OFFSET: AtomicI64 = AtomicI64::new(0);
static FIXED_NOW: AtomicI64 = AtomicI64::new(i64::MIN);

/// `Date.now()`: milliseconds since the epoch, shifted by [`set_now_offset`].
pub fn now() -> f64 {
    let fixed = FIXED_NOW.load(Ordering::Relaxed);
    if fixed != i64::MIN {
        return fixed as f64;
    }
    #[cfg(target_arch = "wasm32")]
    let real = js_sys::Date::now();
    #[cfg(not(target_arch = "wasm32"))]
    let real = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |elapsed| elapsed.as_millis() as f64);
    real + NOW_OFFSET.load(Ordering::Relaxed) as f64
}

/// Shift what [`now`] returns by `offset` milliseconds, for tests that need a fixed clock.
pub fn set_now_offset(offset: i64) {
    NOW_OFFSET.store(offset, Ordering::Relaxed);
}

/// Test hook: make [`now`] return `ms` until called with `None`, so timestamps compare exactly with the JS backend.
/// The clock is process-global; tests that set it must not run concurrently with others that read it.
///
/// # Arguments
/// * `ms` - The fixed time in milliseconds since the epoch, or `None` to use the real clock again.
pub fn set_fixed_now(ms: Option<i64>) {
    FIXED_NOW.store(ms.unwrap_or(i64::MIN), Ordering::Relaxed);
}

/// Days since 1970-01-01 of a proleptic Gregorian date.
pub fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The date `(year, month 1-12, day 1-31)` of a day count since 1970-01-01.
pub fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

/// `new Date(ms).toISOString()`, or `None` for an invalid date (where JS throws).
pub fn iso_from_ms(ms: f64) -> Option<String> {
    if !ms.is_finite() || ms.abs() > MAX_MS {
        return None;
    }
    let ms = ms.trunc() as i64;
    let days = ms.div_euclid(MS_PER_DAY);
    let in_day = ms.rem_euclid(MS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 {
        format!("-{:06}", -year)
    } else {
        format!("+{year:06}")
    };
    Some(format!(
        "{year}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        in_day / 3_600_000,
        in_day / 60_000 % 60,
        in_day / 1000 % 60,
        in_day % 1000
    ))
}

/// Milliseconds of a UTC date and time, NaN when a field is out of range. Days up to 31 roll into the next month, as
/// `Date.parse("2026-02-30")` gives March 2.
pub fn utc(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64, ms: i64) -> f64 {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 24 || minute > 59 || second > 59 {
        return f64::NAN;
    }
    if hour == 24 && (minute, second, ms) != (0, 0, 0) {
        return f64::NAN;
    }
    let value = (days_from_civil(year, month, day) * MS_PER_DAY + ((hour * 60 + minute) * 60 + second) * 1000 + ms) as f64;
    if value.abs() > MAX_MS { f64::NAN } else { value }
}

/// `Date.parse(text)`: milliseconds since the epoch, or NaN.
pub fn parse(text: &str) -> f64 {
    parse_iso(text).unwrap_or_else(|| parse_legacy(text))
}

fn digits(text: &str, at: &mut usize, count: usize) -> Option<i64> {
    let slice = text.get(*at..*at + count)?;
    if !slice.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    *at += count;
    slice.parse().ok()
}

fn parse_iso(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut at = 0;
    let year = match bytes.first()? {
        b'+' | b'-' => {
            let negative = bytes[0] == b'-';
            at = 1;
            let value = digits(text, &mut at, 6)?;
            if negative && value == 0 {
                return None;
            }
            if negative { -value } else { value }
        }
        _ => digits(text, &mut at, 4)?,
    };
    let (mut month, mut day) = (1, 1);
    if bytes.get(at) == Some(&b'-') {
        at += 1;
        month = digits(text, &mut at, 2)?;
        if bytes.get(at) == Some(&b'-') {
            at += 1;
            day = digits(text, &mut at, 2)?;
        }
    }
    if at == bytes.len() {
        return Some(utc(year, month, day, 0, 0, 0, 0));
    }
    if bytes.get(at) != Some(&b'T') && bytes.get(at) != Some(&b't') && bytes.get(at) != Some(&b' ') {
        return None;
    }
    at += 1;
    let hour = digits(text, &mut at, 2)?;
    if bytes.get(at) != Some(&b':') {
        return None;
    }
    at += 1;
    let minute = digits(text, &mut at, 2)?;
    let (mut second, mut ms) = (0, 0);
    if bytes.get(at) == Some(&b':') {
        at += 1;
        second = digits(text, &mut at, 2)?;
        if bytes.get(at) == Some(&b'.') || bytes.get(at) == Some(&b',') {
            at += 1;
            let start = at;
            while bytes.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
            if at == start {
                return None;
            }
            ms = format!("{:0<3}", &text[start..(start + 3).min(at)]).parse().ok()?;
        }
    }
    let offset = match bytes.get(at) {
        None => 0,
        Some(b'Z' | b'z') => {
            at += 1;
            0
        }
        Some(&sign @ (b'+' | b'-')) => {
            at += 1;
            let hours = digits(text, &mut at, 2)?;
            if bytes.get(at) == Some(&b':') {
                at += 1;
            }
            let minutes = digits(text, &mut at, 2)?;
            let total = (hours * 60 + minutes) * 60_000;
            if sign == b'+' { total } else { -total }
        }
        _ => return None,
    };
    if at != bytes.len() {
        return None;
    }
    let value = utc(year, month, day, hour, minute, second, ms);
    Some(value - offset as f64)
}

const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

fn parse_legacy(text: &str) -> f64 {
    let mut year = None;
    let mut month = None;
    let mut day = None;
    let mut time = (0, 0, 0);
    let mut offset = 0_i64;
    let mut numbers = Vec::new();
    let cleaned = text.replace(',', " ");
    for token in cleaned.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if let Some(index) = MONTHS.iter().position(|name| lower.starts_with(name)) {
            month = Some(index as i64 + 1);
        } else if token.contains(':') {
            let parts: Vec<_> = token.split(':').map(str::parse::<i64>).collect();
            match parts.as_slice() {
                [Ok(h), Ok(m)] => time = (*h, *m, 0),
                [Ok(h), Ok(m), Ok(s)] => time = (*h, *m, *s),
                _ => return f64::NAN,
            }
        } else if matches!(lower.as_str(), "gmt" | "utc" | "ut" | "z") {
        } else if let Some(zone) = lower.strip_prefix("gmt").or_else(|| lower.strip_prefix("utc")).filter(|zone| !zone.is_empty()).or(
            (lower.starts_with(['+', '-']) && lower.len() == 5).then_some(lower.as_str()),
        ) {
            let sign = if zone.starts_with('-') { -1 } else { 1 };
            let Ok(value) = zone[1..].replace(':', "").parse::<i64>() else { return f64::NAN };
            offset = sign * ((value / 100) * 60 + value % 100) * 60_000;
        } else if token.contains('/') || token[1..].contains('-') {
            let parts: Vec<_> = token.split(['/', '-']).map(str::parse::<i64>).collect();
            match parts.as_slice() {
                [Ok(y), Ok(m), Ok(d)] if *y > 31 => (year, month, day) = (Some(*y), Some(*m), Some(*d)),
                [Ok(m), Ok(d), Ok(y)] => (year, month, day) = (Some(*y), Some(*m), Some(*d)),
                _ => return f64::NAN,
            }
        } else if let Ok(number) = token.parse::<i64>() {
            numbers.push(number);
        } else if ["mon", "tue", "wed", "thu", "fri", "sat", "sun"].iter().any(|name| lower.starts_with(name)) {
        } else {
            return f64::NAN;
        }
    }
    for number in numbers {
        if day.is_none() && (1..=31).contains(&number) && month.is_some() {
            day = Some(number);
        } else if year.is_none() {
            year = Some(if number < 50 { 2000 + number } else if number < 100 { 1900 + number } else { number });
        } else {
            return f64::NAN;
        }
    }
    match (year, month) {
        (Some(year), Some(month)) => utc(year, month, day.unwrap_or(1), time.0, time.1, time.2, 0) - offset as f64,
        _ => f64::NAN,
    }
}
