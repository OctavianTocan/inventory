//! Synthetic tests for `mymind try-ai` over a local database and media directory.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::path::Path;

use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_sqlite::Sqlite;
use tools_common::{Out, ai_fake, closed_url, request_lines, run_rust, tmproot};

fn seed_library(root: &Path) {
    let data_dir = root.join("data");
    std::fs::create_dir_all(data_dir.join("media/acct/c1")).unwrap();
    std::fs::create_dir_all(data_dir.join("media/acct/c2")).unwrap();
    std::fs::write(data_dir.join("media/acct/c1/photo.jpg"), b"\xff\xd8\xff\xe0synthetic-jpeg-bytes").unwrap();
    std::fs::write(data_dir.join("media/acct/c2/cover.webp"), b"RIFFxxxxWEBPVP8 synthetic-webp").unwrap();
    let db = std::rc::Rc::new(Sqlite::open(data_dir.join("mymind.db"), false).unwrap());
    let _store = Store::new(db.clone()).unwrap();
    db.execute(
        "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES ('u1', 'acct', '00000002UnE', 0, '{}')",
        &[],
    )
    .unwrap();
    let cards = [
        (
            "00000002UnE0O000000003",
            "Image",
            r#"{"type":"Image","title":"Mountain Sunrise","description":"Golden light over alpine peaks in autumn.","domain":"photos.example.com","source":{"url":"https://photos.example.com/1"},"tags":[{"id":"t1","name":"photography"},{"id":"t2","name":"nature"}]}"#,
            r#"<mymind-card data-id="00000002UnE0O000000003"><img src="/media/acct/c1/photo.jpg"></mymind-card>"#,
            300i64,
        ),
        (
            "00000002UnE0O000000002",
            "WebPage",
            r#"{"type":"WebPage","title":"Design Systems Handbook","description":"Component tokens, typography scales and accessible layout patterns for web interfaces.","domain":"design.example.com","source":{"url":"https://design.example.com/handbook"},"tags":[{"id":"t3","name":"design"},{"id":"t4","name":"ui"}]}"#,
            r#"<mymind-card data-id="00000002UnE0O000000002"><img src="/media/acct/c2/cover.webp"></mymind-card>"#,
            200i64,
        ),
        (
            "00000002UnE0O000000001",
            "Article",
            r#"{"type":"Article","title":"Async Rust Architecture","description":"Building deterministic single-threaded Tokio services with SQLite and zero external recordings.","domain":"rust.example.com","source":{"url":"https://rust.example.com/async"},"tags":[{"id":"t5","name":"rust"},{"id":"t6","name":"design"}]}"#,
            r#"<mymind-card data-id="00000002UnE0O000000001"><h3>Async Rust Architecture</h3></mymind-card>"#,
            100i64,
        ),
    ];
    for (id, kind, data, html, ts) in cards {
        db.execute(
            "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns) VALUES (?, 'u1', ?, 0, ?, ?, '{}', ?, ?, ?)",
            &[id.into(), kind.into(), data.into(), html.into(), ts.into(), ts.into(), ts.into()],
        )
        .unwrap();
    }
}

fn run_with_fake(
    label: &str,
    args: &[&str],
    extra_env: &[(&str, &str)],
    dot_env: Option<&str>,
    base_override: Option<&str>,
) -> (Out, Vec<String>) {
    let root = tmproot(label);
    seed_library(&root);
    let fake = ai_fake(true);
    let url = base_override.map_or_else(|| fake.url(), str::to_owned);
    let chat = format!("{url}/v1beta/openai");
    if let Some(contents) = dot_env {
        std::fs::write(root.join("fixture-vars"), contents).unwrap();
    }
    let mut env: Vec<(&str, &str)> = vec![
        ("GEMINI_BASE_URL", &url),
        ("AI_CHAT_BASE_URL", &chat),
        ("TYPESAFE_BASE_URL", &url),
    ];
    env.extend_from_slice(extra_env);
    let out = run_rust(&root, "try-ai", args, &env, None);
    (out, request_lines(&fake))
}

