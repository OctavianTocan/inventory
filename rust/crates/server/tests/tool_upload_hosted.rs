//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.
//! receives in memory and answers the digest from them: the same printed lines, status, requests (sorted where the
//! JS runs uploads side by side) and password-user payload for every command.

#[path = "../../core/tests/common/mod.rs"]
mod common;
#[path = "tools_common/mod.rs"]
mod tools_common;

use std::collections::HashMap;
use std::path::Path;

use mm_core::library_rows::{insert_rows, table_digest};
use mm_core::sql::Sql;
use mm_core::{Value, obj};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};
use tools_common::{Fake, Out, Req, Resp, run_js, run_rust, tmproot};

const ADMIN: &str = "fixture-admin";

#[derive(Clone, Debug, Default)]
struct Mode {
    existing: Vec<(String, usize)>,
    drop_table: Option<String>,
    tamper: Option<String>,
    fail: Option<(String, String)>,
    plain_listing: bool,
    libraries: Vec<String>,
}

fn schema() -> Vec<String> {
    let db = Sqlite::open(common::hosted_db(), true).unwrap();
    db.query("SELECT sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' ORDER BY rowid", &[]).unwrap().iter().map(|row| row.text("sql").unwrap().to_owned()).collect()
}

fn names() -> Vec<String> {
    let db = Sqlite::open(common::hosted_db(), true).unwrap();
    db.query("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' ORDER BY rowid", &[]).unwrap().iter().map(|row| row.text("name").unwrap().to_owned()).collect()
}

