//! What a saved link's own site says beyond its page, read once the page is captured ([`capture_link`]): a Reddit
//! post's words and top comments from Reddit's public JSON, a YouTube video's length from its watch page, Bluesky and
//! Mastodon posts from their public APIs, and the article an X post carries from X's embed data.
//!
//! It runs as its own stage after the capture so the capture's stages stay as they were. A site that does not answer,
//! or answers with something else, leaves the card as the capture left it.
//!
//! [`capture_link`]: crate::capture_link::capture_link

use std::sync::LazyLock;

use mm_js::{Value, json, text};
use regex::Regex;

use crate::capture_html::{plain_text, sanitize_post};
use crate::capture_link::CaptureOptions;
use crate::host::{MediaInput, Resource, ResourceOptions};
use crate::render::encoding::escape_html;
use crate::{Result, arr, obj};

static REDDIT_POST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://(?:[a-z]+\.)?reddit\.com(/r/[^/?#]+/comments/[^/?#]+)").expect("pattern"));
static YOUTUBE_LENGTH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""lengthSeconds":"([0-9]{1,6})""#).expect("pattern"));
static MARKDOWN_LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\([^)\s]+\)").expect("pattern"));
static BLANK_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n\s*\n").expect("pattern"));
static BLUESKY_POST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://(?:www\.)?bsky\.app/profile/([A-Za-z0-9._:-]+)/post/([A-Za-z0-9]+)").expect("pattern"));
static MASTODON_STATUS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https://([A-Za-z0-9.-]+\.[A-Za-z]{2,})/@[A-Za-z0-9_]+(?:@[A-Za-z0-9.-]+)?/([0-9]{1,20})/?(?:[?#]|$)").expect("pattern"));
static X_STATUS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://(?:www\.|mobile\.)?(?:x|twitter)\.com/[A-Za-z0-9_]+/status(?:es)?/([0-9]{1,20})").expect("pattern"));
static EMOJI_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":[A-Za-z0-9_]+:").expect("pattern"));
static EMPTY_PARAGRAPH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<p>\s*</p>").expect("pattern"));

/// How many of a Reddit post's top comments a card keeps.
pub const REDDIT_COMMENTS: usize = 3;

/// Longest comment a card keeps, in characters; a longer one is cut at a word with an ellipsis.
const COMMENT_CHARS: usize = 600;

/// How many of a Bluesky or Mastodon post's pictures a card keeps, as the networks show at most four.
const POST_IMAGES: usize = 4;

/// A Bluesky or Mastodon post as its network's public API gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct SocialPost {
    /// The card type: `BlueskyPost` or `MastodonPost`.
    pub kind: &'static str,
    /// The author's display name, or their handle when they have none.
    pub name: String,
    /// The post's words as [`sanitize_post`] leaves them.
    pub html: String,
    /// Its pictures, a video's poster or a linked page's picture, at most [`POST_IMAGES`].
    pub images: Vec<String>,
    /// Whether it holds a video.
    pub video: bool,
}

/// Read what a captured card's own site adds and commit it as one more `{ data, extras }` patch.
///
/// # Arguments
/// * `card` - The card as the capture left it: `{ data, extras }`.
/// * `options` - The capture's options; only `update` and `offline` are used.
/// * `resource` - Fetches public URLs under the capture's destination policy.
///
/// # Errors
/// Only the error `update` returns. A site that fails or whose address is refused commits nothing.
pub async fn capture_more(card: &Value, options: &CaptureOptions<'_>, resource: &dyn Resource) -> Result<()> {
    let data = card.get("data");
    let extras = card.get("extras");
    let link = extras.get("linkUrl").as_str().or_else(|| data.get("source").get("url").as_str()).unwrap_or("");
    let offline = options.offline;
    let patch = match data.get("type").as_str() {
        Some("RedditPost") => reddit(link, extras, offline, resource).await.map(extras_patch),
        Some("YouTubeVideo") => youtube_length(link, offline, resource).await.map(extras_patch),
        Some("XPost") => x_article(link, extras, options, resource).await,
        Some("WebPage" | "Article") => match social_post(link, offline, resource).await {
            Some(post) => Some(social_patch(post, data, options, resource).await),
            None => None,
        },
        _ => None,
    };
    match patch {
        Some(patch) => (options.update)(patch),
        None => Ok(()),
    }
}

