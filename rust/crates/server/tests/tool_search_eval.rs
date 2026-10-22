//! `mymind search-eval` over a synthetic library and a loopback Gemini: the queries file, the report and the guards.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::path::Path;
use std::rc::Rc;

use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Value, obj};
use mm_sqlite::Sqlite;
use tools_common::{Fake, Req, Resp, run_rust, tmproot};

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

// Four cards at similarity 1, 0.8, 0.68 and 0.45 to "quiet design"; returns their ids in that order.
fn library(path: &Path) -> Vec<String> {
    mymind::runtime::run(async { cards(path) })
}

fn cards(path: &Path) -> Vec<String> {
    let db = Rc::new(Sqlite::open(path, false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users VALUES ('eval-user', 'eval-account', 'fixture0000', 0, '{\"objectLimit\":10000}')", &[]).unwrap();
    let writer = Writer::new(store, |_, _| {}).unwrap();
    [("Quiet design", "rooms with soft light", 1.0), ("Calm technology", "interfaces at the periphery", 0.8), ("Slow software", "tools that wait for you", 0.68), ("Gardening notes", "tomatoes and basil", 0.45)]
        .into_iter()
        .map(|(title, text, similarity)| {
            let created = writer.run(|writer| writer.create(&obj! { "title" => title, "prose" => doc(text) }.into(), &Value::Undefined)).unwrap();
            let id = created.get("body").get("id").as_str().unwrap().to_owned();
            db.execute("INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, 'eval-user', 768, ?)", &[id.as_str().into(), SqlValue::from(vector(similarity))]).unwrap();
            id
        })
        .collect()
}

// Gemini stand-in: "quiet design" embeds on the first axis, anything else on an axis no card uses.
fn gemini() -> Fake {
    Fake::start(|| {
        Box::new(|request: &Req| {
            let body = mm_js::json::parse(&String::from_utf8_lossy(&request.body)).unwrap_or(Value::Null);
            let text = body.get("content").get("parts").at(0).get("text").to_js_string();
            let axis = if text.contains("quiet design") { 0 } else { 5 };
            let values: Vec<String> = (0..768).map(|i| if i == axis { "1".to_owned() } else { "0".to_owned() }).collect();
            Resp::json(200, &format!("{{\"embedding\":{{\"values\":[{}]}}}}", values.join(",")))
        })
    })
}

#[test]
fn reports_ranks_for_each_mode_and_the_best_blend() {
    let root = tmproot("search-eval");
    let db = root.join("library.db");
    let ids = library(&db);
    let before = std::fs::read(&db).unwrap();
    std::fs::write(
        root.join("queries.jsonl"),
        format!(
            "# what I would type, and the card I meant\n\n{{\"q\": \"quiet design\", \"expect\": \"calm technology\"}}\n{{\"q\": \"soft light\", \"expect\": [\"{}\"]}}\n{{\"q\": \"tomatoes\", \"expect\": \"Recipes\"}}\n{{\"q\": \"basil\", \"expect\": \"e\"}}\n",
            ids[0]
        ),
    )
    .unwrap();
    let fake = gemini();
    let url = fake.url();
    let env = [("GEMINI_API_KEY", "test-key"), ("GEMINI_BASE_URL", url.as_str()), ("OFFLINE", "1")];
    let out = run_rust(&root, "search-eval", &["queries.jsonl", "--db", "library.db"], &env, None);
    assert_eq!(out.status, 0, "{}", out.stderr);
    let report = out.stdout;
    for line in [
        "Library: 4 cards, 4 with an embedding.",
        "Queries: 4, 2 lines skipped.",
        "  line 5: no card has the title, link or id \"Recipes\"",
        "  line 6: \"e\" is in 4 titles, such as",
        "Best blend: boost 1, floor 0.7, at most 20 cards by meaning alone.",
    ] {
        assert!(report.contains(line), "missing {line:?} in\n{report}");
    }
    let row = |label: &str| report.lines().find(|line| line.starts_with(label)).unwrap_or_else(|| panic!("no {label} row in\n{report}")).split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(row("words (the app today)").starts_with("words (the app today) 1/2 1/2 1/2 1"), "{report}");
    assert!(row("meaning only").starts_with("meaning only 1/2 2/2 2/2 4"), "{report}");
    assert!(row("both, boost 2, floor 0.6 *").starts_with("both, boost 2, floor 0.6 * 1/2 2/2 2/2 2"), "{report}");
    assert!(report.contains("    3       -      2 (0.80)       2  quiet design"), "{report}");
    assert!(report.contains("    4       1      1 (0.00)       1  soft light"), "{report}");
    assert_eq!(fake.log().len(), 2, "one Gemini call per query");
    assert_eq!(std::fs::read(&db).unwrap(), before, "the library is not written");
}

#[test]
fn guards_and_usage() {
    let root = tmproot("search-eval-usage");
    let help = run_rust(&root, "search-eval", &["--help"], &[], None);
    assert_eq!(help.status, 0);
    assert!(help.stdout.starts_with("Usage: mymind search-eval <queries.jsonl>"), "{}", help.stdout);
    let key = [("GEMINI_API_KEY", "test-key")];
    for (args, message) in [
        (&[][..], "Usage: mymind search-eval"),
        (&["queries.jsonl", "--floor", "2"], "--floor takes numbers from -1 to 1"),
        (&["queries.jsonl", "--take", "x"], "--take must be a nonnegative integer"),
        (&["queries.jsonl", "--bogus"], "Usage: mymind search-eval"),
    ] {
        let out = run_rust(&root, "search-eval", args, &key, None);
        assert_eq!(out.status, 1, "{args:?}");
        assert!(out.stderr.contains(message), "{args:?}: {}", out.stderr);
    }
    let out = run_rust(&root, "search-eval", &["queries.jsonl"], &[], None);
    assert!(out.status == 1 && out.stderr.contains("GEMINI_API_KEY"), "{}", out.stderr);
    let db = root.join("library.db");
    library(&db);
    std::fs::write(root.join("bad.jsonl"), "{\"q\": \"quiet design\"}\n").unwrap();
    let out = run_rust(&root, "search-eval", &["bad.jsonl", "--db", "library.db"], &key, None);
    assert!(out.status == 1 && out.stderr.contains("line 1: expected {\"q\""), "{}", out.stderr);
}
