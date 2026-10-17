//! Port of `lib/render/prose.mjs`: the ProseMirror card preview.

use mm_js::Value;
use mm_js::text::len;

use super::access::{member, optional, prototype_member, text};
use super::encoding::escape_prose as e;
use crate::{Error, Result};

/// Visible text and lines of a node and its descendants.
struct Measure {
    chars: f64,
    lines: f64,
}

/// The items of `(node.content ?? []).map(...)`; anything but an array or nullish has no `map`.
fn content(node: &Value) -> Result<&[Value]> {
    match member(node, "content")? {
        Value::Array(items) => Ok(items),
        value if value.is_nullish() => Ok(&[]),
        _ => Err(Error::invalid("node.content.map is not a function")),
    }
}

/// `value.length` as a number, NaN where JS reads `undefined`.
fn length(value: &Value) -> f64 {
    match value {
        Value::String(text) => len(text) as f64,
        Value::Array(items) => items.len() as f64,
        _ => f64::NAN,
    }
}

/// Count visible text and lines before deciding where a card preview ends.
fn measure(node: &Value) -> Result<Measure> {
    let mut children = Vec::new();
    for child in content(node)? {
        children.push(measure(child)?);
    }
    let text = member(node, "text")?;
    let own = if text.is_nullish() { 0.0 } else { length(text) };
    let block = matches!(node.get("type").as_str(), Some("hardBreak" | "paragraph" | "heading" | "image"));
    Ok(Measure {
        chars: own + children.iter().fold(0.0, |n, x| n + x.chars),
        lines: f64::from(u8::from(block)) + children.iter().fold(0.0, |n, x| n + x.lines),
    })
}

/// The tag an inline mark wraps its text in.
fn mark_tag(kind: &str) -> Option<String> {
    let tag = match kind {
        "bold" => "strong",
        "italic" => "em",
        "underline" => "u",
        "strike" => "s",
        "code" => "code",
        "highlight" => "mark",
        "subscript" => "sub",
        "superscript" => "sup",
        _ => return prototype_member(kind),
    };
    Some(tag.into())
}

/// The block tag for a node type.
fn block_tag(node: &Value) -> Option<String> {
    let kind = node.get("type").to_js_string();
    let tag = match kind.as_str() {
        "paragraph" => "p".to_owned(),
        "heading" => format!("h{}", optional(node.get("attrs"), "level").or(&Value::Number(1.0)).to_js_string()),
        "bulletList" => "ul".into(),
        "orderedList" => "ol".into(),
        "listItem" => "li".into(),
        "blockquote" => "blockquote".into(),
        "codeBlock" => "pre".into(),
        _ => return prototype_member(&kind),
    };
    Some(tag)
}

/// A text node with its marks applied innermost first.
fn render_text(node: &Value) -> Result<String> {
    let mut out = e(&text(node.get("text")));
    let marks = match node.get("marks") {
        Value::Array(items) => items.as_slice(),
        Value::String(_) => &[],
        value if value.is_nullish() => &[],
        _ => return Err(Error::invalid("node.marks is not iterable")),
    };
    for mark in marks.iter().rev() {
        let kind = member(mark, "type")?;
        if let Some(tag) = mark_tag(&kind.to_js_string()) {
            out = format!("<{tag}>{out}</{tag}>");
        } else if kind.as_str() == Some("link") {
            out = format!("<a href=\"{}\">{out}</a>", e(&text(optional(mark.get("attrs"), "href"))));
        }
    }
    Ok(out)
}

/// Serialize the document nodes supported by the recorded ProseMirror documents.
fn render_node(node: &Value) -> Result<String> {
    let mut children = String::new();
    for child in content(node)? {
        children.push_str(&render_node(child)?);
    }
    let attrs = node.get("attrs");
    let attr = |key: &str| optional(attrs, key);
    let kind = node.get("type").as_str().unwrap_or("");
    match kind {
        "text" => return render_text(node),
        "hardBreak" => return Ok("<br />".into()),
        "doc" => return Ok(children),
        "wikiLink" => {
            let attributes: String = [("id", "id"), ("target", "target"), ("targetId", "target-id"), ("targetType", "target-type"), ("text", "text")]
                .iter()
                .filter(|(key, _)| attr(key).truthy())
                .map(|(key, name)| format!(" data-{name}=\"{}\"", e(&text(attr(key)))))
                .collect();
            let label = attr("text").or(attr("target")).or(&Value::String(String::new())).to_js_string();
            return Ok(format!("<wiki-link{attributes}><span>{}</span></wiki-link>", e(&label)));
        }
        "taskList" => return Ok(format!("<ul data-type=\"taskList\">{children}</ul>")),
        "taskItem" => {
            let checked = attr("checked").truthy();
            return Ok(format!(
                "<li data-type=\"taskItem\" data-checked=\"{checked}\"><label><input type=\"checkbox\"{} disabled /><span></span></label><div>{children}</div></li>",
                if checked { " checked" } else { "" }
            ));
        }
        "table" => return Ok(format!("<table><tbody>{children}</tbody></table>")),
        "tableRow" => return Ok(format!("<tr>{children}</tr>")),
        "tableCell" | "tableHeader" => {
            let tag = if kind == "tableCell" { "td" } else { "th" };
            let span = |key: &str| e(&attr(key).or(&Value::Number(1.0)).to_js_string());
            return Ok(format!("<{tag} colspan=\"{}\" rowspan=\"{}\">{children}</{tag}>", span("colspan"), span("rowspan")));
        }
        _ => {}
    }
    if let Some(tag) = block_tag(node) {
        return Ok(format!("<{tag}>{children}</{tag}>"));
    }
    match kind {
        "horizontalRule" => Ok("<hr />".into()),
        "image" => Ok(format!("<img src=\"{}\" />", e(&text(attr("src"))))),
        _ => Err(Error::invalid(format!("Unsupported prose node: {}", node.get("type").to_js_string()))),
    }
}

/// Render a ProseMirror document's card preview, ending at a block boundary (`renderProse`).
///
/// # Arguments
/// * `doc` - The card's prose document `{type, content?}`.
///
/// # Returns
/// The inner prose div, including its trailing newline.
///
/// # Errors
/// A node is `null`, has a non-array `content`, or has a type the recorded documents never use.
pub fn render_prose(doc: &Value) -> Result<String> {
    let total = measure(doc)?;
    let (mut chars, mut lines) = (0.0, 0.0);
    let mut blocks = String::new();
    for node in content(doc)? {
        blocks.push_str(&render_node(node)?);
        let size = measure(node)?;
        chars += size.chars;
        lines += size.lines;
        if chars > 1000.0 || lines > 20.0 {
            break;
        }
    }
    let long = if total.chars > 500.0 || total.lines > 10.0 { "long" } else { "" };
    Ok(format!("<div class=\"inner body prose {long}\">\n  {blocks}\n</div>\n"))
}