/// The address of a Reddit post's public JSON, with the top comments first.
///
/// # Arguments
/// * `link` - The post's address, such as `https://www.reddit.com/r/hiking/comments/1abc/the_longest_walk/`.
///
/// # Returns
/// Such as `https://www.reddit.com/r/hiking/comments/1abc.json?raw_json=1&sort=top&limit=8&depth=1`, or `None` for an
/// address that is not a Reddit post.
pub fn reddit_json_url(link: &str) -> Option<String> {
    let path = REDDIT_POST.captures(link)?.get(1)?.as_str();
    Some(format!("https://www.reddit.com{path}.json?raw_json=1&sort=top&limit=8&depth=1"))
}

/// The words and top comments Reddit's JSON gives for a post, as the card keeps them.
///
/// # Arguments
/// * `listing` - Reddit's answer: the post's listing, then its comments' listing.
/// * `reddit` - The card's `extras.reddit`, whose title and subreddit stay.
///
/// # Returns
/// `{ reddit, post? }`: `reddit` with up to [`REDDIT_COMMENTS`] comments as `{ author, body }`, skipping pinned,
/// moderators' and removed ones; `post` with the post's own words as paragraphs, when it has words. `None` when the
/// answer is not a post.
pub fn reddit_extras(listing: &Value, reddit: &Value) -> Option<Value> {
    let post = listing.at(0).get("data").get("children").at(0).get("data");
    if !post.is_object() {
        return None;
    }
    let mut comments = Vec::new();
    for child in listing.at(1).get("data").get("children").as_array().map(Vec::as_slice).unwrap_or_default() {
        let comment = child.get("data");
        let body = comment.get("body").as_str().unwrap_or("").trim();
        let author = comment.get("author").as_str().unwrap_or("");
        let removed = matches!(body, "" | "[deleted]" | "[removed]") || matches!(author, "" | "[deleted]" | "AutoModerator");
        if child.get("kind").as_str() != Some("t1") || removed || comment.get("stickied").truthy() || comment.get("distinguished").truthy() {
            continue;
        }
        comments.push(Value::from(obj! { "author" => author, "body" => shortened(&plain_markdown(body), COMMENT_CHARS) }));
        if comments.len() == REDDIT_COMMENTS {
            break;
        }
    }
    let title = if reddit.get("title").is_nullish() { post.get("title").clone() } else { reddit.get("title").clone() };
    let subreddit = if reddit.get("subreddit").truthy() { reddit.get("subreddit").clone() } else { post.get("subreddit").clone() };
    let mut extras = obj! { "reddit" => obj! { "title" => title, "subreddit" => subreddit, "comments" => Value::Array(comments) } };
    let words = plain_markdown(post.get("selftext").as_str().unwrap_or("").trim());
    if !words.is_empty() {
        let author = post.get("author").as_str().map(|name| format!("by u/{name}")).unwrap_or_default();
        extras.insert(
            "post",
            obj! { "bodyHtml" => paragraphs_html(&words), "author" => author, "hasVideo" => post.get("is_video").truthy(), "gallery" => arr![] },
        );
    }
    Some(extras.into())
}

