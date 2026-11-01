//! Names for notes over a synthetic library and a scripted chat model: a new note takes the model's name, a name the
//! user gave is kept, and the sweep names the notes saved before, newest first, within its limit.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

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
use mm_core::{Result, Value, obj};
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

// The chat model names each note by the first word of its text, as `titles` says; a word it has no name for makes it
// fail. Every chat request's text is kept.
struct Model {
    titles: HashMap<&'static str, &'static str>,
    asked: RefCell<Vec<String>>,
}

impl Model {
    fn new(titles: &[(&'static str, &'static str)]) -> Rc<Self> {
        Rc::new(Self { titles: titles.iter().copied().collect(), asked: RefCell::default() })
    }

    fn answer(body: String) -> Result<Response> {
        Ok(Response::new(200, Headers::from([("content-type", "application/json".to_owned())]), Body::Bytes(body.into_bytes())))
    }
}

#[async_trait(?Send)]
impl Fetch for Model {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = String::from_utf8(request.take_body().bytes().await?).unwrap();
        let vector = || Value::from(obj! { "values" => (0..768).map(|n| Value::from(if n == 0 { 1.0 } else { 0.0 })).collect::<Vec<_>>() });
        if request.url.as_str().ends_with(":embedContent") {
            return Self::answer(json::stringify(&obj! { "embedding" => vector() }.into()).unwrap().unwrap());
        }
        if request.url.as_str().ends_with(":batchEmbedContents") {
            let count = json::parse(&body).unwrap().get("requests").as_array().map_or(0, Vec::len);
            return Self::answer(json::stringify(&obj! { "embeddings" => (0..count).map(|_| vector()).collect::<Vec<_>>() }.into()).unwrap().unwrap());
        }
        assert!(request.url.as_str().ends_with("/chat/completions"), "{}", request.url);
        let sent = json::parse(&body).unwrap();
        let prompt = sent.get("messages").at(1).get("content").at(0).get("text").to_js_string();
        let words = prompt.split("Text: ").nth(1).unwrap_or("").to_owned();
        self.asked.borrow_mut().push(prompt);
        let first = words.split_whitespace().next().unwrap_or("").trim_end_matches(':');
        let Some(title) = self.titles.get(first) else {
            return Ok(Response::new(400, Headers::new(), Body::Bytes(b"refused".to_vec())));
        };
        let card = obj! { "title" => *title, "summary" => format!("About {first}."), "tags" => vec![Value::from("ideas")] };
        let content = json::stringify(&card.into()).unwrap().unwrap();
        let choices = vec![Value::from(obj! { "message" => obj! { "content" => content } })];
        Self::answer(json::stringify(&obj! { "choices" => choices }.into()).unwrap().unwrap())
    }
}

struct Unused;

#[async_trait(?Send)]
impl Media for Unused {
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
        runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned())]));
        let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
        let store = Rc::new(Store::new(db.clone()).unwrap());
        db.execute("INSERT INTO users VALUES ('note-user', 'note-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
        let writer = Rc::new(Writer::new(store.clone(), |_, _| {}).unwrap());
        Self { db, store, writer }
    }

    // A note of `words`, saved as the app saves one.
    fn note(&self, words: &str) -> String {
        self.save(obj! { "content" => words }.into())
    }

    fn save(&self, body: Value) -> String {
        let made = self.writer.run(|writer| writer.create(&body, &Value::Undefined)).unwrap();
        made.get("body").get("id").as_str().unwrap().to_owned()
    }

    fn title(&self, id: &str) -> Option<String> {
        let row = self.store.card(id, None).unwrap().unwrap();
        json::parse(&row.data).unwrap().get("title").as_str().map(str::to_owned)
    }

    fn jobs(&self, model: Rc<Model>) -> Enrichment {
        Enrichment::new(EnrichmentOptions {
            store: self.store.clone(),
            writer: self.writer.clone(),
            media: Rc::new(Unused),
            resource: Rc::new(Unused),
            screenshot: Rc::new(Unused),
            fetch: model,
            offline: false,
            readonly: false,
        })
    }
}

