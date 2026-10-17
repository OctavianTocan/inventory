//! `Api` against `createApi` of `lib/api.mjs` over copies of the owner fixture. Scripted requests in three phases
//! (writes with the hosted AI off, writes with a fake hosted AI, read-only) must answer alike, log and call the fakes
//! alike, broadcast the same hub events, and leave the same rows and upload files. Background enrichment finishes
//! after every request on both sides.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use mm_core::api::{Api, ApiOptions};
use mm_core::host::{AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::site::{LibraryApi, localize};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Error, Result, Value, arr, obj};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use mymind::uploads_fs::DirectoryUploads;
use regex::Regex;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/api.mjs");
const NOW: i64 = 1_787_000_000_000;
const TABLES: [&str; 13] = ["cards", "changes", "annotations", "spaces", "space_cards", "pins", "shares", "revisions", "users", "tag_names", "card_content", "uploads", "embeddings"];
const DESKTOP: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";
const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148";
const ANDROID: &str = "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Mobile Safari/537.36";

static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{(\w+)(?::(\d+))?\}\}").unwrap());
static UUID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"mymind-image-[0-9a-f-]{36}").unwrap());
static ELAPSED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""elapsed":\d+"#).unwrap());
static COUNTER: AtomicU8 = AtomicU8::new(0);

thread_local! {
    static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static FETCHES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn log(line: String) {
    LOG.with(|log| log.borrow_mut().push(line));
}

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        tokio::task::spawn_local(task);
    }

    fn sleep(&self, ms: u64) -> Task {
        Box::pin(tokio::time::sleep(Duration::from_millis(ms)))
    }

    fn log(&self, line: &str) {
        log(line.to_owned());
    }
}

fn random(size: usize) -> Vec<u8> {
    vec![COUNTER.fetch_add(1, Ordering::SeqCst).wrapping_add(1); size]
}

fn lists(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "card" => "SELECT id FROM cards ORDER BY id",
        "live" => "SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id",
        "deleted" => "SELECT id FROM cards WHERE deleted_ns IS NOT NULL ORDER BY id",
        "local" => "SELECT id FROM cards WHERE json_extract(extras, '$.local') = 1 ORDER BY id",
        "note" => "SELECT id FROM cards WHERE type = 'Note' ORDER BY id",
        "article" => "SELECT id FROM cards WHERE type = 'Article' ORDER BY id",
        "pinned" => "SELECT id FROM cards WHERE (flags & 512) != 0 ORDER BY id",
        "space" => "SELECT id FROM spaces ORDER BY id",
        "query" => "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id",
        "annotation" => "SELECT id FROM annotations ORDER BY id",
        "tag" => "SELECT DISTINCT name AS id FROM annotations WHERE name NOT GLOB '*[#&+%?]*' ORDER BY name",
        "share" => "SELECT id FROM shares ORDER BY created_ns DESC, id",
        "revision" => "SELECT id FROM revisions ORDER BY id",
        "user" => "SELECT id FROM users ORDER BY id",
        "account" => "SELECT account_uid AS id FROM users ORDER BY id",
        _ => return None,
    })
}

fn fetched(text: &[u8], url: &str, mime: &str) -> FetchedResource {
    FetchedResource { bytes: text.to_vec(), url: url.to_owned(), mime: mime.to_owned(), status: 200, headers: Headers::new() }
}

fn blocked() -> Error {
    Error::status(400, "Unsupported capture destination").with_code("CAPTURE_BLOCKED")
}

// The page or file a captured URL answers with, by the words in it.
fn resource(url: &str) -> Result<FetchedResource> {
    if url.contains("blocked") {
        return Err(blocked());
    }
    if url.contains("fail") {
        return Err(Error::internal("Page unavailable"));
    }
    if url.contains("img.example") {
        return Ok(fetched(if url.contains("bad") { b"XIMG" } else { b"PNGDATA" }, url, "image/png"));
    }
    if url.ends_with(".pdf") {
        return Ok(fetched(b"%PDF-1.4", url, "application/pdf"));
    }
    if url.contains("page") {
        let page = r#"<title>Page T | Example</title><meta property="og:image" content="https://img.example/1.png"><meta name="description" content="Described"><article><p>First paragraph.</p><p>Second.</p></article>"#;
        return Ok(fetched(page.as_bytes(), url, "text/html; charset=utf-8"));
    }
    Ok(fetched(b"<title>Plain</title><p>Body</p>", url, "text/html"))
}

struct FakeMedia;

