//! Port of `lib/ai.mjs`: hosted-API AI features for saved cards (embeddings, TL;DR summaries, tag suggestions and tag
//! picking), with the same requests, retries and errors.
//!
//! Requests go through [`Fetch::fetch_unfollowed`]: the JS sent `redirect: 'manual'`, so a 3xx answer is an error
//! here, and a followed redirect would carry the API key to another host. A [`Fetch`] error's message is the
//! reason shown to the caller (`ECONNREFUSED`, `TimeoutError`); an empty message reads as `network error`.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::{LazyLock, Mutex};

use ai_kit::embed;
use futures_util::future::{Either, select, try_join, try_join3};
use mm_js::json;
use mm_js::number::string_to_number;
use mm_js::text::{decode_utf8, is_whitespace, len, slice, to_lower, trim};
use mm_js::{Value, base64, obj};
use regex::Regex;

use crate::ai_tags::{curate_tags, tag_key};
use crate::host::Fetch;
use crate::http::{Body, Headers, Request, Response};
use crate::render::access::text;
use crate::runtime;
use crate::Error;

const CHAT_TEXT_CHARS: usize = 8000;
const JEV_TEXT_CHARS: usize = 6000;
const JEV_MODEL: &str = "jev-latest";
const JEV_MIN_P: f64 = 0.6;
const JEV_QUESTIONS_PER_REQUEST: usize = 100;
const MAX_TAGS: usize = 10;
const NEAREST_TAGS: usize = 20;
const CHAT_IMAGE_MAX_BYTES: usize = 10 * 1024 * 1024;
const JEV_MIN_SIDE: u32 = 256;
const JEV_MAX_SIDE: u32 = 1568;
const JEV_MIN_PIXELS: f64 = 78_400.0;
const JEV_MAX_PIXELS: f64 = 2_458_624.0;
const JEV_MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MS: u64 = 60_000;

const SUMMARY_PROMPT_LINES: [&str; 9] = [
    "You file items saved to a personal visual notebook, like mymind. Reply with one JSON object: {\"title\": string, \"summary\": string, \"tags\": string[]}.",
    "title: a specific short headline of at most 8 words, in the language of the item.",
    "For a Note or Content item, which the person wrote or kept, the title is its name, the way they would name it: 2 to 6 plain words in sentence case saying what it is about, never \"Note\", \"Untitled\" or a date, no quotes, no full stop.",
    "summary: the TL;DR. One or two plain sentences (at most 40 words) saying what the item is and what it is about.",
    "For articles, web pages, repositories, apps and posts, derive the summary and tags from the supplied text. A cover illustration is not the subject.",
    "Use the image to inform the summary and tags only for Image, Screenshot or Video items, or when the supplied text has fewer than 200 characters.",
    "No markdown, no quotes, no filler such as \"This article\" or \"The image shows\". Use the language of the item.",
    "tags: 3 to 8 short lowercase labels (one or two words each) for the subject, the field and the kind of thing it is,",
    "such as \"typography\", \"recipe\" or \"tutorial\". No \"#\", no site names, no generic words like \"website\", \"link\", \"page\" or \"image\".",
];

static NULL: Value = Value::Null;

static VOCABULARY: LazyLock<Mutex<HashMap<String, Vec<f32>>>> = LazyLock::new(Default::default);

static FENCES: LazyLock<Regex> = LazyLock::new(|| {
    const WS: &str = r"[\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";
    Regex::new(&format!("^{WS}*```(?:json)?{WS}*|{WS}*```{WS}*$")).expect("fence pattern")
});

/// The hosted API a failed call talked to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    /// Gemini embeddings.
    Gemini,
    /// The chat model.
    Chat,
    /// TypeSafe Jev.
    Typesafe,
}

impl Service {
    /// The name the JS gave the service (`gemini`, `chat`, `typesafe`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::Chat => "chat",
            Self::Typesafe => "typesafe",
        }
    }
}

/// Failure of a hosted AI call, after retries; never carries credentials (`AiError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiError {
    /// Human-readable cause.
    pub message: String,
    /// The API the call used, when the failure came from one.
    pub service: Option<Service>,
    /// The final HTTP status, when there was one.
    pub status: Option<u16>,
    /// `Retry-After` on a final 429, in milliseconds, capped at 30 seconds.
    pub retry_after_ms: Option<u64>,
    invalid: bool,
}

impl AiError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), service: None, status: None, retry_after_ms: None, invalid: false }
    }

    fn service(message: impl Into<String>, service: Service) -> Self {
        Self { service: Some(service), ..Self::new(message) }
    }

    /// Whether the JS threw a `TypeError` here instead of an `AiError` (a card type naming an `Object.prototype` member).
    pub fn is_type_error(&self) -> bool {
        self.invalid
    }
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AiError {}

impl From<Error> for AiError {
    fn from(error: Error) -> Self {
        match error {
            Error::Invalid(message) => Self { invalid: true, ..Self::new(message) },
            other => Self::new(other.to_string()),
        }
    }
}

impl From<AiError> for Error {
    /// An AI failure is an internal failure; a `TypeError` stays one.
    fn from(error: AiError) -> Self {
        if error.invalid { Error::invalid(error.message) } else { Error::internal(error.message) }
    }
}

