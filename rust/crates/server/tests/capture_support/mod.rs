//! Isolated synthetic services for the capture regression suite.

#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use mm_core::api::{Api, ApiOptions};
use mm_core::enrichment::{Enrichment, EnrichmentOptions};
use mm_core::host::{Fetch, Screenshot};
use mm_core::http::{Headers, Request, Response};
use mm_core::hub::{Hub, Socket};
use mm_core::site::LibraryApi;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Error, Result, Value, obj, runtime};
use mm_js::json;
use mm_sqlite::Sqlite;
use mymind::capture_http::CaptureTransport;
use mymind::fetch::NativeFetch;
use mymind::http::{Handler, serve};
use mymind::local_media::LocalMedia;
use mymind::tool_capture_fixtures::{Fixtures, Media, fixture_media};
use mymind::uploads_fs::DirectoryUploads;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// A process guard exposing only Cargo scratch storage through a short parent-owned directory fd.
/// Hold the target's shared test mutex, construct before runtime threads start, and retain it until they stop.
pub struct ScratchTemp {
    _directory: std::fs::File,
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    environment: PathBuf,
}

impl ScratchTemp {
    /// Give child browsers a short stable directory path that resolves entirely inside Cargo scratch storage.
    ///
    /// # Returns
    /// A process-scoped guard holding the directory descriptor open and restoring the environment on drop.
    /// Call only while holding the target's shared test mutex, before starting the runtime.
    ///
    /// # Panics
    /// If Cargo's scratch directory cannot be created or opened.
    pub fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        std::fs::create_dir_all(&root).unwrap();
        let directory = std::fs::File::open(&root).unwrap();
        let environment = root.join(format!(
            "browser-env-{}",
            mm_js::base64::hex(&runtime::random_bytes(4))
        ));
        let previous = ["TMPDIR", "HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME"]
            .map(|name| (name, std::env::var_os(name)))
            .to_vec();
        let short = format!("/proc/{}/fd/{}", std::process::id(), directory.as_raw_fd());
        let mut variables = vec![("TMPDIR", PathBuf::from(short))];
        for (name, child) in [
            ("HOME", "home"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_CACHE_HOME", "cache"),
        ] {
            let path = environment.join(child);
            std::fs::create_dir_all(&path).unwrap();
            variables.push((name, path));
        }
        // Each target serializes its tests, and this guard is installed before any runtime threads start.
        for (name, path) in variables {
            unsafe { std::env::set_var(name, path) };
        }
        Self {
            _directory: directory,
            previous,
            environment,
        }
    }
}

impl Default for ScratchTemp {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScratchTemp {
    fn drop(&mut self) {
        // The serialized test has shut its runtime down before restoring its original process setting.
        unsafe {
            for (name, previous) in &self.previous {
                if let Some(previous) = previous {
                    std::env::set_var(name, previous);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&self.environment);
    }
}

/// A persisted synthetic card read independently from the API serializer; all fields retain stored bytes/values.
pub struct Saved {
    /// Parsed `cards.data` JSON.
    pub data: Value,
    /// Parsed `cards.extras` JSON.
    pub extras: Value,
    /// Exact persisted `cards.html` text.
    pub html: String,
}

/// A loopback-only screenshot double returning fixed PNG bytes or the original synthetic screenshot failure.
pub struct Shot {
    /// Complete synthetic image returned on success.
    pub png: Vec<u8>,
    /// Number of calls accepted by the double.
    pub calls: Cell<usize>,
    /// Return the fixture failure after incrementing the call count.
    pub fail: bool,
}

#[async_trait(?Send)]
impl Screenshot for Shot {
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        assert!(offline);
        assert_eq!(
            mm_js::url::parse(url, None).unwrap().host_str(),
            Some("127.0.0.1")
        );
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(Error::internal("Fixture download has no screenshot"))
        } else {
            Ok(self.png.clone())
        }
    }
}

/// Native HTTP for hosted API fixtures, refusing requests outside one assigned loopback origin before sending.
pub struct LoopbackFetch {
    /// Exact fixture origin; request origins must match byte for byte.
    pub origin: String,
    native: NativeFetch,
}

#[async_trait(?Send)]
impl Fetch for LoopbackFetch {
    async fn fetch(&self, request: Request) -> Result<Response> {
        assert_eq!(
            mm_js::url::origin(&request.url),
            self.origin,
            "test outbound fetch left loopback fixture"
        );
        panic!(
            "AI fixture calls must request fetch_unfollowed; following provider POST redirects is forbidden"
        )
    }

