//! Synthetic end-to-end contracts for screenshot covers, Content provenance and Markdown exports.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::api::{Api, ApiOptions};
use mm_core::auth::{Auth, AuthOptions, SigninPages};
use mm_core::auth_store::install_auth_schema;
use mm_core::host::{AnalysisImage, Fetch, FetchedResource, Media, MediaInput, MediaResult, MediaUser, PreparedMedia, Resource, ResourceOptions, Screenshot, UploadStorage};
use mm_core::http::{Body, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::site::LibraryApi;
use mm_core::sql::{Sql, SqlValue};
use mm_core::store::Store;
use mm_core::{Result, Value, obj};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

struct TestRuntime;
impl Runtime for TestRuntime {
    fn spawn(&self, _task: Task) {}
    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }
    fn log(&self, _line: &str) {}
}

#[derive(Default)]
struct Memory(RefCell<HashMap<String, Vec<u8>>>);
#[async_trait(?Send)]
impl UploadStorage for Memory {
    fn key(&self) -> String {
        format!("backend-gaps-{:p}", self)
    }
    async fn names(&self) -> Result<Vec<String>> {
        Ok(self.0.borrow().keys().cloned().collect())
    }
    async fn size(&self, id: &str) -> Result<Option<u64>> {
        Ok(self.0.borrow().get(id).map(|b| b.len() as u64))
    }
    async fn truncate(&self, id: &str, size: u64) -> Result<()> {
        if let Some(bytes) = self.0.borrow_mut().get_mut(id) {
            bytes.truncate(size as usize);
        }
        Ok(())
    }
    async fn remove(&self, id: &str) -> Result<()> {
        self.0.borrow_mut().remove(id);
        Ok(())
    }
    fn input(&self, id: &str) -> MediaInput {
        MediaInput::Key(id.to_owned())
    }
    async fn write(&self, id: &str, start: u64, chunks: Body, _length: Option<u64>) -> Result<()> {
        let bytes = chunks.bytes().await?;
        let mut all = self.0.borrow_mut();
        let target = all.entry(id.to_owned()).or_default();
        target.truncate(start as usize);
        target.extend(bytes);
        Ok(())
    }
}

struct Host;
#[async_trait(?Send)]
impl Media for Host {
    fn cover(&self, blob: &Value) -> Result<Value> {
        mm_core::media_files::cover(blob, 800.0, 1000.0)
    }
    async fn prepare(&self, input: MediaInput, content_type: &str, _name: &str, _user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let name = if matches!(input, MediaInput::Bytes(bytes) if bytes == b"alt") { "different" } else { "original" };
        let blob = obj! { "path" => format!("synthetic/{id}/{name}.png"), "width" => 1600, "height" => 900, "dominantColor" => "#123456", "type" => content_type };
        Ok(PreparedMedia { data: obj! { "type" => if content_type.starts_with("image/") { "Image" } else { "File" }, "blob" => blob }.into(), extras: obj! {}.into() })
    }
    async fn send(&self, _result: &MediaResult, _method: &str, _range: Option<&str>) -> Result<Response> {
        unreachable!()
    }
    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        Ok(Some(MediaResult { key: relative.to_owned(), size: 3, content_type: "image/png".into() }))
    }
    async fn image(&self, _source: &Value) -> Result<Option<AnalysisImage>> {
        Ok(None)
    }
}
#[async_trait(?Send)]
impl Fetch for Host {
    async fn fetch(&self, _request: Request) -> Result<Response> {
        panic!("no network in synthetic tests")
    }
}
#[async_trait(?Send)]
impl Resource for Host {
    async fn fetch(&self, _url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        panic!("no network in synthetic tests")
    }
}
#[async_trait(?Send)]
impl Screenshot for Host {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        panic!("no browser in synthetic tests")
    }
}
#[async_trait(?Send)]
impl SigninPages for Host {
    async fn page(&self, _name: &str) -> Result<String> {
        Ok("sign in".into())
    }
}

// Construct an empty synthetic library with no filesystem or external services.
fn setup() -> (Rc<Store>, Api) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(HashMap::from([("OFFLINE".into(), "1".into())]));
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute("INSERT INTO users (id, account_uid, id_prefix, settings) VALUES ('test', 'synthetic', '00000000000', '{}')", &[]).unwrap();
    let api = block_on(Api::new(ApiOptions {
        store: store.clone(),
        media: Rc::new(Host),
        upload_storage: Rc::new(Memory::default()),
        screenshot: Rc::new(Host),
        resource: Rc::new(Host),
        fetch: Rc::new(Host),
        readonly: false,
        localize: str::to_owned,
        push: Box::new(|_, _| {}),
        has_replay: Box::new(|_| Ok(false)),
        random: runtime::random_bytes,
    }))
    .unwrap();
    (store, api)
}

