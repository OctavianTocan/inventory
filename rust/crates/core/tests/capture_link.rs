//! `capture_link` against `captureLink` of `lib/capture-link.mjs` with identical table-driven fakes: the events (every
//! resource, screenshot and media call and every committed update, in order) and the outcome must match.

mod common;

use std::cell::RefCell;

use async_trait::async_trait;
use common::Random;
use mm_core::capture_link::{CaptureOptions, capture_link};
use mm_core::host::{AnalysisImage, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot};
use mm_core::http::{Headers, Response};
use mm_core::{Error, Result, Value, obj};

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/capture-link.mjs");

const OEMBED: [&str; 6] = [
    "{\"html\":\"<blockquote class=\\\"twitter-tweet\\\"><p>hi &amp; bye <a href=\\\"https://t.co/x\\\">l</a></p></blockquote>\",\"author_name\":\"Ann\",\"title\":\"Embed T\",\"thumbnail_url\":\"https://img.example/t.png\",\"type\":\"video\",\"author_avatar_url\":\"https://img.example/a.png\"}",
    "{\"title\":\"Only title\",\"author_name\":\"By Bob\",\"thumbnail_url\":\"/rel-thumb.png\",\"author_image_url\":\"https://img.example/1.png\"}",
    "not json at all",
    "{\"html\":\"<p>plain</p>\",\"type\":\"rich\",\"avatar_url\":\"https://pbs.twimg.com/profile_images/1/a.jpg\"}",
    "null",
    "{\"title\":7,\"author_name\":null,\"html\":\"\",\"thumbnail_url\":\"https://img.example/2.png\"}",
];

const MAIN_URLS: &[&str] = &[
    "https://example.com/a", "https://example.com/files/report.pdf", "https://x.com/u/status/123", "https://www.youtube.com/watch?v=1",
    "https://old.reddit.com/r/rust/comments/1/t", "https://www.instagram.com/reel/abc/", "https://threads.net/@u/post/1", "https://u.substack.com/note/c-1",
    "https://github.com/o/r", "https://apps.apple.com/us/app/x/id1", "https://play.google.com/store/apps/details?id=x", "https://shop.example.com/p",
    "https://example.com/app.apk", "not a url", "https://example.com/%E0%A4%A", "https://example.com/dir/", "https://www.example.co.uk/blog/post",
];

const MIMES: &[&str] = &[
    "text/html; charset=utf-8", "text/html", "text/html", "application/pdf", "application/json", "image/png", "application/octet-stream", "", "text/plain",
    "application/xhtml+xml",
];

const PIECES: &[&str] = &[
    "<title>Page T | Example</title>", "<title></title>", "<meta property=\"og:title\" content=\"Og Title - Example\">", "<meta name=\"description\" content=\"Plain description\">",
    "<meta property=\"og:description\" content=\"Og description &amp; more\">", "<meta property=\"og:site_name\" content=\"Example Site\">",
    "<meta property=\"og:image\" content=\"https://img.example/1.png\">", "<meta property=\"og:image\" content=\"/rel.png\">",
    "<meta property=\"og:image\" content=\"https://img.example/2.png\">", "<meta property=\"og:image\" content=\"https://img.example/1.png\">",
    "<meta name=\"twitter:image\" content=\"//cdn.example/tw.png\">", "<meta property=\"og:image\" content=\"https://pbs.twimg.com/profile_images/1/a.jpg\">",
    "<meta property=\"og:image\" content=\"https://img.example/3.png\"><meta property=\"og:image\" content=\"https://img.example/4.png\"><meta property=\"og:image\" content=\"https://img.example/5.png\"><meta property=\"og:image\" content=\"https://img.example/6.png\"><meta property=\"og:image\" content=\"https://img.example/7.png\">",
    "<meta property=\"og:type\" content=\"article\">", "<meta name=\"author\" content=\"Jane Writer\">",
    "<link rel=\"canonical\" href=\"https://github.com/o/r\">", "<link rel=\"canonical\" href=\"https://www.youtube.com/watch?v=9\">",
    "<link rel=\"canonical\" href=\"https://x.com/a/status/42\">", "<link rel=\"canonical\" href=\"https://apps.apple.com/us/app/y/id2\">",
    "<link rel=\"canonical\" href=\"https://play.google.com/store/apps/details?id=y\">", "<link rel=\"canonical\" href=\"https://www.reddit.com/r/rust/comments/2/u\">",
    "<link rel=\"canonical\" href=\"/relative-canonical\">", "<link rel=\"canonical\" href=\"https://blog.example.org/post\">",
    "<link type=\"application/json+oembed\" href=\"https://oembed.example/o?u=1\">", "<link type=\"application/json+oembed\" href=\"/oembed?u=2\">",
    "<script type=\"application/ld+json\">{\"@type\":\"Product\",\"name\":\"Gadget\",\"brand\":{\"name\":\"Acme\"},\"offers\":{\"price\":\"9.99\",\"priceCurrency\":\"EUR\"},\"image\":\"/prod.png\"}</script>",
    "<script type=\"application/ld+json\">{\"@type\":\"Product\",\"name\":\"Bare\",\"brand\":\"Solo\",\"offers\":[{\"price\":5}]}</script>",
    "<script type=\"application/ld+json\">{\"@type\":\"Article\",\"headline\":\"Headline\",\"author\":{\"@type\":\"Person\",\"name\":\"Ann\",\"image\":\"/ann.png\"},\"articleBody\":\"Body text\"}</script>",
    "<script type=\"application/ld+json\">{\"@type\":\"SoftwareApplication\",\"name\":\"App Name on the App Store\"}</script>",
    "<script type=\"application/ld+json\">{bad</script>", "<article><p>First paragraph.</p><p>Second &amp; third.</p></article>", "<p>Loose paragraph</p>",
    "<nav>menu</nav><footer>foot</footer>", "<p></p>",
];

