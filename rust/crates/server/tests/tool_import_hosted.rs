//! `mymind import-hosted` against `tools/import-hosted.mjs`: stdout, the produced databases and the object stores.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
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

// A project root for the JS tool: the tools are copied, the rest of the repository is linked.
fn js_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    if root.exists() {
        return root;
    }
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
        .arg(root.join("tools/import-hosted.mjs"))
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
        .arg("import-hosted")
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
    let (js, rust) = (dump(js), dump(rust));
    assert_eq!(js.iter().map(|table| &table.0).collect::<Vec<_>>(), rust.iter().map(|table| &table.0).collect::<Vec<_>>());
    for (a, b) in js.iter().zip(&rust) {
        assert_eq!(a.1.len(), b.1.len(), "rows of {}", a.0);
        for (index, (x, y)) in a.1.iter().zip(&b.1).enumerate() {
            assert_eq!(x, y, "table {} row {index}", a.0);
        }
    }
    js.iter().map(|table| table.1.len()).sum()
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

// Same relative paths, sizes and contents; files linked from one source are the same by construction.
fn same_tree(js: &Path, rust: &Path) -> usize {
    let (js, rust) = (tree(js), tree(rust));
    assert_eq!(js.keys().collect::<Vec<_>>(), rust.keys().collect::<Vec<_>>());
    for (key, a) in &js {
        let (x, y) = (std::fs::metadata(a).unwrap(), std::fs::metadata(&rust[key]).unwrap());
        assert_eq!(x.len(), y.len(), "size of {key}");
        if (x.dev(), x.ino()) != (y.dev(), y.ino()) {
            assert_eq!(Sha256::digest(std::fs::read(a).unwrap()), Sha256::digest(std::fs::read(&rust[key]).unwrap()), "content of {key}");
        }
    }
    js.len()
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
    let js = run_js(&root, directory, &js_args.iter().map(String::as_str).collect::<Vec<_>>());
    let rust = run_rust(&root, directory, &rust_args.iter().map(String::as_str).collect::<Vec<_>>());
    assert_eq!(js.status.code(), rust.status.code(), "status; JS stderr {}", String::from_utf8_lossy(&js.stderr));
    assert_eq!(js.status.code(), Some(0), "{}", String::from_utf8_lossy(&js.stderr));
    let (js_out, rust_out) = (text(&js.stdout, &out.js), text(&rust.stdout, &out.rust));
    assert_eq!(js_out, rust_out);
    assert_eq!(text(&js.stderr, &out.js), text(&rust.stderr, &out.rust));
    (out, js_out)
}

#[test]
fn bucket_matches() {
    let directory = work("bucket");
    let (out, stdout) = compare(&directory, "", &["bucket", "--out", "<out>"]);
    assert!(stdout.starts_with("{ static: "), "{stdout}");
    let files = same_tree(&out.js, &out.rust);
    assert!(files > 300, "{files}");
}

#[test]
fn library_with_recordings_and_runs_matches() {
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
    assert!(stdout.contains("media: 5"), "{stdout}");
    assert!(same_tables(&out.js.join("test.db"), &out.rust.join("test.db")) > 1000);
    let files = same_tree(&out.js.join("bucket"), &out.rust.join("bucket"));
    assert!(files > 5000, "{files}");
    assert!(out.rust.join("bucket/libraries/test/renditions/x/r2.webp").is_file());
    assert!(!out.rust.join("bucket/libraries/test/media/x.page-1.png").exists());
}

#[test]
fn library_without_recordings_matches() {
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
    let directory = work("relative");
    let root = js_root(&directory);
    let db = common::temp_copy(&common::hosted_db());
    for side in ["jscwd", "rustcwd"] {
        std::fs::create_dir_all(directory.join(side)).unwrap();
        std::fs::copy(&db, directory.join(side).join("in.db")).unwrap();
    }
    let args = ["library", "rel", "--db", "in.db", "--out", "o/ut"];
    let js = run_js(&root, &directory.join("jscwd"), &args);
    let rust = run_rust(&root, &directory.join("rustcwd"), &args);
    assert_eq!(js.status.code(), Some(0), "{}", String::from_utf8_lossy(&js.stderr));
    assert_eq!(rust.status.code(), Some(0), "{}", String::from_utf8_lossy(&rust.stderr));
    let (js_out, rust_out) = (directory.join("jscwd/o/ut"), directory.join("rustcwd/o/ut"));
    assert_eq!(text(&js.stdout, &js_out).replace("jscwd", "X"), text(&rust.stdout, &rust_out).replace("rustcwd", "X"));
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
        let js = run_js(&root, &js_cwd, &args);
        let rust = run_rust(&root, &rust_cwd, &args);
        assert_eq!(js.status.code(), rust.status.code(), "{args:?}; JS stderr {}", String::from_utf8_lossy(&js.stderr));
        assert_eq!(js.stdout, rust.stdout, "{args:?}");
        let (js_err, rust_err) = (String::from_utf8_lossy(&js.stderr).into_owned(), String::from_utf8_lossy(&rust.stderr).into_owned());
        match js.status.code() {
            Some(2) => assert_eq!(js_err, rust_err, "{args:?}"),
            Some(1) => assert_eq!(rust_err, format!("Error: {}\n", js_message(&js_err)), "{args:?}"),
            other => panic!("{args:?}: status {other:?}"),
        }
    }
}