fn answer(db: &Sqlite, mode: &Mode, objects: &mut HashMap<String, (usize, String)>, request: &Req) -> Resp {
    if request.header("authorization") != Some(&format!("Bearer {ADMIN}")) {
        return Resp::json(401, "{\"error\":\"unauthorized\"}");
    }
    if let Some((method, part)) = &mode.fail
        && &request.method == method
        && request.path.contains(part.as_str())
    {
        return Resp { status: 500, headers: Vec::new(), body: format!("boom {}", "x".repeat(400)).into_bytes() };
    }
    let route = request.path.strip_prefix("/_admin/").unwrap_or("");
    let body = json::parse(&String::from_utf8_lossy(&request.body)).unwrap_or(Value::Null);
    if let Some(key) = route.strip_prefix("bucket/") {
        let key = format!("bucket/{key}");
        return match request.method.as_str() {
            "HEAD" => match objects.get(&key) {
                Some((size, _)) => Resp { status: 200, headers: vec![("x-object-size".into(), size.to_string())], body: Vec::new() },
                None => Resp { status: 404, headers: Vec::new(), body: Vec::new() },
            },
            _ => {
                objects.insert(key, (request.body.len(), request.header("x-object-headers").unwrap_or("").to_owned()));
                Resp::json(200, "{\"ok\":true}")
            }
        };
    }
    if route == "bucket" {
        if mode.plain_listing {
            return Resp { status: 200, headers: Vec::new(), body: b"listing".to_vec() };
        }
        let prefix = request.query.strip_prefix("prefix=").unwrap_or("");
        let prefix = format!("bucket/{prefix}");
        let (count, bytes) = objects.iter().filter(|(key, _)| key.starts_with(&prefix) || prefix == "bucket/").fold((0, 0), |(count, bytes), (_, (size, _))| (count + 1, bytes + size));
        return Resp::json(200, &format!("{{\"objects\":{count},\"bytes\":{bytes}}}"));
    }
    if route == "auth" {
        let libraries: Vec<String> = mode.libraries.iter().map(|name| format!("{{\"library\":\"{name}\",\"extra\":1}}")).collect();
        return Resp::json(200, &format!("{{\"users\":2,\"libraries\":[{}],\"sessions\":5,\"appTokens\":0,\"ignored\":true}}", libraries.join(",")));
    }
    if route == "auth/user" || route == "auth/library" {
        return Resp::json(200, "{\"ok\":true}");
    }
    let Some(rest) = route.strip_prefix("library/") else { return Resp::json(404, "{}") };
    match rest.split_once('/').map(|(_, action)| action) {
        Some("reset") => {
            let mut tables = Vec::new();
            for name in names() {
                let _ = db.exec_batch(&format!("DELETE FROM \"{name}\""));
                if mode.drop_table.as_deref() != Some(name.as_str()) {
                    tables.push(format!("\"{name}\":0"));
                }
            }
            Resp::json(200, &format!("{{\"tables\":{{{}}}}}", tables.join(",")))
        }
        Some("rows") => {
            let columns: Vec<String> = body.get("columns").as_array().map(|list| list.iter().filter_map(|value| value.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            let rows: Vec<Vec<Value>> = body.get("rows").as_array().map(|list| list.iter().map(|row| row.as_array().cloned().unwrap_or_default()).collect()).unwrap_or_default();
            match insert_rows(db, body.get("table").as_str().unwrap_or(""), &columns, &rows) {
                Ok(count) => Resp::json(200, &format!("{{\"inserted\":{count}}}")),
                Err(error) => Resp::json(400, &format!("{{\"error\":{:?}}}", error.to_string())),
            }
        }
        Some("finish") => {
            let rows: Vec<Vec<Value>> = body.get("dirty").as_array().map(|list| list.iter().map(|row| row.as_array().cloned().unwrap_or_default()).collect()).unwrap_or_default();
            let _ = insert_rows(db, "search_dirty", &["id".to_owned()], &rows);
            Resp::json(200, "{\"ok\":true}")
        }
        Some("digest") => {
            let mut parts = Vec::new();
            for item in body.get("tables").as_array().cloned().unwrap_or_default() {
                let table = item.get("table").as_str().unwrap_or("").to_owned();
                let columns: Vec<String> = item.get("columns").as_array().map(|list| list.iter().filter_map(|value| value.as_str().map(str::to_owned)).collect()).unwrap_or_default();
                let digest = table_digest(db, &table, &columns).unwrap();
                let rows = digest.rows + usize::from(mode.tamper.as_deref() == Some(table.as_str()));
                parts.push(format!("\"{table}\":{{\"rows\":{rows},\"sha256\":\"{}\"}}", digest.sha256));
            }
            Resp::json(200, &format!("{{{}}}", parts.join(",")))
        }
        _ => Resp::json(404, "{}"),
    }
}

fn worker(mode: Mode) -> Fake {
    let schema = schema();
    Fake::start(move || {
        let db = Sqlite::open(":memory:", false).unwrap();
        for sql in &schema {
            db.exec_batch(sql).unwrap();
        }
        let mut objects: HashMap<String, (usize, String)> = mode.existing.iter().map(|(key, size)| (key.clone(), (*size, String::new()))).collect();
        Box::new(move |request: &Req| answer(&db, &mode, &mut objects, request))
    })
}

fn request_lines(fake: &Fake) -> Vec<String> {
    fake.log()
        .iter()
        .map(|request| {
            let body = if request.path.ends_with("/auth/user") {
                let parsed = json::parse(&String::from_utf8_lossy(&request.body)).unwrap();
                format!("user {} created {}", parsed.get("username").to_js_string(), parsed.get("created_ms").as_f64().is_some())
            } else {
                format!("{} bytes sha256 {}", request.body.len(), Sha256::digest(&request.body).iter().map(|byte| format!("{byte:02x}")).collect::<String>())
            };
            let header = |name: &str| request.header(name).unwrap_or("-");
            format!("{} {}?{} auth={} type={} object-headers={} {body}", request.method, request.path, request.query, header("authorization"), header("content-type"), header("x-object-headers"))
        })
        .collect()
}

#[derive(Debug)]
struct Case {
    name: &'static str,
    args: Vec<String>,
    env: Vec<(&'static str, String)>,
    mode: Mode,
    sorted: bool,
    exact_stderr: bool,
    dot_env: Option<String>,
    bucket: bool,
    db: bool,
}

fn case(name: &'static str, args: &[&str]) -> Case {
    Case { name, args: args.iter().map(|arg| (*arg).to_owned()).collect(), env: vec![("ADMIN_TOKEN", ADMIN.to_owned())], mode: Mode::default(), sorted: false, exact_stderr: false, dot_env: None, bucket: false, db: false }
}

fn bucket_files(root: &Path, many: bool) {
    let write = |key: &str, bytes: &[u8]| {
        let path = root.join("bucket").join(key);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    };
    write("a.txt", b"hello");
    write("dir/b.bin", &(0..2000u32).map(|n| (n % 251) as u8).collect::<Vec<_>>());
    write("dir/sub/c y&z.txt", b"spaces and an ampersand");
    write("dir/\u{e9}t\u{e9}.txt", "accents".as_bytes());
    write("zeta/one.txt", b"");
    write("skipme.tmp", b"partial download");
    write("dir/partial.tmp", b"partial");
    write(".headers/a.txt.json", b"{ \"content-type\" : \"text/plain\", \"cache-control\": \"max-age=60\" }");
    write(".headers/dir/b.bin.json", b"");
    write(".headers/zeta/one.txt.json", b"{\"content-type\":\"text/x-zeta\"}");
    if many {
        for index in 0..260 {
            write(&format!("many/{index:03}.txt"), format!("file {index}").as_bytes());
        }
    }
}

fn side(root: &Path, case: &Case, js: bool, many: bool) -> (Out, Vec<String>) {
    let fake = worker(case.mode.clone());
    let url = fake.url();
    if case.bucket {
        bucket_files(root, many);
    }
    if let Some(text) = &case.dot_env {
        if text == "<directory>" {
            std::fs::create_dir_all(root.join("fixture-vars")).unwrap();
        } else {
            std::fs::write(root.join("fixture-vars"), text).unwrap();
        }
    }
    let db = if case.db { Some(common::temp_copy(&common::hosted_db())) } else { None };
    let fill = |text: &str| {
        let text = text.replace("{url}", &url).replace("{root}", root.to_str().unwrap());
        match &db {
            Some(db) => text.replace("{db}", db.to_str().unwrap()),
            None => text,
        }
    };
    let args: Vec<String> = case.args.iter().map(|arg| fill(arg)).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let env: Vec<(&str, String)> = case.env.iter().map(|(key, value)| (*key, fill(value))).collect();
    let env: Vec<(&str, &str)> = env.iter().map(|(key, value)| (*key, value.as_str())).collect();
    let out = if js { run_js(root, "upload-hosted", &args, &env, None) } else { run_rust(root, "upload-hosted", &args, &env, None) };
    let mut lines = request_lines(&fake);
    if case.sorted {
        lines.sort();
    }
    (out, lines)
}

fn show(out: &Out, exact_stderr: bool) -> String {
    format!("status {}\nstdout {:?}\nstderr {:?}", out.status, out.stdout, if exact_stderr { out.stderr.as_str() } else { "<not compared>" })
}

fn compare(case: Case) -> (Out, Vec<String>) {
    let many = case.args.iter().any(|arg| arg == "many/");
    let (js_root, rust_root) = (tmproot("upload-hosted-js"), tmproot("upload-hosted"));
    let js = tools_common::recording().then(|| side(&js_root, &case, true, many));
    let rust = side(&rust_root, &case, false, many);
    let actual = format!("{}\nrequests {:?}", show(&rust.0, case.exact_stderr), rust.1);
    let oracle = js.as_ref().map(|out| format!("{}\nrequests {:?}", show(&out.0, case.exact_stderr), out.1));
    tools_common::check(&format!("{}: {case:?}", case.name), actual.as_bytes(), oracle.as_deref().map(str::as_bytes));
    rust
}

fn bucket(name: &'static str, extra: &[&str]) -> Case {
    let mut args = vec!["{url}", "bucket", "--bucket", "{root}/bucket"];
    args.extend(extra);
    let mut built = case(name, &args);
    built.bucket = true;
    built.sorted = true;
    built
}

#[test]
fn usage_and_environment() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut none = case("no arguments", &[]);
    none.exact_stderr = true;
    assert_eq!(compare(none).0.status, 2);
    let mut only_url = case("only a url", &["{url}"]);
    only_url.exact_stderr = true;
    assert_eq!(compare(only_url).0.status, 2);
    let mut unknown = case("unknown command", &["{url}", "frobnicate"]);
    unknown.exact_stderr = true;
    let (out, requests) = compare(unknown);
    assert_eq!((out.status, requests.len()), (2, 0));
    let mut no_token = case("no admin token", &["{url}", "summary"]);
    no_token.env.clear();
    let (out, requests) = compare(no_token);
    assert_eq!((out.status, requests.len()), (1, 0));
    assert!(out.stderr.contains("ADMIN_TOKEN is not set"));
    assert_eq!(compare(case("unknown option", &["{url}", "summary", "--bogus"])).0.status, 1);
    assert_eq!(compare(case("wrong token", &["{url}", "summary"])).0.status, 0);
    let mut wrong = case("wrong token", &["{url}", "summary"]);
    wrong.env = vec![("ADMIN_TOKEN", "not-the-token".into())];
    assert_eq!(compare(wrong).0.status, 1);
}

#[test]
fn reads_dot_env() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut file = case("token from .env", &["{url}", "summary"]);
    file.env.clear();
    file.dot_env = Some("# comment\nexport ADMIN_TOKEN=\"fixture-admin\"\n".into());
    assert_eq!(compare(file).0.status, 0);
    let mut shadowed = case("environment wins over .env", &["{url}", "summary"]);
    shadowed.dot_env = Some("ADMIN_TOKEN=from-the-file\n".into());
    assert_eq!(compare(shadowed).0.status, 0);
    let mut broken = case("unreadable .env", &["{url}", "summary"]);
    broken.dot_env = Some("<directory>".into());
    let (out, requests) = compare(broken);
    assert_eq!((out.status, requests.len()), (1, 0));
}

#[test]
fn summary_and_account() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut small = case("summary", &["{url}", "summary"]);
    small.mode.libraries = vec!["main".into(), "other".into()];
    let (out, _) = compare(small);
    assert!(out.stdout.contains("users: 2"), "{}", out.stdout);
    let mut wide = case("summary with many libraries", &["{url}/", "summary"]);
    wide.mode.libraries = (0..14).map(|index| format!("library-number-{index}")).collect();
    compare(wide);
    compare(case("summary without libraries", &["{url}", "summary"]));
    let mut plain = case("summary, plain bucket answer", &["{url}", "summary"]);
    plain.mode.plain_listing = true;
    let (out, _) = compare(plain);
    assert!(out.stdout.contains("null"), "{}", out.stdout);
    let (out, requests) = compare(case("account", &["{url}", "account", "someone@example.com", "main"]));
    assert!(out.stdout.contains("someone@example.com reaches library main"));
    assert_eq!(requests.len(), 1);
    compare(case("account, one argument", &["{url}", "account", "someone@example.com"]));
    compare(case("account, no argument", &["{url}", "account"]));
    let mut failing = case("account, worker error", &["{url}", "account", "a", "b"]);
    failing.mode.fail = Some(("POST".into(), "auth/library".into()));
    assert_eq!(compare(failing).0.status, 1);
}

