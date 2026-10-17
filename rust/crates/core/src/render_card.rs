//! Port of `lib/render-card.mjs`: one grid card's server-style HTML.

use mm_js::Value;

use crate::render::access::{member, optional, text, to_int32};
use crate::render::encoding::{escape_html as e, escape_prose};
use crate::render::media::{image_cover, render_media, render_video};
use crate::render::prose::render_prose;
use crate::{Error, Result};

/// The CSS classes of a card type.
fn class_of(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "XPost" => "x-post post",
        "ThreadsPost" => "threads-post post",
        "SubstackNote" => "substack-note post",
        "WebPage" => "web-page",
        "Article" => "article",
        "Repository" => "repository",
        "Image" => "image",
        "Screenshot" => "screenshot",
        "Video" => "video",
        "YouTubeVideo" => "youtube-video video",
        "InstagramReel" => "instagram-reel",
        "Note" => "note content",
        "Content" => "content",
        "SoftwareApplication" => "software-application",
        "Product" => "product",
        "Document" => "document",
        "RedditPost" => "reddit-post",
        "Placeholder" => "placeholder unavailable",
        _ => return None,
    })
}

/// The type-specific label shown in the external-link bubble (`linkLabel`).
fn link_label(data: &Value, kind: &str) -> String {
    let label = match kind {
        "XPost" => "X Post",
        "ThreadsPost" => "Threads Post",
        "SubstackNote" => "Substack Note",
        "RedditPost" => "Reddit Post",
        "InstagramReel" => "Reel",
        "SoftwareApplication" => "App",
        "YouTubeVideo" => "YouTube Video",
        "Article" => "Article",
        "Repository" => "Repository",
        "Placeholder" => "Placeholder",
        _ => return text(data.get("siteName").or(data.get("domain"))),
    };
    label.into()
}

/// The inaccessible-site illustration and message.
fn inaccessible(data: &Value, placeholder: bool) -> String {
    let message = format!("It looks like {} is blocking us from displaying information.", e(&text(data.get("siteName").or(data.get("domain")))));
    let indent = if placeholder { "  " } else { "    " };
    let guts = format!(
        "<div class=\"inaccessible-guts\">\n{indent}<img src=\"https://static.accelerator.net/134/0.109.0/icons/sadness.svg\" width=\"30\" height=\"30\"/>\n{indent}<span class=\"big\">Oh my mind</span>\n"
    );
    if placeholder {
        format!("\n\n\n{guts}  \n  \n\n  \n  <span class=\"tiny\">{message}</span>\n  \n</div>\n\n")
    } else {
        format!("\n\n\n\n\n  \n\n  \n  {guts}    <span class=\"tiny\">{message}</span>\n  </div>\n  \n\n\n\n\n\n")
    }
}

/// `value.length` truthiness and items for a gallery, as `gallery.length ? gallery.map(...)` reads it.
fn gallery_items(gallery: &Value) -> Result<&[Value]> {
    match gallery {
        Value::Array(items) => Ok(items),
        Value::String(text) if !text.is_empty() => Err(Error::invalid("gallery.map is not a function")),
        Value::Object(object) if object.get("length").is_some_and(Value::truthy) => Err(Error::invalid("gallery.map is not a function")),
        _ => Ok(&[]),
    }
}