#[async_trait(?Send)]
impl Media for FakeMedia {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!("the API never asks for a cover")
    }

    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let size: i64 = match &input {
            MediaInput::Bytes(bytes) => bytes.len() as i64,
            MediaInput::File(path) => std::fs::metadata(path).map_err(|error| Error::internal(error.to_string()))?.len() as i64,
            MediaInput::Upload(_) => -1,
            MediaInput::Key(_) => -2,
        };
        log(format!("[media] prepare {size} {content_type} {name} {} {id}", user.account_uid));
        if matches!(&input, MediaInput::Bytes(bytes) if bytes.first() == Some(&b'X')) {
            return Err(Error::status(422, "Media preview could not be generated"));
        }
        let key = format!("{}/{id}/{size}", user.account_uid);
        Ok(if content_type.starts_with("image/") {
            PreparedMedia {
                data: obj! { "type" => "Image", "title" => name, "blob" => obj! { "path" => format!("{key}.png"), "width" => 40.0, "height" => 30.0 } }.into(),
                extras: obj! { "cover" => obj! { "src" => format!("/media/{key}.png;800x600-t.webp"), "width" => 800.0, "height" => 600.0 } }.into(),
            }
        } else if content_type == "application/pdf" {
            PreparedMedia { data: obj! { "type" => "Document", "title" => name, "blob" => obj! { "path" => format!("{key}.pdf") }, "format" => "pdf" }.into(), extras: obj! {}.into() }
        } else {
            PreparedMedia { data: obj! { "type" => "File", "title" => name, "blob" => obj! { "path" => format!("{key}.bin") } }.into(), extras: obj! {}.into() }
        })
    }

    async fn send(&self, result: &MediaResult, method: &str, range: Option<&str>) -> Result<Response> {
        let headers = Headers::from_pairs([("content-type", result.content_type.as_str()), ("x-media", result.key.as_str()), ("x-range", range.unwrap_or(""))]);
        let body = if method == "HEAD" { String::new() } else { format!("MEDIA:{}", result.key) };
        Ok(Response::new(200, headers, Body::Bytes(body.into_bytes())))
    }

    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        Ok((!relative.contains("gone")).then(|| MediaResult { key: relative.to_owned(), size: 7, content_type: "image/png".into() }))
    }

    async fn image(&self, source: &Value) -> Result<Option<AnalysisImage>> {
        if !matches!(source, Value::Object(_) | Value::Array(_)) {
            return Ok(None);
        }
        let place = if source.get("path").is_nullish() { source.get("src") } else { source.get("path") };
        if !place.truthy() {
            return Ok(None);
        }
        Ok(Some(AnalysisImage { mime: "image/png".into(), data: format!("IMG:{}", place.to_js_string()).into_bytes() }))
    }
}

struct Capture;

#[async_trait(?Send)]
impl Resource for Capture {
    async fn fetch(&self, url: &str, options: ResourceOptions) -> Result<FetchedResource> {
        log(format!("[resource] {url} {} {}", options.offline, options.limit.map_or_else(|| "undefined".to_owned(), |limit| limit.to_string())));
        resource(url)
    }
}

#[async_trait(?Send)]
impl Screenshot for Capture {
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        log(format!("[screenshot] {url} {offline}"));
        if url.contains("noshot") {
            return Err(Error::internal("browser failed"));
        }
        Ok(b"SHOT".to_vec())
    }
}

fn emb(seed: u64) -> Value {
    Value::Array((0..768u64).map(|i| Value::Number(((i * 7 + seed * 3) % 13) as f64 / 5.0 - 1.1)).collect())
}

fn chat() -> String {
    let content = json::to_string(&obj! { "title" => "Generated title", "summary" => "A generated summary.", "tags" => arr!["alpha", "Beta tag", "design"] }.into());
    json::to_string(&obj! { "choices" => arr![obj! { "message" => obj! { "content" => content } }] }.into())
}

struct FakeFetch;

#[async_trait(?Send)]
impl Fetch for FakeFetch {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let url = request.url.as_str().to_owned();
        let route = [(":embedContent", "embedContent"), (":batchEmbedContents", "batchEmbedContents"), ("/chat/completions", "chat"), ("/v1/systemone", "systemone")]
            .iter()
            .find(|(suffix, _)| url.ends_with(suffix))
            .map_or("other", |(_, route)| route);
        let body = UUID.replace_all(&String::from_utf8_lossy(&request.take_body().bytes().await?), "mymind-image-UUID").into_owned();
        let headers: Vec<Value> = request.headers.iter().map(|(name, value)| arr![name, value]).collect();
        let entry = obj! { "route" => route, "url" => url.as_str(), "method" => request.method.as_str(), "headers" => Value::Array(headers), "body" => body.as_str() };
        FETCHES.with(|fetches| fetches.borrow_mut().push(json::to_string(&entry.into())));
        let (status, text) = match route {
            "embedContent" => (200, json::to_string(&obj! { "embedding" => obj! { "values" => emb(body.bytes().map(u64::from).sum::<u64>() % 11) } }.into())),
            "batchEmbedContents" => {
                let count = json::parse(&body)?.get("requests").as_array().map_or(0, Vec::len);
                let entries: Vec<Value> = (0..count as u64).map(|i| obj! { "values" => emb(i) }.into()).collect();
                (200, json::to_string(&obj! { "embeddings" => Value::Array(entries), "usageMetadata" => obj! { "totalTokenCount" => 3.0 } }.into()))
            }
            "chat" => (200, chat()),
            _ => (404, "{}".to_owned()),
        };
        Ok(Response::new(status, Headers::from_pairs([("content-type", "application/json")]), Body::Bytes(text.into_bytes())))
    }
}

