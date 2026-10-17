//! `hosted-media` against the JS backend: `createHostedMedia` and `bucketUploads` over a directory store on a sample of
//! the captured media, and the Rust port over an in-memory store holding the same objects. Both sides run the same
//! scripted steps with a fake processor, and every result, response, processor call and store operation is compared.

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use common::{JsError, text};
use futures_util::future::join_all;
use futures_util::stream;
use mm_core::host::{
    AnalysisImage, Bucket, ByteSpan, ListedObject, MediaInput, MediaProcessor, MediaUser, ObjectInfo, PreparedMedia, RenditionOutput,
    RenditionSource, StoredObject, UploadStorage,
};
use mm_core::hosted_media::{BucketUploads, HostedMedia, MediaResult};
use mm_core::http::Body;
use mm_core::{Error, Value, arr, obj};
use sha2::{Digest, Sha256};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            return Poll::Ready(());
        }
        self.0 = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn invalid_key() -> Error {
    Error::status(400, "Invalid object key")
}

/// The key `directoryBucket` would resolve inside its root, or its 400.
fn normalize(key: &str) -> Result<String, Error> {
    if key.starts_with('/') {
        return Err(invalid_key());
    }
    let mut parts: Vec<&str> = Vec::new();
    for part in key.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or_else(invalid_key)?;
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() { Err(invalid_key()) } else { Ok(parts.join("/")) }
}

#[derive(Default)]
struct MemoryBucket {
    objects: RefCell<BTreeMap<String, Vec<u8>>>,
    log: RefCell<Vec<Value>>,
}

fn bytes_stream(bytes: Vec<u8>) -> mm_core::http::ByteStream {
    Box::pin(stream::once(async move { Ok(bytes) }))
}