#[test]
fn uploads_the_bucket() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut first = bucket("bucket", &[]);
    first.mode.existing = vec![("bucket/a.txt".into(), 99), ("bucket/zeta/one.txt".into(), 0), ("bucket/dir/b.bin".into(), 7)];
    let (out, requests) = compare(first);
    assert!(out.stdout.contains("objects: 5") && out.stdout.contains("skipped: 1"), "{}", out.stdout);
    assert!(requests.iter().any(|line| line.contains("object-headers=") && line.contains("max-age=60")), "{requests:?}");
    assert!(requests.iter().all(|line| !line.contains(".tmp")));
    compare(bucket("bucket, forced", &["--force"]));
    compare(bucket("bucket, prefix", &["--prefix", "dir/"]));
    compare(bucket("bucket, prefix without matches", &["--prefix", "nothing/"]));
    compare(bucket("bucket, one at a time", &["--concurrency", "1"]));
    compare(bucket("bucket, three at a time", &["--concurrency", "3"]));
    for value in ["0", "abc", "1.5", "-2", "", "  2 ", "1e1"] {
        compare(bucket("bucket, odd concurrency", &["--concurrency", value]));
    }
    let mut plain = bucket("bucket, plain listing", &[]);
    plain.mode.plain_listing = true;
    let (out, _) = compare(plain);
    assert!(out.stdout.trim_end().ends_with("null"), "{}", out.stdout);
}