/// A cover image of a card.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AiImage {
    /// The declared MIME type; the APIs go by the sniffed one.
    pub mime: String,
    /// The image file.
    pub data: Vec<u8>,
}

/// A card to analyze or embed (`AiItem`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AiItem {
    /// Card title; may be empty.
    pub title: Option<String>,
    /// Card type; controls when images inform summaries and tags.
    pub kind: Option<String>,
    /// Description or body text; for a query embedding, the query string.
    pub text: Option<String>,
    /// Source URL.
    pub url: Option<String>,
    /// Source domain.
    pub domain: Option<String>,
    /// Cover image; used only when the target API accepts its format and size.
    pub image: Option<AiImage>,
}

/// How an embedding is meant to be compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbedKind {
    /// A card, formatted as `title: … | text: …`.
    Document,
    /// A search query or tag name, formatted as `task: search result | query: …`.
    Query,
}

/// One text for [`Ai::embed_texts`].
#[derive(Clone, Copy, Debug)]
pub enum EmbedInput<'a> {
    /// A bare string.
    Text(&'a str),
    /// A card.
    Item(&'a AiItem),
}

/// A tag vocabulary entry, optionally with its stored embedding.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VocabularyEntry {
    /// The tag name.
    pub name: String,
    /// A stored 768-dimension embedding that saves the API call; other lengths are ignored.
    pub embedding: Option<Vec<f64>>,
}

impl From<&str> for VocabularyEntry {
    fn from(name: &str) -> Self {
        Self { name: name.to_owned(), embedding: None }
    }
}

impl From<&String> for VocabularyEntry {
    fn from(name: &String) -> Self {
        Self::from(name.as_str())
    }
}

/// A vocabulary tag with its query embedding.
#[derive(Clone, Debug, PartialEq)]
pub struct VocabularyTag {
    /// The tag name in the first spelling given.
    pub name: String,
    /// Unit-length 768-dimension vector.
    pub embedding: Vec<f32>,
}

/// What [`Ai::describe`] returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    /// At most eight words.
    pub title: String,
    /// The TL;DR.
    pub summary: String,
    /// At most ten normalized labels.
    pub tags: Vec<String>,
}

/// A tag Jev judged.
#[derive(Clone, Debug, PartialEq)]
pub struct PickedTag {
    /// The caller's spelling.
    pub name: String,
    /// Probability that the tag fits.
    pub p: f64,
}

/// Where a final tag came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagSource {
    /// A tag the library already has.
    Vocabulary,
    /// A new tag from the chat model.
    Chat,
}

/// A final tag of [`Ai::analyze`].
#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisTag {
    /// The tag name, spelled as in the vocabulary when it is there.
    pub name: String,
    /// Jev's probability, or `None` when Jev was not used.
    pub p: Option<f64>,
    /// Whether the library already had the tag.
    pub source: TagSource,
}

/// What [`Ai::analyze`] returns.
#[derive(Clone, Debug, PartialEq)]
pub struct Analysis {
    /// At most eight words.
    pub title: String,
    /// The TL;DR.
    pub summary: String,
    /// At most ten tags.
    pub tags: Vec<AnalysisTag>,
    /// The document vector.
    pub embedding: Vec<f32>,
}

impl Analysis {
    /// The object `analyze` resolved: `{ title, summary, tags: [{ name, p, source }], embedding }`.
    pub fn to_value(&self) -> Value {
        let tags = self
            .tags
            .iter()
            .map(|tag| {
                let source = if tag.source == TagSource::Vocabulary { "vocabulary" } else { "chat" };
                obj! { "name" => tag.name.as_str(), "p" => tag.p, "source" => source }.into()
            })
            .collect();
        let embedding = self.embedding.iter().map(|value| Value::Number(f64::from(*value))).collect();
        obj! { "title" => self.title.as_str(), "summary" => self.summary.as_str(), "tags" => Value::Array(tags), "embedding" => Value::Array(embedding) }
            .into()
    }
}

fn default_random() -> f64 {
    let bytes: [u8; 8] = runtime::random_bytes(8).try_into().expect("eight bytes");
    (u64::from_le_bytes(bytes) >> 11) as f64 / 9_007_199_254_740_992.0
}

fn env_or(name: &str, fallback: &str) -> String {
    runtime::env(name).filter(|value| !value.is_empty()).unwrap_or_else(|| fallback.to_owned())
}

fn trim_base(url: String) -> String {
    url.trim_end_matches('/').to_owned()
}

fn gemini_base() -> String {
    trim_base(env_or("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta"))
}

fn chat_base() -> String {
    trim_base(env_or("AI_CHAT_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai"))
}

fn typesafe_base() -> String {
    trim_base(env_or("TYPESAFE_BASE_URL", "https://api.typesafe.ai"))
}

fn require_key(name: &str, purpose: &str) -> std::result::Result<String, AiError> {
    runtime::env(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AiError::new(format!("{name} is not set (needed for {purpose}); add it to .env")))
}

fn redact(value: &str) -> String {
    let mut out = value.to_owned();
    for name in ["GEMINI_API_KEY", "AI_CHAT_API_KEY", "TYPESAFE_API_KEY"] {
        if let Some(secret) = runtime::env(name).filter(|secret| !secret.is_empty()) {
            out = out.replace(&secret, "[redacted]");
        }
    }
    out
}

