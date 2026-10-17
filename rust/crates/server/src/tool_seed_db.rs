//! `mymind seed-db`: port of `tools/seed-db.mjs` with the `recordingExtras` helpers of `tools/check-render.mjs`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use indexmap::IndexMap;
use mm_core::render::encoding::unescape_html;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::{Store, ns_from_iso};
use mm_core::{Error, Result, params};
use mm_js::number::string_to_number;
use mm_js::{Object, Value, json, obj, text};
use mm_msgpack::{DecodeOptions, decode_multi};
use mm_sqlite::Sqlite;
use regex::Regex;

use crate::node_path::{join, resolve};
use crate::read_metadata::import_read_metadata;
use crate::tools::embedding_item;

const API: &str = "https://access.mymind.com";
const TIE_STEP_NS: i128 = 1_000_000;
const USAGE: &str = "Usage: node tools/seed-db.mjs [--db data/mymind.db]";
const SPACE: &str = r"[\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";

static CARD_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r#"^<mymind-card(?-u:\b)[^>]*{SPACE}data-id="([^"]+)""#)).unwrap());
static CONFIG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<script id="config" type="application/json">([\s\S]*?)</script>"#).unwrap());
static CARD_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<mymind-card(?-u:\b)[^>]*>").unwrap());
static LINK_BUBBLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<a class="link-bubble"[^>]*>"#).unwrap());
static H3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<h3>([\s\S]*?)</h3>").unwrap());
static IMAGE_BOX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<mymind-image(?-u:\b)([^>]*)>([\s\S]*?)</mymind-image>").unwrap());
static RATIO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"aspect-ratio:([0-9]+)/([0-9]+)").unwrap());
static MEDIA_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(?:img|video)(?-u:\b)[^>]*>").unwrap());
static RENDITION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";([0-9]+)x([0-9]+)").unwrap());
static BACKGROUND: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"background-color:([^;]+)").unwrap());
static POST_BODY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<i class="platform"></i>\n\n    ([\s\S]*?)\n\n    "#).unwrap());
static AUTHOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<p class="author">([\s\S]*?)</p>"#).unwrap());
static GALLERY: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r#"<div class="gallery">([\s\S]*?)</div>{SPACE}*</div>"#)).unwrap());
static GALLERY_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<img class="lazy" data-(srcset|src)="([^"]*)" width="([0-9]+)" height="([0-9]+)"/>"#).unwrap());
static MAIN_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<img class="lazy main-img"[^>]*>"#).unwrap());
static APP_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<img srcset="[^"]*"[^>]*>"#).unwrap());
static DISTRIBUTOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"class="inner distributor_([^"]*)""#).unwrap());
static PRICE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<span class="price">([^<]*)</span>"#).unwrap());
static LABEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<div class="label">([\s\S]*?)</div>"#).unwrap());
static SPACE_DETAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^GET https://access.mymind.com/spaces/([^/?]+)$").unwrap());

struct Recording {
    html: String,
    json: String,
}

struct Recordings {
    capture: Vec<Recording>,
    stream: Vec<Recording>,
    trash: Vec<Recording>,
    spaces: Value,
    manual: Value,
    config: Value,
}

// The `error.message` the JS tool printed for an error.
fn message(error: &Error) -> String {
    let text = error.to_string();
    for prefix in ["SyntaxError: ", "TypeError: ", "RangeError: ", "Error: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return rest.to_owned();
        }
    }
    text
}

// Reading a property of `undefined` or `null`.
fn prop<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    if value.is_nullish() {
        return Err(Error::invalid(format!("Cannot read properties of {} (reading '{key}')", if value.is_undefined() { "undefined" } else { "null" })));
    }
    Ok(value.get(key))
}

fn bind(value: &Value) -> Result<SqlValue> {
    match value {
        Value::String(text) => Ok(SqlValue::Text(text.clone())),
        Value::Number(number) => Ok(SqlValue::Real(*number)),
        Value::Null => Ok(SqlValue::Null),
        Value::BigInt(number) => Ok(SqlValue::big(*number)),
        Value::Bytes(bytes) => Ok(SqlValue::Blob(bytes.clone())),
        _ => Err(Error::invalid("Provided value cannot be bound to SQLite parameter.")),
    }
}