// `JSON.stringify` with BigInts written as `<digits>n`, as the driver writes hub events.
fn plain(value: &Value) -> Value {
    match value {
        Value::BigInt(number) => Value::from(format!("{number}n")),
        Value::Array(items) => Value::Array(items.iter().map(plain).collect()),
        Value::Object(object) => Value::Object(object.iter().map(|(key, item)| (key.into_owned(), plain(item))).collect()),
        other => other.clone(),
    }
}

struct Names {
    uploads: Vec<String>,
    tokens: Vec<String>,
    last: String,
}

fn resolve(text: &str, db: &Sqlite, names: &Names) -> String {
    PLACEHOLDER
        .replace_all(text, |found: &regex::Captures| {
            let index: usize = found.get(2).map_or(0, |index| index.as_str().parse().unwrap());
            match &found[1] {
                "new" => names.last.clone(),
                "upload" => names.uploads.get(index).cloned().unwrap_or_else(|| "missing".into()),
                "token" => names.tokens.get(index).cloned().unwrap_or_else(|| "missing".into()),
                kind => match lists(kind) {
                    Some(sql) => db.query(sql, &[]).unwrap().get(index).and_then(|row| row.text("id").map(str::to_owned)).unwrap_or_else(|| "missing".into()),
                    None => found[0].to_owned(),
                },
            }
        })
        .into_owned()
}

fn remember(answer: &Value, names: &mut Names) {
    let status = answer.get("status").as_f64().unwrap_or(0.0);
    let json = answer.get("headers").as_array().is_some_and(|headers| {
        headers.iter().any(|header| header.at(0).as_str() == Some("content-type") && header.at(1).as_str().is_some_and(|value| value.contains("json")))
    });
    if !(200.0..300.0).contains(&status) || !json {
        return;
    }
    let Ok(parsed) = json::parse(&String::from_utf8_lossy(&base64::decode(answer.get("body").as_str().unwrap()))) else { return };
    let items = match parsed {
        Value::Array(items) => items,
        other => vec![other],
    };
    for item in items {
        if let Some(id) = item.get("id").as_str() {
            names.last = id.to_owned();
        }
        if let Some(url) = item.get("url").as_str()
            && url.starts_with("/uploads/")
            && item.get("token").truthy()
        {
            names.uploads.push(url.rsplit('/').next().unwrap().to_owned());
            names.tokens.push(item.get("token").to_js_string());
        }
    }
}