fn collapse(value: &str) -> String {
    value.split(is_whitespace).filter(|word| !word.is_empty()).collect::<Vec<_>>().join(" ")
}

fn clip(value: Option<&str>, max: usize) -> String {
    slice(&collapse(value.unwrap_or("")), 0, Some(max as i64))
}

fn status_text(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        509 => "Bandwidth Limit Exceeded",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "",
    }
}

fn header_ci(headers: &Headers, name: &str) -> Option<String> {
    let values: Vec<&str> = headers.iter().filter(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value).collect();
    (!values.is_empty()).then(|| values.join(", "))
}

// `Retry-After` as milliseconds, capped at 30s, matching the retry delay.
fn retry_after_ms(headers: &Headers) -> Option<u64> {
    let seconds = string_to_number(&header_ci(headers, "retry-after")?);
    (seconds.is_finite() && seconds >= 0.0).then(|| (seconds.min(30.0) * 1000.0) as u64)
}

struct Deadline {
    timer: Pin<Box<dyn Future<Output = ()>>>,
    expired: bool,
}

impl Deadline {
    fn start() -> Self {
        Self { timer: Box::pin(runtime::sleep(TIMEOUT_MS)), expired: false }
    }

    /// Runs `work` until the attempt's timer fires; `None` once it has.
    async fn run<T>(&mut self, work: impl Future<Output = T>) -> Option<T> {
        if self.expired {
            return None;
        }
        match select(pin!(work), &mut self.timer).await {
            Either::Left((value, _)) => Some(value),
            Either::Right(_) => {
                self.expired = true;
                None
            }
        }
    }
}

async fn read_text(deadline: &mut Deadline, body: Body) -> Option<String> {
    match deadline.run(body.bytes()).await {
        Some(Ok(bytes)) => Some(decode_utf8(&bytes)),
        _ => None,
    }
}

fn error_detail_text(raw: &str) -> Option<String> {
    let mut detail = Some(raw.to_owned());
    if let Ok(parsed) = json::parse(raw) {
        let body = match &parsed {
            Value::Array(items) => items.first().unwrap_or(&mm_js::UNDEFINED),
            other => other,
        };
        let candidates = [body.get("error").get("message"), body.get("message"), body.get("detail"), body.get("error")];
        let found = candidates.into_iter().find(|value| !value.is_nullish()).unwrap_or(body);
        detail = match found {
            Value::String(text) => Some(text.clone()),
            other => json::stringify(other).ok().flatten(),
        };
    }
    detail
}

fn sniff_image(data: &[u8]) -> Option<Sniffed> {
    let be32 = |at: usize| u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let be16 = |at: usize| u16::from_be_bytes([data[at], data[at + 1]]);
    let length = data.len();
    if length >= 24 && be32(0) == 0x8950_4e47 && be32(4) == 0x0d0a_1a0a {
        let mut animated = false;
        let mut at: u64 = 8;
        while at + 8 <= length as u64 {
            let start = at as usize;
            let kind = &data[start + 4..start + 8];
            if kind == b"acTL" {
                animated = true;
            }
            if kind == b"IDAT" {
                break;
            }
            at += 12 + u64::from(be32(start));
        }
        return Some(Sniffed { mime: "image/png", width: be32(16), height: be32(20), animated, bytes: length });
    }
    if length >= 4 && data[0] == 0xff && data[1] == 0xd8 {
        let mut at = 2;
        while at + 4 <= length {
            if data[at] != 0xff {
                at += 1;
                continue;
            }
            let marker = data[at + 1];
            if marker == 0xff {
                at += 1;
                continue;
            }
            if marker == 0x01 || (0xd0..=0xd9).contains(&marker) {
                at += 2;
                continue;
            }
            let block = usize::from(be16(at + 2));
            let is_frame = (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker);
            if is_frame && at + 9 <= length {
                return Some(Sniffed { mime: "image/jpeg", width: u32::from(be16(at + 7)), height: u32::from(be16(at + 5)), animated: false, bytes: length });
            }
            if marker == 0xda {
                break;
            }
            at += 2 + block;
        }
        return None;
    }
    if length >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some(Sniffed { mime: "image/webp", width: 0, height: 0, animated: false, bytes: length });
    }
    None
}

struct Sniffed {
    mime: &'static str,
    width: u32,
    height: u32,
    animated: bool,
    bytes: usize,
}

fn embed_image(item: &AiItem) -> Option<(&'static str, String)> {
    let data = &item.image.as_ref()?.data;
    let info = sniff_image(data)?;
    if info.animated || (info.mime != "image/png" && info.mime != "image/jpeg") {
        return None;
    }
    Some((info.mime, base64::encode(data)))
}

fn chat_image(item: &AiItem) -> Option<(&'static str, String)> {
    let data = &item.image.as_ref()?.data;
    let info = sniff_image(data)?;
    (info.bytes <= CHAT_IMAGE_MAX_BYTES).then(|| (info.mime, base64::encode(data)))
}