fn nullish_or<'a>(value: &'a Value, fallback: &'a Value) -> &'a Value {
    if value.is_nullish() { fallback } else { value }
}

// `nsFromIso(value)`, which reads a non-string through its string form.
fn ns(value: &Value) -> Result<i128> {
    match value {
        Value::String(iso) => ns_from_iso(iso),
        other => ns_from_iso(&other.to_js_string()),
    }
}

fn read(file: &Path) -> Result<Vec<u8>> {
    std::fs::read(file).map_err(|error| Error::internal(format!("{error}: {}", file.display())))
}

fn read_text(file: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&read(file)?).into_owned())
}

fn is_js_space(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

// Read a quoted attribute without interpreting or executing the recorded markup.
fn attr(tag: Option<&str>, name: &str) -> Result<Option<String>> {
    let Some(tag) = tag else { return Ok(None) };
    let needle = format!("{name}=\"");
    let mut starts = vec![0];
    starts.extend(tag.char_indices().filter(|(_, character)| is_js_space(*character)).map(|(at, character)| at + character.len_utf8()));
    for start in starts {
        let Some(rest) = tag[start..].strip_prefix(&needle) else { continue };
        if let Some(end) = rest.find('"') {
            return unescape_html(&rest[..end]).map(Some);
        }
    }
    Ok(None)
}

fn capture(pattern: &Regex, haystack: &str, group: usize) -> Option<String> {
    pattern.captures(haystack).and_then(|found| found.get(group)).map(|found| found.as_str().to_owned())
}

fn number(digits: &str) -> Value {
    Value::Number(string_to_number(digits))
}

fn optional(value: Option<String>) -> Value {
    value.map_or(Value::Undefined, Value::from)
}

// Recover media metadata that the card JSON omits, without passing layout markup through.
fn cover_from_html(html: &str) -> Result<Value> {
    let Some(found) = IMAGE_BOX.captures(html) else { return Ok(Value::Undefined) };
    let (head, inner) = (found.get(1).map_or("", |m| m.as_str()), found.get(2).map_or("", |m| m.as_str()));
    let style = attr(Some(head), "style")?;
    let ratio = style.as_deref().and_then(|style| RATIO.captures(style));
    let tag = MEDIA_TAG.find(inner).map(|m| m.as_str());
    let (Some(ratio), Some(tag)) = (ratio, tag) else { return Ok(Value::Undefined) };
    let src = attr(Some(tag), "data-src")?;
    let rendition = src.as_deref().and_then(|src| RENDITION.captures(src));
    let (width, height) = match &rendition {
        Some(found) => (number(&found[1]), number(&found[2])),
        None => (number(&ratio[1]), number(&ratio[2])),
    };
    let size = |name: &str| -> Result<Value> { Ok(attr(Some(tag), name)?.map_or(Value::Undefined, |text| number(&text))) };
    Ok(obj! {
        "src" => optional(src),
        "width" => width,
        "height" => height,
        "dominantColor" => optional(style.as_deref().and_then(|style| capture(&BACKGROUND, style, 1))),
        "type" => if tag.starts_with("<video") { "video" } else { "image" },
        "svg" => attr(Some(head), "class")?.as_deref() == Some("svg"),
        "imageWidth" => size("width")?,
        "imageHeight" => size("height")?,
    }
    .into())
}

/// Recover only the source and entity fields missing from a recorded card's JSON (`recordingExtras` of
/// `tools/check-render.mjs`).
///
/// # Arguments
/// * `html` - The recorded card markup.
/// * `json` - The recorded card JSON text.
///
/// # Returns
/// The metadata object `renderCard` accepts as extras.
///
/// # Errors
/// Wherever the JS threw: unparsable JSON, a missing `blob` or cover on an image card, or a numeric entity past
/// U+10FFFF.
pub fn recording_extras(html: &str, json: &str) -> Result<Value> {
    let data = json::parse(json)?;
    let kind = prop(&data, "type")?.as_str().map(str::to_owned);
    let is = |names: &[&str]| kind.as_deref().is_some_and(|kind| names.contains(&kind));
    let mut extras = Object::new();
    extras.insert("id", optional(attr(CARD_TAG.find(html).map(|m| m.as_str()), "data-id")?));
    let link = attr(LINK_BUBBLE.find(html).map(|m| m.as_str()), "href")?;
    if let Some(link) = link {
        let source = data.get("source");
        let source_url = if source.is_nullish() { &Value::Undefined } else { source.get("url") };
        if source_url.as_str() != Some(&link) {
            extras.insert("linkUrl", link);
        }
    }
    if is(&["Article"]) {
        let title = data.get("title");
        let clipped = match title {
            Value::String(text) => text::len(text) == 200,
            Value::Array(items) => items.len() == 200,
            Value::Object(object) => object.get("length").and_then(Value::as_f64) == Some(200.0),
            _ => false,
        };
        if clipped {
            let Value::String(clip) = title else { return Err(Error::invalid("data.title.startsWith is not a function")) };
            let full = unescape_html(&capture(&H3, html, 1).unwrap_or_default())?;
            if full.starts_with(clip.as_str()) && full != *clip {
                extras.insert("title", full);
            }
        }
    }
    if !is(&["Image", "Video", "Document", "Note", "Content", "Repository"]) {
        extras.insert("cover", cover_from_html(html)?);
    }
    if is(&["Image"]) {
        let cover = cover_from_html(html)?;
        if cover.is_undefined() {
            return Err(Error::invalid("Cannot read properties of undefined (reading 'dominantColor')"));
        }
        let blob = prop(&data, "blob")?;
        let color = prop(blob, "dominantColor")?;
        let same = match (cover.get("dominantColor"), color) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        };
        let differs = !same;
        if differs || cover.get("src").as_str().is_some_and(|src| src.ends_with("-t.webp")) {
            extras.insert("cover", cover);
        }
    }
    if is(&["XPost", "ThreadsPost", "SubstackNote"]) {
        let gallery_html = GALLERY.captures(html).and_then(|found| found.get(1)).map_or("", |m| m.as_str());
        let mut gallery = Vec::new();
        for found in GALLERY_IMAGE.captures_iter(gallery_html) {
            let mut item = Object::new();
            item.insert(found[1].to_owned(), unescape_html(&found[2])?);
            item.insert("width", number(&found[3]));
            item.insert("height", number(&found[4]));
            gallery.push(Value::from(item));
        }
        let has_video = html.contains("class=\"fake-player\"") || extras.get("cover").is_some_and(|cover| cover.get("type").as_str() == Some("video"));
        extras.insert(
            "post",
            obj! {
                "bodyHtml" => optional(capture(&POST_BODY, html, 1)),
                "author" => unescape_html(&capture(&AUTHOR, html, 1).unwrap_or_default())?,
                "hasVideo" => has_video,
                "gallery" => gallery,
            },
        );
    }
    if is(&["InstagramReel"]) {
        let image = MAIN_IMAGE.find(html).map(|m| m.as_str());
        let size = |name: &str| -> Result<Value> { Ok(number_of(attr(image, name)?)) };
        extras.insert(
            "cover",
            obj! {
                "src" => optional(attr(image, "data-src")?),
                "width" => size("width")?,
                "height" => size("height")?,
                "isThumbnail" => !html.contains("instagram-reel not-thumbnail"),
            },
        );
    }
    if is(&["SoftwareApplication"]) {
        let image = APP_IMAGE.find(html).map(|m| m.as_str());
        extras.insert(
            "app",
            obj! {
                "distributor" => optional(capture(&DISTRIBUTOR, html, 1)),
                "srcset" => optional(attr(image, "srcset")?),
                "width" => number_of(attr(image, "width")?),
                "height" => number_of(attr(image, "height")?),
            },
        );
    }
    if is(&["Product"]) {
        extras.insert("product", obj! { "price" => unescape_html(&capture(&PRICE, html, 1).unwrap_or_default())? });
    }
    if is(&["RedditPost"]) {
        extras.insert(
            "reddit",
            obj! {
                "title" => unescape_html(&capture(&H3, html, 1).unwrap_or_default())?,
                "subreddit" => unescape_html(&capture(&LABEL, html, 1).unwrap_or_default())?,
            },
        );
    }
    Ok(extras.into())
}