/// The social platform's original rich text and media inside its card frame (`renderPost`).
fn render_post(data: &Value, extras: &Value) -> Result<String> {
    let none = Value::Object(mm_js::Object::new());
    let post = extras.get("post").or(&none);
    let platform = match data.get("type").as_str() {
        Some("XPost") => "x",
        Some("ThreadsPost") => "threads",
        _ => "substack",
    };
    let body = if post.get("bodyHtml").is_nullish() {
        let description = match data.get("description") {
            Value::String(description) => description.as_str(),
            value if value.is_nullish() => "",
            _ => return Err(Error::invalid("description.split is not a function")),
        };
        let lines: Vec<String> = split_lines(description).into_iter().map(|line| escape_prose(&line)).collect();
        format!("<p>{}</p>", lines.join("<br/>"))
    } else {
        post.get("bodyHtml").to_js_string()
    };
    let gallery = gallery_items(post.get("gallery"))?;
    let cover = extras.get("cover");
    let media = if cover.truthy() { format!("\n    <p>{}</p>\n    ", render_media(cover)) } else { String::new() };
    let mut gallery_html = String::new();
    if !gallery.is_empty() {
        let mut images = String::new();
        for image in gallery {
            let source = if member(image, "srcset")?.truthy() {
                format!("data-srcset=\"{}\"", e(&text(image.get("srcset"))))
            } else {
                format!("data-src=\"{}\"", e(&text(image.get("src"))))
            };
            images.push_str(&format!("\n        <img class=\"lazy\" {source} width=\"{}\" height=\"{}\"/>\n        ", image.get("width").to_js_string(), image.get("height").to_js_string()));
        }
        let three = if gallery.len() == 3 { "gallery-grid--three" } else { "" };
        gallery_html = format!(
            "\n    <div class=\"gallery\">\n      <div class=\"gallery-wrap gallery-grid {three}\">\n        {images}\n      </div>\n    </div>\n    "
        );
    }
    let player = if post.get("hasVideo").truthy() && optional(cover, "type").as_str() != Some("video") {
        "      \n    <div class=\"fake-player\">\n      <div class=\"fp-button\">\n        <span class=\"fp-play\"></span>\n      </div>\n    </div>    "
    } else {
        ""
    };
    let author = post.get("author").or(data.get("author"));
    let author_html = if author.truthy() { format!("    <p class=\"author\">{}</p>\n    \n", e(&text(author))) } else { String::new() };
    Ok(format!(
        "<div class=\"inner\">\n  \n\n  \n  <div class=\"body {platform}\">\n    <i class=\"platform\"></i>\n\n    {body}\n\n    {media}\n\n    {gallery_html}\n\n    {player}{}\n    \n    \n{author_html}  </div>\n  \n</div>\n",
        if player.is_empty() { "" } else { "\n    " }
    ))
}

/// `description.split(/\r\n|\r|\n/)`.
fn split_lines(description: &str) -> Vec<String> {
    let mut lines = vec![String::new()];
    let mut chars = description.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                lines.push(String::new());
            }
            '\n' => lines.push(String::new()),
            other => lines.last_mut().expect("a line").push(other),
        }
    }
    lines
}

