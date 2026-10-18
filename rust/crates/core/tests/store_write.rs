//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.
//! comparing step results, broadcast events and the resulting rows.

mod common;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;

use common::{assert_same, calls, hosted_db, temp_copy, test_db};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Error, Result, Value, arr, obj};
use mm_js::date;
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:store-write";
const TABLES: [&str; 12] = ["embeddings", "cards", "changes", "annotations", "spaces", "space_cards", "pins", "shares", "revisions", "users", "tag_names", "card_content"];

static CLOCK: Mutex<()> = Mutex::new(());
static COUNTER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn random(size: usize) -> Vec<u8> {
    let next = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    vec![next; size]
}

fn resolve(db: &dyn Sql, arg: &Value) -> Result<Value> {
    let Some(text) = arg.as_str().and_then(|text| text.strip_prefix('@')) else { return Ok(arg.clone()) };
    let (kind, index) = text.split_once(':').unwrap_or((text, "0"));
    if kind == "baseline" {
        let rows = db.query("SELECT data, extras FROM cards ORDER BY id", &[])?;
        let row = &rows[index.parse::<usize>().unwrap()];
        return Ok(obj! { "data" => mm_js::json::parse(row.text("data").unwrap_or(""))?, "extras" => mm_js::json::parse(row.text("extras").unwrap_or(""))? }.into());
    }
    let sql = match kind {
        "card" => "SELECT id FROM cards ORDER BY id",
        "annotation" => "SELECT id FROM annotations ORDER BY id",
        "space" => "SELECT id FROM spaces ORDER BY id",
        "revision" => "SELECT id FROM revisions ORDER BY id",
        "share" => "SELECT id FROM shares ORDER BY id",
        _ => "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id",
    };
    let rows = db.query(sql, &[])?;
    Ok(rows.get(index.parse::<usize>().unwrap()).and_then(|row| row.text("id")).map_or("missing", |id| id).into())
}

fn id(args: &[Value], at: usize) -> String {
    args[at].to_js_string()
}

fn dispatch(writer: &Writer, name: &str, args: &[Value]) -> Result<Value> {
    let flag = |at: usize| args.get(at).is_some_and(Value::truthy);
    let missing = Value::Undefined;
    let arg = |at: usize| args.get(at).unwrap_or(&missing);
    match name {
        "create" => writer.create(arg(0), arg(1)),
        "patch" => writer.patch(&id(args, 0), arg(1)),
        "prose" => writer.prose(&id(args, 0), arg(1), args.get(2).and_then(Value::as_str).unwrap_or("prose")),
        "commit" => writer.commit(&id(args, 0), arg(1)),
        "addTag" => writer.add_tag(&id(args, 0), arg(1)),
        "removeTag" => writer.remove_tag(&id(args, 0)),
        "flag" => writer.flag(&id(args, 0), arg(1).to_number() as i32, flag(2), arg(3)),
        "reorderPins" => writer.reorder_pins(arg(0)),
        "trash" => writer.trash(&id(args, 0), flag(1)),
        "createSpace" => writer.create_space(arg(0)),
        "patchSpace" => writer.patch_space(&id(args, 0), arg(1)),
        "reorderSpaces" => writer.reorder_spaces(arg(0)),
        "deleteSpace" => writer.delete_space(&id(args, 0)),
        "membership" => writer.membership(&id(args, 0), &id(args, 1), flag(2)),
        "exclude" => writer.exclude(&id(args, 0), &id(args, 1)),
        "settings" => writer.settings(arg(0)),
        "remember" => writer.remember(&id(args, 0), arg(1)),
        "onboard" => writer.onboard(arg(0).to_number() as i32, flag(1)),
        "share" => writer.share(&id(args, 0)),
        "revokeShare" => writer.revoke_share(&id(args, 0), &id(args, 1)),
        "restoreRevision" => writer.restore_revision(&id(args, 0), &id(args, 1)),
        "removeThumbnail" => writer.remove_thumbnail(&id(args, 0)),
        "enrich" => {
            let mut baseline = arg(2).clone();
            let changed = writer.enrich(&id(args, 0), arg(1), &mut baseline)?;
            Ok(Value::Bool(changed))
        }
        "processing" => {
            writer.processing(&id(args, 0), flag(1), flag(2))?;
            Ok(Value::Undefined)
        }
        "analysis" => {
            writer.analysis(&id(args, 0), arg(1))?;
            Ok(Value::Undefined)
        }
        "checkQuery" => Ok(writer.check_query(&id(args, 0), flag(1))?.to_value()),
        other => panic!("unknown step {other}"),
    }
}

