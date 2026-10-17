//! `ai` against `lib/ai.mjs`: the same scripted fake of the hosted APIs answers both, and the requests they send, the
//! sleeps they ask for and their results or errors must agree.

mod common;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, Once};
use std::task::Poll;

use async_trait::async_trait;
use common::{hosted_db, root, test_db, text};
use futures_executor::block_on;
use mm_core::ai::{Ai, AiError, AiImage, AiItem, EmbedInput, EmbedKind, VocabularyEntry, cosine, document_text};
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::sql::Sql;
use mm_core::{Error, Value, obj};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;

const RANDOM: [f64; 4] = [0.1, 0.35, 0.6, 0.85];
static LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static LOG: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
    static HANG: Cell<bool> = const { Cell::new(false) };
}

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        block_on(task);
    }

    fn sleep(&self, ms: u64) -> Task {
        if ms == 60_000 {
            return Box::pin(std::future::poll_fn(|_| if HANG.replace(false) { Poll::Ready(()) } else { Poll::Pending }));
        }
        LOG.with(|log| log.borrow_mut().push(obj! { "sleep" => ms as f64 }.into()));
        Box::pin(std::future::ready(()))
    }

    fn log(&self, _line: &str) {}
}

fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
}

struct Fake {
    script: Value,
    counters: RefCell<HashMap<&'static str, usize>>,
}

