//! New-save enrichment and the resumable backfill, against stubbed Gemini and OpenAI Decisions.

#[path = "tools_common/mod.rs"]
mod tools_common;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use async_trait::async_trait;
use mm_core::ai::forget_vocabulary;
use mm_core::api::{Api, ApiOptions};
use mm_core::host::{AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime;
use mm_core::site::LibraryApi;
use mm_core::sql::{Sql, SqlExt};
use mm_core::store::Store;
use mm_core::{Error, Result, Value, obj};
use mm_js::json;
use mm_sqlite::Sqlite;
use mymind::uploads_fs::DirectoryUploads;
use tools_common::{ai_fake, run_rust, tmproot};

static LOCK: Mutex<()> = Mutex::new(());

struct Calls(RefCell<Vec<(u16, String, String)>>);

struct Stub {
    calls: Rc<Calls>,
    chat_status: u16,
}

#[async_trait(?Send)]
impl Fetch for Stub {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let url = request.url.to_string();
        let body = String::from_utf8_lossy(&request.take_body().bytes().await?).into_owned();
        let chat = url.contains("/chat/completions");
        let status = if chat { self.chat_status } else { 200 };
        self.calls.0.borrow_mut().push((status, url.clone(), body.clone()));
        let text = if status == 429 {
            "{}".to_owned()
        } else if url.contains(":batchEmbedContents") {
            let count = json::parse(&body)?.get("requests").as_array().map_or(0, Vec::len);
            let entries: Vec<Value> = (0..count).map(|_| obj! { "values" => vector() }.into()).collect();
            json::to_string(&obj! { "embeddings" => Value::Array(entries) }.into())
        } else if url.contains(":embedContent") {
            json::to_string(&obj! { "embedding" => obj! { "values" => vector() } }.into())
        } else if chat {
            let content = json::to_string(&obj! { "title" => "Generated title", "summary" => "A generated summary.", "tags" => mm_core::arr!["alpha", "design"] }.into());
            json::to_string(&obj! { "choices" => mm_core::arr![obj! { "message" => obj! { "content" => content } }] }.into())
        } else if url.contains("/v1/decisions") {
            r#"{"answers":[{"type":"predicate","name":"t0","probability":0.9},{"type":"predicate","name":"t1","probability":0.8}]}"#.to_owned()
        } else {
            "{}".to_owned()
        };
        let headers = if status == 429 { Headers::from([("retry-after", "0"), ("content-type", "application/json")]) } else { Headers::from([("content-type", "application/json")]) };
        Ok(Response::new(status, headers, Body::Bytes(text.into_bytes())))
    }
}

struct Quiet;

#[async_trait(?Send)]
impl Media for Quiet {
    fn cover(&self, _: &Value) -> Result<Value> {
        Err(Error::internal("unused"))
    }
    async fn prepare(&self, _: MediaInput, _: &str, _: &str, _: &MediaUser, _: &str) -> Result<PreparedMedia> {
        Err(Error::internal("unused"))
    }
    async fn send(&self, _: &MediaResult, _: &str, _: Option<&str>) -> Result<Response> {
        Err(Error::internal("unused"))
    }
    async fn response(&self, _: &str) -> Result<Option<MediaResult>> {
        Ok(None)
    }
    async fn image(&self, _: &Value) -> Result<Option<AnalysisImage>> {
        Ok(None)
    }
}

#[async_trait(?Send)]
impl Resource for Quiet {
    async fn fetch(&self, _: &str, _: ResourceOptions) -> Result<FetchedResource> {
        Err(Error::internal("unused"))
    }
}

#[async_trait(?Send)]
impl Screenshot for Quiet {
    async fn capture(&self, _: &str, _: bool) -> Result<Vec<u8>> {
        Err(Error::internal("unused"))
    }
}

struct Harness {
    api: Api,
    db: Rc<Sqlite>,
    calls: Rc<Calls>,
}

fn vector() -> Value {
    Value::Array((0..768).map(|index| Value::from((index % 5) as f64 / 5.0)).collect())
}

