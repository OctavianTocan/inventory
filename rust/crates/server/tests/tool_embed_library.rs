//! `mymind embed-library` against `tools/embed-library.mjs`, with a fake Gemini on 127.0.0.1: the same printed lines,
//! status, requests and database contents, for the estimate, `--limit`, `--run`, the guards and the failures.

#[path = "../../core/tests/common/mod.rs"]
mod common;
#[path = "tools_common/mod.rs"]
mod tools_common;

use std::path::Path;

use mm_core::sql::Sql;

use tools_common::{Out, ai_fake, assert_requests, db_state, request_lines, run_js, run_rust, tmproot};

struct Case {
    name: &'static str,
    args: Vec<&'static str>,
    env: Vec<(&'static str, String)>,
    usage: bool,
}

fn case(name: &'static str, args: &[&'static str]) -> Case {
    Case { name, args: args.to_vec(), env: Vec::new(), usage: true }
}

fn with(mut case: Case, key: &'static str, value: &str) -> Case {
    case.env.push((key, value.to_owned()));
    case
}

fn side(root: &Path, case: &Case, js: bool) -> (Out, Vec<String>, Vec<String>) {
    let db = common::temp_copy(&common::test_db());
    mm_sqlite::Sqlite::open(&db, false).unwrap().exec_batch("DELETE FROM embeddings WHERE card_id IN (SELECT card_id FROM embeddings ORDER BY card_id LIMIT 140)").unwrap();
    let fake = ai_fake(case.usage);
    let url = fake.url();
    let mut env: Vec<(&str, &str)> = case.env.iter().map(|(key, value)| (*key, value.as_str())).collect();
    if !env.iter().any(|(key, _)| *key == "GEMINI_BASE_URL") {
        env.push(("GEMINI_BASE_URL", &url));
    }
    let mut args: Vec<&str> = case.args.clone();
    args.extend(["--db", db.to_str().unwrap()]);
    let out = if js { run_js(root, "embed-library", &args, &env, None) } else { run_rust(root, "embed-library", &args, &env, None) };
    (out, request_lines(&fake), db_state(&db))
}

fn show(out: &Out) -> String {
    format!("status {}\nstdout {:?}\nstderr {:?}", out.status, out.stdout, out.stderr)
}

fn compare(case: Case) -> (Out, Vec<String>) {
    let (js_root, rust_root) = (tmproot("embed-library-js"), tmproot("embed-library"));
    let js = side(&js_root, &case, true);
    let rust = side(&rust_root, &case, false);
    assert_eq!(show(&rust.0), show(&js.0), "{}: output", case.name);
    assert_requests(case.name, &rust.1, &js.1);
    assert_eq!(rust.2, js.2, "{}: database", case.name);
    (js.0, js.1)
}

fn key(case: Case) -> Case {
    with(case, "GEMINI_API_KEY", "fixture-key")
}

#[test]
fn usage_errors() {
    for (name, args) in [("unknown flag", &["--bogus"][..]), ("limit without a value", &["--limit"]), ("limit not a number", &["--limit", "abc"]), ("limit fractional", &["--limit", "1.5"]), ("limit too large", &["--limit", "9007199254740993"]), ("stray word", &["run"])] {
        compare(case(name, args));
    }
}

#[test]
fn estimates_without_running() {
    let (out, _) = compare(case("estimate", &[]));
    assert!(out.stdout.contains("Estimate only. Add --run"), "{}", out.stdout);
    compare(case("estimate with a limit", &["--limit", "7"]));
    compare(case("estimate with limit zero", &["--limit", "0"]));
}

#[test]
fn run_guards() {
    compare(with(case("readonly", &["--run", "--limit", "4"]), "STORE_READONLY", "1"));
    compare(case("no key", &["--run", "--limit", "4"]));
    compare(with(with(case("offline, remote endpoint", &["--run", "--limit", "4"]), "OFFLINE", "1"), "GEMINI_BASE_URL", "http://example.invalid/v1beta"));
    compare(key(case("nothing pending", &["--run", "--limit", "0"])));
}

#[test]
fn embeds_pending_cards() {
    let (out, requests) = compare(key(case("run with a small limit", &["--run", "--limit", "4"])));
    assert!(out.stdout.contains("Embedded 4/4; API input tokens: 44"), "{}", out.stdout);
    assert_eq!(requests.len(), 1);
    compare(with(key(case("offline loopback", &["--run", "--limit", "3"])), "OFFLINE", "1"));
    let mut quiet = key(case("usage not reported", &["--run", "--limit", "5"]));
    quiet.usage = false;
    let (out, _) = compare(quiet);
    assert!(out.stdout.contains("API input tokens: not reported"));
    let (out, requests) = compare(key(case("two batches", &["--run", "--limit", "130"])));
    assert!(out.stdout.contains("Embedded 130/130"), "{}", out.stdout);
    assert_eq!(requests.len(), 2);
}

#[test]
fn a_failing_api_stops_the_run() {
    compare(with(key(case("dead endpoint", &["--run", "--limit", "2"])), "GEMINI_BASE_URL", &tools_common::closed_url()));
}
