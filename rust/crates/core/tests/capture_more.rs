//! What a saved link's own site adds after its capture: a Reddit post's words and top comments, a YouTube video's
//! length, Bluesky and Mastodon posts, the article an X post carries and the thread it ends, and sites that do not answer or are refused
//! leaving the card as it was.

use std::cell::RefCell;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::capture_link::CaptureOptions;
use mm_core::capture_http::forbidden;
use mm_core::capture_more::{bluesky_post, capture_more, mastodon_post, reddit_extras, reddit_json_url, syndication_token, x_article_extras, x_thread_post, youtube_seconds};
use mm_core::host::{AnalysisImage, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions};
use mm_core::http::{Headers, Response};
use mm_core::{Error, Result, Value, obj};
use mm_js::json;

struct Pages(Vec<(&'static str, u16, &'static str)>, RefCell<Vec<String>>);

#[async_trait(?Send)]
impl Resource for Pages {
    async fn fetch(&self, url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        self.1.borrow_mut().push(url.to_owned());
        if url.starts_with("http://169.254.") {
            return Err(forbidden());
        }
        let Some((_, status, body)) = self.0.iter().find(|(address, _, _)| *address == url) else { return Err(Error::internal("fetch failed")) };
        Ok(FetchedResource { bytes: body.as_bytes().to_vec(), url: url.to_owned(), mime: "application/json".into(), status: *status, headers: Headers::default() })
    }
}

struct NoMedia;

// Stores every picture as a cover named after its bytes; `broken` bytes are not pictures.
struct Covers;

#[async_trait(?Send)]
impl Media for Covers {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }
    async fn prepare(&self, input: MediaInput, _content_type: &str, name: &str, _user: &MediaUser, _id: &str) -> Result<PreparedMedia> {
        assert_eq!(name, "cover");
        let MediaInput::Bytes(bytes) = input else { unreachable!() };
        let named = String::from_utf8(bytes).unwrap();
        if named == "broken" {
            return Err(Error::invalid("not a picture"));
        }
        let cover = obj! { "src" => format!("/media/{named}"), "width" => 640, "height" => 480 };
        Ok(PreparedMedia { data: obj! { "type" => "Image" }.into(), extras: obj! { "cover" => cover }.into() })
    }
    async fn send(&self, _result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, _relative: &str) -> Result<Option<MediaResult>> {
        unreachable!()
    }
    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        unreachable!()
    }
}

#[async_trait(?Send)]
impl Media for NoMedia {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }
    async fn prepare(&self, _input: MediaInput, _content_type: &str, _name: &str, _user: &MediaUser, _id: &str) -> Result<PreparedMedia> {
        unreachable!()
    }
    async fn send(&self, _result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, _relative: &str) -> Result<Option<MediaResult>> {
        unreachable!()
    }
    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        unreachable!()
    }
}

const LINK: &str = "https://www.reddit.com/r/hiking/comments/1abc/the_longest_walk/";
const JSON: &str = "https://www.reddit.com/r/hiking/comments/1abc.json?raw_json=1&sort=top&limit=8&depth=1";

fn listing() -> Value {
    json::parse(
        r#"[
          {"data":{"children":[{"kind":"t3","data":{"title":"The longest walk?","subreddit":"hiking","author":"fernhollow",
            "selftext":"Took the [long way](https://example.test/map) home.\n\nWhat is the longest day you have walked?","is_video":false}}]}},
          {"data":{"children":[
            {"kind":"t1","data":{"author":"AutoModerator","body":"Remember the rules.","stickied":true}},
            {"kind":"t1","data":{"author":"moss_and_map","body":"Forty-one km along the coast, & every step <worth> it."}},
            {"kind":"t1","data":{"author":"[deleted]","body":"[removed]"}},
            {"kind":"t1","data":{"author":"ridgeline","body":"Thirty, in the rain."}},
            {"kind":"t1","data":{"author":"a_mod","body":"Locked.","distinguished":"moderator"}},
            {"kind":"t1","data":{"author":"third","body":"Twenty."}},
            {"kind":"t1","data":{"author":"fourth","body":"Ten."}},
            {"kind":"more","data":{"count":40}}
          ]}}
        ]"#,
    )
    .unwrap()
}

