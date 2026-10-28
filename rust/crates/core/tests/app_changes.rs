//! The app's sync feed (`GET /app/v1/changes`) over a synthetic library: the first full read, paging, what each kind
//! of write puts in the feed, and a library whose log started again.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::{Sql, SqlExt};
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

struct Library {
    db: Rc<Sqlite>,
    api: AppApi,
    writer: Writer,
    ids: Vec<String>,
}

fn library() -> Library {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::new());
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users VALUES ('sync-user', 'sync-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let ids = ["First", "Second", "Third"]
        .into_iter()
        .map(|title| writer.run(|writer| writer.create(&obj! { "title" => title }.into(), &Value::Undefined)).unwrap().get("body").get("id").as_str().unwrap().to_owned())
        .collect();
    let search = Rc::new(Search::new(store.clone(), None, false).unwrap());
    Library { db, api: AppApi::new(store, search, |text| text.to_owned()), writer, ids }
}

fn get(library: &Library, target: &str) -> (u16, Value) {
    let address = url::parse(target, Some("http://localhost")).unwrap();
    let response = block_on(library.api.handle("GET", &address)).unwrap().unwrap();
    let body = json::parse(&String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()).unwrap();
    (response.status, body)
}

fn changes(library: &Library, query: &str) -> Value {
    let (status, body) = get(library, &format!("/app/v1/changes{query}"));
    assert_eq!(status, 200, "{body:?}");
    body
}

fn card_ids(body: &Value) -> Vec<String> {
    body.get("cards").as_array().unwrap().iter().map(|card| card.get("id").as_str().unwrap().to_owned()).collect()
}

fn removed(body: &Value) -> Vec<String> {
    body.get("removed").as_array().unwrap().iter().map(|id| id.as_str().unwrap().to_owned()).collect()
}

fn since(body: &Value) -> String {
    format!("?since={}", body.get("next").as_str().unwrap())
}

#[test]
fn the_first_read_lists_the_whole_library_and_the_spaces() {
    let library = library();
    assert!(library.db.one("SELECT 1 FROM sqlite_master WHERE name = 'app_sync'", &[]).unwrap().is_none(), "the log starts on first use");
    let all = changes(&library, "");
    assert_eq!(card_ids(&all), library.ids);
    assert_eq!(all.get("cards").at(0).get("title").as_str(), Some("First"));
    assert!(removed(&all).is_empty());
    assert_eq!(all.get("more"), &Value::Bool(false));
    assert_eq!(all.get("reset"), &Value::Bool(false));
    assert!(all.get("spaces").as_array().is_some());
    let quiet = changes(&library, &since(&all));
    assert!(card_ids(&quiet).is_empty() && removed(&quiet).is_empty());
    assert_eq!(quiet.get("next"), all.get("next"));
}

#[test]
fn pages_follow_the_log_and_only_the_last_carries_the_spaces() {
    let library = library();
    let first = changes(&library, "?limit=2");
    assert_eq!(card_ids(&first), library.ids[..2]);
    assert_eq!(first.get("more"), &Value::Bool(true));
    assert!(first.get("spaces").is_undefined());
    let rest = changes(&library, &format!("{}&limit=2", since(&first)));
    assert_eq!(card_ids(&rest), library.ids[2..]);
    assert_eq!(rest.get("more"), &Value::Bool(false));
    assert!(rest.get("spaces").as_array().is_some());
}