fn env(decisions: bool) -> Vec<(String, String)> {
    let mut pairs = vec![
        ("OFFLINE", "1"),
        ("GEMINI_API_KEY", "test-key"),
        ("GEMINI_BASE_URL", "http://127.0.0.1:9/v1beta"),
        ("AI_CHAT_BASE_URL", "http://127.0.0.1:9/v1beta/openai"),
        ("OWNER_EMAIL", "owner@example.com"),
    ];
    if decisions {
        pairs.push(("OPENAI_API_KEY", "test-openai"));
        pairs.push(("OPENAI_BASE_URL", "http://127.0.0.1:9/v1"));
    }
    pairs.into_iter().map(|(key, value)| (key.to_owned(), value.to_owned())).collect()
}

fn localize(text: &str) -> String {
    text.to_owned()
}

fn random(size: usize) -> Vec<u8> {
    vec![1; size]
}

async fn open(chat_status: u16, decisions: bool) -> Harness {
    forget_vocabulary();
    runtime::set_env(env(decisions).into_iter().collect());
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("enrich-{}-{}", std::process::id(), runtime::random_bytes(3).iter().map(|byte| format!("{byte:02x}")).collect::<String>()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = Rc::new(Sqlite::open(dir.join("library.db"), false).unwrap());
    let store = Rc::new(Store::new(Rc::clone(&db) as Rc<dyn Sql>).unwrap());
    store.db().execute("INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES ('u1', 'acct', '00000002UnE', 0, '{}')", &[]).unwrap();
    let calls = Rc::new(Calls(RefCell::new(Vec::new())));
    let logged = Rc::clone(&calls);
    let api = Api::new(ApiOptions {
        store,
        media: Rc::new(Quiet),
        upload_storage: Rc::new(DirectoryUploads::new(dir.join("uploads"))),
        screenshot: Rc::new(Quiet),
        resource: Rc::new(Quiet),
        fetch: Rc::new(Stub { calls: logged, chat_status }),
        readonly: false,
        localize,
        push: Box::new(|_, _| {}),
        random,
    })
    .await
    .unwrap();
    api.jobs().drain().await;
    Harness { api, db, calls }
}

async fn exchange(api: &Api, method: &str, url: &str, body: &str, email: Option<&str>) -> (u16, String) {
    let mut request = Request::new(method, &format!("http://127.0.0.1{url}")).unwrap();
    request.headers.set("content-type", "application/json");
    if let Some(email) = email {
        request.headers.set("x-mymind-email", email);
    }
    request.body = Body::Bytes(body.as_bytes().to_vec());
    let response = api.handle(&mut request).await.unwrap().unwrap();
    let status = response.status;
    let text = String::from_utf8(response.body.bytes().await.unwrap()).unwrap();
    api.jobs().drain().await;
    (status, text)
}

fn summary_of(db: &Sqlite, id: &str) -> String {
    let row = db.one("SELECT data, extras FROM cards WHERE id = ?", &[id.into()]).unwrap().unwrap();
    json::parse(row.text("data").unwrap()).unwrap().get("summary").to_js_string()
}

fn extras_flag(db: &Sqlite, id: &str, name: &str) -> bool {
    let row = db.one("SELECT extras FROM cards WHERE id = ?", &[id.into()]).unwrap().unwrap();
    json::parse(row.text("extras").unwrap()).unwrap().get(name).truthy()
}

fn embedding_len(db: &Sqlite, id: &str) -> Option<usize> {
    db.one("SELECT vector FROM embeddings WHERE card_id = ?", &[id.into()]).unwrap().and_then(|row| row.blob("vector").map(<[u8]>::len))
}

fn tag_flags(db: &Sqlite, id: &str) -> Vec<(String, i64)> {
    db.query("SELECT name, flags FROM annotations WHERE card_id = ? ORDER BY position", &[id.into()])
        .unwrap()
        .iter()
        .map(|row| (row.text("name").unwrap_or("").to_owned(), row.int("flags").unwrap_or(0)))
        .collect()
}

fn insert_card(db: &Sqlite, id: &str, data: &str, extras: &str) {
    let mut extras_value = json::parse(extras).unwrap();
    if !extras_value.get("id").truthy() {
        extras_value.set("id", id);
    }
    db.execute(
        "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns) VALUES (?, 'u1', 'Note', 0, ?, '', ?, 1, 1, 1)",
        &[id.into(), data.into(), json::to_string(&extras_value).into()],
    )
    .unwrap();
}

fn insert_vector(db: &Sqlite, id: &str, byte: u8) {
    db.execute("INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, 'u1', 768, ?)", &[id.into(), vec![byte; 3072].into()]).unwrap();
}

fn insert_tag(db: &Sqlite, id: &str, name: &str, flags: i64) {
    db.execute("INSERT INTO annotations (id, user_id, card_id, name, flags, position) VALUES (?, 'u1', ?, ?, ?, 0)", &[format!("tag-{id}-{name}").into(), id.into(), name.into(), flags.into()]).unwrap();
}

#[test]
fn new_save_enriches_on_the_worker_path() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, true).await;
        let (status, body) = exchange(&h.api, "POST", "/objects", r#"{"type":"Note","content":"Alpine light over a quiet lake."}"#, None).await;
        assert_eq!(status, 201, "{body}");
        let id = json::parse(&body).unwrap().get("id").to_js_string();
        assert_eq!(summary_of(&h.db, &id), "A generated summary.");
        assert!(tag_flags(&h.db, &id).iter().any(|(_, flags)| *flags == 2));
        assert_eq!(embedding_len(&h.db, &id), Some(3072));
        assert!(h.calls.0.borrow().iter().any(|(_, url, _)| url.contains("/v1/decisions")));
    });
}

