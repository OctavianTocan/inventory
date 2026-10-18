//! Synthetic tests for `mymind seed-db` without recordings.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_sqlite::Sqlite;

fn work(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tool-seed-db-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn run_rust(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mymind"))
        .arg("seed-db")
        .args(args)
        .env_clear()
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("MYMIND_ROOT", root)
        .env("MYMIND_ENV_FILE", "fixture-vars")
        .env("STORE_LOAD_ENV", "0")
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn add_vectors(file: &Path, count: usize) {
    let db = Sqlite::open(file, false).unwrap();
    let cards = db.query("SELECT id, user_id FROM cards ORDER BY id LIMIT ?", &[SqlValue::from(count)]).unwrap();
    for (index, card) in cards.iter().enumerate() {
        let vector: Vec<u8> = (0..768).flat_map(|at| (((index * 31 + at * 7) % 97) as f32 / 97.0).to_le_bytes()).collect();
        db.execute("INSERT OR REPLACE INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, ?, 768, ?)", &[card.get("id").clone(), card.get("user_id").clone(), vector.into()]).unwrap();
    }
}

#[test]
fn fresh_seed_builds_synthetic_library_without_recordings() {
    let directory = work("fresh");
    let root = directory.join("empty-root");
    std::fs::create_dir_all(&root).unwrap();
    let db = directory.join("rust/b.db");
    let arg = format!("--db={}", db.display());
    let out = run_rust(&root, &directory, &[&arg]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("cards                  108"), "{stdout}");
    assert!(stdout.contains("trash                  2"), "{stdout}");
    let conn = Sqlite::open(&db, true).unwrap();
    let docs = conn.one("SELECT COUNT(*) AS n FROM search_documents", &[]).unwrap().unwrap().int("n").unwrap();
    assert_eq!(docs, 110);
}

#[test]
fn relative_db_and_argument_errors() {
    let directory = work("args");
    let root = directory.join("empty-root");
    let cwd = directory.join("rustcwd");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&cwd).unwrap();
    let out = run_rust(&root, &cwd, &["--db", "sub/x.db"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(cwd.join("sub/x.db").is_file());
    for args in [&["--db"][..], &["--bogus"], &["--db", ""], &["extra"]] {
        let rust = run_rust(&root, &cwd, args);
        assert_eq!(rust.status.code(), Some(1), "{args:?}");
    }
}

#[test]
fn reseed_keeps_embeddings() {
    let directory = work("reseed");
    let root = directory.join("empty-root");
    std::fs::create_dir_all(&root).unwrap();
    let db = directory.join("seeded.db");
    let out = run_rust(&root, &directory, &["--db", &db.to_string_lossy()]);
    assert_eq!(out.status.code(), Some(0));
    add_vectors(&db, 7);
    let out = run_rust(&root, &directory, &["--db", &db.to_string_lossy()]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("embeddingsKept         7"), "{stdout}");
}