fn jev_image(item: &AiItem) -> Option<(&'static str, String)> {
    let data = &item.image.as_ref()?.data;
    let info = sniff_image(data)?;
    if info.animated || (info.mime != "image/png" && info.mime != "image/jpeg") {
        return None;
    }
    let pixels = f64::from(info.width) * f64::from(info.height);
    let in_range = info.width >= JEV_MIN_SIDE
        && info.height >= JEV_MIN_SIDE
        && info.width <= JEV_MAX_SIDE
        && info.height <= JEV_MAX_SIDE
        && (JEV_MIN_PIXELS..=JEV_MAX_PIXELS).contains(&pixels)
        && info.bytes <= JEV_MAX_BYTES;
    in_range.then(|| (if info.mime == "image/png" { "png" } else { "jpg" }, base64::encode(data)))
}

fn normalize_tag(value: &Value) -> String {
    let name = text(value);
    let name = trim(&name).trim_start_matches('#');
    let mut spaced = String::with_capacity(name.len());
    let mut in_run = false;
    for character in name.chars() {
        if is_whitespace(character) {
            if !in_run {
                spaced.push(' ');
            }
            in_run = true;
        } else {
            in_run = false;
            spaced.push(character);
        }
    }
    let name = to_lower(&spaced);
    if len(&name) > 0 && len(&name) <= 40 && name.split(' ').count() <= 4 { name } else { String::new() }
}

/// The answer's `values` as a unit vector; the checks and the normalising are the kit's.
fn to_vector(values: &Value) -> std::result::Result<Vec<f32>, AiError> {
    let numbers = match values {
        Value::Array(items) => Some(items.iter().map(Value::to_number).collect::<Vec<f64>>()),
        _ => None,
    };
    embed::unit_vector(numbers.as_deref()).map_err(|error| AiError::service(error.to_string(), Service::Gemini))
}

fn has_embeddable_content(item: &AiItem) -> bool {
    !clip(item.title.as_deref(), 1).is_empty() || !clip(item.text.as_deref(), 1).is_empty() || embed_image(item).is_some()
}

fn visual_analysis(item: &AiItem) -> bool {
    ["Image", "Screenshot", "Video", "YouTubeVideo", "InstagramReel"].contains(&item.kind.as_deref().unwrap_or(""))
        || len(&clip(item.text.as_deref(), 300)) < 200
}

fn gemini_headers() -> std::result::Result<Headers, AiError> {
    Ok(Headers::from([("x-goog-api-key", require_key("GEMINI_API_KEY", "Gemini embeddings and chat")?)]))
}

fn text_part(value: String) -> Value {
    obj! { "text" => value }.into()
}

/// Format the text portion of a document embedding identically for saves and backfill batches (`documentText`).
///
/// # Arguments
/// * `item` - Card title and body text.
///
/// # Returns
/// Bounded document prompt, or an empty string without text.
pub fn document_text(item: &AiItem) -> String {
    embed::document_text(item.title.as_deref(), item.text.as_deref())
}

/// Cosine similarity of two embeddings (`cosine`), from the AI kit shared with Pully.
pub use ai_kit::embed::cosine;

/// Forget the vocabulary vectors computed in this process (for tests; the JS cache only ends with the process).
pub fn forget_vocabulary() {
    VOCABULARY.lock().expect("vocabulary lock").clear();
}

/// The hosted AI calls of one host: its outbound HTTP and, for tests, its backoff jitter.
pub struct Ai<'a> {
    fetch: &'a dyn Fetch,
    random: &'a dyn Fn() -> f64,
}

impl<'a> Ai<'a> {
    /// The AI calls over a host's outbound HTTP, with random backoff jitter.
    ///
    /// # Arguments
    /// * `fetch` - The host's outbound HTTP.
    pub fn new(fetch: &'a dyn Fetch) -> Self {
        Self { fetch, random: &default_random }
    }

    /// The same calls with `Math.random` replaced, so a test can fix the backoff delays.
    ///
    /// # Arguments
    /// * `random` - A number in [0, 1) per call.
    #[must_use]
    pub fn with_random(self, random: &'a dyn Fn() -> f64) -> Self {
        Self { random, ..self }
    }

    fn retry_delay(&self, headers: Option<&Headers>, attempt: u32) -> u64 {
        let header = headers.and_then(|headers| header_ci(headers, "retry-after")).map_or(f64::NAN, |raw| string_to_number(&raw));
        let ms = if header.is_finite() && header >= 0.0 {
            header.min(30.0) * 1000.0
        } else {
            1000.0 * 2f64.powi(attempt as i32) * (0.75 + (self.random)() * 0.5)
        };
        ms as u64
    }

