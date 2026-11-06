//! Usage analytics (`docs/analytics.md`): the fixed column layout, the owner's bounded SQL, the events push and
//! search record, and a library's daily totals kept in the sign-in database.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use futures_executor::block_on;
use mm_core::analytics::{self, Event, Origin, ROW_LIMIT, TEXT_LIMIT};
use mm_core::app_api::AppApi;
use mm_core::app_push::push;
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Value, obj};
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

fn recorded(store: &Store) -> Rc<RefCell<Vec<Event>>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let kept = events.clone();
    store.set_analytics(Some(Box::new(move |event| {
        kept.borrow_mut().push(event.clone())
    })));
    events
}

struct Library {
    db: Rc<Sqlite>,
    store: Rc<Store>,
    writer: Writer,
    api: AppApi,
    ids: Vec<String>,
}

fn library() -> Library {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::new());
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users VALUES ('stats-user', 'stats-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let ids = ["Lighthouse", "Harbour walk"]
        .into_iter()
        .map(|title| {
            writer
                .run(|writer| writer.create(&obj! { "title" => title }.into(), &Value::Undefined))
                .unwrap()
                .get("body")
                .get("id")
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    let search = Rc::new(Search::new(store.clone(), None, false).unwrap());
    Library {
        api: AppApi::new(store.clone(), search, |text| text.to_owned()),
        db,
        store,
        writer,
        ids,
    }
}

#[test]
fn every_field_lands_in_its_fixed_column() {
    let event = Event {
        name: "search".into(),
        screen: "gallery".into(),
        card_id: "card1".into(),
        card_kind: "Article".into(),
        words: "harbour".into(),
        detail: "words".into(),
        client: "android".into(),
        count: 2.0,
        results: 7.0,
        duration_ms: 31.0,
        status: 200.0,
    };
    let origin = Origin {
        library: "u-tavi-abc".into(),
        email: "tavi@example.com".into(),
        version: "0.7.0".into(),
        batch: "b1".into(),
    };
    let point = analytics::point(&event, &origin);
    // Analytics Engine columns are positional and phones keep sending old layouts: a move here breaks every query.
    assert_eq!(
        point.blobs,
        [
            "search",
            "0.7.0",
            "gallery",
            "card1",
            "Article",
            "harbour",
            "words",
            "b1",
            "android",
            "tavi@example.com"
        ]
    );
    assert_eq!(
        point.doubles,
        [analytics::LAYOUT_VERSION, 2.0, 7.0, 31.0, 200.0]
    );
    assert_eq!(point.index, analytics::index_of("u-tavi-abc"));
    assert_eq!(
        point.index.len(),
        32,
        "an index stays under Analytics Engine's 96 bytes"
    );
}

#[test]
fn long_text_is_cut() {
    let long = "x".repeat(TEXT_LIMIT + 50);
    let point = analytics::point(
        &Event {
            words: long,
            ..Event::named("search")
        },
        &Origin::default(),
    );
    assert_eq!(point.blobs[5].chars().count(), TEXT_LIMIT);
}

#[test]
fn owner_questions_are_one_bounded_select() {
    assert_eq!(
        analytics::bounded_select("SELECT blob1 FROM thymind_events;").unwrap(),
        "SELECT blob1 FROM thymind_events LIMIT 200"
    );
    assert_eq!(
        analytics::bounded_select(" select 1 limit 20 ").unwrap(),
        "select 1 limit 20"
    );
    assert!(analytics::bounded_select("WITH t AS (SELECT 1) SELECT * FROM t").is_ok());
    for refused in [
        "",
        "DROP TABLE x",
        "SELECT 1; SELECT 2",
        "SELECT 1 LIMIT 5000",
        "SELECT 1 LIMIT x",
    ] {
        let error = analytics::bounded_select(refused).unwrap_err();
        assert_eq!(error.http_status(), Some(400), "{refused:?} is refused");
    }
}

#[test]
fn answers_are_shortened_for_an_agent() {
    let data: Vec<Value> = (0..ROW_LIMIT + 5)
        .map(|n| obj! { "n" => n as f64, "words" => "w".repeat(TEXT_LIMIT * 2) }.into())
        .collect();
    let answer = obj! { "meta" => vec![Value::from(obj! { "name" => "n" }), obj! { "name" => "words" }.into()], "data" => data }.into();
    let short = analytics::shorten(&answer);
    assert_eq!(short.get("rows").as_array().unwrap().len(), ROW_LIMIT);
    assert_eq!(short.get("total").as_f64(), Some((ROW_LIMIT + 5) as f64));
    assert_eq!(short.get("cut").as_bool(), Some(true));
    assert_eq!(
        short.get("rows").at(0).get("words").as_str().unwrap().len(),
        TEXT_LIMIT
    );
    assert_eq!(short.get("columns").at(1).as_str(), Some("words"));
}

#[test]
fn routes_and_clients_are_named() {
    assert_eq!(
        analytics::route_of("/app/v1/cards/abc").as_deref(),
        Some("app/v1/cards")
    );
    assert_eq!(
        analytics::route_of("/app/v1/push").as_deref(),
        Some("app/v1/push")
    );
    assert_eq!(
        analytics::route_of("/objects/abc/restore").as_deref(),
        Some("objects")
    );
    assert_eq!(analytics::route_of("/media/x.jpg"), None);
    assert_eq!(analytics::route_of("/everything"), None);
    assert_eq!(analytics::client_of("okhttp/5.1.0"), "android");
    assert_eq!(analytics::client_of("ThyMind-MCP"), "mcp");
    assert_eq!(analytics::client_of("Mozilla/5.0"), "web");
    assert!(analytics::enabled(None));
    assert!(!analytics::enabled(Some("off")));
}

#[test]
fn push_records_each_change_and_search_records_its_words() {
    let library = library();
    let events = recorded(&library.store);
    let first = library.ids[0].as_str();
    let body = obj! {
        "clientId" => "phone-a",
        "changes" => vec![
            Value::from(obj! { "id" => 1.0, "name" => "pin", "args" => obj! { "cardId" => first, "pinned" => true } }),
            obj! { "id" => 2.0, "name" => "trash", "args" => obj! { "cardId" => "gone" } }.into(),
        ],
    };
    push(&library.store, &library.writer, &body.into()).unwrap();
    let address = url::parse("/app/v1/search?q=lighthouse", Some("http://localhost")).unwrap();
    block_on(library.api.handle("GET", &address))
        .unwrap()
        .unwrap();
    library.store.set_analytics(None);
    let events = events.borrow();
    assert_eq!(events.len(), 3, "{events:?}");
    assert_eq!(
        (
            events[0].name.as_str(),
            events[0].detail.as_str(),
            events[0].card_id.as_str(),
            events[0].status
        ),
        ("change", "pin", first, 200.0)
    );
    assert_eq!(
        (
            events[1].detail.as_str(),
            events[1].card_id.as_str(),
            events[1].status
        ),
        ("trash", "gone", 409.0)
    );
    assert_eq!(
        (
            events[2].name.as_str(),
            events[2].words.as_str(),
            events[2].results
        ),
        ("search", "lighthouse", 1.0)
    );
}

#[test]
fn nothing_is_recorded_without_a_sink() {
    let library = library();
    library.store.record(Event::named("request"));
}

#[test]
fn daily_totals_count_a_library_and_a_day_written_twice_is_kept_once() {
    let library = library();
    library
        .writer
        .run(|writer| writer.trash(&library.ids[1], false))
        .unwrap();
    library
        .writer
        .run(|writer| writer.create_space(&obj! { "name" => "Reading" }.into()))
        .unwrap();
    let totals = analytics::library_totals(&*library.db).unwrap();
    assert_eq!(totals.get("cards").as_f64(), Some(1.0));
    assert_eq!(totals.get("trashed").as_f64(), Some(1.0));
    assert_eq!(totals.get("spaces").as_f64(), Some(1.0));
    let kinds = totals.get("kinds").as_object().unwrap();
    assert_eq!(kinds.values().filter_map(Value::as_f64).sum::<f64>(), 1.0);

    let auth = Sqlite::open(":memory:", false).unwrap();
    analytics::put_daily(
        &auth,
        "2026-11-05",
        "u-a",
        "a@example.com",
        &obj! { "cards" => 1.0 }.into(),
    )
    .unwrap();
    analytics::put_daily(&auth, "2026-11-05", "u-a", "a@example.com", &totals).unwrap();
    analytics::put_daily(
        &auth,
        "2026-11-06",
        "u-b",
        "b@example.com",
        &obj! { "cards" => 9.0 }.into(),
    )
    .unwrap();
    let all = analytics::daily(&auth, "2026-11-01", "").unwrap();
    let days = all.get("days").as_array().unwrap();
    assert_eq!(days.len(), 2);
    assert_eq!(
        days[0].get("day").as_str(),
        Some("2026-11-06"),
        "newest day first"
    );
    assert_eq!(
        days[1].get("totals").get("trashed").as_f64(),
        Some(1.0),
        "a second write replaces the first"
    );
    let one = analytics::daily(&auth, "2026-11-01", "u-b").unwrap();
    assert_eq!(
        json::stringify(one.get("days").at(0).get("account"))
            .unwrap()
            .as_deref(),
        Some("\"b@example.com\"")
    );
}
