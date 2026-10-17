//! Port of the non-Chromium half of `lib/capture-link.mjs`: enriching a placeholder link card in committable stages.

use std::sync::LazyLock;

use mm_js::{Value, json, text, uri};
use regex::Regex;

use crate::capture_html::{PageMetadata, capture_title, detect_type, page_metadata, plain_text, plain_text_value, profile_image, sanitize_post, source_url};
use crate::capture_http::is_blocked;
use crate::host::{Media, MediaInput, MediaUser, Resource, ResourceOptions, Screenshot};
use crate::{Error, Result, arr, obj};

static NON_BINARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:$|(?i-u:text/|application/(?:xhtml\+xml|xml|json|ld\+json|pdf)))").expect("pattern"));
static APK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i-u:\.apk)$").expect("pattern"));
static SUBREDDIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/r/([^/]+)").expect("pattern"));
static BY_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?i-u:by)\s+").expect("pattern"));
static PRODUCT_TYPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|/)Product$").expect("pattern"));
static COVER_SIZE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";[0-9]+x[0-9]+-t\.webp$").expect("pattern"));

/// What [`capture_link`] needs besides the card (`options` of `captureLink`, without the transports).
pub struct CaptureOptions<'a> {
    /// Card id used for local media layout.
    pub id: &'a str,
    /// Account the media is stored for.
    pub user: &'a MediaUser,
    /// Media handle that stores originals and makes covers.
    pub media: &'a dyn Media,
    /// Commits a `{ data, extras }` patch in a fresh transaction; its error aborts the stage like a throw.
    pub update: &'a dyn Fn(Value) -> Result<()>,
    /// Allow loopback fixture reads only.
    pub offline: bool,
}

fn platform_embed(url: &str, kind: &str) -> Option<String> {
    let endpoint = match kind {
        "XPost" => "https://publish.twitter.com/oembed",
        "YouTubeVideo" => "https://www.youtube.com/oembed",
        "RedditPost" => "https://www.reddit.com/oembed",
        "InstagramReel" => "https://www.instagram.com/api/v1/oembed/",
        _ => return None,
    };
    let mut result = mm_js::url::parse(endpoint, None)?;
    result.query_pairs_mut().append_pair("url", url).append_pair("format", "json").append_pair("omit_script", "true");
    Some(result.into())
}

fn parsed(url: &str) -> Result<url::Url> {
    mm_js::url::parse(url, None).ok_or_else(|| Error::invalid("Invalid URL"))
}

fn first_truthy<'a>(values: &[&'a Value]) -> Option<&'a Value> {
    values.iter().copied().find(|value| value.truthy())
}

fn cover_of(prepared: &crate::host::PreparedMedia) -> Result<Value> {
    if prepared.extras.is_nullish() {
        return Err(Error::invalid("Cannot read properties of undefined"));
    }
    Ok(prepared.extras.get("cover").clone())
}

fn is_product(item: &Value) -> bool {
    let kinds = match item.get("@type") {
        Value::Array(items) => items.clone(),
        other => vec![other.clone()],
    };
    kinds.iter().any(|kind| PRODUCT_TYPE.is_match(&kind.to_js_string()))
}

fn author_name(embed: &Value, metadata: &PageMetadata) -> String {
    let named = embed.get("author_name");
    if !named.is_nullish() {
        return named.to_js_string();
    }
    let author = if metadata.author.type_of() == "object" { metadata.author.get("name") } else { &metadata.author };
    if author.is_nullish() { String::new() } else { author.to_js_string() }
}

async fn fetch_image(resource: &dyn Resource, options: &CaptureOptions<'_>, image: &str, extras: &mut Value, metadata: &PageMetadata) -> Result<()> {
    let fetched = resource.fetch(image, ResourceOptions { offline: options.offline, limit: Some(25 * 1024 * 1024) }).await?;
    let prepared = options.media.prepare(MediaInput::Bytes(fetched.bytes), &fetched.mime, "cover", options.user, options.id).await?;
    if prepared.data.get("type").as_str() != Some("Image") {
        return Ok(());
    }
    let cover = cover_of(&prepared)?;
    if extras.get("post").truthy() && metadata.images.len() > 1 && metadata.images.iter().any(|known| known == image)
        && let Some(Value::Array(gallery)) = extras.get_mut("post").and_then(|post| post.get_mut("gallery")) {
            gallery.push(cover.clone());
        }
    if !extras.get("cover").truthy() {
        extras.set("cover", cover);
    }
    (options.update)(obj! { "data" => obj! {}, "extras" => extras.clone() }.into())
}