    async fn fetch_unfollowed(&self, request: Request) -> Result<Response> {
        assert_eq!(
            mm_js::url::origin(&request.url),
            self.origin,
            "test outbound fetch left loopback fixture"
        );
        self.native.fetch_unfollowed(request).await
    }
}

#[derive(Default)]
/// A memory-backed raw socket for the real WebSocket encoder; writes preserve complete frame bytes and order.
pub struct Wire {
    bytes: RefCell<Vec<Vec<u8>>>,
    closed: Cell<bool>,
}

impl Socket for Wire {
    fn write(&self, bytes: &[u8]) {
        self.bytes.borrow_mut().push(bytes.to_vec());
    }
    fn end(&self) {
        self.closed.set(true);
    }
    fn destroy(&self) {
        self.closed.set(true);
    }
    fn is_destroyed(&self) -> bool {
        self.closed.get()
    }
}

impl Wire {
    /// Decode invocations from the real hub's raw WebSocket output.
    ///
    /// # Returns
    /// Every type-1 SignalR invocation, in write order.
    pub fn messages(&self) -> Vec<Value> {
        self.bytes
            .borrow()
            .iter()
            .filter(|bytes| !bytes.starts_with(b"HTTP/1.1"))
            .flat_map(|bytes| {
                let offset = match bytes[1] & 127 {
                    126 => 4,
                    127 => 10,
                    _ => 2,
                };
                String::from_utf8(bytes[offset..].to_vec())
                    .unwrap()
                    .split('\x1e')
                    .filter(|frame| !frame.is_empty())
                    .map(|frame| json::parse(frame).unwrap())
                    .filter(|frame| frame.get("type").as_f64() == Some(1.0))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

struct ApiHandler(Rc<Api>);

#[async_trait(?Send)]
impl Handler for ApiHandler {
    async fn handle(&self, mut request: Request) -> Response {
        match self.0.handle(&mut request).await {
            Ok(Some(response)) => response,
            Ok(None) => Response::send(404, "missing test route", &Headers::new()),
            Err(error) => Response::send(500, error.to_js_string(), &Headers::new()),
        }
    }
}

/// A fully collected synthetic HTTP response, preserving status, headers and exact body bytes.
pub struct Answer {
    /// HTTP status code.
    pub status: u16,
    /// Response headers as received by the loopback client.
    pub headers: Headers,
    /// Complete unmodified response bytes.
    pub bytes: Vec<u8>,
}

impl Answer {
    /// Decode a synthetic JSON response.
    ///
    /// # Returns
    /// The complete response body as JSON; panics for a non-JSON response.
    pub fn value(&self) -> Value {
        json::parse(std::str::from_utf8(&self.bytes).unwrap()).unwrap()
    }

    /// Decode a textual synthetic response.
    ///
    /// # Returns
    /// Its complete UTF-8 text; panics for invalid UTF-8.
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap()
    }
}

/// An isolated synthetic library with real native HTTP, capture transport, media processing and provider fixtures.
/// Call within the native LocalSet; finish dependent enrichment queues before dropping it.
pub struct Harness {
    /// Private per-run directory under Cargo's scratch storage.
    pub dir: PathBuf,
    /// Loopback HTTP/provider fixture server and its request log.
    pub fixtures: Fixtures,
    /// Generated PNG, WebP, PDF and MP4 fixture bytes.
    pub media_bytes: Media,
    /// Synthetic persisted library, containing no owner rows.
    pub store: Rc<Store>,
    /// Native media storage rooted in the per-run scratch directory.
    pub media: Rc<LocalMedia>,
    /// Policy-checking native capture transport.
    pub resource: Rc<CaptureTransport>,
    /// Provider HTTP restricted to the assigned fixture origin.
    pub fetch: Rc<LoopbackFetch>,
    /// Default successful screenshot double.
    pub shot: Rc<Shot>,
    /// Independent transaction writer used for race and enrichment assertions.
    pub writer: Rc<Writer>,
    /// API instance served through the real HTTP listener.
    pub api: Rc<Api>,
    /// Complete real hub frames, available as decoded SignalR invocations.
    pub wire: Rc<Wire>,
    /// Assigned native API loopback origin.
    pub base: String,
    /// Persisted row snapshots taken synchronously for each committed card event.
    pub snapshots: Rc<RefCell<Vec<(String, String, Saved)>>>,
    task: JoinHandle<()>,
    client: reqwest::Client,
}

impl Harness {
    /// Build an empty library and real loopback API listener with synthetic media and hosted API fixtures.
    ///
    /// # Arguments
    /// * `name` - Unique scratch-directory suffix in this process.
    /// * `readonly` - Whether the real API acknowledges writes without executing them.
    ///
    /// # Returns
    /// The running isolated test services, stopped when dropped.
    ///
    /// # Panics
    /// If scratch storage, synthetic media generation, database initialization or either loopback listener fails.
    pub async fn new(name: &str, readonly: bool) -> Self {
        mm_core::ai::forget_vocabulary();
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stage-b-capture-{}-{name}-{}",
            std::process::id(),
            mm_js::base64::hex(&runtime::random_bytes(4))
        ));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!dir.join("absent.env").exists());
        let media_bytes = fixture_media(&dir).unwrap();
        let fixtures = Fixtures::start(media_bytes.clone()).unwrap();
        let store = Rc::new(
            Store::new(Rc::new(
                Sqlite::open(dir.join("capture.db"), false).unwrap(),
            ))
            .unwrap(),
        );
        store.db().execute("INSERT INTO users (id, account_uid, id_prefix, settings) VALUES ('test', 'synthetic', '00000000000', '{}')", &[]).unwrap();
        let media = Rc::new(LocalMedia::new(dir.join("media")));
        let resource = Rc::new(CaptureTransport::new());
        let fetch = Rc::new(LoopbackFetch {
            origin: fixtures.base.clone(),
            native: NativeFetch::new(),
        });
        let shot = Rc::new(Shot {
            png: media_bytes.png.clone(),
            calls: Cell::new(0),
            fail: false,
        });
        let hub = Hub::new(Rc::new(|_| Ok(true)));
        let wire = Rc::new(Wire::default());
        let mut request = Request::new("GET", "http://127.0.0.1/hub").unwrap();
        request
            .headers
            .set("sec-websocket-key", "bG9jYWwtZml4dHVyZQ==");
        let connection = hub.upgrade(&request, wire.clone(), &[]).unwrap();
        let handshake = b"{\"protocol\":\"json\",\"version\":1}\x1e";
        connection.data(
            &[
                vec![0x81, 0x80 | handshake.len() as u8, 0, 0, 0, 0],
                handshake.to_vec(),
            ]
            .concat(),
        );
        let snapshots = Rc::new(RefCell::new(Vec::new()));
        let push = |hub: Rc<Hub>,
                    snapshots: Rc<RefCell<Vec<(String, String, Saved)>>>,
                    store: Rc<Store>| {
            move |target: &str, payload: &Value| {
                hub.broadcast(target, payload).unwrap();
                let id = payload
                    .get("id")
                    .as_str()
                    .or_else(|| payload.get("objectId").as_str());
                if let Some(id) = id
                    && let Some(row) = store.card(id, None).unwrap()
                {
                    snapshots.borrow_mut().push((
                        target.to_owned(),
                        id.to_owned(),
                        Saved {
                            data: json::parse(&row.data).unwrap(),
                            extras: json::parse(&row.extras).unwrap(),
                            html: row.html,
                        },
                    ));
                }
            }
        };
        let writer = Rc::new(
            Writer::new(
                store.clone(),
                push(hub.clone(), snapshots.clone(), store.clone()),
            )
            .unwrap(),
        );
        runtime::set_env(HashMap::from([
            ("OFFLINE".into(), "1".into()),
            (
                "MYMIND_ENV_FILE".into(),
                dir.join("absent.env").to_string_lossy().into_owned(),
            ),
        ]));
        let api = Rc::new(
            Api::new(ApiOptions {
                store: store.clone(),
                media: media.clone(),
                upload_storage: Rc::new(DirectoryUploads::new(dir.join("uploads"))),
                screenshot: shot.clone(),
                resource: resource.clone(),
                fetch: fetch.clone(),
                readonly,
                localize: str::to_owned,
                push: Box::new(push(hub, snapshots.clone(), store.clone())),
                random: runtime::random_bytes,
            })
            .await
            .unwrap(),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let task = tokio::task::spawn_local(serve(listener, Rc::new(ApiHandler(api.clone()))));
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        Self {
            dir,
            fixtures,
            media_bytes,
            store,
            media,
            resource,
            fetch,
            shot,
            writer,
            api,
            wire,
            base,
            snapshots,
            task,
            client,
        }
    }

    /// Install only synthetic provider settings in the core runtime.
    ///
    /// # Arguments
    /// * `ai` - Enable the local Gemini/chat fixture.
    /// * `decisions` - Enable tag picking against the local OpenAI Decisions fixture.
    ///
    /// # Returns
    /// Nothing; no process or environment file is read.
    pub fn env(&self, ai: bool, decisions: bool) {
        let mut vars = HashMap::from([
            ("OFFLINE".into(), "1".into()),
            (
                "MYMIND_ENV_FILE".into(),
                self.dir.join("absent.env").to_string_lossy().into_owned(),
            ),
        ]);
        if ai {
            vars.insert("GEMINI_API_KEY".into(), "fixture-key".into());
        }
        if decisions {
            vars.insert("OPENAI_API_KEY".into(), "fixture-openai-key".into());
        }
        for name in ["GEMINI_BASE_URL", "AI_CHAT_BASE_URL"] {
            vars.insert(name.into(), self.fixtures.base.clone());
        }
        vars.insert("OPENAI_BASE_URL".into(), format!("{}/v1", self.fixtures.base.trim_end_matches('/')));
        runtime::set_env(vars);
    }

    /// Send an HTTP request through the native server on its assigned loopback port.
    ///
    /// # Arguments
    /// * `method` - HTTP method.
    /// * `path` - Relative request target or a URL at this exact API origin.
    /// * `bytes` - Exact request body.
    /// * `headers` - Ordered headers.
    ///
    /// # Returns
    /// The complete status, headers and body.
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        bytes: &[u8],
        headers: &[(&str, &str)],
    ) -> Answer {
        let url = if path.starts_with("http") {
            path.to_owned()
        } else {
            format!("{}{path}", self.base)
        };
        assert_eq!(
            mm_js::url::origin(&mm_js::url::parse(&url, None).unwrap()),
            self.base
        );
        let mut request = self
            .client
            .request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                &url,
            )
            .body(bytes.to_vec());
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await.unwrap();
        let status = response.status().as_u16();
        let headers =
            Headers::from_pairs(response.headers().iter().map(|(name, value)| {
                (name.as_str().to_owned(), value.to_str().unwrap().to_owned())
            }));
        Answer {
            status,
            headers,
            bytes: response.bytes().await.unwrap().to_vec(),
        }
    }

