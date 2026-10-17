//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

#[path = "../../core/tests/common/mod.rs"]
mod common;

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::Rc;

use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_sqlite::Sqlite;

fn work(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tool-seed-db-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

// Both runners use isolated roots containing only immutable recording links by default.
fn js_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    std::fs::create_dir_all(&root).unwrap();
    tools_common::prepare_root(&root);
    root
}

fn run_js(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    let _ = (root, cwd, args);
    panic!("Legacy recording requires the recorded pre-switch revision")
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

fn cell(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => "null".to_owned(),
        SqlValue::Integer(number) => format!("i{number}"),
        SqlValue::Real(number) => format!("r{number}"),
        SqlValue::Text(text) => format!("t{text:?}"),
        SqlValue::Blob(bytes) => format!("b{}", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>()),
    }
}

// Every table below the FTS shadow tables, row by row in rowid order.
fn dump(file: &Path) -> Vec<(String, Vec<String>)> {
    let db = Sqlite::open(file, true).unwrap();
    let tables = db.query("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%' ORDER BY name", &[]).unwrap();
    tables
        .iter()
        .map(|table| {
            let name = table.text("name").unwrap().to_owned();
            let rows = db.query(&format!("SELECT * FROM \"{name}\" ORDER BY rowid"), &[]).or_else(|_| db.query(&format!("SELECT * FROM \"{name}\""), &[])).unwrap();
            (name, rows.iter().map(|row| row.values().iter().map(cell).collect::<Vec<_>>().join("|")).collect())
        })
        .collect()
}

fn same_tables(js: &Path, rust: &Path) {
    let rust_columns = tools_common::table_columns(rust);
    let js_columns = tools_common::recording().then(|| tools_common::table_columns(js));
    let rust = dump(rust);
    let js = tools_common::recording().then(|| dump(js));
    if let Some(js) = &js {
        assert!(rust == *js, "complete typed table rows and ordering differ");
    }
    let actual = format!("columns={rust_columns:?}\n{}", tools_common::row_manifest(&rust));
    let oracle = js.as_ref().map(|rows| format!("columns={:?}\n{}", js_columns.as_ref().unwrap(), tools_common::row_manifest(rows)));
    tools_common::check("ordered typed table rows", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
}

fn text(bytes: &[u8], path: &Path, label: &str) -> String {
    tools_common::legacy_help(&String::from_utf8_lossy(bytes)).replace(&path.to_string_lossy().into_owned(), label)
}

fn compare(root: &Path, cwd: &Path, rust_cwd: &Path, js_args: &[&str], rust_args: &[&str], js_db: &Path, rust_db: &Path) -> String {
    let js = tools_common::recording().then(|| run_js(root, cwd, js_args));
    let rust = run_rust(root, rust_cwd, rust_args);
    let show = |out: &Output, db: &Path| format!("status {:?}\nstdout {:?}\nstderr {:?}", out.status.code(), text(&out.stdout, db, "<db>"), text(&out.stderr, db, "<db>"));
    let actual = show(&rust, rust_db);
    let oracle = js.as_ref().map(|out| show(out, js_db));
    let request = format!("seed CLI {:?}", js_args).replace(&js_db.to_string_lossy().into_owned(), "<db>");
    tools_common::check(&request, actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    text(&rust.stdout, rust_db, "<db>")
}

fn embedding_count(file: &Path) -> i64 {
    let db = Sqlite::open(file, true).unwrap();
    db.query("SELECT COUNT(*) AS n FROM embeddings", &[]).unwrap()[0].int("n").unwrap()
}

// Give `count` cards a deterministic vector.
fn add_vectors(file: &Path, count: usize) {
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(file, false).unwrap());
    let _store = Store::new(db.clone()).unwrap();
    let cards = db.query("SELECT id, user_id FROM cards ORDER BY id LIMIT ?", &[SqlValue::from(count)]).unwrap();
    for (index, card) in cards.iter().enumerate() {
        let vector: Vec<u8> = (0..768).flat_map(|at| (((index * 31 + at * 7) % 97) as f32 / 97.0).to_le_bytes()).collect();
        db.execute("INSERT OR REPLACE INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, ?, 768, ?)", &[card.get("id").clone(), card.get("user_id").clone(), vector.into()]).unwrap();
    }
}

#[test]
fn fresh_seed_matches() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_seed_db.rs")));
    let directory = work("fresh");
    let root = js_root(&directory);
    let (js_db, rust_db) = (directory.join("js/a.db"), directory.join("rust/b.db"));
    let (js_arg, rust_arg) = (js_db.to_string_lossy().into_owned(), format!("--db={}", rust_db.display()));
    let out = compare(&root, &directory, &directory, &["--db", &js_arg], &[&rust_arg], &js_db, &rust_db);
    assert!(out.starts_with("Seeded <db>\n  cards "), "{out}");
    assert!(out.contains("\n  extrasFailures         "), "{out}");
    same_tables(&js_db, &rust_db);
    assert!(dump(&rust_db).iter().find(|table| table.0 == "search_documents").is_some_and(|table| !table.1.is_empty()));
}

