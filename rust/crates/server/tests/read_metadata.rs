//! Frozen read-metadata imports over owner fixture copies with imported metadata removed. Repository and synthetic
//! recordings cover byte order marks, invalid UTF-8, reader pages, unknown cards, a missing body and idempotence.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Value, obj};
use mm_js::json;
use mm_sqlite::Sqlite;
use mymind::read_metadata::import_read_metadata;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/read-metadata.mjs");

// A fixture copy whose imported metadata and reader documents are gone, so the import runs again.
fn fresh_copy() -> PathBuf {
    let copy = common::temp_copy(&common::test_db());
    let db = Rc::new(Sqlite::open(&copy, false).unwrap());
    // The store registers the functions the content triggers call.
    let _store = Store::new(db.clone()).unwrap();
    db.exec_batch("DELETE FROM api_metadata; DELETE FROM card_content;").unwrap();
    copy
}

fn card_ids(count: usize) -> Vec<String> {
    let db = Sqlite::open(common::test_db(), true).unwrap();
    db.query("SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id", &[]).unwrap().iter().take(count).filter_map(|row| row.text("id").map(str::to_owned)).collect()
}

fn write(root: &Path, file: &str, bytes: &[u8]) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn entry(file: &str, headers: Value) -> Value {
    obj! { "file" => file, "status" => 200, "headers" => headers }.into()
}

