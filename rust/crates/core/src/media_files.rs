//! Port of `lib/media-files.mjs`: media content types, covers, rendition grammar and byte ranges.

use std::sync::LazyLock;

use mm_js::{Value, number::{is_safe_integer, number_to_string, string_to_number}, obj};
use regex::Regex;

use crate::render::access::{member, min};
use crate::{Error, Result};

/// Content types of the stored media formats, keyed by extension in `MIME` order.
pub const MIME: [(&str, &str); 11] = [
    ("png", "image/png"), ("jpg", "image/jpeg"), ("jpeg", "image/jpeg"), ("webp", "image/webp"), ("gif", "image/gif"),
    ("avif", "image/avif"), ("svg", "image/svg+xml"), ("mp4", "video/mp4"), ("webm", "video/webm"),
    ("mov", "video/quicktime"), ("pdf", "application/pdf"),
];

/// The content type of a file extension, or `None` for an unknown one.
pub fn mime(extension: &str) -> Option<&'static str> {
    MIME.iter().find(|(key, _)| *key == extension).map(|(_, value)| *value)
}

fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}

/// Fit intrinsic dimensions to the cover bounding box.
///
/// # Arguments
/// * `blob` - Stored original's metadata: `path`, `width`, `height` and optional `dominantColor`.
/// * `max_width` - Box width (JS default 800).
/// * `max_height` - Box height (JS default 1000).
///
/// # Returns
/// `{src, width, height, dominantColor}` pointing at a WebP rendition; `dominantColor` is always present.
///
/// # Errors
/// `blob` is null or undefined.
pub fn cover(blob: &Value, max_width: f64, max_height: f64) -> Result<Value> {
    let (width, height) = (member(blob, "width")?.to_number(), blob.get("height").to_number());
    let scale = min(min(1.0, max_width / width), max_height / height);
    let width = max(1.0, (width * scale).floor());
    let height = max(1.0, (height * scale).floor());
    let src = format!("/media/{};{}x{}-t.webp", blob.get("path").to_js_string(), number_to_string(width), number_to_string(height));
    Ok(obj! { "src" => src, "width" => width, "height" => height, "dominantColor" => blob.get("dominantColor").clone() }.into())
}

/// The original or rendition a media path asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Rendition {
    /// The stored original.
    Original { source: String },
    /// A derived image or video.
    Derived {
        source: String,
        suffix: String,
        width: u32,
        height: u32,
        /// Fit mode `t` or `c`, when given.
        mode: Option<char>,
        format: String,
        page: u32,
        /// A white background was requested.
        flatten: bool,
        content_type: String,
    },
}

impl Rendition {
    /// The `parseRendition` result object.
    pub fn to_value(&self) -> Value {
        match self {
            Rendition::Original { source } => obj! { "source" => source.as_str(), "suffix" => Value::Null }.into(),
            Rendition::Derived { source, suffix, width, height, mode, format, page, flatten, content_type } => {
                let mut object = obj! { "source" => source.as_str(), "suffix" => suffix.as_str(), "width" => *width, "height" => *height };
                object.insert("mode", mode.map_or(Value::Undefined, |mode| Value::from(mode.to_string())));
                object.insert("format", format.as_str());
                object.insert("page", *page);
                object.insert("flatten", *flatten);
                object.insert("type", content_type.as_str());
                object.into()
            }
        }
    }
}

static ORIGINAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^original\.[a-z0-9]+$").expect("regex"));
static SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|/)([0-9]+)x([0-9]+)(?:-([tc]))?\.(webp|jpe?g|png|mp4)(?:/hevc\.mp4)?$").expect("regex"));
static VIDEO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|/)([0-9]+)x([0-9]+)/hevc\.mp4$").expect("regex"));
static PAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"page\(([0-9]+)\)").expect("regex"));
static RANGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^bytes=([0-9]*)-([0-9]*)$").expect("regex"));

