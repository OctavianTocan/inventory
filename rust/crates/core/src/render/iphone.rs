//! Port of `lib/render/iphone.mjs`: the grid card as mymind serves it to an iPhone.

use std::sync::LazyLock;

use mm_js::number::{number_to_string, string_to_number};
use mm_js::text::trim;
use mm_js::json;
use regex::Regex;

use super::access::member;
use super::media::{Size, fit, fit_dimensions, gcd};
use crate::{Error, Result};

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(/?)([A-Za-z0-9_-]+)([^>]*)>").expect("tag pattern"));
static CLASS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"class="([^"]*)""#).expect("class pattern"));
static IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<mymind-image(?-u:\b)([^>]*)>((?s:.*?))</mymind-image>").expect("image pattern"));
static RENDITION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r";([0-9]+)x([0-9]+)(-[tc])?\.(webp|jpe?g|png|mp4)").expect("rendition pattern"));
static RATIO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"aspect-ratio:[0-9]+/[0-9]+").expect("ratio pattern"));

/// An open element: its tag name and class attribute.
struct Element {
    name: String,
    class_name: String,
}

fn is_void(name: &str) -> bool {
    ["img", "br", "source", "input", "meta", "link", "hr", "wbr"].iter().any(|void| name.eq_ignore_ascii_case(void))
}

/// Open elements enclosing `offset`, outermost first (`ancestors`).
fn ancestors(html: &str, offset: usize) -> Vec<Element> {
    let mut stack: Vec<Element> = Vec::new();
    for tag in TAG.captures_iter(&html[..offset]) {
        let name = &tag[2];
        if !tag[1].is_empty() {
            if let Some(open) = stack.iter().rposition(|element| element.name == name) {
                stack.truncate(open);
            }
        } else if !is_void(name) && !tag[3].ends_with('/') {
            let class_name = CLASS.captures(&tag[3]).map_or_else(String::new, |class| class[1].to_owned());
            stack.push(Element { name: name.to_owned(), class_name });
        }
    }
    stack
}

/// Convert a card's grid HTML to the version mymind serves an iPhone, whose cover renditions fit a 400x500 box (400x300
/// for YouTube) instead of the desktop's (`iphoneCard`).
///
/// # Arguments
/// * `html` - Card HTML as rendered for desktop browsers.
/// * `json` - The card's data JSON; its type and video blob size decide the box and the rounding.
///
/// # Returns
/// The iPhone card HTML; post bodies, reels and other images are unchanged.
///
/// # Errors
/// `json` is not valid JSON, or parses to `null`.
pub fn iphone_card(html: &str, json: &str) -> Result<String> {
    let data = json::parse(json).map_err(Error::from)?;
    let (box_width, box_height) = if member(&data, "type")?.as_str() == Some("YouTubeVideo") { (400.0, 300.0) } else { (400.0, 500.0) };
    let blob = data.get("blob");
    let blob_sized = blob.get("width").truthy();
    let mut out = String::with_capacity(html.len());
    let mut last = 0;
    for image in IMAGE.captures_iter(html) {
        let whole = image.get(0).expect("whole match");
        out.push_str(&html[last..whole.start()]);
        last = whole.end();
        let stack = ancestors(html, whole.start());
        let nested = stack.first().is_some_and(|card| card.name == "mymind-card")
            && stack.iter().skip(1).all(|element| element.name == "div" && trim(&element.class_name) == "inner");
        if !nested {
            out.push_str(whole.as_str());
            continue;
        }
        let mut ratio: Option<String> = None;
        let mut body = String::new();
        let mut copied = 0;
        let inner = &image[2];
        for token in RENDITION.captures_iter(inner) {
            let span = token.get(0).expect("whole match");
            body.push_str(&inner[copied..span.start()]);
            copied = span.end();
            let (width, height) = (string_to_number(&token[1]), string_to_number(&token[2]));
            let extension = &token[4];
            let video = extension == "jpeg" || extension == "mp4";
            let size = if video && blob_sized {
                fit(blob, box_width, box_height, video)?
            } else {
                fit_dimensions(width, height, box_width, box_height, video)
            };
            if matches!(size, Size { width: w, height: h } if w == width && h == height) {
                body.push_str(span.as_str());
                continue;
            }
            let divisor = gcd(size.width, size.height);
            ratio = Some(format!("{}/{}", number_to_string(size.width / divisor), number_to_string(size.height / divisor)));
            body.push_str(&format!(";{}x{}{}.{extension}", number_to_string(size.width), number_to_string(size.height), token.get(3).map_or("", |mode| mode.as_str())));
        }
        body.push_str(&inner[copied..]);
        let attributes = &image[1];
        let head = match &ratio {
            Some(ratio) => RATIO.replacen(attributes, 1, |_: &regex::Captures| format!("aspect-ratio:{ratio}")).into_owned(),
            None => attributes.to_owned(),
        };
        out.push_str(&format!("<mymind-image{head}>{body}</mymind-image>"));
    }
    out.push_str(&html[last..]);
    Ok(out)
}