// Made-up recordings: `account` puts an account in the replay index, `broken` names a body file that is missing.
fn recordings(name: &str, account: bool, broken: bool) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("read-metadata-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let ids = card_ids(6);
    let doc = |text: &str| format!(r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{text}"}}]}}]}}"#);
    let origin = "GET https://access.mymind.com";
    let headers = |extra: &[(&str, &str)]| {
        let mut all = mm_js::Object::new();
        for (name, value) in [("content-type", "application/json; charset=utf-8"), ("x-partial", "Grid"), ("set-cookie", "a=b"), ("access-control-expose-headers", "x-checkpoint")] {
            all.insert(name, value);
        }
        for (name, value) in extra {
            all.insert(*name, *value);
        }
        Value::Object(all)
    };
    let mut index = mm_js::Object::new();
    write(&root, "replay/bodies/bom.json", format!("\u{feff}{}", doc("bom")).as_bytes());
    index.insert(format!("{origin}/objects/{}/content", ids[0]), entry("bom.json", headers(&[])));
    write(&root, "replay/bodies/doc.json", doc("plain").as_bytes());
    index.insert(format!("{origin}/objects/{}/content", ids[1]), entry("doc.json", headers(&[])));
    write(&root, "replay/bodies/other.json", br#"{"type":"paragraph"}"#);
    index.insert(format!("{origin}/objects/{}/content", ids[2]), entry("other.json", headers(&[])));
    write(&root, "replay/bodies/blank.json", "\u{feff} \n\t".as_bytes());
    index.insert(format!("{origin}/objects/{}/content?x=1", ids[3]), entry("blank.json", headers(&[])));
    let reader = format!("<html><mymind-editor class=\"x\"><div>ignored</div><script type=\"application/json\">{}</script></mymind-editor>", doc("reader"));
    write(&root, "replay/bodies/reader.html", reader.as_bytes());
    index.insert(format!("{origin}/cards/{}/reader", ids[4]), entry("reader.html", headers(&[])));
    let mut invalid = b"<mymind-editor><script>".to_vec();
    invalid.extend_from_slice(&[0xff, 0xfe]);
    invalid.extend_from_slice(doc("bad bytes").as_bytes());
    invalid.extend_from_slice(b"</script>");
    write(&root, "replay/bodies/invalid.html", &invalid);
    index.insert(format!("{origin}/cards/{}/reader", ids[5]), entry("invalid.html", headers(&[])));
    write(&root, "replay/bodies/nope.json", doc("nope").as_bytes());
    index.insert(format!("{origin}/objects/nope/content"), entry("nope.json", headers(&[])));
    write(&root, "replay/bodies/space.json", br#"{"id":"s1","query":{"checked":"2026-01-02T03:04:05Z","q":"x"}}"#);
    index.insert(format!("{origin}/spaces/s1"), entry("space.json", headers(&[("x-partial", "Space")])));
    write(&root, "replay/bodies/space2.json", br#"{"id":"s2","query":null}"#);
    index.insert(format!("{origin}/spaces/s2"), entry("space2.json", headers(&[])));
    write(&root, "replay/bodies/list.json", b"[]");
    for path in ["/cards", "/cards?since=4", "/cards/abc", "/spaces", "/tags/manual", "/tags/recent", "/changes?since=9", "/cards/abc/detail", "/tags/other"] {
        index.insert(format!("{origin}{path}"), entry("list.json", headers(&[])));
    }
    index.insert("POST https://access.mymind.com/cards".to_owned(), entry("list.json", headers(&[])));
    index.insert("iphone|GET https://access.mymind.com/spaces".to_owned(), entry("list.json", headers(&[])));
    index.insert("GET https://example.com/cards".to_owned(), entry("list.json", headers(&[])));
    if account {
        write(&root, "replay/bodies/account.json", br#"{"id":"replayed","plan":"pro"}"#);
        index.insert(format!("{origin}/account.json"), entry("account.json", headers(&[])));
    }
    if broken {
        index.insert(format!("{origin}/objects/{}/content", ids[2]), entry("missing.json", headers(&[])));
    }
    write(&root, "replay/index.json", json::to_string(&Value::Object(index)).as_bytes());
    let mut responses = mm_js::Object::new();
    write(&root, "capture/responses/content.json", doc("captured").as_bytes());
    responses.insert(format!("GET /objects/{}/content", ids[2]), Value::from(obj! { "file" => "responses/content.json" }));
    responses.insert(format!("GET /objects/{}/content", ids[1]), Value::from(obj! { "file" => "responses/content.json" }));
    write(&root, "capture/responses/account.json", br#"{"id":"captured"}"#);
    responses.insert("GET /account.json".to_owned(), Value::from(obj! { "file" => "responses/account.json" }));
    write(&root, "capture/responses.json", json::to_string(&Value::Object(responses)).as_bytes());
    root
}

fn rust_run(file: &Path, root: &Path) -> Value {
    mymind::runtime::run(async {
        let db = Rc::new(Sqlite::open(file, false).unwrap());
        let store = Store::new(db.clone()).unwrap();
        let threw = import_read_metadata(&store, root).await.is_err();
        let dump = |sql: &str| -> Vec<Value> { db.query(sql, &[]).unwrap().iter().map(|row| Value::from(json::to_string(&Value::Object(row.to_object())))).collect() };
        obj! { "threw" => threw, "metadata" => dump("SELECT * FROM api_metadata ORDER BY 1"), "content" => dump("SELECT * FROM card_content ORDER BY 1") }.into()
    })
}

#[test]
fn imports_as_import_read_metadata() {
    let empty = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("read-metadata-{}-empty", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let roots = [("repository", common::root()), ("made up", recordings("account", true, false)), ("captured account", recordings("captured", false, false)), ("missing body", recordings("broken", true, true)), ("no recordings", empty)];
    for (name, root) in roots {
        let (js_copy, rust_copy) = (fresh_copy(), fresh_copy());
        let expected = common::call(DRIVER, "run", vec![js_copy.to_string_lossy().as_ref().into(), root.to_string_lossy().as_ref().into()]);
        let rust = rust_run(&rust_copy, &root);
        common::assert_value(&rust, &expected, name);
        if name != "no recordings" && name != "missing body" {
            assert!(!rust.get("metadata").as_array().unwrap().is_empty(), "{name} imported nothing");
        }
        if name == "made up" {
            // The plain document, the reader page and the captured one; JSON.parse refuses the byte order mark and
            // the replaced bytes.
            let content = rust.get("content").as_array().unwrap();
            assert!(content.len() == 3, "{name} imported {} documents", content.len());
        }
        // A second import leaves existing metadata alone.
        assert_eq!(common::text(&rust_run(&rust_copy, &root)), common::text(&rust), "{name}, again");
    }
}
