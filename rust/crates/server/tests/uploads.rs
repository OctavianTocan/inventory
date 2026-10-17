//! Frozen `uploads` and `DirectoryUploads` expectations: generated authorize, put, prepare, claim, cleanup, source
//! and sweep programs retain all answers, rows and files.
//!
//! Ids, tokens and generated card ids are random, so both sides name them by position (`@u0`, `@t0`, `@new`) and
//! replace the real values in every answer, row and file name before comparing. Time is fixed on both sides.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use async_trait::async_trait;
use common::Random;
use mm_core::host::{AnalysisImage, Media, MediaInput, MediaResult, MediaUser, PreparedMedia};
use mm_core::http::{Body, Headers, Response};
use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_core::uploads::{UploadOptions, UploadRequest, Uploads};
use mm_core::{Error, Result, Value, obj};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use mymind::uploads_fs::DirectoryUploads;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/uploads.mjs");
const NOW: i64 = 1_800_000_000_000;

struct FakeMedia {
    fresh: std::cell::RefCell<Option<String>>,
}

#[async_trait(?Send)]
impl Media for FakeMedia {
    fn cover(&self, _blob: &Value) -> Result<Value> {
        unreachable!()
    }

    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        *self.fresh.borrow_mut() = Some(id.to_owned());
        if name == "bad.bin" {
            return Err(Error::internal("cannot preview"));
        }
        if name == "status.bin" {
            return Err(Error::status(507, "quota"));
        }
        let (size, reported) = match &input {
            MediaInput::File(path) => (Value::from(std::fs::metadata(path).map_or(0.0, |meta| meta.len() as f64)), Value::from(path.as_str())),
            MediaInput::Key(key) => (Value::Null, Value::from(key.as_str())),
            _ => (Value::Null, Value::Null),
        };
        Ok(PreparedMedia {
            data: obj! { "type" => "Image", "title" => name, "blob" => obj! { "path" => format!("p/{id}") }, "size" => size, "mime" => content_type, "input" => reported, "user" => user.account_uid.as_str() }.into(),
            extras: obj! {}.into(),
        })
    }

    async fn send(&self, _result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        unreachable!()
    }

    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        Ok((!relative.contains("gone")).then(|| MediaResult { key: relative.to_owned(), size: 0, content_type: "image/jpeg".into() }))
    }

    async fn image(&self, _blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        unreachable!()
    }
}

fn names(state: &Names, text: &str) -> String {
    let mut text = text.to_owned();
    let mut swaps: Vec<(String, String)> = vec![(state.dir.clone(), "@dir".into())];
    swaps.extend(state.tokens.iter().enumerate().map(|(i, token)| (token.clone(), format!("@t{i}"))));
    swaps.extend(state.ids.iter().enumerate().map(|(i, id)| (id.clone(), format!("@u{i}"))));
    swaps.extend(state.cards.iter().map(|(key, id)| (id.clone(), format!("@c{key}"))));
    if let Some(fresh) = &state.fresh {
        swaps.push((fresh.clone(), "@new".into()));
    }
    for (from, to) in swaps {
        text = text.replace(&from, &to);
    }
    text
}

struct Names {
    dir: String,
    ids: Vec<String>,
    tokens: Vec<String>,
    cards: Vec<(String, String)>,
    fresh: Option<String>,
}

