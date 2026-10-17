//! Port of `lib/capture-html.mjs`: scraped-page metadata, post sanitizing and card-type detection.
//!
//! Patterns are translated for JS semantics: `\s` is JS's whitespace set, `\b` and `\w` are ASCII, and the `i` flag
//! folds ASCII only. A backreference such as `<(script|style)...</\1>` becomes one alternative per tag name.

use std::collections::HashMap;
use std::sync::LazyLock;

use mm_js::{Object, Value, json, text, uri};
use regex::Regex;

use crate::capture_entities::decode_entities;
use crate::render::encoding::escape_html;
use crate::{Error, Result};

const JS_SPACE: &str = r"\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}";

fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|error| panic!("bad pattern {pattern}: {error}"))
}

/// `<(a|b)\b[^>]*>[\s\S]*?<\/\1\s*>` with `\1` expanded.
fn block(names: &[&str]) -> Regex {
    let alternatives: Vec<String> = names
        .iter()
        .map(|name| format!(r"<(?i-u:{name})(?-u:\b)[^>]*>(?s:.*?)</(?i-u:{name})[{JS_SPACE}]*>"))
        .collect();
    compile(&alternatives.join("|"))
}

static SCRIPT_STYLE: LazyLock<Regex> = LazyLock::new(|| block(&["script", "style"]));
static POST_BLOCKS: LazyLock<Regex> = LazyLock::new(|| block(&["script", "style", "iframe", "object", "svg", "math", "cite", "footer"]));
static READABLE_BLOCKS: LazyLock<Regex> = LazyLock::new(|| block(&["nav", "header", "footer", "aside", "script", "style"]));
static ANY_TAG: LazyLock<Regex> = LazyLock::new(|| compile(r"<[^>]*>"));
static SPACES: LazyLock<Regex> = LazyLock::new(|| compile(&format!("[{JS_SPACE}]+")));
static COMMENT: LazyLock<Regex> = LazyLock::new(|| compile(r"<!--(?s:.*?)-->"));
static BLOCKQUOTE: LazyLock<Regex> =
    LazyLock::new(|| compile(&format!(r"<(?i-u:blockquote)(?-u:\b)[^>]*>((?s:.*?))</(?i-u:blockquote)[{JS_SPACE}]*>")));
