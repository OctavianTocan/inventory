//! The app's push (`POST /app/v1/push`) over a synthetic library: changes applied in order and once, changes the
//! library turns down, one record per phone, spaces and cards made on the phone, and malformed pushes.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::app_push::{PUSH_LIMIT, Pushed, push};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::Sql;
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

struct Library {
    store: Rc<Store>,
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
    db.execute("INSERT INTO users VALUES ('push-user', 'push-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let ids = ["First", "Second"]
        .into_iter()
        .map(|title| writer.run(|writer| writer.create(&obj! { "title" => title }.into(), &Value::Undefined)).unwrap().get("body").get("id").as_str().unwrap().to_owned())
        .collect();
    let search = Rc::new(Search::new(store.clone(), None, false).unwrap());
    Library { api: AppApi::new(store.clone(), search, |text| text.to_owned()), store, writer, ids }
}

fn change(id: f64, name: &str, args: Value) -> Value {
    obj! { "id" => id, "name" => name, "args" => args }.into()
}

fn pushed(library: &Library, client: &str, changes: Vec<Value>) -> Result<Pushed> {
    push(&library.store, &library.writer, &obj! { "clientId" => client, "changes" => Value::Array(changes) }.into())
}

fn send(library: &Library, client: &str, changes: Vec<Value>) -> Result<Value> {
    pushed(library, client, changes).map(|pushed| pushed.answer)
}

fn card(library: &Library, id: &str) -> Value {
    let address = url::parse(&format!("/app/v1/cards/{id}"), Some("http://localhost")).unwrap();
    let response = block_on(library.api.handle("GET", &address)).unwrap().unwrap();
    json::parse(&String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()).unwrap()
}

fn tag_names(card: &Value) -> Vec<String> {
    card.get("tags").as_array().map(|tags| tags.iter().map(|tag| tag.get("name").as_str().unwrap().to_owned()).collect()).unwrap_or_default()
}

fn numbers(value: &Value) -> Vec<f64> {
    value.as_array().unwrap().iter().map(|item| item.as_f64().unwrap()).collect()
}

#[test]
fn changes_apply_in_order_and_a_push_sent_again_changes_nothing() {
    let library = library();
    let [first, second] = [library.ids[0].as_str(), library.ids[1].as_str()];
    let space = library.writer.run(|writer| writer.create_space(&obj! { "name" => "Reading" }.into())).unwrap().get("id").as_str().unwrap().to_owned();
    let changes = vec![
        change(1.0, "pin", obj! { "cardId" => first, "pinned" => true }.into()),
        change(2.0, "note", obj! { "cardId" => second, "text" => "line one\nline two" }.into()),
        change(3.0, "add-tag", obj! { "cardId" => second, "name" => "later" }.into()),
        change(4.0, "add-to-space", obj! { "cardId" => first, "spaceId" => space.as_str() }.into()),
    ];
    let answer = send(&library, "phone-a", changes.clone()).unwrap();
    assert_eq!(answer.get("lastChangeId").as_f64(), Some(4.0));
    assert!(numbers(answer.get("refused")).is_empty());
    let tag = card(&library, second).get("tags").at(0).get("id").as_str().unwrap().to_owned();
    assert_eq!(answer.get("tagIds").get("3").as_str(), Some(tag.as_str()), "the push answers the id of the tag it added");
    assert_eq!(card(&library, first).get("flags").as_f64().map(|flags| flags as i64 & 512), Some(512));
    assert_eq!(card(&library, first).get("spaces").at(0).as_str(), Some(space.as_str()));
    assert_eq!(card(&library, second).get("note").as_str(), Some("line one\nline two"));

    library.writer.run(|writer| writer.remove_tag(&tag)).unwrap();
    library.writer.run(|writer| writer.flag(first, 512, false, &Value::Undefined)).unwrap();
    let again = send(&library, "phone-a", changes).unwrap();
    assert_eq!(again.get("lastChangeId").as_f64(), Some(4.0));
    assert!(again.get("tagIds").as_object().unwrap().is_empty());
    assert!(tag_names(&card(&library, second)).is_empty(), "a change already applied is not applied again");
    assert_eq!(card(&library, first).get("flags").as_f64().map(|flags| flags as i64 & 512), Some(0));

    let next = send(&library, "phone-a", vec![change(5.0, "trash", obj! { "cardId" => second }.into())]).unwrap();
    assert_eq!(next.get("lastChangeId").as_f64(), Some(5.0));
    assert!(card(&library, second).get("deleted").as_str().is_some());
}

#[test]
fn a_change_the_library_turns_down_is_handled_and_the_rest_still_apply() {
    let library = library();
    let first = library.ids[0].as_str();
    let answer = send(
        &library,
        "phone-a",
        vec![
            change(1.0, "trash", obj! { "cardId" => "gone" }.into()),
            change(2.0, "rename", obj! { "cardId" => first }.into()),
            change(3.0, "add-tag", obj! { "cardId" => first, "name" => " " }.into()),
            change(4.0, "add-tag", obj! { "cardId" => first, "name" => "kept" }.into()),
        ],
    )
    .unwrap();
    assert_eq!(numbers(answer.get("refused")), vec![1.0, 2.0, 3.0]);
    assert_eq!(answer.get("lastChangeId").as_f64(), Some(4.0));
    assert_eq!(tag_names(&card(&library, first)), vec!["kept"]);
    let again = send(&library, "phone-a", vec![change(1.0, "trash", obj! { "cardId" => "gone" }.into())]).unwrap();
    assert!(numbers(again.get("refused")).is_empty(), "a change turned down once counts as handled");
}

#[test]
fn each_phone_keeps_its_own_count() {
    let library = library();
    let [first, second] = [library.ids[0].as_str(), library.ids[1].as_str()];
    send(&library, "phone-a", vec![change(7.0, "add-tag", obj! { "cardId" => first, "name" => "a" }.into())]).unwrap();
    let other = send(&library, "phone-b", vec![change(1.0, "add-tag", obj! { "cardId" => second, "name" => "b" }.into())]).unwrap();
    assert_eq!(other.get("lastChangeId").as_f64(), Some(1.0));
    assert_eq!(tag_names(&card(&library, second)), vec!["b"]);
    let quiet = send(&library, "phone-a", Vec::new()).unwrap();
    assert_eq!(quiet.get("lastChangeId").as_f64(), Some(7.0));
}

#[test]
fn a_space_made_on_the_phone_keeps_the_phones_id() {
    let library = library();
    let first = library.ids[0].as_str();
    let id = "0a1b2c3d4e5f4a6b8c7d9e0f1a2b3c4d";
    let answer = send(
        &library,
        "phone-a",
        vec![
            change(1.0, "create-space", obj! { "spaceId" => id, "name" => "Reading", "color" => "#75ffc0" }.into()),
            change(2.0, "add-to-space", obj! { "cardId" => first, "spaceId" => id }.into()),
            change(3.0, "create-space", obj! { "spaceId" => id, "name" => "Again" }.into()),
            change(4.0, "create-space", obj! { "spaceId" => "gl0000000001", "name" => "Theirs" }.into()),
        ],
    )
    .unwrap();
    assert_eq!(numbers(answer.get("refused")), vec![3.0, 4.0], "an id in use, or one shaped like the library's own, is turned down");
    let space = library.store.space(id).unwrap().unwrap();
    assert_eq!((space.name.as_str(), space.color.as_str()), ("Reading", "#75ffc0"));
    assert_eq!(card(&library, first).get("spaces").at(0).as_str(), Some(id));
    let made = library.writer.run(|writer| writer.create_space(&obj! { "name" => "Next" }.into())).unwrap();
    assert_eq!(made.get("id").as_str(), Some("fixture0000gl000000001"), "the library's own ids keep their own count");

    send(&library, "phone-a", vec![change(5.0, "delete-space", obj! { "spaceId" => id }.into())]).unwrap();
    assert!(library.store.space(id).unwrap().is_none());
    assert!(card(&library, first).get("spaces").as_array().is_none_or(Vec::is_empty));
}

#[test]
fn a_card_shared_on_the_phone_keeps_the_phones_id_unless_it_is_a_link_already_saved() {
    let library = library();
    let [link, again, words, other] =
        ["1a2b3c4d5e6f4a7b8c9d0e1f2a3b4c5d", "2a2b3c4d5e6f4a7b8c9d0e1f2a3b4c5d", "3a2b3c4d5e6f4a7b8c9d0e1f2a3b4c5d", "4a2b3c4d5e6f4a7b8c9d0e1f2a3b4c5d"];
    let create = |id: f64, card: &str, content: &str, note: Option<&str>| {
        let mut args: Value = obj! { "cardId" => card, "content" => content, "source" => "clipboard" }.into();
        if let Some(note) = note {
            args.set("note", note);
        }
        change(id, "create-card", args)
    };
    let done = pushed(
        &library,
        "phone-a",
        vec![
            create(1.0, link, "https://example.com/a", Some("first words")),
            create(2.0, again, "https://example.com/a", Some("other words")),
            change(3.0, "add-tag", obj! { "cardId" => again, "name" => "later" }.into()),
            create(4.0, words, "line one\nline two", None),
            create(5.0, words, "line three", None),
            create(6.0, "gl0000000001", "line four", None),
            create(7.0, other, "https://example.com/b", None),
            create(8.0, "5a2b3c4d5e6f4a7b8c9d0e1f2a3b4c5d", "https://example.com/b", Some("words for b")),
        ],
    )
    .unwrap();
    assert_eq!(done.created, vec![link, words, other], "only new cards are handed on for enrichment");
    let answer = done.answer;
    assert_eq!(numbers(answer.get("refused")), vec![5.0, 6.0], "an id in use, or one shaped like the library's own, is turned down");
    assert_eq!(answer.get("cardIds").get("1").as_str(), Some(link));
    assert_eq!(answer.get("cardIds").get("2").as_str(), Some(link), "a link already saved answers the saved card");
    let saved = card(&library, link);
    assert_eq!((saved.get("type").as_str(), saved.get("url").as_str()), (Some("Placeholder"), Some("https://example.com/a")));
    assert_eq!(saved.get("note").as_str(), Some("first words"), "a link saved before keeps its note");
    assert_eq!(tag_names(&saved), vec!["later"], "a change to the phone's id reaches the saved card");
    assert_eq!(card(&library, words).get("type").as_str(), Some("Note"));
    assert_eq!(card(&library, other).get("note").as_str(), Some("words for b"), "a link saved without a note takes the shared words");
}

#[test]
fn a_malformed_push_is_rejected_whole() {
    let library = library();
    let first = library.ids[0].as_str();
    let pin = |id: f64| change(id, "pin", obj! { "cardId" => first, "pinned" => true }.into());
    let status = |result: Result<Value>| result.unwrap_err().http_status();
    assert_eq!(status(push(&library.store, &library.writer, &obj! { "changes" => Value::Array(vec![pin(1.0)]) }.into()).map(|pushed| pushed.answer)), Some(400));
    assert_eq!(status(send(&library, "phone-a", vec![pin(1.0), pin(0.0)])), Some(400));
    assert_eq!(status(send(&library, "phone-a", vec![pin(1.5)])), Some(400));
    assert_eq!(status(send(&library, "phone-a", (1..=PUSH_LIMIT + 1).map(|id| pin(id as f64)).collect())), Some(400));
    assert_eq!(card(&library, first).get("flags").as_f64().map(|flags| flags as i64 & 512), Some(0), "nothing of a rejected push applies");
}

#[test]
fn a_picture_turned_on_the_phone_stays_turned_and_a_full_turn_takes_it_off() {
    let library = library();
    let first = library.ids[0].as_str();
    let answer = send(
        &library,
        "phone-a",
        vec![
            change(1.0, "rotate", obj! { "cardId" => first, "rotation" => 90.0 }.into()),
            change(2.0, "rotate", obj! { "cardId" => first, "rotation" => 45.0 }.into()),
            change(3.0, "rotate", obj! { "cardId" => first, "rotation" => "90" }.into()),
        ],
    )
    .unwrap();
    assert_eq!(numbers(answer.get("refused")), vec![2.0, 3.0], "only quarter turns are kept");
    assert_eq!(card(&library, first).get("rotation").as_f64(), Some(90.0));

    send(&library, "phone-a", vec![change(4.0, "rotate", obj! { "cardId" => first, "rotation" => -90.0 }.into())]).unwrap();
    assert_eq!(card(&library, first).get("rotation").as_f64(), Some(270.0), "a turn the other way is kept as the clockwise one");

    send(&library, "phone-a", vec![change(5.0, "rotate", obj! { "cardId" => first, "rotation" => 360.0 }.into())]).unwrap();
    assert!(card(&library, first).get("rotation").is_nullish(), "an upright picture sends no rotation");
}
