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

fn eager_media(html: &str) -> String {
    MEDIA_TAG
        .replace_all(html, |found: &regex::Captures| {
            let mut tag = found[0].to_owned();
            for attribute in ["src", "srcset"] {
                let lazy = format!(" data-{attribute}=");
                if tag.contains(&lazy) {
                    let plain = Regex::new(&format!(r#" {attribute}="[^"]*""#)).expect("pattern");
                    tag = plain.replace_all(&tag, "").replacen(&lazy, &format!(" {attribute}="), 1);
                }
            }
            tag.replace(" class=\"lazy\"", "").replace(" class=\"lazy ", " class=\"").replace(" lazy\"", "\"")
        })
        .into_owned()
}

/// Render a draggable pinned preview from a card's grid markup (`renderPinned`).
///
/// # Arguments
/// * `row` - Stored card whose `html` and `id` are used.
///
/// # Returns
/// The Top of Mind item.
pub fn render_pinned(row: &CardRow) -> String {
    let html = eager_media(&row.html);
    let cover = COVER.replace(&html, r#"<div class="tom__item-cover $1">"#).into_owned();
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
    let extras = json::parse(&row.extras).unwrap_or(Value::Null);
    let id = escape_html(&row.id);
    let class = CLASS.captures(&row.html).map_or(String::new(), |found| found[1].to_owned());
    let watchable = matches!(class.split(' ').next(), Some("youtube-video" | "instagram-reel"));
    let summary = if data.get("summary").truthy() { data.get("summary") } else { extras.get("summary") };
    let tldr = if summary.truthy() { format!(r#"<mymind-tldr data-watchable="{watchable}"><p>{}</p></mymind-tldr>"#, e(summary)) } else { String::new() };
    let reader = if content.is_nullish() { data.get("prose") } else { content };
    let kind = data.get("type");
    let body = if matches!(kind.as_str(), Some("Note" | "Content")) {
        editor(&row.id, "prose", data.get("prose"))?
    } else if kind.as_str() == Some("Article") {
        editor(&row.id, "content", reader)?
    } else {
        eager_media(&row.html)
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
    let title = if data.get("title").truthy() {
        e(data.get("title"))
    } else if extras.get("title").truthy() {
        e(extras.get("title"))
    } else {
        e(source.get("url"))
    };
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
    let html = eager_media(&row.html);
    let title = data.get("title");
    let title = if title.is_nullish() { "Shared card".to_owned() } else { escape_html(&title.to_js_string()) };
    Ok(format!(r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title></head><body>{html}</body></html>"#))
}

const NEW_SPACE_MODAL: &str = r##"<dialog class="new-space-modal modal animate-up" escapable backdrop-on-click="modal:close">
  <button class="close big" on-click="modal:close"></button>
  <div class="box">
    <div class="step step-1">
      <div class="inner">
        <h2 class="static">Create new space</h2>
        <h2 class="dynamic">Create smart space</h2>
        <p class="dynamic description">This "smart space" is a dynamic collection of one or more search terms saved into one space.<mymind-tip><span class="content">A regular Space is static, meaning it only changes when you manually add cards to it. <br/> <br/>A Smart Space is dynamic. It grows automatically when any new card matches the search terms of your space.</span></mymind-tip></p>
        <p class="static">A space is a collection of cards inside your mind. Upload directly into a space, or pick a card from the overview.</p>
        <div class="new-space__filters dynamic"></div>
        <form>
          <div class="field"><input type="text" placeholder="Name your new space" required/></div>
          <button class="button-m button-ggg" disabled>NEXT STEP</button>
        </form>
      </div>
      <div class="static note tidbit"><p>Do you want to create a Smart Space instead?</p><p><a on-click="smartSpace:showIntro">Here is how it works.</a></p></div>
    </div>
    <div class="step step-2">
      <div class="inner">
        <h2>Pick a color</h2>
        <p>Color coding your space helps you <br>to spot it a lot easier when you need it.</p>
        <div class="color-chooser">
          <div class="space-color-option" data-hex="#75ffc0"><mm-circle style="--color:#75ffc0"></mm-circle></div>
          <div class="space-color-option" data-hex="#c0c2ce"><mm-circle style="--color:#c0c2ce"></mm-circle></div>
          <div class="space-color-option" data-hex="#ff8fa4"><mm-circle style="--color:#ff8fa4"></mm-circle></div>
          <div class="space-color-option" data-hex="#000"><mm-circle style="--color:#000"></mm-circle></div>
          <div class="space-color-option" data-hex="#19aad1"><mm-circle style="--color:#19aad1"></mm-circle></div>
          <div class="space-color-option" data-hex="#17c37b"><mm-circle style="--color:#17c37b"></mm-circle></div>
          <div class="space-color-option" data-hex="#ffdcd0"><mm-circle style="--color:#ffdcd0"></mm-circle></div>
          <div class="space-color-option" data-hex="#166ff4"><mm-circle style="--color:#166ff4"></mm-circle></div>
          <div class="space-color-option" data-hex="#cdff06"><mm-circle style="--color:#cdff06"></mm-circle></div>
          <div class="space-color-option" data-hex="#f96"><mm-circle style="--color:#f96"></mm-circle></div>
          <div class="space-color-option" data-hex="#bfb5d7"><mm-circle style="--color:#bfb5d7"></mm-circle></div>
          <div class="space-color-option" data-hex="#96cbd1"><mm-circle style="--color:#96cbd1"></mm-circle></div>
          <div class="space-color-option" data-hex="#cba0aa"><mm-circle style="--color:#cba0aa"></mm-circle></div>
          <div class="space-color-option" data-hex="#fdf06f"><mm-circle style="--color:#fdf06f"></mm-circle></div>
          <div class="space-color-option" data-hex="#7a30cf"><mm-circle style="--color:#7a30cf"></mm-circle></div>
          <div class="space-color-option" data-hex="#ef3e4a"><mm-circle style="--color:#ef3e4a"></mm-circle></div>
        </div>
        <button class="button-m button-orange" disabled>FINISH &amp; SAVE</button>
      </div>
    </div>
  </div>
</dialog>"##;

const TOM_INTRODUCTION_MODAL: &str = r#"<dialog class="tom-introduction-modal modal animate-up" escapable data-name="tom-introduction-modal" on-closed="modal:destroy">
  <div class="box">
    <button class="close big" on-click="modal:close"></button>
    <div class="lhs">
      <span>NEW FEATURE</span>
      <h3>Introducing<br>Top of Mind</h3>
      <p>Keep important cards at the top of your mind as reminders or inspiration. Just drag &amp; drop to pin them, and drag to remove.</p>
      <button class="orange" on-click="modal:close">GOT IT!</button>
    </div>
    <div class="rhs"></div>
  </div>
</dialog>"#;

/// Render an HTML partial for `/components/<name>`.
pub fn render_component(name: &str) -> String {
    match name {
        "NewSpaceModal" => NEW_SPACE_MODAL.to_owned(),
        "TomIntroductionModal" => TOM_INTRODUCTION_MODAL.to_owned(),
        "DeleteSpaceConfirmation" => r#"<dialog class="delete-space-confirmation modal" escapable data-name="delete-space-confirmation"><div class="box"><button class="close big" on-click="modal:close"></button><button class="delete button-danger">Delete</button><a class="cancel" on-click="modal:close">Cancel</a></div></dialog>"#.to_owned(),
        "EmptySpace" => r#"<div class="empty-space"><div class="middle"><p>This space is empty.</p></div><div class="bottom-circle"></div></div>"#.to_owned(),
        "EmptySpaceList" => r#"<div class="empty-space-list"><div class="top-circle"></div><div class="middle"><p>No spaces yet.</p></div><div class="bottom-circle"></div></div>"#.to_owned(),
        "OnboardingChecklist" => r#"<div class="onboarding-checklist"></div>"#.to_owned(),
        other => {
            let slug = escape_html(other);
            format!(r#"<dialog class="modal" escapable data-name="{slug}"><div class="box"><button class="close big" on-click="modal:close"></button></div></dialog>"#)
        }
    }
}

/// Render the `/serendipity` partial HTML for up to 18 cards from the library.
pub fn render_serendipity(rows: &[CardRow], localize: fn(&str) -> String) -> String {
    let wrap = |card_html: &str| {
        format!(
            r#"<div class="circle"><div class="card"><div class="card-rotate"><div class="card-anim">{card_html}<span class="button expand-button"><span class="icon"></span><span class="tooltip">Expand</span></span></div></div></div></div>"#
        )
    };
    let items: Vec<String> = if rows.is_empty() {
        vec![wrap("")]
    } else {
        rows.iter().take(18).map(|row| wrap(&localize(&row.html))).collect()
    };
    let c1 = items.iter().take(6).cloned().collect::<Vec<_>>().join("");
    let c2 = items.iter().skip(6).take(7).cloned().collect::<Vec<_>>().join("");
    let c3 = items.iter().skip(13).take(5).cloned().collect::<Vec<_>>().join("");
    format!(
        r#"<script id="sound-effects">{{"music":"/_static/134/0.26.0/serendipity/mp3/loop1.mp3;codec(aac)/96kbs.mp4","hover":"/_static/134/0.26.0/serendipity/mp3/hover.wav;codec(aac)/96kbs.mp4","movement":"/_static/134/0.26.0/serendipity/mp3/movement.wav;codec(aac)/96kbs.mp4"}}</script>
<section class="serendipity-outro serendipity-panel"><div class="steps"><div class="step stepOutro"><div class="sentences"><p class="big-text big-text-2">Peace of mind.</p></div><button on-click="serendipity:restart" class="button-step toRestart"><span>Do it again?</span></button></div></div><a class="back-link" href="/everything">Please take me back to my mind</a></section>
<div class="circles"><div class="circle-1"><div class="inner-circle">{c1}</div></div><div class="circle-2"><div class="inner-circle">{c2}</div></div><div class="circle-3"><div class="inner-circle">{c3}</div></div></div>
<div class="btns"><button class="btn forget-button" on-click="serendipity:forget"><span class="designedBtn"><span>Forget</span></span><span class="back-btn"></span></button><button class="btn keep-button" on-click="serendipity:keep"><span class="designedBtn"><span>Keep</span></span><span class="back-btn"></span></button><div class="progress" aria-hidden="true"><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div></div></div>
<section class="bottom-ui"><button class="toggle-sound-button"><span>Sound</span><span class="strokes" aria-hidden="true"><span></span><span></span><span></span><span></span></span></button></section>"#
    )
}