fn scenario(file: &Path, steps: &[Value]) -> Result<Value> {
    let db = Rc::new(Sqlite::open(file, false)?);
    let store = Rc::new(Store::new(db.clone())?);
    let events = Rc::new(RefCell::new(Vec::<Value>::new()));
    let sink = events.clone();
    COUNTER.store(0, std::sync::atomic::Ordering::SeqCst);
    let writer = Writer::new(store.clone(), move |target, payload| sink.borrow_mut().push(arr![target, payload.clone()]))?.with_random(random);
    let mut results = Vec::new();
    for step in steps {
        let items = step.as_array().unwrap();
        let name = items[0].as_str().unwrap();
        let mut args = Vec::new();
        for raw in &items[1..] {
            args.push(resolve(&*db, raw)?);
        }
        results.push(match writer.run(|w| dispatch(w, name, &args)) {
            Ok(mut value) => {
                if let Some(elapsed) = value.get("elapsed").as_f64() {
                    assert!(elapsed >= 0.0 && elapsed.fract() == 0.0);
                    value.set("elapsed", 0.0);
                }
                obj! { "ok" => if value.is_undefined() { Value::Null } else { value } }.into()
            }
            Err(Error::Status { status, message, .. }) => obj! { "status" => status, "message" => message }.into(),
            Err(Error::Invalid(message)) => {
                let kind = if message.contains("RangeError") { "RangeError" } else { "TypeError" };
                obj! { "error" => kind }.into()
            }
            Err(other) => return Err(other),
        });
    }
    let mut state = mm_core::Object::new();
    for table in TABLES {
        let mut rows = Vec::new();
        let columns: Vec<String> = db
            .query(&format!("PRAGMA table_info({table})"), &[])?
            .iter()
            .map(|column| {
                let name = column.text("name").unwrap_or("");
                format!("CASE WHEN typeof(\"{name}\") = 'integer' AND abs(\"{name}\") > 9007199254740991 THEN CAST(\"{name}\" AS TEXT) WHEN typeof(\"{name}\") = 'blob' THEN hex(\"{name}\") ELSE \"{name}\" END AS \"{name}\"")
            })
            .collect();
        for row in db.query(&format!("SELECT {} FROM {table} ORDER BY 1, 2", columns.join(", ")), &[])? {
            let mut object = row.to_object();
            object.remove("html");
            rows.push(Value::Object(object));
        }
        state.insert(table, Value::Array(rows));
    }
    Ok(obj! { "results" => results, "events" => events.borrow().clone(), "state" => state }.into())
}

fn doc(text: &str) -> Value {
    obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => text })] })] }.into()
}