#[test]
fn relative_db_and_argument_errors() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_seed_db.rs")));
    let directory = work("args");
    let root = js_root(&directory);
    let (js_cwd, rust_cwd) = (directory.join("jscwd"), directory.join("rustcwd"));
    std::fs::create_dir_all(&js_cwd).unwrap();
    std::fs::create_dir_all(&rust_cwd).unwrap();
    let out = compare(&root, &js_cwd, &rust_cwd, &["--db", "sub/x.db"], &["--db", "sub/x.db"], &js_cwd.join("sub/x.db"), &rust_cwd.join("sub/x.db"));
    assert!(out.contains("Seeded <db>"));
    same_tables(&js_cwd.join("sub/x.db"), &rust_cwd.join("sub/x.db"));
    for args in [&["--db"][..], &["--bogus"], &["--db", ""], &["extra"]] {
        let js = tools_common::recording().then(|| run_js(&root, &js_cwd, args));
        let rust = run_rust(&root, &rust_cwd, args);
        assert_eq!(rust.status.code(), Some(1), "{args:?}");
        let stderr = tools_common::legacy_help(&String::from_utf8_lossy(&rust.stderr)).into_bytes();
        let actual = format!("status {:?} stdout {:?} stderr {:?}", rust.status.code(), rust.stdout, stderr);
        let oracle = js.as_ref().map(|out| format!("status {:?} stdout {:?} stderr {:?}", out.status.code(), out.stdout, out.stderr));
        tools_common::check(&format!("argument error {args:?}"), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    }
}

#[test]
fn reseed_keeps_embeddings() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_seed_db.rs")));
    let directory = work("reseed");
    let root = js_root(&directory);
    let (js_db, rust_db) = (directory.join("js/a.db"), directory.join("rust/b.db"));
    let (js_arg, rust_arg) = (js_db.to_string_lossy().into_owned(), rust_db.to_string_lossy().into_owned());

    // A fixture copy: whatever vectors it has are kept or dropped the same way.
    std::fs::create_dir_all(js_db.parent().unwrap()).unwrap();
    std::fs::create_dir_all(rust_db.parent().unwrap()).unwrap();
    let fixture = common::test_db();
    std::fs::copy(&fixture, &js_db).unwrap();
    std::fs::copy(&fixture, &rust_db).unwrap();
    if embedding_count(&js_db) == 0 {
        add_vectors(&js_db, 5);
        add_vectors(&rust_db, 5);
    }
    assert!(embedding_count(&js_db) > 0);
    compare(&root, &directory, &directory, &["--db", &js_arg], &["--db", &rust_arg], &js_db, &rust_db);
    same_tables(&js_db, &rust_db);

    // A seeded database with vectors whose card text is unchanged keeps all of them.
    let seeded = directory.join("seeded.db");
    let js_seeded = directory.join("legacy-seeded.db");
    let out = compare(&root, &directory, &directory, &["--db", &js_seeded.to_string_lossy()], &["--db", &seeded.to_string_lossy()], &js_seeded, &seeded);
    assert!(out.starts_with("Seeded <db>"));
    same_tables(&js_seeded, &seeded);
    std::fs::copy(&seeded, &js_db).unwrap();
    std::fs::copy(&seeded, &rust_db).unwrap();
    add_vectors(&js_db, 7);
    add_vectors(&rust_db, 7);
    let out = compare(&root, &directory, &directory, &["--db", &js_arg], &["--db", &rust_arg], &js_db, &rust_db);
    assert!(out.contains("embeddingsKept         7"), "{out}");
    same_tables(&js_db, &rust_db);
}