/// Split a media path into its original and the rendition it asks for, using mymind's rendition grammar
/// (`path;WxH[-t|-c].ext`, `path;WxH/hevc.mp4`, optional `background(fff)` and `page(n)`; `path;original.ext` is the
/// original itself).
///
/// # Arguments
/// * `relative` - Path below `/media/`, already decoded.
///
/// # Returns
/// The original's path, with the rendition's size, fit mode, format, PDF page, background and content type when one
/// is asked for.
///
/// # Errors
/// Status 400 "Invalid media rendition" when the suffix is malformed or out of bounds.
pub fn parse_rendition(relative: &str) -> Result<Rendition> {
    let mut parts = relative.split(';');
    let source = parts.next().unwrap_or("").to_string();
    let suffixes: Vec<&str> = parts.collect();
    if suffixes.is_empty() {
        return Ok(Rendition::Original { source });
    }
    let suffix = suffixes.join(";");
    if ORIGINAL.is_match(&suffix) {
        return Ok(Rendition::Original { source });
    }
    let size = SIZE.captures(&suffix);
    let video = VIDEO.captures(&suffix);
    let page = PAGE.captures(&suffix).map_or(1.0, |found| string_to_number(&found[1]));
    let Some(chosen) = size.as_ref().or(video.as_ref()) else { return Err(invalid()) };
    let (width, height) = (string_to_number(&chosen[1]), string_to_number(&chosen[2]));
    if width < 1.0 || height < 1.0 || width > 4096.0 || height > 4096.0 || page < 1.0 || page > 1000.0 {
        return Err(invalid());
    }
    let format = if video.is_some() { "mp4".to_string() } else { size.as_ref().expect("size")[4].to_string() };
    let content_type = if format == "mp4" { "video/mp4".to_string() } else { mime(&format).unwrap_or_default().to_string() };
    Ok(Rendition::Derived {
        source,
        flatten: suffix.contains("background(fff)"),
        mode: size.as_ref().and_then(|found| found.get(3)).and_then(|found| found.as_str().chars().next()),
        suffix,
        width: width as u32,
        height: height as u32,
        format,
        page: page as u32,
        content_type,
    })
}

fn invalid() -> Error {
    Error::status(400, "Invalid media rendition")
}

/// The status and inclusive byte span of a `Range` request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ByteRange {
    pub status: u16,
    pub start: f64,
    pub end: f64,
}

impl ByteRange {
    /// The `byteRange` result object.
    pub fn to_value(&self) -> Value {
        obj! { "status" => self.status, "start" => self.start, "end" => self.end }.into()
    }
}

/// Resolve a `Range` header against a body size, as the media routes answer it.
///
/// # Arguments
/// * `range` - The request's Range header, or `None` for the whole body.
/// * `size` - Body size in bytes.
///
/// # Returns
/// The status to send (200, 206 or 416) and the inclusive byte span; a 416 carries `0` and `-1`.
pub fn byte_range(range: Option<&str>, size: f64) -> ByteRange {
    let Some(range) = range else { return ByteRange { status: 200, start: 0.0, end: size - 1.0 } };
    let found = RANGE.captures(range);
    let number = |index: usize| {
        found.as_ref().and_then(|found| found.get(index)).filter(|found| !found.as_str().is_empty()).map(|found| string_to_number(found.as_str()))
    };
    let (first, last) = (number(1), number(2));
    let invalid = found.is_none() || (first.is_none() && last.is_none()) || [first, last].iter().flatten().any(|value| !is_safe_integer(*value));
    let start = first.unwrap_or_else(|| max(0.0, size - last.unwrap_or(0.0)));
    let end = match (first, last) {
        (Some(_), Some(last)) => min(last, size - 1.0),
        _ => size - 1.0,
    };
    if invalid || start > end || start >= size || (first.is_none() && last == Some(0.0)) {
        return ByteRange { status: 416, start: 0.0, end: -1.0 };
    }
    ByteRange { status: 206, start, end }
}
