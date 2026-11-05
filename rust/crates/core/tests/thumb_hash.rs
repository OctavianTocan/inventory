//! ThumbHash previews of gallery pictures over a synthetic library: the hash of a WebP, where a card keeps it, the
//! card rows the app reads, pictures that cannot be hashed, and the sweep through the pictures saved before.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use futures_executor::block_on;
use image_webp::{ColorType, WebPEncoder};
use mm_core::app_api::AppApi;
use mm_core::enrichment::{Enrichment, EnrichmentOptions};
use mm_core::host::{
    AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot,
};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::thumb_hash::{fill, missing, of_webp};
use mm_core::{Error, Result, Value, obj};
use mm_js::{base64, json, url};
use mm_sqlite::Sqlite;

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        block_on(task);
    }

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::ready(()))
    }

    fn log(&self, _line: &str) {}
}

// Renditions by their path below `/media/`; any other path is a picture the library does not have, and `broken` ones
// fail as a missing media container would.
#[derive(Default)]
struct Pictures {
    files: HashMap<String, Vec<u8>>,
    broken: Vec<String>,
    asked: RefCell<Vec<String>>,
}

#[async_trait(?Send)]
impl Media for Pictures {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }
    async fn prepare(&self, _input: MediaInput, _content_type: &str, _name: &str, _user: &MediaUser, _id: &str) -> Result<PreparedMedia> {
        unreachable!()
    }
    async fn send(&self, result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        Ok(Response::new(200, Headers::new(), Body::Bytes(self.files[&result.key].clone())))
    }
    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        self.asked.borrow_mut().push(relative.to_owned());
        if self.broken.iter().any(|path| path == relative) {
            return Err(Error::internal("media container unreachable"));
        }
        Ok(self.files.get(relative).map(|bytes| MediaResult { key: relative.to_owned(), size: bytes.len() as u64, content_type: "image/webp".into() }))
    }
    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        unreachable!()
    }
}

struct Unused;

#[async_trait(?Send)]
impl Fetch for Unused {
    async fn fetch(&self, _request: Request) -> Result<Response> {
        unreachable!()
    }
}

#[async_trait(?Send)]
impl Resource for Unused {
    async fn fetch(&self, _url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        unreachable!()
    }
}

#[async_trait(?Send)]
impl Screenshot for Unused {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        unreachable!()
    }
}

struct Library {
    db: Rc<Sqlite>,
    store: Rc<Store>,
    writer: Rc<Writer>,
}

impl Library {
    fn new() -> Self {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| runtime::install(&TestRuntime));
        runtime::set_env(HashMap::new());
        let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
        let store = Rc::new(Store::new(db.clone()).unwrap());
        db.execute("INSERT INTO users VALUES ('hash-user', 'hash-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
        let writer = Rc::new(Writer::new(store.clone(), |_, _| {}).unwrap());
        Self { db, store, writer }
    }

    // A card whose `extras.cover` or `data.blob` is `picture`.
    fn card(&self, title: &str, place: &str, picture: Value) -> String {
        let made = self.writer.run(|writer| writer.create(&obj! { "title" => title }.into(), &Value::Undefined)).unwrap();
        let id = made.get("body").get("id").as_str().unwrap().to_owned();
        let (column, kind) = if place == "cover" { ("extras", "WebPage") } else { ("data", "Image") };
        let picture = json::stringify(&picture).unwrap().unwrap();
        self.db.execute(&format!("UPDATE cards SET {column} = json_set({column}, '$.{place}', json(?)) WHERE id = ?"), &[picture.into(), id.as_str().into()]).unwrap();
        self.db.execute("UPDATE cards SET data = json_set(data, '$.type', ?), type = ? WHERE id = ?", &[kind.into(), kind.into(), id.as_str().into()]).unwrap();
        id
    }

    fn row(&self, id: &str) -> (Value, Value) {
        let row = self.store.card(id, None).unwrap().unwrap();
        (json::parse(&row.data).unwrap(), json::parse(&row.extras).unwrap())
    }