/// A Bluesky post read from `app.bsky.feed.getPosts`.
///
/// # Arguments
/// * `answer` - Bluesky's answer, `{ posts: [...] }`.
/// * `rkey` - The post's key, the last part of its address, which the answer must be for.
///
/// # Returns
/// The post with its links, mentions and tags as links, or `None` when the answer holds no such post.
pub fn bluesky_post(answer: &Value, rkey: &str) -> Option<SocialPost> {
    let post = answer.get("posts").at(0);
    let uri = post.get("uri").as_str()?;
    let record = post.get("record");
    let words = record.get("text").as_str()?;
    if uri.rsplit('/').next() != Some(rkey) {
        return None;
    }
    let author = post.get("author");
    let handle = author.get("handle").as_str().unwrap_or("");
    let name = author.get("displayName").as_str().map(str::trim).filter(|name| !name.is_empty()).unwrap_or(handle);
    let embed = post.get("embed");
    let media = if embed.get("media").is_object() { embed.get("media") } else { embed };
    let mut images: Vec<String> = Vec::new();
    for image in media.get("images").as_array().or_else(|| media.get("items").as_array()).map(Vec::as_slice).unwrap_or_default() {
        images.extend(image.get("fullsize").as_str().or_else(|| image.get("thumb").as_str()).or_else(|| image.get("thumbnail").as_str()).map(str::to_owned));
    }
    let video = media.get("$type").as_str() == Some("app.bsky.embed.video#view");
    if video {
        images.extend(media.get("thumbnail").as_str().map(str::to_owned));
    }
    images.extend(media.get("external").get("thumb").as_str().map(str::to_owned));
    images.truncate(POST_IMAGES);
    let html = sanitize_post(&paragraphs(&bluesky_words(words, record.get("facets"))), "https://bsky.app/");
    Some(SocialPost { kind: "BlueskyPost", name: name.to_owned(), html, images, video })
}

/// A Mastodon post read from `/api/v1/statuses/{id}`.
///
/// # Arguments
/// * `status` - The server's answer.
/// * `id` - The post's id, which the answer must be for.
/// * `base` - The post's address, for relative links.
///
/// # Returns
/// The post, its content warning first, or `None` when the answer is not that post.
pub fn mastodon_post(status: &Value, id: &str, base: &str) -> Option<SocialPost> {
    let content = status.get("content").as_str()?;
    if status.get("id").as_str() != Some(id) {
        return None;
    }
    let account = status.get("account");
    let display = EMOJI_CODE.replace_all(account.get("display_name").as_str().unwrap_or(""), "").split_whitespace().collect::<Vec<_>>().join(" ");
    let name = if display.is_empty() { account.get("username").as_str().unwrap_or("").to_owned() } else { display };
    let warning = status.get("spoiler_text").as_str().unwrap_or("").trim();
    let warning = if warning.is_empty() { String::new() } else { format!("<p>{}</p>", escape_html(warning)) };
    let mut images = Vec::new();
    let mut video = false;
    for attachment in status.get("media_attachments").as_array().map(Vec::as_slice).unwrap_or_default() {
        let picture = match attachment.get("type").as_str() {
            Some("image") => attachment.get("url").as_str(),
            Some("video" | "gifv") => {
                video = true;
                attachment.get("preview_url").as_str()
            }
            _ => None,
        };
        images.extend(picture.map(str::to_owned));
    }
    if images.is_empty() {
        images.extend(status.get("card").get("image").as_str().map(str::to_owned));
    }
    images.truncate(POST_IMAGES);
    Some(SocialPost { kind: "MastodonPost", name, html: sanitize_post(&format!("{warning}{content}"), base), images, video })
}

/// The article an X post carries, from X's embed data for the post.
///
/// # Arguments
/// * `tweet` - X's answer from `cdn.syndication.twimg.com/tweet-result`.
///
/// # Returns
/// `{ title, preview, url, image? }`: the article's title, its opening lines, its address on X and its cover
/// picture's address; `None` when the post carries no article.
pub fn x_article_extras(tweet: &Value) -> Option<Value> {
    let article = tweet.get("article");
    let title = article.get("title").as_str().map(str::trim).filter(|title| !title.is_empty())?;
    let id = article.get("rest_id").as_str().filter(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))?;
    let preview = article.get("preview_text").as_str().unwrap_or("").trim();
    let mut extras = obj! { "title" => title, "preview" => preview, "url" => format!("https://x.com/i/article/{id}") };
    if let Some(image) = article.get("cover_media").get("media_info").get("original_img_url").as_str() {
        extras.insert("image", image);
    }
    Some(extras.into())
}

