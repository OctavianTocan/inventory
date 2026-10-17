//! `util.inspect` with Node's default options, as `console.log` prints a value that is not a string.

use crate::{Object, Value, number_to_string};

const BREAK_LENGTH: usize = 80;
const COMPACT: usize = 3;
const DEPTH: usize = 2;
const MAX_ARRAY_LENGTH: usize = 100;
const MAX_STRING_LENGTH: usize = 10_000;
const MIN_LINE_WIDTH: usize = 16;

struct Context {
    indentation: usize,
    current_depth: usize,
}

/// Format `value` as `util.inspect(value)` does with the default options (depth 2, `breakLength` 80, `compact` 3).
///
/// # Arguments
/// * `value` - A JSON-like value; bytes print as `<Buffer …>` and dates as their ISO string.
///
/// # Returns
/// The text Node prints for it, lines joined with `\n`.
pub fn inspect(value: &Value) -> String {
    format_value(&mut Context { indentation: 0, current_depth: 0 }, value, 0)
}

/// Format `console.log`'s arguments: strings as they are, other values through [`inspect`], joined by spaces.
///
/// # Arguments
/// * `values` - The arguments; a first string is not read for `%` directives.
///
/// # Returns
/// The line `console.log` writes, without its newline.
pub fn console_line(values: &[Value]) -> String {
    values.iter().map(|value| if let Value::String(text) = value { text.clone() } else { inspect(value) }).collect::<Vec<_>>().join(" ")
}

fn units(text: &str) -> usize {
    text.encode_utf16().count()
}

fn format_value(context: &mut Context, value: &Value, recurse_times: usize) -> String {
    match value {
        Value::Undefined => "undefined".to_owned(),
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::BigInt(number) => format!("{number}n"),
        Value::Number(number) => if *number == 0.0 && number.is_sign_negative() { "-0".to_owned() } else { number_to_string(*number) },
        Value::String(text) => format_string(context, text),
        Value::Bytes(bytes) => {
            let shown: Vec<String> = bytes.iter().take(50).map(|byte| format!("{byte:02x}")).collect();
            let more = if bytes.len() > 50 { format!(" ... {} more byte{}", bytes.len() - 50, if bytes.len() - 50 > 1 { "s" } else { "" }) } else { String::new() };
            format!("<Buffer{}{}{more}>", if shown.is_empty() { "" } else { " " }, shown.join(" "))
        }
        Value::Date(ms) => crate::date::iso_from_ms(*ms).unwrap_or_else(|| "Invalid Date".to_owned()),
        Value::Ext(kind, bytes) => format!("Ext {{ type: {kind}, data: {} }}", format_value(context, &Value::Bytes(bytes.clone()), recurse_times)),
        Value::Array(items) => format_array(context, items, recurse_times),
        Value::Object(object) => format_object(context, object, recurse_times),
    }
}

fn format_string(context: &Context, text: &str) -> String {
    let length = units(text);
    let (text, trailer) = if length > MAX_STRING_LENGTH {
        let remaining = length - MAX_STRING_LENGTH;
        (crate::text::slice(text, 0, Some(MAX_STRING_LENGTH as i64)), format!("... {remaining} more character{}", if remaining > 1 { "s" } else { "" }))
    } else {
        (text.to_owned(), String::new())
    };
    let length = units(&text);
    if length > MIN_LINE_WIDTH && length + context.indentation + 4 > BREAK_LENGTH {
        let lines: Vec<String> = text.split_inclusive('\n').map(escape).collect();
        return lines.join(&format!(" +\n{}", " ".repeat(context.indentation + 2))) + &trailer;
    }
    escape(&text) + &trailer
}