#[test]
fn new_save_keeps_chat_tags_without_decisions() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, false).await;
        let (status, body) = exchange(&h.api, "POST", "/objects", r#"{"type":"Note","content":"A note about typography and paper."}"#, None).await;
        assert_eq!(status, 201, "{body}");
        let id = json::parse(&body).unwrap().get("id").to_js_string();
        assert!(tag_flags(&h.db, &id).iter().any(|(_, flags)| *flags == 2));
        assert!(h.calls.0.borrow().iter().all(|(_, url, _)| !url.contains("/v1/decisions")));
    });
}

#[test]
fn hand_written_summary_and_manual_tag_survive_reanalysis() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, false).await;
        insert_card(&h.db, "owned", r#"{"type":"Note","title":"Owned","description":"A paragraph the user already summarized.","summary":"Hand written original"}"#, "{}");
        insert_tag(&h.db, "owned", "kept", 8);
        h.api.jobs().schedule("owned", false);
        h.api.jobs().drain().await;
        assert_eq!(summary_of(&h.db, "owned"), "Hand written original");
        assert!(tag_flags(&h.db, "owned").iter().any(|(name, flags)| name == "kept" && *flags == 8));

        let (status, _) = exchange(&h.api, "PATCH", "/objects/owned", r#"{"summary":"Typed by hand"}"#, None).await;
        assert_eq!(status, 200);
        assert!(extras_flag(&h.db, "owned", "summaryEdited"));
        h.api.jobs().schedule("owned", false);
        h.api.jobs().drain().await;
        assert_eq!(summary_of(&h.db, "owned"), "Typed by hand");
        assert!(tag_flags(&h.db, "owned").iter().any(|(name, flags)| name == "kept" && *flags == 8));
    });
}

#[test]
fn backfill_batches_resumes_and_skips_complete_cards() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, false).await;
        for (id, title) in [("e1", "One"), ("e2", "Two"), ("e3", "Three")] {
            insert_card(&h.db, id, &format!(r#"{{"type":"Note","title":"{title}","description":"Body of {title} for the embedding.","summary":"Already"}}"#), r#"{"summarySource":"ai"}"#);
            insert_tag(&h.db, id, "filed", 2);
        }
        insert_card(&h.db, "done", r#"{"type":"Note","title":"Done","description":"Complete card.","summary":"Done summary"}"#, r#"{"summarySource":"ai"}"#);
        insert_tag(&h.db, "done", "filed", 2);
        insert_vector(&h.db, "done", 7);
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill?limit=2", "", Some("owner@example.com")).await;
        assert_eq!(status, 200, "{body}");
        let report = json::parse(&body).unwrap();
        assert_eq!(report.get("embedded").as_f64(), Some(2.0));
        assert_eq!(report.get("remaining").as_f64(), Some(1.0));
        let batches: Vec<usize> = h.calls.0.borrow().iter().filter(|(_, url, _)| url.contains(":batchEmbedContents")).map(|(_, _, body)| json::parse(body).unwrap().get("requests").as_array().map_or(0, Vec::len)).collect();
        assert_eq!(batches, vec![2]);
        assert!(h.calls.0.borrow().iter().all(|(_, url, _)| !url.contains("/chat/completions")));
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill?limit=2", "", Some("owner@example.com")).await;
        assert_eq!(status, 200, "{body}");
        assert_eq!(json::parse(&body).unwrap().get("remaining").as_f64(), Some(0.0));
        let before = h.calls.0.borrow().len();
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill", "", Some("owner@example.com")).await;
        assert_eq!(status, 200, "{body}");
        assert_eq!(json::parse(&body).unwrap().get("remaining").as_f64(), Some(0.0));
        assert_eq!(h.calls.0.borrow().len(), before);
        assert_eq!(embedding_len(&h.db, "done"), Some(3072));
        let kept = h.db.one("SELECT vector FROM embeddings WHERE card_id = 'done'", &[]).unwrap().unwrap().blob("vector").unwrap().to_vec();
        assert!(kept.iter().all(|byte| *byte == 7));
    });
}

