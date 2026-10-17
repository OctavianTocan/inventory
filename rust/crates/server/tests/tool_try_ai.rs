//! `mymind try-ai` against `tools/try-ai.mjs`, with a fake Gemini, chat model and Jev on 127.0.0.1: the same printed
//! lines, status and requests over the repository's recorded cards.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::path::Path;

use tools_common::{Out, ai_fake, assert_requests, request_lines, run_js_flags, run_rust, tmproot};

struct Case {
    name: &'static str,
    args: Vec<&'static str>,
    env: Vec<(&'static str, String)>,
    dot_env: Option<String>,
    base_override: Option<&'static str>,
}

fn case(name: &'static str, args: &[&'static str]) -> Case {
    Case { name, args: args.to_vec(), env: vec![("GEMINI_API_KEY", "fixture-key".into()), ("TYPESAFE_API_KEY", "fixture-jev".into())], dot_env: None, base_override: None }
}

fn without(mut case: Case, key: &str) -> Case {
    case.env.retain(|(name, _)| *name != key);
    case
}

fn side(root: &Path, case: &Case, js: bool) -> (Out, Vec<String>) {
    let fake = ai_fake(true);
    let url = fake.url();
    let base = case.base_override.unwrap_or(&url).to_owned();
    let mut env: Vec<(&str, &str)> = case.env.iter().map(|(key, value)| (*key, value.as_str())).collect();
    env.extend([("GEMINI_BASE_URL", base.as_str()), ("AI_CHAT_BASE_URL", base.as_str()), ("TYPESAFE_BASE_URL", base.as_str())]);
    let flags: &[&str] = if case.dot_env.is_some() { &["--env-file-if-exists=.env"] } else { &[] };
    if let Some(text) = &case.dot_env {
        std::fs::write(root.join(".env"), text).unwrap();
    }
    let out = if js { run_js_flags(root, flags, "try-ai", &case.args, &env, None) } else { run_rust(root, "try-ai", &case.args, &env, None) };
    let mut lines = request_lines(&fake);
    lines.sort();
    (out, lines)
}

fn compare(case: Case) -> (Out, Vec<String>) {
    let (js_root, rust_root) = (tmproot("try-ai-js"), tmproot("try-ai"));
    let js = side(&js_root, &case, true);
    let rust = side(&rust_root, &case, false);
    // With every endpoint refusing, the chat and Gemini requests fail at once, and which rejection wins the race
    // (`Promise.all` in JS, `try_join3` in Rust) is timing-dependent in both runtimes.
    let refused = |text: &str| if case.base_override.is_some() { text.replace("gemini request failed", "<service> request failed").replace("chat request failed", "<service> request failed") } else { text.to_owned() };
    let show = |out: &Out| format!("status {}\nstdout {:?}\nstderr {:?}", out.status, out.stdout, refused(&out.stderr));
    assert_eq!(show(&rust.0), show(&js.0), "{}: output", case.name);
    assert_requests(case.name, &rust.1, &js.1);
    js
}

#[test]
fn usage_errors() {
    for (name, args) in [
        ("cards zero", &["--cards", "0"][..]),
        ("cards not a number", &["--cards", "x"]),
        ("cards fractional", &["--cards", "1.5"]),
        ("vocab negative", &["--vocab", "-1"]),
        ("unknown option", &["--bogus"]),
        ("stray word", &["hello"]),
        ("cards without a value", &["--cards"]),
    ] {
        let (out, requests) = compare(case(name, args));
        assert_eq!((out.status, out.stdout.as_str(), requests.len()), (1, "", 0), "{name}");
    }
}

#[test]
fn analyzes_recorded_cards() {
    let (out, requests) = compare(case("defaults", &[]));
    assert!(out.stdout.starts_with("library: "), "{}", out.stdout);
    assert!(out.stdout.contains("query \""), "{}", out.stdout);
    assert!(!requests.is_empty());
    compare(case("two cards", &["--cards", "2", "--vocab", "30"]));
    compare(case("one card, no vocabulary", &["--cards", "1", "--vocab", "0"]));
    compare(case("many cards", &["--cards", "9", "--vocab", "50"]));
}

#[test]
fn ranks_given_queries() {
    let (out, _) = compare(case("queries", &["--cards", "3", "--query", "dark mode interfaces", "--query", "recipes"]));
    assert!(out.stdout.contains("query \"recipes\""), "{}", out.stdout);
    compare(case("an empty query", &["--cards", "2", "--query", ""]));
}

#[test]
fn missing_keys_and_endpoints() {
    compare(without(case("no Jev key", &["--cards", "2"]), "TYPESAFE_API_KEY"));
    let (out, _) = compare(without(case("no Gemini key", &["--cards", "2"]), "GEMINI_API_KEY"));
    assert_eq!(out.status, 1);
    let mut dead = case("dead endpoint", &["--cards", "1"]);
    dead.base_override = Some(Box::leak(tools_common::closed_url().into_boxed_str()));
    compare(dead);
}

#[test]
fn reads_keys_from_dot_env_without_overriding_the_environment() {
    let mut file = case("keys from .env", &["--cards", "2"]);
    file.env.clear();
    file.dot_env = Some("GEMINI_API_KEY=fixture-key\nTYPESAFE_API_KEY=fixture-jev\n".into());
    let (out, requests) = compare(file);
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert!(requests.iter().all(|line| line.contains("fixture-key") || line.contains("fixture-jev")));
    let mut mixed = case("environment wins over .env", &["--cards", "2"]);
    mixed.dot_env = Some("GEMINI_API_KEY=from-the-file\nTYPESAFE_API_KEY=from-the-file\n".into());
    let (_, requests) = compare(mixed);
    assert!(requests.iter().all(|line| !line.contains("from-the-file")));
}