impl Names {
    fn resolve(&self, text: &str) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(at) = rest.find('@') {
            out.push_str(&rest[..at]);
            let tail = &rest[at + 1..];
            let kind = tail.chars().next();
            let digits: String = tail.chars().skip(1).take_while(|c| c.is_ascii_digit() || c.is_ascii_lowercase()).collect();
            let found = match kind {
                Some('u') => digits.parse::<usize>().ok().and_then(|i| self.ids.get(i)).cloned(),
                Some('t') => digits.parse::<usize>().ok().and_then(|i| self.tokens.get(i)).cloned(),
                Some('c') => self.cards.iter().find(|(key, _)| *key == digits).map(|(_, id)| id.clone()),
                _ => None,
            };
            match (found, kind) {
                (Some(found), _) if !digits.is_empty() => {
                    out.push_str(&found);
                    rest = &tail[1 + digits.len()..];
                }
                _ => {
                    out.push('@');
                    rest = tail;
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn deep(&self, value: &Value) -> Value {
        match value {
            Value::String(text) => Value::from(self.resolve(text)),
            Value::Array(items) => Value::Array(items.iter().map(|item| self.deep(item)).collect()),
            Value::Object(object) => Value::Object(object.iter().map(|(key, item)| (key.to_string(), self.deep(item))).collect()),
            other => other.clone(),
        }
    }

    fn normalize(&self, value: &Value) -> Value {
        json::parse(&names(self, &json::to_string(value))).expect("normalized json")
    }
}

fn request(op: &Value, names: &Names) -> UploadRequest {
    let headers = names.deep(op.get("headers"));
    let headers = Headers::from_pairs(headers.as_object().unwrap().iter().map(|(key, value)| (key.to_string(), value.as_str().unwrap().to_owned())));
    let body = op.get("body");
    let length = body.get("length").as_f64().unwrap() as usize;
    let byte = body.get("byte").as_f64().unwrap() as usize;
    let data: Vec<u8> = (0..length).map(|i| ((i * 7 + byte) % 251) as u8).collect();
    let sizes: Vec<usize> = body.get("chunks").as_array().unwrap().iter().map(|size| size.as_f64().unwrap() as usize).collect();
    let fail_after = body.get("failAfter").as_f64().map(|k| k as usize);
    let mut pieces: Vec<Result<Vec<u8>>> = Vec::new();
    let (mut at, mut k) = (0, 0);
    while at < data.len() {
        let size = sizes[k % sizes.len()].max(1);
        let end = (at + size).min(data.len());
        if fail_after == Some(pieces.len()) {
            pieces.push(Err(Error::internal("boom")));
            break;
        }
        pieces.push(Ok(data[at..end].to_vec()));
        at = end;
        k += 1;
    }
    UploadRequest { headers, body: Body::Stream(Box::pin(futures_util::stream::iter(pieces))) }
}

fn answer(state: &Names, result: Result<Value>) -> Value {
    match result {
        Ok(value) => obj! { "ok" => state.normalize(&value) }.into(),
        Err(Error::Status { status, message, .. }) => obj! { "error" => obj! { "status" => f64::from(status), "message" => message } }.into(),
        Err(_) => obj! { "error" => obj! { "status" => Value::Null, "message" => "other" } }.into(),
    }
}

fn source_value(source: mm_core::uploads::UploadSource) -> Value {
    let input = match source.input {
        MediaInput::File(file) => obj! { "file" => file },
        MediaInput::Key(key) => obj! { "key" => key },
        MediaInput::Upload(prefix) => obj! { "upload" => prefix },
        MediaInput::Bytes(_) => obj! { "bytes" => true },
    };
    obj! { "input" => input, "type" => source.content_type, "name" => source.name }.into()
}

async fn execute(op: &Value, state: &mut Names, uploads: &Uploads, db: &Sqlite, _media: &FakeMedia, now: &mut i64, dir: &Path) -> Result<Value> {
    let resolved = state.deep(op);
    let text = |key: &str| resolved.get(key).as_str().unwrap_or("").to_owned();
    match op.get("op").as_str().unwrap() {
        "authorize" => {
            let result = uploads.authorize(op.get("count")).await?;
            let items: Vec<Value> = match &result {
                Value::Array(items) => items.clone(),
                single => vec![single.clone()],
            };
            for item in items {
                state.ids.push(item.get("url").as_str().unwrap().rsplit('/').next().unwrap().to_owned());
                state.tokens.push(item.get("token").as_str().unwrap().to_owned());
            }
            Ok(result)
        }
        "put" => uploads.put(&text("upload"), request(op, state)).await,
        "prepare" => {
            let result = uploads.prepare(&text("upload")).await?;
            if let Some(id) = result.get("id").as_str() {
                state.fresh = Some(id.to_owned());
            }
            Ok(result)
        }
        "claim" => uploads.claim(&text("upload"), &text("card"), resolved.get("reuse").truthy()).map(|()| Value::Null),
        "cleanup" => uploads.cleanup_claimed(&text("upload")).await.map(|()| Value::Null),
        "source" => uploads.source(&text("upload")).await.map(source_value),
        "sweep" => uploads.sweep().await.map(|()| Value::Null),
        "time" => {
            *now += op.get("ms").as_f64().unwrap() as i64;
            mm_js::date::set_fixed_now(Some(*now));
            Ok(Value::Null)
        }
        "sql" => {
            let params: Vec<SqlValue> = resolved.get("params").as_array().unwrap().iter().map(|item| SqlValue::from(item.as_str().unwrap())).collect();
            db.execute(&text("sql"), &params).map(|_| Value::Null)
        }
        "plant" => {
            std::fs::create_dir_all(dir).map_err(|error| Error::internal(error.to_string()))?;
            std::fs::write(dir.join(text("name")), text("text")).map_err(|error| Error::internal(error.to_string()))?;
            Ok(Value::Null)
        }
        "tail" => {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new().append(true).create(true).open(dir.join(text("upload"))).map_err(|error| Error::internal(error.to_string()))?;
            file.write_all(text("text").as_bytes()).map_err(|error| Error::internal(error.to_string()))?;
            Ok(Value::Null)
        }
        "race" => unreachable!("handled by the caller"),
        other => panic!("unknown op {other}"),
    }
}

async fn attempt(op: &Value, state: &mut Names, uploads: &Uploads, db: &Sqlite, media: &FakeMedia, now: &mut i64, dir: &Path) -> Value {
    let result = execute(op, state, uploads, db, media, now, dir).await;
    if let Some(fresh) = media.fresh.borrow().clone() {
        state.fresh = Some(fresh);
    }
    answer(state, result)
}

fn rust_program(db: &Rc<Sqlite>, store: &Rc<Store>, program: &Value, dir: &Path, cards: &[(String, String)]) -> Value {
    mymind::runtime::run(async {
        mm_js::date::set_fixed_now(Some(NOW));
        db.exec_batch("DELETE FROM uploads").unwrap();
        let media = Rc::new(FakeMedia { fresh: std::cell::RefCell::new(None) });
        let mut options = UploadOptions::default();
        if let Some(max) = program.get("maxBytes").as_f64() {
            options.max_bytes = max as u64;
        }
        let uploads = Uploads::new(store.clone(), Rc::new(DirectoryUploads::new(dir)), media.clone(), options).unwrap();
        let mut state = Names { dir: dir.to_string_lossy().into_owned(), ids: Vec::new(), tokens: Vec::new(), cards: cards.to_vec(), fresh: None };
        let mut now = NOW;
        let mut results = Vec::new();
        for op in program.get("ops").as_array().unwrap() {
            if op.get("op").as_str() == Some("race") {
                let ops = op.get("ops").as_array().unwrap();
                let (first, second) = (&ops[0], &ops[1]);
                let resolved: Vec<Value> = ops.iter().map(|item| state.deep(item)).collect();
                let (a, b) = futures_util::future::join(
                    uploads.put(resolved[0].get("upload").as_str().unwrap(), request(first, &state)),
                    uploads.put(resolved[1].get("upload").as_str().unwrap(), request(second, &state)),
                )
                .await;
                results.push(obj! { "ok" => Value::Array(vec![answer(&state, a), answer(&state, b)]) }.into());
                continue;
            }
            results.push(attempt(op, &mut state, &uploads, db, &media, &mut now, dir).await);
        }
        let rows: Vec<Value> = db
            .query("SELECT * FROM uploads ORDER BY rowid", &[])
            .unwrap()
            .iter()
            .map(|row| {
                let mut object = row.to_object();
                object.remove("token_hash");
                state.normalize(&Value::Object(object))
            })
            .collect();
        let mut files: Vec<(String, String)> = std::fs::read_dir(dir)
            .map(|entries| entries.flatten().map(|entry| (names(&state, entry.file_name().to_str().unwrap()), base64::encode(&std::fs::read(entry.path()).unwrap()))).collect())
            .unwrap_or_default();
        files.sort();
        obj! { "results" => Value::Array(results), "rows" => Value::Array(rows), "files" => Value::Array(files.into_iter().map(|(name, data)| Value::Array(vec![name.into(), data.into()])).collect()) }.into()
    })
}

fn pick<'a, T>(random: &mut Random, items: &'a [T]) -> &'a T {
    &items[random.below(items.len() as u64) as usize]
}

struct Known {
    received: u64,
    total: Option<u64>,
    name: String,
    mime: String,
}

fn put_op(random: &mut Random, index: usize, known: &mut Known, good: bool) -> Value {
    let start = if good || random.below(8) != 0 { known.received } else { random.below(30) };
    let length = 1 + random.below(24);
    let mut end = start + length - 1;
    if random.below(40) == 0 {
        end = start.saturating_sub(1);
    }
    let total = match known.total {
        Some(total) if good || random.below(8) != 0 => total,
        _ => (end + 1).saturating_sub(random.below(3)) + random.below(40),
    };
    let token = if good || random.below(8) != 0 { format!("Bearer @t{index}") } else { (*pick(random, &["Bearer nope", "@t0", "Bearer", "bearer @t0", "Bearer @t1"])).to_owned() };
    let mut headers = obj! {};
    if good || random.below(30) != 0 {
        headers.insert("authorization", token);
    }
    let range = match random.below(40) {
        0 => "bytes 0-5".to_owned(),
        1 => "".to_owned(),
        2 => format!("bytes {start}-{end}/{total} "),
        3 => format!("bytes 99999999999999999999-{end}/{total}"),
        _ => format!("bytes {start}-{end}/{total}"),
    };
    if !range.is_empty() {
        headers.insert("content-range", range);
    }
    let mime = if known.received > 0 && random.below(10) == 0 { "text/plain".to_owned() } else { known.mime.clone() };
    match random.below(8) {
        0 => {}
        1 => headers.insert("content-type", format!("{mime}; charset=utf-8")),
        _ => headers.insert("content-type", mime),
    }
    let name = match random.below(14) {
        0 => "%E0%A4%A".to_owned(),
        1 => "a%20b%2Fc.bin".to_owned(),
        2 => "x".repeat(300),
        3 => "bad.bin".to_owned(),
        4 => "status.bin".to_owned(),
        _ => known.name.clone(),
    };
    if random.below(10) != 0 {
        headers.insert("x-file-name", if known.received > 0 && random.below(12) == 0 { "other.bin".to_owned() } else { name.clone() });
    }
    match random.below(10) {
        0 => headers.insert("x-upload-id", "wrong"),
        1 => headers.insert("x-upload-id", format!("@u{index}")),
        _ => {}
    }
    let body_length = match random.below(12) {
        0 => length + 1,
        1 => length.saturating_sub(1),
        _ => length,
    };
    let failing = random.below(16) == 0;
    let chunks: Vec<Value> = (0..1 + random.below(4)).map(|_| Value::from((1 + random.below(9)) as f64)).collect();
    let mut op = obj! {
        "op" => "put", "upload" => format!("@u{index}"), "headers" => headers,
        "body" => obj! { "length" => body_length as f64, "byte" => random.below(200) as f64, "chunks" => Value::Array(chunks),
            "failAfter" => if failing { Value::from(random.below(3) as f64) } else { Value::Null } },
    };
    if good && body_length == length && !failing {
        known.received = end + 1;
        known.total = Some(total);
        known.name = name;
    }
    if random.below(40) == 0 {
        op.insert("huge", true);
    }
    op.into()
}

fn program(random: &mut Random, cards: &[(String, String)], dir: &str) -> Value {
    let mut ops: Vec<Value> = Vec::new();
    let mut allocated = 0usize;
    let mut knowns: Vec<Known> = Vec::new();
    let authorize = |random: &mut Random, ops: &mut Vec<Value>, allocated: &mut usize, knowns: &mut Vec<Known>| {
        let (count, made): (Value, usize) = match random.below(12) {
            0..=2 => (Value::Null, 1),
            3 | 4 => (Value::from(1.0), 1),
            5 => (Value::from(2.0), 2),
            6 => (Value::from(3.0), 3),
            7 => (Value::from("2"), 2),
            8 => (Value::from(*pick(random, &[0.0, 101.0, 2.5, -1.0])), 0),
            9 => (Value::from("abc"), 0),
            10 => (Value::Bool(true), 1),
            _ => (Value::Undefined, 0),
        };
        ops.push(obj! { "op" => "authorize", "count" => count }.into());
        for _ in 0..made {
            let mime = (*pick(random, &["image/png", "text/plain", "application/pdf", ""])).to_owned();
            knowns.push(Known { received: 0, total: None, name: format!("file{}.bin", *allocated), mime });
            *allocated += 1;
        }
    };
    authorize(random, &mut ops, &mut allocated, &mut knowns);
    for _ in 0..(30 + random.below(30)) {
        if allocated == 0 {
            authorize(random, &mut ops, &mut allocated, &mut knowns);
            continue;
        }
        let index = random.below(allocated as u64 + 1).min(allocated as u64 - 1) as usize;
        let index = if random.below(30) == 0 { allocated + 3 } else { index };
        let target = format!("@u{index}");
        match random.below(100) {
            0..=9 => authorize(random, &mut ops, &mut allocated, &mut knowns),
            10..=54 if index < allocated => {
                let good = random.below(5) != 0;
                let op = put_op(random, index, &mut knowns[index], good);
                ops.push(op);
            }
            55..=62 => ops.push(obj! { "op" => "prepare", "upload" => target }.into()),
            63..=70 => ops.push(obj! { "op" => "claim", "upload" => target, "card" => *pick(random, &["@ca", "@cb", "@cc", "@ca", "@cnope"]), "reuse" => random.below(3) == 0 }.into()),
            71..=74 => ops.push(obj! { "op" => "cleanup", "upload" => target }.into()),
            75..=82 => ops.push(obj! { "op" => "source", "upload" => target }.into()),
            83..=86 => ops.push(obj! { "op" => "sweep" }.into()),
            87..=89 => ops.push(obj! { "op" => "time", "ms" => *pick(random, &[3_600_000.0, 90_000_000.0, 1000.0]) }.into()),
            90..=92 => {
                let which = *pick(random, &[0, 1, 2]);
                let (sql, params): (&str, Vec<Value>) = match which {
                    0 => ("UPDATE cards SET data = json_set(data, '$.blob.path', ?) WHERE id = ?", vec![(*pick(random, &["p/ok.jpg", "p/gone.jpg"])).into(), "@ca".into()]),
                    1 => (
                        "UPDATE cards SET extras = json_set(extras, '$.attachments', json(?)) WHERE id = ?",
                        vec![format!("[{{\"uploadId\":\"@u{index}\",\"path\":\"/media/{}\"}}]", pick(random, &["x/y.png", "gone/y.png"])).into(), (*pick(random, &["@cb", "@ca"])).into()],
                    ),
                    _ => ("UPDATE cards SET data = json_set(data, '$.blob.path', ?) WHERE id = ?", vec!["p/shared.jpg".into(), "@cb".into()]),
                };
                ops.push(obj! { "op" => "sql", "sql" => sql, "params" => Value::Array(params) }.into());
            }
            93..=95 => {
                let name = *pick(random, &["0123456789abcdef012345", "stray.txt", "0123456789ABCDEF012345", "@u0", "ffffffffffffffffffffff"]);
                ops.push(obj! { "op" => "plant", "name" => name, "text" => "stray" }.into());
            }
            96..=97 if index < allocated => ops.push(obj! { "op" => "tail", "upload" => target, "text" => "TAIL" }.into()),
            _ if index < allocated => {
                let mut first = put_op(random, index, &mut knowns[index], true);
                let second = first.clone();
                if random.below(2) == 0 {
                    first.set("upload", format!("@u{}", (index + 1) % allocated));
                }
                ops.push(obj! { "op" => "race", "ops" => Value::Array(vec![first, second]) }.into());
            }
            _ => ops.push(obj! { "op" => "sweep" }.into()),
        }
    }
    let _ = cards;
    let mut out = obj! { "dir" => dir, "now" => NOW as f64, "ops" => Value::Array(ops) };
    if random.below(3) == 0 {
        out.insert("maxBytes", *pick(random, &[30.0, 60.0, 200.0]));
    }
    out.into()
}

fn scratch(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("uploads-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn uploads_match_node() {
    let js_copy = common::temp_copy(&common::test_db());
    let rust_copy = common::temp_copy(&common::test_db());
    let db = Rc::new(Sqlite::open(&rust_copy, false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    let ids: Vec<String> = db.query("SELECT id FROM cards ORDER BY id LIMIT 3", &[]).unwrap().iter().map(|row| row.text("id").unwrap().to_owned()).collect();
    assert_eq!(ids.len(), 3);
    let cards: Vec<(String, String)> = vec![("a".into(), ids[0].clone()), ("b".into(), ids[1].clone()), ("cnope".into(), "nope".into())];
    let cards_js: Vec<(String, String)> = vec![("a".into(), ids[0].clone()), ("b".into(), ids[1].clone()), ("c".into(), ids[2].clone()), ("nope".into(), "nope".into())];
    let cards_rust: Vec<(String, String)> = cards_js.clone();
    let _ = cards;
    let js_root = scratch("js");
    let rust_root = scratch("rust");
    let mut random = Random(0x7570_6c64);
    let mut programs = Vec::new();
    for index in 0..60 {
        programs.push(program(&mut random, &cards_js, &js_root.join(format!("p{index}")).to_string_lossy()));
    }
    let card_map = Value::Object(cards_js.iter().map(|(key, id)| (key.clone(), Value::from(id.as_str()))).collect());
    let js_programs = Value::Array(
        programs
            .iter()
            .map(|program| {
                let mut copy = program.clone();
                copy.set("cards", card_map.clone());
                copy
            })
            .collect(),
    );
    let expected = common::call(DRIVER, "run", vec![js_copy.to_string_lossy().as_ref().into(), js_programs]);
    let (mut oks, mut errors, mut sources) = (0usize, 0usize, 0usize);
    let mut answers = Vec::with_capacity(programs.len());
    assert_eq!(programs.len(), 60, "seeded upload program count");
    for (index, program) in programs.iter().enumerate() {
        let dir = rust_root.join(format!("p{index}"));
        let rust = rust_program(&db, &store, program, &dir, &cards_rust);
        let results = rust.get("results").as_array().expect("upload results");
        let ops = program.get("ops").as_array().unwrap();
        assert_eq!(results.len(), ops.len(), "program {index}: operation count");
        for (step, result) in results.iter().enumerate() {
            oks += usize::from(!result.get("ok").is_undefined());
            errors += usize::from(!result.get("error").is_undefined());
            sources += usize::from(common::text(&ops[step]).contains("\"source\"") && !result.get("ok").is_undefined());
        }
        answers.push(rust);
    }
    common::assert_value(&Value::Array(answers), &expected, "all upload programs, answers, rows and files");
    mm_js::date::set_fixed_now(None);
    assert!(oks > 400 && errors > 200 && sources >= 3, "ok {oks}, errors {errors}, sources {sources}");
    println!("uploads programs: {} ({oks} answers, {errors} errors, {sources} sources)", programs.len());
    for root in [js_root, rust_root] {
        let _ = std::fs::remove_dir_all(root);
    }
}