/// Enrich a committed URL placeholder in independently committable metadata and media stages (`captureLink`).
///
/// # Arguments
/// * `card` - The saved card JSON as the caller now holds it, with `source.url` and an optional requested `Screenshot`
///   type; the failure placeholder reads it again after the stages' updates.
/// * `options` - Card id, user, media handle, the `update` committer and the offline flag.
/// * `resource` - Validated transport for the page, oEmbed and image requests.
/// * `screenshot` - Page screenshot implementation.
///
/// # Errors
/// Only a `card` without a usable `source.url`, or an `update` failure while writing the failure placeholder; every
/// capture failure is committed as a placeholder instead.
pub async fn capture_link(card: &dyn Fn() -> Value, options: &CaptureOptions<'_>, resource: &dyn Resource, screenshot: &dyn Screenshot) -> Result<()> {
    let saved = card();
    let Some(url) = saved.get("source").get("url").as_str().map(str::to_owned) else {
        return Err(Error::invalid("Invalid URL"));
    };
    if stages(&saved, &url, options, resource, screenshot).await.is_err() {
        let card = card();
        let pick = |value: &Value, fallback: &dyn Fn(&url::Url) -> String| -> Result<Value> {
            if value.truthy() { Ok(value.clone()) } else { Ok(Value::String(fallback(&parsed(&url)?))) }
        };
        let data = obj! {
            "type" => "Placeholder",
            "title" => pick(card.get("title"), &mm_js::url::host)?,
            "domain" => pick(card.get("domain"), &|parsed| parsed.host_str().unwrap_or("").to_owned())?,
            "siteName" => pick(card.get("siteName"), &|parsed| parsed.host_str().unwrap_or("").to_owned())?,
        };
        let extras = obj! { "linkUrl" => url.as_str(), "cover" => Value::Undefined, "post" => Value::Undefined, "captureFailed" => true };
        (options.update)(obj! { "data" => data, "extras" => extras }.into())?;
    }
    Ok(())
}