// `Number(value)`, with `Number(undefined)` as NaN.
fn number_of(value: Option<String>) -> Value {
    value.map_or(Value::Number(f64::NAN), |text| number(&text))
}

fn entry<'a>(index: &'a Value, key: &str) -> Result<&'a Value> {
    let found = index.get(key);
    if found.truthy() { Ok(found) } else { Err(Error::internal(format!("Missing recording: {key}"))) }
}

fn body(root: &Path, index: &Value, key: &str) -> Result<Vec<u8>> {
    let Value::String(file) = entry(index, key)?.get("file") else {
        return Err(Error::invalid("The \"path\" argument must be of type string"));
    };
    read(&join(root, &format!("replay/bodies/{file}")))
}

fn json_body(root: &Path, index: &Value, key: &str) -> Result<Value> {
    Ok(json::parse(&String::from_utf8_lossy(&body(root, index, key)?))?)
}

fn stream(root: &Path, index: &Value, key: &str) -> Result<Vec<Value>> {
    Ok(decode_multi(&body(root, index, key)?, DecodeOptions::default())?)
}

fn recording_of(value: &Value) -> Result<Recording> {
    let field = |name: &str| -> Result<String> {
        match prop(value, name)? {
            Value::String(text) => Ok(text.clone()),
            _ => Err(Error::invalid(format!("Recorded card has no {name} text"))),
        }
    };
    Ok(Recording { html: field("html")?, json: field("json")? })
}