// `strEscape`: the quote that needs no escaping, and escapes for control characters, backslashes and that quote.
fn escape(text: &str) -> String {
    let quote = if !text.contains('\'') {
        '\''
    } else if !text.contains('"') {
        '"'
    } else if !text.contains('`') && !text.contains("${") {
        '`'
    } else {
        '\''
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for character in text.chars() {
        let point = character as u32;
        match character {
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{b}' => out.push_str("\\x0B"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            _ if character == quote && quote == '\'' => out.push_str("\\'"),
            _ if point < 32 || (127..160).contains(&point) => out.push_str(&format!("\\x{point:02X}")),
            _ => out.push(character),
        }
    }
    out.push(quote);
    out
}

fn is_identifier(key: &str) -> bool {
    let mut characters = key.chars();
    characters.next().is_some_and(|first| first.is_ascii_alphabetic() || first == '_') && characters.all(|rest| rest.is_ascii_alphanumeric() || rest == '_')
}

fn property(context: &mut Context, value: &Value, recurse_times: usize) -> String {
    context.indentation += 2;
    let text = format_value(context, value, recurse_times);
    context.indentation -= 2;
    text
}

fn format_object(context: &mut Context, object: &Object, recurse_times: usize) -> String {
    if object.is_empty() {
        return "{}".to_owned();
    }
    if recurse_times > DEPTH {
        return "[Object]".to_owned();
    }
    let recurse_times = recurse_times + 1;
    context.current_depth = recurse_times;
    let output: Vec<String> = object
        .iter()
        .map(|(key, value)| {
            let name = if is_identifier(&key) { key.into_owned() } else { escape(&key) };
            format!("{name}: {}", property(context, value, recurse_times))
        })
        .collect();
    reduce(context, output, ("{", "}"), None, recurse_times)
}

fn format_array(context: &mut Context, items: &[Value], recurse_times: usize) -> String {
    if items.is_empty() {
        return "[]".to_owned();
    }
    if recurse_times > DEPTH {
        return "[Array]".to_owned();
    }
    let recurse_times = recurse_times + 1;
    context.current_depth = recurse_times;
    let mut output: Vec<String> = items.iter().take(MAX_ARRAY_LENGTH).map(|item| property(context, item, recurse_times)).collect();
    if items.len() > MAX_ARRAY_LENGTH {
        let remaining = items.len() - MAX_ARRAY_LENGTH;
        output.push(format!("... {remaining} more item{}", if remaining > 1 { "s" } else { "" }));
    }
    reduce(context, output, ("[", "]"), Some(items), recurse_times)
}

// `reduceToSingleString`: one line when the innermost levels fit within the break length, else one entry a line.
fn reduce(context: &Context, mut output: Vec<String>, braces: (&str, &str), array: Option<&[Value]>, recurse_times: usize) -> String {
    let entries = output.len();
    if let Some(items) = array
        && entries > 6
    {
        output = group(context, output, items);
    }
    if context.current_depth - recurse_times < COMPACT && entries == output.len() {
        let start = output.len() + context.indentation + braces.0.len() + 10;
        if below_break_length(&output, start) {
            let joined = output.join(", ");
            if !joined.contains('\n') {
                return format!("{} {joined} {}", braces.0, braces.1);
            }
        }
    }
    let indentation = format!("\n{}", " ".repeat(context.indentation));
    format!("{}{indentation}  {}{indentation}{}", braces.0, output.join(&format!(",{indentation}  ")), braces.1)
}

fn below_break_length(output: &[String], start: usize) -> bool {
    let mut total = output.len() + start;
    if total + output.len() > BREAK_LENGTH {
        return false;
    }
    for entry in output {
        total += units(entry);
        if total > BREAK_LENGTH {
            return false;
        }
    }
    true
}

// `groupArrayElements`: more than six short entries laid out in aligned columns.
fn group(context: &Context, output: Vec<String>, items: &[Value]) -> Vec<String> {
    let separator = 2;
    let length = if items.len() > MAX_ARRAY_LENGTH { output.len() - 1 } else { output.len() };
    let widths: Vec<usize> = output[..length].iter().map(|entry| units(entry)).collect();
    let total: usize = widths.iter().map(|width| width + separator).sum();
    let max = widths.iter().copied().max().unwrap_or(0);
    let actual_max = max + separator;
    if !(actual_max * 3 + context.indentation < BREAK_LENGTH && (total as f64 / actual_max as f64 > 5.0 || max <= 6)) {
        return output;
    }
    let average_bias = (actual_max as f64 - total as f64 / output.len() as f64).sqrt();
    let biased_max = (actual_max as f64 - 3.0 - average_bias).max(1.0);
    let columns = [((2.5 * biased_max * length as f64).sqrt() / biased_max).round() as usize, (BREAK_LENGTH - context.indentation) / actual_max, COMPACT * 4, 15]
        .into_iter()
        .min()
        .unwrap();
    if columns <= 1 {
        return output;
    }
    let column_widths: Vec<usize> = (0..columns).map(|column| (column..output.len()).step_by(columns).filter_map(|at| widths.get(at)).copied().max().unwrap_or(0) + separator).collect();
    let numbers = items.iter().take(output.len()).all(|item| matches!(item, Value::Number(_)));
    let mut grouped = Vec::new();
    for start in (0..length).step_by(columns) {
        let end = (start + columns).min(length);
        let mut line = String::new();
        for at in start..end - 1 {
            let cell = format!("{}, ", output[at]);
            line.push_str(&pad(&cell, column_widths[at - start], numbers));
        }
        let last = end - 1;
        if numbers {
            line.push_str(&pad(&output[last], column_widths[last - start] - separator, true));
        } else {
            line.push_str(&output[last]);
        }
        grouped.push(line);
    }
    if items.len() > MAX_ARRAY_LENGTH {
        grouped.push(output[length].clone());
    }
    grouped
}

fn pad(text: &str, width: usize, start: bool) -> String {
    let fill = " ".repeat(width.saturating_sub(units(text)));
    if start { fill + text } else { format!("{text}{fill}") }
}
