//! A shared link's tags arrive while the rest of its capture (pictures, screenshot) is still running: the analysis
//! starts as soon as the page's words are stored, and it is not run a second time when only the picture changed.
//! A link that is understood from its picture still waits for it.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::poll_fn;
use std::rc::Rc;
use std::sync::Once;
use std::task::Poll;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::enrichment::{Enrichment, EnrichmentOptions};
use mm_core::host::{
    AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot,
};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Error, Result, Value, obj};
use mm_js::json;
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

// Embeddings are one fixed vector; the chat model tags every card "reading". Chat requests are counted.
#[derive(Default)]
struct Model {
    chats: Cell<usize>,
}

fn answer(body: &Value) -> Result<Response> {
    let text = json::stringify(body).unwrap().unwrap();
    Ok(Response::new(200, Headers::from([("content-type", "application/json".to_owned())]), Body::Bytes(text.into_bytes())))
}

#[async_trait(?Send)]
impl Fetch for Model {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = String::from_utf8(request.take_body().bytes().await?).unwrap();
        let vector = || Value::from(obj! { "values" => (0..768).map(|n| Value::from(if n == 0 { 1.0 } else { 0.0 })).collect::<Vec<_>>() });
        if request.url.as_str().ends_with(":embedContent") {
            return answer(&obj! { "embedding" => vector() }.into());
        }
        if request.url.as_str().ends_with(":batchEmbedContents") {
            let count = json::parse(&body).unwrap().get("requests").as_array().map_or(0, Vec::len);
            return answer(&obj! { "embeddings" => (0..count).map(|_| vector()).collect::<Vec<_>>() }.into());
        }
        if request.url.as_str().ends_with("/v1/decisions") {
            // Every tag fits; a Space question picks the first Space offered.
            let questions = json::parse(&body).unwrap().get("questions").as_array().cloned().unwrap_or_default();
            let answers: Vec<Value> = questions
                .iter()
                .map(|question| {
                    let name = question.get("name").clone();
                    if question.get("type").as_str() == Some("choice") {
                        obj! { "type" => "choice", "name" => name, "choice" => "s0", "probabilities" => vec![Value::from(obj! { "value" => "s0", "probability" => 0.8 })] }.into()
                    } else {
                        obj! { "type" => "predicate", "name" => name, "probability" => 0.9 }.into()
                    }
                })
                .collect();
            return answer(&obj! { "answers" => answers }.into());
        }
        assert!(request.url.as_str().ends_with("/chat/completions"), "{}", request.url);
        self.chats.set(self.chats.get() + 1);
        let card = obj! { "title" => "A page", "summary" => "About a page.", "tags" => vec![Value::from("reading")] };
        let content = json::stringify(&card.into()).unwrap().unwrap();
        answer(&obj! { "choices" => vec![Value::from(obj! { "message" => obj! { "content" => content } })] }.into())
    }
}

// Serves one HTML page with no picture of its own, so capture goes on to take a screenshot.
struct Page {
    html: String,
}

#[async_trait(?Send)]
impl Resource for Page {
    async fn fetch(&self, url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        Ok(FetchedResource { bytes: self.html.clone().into_bytes(), url: url.to_owned(), mime: "text/html".to_owned(), status: 200, headers: Headers::new() })
    }
}

// A slow screenshot: it gives the other work up to a hundred turns, ends as soon as the card has tags, and records
// whether it saw them. It always fails, so no picture is stored.
struct SlowShot {
    store: Rc<Store>,
    card: RefCell<String>,
    saw_tags: Cell<bool>,
}

#[async_trait(?Send)]
impl Screenshot for SlowShot {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        let mut turns = 0;
        poll_fn(|context| {
            turns += 1;
            if !self.store.card_tags(&self.card.borrow()).unwrap().is_empty() {
                self.saw_tags.set(true);
                return Poll::Ready(());
            }
            if turns > 100 {
                return Poll::Ready(());
            }
            context.waker().wake_by_ref();
            Poll::Pending
        })
        .await;
        Err(Error::invalid("no browser in this test"))
    }
}

struct NoMedia;

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
        Ok(None)
    }
    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        Ok(None)
    }
}