#[test]
fn bucket_failures() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut empty_dir = case("bucket directory missing", &["{url}", "bucket", "--bucket", "{root}/nowhere"]);
    empty_dir.sorted = true;
    let (out, requests) = compare(empty_dir);
    assert_eq!((out.status, requests.len()), (1, 0));
    let mut put = bucket("bucket, put fails", &["--concurrency", "1"]);
    put.mode.fail = Some(("PUT".into(), "bucket/dir/b.bin".into()));
    assert_eq!(compare(put).0.status, 1);
    let mut head = bucket("bucket, head fails", &["--concurrency", "1"]);
    head.mode.fail = Some(("HEAD".into(), "bucket/a.txt".into()));
    assert_eq!(compare(head).0.status, 1);
    let mut default_dir = case("default bucket directory", &["{url}", "bucket"]);
    default_dir.sorted = true;
    assert_eq!(compare(default_dir).0.status, 1);
}

#[test]
fn logs_every_250_uploads() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut many = bucket("bucket, 260 files", &["--prefix", "many/", "--force", "--concurrency", "1"]);
    many.mode.libraries = Vec::new();
    let (out, requests) = compare(many);
    assert!(out.stdout.starts_with("250 uploaded, 0 skipped of 260\n"), "{}", out.stdout);
    assert_eq!(requests.len(), 261);
}