fn card(kind: &str, link: &str) -> Value {
    obj! {
        "data" => obj! { "type" => kind, "title" => "Saved" },
        "extras" => obj! { "linkUrl" => link, "reddit" => obj! { "title" => "The longest walk?", "subreddit" => "hiking" } },
    }
    .into()
}

fn run(card: &Value, pages: &Pages) -> Vec<Value> {
    run_with(card, pages, &NoMedia)
}

fn run_with(card: &Value, pages: &Pages, media: &dyn Media) -> Vec<Value> {
    let patches = RefCell::new(Vec::new());
    let update = |patch: Value| -> Result<()> {
        patches.borrow_mut().push(patch);
        Ok(())
    };
    let user = MediaUser { account_uid: "u".into(), id_prefix: "p".into() };
    let options = CaptureOptions { id: "c", user: &user, media, update: &update, offline: false };
    block_on(capture_more(card, &options, pages)).unwrap();
    patches.into_inner()
}

#[test]
fn a_reddit_post_reads_its_public_json() {
    assert_eq!(reddit_json_url(LINK).as_deref(), Some(JSON));
    assert_eq!(reddit_json_url("https://old.reddit.com/r/hiking/comments/1abc").as_deref(), Some(JSON));
    assert_eq!(reddit_json_url("https://www.reddit.com/r/hiking/"), None);
    assert_eq!(reddit_json_url("https://example.test/r/hiking/comments/1abc"), None);
}

#[test]
fn a_reddit_post_keeps_its_words_and_three_top_comments() {
    let extras = reddit_extras(&listing(), card("RedditPost", LINK).get("extras").get("reddit")).unwrap();
    let comments: Vec<(String, String)> = extras.get("reddit").get("comments").as_array().unwrap().iter()
        .map(|comment| (comment.get("author").to_js_string(), comment.get("body").to_js_string()))
        .collect();
    assert_eq!(comments, [
        ("moss_and_map".to_owned(), "Forty-one km along the coast, & every step <worth> it.".to_owned()),
        ("ridgeline".to_owned(), "Thirty, in the rain.".to_owned()),
        ("third".to_owned(), "Twenty.".to_owned()),
    ]);
    assert_eq!(extras.get("reddit").get("title").as_str(), Some("The longest walk?"));
    assert_eq!(extras.get("reddit").get("subreddit").as_str(), Some("hiking"));
    assert_eq!(
        extras.get("post").get("bodyHtml").as_str(),
        Some("<p>Took the long way home.</p><p>What is the longest day you have walked?</p>")
    );
    assert_eq!(extras.get("post").get("author").as_str(), Some("by u/fernhollow"));
}

#[test]
fn a_link_post_without_words_keeps_only_its_comments() {
    let mut listing = listing();
    let text = json::to_string(&listing).replace("Took the [long way](https://example.test/map) home.\\n\\nWhat is the longest day you have walked?", "");
    listing = json::parse(&text).unwrap();
    let extras = reddit_extras(&listing, &Value::Undefined).unwrap();
    assert!(extras.get("post").is_undefined());
    assert_eq!(extras.get("reddit").get("comments").as_array().map(Vec::len), Some(3));
    assert_eq!(extras.get("reddit").get("subreddit").as_str(), Some("hiking"), "the post names its subreddit");
}

#[test]
fn a_long_comment_is_cut_at_a_word() {
    let long = "word ".repeat(200);
    let text = json::to_string(&listing()).replace("Thirty, in the rain.", &long);
    let extras = reddit_extras(&json::parse(&text).unwrap(), &Value::Undefined).unwrap();
    let body = extras.get("reddit").get("comments").at(1).get("body").to_js_string();
    assert!(body.ends_with("word…"), "{body}");
    assert!(body.chars().count() <= 601);
}

#[test]
fn capture_commits_reddit_words_as_one_more_stage() {
    let pages = Pages(vec![(JSON, 200, Box::leak(json::to_string(&listing()).into_boxed_str()))], RefCell::default());
    let patches = run(&card("RedditPost", LINK), &pages);
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].get("extras").get("reddit").get("comments").as_array().map(Vec::len), Some(3));
    assert_eq!(pages.1.borrow().as_slice(), [JSON]);
}