// Tests set the process environment, so they run one at a time.
static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Library {
    _env: std::sync::MutexGuard<'static, ()>,
    store: Rc<Store>,
    writer: Rc<Writer>,
    shot: Rc<SlowShot>,
    model: Rc<Model>,
}

impl Library {
    fn new() -> Self {
        let env = ENV.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        static ONCE: Once = Once::new();
        ONCE.call_once(|| runtime::install(&TestRuntime));
        runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned())]));
        let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
        let store = Rc::new(Store::new(db.clone()).unwrap());
        db.execute("INSERT INTO users VALUES ('tag-user', 'tag-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
        let writer = Rc::new(Writer::new(store.clone(), |_, _| {}).unwrap());
        let shot = Rc::new(SlowShot { store: store.clone(), card: RefCell::default(), saw_tags: Cell::new(false) });
        Self { _env: env, store, writer, shot, model: Rc::default() }
    }

    // Save `url` as a shared link and capture it from a page whose body says `words`.
    fn share(&self, url: &str, words: &str) -> String {
        let made = self.writer.run(|writer| writer.create(&obj! { "content" => url }.into(), &Value::Undefined)).unwrap();
        let id = made.get("body").get("id").as_str().unwrap().to_owned();
        self.capture(&id, words);
        id
    }

    // Capture and analyse a saved link from a page whose body says `words`.
    fn capture(&self, id: &str, words: &str) {
        let id = id.to_owned();
        *self.shot.card.borrow_mut() = id.clone();
        let html = format!("<html><head><title>A page</title></head><body><main><p>{words}</p></main></body></html>");
        let jobs = Enrichment::new(EnrichmentOptions {
            store: self.store.clone(),
            writer: self.writer.clone(),
            media: Rc::new(NoMedia),
            resource: Rc::new(Page { html }),
            screenshot: self.shot.clone(),
            fetch: self.model.clone(),
            offline: false,
            readonly: false,
        });
        jobs.schedule(&id, true);
        block_on(jobs.drain());
    }

    fn tags(&self, id: &str) -> Vec<String> {
        self.store.card_tags(id).unwrap().into_iter().map(|tag| tag.name).collect()
    }
}

#[test]
fn a_links_tags_arrive_before_its_screenshot_and_are_judged_once() {
    let library = Library::new();
    let words = "Long-form writing about gardens, soil and the slow work of growing vegetables through a cold spring. ".repeat(4);
    let id = library.share("https://example.com/gardens", &words);
    assert!(library.shot.saw_tags.get(), "tags were on the card while the screenshot was still being taken");
    assert_eq!(library.tags(&id), ["reading"]);
    assert_eq!(library.model.chats.get(), 1, "the analysis after capture is skipped when only the picture changed");
    let row = library.store.card(&id, None).unwrap().unwrap();
    assert_eq!(json::parse(&row.data).unwrap().get("summary").as_str(), Some("About a page."));
}

#[test]
fn a_link_understood_from_its_picture_waits_for_it() {
    let library = Library::new();
    let id = library.share("https://example.com/sunset", "A sunset.");
    assert!(!library.shot.saw_tags.get(), "a card with few words is analysed after its picture");
    assert_eq!(library.tags(&id), ["reading"]);
    assert_eq!(library.model.chats.get(), 1);
}

#[test]
fn a_shared_link_gets_the_space_openai_picks() {
    let library = Library::new();
    runtime::set_env(HashMap::from([
        ("GEMINI_API_KEY".to_owned(), "test-key".to_owned()),
        ("OPENAI_API_KEY".to_owned(), "test-openai".to_owned()),
    ]));
    let db = library.store.db();
    db.execute("INSERT INTO spaces VALUES ('smart-1', 'tag-user', 'Smart', 'blue', 0, '2026-01-01', '{\"q\":\"x\"}')", &[]).unwrap();
    db.execute("INSERT INTO spaces VALUES ('garden-1', 'tag-user', 'Garden', 'green', 1, '2026-01-01', NULL)", &[]).unwrap();
    let words = "Long-form writing about gardens, soil and the slow work of growing vegetables through a cold spring. ".repeat(4);
    let id = library.share("https://example.com/gardens", &words);
    let row = library.store.card(&id, None).unwrap().unwrap();
    let extras = json::parse(&row.extras).unwrap();
    assert_eq!(extras.get("suggestedSpace").get("id").as_str(), Some("garden-1"), "smart spaces are never offered");
    assert_eq!(extras.get("suggestedSpace").get("p").as_f64(), Some(0.8));
    runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned())]));
}