// Send one request directly through the shared core API.
fn send(api: &Api, method: &str, path: &str, body: &str, headers: &[(&str, &str)]) -> Response {
    let mut req = Request::new(method, &format!("http://localhost:3000{path}")).unwrap();
    req.body = Body::Bytes(body.as_bytes().to_vec());
    for (name, value) in headers {
        req.headers.set(*name, *value);
    }
    block_on(api.handle(&mut req)).unwrap().expect("core route must handle request")
}

// Read an API response as UTF-8.
fn text(response: Response) -> String {
    String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()
}

// Create a synthetic card using the public write route.
fn create(api: &Api, body: Value) -> String {
    let response = send(api, "POST", "/objects", &json::to_string(&body), &[("content-type", "application/json")]);
    assert_eq!(response.status, 201);
    json::parse(&text(response)).unwrap().get("id").as_str().unwrap().to_owned()
}

#[test]
fn screenshot_attachment_sets_cover_and_retains_upload_source() {
    let (store, api) = setup();
    let id = create(&api, obj! { "content" => "https://example.test/page" }.into());
    let before = store.card(&id, None).unwrap().unwrap();
    let response = send(&api, "POST", &format!("/objects/{id}/attachments?type=screenshot"), "png", &[("content-type", "image/png")]);
    assert_eq!(response.status, 200);
    let attachment = json::parse(&text(response)).unwrap();
    let card = store.card(&id, None).unwrap().unwrap();
    let extras = json::parse(&card.extras).unwrap();
    let cover = extras.get("cover");
    assert_eq!(cover.get("src").as_str(), Some(format!("/media/synthetic/{id}/original.png;800x450-t.webp").as_str()));
    assert_eq!(cover.get("width").as_f64(), Some(800.0));
    assert_eq!(cover.get("height").as_f64(), Some(450.0));
    assert!(card.html.contains(cover.get("src").as_str().unwrap()));
    assert!(!card.html.contains("processing-guts"));
    assert!(!extras.get("capturePending").truthy());
    assert_eq!(card.kind, "WebPage");
    let object = json::parse(&text(send(&api, "GET", &format!("/objects/{id}"), "", &[]))).unwrap();
    assert_eq!(object.get("mainEntity").get("cover"), cover);
    assert!(card.modified_ns > before.modified_ns);
    assert_eq!(json::parse(&card.data).unwrap().get("source"), json::parse(&before.data).unwrap().get("source"));
    assert_eq!(extras.get("attachments").at(0).get("path"), attachment.get("path"));
    assert!(json::parse(&card.data).unwrap().get("blob").is_undefined());

    let ordinary = send(&api, "POST", &format!("/objects/{id}/attachments"), "alt", &[("content-type", "image/png")]);
    assert_eq!(ordinary.status, 200);
    let card = store.card(&id, None).unwrap().unwrap();
    assert_eq!(json::parse(&card.extras).unwrap().get("cover"), cover);

    let authorization = json::parse(&text(send(&api, "POST", "/uploads/authorize", "", &[]))).unwrap();
    let upload_path = authorization.get("url").as_str().unwrap();
    let upload_id = upload_path.rsplit('/').next().unwrap();
    let bearer = format!("Bearer {}", authorization.get("token").as_str().unwrap());
    let response = send(&api, "PUT", upload_path, "png", &[("content-type", "image/png"), ("content-range", "bytes 0-2/3"), ("authorization", &bearer)]);
    assert_eq!(response.status, 200);
    for _ in 0..2 {
        let body = json::to_string(&obj! { "uploadId" => upload_id }.into());
        let response = send(&api, "POST", &format!("/objects/{id}/attachments?type=screenshot"), &body, &[("content-type", "application/json")]);
        assert_eq!(response.status, 200, "claimed upload remains available through attachments");
    }
    let rejected = send(&api, "POST", &format!("/objects/{id}/attachments?type=screenshot"), "pdf", &[("content-type", "application/pdf")]);
    assert_eq!(rejected.status, 415);
}