#[test]
fn a_youtube_video_reads_its_length_from_its_page() {
    assert_eq!(youtube_seconds(r#"..."videoDetails":{"videoId":"x","lengthSeconds":"634","keywords":[]}"#), Some(634));
    assert_eq!(youtube_seconds(r#""lengthSeconds":"0""#), None);
    assert_eq!(youtube_seconds("<html></html>"), None);
    let watch = "https://www.youtube.com/watch?v=x";
    let pages = Pages(vec![(watch, 200, r#"<script>var a = {"lengthSeconds":"75"};</script>"#)], RefCell::default());
    let patches = run(&card("YouTubeVideo", watch), &pages);
    assert_eq!(patches[0].get("extras").get("videoLength").as_f64(), Some(75.0));
}

#[test]
fn a_site_that_does_not_answer_leaves_the_card_as_it_was() {
    let refused = Pages(vec![(JSON, 429, "Too Many Requests")], RefCell::default());
    assert!(run(&card("RedditPost", LINK), &refused).is_empty());
    let garbled = Pages(vec![(JSON, 200, "<html>blocked</html>")], RefCell::default());
    assert!(run(&card("RedditPost", LINK), &garbled).is_empty());
    let silent = Pages(vec![], RefCell::default());
    assert!(run(&card("YouTubeVideo", "https://www.youtube.com/watch?v=x"), &silent).is_empty());
    assert!(run(&card("WebPage", "https://example.test/"), &silent).is_empty());
    assert!(silent.1.borrow().len() == 1, "only the video's page is asked for");
}

const BLUESKY_LINK: &str = "https://bsky.app/profile/fern.example.social/post/3kfern";
const BLUESKY_HANDLE: &str = "https://public.api.bsky.app/xrpc/com.atproto.identity.resolveHandle?handle=fern.example.social";
const BLUESKY_POSTS: &str = "https://public.api.bsky.app/xrpc/app.bsky.feed.getPosts?uris=at%3A%2F%2Fdid%3Aplc%3Afern%2Fapp.bsky.feed.post%2F3kfern";

fn bluesky_answer(embed: &str) -> String {
    let text = "Café walk ☕ with @moss.example.social\n\nmap: example.test/map #hiking";
    let at = |words: &str| {
        let start = text.find(words).unwrap();
        format!(r#"{{"byteStart":{start},"byteEnd":{}}}"#, start + words.len())
    };
    format!(
        r#"{{"posts":[{{"uri":"at://did:plc:fern/app.bsky.feed.post/3kfern",
          "author":{{"did":"did:plc:fern","handle":"fern.example.social","displayName":" Fern Hollow "}},
          "record":{{"text":{},
            "facets":[
              {{"index":{},"features":[{{"$type":"app.bsky.richtext.facet#mention","did":"did:plc:moss"}}]}},
              {{"index":{},"features":[{{"$type":"app.bsky.richtext.facet#link","uri":"https://example.test/map"}}]}},
              {{"index":{},"features":[{{"$type":"app.bsky.richtext.facet#tag","tag":"hiking"}}]}},
              {{"index":{{"byteStart":5,"byteEnd":400}},"features":[{{"$type":"app.bsky.richtext.facet#link","uri":"https://example.test/"}}]}}
            ]}},
          "embed":{embed}}}]}}"#,
        json::to_string(&Value::from(text)),
        at("@moss.example.social"),
        at("example.test/map"),
        at("#hiking"),
    )
}

fn saved_page(kind: &str, link: &str) -> Value {
    obj! {
        "data" => obj! { "type" => kind, "title" => "Fern Hollow (@fern.example.social)", "source" => obj! { "url" => link } },
        "extras" => obj! { "linkUrl" => link, "cover" => obj! { "src" => "/media/avatar" } },
    }
    .into()
}

#[test]
fn a_bluesky_post_keeps_its_words_with_their_links() {
    let answer = json::parse(&bluesky_answer("null")).unwrap();
    let post = bluesky_post(&answer, "3kfern").unwrap();
    assert_eq!(post.kind, "BlueskyPost");
    assert_eq!(post.name, "Fern Hollow");
    assert_eq!(
        post.html,
        "<p>Caf&#xE9; walk &#x2615; with <a href=\"https://bsky.app/profile/did:plc:moss\">@moss.example.social</a></p>\
         <p>map: <a href=\"https://example.test/map\">example.test/map</a> <a href=\"https://bsky.app/hashtag/hiking\">#hiking</a></p>"
    );
    assert!(post.images.is_empty() && !post.video);
    assert_eq!(bluesky_post(&answer, "3kother"), None, "the answer must be for the saved post");
    assert_eq!(bluesky_post(&json::parse(r#"{"posts":[]}"#).unwrap(), "3kfern"), None);
}

#[test]
fn a_bluesky_post_keeps_its_pictures_or_its_video_poster() {
    let images = r#"{"$type":"app.bsky.embed.images#view","images":[{"thumb":"https://cdn.example.test/t1","fullsize":"https://cdn.example.test/f1"},{"thumb":"https://cdn.example.test/t2"}]}"#;
    let post = bluesky_post(&json::parse(&bluesky_answer(images)).unwrap(), "3kfern").unwrap();
    assert_eq!(post.images, ["https://cdn.example.test/f1", "https://cdn.example.test/t2"]);
    let gallery = r#"{"$type":"app.bsky.embed.recordWithMedia#view","media":{"$type":"app.bsky.embed.gallery#view","items":[{"thumbnail":"https://cdn.example.test/t1","fullsize":"https://cdn.example.test/f1"}]}}"#;
    assert_eq!(bluesky_post(&json::parse(&bluesky_answer(gallery)).unwrap(), "3kfern").unwrap().images, ["https://cdn.example.test/f1"]);
    let video = r#"{"$type":"app.bsky.embed.video#view","thumbnail":"https://video.example.test/poster.jpg","playlist":"https://video.example.test/p.m3u8"}"#;
    let post = bluesky_post(&json::parse(&bluesky_answer(video)).unwrap(), "3kfern").unwrap();
    assert!(post.video);
    assert_eq!(post.images, ["https://video.example.test/poster.jpg"]);
}

#[test]
fn capture_turns_a_saved_bluesky_page_into_its_post() {
    let images = r#"{"$type":"app.bsky.embed.images#view","images":[{"fullsize":"https://cdn.example.test/f1"},{"fullsize":"https://cdn.example.test/f2"}]}"#;
    let answer: &'static str = Box::leak(bluesky_answer(images).into_boxed_str());
    let pages = Pages(
        vec![
            (BLUESKY_HANDLE, 200, r#"{"did":"did:plc:fern"}"#),
            (BLUESKY_POSTS, 200, answer),
            ("https://cdn.example.test/f1", 200, "one"),
            ("https://cdn.example.test/f2", 200, "two"),
        ],
        RefCell::default(),
    );
    let patches = run_with(&saved_page("Article", BLUESKY_LINK), &pages, &Covers);
    assert_eq!(patches.len(), 1);
    let (data, extras) = (patches[0].get("data"), patches[0].get("extras"));
    assert_eq!(data.get("type").as_str(), Some("BlueskyPost"));
    assert_eq!(data.get("author").as_str(), Some("by Fern Hollow"));
    assert_eq!(data.get("description").as_str(), Some("Café walk ☕ with @moss.example.social map: example.test/map #hiking"));
    assert_eq!(extras.get("post").get("author").as_str(), Some("by Fern Hollow"));
    let gallery: Vec<String> = extras.get("post").get("gallery").as_array().unwrap().iter().map(|cover| cover.get("src").to_js_string()).collect();
    assert_eq!(gallery, ["/media/one", "/media/two"]);
    assert!(extras.as_object().unwrap().get("cover").is_some_and(Value::is_undefined), "two pictures show as the post's gallery");
    assert_eq!(extras.get("capturedTitle").as_str(), Some("Fern Hollow (@fern.example.social)"));
}

#[test]
fn a_bluesky_post_without_pictures_drops_the_page_picture() {
    let pages = Pages(vec![(BLUESKY_HANDLE, 200, r#"{"did":"did:plc:fern"}"#), (BLUESKY_POSTS, 200, Box::leak(bluesky_answer("null").into_boxed_str()))], RefCell::default());
    let patches = run_with(&saved_page("WebPage", BLUESKY_LINK), &pages, &Covers);
    let extras = patches[0].get("extras");
    assert!(extras.as_object().unwrap().get("cover").is_some_and(Value::is_undefined), "the page's picture is the author's face");
    assert_eq!(extras.get("post").get("gallery").as_array().map(Vec::len), Some(0));
}

#[test]
fn a_post_whose_pictures_cannot_be_fetched_keeps_the_captured_one() {
    let images = r#"{"$type":"app.bsky.embed.images#view","images":[{"fullsize":"https://cdn.example.test/f1"},{"fullsize":"http://169.254.169.254/latest"}]}"#;
    let pages = Pages(
        vec![(BLUESKY_HANDLE, 200, r#"{"did":"did:plc:fern"}"#), (BLUESKY_POSTS, 200, Box::leak(bluesky_answer(images).into_boxed_str())), ("https://cdn.example.test/f1", 200, "broken")],
        RefCell::default(),
    );
    let patches = run_with(&saved_page("WebPage", BLUESKY_LINK), &pages, &Covers);
    assert_eq!(patches.len(), 1, "a refused picture is skipped, not an error");
    assert!(patches[0].get("extras").as_object().unwrap().get("cover").is_none());
}

const MASTODON_LINK: &str = "https://social.example.test/@fern/113000000000000001";
const MASTODON_STATUS: &str = "https://social.example.test/api/v1/statuses/113000000000000001";

fn mastodon_status() -> Value {
    json::parse(
        r#"{"id":"113000000000000001","content":"<p>Low tide at <a href=\"https://social.example.test/tags/coast\" class=\"mention hashtag\">#<span>coast</span></a><script>x</script></p>",
          "spoiler_text":"","account":{"username":"fern","display_name":"Fern :verified: Hollow"},
          "media_attachments":[{"type":"video","url":"https://files.example.test/v.mp4","preview_url":"https://files.example.test/v.jpg"},{"type":"audio","url":"https://files.example.test/a.mp3"}],
          "card":{"image":"https://files.example.test/card.jpg"}}"#,
    )
    .unwrap()
}

#[test]
fn a_mastodon_post_keeps_its_words_and_its_video_poster() {
    let post = mastodon_post(&mastodon_status(), "113000000000000001", MASTODON_LINK).unwrap();
    assert_eq!(post.kind, "MastodonPost");
    assert_eq!(post.name, "Fern Hollow", "custom emoji codes are dropped");
    assert_eq!(post.html, "<p>Low tide at <a href=\"https://social.example.test/tags/coast\">#<span>coast</span></a></p>");
    assert!(post.video);
    assert_eq!(post.images, ["https://files.example.test/v.jpg"]);
    assert_eq!(mastodon_post(&mastodon_status(), "113000000000000002", MASTODON_LINK), None);
    let text = json::to_string(&mastodon_status()).replace(r#""spoiler_text":"""#, r#""spoiler_text":"Sea & sand""#).replace(r#""type":"video""#, r#""type":"unknown""#);
    let warned = mastodon_post(&json::parse(&text).unwrap(), "113000000000000001", MASTODON_LINK).unwrap();
    assert!(warned.html.starts_with("<p>Sea &amp; sand</p><p>Low tide"), "{}", warned.html);
    assert_eq!(warned.images, ["https://files.example.test/card.jpg"], "a post without pictures shows its link's");
}

#[test]
fn capture_turns_a_saved_mastodon_page_into_its_post() {
    let status: &'static str = Box::leak(json::to_string(&mastodon_status()).into_boxed_str());
    let pages = Pages(vec![(MASTODON_STATUS, 200, status), ("https://files.example.test/v.jpg", 200, "poster")], RefCell::default());
    let patches = run_with(&saved_page("Article", MASTODON_LINK), &pages, &Covers);
    let (data, extras) = (patches[0].get("data"), patches[0].get("extras"));
    assert_eq!(data.get("type").as_str(), Some("MastodonPost"));
    assert_eq!(extras.get("cover").get("src").as_str(), Some("/media/poster"));
    assert!(extras.get("post").get("hasVideo").truthy());
    assert!(run(&saved_page("WebPage", "https://example.test/@fern/about"), &Pages(vec![], RefCell::default())).is_empty());
    let other = Pages(vec![(MASTODON_STATUS, 404, r#"{"error":"Record not found"}"#)], RefCell::default());
    assert!(run(&saved_page("WebPage", MASTODON_LINK), &other).is_empty(), "a site that is not Mastodon leaves the page");
}

const X_LINK: &str = "https://x.com/fernhollow/status/2010751592346030461";

fn x_tweet() -> &'static str {
    r#"{"id_str":"2010751592346030461","text":"New piece https://t.co/AbC123",
      "entities":{"urls":[{"url":"https://t.co/AbC123","expanded_url":"http://x.com/i/article/2010742786430021632"}]},
      "article":{"rest_id":"2010742786430021632","title":" The quiet economics of walking cities ","preview_text":"A city that is good to walk in.",
        "cover_media":{"media_info":{"original_img_url":"https://pbs.example.test/cover.jpg"}}}}"#
}

#[test]
fn x_embed_data_names_the_article_a_post_carries() {
    assert_eq!(syndication_token(2010751592346030461), "4vgynb6ldq");
    assert_eq!(syndication_token(1234567890123456789), "2zqic77uqyk");
    assert_eq!(syndication_token(20), "6dq1a2xwd93");
    let article = x_article_extras(&json::parse(x_tweet()).unwrap()).unwrap();
    assert_eq!(article.get("title").as_str(), Some("The quiet economics of walking cities"));
    assert_eq!(article.get("preview").as_str(), Some("A city that is good to walk in."));
    assert_eq!(article.get("url").as_str(), Some("https://x.com/i/article/2010742786430021632"));
    assert_eq!(article.get("image").as_str(), Some("https://pbs.example.test/cover.jpg"));
    assert_eq!(x_article_extras(&json::parse(r#"{"id_str":"1","text":"Just words"}"#).unwrap()), None);
}

#[test]
fn capture_gives_an_x_post_its_article_card() {
    let url = format!("https://cdn.syndication.twimg.com/tweet-result?id=2010751592346030461&lang=en&token={}", syndication_token(2010751592346030461));
    let pages = Pages(vec![(Box::leak(url.into_boxed_str()), 200, x_tweet()), ("https://pbs.example.test/cover.jpg", 200, "cover")], RefCell::default());
    let card: Value = obj! {
        "data" => obj! { "type" => "XPost", "title" => "Fern on X" },
        "extras" => obj! { "linkUrl" => X_LINK, "post" => obj! { "bodyHtml" => "<p>New piece <a href=\"https://t.co/AbC123\">https://t.co/AbC123</a></p>", "author" => "by Fern" } },
    }
    .into();
    let patches = run_with(&card, &pages, &Covers);
    let (data, extras) = (patches[0].get("data"), patches[0].get("extras"));
    assert_eq!(data.get("title").as_str(), Some("The quiet economics of walking cities"));
    assert_eq!(data.get("description").as_str(), Some("A city that is good to walk in."));
    assert_eq!(extras.get("article").get("url").as_str(), Some("https://x.com/i/article/2010742786430021632"));
    assert!(extras.get("article").get("image").is_undefined(), "the picture is the card's cover");
    assert_eq!(extras.get("cover").get("src").as_str(), Some("/media/cover"));
    assert_eq!(extras.get("post").get("bodyHtml").as_str(), Some("<p>New piece </p>"), "the article's own link goes");
    assert_eq!(extras.get("post").get("author").as_str(), Some("by Fern"));
    let plain = Pages(vec![], RefCell::default());
    assert!(run(&card, &plain).is_empty(), "a post X says nothing about stays as it was");
}

fn syndication(id: u64) -> &'static str {
    Box::leak(format!("https://cdn.syndication.twimg.com/tweet-result?id={id}&lang=en&token={}", syndication_token(id)).into_boxed_str())
}

const THREAD_FIRST: &str = r#"{"id_str":"100","text":"I spent a year redrawing every map in a 1931 walking guide. https://t.co/pic1",
  "display_text_range":[0,59],"user":{"id_str":"7"},"entities":{"urls":[]},
  "mediaDetails":[{"type":"photo","media_url_https":"https://pbs.example.test/hills.jpg"}]}"#;
const THREAD_SECOND: &str = r#"{"id_str":"200","text":"@theo_maps Every contour drawn with one pen & no shading. See https://t.co/AbC",
  "display_text_range":[11,78],"user":{"id_str":"7"},"in_reply_to_status_id_str":"100","in_reply_to_user_id_str":"7",
  "entities":{"urls":[{"url":"https://t.co/AbC","expanded_url":"https://theopark.example/1931","display_url":"theopark.example/1931"}]}}"#;
const THREAD_LAST: &str = r#"{"id_str":"300","text":"@theo_maps A folding booklet this winter.","display_text_range":[11,41],
  "user":{"id_str":"7"},"in_reply_to_status_id_str":"200","in_reply_to_user_id_str":"7",
  "mediaDetails":[{"type":"video","media_url_https":"https://pbs.example.test/poster.jpg"}]}"#;

#[test]
fn a_thread_post_keeps_its_own_words_links_and_pictures() {
    let second = x_thread_post(&json::parse(THREAD_SECOND).unwrap());
    assert_eq!(second.html, "<p>Every contour drawn with one pen &amp; no shading. See <a href=\"https://theopark.example/1931\">theopark.example/1931</a></p>");
    assert!(second.images.is_empty() && !second.video);
    let first = x_thread_post(&json::parse(THREAD_FIRST).unwrap());
    assert_eq!(first.html, "<p>I spent a year redrawing every map in a 1931 walking guide.</p>", "the picture's own link goes");
    assert_eq!(first.images, vec!["https://pbs.example.test/hills.jpg".to_owned()]);
    let last = x_thread_post(&json::parse(THREAD_LAST).unwrap());
    assert!(last.video);
}

#[test]
fn capture_keeps_the_thread_a_saved_x_post_ends() {
    let pages = Pages(
        vec![
            (syndication(300), 200, THREAD_LAST),
            (syndication(200), 200, THREAD_SECOND),
            (syndication(100), 200, THREAD_FIRST),
            ("https://pbs.example.test/hills.jpg", 200, "hills"),
            ("https://pbs.example.test/poster.jpg", 200, "poster"),
        ],
        RefCell::default(),
    );
    let card: Value = obj! {
        "data" => obj! { "type" => "XPost", "title" => "Theo Park on X" },
        "extras" => obj! { "linkUrl" => "https://x.com/theo_maps/status/300", "post" => obj! { "bodyHtml" => "<p>A folding booklet this winter.</p>", "author" => "by Theo Park" } },
    }
    .into();
    let patches = run_with(&card, &pages, &Covers);
    assert_eq!(patches.len(), 1);
    let extras = patches[0].get("extras");
    let thread = extras.get("post").get("thread").as_array().unwrap();
    assert_eq!(thread.len(), 3, "oldest first, the saved post last");
    assert!(thread[0].get("bodyHtml").as_str().unwrap().contains("1931 walking guide"));
    assert_eq!(thread[0].get("gallery").at(0).get("src").as_str(), Some("/media/hills"));
    assert_eq!(thread[2].get("hasVideo"), &Value::Bool(true));
    assert_eq!(extras.get("post").get("author").as_str(), Some("by Theo Park"), "the saved post stays as it was");
    assert_eq!(extras.get("cover").get("src").as_str(), Some("/media/hills"), "the thread's first picture is the cover");
    assert!(extras.get("captureText").as_str().unwrap().starts_with("I spent a year"));
    assert!(extras.get("captureText").as_str().unwrap().ends_with("A folding booklet this winter."));
    assert!(patches[0].get("data").get("title").is_undefined(), "the title is left for the analysis to name");
}

#[test]
fn a_reply_to_someone_else_is_not_a_thread() {
    let reply = r#"{"id_str":"300","text":"Agreed.","user":{"id_str":"7"},"in_reply_to_status_id_str":"200","in_reply_to_user_id_str":"9"}"#;
    let pages = Pages(vec![(syndication(300), 200, reply)], RefCell::default());
    let card: Value = obj! {
        "data" => obj! { "type" => "XPost" },
        "extras" => obj! { "linkUrl" => "https://x.com/theo_maps/status/300", "post" => obj! { "bodyHtml" => "<p>Agreed.</p>" } },
    }
    .into();
    assert!(run(&card, &pages).is_empty());
    assert_eq!(pages.1.borrow().len(), 1, "X is asked only for the saved post");
}