/// The `token` X's embed data asks for: the post's id over 10^15 times π, written in base 36 as JavaScript writes
/// it, without its zeros and its point, as X's own embeds make it.
///
/// # Arguments
/// * `id` - The post's id.
///
/// # Returns
/// Such as `4vgynb6ldq` for `2010751592346030461`.
pub fn syndication_token(id: u64) -> String {
    js_base36(id as f64 / 1e15 * std::f64::consts::PI).replace(['0', '.'], "")
}

/// A video's length on YouTube's watch page.
///
/// # Arguments
/// * `page` - The watch page's HTML.
///
/// # Returns
/// The length in seconds, or `None` when the page does not say.
pub fn youtube_seconds(page: &str) -> Option<u32> {
    YOUTUBE_LENGTH.captures(page)?.get(1)?.as_str().parse().ok().filter(|seconds| *seconds > 0)
}

async fn reddit(link: &str, extras: &Value, offline: bool, resource: &dyn Resource) -> Option<Value> {
    let body = fetch_text(&reddit_json_url(link)?, 2 * 1024 * 1024, offline, resource).await?;
    reddit_extras(&json::parse(&body).ok()?, extras.get("reddit"))
}

async fn youtube_length(link: &str, offline: bool, resource: &dyn Resource) -> Option<Value> {
    if link.is_empty() {
        return None;
    }
    let page = fetch_text(link, 8 * 1024 * 1024, offline, resource).await?;
    youtube_seconds(&page).map(|seconds| obj! { "videoLength" => seconds }.into())
}

async fn social_post(link: &str, offline: bool, resource: &dyn Resource) -> Option<SocialPost> {
    if let Some(found) = BLUESKY_POST.captures(link) {
        let (actor, rkey) = (&found[1], &found[2]);
        let did = if actor.starts_with("did:") {
            actor.to_owned()
        } else {
            let url = api_url("https://public.api.bsky.app/xrpc/com.atproto.identity.resolveHandle", "handle", actor);
            let answer = json::parse(&fetch_text(&url, 64 * 1024, offline, resource).await?).ok()?;
            answer.get("did").as_str().filter(|did| did.starts_with("did:"))?.to_owned()
        };
        let url = api_url("https://public.api.bsky.app/xrpc/app.bsky.feed.getPosts", "uris", &format!("at://{did}/app.bsky.feed.post/{rkey}"));
        return bluesky_post(&json::parse(&fetch_text(&url, 1024 * 1024, offline, resource).await?).ok()?, rkey);
    }
    let found = MASTODON_STATUS.captures(link)?;
    let (host, id) = (&found[1], &found[2]);
    let body = fetch_text(&format!("https://{host}/api/v1/statuses/{id}"), 1024 * 1024, offline, resource).await?;
    mastodon_post(&json::parse(&body).ok()?, id, link)
}

// The card as its network shows the post: its type, words and author, and its pictures in place of the page's.
async fn social_patch(post: SocialPost, data: &Value, options: &CaptureOptions<'_>, resource: &dyn Resource) -> Value {
    let mut covers = Vec::new();
    for image in &post.images {
        covers.extend(prepared_image(image, options, resource).await);
    }
    let words = plain_text(&post.html);
    let author = format!("by {}", post.name);
    let gallery = if covers.len() > 1 { covers.clone() } else { Vec::new() };
    let mut extras = obj! {
        "post" => obj! { "bodyHtml" => post.html.as_str(), "author" => author.as_str(), "hasVideo" => post.video, "gallery" => gallery },
        "captureText" => words.as_str(),
        "capturedTitle" => data.get("title").clone(),
    };
    match covers.as_slice() {
        [cover] => extras.insert("cover", cover.clone()),
        [] if !post.images.is_empty() => {}
        _ => extras.insert("cover", Value::Undefined),
    }
    obj! { "data" => obj! { "type" => post.kind, "author" => author, "description" => words }, "extras" => extras }.into()
}

