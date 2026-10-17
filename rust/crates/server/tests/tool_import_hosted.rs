//! `mymind import-hosted` against `tools/import-hosted.mjs`: stdout, the produced databases and the object stores.

#[path = "../../core/tests/common/mod.rs"]
mod common;

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mm_core::sql::{Sql, SqlValue};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

fn work(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tool-import-hosted-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

// Legacy code is available only while explicitly recording the immutable oracle.
fn js_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    if !root.exists() {
        std::fs::create_dir_all(&root).unwrap();
        tools_common::prepare_root(&root);
    }
    root
}

fn run_js(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    assert!(tools_common::recording(), "legacy import requires recording mode");
    Command::new("node")
        .arg(root.join("tools/import-hosted.mjs"))
        .args(args)
        .env_clear()
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", std::env::var_os("HOME").unwrap())
        .env("NODE_NO_WARNINGS", "1")
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn run_rust(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mymind"))
        .arg("import-hosted")
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
        SqlValue::Blob(bytes) => format!("b{}:{:x}", bytes.len(), Sha256::digest(bytes)),
    }
}

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

fn same_tables(js: &Path, rust: &Path) -> usize {
    let rust_columns = tools_common::table_columns(rust);
    let js_columns = tools_common::recording().then(|| tools_common::table_columns(js));
    let rust = dump(rust);
    let js = tools_common::recording().then(|| dump(js));
    if let Some(js) = &js {
        assert!(rust == *js, "complete typed database rows differ");
    }
    let actual = format!("columns={rust_columns:?}\n{}", tools_common::row_manifest(&rust));
    let oracle = js.as_ref().map(|rows| format!("columns={:?}\n{}", js_columns.as_ref().unwrap(), tools_common::row_manifest(rows)));
    tools_common::check("imported ordered typed tables", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    rust.iter().map(|table| table.1.len()).sum()
}

fn tree(base: &Path) -> BTreeMap<String, PathBuf> {
    let mut found = BTreeMap::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                pending.push(path);
            } else {
                found.insert(path.strip_prefix(base).unwrap().to_string_lossy().into_owned(), path);
            }
        }
    }
    found
}

// Relative object names, lengths and content hashes exclude timestamps and inodes.
fn same_tree(js: &Path, rust: &Path) -> usize {
    let manifest = |base: &Path| {
        tree(base).into_iter().map(|(name, path)| {
            let bytes = std::fs::read(path).unwrap();
            format!("{name:?} {} {:x}", bytes.len(), Sha256::digest(bytes))
        }).collect::<Vec<_>>()
    };
    let rust = manifest(rust);
    let js = tools_common::recording().then(|| manifest(js));
    let actual = format!("{rust:?}");
    let oracle = js.as_ref().map(|rows| format!("{rows:?}"));
    tools_common::check("imported object tree paths, sizes and hashes", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    rust.len()
}

fn text(bytes: &[u8], out: &Path) -> String {
    String::from_utf8_lossy(bytes).replace(&out.to_string_lossy().into_owned(), "<out>")
}

fn media_folder(directory: &Path) -> PathBuf {
    let media = directory.join("media-in");
    for (name, body) in [
        ("a.jpg", "jpeg"),
        ("sub/b.png", "png"),
        ("d.mp4", "mp4"),
        (".renditions/r1.webp", "one"),
        (".renditions/x/r2.webp", "two"),
        ("x.page-1.png", "page"),
        ("c.tmp", "temp"),
    ] {
        let file = media.join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, body).unwrap();
    }
    media
}

struct Outputs {
    js: PathBuf,
    rust: PathBuf,
}

// Run both implementations into outputs of one length, compare their output, and return the outputs.
fn compare(directory: &Path, tag: &str, args: &[&str]) -> (Outputs, String) {
    let root = js_root(directory);
    let out = Outputs { js: directory.join(format!("j{tag}/out")), rust: directory.join(format!("r{tag}/out")) };
    let with = |out: &Path| -> Vec<String> { args.iter().map(|arg| arg.replace("<out>", &out.to_string_lossy())).collect() };
    let (js_args, rust_args) = (with(&out.js), with(&out.rust));
    let js = tools_common::recording().then(|| run_js(&root, directory, &js_args.iter().map(String::as_str).collect::<Vec<_>>()));
    let rust = run_rust(&root, directory, &rust_args.iter().map(String::as_str).collect::<Vec<_>>());
    assert_eq!(rust.status.code(), Some(0), "{}", String::from_utf8_lossy(&rust.stderr));
    let show = |result: &Output, target: &Path| format!("status {:?}\nstdout {:?}\nstderr {:?}", result.status.code(), text(&result.stdout, target), text(&result.stderr, target));
    let actual = show(&rust, &out.rust);
    let oracle = js.as_ref().map(|result| show(result, &out.js));
    tools_common::check(&format!("import CLI {tag} {args:?}"), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    let stdout = text(&rust.stdout, &out.rust);
    (out, stdout)
}

#[test]
fn bucket_matches() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_import_hosted.rs")));
    let directory = work("bucket");
    let (out, stdout) = compare(&directory, "", &["bucket", "--out", "<out>"]);
    assert!(stdout.starts_with("{ static: "), "{stdout}");
    let files = same_tree(&out.js, &out.rust);
    assert!(files > 300, "{files}");
}