    fn app_card(&self, id: &str) -> Value {
        let search = Rc::new(Search::new(self.store.clone(), None, false).unwrap());
        let api = AppApi::new(self.store.clone(), search, |text| text.to_owned());
        let response = block_on(api.handle("GET", &url::parse(&format!("http://local/app/v1/cards/{id}"), None).unwrap())).unwrap().unwrap();
        json::parse(&String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()).unwrap()
    }

    fn jobs(&self, media: Rc<Pictures>) -> Enrichment {
        Enrichment::new(EnrichmentOptions {
            store: self.store.clone(),
            writer: self.writer.clone(),
            media,
            resource: Rc::new(Unused),
            screenshot: Rc::new(Unused),
            fetch: Rc::new(Unused),
            offline: true,
            readonly: false,
        })
    }
}

// A lossless WebP of one colour.
fn webp(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let pixels: Vec<u8> = (0..width * height).flat_map(|_| rgb).collect();
    let mut out = Vec::new();
    WebPEncoder::new(&mut out).encode(&pixels, width, height, ColorType::Rgb8).unwrap();
    out
}

fn average(hash: &str) -> (f32, f32, f32) {
    let (r, g, b, _) = thumbhash::thumb_hash_to_average_rgba(&base64::decode(hash)).unwrap();
    (r, g, b)
}

#[test]
fn a_webp_gives_a_hash_of_its_colours_and_shape() {
    let hash = of_webp(&webp(400, 240, [200, 30, 40])).unwrap();
    let (r, g, b) = average(&hash);
    assert!((r - 200.0 / 255.0).abs() < 0.05 && (g - 30.0 / 255.0).abs() < 0.05 && (b - 40.0 / 255.0).abs() < 0.05, "{r} {g} {b}");
    let ratio = thumbhash::thumb_hash_to_approximate_aspect_ratio(&base64::decode(&hash)).unwrap();
    assert!((ratio - 400.0 / 240.0).abs() < 0.2, "aspect {ratio}");
    assert!(base64::decode(&hash).len() <= 25);
    assert!(of_webp(b"not a picture").is_err());
}

#[test]
fn a_cover_and_a_picture_without_one_keep_their_hash_in_the_card_row() {
    let library = Library::new();
    let link = library.card("Link", "cover", obj! { "src" => "/media/a/1/p.png;800x480-t.webp", "width" => 800, "height" => 480, "dominantColor" => "#c81e28" }.into());
    let photo = library.card("Photo", "blob", obj! { "path" => "a/2/q.jpg", "type" => "image/jpeg", "width" => 1600, "height" => 1200 }.into());
    let media = Pictures {
        files: HashMap::from([("a/1/p.png;800x480-t.webp".into(), webp(800, 480, [200, 30, 40])), ("a/2/q.jpg;800x600-t.webp".into(), webp(800, 600, [20, 120, 220]))]),
        ..Pictures::default()
    };
    assert!(block_on(fill(&library.store, &library.writer, &media, &link)).unwrap());
    assert!(block_on(fill(&library.store, &library.writer, &media, &photo)).unwrap());
    let cover = library.app_card(&link).get("cover").clone();
    assert_eq!(cover.get("src").as_str(), Some("/media/a/1/p.png;800x480-t.webp"));
    assert!(average(cover.get("thumbHash").as_str().unwrap()).0 > 0.7);
    let card = library.app_card(&photo);
    let hash = card.get("blob").get("thumbHash").as_str().unwrap().to_owned();
    assert!(average(&hash).2 > 0.7);
    assert_eq!(card.get("cover").get("thumbHash").as_str(), Some(hash.as_str()), "the cover made from the picture carries it too");
    assert!(!block_on(fill(&library.store, &library.writer, &media, &link)).unwrap(), "a card hashed already is left alone");
    assert_eq!(media.asked.borrow().len(), 2);
}

