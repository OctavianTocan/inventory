//! Port of `new Response(body, { headers: { 'content-type': type } }).formData()` as Node 24.12 runs it: the WHATWG
//! MIME type of the header, the multipart parser of its bundled undici 7.16.0 with that parser's quirks, and Node's
//! `URLSearchParams` for URL-encoded bodies.

use mm_js::uri::decode_uri_component;

/// A value of a parsed form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormValue {
    /// A text field.
    Text(String),
    /// A file part.
    File(FormFile),
}

/// A file part, as the `File` undici makes of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFile {
    /// `file.name`.
    pub name: String,
    /// `file.type`: lower case, or empty.
    pub content_type: String,
    /// The file's bytes.
    pub bytes: Vec<u8>,
}

/// Why a body is not a form: whatever `formData()` rejects with (a `TypeError`, a `URIError` or an assertion).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormError(pub String);

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A form's entries in body order.
pub type Form = Vec<(String, FormValue)>;

/// The first value named `name` (`form.get(name)`).
///
/// # Arguments
/// * `form` - Parsed form.
/// * `name` - Entry name.
pub fn get<'a>(form: &'a Form, name: &str) -> Option<&'a FormValue> {
    form.iter().find(|(key, _)| key == name).map(|(_, value)| value)
}

/// Parse a request body as a form (`Response.prototype.formData`).
///
/// # Arguments
/// * `content_type` - The `content-type` header the response is made with.
/// * `body` - The body's bytes.
///
/// # Errors
/// A header the `Headers` constructor refuses, a type other than `multipart/form-data` or
/// `application/x-www-form-urlencoded`, or a malformed multipart body.
pub fn parse(content_type: &str, body: &[u8]) -> Result<Form, FormError> {
    let Some(mime) = extract_mime_type(content_type)? else { return Err(not_a_form()) };
    match mime.essence.as_str() {
        "multipart/form-data" => {
            let Some((_, boundary)) = mime.parameters.iter().find(|(name, _)| name == "boundary") else {
                return Err(parsing_error("missing boundary in content-type header"));
            };
            let boundary: String = boundary.iter().collect();
            multipart(body, &boundary)
        }
        "application/x-www-form-urlencoded" => {
            Ok(search_params(&String::from_utf8_lossy(body)).into_iter().map(|(name, value)| (name, FormValue::Text(value))).collect())
        }
        _ => Err(not_a_form()),
    }
}

fn not_a_form() -> FormError {
    FormError("Content-Type was not one of \"multipart/form-data\" or \"application/x-www-form-urlencoded\".".into())
}

fn parsing_error(cause: &str) -> FormError {
    FormError(format!("Failed to parse body as FormData: {cause}"))
}

struct MimeType {
    essence: String,
    parameters: Vec<(String, Vec<char>)>,
}

fn is_token<T: Copy + Into<u32>>(text: &[T]) -> bool {
    !text.is_empty()
        && text.iter().all(|&unit| {
            char::from_u32(unit.into()).is_some_and(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_|~".contains(c))
        })
}

fn is_quoted_string_token(text: &[char]) -> bool {
    !text.is_empty() && text.iter().all(|&c| c == '\t' || ('\u{20}'..='\u{7e}').contains(&c) || ('\u{80}'..='\u{ff}').contains(&c))
}

fn http_space(c: char) -> bool {
    matches!(c, '\r' | '\n' | '\t' | ' ')
}

// `removeChars`, whose trailing loop never removes the first character.
fn remove_chars<T: Copy>(buf: &[T], leading: bool, trailing: bool, predicate: impl Fn(T) -> bool) -> &[T] {
    let mut lead = 0;
    let mut trail = buf.len() as isize - 1;
    if leading {
        while lead < buf.len() && predicate(buf[lead]) {
            lead += 1;
        }
    }
    if trailing {
        while trail > 0 && predicate(buf[trail as usize]) {
            trail -= 1;
        }
    }
    let end = (trail + 1) as usize;
    if lead >= end { &buf[..0] } else { &buf[lead..end] }
}

fn collect<'a, T: Copy>(condition: impl Fn(T) -> bool, input: &'a [T], position: &mut usize) -> &'a [T] {
    let start = (*position).min(input.len());
    let mut end = start;
    while end < input.len() && condition(input[end]) {
        end += 1;
    }
    if end > *position {
        *position = end;
    }
    &input[start..end]
}

