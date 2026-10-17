//! Port of `lib/search-document.mjs`: the text a card contributes to the search index and to AI analysis.

use mm_js::{Object, Value, json, text};

use crate::capture_html::plain_text_value;
use crate::{Error, Result};

/// Extract visible text from editor nodes, ignoring metadata and node attributes (`proseText`).
///
/// # Arguments
/// * `node` - Prose document or child node; anything else yields the empty string.
///
/// # Returns
/// The `text` of the node and of its `content` children, space-separated, skipping empty parts.
///
/// # Errors
/// An invalid-data error when a node's `content` is neither an array nor nullish.
pub fn prose_text(node: &Value) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    let own = node.get("text");
    if own.truthy() {
        parts.push(own.to_js_string());
    }
    match node.get("content") {
        Value::Undefined | Value::Null => {}
        Value::Array(children) => {
            for child in children {
                let part = prose_text(child)?;
                if !part.is_empty() {
                    parts.push(part);
                }
            }
        }
        _ => return Err(Error::invalid("content.map is not a function")),
    }
    Ok(parts.join(" "))
}

/// The analysis item of a card (`analysisItem`'s result).
#[derive(Clone, Debug)]
pub struct AnalysisItem {
    /// The card's `data.type`.
    pub kind: Value,
    /// The card's `data.title`.
    pub title: Value,
    /// Searchable body text, at most 30,000 UTF-16 units.
    pub text: String,
    /// The card's `data.source.url`.
    pub url: Value,
    /// The card's `data.domain`.
    pub domain: Value,
}

impl AnalysisItem {
    /// The object `analysisItem` returned, absent fields kept as undefined.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", self.kind.clone());
        object.insert("title", self.title.clone());
        object.insert("text", self.text.clone());
        object.insert("url", self.url.clone());
        object.insert("domain", self.domain.clone());
        Value::Object(object)
    }
}

fn parse(source: &str) -> Result<Value> {
    json::parse(source).map_err(Error::from)
}

fn dedupe_join(parts: Vec<Value>, separator: &str) -> String {
    let mut kept: Vec<Value> = Vec::new();
    for part in parts {
        let duplicate = kept.iter().any(|seen| match (seen, &part) {
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right || (left.is_nan() && right.is_nan()),
            (Value::Bool(left), Value::Bool(right)) => left == right,
            _ => false,
        });
        if part.truthy() && !duplicate {
            kept.push(part);
        }
    }
    kept.iter().map(Value::to_js_string).collect::<Vec<_>>().join(separator)
}

/// Build the analysis item shared by newly saved cards and the embedding backfill (`analysisItem`).
///
/// # Arguments
/// * `data` - The card row's serialized `data`.
/// * `extras` - The card row's serialized `extras`.
/// * `content` - The current Article reader document, when there is one.
///
/// # Returns
/// The item's type, title, text (capped at 30,000 UTF-16 units), source URL and domain.
///
/// # Errors
/// An internal error for malformed JSON; an invalid-data error where the JS read a property of null.
pub fn analysis_item(data: &str, extras: &str, content: Option<&Value>) -> Result<AnalysisItem> {
    let data = parse(data)?;
    let extras = parse(extras)?;
    if matches!(data, Value::Null) {
        return Err(Error::invalid("Cannot read properties of null"));
    }
    let body = if data.get("type").as_str() == Some("Article") {
        let prose = content.filter(|content| !content.is_nullish()).unwrap_or_else(|| data.get("prose"));
        prose_text(prose)?
    } else {
        if matches!(extras, Value::Null) {
            return Err(Error::invalid("Cannot read properties of null"));
        }
        let post_html = extras.get("post").get("bodyHtml");
        dedupe_join(
            vec![
                data.get("description").clone(),
                Value::String(prose_text(data.get("prose"))?),
                extras.get("captureText").clone(),
                Value::String(plain_text_value(post_html)),
            ],
            "\n",
        )
    };
    Ok(AnalysisItem {
        kind: data.get("type").clone(),
        title: data.get("title").clone(),
        text: text::slice(&body, 0, Some(30000)),
        url: data.get("source").get("url").clone(),
        domain: data.get("domain").clone(),
    })
}

/// Extract the searchable fields of a card for the external-content FTS table (`searchDocument`).
///
/// # Arguments
/// * `data_json` - Serialized card data.
/// * `extras_json` - Serialized capture and render metadata.
/// * `content_json` - Serialized current reader body; empty or absent means the card's own `prose`.
/// * `tags` - Annotation names joined with newlines.
///
/// # Returns
/// JSON text with the keys title, description, summary, tags, url, body, note and ocr.
///
/// # Errors
/// An internal error for malformed JSON; an invalid-data error where the JS read a property of null.
pub fn search_document(data_json: &str, extras_json: &str, content_json: Option<&str>, tags: Option<&str>) -> Result<String> {
    let data = parse(data_json)?;
    let extras = parse(extras_json)?;
    let prose = match content_json.filter(|content| !content.is_empty()) {
        Some(content) => parse(content)?,
        None => data.get("prose").clone(),
    };
    if matches!(data, Value::Null) {
        return Err(Error::invalid("Cannot read properties of null"));
    }
    let is_article = data.get("type").as_str() == Some("Article") && prose.truthy();
    let body = if is_article {
        prose_text(&prose)?
    } else {
        if matches!(extras, Value::Null) {
            return Err(Error::invalid("Cannot read properties of null"));
        }
        [Value::String(prose_text(&prose)?), extras.get("captureText").clone(), Value::String(plain_text_value(extras.get("post").get("bodyHtml")))]
            .iter()
            .filter(|part| part.truthy())
            .map(Value::to_js_string)
            .collect::<Vec<_>>()
            .join("\n")
    };
    if matches!(extras, Value::Null) {
        return Err(Error::invalid("Cannot read properties of null"));
    }
    let title = data.get("title").or(extras.get("title")).or(&Value::String(String::new())).clone();
    let url = [data.get("source").get("url"), extras.get("linkUrl"), data.get("domain"), data.get("siteName")]
        .into_iter()
        .filter(|part| part.truthy())
        .map(Value::to_js_string)
        .collect::<Vec<_>>()
        .join(" ");
    let mut object = Object::new();
    object.insert("title", title);
    object.insert("description", plain_text_value(data.get("description")));
    object.insert("summary", data.get("summary").or(&Value::String(String::new())).clone());
    object.insert("tags", tags.unwrap_or(""));
    object.insert("url", url);
    object.insert("body", body);
    object.insert("note", prose_text(data.get("note").get("prose"))?);
    object.insert("ocr", data.get("ocr").or(&Value::String(String::new())).clone());
    json::stringify(&Value::Object(object))
        .map_err(|error| Error::internal(error.0))?
        .ok_or_else(|| Error::internal("search document is undefined"))
}
