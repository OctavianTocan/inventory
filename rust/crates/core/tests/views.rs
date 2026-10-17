//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::rc::Rc;

use common::{assert_same, calls, hosted_db, temp_copy, test_db};
use mm_core::render::views::{render_detail, render_pinned, render_share};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Error, Result, Value, obj};
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:views";

fn attempt(work: impl FnOnce() -> Result<String>) -> Value {
    match work() {
        Ok(html) => html.into(),
        Err(Error::Invalid(message)) => obj! { "threw" => if message.contains("RangeError") { "RangeError" } else { "TypeError" } }.into(),
        Err(other) => obj! { "threw" => format!("{other}") }.into(),
    }
}

fn all(file: &std::path::Path) -> Result<Value> {
    let db = Rc::new(Sqlite::open(file, false)?);
    let store = Store::new(db.clone())?;
    let share: Value = obj! { "url" => "/shares/abc<d>", "expires" => "2026-10-18T00:00:00.000Z" }.into();
    let content: Value = obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => "reader <b>" })] })] }.into();
    let mut out = Vec::new();
    for found in db.query("SELECT id FROM cards ORDER BY id", &[])? {
        let id = found.text("id").unwrap_or("").to_owned();
        let row = store.card(&id, None)?.expect("card");
        out.push(
            obj! {
                "id" => id.as_str(),
                "pinned" => render_pinned(&row),
                "detail" => attempt(|| render_detail(&row, &Value::Null, &Value::Null)),
                "shared" => attempt(|| render_detail(&row, &share, &content)),
                "share" => attempt(|| render_share(&row)),
            }
            .into(),
        );
    }
    Ok(Value::Array(out))
}

#[test]
fn views_match_node() {
    let _goldens = common::test_guard();
    for fixture in [test_db(), hosted_db()] {
        let copy = temp_copy(&fixture);
        let js = calls(DRIVER, &[("all", vec![Value::from(copy.to_str().unwrap())])]).remove(0);
        assert_same(&all(&copy), &js, &format!("views on {}", fixture.display()));
    }
}