#[test]
fn content_create_preserves_recorded_source_shape() {
    let (store, api) = setup();
    let id = create(&api, obj! { "type" => "Content", "content" => "Selected words", "source" => "https://example.test/page?q=a#section" }.into());
    let row = store.card(&id, None).unwrap().unwrap();
    let data = json::parse(&row.data).unwrap();
    assert_eq!(data.get("source"), &Value::from(obj! { "url" => "https://example.test/page?q=a#section" }));
    assert_eq!(data.get("type").as_str(), Some("Content"));
    assert_eq!(data.get("prose").get("content").at(0).get("content").at(0).get("text").as_str(), Some("Selected words"));
    assert!(row.html.contains("Selected words"));
    let note = create(&api, obj! { "type" => "Note", "content" => "Ordinary note", "source" => "clipboard" }.into());
    assert!(json::parse(&store.card(&note, None).unwrap().unwrap().data).unwrap().get("source").is_undefined());
    let url_text = create(&api, obj! { "type" => "Content", "content" => "https://selected.test", "source" => "https://page.test" }.into());
    assert_eq!(store.card(&url_text, None).unwrap().unwrap().kind, "Content");
    let no_source = create(&api, obj! { "type" => "Content", "content" => "Without source" }.into());
    assert!(json::parse(&store.card(&no_source, None).unwrap().unwrap().data).unwrap().get("source").is_undefined());
}

// Make real synthetic session and app-token rows to test the existing auth gate.
fn auth() -> Auth {
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    install_auth_schema(&*db).unwrap();
    let now = runtime::now_ms();
    for (table, token) in [("sessions", "test-session"), ("app_tokens", "test-app-token")] {
        let hash = base64::hex(&Sha256::digest(token.as_bytes()));
        let extra = if table == "app_tokens" { ", kind, device" } else { "" };
        let values = if table == "app_tokens" { ", 'google', '{}'" } else { "" };
        db.execute(
            &format!("INSERT INTO {table} (hash, sub, email, created_ms, expires_ms, used_ms{extra}) VALUES (?, 'test', 'test@example.test', ?, ?, ?{values})"),
            &[SqlValue::from(hash), SqlValue::from(now), SqlValue::from(now + 86_400_000.0), SqlValue::from(now)],
        )
        .unwrap();
    }
    let env = HashMap::from([
        ("AUTH".into(), "google".into()),
        ("GOOGLE_CLIENT_ID".into(), "synthetic".into()),
        ("GOOGLE_CLIENT_SECRET".into(), "synthetic".into()),
        ("OWNER_EMAIL".into(), "test@example.test".into()),
    ]);
    Auth::new(AuthOptions::new(env, 3000, db, Rc::new(Host), Rc::new(Host))).unwrap().unwrap()
}

#[test]
fn markdown_route_works_for_cookie_extension_and_app_sessions() {
    let (_store, api) = setup();
    let id = create(&api, obj! { "type" => "Note", "content" => "Hello\nworld" }.into());
    let auth = auth();
    for (header, value) in [("cookie", "mm_session=test-session"), ("authorization", "Bearer test-session"), ("authorization", "Bearer test-app-token")] {
        let mut req = Request::new("GET", &format!("http://localhost:3000/cards/{id}/markdown")).unwrap();
        req.headers.set("host", "localhost:3000");
        req.headers.set(header, value);
        req.headers.set("accept", "text/markdown");
        assert!(block_on(auth.handle(&mut req)).unwrap().is_none());
        assert!(auth.allows(&req).unwrap());
        let response = block_on(api.handle(&mut req)).unwrap().expect("markdown route");
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("content-type"), Some("text/markdown; charset=utf-8"));
        assert_eq!(text(response), "Hello\n\nworld\n");
    }
    let missing = send(&api, "GET", "/cards/missing/markdown", "", &[]);
    assert_eq!(missing.status, 404);
    let req = Request::new("GET", &format!("http://localhost:3000/cards/{id}/markdown")).unwrap();
    assert!(!auth.allows(&req).unwrap());
}

#[test]
fn markdown_preserves_formatting_nested_lists_and_literal_syntax() {
    let (_store, api) = setup();
    let prose = json::parse(
        r#"{"type":"doc","content":[
        {"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Heading"}]},
        {"type":"paragraph","content":[
            {"type":"text","text":"bold","marks":[{"type":"bold"}]},
            {"type":"text","text":" and "},
            {"type":"text","text":"italic","marks":[{"type":"italic"}]},
            {"type":"hardBreak"},
            {"type":"text","text":"[literal] *text* <tag>"}
        ]},
        {"type":"paragraph","content":[{"type":"text","text":"link","marks":[{"type":"link","attrs":{"href":"https://example.test/a%20b(c)"}}]}]},
        {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"quote"}]}]},
        {"type":"orderedList","attrs":{"start":3},"content":[{"type":"listItem","content":[
            {"type":"paragraph","content":[{"type":"text","text":"first"}]},
            {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"nested"}]}]}]}
        ]}]},
        {"type":"taskList","content":[{"type":"taskItem","attrs":{"checked":true},"content":[{"type":"paragraph","content":[{"type":"text","text":"done"}]}]}]},
        {"type":"paragraph","content":[{"type":"text","text":"a`b","marks":[{"type":"code"}]}]},
        {"type":"codeBlock","attrs":{"language":"js"},"content":[{"type":"text","text":"const ticks = `a`;\n```"}]},
        {"type":"horizontalRule"},
        {"type":"image","attrs":{"src":"/media/synthetic.png","alt":"An image"}}
    ]}"#,
    )
    .unwrap();
    let id = create(&api, obj! { "type" => "Note", "prose" => prose }.into());
    let response = send(&api, "GET", &format!("/cards/{id}/markdown"), "", &[]);
    assert_eq!(
        text(response),
        concat!(
            "## Heading\n\n**bold** and *italic*  \n\\[literal\\] \\*text\\* \\<tag>\n\n",
            "[link](<https://example.test/a%20b(c)>)\n\n> quote\n\n",
            "3. first\n   \n   - nested\n\n- [x] done\n\n``a`b``\n\n",
            "````js\nconst ticks = `a`;\n```\n````\n\n---\n\n![An image](</media/synthetic.png>)\n"
        )
    );
}