#[test]
fn a_new_note_takes_the_models_name_and_keeps_one_the_user_gave() {
    let library = Library::new();
    let model = Model::new(&[("groceries", "\"Weekend groceries for the cabin.\""), ("plans", "Summer travel plans")]);
    let jobs = library.jobs(model.clone());
    let note = library.note("groceries: eggs, coffee, two lemons");
    let named = library.save(obj! { "content" => "plans for the summer", "title" => "Trips" }.into());
    jobs.schedule(&note, false);
    jobs.schedule(&named, false);
    block_on(jobs.drain());
    assert_eq!(library.title(&note).as_deref(), Some("Weekend groceries for the cabin"), "quotes and the full stop go");
    assert_eq!(library.title(&named).as_deref(), Some("Trips"), "a name chosen when saving is kept");
    assert!(model.asked.borrow()[0].starts_with("Title: (none)\nType: Note\n"), "{}", model.asked.borrow()[0]);

    library.writer.run(|writer| writer.patch(&note, &obj! { "title" => "Shopping" }.into())).unwrap();
    jobs.schedule(&note, false);
    block_on(jobs.drain());
    assert_eq!(library.title(&note).as_deref(), Some("Shopping"), "a name the user gave later is kept too");
}

#[test]
fn a_name_the_model_gave_follows_the_note_when_its_words_change() {
    let library = Library::new();
    let model = Model::new(&[("groceries", "Weekend groceries"), ("plans", "Summer travel plans")]);
    let jobs = library.jobs(model);
    let note = library.note("groceries: eggs, coffee");
    jobs.schedule(&note, false);
    block_on(jobs.drain());
    assert_eq!(library.title(&note).as_deref(), Some("Weekend groceries"));
    let prose = json::parse(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"plans for the summer"}]}]}"#).unwrap();
    library.writer.run(|writer| writer.prose(&note, &prose, "prose")).unwrap();
    jobs.schedule(&note, false);
    block_on(jobs.drain());
    assert_eq!(library.title(&note).as_deref(), Some("Summer travel plans"));
}

#[test]
fn the_sweep_names_the_notes_saved_before_newest_first_and_stops_at_its_limit() {
    let library = Library::new();
    let model = Model::new(&[("one", "First idea"), ("three", "Third idea"), ("four", "Fourth idea")]);
    let ids: Vec<String> = ["one", "two", "three", "four"].iter().map(|word| library.note(&format!("{word} thing to remember"))).collect();
    let kept = library.save(obj! { "content" => "four more", "title" => "Mine" }.into());
    let link = library.save(obj! { "content" => "https://example.com/page" }.into());
    library.db.execute("UPDATE cards SET extras = json_remove(extras, '$.capturePending') WHERE id = ?", &[link.as_str().into()]).unwrap();
    let jobs = library.jobs(model.clone());
    jobs.name_notes(2);
    block_on(jobs.drain());
    assert_eq!(library.title(&ids[3]).as_deref(), Some("Fourth idea"), "newest first");
    assert_eq!(library.title(&ids[2]).as_deref(), Some("Third idea"));
    assert_eq!(library.title(&ids[1]), None, "two at most");
    assert_eq!(model.asked.borrow().len(), 2);
    jobs.name_notes(2);
    block_on(jobs.drain());
    assert_eq!(model.asked.borrow().len(), 3, "a failure ends this call's naming");
    assert_eq!(library.title(&ids[1]), None);
    assert_eq!(library.title(&ids[0]), None);
    jobs.name_notes(usize::MAX);
    block_on(jobs.drain());
    assert_eq!(library.title(&ids[0]).as_deref(), Some("First idea"), "the note that failed waits for the next queue");
    assert_eq!(library.title(&ids[1]), None);
    assert_eq!(library.title(&kept).as_deref(), Some("Mine"));
    assert_eq!(library.title(&link), None, "only notes are named");
    let asked = model.asked.borrow().len();
    assert_eq!(asked, 4);
    jobs.name_notes(usize::MAX);
    block_on(jobs.drain());
    assert_eq!(model.asked.borrow().len(), asked, "a finished sweep does not start again");
}
