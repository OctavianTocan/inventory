//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use common::{assert_same, calls, hosted_db, temp_copy, test_db};
use futures_executor::block_on;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Error, Result, Value, arr, obj};
use mm_js::json;
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:search";

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

struct Fake;

#[async_trait(?Send)]
impl Fetch for Fake {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = String::from_utf8(request.take_body().bytes().await?).expect("utf-8");
        let parsed = json::parse(&body)?;
        let text = parsed.get("content").get("parts").at(0).get("text").to_js_string();
        let seed: u64 = text.encode_utf16().map(u64::from).sum();
        let values: Vec<Value> = (0..768u64).map(|i| Value::Number((((seed * 31 + i * 17) % 101) as f64 - 50.0) / 50.0)).collect();
        let answer = json::to_string(&obj! { "embedding" => obj! { "values" => values } }.into());
        Ok(Response::new(200, Headers::new(), Body::Bytes(answer.into_bytes())))
    }
}

fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned()), ("GEMINI_BASE_URL".to_owned(), "http://127.0.0.1:1/v1beta".to_owned())]));
}

fn answer(result: Result<Value>) -> Value {
    match result {
        Ok(Value::Object(mut object)) => {
            if let Some(elapsed) = object.get("elapsed").and_then(Value::as_f64) {
                assert!(elapsed >= 0.0 && elapsed.fract() == 0.0, "elapsed {elapsed}");
                object.insert("elapsed", 0.0);
            }
            obj! { "ok" => Value::Object(object) }.into()
        }
        Ok(other) => obj! { "ok" => other }.into(),
        Err(Error::Status { status, message, .. }) => obj! { "status" => status, "message" => message }.into(),
        Err(Error::Invalid(message)) => obj! { "error" => if message.contains("RangeError") { "RangeError" } else { "TypeError" } }.into(),
        Err(other) => obj! { "error" => format!("{other}") }.into(),
    }
}

fn run(file: &std::path::Path, cases: &[Value]) -> Result<Value> {
    install();
    let db = Rc::new(Sqlite::open(file, false)?);
    let store = Rc::new(Store::new(db)?);
    let online = Search::new(store.clone(), Some(Rc::new(Fake)), false)?;
    let offline = Search::new(store.clone(), Some(Rc::new(Fake)), true)?;
    let mut out = Vec::new();
    for case in cases {
        let items = case.as_array().unwrap();
        let search = if items[0].truthy() { &offline } else { &online };
        let name = items[1].as_str().unwrap();
        let first = items.get(2).cloned().unwrap_or(Value::Undefined);
        let options = items.get(3).cloned().unwrap_or_else(|| Value::Object(mm_core::Object::new()));
        let text = first.to_js_string();
        out.push(match name {
            "text" => answer(search.text(&text, &options).map(|found| found.to_value())),
            "search" => answer(block_on(search.search(&text, &options)).map(|found| found.to_value())),
            "embeddings" => answer(block_on(search.embeddings(&text, &options)).map(|found| found.to_value())),
            "related" => answer(search.related(&text, &options).map(|found| found.to_value())),
            "filters" => answer(search.filters(&first).map(|found| Value::Array(found.iter().map(|item| obj! { "id" => item.id.as_str(), "score" => item.score }.into()).collect()))),
            "suggestions" => answer(search.suggestions(&text).map(|found| Value::Array(found.iter().map(|item| item.to_value()).collect()))),
            other => panic!("unknown {other}"),
        });
    }
    Ok(Value::Array(out))
}

const QUERIES: [&str; 26] = [
    "", " ", "design", "ui design", "\"new era\"", "-design", "design OR art", "design AND ui", "(design OR art) -ui", "tag:ui", "tag:ui*", "label:design", "type:article", "type:note", "site:x.com", "domain:www.youtube.com",
    "created:>=2026-01-01", "before:2026-02-01", "after:2025-12-01", "bumped:[2026-01-01 TO *]", "completed:true", "action:read", "format:png", "(((", "title:design", "dé\u{301}sign",
];

fn cases(ids: &[String]) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    let sorts = ["", "created asc", "bumped desc", "modified", "bogus"];
    for query in QUERIES {
        for sort in sorts {
            let options: Value = if sort.is_empty() { obj! {}.into() } else { obj! { "sortBy" => sort }.into() };
            out.push(arr![false, "text", query, options]);
        }
        out.push(arr![false, "text", query, obj! { "wildcard" => true }]);
        for options in [obj! {}, obj! { "semantic" => true }, obj! { "semantic" => true, "semanticBoost" => 50.0 }, obj! { "semantic" => true, "semanticBoost" => 0.0 }, obj! { "semantic" => true, "sortBy" => "created" }] {
            for offline in [false, true] {
                out.push(arr![offline, "search", query, options.clone()]);
            }
        }
        for options in [obj! {}, obj! { "take" => 5.0 }, obj! { "take" => -1.0 }, obj! { "filter" => "tag:ui" }, obj! { "filter" => "(((" }, obj! { "take" => "x" }] {
            out.push(arr![false, "embeddings", query, options]);
        }
    }
    for id in ids {
        out.push(arr![false, "search", format!("similar:{id}"), obj! {}]);
        for options in [obj! {}, obj! { "take" => 3.0 }, obj! { "take" => -1.0 }, obj! { "take" => 5000.0 }] {
            out.push(arr![false, "related", id.as_str(), options]);
        }
    }
    out.push(arr![false, "related", "nope", obj! {}]);
    for q in ["", " ", "de", "#ui", "tag:u", "zzz", "A", "label:D"] {
        out.push(arr![false, "suggestions", q]);
    }
    let filters: [Value; 7] = [arr![], arr!["design"], arr!["tag:ui", "type:article"], arr!["-design"], "x".into(), arr![1.0], arr!["(("]];
    for filter in filters {
        out.push(arr![false, "filters", filter]);
    }
    out
}

#[test]
fn search_matches_node() {
    let _goldens = common::test_guard();
    for fixture in [test_db(), hosted_db()] {
        let db = Sqlite::open(&fixture, true).unwrap();
        let ids: Vec<String> = db.query("SELECT card_id FROM embeddings ORDER BY card_id LIMIT 3", &[]).unwrap().iter().map(|row| row.text("card_id").unwrap_or("").to_owned()).collect();
        let cases = cases(&ids);
        let copy = temp_copy(&fixture);
        let js = calls(DRIVER, &[("run", vec![Value::from(copy.to_str().unwrap()), Value::Array(cases.clone())])]).remove(0);
        let rust = run(&copy, &cases);
        if let Ok(Value::Array(rust)) = &rust {
            assert_eq!(rust.len(), cases.len(), "every search case is answered");
            let populated = rust.iter().zip(&cases).filter(|(found, case)| case.as_array().unwrap()[1].as_str() == Some("embeddings") && found.get("ok").get("count").as_f64().is_some_and(|count| count > 0.0)).count();
            assert!(populated > 10, "embedding searches found nothing: {populated}");

        }
        assert_same(&rust, &js, &format!("search on {}", fixture.display()));
    }
}