#[test]
fn every_kind_of_write_puts_the_card_in_the_feed_once() {
    let library = library();
    let start = changes(&library, "");
    let [first, second, third] = [&library.ids[0], &library.ids[1], &library.ids[2]];
    let space = library.writer.run(|writer| writer.create_space(&obj! { "name" => "Reading" }.into())).unwrap().get("id").as_str().unwrap().to_owned();
    library.writer.run(|writer| writer.add_tag(second, &"calm".into())).unwrap();
    library.writer.run(|writer| writer.membership(&space, second, false)).unwrap();
    library.writer.run(|writer| writer.flag(first, 512, true, &Value::Undefined)).unwrap();
    library.writer.run(|writer| writer.trash(third, false)).unwrap();
    let after = changes(&library, &since(&start));
    assert_eq!(card_ids(&after), vec![second.clone(), first.clone()]);
    assert_eq!(removed(&after), vec![third.clone()]);
    let tagged = after.get("cards").at(0);
    assert_eq!(tagged.get("tags").at(0).get("name").as_str(), Some("calm"));
    assert_eq!(tagged.get("spaces").at(0).as_str(), Some(space.as_str()));
    assert_eq!(tagged.get("spaceRanks").get(&space).as_f64(), Some(0.0), "the card's place in the space travels with it");
    assert_eq!(after.get("cards").at(1).get("pinPosition").as_f64(), Some(0.0));
    assert_eq!(after.get("spaces").at(0).get("name").as_str(), Some("Reading"));

    library.writer.run(|writer| writer.flag(second, 512, true, &Value::Undefined)).unwrap();
    let pinned = changes(&library, &since(&after));
    assert_eq!(card_ids(&pinned), vec![first.clone(), second.clone()], "pinning in front moves the other pin back");
    assert_eq!(pinned.get("cards").at(0).get("pinPosition").as_f64(), Some(1.0));
    assert_eq!(pinned.get("cards").at(1).get("pinPosition").as_f64(), Some(0.0));

    library.writer.run(|writer| writer.trash(third, true)).unwrap();
    library.db.execute("DELETE FROM cards WHERE id = ?", &[first.as_str().into()]).unwrap();
    let last = changes(&library, &since(&pinned));
    assert_eq!(card_ids(&last), vec![third.clone()]);
    assert_eq!(removed(&last), vec![first.clone()]);
}

#[test]
fn a_log_started_again_asks_the_phone_to_start_over() {
    let library = library();
    let old = changes(&library, "");
    library.db.exec_batch("DROP TABLE app_sync; DROP TABLE app_sync_epoch;").unwrap();
    for trigger in library.db.query("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'app_sync%'", &[]).unwrap() {
        library.db.exec_batch(&format!("DROP TRIGGER {}", trigger.text("name").unwrap())).unwrap();
    }
    let fresh = changes(&library, &since(&old));
    assert_eq!(fresh.get("reset"), &Value::Bool(true));
    assert_eq!(card_ids(&fresh), library.ids);
    assert_ne!(fresh.get("next"), old.get("next"));
}

#[test]
fn bad_tokens_and_limits_are_refused() {
    let library = library();
    for (query, error) in [("?since=nope", "bad-cursor"), ("?since=WyJ4Il0", "bad-cursor"), ("?limit=0", "bad-limit"), ("?limit=201", "bad-limit")] {
        let (status, body) = get(&library, &format!("/app/v1/changes{query}"));
        assert_eq!((status, body.get("error").as_str()), (400, Some(error)), "{query}");
    }
    let address = url::parse("/app/v1/changes", Some("http://localhost")).unwrap();
    assert_eq!(block_on(library.api.handle("POST", &address)).unwrap().unwrap().status, 405);
}

#[test]
fn cards_carry_their_place_in_each_space() {
    let library = library();
    let [first, second, third] = [&library.ids[0], &library.ids[1], &library.ids[2]];
    let space = library.writer.run(|writer| writer.create_space(&obj! { "name" => "Reading" }.into())).unwrap().get("id").as_str().unwrap().to_owned();
    for id in [third, first, second] {
        library.writer.run(|writer| writer.membership(&space, id, false)).unwrap();
    }
    let all = changes(&library, "");
    let rank = |id: &str| all.get("cards").as_array().unwrap().iter().find(|card| card.get("id").as_str() == Some(id)).unwrap().get("spaceRanks").get(&space).as_f64();
    assert_eq!([rank(third), rank(first), rank(second)], [Some(0.0), Some(1.0), Some(2.0)]);
    let (_, page) = get(&library, &format!("/app/v1/spaces/{space}/cards"));
    assert_eq!(card_ids(&page), vec![third.clone(), first.clone(), second.clone()], "the space lists its cards in the same order");
}

#[test]
fn a_token_from_before_the_cards_carried_space_ranks_starts_over() {
    let library = library();
    let current = changes(&library, "");
    let epoch = library.db.one("SELECT epoch FROM app_sync_epoch", &[]).unwrap().unwrap().text("epoch").unwrap().to_owned();
    let raw = json::stringify(&Value::Array(vec![Value::String(epoch), Value::String("3".to_owned())])).unwrap().unwrap();
    let old = changes(&library, &format!("?since={}", mm_js::base64::encode_url(raw.as_bytes())));
    assert_eq!(old.get("reset"), &Value::Bool(true));
    assert_eq!(card_ids(&old), library.ids);
    assert_eq!(old.get("next"), current.get("next"));
}
