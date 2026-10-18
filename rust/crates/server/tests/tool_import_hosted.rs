//! Synthetic tests for `mymind import-hosted` without recordings.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::Rc;

use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_sqlite::Sqlite;

fn work(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tool-import-hosted-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn synthetic_root(directory: &Path) -> PathBuf {
    let root = directory.join("root");
    let mirror = root.join("mirror/static/134/0.109.0/scripts");
    std::fs::create_dir_all(&mirror).unwrap();
    std::fs::write(mirror.join("app.js"), "const u = 'https://static.accelerator.net/x.js';").unwrap();
    std::fs::write(root.join("mirror/static/favicon.png"), b"\x89PNG").unwrap();
    root
}

fn synthetic_db(path: &Path) -> PathBuf {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(path, false).unwrap());
    let _store = Store::new(db.clone()).unwrap();
    db.execute(
        "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES ('u1', 'a1', '00000000001', 0, '{}')",
        &[],
    )
    .unwrap();
    db.execute(
        "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
         VALUES ('00000000001abc', 'u1', 'Note', 0, '{\"type\":\"Note\"}', '<mymind-card></mymind-card>', '{}', 1, 1, 1)",
        &[],
    )
    .unwrap();
    path.to_path_buf()
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

#[test]
fn bucket_imports_mirror_static_and_localizes_text() {
    let directory = work("bucket");
    let root = synthetic_root(&directory);
    let out = directory.join("out");
    let res = run_rust(&root, &directory, &["bucket", "--out", &out.to_string_lossy()]);
    assert_eq!(res.status.code(), Some(0), "{}", String::from_utf8_lossy(&res.stderr));
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(stdout.contains("static: 2"), "{stdout}");
    let js = std::fs::read_to_string(out.join("bucket/static/134/0.109.0/scripts/app.js")).unwrap();
    assert!(js.contains("/_static/x.js"));
    assert!(out.join("bucket/static/favicon.png").is_file());
}

#[test]
fn library_imports_database_and_media_without_recordings() {
    let directory = work("library");
    let root = synthetic_root(&directory);
    let db = synthetic_db(&directory.join("source.db"));
    let media = media_folder(&directory);
    let out = directory.join("out");
    let res = run_rust(
        &root,
        &directory,
        &["library", "plain-1", "--db", &db.to_string_lossy(), "--media", &media.to_string_lossy(), "--out", &out.to_string_lossy()],
    );
    assert_eq!(res.status.code(), Some(0), "{}", String::from_utf8_lossy(&res.stderr));
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(stdout.contains("cards: 1"), "{stdout}");
    assert!(stdout.contains("media: 5"), "{stdout}");
    assert!(out.join("plain-1.db").is_file());
    assert!(out.join("bucket/libraries/plain-1/renditions/x/r2.webp").is_file());
    assert!(!out.join("bucket/libraries/plain-1/media/x.page-1.png").exists());
}

#[test]
fn usage_and_errors_are_enforced() {
    let directory = work("errors");
    let root = synthetic_root(&directory);
    let db = synthetic_db(&directory.join("source.db")).to_string_lossy().into_owned();
    for args in [
        vec![],
        vec!["bogus"],
        vec!["library"],
        vec!["library", "Bad_Name", "--db", &db],
        vec!["library", "ok"],
        vec!["library", "ok", "--db", ""],
        vec!["library", "ok", "--db", "/nonexistent/none.db", "--out", "o"],
        vec!["--bogus"],
        vec!["-x"],
        vec!["--db"],
    ] {
        let res = run_rust(&root, &directory, &args);
        assert!(matches!(res.status.code(), Some(1 | 2)), "{args:?}");
    }
}