#[test]
fn usage_errors() {
    let default_keys = [("GEMINI_API_KEY", "fixture-key"), ("TYPESAFE_API_KEY", "fixture-jev")];
    for (label, args, message) in [
        ("unknown", &["--unknown"][..], "Unknown option '--unknown'"),
        ("zero-cards", &["--cards", "0"][..], "Usage: node tools/try-ai.mjs"),
        ("float-cards", &["--cards", "1.5"][..], "Usage: node tools/try-ai.mjs"),
        ("nan-cards", &["--cards", "abc"][..], "Usage: node tools/try-ai.mjs"),
        ("neg-vocab", &["--vocab=-1"][..], "Usage: node tools/try-ai.mjs"),
        ("Missing-val", &["--query"][..], "Option '--query <value>' argument missing"),
    ] {
        let (out, requests) = run_with_fake(label, args, &default_keys, None, None);
        assert_eq!(out.status, 1, "{label}");
        assert_eq!(out.stdout, "", "{label}");
        assert!(out.stderr.contains(message), "{label}: {}", out.stderr);
        assert!(requests.is_empty(), "{label}");
    }
}

#[test]
fn analyzes_recorded_cards() {
    let default_keys = [("GEMINI_API_KEY", "fixture-key"), ("TYPESAFE_API_KEY", "fixture-jev")];
    let (out, requests) = run_with_fake("defaults", &[], &default_keys, None, None);
    assert_eq!(out.status, 0, "stderr: {}", out.stderr);
    assert!(out.stdout.starts_with("library: 3 cards, vocabulary of 5 tags, analyzing 3"), "{}", out.stdout);
    assert!(out.stdout.contains("photo.jpg (image/jpeg)"), "{}", out.stdout);
    assert!(out.stdout.contains("cover.webp (image/webp)"), "{}", out.stdout);
    assert!(out.stdout.contains("embedding: 768 dims"), "{}", out.stdout);
    assert!(!requests.is_empty());
}

#[test]
fn ranks_given_queries() {
    let default_keys = [("GEMINI_API_KEY", "fixture-key"), ("TYPESAFE_API_KEY", "fixture-jev")];
    let (out, _) = run_with_fake(
        "queries",
        &["--cards", "2", "--vocab", "3", "--query", "design systems", "--query", "rust"],
        &default_keys,
        None,
        None,
    );
    assert_eq!(out.status, 0, "stderr: {}", out.stderr);
    assert!(out.stdout.contains("query \"design systems\""), "{}", out.stdout);
    assert!(out.stdout.contains("query \"rust\""), "{}", out.stdout);
}

#[test]
fn missing_keys_and_endpoints() {
    let (no_gemini, _) = run_with_fake("no-gemini", &["--cards", "1"], &[("TYPESAFE_API_KEY", "fixture-jev")], None, None);
    assert_eq!(no_gemini.status, 1);
    assert!(no_gemini.stderr.contains("GEMINI_API_KEY is not set"), "{}", no_gemini.stderr);

    let (no_jev, _) = run_with_fake("no-jev", &["--cards", "1"], &[("GEMINI_API_KEY", "fixture-key")], None, None);
    assert_eq!(no_jev.status, 0, "stderr: {}", no_jev.stderr);
    assert!(no_jev.stdout.contains("[chat]"), "{}", no_jev.stdout);

    let closed = closed_url();
    let default_keys = [("GEMINI_API_KEY", "fixture-key"), ("TYPESAFE_API_KEY", "fixture-jev")];
    let (dead, _) = run_with_fake("closed", &["--cards", "1"], &default_keys, None, Some(&closed));
    assert_eq!(dead.status, 1);
    assert!(dead.stderr.starts_with("error: "), "{}", dead.stderr);
}

#[test]
fn reads_keys_from_dot_env_without_overriding_the_environment() {
    let dot = "GEMINI_API_KEY=from-dot-env\nTYPESAFE_API_KEY=jev-from-dot-env\n";
    let (out, requests) = run_with_fake("dot-env", &["--cards", "1", "--query", "design"], &[], Some(dot), None);
    assert_eq!(out.status, 0, "stderr: {}", out.stderr);
    assert!(requests.iter().any(|line| line.contains("from-dot-env") || line.contains("jev-from-dot-env")), "{requests:?}");

    let (override_out, override_reqs) = run_with_fake(
        "dot-env-override",
        &["--cards", "1", "--query", "design"],
        &[("GEMINI_API_KEY", "env-wins"), ("TYPESAFE_API_KEY", "jev-env-wins")],
        Some(dot),
        None,
    );
    assert_eq!(override_out.status, 0, "stderr: {}", override_out.stderr);
    assert!(override_reqs.iter().any(|line| line.contains("jev-env-wins")), "{override_reqs:?}");
    assert!(!override_reqs.iter().any(|line| line.contains("jev-from-dot-env")), "{override_reqs:?}");
}