    /// Send a JSON request through the real HTTP listener.
    ///
    /// # Arguments
    /// * `method` - HTTP method.
    /// * `path` - Relative API target.
    /// * `body` - Complete synthetic JSON body.
    ///
    /// # Returns
    /// The complete HTTP answer.
    pub async fn json(&self, method: &str, path: &str, body: Value) -> Answer {
        self.request(
            method,
            path,
            json::to_string(&body).as_bytes(),
            &[("content-type", "application/json")],
        )
        .await
    }

    /// Save a fixture URL through the real API.
    ///
    /// # Arguments
    /// * `name` - Fixture path and optional query string.
    ///
    /// # Returns
    /// The new card id after asserting status 201.
    pub async fn save(&self, name: &str) -> String {
        let result = self
            .json(
                "POST",
                "/objects",
                obj! { "content" => format!("{}/{name}", self.fixtures.base) }.into(),
            )
            .await;
        assert_eq!(result.status, 201, "check-capture.mjs:72 URL save status");
        result.value().get("id").as_str().unwrap().to_owned()
    }

    /// Read persisted state independently from the HTTP serializer.
    ///
    /// # Arguments
    /// * `id` - Existing synthetic card id.
    ///
    /// # Returns
    /// The exact persisted card data, extras and rendered HTML.
    pub fn row(&self, id: &str) -> Saved {
        let row = self.store.card(id, None).unwrap().unwrap();
        Saved {
            data: json::parse(&row.data).unwrap(),
            extras: json::parse(&row.extras).unwrap(),
            html: row.html,
        }
    }