#[async_trait(?Send)]
impl Bucket for MemoryBucket {
    async fn head(&self, key: &str) -> mm_core::Result<Option<ObjectInfo>> {
        self.log.borrow_mut().push(arr!["head", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        Ok(self.objects.borrow().get(&key).map(|bytes| ObjectInfo { size: bytes.len() as u64, headers: None }))
    }

    async fn read(&self, key: &str, range: Option<ByteSpan>) -> mm_core::Result<Option<StoredObject>> {
        let span = range.map_or(Value::Null, |span| arr![span.offset as f64, span.length as f64]);
        self.log.borrow_mut().push(arr!["read", key, span]);
        YieldNow(false).await;
        let key = normalize(key)?;
        let Some(bytes) = self.objects.borrow().get(&key).cloned() else { return Ok(None) };
        let size = bytes.len() as u64;
        let body = match range {
            Some(span) if span.length == 0 => Vec::new(),
            Some(span) => {
                let start = (span.offset as usize).min(bytes.len());
                let end = (span.offset.saturating_add(span.length) as usize).min(bytes.len());
                bytes[start..end].to_vec()
            }
            None => bytes,
        };
        Ok(Some(StoredObject { size, headers: None, body: bytes_stream(body) }))
    }

    async fn bytes(&self, key: &str) -> mm_core::Result<Option<Vec<u8>>> {
        self.log.borrow_mut().push(arr!["bytes", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        Ok(self.objects.borrow().get(&key).cloned())
    }

    async fn write(&self, key: &str, body: Body, length: Option<u64>, headers: Option<&mm_core::http::Headers>) -> mm_core::Result<()> {
        self.log.borrow_mut().push(arr!["write", key, length.map(|length| length as f64), headers.is_some()]);
        YieldNow(false).await;
        let key = normalize(key)?;
        let bytes = body.bytes().await?;
        self.objects.borrow_mut().insert(key, bytes);
        Ok(())
    }

    async fn remove(&self, key: &str) -> mm_core::Result<()> {
        self.log.borrow_mut().push(arr!["remove", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        self.objects.borrow_mut().remove(&key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> mm_core::Result<Vec<ListedObject>> {
        self.log.borrow_mut().push(arr!["list", prefix]);
        YieldNow(false).await;
        if let Some(index) = prefix.rfind('/').filter(|index| *index > 0) {
            normalize(&prefix[..index])?;
        }
        Ok(self
            .objects
            .borrow()
            .iter()
            .filter(|(key, _)| key.starts_with(prefix) && !key.starts_with(".headers/") && !key.ends_with(".tmp"))
            .map(|(key, bytes)| ListedObject { key: key.clone(), size: bytes.len() as u64 })
            .collect())
    }
}

struct FakeProcessor {
    log: Rc<RefCell<Vec<Value>>>,
}

fn extension(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => "png",
        "application/pdf" => "pdf",
        "video/mp4" => "mp4",
        _ => "bin",
    }
}

#[async_trait(?Send)]
impl MediaProcessor for FakeProcessor {
    async fn prepare(&self, body: Body, content_type: &str, name: &str, user: &MediaUser, id: &str) -> mm_core::Result<PreparedMedia> {
        let bytes = body.bytes().await?;
        let digest = sha(&bytes);
        let who = obj! { "accountUid" => user.account_uid.as_str(), "idPrefix" => user.id_prefix.as_str() };
        self.log.borrow_mut().push(arr![
            "prepare",
            obj! { "length" => bytes.len() as f64, "sha" => digest.as_str(), "type" => content_type, "name" => name, "user" => who, "id" => id }
        ]);
        match name {
            "fail-plain" => return Err(Error::internal("boom")),
            "fail-415" => return Err(Error::status(415, "Unsupported media")),
            "noblob" => return Ok(PreparedMedia { data: obj! { "type" => "Image" }.into(), extras: obj! {}.into() }),
            "nopath" => return Ok(PreparedMedia { data: obj! { "type" => "Image", "blob" => obj! {} }.into(), extras: obj! {}.into() }),
            _ => {}
        }
        let file = match name {
            "travpath" => "../escape.bin".to_owned(),
            "escape" => "../../../../escape.bin".to_owned(),
            _ => format!("{}/{}/{}.{}", user.account_uid, id.get(user.id_prefix.len()..).unwrap_or(""), &digest[..8], extension(content_type)),
        };
        let blob = obj! { "id" => id, "path" => file.as_str(), "type" => content_type, "size" => bytes.len() as f64, "name" => name };
        let cover = obj! { "src" => format!("/media/{file};800x450-t.webp"), "width" => 800.0, "height" => 450.0 };
        Ok(PreparedMedia {
            data: obj! { "type" => "Image", "title" => name, "blob" => blob }.into(),
            extras: obj! { "cover" => cover }.into(),
        })
    }

    async fn rendition(&self, original: RenditionSource, relative: &str) -> mm_core::Result<Option<RenditionOutput>> {
        let size = original.size;
        let name = original.name.clone();
        let bytes = original.body.bytes().await?;
        let digest = sha(&bytes);
        self.log.borrow_mut().push(arr![
            "rendition",
            obj! { "name" => name, "size" => size as f64, "sha" => digest.as_str(), "relative" => relative }
        ]);
        if relative.contains("null") {
            return Ok(None);
        }
        if relative.contains("boom") {
            return Err(Error::internal("rendition boom"));
        }
        if relative.contains("teapot") {
            return Err(Error::status(418, "teapot"));
        }
        let body = format!("R:{relative}:{digest}").into_bytes();
        if relative.contains("nosize") {
            return Ok(Some(RenditionOutput { body: Body::Bytes(body), size: None }));
        }
        if relative.contains("streamy") {
            return Ok(Some(RenditionOutput { body: Body::Stream(bytes_stream(body)), size: None }));
        }
        if relative.contains("liar") {
            return Ok(Some(RenditionOutput { body: Body::Bytes(body), size: Some(3) }));
        }
        let length = body.len() as u64;
        Ok(Some(RenditionOutput { body: Body::Bytes(body), size: Some(length) }))
    }

    async fn image(&self, body: Body) -> mm_core::Result<AnalysisImage> {
        let bytes = body.bytes().await?;
        let digest = sha(&bytes);
        self.log.borrow_mut().push(arr!["image", obj! { "length" => bytes.len() as f64, "sha" => digest.as_str() }]);
        Ok(AnalysisImage { mime: "image/png".to_owned(), data: format!("I:{digest}").into_bytes() })
    }
}

#[derive(Clone)]
enum Content {
    File(String),
    Text(String),
}

#[derive(Clone)]
struct Stored {
    key: String,
    content: Content,
}

impl Stored {
    fn spec(&self) -> Value {
        match &self.content {
            Content::File(file) => obj! { "key" => self.key.as_str(), "file" => file.as_str() }.into(),
            Content::Text(text) => obj! { "key" => self.key.as_str(), "text" => text.as_str() }.into(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        match &self.content {
            Content::File(file) => std::fs::read(common::root().join(file)).expect("sampled media file"),
            Content::Text(text) => text.clone().into_bytes(),
        }
    }
}

#[derive(Clone)]
enum Input {
    File(String),
    Text(String),
    Key(String),
    Upload(String),
    Disk,
}

#[derive(Clone)]
enum UploadOp {
    Key,
    Input(String),
    Write(String, u64, String),
    Names,
    Size(String),
    Truncate(String, u64),
    Remove(String),
}

#[derive(Clone)]
enum Step {
    Response(String),
    Send { relative: String, method: &'static str, range: Option<String> },
    SendResult { result: MediaResult, method: &'static str, range: Option<String> },
    Prepare { input: Input, content_type: String, name: String, id: String },
    Image(Value),
    Cover(Value),
    Concurrent(Vec<String>),
    Upload(UploadOp),
}

fn user() -> MediaUser {
    MediaUser { account_uid: "acct1".to_owned(), id_prefix: "card_".to_owned() }
}

fn user_spec() -> Value {
    obj! { "accountUid" => "acct1", "idPrefix" => "card_" }.into()
}

impl Step {
    fn unordered(&self) -> bool {
        matches!(self, Self::Concurrent(_) | Self::Prepare { input: Input::Upload(_), .. })
    }

    fn spec(&self) -> Value {
        let mut step = match self {
            Self::Response(relative) => obj! { "op" => "response", "relative" => relative.as_str() },
            Self::Send { relative, method, range } => {
                obj! { "op" => "send", "relative" => relative.as_str(), "method" => *method, "range" => range.as_deref() }
            }
            Self::SendResult { result, method, range } => {
                obj! { "op" => "sendResult", "result" => result.to_value(), "method" => *method, "range" => range.as_deref() }
            }
            Self::Prepare { input, content_type, name, id } => {
                let input = match input {
                    Input::File(file) => obj! { "bytesFile" => file.as_str() },
                    Input::Text(text) => obj! { "bytesText" => text.as_str() },
                    Input::Key(key) => obj! { "key" => key.as_str() },
                    Input::Upload(upload) => obj! { "upload" => upload.as_str() },
                    Input::Disk => obj! { "file" => "/nowhere" },
                };
                obj! { "op" => "prepare", "input" => input, "type" => content_type.as_str(), "name" => name.as_str(), "user" => user_spec(), "id" => id.as_str() }
            }
            Self::Image(arg) => obj! { "op" => "image", "arg" => arg },
            Self::Cover(blob) => obj! { "op" => "cover", "blob" => blob },
            Self::Concurrent(relatives) => {
                obj! { "op" => "concurrent", "relatives" => Value::Array(relatives.iter().map(|relative| Value::from(relative.as_str())).collect()) }
            }
            Self::Upload(op) => match op {
                UploadOp::Key => obj! { "op" => "upload", "fn" => "key" },
                UploadOp::Input(id) => obj! { "op" => "upload", "fn" => "input", "id" => id.as_str() },
                UploadOp::Write(id, start, body) => {
                    obj! { "op" => "upload", "fn" => "write", "id" => id.as_str(), "start" => *start as f64, "text" => body.as_str() }
                }
                UploadOp::Names => obj! { "op" => "upload", "fn" => "names", "args" => arr![] },
                UploadOp::Size(id) => obj! { "op" => "upload", "fn" => "size", "args" => arr![id.as_str()] },
                UploadOp::Truncate(id, size) => obj! { "op" => "upload", "fn" => "truncate", "args" => arr![id.as_str(), *size as f64] },
                UploadOp::Remove(id) => obj! { "op" => "upload", "fn" => "remove", "args" => arr![id.as_str()] },
            },
        };
        if self.unordered() {
            step.insert("unordered", true);
        }
        step.into()
    }
}

struct Outcome {
    result: Result<Value, Error>,
    processor: Vec<Value>,
    bucket: Vec<Value>,
}

async fn sent(hosted: &HostedMedia, found: &MediaResult, method: &str, range: Option<&str>) -> Result<Value, Error> {
    let response = hosted.send(found, method, range).await?;
    let headers: Vec<Value> = response.headers.iter().map(|(name, value)| arr![name, value]).collect();
    let bytes = response.body.bytes().await?;
    Ok(obj! { "status" => response.status as f64, "headers" => Value::Array(headers), "length" => bytes.len() as f64, "sha" => sha(&bytes) }.into())
}

async fn execute(step: &Step, hosted: &HostedMedia, uploads: &BucketUploads) -> Result<Value, Error> {
    match step {
        Step::Response(relative) => Ok(hosted.response(relative).await?.map_or(Value::Null, |found| found.to_value())),
        Step::Send { relative, method, range } => match hosted.response(relative).await? {
            None => Ok(Value::Null),
            Some(found) => sent(hosted, &found, method, range.as_deref()).await,
        },
        Step::SendResult { result, method, range } => sent(hosted, result, method, range.as_deref()).await,
        Step::Prepare { input, content_type, name, id } => {
            let input = match input {
                Input::File(file) => MediaInput::Bytes(std::fs::read(common::root().join(file)).expect("sampled media file")),
                Input::Text(text) => MediaInput::Bytes(text.clone().into_bytes()),
                Input::Key(key) => MediaInput::Key(key.clone()),
                Input::Upload(upload) => MediaInput::Upload(upload.clone()),
                Input::Disk => MediaInput::File("/nowhere".to_owned()),
            };
            hosted.prepare(input, content_type, name, &user(), id).await.map(|prepared| prepared.to_value())
        }
        Step::Image(arg) => Ok(hosted.image(arg).await?.map_or(Value::Null, |image| {
            obj! { "mime" => image.mime, "data" => String::from_utf8(image.data).unwrap() }.into()
        })),
        Step::Cover(blob) => HostedMedia::cover(blob),
        Step::Concurrent(relatives) => {
            let answers = join_all(relatives.iter().map(|relative| hosted.response(relative))).await;
            let mut values = Vec::new();
            for answer in answers {
                values.push(answer?.map_or(Value::Null, |found| found.to_value()));
            }
            Ok(Value::Array(values))
        }
        Step::Upload(op) => match op {
            UploadOp::Key => Ok(uploads.key().into()),
            UploadOp::Input(id) => match uploads.input(id) {
                MediaInput::Upload(upload) => Ok(obj! { "upload" => upload }.into()),
                other => panic!("unexpected input {other:?}"),
            },
            UploadOp::Write(id, start, body) => {
                uploads.write(id, *start, Body::Bytes(body.clone().into_bytes()), Some(body.len() as u64)).await?;
                Ok(Value::Null)
            }
            UploadOp::Names => Ok(Value::Array(uploads.names().await?.into_iter().map(Value::from).collect())),
            UploadOp::Size(id) => Ok(uploads.size(id).await?.map_or(Value::Null, |size| Value::from(size as f64))),
            UploadOp::Truncate(id, size) => uploads.truncate(id, *size).await.map(|()| Value::Null),
            UploadOp::Remove(id) => uploads.remove(id).await.map(|()| Value::Null),
        },
    }
}

fn run_rust(prefix: &str, objects: &[Stored], steps: &[Step]) -> Vec<Outcome> {
    futures_executor::block_on(async {
        let bucket = Rc::new(MemoryBucket::default());
        for stored in objects {
            bucket.objects.borrow_mut().insert(normalize(&stored.key).unwrap(), stored.bytes());
        }
        let processor = Rc::new(FakeProcessor { log: Rc::default() });
        let hosted = HostedMedia::new(bucket.clone(), prefix, processor.clone());
        let uploads = BucketUploads::new(bucket.clone(), prefix);
        let mut outcomes = Vec::new();
        for step in steps {
            bucket.log.borrow_mut().clear();
            processor.log.borrow_mut().clear();
            let result = execute(step, &hosted, &uploads).await;
            outcomes.push(Outcome { result, processor: processor.log.take(), bucket: bucket.log.take() });
        }
        outcomes
    })
}

fn run_js(prefix: &str, objects: &[Stored], steps: &[Step]) -> common::Answer {
    // Preserve the historical frozen namespace without opening a driver.
    let driver = "driver:core:hosted-media";
    let spec = obj! {
        "prefix" => prefix,
        "objects" => Value::Array(objects.iter().map(Stored::spec).collect()),
        "steps" => Value::Array(steps.iter().map(Step::spec).collect()),
    };
    common::call(driver, "scenario", vec![spec.into()])
}

fn sorted(entries: &[Value]) -> Vec<String> {
    let mut texts: Vec<String> = entries.iter().map(text).collect();
    texts.sort();
    texts
}

/// Run `steps` on both sides and compare each step's result, processor calls and store operations.
fn compare(label: &str, prefix: &str, objects: &[Stored], steps: &[Step]) {
    let js = run_js(prefix, objects, steps);
    let rust = run_rust(prefix, objects, steps);
    assert_eq!(rust.len(), steps.len(), "{label}: Rust ran every step");
    let (mut errors, mut nulls) = (0, 0);
    let mut actual = Vec::new();
    for (step, outcome) in steps.iter().zip(&rust) {
        let result = match &outcome.result {
            Ok(value) => {
                nulls += usize::from(matches!(value, Value::Null));
                obj! { "ok" => value.clone() }
            }
            Err(error) => {
                errors += 1;
                obj! { "error" => common::error_class(error) }
            }
        };
        let processor = if step.unordered() { Value::Array(sorted(&outcome.processor).into_iter().map(Value::from).collect()) } else { Value::Array(outcome.processor.clone()) };
        let bucket = if step.unordered() { Value::Array(sorted(&outcome.bucket).into_iter().map(Value::from).collect()) } else { Value::Array(outcome.bucket.clone()) };
        actual.push(Value::from(obj! { "result" => result, "processor" => processor, "bucket" => bucket }));
    }
    let expected = js.oracle().map(|answer| {
        let outputs = answer.as_ref().expect("hosted scenario runs").as_array().expect("hosted outcomes");
        assert_eq!(outputs.len(), steps.len(), "{label}: JS ran every step");
        let mut expected = Vec::new();
        for (step, output) in steps.iter().zip(outputs) {
            let result = if let Value::Object(error) = output.get("error") {
                let thrown = JsError {
                    name: error.get("name").and_then(Value::as_str).unwrap_or("Error").to_owned(),
                    message: error.get("message").and_then(Value::as_str).unwrap_or("").to_owned(),
                    status: error.get("status").and_then(Value::as_f64).map(|status| status as u16),
                    code: None,
                };
                obj! { "error" => common::js_error_class(&thrown) }
            } else { obj! { "ok" => output.get("ok").clone() } };
            let processor = if step.unordered() {
                Value::Array(sorted(output.get("processor").as_array().expect("processor log")).into_iter().map(Value::from).collect())
            } else { output.get("processor").clone() };
            expected.push(Value::from(obj! { "result" => result, "processor" => processor, "bucket" => output.get("bucket").clone() }));
        }
        text(&Value::Array(expected)).into_bytes()
    });
    js.check(text(&Value::Array(actual)).as_bytes(), expected.as_deref());
    println!("{label}: {} steps compared ({errors} errors, {nulls} nulls)", steps.len());
}

fn walk(directory: &Path, base: &Path, out: &mut Vec<(String, u64)>) {
    for entry in std::fs::read_dir(directory).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, base, out);
        } else {
            let relative = path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
            out.push((relative, entry.metadata().unwrap().len()));
        }
    }
}

struct Library {
    objects: Vec<Stored>,
    found: Vec<String>,
    recorded: Vec<String>,
    synthetic: Vec<String>,
}

const SYNTHETIC: [(&str, &str); 14] = [
    ("synth/empty.bin", ""),
    ("synth/a b/c%20d.png", "png-bytes-1"),
    ("synth/noext", "n"),
    ("synth/UP.PNG", "u"),
    ("synth/x.png.tmp", "t"),
    ("synth/n/o.png", "normalized"),
    ("synth/d/pic.page-1.webp", "p1"),
    ("synth/d/pic.sub/deep.png", "deep"),
    ("synth/e/pic.page-1.webp", "p"),
    ("synth/e/pic.a/b.png", "nested"),
    ("synth/e/pic.png", "real"),
    ("synth/f/m.gif", "g"),
    ("synth/f/m.jpg", "j"),
    ("synth/tiny.txt", "abc"),
];

const SYNTHETIC_QUERIES: [&str; 33] = [
    "synth/empty.bin",
    "synth/a b/c%20d.png",
    "synth/noext",
    "synth/UP.PNG",
    "synth/up.png",
    "synth/x.png.tmp",
    "synth/x.png",
    "synth/n//o.png",
    "synth/n/o.png",
    "synth/d/pic",
    "synth/e/pic",
    "synth/f/m",
    "synth/tiny.txt",
    "synth/missing.png",
    "synth",
    "synth/",
    "",
    ";",
    "/abs",
    "../x",
    "a/../b",
    "synth/./tiny.txt",
    "synth/../synth/tiny.txt",
    "..",
    ".",
    " ",
    "synth//tiny.txt",
    "synth/tiny.txt;original.txt",
    "synth/tiny.txt;",
    "synth/tiny.txt;;",
    "synth/missing.png;10x10.webp",
    "../x;10x10.webp",
    "synth/tiny.txt;10x10.gif",
];

fn library(prefix: &str) -> Library {
    let base = common::root().join("capture/media");
    let mut files = Vec::new();
    walk(&base, &base, &mut files);
    files.sort();
    files.retain(|(relative, size)| {
        let document = relative.ends_with(".pdf") || relative.ends_with(".svg");
        (*size <= 150_000) || (document && *size <= 500_000)
    });
    let stride = (files.len() / 300).max(1);
    let mut objects: Vec<Stored> = Vec::new();
    let (mut found, mut recorded) = (Vec::new(), Vec::new());
    let mut seen = std::collections::HashSet::new();
    let mut picked = 0;
    for (index, (relative, _)) in files.iter().enumerate() {
        let document = relative.ends_with(".pdf") || relative.ends_with(".svg");
        if index % stride != 0 && !document {
            continue;
        }
        picked += 1;
        let (directory, name) = relative.rsplit_once('/').unwrap();
        let stem = name.split(';').next().unwrap();
        let extension = name.rsplit('.').next().unwrap();
        let file = format!("capture/media/{relative}");
        if picked % 11 == 0 {
            objects.push(Stored { key: format!("{prefix}media/{directory}/{name}"), content: Content::File(file) });
            recorded.push(format!("{directory}/{name}"));
            continue;
        }
        let key = format!("{prefix}media/{directory}/{stem}.{extension}");
        if seen.insert(key.clone()) {
            objects.push(Stored { key, content: Content::File(file) });
        }
        found.push(if picked % 5 == 0 { format!("{directory}/{stem}") } else { format!("{directory}/{stem}.{extension}") });
    }
    for (relative, body) in SYNTHETIC {
        objects.push(Stored { key: format!("{prefix}media/{relative}"), content: Content::Text(body.to_owned()) });
    }
    Library {
        objects,
        found,
        recorded,
        synthetic: SYNTHETIC_QUERIES.iter().map(|query| (*query).to_owned()).collect(),
    }
}

const RANGES: [Option<&str>; 22] = [
    None,
    Some("bytes=0-1023"),
    Some("bytes=1000-"),
    Some("bytes=-500"),
    Some("bytes=0-0"),
    Some("bytes=5-2"),
    Some("bytes=abc"),
    Some("bytes=-"),
    Some("bytes=-0"),
    Some("bytes=99999999-"),
    Some("bytes=0-99999999"),
    Some("bytes=1-2,4-5"),
    Some("items=0-1"),
    Some(""),
    Some("bytes=9007199254740993-"),
    Some("bytes=00-01"),
    Some(" bytes=0-1"),
    Some("bytes=2-"),
    Some("bytes=-1"),
    Some("bytes=-3"),
    Some("bytes=-4"),
    Some("bytes=3-"),
];

fn sends(relative: &str, ranges: &[Option<&str>], methods: &[&'static str]) -> Vec<Step> {
    let mut steps = Vec::new();
    for method in methods {
        for range in ranges {
            steps.push(Step::Send { relative: relative.to_owned(), method, range: range.map(str::to_owned) });
        }
    }
    steps
}

#[test]
fn objects_ranges_and_missing_keys() {
    let _goldens = common::test_guard();
    for prefix in ["", "libraries/owner/"] {
        let library = library(prefix);
        let mut steps = Vec::new();
        for (index, relative) in library.found.iter().enumerate() {
            steps.push(Step::Response(relative.clone()));
            if index % 3 == 0 {
                steps.extend(sends(relative, &RANGES[..17], &["GET"]));
                steps.extend(sends(relative, &[None, RANGES[1], RANGES[3], RANGES[9]], &["HEAD"]));
            } else {
                steps.extend(sends(relative, &RANGES[..4], &["GET"]));
                steps.extend(sends(relative, &RANGES[..1], &["HEAD"]));
            }
        }
        for relative in library.recorded.iter().chain(&library.synthetic) {
            steps.push(Step::Response(relative.clone()));
            steps.extend(sends(relative, &RANGES[..2], &["GET", "HEAD"]));
        }
        steps.extend(sends("synth/tiny.txt", &RANGES, &["GET", "HEAD"]));
        let tiny = format!("{prefix}media/synth/tiny.txt");
        for (key, size, content_type) in [
            (tiny.as_str(), 3, "text/plain"),
            (tiny.as_str(), 10, "text/plain"),
            (tiny.as_str(), 0, "text/plain"),
            (tiny.as_str(), 3, "a/b"),
            ("missing/key", 5, "image/png"),
            ("../../x", 5, "image/png"),
            ("/abs", 5, "image/png"),
        ] {
            let result = MediaResult { key: key.to_owned(), size, content_type: content_type.to_owned() };
            for method in ["GET", "HEAD"] {
                for range in &RANGES[..8] {
                    steps.push(Step::SendResult { result: result.clone(), method, range: range.map(str::to_owned) });
                }
            }
        }
        assert!(library.found.len() > 200, "sampled {} originals", library.found.len());
        assert!(!library.recorded.is_empty());
        compare(&format!("objects prefix {prefix:?}"), prefix, &library.objects, &steps);
    }
}

const ACCEPTED: [&str; 14] = [
    "800x450.webp",
    "800x1000-t.webp",
    "1x1-c.jpeg",
    "640x480.png",
    "800x600.jpg",
    "4096x4096.webp",
    "100x100/hevc.mp4",
    "300x300.mp4",
    "page(2)/300x300.webp",
    "800x600.webp;background(fff)",
    "original.pdf",
    "original.webp",
    "300x200-t.webp/hevc.mp4",
    "a/b/10x10.png",
];

const REJECTED: [&str; 24] = [
    "",
    "abc",
    "0x10.webp",
    "10x0.png",
    "4097x10.webp",
    "10x4097.webp",
    "10x10.gif",
    "10x10-x.webp",
    "page(0)/10x10.webp",
    "page(1001)/10x10.webp",
    "10x10.webp.bak",
    "original.",
    "ORIGINAL.png",
    "original.a.b",
    "10x10.webp ",
    "-10x10.webp",
    "10x10-t",
    "hevc.mp4",
    "99999999999999999999x10.webp",
    "10x10.webp;",
    "x10x10.webp",
    "10X10.webp",
    "100x100/hevc.mp4/hevc.mp4",
    "1000000x1/hevc.mp4",
];

#[test]
fn renditions_cache_and_processor_failures() {
    let _goldens = common::test_guard();
    let prefix = "libraries/owner/";
    let library = library(prefix);
    let mut steps = Vec::new();
    for relative in library.found.iter().step_by(6) {
        for suffix in ACCEPTED {
            let derived = format!("{relative};{suffix}");
            steps.push(Step::Response(derived.clone()));
            steps.push(Step::Send { relative: derived.clone(), method: "GET", range: None });
            steps.push(Step::Send { relative: derived, method: "GET", range: Some("bytes=0-9".to_owned()) });
        }
        for suffix in REJECTED {
            steps.push(Step::Response(format!("{relative};{suffix}")));
        }
    }
    for relative in library.recorded.iter().take(5) {
        steps.push(Step::Response(format!("{relative};800x450.webp")));
        steps.push(Step::Response(format!("{relative};bad")));
    }
    for relative in library.found.iter().take(6) {
        for script in ["null", "boom", "teapot", "nosize", "streamy", "liar"] {
            let derived = format!("{relative};{script}/10x10.webp");
            steps.push(Step::Response(derived.clone()));
            steps.push(Step::Response(derived.clone()));
            steps.push(Step::Send { relative: derived.clone(), method: "GET", range: None });
            steps.push(Step::Send { relative: derived, method: "HEAD", range: None });
        }
    }
    for relative in library.found.iter().skip(7).take(3) {
        let plain = relative.clone();
        let one = format!("{relative};5x5.webp");
        let two = format!("{relative};6x6.png");
        steps.push(Step::Concurrent(vec![one.clone(), one.clone(), two.clone(), one.clone(), plain.clone(), two.clone()]));
        steps.push(Step::Concurrent(vec![format!("{relative};boom/7x7.webp"); 3]));
        steps.push(Step::Concurrent(vec![format!("{relative};null/7x7.webp"), format!("{relative};null/7x7.webp"), plain]));
        steps.push(Step::Concurrent(vec![format!("{relative};8x8.webp;9x9.png"); 3]));
        steps.push(Step::Concurrent(vec![one, two]));
    }
    compare("renditions", prefix, &library.objects, &steps);
}

#[test]
fn prepare_image_cover_and_uploads() {
    let _goldens = common::test_guard();
    let prefix = "lib/x/";
    let mut library = library(prefix);
    for (relative, body) in [
        ("seed1/000000000000", "alpha "),
        ("seed1/000000000006", "beta "),
        ("seed1/000000000011", "gamma"),
        ("num/5", "e"),
        ("num/10", "j"),
        ("num/9", "i"),
        ("num/0x10", "h"),
        ("num/1e1", "t"),
        ("other/000000000000", "zz"),
    ] {
        library.objects.push(Stored { key: format!("{prefix}uploads/{relative}"), content: Content::Text(body.to_owned()) });
    }
    let found = library.found.clone();
    let existing = |index: usize| found[index].clone();
    let mut steps = Vec::new();

    let upload_id = |name: &str| format!("{prefix}uploads/{name}/");
    let prepare = |input: Input, content_type: &str, name: &str, id: &str| Step::Prepare {
        input,
        content_type: content_type.to_owned(),
        name: name.to_owned(),
        id: id.to_owned(),
    };
    let files: Vec<String> = library
        .objects
        .iter()
        .filter_map(|stored| match &stored.content {
            Content::File(file) => Some(file.clone()),
            Content::Text(_) => None,
        })
        .take(12)
        .collect();
    for (index, file) in files.into_iter().enumerate() {
        let content_type = ["image/png", "application/pdf", "video/mp4", "text/plain"][index % 4];
        steps.push(prepare(Input::File(file), content_type, &format!("sample{index}.bin"), &format!("card_{index:04}")));
    }
    steps.push(prepare(Input::Text("hello bytes".to_owned()), "image/png", "hello.png", "card_text1"));
    steps.push(prepare(Input::Text(String::new()), "application/pdf", "empty.pdf", "card_text2"));
    for name in ["fail-plain", "fail-415", "noblob", "nopath", "travpath", "escape"] {
        steps.push(prepare(Input::Text("payload".to_owned()), "image/png", name, "card_fail"));
    }
    steps.push(Step::Response("undefined".to_owned()));
    steps.push(Step::Response(format!("acct1/text1/{}.png", &sha(b"hello bytes")[..8])));
    steps.push(prepare(Input::Key(format!("{prefix}media/synth/tiny.txt")), "text/plain", "tiny.txt", "card_key1"));
    steps.push(prepare(Input::Key(format!("{prefix}media/synth/missing.txt")), "text/plain", "m.txt", "card_key2"));
    steps.push(prepare(Input::Key("../../x".to_owned()), "text/plain", "m.txt", "card_key3"));
    steps.push(prepare(Input::Key(String::new()), "text/plain", "m.txt", "card_key4"));
    steps.push(prepare(Input::Upload(String::new()), "text/plain", "m.txt", "card_up0"));
    steps.push(prepare(Input::Disk, "text/plain", "m.txt", "card_disk"));
    steps.push(prepare(Input::Key(format!("{prefix}media/synth/tiny.txt")), "video/mp4", "fail-plain", "card_key5"));
    for name in ["seed1", "num", "none", "other"] {
        steps.push(prepare(Input::Upload(upload_id(name)), "application/pdf", "upload.pdf", &format!("card_{name}")));
    }
    steps.push(prepare(Input::Upload(upload_id("seed1")), "image/png", "fail-415", "card_upfail"));
    steps.push(Step::Response(format!("acct1/seed1/{}.png", &sha(b"alpha beta gamma")[..8])));
    steps.push(Step::Response(format!("acct1/num/{}.pdf", &sha(b"eijth")[..8])));

    for index in [0, 5, 9, 14, 20, 33] {
        let relative = existing(index);
        steps.push(Step::Image(obj! { "path" => relative.as_str() }.into()));
        steps.push(Step::Image(obj! { "src" => format!("/media/{relative};800x450-t.webp") }.into()));
        steps.push(Step::Image(obj! { "src" => format!("{relative};100x100.png"), "path" => "ignored" }.into()));
        steps.push(Step::Image(obj! { "src" => Value::Null, "path" => relative.as_str() }.into()));
        steps.push(Step::Image(obj! { "src" => format!("/media/{relative};boom/9x9.webp") }.into()));
        steps.push(Step::Image(obj! { "src" => format!("/media/{relative};null/9x9.webp") }.into()));
        steps.push(Step::Image(obj! { "src" => format!("/media/{relative};9x9.gif") }.into()));
    }
    for arg in [
        Value::Null,
        Value::Bool(false),
        Value::Number(0.0),
        Value::from(""),
        obj! {}.into(),
        Value::from("abc"),
        obj! { "path" => "no/such.png" }.into(),
        obj! { "src" => "/media/no/such.png;1x1.png" }.into(),
        obj! { "src" => "" }.into(),
        obj! { "src" => "/media/" }.into(),
        obj! { "src" => 5.0 }.into(),
        obj! { "src" => true }.into(),
        obj! { "src" => arr![] }.into(),
        obj! { "src" => obj! {} }.into(),
        Value::Number(7.0),
        arr![1.0],
    ] {
        steps.push(Step::Image(arg));
    }

    for blob in [
        obj! { "width" => 1600.0, "height" => 900.0 }.into(),
        obj! { "width" => 100.0, "height" => 100.0 }.into(),
        obj! { "width" => 0.0, "height" => 0.0 }.into(),
        obj! {}.into(),
        obj! { "width" => "800", "height" => "1000" }.into(),
        obj! { "width" => 800.0, "height" => 4000.0 }.into(),
        Value::Null,
    ] {
        steps.push(Step::Cover(blob));
    }

    steps.push(Step::Upload(UploadOp::Key));
    steps.push(Step::Upload(UploadOp::Input("abc".to_owned())));
    steps.push(Step::Upload(UploadOp::Names));
    for id in ["seed1", "num", "other", "none", "new1", "../../x", "", "a/b"] {
        steps.push(Step::Upload(UploadOp::Size(id.to_owned())));
    }
    steps.push(Step::Upload(UploadOp::Write("new1".to_owned(), 0, "hello ".to_owned())));
    steps.push(Step::Upload(UploadOp::Write("new1".to_owned(), 6, "world".to_owned())));
    steps.push(Step::Upload(UploadOp::Write("new1".to_owned(), 6, "WORLD!".to_owned())));
    steps.push(Step::Upload(UploadOp::Write("new2".to_owned(), 1234, "x".to_owned())));
    steps.push(Step::Upload(UploadOp::Write("../../x".to_owned(), 0, "x".to_owned())));
    steps.push(Step::Upload(UploadOp::Size("new1".to_owned())));
    steps.push(Step::Upload(UploadOp::Names));
    steps.push(prepare(Input::Upload(upload_id("new1")), "image/png", "joined.png", "card_joined"));
    steps.push(Step::Response(format!("acct1/joined/{}.png", &sha(b"hello WORLD!")[..8])));
    steps.push(Step::Upload(UploadOp::Truncate("seed1".to_owned(), 6)));
    steps.push(Step::Upload(UploadOp::Size("seed1".to_owned())));
    steps.push(Step::Upload(UploadOp::Truncate("seed1".to_owned(), 0)));
    steps.push(Step::Upload(UploadOp::Size("seed1".to_owned())));
    steps.push(Step::Upload(UploadOp::Truncate("missing".to_owned(), 0)));
    steps.push(Step::Upload(UploadOp::Truncate("../../x".to_owned(), 0)));
    steps.push(Step::Upload(UploadOp::Remove("num".to_owned())));
    steps.push(Step::Upload(UploadOp::Remove("num".to_owned())));
    steps.push(Step::Upload(UploadOp::Remove("../../x".to_owned())));
    steps.push(Step::Upload(UploadOp::Names));
    steps.push(Step::Upload(UploadOp::Remove("other".to_owned())));
    steps.push(Step::Upload(UploadOp::Remove("new1".to_owned())));
    steps.push(Step::Upload(UploadOp::Remove("new2".to_owned())));
    steps.push(Step::Upload(UploadOp::Names));

    compare("prepare, image, cover, uploads", prefix, &library.objects, &steps);
}