static PARAGRAPH: LazyLock<Regex> = LazyLock::new(|| compile(&format!(r"<(?i-u:p)(?-u:\b)[^>]*>(?s:.*?)</(?i-u:p)[{JS_SPACE}]*>")));
static TOKEN: LazyLock<Regex> = LazyLock::new(|| compile(r"<[^>]*>|[^<]+|<"));
static TAG_NAME: LazyLock<Regex> = LazyLock::new(|| compile(&format!(r"^<(/?)[{JS_SPACE}]*([A-Za-z0-9_:\-]+)")));
static TAG_HEAD: LazyLock<Regex> = LazyLock::new(|| compile(r"^</?[A-Za-z0-9_:\-]+"));
static TAG_TAIL: LazyLock<Regex> = LazyLock::new(|| compile(&format!(r"/?[{JS_SPACE}]*>$")));
static ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    compile(&format!(
        r#"([^{JS_SPACE}=<>"'`]+)(?:[{JS_SPACE}]*=[{JS_SPACE}]*(?:"([^"]*)"|'([^']*)'|([^{JS_SPACE}<>`]+)))?"#
    ))
});
static META_TAG: LazyLock<Regex> = LazyLock::new(|| compile(r#"<(?i-u:meta)(?-u:\b)(?:[^>"']|"[^"]*"|'[^']*')*>"#));
static LINK_TAG: LazyLock<Regex> = LazyLock::new(|| compile(r#"<(?i-u:link)(?-u:\b)(?:[^>"']|"[^"]*"|'[^']*')*>"#));
static SCRIPT_TAG: LazyLock<Regex> =
    LazyLock::new(|| compile(&format!(r"<(?i-u:script)(?-u:\b)([^>]*)>((?s:.*?))</(?i-u:script)[{JS_SPACE}]*>")));
static ARTICLE: LazyLock<Regex> = LazyLock::new(|| compile(r"<(?i-u:article)(?-u:\b)[^>]*>((?s:.*?))</(?i-u:article)>"));
static MAIN: LazyLock<Regex> = LazyLock::new(|| compile(r"<(?i-u:main)(?-u:\b)[^>]*>((?s:.*?))</(?i-u:main)>"));
static TITLE: LazyLock<Regex> = LazyLock::new(|| compile(r"<(?i-u:title)(?-u:\b)[^>]*>((?s:.*?))</(?i-u:title)>"));
static ENTITY_TYPE: LazyLock<Regex> = LazyLock::new(|| compile(r"(?:Article|BlogPosting|Product|SoftwareApplication)$"));
static AUTHORITY: LazyLock<Regex> = LazyLock::new(|| compile(r"^(?i-u:(?:https?:)?)//([^/?#]+)"));
static PORT: LazyLock<Regex> = LazyLock::new(|| compile(r":[0-9]+$"));
static NON_ALNUM: LazyLock<Regex> = LazyLock::new(|| compile(r"[^\p{L}\p{N}]"));
static TITLE_SUFFIX: LazyLock<Regex> = LazyLock::new(|| compile(&format!(r"[{JS_SPACE}]+(?:-|\||—)[{JS_SPACE}]+([^|]+)$")));
static APP_STORE: LazyLock<Regex> = LazyLock::new(|| {
    compile(&format!(r"[{JS_SPACE}]*(?:-[{JS_SPACE}]*(?i-u:Apps on Google Play)|(?i-u:on the App Store))[{JS_SPACE}]*$"))
});
static PROFILE_PATH: LazyLock<Regex> = LazyLock::new(|| {
    compile(
        r"(?:^|[/_.\-])(?:(?i-u:profile)(?:[_\-]?(?i-u:(?:images?|pictures?|photos?|pics?|banners?)))?|(?i-u:avatars?))(?:[/_.\-]|$)",
    )
});
static PROFILE_HOST: LazyLock<Regex> = LazyLock::new(|| compile(r"^(?i-u:(?:avatars?\.|(?:www\.)?unavatar\.))"));
static GOOGLE_PATH: LazyLock<Regex> = LazyLock::new(|| compile(r"^/a(?:/|-)"));
static INSTAGRAM_PATH: LazyLock<Regex> = LazyLock::new(|| compile(r"/t51\.[0-9]+-19/"));
static STATUS_PATH: LazyLock<Regex> = LazyLock::new(|| compile(r"/status/[0-9]+"));
static REPO_PATH: LazyLock<Regex> = LazyLock::new(|| compile(r"^/[^/]+/[^/]+/?$"));
static PDF_PATH: LazyLock<Regex> = LazyLock::new(|| compile(r"(?i-u:\.pdf)$"));

/// Parsed attributes of one tag; the first occurrence of a name wins (`attributes`).
fn attributes(tag: &str) -> HashMap<String, String> {
    let head_len = TAG_HEAD.find(tag).map_or(0, |found| found.end());
    let without_head = &tag[head_len..];
    let source = match TAG_TAIL.find(without_head) {
        Some(found) => &without_head[..found.start()],
        None => without_head,
    };
    let mut result = HashMap::new();
    for captures in ATTRIBUTE.captures_iter(source) {
        let name = text::to_lower(&captures[1]);
        result.entry(name).or_insert_with(|| {
            let raw = captures.get(2).or_else(|| captures.get(3)).or_else(|| captures.get(4)).map_or("", |found| found.as_str());
            decode_entities(raw)
        });
    }
    result
}

fn attribute_value(attributes: &HashMap<String, String>, name: &str) -> Value {
    attributes.get(name).map_or(Value::Undefined, |value| Value::String(value.clone()))
}

fn numeric_host(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4 && parts.iter().all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Resolve a scraped HTTP(S) URL, excluding credentials and the live account origin (`sourceUrl`).
///
/// # Arguments
/// * `value` - Relative or absolute URL text, or a schema.org image object (`url` or `contentUrl`).
/// * `base` - Page URL used for relative resolution.
///
/// # Returns
/// The absolute URL, keeping numeric host syntax for the transport policy; `None` when it is unusable.
pub fn source_url(value: &Value, base: &str) -> Option<String> {
    let raw_value = if value.type_of() == "object" {
        let url = value.get("url");
        if url.is_nullish() { value.get("contentUrl") } else { url }
    } else {
        value
    };
    let raw = raw_value.to_js_string();
    let url = mm_js::url::parse(&raw, Some(base))?;
    if !value.truthy() || !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    if !url.username().is_empty() || url.password().is_some_and(|password| !password.is_empty()) {
        return None;
    }
    let hostname = url.host_str().unwrap_or("");
    if hostname.strip_suffix('.').unwrap_or(hostname) == "access.mymind.com" {
        return None;
    }
    let authority = AUTHORITY.captures(&raw).map(|captures| text::to_lower(&PORT.replace(&captures[1], "")));
    let lower = raw.to_ascii_lowercase();
    let starts_like_url = lower.starts_with("http:") || lower.starts_with("https:") || matches!(raw.as_bytes(), [b'\\' | b'/', b'\\' | b'/', ..]);
    if numeric_host(hostname) && starts_like_url && authority.as_deref() != Some(hostname) {
        return Some(if raw.starts_with("//") { format!("{}:{raw}", url.scheme()) } else { raw });
    }
    Some(url.as_str().to_owned())
}

/// Extract readable text from untrusted HTML without keeping executable markup (`plainText`).
///
/// # Arguments
/// * `html` - Third-party HTML fragment.
///
/// # Returns
/// Entity-decoded text with whitespace collapsed to single spaces and trimmed.
pub fn plain_text(html: &str) -> String {
    let stripped = SCRIPT_STYLE.replace_all(html, "");
    let spaced = ANY_TAG.replace_all(&stripped, " ");
    let decoded = decode_entities(&spaced);
    text::trim(&SPACES.replace_all(&decoded, " ")).to_owned()
}

/// `plainText(value)` for a value of any type: `String(value ?? '')` first.
///
/// # Arguments
/// * `value` - Any value; nullish counts as the empty string.
///
/// # Returns
/// The same text as [`plain_text`] of the value's string form.
pub fn plain_text_value(value: &Value) -> String {
    if value.is_nullish() { String::new() } else { plain_text(&value.to_js_string()) }
}

/// Rebuild source post HTML using only paragraph, break, span, strong and link elements (`sanitizePost`).
///
/// # Arguments
/// * `html` - Untrusted oEmbed or source body.
/// * `base` - Page URL for relative links.
///
/// # Returns
/// Sanitized HTML for `extras.post.bodyHtml`; links keep a safe `href` only.
pub fn sanitize_post(html: &str, base: &str) -> String {
    let mut stack: Vec<String> = Vec::new();
    let mut result = String::new();
    let without_comments = COMMENT.replace_all(html, "");
    let mut clean = POST_BLOCKS.replace_all(&without_comments, "").into_owned();
    let quote = BLOCKQUOTE.captures(&clean).map(|captures| captures[1].to_owned()).filter(|quote| !quote.is_empty());
    if let Some(quote) = quote {
        let paragraphs: Vec<&str> = PARAGRAPH.find_iter(&quote).map(|found| found.as_str()).collect();
        clean = if paragraphs.is_empty() { quote } else { paragraphs.concat() };
    }
    for token in TOKEN.find_iter(&clean).map(|found| found.as_str()) {
        if !token.starts_with('<') || token == "<" {
            result.push_str(&escape_html(&decode_entities(token)));
            continue;
        }
        let Some(captures) = TAG_NAME.captures(token) else { continue };
        let name = captures[2].to_ascii_lowercase();
        if !matches!(name.as_str(), "p" | "br" | "span" | "strong" | "a") {
            continue;
        }
        if !captures[1].is_empty() {
            if let Some(index) = stack.iter().rposition(|open| *open == name) {
                while stack.len() > index {
                    result.push_str(&format!("</{}>", stack.pop().unwrap_or_default()));
                }
            }
        } else if name == "br" {
            result.push_str("<br/>");
        } else {
            let href = if name == "a" { source_url(&attribute_value(&attributes(token), "href"), base) } else { None };
            result.push('<');
            result.push_str(&name);
            if let Some(href) = href {
                result.push_str(&format!(" href=\"{}\"", escape_html(&href)));
            }
            result.push('>');
            stack.push(name);
        }
    }
    while let Some(open) = stack.pop() {
        result.push_str(&format!("</{open}>"));
    }
    result
}

/// What [`page_metadata`] read from a page (`pageMetadata`'s result object).
#[derive(Clone, Debug, Default)]
pub struct PageMetadata {
    /// `<meta>` values by lower-cased `property`/`name`; only non-empty `content` is kept.
    pub meta: Object,
    /// JSON-LD objects that carry an `@type`, in document order.
    pub entities: Vec<Value>,
    /// Canonical page URL.
    pub canonical: String,
    /// oEmbed discovery URL, when the page declares one.
    pub oembed: Option<String>,
    /// Page title; any JSON value when it came from JSON-LD.
    pub title: Value,
    /// Page description; any JSON value when it came from JSON-LD.
    pub description: Value,
    /// Publisher name or the page's host name.
    pub site_name: String,
    /// Author: a meta string, or whatever JSON-LD held; undefined when absent.
    pub author: Value,
    /// Readable text, at most 30,000 UTF-16 units.
    pub text: String,
    /// Absolute author image URLs.
    pub author_images: Vec<String>,
    /// Absolute, de-duplicated image URLs.
    pub images: Vec<String>,
}

impl PageMetadata {
    /// The object `pageMetadata` returned, with its key order.
    ///
    /// # Returns
    /// An object whose absent `oembed` and `author` stay present as undefined.
    pub fn to_value(&self) -> Value {
        let strings = |items: &[String]| Value::Array(items.iter().map(|item| Value::String(item.clone())).collect());
        let mut object = Object::new();
        object.insert("meta", Value::Object(self.meta.clone()));
        object.insert("entities", Value::Array(self.entities.clone()));
        object.insert("canonical", self.canonical.clone());
        object.insert("oembed", self.oembed.clone().map_or(Value::Undefined, Value::String));
        object.insert("title", self.title.clone());
        object.insert("description", self.description.clone());
        object.insert("siteName", self.site_name.clone());
        object.insert("author", self.author.clone());
        object.insert("text", self.text.clone());
        object.insert("authorImages", strings(&self.author_images));
        object.insert("images", strings(&self.images));
        Value::Object(object)
    }
}

fn collect_entities(value: &Value, depth: usize, entities: &mut Vec<Value>) {
    if !value.truthy() || value.type_of() != "object" || depth > 12 {
        return;
    }
    if value.get("@type").truthy() {
        entities.push(value.clone());
    }
    let children: Vec<&Value> = match value {
        Value::Object(object) => object.values().collect(),
        Value::Array(items) => items.iter().collect(),
        _ => Vec::new(),
    };
    for child in children {
        if child.type_of() != "object" {
            continue;
        }
        if let Value::Array(items) = child {
            for item in items {
                collect_entities(item, depth + 1, entities);
            }
        } else {
            collect_entities(child, depth + 1, entities);
        }
    }
}

/// `[item['@type']].flat()` for an entity.
fn types_of(item: &Value) -> Result<Vec<Value>> {
    if item.is_nullish() {
        return Err(Error::invalid("Cannot read properties of null"));
    }
    Ok(match item.get("@type") {
        Value::Array(items) => items.clone(),
        other => vec![other.clone()],
    })
}

fn has_type(item: &Value, wanted: &str) -> Result<bool> {
    Ok(types_of(item)?.iter().any(|kind| matches!(kind, Value::String(name) if name == wanted)))
}

fn flat_one(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.clone(),
        other => vec![other.clone()],
    }
}

/// Read OpenGraph, Twitter, JSON-LD and oEmbed discovery metadata from fetched HTML (`pageMetadata`).
///
/// # Arguments
/// * `html` - Bounded page response, never executed.
/// * `url` - Final fetched page URL.
///
/// # Returns
/// Normalized metadata and the typed schema.org entities.
///
/// # Errors
/// An invalid-data error when `url` is not a URL and the page has no `og:site_name`.
pub fn page_metadata(html: &str, url: &str) -> Result<PageMetadata> {
    let mut meta = Object::new();
    for found in META_TAG.find_iter(html) {
        let attrs = attributes(found.as_str());
        let key = text::to_lower(attrs.get("property").or_else(|| attrs.get("name")).map_or("", String::as_str));
        if let Some(content) = attrs.get("content").filter(|content| !content.is_empty()) {
            match meta.get_mut(&key) {
                Some(Value::Array(items)) => items.push(Value::String(content.clone())),
                _ => meta.insert(key, Value::Array(vec![Value::String(content.clone())])),
            }
        }
    }
    let mut entities = Vec::new();
    for captures in SCRIPT_TAG.captures_iter(html) {
        let attrs = attributes(&format!("<script {}>", &captures[1]));
        if attrs.get("type").map(|kind| text::to_lower(kind)).as_deref() != Some("application/ld+json") {
            continue;
        }
        if let Ok(parsed) = json::parse(&captures[2]) {
            collect_entities(&parsed, 0, &mut entities);
        }
    }
    let (mut canonical, mut oembed) = (None, None);
    for found in LINK_TAG.find_iter(html) {
        let attrs = attributes(found.as_str());
        if attrs.get("rel").is_some_and(|rel| rel.split(text::is_whitespace).any(|part| part == "canonical")) {
            canonical = source_url(&attribute_value(&attrs, "href"), url);
        }
        if attrs.get("type").is_some_and(|kind| text::to_lower(kind) == "application/json+oembed") {
            oembed = source_url(&attribute_value(&attrs, "href"), url);
        }
    }
    let first = |key: &str| -> Option<String> {
        match meta.get(key) {
            Some(Value::Array(items)) => items.first().and_then(Value::as_str).map(str::to_owned),
            _ => None,
        }
    };
    let all = |key: &str| -> Option<Vec<Value>> {
        match meta.get(key) {
            Some(Value::Array(items)) => Some(items.clone()),
            _ => None,
        }
    };
    let mut entity = None;
    for item in &entities {
        if types_of(item)?.iter().any(|kind| ENTITY_TYPE.is_match(&kind.to_js_string())) {
            entity = Some(item);
            break;
        }
    }
    let entity = entity.or_else(|| entities.iter().find(|item| item.get("headline").truthy() || item.get("name").truthy()));
    let empty = Value::Object(Object::new());
    let entity = entity.unwrap_or(&empty);
    let mut image_sources: Vec<Value> = all("og:image:secure_url").unwrap_or_default();
    image_sources.extend(all("og:image").unwrap_or_default());
    image_sources.extend(all("twitter:image").or_else(|| all("twitter:image:src")).unwrap_or_default());
    let entity_image = entity.get("image").or(entity.get("thumbnailUrl"));
    image_sources.extend(flat_one(entity_image).into_iter().filter(Value::truthy));
    let mut images: Vec<String> = Vec::new();
    for image in image_sources.iter().filter_map(|image| source_url(image, url)) {
        if !images.contains(&image) {
            images.push(image);
        }
    }
    let main = ARTICLE
        .captures(html)
        .or_else(|| MAIN.captures(html))
        .map_or(html, |captures| captures.get(1).map_or("", |found| found.as_str()));
    let readable = READABLE_BLOCKS.replace_all(main, "");
    let paragraphs_found: Vec<String> = PARAGRAPH.find_iter(&readable).map(|found| plain_text(found.as_str())).filter(|part| !part.is_empty()).collect();
    let paragraphs = if PARAGRAPH.is_match(&readable) { paragraphs_found.join("\n") } else { plain_text(&readable) };
    let description = match first("og:description").or_else(|| first("twitter:description")).or_else(|| first("description")) {
        Some(found) => Value::String(found),
        None => entity.get("description").or(&Value::String(String::new())).clone(),
    };
    let mut authors: Vec<Value> = Vec::new();
    for item in &entities {
        if has_type(item, "Person")? {
            authors.push(item.clone());
        }
    }
    authors.extend(flat_one(entity.get("author")).into_iter().filter(|item| item.type_of() == "object" && item.truthy()));
    let mut author_images = Vec::new();
    for author in &authors {
        for image in flat_one(author.get("image")) {
            if let Some(resolved) = source_url(&image, url) {
                author_images.push(resolved);
            }
        }
    }
    let title_text = TITLE.captures(html).map(|captures| plain_text(&captures[1])).unwrap_or_default();
    let title = match first("og:title").or_else(|| first("twitter:title")) {
        Some(found) => Value::String(found),
        None => {
            let headline = entity.get("headline");
            if !headline.is_nullish() {
                headline.clone()
            } else if !entity.get("name").is_nullish() {
                entity.get("name").clone()
            } else {
                Value::String(title_text)
            }
        }
    };
    let site_name = match first("og:site_name") {
        Some(name) => name,
        None => mm_js::url::parse(url, None).ok_or_else(|| Error::invalid("Invalid URL"))?.host_str().unwrap_or("").to_owned(),
    };
    let author = match first("author").or_else(|| first("article:author")) {
        Some(found) => Value::String(found),
        None => {
            let named = entity.get("author").get("name");
            if named.is_nullish() { entity.get("author").clone() } else { named.clone() }
        }
    };
    let article_body = entity.get("articleBody").or(entity.get("text"));
    let mut parts: Vec<Value> = Vec::new();
    for part in [description.clone(), Value::String(plain_text_value(article_body)), Value::String(paragraphs)] {
        let duplicate = parts.iter().any(|seen| matches!((seen, &part), (Value::String(left), Value::String(right)) if left == right));
        if part.truthy() && !duplicate {
            parts.push(part);
        }
    }
    let joined = parts.iter().map(Value::to_js_string).collect::<Vec<_>>().join("\n");
    let canonical = canonical.or_else(|| source_url(&first("og:url").map_or(Value::Undefined, Value::String), url)).unwrap_or_else(|| url.to_owned());
    Ok(PageMetadata {
        meta,
        entities,
        canonical,
        oembed,
        title,
        description,
        site_name,
        author,
        text: text::slice(&joined, 0, Some(30000)),
        author_images,
        images,
    })
}

fn key_of(value: &Value) -> String {
    let lower = if value.is_nullish() { String::new() } else { text::to_lower(&value.to_js_string()) };
    NON_ALNUM.replace_all(&lower, "").into_owned()
}

/// Remove publisher or store boilerplate from a scraped title for the types that carry one (`captureTitle`).
///
/// # Arguments
/// * `title` - Scraped title, already entity-decoded.
/// * `kind` - Detected card type.
/// * `metadata` - Page metadata with `canonical`, `siteName` and `entities` (a [`PageMetadata::to_value`] or a partial object).
///
/// # Returns
/// The repository or app name, the title without a matching trailing site name, or the title unchanged.
///
/// # Errors
/// An invalid-data error when `metadata.canonical` is not a URL or `entities` is not an array where it is read; an
/// internal error when the repository path is not valid percent-encoding.
pub fn capture_title(title: &str, kind: &str, metadata: &Value) -> Result<String> {
    let url = mm_js::url::parse(&metadata.get("canonical").to_js_string(), None).ok_or_else(|| Error::invalid("Invalid URL"))?;
    if kind == "Repository" {
        let last = url.path().split('/').rfind(|part| !part.is_empty()).unwrap_or("undefined");
        let decoded = uri::decode_uri_component(last).map_err(|_| Error::internal("URI malformed"))?;
        return Ok(decoded.strip_suffix(".git").unwrap_or(&decoded).to_owned());
    }
    if kind == "SoftwareApplication" {
        let Value::Array(entities) = metadata.get("entities") else { return Err(Error::invalid("entities is not an array")) };
        let mut name = Value::Undefined;
        for item in entities {
            if has_type(item, "SoftwareApplication")? {
                name = item.get("name").clone();
                break;
            }
        }
        let name = if name.is_nullish() { title.to_owned() } else { name.to_js_string() };
        return Ok(text::trim(&APP_STORE.replace(&name, "")).to_owned());
    }
    if kind != "Article" && kind != "WebPage" {
        return Ok(title.to_owned());
    }
    let hostname = url.host_str().unwrap_or("");
    let bare = hostname.strip_prefix("www.").unwrap_or(hostname);
    let parts: Vec<&str> = bare.split('.').collect();
    let at = |from_end: usize| (parts.len() >= from_end).then(|| parts[parts.len() - from_end]);
    let second_level = at(2).is_some_and(|part| matches!(part, "co" | "com" | "org" | "net"));
    let brand = if second_level && at(1).is_some_and(|part| text::len(part) == 2) { at(3) } else { at(2) };
    if let Some(suffix) = TITLE_SUFFIX.captures(title) {
        let wanted = key_of(&Value::String(suffix[1].to_owned()));
        let sites = [metadata.get("siteName").clone(), Value::String(hostname.to_owned()), brand.map_or(Value::Undefined, |brand| Value::String(brand.to_owned()))];
        if sites.iter().any(|site| !key_of(site).is_empty() && key_of(site) == wanted) {
            let start = suffix.get(0).map_or(0, |found| found.start());
            return Ok(text::trim(&title[..start]).to_owned());
        }
    }
    Ok(title.to_owned())
}

/// Whether a URL is a known profile-image endpoint rather than post media (`profileImage`).
///
/// # Arguments
/// * `value` - Absolute candidate image URL.
///
/// # Errors
/// An invalid-data error when `value` is not a URL.
pub fn profile_image(value: &str) -> Result<bool> {
    let url = mm_js::url::parse(value, None).ok_or_else(|| Error::invalid("Invalid URL"))?;
    let path = uri::decode_uri_component(url.path()).unwrap_or_else(|_| url.path().to_owned());
    let host = url.host_str().unwrap_or("");
    Ok(PROFILE_PATH.is_match(&path)
        || PROFILE_HOST.is_match(host)
        || (host.ends_with("googleusercontent.com") && GOOGLE_PATH.is_match(&path))
        || INSTAGRAM_PATH.is_match(&path))
}

fn includes_article(value: &Value) -> Result<bool> {
    match value {
        Value::Undefined | Value::Null => Ok(false),
        Value::Array(items) => Ok(items.iter().any(|item| matches!(item, Value::String(text) if text == "article"))),
        Value::String(text) => Ok(text.contains("article")),
        _ => Err(Error::invalid("includes is not a function")),
    }
}

/// Detect the renderer type from a source URL, response MIME and publisher metadata (`detectType`).
///
/// # Arguments
/// * `value` - Saved or canonical entity URL.
/// * `metadata` - A [`PageMetadata::to_value`] or a partial object with `entities` and `meta`; undefined counts as `{}`.
/// * `mime` - HTTP Content-Type, or empty.
///
/// # Returns
/// The supported `renderCard` type name.
///
/// # Errors
/// An invalid-data error when `value` is not a URL or the metadata shapes cannot be read.
pub fn detect_type(value: &str, metadata: &Value, mime: &str) -> Result<&'static str> {
    let url = mm_js::url::parse(value, None).ok_or_else(|| Error::invalid("Invalid URL"))?;
    let lowered = text::to_lower(url.host_str().unwrap_or(""));
    let host = lowered.strip_prefix("www.").unwrap_or(&lowered);
    let path = url.path();
    if mime.contains("application/pdf") || PDF_PATH.is_match(path) {
        return Ok("Document");
    }
    if matches!(host, "x.com" | "twitter.com" | "mobile.twitter.com") && STATUS_PATH.is_match(path) {
        return Ok("XPost");
    }
    if matches!(host, "threads.net" | "threads.com") && path.contains("/post/") {
        return Ok("ThreadsPost");
    }
    if (host == "substack.com" || host.ends_with(".substack.com")) && (path.contains("/note/") || path.contains("/notes/")) {
        return Ok("SubstackNote");
    }
    if matches!(host, "youtube.com" | "m.youtube.com" | "youtu.be") {
        return Ok("YouTubeVideo");
    }
    if host == "instagram.com" && (path.contains("/reel/") || path.contains("/reels/")) {
        return Ok("InstagramReel");
    }
    if host == "github.com" && REPO_PATH.is_match(path) {
        return Ok("Repository");
    }
    if (host == "reddit.com" || host.ends_with(".reddit.com")) && path.contains("/comments/") {
        return Ok("RedditPost");
    }
    if host == "apps.apple.com" || host == "play.google.com" {
        return Ok("SoftwareApplication");
    }
    if matches!(metadata, Value::Null) {
        return Err(Error::invalid("Cannot read properties of null"));
    }
    let entities: Vec<Value> = match metadata.get("entities") {
        Value::Array(items) => items.clone(),
        Value::Undefined | Value::Null => Vec::new(),
        _ => return Err(Error::invalid("entities.some is not a function")),
    };
    for item in &entities {
        if types_of(item)?.iter().any(|kind| {
            let name = kind.to_js_string();
            name == "Product" || name.ends_with("/Product")
        }) {
            return Ok("Product");
        }
    }
    if includes_article(metadata.get("meta").get("og:type"))? {
        return Ok("Article");
    }
    for item in &entities {
        if types_of(item)?.iter().any(|kind| {
            let name = kind.to_js_string();
            name.ends_with("Article") || name.ends_with("BlogPosting")
        }) {
            return Ok("Article");
        }
    }
    Ok("WebPage")
}