#[test]
fn backfill_does_not_clobber_a_hand_summary_or_vector() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, false).await;
        insert_card(&h.db, "hand", r#"{"type":"Note","title":"Hand","description":"A card with a summary the user wrote and no tags.","summary":"Hand written"}"#, "{}");
        insert_tag(&h.db, "hand", "kept", 8);
        insert_vector(&h.db, "hand", 9);
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill?limit=4", "", Some("owner@example.com")).await;
        assert_eq!(status, 200, "{body}");
        assert_eq!(summary_of(&h.db, "hand"), "Hand written");
        assert!(tag_flags(&h.db, "hand").iter().any(|(name, flags)| name == "kept" && *flags == 8));
        assert!(tag_flags(&h.db, "hand").iter().any(|(_, flags)| *flags == 2));
        let vector = h.db.one("SELECT vector FROM embeddings WHERE card_id = 'hand'", &[]).unwrap().unwrap().blob("vector").unwrap().to_vec();
        assert!(vector.iter().all(|byte| *byte == 9));
        assert!(extras_flag(&h.db, "hand", "aiTagsDone"));
    });
}

#[test]
fn backfill_stops_on_429_and_keeps_earlier_cards() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(429, false).await;
        insert_card(&h.db, "a-embed", r#"{"type":"Note","title":"Embedded","description":"Only a vector is missing.","summary":"Present"}"#, r#"{"summarySource":"ai"}"#);
        insert_tag(&h.db, "a-embed", "filed", 2);
        insert_card(&h.db, "b-chat", r#"{"type":"Note","title":"Needs words","description":"This card still needs a summary."}"#, "{}");
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill?limit=20", "", Some("owner@example.com")).await;
        assert_eq!(status, 429, "{body}");
        let report = json::parse(&body).unwrap();
        assert_eq!(report.get("retryAfterMs").as_f64(), Some(0.0));
        assert_eq!(embedding_len(&h.db, "a-embed"), Some(3072));
        assert_ne!(summary_of(&h.db, "b-chat"), "A generated summary.");
        assert_eq!(embedding_len(&h.db, "b-chat"), None);
    });
}