fn state(db: &Sqlite) -> Value {
    let mut state = obj! {};
    for table in TABLES {
        let columns: Vec<String> = db
            .query(&format!("PRAGMA table_info({table})"), &[])
            .unwrap()
            .iter()
            .map(|column| {
                let name = column.text("name").unwrap();
                format!(r#"CASE WHEN typeof("{name}") = 'integer' AND abs("{name}") > 9007199254740991 THEN CAST("{name}" AS TEXT) WHEN typeof("{name}") = 'blob' THEN hex("{name}") ELSE "{name}" END AS "{name}""#)
            })
            .collect();
        let rows: Vec<Value> =
            db.query(&format!("SELECT {} FROM {table} ORDER BY 1, 2", columns.join(", ")), &[]).unwrap().iter().map(|row| json::to_string(&Value::Object(row.to_object())).into()).collect();
        state.insert(table, Value::Array(rows));
    }
    state.into()
}

async fn rust_phase(file: &Path, upload_dir: &Path, phase: &Value, shell: &str) -> Value {
    COUNTER.store(0, Ordering::SeqCst);
    LOG.with(|log| log.borrow_mut().clear());
    FETCHES.with(|fetches| fetches.borrow_mut().clear());
    let env = phase.get("env").as_object().unwrap().iter().map(|(name, value)| (name.into_owned(), value.to_js_string())).collect::<HashMap<_, _>>();
    runtime::set_env(env);
    mm_js::date::set_fixed_now(Some(NOW));
    let db = Rc::new(Sqlite::open(file, false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    let events = Rc::new(RefCell::new(Vec::<Value>::new()));
    let sink = Rc::clone(&events);
    let api = Api::new(ApiOptions {
        store,
        media: Rc::new(FakeMedia),
        upload_storage: Rc::new(DirectoryUploads::new(upload_dir)),
        screenshot: Rc::new(Capture),
        resource: Rc::new(Capture),
        fetch: Rc::new(FakeFetch),
        readonly: phase.get("readonly").truthy(),
        localize,
        push: Box::new(move |target, payload| sink.borrow_mut().push(json::to_string(&plain(&arr![target, payload.clone()])).into())),
        has_replay: Box::new(|request| Ok(request.header("x-replay") == Some("1"))),
        random,
    })
    .await
    .unwrap();
    api.jobs().drain().await;
    let mut names = Names { uploads: Vec::new(), tokens: Vec::new(), last: "missing".into() };
    let mut answers = Vec::new();
    for step in phase.get("steps").as_array().unwrap() {
        if step.get("op").as_str() == Some("shell") {
            answers.push(obj! { "shell" => api.shell(shell).unwrap() }.into());
            continue;
        }
        let target = resolve(step.get("target").as_str().unwrap(), &db, &names);
        let url = mm_js::url::parse(&target, Some("http://localhost")).unwrap();
        let mut request = Request::new(step.get("method").as_str().unwrap(), url.as_str()).unwrap();
        request.target = target;
        if let Value::Object(headers) = step.get("headers") {
            request.headers = Headers::from_pairs(headers.iter().map(|(name, value)| (name.into_owned(), resolve(value.as_str().unwrap(), &db, &names))));
        }
        let body = match step.get("base64").as_str() {
            Some(encoded) => base64::decode(encoded),
            None => resolve(step.get("body").as_str().unwrap_or(""), &db, &names).into_bytes(),
        };
        request.body = Body::Bytes(body);
        let answer: Value = match api.handle(&mut request).await {
            Ok(Some(response)) => {
                let headers: Vec<Value> = response.headers.iter().map(|(name, value)| arr![name, value]).collect();
                let body = response.body.bytes().await.unwrap();
                obj! { "status" => f64::from(response.status), "headers" => Value::Array(headers), "body" => base64::encode(&body) }.into()
            }
            Ok(None) => obj! { "handled" => false }.into(),
            Err(_) => obj! { "threw" => true }.into(),
        };
        api.jobs().drain().await;
        remember(&answer, &mut names);
        answers.push(answer);
    }
    api.close();
    let mut files: Vec<Value> = std::fs::read_dir(upload_dir)
        .map(|entries| entries.flatten().map(|entry| (entry.file_name().into_string().unwrap(), base64::encode(&std::fs::read(entry.path()).unwrap()))).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .map(|(name, data)| arr![name, data])
        .collect();
    files.sort_by_key(|file| file.at(0).to_js_string());
    let mut fetches = FETCHES.with(|fetches| fetches.borrow().clone());
    fetches.sort();
    obj! {
        "answers" => Value::Array(answers),
        "events" => events.borrow().clone(),
        "log" => Value::Array(LOG.with(|log| log.borrow().iter().map(|line| Value::from(line.as_str())).collect())),
        "fetches" => Value::Array(fetches.into_iter().map(Value::from).collect()),
        "state" => state(&db),
        "files" => Value::Array(files),
    }
    .into()
}

fn doc(text: &str) -> Value {
    obj! { "type" => "doc", "content" => arr![obj! { "type" => "paragraph", "content" => arr![obj! { "type" => "text", "text" => text }] }] }.into()
}

struct Steps(Vec<Value>);

impl Steps {
    fn request(&mut self, method: &str, target: &str, headers: &[(&str, &str)], body: &str) -> &mut Self {
        let mut all = obj! { "user-agent" => DESKTOP };
        for (name, value) in headers {
            all.insert(*name, *value);
        }
        self.0.push(obj! { "method" => method, "target" => target, "headers" => all, "body" => body }.into());
        self
    }

    fn get(&mut self, target: &str) -> &mut Self {
        self.request("GET", target, &[], "")
    }

    fn json(&mut self, method: &str, target: &str, body: &Value) -> &mut Self {
        self.request(method, target, &[("content-type", "application/json")], &json::to_string(body))
    }

    fn bytes(&mut self, method: &str, target: &str, headers: &[(&str, &str)], body: &[u8]) -> &mut Self {
        self.request(method, target, headers, "");
        self.0.last_mut().unwrap().set("base64", base64::encode(body));
        self
    }

    fn shell(&mut self) -> &mut Self {
        self.0.push(obj! { "op" => "shell" }.into());
        self
    }
}

// A form part: name, file name and type when it is a file, and the bytes.
type Part<'a> = (&'a str, Option<(&'a str, &'a str)>, &'a [u8]);

fn multipart(parts: &[Part<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, file, data) in parts {
        out.extend_from_slice(b"--XBOUNDARY\r\n");
        match file {
            Some((filename, kind)) => out.extend_from_slice(format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {kind}\r\n\r\n").as_bytes()),
            None => out.extend_from_slice(format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes()),
        }
        out.extend_from_slice(data);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"--XBOUNDARY--\r\n");
    out
}

fn reads(steps: &mut Steps) {
    steps.shell().get("/cards").request("GET", "/cards", &[("user-agent", IPHONE)], "").get("/cards/trash").get("/spaces");
    for target in ["/spaces/{{space:0}}", "/spaces/00000002UnEglvwXKc2ws8", "/spaces/00000002UnEglw6cWhkxlS", "/spaces/{{space:5}}", "/spaces/nope"] {
        steps.get(target);
    }
    for target in ["/tags/manual", "/tags/recent", "/tags/recent?take=3", "/tags/recent?take=abc", "/tags/recent?take=-2", "/tags/recent?take=10"] {
        steps.get(target);
    }
    steps.get("/changes?since=0").request("GET", "/changes?since=1786990000000000000", &[("user-agent", IPHONE)], "");
    for target in ["/changes?since=abc", "/changes?since=99999999999999999999", "/changes?since=-1", "/changes?since=1787000000000000000"] {
        steps.get(target);
    }
    steps.get("/cards/{{card:0}}").request("GET", "/cards/{{card:0}}", &[("accept", "application/json")], "");
    steps.request("GET", "/cards/{{card:1}}", &[("user-agent", IPHONE)], "").request("GET", "/cards/{{card:1}}", &[("user-agent", ANDROID)], "");
    for target in ["/cards/nope", "/cards/{{card:2}}?template=PinnedCard", "/cards/00000002UnE0O4GN0NLA6A?template=PinnedCard", "/cards/{{card:2}}?template=Other", "/cards/{{card:3}}/detail", "/cards/nope/detail"] {
        steps.get(target);
    }
    steps.request("GET", "/cards/{{card:3}}/detail", &[("x-replay", "1")], "");
    for target in ["/objects/{{card:0}}", "/objects/{{card:0}}/tags", "/objects/{{card:0}}/content", "/objects/{{article:0}}/content", "/objects/{{card:0}}/revisions", "/objects/nope", "/objects/{{note:0}}"] {
        steps.get(target);
    }
    for target in ["/tags/suggest?q=de", "/tags/suggest", "/tags/detail?name={{tag:0}}", "/tags/detail?name={{tag:7}}", "/quota", "/quotas/objects", "/account.json"] {
        steps.get(target);
    }
    for target in [
        "/search?q=design",
        "/search?q=a&wildcard=true&semantic=true",
        "/search?q=type:note&q=design",
        "/search/suggestions?q=de",
        "/embeddings/search?q=hello",
        "/embeddings/related/{{card:0}}",
        "/queries/{{query:0}}/check",
        "/queries/nope/check",
        "/app/v1/cards?limit=3",
        "/app/v1/spaces",
    ] {
        steps.get(target);
    }
    steps.json("POST", "/cards/grid", &arr!["{{card:0}}", "{{card:1}}", "nope"]);
    steps.request("POST", "/cards/grid", &[("user-agent", IPHONE)], r#"["{{card:4}}"]"#);
    steps.request("POST", "/cards/grid", &[], r#"{"a":1}"#).request("POST", "/cards/grid", &[], "[1,null]").request("POST", "/cards/grid", &[], "[true]");
    steps.request("POST", "/cards/grid", &[], "not json").request("POST", "/cards/grid", &[], "");
    steps.get("/achievements/first?dismiss=true").get("/achievements/first?dismiss=true").get("/achievements/first");
    steps.get("/media/a/b.png").get("/media/gone.png").request("HEAD", "/_mediahost/a.png", &[("range", "bytes=0-3")], "");
    steps.request("GET", "/everything", &[("sec-fetch-dest", "document")], "").get("/shares/nope").request("GET", "/shares/nope", &[("user-agent", IPHONE)], "");
    steps.get("/%E0%A4%A").get("/unknown");
}

fn writes(steps: &mut Steps) {
    for body in [
        obj! { "content" => "hello\nworld" }.into(),
        obj! { "content" => "https://example.com/page/one" }.into(),
        obj! { "content" => "https://example.com/blocked" }.into(),
        obj! { "content" => "https://example.com/fail/x" }.into(),
        obj! { "content" => "https://example.com/doc.pdf" }.into(),
        obj! { "content" => "https://example.com/noshot" }.into(),
        obj! { "prose" => doc("draft"), "isDraft" => true }.into(),
        Value::Null,
        obj! { "type" => "Media" }.into(),
        obj! { "uploadId" => "nope" }.into(),
        obj! { "uploadId" => 5.0 }.into(),
        obj! { "uploadId" => true }.into(),
    ] {
        steps.json("POST", "/objects", &body);
    }
    steps.request("POST", "/objects", &[], "not json").request("POST", "/objects", &[("content-type", "text/markdown")], "# hi");
    steps.json("POST", "/spaces", &obj! { "name" => "One" }.into()).json("POST", "/spaces", &obj! { "name" => "Smart", "query" => obj! { "filters" => Value::Array(vec![]) } }.into());
    steps.json("POST", "/spaces", &obj! { "name" => " " }.into()).json("POST", "/spaces/reorder", &arr!["{{space:1}}", "{{space:0}}"]);
    steps.json("PATCH", "/spaces/{{space:0}}", &obj! { "name" => "Renamed" }.into()).json("PATCH", "/spaces/nope", &obj! {}.into());
    steps.request("PUT", "/spaces/{{space:0}}/objects/{{card:0}}", &[], "").request("PUT", "/spaces/{{space:0}}/objects/{{card:1}}", &[], "");
    steps.request("DELETE", "/spaces/{{space:0}}/objects/{{card:1}}", &[], "").request("POST", "/queries/{{query:0}}/objects/{{card:0}}/exclude", &[], "");
    steps.get("/spaces/{{space:0}}").get("/queries/{{query:0}}/check");
    steps.json("PATCH", "/objects/{{card:0}}", &obj! { "title" => "T", "description" => "D" }.into());
    steps.json("PATCH", "/objects/{{note:0}}/entity", &obj! { "title" => "Entity" }.into());
    steps.json("PUT", "/objects/{{note:0}}/prose", &doc("p1")).json("PATCH", "/objects/{{note:0}}/prose", &doc("p2")).json("PUT", "/objects/{{note:1}}/note", &doc("n1"));
    steps.json("PUT", "/objects/{{article:0}}/content", &doc("article body"));
    steps.json("PATCH", "/objects/{{note:0}}/commit", &obj! { "prose" => doc("c1") }.into()).json("POST", "/objects/{{note:1}}/commit", &obj! { "prose" => doc("c2") }.into());
    steps.request("POST", "/objects/{{note:1}}/commit", &[], "null");
    steps.get("/objects/{{note:0}}/revisions").get("/objects/{{article:0}}/content").get("/objects/{{note:0}}/content");
    steps.request("POST", "/objects/{{note:0}}/revisions/{{revision:0}}/restore", &[], "").request("POST", "/objects/{{note:0}}/revisions/zzz/restore", &[], "");
    steps.json("POST", "/objects/{{card:0}}/tags", &obj! { "name" => "plain" }.into()).json("POST", "/objects/{{card:0}}/tags", &obj! { "name" => "#private" }.into());
    steps.json("POST", "/objects/{{card:0}}/tags", &obj! {}.into()).request("DELETE", "/annotations/{{annotation:0}}", &[], "");
    steps.json("POST", "/objects/{{card:0}}/pin", &obj! { "position" => 1.0 }.into()).json("POST", "/objects/{{card:1}}/pin", &obj! {}.into());
    steps.json("POST", "/objects/{{card:1}}/unpin", &obj! {}.into()).request("POST", "/objects/{{card:2}}/complete", &[], "").request("POST", "/objects/{{card:2}}/uncomplete", &[], "");
    steps.request("POST", "/objects/{{card:3}}/pin", &[], "").json("POST", "/objects/reorder-pins", &arr!["{{pinned:1}}", "{{pinned:0}}"]);
    steps.shell().get("/cards/{{local:0}}?template=PinnedCard").get("/cards/{{local:1}}?template=PinnedCard").get("/cards/{{local:0}}/detail");
    steps.json("POST", "/batch/tag", &obj! { "objectIds" => arr!["{{card:4}}", "{{card:5}}"], "tag" => "batch" }.into());
    steps.json("POST", "/batch/tag", &obj! { "objectIds" => "x" }.into());
    steps.json("POST", "/batch/add-to-space", &obj! { "objectIds" => arr!["{{card:6}}"], "spaceId" => "{{space:0}}" }.into());
    steps.json("POST", "/batch/delete", &obj! { "objectIds" => arr!["{{card:7}}", obj! { "a" => 1.0 }] }.into());
    steps.json("POST", "/batch/delete", &obj! { "objectIds" => arr!["{{card:7}}", "{{card:8}}"] }.into());
    steps.request("POST", "/objects/{{deleted:0}}/restore", &[], "").request("DELETE", "/objects/{{card:9}}", &[], "").get("/cards/trash");
    steps.json("PATCH", "/users/{{user:0}}", &obj! { "colorScheme" => "dark", "onboardFlags" => 128.0 }.into());
    steps.json("PATCH", "/users/{{account:0}}", &obj! { "soundEnabled" => false }.into()).json("PATCH", "/users/nope", &obj! {}.into());
    for target in ["/onboard/flags/4/set", "/onboard/flags/4/unset", "/onboard/flags/99999999999/set"] {
        steps.request("POST", target, &[], "");
    }
    steps.shell();
    steps.request("POST", "/cards/{{card:0}}/shares", &[], "").get("/cards/{{card:0}}/detail").get("/shares/{{share:0}}");
    steps.request("DELETE", "/cards/{{card:0}}/shares/{{share:0}}", &[], "").get("/shares/{{share:0}}");
    steps.request("POST", "/cards/{{card:0}}/remove-ai-thumbnail", &[], "").request("POST", "/promotions/x/dismiss", &[], "");
    steps.request("POST", "/serendipity/complete", &[], "").json("POST", "/jserrors", &obj! { "m" => 1.0 }.into());
    let markdown = "# Title\n## Sub **bold**\n- a\n- **b**\n* star\n1. one\n2. two\n> quote *it*\n---\n\n```\ncode\nmore\n```\n[link](http://x)\n*it* `c` plain\r\n```";
    steps.request("POST", "/convert", &[("content-type", "text/markdown")], markdown).json("POST", "/convert", &obj! { "a" => 1.0 }.into());
    steps.request("POST", "/convert", &[], "\"plain *text*\"");
    steps.request("POST", "/cards/{{card:0}}/analyze", &[], "").request("POST", "/cards/nope/analyze", &[], "");
    steps.request("DELETE", "/spaces/{{space:0}}", &[], "").request("POST", "/unknown", &[], "").request("PUT", "/objects/{{card:0}}/unknown", &[], "");
    steps.request("POST", "/objects/{{card:0}}", &[], "").get("/changes?since=1786999999000000000");
    steps.request("POST", "/uploads/authorize", &[], "").request("POST", "/uploads/authorize?count=2", &[], "").request("POST", "/uploads/authorize?count=abc", &[], "");
    let put = |steps: &mut Steps, upload: &str, token: &str, kind: &str, name: &str, data: &[u8]| {
        let range = format!("bytes 0-{}/{}", data.len() - 1, data.len());
        let target = format!("/uploads/{upload}");
        let authorization = format!("Bearer {token}");
        steps.bytes("PUT", &target, &[("authorization", &authorization), ("content-range", &range), ("content-type", kind), ("x-file-name", name)], data);
    };
    put(steps, "{{upload:0}}", "{{token:0}}", "image/png", "pic.png", b"0123456789");
    steps.json("POST", "/objects", &obj! { "uploadId" => "{{upload:0}}" }.into()).json("POST", "/objects", &obj! { "uploadId" => "{{upload:0}}", "title" => "again" }.into());
    put(steps, "{{upload:1}}", "{{token:1}}", "application/pdf", "doc.pdf", b"%PDF-");
    steps.json("POST", "/objects/{{card:0}}/attachments", &obj! { "uploadId" => "{{upload:1}}" }.into());
    put(steps, "{{upload:2}}", "{{token:2}}", "image/jpeg", "photo.jpg", b"JPEGDATA");
    steps.json("POST", "/objects/{{card:0}}/attachments", &obj! { "uploadId" => "{{upload:2}}" }.into());
    put(steps, "{{upload:1}}", "{{token:0}}", "image/png", "wrong.png", b"abc");
    steps.json("POST", "/objects/{{card:1}}/attachments", &obj! { "remoteUrl" => "https://img.example/2.png" }.into());
    steps.json("POST", "/objects/{{card:1}}/attachments", &obj! { "remoteUrl" => "https://img.example/bad.png" }.into());
    steps.json("POST", "/objects/{{card:1}}/attachments", &obj! { "remoteUrl" => "https://blocked.example/x.png" }.into());
    steps.json("POST", "/objects/{{card:1}}/attachments", &obj! { "remoteUrl" => 5.0 }.into()).json("POST", "/objects/{{card:1}}/attachments", &obj! {}.into());
    steps.request("POST", "/objects/{{card:1}}/attachments", &[("content-type", "application/json")], "null");
    steps.request("POST", "/objects/{{card:1}}/attachments", &[("content-type", "application/json")], "bad");
    steps.bytes("POST", "/objects/{{card:1}}/attachments", &[("content-type", "image/png")], b"PNGRAW");
    steps.bytes("POST", "/objects/{{card:1}}/attachments", &[("content-type", "image/png")], b"XBAD");
    steps.bytes("POST", "/objects/nope/attachments", &[("content-type", "image/png")], b"PNGRAW");
    let form = "multipart/form-data; boundary=XBOUNDARY";
    let mut bom = b"\xEF\xBB\xBF".to_vec();
    bom.extend_from_slice(br#"{"title":"From file"}"#);
    steps.bytes("POST", "/objects", &[("content-type", form)], &multipart(&[("metadata", None, br#"{"title":"Pasted"}"#), ("blob", Some(("clip.png", "image/png")), b"PNGBYTES")]));
    steps.bytes("POST", "/objects", &[("content-type", form)], &multipart(&[("metadata", Some(("m.json", "application/json")), &bom), ("blob", Some(("clip.pdf", "application/pdf")), b"%PDF")]));
    steps.bytes("POST", "/objects", &[("content-type", form)], &multipart(&[("metadata", None, b"")]));
    steps.bytes("POST", "/objects", &[("content-type", form)], &multipart(&[("metadata", None, b"{bad"), ("blob", Some(("a.png", "image/png")), b"P")]));
    steps.bytes("POST", "/objects", &[("content-type", form)], b"garbage");
    steps.bytes("POST", "/objects", &[("content-type", form)], &multipart(&[("blob", Some(("x.png", "image/png")), b"XNOPE")]));
    steps.get("/cards/{{card:0}}").get("/objects/{{card:1}}").shell().get("/changes?since=1786999999000000000").get("/cards/trash");
}

fn ai(steps: &mut Steps) {
    steps.json("POST", "/objects", &obj! { "content" => "An idea about design systems" }.into());
    steps.json("POST", "/objects", &obj! { "content" => "https://example.com/page/two" }.into());
    steps.json("POST", "/objects", &obj! { "content" => "https://example.com/fail/three" }.into());
    steps.request("POST", "/cards/{{note:0}}/analyze", &[], "").request("POST", "/cards/{{local:0}}/analyze", &[], "");
    steps.json("PUT", "/objects/{{local:0}}/prose", &doc("edited"));
    steps.get("/search?q=design&semantic=true").get("/embeddings/search?q=hello").get("/embeddings/related/{{local:0}}").get("/search?q=idea");
    steps.get("/objects/{{local:0}}").get("/objects/{{local:1}}/tags").get("/changes?since=1786999999000000000");
}

fn readonly(steps: &mut Steps) {
    steps.shell().get("/cards").get("/search?q=design").request("GET", "/search?q=design", &[("x-replay", "1")], "");
    steps.request("GET", "/queries/{{query:0}}/check", &[("x-replay", "1")], "").get("/queries/{{query:0}}/check");
    steps.json("POST", "/objects", &obj! { "content" => "x" }.into()).request("POST", "/uploads/authorize", &[], "");
    steps.get("/achievements/first?dismiss=true").get("/cards/{{card:0}}/detail").request("POST", "/cards/{{card:0}}/analyze", &[], "");
    steps.bytes("POST", "/objects/{{card:1}}/attachments", &[("content-type", "image/png")], b"PNGRAW");
}

fn phase(name: &str, readonly: bool, env: Value, build: fn(&mut Steps)) -> (String, Value) {
    let mut steps = Steps(Vec::new());
    build(&mut steps);
    (name.to_owned(), obj! { "readonly" => readonly, "env" => env, "steps" => Value::Array(steps.0) }.into())
}

fn scratch(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("api-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

// Search answers carry their own timing.
fn untimed(answer: &Value) -> Value {
    let mut copy = answer.clone();
    if let Some(body) = answer.get("body").as_str() {
        let text = String::from_utf8_lossy(&base64::decode(body)).into_owned();
        if ELAPSED.is_match(&text) {
            copy.set("body", base64::encode(ELAPSED.replace_all(&text, r#""elapsed":0"#).as_bytes()));
        }
    }
    copy
}

fn compare(name: &str, phase: &Value, rust: &Value, js: &Value) -> usize {
    let steps = phase.get("steps").as_array().unwrap();
    let (rust_answers, js_answers) = (rust.get("answers").as_array().unwrap(), js.get("answers").as_array().unwrap());
    assert_eq!(rust_answers.len(), js_answers.len(), "{name}: answer count");
    let mut handled = 0;
    for (index, (rust_answer, js_answer)) in rust_answers.iter().zip(js_answers).enumerate() {
        let (rust_answer, js_answer) = (&untimed(rust_answer), &untimed(js_answer));
        let context = format!("{name} step {index}: {}", common::text(&steps[index]));
        if rust_answer != js_answer {
            let show = |answer: &Value| {
                let other = if std::ptr::eq(answer, rust_answer) { js_answer } else { rust_answer };
                let mut copy = answer.clone();
                for (key, encoded) in [("body", true), ("shell", false)] {
                    let Some(field) = answer.get(key).as_str() else { continue };
                    let decode = |text: &str| if encoded { base64::decode(text) } else { text.as_bytes().to_vec() };
                    let (mine, theirs) = (decode(field), decode(other.get(key).as_str().unwrap_or("")));
                    let at = mine.iter().zip(&theirs).position(|(a, b)| a != b).unwrap_or(mine.len().min(theirs.len()));
                    let window = &mine[at.saturating_sub(300)..(at + 300).min(mine.len())];
                    copy.set(key, format!("[{} bytes, first difference at {at}] …{}", mine.len(), String::from_utf8_lossy(window)));
                }
                common::text(&copy)
            };
            panic!("{context}\nRust: {}\nJS:   {}", show(rust_answer), show(js_answer));
        }
        handled += usize::from(!rust_answer.get("status").is_undefined());
    }
    for key in ["events", "log", "fetches", "files"] {
        assert_eq!(common::text(rust.get(key)), common::text(js.get(key)), "{name}: {key}");
    }
    for table in TABLES {
        let (rust_rows, js_rows) = (rust.get("state").get(table).as_array().unwrap(), js.get("state").get(table).as_array().unwrap());
        for (rust_row, js_row) in rust_rows.iter().zip(js_rows) {
            assert_eq!(rust_row, js_row, "{name}: {table} row");
        }
        assert_eq!(rust_rows.len(), js_rows.len(), "{name}: {table} rows");
    }
    handled
}

#[test]
fn api_matches_node() {
    runtime::install(&TestRuntime);
    let shell = String::from_utf8_lossy(&std::fs::read(common::root().join("capture/shell.html")).unwrap()).into_owned();
    let offline = obj! { "OFFLINE" => "1" }.into();
    let hosted_ai = obj! {
        "OFFLINE" => "1",
        "GEMINI_API_KEY" => "fake-gemini-key",
        "GEMINI_BASE_URL" => "http://127.0.0.1:9/v1beta",
        "AI_CHAT_BASE_URL" => "http://127.0.0.1:9/v1beta/openai",
    }
    .into();
    let phases = [
        phase("reads", false, obj! { "OFFLINE" => "1" }.into(), reads),
        phase("writes", false, offline, writes),
        phase("ai", false, hosted_ai, ai),
        phase("readonly", true, obj! { "OFFLINE" => "1" }.into(), readonly),
    ];
    let mut total = 0;
    for (name, phase) in &phases {
        let (js_copy, rust_copy) = (common::temp_copy(&common::test_db()), common::temp_copy(&common::test_db()));
        let (js_dir, rust_dir) = (scratch(&format!("{name}-js")), scratch(&format!("{name}-rust")));
        let js = common::call(DRIVER, "run", vec![js_copy.to_string_lossy().as_ref().into(), js_dir.join("uploads").to_string_lossy().as_ref().into(), phase.clone()]).unwrap();
        let rust = mymind::runtime::run(rust_phase(&rust_copy, &rust_dir.join("uploads"), phase, &shell));
        let handled = compare(name, phase, &rust, &js);
        println!("{name}: {} steps, {handled} answered, {} log lines, {} fetches", phase.get("steps").as_array().unwrap().len(), rust.get("log").as_array().unwrap().len(), rust.get("fetches").as_array().unwrap().len());
        total += handled;
        for dir in [js_dir, rust_dir] {
            let _ = std::fs::remove_dir_all(dir);
        }
        for copy in [js_copy, rust_copy] {
            let _ = std::fs::remove_dir_all(copy.parent().unwrap());
        }
    }
    mm_js::date::set_fixed_now(None);
    assert!(total > 180, "answered {total}");
}