async fn yield_once() {
    let mut yielded = false;
    std::future::poll_fn(|cx| {
        if yielded {
            return Poll::Ready(());
        }
        yielded = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await;
}

fn hang_forever() -> Body {
    Body::Stream(Box::pin(futures_util::stream::poll_fn(|cx| {
        HANG.set(true);
        cx.waker().wake_by_ref();
        Poll::Pending
    })))
}

#[async_trait(?Send)]
impl Fetch for Fake {
    async fn fetch(&self, mut request: Request) -> mm_core::Result<Response> {
        yield_once().await;
        let url = request.url.as_str().to_owned();
        let route = [(":embedContent", "embedContent"), (":batchEmbedContents", "batchEmbedContents"), ("/chat/completions", "chat"), ("/v1/systemone", "systemone")]
            .iter()
            .find(|(suffix, _)| url.ends_with(suffix))
            .map_or("other", |(_, route)| route);
        let steps = match self.script.get(route) {
            Value::Array(steps) => steps.clone(),
            _ => Vec::new(),
        };
        let index = {
            let mut counters = self.counters.borrow_mut();
            let next = counters.get(route).map_or(0, |count| count + 1);
            counters.insert(route, next);
            next.min(steps.len().saturating_sub(1))
        };
        let step = steps.get(index).cloned().unwrap_or_else(|| obj! { "fail" => "ECONNREFUSED" }.into());
        let body = String::from_utf8(request.take_body().bytes().await?).expect("utf-8 request body");
        let headers: Vec<Value> = request.headers.iter().map(|(name, value)| Value::Array(vec![name.into(), value.into()])).collect();
        let entry: Value = obj! {
            "route" => route,
            "url" => url,
            "method" => request.method.as_str(),
            "headers" => headers,
            "body" => normalize_uuid(&body),
            "timeoutMs" => 60_000.0,
        }
        .into();
        LOG.with(|log| log.borrow_mut().push(entry));
        if step.get("hang").truthy() {
            HANG.set(true);
            std::future::pending::<()>().await;
        }
        if let Some(fail) = step.get("fail").as_str() {
            return Err(Error::internal(fail));
        }
        let mut headers = Headers::new();
        if let Value::Object(object) = step.get("headers") {
            for (name, value) in object.iter() {
                headers.set(name.into_owned(), value.to_js_string());
            }
        }
        let body = if step.get("hangBody").truthy() { hang_forever() } else { Body::Bytes(step.get("body").as_str().unwrap_or("").as_bytes().to_vec()) };
        Ok(Response::new(step.get("status").as_f64().expect("status") as u16, headers, body))
    }
}

fn normalize_uuid(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("mymind-image-") {
        out.push_str(&rest[..at]);
        out.push_str("mymind-image-UUID");
        rest = &rest[at + "mymind-image-".len() + 36..];
    }
    out.push_str(rest);
    out
}

fn j(source: &str) -> Value {
    json::parse(source).expect("json literal")
}

fn string_of(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

fn item_of(value: &Value) -> AiItem {
    AiItem {
        title: string_of(value.get("title")),
        kind: string_of(value.get("kind")),
        text: string_of(value.get("text")),
        url: string_of(value.get("url")),
        domain: string_of(value.get("domain")),
        image: match value.get("image") {
            Value::Object(image) => Some(AiImage {
                mime: image.get("mime").and_then(Value::as_str).unwrap_or("").to_owned(),
                data: base64::decode(image.get("b64").and_then(Value::as_str).unwrap_or("")),
            }),
            _ => None,
        },
    }
}

fn vector(values: &[f32]) -> Value {
    Value::Array(values.iter().map(|value| Value::Number(f64::from(*value))).collect())
}

fn kind_of(value: &Value) -> EmbedKind {
    if value.as_str() == Some("query") { EmbedKind::Query } else { EmbedKind::Document }
}

fn vocabulary_of(value: &Value) -> Vec<VocabularyEntry> {
    let Value::Array(entries) = value else { return Vec::new() };
    entries
        .iter()
        .map(|entry| match entry {
            Value::String(name) => VocabularyEntry::from(name.as_str()),
            other => VocabularyEntry {
                name: other.get("name").as_str().unwrap_or("").to_owned(),
                embedding: match other.get("embedding") {
                    Value::Array(numbers) => Some(numbers.iter().map(|number| number.as_f64().unwrap()).collect()),
                    _ => None,
                },
            },
        })
        .collect()
}

fn vocab_rows(tags: &[mm_core::ai::VocabularyTag]) -> Value {
    Value::Array(tags.iter().map(|tag| obj! { "name" => tag.name.as_str(), "embedding" => vector(&tag.embedding) }.into()).collect())
}

fn tag_rows(tags: &[mm_core::ai::AnalysisTag]) -> Value {
    Value::Array(
        tags.iter()
            .map(|tag| {
                let source = if tag.source == mm_core::ai::TagSource::Vocabulary { "vocabulary" } else { "chat" };
                obj! { "name" => tag.name.as_str(), "p" => tag.p, "source" => source }.into()
            })
            .collect(),
    )
}

async fn call(ai: &Ai<'_>, name: &str, args: &Value) -> Result<Value, AiError> {
    Ok(match name {
        "embed" => vector(&ai.embed(&item_of(args.get("item")), kind_of(args.get("kind"))).await?),
        "embedTexts" => {
            let Value::Array(texts) = args.get("texts") else { panic!("texts") };
            let items: Vec<AiItem> = texts.iter().map(item_of).collect();
            let inputs: Vec<EmbedInput<'_>> =
                texts.iter().zip(&items).map(|(text, item)| text.as_str().map_or(EmbedInput::Item(item), EmbedInput::Text)).collect();
            let mut usage = Vec::new();
            let mut on_usage = |value: &Value| usage.push(value.clone());
            let vectors = ai.embed_texts(&inputs, kind_of(args.get("kind")), Some(&mut on_usage)).await?;
            obj! { "vectors" => vectors.iter().map(|v| vector(v)).collect::<Vec<_>>(), "usage" => usage }.into()
        }
        "describe" => {
            let description = ai.describe(&item_of(args.get("item"))).await?;
            obj! { "title" => description.title, "summary" => description.summary, "tags" => description.tags.iter().map(|tag| Value::from(tag.as_str())).collect::<Vec<_>>() }.into()
        }
        "pickTags" => {
            let Value::Array(names) = args.get("candidates") else { panic!("candidates") };
            let names: Vec<String> = names.iter().map(|name| name.as_str().unwrap().to_owned()).collect();
            let picked = ai.pick_tags(&item_of(args.get("item")), &names).await?;
            Value::Array(picked.iter().map(|tag| obj! { "name" => tag.name.as_str(), "p" => tag.p }.into()).collect())
        }
        "embedVocabulary" => vocab_rows(&ai.embed_vocabulary(&vocabulary_of(args.get("vocabulary"))).await?),
        "embedVocabularyTwice" => {
            let first = vocab_rows(&ai.embed_vocabulary(&vocabulary_of(args.get("first"))).await?);
            let second = vocab_rows(&ai.embed_vocabulary(&vocabulary_of(args.get("second"))).await?);
            Value::Array(vec![first, second])
        }
        "analyze" => {
            let analysis = ai.analyze(&item_of(args.get("item")), &vocabulary_of(args.get("vocabulary"))).await?;
            obj! { "title" => analysis.title, "summary" => analysis.summary, "tags" => tag_rows(&analysis.tags), "embedding" => vector(&analysis.embedding) }.into()
        }
        other => panic!("unknown fn {other}"),
    })
}

fn error_view(error: &AiError) -> Value {
    let service = error.service.map(|service| service.as_str());
    obj! {
        "name" => if error.is_type_error() { "TypeError" } else { "AiError" },
        "message" => error.message.as_str(),
        "service" => service,
        "status" => error.status.map(f64::from),
    }
    .into()
}

fn run(case: &Value) -> (Result<Value, Value>, Vec<Value>) {
    let mut vars = HashMap::new();
    if let Value::Object(env) = case.get("env") {
        for (name, value) in env.iter() {
            vars.insert(name.into_owned(), value.to_js_string());
        }
    }
    runtime::set_env(vars);
    mm_core::ai::forget_vocabulary();
    LOG.with(|log| log.borrow_mut().clear());
    HANG.set(false);
    let fake = Fake { script: case.get("script").clone(), counters: RefCell::default() };
    let draws = Cell::new(0);
    let random = || {
        draws.set(draws.get() + 1);
        RANDOM[(draws.get() - 1) % RANDOM.len()]
    };
    let ai = Ai::new(&fake).with_random(&random);
    let name = case.get("fn").as_str().unwrap();
    let result = match name {
        "documentText" => Ok(Value::from(document_text(&item_of(case.get("args").get("item"))))),
        "cosine" => {
            let numbers = |value: &Value| -> Vec<f64> {
                let Value::Array(items) = value else { panic!("array") };
                items.iter().map(|item| item.as_f64().unwrap()).collect()
            };
            Ok(Value::Number(cosine(&numbers(case.get("args").get("a")), &numbers(case.get("args").get("b")))))
        }
        _ => block_on(call(&ai, name, case.get("args"))),
    };
    (result.map_err(|error| error_view(&error)), LOG.with(|log| log.take()))
}

fn grouped(log: &[Value]) -> String {
    let mut routes: Vec<String> = Vec::new();
    let mut sleeps: Vec<f64> = Vec::new();
    for entry in log {
        match entry.get("sleep").as_f64() {
            Some(ms) => sleeps.push(ms),
            None => routes.push(format!("{}\u{1}{}", entry.get("route").to_js_string(), text(entry))),
        }
    }
    routes.sort_by(|a, b| a.split('\u{1}').next().cmp(&b.split('\u{1}').next()));
    sleeps.sort_by(f64::total_cmp);
    format!("{routes:?}{sleeps:?}")
}

fn compare(cases: Vec<Value>) {
    let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    install();
    let request = Value::from(obj! { "cases" => cases.clone() });
    let mut child = Command::new("node")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/drivers/ai.mjs"))
        .env("MYMIND_ROOT", root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node runs");
    let mut stdin = child.stdin.take().unwrap();
    let payload = json::to_string(&request);
    let writer = std::thread::spawn(move || stdin.write_all(payload.as_bytes()).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success(), "driver failed");
    let Value::Array(answers) = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap() else { panic!("no array") };
    assert_eq!(answers.len(), cases.len());
    for (index, (case, js)) in cases.iter().zip(&answers).enumerate() {
        let name = case.get("name").as_str().unwrap_or("?");
        let context = format!("case {index} {name} ({})", case.get("fn").to_js_string());
        let (rust, rust_log) = run(case);
        let js_log = match js.get("log") {
            Value::Array(log) => log.clone(),
            _ => Vec::new(),
        };
        match (&rust, js.get("error")) {
            (Ok(rust), Value::Undefined) => assert_eq!(text(rust), text(js.get("result")), "{context}"),
            (Err(rust), error @ Value::Object(_)) => assert_eq!(text(rust), text(error), "{context}"),
            (rust, js) => panic!("{context}: Rust {rust:?}, JS {js:?}"),
        }
        let concurrent = case.get("fn").as_str() == Some("analyze");
        if concurrent && rust.is_err() {
            continue;
        }
        if concurrent {
            assert_eq!(grouped(&rust_log), grouped(&js_log), "{context}: requests");
        } else {
            assert_eq!(text(&Value::Array(rust_log)), text(&Value::Array(js_log)), "{context}: requests");
        }
    }
    eprintln!("{} cases compared", cases.len());
}

fn case(name: &str, function: &str, args: Value, script: Value, env: Value) -> Value {
    obj! { "name" => name, "fn" => function, "args" => args, "script" => script, "env" => env }.into()
}

fn keys() -> Value {
    j(r#"{"GEMINI_API_KEY":"fake-gemini-key-123","TYPESAFE_API_KEY":"fake-typesafe-key-456"}"#)
}

fn gemini_only() -> Value {
    j(r#"{"GEMINI_API_KEY":"fake-gemini-key-123"}"#)
}

fn emb_values(seed: usize) -> Vec<Value> {
    (0..768).map(|i| Value::Number(((i * 7 + seed * 3) % 13) as f64 / 5.0 - 1.1)).collect()
}

fn single(seed: usize) -> String {
    json::to_string(&obj! { "embedding" => obj! { "values" => emb_values(seed) } }.into())
}

fn batch(count: usize) -> String {
    let entries: Vec<Value> = (0..count).map(|seed| obj! { "values" => emb_values(seed) }.into()).collect();
    json::to_string(&obj! { "embeddings" => entries, "usageMetadata" => obj! { "totalTokenCount" => 12.0 } }.into())
}

fn step(status: u16, body: &str) -> Value {
    obj! { "status" => f64::from(status), "body" => body }.into()
}

fn step_with(status: u16, body: &str, headers: &str) -> Value {
    obj! { "status" => f64::from(status), "body" => body, "headers" => j(headers) }.into()
}

fn script(route: &str, steps: Vec<Value>) -> Value {
    obj! { route => steps }.into()
}

fn scripts(routes: Vec<(&str, Vec<Value>)>) -> Value {
    Value::Object(routes.into_iter().map(|(route, steps)| (route.to_owned(), Value::Array(steps))).collect())
}

fn fail(name: &str) -> Value {
    obj! { "fail" => name }.into()
}

fn hang() -> Value {
    obj! { "hang" => true }.into()
}

fn hang_body(status: u16) -> Value {
    obj! { "status" => f64::from(status), "hangBody" => true }.into()
}

fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = (payload.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(kind);
    out.extend_from_slice(payload);
    out.extend_from_slice(&[0; 4]);
    out
}

fn png(width: u32, height: u32, animated: bool, total: usize) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut header = width.to_be_bytes().to_vec();
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    out.extend(chunk(b"IHDR", &header));
    if animated {
        out.extend(chunk(b"acTL", &[0, 0, 0, 2, 0, 0, 0, 0]));
    }
    out.extend(chunk(b"IDAT", &[1, 2, 3, 4]));
    out.resize(out.len().max(total), 0);
    out
}

fn jpeg(width: u16, height: u16, total: usize) -> Vec<u8> {
    let mut out = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x4a, 0x46];
    out.extend_from_slice(&[0xff, 0xc0, 0x00, 0x11, 0x08]);
    out.extend_from_slice(&height.to_be_bytes());
    out.extend_from_slice(&width.to_be_bytes());
    out.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
    out.resize(out.len().max(total), 0);
    out
}

fn webp() -> Vec<u8> {
    let mut out = b"RIFF\x20\0\0\0WEBPVP8 ".to_vec();
    out.resize(40, 0);
    out
}

fn image(mime: &str, data: &[u8]) -> Value {
    obj! { "mime" => mime, "b64" => base64::encode(data) }.into()
}

fn with_image(mut base: Value, mime: &str, data: &[u8]) -> Value {
    if let Value::Object(object) = &mut base {
        object.insert("image", image(mime, data));
    }
    base
}

fn item(title: &str, text: &str) -> Value {
    obj! { "title" => title, "text" => text, "kind" => "Article", "url" => "https://example.com/a?b=1", "domain" => "example.com" }.into()
}

fn images() -> Vec<(&'static str, &'static str, Vec<u8>)> {
    vec![
        ("png-300", "image/png", png(300, 300, false, 0)),
        ("png-animated", "image/png", png(300, 300, true, 0)),
        ("png-small", "image/png", png(100, 100, false, 0)),
        ("png-wide", "image/png", png(1600, 400, false, 0)),
        ("png-huge", "image/png", png(1568, 1568, false, 0)),
        ("png-edge", "image/png", png(1568, 1568 - 1000, false, 0)),
        ("png-short", "image/png", vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0]),
        ("jpeg-640", "image/jpeg", jpeg(640, 480, 0)),
        ("jpeg-small", "image/jpeg", jpeg(200, 200, 0)),
        ("jpeg-noframe", "image/jpeg", vec![0xff, 0xd8, 0xff, 0xda, 0, 2, 0, 0, 0, 0]),
        ("webp", "image/webp", webp()),
        ("gif", "image/gif", b"GIF89a\x01\0\x01\0\0\0\0;;;;;;;;;;;;;;;;;;;;;;;;;;".to_vec()),
        ("garbage", "image/png", vec![1, 2, 3]),
        ("empty", "image/png", vec![]),
    ]
}

#[test]
fn embed_matches_node() {
    let doc = item("A title", "Some body text");
    let ok = || script("embedContent", vec![step(200, &single(1))]);
    let mut cases = vec![
        case("document", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), ok(), keys()),
        case("query", "embed", obj! { "item" => item("", "find cats"), "kind" => "query" }.into(), ok(), keys()),
        case("query-title", "embed", obj! { "item" => item("only title", ""), "kind" => "query" }.into(), ok(), keys()),
        case("query-empty", "embed", obj! { "item" => item("", "  \n "), "kind" => "query" }.into(), ok(), keys()),
        case("document-empty", "embed", obj! { "item" => item("", ""), "kind" => "document" }.into(), ok(), keys()),
        case("no-key", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), ok(), j("{}")),
        case("empty-key", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), ok(), j(r#"{"GEMINI_API_KEY":""}"#)),
        case("empty-query-no-key", "embed", obj! { "item" => item("", ""), "kind" => "query" }.into(), ok(), j("{}")),
        case("title-only", "embed", obj! { "item" => item("t", ""), "kind" => "document" }.into(), ok(), keys()),
        case("text-only", "embed", obj! { "item" => item("", "t"), "kind" => "document" }.into(), ok(), keys()),
        case("base-url", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), ok(), j(r#"{"GEMINI_API_KEY":"fake-gemini-key-123","GEMINI_BASE_URL":"https://gemini.example.test/v9//"}"#)),
        case("zeros", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, &json::to_string(&obj! { "embedding" => obj! { "values" => vec![Value::Number(0.0); 768] } }.into()))]), keys()),
        case("strings", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, &json::to_string(&obj! { "embedding" => obj! { "values" => (0..768).map(|i| Value::from(format!("{}.5", i % 9))).collect::<Vec<_>>() } }.into()))]), keys()),
        case("infinity", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, &json::to_string(&obj! { "embedding" => obj! { "values" => (0..768).map(|i| if i == 5 { Value::from("1e999") } else { Value::Number(1.0) }).collect::<Vec<_>>() } }.into()))]), keys()),
        case("nulls-and-bools", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, &json::to_string(&obj! { "embedding" => obj! { "values" => (0..768).map(|i| match i % 3 { 0 => Value::Null, 1 => Value::Bool(true), _ => Value::Number(2.0) }).collect::<Vec<_>>() } }.into()))]), keys()),
        case("huge-values", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, &json::to_string(&obj! { "embedding" => obj! { "values" => (0..768).map(|i| Value::Number(if i == 0 { 3e38 } else { 1e-30 })).collect::<Vec<_>>() } }.into()))]), keys()),
        case("short-vector", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, r#"{"embedding":{"values":[1,2,3]}}"#)]), keys()),
        case("no-values", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, r#"{"embedding":{}}"#)]), keys()),
        case("null-body", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, "null")]), keys()),
        case("not-json", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, "<html>")]), keys()),
        case("empty-body", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![step(200, "")]), keys()),
        case("hang-body-ok", "embed", obj! { "item" => doc.clone(), "kind" => "document" }.into(), script("embedContent", vec![hang_body(200)]), keys()),
        case("long-text", "embed", obj! { "item" => item(&"T".repeat(900), &format!("{}{}", "é".repeat(7000), "end")), "kind" => "document" }.into(), ok(), keys()),
        case("emoji-text", "embed", obj! { "item" => item("😀 title \u{a0}\u{2003}x", &"😀".repeat(3001)), "kind" => "document" }.into(), ok(), keys()),
        case("exact-limits", "embed", obj! { "item" => item(&"a".repeat(500), &"b".repeat(6000)), "kind" => "document" }.into(), ok(), keys()),
        case("over-limits", "embed", obj! { "item" => item(&"a".repeat(501), &"b".repeat(6001)), "kind" => "document" }.into(), ok(), keys()),
        case("whitespace", "embed", obj! { "item" => item("  a \n\t b\u{feff}c\u{85}d ", "x\u{2028}y\u{2029}z\u{a0}w"), "kind" => "document" }.into(), ok(), keys()),
        case("long-query", "embed", obj! { "item" => item("", &"q ".repeat(5000)), "kind" => "query" }.into(), ok(), keys()),
    ];
    for (name, mime, data) in images() {
        cases.push(case(&format!("image-{name}"), "embed", obj! { "item" => with_image(doc.clone(), mime, &data), "kind" => "document" }.into(), ok(), keys()));
        cases.push(case(&format!("image-only-{name}"), "embed", obj! { "item" => with_image(item("", ""), mime, &data), "kind" => "document" }.into(), ok(), keys()));
        cases.push(case(&format!("image-query-{name}"), "embed", obj! { "item" => with_image(item("", "q"), mime, &data), "kind" => "query" }.into(), ok(), keys()));
    }
    let retry = |first: Value| script("embedContent", vec![first, step(200, &single(2))]);
    let doc_args = || -> Value { obj! { "item" => doc.clone(), "kind" => "document" }.into() };
    cases.push(case("429-plain", "embed", doc_args(), retry(step(429, "slow down")), keys()));
    for (index, value) in ["2", "0", "0.5", "45", "-1", "abc", " 3 ", "", "1e1", "0x10", "Infinity", "31"].iter().enumerate() {
        let headers: Value = obj! { "Retry-After" => *value }.into();
        cases.push(case(&format!("429-retry-after-{index}"), "embed", doc_args(), retry(obj! { "status" => 429.0, "body" => "", "headers" => headers }.into()), keys()));
    }
    cases.push(case("503-retry-after", "embed", doc_args(), retry(step_with(503, "", r#"{"retry-after":"4"}"#)), keys()));
    cases.push(case("500-then-ok", "embed", doc_args(), retry(step(500, "oops")), keys()));
    cases.push(case("400-no-retry", "embed", doc_args(), retry(step(400, r#"{"error":{"message":"bad request"}}"#)), keys()));
    cases.push(case("500-forever", "embed", doc_args(), script("embedContent", vec![step(500, r#"{"error":{"message":"boom"}}"#)]), keys()));
    cases.push(case("429-forever", "embed", doc_args(), script("embedContent", vec![step_with(429, r#"{"message":"quota"}"#, r#"{"retry-after":"1"}"#)]), keys()));
    cases.push(case("net-then-ok", "embed", doc_args(), script("embedContent", vec![fail("ECONNRESET"), fail("ETIMEDOUT"), step(200, &single(3))]), keys()));
    cases.push(case("net-forever", "embed", doc_args(), script("embedContent", vec![fail("ECONNREFUSED")]), keys()));
    cases.push(case("net-error-name", "embed", doc_args(), script("embedContent", vec![fail("TypeError")]), keys()));
    cases.push(case("timeout-then-ok", "embed", doc_args(), script("embedContent", vec![hang(), step(200, &single(4))]), keys()));
    cases.push(case("timeout-forever", "embed", doc_args(), script("embedContent", vec![hang()]), keys()));
    cases.push(case("timeout-body-503", "embed", doc_args(), script("embedContent", vec![hang_body(503), step(200, &single(5))]), keys()));
    cases.push(case("timeout-body-400", "embed", doc_args(), script("embedContent", vec![hang_body(400)]), keys()));
    cases.push(case("timeout-body-503-forever", "embed", doc_args(), script("embedContent", vec![hang_body(503)]), keys()));
    cases.push(case("timeout-body-404", "embed", doc_args(), script("embedContent", vec![hang_body(404)]), keys()));
    for (status, body) in [
        (400, r#"[{"error":{"message":"array bad"}}]"#),
        (400, "[]"),
        (400, "{}"),
        (400, r#"{"error":"plain error"}"#),
        (400, r#"{"error":{"code":1}}"#),
        (400, r#"{"error":{"message":null},"message":"fallback"}"#),
        (400, r#"{"detail":"d"}"#),
        (400, r#"{"detail":{"a":1}}"#),
        (400, "null"),
        (400, "123"),
        (400, "true"),
        (400, r#""just a string""#),
        (400, "plain text   with\n whitespace"),
        (400, "key fake-gemini-key-123 leaked fake-typesafe-key-456"),
        (400, r#"{"error":{"message":"bad key fake-gemini-key-123"}}"#),
        (400, &"x".repeat(400)),
        (400, &"é".repeat(400)),
        (401, ""),
        (403, "forbidden"),
        (404, ""),
        (302, ""),
        (418, ""),
        (422, "   "),
        (451, ""),
        (599, ""),
        (300, ""),
    ] {
        cases.push(case(&format!("error-{status}-{}", &body[..body.len().min(12)]), "embed", doc_args(), script("embedContent", vec![step(status, body)]), keys()));
    }
    compare(cases);
}

#[test]
fn embed_texts_matches_node() {
    let ok = |count: usize| script("batchEmbedContents", vec![step(200, &batch(count))]);
    let texts = |names: &[&str]| -> Value { Value::Array(names.iter().map(|name| Value::from(*name)).collect()) };
    let many = |count: usize| -> Value { Value::Array((0..count).map(|i| Value::from(format!("tag {i}"))).collect()) };
    let big = |count: usize, first: usize| {
        let entries: Vec<Value> = (0..count).map(|seed| obj! { "values" => emb_values(seed) }.into()).collect();
        let _ = first;
        json::to_string(&obj! { "embeddings" => entries }.into())
    };
    let cases = vec![
        case("two-query", "embedTexts", obj! { "texts" => texts(&["alpha", "beta"]), "kind" => "query" }.into(), ok(2), keys()),
        case("two-document", "embedTexts", obj! { "texts" => texts(&["alpha", "beta"]), "kind" => "document" }.into(), ok(2), keys()),
        case("items-document", "embedTexts", obj! { "texts" => vec![item("T", "body"), item("", "only")], "kind" => "document" }.into(), ok(2), keys()),
        case("items-query", "embedTexts", obj! { "texts" => vec![item("T", "body")], "kind" => "query" }.into(), ok(1), keys()),
        case("empty-list", "embedTexts", obj! { "texts" => texts(&[]), "kind" => "query" }.into(), ok(0), j("{}")),
        case("no-key", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), ok(1), j("{}")),
        case("empty-text", "embedTexts", obj! { "texts" => texts(&["a", "  "]), "kind" => "query" }.into(), ok(2), keys()),
        case("empty-doc", "embedTexts", obj! { "texts" => texts(&["a", ""]), "kind" => "document" }.into(), ok(2), keys()),
        case("empty-item-doc", "embedTexts", obj! { "texts" => vec![item("", "")], "kind" => "document" }.into(), ok(1), keys()),
        case("100", "embedTexts", obj! { "texts" => many(100), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &big(100, 0))]), keys()),
        case("101", "embedTexts", obj! { "texts" => many(101), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &big(100, 0)), step(200, &big(1, 0))]), keys()),
        case("250", "embedTexts", obj! { "texts" => many(250), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &big(100, 0)), step(200, &big(100, 0)), step(200, &big(50, 0))]), keys()),
        case("second-batch-fails", "embedTexts", obj! { "texts" => many(150), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &big(100, 0)), step(400, "no")]), keys()),
        case("count-mismatch", "embedTexts", obj! { "texts" => texts(&["a", "b"]), "kind" => "query" }.into(), ok(1), keys()),
        case("embeddings-string", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":"abc"}"#)]), keys()),
        case("embeddings-object-length", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":{"length":7}}"#)]), keys()),
        case("embeddings-object", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":{}}"#)]), keys()),
        case("embeddings-number", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":5}"#)]), keys()),
        case("embeddings-missing", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, "{}")]), keys()),
        case("bad-entry", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":[{"values":[1]}]}"#)]), keys()),
        case("null-entry", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, r#"{"embeddings":[null]}"#)]), keys()),
        case("usage-snake", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &format!(r#"{{"embeddings":[{{"values":{}}}],"usage_metadata":{{"n":1}}}}"#, json::to_string(&Value::Array(emb_values(1)))))]), keys()),
        case("usage-none", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(200, &format!(r#"{{"embeddings":[{{"values":{}}}]}}"#, json::to_string(&Value::Array(emb_values(1)))))]), keys()),
        case("retry", "embedTexts", obj! { "texts" => texts(&["a"]), "kind" => "query" }.into(), script("batchEmbedContents", vec![step(503, ""), fail("ECONNRESET"), step(200, &batch(1))]), keys()),
        case("unicode", "embedTexts", obj! { "texts" => texts(&["日本語 tag", "emoji 😀 ", "Ünï\u{a0}code"]), "kind" => "query" }.into(), ok(3), keys()),
        case("long", "embedTexts", obj! { "texts" => texts(&[&"w ".repeat(4000), &"é".repeat(7000)]), "kind" => "document" }.into(), ok(2), keys()),
    ];
    compare(cases);
}

fn chat_body(content: &str) -> String {
    json::to_string(&obj! { "choices" => vec![Value::from(obj! { "message" => obj! { "content" => content } })] }.into())
}

fn good_chat() -> String {
    chat_body(r##"{"title":"One two three four five six seven eight nine ten","summary":"  A short   summary. ","tags":["#Cats","  Big  Dogs ","a b c d e","toolongtagtoolongtagtoolongtagtoolongtagtoolong","cats","",7,null,"Mixed CASE"]}"##)
}

#[test]
fn describe_matches_node() {
    let doc = item("A title", "Some body text");
    let args = |item: Value| -> Value { obj! { "item" => item }.into() };
    let ok = || script("chat", vec![step(200, &good_chat())]);
    let chat_env = j(r#"{"GEMINI_API_KEY":"fake-gemini-key-123","AI_CHAT_API_KEY":"fake-chat-key-789","AI_CHAT_MODEL":"my-model","AI_CHAT_BASE_URL":"https://chat.example.test/v1/"}"#);
    let mut cases = vec![
        case("basic", "describe", args(doc.clone()), ok(), gemini_only()),
        case("chat-key", "describe", args(doc.clone()), ok(), chat_env.clone()),
        case("chat-key-only", "describe", args(doc.clone()), ok(), j(r#"{"AI_CHAT_API_KEY":"k"}"#)),
        case("no-key", "describe", args(doc.clone()), ok(), j("{}")),
        case("empty-fields", "describe", args(obj! {}.into()), ok(), gemini_only()),
        case("null-fields", "describe", args(j(r#"{"title":null,"kind":null}"#)), ok(), gemini_only()),
        case("fenced", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("```json\n{\"title\":\"T\",\"summary\":\"S\",\"tags\":[\"x\"]}\n```"))]), gemini_only()),
        case("fenced-bare", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("  ```{\"title\":\"T\",\"summary\":\"S\",\"tags\":[\"x\"]}```  "))]), gemini_only()),
        case("fenced-unicode-space", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("\u{feff}\u{a0}```json\u{3000}{\"title\":\"T\",\"summary\":\"S\",\"tags\":[]}\u{2003}```\u{a0}"))]), gemini_only()),
        case("fenced-nel", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("\u{85}```json{\"title\":\"T\",\"summary\":\"S\",\"tags\":[]}```"))]), gemini_only()),
        case("fence-upper", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("```JSON{\"title\":\"T\",\"summary\":\"S\",\"tags\":[]}```"))]), gemini_only()),
        case("invalid-twice", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("not json"))]), gemini_only()),
        case("invalid-then-ok", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("not json")), step(200, &good_chat())]), gemini_only()),
        case("falsy-then-ok", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("null")), step(200, &chat_body("0")), step(200, &chat_body("\"\"")), step(200, &chat_body("false")), step(200, &good_chat())]), gemini_only()),
        case("falsy-then-invalid", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("null")), step(200, &chat_body("{"))]), gemini_only()),
        case("number-content", "describe", args(doc.clone()), script("chat", vec![step(200, r#"{"choices":[{"message":{"content":123}}]}"#)]), gemini_only()),
        case("array-content", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("[]"))]), gemini_only()),
        case("string-content", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body("\"text\""))]), gemini_only()),
        case("no-choices", "describe", args(doc.clone()), script("chat", vec![step(200, "{}")]), gemini_only()),
        case("null-body", "describe", args(doc.clone()), script("chat", vec![step(200, "null")]), gemini_only()),
        case("no-summary", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":"T","tags":[]}"#))]), gemini_only()),
        case("blank-summary", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":"T","summary":"  \n ","tags":[]}"#))]), gemini_only()),
        case("summary-number", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":5,"summary":42,"tags":"x"}"#))]), gemini_only()),
        case("summary-object", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":{"a":1},"summary":["x","y"],"tags":{"0":"a"}}"#))]), gemini_only()),
        case("tags-missing", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":"T","summary":"S"}"#))]), gemini_only()),
        case("tags-many", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(&format!(r#"{{"title":"T","summary":"S","tags":[{}]}}"#, (0..30).map(|i| format!("\"tag number {i}\"")).collect::<Vec<_>>().join(","))))]), gemini_only()),
        case("tags-edge", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(&format!(r###"{{"title":"T","summary":"S","tags":["{}","{}","##x y","a  b   c d","a b c d e", "ÜBER", "İstanbul"]}}"###, "a".repeat(40), "b".repeat(41))))]), gemini_only()),
        case("long-title", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(&format!(r#"{{"title":"{}","summary":"{}","tags":[]}}"#, "word ".repeat(100), "s".repeat(900))))]), gemini_only()),
        case("title-spaces", "describe", args(doc.clone()), script("chat", vec![step(200, &chat_body(r#"{"title":"  a   b ","summary":"S","tags":[]}"#))]), gemini_only()),
        case("404", "describe", args(doc.clone()), script("chat", vec![step(404, r#"{"error":{"message":"model gone"}}"#)]), gemini_only()),
        case("403-model", "describe", args(doc.clone()), script("chat", vec![step(403, "no")]), chat_env.clone()),
        case("500-retry", "describe", args(doc.clone()), script("chat", vec![step(500, ""), step(200, &good_chat())]), gemini_only()),
        case("401", "describe", args(doc.clone()), script("chat", vec![step(401, "bad key fake-gemini-key-123")]), gemini_only()),
        case("timeout", "describe", args(doc.clone()), script("chat", vec![hang()]), gemini_only()),
        case("long-text", "describe", args(item(&"T".repeat(900), &"x".repeat(9000))), ok(), gemini_only()),
        case("unicode", "describe", args(item("日本語 😀", "café \u{a0}\u{2003}text 😀😀")), ok(), gemini_only()),
        case("source-parts", "describe", args(obj! { "title" => "t", "domain" => "d.example", "kind" => "Article" }.into()), ok(), gemini_only()),
        case("source-url-only", "describe", args(obj! { "title" => "t", "url" => "https://x.example/p" }.into()), ok(), gemini_only()),
    ];
    for kind in ["Article", "Image", "Screenshot", "Video", "YouTubeVideo", "InstagramReel", "constructor", "__proto__", "toString", "hasOwnProperty", "Note", "WebPage"] {
        let mut value = item("t", "text");
        if let Value::Object(object) = &mut value {
            object.insert("kind", kind);
        }
        cases.push(case(&format!("kind-{kind}"), "describe", args(value), ok(), gemini_only()));
    }
    let long_text = "y".repeat(250);
    for (name, mime, data) in images() {
        for (suffix, kind, text) in [("article-short", "Article", "short"), ("image-long", "Image", long_text.as_str()), ("article-long", "Article", long_text.as_str())] {
            let mut value = with_image(item("t", text), mime, &data);
            if let Value::Object(object) = &mut value {
                object.insert("kind", kind);
            }
            cases.push(case(&format!("image-{name}-{suffix}"), "describe", args(value), ok(), gemini_only()));
        }
    }
    let mut big = png(300, 300, false, 10 * 1024 * 1024);
    cases.push(case("image-10mb", "describe", args(with_image(item("t", "s"), "image/png", &big)), ok(), gemini_only()));
    big.push(0);
    cases.push(case("image-over-10mb", "describe", args(with_image(item("t", "s"), "image/png", &big)), ok(), gemini_only()));
    compare(cases);
}

#[test]
fn pick_tags_matches_node() {
    let doc = item("A title", "Some body text");
    let answers = |count: usize, p: &dyn Fn(usize) -> Value| -> String {
        let entries: Vec<(String, Value)> = (0..count).map(|i| (format!("t{i}"), obj! { "noul" => p(i) }.into())).collect();
        json::to_string(&obj! { "answers" => Value::Object(entries.into_iter().collect()) }.into())
    };
    let fixed = |ps: &[f64]| -> String {
        let entries: Vec<(String, Value)> = ps.iter().enumerate().map(|(i, p)| (format!("t{i}"), obj! { "noul" => *p }.into())).collect();
        json::to_string(&obj! { "answers" => Value::Object(entries.into_iter().collect()) }.into())
    };
    let names = |names: &[&str]| -> Value { Value::Array(names.iter().map(|name| Value::from(*name)).collect()) };
    let args = |item: Value, candidates: Value| -> Value { obj! { "item" => item, "candidates" => candidates }.into() };
    let ok = |body: String| script("systemone", vec![step(200, &body)]);
    let mut cases = vec![
        case("basic", "pickTags", args(doc.clone(), names(&["cats", "dogs", "birds"])), ok(fixed(&[0.9, 0.5, 0.7])), keys()),
        case("no-key", "pickTags", args(doc.clone(), names(&["cats"])), ok(fixed(&[0.9])), gemini_only()),
        case("no-candidates", "pickTags", args(doc.clone(), names(&[])), ok(fixed(&[])), keys()),
        case("no-candidates-no-key", "pickTags", args(doc.clone(), names(&[])), ok(fixed(&[])), j("{}")),
        case("blank-candidates", "pickTags", args(doc.clone(), names(&["", "  ", "\n"])), ok(fixed(&[])), keys()),
        case("dedupe", "pickTags", args(doc.clone(), names(&["Cats", " cats ", "CATS", "Dogs", "dogs"])), ok(fixed(&[0.8, 0.8])), keys()),
        case("dedupe-unicode", "pickTags", args(doc.clone(), names(&["Ünï", "ünï", "İ", "i̇", "日本"])), ok(fixed(&[0.8, 0.8, 0.8, 0.8])), keys()),
        case("no-content", "pickTags", args(obj! {}.into(), names(&["cats"])), ok(fixed(&[0.9])), keys()),
        case("blank-content", "pickTags", args(item(" ", " "), names(&["cats"])), ok(fixed(&[0.9])), keys()),
        case("only-url", "pickTags", args(obj! { "url" => "https://x.example" }.into(), names(&["cats"])), ok(fixed(&[0.9])), keys()),
        case("only-domain", "pickTags", args(obj! { "domain" => "x.example" }.into(), names(&["cats"])), ok(fixed(&[0.9])), keys()),
        case("threshold", "pickTags", args(doc.clone(), names(&["a", "b", "c", "d"])), ok(fixed(&[0.6, 0.59999, 0.6000001, 1.0])), keys()),
        case("ties", "pickTags", args(doc.clone(), names(&["a", "b", "c", "d", "e"])), ok(fixed(&[0.7, 0.9, 0.7, 0.9, 0.7])), keys()),
        case("twelve", "pickTags", args(doc.clone(), Value::Array((0..12).map(|i| Value::from(format!("tag{i}"))).collect())), ok(answers(12, &|i| Value::Number(0.99 - i as f64 * 0.01))), keys()),
        case("non-numbers", "pickTags", args(doc.clone(), names(&["a", "b", "c", "d"])), ok(r#"{"answers":{"t0":{"noul":"0.9"},"t1":{"noul":null},"t2":{},"t3":{"noul":0.8}}}"#.into()), keys()),
        case("missing-answers", "pickTags", args(doc.clone(), names(&["a", "b"])), ok("{}".into()), keys()),
        case("answers-null", "pickTags", args(doc.clone(), names(&["a", "b"])), ok(r#"{"answers":null}"#.into()), keys()),
        case("quotes", "pickTags", args(doc.clone(), names(&["say \"hi\"", "back\\slash", "tab\there", "new\nline", "日本語 😀", "</script>"])), ok(fixed(&[0.9; 6])), keys()),
        case("101", "pickTags", args(doc.clone(), Value::Array((0..101).map(|i| Value::from(format!("tag{i}"))).collect())), script("systemone", vec![step(200, &answers(100, &|_| Value::Number(0.65))), step(200, &fixed(&[0.95]))]), keys()),
        case("250", "pickTags", args(doc.clone(), Value::Array((0..250).map(|i| Value::from(format!("tag{i}"))).collect())), script("systemone", vec![step(200, &answers(100, &|i| Value::Number(0.6 + i as f64 / 1000.0))), step(200, &answers(100, &|_| Value::Number(0.1))), step(200, &answers(50, &|_| Value::Number(0.9)))]), keys()),
        case("second-fails", "pickTags", args(doc.clone(), Value::Array((0..150).map(|i| Value::from(format!("tag{i}"))).collect())), script("systemone", vec![step(200, &answers(100, &|_| Value::Number(0.9))), step(400, "{\"detail\":\"no\"}")]), keys()),
        case("500-then-ok", "pickTags", args(doc.clone(), names(&["a"])), script("systemone", vec![step(500, ""), step(200, &fixed(&[0.9]))]), keys()),
        case("401", "pickTags", args(doc.clone(), names(&["a"])), script("systemone", vec![step(401, "bad fake-typesafe-key-456")]), keys()),
        case("not-json", "pickTags", args(doc.clone(), names(&["a"])), script("systemone", vec![step(200, "x")]), keys()),
        case("timeout", "pickTags", args(doc.clone(), names(&["a"])), script("systemone", vec![hang()]), keys()),
        case("base-url", "pickTags", args(doc.clone(), names(&["a"])), ok(fixed(&[0.9])), j(r#"{"TYPESAFE_API_KEY":"fake-typesafe-key-456","TYPESAFE_BASE_URL":"https://ts.example.test/"}"#)),
        case("long", "pickTags", args(item(&"T".repeat(900), &"é".repeat(7000)), names(&["a"])), ok(fixed(&[0.9])), keys()),
        case("exact-limits", "pickTags", args(obj! { "title" => "a".repeat(500), "domain" => "d".repeat(200), "url" => "u".repeat(500), "text" => "t".repeat(6000), "kind" => "Article" }.into(), names(&["a"])), ok(fixed(&[0.9])), keys()),
        case("over-limits", "pickTags", args(obj! { "title" => "a".repeat(501), "domain" => "d".repeat(201), "url" => "u".repeat(501), "text" => "t".repeat(6001), "kind" => "Article" }.into(), names(&["a"])), ok(fixed(&[0.9])), keys()),
    ];
    let long_text = "z".repeat(250);
    for (name, mime, data) in images() {
        for (suffix, kind, text) in [("image-short", "Image", "short"), ("article-long", "Article", long_text.as_str()), ("article-short", "Article", "short")] {
            let mut value = with_image(item("t", text), mime, &data);
            if let Value::Object(object) = &mut value {
                object.insert("kind", kind);
            }
            cases.push(case(&format!("image-{name}-{suffix}"), "pickTags", args(value, names(&["a", "b"])), ok(fixed(&[0.9, 0.7])), keys()));
        }
        cases.push(case(&format!("image-only-{name}"), "pickTags", args(with_image(obj! {}.into(), mime, &data), names(&["a"])), ok(fixed(&[0.9])), keys()));
    }
    let bytes_limit = png(300, 300, false, 16 * 1024 * 1024);
    cases.push(case("image-16mb", "pickTags", args(with_image(item("t", "s"), "image/png", &bytes_limit), names(&["a"])), ok(fixed(&[0.9])), keys()));
    let mut over = bytes_limit;
    over.push(0);
    cases.push(case("image-over-16mb", "pickTags", args(with_image(item("t", "s"), "image/png", &over), names(&["a"])), ok(fixed(&[0.9])), keys()));
    for (name, data) in [("png-min", png(256, 306, false, 0)), ("png-below-pixels", png(256, 305, false, 0)), ("png-side-max", png(1568, 1568, false, 0)), ("png-pixels-max", png(1568, 1568, false, 0)), ("png-too-many-pixels", png(1568, 1569, false, 0)), ("png-255", png(255, 400, false, 0)), ("png-1569", png(1569, 256, false, 0))] {
        let mut value = with_image(item("t", "s"), "image/png", &data);
        if let Value::Object(object) = &mut value {
            object.insert("kind", "Image");
        }
        cases.push(case(&format!("dims-{name}"), "pickTags", args(value, names(&["a"])), ok(fixed(&[0.9])), keys()));
    }
    compare(cases);
}

#[test]
fn embed_vocabulary_matches_node() {
    let ok = |count: usize| script("batchEmbedContents", vec![step(200, &batch(count))]);
    let names = |names: &[&str]| -> Value { Value::Array(names.iter().map(|name| Value::from(*name)).collect()) };
    let args = |vocabulary: Value| -> Value { obj! { "vocabulary" => vocabulary }.into() };
    let stored = |name: &str, length: usize| -> Value { obj! { "name" => name, "embedding" => (0..length).map(|i| Value::Number(i as f64 / 1000.0 - 0.3)).collect::<Vec<_>>() }.into() };
    let twice = |first: Value, second: Value| -> Value { obj! { "first" => first, "second" => second }.into() };
    let cases = vec![
        case("names", "embedVocabulary", args(names(&["cats", "dogs"])), ok(2), keys()),
        case("empty", "embedVocabulary", args(names(&[])), ok(0), j("{}")),
        case("duplicates", "embedVocabulary", args(names(&["Cats", "cats", " CATS ", "dogs", ""])), ok(2), keys()),
        case("blank", "embedVocabulary", args(names(&["", "  "])), ok(0), j("{}")),
        case("trim", "embedVocabulary", args(names(&["  spaced  tag ", "ÜBER"])), ok(2), keys()),
        case("stored", "embedVocabulary", args(Value::Array(vec![stored("cats", 768), stored("dogs", 768)])), ok(0), j("{}")),
        case("stored-wrong-length", "embedVocabulary", args(Value::Array(vec![stored("cats", 767), stored("dogs", 768)])), ok(1), keys()),
        case("stored-mixed", "embedVocabulary", args(Value::Array(vec![stored("cats", 768), "dogs".into(), obj! { "name" => "birds" }.into()])), ok(2), keys()),
        case("stored-duplicate", "embedVocabulary", args(Value::Array(vec!["cats".into(), stored("Cats", 768)])), ok(1), keys()),
        case("object-no-name", "embedVocabulary", args(Value::Array(vec![obj! { "embedding" => vec![Value::Number(1.0)] }.into(), "x".into()])), ok(1), keys()),
        case("no-key", "embedVocabulary", args(names(&["cats"])), ok(1), j("{}")),
        case("fails", "embedVocabulary", args(names(&["cats"])), script("batchEmbedContents", vec![step(500, "x")]), keys()),
        case("101", "embedVocabulary", args(Value::Array((0..101).map(|i| Value::from(format!("tag{i}"))).collect())), script("batchEmbedContents", vec![step(200, &batch(100)), step(200, &batch(1))]), keys()),
        case("cache", "embedVocabularyTwice", twice(names(&["cats", "dogs"]), names(&["dogs", "birds", "CATS"])), ok(2), keys()),
        case("cache-stored", "embedVocabularyTwice", twice(Value::Array(vec![stored("cats", 768)]), names(&["cats", "dogs"])), ok(1), keys()),
        case("cache-overwrite", "embedVocabularyTwice", twice(names(&["cats"]), Value::Array(vec![stored("cats", 768)])), ok(1), keys()),
        case("cache-after-failure", "embedVocabularyTwice", twice(names(&["cats"]), names(&["cats"])), script("batchEmbedContents", vec![step(200, &batch(1))]), keys()),
    ];
    compare(cases);
}

#[test]
fn analyze_matches_node() {
    let doc = item("A title", "Some body text");
    let args = |item: Value, vocabulary: Value| -> Value { obj! { "item" => item, "vocabulary" => vocabulary }.into() };
    let names = |names: &[&str]| -> Value { Value::Array(names.iter().map(|name| Value::from(*name)).collect()) };
    let answers = |count: usize| -> String {
        let entries: Vec<(String, Value)> = (0..count).map(|i| (format!("t{i}"), obj! { "noul" => 0.95 - (i % 7) as f64 * 0.05 }.into())).collect();
        json::to_string(&obj! { "answers" => Value::Object(entries.into_iter().collect()) }.into())
    };
    let all = |chat: &str, vocab: usize, ts: usize| {
        scripts(vec![
            ("embedContent", vec![step(200, &single(1))]),
            ("batchEmbedContents", vec![step(200, &batch(vocab))]),
            ("chat", vec![step(200, chat)]),
            ("systemone", vec![step(200, &answers(ts))]),
        ])
    };
    let vocab = names(&["cats", "Dogs", "birds", "fish"]);
    let stored = |name: &str, seed: usize| -> Value { obj! { "name" => name, "embedding" => emb_values(seed) }.into() };
    let mut cases = vec![
        case("with-jev", "analyze", args(doc.clone(), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("without-jev", "analyze", args(doc.clone(), vocab.clone()), all(&good_chat(), 4, 0), gemini_only()),
        case("empty-vocabulary", "analyze", args(doc.clone(), names(&[])), all(&good_chat(), 0, 100), keys()),
        case("empty-vocabulary-no-jev", "analyze", args(doc.clone(), names(&[])), all(&good_chat(), 0, 0), gemini_only()),
        case("stored-vocabulary", "analyze", args(doc.clone(), Value::Array(vec![stored("cats", 3), stored("Dogs", 4), stored("birds", 5)])), all(&good_chat(), 0, 100), keys()),
        case("stored-vocabulary-no-jev", "analyze", args(doc.clone(), Value::Array(vec![stored("Cats", 3), stored("dogs", 4)])), all(&good_chat(), 0, 0), gemini_only()),
        case("spelling", "analyze", args(doc.clone(), names(&["Cats", "Big Dogs", "mixed case"])), all(&good_chat(), 3, 100), keys()),
        case("spelling-no-jev", "analyze", args(doc.clone(), names(&["Cats", "Big Dogs", "mixed case"])), all(&good_chat(), 3, 0), gemini_only()),
        case("many-vocabulary", "analyze", args(doc.clone(), Value::Array((0..30).map(|i| Value::from(format!("voc{i}"))).collect())), scripts(vec![("embedContent", vec![step(200, &single(1))]), ("batchEmbedContents", vec![step(200, &batch(30))]), ("chat", vec![step(200, &good_chat())]), ("systemone", vec![step(200, &answers(100))])]), keys()),
        case("kind-image", "analyze", args(obj! { "title" => "t", "text" => "s", "kind" => "Image" }.into(), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("kind-object-proto", "analyze", args(obj! { "title" => "t", "text" => "s", "kind" => "constructor" }.into(), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("chat-empty-tags", "analyze", args(doc.clone(), vocab.clone()), all(&chat_body(r#"{"title":"T","summary":"S","tags":[]}"#), 4, 100), keys()),
        case("jev-low", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step(200, &single(1))]), ("batchEmbedContents", vec![step(200, &batch(4))]), ("chat", vec![step(200, &good_chat())]), ("systemone", vec![step(200, &answers(0))])]), keys()),
        case("retry-after", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step_with(429, "", r#"{"retry-after":"2"}"#), step(200, &single(1))]), ("batchEmbedContents", vec![step_with(503, "", r#"{"retry-after":"3"}"#), step(200, &batch(4))]), ("chat", vec![step_with(429, "", r#"{"retry-after":"1"}"#), step(200, &good_chat())]), ("systemone", vec![step(200, &answers(100))])]), keys()),
        case("no-embeddable", "analyze", args(with_image(obj! { "kind" => "Image" }.into(), "image/webp", &webp()), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("no-embeddable-gif", "analyze", args(with_image(obj! { "kind" => "Image" }.into(), "image/gif", &images()[11].2), vocab.clone()), all(&good_chat(), 4, 0), gemini_only()),
        case("no-embeddable-animated", "analyze", args(with_image(obj! { "kind" => "Image" }.into(), "image/png", &png(300, 300, true, 0)), names(&[])), all(&good_chat(), 0, 100), keys()),
        case("embeddable-png", "analyze", args(with_image(obj! { "kind" => "Image" }.into(), "image/png", &png(300, 300, false, 0)), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("whitespace-only", "analyze", args(item(" ", " "), vocab.clone()), all(&good_chat(), 4, 100), keys()),
        case("no-key", "analyze", args(doc.clone(), vocab.clone()), all(&good_chat(), 4, 100), j("{}")),
        case("chat-fails", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step(200, &single(1))]), ("batchEmbedContents", vec![step(200, &batch(4))]), ("chat", vec![step(400, r#"{"error":{"message":"nope"}}"#)])]), keys()),
        case("embed-fails", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step(400, "bad")]), ("batchEmbedContents", vec![step(200, &batch(4))]), ("chat", vec![step(200, &good_chat())])]), keys()),
        case("vocab-fails", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step(200, &single(1))]), ("batchEmbedContents", vec![step(400, "bad")]), ("chat", vec![step(200, &good_chat())])]), keys()),
        case("jev-fails", "analyze", args(doc.clone(), vocab.clone()), scripts(vec![("embedContent", vec![step(200, &single(1))]), ("batchEmbedContents", vec![step(200, &batch(4))]), ("chat", vec![step(200, &good_chat())]), ("systemone", vec![step(401, "bad")])]), keys()),
        case("no-summary", "analyze", args(with_image(obj! { "kind" => "Image" }.into(), "image/webp", &webp()), vocab.clone()), all(&chat_body(r#"{"title":"T","tags":[]}"#), 4, 100), keys()),
    ];
    cases.push(case("unicode", "analyze", args(item("日本語 😀", "café text"), names(&["日本語", "Ünï"])), all(&good_chat(), 2, 100), keys()));
    compare(cases);
}

#[test]
fn pure_functions_match_node() {
    let mut cases = Vec::new();
    let texts = [("", ""), ("t", ""), ("", "b"), ("  t \n u ", "\tb  c "), ("日本語 😀", "é\u{a0}x"), (&"T".repeat(501) as &str, &"b".repeat(6001) as &str)];
    for (title, text) in texts {
        cases.push(case("text", "documentText", obj! { "item" => item(title, text) }.into(), Value::Null, j("{}")));
    }
    cases.push(case("null-fields", "documentText", obj! { "item" => j(r#"{"title":null,"text":null}"#) }.into(), Value::Null, j("{}")));
    cases.push(case("emoji-cut", "documentText", obj! { "item" => item(&"😀".repeat(251), &"😀".repeat(3001)) }.into(), Value::Null, j("{}")));
    let numbers = |values: &[f64]| -> Value { Value::Array(values.iter().map(|value| Value::Number(*value)).collect()) };
    for (a, b) in [
        (vec![1.0, 0.0], vec![1.0, 0.0]),
        (vec![1.0, 0.0], vec![0.0, 1.0]),
        (vec![1.0, 2.0, 3.0], vec![-1.0, -2.0, -3.0]),
        (vec![0.0, 0.0], vec![1.0, 1.0]),
        (vec![1.0, 1.0], vec![0.0, 0.0]),
        (vec![], vec![]),
        (vec![1.0, 2.0], vec![1.0]),
        (vec![1.0], vec![1.0, 2.0]),
        (vec![1e200, 1e200], vec![1e200, 1e200]),
        (vec![1e-200, 1e-200], vec![1e-200, 1e-200]),
        (vec![0.1, 0.2, 0.3], vec![0.3, 0.2, 0.1]),
    ] {
        cases.push(case("cosine", "cosine", obj! { "a" => numbers(&a), "b" => numbers(&b) }.into(), Value::Null, j("{}")));
    }
    let mut vectors: Vec<Vec<f32>> = Vec::new();
    for path in [test_db(), hosted_db()] {
        let db = Sqlite::open(path, true).unwrap();
        for row in db.query("SELECT vector FROM embeddings ORDER BY rowid LIMIT 120", &[]).unwrap() {
            let blob = row.blob("vector").unwrap();
            vectors.push(blob.as_chunks::<4>().0.iter().map(|bytes| f32::from_le_bytes(*bytes)).collect());
        }
    }
    assert!(vectors.len() > 100, "the fixtures hold embeddings");
    for pair in vectors.windows(2) {
        let as_numbers = |v: &Vec<f32>| Value::Array(v.iter().map(|value| Value::Number(f64::from(*value))).collect());
        cases.push(case("fixture-cosine", "cosine", obj! { "a" => as_numbers(&pair[0]), "b" => as_numbers(&pair[1]) }.into(), Value::Null, j("{}")));
    }
    for path in [test_db(), hosted_db()] {
        let db = Sqlite::open(path, true).unwrap();
        for row in db.query("SELECT data, extras FROM cards ORDER BY rowid", &[]).unwrap() {
            let data = json::parse(row.text("data").unwrap()).unwrap();
            let extras = json::parse(row.text("extras").unwrap()).unwrap();
            let text: Vec<String> = [data.get("description"), data.get("prose"), extras.get("captureText")]
                .into_iter()
                .filter(|value| value.truthy())
                .map(|value| value.to_js_string())
                .collect();
            let title = data.get("title").clone();
            let item: Value = obj! { "title" => title, "text" => text.join("\n") }.into();
            cases.push(case("fixture-card", "documentText", obj! { "item" => item }.into(), Value::Null, j("{}")));
        }
    }
    compare(cases);
}