#[test]
fn uploads_a_library() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut full = case("library", &["{url}", "library", "test", "--db", "{db}"]);
    full.db = true;
    let (out, requests) = compare(full);
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert!(out.stdout.contains("same cards: 2280 rows here, 2280 on the Worker"), "{}", out.stdout);
    assert!(out.stdout.trim_end().ends_with("library test: every table matches"));
    assert!(requests.len() > 4);
}

#[test]
fn library_differences_and_failures() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let mut tampered = case("library, digest differs", &["{url}", "library", "test", "--db", "{db}"]);
    tampered.db = true;
    tampered.mode.tamper = Some("spaces".into());
    let (out, _) = compare(tampered);
    assert_eq!(out.status, 1);
    assert!(out.stdout.contains("DIFFERENT spaces: 16 rows here, 17 on the Worker"), "{}", out.stdout);
    assert!(out.stdout.trim_end().ends_with("library test: tables differ"));
    let mut missing = case("library, Worker lacks a table", &["{url}", "library", "test", "--db", "{db}"]);
    missing.db = true;
    missing.mode.drop_table = Some("pins".into());
    let (out, requests) = compare(missing);
    assert_eq!((out.status, requests.len()), (1, 1));
    assert!(out.stderr.contains("The Worker's schema has no table pins"), "{}", out.stderr);
    let mut rows_fail = case("library, rows rejected", &["{url}", "library", "test", "--db", "{db}"]);
    rows_fail.db = true;
    rows_fail.mode.fail = Some(("POST".into(), "/rows".into()));
    assert_eq!(compare(rows_fail).0.status, 1);
    for (name, args) in [("library, no name", &["{url}", "library"][..]), ("library, bad name", &["{url}", "library", "Bad_Name"]), ("library, missing database", &["{url}", "library", "absent"]), ("library, missing database file", &["{url}", "library", "absent", "--db", "{root}/absent.db"])] {
        let (out, requests) = compare(case(name, args));
        assert_eq!((out.status, requests.len()), (1, 0), "{name}");
    }
    let mut reset_fail = case("library, reset rejected", &["{url}", "library", "test", "--db", "{db}"]);
    reset_fail.db = true;
    reset_fail.mode.fail = Some(("POST".into(), "/reset".into()));
    assert_eq!(compare(reset_fail).0.status, 1);
}