    async fn post_json(&self, service: Service, url: &str, headers: &Headers, body: &Value) -> std::result::Result<Value, AiError> {
        let name = service.as_str();
        let payload = json::to_string(body);
        let mut attempt = 0;
        loop {
            let mut deadline = Deadline::start();
            let sent: std::result::Result<Response, String> = match Request::new("POST", url) {
                Err(_) => Err("ERR_INVALID_URL".into()),
                Ok(mut request) => {
                    request.headers = Headers::from([("content-type", "application/json")]).with(headers);
                    request.body = Body::from(payload.clone());
                    match deadline.run(self.fetch.fetch_unfollowed(request)).await {
                        Some(Ok(response)) => Ok(response),
                        Some(Err(error)) => Err(if error.to_string().is_empty() { "network error".into() } else { error.to_string() }),
                        None => Err("TimeoutError".into()),
                    }
                }
            };
            let response = match sent {
                Ok(response) => response,
                Err(reason) => {
                    if attempt < MAX_RETRIES {
                        runtime::sleep(self.retry_delay(None, attempt)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(AiError::service(format!("{name} request failed ({reason}); check the connection"), service));
                }
            };
            let status = response.status;
            if (200..300).contains(&status) {
                let non_json = || AiError { status: Some(status), ..AiError::service(format!("{name} returned a non-JSON body"), service) };
                let raw = read_text(&mut deadline, response.body).await.ok_or_else(non_json)?;
                return json::parse(&raw).map_err(|_| non_json());
            }
            if (status == 429 || status >= 500) && attempt < MAX_RETRIES {
                let _ = read_text(&mut deadline, response.body).await;
                runtime::sleep(self.retry_delay(Some(&response.headers), attempt)).await;
                attempt += 1;
                continue;
            }
            let retry_after_ms = (status == 429).then(|| retry_after_ms(&response.headers)).flatten();
            let raw = read_text(&mut deadline, response.body).await.unwrap_or_default();
            let detail = clip(Some(&redact(&error_detail_text(&raw).unwrap_or_else(|| "undefined".into()))), 300);
            let detail = if detail.is_empty() { status_text(status).to_owned() } else { detail };
            return Err(AiError { status: Some(status), retry_after_ms, ..AiError::service(format!("{name} {status}: {detail}"), service) });
        }
    }

    /// Embed one item with gemini-embedding-2 into the shared 768-dimension space (`embed`).
    ///
    /// # Arguments
    /// * `item` - For a document the card; for a query only `text` (or `title`) is read and holds the query.
    /// * `kind` - `Document` formats the text as `title: … | text: …` and attaches the image when it is PNG or JPEG;
    ///   `Query` formats `task: search result | query: …` and ignores any image.
    ///
    /// # Returns
    /// Unit-length vector of 768 floats; compare with [`cosine`].
    ///
    /// # Errors
    /// `GEMINI_API_KEY` is missing, there is nothing to embed, or the API fails after retries.
    pub async fn embed(&self, item: &AiItem, kind: EmbedKind) -> std::result::Result<Vec<f32>, AiError> {
        let mut parts = Vec::new();
        if kind == EmbedKind::Query {
            let source = item.text.as_deref().filter(|text| !text.is_empty()).or(item.title.as_deref());
            let Some(query) = embed::query_text(source.unwrap_or("")) else {
                return Err(AiError::new("embed: the query is empty"));
            };
            parts.push(text_part(query));
        } else {
            let document = document_text(item);
            let image = embed_image(item);
            if document.is_empty() && image.is_none() {
                return Err(AiError::new("embed: the item has no title, text or usable image"));
            }
            if !document.is_empty() {
                parts.push(text_part(document));
            }
            if let Some((mime, data)) = image {
                parts.push(obj! { "inline_data" => obj! { "mime_type" => mime, "data" => data } }.into());
            }
        }
        let url = embed::embed_url(&gemini_base());
        let body: Value = obj! {
            "model" => embed::MODEL_PATH,
            "content" => obj! { "parts" => parts },
            "output_dimensionality" => embed::DIMENSIONS,
        }
        .into();
        let data = self.post_json(Service::Gemini, &url, &gemini_headers()?, &body).await?;
        to_vector(data.get("embedding").get("values"))
    }

    /// Embed many short texts (tag names) with gemini-embedding-2 batch embedding (`embedTexts`).
    ///
    /// # Arguments
    /// * `texts` - Non-empty strings, or document items; any number (sent in batches of 100).
    /// * `kind` - `Query` formats tag and query text; `Document` uses [`document_text`].
    /// * `on_usage` - Receives each batch's reported usage metadata, or null.
    ///
    /// # Returns
    /// One 768-dimension vector per input, in order.
    ///
    /// # Errors
    /// `GEMINI_API_KEY` is missing, a text is empty, or the API fails after retries.
    pub async fn embed_texts(
        &self,
        texts: &[EmbedInput<'_>],
        kind: EmbedKind,
        mut on_usage: Option<&mut dyn FnMut(&Value)>,
    ) -> std::result::Result<Vec<Vec<f32>>, AiError> {
        let mut prepared = Vec::with_capacity(texts.len());
        for input in texts {
            if kind == EmbedKind::Document {
                let formatted = match input {
                    EmbedInput::Text(body) => embed::document_text(None, Some(body)),
                    EmbedInput::Item(item) => document_text(item),
                };
                if formatted.is_empty() {
                    return Err(AiError::new("embedTexts: empty document"));
                }
                prepared.push(formatted);
            } else {
                let Some(query) = embed::query_text(match input { EmbedInput::Text(body) => body, EmbedInput::Item(_) => "[object Object]" }) else {
                    return Err(AiError::new("embedTexts: empty text"));
                };
                prepared.push(query);
            }
        }
        let mut vectors = Vec::with_capacity(prepared.len());
        for batch in prepared.chunks(embed::BATCH_SIZE) {
            let url = embed::batch_url(&gemini_base());
            let requests: Vec<Value> = batch
                .iter()
                .map(|body| {
                    obj! {
                        "model" => embed::MODEL_PATH,
                        "content" => obj! { "parts" => vec![text_part(body.clone())] },
                        "output_dimensionality" => embed::DIMENSIONS,
                    }
                    .into()
                })
                .collect();
            let body: Value = obj! { "requests" => requests }.into();
            let data = self.post_json(Service::Gemini, &url, &gemini_headers()?, &body).await?;
            let embeddings = data.get("embeddings");
            let Value::Array(entries) = embeddings else {
                return Err(self.batch_mismatch(embeddings, batch.len()));
            };
            if entries.len() != batch.len() {
                return Err(self.batch_mismatch(embeddings, batch.len()));
            }
            if let Some(report) = on_usage.as_mut() {
                report(data.get("usageMetadata").or(data.get("usage_metadata")).or(&NULL));
            }
            for entry in entries {
                vectors.push(to_vector(entry.get("values"))?);
            }
        }
        Ok(vectors)
    }

    fn batch_mismatch(&self, embeddings: &Value, expected: usize) -> AiError {
        let count = match embeddings {
            Value::Array(items) => items.len().to_string(),
            Value::String(body) => len(body).to_string(),
            Value::Object(object) => match object.get("length") {
                Some(length) if !length.is_nullish() => length.to_js_string(),
                _ => "0".into(),
            },
            _ => "0".into(),
        };
        AiError::service(format!("gemini batch returned {count} embeddings for {expected} texts"), Service::Gemini)
    }

    /// Ask the cheap chat model (`AI_CHAT_MODEL`, default `gemini-2.5-flash-lite`) for a TL;DR and tag suggestions
    /// (`describe`).
    ///
    /// # Arguments
    /// * `item` - The card; supported images inform visual or text-sparse cards only.
    ///
    /// # Returns
    /// At most eight title words, a summary and at most ten labels.
    ///
    /// # Errors
    /// The key is missing, the model returns no usable JSON or summary, or the API fails after retries; a card type
    /// naming an `Object.prototype` member is a `TypeError` ([`AiError::is_type_error`]).
    pub async fn describe(&self, item: &AiItem) -> std::result::Result<Description, AiError> {
        let model = env_or("AI_CHAT_MODEL", "gemini-2.5-flash-lite");
        let key = match runtime::env("AI_CHAT_API_KEY").filter(|key| !key.is_empty()) {
            Some(key) => key,
            None => require_key("GEMINI_API_KEY", "the chat model")?,
        };
        let or = |value: String, fallback: &str| if value.is_empty() { fallback.to_owned() } else { value };
        let source = [clip(item.domain.as_deref(), 200), clip(item.url.as_deref(), 500)].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" ");
        let lines = [
            format!("Title: {}", or(clip(item.title.as_deref(), 500), "(none)")),
            format!("Type: {}", or(clip(item.kind.as_deref(), 100), "(unknown)")),
            format!("Source: {}", or(source, "(unknown)")),
            format!("Text: {}", or(clip(item.text.as_deref(), CHAT_TEXT_CHARS), "(none)")),
        ];
        let mut content: Vec<Value> = vec![obj! { "type" => "text", "text" => lines.join("\n") }.into()];
        let image = if visual_analysis(item) { chat_image(item) } else { None };
        if let Some((mime, data)) = image {
            content.push(obj! { "type" => "image_url", "image_url" => obj! { "url" => format!("data:{mime};base64,{data}") } }.into());
        }
        let url = format!("{}/chat/completions", chat_base());
        let headers = Headers::from([("authorization", format!("Bearer {key}"))]);
        let body: Value = obj! {
            "model" => model.as_str(),
            "temperature" => 0.2,
            "response_format" => obj! { "type" => "json_schema", "json_schema" => obj! { "name" => "card", "schema" => summary_schema() } },
            "messages" => vec![
                Value::from(obj! { "role" => "system", "content" => SUMMARY_PROMPT_LINES.join("\n") }),
                Value::from(obj! { "role" => "user", "content" => content }),
            ],
        }
        .into();
        let mut parsed = Value::Undefined;
        let mut attempt = 1;
        while !parsed.truthy() {
            let data = match self.post_json(Service::Chat, &url, &headers, &body).await {
                Ok(data) => data,
                Err(mut error) => {
                    if matches!(error.status, Some(404 | 403)) {
                        error.message.push_str(&format!(" (model {model}; set AI_CHAT_MODEL to a model this key can use)"));
                    }
                    return Err(error);
                }
            };
            let content = data.get("choices").at(0).get("message").get("content").to_js_string();
            match json::parse(&FENCES.replace_all(&content, "")) {
                Ok(value) => parsed = value,
                Err(_) if attempt >= 2 => {
                    return Err(AiError::service(format!("chat model {model} did not return valid JSON"), Service::Chat));
                }
                Err(_) => {}
            }
            attempt += 1;
        }
        let summary = clip(Some(&text(parsed.get("summary"))), 600);
        if summary.is_empty() {
            return Err(AiError::service(format!("chat model {model} returned no summary"), Service::Chat));
        }
        let proposed: Vec<Value> = match parsed.get("tags") {
            Value::Array(items) => items.iter().map(normalize_tag).filter(|name| !name.is_empty()).map(|name| obj! { "name" => name }.into()).collect(),
            _ => Vec::new(),
        };
        let tags = curate_tags(&proposed, item.kind.as_deref().unwrap_or("undefined"), &[], &[])?
            .iter()
            .map(|tag| tag.get("name").to_js_string())
            .collect();
        let title = clip(Some(&text(parsed.get("title"))), 200).split(' ').take(8).collect::<Vec<_>>().join(" ");
        Ok(Description { title, summary, tags })
    }

    /// Judge candidate tags against an item with TypeSafe Jev, one yes/no question per candidate (`pickTags`).
    ///
    /// # Arguments
    /// * `item` - The card; the image is sent only when it is PNG or JPEG, 256-1568 px per side and within Jev's
    ///   pixel and size limits, otherwise it is left out (never resized).
    /// * `candidates` - Candidate tag names; duplicates (by tag key) and empty names are dropped, and more than 100
    ///   are judged in further requests.
    ///
    /// # Returns
    /// Candidates with probability of fit at least 0.6, highest first, at most 10; `name` keeps the caller's
    /// spelling. Empty when the item has no text and no usable image.
    ///
    /// # Errors
    /// `TYPESAFE_API_KEY` is missing or the API fails after retries.
    pub async fn pick_tags(&self, item: &AiItem, candidates: &[String]) -> std::result::Result<Vec<PickedTag>, AiError> {
        let key = require_key("TYPESAFE_API_KEY", "tag picking with TypeSafe Jev")?;
        let mut seen = HashSet::new();
        let mut names: Vec<String> = Vec::new();
        for candidate in candidates {
            let name = trim(candidate);
            if !name.is_empty() && seen.insert(tag_key(name)) {
                names.push(name.to_owned());
            }
        }
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let image = if visual_analysis(item) { jev_image(item) } else { None };
        let mut state = mm_js::Object::new();
        let mut present = false;
        for (field, value) in [
            ("title", clip(item.title.as_deref(), 500)),
            ("source", clip(item.domain.as_deref(), 200)),
            ("url", clip(item.url.as_deref(), 500)),
            ("text", clip(item.text.as_deref(), JEV_TEXT_CHARS)),
        ] {
            if !value.is_empty() {
                present = true;
                state.insert(field, value);
            }
        }
        let image_ref = image.as_ref().map(|_| format!("mymind-image-{}", runtime::random_uuid()));
        if let Some(reference) = &image_ref {
            present = true;
            state.insert("image", reference.as_str());
        }
        if !present {
            return Ok(Vec::new());
        }
        let mut headers = Headers::from([("authorization", format!("Bearer {key}"))]);
        if image.is_some() {
            headers.set("TypeSafe-Danger-Zone-Images", "true");
        }
        let url = format!("{}/v1/systemone", typesafe_base());
        let mut picked: Vec<PickedTag> = Vec::new();
        for batch in names.chunks(JEV_QUESTIONS_PER_REQUEST) {
            let questions: mm_js::Object = batch
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let instructions = format!(
                        "Someone saved `item` to their personal library and wants to find it later. Would they file it under the tag {}?",
                        json::quote(name)
                    );
                    (format!("t{index}"), Value::from(obj! { "type" => "noul", "instructions" => instructions }))
                })
                .collect();
            let mut body = obj! { "model" => JEV_MODEL, "state" => obj! { "item" => state.clone() } };
            if let (Some((format, data)), Some(reference)) = (&image, &image_ref) {
                body.insert(
                    "assets",
                    obj! { reference.as_str() => obj! { "type" => "image", "format" => *format, "data_base64" => data.as_str() } },
                );
            }
            body.insert("questions", questions);
            let data = self.post_json(Service::Typesafe, &url, &headers, &body.into()).await?;
            for (index, name) in batch.iter().enumerate() {
                if let Value::Number(p) = data.get("answers").get(&format!("t{index}")).get("noul") {
                    picked.push(PickedTag { name: name.clone(), p: *p });
                }
            }
        }
        picked.retain(|tag| tag.p >= JEV_MIN_P);
        picked.sort_by(|a, b| by_difference(b.p - a.p));
        picked.truncate(MAX_TAGS);
        Ok(picked)
    }

