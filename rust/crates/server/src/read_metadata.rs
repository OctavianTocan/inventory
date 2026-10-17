//! Port of `lib/read-metadata.mjs`: the recorded transport headers and reader documents a seeded library starts with.

use std::path::Path;
use std::sync::LazyLock;

use indexmap::IndexMap;
use mm_core::sql::{SqlExt, SqlValue};
use mm_core::store::Store;
use mm_core::{Error, Result, params};
use mm_js::{Object, Value, json, text, url};
use regex::Regex;

use crate::node_path::join;

static CONTENT_PATH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^/(?:objects/([^/]+)/content|cards/([^/]+)/reader)$").unwrap());
static EDITOR_SCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<mymind-editor(?-u:\b)[^>]*>[\s\S]*?<script(?-u:\b)[^>]*>([\s\S]*?)</script>").unwrap());
static METADATA_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/(cards(?:/[^/]+)?|spaces(?:/[^/]+)?|tags/(?:manual|recent)|changes)$").unwrap());
static SPACE_PATH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^/spaces/[^/]+$").unwrap());
static CAPTURED_CONTENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^GET /(objects/[^/]+/content|cards/[^/]+/reader)$").unwrap());

// `readFile(file, 'utf8')`, which keeps a byte order mark.
async fn read_text(file: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&read(file).await?).into_owned())
}

async fn read(file: &Path) -> Result<Vec<u8>> {
    tokio::fs::read(file).await.map_err(|error| Error::internal(format!("{error}: {}", file.display())))
}

// The file a recording entry names, below `dir`.
fn entry_file(root: &Path, dir: &str, entry: &Value) -> Result<std::path::PathBuf> {
    let Value::String(file) = entry.get("file") else {
        return Err(Error::invalid("The \"path\" argument must be of type string"));
    };
    Ok(join(root, &format!("{dir}/{file}")))
}

// Keep a recorded reader document so the card's first local edit starts from the captured prose.
fn remember_content(store: &Store, contents: &mut IndexMap<String, String>, path: &str, bytes: &[u8]) -> Result<()> {
    let Some(found) = CONTENT_PATH.captures(path) else { return Ok(()) };
    let reader = found.get(2).is_some();
    let id = found.get(1).or(found.get(2)).map_or("", |id| id.as_str());
    if store.card(id, None)?.is_none() {
        return Ok(());
    }
    let mut body = String::from_utf8_lossy(bytes).into_owned();
    if reader {
        body = EDITOR_SCRIPT.captures(&body).and_then(|found| found.get(1)).map_or(String::new(), |script| script.as_str().to_owned());
    }
    if text::trim(&body).is_empty() {
        return Ok(());
    }
    if let Ok(doc) = json::parse(&body)
        && doc.get("type").as_str() == Some("doc")
    {
        contents.insert(id.to_owned(), json::to_string(&doc));
    }
    Ok(())
}

fn status_value(status: &Value) -> Result<SqlValue> {
    match status {
        Value::Number(number) => Ok(SqlValue::Real(*number)),
        Value::Null => Ok(SqlValue::Null),
        _ => Err(Error::internal("Provided value cannot be bound to SQLite parameter 2.")),
    }
}

/// Import the recorded transport metadata and reader documents into a seeded library once (`importReadMetadata`).
/// Space detail recordings keep only their query-check time; no recorded card or space body is imported.
///
/// # Arguments
/// * `store` - Open store; a store that already has metadata is left as it is.
/// * `root` - Project directory with `replay/` and `capture/`.
///
/// # Errors
/// A recording that cannot be read or parsed, or the database's error.
pub async fn import_read_metadata(store: &Store, root: &Path) -> Result<()> {
    let db = store.db();
    if db.one("SELECT COUNT(*) AS n FROM api_metadata", &[])?.and_then(|row| row.number("n")).is_some_and(|count| count != 0.0) {
        return Ok(());
    }
    let index = json::parse(&read_text(&root.join("replay/index.json")).await.unwrap_or_else(|_| "{}".to_owned()))?;
    let mut rows: Vec<[SqlValue; 4]> = Vec::new();
    let mut contents: IndexMap<String, String> = IndexMap::new();
    let mut account = Value::Undefined;
    let empty = Object::new();
    for (key, entry) in index.as_object().unwrap_or(&empty).iter() {
        let Some(address) = key.strip_prefix("GET ").filter(|_| key.starts_with("GET https://access.mymind.com/")) else { continue };
        let address = url::parse(address, None).ok_or_else(|| Error::invalid(format!("Invalid URL: {address}")))?;
        let pathname = address.path();
        if CONTENT_PATH.is_match(pathname) {
            let bytes = read(&entry_file(root, "replay/bodies", entry)?).await?;
            remember_content(store, &mut contents, pathname, &bytes)?;
        }
        if pathname == "/account.json" {
            account = json::parse(&read_text(&entry_file(root, "replay/bodies", entry)?).await?)?;
        }
        if !METADATA_PATH.is_match(pathname) {
            continue;
        }
        let Value::Object(recorded) = entry.get("headers") else {
            return Err(Error::invalid("Cannot convert undefined or null to object"));
        };
        let mut headers = Object::new();
        for (name, value) in recorded.iter() {
            if ["content-type", "x-partial", "access-control-expose-headers"].contains(&&*name) {
                headers.insert(&*name, value.clone());
            }
        }
        let mut attributes = Object::new();
        if SPACE_PATH.is_match(pathname) {
            let space = json::parse(&read_text(&entry_file(root, "replay/bodies", entry)?).await?)?;
            let checked = space.get("query").get("checked");
            if checked.truthy() {
                attributes.insert("checked", checked.clone());
            }
        }
        rows.push([
            format!("{pathname}{}", url::search(&address)).into(),
            status_value(entry.get("status"))?,
            json::to_string(&Value::Object(headers)).into(),
            json::to_string(&Value::Object(attributes)).into(),
        ]);
    }
    let responses = json::parse(&read_text(&root.join("capture/responses.json")).await.unwrap_or_else(|_| "{}".to_owned()))?;
    for (key, entry) in responses.as_object().unwrap_or(&empty).iter() {
        let id = key.split('/').nth(2).unwrap_or("");
        if CAPTURED_CONTENT.is_match(&key) && !contents.contains_key(id) {
            let bytes = read(&entry_file(root, "capture", entry)?).await?;
            remember_content(store, &mut contents, &key[4..], &bytes)?;
        }
    }
    if !account.truthy() {
        let entry = responses.get("GET /account.json");
        if entry.truthy() {
            account = json::parse(&read_text(&entry_file(root, "capture", entry)?).await?)?;
        }
    }
    if account.truthy() {
        let attributes = json::to_string(&mm_js::obj! { "account" => account }.into());
        rows.push(["/account.json".into(), SqlValue::Real(200.0), "{\"content-type\":\"application/json; charset=utf-8\"}".into(), attributes.into()]);
    }
    store.transaction(|| {
        for row in &rows {
            db.execute("INSERT OR IGNORE INTO api_metadata VALUES (?, ?, ?, ?)", row)?;
        }
        for (id, body) in &contents {
            db.execute("INSERT OR IGNORE INTO card_content VALUES (?, ?)", params![id.as_str(), body.as_str()])?;
        }
        Ok(())
    })
}