/// Select a card's content template; only absent source metadata comes from extras.
fn render_body(data: &Value, extras: &Value) -> Result<String> {
    let kind = data.get("type").as_str().unwrap_or("");
    let pending = extras.get("capturePending").truthy() && !extras.get("captureFailed").truthy();
    if kind == "Placeholder" && pending {
        return Ok("\n<div class=\"processing-guts\">\n  <img class=\"icon\" src=\"https://static.accelerator.net/134/0.109.0/icons/head_spin.png\" width=\"30\" height=\"30\"/>\n\n  <span class=\"big\">One moment.</span>\n  <span class=\"big\">I'm saving this for you.</span>\n</div>\n\n\n\n".into());
    }
    if kind == "WebPage" && extras.get("screenshotPending").truthy() {
        return Ok("\n\n\n\n\n  \n  <div class=\"processing-guts\">\n   <div class=\"spinner\"></div>\n   <span class=\"text\">Taking screenshot…</span>\n  </div>\n  \n\n  \n\n\n\n\n\n".into());
    }
    if pending {
        return Ok(format!("<div class=\"processing-guts\"><div class=\"spinner\"></div><div class=\"text\">{}</div></div>", e(&text(data.get("domain")))));
    }
    let cover = extras.get("cover");
    Ok(match kind {
        "Repository" => format!(
            "<div class=\"inner github\">\n  <i class=\"icon\"></i>\n  <div class=\"wrap\">    \n    <p>{}</p>\n    <div class=\"label\">{}</div>    \n  </div>\n</div>\n",
            e(&text(data.get("title"))), e(&text(data.get("siteName")))
        ),
        "Image" => {
            let built;
            let used = if cover.is_nullish() {
                built = image_cover(data.get("blob"))?;
                &built
            } else {
                cover
            };
            format!("{}\n", render_media(used))
        }
        "Screenshot" | "YouTubeVideo" => format!("{}\n", render_media(cover)),
        "Video" => format!("{}\n", render_video(data.get("blob"))?),
        "WebPage" => {
            if cover.truthy() {
                format!("\n\n\n{}\n\n\n\n\n\n\n\n", render_media(cover))
            } else {
                inaccessible(data, false)
            }
        }
        "Article" => {
            let heading = if extras.get("title").is_nullish() {
                let title = data.get("title");
                if title.truthy() { text(title) } else { "Untitled".into() }
            } else {
                text(extras.get("title"))
            };
            format!(
                "<div class=\"inner\">\n  <div class=\"article-card-top\">\n    <div class=\"logo\">\n      \n\n      <span class=\"default-logo\"></span>\n    </div>\n\n    <h3>{}</h3>\n  </div>\n\n  {}\n</div>\n",
                e(&heading), render_media(cover)
            )
        }
        "Note" => format!("\n\n{}", render_prose(&prose_of(data))?),
        "Content" => render_prose(&prose_of(data))?,
        "Document" => {
            let path = member(data.get("blob"), "path")?.to_js_string();
            format!(
                "\n\n<div class=\"document-background\">\n  <mymind-image style=\"aspect-ratio:642/840\">\n    <img class=\"lazy\" data-src=\"{}\" width=\"642\" height=\"840\"/>\n  </mymind-image>\n</div>\n\n\n\n\n",
                e(&format!("/media/{path};background(fff)/page(1)/642x840-t.webp"))
            )
        }
        "InstagramReel" => {
            let thumbnail = optional(cover, "isThumbnail");
            let is_thumbnail = if thumbnail.is_nullish() { to_int32(data.get("flags").to_number()) & 8_388_608 == 0 } else { thumbnail.truthy() };
            format!(
                "<div class=\"reel instagram-reel {}\">\n  <i class=\"icon\"></i>\n  <div class=\"body\">\n    \n    <img class=\"lazy main-img\" data-src=\"{}\" width=\"580\" height=\"580\"/>\n    <div class=\"icon-play\"></div>\n    \n    \n  </div>\n</div>\n",
                if is_thumbnail { "" } else { "not-thumbnail" },
                e(&text(optional(cover, "src")))
            )
        }
        "SoftwareApplication" => {
            let none = Value::Object(mm_js::Object::new());
            let app = extras.get("app").or(&none);
            let srcset = text(app.get("srcset")).replace('"', "&quot;");
            let size = |key: &str| app.get(key).or(&Value::Number(250.0)).to_js_string();
            format!(
                "<div class=\"inner distributor_{}\"> <!-- app-store or play-store classes -->\n  <img srcset=\"{srcset}\" width=\"{}\" height=\"{}\"/>\n</div>\n",
                e(&text(app.get("distributor"))), size("width"), size("height")
            )
        }
        "Product" => {
            let price = optional(extras.get("product"), "price");
            let price_html = if price.truthy() { format!("<span class=\"price\">{}</span>\n", e(&text(price))) } else { String::new() };
            format!("\n{price_html}\n\n{}\n\n\n", render_media(cover))
        }
        "Placeholder" => inaccessible(data, true),
        "RedditPost" => {
            let reddit = extras.get("reddit");
            let title = optional(reddit, "title").or(data.get("title"));
            format!(
                "<div class=\"reddit inner\">\n  <i class=\"icon\"></i>\n  <div class=\"body\">\n    <h3>{}</h3>\n    <div class=\"label\">{}</div>\n  </div>\n</div>\n",
                e(&text(title)), e(&text(optional(reddit, "subreddit")))
            )
        }
        "XPost" | "ThreadsPost" | "SubstackNote" => render_post(data, extras)?,
        _ => return Err(Error::invalid(format!("Unsupported card type: {}", data.get("type").to_js_string()))),
    })
}