async fn stages(card: &Value, url: &str, options: &CaptureOptions<'_>, resource: &dyn Resource, screenshot: &dyn Screenshot) -> Result<()> {
    let offline = options.offline;
    let response = resource.fetch(url, ResourceOptions { offline, limit: Some(64 * 1024 * 1024) }).await?;
    if detect_type(&response.url, &Value::Object(obj! {}), &response.mime)? == "Document" {
        let source = parsed(url)?;
        let title = card.get("title");
        let name = if title.as_str() == Some(mm_js::url::host(&source).as_str()) {
            let last = source.path().split('/').next_back().unwrap_or("");
            uri::decode_uri_component(last).map_err(|_| Error::internal("URI malformed"))?
        } else if title.is_nullish() {
            // Every media stores `String(name ?? '')`.
            String::new()
        } else {
            title.to_js_string()
        };
        let prepared = options.media.prepare(MediaInput::Bytes(response.bytes), "application/pdf", &name, options.user, options.id).await?;
        return (options.update)(prepared.to_value());
    }
    let final_url = parsed(&response.url)?;
    let binary = !NON_BINARY.is_match(&response.mime) || APK.is_match(final_url.path());
    let html = if binary { String::new() } else { text::decode_utf8(&response.bytes) };
    let metadata = page_metadata(&html, &response.url)?;
    let metadata_value = metadata.to_value();
    let kind = if card.get("type").as_str() == Some("Screenshot") { "Screenshot" } else { detect_type(&metadata.canonical, &metadata_value, &response.mime)? };

    let mut embed = Value::Object(obj! {});
    let embed_url = metadata.oembed.clone().or_else(|| platform_embed(&metadata.canonical, kind)).filter(|found| !found.is_empty());
    if let Some(embed_url) = embed_url {
        match resource.fetch(&embed_url, ResourceOptions { offline, limit: Some(1024 * 1024) }).await {
            Ok(fetched) => {
                if let Ok(value) = json::parse(&text::decode_utf8(&fetched.bytes)) {
                    embed = value;
                }
            }
            Err(error) if is_blocked(&error) => return Err(error),
            Err(_) => {}
        }
    }
    if matches!(embed, Value::Null) {
        return Err(Error::invalid("Cannot read properties of null"));
    }

    let post = matches!(kind, "XPost" | "ThreadsPost" | "SubstackNote");
    let body_html = if post {
        let html = if embed.get("html").truthy() {
            embed.get("html").to_js_string()
        } else {
            let Value::String(description) = &metadata.description else { return Err(Error::invalid("replaceAll is not a function")) };
            format!("<p>{}</p>", description.replace('&', "&amp;").replace('<', "&lt;"))
        };
        sanitize_post(&html, &response.url)
    } else {
        String::new()
    };
    let title_source = match first_truthy(&[&metadata.title, embed.get("title"), card.get("title")]) {
        Some(value) => value.to_js_string(),
        None => mm_js::url::host(&parsed(url)?),
    };
    let title = capture_title(&title_source, kind, &metadata_value)?;
    let description = if metadata.description.truthy() {
        metadata.description.to_js_string()
    } else if !plain_text(&body_html).is_empty() {
        plain_text(&body_html)
    } else {
        plain_text_value(embed.get("html"))
    };
    let display_name = author_name(&embed, &metadata);
    let author = if post && !display_name.is_empty() { format!("by {}", BY_PREFIX.replace(&display_name, "")) } else { display_name };
    let canonical = parsed(&metadata.canonical)?;
    let canonical_host = canonical.host_str().unwrap_or("").to_owned();

    let mut data = obj! { "type" => kind };
    if !binary {
        data.insert("title", text::slice(&title, 0, Some(200)));
    }
    data.insert("description", description.as_str());
    data.insert("domain", canonical_host.as_str());
    data.insert("siteName", if binary { canonical_host.clone() } else { metadata.site_name.clone() });
    if !author.is_empty() {
        data.insert("author", author.as_str());
    }
    let mut data = Value::Object(data);
    let mut extras = obj! {};
    if !binary {
        extras.insert("title", title.as_str());
    }
    extras.insert("linkUrl", metadata.canonical.as_str());
    extras.insert("captureText", if post { plain_text(&body_html) } else { metadata.text.clone() });
    if post {
        extras.insert("capturedTitle", title.as_str());
    }
    let mut extras = Value::Object(extras);
    if kind == "Article" {
        let text = if metadata.text.is_empty() { description.as_str() } else { metadata.text.as_str() };
        data.set("prose", obj! { "type" => "doc", "content" => arr![obj! { "type" => "paragraph", "content" => arr![obj! { "type" => "text", "text" => text }] }] });
    }
    if post {
        let video = embed.get("type").as_str() == Some("video");
        extras.set("post", obj! { "bodyHtml" => body_html.as_str(), "author" => author.as_str(), "hasVideo" => video, "gallery" => arr![] });
    }
    if kind == "RedditPost" {
        let reddit_title = if embed.get("title").is_nullish() { Value::String(title.clone()) } else { embed.get("title").clone() };
        let subreddit = SUBREDDIT.captures(&metadata.canonical).map_or("", |found| found.get(1).map_or("", |part| part.as_str()));
        extras.set("reddit", obj! { "title" => reddit_title, "subreddit" => subreddit });
    }
    if kind == "Product" {
        let empty = Value::Object(obj! {});
        let product = metadata.entities.iter().find(|item| is_product(item)).unwrap_or(&empty);
        let offer = match product.get("offers") {
            Value::Array(items) => items.first().cloned().unwrap_or(Value::Undefined),
            other => other.clone(),
        };
        let offer = if offer.is_nullish() { empty.clone() } else { offer };
        let price = if offer.get("price").is_undefined() {
            String::new()
        } else {
            [offer.get("priceCurrency"), offer.get("price")]
                .iter()
                .filter(|value| !value.is_undefined())
                .map(|value| if value.is_nullish() { String::new() } else { value.to_js_string() })
                .collect::<Vec<_>>()
                .join(" ")
        };
        extras.set("product", obj! { "price" => price });
        let brand = product.get("brand");
        if brand.truthy() {
            let name = brand.get("name");
            data.set("brand", if name.is_nullish() { brand.clone() } else { name.clone() });
        }
    }
    if kind == "SoftwareApplication" {
        let distributor = if canonical_host == "apps.apple.com" { "app-store" } else { "play-store" };
        extras.set("app", obj! { "distributor" => distributor, "srcset" => "", "width" => 250, "height" => 250 });
    }

    let mut author_images = metadata.author_images.clone();
    for key in ["author_avatar_url", "author_image_url", "avatar_url"] {
        author_images.extend(source_url(embed.get(key), &response.url));
    }
    let mut images: Vec<String> = Vec::new();
    for image in metadata.images.iter().cloned().chain(source_url(embed.get("thumbnail_url"), &response.url)) {
        if !image.is_empty() && !images.contains(&image) {
            images.push(image);
        }
    }
    if post {
        let mut kept = Vec::new();
        for image in images {
            if !profile_image(&image)? && !author_images.contains(&image) {
                kept.push(image);
            }
        }
        images = kept;
    }
    if kind == "WebPage" && images.is_empty() {
        extras.set("screenshotPending", true);
    }
    (options.update)(obj! { "data" => data.clone(), "extras" => extras.clone() }.into())?;

    for image in images.iter().take(6) {
        if let Err(error) = fetch_image(resource, options, image, &mut extras, &metadata).await
            && is_blocked(&error) {
                return Err(error);
            }
    }
    if (kind == "WebPage" && !extras.get("cover").truthy()) || kind == "Screenshot" {
        if kind == "WebPage" && !extras.get("screenshotPending").truthy() {
            extras.set("screenshotPending", true);
            (options.update)(obj! { "data" => obj! {}, "extras" => obj! { "screenshotPending" => true } }.into())?;
        }
        let shot = async {
            let bytes = screenshot.capture(&response.url, offline).await?;
            let prepared = options.media.prepare(MediaInput::Bytes(bytes), "image/png", "screenshot", options.user, options.id).await?;
            cover_of(&prepared)
        };
        match shot.await {
            Ok(cover) => extras.set("cover", cover),
            Err(error) if is_blocked(&error) => return Err(error),
            Err(_) => {
                if kind == "WebPage" && !extras.get("captureText").truthy() && description.is_empty() {
                    (options.update)(obj! { "data" => obj! {}, "extras" => obj! { "screenshotFailed" => true } }.into())?;
                    return Ok(());
                }
            }
        }
    }
    if kind == "SoftwareApplication" && extras.get("cover").truthy() {
        let Value::String(src) = extras.get("cover").get("src") else { return Err(Error::invalid("replace is not a function")) };
        let srcset = format!("{} 1x", COVER_SIZE.replace(src, ";250x250-c.webp"));
        let mut app = extras.get("app").clone();
        app.set("srcset", srcset);
        app.set("width", 250);
        app.set("height", 250);
        extras.set("app", app);
    }
    let gallery_len = extras.get("post").get("gallery").as_array().map_or(0, Vec::len);
    if gallery_len > 1 {
        extras.set("cover", Value::Undefined);
    } else if extras.get("post").truthy()
        && let Some(post) = extras.get_mut("post") {
            post.set("gallery", arr![]);
        }
    if kind == "InstagramReel" && extras.get("cover").truthy()
        && let Some(cover) = extras.get_mut("cover") {
            cover.set("isThumbnail", true);
        }
    if kind == "Screenshot" && !extras.get("cover").truthy() {
        data.set("type", "WebPage");
    }
    (options.update)(obj! { "data" => data, "extras" => extras }.into())
}
