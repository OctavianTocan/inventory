//! Read committed recordings and recover only the source metadata their card JSON omitted.
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use mm_core::render::encoding::unescape_html;
use mm_core::sql::{Sql, SqlValue};
use mm_core::store::{Store, ns_from_iso};
use mm_core::{Error, Result};
use mm_js::number::string_to_number;
use mm_core::render_card::render_card;
use mm_js::{Object, Value, arr, json, obj, text};
use mm_sqlite::Sqlite;
use regex::Regex;

// Reject a property read that JavaScript would throw for a nullish receiver.
fn prop<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    if value.is_nullish() {
        return Err(Error::invalid(format!(
            "Cannot read properties of {} (reading '{key}')",
            if value.is_undefined() {
                "undefined"
            } else {
                "null"
            }
        )));
    }
    Ok(value.get(key))
}

const SPACE: &str = r"[\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";

static CARD_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"^<mymind-card(?-u:\b)[^>]*{SPACE}data-id="([^"]+)""#
    ))
    .unwrap()
});
static CONFIG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<script id="config" type="application/json">([\s\S]*?)</script>"#).unwrap()
});
static CARD_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<mymind-card(?-u:\b)[^>]*>").unwrap());
static LINK_BUBBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a class="link-bubble"[^>]*>"#).unwrap());
static H3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<h3>([\s\S]*?)</h3>").unwrap());
static IMAGE_BOX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<mymind-image(?-u:\b)([^>]*)>([\s\S]*?)</mymind-image>").unwrap()
});
static RATIO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"aspect-ratio:([0-9]+)/([0-9]+)").unwrap());
static MEDIA_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(?:img|video)(?-u:\b)[^>]*>").unwrap());
static RENDITION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";([0-9]+)x([0-9]+)").unwrap());
static BACKGROUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"background-color:([^;]+)").unwrap());
static POST_BODY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<i class="platform"></i>\n\n    ([\s\S]*?)\n\n    "#).unwrap());
static AUTHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<p class="author">([\s\S]*?)</p>"#).unwrap());
static GALLERY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"<div class="gallery">([\s\S]*?)</div>{SPACE}*</div>"#
    ))
    .unwrap()
});
static GALLERY_IMAGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<img class="lazy" data-(srcset|src)="([^"]*)" width="([0-9]+)" height="([0-9]+)"/>"#,
    )
    .unwrap()
});
static MAIN_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img class="lazy main-img"[^>]*>"#).unwrap());
static APP_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img srcset="[^"]*"[^>]*>"#).unwrap());
static DISTRIBUTOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"class="inner distributor_([^"]*)""#).unwrap());
static PRICE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<span class="price">([^<]*)</span>"#).unwrap());
static LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<div class="label">([\s\S]*?)</div>"#).unwrap());
// Identify ECMAScript whitespace in an attribute boundary.
fn is_js_space(character: char) -> bool {
    matches!(
        character,
        '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

// Read a quoted attribute without interpreting or executing the recorded markup.
fn attr(tag: Option<&str>, name: &str) -> Result<Option<String>> {
    let Some(tag) = tag else { return Ok(None) };
    let needle = format!("{name}=\"");
    let mut starts = vec![0];
    starts.extend(
        tag.char_indices()
            .filter(|(_, character)| is_js_space(*character))
            .map(|(at, character)| at + character.len_utf8()),
    );
    for start in starts {
        let Some(rest) = tag[start..].strip_prefix(&needle) else {
            continue;
        };
        if let Some(end) = rest.find('"') {
            return unescape_html(&rest[..end]).map(Some);
        }
    }
    Ok(None)
}

// Return an owned capture group without retaining the recorded markup.
fn capture(pattern: &Regex, haystack: &str, group: usize) -> Option<String> {
    pattern
        .captures(haystack)
        .and_then(|found| found.get(group))
        .map(|found| found.as_str().to_owned())
}

// Convert a recorded dimension with JavaScript number semantics.
fn number(digits: &str) -> Value {
    Value::Number(string_to_number(digits))
}

// Keep absent attributes undefined until JSON serialization.
fn optional(value: Option<String>) -> Value {
    value.map_or(Value::Undefined, Value::from)
}

// Recover media metadata that the card JSON omits, without passing layout markup through.
fn cover_from_html(html: &str) -> Result<Value> {
    let Some(found) = IMAGE_BOX.captures(html) else {
        return Ok(Value::Undefined);
    };
    let (head, inner) = (
        found.get(1).map_or("", |m| m.as_str()),
        found.get(2).map_or("", |m| m.as_str()),
    );
    let style = attr(Some(head), "style")?;
    let ratio = style.as_deref().and_then(|style| RATIO.captures(style));
    let tag = MEDIA_TAG.find(inner).map(|m| m.as_str());
    let (Some(ratio), Some(tag)) = (ratio, tag) else {
        return Ok(Value::Undefined);
    };
    let src = attr(Some(tag), "data-src")?;
    let rendition = src.as_deref().and_then(|src| RENDITION.captures(src));
    let (width, height) = match &rendition {
        Some(found) => (number(&found[1]), number(&found[2])),
        None => (number(&ratio[1]), number(&ratio[2])),
    };
    let size = |name: &str| -> Result<Value> {
        Ok(attr(Some(tag), name)?.map_or(Value::Undefined, |text| number(&text)))
    };
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
    extras.insert(
        "id",
        optional(attr(CARD_TAG.find(html).map(|m| m.as_str()), "data-id")?),
    );
    let link = attr(LINK_BUBBLE.find(html).map(|m| m.as_str()), "href")?;
    if let Some(link) = link {
        let source = data.get("source");
        let source_url = if source.is_nullish() {
            &Value::Undefined
        } else {
            source.get("url")
        };
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
            let Value::String(clip) = title else {
                return Err(Error::invalid("data.title.startsWith is not a function"));
            };
            let full = unescape_html(&capture(&H3, html, 1).unwrap_or_default())?;
            if full.starts_with(clip.as_str()) && full != *clip {
                extras.insert("title", full);
            }
        }
    }
    if !is(&[
        "Image",
        "Video",
        "Document",
        "Note",
        "Content",
        "Repository",
    ]) {
        extras.insert("cover", cover_from_html(html)?);
    }
    if is(&["Image"]) {
        let cover = cover_from_html(html)?;
        if cover.is_undefined() {
            return Err(Error::invalid(
                "Cannot read properties of undefined (reading 'dominantColor')",
            ));
        }
        let blob = prop(&data, "blob")?;
        let color = prop(blob, "dominantColor")?;
        let same = match (cover.get("dominantColor"), color) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        };
        let differs = !same;
        if differs
            || cover
                .get("src")
                .as_str()
                .is_some_and(|src| src.ends_with("-t.webp"))
        {
            extras.insert("cover", cover);
        }
    }
    if is(&["XPost", "ThreadsPost", "SubstackNote"]) {
        let gallery_html = GALLERY
            .captures(html)
            .and_then(|found| found.get(1))
            .map_or("", |m| m.as_str());
        let mut gallery = Vec::new();
        for found in GALLERY_IMAGE.captures_iter(gallery_html) {
            let mut item = Object::new();
            item.insert(found[1].to_owned(), unescape_html(&found[2])?);
            item.insert("width", number(&found[3]));
            item.insert("height", number(&found[4]));
            gallery.push(Value::from(item));
        }
        let has_video = html.contains("class=\"fake-player\"")
            || extras
                .get("cover")
                .is_some_and(|cover| cover.get("type").as_str() == Some("video"));
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
        extras.insert(
            "product",
            obj! { "price" => unescape_html(&capture(&PRICE, html, 1).unwrap_or_default())? },
        );
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

/// A committed card recording with unchanged markup and JSON bytes.
#[derive(Clone)]
pub struct Recording {
    /// Markup captured from the source response.
    pub html: String,
    /// JSON text captured from the source response.
    pub json: String,
}

impl Recording {
    /// Read a response value without rendering or normalizing its payload.
    ///
    /// # Arguments
    /// * `value` - The decoded `{html,json}` response object.
    ///
    /// # Returns
    /// The original two recorded strings, without parsing or rendering them.
    ///
    /// # Panics
    /// When either recorded field is missing or is not a string.
    pub fn from_value(value: &Value) -> Self {
        Self {
            html: value.get("html").as_str().expect("recording html").into(),
            json: value.get("json").as_str().expect("recording json").into(),
        }
    }

    /// Read the card identity from the recorded root element.
    ///
    /// # Returns
    /// The unchanged recorded identity.
    ///
    /// # Panics
    /// When the recording has no card identity.
    pub fn id(&self) -> String {
        capture(&CARD_ID, &self.html, 1).expect("recorded card id")
    }
}

/// The original store check's independent recording expectations.
pub struct Recordings {
    /// Complete synthetic card set.
    pub capture: Vec<Recording>,
    /// Newest desktop grid response.
    pub stream: Vec<Recording>,
    /// Trash response.
    pub trash: Vec<Recording>,
    /// Spaces with ranked members.
    pub spaces: Value,
    /// Individual space responses.
    pub space_details: Vec<(String, Value)>,
    /// Manual tag response, retaining MessagePack dates.
    pub manual: Value,
    /// Recent tag response.
    pub recent: Value,
    /// Grid response headers.
    pub grid_headers: Value,
    /// Only the non-secret shell configuration fields the assertions read.
    pub config: Value,
}

const TYPES: &[&str] = &[
    "WebPage",
    "XPost",
    "Article",
    "YouTubeVideo",
    "Repository",
    "Video",
    "Note",
    "Image",
    "Screenshot",
    "InstagramReel",
    "SoftwareApplication",
    "Product",
    "Document",
    "ThreadsPost",
    "Placeholder",
    "SubstackNote",
    "Content",
    "RedditPost",
];

fn synthetic_recording(index: usize, kind: &str, deleted: Option<&str>) -> Recording {
    let id = format!("00000002UnE0O{index:09}");
    let iso = format!("2026-10-16T00:00:{index:02}.000Z");
    let tag_name = ["design", "architecture", "rust"][(index - 1) % 3];
    let tag_id = format!("00000002UnE0y{index:09}");
    let mut data = obj! {
        "type" => kind,
        "flags" => 0.0,
        "created" => iso.as_str(),
        "bumped" => iso.as_str(),
        "modified" => deleted.unwrap_or(iso.as_str()),
        "tags" => arr![obj! { "id" => tag_id.as_str(), "name" => tag_name, "flags" => 8.0 }],
    };
    if matches!(kind, "Image" | "Video" | "Document") {
        data.insert("blob", obj! { "path" => format!("acct/{id}/file"), "width" => 800.0, "height" => 600.0 });
    }
    if let Some(del) = deleted {
        data.insert("deleted", del);
    }
    let data_val: Value = data.into();
    let json_text = json::to_string(&data_val);
    let html = render_card(&data_val, &obj! { "id" => id.as_str() }.into()).expect("render synthetic card");
    Recording { html, json: json_text }
}

/// Build synthetic card responses covering all 18 card types.
pub fn captured_cards() -> Vec<Recording> {
    let mut cards = Vec::with_capacity(TYPES.len() * 2);
    for index in (1..=TYPES.len() * 2).rev() {
        let kind = TYPES[(index - 1) % TYPES.len()];
        cards.push(synthetic_recording(index, kind, None));
    }
    cards
}

/// Build synthetic check-store expectations covering grid, trash, spaces, tags, quota and config.
pub fn read_recordings() -> Recordings {
    let stream = captured_cards();
    let capture = stream.clone();
    let trash = vec![
        synthetic_recording(38, "Note", Some("2026-10-16T00:00:38.000Z")),
        synthetic_recording(37, "Article", Some("2026-10-16T00:00:37.000Z")),
    ];
    let space_one: Value = obj! {
        "id" => "00000002UnEgl000000001",
        "name" => "Design Systems",
        "color" => "#166ff4",
        "position" => 0.0,
        "objects" => arr![
            obj! { "id" => "00000002UnE0O000000036", "score" => 1.0 },
            obj! { "id" => "00000002UnE0O000000035", "score" => 0.9 }
        ],
        "created" => "2026-10-15T10:00:00.000Z",
    }
    .into();
    let space_two: Value = obj! {
        "id" => "00000002UnEgl000000002",
        "name" => "Smart Articles",
        "color" => "#17c37b",
        "position" => 1.0,
        "objects" => arr![
            obj! { "id" => "00000002UnE0O000000021", "score" => 1.0, "queryId" => "00000002UnEiV000000001" }
        ],
        "created" => "2026-10-15T11:00:00.000Z",
        "query" => obj! {
            "id" => "00000002UnEiV000000001",
            "filters" => arr!["type:article"],
            "checked" => "2026-10-16T00:00:00.000Z",
        },
    }
    .into();
    let spaces = arr![space_one.clone(), space_two.clone()];
    let space_details = vec![
        ("00000002UnEgl000000001".to_owned(), space_one),
        ("00000002UnEgl000000002".to_owned(), space_two),
    ];
    let manual = arr![
        obj! { "name" => "design", "count" => 12.0, "flags" => 0.0, "modified" => Value::Date(1_780_000_000_000.0) },
        obj! { "name" => "architecture", "count" => 12.0, "flags" => 0.0, "modified" => Value::Date(1_780_000_001_000.0) },
        obj! { "name" => "rust", "count" => 12.0, "flags" => 0.0, "modified" => Value::Date(1_780_000_002_000.0) }
    ];
    let recent = arr![
        obj! { "name" => "rust" },
        obj! { "name" => "architecture" },
        obj! { "name" => "design" }
    ];
    let checkpoint = ns_from_iso("2026-10-16T00:00:36.000Z").expect("checkpoint").to_string();
    let grid_headers = obj! {
        "x-checkpoint" => checkpoint.as_str(),
        "x-object-quota" => "{\"count\":36,\"limit\":50000}",
    }
    .into();
    let config = obj! {
        "user" => obj! { "uid" => "00000002UnE0z1M1aBCb", "flags" => 0.0 },
        "account" => obj! { "uid" => "00000002UnE00z1M1aBCb" },
        "colorScheme" => "dark",
        "timeZoneId" => "Europe/Madrid",
        "locale" => "en-US",
        "currency" => "USD",
        "aiLevel" => 3.0,
        "onboardFlags" => 128.0,
        "quotas" => obj! { "objects" => obj! { "limit" => 50_000.0 } },
    }
    .into();
    Recordings {
        capture,
        stream,
        trash,
        spaces,
        space_details,
        manual,
        recent,
        grid_headers,
        config,
    }
}

/// Read the synthetic three-change/five-event input extracted once from the private lifecycle recording.
///
/// # Returns
/// Minimal immutable fields used by rendering and native capture tests.
///
/// # Panics
/// On a malformed fixture or changed lifecycle counts.
pub fn link_save_recording() -> Value {
    let value =
        json::parse(include_str!("../fixtures/link_save.json")).expect("synthetic lifecycle JSON");
    assert_eq!(
        value
            .get("changes")
            .as_array()
            .expect("lifecycle changes")
            .len(),
        3,
        "complete link save changes"
    );
    assert_eq!(
        value
            .get("events")
            .as_array()
            .expect("lifecycle events")
            .len(),
        5,
        "complete link save events"
    );
    value
}

// Read an ISO timestamp with the same nanosecond helper as the recording importer.
fn ns(data: &Value, field: &str) -> i128 {
    ns_from_iso(data.get(field).as_str().expect("recorded ISO timestamp"))
        .expect("valid recorded timestamp")
}

// Bind recording scalars without converting BigInts to imprecise double values.
fn bind(value: &Value) -> SqlValue {
    match value {
        Value::String(value) => value.as_str().into(),
        Value::Number(value) => (*value).into(),
        Value::Null | Value::Undefined => SqlValue::Null,
        _ => panic!("unexpected recording scalar kind"),
    }
}

/// Construct a fresh store from committed recording inputs for independent baseline assertions.
///
/// # Arguments
/// * `path` - A nonexistent temporary database beneath `CARGO_TARGET_TMPDIR`.
/// * `rec` - The committed source recordings, independent of all frozen implementation oracle results.
///
/// # Returns
/// The database and store seeded with original order, raw card bytes, tags, spaces, user and change log.
///
/// # Panics
/// On malformed recordings, duplicate identities, or a SQL/import failure. This fixture builder never loads
/// mutable databases, metadata evidence, credentials, embedding providers or external hosts.
pub fn seed_recorded_store(path: &Path, rec: &Recordings) -> (Rc<Sqlite>, Store) {
    assert!(
        !path.exists(),
        "recording seed requires a new temporary database"
    );
    std::fs::create_dir_all(path.parent().expect("temporary database directory"))
        .expect("create seed directory");
    let db = Rc::new(Sqlite::open(path, false).expect("create recording database"));
    let store = Store::new(db.clone()).expect("initialize recording schema");
    let mut by_id = IndexMap::new();
    for recorded in &rec.capture {
        by_id.insert(recorded.id(), recorded);
    }
    for recorded in &rec.stream {
        let id = recorded.id();
        if let Some(prior) = by_id.get(&id)
            && ns(&json::parse(&recorded.json).expect("grid JSON"), "modified")
                < ns(&json::parse(&prior.json).expect("capture JSON"), "modified")
        {
            continue;
        }
        by_id.insert(id, recorded);
    }
    let trash_ids: HashSet<_> = rec.trash.iter().map(Recording::id).collect();
    let mut ordered = rec
        .stream
        .iter()
        .map(Recording::id)
        .filter(|id| !trash_ids.contains(id))
        .collect::<Vec<_>>();
    let grid_ids: HashSet<_> = ordered.iter().cloned().collect();
    ordered.extend(
        by_id
            .keys()
            .filter(|id| !grid_ids.contains(*id) && !trash_ids.contains(*id))
            .cloned(),
    );
    let seconds = ordered
        .iter()
        .map(|id| {
            ns(
                &json::parse(&by_id[id].json).expect("ordered card JSON"),
                "bumped",
            )
        })
        .collect::<Vec<_>>();
    let mut counts = HashMap::<i128, i128>::new();
    for second in &seconds {
        *counts.entry(*second).or_default() += 1;
    }
    let mut seen = HashMap::<i128, i128>::new();
    let mut pending = Vec::new();
    for (id, second) in ordered.iter().zip(seconds) {
        let position = seen.entry(second).or_default();
        let bumped = second + (counts[&second] - 1 - *position) * 1_000_000;
        *position += 1;
        pending.push((id.clone(), by_id[id], bumped, None));
    }
    for recorded in &rec.trash {
        let data = json::parse(&recorded.json).expect("trash card JSON");
        pending.push((
            recorded.id(),
            recorded,
            ns(&data, "bumped"),
            Some(ns(&data, "deleted")),
        ));
    }
    let prefix = pending
        .first()
        .expect("nonempty recording corpus")
        .0
        .chars()
        .take(11)
        .collect::<String>();
    assert!(
        pending.iter().all(|(id, ..)| id.starts_with(&prefix)),
        "recorded user prefix consistency"
    );
    let c = &rec.config;
    let user_id = c.get("user").get("uid");
    let settings: Value = obj! {
        "colorScheme" => c.get("colorScheme").clone(), "timeZoneId" => c.get("timeZoneId").clone(),
        "locale" => c.get("locale").clone(), "currency" => c.get("currency").clone(),
        "aiLevel" => c.get("aiLevel").clone(), "onboardFlags" => c.get("onboardFlags").clone(),
        "objectLimit" => c.get("quotas").get("objects").get("limit").clone(),
    }
    .into();
    store.transaction(|| {
        db.execute("INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES (?, ?, ?, ?, ?)",
            &[bind(user_id), bind(c.get("account").get("uid")), prefix.as_str().into(), bind(c.get("user").get("flags")), json::to_string(&settings).into()])?;
        let mut changes = Vec::new();
        for (id, recorded, bumped, deleted) in &pending {
            let data = json::parse(&recorded.json)?;
            let extras = recording_extras(&recorded.html, &recorded.json).unwrap_or_else(|_| obj! { "id" => id.as_str() }.into());
            let flags = if data.get("flags").is_nullish() { Value::from(0) } else { data.get("flags").clone() };
            let modified = ns(&data, "modified");
            db.execute("INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns, deleted_ns) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[id.as_str().into(), bind(user_id), bind(data.get("type")), bind(&flags), recorded.json.as_str().into(), recorded.html.as_str().into(), json::to_string(&extras).into(), SqlValue::big(ns(&data, "created")), SqlValue::big(*bumped), SqlValue::big(modified), deleted.map_or(SqlValue::Null, SqlValue::big)])?;
            if let Some(tags) = data.get("tags").as_array() {
                for (position, tag) in tags.iter().enumerate() {
                    let flags = if tag.get("flags").is_nullish() { Value::from(0) } else { tag.get("flags").clone() };
                    db.execute("INSERT INTO annotations (id, user_id, card_id, name, flags, position) VALUES (?, ?, ?, ?, ?, ?)", &[bind(tag.get("id")), bind(user_id), id.as_str().into(), bind(tag.get("name")), bind(&flags), position.into()])?;
                }
            }
            changes.push((modified, id, if deleted.is_none() { 1 } else { 2 }));
        }
        changes.sort_by_key(|change| change.0);
        for (modified, id, kind) in changes {
            db.execute("INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, 9, ?, ?)", &[bind(user_id), SqlValue::big(modified), id.as_str().into(), kind.into()])?;
        }
        for (rank, tag) in rec.manual.as_array().expect("manual tag list").iter().enumerate() {
            let Value::Date(modified) = tag.get("modified") else { panic!("recorded manual tag date") };
            db.execute("INSERT INTO tag_names (user_id, name, modified_ms, manual_rank) VALUES (?, ?, ?, ?)", &[bind(user_id), bind(tag.get("name")), (*modified).into(), rank.into()])?;
        }
        for space in rec.spaces.as_array().expect("recorded space list") {
            let query = if space.get("query").truthy() { SqlValue::Text(json::to_string(space.get("query"))) } else { SqlValue::Null };
            db.execute("INSERT INTO spaces (id, user_id, name, color, position, created, query) VALUES (?, ?, ?, ?, ?, ?, ?)", &[bind(space.get("id")), bind(user_id), bind(space.get("name")), bind(space.get("color")), bind(space.get("position")), bind(space.get("created")), query])?;
            for (rank, member) in space.get("objects").as_array().expect("space members").iter().enumerate() {
                db.execute("INSERT INTO space_cards (space_id, card_id, user_id, rank, score, query_id, excluded) VALUES (?, ?, ?, ?, ?, ?, 0)", &[bind(space.get("id")), bind(member.get("id")), bind(user_id), rank.into(), bind(member.get("score")), bind(member.get("queryId"))])?;
            }
        }
        Ok(())
    }).expect("seed complete recording transaction");
    (db, store)
}
