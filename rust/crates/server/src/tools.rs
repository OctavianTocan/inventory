//! What the operational subcommands share.

use mm_core::Result;
use mm_core::ai::{AiItem, document_text};
use mm_core::search_document::analysis_item;
use mm_js::{Value, json};

// `String(value ?? '')` as `clip` reads it, with null and undefined left out.
fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Undefined | Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => Some(other.to_js_string()),
    }
}

/// The item a card's library embedding is computed from (`embeddingItem` of `tools/embed-library.mjs`); its
/// [`document_text`] identifies the vector's input.
///
/// # Arguments
/// * `data` - The card row's serialized `data`.
/// * `extras` - The card row's serialized `extras`; `"null"` for a NULL column.
/// * `body` - The card's `card_content.body`, if any; an empty one counts as none.
///
/// # Returns
/// The analysis item, its text falling back to the card's `summary` when it has no other text.
///
/// # Errors
/// Malformed JSON, or the errors of `analysis_item`.
pub fn embedding_item(data: &str, extras: &str, body: Option<&str>) -> Result<AiItem> {
    let content = body.filter(|body| !body.is_empty()).map(json::parse).transpose()?;
    let analysis = analysis_item(data, extras, content.as_ref())?;
    let mut item = AiItem {
        title: text_of(&analysis.title),
        kind: text_of(&analysis.kind),
        text: Some(analysis.text),
        url: text_of(&analysis.url),
        domain: text_of(&analysis.domain),
        image: None,
    };
    if document_text(&item).is_empty() {
        item.text = Some(text_of(json::parse(data)?.get("summary")).unwrap_or_default());
    }
    Ok(item)
}
