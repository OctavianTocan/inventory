//! `mymind seed-db` against `tools/seed-db.mjs`: fresh seeds, reseeds that keep embeddings, argument forms and errors.

#[path = "../../core/tests/common/mod.rs"]
mod common;

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

// A project root for the JS tools: the tools are copied, so `ROOT` is this directory; the rest is linked.
fn js_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    std::fs::create_dir_all(&root).unwrap();
    let status = Command::new("cp").arg("-r").arg(common::root().join("tools")).arg(root.join("tools")).status().unwrap();
    assert!(status.success());
    for name in ["lib", "mirror", "capture", "replay", "node_modules", "package.json"] {
        std::os::unix::fs::symlink(common::root().join(name), root.join(name)).unwrap();
    }
    root
}

fn run_js(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new("node")
        .arg(root.join("tools/seed-db.mjs"))
        .args(args)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("NODE_NO_WARNINGS", "1")
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn run_rust(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mymind"))
        .arg("seed-db")
        .args(args)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("MYMIND_ROOT", root)
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
    let (js, rust) = (dump(js), dump(rust));
    assert_eq!(js.iter().map(|table| &table.0).collect::<Vec<_>>(), rust.iter().map(|table| &table.0).collect::<Vec<_>>());
    for (a, b) in js.iter().zip(&rust) {
        assert_eq!(a.1.len(), b.1.len(), "rows of {}", a.0);
        for (index, (x, y)) in a.1.iter().zip(&b.1).enumerate() {
            assert_eq!(x, y, "table {} row {index}", a.0);
        }
    }
}

fn text(bytes: &[u8], path: &Path, label: &str) -> String {
    String::from_utf8_lossy(bytes).replace(&path.to_string_lossy().into_owned(), label)
}

fn compare(root: &Path, cwd: &Path, rust_cwd: &Path, js_args: &[&str], rust_args: &[&str], js_db: &Path, rust_db: &Path) -> String {
    let js = run_js(root, cwd, js_args);
    let rust = run_rust(root, rust_cwd, rust_args);
    assert_eq!(js.status.code(), rust.status.code(), "status; JS stderr {}", String::from_utf8_lossy(&js.stderr));
    assert_eq!(text(&js.stdout, js_db, "<db>"), text(&rust.stdout, rust_db, "<db>"));
    assert_eq!(text(&js.stderr, js_db, "<db>"), text(&rust.stderr, rust_db, "<db>"));
    text(&js.stdout, js_db, "<db>")
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
    let directory = work("args");
    let root = js_root(&directory);
    let (js_cwd, rust_cwd) = (directory.join("jscwd"), directory.join("rustcwd"));
    std::fs::create_dir_all(&js_cwd).unwrap();
    std::fs::create_dir_all(&rust_cwd).unwrap();
    let out = compare(&root, &js_cwd, &rust_cwd, &["--db", "sub/x.db"], &["--db", "sub/x.db"], &js_cwd.join("sub/x.db"), &rust_cwd.join("sub/x.db"));
    assert!(out.contains("Seeded <db>"));
    same_tables(&js_cwd.join("sub/x.db"), &rust_cwd.join("sub/x.db"));
    for args in [&["--db"][..], &["--bogus"], &["--db", ""], &["extra"]] {
        let js = run_js(&root, &js_cwd, args);
        let rust = run_rust(&root, &rust_cwd, args);
        assert_eq!(js.status.code(), Some(1));
        assert_eq!(rust.status.code(), Some(1), "{args:?}");
        assert_eq!(js.stdout, rust.stdout);
        assert_eq!(js.stderr, rust.stderr, "{args:?}");
    }
}

#[test]
fn reseed_keeps_embeddings() {
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
    let status = run_js(&root, &directory, &["--db", &seeded.to_string_lossy()]).status;
    assert!(status.success());
    std::fs::copy(&seeded, &js_db).unwrap();
    std::fs::copy(&seeded, &rust_db).unwrap();
    add_vectors(&js_db, 7);
    add_vectors(&rust_db, 7);
    let out = compare(&root, &directory, &directory, &["--db", &js_arg], &["--db", &rust_arg], &js_db, &rust_db);
    assert!(out.contains("embeddingsKept         7"), "{out}");
    same_tables(&js_db, &rust_db);
}
