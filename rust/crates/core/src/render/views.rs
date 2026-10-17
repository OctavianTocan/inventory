//! Port of `lib/render/views.mjs`: the pinned, detail and share fragments rendered for local cards.

use std::sync::LazyLock;

use mm_js::{Value, json, obj};
use regex::Regex;

use super::encoding::escape_html;
use crate::store::{CardRow, iso_from_ns};
use crate::{Error, Result};

static COVER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^<mymind-card class="([^"]*)"[^>]*>"#).expect("pattern"));
static HOVER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<div class="hover"[^>]*>[\s\S]*?</div>"#).expect("pattern"));
static CLASS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"class="([^"]*)""#).expect("pattern"));
static MEDIA_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(?:img|video)(?-u:\b)[^>]*>").expect("pattern"));

fn e(value: &Value) -> String {
    if value.is_nullish() { String::new() } else { escape_html(&value.to_js_string()) }
}

fn json_text(value: &Value) -> Result<String> {
    Ok(json::stringify(value)?.unwrap_or_default().replace('<', "\\u003c"))
}

fn editor(id: &str, kind: &str, prose: &Value) -> Result<String> {
    let kind_type = if kind == "note" { "note" } else { "card" };
    let prose_class = if kind == "note" { "note" } else { "detail" };
    let doc: Value = obj! { "type" => "doc", "content" => Vec::<Value>::new() }.into();
    let id = escape_html(id);
    Ok(format!(
        r#"<mymind-editor class="editor" data-type="{kind_type}" data-card-id="{id}" data-prose-class="card-{prose_class}-prose" data-action="/objects/{id}/{kind}"><script type="application/json">{}</script></mymind-editor>"#,
        json_text(if prose.is_nullish() { &doc } else { prose })?
    ))
}

/// Render a draggable pinned preview from a card's grid markup (`renderPinned`).
///
/// # Arguments
/// * `row` - Stored card whose `html` and `id` are used.
///
/// # Returns
/// The Top of Mind item.
pub fn render_pinned(row: &CardRow) -> String {
    let cover = COVER.replace(&row.html, r#"<div class="tom__item-cover $1">"#).into_owned();
    let cover = HOVER.replace(&cover, "").into_owned();
    let cover = cover.strip_suffix("</mymind-card>").map_or(cover.clone(), |head| format!("{head}</div>"));
    format!(r#"<div class="tom__item draggable" data-id="{}">{cover}</div>"#, escape_html(&row.id))
}

/// Render the editable desktop detail of a card (`renderDetail`).
///
/// # Arguments
/// * `row` - Stored card with JSON `data`, `extras` and grid `html`.
/// * `share` - Active share model, or `Value::Null`.
/// * `content` - Stored reader prose for an Article, or `Value::Null`.
///
/// # Returns
/// The detail partial.
///
/// # Errors
/// An invalid-data error for malformed card JSON or a card without the fields the template reads.
pub fn render_detail(row: &CardRow, share: &Value, content: &Value) -> Result<String> {
    let data = json::parse(&row.data)?;
    let id = escape_html(&row.id);
    let class = CLASS.captures(&row.html).map_or(String::new(), |found| found[1].to_owned());
    let watchable = matches!(class.split(' ').next(), Some("youtube-video" | "instagram-reel"));
    let summary = data.get("summary");
    let tldr = if summary.truthy() { format!(r#"<mymind-tldr data-watchable="{watchable}"><p>{}</p></mymind-tldr>"#, e(summary)) } else { String::new() };
    let reader = if content.is_nullish() { data.get("prose") } else { content };
    let kind = data.get("type");
    let body = if matches!(kind.as_str(), Some("Note" | "Content")) {
        editor(&row.id, "prose", data.get("prose"))?
    } else if kind.as_str() == Some("Article") && (reader.truthy() || json::parse(&row.extras)?.get("local").truthy()) {
        editor(&row.id, "content", reader)?
    } else {
        row.html.clone()
    };
    let tags = data.get("tags");
    let tags_json = escape_html(&json::stringify(&if tags.is_nullish() { Value::Array(vec![]) } else { tags.clone() })?.unwrap_or_default());
    let highlighted = match tags {
        Value::Undefined | Value::Null => false,
        Value::Array(items) => {
            let mut found = false;
            for tag in items {
                if tag.is_nullish() {
                    return Err(Error::invalid("Cannot read properties of null (reading 'name')"));
                }
                if tag.get("name").as_str() == Some("highlight") {
                    found = true;
                    break;
                }
            }
            found
        }
        _ => return Err(Error::invalid("data.tags?.some is not a function")),
    };
    let highlighted = if highlighted { " highlighted" } else { "" };
    let Some(type_name) = kind.as_str() else { return Err(Error::invalid("Cannot read properties of undefined (reading 'toLowerCase')")) };
    let Some(created) = data.get("created").as_str() else { return Err(Error::invalid("Cannot read properties of undefined (reading 'slice')")) };
    let created_day = mm_js::text::slice(created, 0, Some(10));
    let source = data.get("source");
    let visit = if source.get("url").truthy() { format!(r#"<a href="{}" target="_blank">Visit original source</a>"#, e(source.get("url"))) } else { String::new() };
    let title = e(data.get("title"));
    let shared = if share.truthy() { " shared" } else { "" };
    let card_data = json_text(&obj! { "id" => row.id.as_str(), "type" => kind.clone(), "share" => share.clone() }.into())?;
    let note = editor(&row.id, "note", data.get("note").get("prose"))?;
    let (modified, created_at) = (iso_from_ns(row.modified_ns, 7)?, iso_from_ns(row.created_ns, 7)?);
    let (expires, link) = (e(share.get("expires")), e(share.get("url")));
    let (type_html, slug) = (escape_html(type_name), escape_html(&mm_js::text::to_lower(type_name)));
    Ok(format!(
        r#"<mymind-card-header><div class="inner"><button on-click="card:close"><span class="icon close"></span></button><span class="title">{title}</span><button class="options-button" on-click="dropdown#card-options:toggle"><span class="icon more"></span></button></div><mymind-dropdown data-name="card-options">{visit}</mymind-dropdown></mymind-card-header>
<mymind-card-detail class="{class_html} default-surface default-mode{shared}">
<script class="card-data" type="application/json">{card_data}</script>
<div class="trc-actions"><button class="toggle-sidebar" on-click="card:expand"></button></div>
<div class="guts"><div class="body inner{highlighted}"><div class="wrapper preview-guts">{body}</div></div>
<mymind-revision-controls style="display:none"><p class="ro-title"></p><p class="ro-text"></p><div class="buttons"><button class="restore">Restore</button><button class="cancel">Cancel</button></div></mymind-revision-controls></div>
<mymind-card-sidebar data-object-id="{id}" data-modified="{modified}">
<header><mymind-editable data-action="/objects/{id}"><input class="title" value="{title}" placeholder="Title goes here" maxlength="100"/></mymind-editable><div class="line"><time datetime="{created_at}">{created_day}</time><ul class="object-revision-list"><li class="loading-indicator"></li></ul></div><div class="advanced-options"></div></header>
<div class="scroll">{tldr}<mymind-tag-cloud data-object-id="{id}"><div class="header"><label>Tags</label></div><form class="add-tag-form"><div class="field"><input class="input" autocomplete="off" name="content" maxlength="100" required/><input class="ghost" disabled/><ul class="suggestions"></ul><button class="submit" type="submit">Add</button></div><mymind-recent-tag-block></mymind-recent-tag-block></form><div class="guts"><mymind-tag-list object-type="{type_html}" object-type-name="{type_html}" object-type-slug="{slug}" tags="{tags_json}"></mymind-tag-list><div class="cut-off"></div></div></mymind-tag-cloud>
<mymind-card-note class="card-note"><div class="header"><label>Notes</label></div><div class="guts">{note}</div></mymind-card-note></div>
<div class="bottom"><mymind-card-actions class="actions"><a class="card-action add-to-space-action" on-click="card#{id}:addToSpace"><i class="icon"></i><span class="tooltip">Add to space</span><span class="notice"></span><mymind-space-chooser backdrop="transparent"></mymind-space-chooser></a><a class="card-action accord-action" on-click="card#{id}:share"><i class="icon"></i><i class="countdown" expires="{expires}"></i><i class="progress"></i><span class="tooltip"></span><div class="mobile-progress"><div class="mobile-disk"></div></div></a><a class="card-action delete-action" on-click="card#{id}:delete"><i class="icon"></i><span class="tooltip">Delete</span></a></mymind-card-actions><mymind-share-link on-click="card#{id}:copyShareUrl"><p><label>Share link</label><a href="{link}" target="_blank">{link}</a></p><div class="copy"><i class="icon"></i><div class="tooltip">Copy link</div></div></mymind-share-link></div>
</mymind-card-sidebar></mymind-card-detail>"#,
        class_html = escape_html(&class),
    ))
}

/// Render the standalone share page of a card (`renderShare`).
///
/// # Arguments
/// * `row` - Card whose active share the caller already checked.
///
/// # Returns
/// A complete HTML document holding the card.
///
/// # Errors
/// An invalid-data error for malformed card JSON.
pub fn render_share(row: &CardRow) -> Result<String> {
    let data = json::parse(&row.data)?;
    let html = MEDIA_TAG.replace_all(&row.html, |found: &regex::Captures| {
        let mut tag = found[0].to_owned();
        for attribute in ["src", "srcset"] {
            let lazy = format!(" data-{attribute}=");
            if tag.contains(&lazy) {
                let plain = Regex::new(&format!(r#" {attribute}="[^"]*""#)).expect("pattern");
                tag = plain.replace_all(&tag, "").replacen(&lazy, &format!(" {attribute}="), 1);
            }
        }
        tag
    });
    let title = data.get("title");
    let title = if title.is_nullish() { "Shared card".to_owned() } else { escape_html(&title.to_js_string()) };
    Ok(format!(r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title></head><body>{html}</body></html>"#))
}
