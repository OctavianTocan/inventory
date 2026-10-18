//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::collections::BTreeSet;
use std::rc::Rc;

use common::{assert_same, calls, hosted_db, temp_copy};
use mm_core::http::Request;
use mm_core::recorded::Recorded;
use mm_core::sql::Sql;
use mm_core::{Result, Value, arr};
use mm_js::base64;
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:recorded";

const AGENTS: [&str; 7] = [
    "",
    "Mozilla/5.0 (X11; Linux x86_64) Chrome/130",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)",
    "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X)",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14; Tablet) Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14\nMobile)",
];

fn base_key(key: &str) -> String {
    let mut rest = key;
    if let Some(after) = rest.strip_prefix("run:") {
        rest = after.split_once('|').map_or(after, |(_, tail)| tail);
    }
    if let Some((_, tail)) = rest.split_once('|').filter(|(head, _)| !head.contains(' ')) {
        rest = tail;
    }
    rest.to_owned()
}

fn run_of(key: &str) -> Option<String> {
    key.strip_prefix("run:").and_then(|after| after.split_once('|')).map(|(run, _)| run.to_owned())
}

fn look(recorded: &Recorded, lookup: &(String, String, String)) -> Result<Value> {
    let mut req = Request::new("GET", "https://access.mymind.com/x")?;
    if !lookup.0.is_empty() {
        req.headers.set("user-agent", lookup.0.as_str());
    }
    if !lookup.1.is_empty() {
        req.headers.set("x-replay-run", lookup.1.as_str());
    }
    let entry = recorded.entry(&req, &lookup.2)?;
    let has = recorded.has(&req, &lookup.2)?;
    Ok(match entry {
        None => arr![false, Value::Null, Value::Null, Value::Null, Value::Null, has],
        Some(entry) => {
            let headers = mm_js::json::stringify(&entry.headers).ok().flatten().unwrap_or_default();
            arr![true, entry.status as f64, headers, entry.body.len() as f64, mm_js::base64::hex(&Sha256::digest(&entry.body)), has]
        }
    })
}

fn lookups_value(lookups: &[(String, String, String)]) -> Value {
    Value::Array(lookups.iter().map(|(a, r, k)| arr![a.as_str(), r.as_str(), k.as_str()]).collect())
}

#[test]
fn recorded_matches_node_on_hosted_fixture() {
    let _goldens = common::test_guard();
    let probe = Sqlite::open(hosted_db(), true).unwrap();
    let keys: Vec<String> = probe.query("SELECT key FROM recorded", &[]).unwrap().iter().map(|row| row.text("key").unwrap().to_owned()).collect();
    assert!(keys.len() > 2000);
    let bases: BTreeSet<String> = keys.iter().map(|key| base_key(key)).collect();
    let mut runs: BTreeSet<String> = keys.iter().filter_map(|key| run_of(key)).collect();
    runs.insert("no-such-run".into());
    let mut lookups = Vec::new();
    for base in &bases {
        for agent in AGENTS {
            lookups.push((agent.to_owned(), String::new(), base.clone()));
            for run in &runs {
                lookups.push((agent.to_owned(), run.clone(), base.clone()));
            }
        }
    }
    for missing in ["", "GET https://access.mymind.com/nothing", "shell", "run:x|y"] {
        lookups.push((AGENTS[2].to_owned(), "android-cards".into(), missing.to_owned()));
    }
    let puts: Vec<(String, i64, Value, Vec<u8>)> = vec![
        ("GET https://access.mymind.com/new".into(), 200, arr![], vec![1, 2, 3]),
        ("GET https://access.mymind.com/new".into(), 404, mm_js::json::parse("{\"a\":[1,\"é\"]}").unwrap(), vec![9]),
        (bases.iter().next().unwrap().clone(), 500, mm_js::json::parse("{}").unwrap(), vec![]),
        ("iphone|GET https://access.mymind.com/new".into(), 201, mm_js::json::parse("{\"x-a\":\"b\"}").unwrap(), vec![0, 255]),
    ];
    let after: Vec<(String, String, String)> = vec![
        (String::new(), String::new(), puts[0].0.clone()),
        (AGENTS[2].to_owned(), String::new(), puts[0].0.clone()),
        (String::new(), String::new(), puts[2].0.clone()),
    ];

    let copy = temp_copy(&hosted_db());
    let js_puts = Value::Array(puts.iter().map(|(key, status, headers, body)| arr![key.as_str(), *status as f64, headers.clone(), base64::encode(body)]).collect());
    let js = calls(DRIVER, &[("session", vec![copy.to_str().unwrap().into(), lookups_value(&lookups), js_puts, lookups_value(&after)])]).remove(0);

    let copy = temp_copy(&hosted_db());
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(&copy, false).unwrap());
    let recorded = Recorded::new(db).unwrap();
    let result = (|| -> Result<Value> {
        let first = lookups.iter().map(|lookup| look(&recorded, lookup)).collect::<Result<Vec<_>>>()?;
        let before = recorded.count()?;
        for (key, status, headers, body) in &puts {
            recorded.put(key, *status, headers, body)?;
        }
        let count = recorded.count()?;
        let after = after.iter().map(|lookup| look(&recorded, lookup)).collect::<Result<Vec<_>>>()?;
        let mut object = mm_js::Object::new();
        object.insert("first", Value::Array(first));
        object.insert("before", before as f64);
        object.insert("count", count as f64);
        object.insert("after", Value::Array(after));
        Ok(Value::Object(object))
    })();
    assert_same(&result, &js, &format!("recorded session ({} lookups)", lookups.len()));
}