fn scenarios() -> Vec<(&'static str, Vec<Value>)> {
    let step = |items: Vec<Value>| Value::Array(items);
    vec![
        ("create", vec![
            step(vec!["create".into(), obj! { "content" => "hello\nworld" }.into()]),
            step(vec!["create".into(), obj! { "content" => "https://example.com/a?b=1", "source" => "test" }.into()]),
            step(vec!["create".into(), obj! { "content" => "https://example.com/a?b=1", "spaceId" => "nope" }.into()]),
            step(vec!["create".into(), obj! { "content" => "https://example.com/shot", "type" => "Screenshot", "title" => "Shot" }.into()]),
            step(vec!["create".into(), obj! { "type" => "Media" }.into()]),
            step(vec!["create".into(), obj! { "prose" => doc("x"), "isDraft" => true, "isStub" => true }.into()]),
            step(vec!["create".into(), obj! { "prose" => obj! { "type" => "nope" } }.into()]),
            step(vec!["create".into(), obj! { "content" => "http://[bad/x" }.into()]),
            step(vec!["create".into(), Value::Null]),
            step(vec!["create".into(), obj! { "content" => "m" }.into(), obj! { "id" => "upl1", "data" => obj! { "type" => "Image", "flags" => 0.0 }, "extras" => obj! { "k" => 1.0 } }.into()]),
        ]),
        ("edit", vec![
            step(vec!["patch".into(), "@card:0".into(), obj! { "title" => "T", "description" => "D", "flags" => 5.0 }.into()]),
            step(vec!["patch".into(), "@card:1".into(), obj! { "flags" => -1.0 }.into()]),
            step(vec!["patch".into(), "@card:1".into(), obj! { "prose" => doc("hi"), "author" => "me" }.into()]),
            step(vec!["patch".into(), "nope".into(), obj! {}.into()]),
            step(vec!["prose".into(), "@card:2".into(), doc("a"), "prose".into()]),
            step(vec!["prose".into(), "@card:2".into(), doc("b"), "note".into()]),
            step(vec!["prose".into(), "@card:2".into(), doc("c"), "content".into()]),
            step(vec!["prose".into(), "@card:2".into(), doc("d"), "commit".into()]),
            step(vec!["prose".into(), "@card:2".into(), obj! { "type" => "x" }.into()]),
            step(vec!["commit".into(), "@card:3".into(), doc("e")]),
            step(vec!["restoreRevision".into(), "@card:2".into(), "@revision:0".into()]),
            step(vec!["restoreRevision".into(), "@card:2".into(), "zzz".into()]),
            step(vec!["removeThumbnail".into(), "@card:0".into()]),
        ]),
        ("tags", vec![
            step(vec!["addTag".into(), "@card:0".into(), "#private".into()]),
            step(vec!["addTag".into(), "@card:0".into(), "plain".into()]),
            step(vec!["addTag".into(), "@card:0".into(), "plain".into()]),
            step(vec!["addTag".into(), "@card:0".into(), "highlight".into()]),
            step(vec!["addTag".into(), "@card:0".into(), "  ".into()]),
            step(vec!["addTag".into(), "@card:0".into(), Value::Number(3.0)]),
            step(vec!["removeTag".into(), "@annotation:0".into()]),
            step(vec!["removeTag".into(), "@annotation:1".into()]),
            step(vec!["removeTag".into(), "nope".into()]),
        ]),
        ("pins", vec![
            step(vec!["flag".into(), "@card:0".into(), 512.0.into(), true.into(), 1.0.into()]),
            step(vec!["flag".into(), "@card:1".into(), 512.0.into(), true.into(), 1.0.into()]),
            step(vec!["flag".into(), "@card:2".into(), 512.0.into(), true.into()]),
            step(vec!["flag".into(), "@card:1".into(), 512.0.into(), false.into()]),
            step(vec!["flag".into(), "@card:3".into(), 2.0.into(), true.into()]),
            step(vec!["reorderPins".into(), arr!["@card:2", "@card:0"]]),
            step(vec!["reorderPins".into(), "x".into()]),
            step(vec!["trash".into(), "@card:0".into()]),
            step(vec!["trash".into(), "@card:0".into(), true.into()]),
            step(vec!["trash".into(), "nope".into()]),
        ]),
        ("spaces", vec![
            step(vec!["createSpace".into(), obj! { "name" => "One" }.into()]),
            step(vec!["createSpace".into(), obj! { "name" => "Smart", "color" => "#fff", "query" => obj! { "filters" => Value::Array(vec![]) } }.into()]),
            step(vec!["createSpace".into(), obj! { "name" => " " }.into()]),
            step(vec!["membership".into(), "@space:0".into(), "@card:0".into()]),
            step(vec!["membership".into(), "@space:0".into(), "@card:1".into()]),
            step(vec!["membership".into(), "@space:0".into(), "@card:1".into(), true.into()]),
            step(vec!["patchSpace".into(), "@space:0".into(), obj! { "name" => "Renamed" }.into()]),
            step(vec!["patchSpace".into(), "nope".into(), obj! {}.into()]),
            step(vec!["reorderSpaces".into(), arr!["@space:1", "@space:0"]]),
            step(vec!["exclude".into(), "@query:0".into(), "@card:0".into()]),
            step(vec!["exclude".into(), "nope".into(), "@card:0".into()]),
            step(vec!["deleteSpace".into(), "@space:0".into()]),
            step(vec!["deleteSpace".into(), "nope".into()]),
        ]),
        ("account", vec![
            step(vec!["settings".into(), obj! { "colorScheme" => "dark", "soundEnabled" => false, "locale" => "de" }.into()]),
            step(vec!["settings".into(), obj! { "soundEnabled" => true }.into()]),
            step(vec!["remember".into(), "dismissed".into(), "a".into()]),
            step(vec!["remember".into(), "dismissed".into(), "a".into()]),
            step(vec!["remember".into(), "dismissed".into(), 3.0.into()]),
            step(vec!["onboard".into(), 4.0.into(), true.into()]),
            step(vec!["onboard".into(), 4.0.into(), false.into()]),
            step(vec!["share".into(), "@card:0".into()]),
            step(vec!["share".into(), "nope".into()]),
            step(vec!["revokeShare".into(), "@card:0".into(), "@share:0".into()]),
        ]),
        ("enrich", vec![
            step(vec!["enrich".into(), "@card:0".into(), obj! { "data" => obj! { "description" => "new", "title" => "nt" }, "extras" => obj! { "k" => 1.0 } }.into(), "@baseline:0".into()]),
            step(vec!["enrich".into(), "@card:1".into(), obj! { "data" => obj! { "title" => "x" } }.into(), obj! { "data" => obj! { "source" => obj! { "url" => "zzz" } }, "extras" => obj! {} }.into()]),
            step(vec!["enrich".into(), "@card:2".into(), obj! { "extras" => obj! { "screenshotPending" => true } }.into(), "@baseline:2".into()]),
            step(vec!["enrich".into(), "@card:3".into(), obj! { "extras" => obj! { "screenshotFailed" => true } }.into(), "@baseline:3".into()]),
            step(vec!["enrich".into(), "@card:4".into(), obj! { "data" => obj! { "type" => "Article" }, "extras" => obj! { "screenshotFailed" => true, "captureText" => "t" } }.into(), "@baseline:4".into()]),
            step(vec!["enrich".into(), "nope".into(), obj! {}.into(), obj! { "data" => obj! {}, "extras" => obj! {} }.into()]),
            step(vec!["processing".into(), "@card:1".into(), true.into()]),
            step(vec!["processing".into(), "@card:1".into(), false.into(), true.into()]),
            step(vec!["processing".into(), "@card:3".into(), true.into()]),
            step(vec!["analysis".into(), "@card:0".into(), analysis(768, "alpha")]),
            step(vec!["analysis".into(), "@card:1".into(), analysis(767, "beta")]),
            step(vec!["analysis".into(), "@card:5".into(), analysis(768, "gamma")]),
            step(vec!["trash".into(), "@card:6".into()]),
            step(vec!["analysis".into(), "@card:6".into(), analysis(768, "delta")]),
        ]),
        ("check-query", vec![
            step(vec!["createSpace".into(), obj! { "name" => "Smart", "query" => obj! { "filters" => Value::Array(vec![]) } }.into()]),
            step(vec!["checkQuery".into(), "@query:0".into(), false.into()]),
            step(vec!["membership".into(), "@space:0".into(), "@card:0".into(), true.into()]),
            step(vec!["checkQuery".into(), "@query:0".into(), true.into()]),
            step(vec!["checkQuery".into(), "nope".into(), true.into()]),
        ]),
    ]
}

fn analysis(size: usize, tag: &str) -> Value {
    let embedding: Vec<Value> = (0..size).map(|index| Value::Number((index % 7) as f64 / 7.0)).collect();
    obj! {
        "summary" => "S".repeat(700),
        "title" => "  a b c d e f g h i j ",
        "tags" => vec![Value::from(obj! { "name" => tag, "p" => 0.9, "source" => "chat" }), Value::from(obj! { "name" => "ui", "p" => 0.5, "source" => "vocabulary" })],
        "embedding" => embedding,
    }
    .into()
}

#[test]
fn mutations_match_node() {
    let _goldens = common::test_guard();
    let _guard = CLOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut cases = 0;
    for fixture in [test_db(), hosted_db()] {
        for (name, steps) in scenarios() {
            date::set_fixed_now(Some(1_787_000_000_000));
            let rust = scenario(&temp_copy(&fixture), &steps);
            date::set_fixed_now(None);
            let js = calls(DRIVER, &[("scenario", vec![Value::from(temp_copy(&fixture).to_str().unwrap()), Value::Array(steps.clone())])]).remove(0);
            assert_same(&rust, &js, &format!("{name} on {}", fixture.display()));
            cases += 1;
        }
    }
    assert_eq!(cases, 16);
}