async fn x_article(link: &str, extras: &Value, options: &CaptureOptions<'_>, resource: &dyn Resource) -> Option<Value> {
    let id: u64 = X_STATUS.captures(link)?[1].parse().ok()?;
    let url = format!("https://cdn.syndication.twimg.com/tweet-result?id={id}&lang=en&token={}", syndication_token(id));
    let tweet = json::parse(&fetch_text(&url, 1024 * 1024, options.offline, resource).await?).ok()?;
    let mut article = x_article_extras(&tweet)?;
    let title = article.get("title").to_js_string();
    let preview = article.get("preview").to_js_string();
    let image = article.remove("image");
    let mut patch = obj! {
        "title" => title.as_str(),
        "capturedTitle" => title.as_str(),
        "captureText" => format!("{title}\n{preview}").trim().to_owned(),
        "article" => article,
    };
    let mut post = extras.get("post").clone();
    if let Some(body) = post.get("bodyHtml").as_str() {
        let links = tweet.get("entities").get("urls").as_array().map(Vec::as_slice).unwrap_or_default();
        let mut words = body.to_owned();
        for found in links.iter().filter(|found| found.get("expanded_url").as_str().is_some_and(|url| url.contains("/i/article/"))) {
            if let Some(short) = found.get("url").as_str() {
                let short = escape_html(short);
                words = words.replace(&format!("<a href=\"{short}\">{short}</a>"), "");
            }
        }
        post.set("bodyHtml", EMPTY_PARAGRAPH.replace_all(&words, "").into_owned());
        patch.insert("post", post);
    }
    if let Some(image) = image.as_ref().and_then(Value::as_str)
        && let Some(cover) = prepared_image(image, options, resource).await {
            patch.insert("cover", cover);
        }
    let data = obj! { "title" => text::slice(&title, 0, Some(200)), "description" => preview };
    Some(obj! { "data" => data, "extras" => patch }.into())
}

// A picture fetched and stored as a cover; `None` when it cannot be, its address refused included.
async fn prepared_image(url: &str, options: &CaptureOptions<'_>, resource: &dyn Resource) -> Option<Value> {
    let fetched = resource.fetch(url, ResourceOptions { offline: options.offline, limit: Some(25 * 1024 * 1024) }).await.ok()?;
    if !(200..300).contains(&fetched.status) {
        return None;
    }
    let prepared = options.media.prepare(MediaInput::Bytes(fetched.bytes), &fetched.mime, "cover", options.user, options.id).await.ok()?;
    let cover = prepared.extras.get("cover");
    (prepared.data.get("type").as_str() == Some("Image") && cover.truthy()).then(|| cover.clone())
}

fn extras_patch(extras: Value) -> Value {
    obj! { "data" => obj! {}, "extras" => extras }.into()
}

fn api_url(endpoint: &str, key: &str, value: &str) -> String {
    let mut url = mm_js::url::parse(endpoint, None).expect("endpoint");
    url.query_pairs_mut().append_pair(key, value);
    url.into()
}