// `collectASequenceOfCodePointsFast`: everything up to the next `stop`, or the rest.
fn collect_until<'a>(stop: char, input: &'a [char], position: &mut usize) -> &'a [char] {
    let start = *position;
    match input[start..].iter().position(|&c| c == stop) {
        Some(found) => {
            *position = start + found;
            &input[start..start + found]
        }
        None => {
            *position = input.len();
            &input[start..]
        }
    }
}

// `collectAnHTTPQuotedString`: the unescaped value, or the raw quoted text with its quotes.
fn quoted_string(input: &[char], position: &mut usize, extract: bool) -> Vec<char> {
    let start = *position;
    let mut value = Vec::new();
    *position += 1;
    loop {
        value.extend_from_slice(collect(|c| c != '"' && c != '\\', input, position));
        if *position >= input.len() {
            break;
        }
        let quote_or_backslash = input[*position];
        *position += 1;
        if quote_or_backslash == '\\' {
            if *position >= input.len() {
                value.push('\\');
                break;
            }
            value.push(input[*position]);
            *position += 1;
        } else {
            break;
        }
    }
    if extract { value } else { input[start..*position].to_vec() }
}

fn ascii_lower(text: &[char]) -> String {
    text.iter().map(char::to_ascii_lowercase).collect()
}

// `parseMIMEType`.
fn parse_mime_type(input: &[char]) -> Option<MimeType> {
    let input = remove_chars(input, true, true, http_space);
    let mut position = 0;
    let kind = collect_until('/', input, &mut position);
    if !is_token(kind) || position >= input.len() {
        return None;
    }
    position += 1;
    let subtype = remove_chars(collect_until(';', input, &mut position), false, true, http_space);
    if !is_token(subtype) {
        return None;
    }
    let mut mime = MimeType { essence: format!("{}/{}", ascii_lower(kind), ascii_lower(subtype)), parameters: Vec::new() };
    while position < input.len() {
        position += 1;
        collect(http_space, input, &mut position);
        let name: Vec<char> = collect(|c| c != ';' && c != '=', input, &mut position).iter().flat_map(|c| c.to_lowercase()).collect();
        if position < input.len() {
            if input[position] == ';' {
                continue;
            }
            position += 1;
        }
        if position >= input.len() {
            break;
        }
        let value = if input[position] == '"' {
            let value = quoted_string(input, &mut position, true);
            collect_until(';', input, &mut position);
            value
        } else {
            let value = remove_chars(collect_until(';', input, &mut position), false, true, http_space);
            if value.is_empty() {
                continue;
            }
            value.to_vec()
        };
        let name: String = name.into_iter().collect();
        let chars: Vec<char> = name.chars().collect();
        if is_token(&chars) && (value.is_empty() || is_quoted_string_token(&value)) && !mime.parameters.iter().any(|(known, _)| *known == name) {
            mime.parameters.push((name, value));
        }
    }
    Some(mime)
}

// `gettingDecodingSplitting`: a header value split at the commas outside quoted strings.
fn split_values(input: &[char]) -> Vec<Vec<char>> {
    let mut values = Vec::new();
    let mut position = 0;
    let mut value = Vec::new();
    while position < input.len() {
        value.extend_from_slice(collect(|c| c != '"' && c != ',', input, &mut position));
        if position < input.len() {
            if input[position] == '"' {
                value.extend(quoted_string(input, &mut position, false));
                if position < input.len() {
                    continue;
                }
            } else {
                position += 1;
            }
        }
        values.push(remove_chars(&value, true, true, |c| c == '\t' || c == ' ').to_vec());
        value.clear();
    }
    values
}