fn recordings_of(values: &[Value]) -> Result<Vec<Recording>> {
    values.iter().map(recording_of).collect()
}

/// Read the raw recordings the seed is built from (`readRecordings`).
///
/// # Arguments
/// * `root` - Project root containing `capture/`, `replay/` and `mirror/`.
///
/// # Returns
/// The capture's card list, the grid and trash streams, the spaces, the manual tags and the shell config.
///
/// # Errors
/// A recording that is missing or cannot be decoded.
fn read_recordings(root: &Path) -> Result<Recordings> {
    let vendor = join(root, "mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js");
    if !vendor.exists() {
        return Err(Error::internal(format!("Cannot find module '{}'", vendor.display())));
    }
    let index = json::parse(&read_text(&join(root, "replay/index.json"))?)?;
    let stream_of = |key: &str| stream(root, &index, key);
    let grid_key = format!("GET {API}/cards");
    let trash_key = format!("GET {API}/cards/trash");
    let mut spaces_ok = Ok(());
    for key in index.as_object().map(|object| object.keys().map(|key| key.into_owned()).collect::<Vec<_>>()).unwrap_or_default() {
        if SPACE_DETAIL.is_match(&key) {
            spaces_ok = spaces_ok.and_then(|()| json_body(root, &index, &key).map(drop));
        }
    }
    spaces_ok?;
    let shell = read_text(&join(root, "capture/shell.html"))?;
    let config = CONFIG.captures(&shell).and_then(|found| found.get(1)).ok_or_else(|| Error::invalid("Cannot read properties of null (reading '1')"))?;
    let config = json::parse(config.as_str())?;
    let capture_cards = json::parse(&read_text(&join(root, "capture/cards.json"))?)?;
    let capture = capture_cards.as_array().ok_or_else(|| Error::invalid("recordings is not iterable"))?;
    let stream = stream_of(&grid_key)?;
    let trash = stream_of(&trash_key)?;
    let spaces = json_body(root, &index, &format!("GET {API}/spaces"))?;
    let manual = stream_of(&format!("GET {API}/tags/manual"))?.into_iter().next().unwrap_or(Value::Undefined);
    json_body(root, &index, &format!("GET {API}/tags/recent?take=10"))?;
    entry(&index, &grid_key)?;
    Ok(Recordings { capture: recordings_of(capture)?, stream: recordings_of(&stream)?, trash: recordings_of(&trash)?, spaces, manual, config })
}

