//! `mymind upload-hosted`: the port of `tools/upload-hosted.mjs`, which fills the Cloudflare Worker through its
//! `/_admin/` routes with the bucket, library rows and password user that `import-hosted` built.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use futures_util::future::try_join_all;
use mm_core::auth_store::{install_auth_schema, set_password};
use mm_core::host::Fetch;
use mm_core::http::{Body, Request, Response};
use mm_core::library_rows::{encode_value, table_digest};
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::{Value, obj};
use mm_js::{base64, inspect, json, number, text};
use mm_sqlite::Sqlite;

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::node_path::resolve;
use crate::parse_args::{Kind, Opt, parse};

const BATCH_BYTES: usize = 4 * 1024 * 1024;
const USAGE: &str = "Usage: node tools/upload-hosted.mjs <worker url> bucket|library <name>|user [<name>]|account <account> <library>|summary";

struct Admin {
    base: String,
    token: String,
    fetch: NativeFetch,
}

enum Payload {
    None,
    Json(Value),
    File(PathBuf),
}

struct Table {
    table: String,
    columns: Vec<String>,
    rowid: bool,
    parents: Vec<String>,
}

fn clip_units(text: &str, units: usize) -> String {
    String::from_utf16_lossy(&text.encode_utf16().take(units).collect::<Vec<_>>())
}

// `encodeURIComponent`.
fn encode_component(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

impl Admin {
    // Call one admin route; fails with the Worker's answer when it is not a success.
    async fn call(&self, route: &str, method: &str, body: Payload, headers: &[(String, String)]) -> Result<Response, String> {
        let url = format!("{}/_admin/{route}", self.base.strip_suffix('/').unwrap_or(&self.base));
        let mut request = Request::new(method, &url).map_err(|error| error.to_string())?;
        request.headers.set("authorization", format!("Bearer {}", self.token));
        for (name, value) in headers {
            request.headers.set(name.as_str(), value.as_str());
        }
        let response = match body {
            Payload::Json(value) => {
                request.headers.set("content-type", "application/json");
                request.body = Body::from(json::stringify(&value).ok().flatten().unwrap_or_default());
                self.fetch.fetch(request).await
            }
            Payload::File(path) => self.fetch.fetch_file(request, &path).await,
            Payload::None => self.fetch.fetch(request).await,
        }
        .map_err(|error| error.to_string())?;
        if !(200..300).contains(&response.status) && !(method == "HEAD" && response.status == 404) {
            let status = response.status;
            let body = response.body.bytes().await.map_err(|error| error.to_string())?;
            return Err(format!("{method} {route}: {status} {}", clip_units(&String::from_utf8_lossy(&body), 300)));
        }
        Ok(response)
    }

    // The route's JSON answer, or null when it is not JSON.
    async fn json(&self, route: &str, method: &str, body: Payload) -> Result<Value, String> {
        let response = self.call(route, method, body, &[]).await?;
        let bytes = response.body.bytes().await.map_err(|error| error.to_string())?;
        let decoded = String::from_utf8_lossy(&bytes);
        Ok(json::parse(decoded.strip_prefix('\u{feff}').unwrap_or(&decoded)).unwrap_or(Value::Null))
    }
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            files_under(&entry.path(), out)?;
        } else if kind.is_file() && !entry.file_name().to_string_lossy().ends_with(".tmp") {
            out.push(entry.path());
        }
    }
    Ok(())
}