#[test]
fn a_picture_that_cannot_be_hashed_is_marked_unless_the_failure_may_pass() {
    let library = Library::new();
    let gone = library.card("Gone", "cover", obj! { "src" => "/media/a/3/gone.png;800x800-t.webp", "width" => 800, "height" => 800 }.into());
    let broken = library.card("Broken", "cover", obj! { "src" => "/media/a/4/x.png;800x800-t.webp", "width" => 800, "height" => 800 }.into());
    let elsewhere = library.card("Elsewhere", "cover", obj! { "src" => "https://example.com/x.jpg", "width" => 800, "height" => 800 }.into());
    let media = Pictures { broken: vec!["a/4/x.png;800x800-t.webp".into()], ..Pictures::default() };
    assert!(!block_on(fill(&library.store, &library.writer, &media, &gone)).unwrap());
    assert_eq!(library.row(&gone).1.get("cover").get("thumbHash").as_str(), Some(""), "a missing picture is not asked for again");
    assert!(block_on(fill(&library.store, &library.writer, &media, &broken)).is_err());
    assert!(library.row(&broken).1.get("cover").get("thumbHash").is_undefined(), "an unreachable container leaves the card for later");
    assert!(!block_on(fill(&library.store, &library.writer, &media, &elsewhere)).unwrap());
    assert_eq!(missing(&library.store, 10).unwrap(), vec![broken]);
}

#[test]
fn a_hash_is_kept_only_for_the_picture_it_was_read_from() {
    let library = Library::new();
    let link = library.card("Link", "cover", obj! { "src" => "/media/a/5/new.png;800x800-t.webp", "width" => 800, "height" => 800 }.into());
    let kept = library.writer.run(|writer| writer.picture_hash(&link, "/media/a/5/old.png;800x800-t.webp", "AAAA")).unwrap();
    assert!(!kept);
    assert!(library.row(&link).1.get("cover").get("thumbHash").is_undefined());
}

#[test]
fn the_sweep_hashes_the_pictures_saved_before_newest_first_and_stops_at_its_limit() {
    let library = Library::new();
    let ids: Vec<String> = (0..4)
        .map(|n| library.card(&format!("Card {n}"), "cover", obj! { "src" => format!("/media/a/{n}/p.png;100x100-t.webp"), "width" => 100, "height" => 100 }.into()))
        .collect();
    let mut files: HashMap<String, Vec<u8>> = (0..4).map(|n| (format!("a/{n}/p.png;100x100-t.webp"), webp(100, 100, [10, 200, 10]))).collect();
    files.remove("a/2/p.png;100x100-t.webp");
    let media = Rc::new(Pictures { files, broken: vec!["a/1/p.png;100x100-t.webp".into()], ..Pictures::default() });
    let jobs = library.jobs(media.clone());
    jobs.hash_pictures(2);
    block_on(jobs.drain());
    assert_eq!(*media.asked.borrow(), ["a/3/p.png;100x100-t.webp", "a/2/p.png;100x100-t.webp"], "newest first, two at most");
    jobs.hash_pictures(usize::MAX);
    block_on(jobs.drain());
    assert_eq!(media.asked.borrow().len(), 4, "the broken picture is tried once per queue");
    let hashed = |id: &String| library.row(id).1.get("cover").get("thumbHash").as_str().map(str::to_owned);
    assert!(hashed(&ids[3]).is_some_and(|hash| !hash.is_empty()));
    assert_eq!(hashed(&ids[2]).as_deref(), Some(""));
    assert_eq!(hashed(&ids[1]), None);
    assert!(hashed(&ids[0]).is_some_and(|hash| !hash.is_empty()));
    jobs.hash_pictures(usize::MAX);
    block_on(jobs.drain());
    assert_eq!(media.asked.borrow().len(), 4, "a finished sweep does not start again");
}

#[test]
fn an_audio_file_has_no_picture_to_hash() {
    let library = Library::new();
    let memo = library.card("Audio", "blob", obj! { "path" => "a/3/memo.m4a", "type" => "audio/mp4", "duration" => 12.5 }.into());
    library.db.execute("UPDATE cards SET data = json_set(data, '$.type', 'Audio'), type = 'Audio' WHERE id = ?", &[memo.as_str().into()]).unwrap();
    let media = Pictures::default();
    assert!(!block_on(fill(&library.store, &library.writer, &media, &memo)).unwrap());
    assert!(media.asked.borrow().is_empty(), "the sound file is never sent to be drawn");
    assert!(!missing(&library.store, 10).unwrap().contains(&memo));
}