    /// Embed a tag vocabulary as queries, reusing vectors already computed in this process or supplied by the
    /// caller (`embedVocabulary`).
    ///
    /// # Arguments
    /// * `vocabulary` - Existing tag names; an entry may carry a stored 768-dimension embedding to skip the API
    ///   call. Duplicates (by tag key) keep the first spelling.
    ///
    /// # Returns
    /// One entry per distinct tag, in input order.
    ///
    /// # Errors
    /// Embeddings are needed and `GEMINI_API_KEY` is missing or the API fails.
    pub async fn embed_vocabulary(&self, vocabulary: &[VocabularyEntry]) -> std::result::Result<Vec<VocabularyTag>, AiError> {
        let mut entries: Vec<(String, String)> = Vec::new();
        let mut known = HashSet::new();
        for entry in vocabulary {
            let name = trim(&entry.name).to_owned();
            let key = tag_key(&name);
            if name.is_empty() || !known.insert(key.clone()) {
                continue;
            }
            if let Some(embedding) = entry.embedding.as_ref().filter(|embedding| embedding.len() == embed::DIMENSIONS) {
                VOCABULARY.lock().expect("vocabulary lock").insert(key.clone(), embedding.iter().map(|value| *value as f32).collect());
            }
            entries.push((key, name));
        }
        let missing: Vec<&(String, String)> = {
            let cache = VOCABULARY.lock().expect("vocabulary lock");
            entries.iter().filter(|(key, _)| !cache.contains_key(key)).collect()
        };
        if !missing.is_empty() {
            let inputs: Vec<EmbedInput<'_>> = missing.iter().map(|(_, name)| EmbedInput::Text(name)).collect();
            let vectors = self.embed_texts(&inputs, EmbedKind::Query, None).await?;
            let mut cache = VOCABULARY.lock().expect("vocabulary lock");
            for ((key, _), vector) in missing.iter().zip(vectors) {
                cache.insert(key.clone(), vector);
            }
        }
        let cache = VOCABULARY.lock().expect("vocabulary lock");
        Ok(entries
            .iter()
            .map(|(key, name)| VocabularyTag { name: name.clone(), embedding: cache.get(key).cloned().unwrap_or_default() })
            .collect())
    }