#[test]
fn backfill_is_owner_only() {
    let _guard = LOCK.lock().unwrap();
    mymind::runtime::run(async {
        let h = open(200, false).await;
        insert_card(&h.db, "n1", r#"{"type":"Note","title":"One","description":"Words enough to enrich."}"#, "{}");
        assert_eq!(exchange(&h.api, "GET", "/ai/backfill", "", None).await.0, 403);
        assert_eq!(exchange(&h.api, "GET", "/ai/backfill", "", Some("other@example.com")).await.0, 403);
        let (status, body) = exchange(&h.api, "GET", "/ai/backfill", "", Some("Owner@Example.com")).await;
        assert_eq!(status, 200, "{body}");
        let estimate = json::parse(&body).unwrap();
        assert_eq!(estimate.get("cards").as_f64(), Some(1.0));
        assert!(estimate.get("estimatedUsd").as_f64().unwrap() > 0.0);
        let (status, body) = exchange(&h.api, "POST", "/ai/backfill?limit=1", "", Some("owner@example.com")).await;
        assert_eq!(status, 200, "{body}");
        assert_eq!(json::parse(&body).unwrap().get("processed").as_f64(), Some(1.0));
        assert_eq!(exchange(&h.api, "POST", "/ai/backfill?limit=0", "", Some("owner@example.com")).await.0, 400);
    });
}

#[test]
fn command_reads_dotenv_and_resumes() {
    let _guard = LOCK.lock().unwrap();
    let root = tmproot("enrich-cli");
    let db = root.join("library.db");
    let sql = Rc::new(Sqlite::open(&db, false).unwrap());
    Store::new(Rc::clone(&sql) as Rc<dyn Sql>).unwrap();
    sql.execute("INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES ('u1', 'acct', '00000002UnE', 0, '{}')", &[]).unwrap();
    for (id, title) in [("c1", "First"), ("c2", "Second")] {
        insert_card(&sql, id, &format!(r#"{{"type":"Note","title":"{title}","description":"Paragraph about {title}."}}"#), "{}");
    }
    drop(sql);
    std::fs::write(root.join("fixture-vars"), "GEMINI_API_KEY=from-dotenv\n").unwrap();
    let fake = ai_fake(false);
    let url = fake.url();
    let chat = format!("{url}/v1beta/openai");
    let env = [("OFFLINE", "1"), ("GEMINI_BASE_URL", url.as_str()), ("AI_CHAT_BASE_URL", chat.as_str())];
    let dry = run_rust(&root, "enrich-library", &["--db", "library.db"], &env, None);
    assert_eq!(dry.status, 0, "{}", dry.stderr);
    assert!(dry.stdout.contains("Cards to enrich: 2"), "{}", dry.stdout);
    assert!(fake.log().is_empty());
    let once = run_rust(&root, "enrich-library", &["--db", "library.db", "--limit", "1", "--run"], &env, None);
    assert_eq!(once.status, 0, "{}\n{}", once.stderr, once.stdout);
    assert!(once.stdout.contains("remaining 1"), "{}", once.stdout);
    assert!(!fake.log().is_empty());
    let twice = run_rust(&root, "enrich-library", &["--db", "library.db", "--limit", "1", "--run"], &env, None);
    assert_eq!(twice.status, 0, "{}", twice.stderr);
    assert!(twice.stdout.contains("remaining 0"), "{}", twice.stdout);
}

#[test]
fn command_rejects_a_hosted_url_outside_the_allowlist() {
    let root = tmproot("enrich-host-reject");
    let out = run_rust(&root, "enrich-library", &["--hosted", "https://evil.example", "--cookie", "abc"], &[], None);
    assert_eq!(out.status, 1);
    assert!(out.stderr.contains("only talks to"), "{}", out.stderr);
}

#[test]
fn command_runs_the_hosted_backfill_with_the_session_cookie() {
    let root = tmproot("enrich-hosted");
    let mut posts = 0;
    let fake = tools_common::Fake::start(move || {
        Box::new(move |request: &tools_common::Req| {
            assert_eq!(request.header("cookie"), Some("mm_session=abc"));
            if request.method == "GET" {
                return tools_common::Resp::json(200, r#"{"cards":1,"missingSummary":1,"missingTags":1,"missingEmbedding":1,"skippedComplete":0,"skippedEmpty":0,"chatInputTokens":300,"chatOutputTokens":150,"embedTokens":40,"estimatedUsd":0.000098,"decisions":false}"#);
            }
            posts += 1;
            if posts == 1 {
                return tools_common::Resp::json(429, r#"{"processed":0,"summarized":0,"tagged":0,"embedded":0,"skipped":0,"failed":0,"remaining":1,"retryAfterMs":0,"error":null}"#);
            }
            tools_common::Resp::json(200, r#"{"processed":1,"summarized":1,"tagged":1,"embedded":1,"skipped":0,"failed":0,"remaining":0,"retryAfterMs":null,"error":null}"#)
        })
    });
    let out = run_rust(&root, "enrich-library", &["--hosted", &fake.url(), "--cookie", "abc", "--run"], &[], None);
    assert_eq!(out.status, 0, "{}\n{}", out.stderr, out.stdout);
    assert!(out.stdout.contains("Cards to enrich: 1"), "{}", out.stdout);
    assert!(out.stdout.contains("Rate limited"), "{}", out.stdout);
    assert!(out.stdout.contains("remaining 0"), "{}", out.stdout);
    assert!(!out.stdout.contains("abc"), "{}", out.stdout);
}