fn hash(text: &str, seed: u32) -> u32 {
    let mut value = 2_166_136_261u32 ^ seed;
    for byte in text.bytes() {
        value = (value ^ u32::from(byte)).wrapping_mul(16_777_619);
    }
    value
}

fn blocked() -> Error {
    Error::status(400, "Unsupported capture destination").with_code("CAPTURE_BLOCKED")
}

fn fetched(bytes: &[u8], url: &str, mime: &str) -> FetchedResource {
    FetchedResource { bytes: bytes.to_vec(), url: url.to_owned(), mime: mime.to_owned(), status: 200, headers: Headers::default() }
}

struct Scenario {
    seed: u32,
    main: String,
    page: Option<(String, String, Option<String>)>,
    screenshot: &'static str,
}

struct Fakes<'a> {
    scenario: &'a Scenario,
    events: &'a RefCell<Vec<Value>>,
}

#[async_trait(?Send)]
impl Resource for Fakes<'_> {
    async fn fetch(&self, url: &str, options: ResourceOptions) -> Result<FetchedResource> {
        self.events.borrow_mut().push(obj! { "op" => "resource", "url" => url, "offline" => options.offline, "limit" => options.limit.map_or(Value::Undefined, |limit| (limit as f64).into()) }.into());
        if url == self.scenario.main
            && let Some((mime, text, last)) = &self.scenario.page {
                return Ok(fetched(text.as_bytes(), last.as_deref().unwrap_or(url), mime));
            }
        let h = hash(url, self.scenario.seed);
        if url.contains("oembed") {
            return match h % 8 {
                6 => Err(blocked()),
                7 => Err(Error::internal("Page unavailable")),
                kind => Ok(fetched(OEMBED[kind as usize % OEMBED.len()].as_bytes(), url, "application/json")),
            };
        }
        match h % 8 {
            0 | 1 => Ok(fetched(format!("IMG{}", h % 97).as_bytes(), url, "image/png")),
            2 => Ok(fetched(b"JPG", url, "image/jpeg")),
            3 => Ok(fetched(b"XBAD", url, "image/png")),
            4 => Ok(fetched(b"<html></html>", url, "text/html")),
            5 => Err(Error::internal("Page unavailable")),
            6 => Err(blocked()),
            _ => Ok(fetched(b"WEBP", url, "image/webp")),
        }
    }
}

#[async_trait(?Send)]
impl Screenshot for Fakes<'_> {
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        self.events.borrow_mut().push(obj! { "op" => "screenshot", "url" => url, "offline" => offline }.into());
        match self.scenario.screenshot {
            "blocked" => Err(blocked()),
            "fail" => Err(Error::internal("browser failed")),
            "bad" => Ok(b"XSHOT".to_vec()),
            _ => Ok(b"SHOT".to_vec()),
        }
    }
}

#[async_trait(?Send)]
impl Media for Fakes<'_> {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }

    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let MediaInput::Bytes(bytes) = input else { unreachable!() };
        self.events.borrow_mut().push(
            obj! { "op" => "prepare", "length" => bytes.len() as f64, "mime" => content_type, "name" => name,
                "user" => obj! { "accountUid" => user.account_uid.as_str(), "idPrefix" => user.id_prefix.as_str() }, "id" => id }
            .into(),
        );
        if bytes.first() == Some(&b'X') {
            return Err(Error::status(422, "Media preview could not be generated"));
        }
        let key = format!("{}/{id}/{}", user.account_uid, bytes.len());
        Ok(if content_type == "application/pdf" {
            PreparedMedia { data: obj! { "type" => "Document", "title" => name, "blob" => obj! { "path" => format!("{key}.pdf") }, "format" => "pdf" }.into(), extras: obj! {}.into() }
        } else if content_type.starts_with("image/") {
            PreparedMedia {
                data: obj! { "type" => "Image", "title" => name, "blob" => obj! { "path" => key.as_str() } }.into(),
                extras: obj! { "cover" => obj! { "src" => format!("/media/{key};800x1000-t.webp"), "width" => 800.0, "height" => 1000.0 } }.into(),
            }
        } else {
            PreparedMedia { data: obj! { "type" => "File", "title" => name }.into(), extras: obj! {}.into() }
        })
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