/// `data.prose ?? { type: 'doc' }`.
fn prose_of(data: &Value) -> Value {
    let prose = data.get("prose");
    if prose.is_nullish() { mm_js::obj! { "type" => "doc" }.into() } else { prose.clone() }
}

/// Whether some tag is named `highlight` (`data.tags?.some(...)`).
fn has_highlight(data: &Value) -> Result<bool> {
    match data.get("tags") {
        Value::Array(tags) => {
            for tag in tags {
                if member(tag, "name")?.as_str() == Some("highlight") {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        value if value.is_nullish() => Ok(false),
        _ => Err(Error::invalid("data.tags.some is not a function")),
    }
}

/// Render one grid card without reading recordings or touching the network (`renderCard`).
///
/// The card JSON omits its ID and most scraped entity and media fields, so `extras` carries them: `id`, `linkUrl`,
/// `title`, `cover`, `post`, `app`, `product`, `reddit`, `capturePending`, `captureFailed` and `screenshotPending`,
/// as `lib/render-card.mjs` documents.
///
/// # Arguments
/// * `data` - Parsed card JSON with a supported `type` and that type's content fields.
/// * `extras` - Missing entity metadata; `undefined` stands for an empty object.
///
/// # Returns
/// Server-style grid HTML with the exact template whitespace and no trailing newline.
///
/// # Errors
/// The card type is unsupported, the object ID is missing, or a field has a shape the JS would throw on.
pub fn render_card(data: &Value, extras: &Value) -> Result<String> {
    let none = Value::Object(mm_js::Object::new());
    let extras = if extras.is_undefined() { &none } else { extras };
    let id = member(extras, "id")?;
    let id = if id.is_nullish() { member(data, "id")? } else { id };
    if !id.truthy() {
        return Err(Error::invalid("renderCard requires extras.id or data.id"));
    }
    let key = member(data, "type")?.to_js_string();
    let Some(class) = class_of(&key) else { return Err(Error::invalid(format!("Unsupported card type: {key}"))) };
    let kind = data.get("type").as_str().unwrap_or("");
    let deleted = data.get("deleted").truthy();
    let highlighted = has_highlight(data)?;
    let processing = extras.get("screenshotPending").truthy() || (extras.get("capturePending").truthy() && !extras.get("captureFailed").truthy());
    let base = if kind == "Placeholder" && processing { "placeholder" } else { class };
    let class_name = format!("{base}{}{}{}", if deleted { " deleted" } else { "" }, if highlighted { " highlighted" } else { "" }, if processing { " processing" } else { "" });
    let empty = Value::String(String::new());
    let url = extras.get("linkUrl").or(optional(data.get("source"), "url")).or(&empty);
    let linked = url.truthy() && !matches!(kind, "Image" | "Screenshot" | "Video" | "Note" | "Content" | "Document");
    let bubble = if linked && !deleted {
        format!("<a class=\"link-bubble\" href=\"{}\" on-click=\"card:link\" target=\"_blank\"><span>{}</span></a>", e(&text(url)), e(&link_label(data, &key)))
    } else {
        String::new()
    };
    let title = if extras.get("title").is_nullish() {
        let own = data.get("title");
        if own.truthy() {
            own
        } else if kind == "WebPage" {
            url
        } else {
            &empty
        }
    } else {
        extras.get("title")
    };
    let show_title = title.truthy() && (deleted || !matches!(kind, "Article" | "RedditPost" | "Placeholder"));
    let title_html = if show_title {
        let shown = if linked { format!("<a href=\"{}\" target=\"_blank\">{}</a>", e(&text(url)), e(&text(title))) } else { e(&text(title)) };
        format!("  <p class=\"title\">{shown}</p>\n  \n")
    } else {
        String::new()
    };
    Ok(format!(
        "<mymind-card class=\"{class_name}\" data-id=\"{}\">\n  <div class=\"hover\" on-click=\"card:{}\">{bubble}</div>\n  {}\n  \n{title_html}</mymind-card>",
        e(&text(id)),
        if deleted { "restore" } else { "open" },
        render_body(data, extras)?
    ))
}