/// Read a recorded card's id from its markup (`recordedCardId`).
///
/// # Arguments
/// * `recording` - A `{html, json}` card recording.
///
/// # Returns
/// The `data-id` of the card's root element.
///
/// # Errors
/// The markup has no card id.
fn recorded_card_id(recording: &Recording) -> Result<String> {
    capture(&CARD_ID, &recording.html, 1).ok_or_else(|| Error::internal("Recorded card has no data-id"))
}

struct MergeStats {
    capture: i64,
    added: i64,
    replaced: i64,
    signed_refresh: i64,
}

// Overlay the newer replay card bodies on the capture's card list, keyed by card id.
fn merge_cards(recordings: &Recordings) -> Result<(IndexMap<String, &Recording>, MergeStats)> {
    let mut by_id: IndexMap<String, &Recording> = IndexMap::new();
    for recording in &recordings.capture {
        by_id.insert(recorded_card_id(recording)?, recording);
    }
    let mut stats = MergeStats { capture: by_id.len() as i64, added: 0, replaced: 0, signed_refresh: 0 };
    for recording in &recordings.stream {
        let id = recorded_card_id(recording)?;
        match by_id.get(&id) {
            None => stats.added += 1,
            Some(prior) => {
                if ns(json::parse(&recording.json)?.get("modified"))? < ns(json::parse(&prior.json)?.get("modified"))? {
                    continue;
                } else if recording.json != prior.json {
                    stats.replaced += 1;
                } else if recording.html != prior.html {
                    stats.signed_refresh += 1;
                }
            }
        }
        by_id.insert(id, recording);
    }
    Ok((by_id, stats))
}

// Recorded `bumped` has second precision but a fixed order, so encode that order as sub-second offsets within each second.
fn bumped_ns_by_order(ordered: &[String], by_id: &IndexMap<String, &Recording>) -> Result<(HashMap<String, i128>, i64)> {
    let mut seconds = Vec::new();
    for id in ordered {
        let recording = by_id.get(id).ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'json')"))?;
        seconds.push(ns(json::parse(&recording.json)?.get("bumped"))?);
    }
    let mut sizes: HashMap<i128, i128> = HashMap::new();
    for second in &seconds {
        *sizes.entry(*second).or_default() += 1;
    }
    let mut seen: HashMap<i128, i128> = HashMap::new();
    let mut result = HashMap::new();
    let mut inversions = 0;
    for (index, id) in ordered.iter().enumerate() {
        let taken = seen.entry(seconds[index]).or_default();
        let k = *taken;
        *taken += 1;
        result.insert(id.clone(), seconds[index] + (sizes[&seconds[index]] - 1 - k) * TIE_STEP_NS);
        if index > 0 && seconds[index] > seconds[index - 1] {
            inversions += 1;
        }
    }
    Ok((result, inversions))
}

struct User {
    id: Value,
    account_uid: Value,
    id_prefix: String,
    flags: Value,
    settings: String,
}

// Build the single user row from the recorded shell config.
fn user_record(config: &Value, id_prefix: &str) -> Result<User> {
    let settings = obj! {
        "colorScheme" => config.get("colorScheme").clone(),
        "timeZoneId" => config.get("timeZoneId").clone(),
        "locale" => config.get("locale").clone(),
        "currency" => config.get("currency").clone(),
        "aiLevel" => config.get("aiLevel").clone(),
        "onboardFlags" => config.get("onboardFlags").clone(),
        "objectLimit" => prop(prop(prop(config, "quotas")?, "objects")?, "limit")?.clone(),
    };
    Ok(User {
        id: prop(prop(config, "user")?, "uid")?.clone(),
        account_uid: prop(prop(config, "account")?, "uid")?.clone(),
        id_prefix: id_prefix.to_owned(),
        flags: prop(config, "user")?.get("flags").clone(),
        settings: json::to_string(&settings.into()),
    })
}

struct Previous {
    vector: Vec<u8>,
    text: String,
}