// The `Headers` constructor's checks on the value, then `extractMimeType`.
fn extract_mime_type(header: &str) -> Result<Option<MimeType>, FormError> {
    let value = header.trim_matches(http_space);
    if value.contains(['\0', '\r', '\n']) || value.chars().any(|c| u32::from(c) > 0xff) {
        return Err(FormError("Headers.append: invalid header value".into()));
    }
    let chars: Vec<char> = value.chars().collect();
    Ok(split_values(&chars).iter().filter_map(|value| parse_mime_type(value)).rfind(|mime| mime.essence != "*/*"))
}

fn starts_at(input: &[u8], start: &[u8], position: usize) -> bool {
    input.get(position..).is_some_and(|rest| rest.starts_with(start))
}

fn index_of(input: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(from.min(input.len()));
    }
    input.get(from..)?.windows(needle.len()).position(|window| window == needle).map(|found| from + found)
}

fn crlf_at(input: &[u8], position: usize) -> bool {
    input.get(position) == Some(&b'\r') && input.get(position + 1) == Some(&b'\n')
}

// `new TextDecoder().decode(bytes)`, which drops a leading byte order mark.
fn text_decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes)).into_owned()
}

// `utf8DecodeBytes`: it drops one byte order mark itself and its decoder drops another.
fn utf8_decode_bytes(bytes: &[u8]) -> String {
    text_decode(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes))
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

// `.replace(/%0A/ig, '\n')` and its siblings: one escape, matched without case.
fn replace_escape(text: &str, escape: &str, with: char) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(found) = rest.find('%') {
        out.push_str(&rest[..found]);
        let candidate = rest.get(found..found + 3);
        if candidate.is_some_and(|candidate| candidate.eq_ignore_ascii_case(escape)) {
            out.push(with);
            rest = &rest[found + 3..];
        } else {
            out.push('%');
            rest = &rest[found + 1..];
        }
    }
    out.push_str(rest);
    out
}

// `parseMultipartFormDataName`: a quoted name whose opening quote was just passed.
fn part_name(input: &[u8], position: &mut usize) -> Result<String, FormError> {
    if *position == 0 || input.get(*position - 1) != Some(&b'"') {
        return Err(FormError("The expression evaluated to a falsy value".into()));
    }
    let name = collect(|byte| byte != b'\n' && byte != b'\r' && byte != b'"', input, position);
    if input.get(*position) != Some(&b'"') {
        return Err(parsing_error("expected \""));
    }
    *position += 1;
    let name = replace_escape(&text_decode(name), "%0A", '\n');
    let name = replace_escape(&name, "%0D", '\r');
    Ok(name.replace("%22", "\""))
}

struct PartHeaders {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    encoding: Option<String>,
}