#[test]
fn uploads_the_password_user() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    let named = |name| {
        let mut built = case(name, &["{url}", "user"]);
        built.env.extend([("TEST_USER_NAME", "Fixture-User".to_owned()), ("TEST_USER_PASSWORD", "fixture-password-123".to_owned())]);
        built
    };
    let (out, requests) = compare(named("user from the environment"));
    assert_eq!(out.stdout, "password user fixture-user stored\n");
    assert_eq!(requests.len(), 1);
    let mut explicit = named("user named on the command line");
    explicit.args.push("Fixture-User".into());
    compare(explicit);
    let mut other = named("a different user");
    other.args.push("someone-else".into());
    let (out, requests) = compare(other);
    assert_eq!((out.status, requests.len()), (1, 0));
    let mut no_password = named("no password");
    no_password.env.retain(|(key, _)| *key != "TEST_USER_PASSWORD");
    assert_eq!(compare(no_password).0.status, 1);
    let mut no_name = named("no name");
    no_name.env.retain(|(key, _)| *key != "TEST_USER_NAME");
    assert_eq!(compare(no_name).0.status, 1);
    let mut short = named("short password");
    short.env.retain(|(key, _)| *key != "TEST_USER_PASSWORD");
    short.env.push(("TEST_USER_PASSWORD", "short".into()));
    assert_eq!(compare(short).0.status, 1);
}

#[test]
fn the_uploaded_hash_accepts_the_password() {
    let _golden = tools_common::golden(file!(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "tool_upload_hosted.rs")));
    for js in [true, false].into_iter().filter(|js| !*js || tools_common::recording()) {
        let root = tmproot("upload-hosted-user");
        let fake = worker(Mode::default());
        let env = [("ADMIN_TOKEN", ADMIN), ("TEST_USER_NAME", "fixture-user"), ("TEST_USER_PASSWORD", "fixture-password-123")];
        let url = fake.url();
        let out = if js { run_js(&root, "upload-hosted", &[&url, "user"], &env, None) } else { run_rust(&root, "upload-hosted", &[&url, "user"], &env, None) };
        assert_eq!(out.status, 0, "{}", out.stderr);
        let body = json::parse(&String::from_utf8_lossy(&fake.log()[0].body)).unwrap();
        let salt = base64::decode(body.get("salt").as_str().unwrap());
        let hash = base64::decode(body.get("hash").as_str().unwrap());
        assert_eq!(mm_core::auth_store::hash_password("fixture-password-123", &salt), hash, "js: {js}");
        assert_eq!(body.get("username").as_str(), Some("fixture-user"));
        if !js {
            tools_common::check("uploaded password round trip", b"valid synthetic username, salt, hash and password", tools_common::recording().then_some(b"valid synthetic username, salt, hash and password".as_slice()));
        }
        let _ = obj! {};
    }
}