// Read an existing database's embeddings with the text each was computed from, so a reseed need not pay for them again.
fn previous_embeddings(path: &Path) -> Result<HashMap<String, Previous>> {
    let mut previous = HashMap::new();
    if !path.exists() {
        return Ok(previous);
    }
    let db = Sqlite::open(path, true)?;
    if db.one("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'embeddings'", &[])?.is_none() {
        return Ok(previous);
    }
    for row in db.query(
        "SELECT e.card_id, e.vector, c.type, c.data, c.extras, b.body FROM embeddings e
      JOIN cards c ON c.id = e.card_id LEFT JOIN card_content b ON b.card_id = e.card_id",
        &[],
    )? {
        let item = embedding_item(row.text("data").unwrap_or("null"), row.text("extras").unwrap_or("null"), row.text("body"))?;
        let vector = row.blob("vector").map(<[u8]>::to_vec).ok_or_else(|| Error::internal("vector is not a blob"))?;
        previous.insert(row.text("card_id").unwrap_or_default().to_owned(), Previous { vector, text: mm_core::ai::document_text(&item) });
    }
    Ok(previous)
}

// Keep each previous vector whose card still has exactly the text it was computed from.
fn restore_embeddings(store: &Store, previous: &HashMap<String, Previous>) -> Result<i64> {
    let db = store.db();
    store.transaction(|| {
        let mut kept = 0;
        for card in db.query("SELECT id, user_id, type, data, extras FROM cards", &[])? {
            let id = card.text("id").unwrap_or_default();
            let Some(old) = previous.get(id) else { continue };
            let body = db.one("SELECT body FROM card_content WHERE card_id = ?", params![id])?;
            let item = embedding_item(card.text("data").unwrap_or("null"), card.text("extras").unwrap_or("null"), body.as_ref().and_then(|row| row.text("body")))?;
            if mm_core::ai::document_text(&item) != old.text {
                continue;
            }
            db.execute("INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, ?, 768, ?)", &[id.into(), card.get("user_id").clone(), old.vector.clone().into()])?;
            kept += 1;
        }
        Ok(kept)
    })
}

struct Counts {
    cards: i64,
    trash: i64,
    annotations: i64,
    spaces: i64,
    space_cards: i64,
    tag_names: i64,
    changes: i64,
}

struct Pending<'a> {
    id: String,
    recording: &'a Recording,
    deleted: Option<i128>,
}