fn space_or_tab(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

fn line_value(input: &[u8], position: &mut usize) -> String {
    latin1(remove_chars(collect(|byte| byte != b'\n' && byte != b'\r', input, position), false, true, space_or_tab))
}

// `parseMultipartFormDataHeaders`, keeping its CRLF check that fails only when both bytes are wrong.
fn part_headers(input: &[u8], position: &mut usize) -> Result<PartHeaders, FormError> {
    let (mut name, mut filename, mut content_type, mut encoding) = (None, None, None, None);
    loop {
        if crlf_at(input, *position) {
            let Some(name) = name else { return Err(parsing_error("header name is null")) };
            return Ok(PartHeaders { name, filename, content_type, encoding });
        }
        let header = remove_chars(collect(|byte| byte != b'\n' && byte != b'\r' && byte != b':', input, position), true, true, space_or_tab);
        if !is_token(header) {
            return Err(parsing_error("header name does not match the field-name token production"));
        }
        if input.get(*position) != Some(&b':') {
            return Err(parsing_error("expected :"));
        }
        *position += 1;
        collect(space_or_tab, input, position);
        match header.to_ascii_lowercase().as_slice() {
            b"content-disposition" => {
                filename = None;
                if !starts_at(input, b"form-data; name=\"", *position) {
                    return Err(parsing_error("expected form-data; name=\" for content-disposition header"));
                }
                *position += 17;
                name = Some(part_name(input, position)?);
                if input.get(*position) == Some(&b';') && input.get(*position + 1) == Some(&b' ') {
                    let mut at = *position + 2;
                    if starts_at(input, b"filename", at) {
                        if input.get(at + 8) == Some(&b'*') {
                            at += 10;
                            collect(space_or_tab, input, &mut at);
                            let value = collect(|byte| byte != b' ' && byte != b'\r' && byte != b'\n', input, &mut at);
                            let utf8 = matches!(value.first(), Some(b'u' | b'U'))
                                && matches!(value.get(1), Some(b't' | b'T'))
                                && matches!(value.get(2), Some(b'f' | b'F'))
                                && value.get(3) == Some(&b'-')
                                && value.get(4) == Some(&b'8');
                            if !utf8 {
                                return Err(parsing_error("unknown encoding, expected utf-8''"));
                            }
                            let encoded = text_decode(value.get(7..).unwrap_or_default());
                            filename = Some(decode_uri_component(&encoded).map_err(|_| FormError("URI malformed".into()))?);
                            *position = at;
                        } else {
                            *position += 11;
                            collect(space_or_tab, input, position);
                            *position += 1;
                            filename = Some(part_name(input, position)?);
                        }
                    }
                }
            }
            b"content-type" => content_type = Some(line_value(input, position)),
            b"content-transfer-encoding" => encoding = Some(line_value(input, position)),
            _ => {
                collect(|byte| byte != b'\n' && byte != b'\r', input, position);
            }
        }
        if input.get(*position) != Some(&b'\r') && input.get(*position + 1) != Some(&b'\n') {
            return Err(parsing_error("expected CRLF"));
        }
        *position += 2;
    }
}

fn sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

// `Buffer.from(text, 'base64')`: Node reads each UTF-16 unit as its low byte, skips what is not in either alphabet and
// stops at the first `=`.
fn node_base64(text: &str) -> Vec<u8> {
    let mut sextets = Vec::new();
    for unit in text.encode_utf16() {
        let byte = unit as u8;
        if byte == b'=' {
            break;
        }
        sextets.extend(sextet(byte));
    }
    let mut out = Vec::with_capacity(sextets.len() / 4 * 3 + 2);
    for group in sextets.chunks(4) {
        if group.len() > 1 {
            out.push((group[0] << 2) | (group[1] >> 4));
        }
        if group.len() > 2 {
            out.push((group[1] << 4) | (group[2] >> 2));
        }
        if group.len() > 3 {
            out.push((group[2] << 6) | group[3]);
        }
    }
    out
}

// The `type` a `Blob` keeps: lower case, or empty when a character is outside U+0020 to U+007E.
fn blob_type(content_type: &str) -> String {
    if content_type.chars().all(|c| ('\u{20}'..='\u{7e}').contains(&c)) { content_type.to_ascii_lowercase() } else { String::new() }
}

// `multipartFormDataParser`.
fn multipart(input: &[u8], boundary: &str) -> Result<Form, FormError> {
    let boundary = format!("--{boundary}").into_bytes();
    let mut position = 0;
    while crlf_at(input, position) {
        position += 2;
    }
    let mut trailing = input.len();
    while trailing >= 2 && input[trailing - 1] == b'\n' && input[trailing - 2] == b'\r' {
        trailing -= 2;
    }
    let input = &input[..trailing];
    let mut form = Form::new();
    loop {
        if !starts_at(input, &boundary, position) {
            return Err(parsing_error("expected a value starting with -- and the boundary"));
        }
        position += boundary.len();
        if (position + 2 == input.len() && starts_at(input, b"--", position)) || (position + 4 == input.len() && starts_at(input, b"--\r\n", position)) {
            return Ok(form);
        }
        if !crlf_at(input, position) {
            return Err(parsing_error("expected CRLF"));
        }
        position += 2;
        let headers = part_headers(input, &mut position)?;
        position += 2;
        let Some(found) = index_of(input, &boundary[2..], position) else { return Err(parsing_error("expected boundary after body")) };
        let end = found.saturating_sub(4);
        let mut body = if end > position { input[position..end].to_vec() } else { Vec::new() };
        position += body.len();
        if headers.encoding.as_deref() == Some("base64") {
            body = node_base64(&String::from_utf8_lossy(&body));
        }
        if !crlf_at(input, position) {
            return Err(parsing_error("expected CRLF"));
        }
        position += 2;
        let value = match headers.filename {
            Some(name) => {
                let content_type = headers.content_type.unwrap_or_else(|| "text/plain".into());
                let content_type = if content_type.is_ascii() { blob_type(&content_type) } else { String::new() };
                FormValue::File(FormFile { name, content_type, bytes: body })
            }
            None => FormValue::Text(utf8_decode_bytes(&body)),
        };
        form.push((headers.name, value));
    }
}

fn hex_value(unit: u16) -> Option<u8> {
    char::from_u32(u32::from(unit)).and_then(|c| c.to_digit(16)).map(|digit| digit as u8)
}

// `querystring.unescapeBuffer(text).toString()`: Node keeps the low byte of every other UTF-16 unit.
fn unescape_buffer(text: &str) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut out = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if unit == u16::from(b'%') && index + 2 < units.len() {
            match (hex_value(units[index + 1]), hex_value(units[index + 2])) {
                (Some(high), Some(low)) => {
                    out.push(high * 16 + low);
                    index += 3;
                }
                _ => {
                    out.push(b'%');
                    index += 1;
                }
            }
            continue;
        }
        out.push(unit as u8);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// `querystring.unescape`: `decodeURIComponent`, or the byte-wise fallback when that throws.
fn unescape(text: &str) -> String {
    decode_uri_component(text).unwrap_or_else(|_| unescape_buffer(text))
}

// `new URLSearchParams(text)` as Node parses it (`parseParams` of `lib/internal/url.js`).
fn search_params(text: &str) -> Vec<(String, String)> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let is = |unit: u16, c: u8| unit == u16::from(c);
    let finish = |buffer: &[u16], encoded: bool| {
        let text = String::from_utf16_lossy(buffer);
        if encoded { unescape(&text) } else { text }
    };
    let mut out = Vec::new();
    let mut seen_separator = false;
    let mut buffer: Vec<u16> = Vec::new();
    let mut encoded = false;
    let mut check = 0;
    let start = usize::from(units.first().is_some_and(|&unit| is(unit, b'?')));
    let (mut index, mut pair_start, mut last) = (start, start, start);
    while index < units.len() {
        let unit = units[index];
        if is(unit, b'&') {
            if pair_start != index {
                buffer.extend_from_slice(&units[last..index]);
                out.push(finish(&buffer, encoded));
                if !seen_separator {
                    out.push(String::new());
                }
                (seen_separator, encoded, check) = (false, false, 0);
                buffer.clear();
            }
            (last, pair_start) = (index + 1, index + 1);
        } else if !seen_separator && is(unit, b'=') {
            buffer.extend_from_slice(&units[last..index]);
            out.push(finish(&buffer, encoded));
            (seen_separator, encoded, check) = (true, false, 0);
            buffer.clear();
            last = index + 1;
        } else if is(unit, b'+') {
            buffer.extend_from_slice(&units[last..index]);
            buffer.push(u16::from(b' '));
            last = index + 1;
        } else if !encoded {
            if is(unit, b'%') {
                check = 1;
            } else if check > 0 {
                if hex_value(unit).is_some() {
                    check += 1;
                    encoded = check == 3;
                } else {
                    check = 0;
                }
            }
        }
        index += 1;
    }
    if pair_start != index {
        buffer.extend_from_slice(&units[last..index]);
        out.push(finish(&buffer, encoded));
        if !seen_separator {
            out.push(String::new());
        }
    }
    let mut pairs = Vec::with_capacity(out.len() / 2);
    let mut items = out.into_iter();
    while let (Some(name), Some(value)) = (items.next(), items.next()) {
        pairs.push((name, value));
    }
    pairs
}
