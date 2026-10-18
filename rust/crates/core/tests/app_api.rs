//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use common::{assert_same, calls, hosted_db, temp_copy, test_db};
use futures_executor::block_on;
use mm_core::app_api::AppApi;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::search::Search;
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Error, Result, Value, arr, obj};
use mm_js::{json, url};
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:app-api";

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
        let text = json::parse(&body)?.get("content").get("parts").at(0).get("text").to_js_string();
        let seed: u64 = text.encode_utf16().map(u64::from).sum();
        let values: Vec<Value> = (0..768u64).map(|i| Value::Number((((seed * 31 + i * 17) % 101) as f64 - 50.0) / 50.0)).collect();
        Ok(Response::new(200, Headers::new(), Body::Bytes(json::to_string(&obj! { "embedding" => obj! { "values" => values } }.into()).into_bytes())))
    }
}

fn localize(text: &str) -> String {
    text.replace("https://x.com", "/x")
}

fn run(file: &std::path::Path, requests: &[Value]) -> Result<Value> {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::from([("GEMINI_API_KEY".to_owned(), "test-key".to_owned()), ("GEMINI_BASE_URL".to_owned(), "http://127.0.0.1:1/v1beta".to_owned())]));
    let db = Rc::new(Sqlite::open(file, false)?);
    let store = Rc::new(Store::new(db)?);
    let search = Rc::new(Search::new(store.clone(), Some(Rc::new(Fake)), false)?);
    let api = AppApi::new(store.clone(), search, localize);
    let mut out = Vec::new();
    for request in requests {
        let items = request.as_array().unwrap();
        let (method, target, crawl) = (items[0].as_str().unwrap(), items[1].as_str().unwrap(), items[2].truthy());
        let mut current = target.to_owned();
        for _ in 0..if crawl { 60 } else { 1 } {
            let address = url::parse(&current, Some("http://localhost")).expect("url");
            let reply = match block_on(api.handle(method, &address)) {
                Ok(Some(response)) => {
                    let headers: Vec<Value> = response.headers.iter().map(|(name, value)| arr![name, value]).collect();
                    let status = response.status;
                    let body = String::from_utf8(block_on(response.body.bytes())?).expect("utf-8");
                    obj! { "status" => status, "headers" => headers, "body" => body }.into()
                }
                Ok(None) => Value::Null,
                Err(Error::Invalid(message)) => obj! { "threw" => if message.contains("RangeError") { "RangeError" } else { "TypeError" } }.into(),
                Err(Error::Internal(message)) if message.contains("URI malformed") => obj! { "threw" => "URIError" }.into(),
                Err(other) => return Err(other),
            };
            let next = match (&reply, crawl) {
                (reply, true) if reply.get("status").as_f64() == Some(200.0) => json::parse(reply.get("body").as_str().unwrap())?.get("next").as_str().map(str::to_owned),
                _ => None,
            };
            out.push(arr![current.as_str(), reply]);
            let Some(next) = next else { break };
            current = format!("{target}{}cursor={next}", if target.contains('?') { "&" } else { "?" });
        }
    }
    Ok(Value::Array(out))
}

fn requests(db: &dyn Sql) -> Vec<Value> {
    let get = |target: &str, crawl: bool| -> Value { arr!["GET", target, crawl] };
    let mut out = vec![
        get("/app/v1/cards?limit=200", true),
        get("/app/v1/cards?limit=7", false),
        get("/app/v1/cards?limit=0", false),
        get("/app/v1/cards?limit=201", false),
        get("/app/v1/cards?limit=x", false),
        get("/app/v1/cards?limit=2.5", false),
        get("/app/v1/cards?cursor=bad", false),
        get("/app/v1/cards?cursor=WzVd", false),
        get("/app/v1/trash?limit=200", true),
        get("/app/v1/spaces", false),
        get("/app/v1/tags", false),
        get("/app/v1/cards/nope", false),
        get("/app/v1/cards/%E0%A4%A", false),
        get("/app/v1/search?q=design&limit=50", true),
        get("/app/v1/search?q=%20", false),
        get("/app/v1/search", false),
        get("/app/v1/search?q=similar%3Anope", false),
        get("/app/v1/spaces/nope/cards", false),
        get("/app/v1/unknown", false),
        get("/other", false),
        arr!["POST", "/app/v1/cards", false],
        arr!["DELETE", "/app/v1/search", false],
        arr!["POST", "/app/v1/other", false],
    ];
    for row in db.query("SELECT id FROM cards ORDER BY id LIMIT 40", &[]).unwrap() {
        out.push(get(&format!("/app/v1/cards/{}", row.text("id").unwrap_or("")), false));
    }
    for row in db.query("SELECT id FROM spaces ORDER BY id", &[]).unwrap() {
        let id = row.text("id").unwrap_or("");
        out.push(get(&format!("/app/v1/spaces/{id}/cards?limit=30"), true));
        out.push(get(&format!("/app/v1/spaces/{id}/cards?cursor=bad"), false));
    }
    out
}

#[test]
fn app_api_matches_node() {
    let _goldens = common::test_guard();
    for fixture in [test_db(), hosted_db()] {
        let copy = temp_copy(&fixture);
        let db = Sqlite::open(&copy, false).unwrap();
        let requests = requests(&db);
        let js = calls(DRIVER, &[("run", vec![Value::from(copy.to_str().unwrap()), Value::Array(requests.clone())])]).remove(0);
        let rust = run(&copy, &requests);
        assert_same(&rust, &js, &format!("app api on {}", fixture.display()));
    }
}