async fn upload_bucket(admin: &Admin, cwd: &Path, bucket: &str, prefix: &str, force: bool, concurrency: &str) -> Result<(), String> {
    let root = resolve(cwd, bucket);
    let mut files = Vec::new();
    files_under(&root, &mut files).map_err(|error| error.to_string())?;
    let mut keys: Vec<String> = files
        .iter()
        .filter_map(|file| file.strip_prefix(&root).ok())
        .map(|relative| relative.components().map(|part| part.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/"))
        .filter(|key| !key.starts_with(".headers/") && key.starts_with(prefix))
        .collect();
    keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    let (uploaded, skipped, bytes) = (Cell::new(0u64), Cell::new(0u64), Cell::new(0u64));
    let limit = number::string_to_number(concurrency);
    if limit > 4_294_967_295.0 {
        return Err("Invalid array length".to_owned());
    }
    let workers = if limit >= 1.0 { limit.floor() as usize } else { 0 }.min(keys.len());
    let next = Cell::new(0usize);
    let task = |key: &str| {
        let key = key.to_owned();
        let (root, uploaded, skipped, bytes, total) = (root.clone(), &uploaded, &skipped, &bytes, keys.len());
        async move {
            let file = root.join(&key);
            let size = std::fs::metadata(&file).map_err(|error| error.to_string())?.len();
            let route = format!("bucket/{}", key.split('/').map(encode_component).collect::<Vec<_>>().join("/"));
            if !force {
                let head = admin.call(&route, "HEAD", Payload::None, &[]).await?;
                let reported = head.headers.get("x-object-size").map_or(0.0, number::string_to_number);
                if head.status == 200 && reported == size as f64 {
                    skipped.set(skipped.get() + 1);
                    return Ok::<(), String>(());
                }
            }
            let headers = match std::fs::read(root.join(".headers").join(format!("{key}.json"))) {
                Ok(bytes) if !bytes.is_empty() => {
                    let parsed = json::parse(&String::from_utf8_lossy(&bytes)).map_err(|error| error.message().to_owned())?;
                    vec![("x-object-headers".to_owned(), json::stringify(&parsed).ok().flatten().unwrap_or_default())]
                }
                _ => Vec::new(),
            };
            admin.call(&route, "PUT", Payload::File(file), &headers).await?;
            uploaded.set(uploaded.get() + 1);
            bytes.set(bytes.get() + size);
            if uploaded.get() % 250 == 0 {
                println!("{} uploaded, {} skipped of {total}", uploaded.get(), skipped.get());
            }
            Ok(())
        }
    };
    let pool = (0..workers).map(|_| async {
        while next.get() < keys.len() {
            let at = next.get();
            next.set(at + 1);
            task(&keys[at]).await?;
        }
        Ok::<(), String>(())
    });
    try_join_all(pool).await?;
    let counts = obj! { "objects" => keys.len() as f64, "uploaded" => uploaded.get() as f64, "skipped" => skipped.get() as f64, "bytes" => bytes.get() as f64 };
    let listing = admin.json(&format!("bucket?prefix={}", encode_component(prefix)), "GET", Payload::None).await?;
    println!("{}", inspect::console_line(&[counts.into(), listing]));
    Ok(())
}

fn without_rowid(sql: &str) -> bool {
    let lower = sql.to_lowercase();
    lower.match_indices("without").any(|(at, word)| {
        let rest = &lower[at + word.len()..];
        let trimmed = rest.trim_start_matches(text::is_whitespace);
        trimmed.len() < rest.len() && trimmed.starts_with("rowid")
    })
}

fn visit(tables: &[Table], at: usize, path: &mut Vec<usize>, ordered: &mut Vec<usize>) -> Result<(), String> {
    if ordered.contains(&at) {
        return Ok(());
    }
    if path.contains(&at) {
        return Err(format!("Foreign key cycle through {}", tables[at].table));
    }
    path.push(at);
    for parent in &tables[at].parents {
        let found = tables.iter().position(|candidate| candidate.table == *parent).ok_or("Cannot read properties of undefined (reading 'parents')")?;
        visit(tables, found, path, ordered)?;
    }
    path.pop();
    ordered.push(at);
    Ok(())
}

// The tables to copy, parents before children, with the columns that hold data and whether row ids are kept apart.
fn tables_of(db: &dyn Sql) -> Result<Vec<Table>, String> {
    let names = db
        .query("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' AND name != 'search_dirty'", &[])
        .map_err(|error| error.to_string())?;
    let mut tables = Vec::new();
    for entry in &names {
        let name = entry.text("name").unwrap_or("").to_owned();
        let columns = db.query("SELECT name, type, pk, hidden FROM pragma_table_xinfo(?)", &[name.as_str().into()]).map_err(|error| error.to_string())?;
        let keys: Vec<_> = columns.iter().filter(|column| column.int("pk").unwrap_or(0) != 0).collect();
        let alias = keys.len() == 1 && keys[0].text("type").unwrap_or("").to_uppercase() == "INTEGER";
        let rowid = !without_rowid(entry.text("sql").unwrap_or("")) && !alias;
        let mut parents: Vec<String> = Vec::new();
        for row in db.query("SELECT \"table\" AS parent FROM pragma_foreign_key_list(?)", &[name.as_str().into()]).map_err(|error| error.to_string())? {
            let parent = row.text("parent").unwrap_or("").to_owned();
            if !parents.contains(&parent) {
                parents.push(parent);
            }
        }
        parents.retain(|parent| *parent != name);
        let data = columns.iter().filter(|column| column.int("hidden") == Some(0)).map(|column| column.text("name").unwrap_or("").to_owned());
        let columns: Vec<String> = rowid.then(|| "rowid".to_owned()).into_iter().chain(data).collect();
        tables.push(Table { table: name, columns, rowid, parents });
    }
    let (mut ordered, mut path) = (Vec::new(), Vec::new());
    for at in 0..tables.len() {
        visit(&tables, at, &mut path, &mut ordered)?;
    }
    let mut slots: Vec<Option<Table>> = tables.into_iter().map(Some).collect();
    Ok(ordered.into_iter().filter_map(|at| slots[at].take()).collect())
}

// Pull only one upload batch and its next row, measuring JSON in the original UTF-16 units.
fn batches_of(mut rows: impl Iterator<Item = Result<Value, String>>) -> impl Iterator<Item = Result<Vec<Value>, String>> {
    let (mut pending, mut done) = (None, false);
    std::iter::from_fn(move || {
        if done {
            return None;
        }
        let (mut batch, mut bytes) = (Vec::new(), 0usize);
        loop {
            let (row, size) = match pending.take() {
                Some(row) => row,
                None => match rows.next() {
                    Some(Ok(row)) => {
                        let size = text::len(&json::stringify(&row).ok().flatten().unwrap_or_default());
                        (row, size)
                    }
                    Some(Err(error)) => {
                        done = true;
                        return Some(Err(error));
                    }
                    None => {
                        done = true;
                        return (!batch.is_empty()).then_some(Ok(batch));
                    }
                },
            };
            if bytes + size > BATCH_BYTES && !batch.is_empty() {
                pending = Some((row, size));
                return Some(Ok(batch));
            }
            batch.push(row);
            bytes += size;
        }
    })
}

async fn upload_library(admin: &Admin, cwd: &Path, root: &Path, name: Option<&str>, db_option: Option<&str>) -> Result<u8, String> {
    let name = name.unwrap_or("");
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
        return Err("Library names are lower-case letters, digits and dashes".to_owned());
    }
    let file = match db_option {
        Some(path) => resolve(cwd, path),
        None => resolve(cwd, &root.join("data/hosted").join(format!("{name}.db")).to_string_lossy()),
    };
    let db = Sqlite::open(&file, true).map_err(|error| error.to_string())?;
    let tables = tables_of(&db)?;
    let reset = admin.json(&format!("library/{name}/reset"), "POST", Payload::Json(obj! {}.into())).await?;
    if matches!(reset, Value::Null | Value::Undefined) {
        return Err(format!("Cannot read properties of {} (reading 'tables')", reset.to_js_string()));
    }
    let Value::Object(known) = reset.get("tables") else {
        return Err("Cannot use 'in' operator to search for a table in the Worker's reset answer".to_owned());
    };
    let missing: Vec<&str> = tables.iter().map(|table| table.table.as_str()).filter(|table| known.get(table).is_none()).collect();
    if !missing.is_empty() {
        return Err(format!("The Worker's schema has no table {}", missing.join(", ")));
    }
    for table in &tables {
        let columns = Value::Array(table.columns.iter().map(|column| Value::from(column.as_str())).collect());
        let mut sent = 0.0;
        let list = table.columns.iter().map(|column| format!("\"{column}\"")).collect::<Vec<_>>().join(", ");
        let order = if table.rowid { " ORDER BY rowid" } else { "" };
        let mut statement = db.connection().prepare(&format!("SELECT {list} FROM \"{}\"{order}", table.table)).map_err(|error| error.to_string())?;
        let mut rows = statement.query([]).map_err(|error| error.to_string())?;
        let encoded = std::iter::from_fn(|| {
            let row = match rows.next() {
                Ok(Some(row)) => row,
                Ok(None) => return None,
                Err(error) => return Some(Err(error.to_string())),
            };
            Some((0..table.columns.len()).map(|index| {
                let cell = row.get_ref(index).map_err(|error| error.to_string())?;
                let value = if let Ok(number) = cell.as_i64() {
                    SqlValue::Integer(number)
                } else if let Ok(number) = cell.as_f64() {
                    SqlValue::Real(number)
                } else if let Ok(bytes) = cell.as_blob() {
                    SqlValue::Blob(bytes.to_vec())
                } else if let Ok(bytes) = cell.as_bytes() {
                    SqlValue::Text(String::from_utf8_lossy(bytes).into_owned())
                } else {
                    SqlValue::Null
                };
                encode_value(&value).map_err(|error| error.to_string())
            }).collect::<Result<Vec<Value>, String>>().map(Value::Array))
        });
        for batch in batches_of(encoded) {
            let body = obj! { "table" => table.table.as_str(), "columns" => columns.clone(), "rows" => Value::Array(batch?) };
            let answer = admin.json(&format!("library/{name}/rows"), "POST", Payload::Json(body.into())).await?;
            if matches!(answer, Value::Null) {
                return Err("Cannot read properties of null (reading 'inserted')".to_owned());
            }
            sent += answer.get("inserted").as_f64().unwrap_or(f64::NAN);
        }
        println!("{}: {} rows", table.table, number::number_to_string(sent));
    }
    let dirty = db.query("SELECT id FROM search_dirty", &[]).map_err(|error| error.to_string())?;
    let dirty: Vec<Value> = dirty.iter().map(|row| Value::Array(vec![row.get("id").to_value()])).collect();
    let sequences = db.query("SELECT name, CAST(seq AS TEXT) AS seq FROM sqlite_sequence", &[]).map_err(|error| error.to_string())?;
    let sequences: Vec<Value> = sequences.iter().map(|row| Value::Object(row.to_object())).collect();
    admin.json(&format!("library/{name}/finish"), "POST", Payload::Json(obj! { "dirty" => Value::Array(dirty), "sequences" => Value::Array(sequences) }.into())).await?;
    drop(db);
    let mut checked: Vec<(&str, Vec<String>)> = tables.iter().map(|table| (table.table.as_str(), table.columns.clone())).collect();
    checked.push(("search_dirty", vec!["id".to_owned()]));
    let request = Value::Array(
        checked
            .iter()
            .map(|(table, columns)| obj! { "table" => *table, "columns" => Value::Array(columns.iter().map(|column| Value::from(column.as_str())).collect()) }.into())
            .collect(),
    );
    let remote = admin.json(&format!("library/{name}/digest"), "POST", Payload::Json(obj! { "tables" => request }.into())).await?;
    let local = Sqlite::open(&file, true).map_err(|error| error.to_string())?;
    let mut same = true;
    for (table, columns) in &checked {
        let mine = table_digest(&local, table, columns).map_err(|error| error.to_string())?;
        let theirs = remote.get(table);
        if matches!(theirs, Value::Undefined | Value::Null) {
            return Err("Cannot read properties of undefined (reading 'rows')".to_owned());
        }
        let matches = theirs.get("rows").as_f64() == Some(mine.rows as f64) && theirs.get("sha256").as_str() == Some(mine.sha256.as_str());
        same &= matches;
        println!("{} {table}: {} rows here, {} on the Worker", if matches { "same" } else { "DIFFERENT" }, mine.rows, theirs.get("rows").to_js_string());
    }
    println!("{}", if same { format!("library {name}: every table matches") } else { format!("library {name}: tables differ") });
    Ok(u8::from(!same))
}

async fn upload_user(admin: &Admin, env: &HashMap<String, String>, username: Option<&str>) -> Result<(), String> {
    let configured = env.get("TEST_USER_NAME").map(String::as_str);
    let password = env.get("TEST_USER_PASSWORD").map(String::as_str).filter(|password| !password.is_empty());
    let username = username.or(configured);
    let (Some(username), Some(password)) = (username.filter(|name| !name.is_empty()), password) else {
        return Err("Only the TEST_USER password user from .env can be uploaded".to_owned());
    };
    if Some(username) != configured {
        return Err("Only the TEST_USER password user from .env can be uploaded".to_owned());
    }
    let db = Sqlite::open(":memory:", false).map_err(|error| error.to_string())?;
    install_auth_schema(&db).map_err(|error| error.to_string())?;
    let name = set_password(&db, username, password).map_err(|error| error.to_string())?;
    let row = db
        .one("SELECT username, salt, hash, created_ms FROM users WHERE username = ?", &[SqlValue::from(name.as_str())])
        .map_err(|error| error.to_string())?
        .ok_or("Cannot read properties of undefined (reading 'username')")?;
    let body = obj! {
        "username" => row.text("username").unwrap_or(""),
        "salt" => base64::encode(row.blob("salt").unwrap_or(&[])),
        "hash" => base64::encode(row.blob("hash").unwrap_or(&[])),
        "created_ms" => row.get("created_ms").to_value(),
    };
    admin.call("auth/user", "POST", Payload::Json(body.into()), &[]).await?;
    println!("password user {} stored", row.text("username").unwrap_or(""));
    Ok(())
}

async fn summary(admin: &Admin) -> Result<(), String> {
    let summary = admin.json("auth", "GET", Payload::None).await?;
    if matches!(summary, Value::Null) {
        return Err("Cannot read properties of null (reading 'users')".to_owned());
    }
    let Value::Array(libraries) = summary.get("libraries") else {
        return Err("summary.libraries.map is not a function".to_owned());
    };
    let names = libraries
        .iter()
        .map(|entry| if matches!(entry, Value::Null | Value::Undefined) { Err("Cannot destructure property 'library' of an undefined value".to_owned()) } else { Ok(entry.get("library").clone()) })
        .collect::<Result<Vec<Value>, String>>()?;
    let head = obj! { "users" => summary.get("users").clone(), "libraries" => Value::Array(names), "sessions" => summary.get("sessions").clone(), "appTokens" => summary.get("appTokens").clone() };
    let bucket = admin.json("bucket", "GET", Payload::None).await?;
    println!("{}", inspect::console_line(&[head.into(), bucket]));
    Ok(())
}

async fn work(root: &Path, args: &[String]) -> Result<u8, String> {
    let file = match env_file::load(&env_file::path(root)) {
        Ok(Loaded::Vars(vars)) => vars,
        Ok(Loaded::Missing) => HashMap::new(),
        Err(_) => return Err("Could not load .env".to_owned()),
    };
    let env = env_file::merged(std::env::vars(), file);
    mm_core::runtime::set_env(env.clone());
    let declared = [
        Opt { name: "prefix", kind: Kind::Text },
        Opt { name: "force", kind: Kind::Flag },
        Opt { name: "db", kind: Kind::Text },
        Opt { name: "concurrency", kind: Kind::Text },
        Opt { name: "bucket", kind: Kind::Text },
    ];
    let parsed = parse(args, &declared, true)?;
    let positional = |index: usize| parsed.positionals.get(index).map(String::as_str);
    let (Some(base), Some(command)) = (positional(0).filter(|value| !value.is_empty()), positional(1).filter(|value| !value.is_empty())) else {
        eprintln!("{USAGE}");
        return Ok(2);
    };
    let Some(token) = env.get("ADMIN_TOKEN").filter(|token| !token.is_empty()) else {
        return Err("ADMIN_TOKEN is not set".to_owned());
    };
    let admin = Admin { base: base.to_owned(), token: token.clone(), fetch: NativeFetch::new() };
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    match command {
        "bucket" => {
            let default_bucket = root.join("data/hosted/bucket").to_string_lossy().into_owned();
            let bucket = parsed.text("bucket").unwrap_or(&default_bucket);
            upload_bucket(&admin, &cwd, bucket, parsed.text("prefix").unwrap_or(""), parsed.flag("force"), parsed.text("concurrency").unwrap_or("8")).await?;
            Ok(0)
        }
        "library" => upload_library(&admin, &cwd, root, positional(2), parsed.text("db")).await,
        "user" => upload_user(&admin, &env, positional(2)).await.map(|()| 0),
        "account" => {
            let shown = |index: usize| positional(index).unwrap_or("undefined").to_owned();
            let mut body = mm_js::Object::new();
            for (key, index) in [("account", 2), ("library", 3)] {
                if let Some(value) = positional(index) {
                    body.insert(key, value);
                }
            }
            admin.call("auth/library", "POST", Payload::Json(Value::Object(body)), &[]).await?;
            println!("{} reaches library {}", shown(2), shown(3));
            Ok(0)
        }
        "summary" => summary(&admin).await.map(|()| 0),
        other => {
            eprintln!("Unknown command {other}");
            Ok(2)
        }
    }
}

/// Fill the Worker through its admin routes (`tools/upload-hosted.mjs`).
///
/// # Arguments
/// * `root` - The repository root: `.env` (variables already set win) and the `data/hosted` defaults.
/// * `args` - The arguments after the subcommand name: `<worker url> bucket|library <name>|user [<name>]|account
///   <account> <library>|summary` with `--prefix`, `--force`, `--db`, `--concurrency` and `--bucket`.
///
/// # Returns
/// The exit status: 0, 2 for a usage error, 1 for a failure or a library whose tables differ.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    match work(root, args).await {
        Ok(status) => status,
        Err(message) => {
            eprintln!("Error: {message}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A synthetic single-column row with an exactly measurable JSON size.
    fn row(text: String) -> Value {
        Value::Array(vec![Value::String(text)])
    }

    // Measure the batch as the JavaScript uploader does, excluding the enclosing request.
    fn size(rows: &[Value]) -> usize {
        rows.iter().map(|row| text::len(&json::stringify(row).unwrap().unwrap())).sum()
    }

    #[test]
    fn batches_are_lazy_and_keep_only_one_lookahead_row() {
        let read = Cell::new(0);
        let rows = (0..1000).map(|_| {
            read.set(read.get() + 1);
            Ok(row("a".repeat(BATCH_BYTES / 4 - 4)))
        });
        let mut batches = batches_of(rows);
        assert_eq!(read.get(), 0);
        let first = batches.next().unwrap().unwrap();
        assert_eq!(first.len(), 4);
        assert_eq!(size(&first), BATCH_BYTES);
        assert_eq!(read.get(), 5);
        drop(first);
        let second = batches.next().unwrap().unwrap();
        assert_eq!(second.len(), 4);
        assert_eq!(size(&second), BATCH_BYTES);
        assert_eq!(read.get(), 9);
    }

    #[test]
    fn batches_measure_utf16_and_keep_oversized_rows_as_singletons() {
        let half = || row("\u{1f600}".repeat(BATCH_BYTES / 4 - 2));
        let oversized = row("x".repeat(BATCH_BYTES));
        let rows = [half(), half(), oversized, row("last".to_owned())];
        let batches = batches_of(rows.into_iter().map(Ok)).collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(batches.iter().map(Vec::len).collect::<Vec<_>>(), [2, 1, 1]);
        assert_eq!(size(&batches[0]), BATCH_BYTES);
        assert!(json::stringify(&batches[0][0]).unwrap().unwrap().len() > BATCH_BYTES / 2);
        assert_eq!(size(&batches[1]), BATCH_BYTES + 4);
        assert_eq!(batches[2][0], row("last".to_owned()));
        assert!(batches_of(std::iter::empty()).next().is_none());
    }

    #[test]
    fn later_row_errors_do_not_prevent_an_earlier_batch_upload() {
        let rows = [Ok(row("a".repeat(BATCH_BYTES - 4))), Ok(row("next".to_owned())), Err("late encoding failure".to_owned())];
        let mut batches = batches_of(rows.into_iter());
        assert_eq!(size(&batches.next().unwrap().unwrap()), BATCH_BYTES);
        assert_eq!(batches.next().unwrap(), Err("late encoding failure".to_owned()));
        assert!(batches.next().is_none());
    }
}
