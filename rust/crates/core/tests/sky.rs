//! Back today and the sky (`GET /app/v1/sky`, the push's `visit`) over a synthetic library: three old finds a day by
//! meaning, kept all day; the constellation formed around a recent save; visits that light it and finish it; and the
//! Decisions call that picks its figure.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard, Once, PoisonError};

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::app_push::push;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sky::FIGURES;
use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Result, Value, obj};
use mm_js::{json, url};
use mm_sqlite::Sqlite;

const DAY_MS: f64 = 86_400_000.0;

static LOCK: Mutex<()> = Mutex::new(());

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

// Answers the figure question with `choice` and keeps every request body.
struct Decisions {
    choice: String,
    bodies: RefCell<Vec<Value>>,
}

#[async_trait(?Send)]
impl Fetch for Decisions {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = json::parse(&String::from_utf8(request.take_body().bytes().await?).unwrap()).unwrap();
        self.bodies.borrow_mut().push(body);
        let value = &self.choice;
        let text = format!(
            r#"{{"answers":[{{"type":"choice","name":"figure","choice":"{value}","probabilities":[{{"value":"{value}","probability":0.9}}],"confidence":0.9}}]}}"#
        );
        Ok(Response::new(200, Headers::from([("content-type", "application/json".to_owned())]), Body::Bytes(text.into_bytes())))
    }
}

struct Library {
    db: Rc<Sqlite>,
    store: Rc<Store>,
    writer: Writer,
    api: AppApi,
}

fn setup(vars: &[(&str, &str)]) -> MutexGuard<'static, ()> {
    static ONCE: Once = Once::new();
    let guard = LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(vars.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect::<HashMap<_, _>>());
    guard
}