#[test]
fn a_repeat_save_folds_into_the_earlier_card() {
    let library = Library::new();
    runtime::set_env(HashMap::from([
        ("GEMINI_API_KEY".to_owned(), "test-key".to_owned()),
        ("OPENAI_API_KEY".to_owned(), "test-openai".to_owned()),
    ]));
    let db = library.store.db();
    db.execute("INSERT INTO spaces VALUES ('garden-1', 'tag-user', 'Garden', 'green', 1, '2026-01-01', NULL)", &[]).unwrap();
    let words = "Long-form writing about gardens, soil and the slow work of growing vegetables through a cold spring. ".repeat(4);
    let first = library.share("https://example.com/gardens", &words);
    // The same article at another address; every embedding in this test is the same, and OpenAI says 0.9.
    let made = library.writer.run(|writer| writer.create(&obj! { "content" => "https://example.com/gardens?ref=feed" }.into(), &Value::Undefined)).unwrap();
    let second = made.get("body").get("id").as_str().unwrap().to_owned();
    library.writer.run(|writer| writer.add_tag(&second, &Value::from("to-plant"))).unwrap();
    library.writer.run(|writer| writer.membership("garden-1", &second, false)).unwrap();
    library.capture(&second, &words);

    let repeat = library.store.card(&second, None).unwrap().unwrap();
    assert!(repeat.deleted_ns.is_some(), "the repeat save is in the trash");
    assert_eq!(json::parse(&repeat.extras).unwrap().get("mergedInto").as_str(), Some(first.as_str()));
    assert!(library.tags(&first).contains(&"to-plant".to_owned()), "the repeat's own tag moved over");
    let spaces = db.query("SELECT space_id FROM space_cards WHERE card_id = ? AND excluded = 0", &[first.as_str().into()]).unwrap();
    assert_eq!(spaces.len(), 1, "the repeat's space moved over");
    let earlier = library.store.card(&first, None).unwrap().unwrap();
    assert!(earlier.deleted_ns.is_none());
    assert!(earlier.bumped_ns > repeat.created_ns, "the earlier card comes to the top");

    library.writer.run(|writer| writer.trash(&second, true)).unwrap();
    let restored = library.store.card(&second, None).unwrap().unwrap();
    assert!(restored.deleted_ns.is_none());
    assert!(json::parse(&restored.extras).unwrap().get("mergedInto").is_undefined(), "a restored repeat is its own card again");
    runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned())]));
}

#[test]
fn notes_join_when_both_cards_have_one() {
    let library = Library::new();
    let note = |text: &str| -> Value { obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => text })] })] }.into() };
    let make = |url: &str, text: &str| -> String {
        let made = library.writer.run(|writer| writer.create(&obj! { "content" => url }.into(), &Value::Undefined)).unwrap();
        let id = made.get("body").get("id").as_str().unwrap().to_owned();
        library.writer.run(|writer| writer.prose(&id, &note(text), "note")).unwrap();
        id
    };
    let earlier = make("https://example.com/a", "first thought");
    let repeat = make("https://example.com/b", "second thought");
    assert!(library.writer.run(|writer| writer.merge_into(&repeat, &earlier)).unwrap());
    let data = json::parse(&library.store.card(&earlier, None).unwrap().unwrap().data).unwrap();
    let texts: Vec<String> = data.get("note").get("prose").get("content").as_array().unwrap().iter().map(|block| block.get("content").get("0").get("text").to_js_string()).collect();
    assert_eq!(texts, ["first thought", "second thought"]);
    assert!(!library.writer.run(|writer| writer.merge_into(&repeat, &earlier)).unwrap(), "a trashed repeat is not merged twice");
}
