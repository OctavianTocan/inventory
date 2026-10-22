//! The app's search by words and meaning (`Search::hybrid`, `/app/v1/search?semantic=1`) over a synthetic library
//! whose vectors have chosen similarities to the query.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::{Hybrid, Search};
use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Result, Value, obj};
use mm_js::{json, url};
use mm_sqlite::Sqlite;

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        block_on(task);
    }

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }

    fn log(&self, _line: &str) {}
}

// Gemini stand-in: "quiet design" embeds on the first axis, anything else on an axis no card uses.
struct Gemini {
    calls: Rc<Cell<usize>>,
}

#[async_trait(?Send)]
impl Fetch for Gemini {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        self.calls.set(self.calls.get() + 1);
        let body = String::from_utf8(request.take_body().bytes().await?).expect("utf-8");
        let text = json::parse(&body)?.get("content").get("parts").at(0).get("text").to_js_string();
        let axis = if text.contains("quiet design") { 0 } else { 5 };
        let values: Vec<Value> = (0..768).map(|i| Value::Number(if i == axis { 1.0 } else { 0.0 })).collect();
        Ok(Response::new(200, Headers::new(), Body::Bytes(json::to_string(&obj! { "embedding" => obj! { "values" => values } }.into()).into_bytes())))
    }
}

struct Library {
    store: Rc<Store>,
    search: Rc<Search>,
    calls: Rc<Cell<usize>>,
    word: String,
    close: String,
    middle: String,
    far: String,
}

fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned()), ("GEMINI_BASE_URL".to_owned(), "http://127.0.0.1:1/v1beta".to_owned())]));
}

fn doc(text: &str) -> Value {
    obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => text })] })] }.into()
}

// A vector at `similarity` to the first axis, the rest on the second.
fn vector(similarity: f32) -> Vec<u8> {
    let mut values = vec![0f32; 768];
    values[0] = similarity;
    values[1] = (1.0 - similarity * similarity).sqrt();
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

fn library() -> Library {
    install();
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users VALUES ('hybrid-user', 'hybrid-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let card = |title: &str, text: &str, similarity: f32| -> String {
        let created = writer.run(|writer| writer.create(&obj! { "title" => title, "prose" => doc(text) }.into(), &Value::Undefined)).unwrap();
        let id = created.get("body").get("id").as_str().unwrap().to_owned();
        db.execute(
            "INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, 'hybrid-user', 768, ?)",
            &[id.as_str().into(), SqlValue::from(vector(similarity))],
        )
        .unwrap();
        id
    };
    let word = card("Quiet design", "rooms with soft light", 1.0);
    let close = card("Calm technology", "interfaces that keep to the periphery", 0.8);
    let middle = card("Slow software", "tools that wait for you", 0.7);
    let far = card("Gardening", "tomatoes and basil", 0.5);
    let calls = Rc::new(Cell::new(0));
    let search = Rc::new(Search::new(store.clone(), Some(Rc::new(Gemini { calls: calls.clone() })), false).unwrap());
    Library { store, search, calls, word, close, middle, far }
}

fn ids(result: mm_core::search::SearchResult) -> Vec<String> {
    result.matches.into_iter().map(|found| found.id).collect()
}

#[test]
fn words_alone_find_only_the_word_match() {
    let library = library();
    assert_eq!(ids(block_on(library.search.search("quiet design", &obj! {}.into())).unwrap()), vec![library.word.clone()]);
    assert_eq!(library.calls.get(), 0);
}

#[test]
fn meaning_joins_close_cards_above_the_floor() {
    let library = library();
    let found = ids(block_on(library.search.hybrid("quiet design", &Hybrid::default())).unwrap());
    assert_eq!(found, vec![library.word.clone(), library.close.clone(), library.middle.clone()]);
    let lower = ids(block_on(library.search.hybrid("quiet design", &Hybrid { floor: 0.4, ..Hybrid::default() })).unwrap());
    assert_eq!(lower, vec![library.word.clone(), library.close.clone(), library.middle.clone(), library.far.clone()]);
}

#[test]
fn the_cap_keeps_the_nearest_cards_without_a_word() {
    let library = library();
    let found = ids(block_on(library.search.hybrid("quiet design", &Hybrid { take: 1, ..Hybrid::default() })).unwrap());
    assert_eq!(found, vec![library.word.clone(), library.close.clone()]);
    let none = ids(block_on(library.search.hybrid("quiet design", &Hybrid { take: 0, ..Hybrid::default() })).unwrap());
    assert_eq!(none, vec![library.word.clone()]);
}

#[test]
fn field_terms_still_filter() {
    let library = library();
    let found = ids(block_on(library.search.hybrid("quiet design -title:calm", &Hybrid::default())).unwrap());
    assert_eq!(found, vec![library.word.clone(), library.middle.clone()]);
}

#[test]
fn a_query_is_embedded_once_and_the_web_search_is_unchanged() {
    let library = library();
    block_on(library.search.hybrid("quiet design", &Hybrid::default())).unwrap();
    block_on(library.search.hybrid("quiet design", &Hybrid::default())).unwrap();
    let meaning = ids(block_on(library.search.meaning("quiet design")).unwrap());
    assert_eq!(meaning, vec![library.word.clone(), library.close.clone(), library.middle.clone(), library.far.clone()]);
    assert_eq!(library.calls.get(), 1);
    let options: Value = obj! { "semantic" => true }.into();
    let web = ids(block_on(library.search.search("quiet design", &options)).unwrap());
    block_on(library.search.search("quiet design", &options)).unwrap();
    assert_eq!(library.calls.get(), 3);
    assert_eq!(web, vec![library.word.clone(), library.close.clone(), library.middle.clone(), library.far.clone()]);
}

#[test]
fn without_gemini_the_app_search_is_word_search() {
    let library = library();
    let offline = Search::new(library.store.clone(), None, false).unwrap();
    let found = ids(block_on(offline.hybrid("quiet design", &Hybrid::default())).unwrap());
    assert_eq!(found, vec![library.word.clone()]);
}

#[test]
fn the_app_asks_for_meaning_with_semantic_1() {
    let library = library();
    let api = AppApi::new(library.store.clone(), library.search.clone(), |text| text.to_owned());
    let cards = |target: &str| -> Vec<String> {
        let address = url::parse(target, Some("http://localhost")).unwrap();
        let response = block_on(api.handle("GET", &address)).unwrap().unwrap();
        assert_eq!(response.status, 200);
        let body = json::parse(&String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()).unwrap();
        body.get("cards").as_array().unwrap().iter().map(|card| card.get("id").as_str().unwrap().to_owned()).collect()
    };
    assert_eq!(cards("/app/v1/search?q=quiet%20design"), vec![library.word.clone()]);
    assert_eq!(cards("/app/v1/search?q=quiet%20design&semantic=1"), vec![library.word.clone(), library.close.clone(), library.middle.clone()]);
    assert_eq!(cards("/app/v1/search?q=quiet%20design&semantic=1&limit=2"), vec![library.word.clone(), library.close.clone()]);
    assert_eq!(library.calls.get(), 1);
}
