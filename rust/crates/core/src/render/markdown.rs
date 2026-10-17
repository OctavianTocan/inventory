//! Inferred Markdown export for the web and native Copy actions; no response was recorded.

use std::sync::LazyLock;

use mm_js::Value;
use regex::Regex;

static HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*)(#{1,6})(\s|$)").unwrap());
static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*)(>|-|\+(?:\s|$))").unwrap());
static ORDERED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*\d{1,9})([.)])(\s|$)").unwrap());

// Escape only the inline characters that would start Markdown syntax, so copied text stays readable.
fn escape(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let word = |at: Option<&char>| at.is_some_and(|c| c.is_alphanumeric());
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let intraword = c == '_' && i > 0 && word(chars.get(i - 1)) && word(chars.get(i + 1));
        let tag = c == '<' && chars.get(i + 1).is_some_and(|next| next.is_ascii_alphabetic() || matches!(next, '/' | '!' | '?'));
        if "\\`*[]~".contains(c) || (c == '_' && !intraword) || tag {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

// Escape line-start markers that would turn a line of text into a heading, list or quote.
fn line_starts(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let line = HEADING.replace(line, "$1\\$2$3");
            let line = MARKER.replace(&line, "$1\\$2");
            ORDERED.replace(&line, "$1\\$2$3").into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// Keep destinations inside angle brackets, including URLs with parentheses or spaces.
fn destination(text: &str) -> String {
    format!("<{}>", text.replace('<', "%3C").replace('>', "%3E").replace('\n', "%0A").replace('\r', "%0D"))
}

// Return a node's children, or an empty list for a leaf.
fn children(node: &Value) -> &[Value] {
    node.get("content").as_array().map_or(&[], |items| items.as_slice())
}

// Pick a delimiter longer than any run in the literal text.
fn fence(text: &str, minimum: usize) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat(minimum.max(longest + 1))
}

// Serialize inline marks without interpreting literal text as Markdown.
fn inline(node: &Value) -> String {
    let attrs = node.get("attrs");
    match node.get("type").as_str().unwrap_or("") {
        "hardBreak" => return "  \n".into(),
        "image" => return format!("![{}]({})", escape(attrs.get("alt").as_str().unwrap_or("")), destination(attrs.get("src").as_str().unwrap_or(""))),
        "wikiLink" => return format!("[[{}]]", attrs.get("text").as_str().or(attrs.get("target").as_str()).unwrap_or("")),
        _ => {}
    }
    let raw = node.get("text").as_str();
    let mut out = raw.map_or_else(|| children(node).iter().map(inline).collect(), escape);
    let marks = node.get("marks").as_array().map_or(&[][..], |items| items.as_slice());
    for mark in marks.iter().rev() {
        out = match mark.get("type").as_str().unwrap_or("") {
            "bold" => format!("**{out}**"),
            "italic" => format!("*{out}*"),
            "strike" => format!("~~{out}~~"),
            "code" => {
                let text = raw.unwrap_or(&out);
                let delimiter = fence(text, 1);
                let pad = if text.starts_with(['`', ' ']) || text.ends_with(['`', ' ']) { " " } else { "" };
                format!("{delimiter}{pad}{text}{pad}{delimiter}")
            }
            "link" => format!("[{out}]({})", destination(mark.get("attrs").get("href").as_str().unwrap_or(""))),
            "underline" => format!("<u>{out}</u>"),
            "subscript" => format!("<sub>{out}</sub>"),
            "superscript" => format!("<sup>{out}</sup>"),
            "highlight" => format!("<mark>{out}</mark>"),
            _ => out,
        };
    }
    out
}

// Indent continuation lines underneath a list marker.
fn list_item(prefix: &str, body: &str) -> String {
    let indent = " ".repeat(prefix.len());
    let mut lines = body.lines();
    let mut out = format!("{prefix}{}", lines.next().unwrap_or(""));
    for line in lines {
        out.push_str(&format!("\n{indent}{line}"));
    }
    out
}

// Serialize block nodes, keeping complete documents rather than card-preview truncation.
fn block(node: &Value) -> String {
    let attrs = node.get("attrs");
    let parts = || children(node).iter().map(block).collect::<Vec<_>>().join("\n\n");
    match node.get("type").as_str().unwrap_or("") {
        "doc" | "listItem" | "taskItem" => parts(),
        "paragraph" => line_starts(&children(node).iter().map(inline).collect::<String>()),
        "heading" => format!("{} {}", "#".repeat(attrs.get("level").as_f64().unwrap_or(1.0).clamp(1.0, 6.0) as usize), children(node).iter().map(inline).collect::<String>()),
        "blockquote" => parts().lines().map(|line| format!("> {line}")).collect::<Vec<_>>().join("\n"),
        "bulletList" | "orderedList" | "taskList" => {
            let kind = node.get("type").as_str().unwrap_or("");
            let start = attrs.get("start").as_f64().unwrap_or(1.0) as i64;
            children(node)
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let prefix = match kind {
                        "orderedList" => format!("{}. ", start.saturating_add(index as i64)),
                        "taskList" => format!("- [{}] ", if item.get("attrs").get("checked").truthy() { "x" } else { " " }),
                        _ => "- ".into(),
                    };
                    list_item(&prefix, &block(item))
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "codeBlock" => {
            let text = children(node).iter().filter_map(|n| n.get("text").as_str()).collect::<String>();
            let delimiter = fence(&text, 3);
            let language = attrs.get("language").as_str().unwrap_or("").replace(['`', '\r', '\n'], "");
            format!("{delimiter}{language}\n{text}{}{delimiter}", if text.ends_with('\n') { "" } else { "\n" })
        }
        "horizontalRule" => "---".into(),
        "table" => {
            let rows: Vec<Vec<String>> = children(node).iter().map(|row| children(row).iter().map(|cell| block(cell).replace('|', "\\|").replace('\n', "<br>")).collect()).collect();
            let width = rows.iter().map(Vec::len).max().unwrap_or(0);
            let mut out = Vec::new();
            for (index, row) in rows.iter().enumerate() {
                out.push(format!("| {} |", (0..width).map(|i| row.get(i).map_or("", String::as_str)).collect::<Vec<_>>().join(" | ")));
                if index == 0 {
                    out.push(format!("| {} |", vec!["---"; width].join(" | ")));
                }
            }
            out.join("\n")
        }
        "tableCell" | "tableHeader" => parts(),
        _ => inline(node),
    }
}

/// Export a card as UTF-8 Markdown for Copy and file export.
///
/// # Arguments
/// * `data` - Parsed card JSON, including prose, source and media metadata.
/// * `content` - Current editor document from `card_content`, or null to use the card's prose.
///
/// # Returns
/// Complete prose with a final newline, or an image reference, description or source URL when no prose exists.
/// Empty cards return an empty string. This format is inferred from client use, not a recorded response.
pub fn card_markdown(data: &Value, content: &Value) -> String {
    let prose = if content.is_nullish() { data.get("prose") } else { content };
    let body = if !prose.is_nullish() {
        block(prose)
    } else if data.get("type").as_str() == Some("Image") && data.get("blob").get("path").as_str().is_some() {
        format!("![{}]({})", escape(data.get("title").as_str().unwrap_or("")), destination(&format!("/media/{}", data.get("blob").get("path").as_str().unwrap_or(""))))
    } else {
        data.get("description").as_str().map(|text| line_starts(&escape(text))).unwrap_or_else(|| data.get("source").get("url").as_str().unwrap_or("").to_owned())
    };
    if body.is_empty() { body } else { format!("{}\n", body.trim_end_matches('\n')) }
}