// A Bluesky post's text, escaped, with its facets as links; facets count UTF-8 bytes.
fn bluesky_words(words: &str, facets: &Value) -> String {
    let mut links: Vec<(usize, usize, String)> = facets
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|facet| {
            let start = facet.get("index").get("byteStart").as_f64()? as usize;
            let end = facet.get("index").get("byteEnd").as_f64()? as usize;
            let feature = facet.get("features").at(0);
            let href = match feature.get("$type").as_str()? {
                "app.bsky.richtext.facet#link" => feature.get("uri").as_str()?.to_owned(),
                "app.bsky.richtext.facet#mention" => format!("https://bsky.app/profile/{}", feature.get("did").as_str()?),
                "app.bsky.richtext.facet#tag" => format!("https://bsky.app/hashtag/{}", feature.get("tag").as_str()?),
                _ => return None,
            };
            let fits = start < end && end <= words.len() && words.is_char_boundary(start) && words.is_char_boundary(end);
            fits.then_some((start, end, href))
        })
        .collect();
    links.sort_by_key(|(start, _, _)| *start);
    let mut html = String::new();
    let mut at = 0;
    for (start, end, href) in links {
        if start < at {
            continue;
        }
        html.push_str(&escape(&words[at..start]));
        html.push_str(&format!("<a href=\"{}\">{}</a>", escape(&href).replace('"', "&quot;"), escape(&words[start..end])));
        at = end;
    }
    html.push_str(&escape(&words[at..]));
    html
}

// `value.toString(36)` for a finite number of at least zero, digit for digit as V8 writes it: as many fraction digits
// as tell the number apart from its neighbours, the last one rounded.
fn js_base36(value: f64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut integer = value.floor();
    let mut fraction = value - integer;
    let mut delta = (0.5 * (f64::from_bits(value.to_bits() + 1) - value)).max(f64::from_bits(1));
    let mut decimals: Vec<u8> = Vec::new();
    if fraction >= delta {
        loop {
            fraction *= 36.0;
            delta *= 36.0;
            let digit = fraction as usize;
            decimals.push(DIGITS[digit]);
            fraction -= digit as f64;
            if (fraction > 0.5 || (fraction == 0.5 && digit % 2 == 1)) && fraction + delta > 1.0 {
                loop {
                    let Some(last) = decimals.pop() else {
                        integer += 1.0;
                        break;
                    };
                    let place = DIGITS.iter().position(|known| *known == last).unwrap_or(0);
                    if place + 1 < 36 {
                        decimals.push(DIGITS[place + 1]);
                        break;
                    }
                }
                break;
            }
            if fraction < delta {
                break;
            }
        }
    }
    let mut whole = Vec::new();
    loop {
        let remainder = integer % 36.0;
        whole.push(DIGITS[remainder as usize]);
        integer = (integer - remainder) / 36.0;
        if integer <= 0.0 {
            break;
        }
    }
    whole.reverse();
    let mut written = String::from_utf8(whole).unwrap_or_default();
    if !decimals.is_empty() {
        written.push('.');
        written.push_str(&String::from_utf8(decimals).unwrap_or_default());
    }
    written
}

// A public page's text; `None` when it cannot be read, its address refused included.
async fn fetch_text(url: &str, limit: u64, offline: bool, resource: &dyn Resource) -> Option<String> {
    let fetched = resource.fetch(url, ResourceOptions { offline, limit: Some(limit) }).await.ok()?;
    (200..300).contains(&fetched.status).then(|| text::decode_utf8(&fetched.bytes))
}

// Reddit's markdown as plain words: links keep their text.
fn plain_markdown(words: &str) -> String {
    MARKDOWN_LINK.replace_all(words, "$1").replace("&#x200B;", "").trim().to_owned()
}

// Paragraphs at blank lines, line breaks inside them, every character escaped.
fn paragraphs_html(words: &str) -> String {
    paragraphs(&escape(words))
}

// HTML already escaped, in paragraphs at blank lines with line breaks inside them.
fn paragraphs(html: &str) -> String {
    BLANK_LINES
        .split(html)
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .map(|paragraph| format!("<p>{}</p>", paragraph.replace('\n', "<br>")))
        .collect()
}

fn escape(words: &str) -> String {
    words.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// At most `chars` characters, cut at the last space before the limit, with an ellipsis.
fn shortened(words: &str, chars: usize) -> String {
    if words.chars().count() <= chars {
        return words.to_owned();
    }
    let cut: String = words.chars().take(chars).collect();
    let at = cut.rfind(' ').unwrap_or(cut.len());
    format!("{}…", cut[..at].trim_end())
}