/// Rebuild the SQLite database from the recordings, replacing any existing file (`seedDatabase`).
///
/// # Arguments
/// * `db_path` - Database file path; the file and its WAL sidecars are deleted first, keeping only the embeddings
///   of cards whose text is unchanged.
/// * `root` - Project root containing the recordings.
///
/// # Returns
/// The summary lines in print order: what was written, then `tieInversions`, `extrasFailures` and
/// `leftoverCaptureCards`, which are 0 for a clean seed.
///
/// # Errors
/// A missing or malformed recording, or the database's error.
pub async fn seed_database(db_path: &Path, root: &Path) -> Result<Vec<(String, i64)>> {
    let embeddings = previous_embeddings(db_path)?;
    for suffix in ["", "-wal", "-shm"] {
        let mut file = db_path.as_os_str().to_owned();
        file.push(suffix);
        let _ = std::fs::remove_file(file);
    }
    let recordings = read_recordings(root)?;
    let (by_id, stats) = merge_cards(&recordings)?;

    let trash_ids: Vec<String> = recordings.trash.iter().map(recorded_card_id).collect::<Result<_>>()?;
    let trash_set: HashSet<&String> = trash_ids.iter().collect();
    let mut trash_by_id: HashMap<&String, &Recording> = HashMap::new();
    for (id, recording) in trash_ids.iter().zip(&recordings.trash) {
        trash_by_id.insert(id, recording);
    }
    let grid_ids: Vec<String> = recordings.stream.iter().map(recorded_card_id).collect::<Result<Vec<_>>>()?.into_iter().filter(|id| !trash_set.contains(id)).collect();
    let in_grid: HashSet<&String> = grid_ids.iter().collect();
    let leftover: Vec<String> = by_id.keys().filter(|id| !in_grid.contains(id) && !trash_set.contains(id)).cloned().collect();
    let ordered: Vec<String> = grid_ids.iter().chain(&leftover).cloned().collect();
    let (bumped_ns, inversions) = bumped_ns_by_order(&ordered, &by_id)?;

    let mut all_cards: Vec<Pending> = Vec::new();
    for id in &ordered {
        all_cards.push(Pending { id: id.clone(), recording: by_id[id], deleted: None });
    }
    let mut trash_unique: Vec<&String> = Vec::new();
    for id in &trash_ids {
        if !trash_unique.contains(&id) {
            trash_unique.push(id);
        }
    }
    for id in trash_unique {
        let recording = trash_by_id[id];
        all_cards.push(Pending { id: id.clone(), recording, deleted: Some(ns(json::parse(&recording.json)?.get("deleted"))?) });
    }
    let first = all_cards.first().ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'id')"))?;
    let id_prefix = text::slice(&first.id, 0, Some(11));
    let stray = all_cards.iter().filter(|card| !card.id.starts_with(&id_prefix)).count();
    if stray > 0 {
        return Err(Error::internal(format!("{stray} card ids do not share the prefix {id_prefix}")));
    }

    let db: std::rc::Rc<dyn Sql> = std::rc::Rc::new(Sqlite::open(db_path, false)?);
    let store = Store::new(db.clone())?;
    let mut counts = Counts { cards: 0, trash: 0, annotations: 0, spaces: 0, space_cards: 0, tag_names: 0, changes: 0 };
    let mut extras_failures = 0;
    store.transaction(|| {
        let user = user_record(&recordings.config, &id_prefix)?;
        db.execute(
            "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES (?, ?, ?, ?, ?)",
            &[bind(&user.id)?, bind(&user.account_uid)?, user.id_prefix.as_str().into(), bind(&user.flags)?, user.settings.as_str().into()],
        )?;
        let user_id = bind(&user.id)?;
        let mut changes: Vec<(i128, String, f64)> = Vec::new();
        for Pending { id, recording, deleted } in &all_cards {
            let data = json::parse(&recording.json)?;
            let extras = match recording_extras(&recording.html, &recording.json) {
                Ok(extras) => extras,
                Err(_) => {
                    extras_failures += 1;
                    obj! { "id" => id.as_str() }.into()
                }
            };
            let modified = ns(prop(&data, "modified")?)?;
            let bumped = match deleted {
                None => bumped_ns[id],
                Some(_) => ns(data.get("bumped"))?,
            };
            db.execute(
                "INSERT INTO cards
      (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns, deleted_ns)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    id.as_str().into(),
                    user_id.clone(),
                    bind(data.get("type"))?,
                    bind(nullish_or(data.get("flags"), &Value::Number(0.0)))?,
                    recording.json.as_str().into(),
                    recording.html.as_str().into(),
                    json::to_string(&extras).into(),
                    SqlValue::big(ns(data.get("created"))?),
                    SqlValue::big(bumped),
                    SqlValue::big(modified),
                    deleted.map_or(SqlValue::Null, SqlValue::big),
                ],
            )?;
            let no_tags = Value::Array(Vec::new());
            let tags = nullish_or(data.get("tags"), &no_tags);
            for (position, tag) in tags.as_array().ok_or_else(|| Error::invalid("(data.tags ?? []).forEach is not a function"))?.iter().enumerate() {
                db.execute(
                    "INSERT INTO annotations (id, user_id, card_id, name, flags, position) VALUES (?, ?, ?, ?, ?, ?)",
                    &[bind(prop(tag, "id")?)?, user_id.clone(), id.as_str().into(), bind(prop(tag, "name")?)?, bind(nullish_or(tag.get("flags"), &Value::Number(0.0)))?, position.into()],
                )?;
                counts.annotations += 1;
            }
            changes.push((modified, id.clone(), if deleted.is_none() { 1.0 } else { 2.0 }));
            if deleted.is_none() {
                counts.cards += 1;
            } else {
                counts.trash += 1;
            }
        }
        changes.sort_by_key(|change| change.0);
        for (ts, id, kind) in &changes {
            db.execute("INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, 9, ?, ?)", &[user_id.clone(), SqlValue::big(*ts), id.as_str().into(), (*kind).into()])?;
            counts.changes += 1;
        }

        let manual = recordings.manual.as_array().ok_or_else(|| Error::invalid("recordings.manual.forEach is not a function"))?;
        for (rank, tag) in manual.iter().enumerate() {
            let Value::Date(modified) = prop(tag, "modified")? else { return Err(Error::invalid("tag.modified.getTime is not a function")) };
            db.execute("INSERT INTO tag_names (user_id, name, modified_ms, manual_rank) VALUES (?, ?, ?, ?)", &[user_id.clone(), bind(tag.get("name"))?, (*modified).into(), rank.into()])?;
            counts.tag_names += 1;
        }

        let spaces = recordings.spaces.as_array().ok_or_else(|| Error::invalid("recordings.spaces is not iterable"))?;
        for space in spaces {
            let query = space.get("query");
            let query_text = if query.truthy() { SqlValue::Text(json::to_string(query)) } else { SqlValue::Null };
            db.execute(
                "INSERT INTO spaces (id, user_id, name, color, position, created, query) VALUES (?, ?, ?, ?, ?, ?, ?)",
                &[bind(space.get("id"))?, user_id.clone(), bind(space.get("name"))?, bind(space.get("color"))?, bind(space.get("position"))?, bind(space.get("created"))?, query_text],
            )?;
            let members = space.get("objects").as_array().ok_or_else(|| Error::invalid("space.objects.forEach is not a function"))?;
            for (rank, member) in members.iter().enumerate() {
                db.execute(
                    "INSERT INTO space_cards (space_id, card_id, user_id, rank, score, query_id, excluded)
      VALUES (?, ?, ?, ?, ?, ?, 0)",
                    &[bind(space.get("id"))?, bind(prop(member, "id")?)?, user_id.clone(), rank.into(), bind(member.get("score"))?, bind(nullish_or(member.get("queryId"), &Value::Null))?],
                )?;
                counts.space_cards += 1;
            }
            counts.spaces += 1;
        }
        Ok(())
    })?;
    import_read_metadata(&store, root).await?;
    let kept = restore_embeddings(&store, &embeddings)?;
    drop(store);
    Ok(vec![
        ("cards".to_owned(), counts.cards),
        ("trash".to_owned(), counts.trash),
        ("annotations".to_owned(), counts.annotations),
        ("spaces".to_owned(), counts.spaces),
        ("spaceCards".to_owned(), counts.space_cards),
        ("tagNames".to_owned(), counts.tag_names),
        ("changes".to_owned(), counts.changes),
        ("embeddingsKept".to_owned(), kept),
        ("merge_capture".to_owned(), stats.capture),
        ("merge_added".to_owned(), stats.added),
        ("merge_replaced".to_owned(), stats.replaced),
        ("merge_signedRefresh".to_owned(), stats.signed_refresh),
        ("tieInversions".to_owned(), inversions),
        ("extrasFailures".to_owned(), extras_failures),
        ("leftoverCaptureCards".to_owned(), leftover.len() as i64),
    ])
}

// Resolve the database path from the command-line flags.
fn parse_args(root: &Path, args: &[String]) -> Result<PathBuf> {
    let mut db = join(root, "data/mymind.db");
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--db" && args.get(index + 1).is_some_and(|next| !next.is_empty()) {
            index += 1;
            db = resolve(&cwd, &args[index]);
        } else if let Some(path) = args[index].strip_prefix("--db=") {
            db = resolve(&cwd, path);
        } else {
            return Err(Error::internal(USAGE));
        }
        index += 1;
    }
    Ok(db)
}

/// Run `mymind seed-db [--db path | --db=path]` (the main block of `tools/seed-db.mjs`).
///
/// # Arguments
/// * `root` - Project root, the JS `ROOT`; the default database is `data/mymind.db` below it.
/// * `args` - The arguments after the subcommand name.
///
/// # Returns
/// The exit status: 0, or 1 after the error message was printed to standard error.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    let result = async {
        let path = parse_args(root, args)?;
        let summary = seed_database(&path, root).await?;
        println!("Seeded {}", path.display());
        for (key, value) in summary {
            println!("  {} {value}", text::pad_end(&key, 22, " "));
        }
        Ok::<(), Error>(())
    }
    .await;
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", message(&error));
            1
        }
    }
}