#[test]
fn markdown_uses_current_editor_document_and_does_not_truncate() {
    let (store, api) = setup();
    let id = create(&api, obj! { "type" => "Content", "content" => "stale card text", "source" => "https://example.test" }.into());
    let full = "all the text ".repeat(200);
    let prose = obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => full.as_str() })] })] };
    store.db().execute("INSERT INTO card_content (card_id, body) VALUES (?, ?)", &[id.as_str().into(), json::to_string(&prose.into()).into()]).unwrap();
    let response = send(&api, "GET", &format!("/cards/{id}/markdown"), "", &[]);
    assert_eq!(text(response), format!("{full}\n"));
    assert_eq!(
        json::parse(&store.card(&id, None).unwrap().unwrap().data).unwrap().get("prose").get("content").at(0).get("content").at(0).get("text").as_str(),
        Some("stale card text")
    );
}

#[test]
fn markdown_image_empty_and_readonly_exports() {
    let (store, api) = setup();
    let id = create(&api, obj! { "type" => "Note", "content" => "" }.into());
    assert_eq!(text(send(&api, "GET", &format!("/cards/{id}/markdown"), "", &[])), "");
    let image = obj! { "type" => "Image", "title" => "A [picture]", "blob" => obj! { "path" => "synthetic/image.png" } };
    store.db().execute("UPDATE cards SET data = ?, type = 'Image' WHERE id = ?", &[json::to_string(&image.into()).into(), id.as_str().into()]).unwrap();
    let readonly = block_on(Api::new(ApiOptions {
        store,
        media: Rc::new(Host),
        upload_storage: Rc::new(Memory::default()),
        screenshot: Rc::new(Host),
        resource: Rc::new(Host),
        fetch: Rc::new(Host),
        readonly: true,
        localize: str::to_owned,
        push: Box::new(|_, _| {}),
        has_replay: Box::new(|_| Ok(false)),
        random: runtime::random_bytes,
    }))
    .unwrap();
    assert_eq!(text(send(&readonly, "GET", &format!("/cards/{id}/markdown"), "", &[])), "![A \\[picture\\]](</media/synthetic/image.png>)\n");
}

#[test]
fn markdown_preserves_tables_and_escapes_literal_lists() {
    let (_store, api) = setup();
    let prose = json::parse(
        r#"{"type":"doc","content":[
        {"type":"paragraph","content":[{"type":"text","text":"1. literal\n- not a list\n&amp;"}]},
        {"type":"table","content":[
            {"type":"tableRow","content":[
                {"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"A|B"}]}]},
                {"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"C"}]}]}
            ]},
            {"type":"tableRow","content":[
                {"type":"tableCell","content":[{"type":"paragraph","content":[{"type":"text","text":"value"}]}]},
                {"type":"tableCell","content":[{"type":"paragraph","content":[{"type":"text","text":"two"}]}]}
            ]}
        ]}
    ]}"#,
    )
    .unwrap();
    let id = create(&api, obj! { "type" => "Note", "prose" => prose }.into());
    assert_eq!(text(send(&api, "GET", &format!("/cards/{id}/markdown"), "", &[])), "1\\. literal\n\\- not a list\n&amp;\n\n| A\\|B | C |\n| --- | --- |\n| value | two |\n");
}

#[test]
fn markdown_leaves_ordinary_punctuation_alone() {
    let (_store, api) = setup();
    let plain = "Meet at 3.30 - bring snake_case notes & a+b (maybe!) #1 <3";
    let id = create(&api, obj! { "type" => "Note", "content" => plain }.into());
    assert_eq!(text(send(&api, "GET", &format!("/cards/{id}/markdown"), "", &[])), format!("{plain}\n"));
}