    /// Wait for an observable background state with the original bounded deadline.
    ///
    /// # Arguments
    /// * `predicate` - Current-state probe, called after every yield.
    ///
    /// # Returns
    /// Nothing after success; panics after the original 15-second deadline.
    pub async fn until(&self, predicate: impl Fn() -> bool) {
        self.until_for(predicate, 15).await;
    }

    // Wait for a synthetic background state for the caller's original bounded deadline.
    async fn until_for(&self, predicate: impl Fn() -> bool, seconds: u64) {
        let end = Instant::now() + Duration::from_secs(seconds);
        while !predicate() {
            assert!(
                Instant::now() < end,
                "capture background completion timed out"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    /// Wait for capture and AI processing to settle.
    ///
    /// # Arguments
    /// * `id` - Saved card id.
    ///
    /// # Returns
    /// Its complete persisted state.
    pub async fn captured(&self, id: &str) -> Saved {
        self.until_for(
            || {
                let row = self.row(id);
                !row.extras.get("capturePending").truthy() && flags(&row.data) & 1 == 0
            },
            45,
        )
        .await;
        self.row(id)
    }

    /// Construct a separately drainable enrichment queue over the same real store and media services.
    ///
    /// # Arguments
    /// * `screenshot` - Screenshot fixture, or real Chromium host.
    ///
    /// # Returns
    /// A queue whose mutations broadcast through the real hub encoder.
    pub fn jobs(&self, screenshot: Rc<dyn Screenshot>) -> Enrichment {
        Enrichment::new(EnrichmentOptions {
            store: self.store.clone(),
            writer: self.writer.clone(),
            media: self.media.clone(),
            resource: self.resource.clone(),
            screenshot,
            fetch: self.fetch.clone(),
            offline: true,
            readonly: false,
        })
    }

    /// Create a committed card with the independent writer, before any queued work can run.
    ///
    /// # Arguments
    /// * `body` - Synthetic creation data.
    ///
    /// # Returns
    /// Its id.
    pub fn create(&self, body: Value) -> String {
        self.writer
            .run(|writer| writer.create(&body, &Value::Undefined))
            .unwrap()
            .get("body")
            .get("id")
            .as_str()
            .unwrap()
            .to_owned()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.api.close();
        self.task.abort();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Read numeric card flags with the same zero fallback as the JS assertion bit operations.
///
/// # Arguments
/// * `data` - Persisted synthetic card JSON.
///
/// # Returns
/// The flags as an integer bit set.
pub fn flags(data: &Value) -> i64 {
    data.get("flags").as_f64().unwrap_or(0.0) as i64
}

/// Build the same reader document the original capture-quality suite edits.
///
/// # Arguments
/// * `text` - The complete synthetic paragraph text.
///
/// # Returns
/// One paragraph in a ProseMirror document.
pub fn doc(text: &str) -> Value {
    obj! { "type" => "doc", "content" => vec![Value::from(obj! { "type" => "paragraph", "content" => vec![Value::from(obj! { "type" => "text", "text" => text })] })] }.into()
}