    /// Auto-label a card: summary, final tags and embedding (`analyze`). The chat model proposes tags, the
    /// vocabulary's 20 nearest tags by embedding similarity join them as candidates, and Jev picks the final set;
    /// without `TYPESAFE_API_KEY` the chat model's tags are returned unjudged. An item with no title, no text and no
    /// PNG/JPEG image is embedded from its generated summary instead.
    ///
    /// # Arguments
    /// * `item` - The card to analyze.
    /// * `vocabulary` - The library's existing tag names, most used first; entries may carry stored embeddings.
    ///
    /// # Returns
    /// The summary, at most 10 tags (`Vocabulary` when the name is an existing library tag, spelled as there) and
    /// the document vector.
    ///
    /// # Errors
    /// A required key is missing or any hosted API fails after retries.
    pub async fn analyze(&self, item: &AiItem, vocabulary: &[VocabularyEntry]) -> std::result::Result<Analysis, AiError> {
        let (embedding, description, vocab) = if has_embeddable_content(item) {
            let (embedding, description, vocab) =
                try_join3(self.embed(item, EmbedKind::Document), self.describe(item), self.embed_vocabulary(vocabulary)).await?;
            (embedding, description, vocab)
        } else {
            let chained = async {
                let description = self.describe(item).await?;
                let summary = AiItem { text: Some(description.summary.clone()), ..AiItem::default() };
                let embedding = self.embed(&summary, EmbedKind::Document).await?;
                Ok::<_, AiError>((description, embedding))
            };
            let ((description, embedding), vocab) = try_join(chained, self.embed_vocabulary(vocabulary)).await?;
            (embedding, description, vocab)
        };
        let Description { title, summary, tags: proposed } = description;
        let spelling: HashMap<String, &str> = vocab.iter().map(|tag| (tag_key(&tag.name), tag.name.as_str())).collect();
        let vocabulary_names: Vec<String> = vocab.iter().map(|tag| tag.name.clone()).collect();
        let kind = item.kind.as_deref().unwrap_or("undefined");
        let source_of = |name: &str| if spelling.contains_key(&tag_key(name)) { TagSource::Vocabulary } else { TagSource::Chat };
        let resolve = |name: &str| spelling.get(&tag_key(name)).map_or_else(|| name.to_owned(), |known| (*known).to_owned());
        let record = |name: &str, p: Option<f64>| -> Value {
            obj! { "name" => name, "p" => p, "source" => if source_of(name) == TagSource::Vocabulary { "vocabulary" } else { "chat" } }.into()
        };

        let judged: Vec<Value> = if runtime::env("TYPESAFE_API_KEY").is_none_or(|key| key.is_empty()) {
            proposed.iter().map(|name| resolve(name)).map(|name| record(&name, None)).collect()
        } else {
            let mut scored: Vec<(&str, f64)> = vocab.iter().map(|tag| (tag.name.as_str(), cosine(&embedding, &tag.embedding))).collect();
            scored.sort_by(|a, b| by_difference(b.1 - a.1));
            let mut candidates: Vec<String> = scored.iter().take(NEAREST_TAGS).map(|(name, _)| (*name).to_owned()).collect();
            candidates.extend(proposed.iter().map(|name| resolve(name)));
            self.pick_tags(item, &candidates).await?.iter().map(|tag| record(&tag.name, Some(tag.p))).collect()
        };
        let tags = curate_tags(&judged, kind, &vocabulary_names, &[])?
            .iter()
            .map(|tag| AnalysisTag {
                name: tag.get("name").to_js_string(),
                p: tag.get("p").as_f64(),
                source: if tag.get("source").as_str() == Some("vocabulary") { TagSource::Vocabulary } else { TagSource::Chat },
            })
            .collect();
        Ok(Analysis { title, summary, tags, embedding })
    }
}

/// A sort comparator result as V8 reads a number: negative first, positive last, zero or NaN equal.
fn by_difference(difference: f64) -> std::cmp::Ordering {
    if difference < 0.0 {
        std::cmp::Ordering::Less
    } else if difference > 0.0 {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

fn summary_schema() -> Value {
    let string = || Value::from(obj! { "type" => "string" });
    obj! {
        "type" => "object",
        "properties" => obj! {
            "title" => string(),
            "summary" => string(),
            "tags" => obj! { "type" => "array", "items" => string() },
        },
        "required" => mm_js::arr!["title", "summary", "tags"],
    }
    .into()
}