fn pick<'a, T>(random: &mut Random, items: &'a [T]) -> &'a T {
    &items[random.below(items.len() as u64) as usize]
}

fn make(random: &mut Random) -> (Scenario, Value, bool) {
    let main = (*pick(random, MAIN_URLS)).to_owned();
    let length = random.below(8) as usize;
    let html: String = (0..length).map(|_| *pick(random, PIECES)).collect();
    let mime = (*pick(random, MIMES)).to_owned();
    let last = match random.below(5) {
        0 => Some((*pick(random, MAIN_URLS)).to_owned()).filter(|url| url.starts_with("http")),
        _ => None,
    };
    let page = (random.below(12) != 0).then(|| (mime.clone(), html.clone(), last.clone()));
    let screenshot = *pick(random, &["ok", "ok", "blocked", "fail", "bad"]);
    let offline = random.below(4) == 0;
    let mut card = obj! { "title" => *pick(random, &["", "example.com", "Saved title", "report.pdf", "shop.example.com"]), "source" => obj! { "url" => main.as_str() } };
    if random.below(3) == 0 {
        card.insert("domain", "kept.example");
    }
    if random.below(3) == 0 {
        card.insert("siteName", "Kept Site");
    }
    if random.below(5) == 0 {
        card.insert("type", "Screenshot");
    }
    let seed = random.below(1000) as u32;
    let scenario = Scenario { seed, main: main.clone(), page: page.clone(), screenshot };
    let mut js = obj! { "seed" => f64::from(seed), "card" => card.clone(), "offline" => offline, "screenshot" => screenshot };
    if let Some((mime, text, last)) = page {
        let mut spec = obj! { "mime" => mime, "text" => text };
        if let Some(last) = last {
            spec.insert("final", last);
        }
        js.insert("page", spec);
    }
    (scenario, js.into(), offline)
}

#[test]
fn capture_link_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(0x4c49_4e4b);
    let mut made = Vec::new();
    for _ in 0..500 {
        made.push(make(&mut random));
    }
    let scenarios = Value::Array(made.iter().map(|(_, js, _)| js.clone()).collect());
    let js = common::call(DRIVER, "run", vec![scenarios.clone()]);
    let mut answers = Vec::new();
    let Value::Array(inputs) = scenarios else { unreachable!() };
    let user = MediaUser { account_uid: "acc".into(), id_prefix: "p".into() };
    let (mut oks, mut covers, mut shots, mut failed) = (0, 0, 0, 0);
    for ((scenario, _, offline), input) in made.iter().zip(&inputs) {
        let events = RefCell::new(Vec::new());
        let fakes = Fakes { scenario, events: &events };
        let update = |patch: Value| {
            events.borrow_mut().push(obj! { "op" => "update", "patch" => patch }.into());
            Ok(())
        };
        let options = CaptureOptions { id: "card1", user: &user, media: &fakes, update: &update, offline: *offline };
        let card = inputs_card(input);
        let outcome = mymind_block_on(capture_link(&|| card.clone(), &options, &fakes, &fakes));
        let result = match outcome {
            Ok(()) => "ok",
            Err(Error::Invalid(_)) => "invalid",
            Err(_) => "other",
        };
        let rust: Value = obj! { "result" => result, "events" => Value::Array(events.into_inner()) }.into();
        answers.push(rust.clone());
        let text = common::text(&rust);
        oks += usize::from(result == "ok");
        covers += usize::from(text.contains("\"cover\":{\"src\""));
        shots += usize::from(text.contains("\"op\":\"screenshot\""));
        failed += usize::from(text.contains("captureFailed"));
    }
    assert_eq!(answers.len(), made.len(), "every capture scenario is answered");
    common::assert_value(&Value::Array(answers), &js, "capture link scenarios and events");
    assert!(oks > 300 && covers > 100 && shots > 20 && failed > 5, "ok {oks}, covers {covers}, screenshots {shots}, failed {failed}");
    println!("captureLink scenarios: {} (ok {oks}, with cover {covers}, screenshots {shots}, placeholders {failed})", made.len());
}

fn inputs_card(input: &Value) -> Value {
    input.get("card").clone()
}

fn mymind_block_on<F: std::future::Future>(future: F) -> F::Output {
    futures_executor::block_on(future)
}