#[test]
fn library_with_recordings_and_runs_matches() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_import_hosted.rs")));
    let directory = work("runs");
    let db = common::temp_copy(&common::test_db());
    let media = media_folder(&directory);
    let started = std::time::Instant::now();
    let (out, stdout) = compare(
        &directory,
        "",
        &["library", "test", "--db", &db.to_string_lossy(), "--recordings", "--runs", "--media", &media.to_string_lossy(), "--out", "<out>"],
    );
    eprintln!("both runs took {:?}", started.elapsed());
    assert!(stdout.contains("captured: {"), "{stdout}");
    assert!(stdout.contains("media: 4"), "{stdout}");
    assert!(same_tables(&out.js.join("test.db"), &out.rust.join("test.db")) > 1000);
    let files = same_tree(&out.js.join("bucket"), &out.rust.join("bucket"));
    assert!(files > 5000, "{files}");
    assert!(out.rust.join("bucket/libraries/test/renditions/x/r2.webp").is_file());
    assert!(!out.rust.join("bucket/libraries/test/media/x.page-1.png").exists());
}

#[test]
fn library_without_recordings_matches() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_import_hosted.rs")));
    let directory = work("plain");
    let db = common::temp_copy(&common::hosted_db());
    let media = media_folder(&directory);
    let (out, stdout) = compare(&directory, "1", &["library", "plain-1", "--db", &db.to_string_lossy(), "--media", &media.to_string_lossy(), "--out", "<out>"]);
    assert!(!stdout.contains("captured"), "{stdout}");
    same_tables(&out.js.join("plain-1.db"), &out.rust.join("plain-1.db"));
    assert_eq!(same_tree(&out.js.join("bucket"), &out.rust.join("bucket")), 5);

    let (out, _) = compare(&directory, "2", &["library", "bare", "--db", &db.to_string_lossy(), "--out", "<out>"]);
    same_tables(&out.js.join("bare.db"), &out.rust.join("bare.db"));
}

#[test]
fn relative_paths_resolve_against_the_cwd() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_import_hosted.rs")));
    let directory = work("relative");
    let root = js_root(&directory);
    let db = common::temp_copy(&common::hosted_db());
    for side in ["jscwd", "rustcwd"] {
        std::fs::create_dir_all(directory.join(side)).unwrap();
        std::fs::copy(&db, directory.join(side).join("in.db")).unwrap();
    }
    let args = ["library", "rel", "--db", "in.db", "--out", "o/ut"];
    let js = tools_common::recording().then(|| run_js(&root, &directory.join("jscwd"), &args));
    let rust = run_rust(&root, &directory.join("rustcwd"), &args);
    assert_eq!(rust.status.code(), Some(0), "{}", String::from_utf8_lossy(&rust.stderr));
    let (js_out, rust_out) = (directory.join("jscwd/o/ut"), directory.join("rustcwd/o/ut"));
    let show = |result: &Output, target: &Path| format!("status {:?} stdout {:?} stderr {:?}", result.status.code(), text(&result.stdout, target), text(&result.stderr, target));
    let actual = show(&rust, &rust_out).replace("rustcwd", "X");
    let oracle = js.as_ref().map(|result| show(result, &js_out).replace("jscwd", "X"));
    tools_common::check("relative import cwd", actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    same_tables(&js_out.join("rel.db"), &rust_out.join("rel.db"));
}

// The `Error` message of an uncaught JS error, from the stack trace Node prints.
fn js_message(stderr: &str) -> String {
    let mut lines = stderr.lines().skip_while(|line| !(line.starts_with("Error") || line.starts_with("TypeError")));
    let first = lines.next().unwrap_or_default();
    let start = first.find("]: ").map(|at| at + 3).or_else(|| first.find("Error: ").map(|at| at + 7)).unwrap_or(0);
    let mut message = vec![first[start..].to_owned()];
    for line in lines {
        if line.is_empty() || line.starts_with(' ') || line.starts_with('{') || line.starts_with("Node.js") {
            break;
        }
        message.push(line.to_owned());
    }
    message.join("\n")
}

#[test]
fn usage_and_errors_match() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_import_hosted.rs")));
    let directory = work("errors");
    let root = js_root(&directory);
    let (js_cwd, rust_cwd) = (directory.join("jscwd"), directory.join("rustcwd"));
    std::fs::create_dir_all(&js_cwd).unwrap();
    std::fs::create_dir_all(&rust_cwd).unwrap();
    let db = common::temp_copy(&common::hosted_db());
    let db = db.to_string_lossy().into_owned();
    let cases: Vec<Vec<&str>> = vec![
        vec![],
        vec!["bogus"],
        vec!["--out", "o", "nothing"],
        vec!["library"],
        vec!["library", "Bad_Name", "--db", &db],
        vec!["library", "ok"],
        vec!["library", "ok", "--db", ""],
        vec!["library", "ok", "--db", &db, "--runs"],
        vec!["library", "ok", "--db", "/nonexistent/none.db", "--out", "o"],
        vec!["--bogus"],
        vec!["-x"],
        vec!["--db"],
        vec!["--media"],
        vec!["--out"],
        vec!["--db", "--media", "x"],
        vec!["--recordings=1"],
        vec!["--runs=yes"],
    ];
    for args in cases {
        let js = tools_common::recording().then(|| run_js(&root, &js_cwd, &args));
        let rust = run_rust(&root, &rust_cwd, &args);
        let rust_err = String::from_utf8_lossy(&rust.stderr).into_owned();
        assert!(matches!(rust.status.code(), Some(1 | 2)), "{args:?}");
        let actual = format!("status {:?} stdout {:?} stderr {:?}", rust.status.code(), rust.stdout, rust_err);
        let oracle = js.as_ref().map(|out| {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            let stderr = if out.status.code() == Some(1) { format!("Error: {}\n", js_message(&stderr)) } else { stderr };
            format!("status {:?} stdout {:?} stderr {:?}", out.status.code(), out.stdout, stderr)
        });
        tools_common::check(&format!("import argument error {args:?}"), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    }
}