fn library(decisions: Option<Rc<dyn Fetch>>) -> Library {
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users VALUES ('sky-user', 'sky-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let search = Rc::new(Search::new(store.clone(), None, false).unwrap());
    let api = AppApi::new(store.clone(), search, |text| text.to_owned()).with_sky(decisions, false);
    Library { db, store, writer, api }
}

// A unit vector along `axis`, leaning a little towards `lean` by `amount`.
fn direction(axis: usize, lean: usize, amount: f32) -> Vec<u8> {
    let mut vector = vec![0f32; 768];
    vector[axis] = 1.0;
    vector[lean] += amount;
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    vector.iter().flat_map(|x| (x / norm).to_le_bytes()).collect()
}

// A card saved `days` ago, with an embedding when given one.
fn saved(library: &Library, title: &str, days: f64, vector: Option<Vec<u8>>) -> String {
    let id = library.writer.run(|writer| writer.create(&obj! { "title" => title }.into(), &Value::Undefined)).unwrap().get("body").get("id").as_str().unwrap().to_owned();
    let ns = SqlValue::big(((runtime::now_ms() - days * DAY_MS) as i128) * 1_000_000);
    library.db.execute("UPDATE cards SET created_ns = ?, bumped_ns = ? WHERE id = ?", &[ns.clone(), ns, id.as_str().into()]).unwrap();
    if let Some(bytes) = vector {
        library.db.execute("INSERT INTO embeddings VALUES (?, 'sky-user', 768, ?)", &[id.as_str().into(), bytes.into()]).unwrap();
    }
    id
}

fn get(library: &Library, path: &str) -> (u16, Value) {
    let address = url::parse(path, Some("http://localhost")).unwrap();
    let response = block_on(library.api.handle("GET", &address)).unwrap().unwrap();
    (response.status, json::parse(&String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()).unwrap())
}

fn sky(library: &Library, day: &str) -> Value {
    let (status, body) = get(library, &format!("/app/v1/sky?day={day}"));
    assert_eq!(status, 200, "{body:?}");
    body
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().unwrap().iter().map(|item| item.as_str().unwrap().to_owned()).collect()
}

fn visit(library: &Library, change: f64, card: &str) -> Value {
    let change = obj! { "id" => change, "name" => "visit", "args" => obj! { "cardId" => card } };
    push(&library.store, &library.writer, &obj! { "clientId" => "phone", "changes" => mm_core::arr![change] }.into()).unwrap().answer
}

// Six old finds about one thing (the first five nearest to it), four about another, and a save from yesterday about the
// first: the save seeds a constellation of the five.
struct Stocked {
    library: Library,
    seed: String,
    near: Vec<String>,
    other: Vec<String>,
}

fn stocked(decisions: Option<Rc<dyn Fetch>>) -> Stocked {
    let library = library(decisions);
    let near: Vec<String> = (0..6).map(|at| saved(&library, &format!("Harbour {at}"), 200.0 + at as f64, Some(direction(0, 10 + at, 0.1 + 0.1 * at as f32)))).collect();
    let other: Vec<String> = (0..4).map(|at| saved(&library, &format!("Recipe {at}"), 300.0, Some(direction(100, 110 + at, 0.1)))).collect();
    let seed = saved(&library, "Ferry timetable", 1.0, Some(direction(0, 50, 0.05)));
    Stocked { library, seed, near, other }
}

#[test]
fn picks_three_old_finds_by_meaning_and_keeps_them_all_day() {
    let _guard = setup(&[]);
    let Stocked { library, seed, near, other } = stocked(None);
    let body = sky(&library, "2026-11-05");
    let active = body.get("active");
    assert_eq!(strings(active.get("cards")), [std::slice::from_ref(&seed), &near[..5]].concat(), "the seed, then its five nearest old finds");
    assert_eq!(strings(active.get("lit")), std::slice::from_ref(&seed), "only the seed is lit at first");
    let figure = FIGURES.iter().find(|(key, _, _)| active.get("figure").as_str() == Some(*key)).expect("a known figure");
    assert_eq!(active.get("name").as_str(), Some(figure.1));
    assert!(active.get("completed") == &Value::Null);

    let back = strings(body.get("back"));
    assert_eq!(back, [near[0].clone(), near[1].clone(), near[5].clone()], "two dark stars, then the nearest old find outside the constellation");
    assert!(!back.contains(&seed) && back.iter().all(|id| !other.contains(id)));
    assert_eq!(body.get("day").as_str(), Some("2026-11-05"));
    assert!(body.get("done").as_array().unwrap().is_empty() && body.get("stars").as_array().unwrap().is_empty());

    assert_eq!(json::stringify(&sky(&library, "2026-11-05")).unwrap(), json::stringify(&body).unwrap(), "the same picks all day");
}

#[test]
fn visits_light_stars_and_finish_the_constellation() {
    let _guard = setup(&[]);
    let Stocked { library, seed, near, other } = stocked(None);
    let first = strings(sky(&library, "2026-11-05").get("back"));
    for (at, id) in near[..4].iter().enumerate() {
        assert!(visit(&library, at as f64 + 1.0, id).get("refused").as_array().unwrap().is_empty());
    }
    let body = sky(&library, "2026-11-05");
    assert_eq!(strings(body.get("active").get("lit")), [std::slice::from_ref(&seed), &near[..4]].concat());
    assert_eq!(strings(body.get("back")), first, "visiting keeps the day's picks");

    visit(&library, 5.0, &near[4]);
    let body = sky(&library, "2026-11-05");
    assert!(body.get("active") == &Value::Null, "no recent save is left to seed another");
    let done = body.get("done").as_array().unwrap();
    assert_eq!(done.len(), 1);
    assert_eq!(strings(done[0].get("lit")), [std::slice::from_ref(&seed), &near[..5]].concat());
    assert!(done[0].get("completed").is_string());
    let stars: Vec<String> = body.get("stars").as_array().unwrap().iter().map(|star| star.get("id").to_js_string()).collect();
    assert_eq!(stars.len(), 5);

    let next = strings(sky(&library, "2026-11-06").get("back"));
    assert_eq!(next.len(), 3);
    assert!(next.iter().all(|id| other.contains(id)), "finds opened rest, and yesterday's picks wait a month: {next:?}");
}

#[test]
fn a_find_in_the_trash_no_longer_holds_its_constellation_back() {
    let _guard = setup(&[]);
    let Stocked { library, near, .. } = stocked(None);
    sky(&library, "2026-11-05");
    library.writer.run(|writer| writer.trash(&near[0], false)).unwrap();
    for (at, id) in near[1..5].iter().enumerate() {
        visit(&library, at as f64 + 1.0, id);
    }
    let body = sky(&library, "2026-11-05");
    assert_eq!(body.get("done").as_array().unwrap().len(), 1);
    assert_eq!(strings(body.get("back")), [near[1].clone(), near[5].clone()], "a trashed pick leaves the band");
}

#[test]
fn without_embeddings_old_finds_come_in_a_daily_order() {
    let _guard = setup(&[]);
    let library = library(None);
    let old: Vec<String> = (0..6).map(|at| saved(&library, &format!("Old {at}"), 120.0, None)).collect();
    let fresh = saved(&library, "Fresh", 10.0, None);
    let body = sky(&library, "2026-11-05");
    let back = strings(body.get("back"));
    assert_eq!(back.len(), 3);
    assert!(back.iter().all(|id| old.contains(id)) && !back.contains(&fresh));
    assert!(body.get("active") == &Value::Null, "a constellation needs meaning to form");
}

#[test]
fn decisions_picks_an_unused_figure() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai"), ("OPENAI_BASE_URL", "https://openai.example/v1/")]);
    let fake = Rc::new(Decisions { choice: "lantern".into(), bodies: RefCell::default() });
    let Stocked { library, .. } = stocked(Some(fake.clone()));
    let active = sky(&library, "2026-11-05").get("active").clone();
    assert_eq!(active.get("figure").as_str(), Some("lantern"));
    assert_eq!(active.get("name").as_str(), Some("The Lantern"));
    let bodies = fake.bodies.borrow();
    assert_eq!(bodies.len(), 1, "asked once, when the constellation forms");
    let question = bodies[0].get("questions").get("0");
    assert_eq!(question.get("type").as_str(), Some("choice"));
    assert_eq!(question.get("choices").as_array().unwrap().len(), FIGURES.len());
    let input = bodies[0].get("input").to_js_string();
    assert!(input.contains("Ferry timetable") && input.contains("Harbour 0"), "{input}");
}

#[test]
fn refuses_bad_days_writes_and_unknown_finds() {
    let _guard = setup(&[]);
    let library = library(None);
    let (status, body) = get(&library, "/app/v1/sky?day=2026-02-30");
    assert_eq!((status, body.get("error").as_str()), (400, Some("bad-day")));
    let address = url::parse("/app/v1/sky", Some("http://localhost")).unwrap();
    assert_eq!(block_on(library.api.handle("POST", &address)).unwrap().unwrap().status, 405);
    let answer = visit(&library, 1.0, "missing");
    assert_eq!(answer.get("refused").as_array().unwrap().len(), 1);
    let (status, body) = get(&library, "/app/v1/sky");
    assert_eq!(status, 200);
    assert_eq!(body.get("day").to_js_string().len(), 10, "the day defaults to today");
}
