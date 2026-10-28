//! The `Library` Durable Object: one person's library and its live `/hub` (`worker/library-object.mjs`).

use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::rc::{Rc, Weak};

use futures_util::FutureExt;
use futures_util::future::{LocalBoxFuture, Shared};
use js_sys::{Object, Reflect};
use mm_core::auth_store::generated_library;
use mm_core::hosted::{Hosted, HostedOptions};
use mm_core::http::{Headers, Response};
use mm_core::library_rows::{insert_rows, table_digest};
use mm_core::site::Site;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::Store;
use mm_core::store_member::{self, profile_from_session};
use mm_core::{Error, Result};
use mm_js::{Value, json, obj};
use wasm_bindgen::JsValue;
use worker::{State, WebSocketIncomingMessage, WebSocketRequestResponsePair, durable_object};

use crate::auth_object::CALL;
use crate::bucket_r2::R2Bucket;
use crate::fetch::WorkerFetch;
use crate::http::{request_from, response_from, text};
use crate::media_client::MediaClient;
use crate::sql_do::DoSql;
use crate::util::{call, get, js_error, worker_error};

const PING: &str = "{\"type\":6}\u{1e}";
const RECORD: char = '\u{1e}';
const MAX_MESSAGE: usize = 1_048_576;
const SESSION_CHECK_MS: i64 = 60_000;
// Pictures given a ThumbHash after each sync of a phone, a few seconds of the object's time, so the library's older
// pictures fill in while it is in use.
const PICTURES_PER_SYNC: usize = 50;

type Opening = Shared<LocalBoxFuture<'static, Result<Rc<Hosted>>>>;

struct Inner {
    state: State,
    env: worker::Env,
    db: Result<Rc<DoSql>>,
    opening: RefCell<Option<Opening>>,
    queue: RefCell<VecDeque<String>>,
    draining: Cell<bool>,
}

/// One person's library: its database lives in this object's SQLite storage, its media in R2, and its live updates
/// (`/hub`) on WebSockets that only this library's events reach.
#[durable_object]
pub struct Library {
    inner: Rc<Inner>,
}

struct Attachment {
    credential: String,
    library: String,
    ready: bool,
    text: String,
}

fn read_attachment(socket: &worker::WebSocket) -> Option<Attachment> {
    let raw: &web_sys::WebSocket = socket.as_ref();
    let value = call(raw.as_ref(), "deserializeAttachment", &[]).ok()?;
    if !value.is_object() {
        return None;
    }
    let field = |name: &str| get(&value, name).ok();
    Some(Attachment {
        credential: field("credential").and_then(|v| v.as_string()).unwrap_or_default(),
        library: field("library").and_then(|v| v.as_string()).unwrap_or_default(),
        ready: field("ready").and_then(|v| v.as_bool()).unwrap_or(false),
        text: field("text").and_then(|v| v.as_string()).unwrap_or_default(),
    })
}

fn write_attachment(socket: &worker::WebSocket, attachment: &Attachment) -> Result<()> {
    let object = Object::new();
    let set = |name: &str, value: JsValue| Reflect::set(&object, &name.into(), &value).map_err(js_error);
    set("credential", JsValue::from_str(&attachment.credential))?;
    set("library", JsValue::from_str(&attachment.library))?;
    set("ready", JsValue::from_bool(attachment.ready))?;
    set("text", JsValue::from_str(&attachment.text))?;
    let raw: &web_sys::WebSocket = socket.as_ref();
    call(raw.as_ref(), "serializeAttachment", &[object.into()])?;
    Ok(())
}

fn valid_library(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn failure(status: u16, message: &str) -> worker::Result<worker::Response> {
    let body = json::stringify(&obj! { "error" => message }.into()).map_err(|error| worker::Error::RustError(error.0.into()))?.unwrap_or_default();
    response_from(Response::new(status, Headers::from([("content-type", "application/json")]), body)).map_err(to_worker)
}

fn to_worker(error: Error) -> worker::Error {
    worker::Error::RustError(error.to_string())
}

fn rows_of(value: &Value) -> Vec<Vec<Value>> {
    value.as_array().into_iter().flatten().map(|row| row.as_array().cloned().unwrap_or_default()).collect()
}

fn strings_of(value: &Value) -> Vec<String> {
    value.as_array().into_iter().flatten().map(Value::to_js_string).collect()
}

fn parameter(value: &Value) -> SqlValue {
    match value {
        Value::Number(number) => SqlValue::Real(*number),
        other => SqlValue::Text(other.to_js_string()),
    }
}

impl Inner {
    fn db(&self) -> Result<&Rc<DoSql>> {
        self.db.as_ref().map_err(Clone::clone)
    }

    fn tables(&self) -> Result<Value> {
        let db = self.db()?;
        let names = db.query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_%' ESCAPE '\\' AND name NOT LIKE 'search_fts%' ORDER BY name",
            &[],
        )?;
        let mut tables = mm_js::Object::new();
        for row in &names {
            let name = row.text("name").unwrap_or_default();
            let count = db.one(&format!("SELECT COUNT(*) AS n FROM \"{name}\""), &[])?.and_then(|row| row.number("n")).unwrap_or(0.0);
            tables.insert(name, count);
        }
        Ok(Value::Object(tables))
    }

    async fn admin(&self, step: &str, library: &str, request: &web_sys::Request) -> Result<Value> {
        let post = request.method() == "POST";
        let body = if post { json::parse(&String::from_utf8_lossy(&request_from(request)?.body.bytes().await?))? } else { obj! {}.into() };
        let db = self.db()?;
        match step {
            "reset" => {
                let open = self.opening.borrow_mut().take();
                if let Some(open) = open
                    && let Ok(site) = open.await
                {
                    site.close();
                }
                self.state.storage().delete_all().await.map_err(worker_error)?;
                let sql: Rc<dyn Sql> = db.clone();
                Store::new(sql)?;
                Ok(obj! { "library" => library, "tables" => self.tables()? }.into())
            }
            "rows" => {
                let inserted = insert_rows(&**db, body.get("table").as_str().unwrap_or_default(), &strings_of(body.get("columns")), &rows_of(body.get("rows")))?;
                Ok(obj! { "inserted" => inserted }.into())
            }
            "finish" => {
                db.transaction(|| {
                    db.exec_batch("DELETE FROM search_dirty")?;
                    insert_rows(&**db, "search_dirty", &["id".to_owned()], &rows_of(body.get("dirty")))?;
                    for sequence in body.get("sequences").as_array().into_iter().flatten() {
                        let (name, seq) = (parameter(sequence.get("name")), parameter(sequence.get("seq")));
                        if db.execute("UPDATE sqlite_sequence SET seq = CAST(? AS INTEGER) WHERE name = ?", &[seq.clone(), name.clone()])? == 0 {
                            db.execute("INSERT INTO sqlite_sequence (name, seq) VALUES (?, CAST(? AS INTEGER))", &[name, seq])?;
                        }
                    }
                    Ok(())
                })?;
                db.exec_batch("INSERT INTO search_fts(search_fts) VALUES ('integrity-check')")?;
                Ok(obj! { "library" => library, "tables" => self.tables()? }.into())
            }
            "digest" => {
                let mut digests = mm_js::Object::new();
                for entry in body.get("tables").as_array().into_iter().flatten() {
                    let table = entry.get("table").to_js_string();
                    digests.insert(table.clone(), table_digest(&**db, &table, &strings_of(entry.get("columns")))?.to_value());
                }
                Ok(Value::Object(digests))
            }
            "counts" => Ok(obj! { "library" => library, "tables" => self.tables()? }.into()),
            _ => Err(Error::status(404, "Unknown import step")),
        }
    }

    async fn signed_in(&self, sockets: Vec<worker::WebSocket>) -> Result<Vec<worker::WebSocket>> {
        let states = sockets.iter().map(read_attachment).collect::<Vec<_>>();
        let library = states.first().and_then(Option::as_ref).map(|state| state.library.clone()).unwrap_or_default();
        let mut credentials = Vec::new();
        for state in states.iter().flatten() {
            if !state.credential.is_empty() && !credentials.contains(&state.credential) {
                credentials.push(state.credential.clone());
            }
        }
        let live: HashSet<String> = if !library.is_empty() && !credentials.is_empty() { self.active(&credentials, &library).await?.into_iter().collect() } else { HashSet::new() };
        let mut kept = Vec::new();
        for (socket, state) in sockets.into_iter().zip(states) {
            let state = state.unwrap_or(Attachment { credential: String::new(), library: String::new(), ready: false, text: String::new() });
            if state.library == library && live.contains(&state.credential) {
                kept.push(socket);
            } else {
                let _ = socket.close(Some(1008), Some("Signed out"));
            }
        }
        Ok(kept)
    }

    async fn active(&self, credentials: &[String], library: &str) -> Result<Vec<String>> {
        let stub = self.env.durable_object("AUTH_OBJECT").map_err(worker_error)?.get_by_name("auth").map_err(worker_error)?;
        let body = obj! { "credentials" => credentials.iter().map(Value::from).collect::<Vec<_>>(), "library" => library };
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        let headers = web_sys::Headers::new().map_err(js_error)?;
        headers.append(CALL, "active").map_err(js_error)?;
        init.set_headers(&headers);
        init.set_body(&JsValue::from_str(&json::stringify(&body.into())?.unwrap_or_default()));
        let request = web_sys::Request::new_with_str_and_init("http://auth/active", &init).map_err(js_error)?;
        let mut response = stub.fetch_with_request(worker::Request::from(request)).await.map_err(worker_error)?;
        let live = json::parse(&response.text().await.map_err(worker_error)?)?;
        Ok(live.as_array().into_iter().flatten().filter_map(|item| item.as_str().map(str::to_owned)).collect())
    }

    // The library's server, opened on its first request and again after a failed opening.
    async fn site(self: &Rc<Self>, library: &str) -> Result<Rc<Hosted>> {
        let current = self.opening.borrow().clone();
        let opening = match current {
            Some(opening) => opening,
            None => {
                let opening = Hosted::new(self.options(library)?).map(|opened| opened.map(Rc::new)).boxed_local().shared();
                *self.opening.borrow_mut() = Some(opening.clone());
                opening
            }
        };
        let opened = opening.clone().await;
        if opened.is_err() && self.opening.borrow().as_ref().is_some_and(|current| current.ptr_eq(&opening)) {
            self.opening.borrow_mut().take();
        }
        opened
    }

    fn options(self: &Rc<Self>, library: &str) -> Result<HostedOptions> {
        let media = Rc::new(MediaClient::new(&self.env)?);
        let inner = Rc::downgrade(self);
        let flag = |name: &str| mm_core::runtime::env(name).as_deref() == Some("1");
        Ok(HostedOptions {
            db: self.db()?.clone(),
            bucket: Rc::new(R2Bucket(self.env.bucket("BUCKET").map_err(worker_error)?)),
            library: library.to_owned(),
            processor: media.clone(),
            push: Box::new(move |target, payload| push(&inner, target, payload)),
            readonly: flag("STORE_READONLY"),
            offline: flag("OFFLINE"),
            screenshot: media.clone(),
            resource: media,
            fetch: Rc::new(WorkerFetch),
            random: mm_core::runtime::random_bytes,
        })
    }

    async fn drain(self: Rc<Self>) {
        loop {
            let next = self.queue.borrow_mut().pop_front();
            let Some(message) = next else { break };
            let ready = self.state.get_websockets().into_iter().filter(|socket| read_attachment(socket).is_some_and(|state| state.ready)).collect::<Vec<_>>();
            if ready.is_empty() {
                continue;
            }
            match self.signed_in(ready).await {
                Ok(sockets) => sockets.iter().for_each(|socket| drop(socket.send_with_str(&message))),
                Err(error) => web_sys::console::error_2(&"[hub]".into(), &error.to_string().into()),
            }
        }
        self.draining.set(false);
    }
}

// Queue one committed change for this library's signed-in sockets, sent in call order (`broadcast`).
fn push(inner: &Weak<Inner>, target: &str, payload: &Value) {
    let Some(inner) = inner.upgrade() else { return };
    let Ok(Some(message)) = json::stringify(&obj! { "type" => 1, "target" => target, "arguments" => vec![payload.clone()] }.into()) else { return };
    inner.queue.borrow_mut().push_back(format!("{message}{RECORD}"));
    if !inner.draining.replace(true) {
        wasm_bindgen_futures::spawn_local(inner.drain());
    }
}

impl Library {
    // A signed-in request to the library's server; its own failures answer 500 as `send(res, 500, String(error))`.
    async fn hosted_site(&self, request: &web_sys::Request, library: &str) -> Result<Response> {
        self.prepare_member(request, library)?;
        let site = self.inner.site(library).await?;
        let mut incoming = request_from(request)?;
        if incoming.pathname() == "/app/v1/changes" {
            site.api().jobs().hash_pictures(PICTURES_PER_SYNC);
        }
        match site.handle(&mut incoming).await {
            Ok(response) => Ok(response),
            Err(error) => {
                let text = error.to_js_string();
                let key = format!("[error] {} {}{}", incoming.method, incoming.pathname(), incoming.search());
                web_sys::console::error_2(&key.into(), &text.clone().into());
                Ok(Response::send(500, text, &Headers::new()))
            }
        }
    }

    // Insert the user row of a library created at first sign-in. An imported library, and any library that already
    // has a user, is left unchanged.
    fn prepare_member(&self, request: &web_sys::Request, library: &str) -> Result<()> {
        let header = |name: &str| request.headers().get(name).ok().flatten().unwrap_or_default();
        let sub = header("x-mymind-sub");
        let email = header("x-mymind-email");
        if sub.is_empty() && email.is_empty() {
            return Ok(());
        }
        let profile = profile_from_session(&sub, &email);
        if !generated_library(&profile.account, library) {
            return Ok(());
        }
        let db = self.inner.db()?.clone();
        let sql: Rc<dyn Sql> = db;
        let store = Store::new(sql)?;
        store_member::ensure_member(&store, &profile)?;
        Ok(())
    }

    fn accept_hub(&self, request: &web_sys::Request, library: &str) -> worker::Result<worker::Response> {
        if request.headers().get("sec-websocket-key").ok().flatten().is_none() {
            return worker::Response::builder().with_status(400).ok("").map(|response| response.with_headers(worker::Headers::new()));
        }
        let pair = worker::WebSocketPair::new()?;
        self.inner.state.accept_web_socket(&pair.server);
        let credential = request.headers().get("x-mymind-credential").ok().flatten().unwrap_or_default();
        write_attachment(&pair.server, &Attachment { credential, library: library.to_owned(), ready: false, text: String::new() }).map_err(to_worker)?;
        let state = self.inner.state.storage();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(None) = state.get_alarm().await {
                let _ = state.set_alarm(js_sys::Date::now() as i64 + SESSION_CHECK_MS).await;
            }
        });
        worker::Response::from_websocket(pair.client)
    }
}

impl worker::DurableObject for Library {
    fn new(state: State, env: worker::Env) -> Self {
        crate::runtime::configure(&env);
        crate::runtime::bind_keepalive(&state);
        let db = DoSql::open(&state);
        if let Ok(pair) = WebSocketRequestResponsePair::new(PING, PING) {
            state.set_websocket_auto_response(&pair);
        }
        Self { inner: Rc::new(Inner { state, env, db, opening: RefCell::new(None), queue: RefCell::new(VecDeque::new()), draining: Cell::new(false) }) }
    }

    async fn fetch(&self, req: worker::Request) -> worker::Result<worker::Response> {
        let request = req.inner();
        let header = |name: &str| request.headers().get(name).ok().flatten();
        let library = header("x-mymind-library").unwrap_or_default();
        if !valid_library(&library) {
            return worker::Response::builder().with_status(400).with_header("content-type", "text/plain;charset=UTF-8")?.ok("Unknown library");
        }
        if let Some(step) = header("x-mymind-admin") {
            return match self.inner.admin(&step, &library, request).await {
                Ok(value) => {
                    let body = json::stringify(&value).map_err(|error| worker::Error::RustError(error.0.into()))?.unwrap_or_default();
                    response_from(Response::new(200, Headers::from([("content-type", "application/json")]), body)).map_err(to_worker)
                }
                Err(error) => failure(error.http_status().unwrap_or(500), &error.to_string()),
            };
        }
        let url = req.url()?;
        if url.path() == "/hub" && header("upgrade").is_some_and(|value| value.eq_ignore_ascii_case("websocket")) {
            return self.accept_hub(request, &library);
        }
        match self.hosted_site(request, &library).await {
            Ok(response) => response_from(response).map_err(to_worker),
            Err(error) => response_from(text(500, &error.to_js_string(), &Headers::new())).map_err(to_worker),
        }
    }

    async fn alarm(&self) -> worker::Result<worker::Response> {
        let sockets = self.inner.state.get_websockets();
        if !sockets.is_empty() {
            self.inner.signed_in(sockets).await.map_err(to_worker)?;
            if !self.inner.state.get_websockets().is_empty() {
                self.inner.state.storage().set_alarm(js_sys::Date::now() as i64 + SESSION_CHECK_MS).await?;
            }
        }
        worker::Response::empty()
    }

    async fn websocket_message(&self, ws: worker::WebSocket, message: WebSocketIncomingMessage) -> worker::Result<()> {
        let Some(mut state) = read_attachment(&ws) else { return Ok(()) };
        match message {
            WebSocketIncomingMessage::String(text) => state.text.push_str(&text),
            WebSocketIncomingMessage::Binary(bytes) => state.text.push_str(&String::from_utf8_lossy(&bytes)),
        }
        if state.text.encode_utf16().count() > MAX_MESSAGE {
            return ws.close(Some(1009), Some("Message too large"));
        }
        while let Some(end) = state.text.find(RECORD) {
            let record = state.text[..end].to_owned();
            state.text.drain(..end + RECORD.len_utf8());
            let Ok(parsed) = json::parse(&record) else { return ws.close(Some(1002), Some("Invalid record")) };
            if parsed.is_nullish() {
                return Err(worker::Error::RustError("Cannot read properties of null".into()));
            }
            if !state.ready && parsed.get("protocol").as_str() == Some("json") && parsed.get("version").as_f64() == Some(1.0) {
                ws.send_with_str(format!("{{}}{RECORD}"))?;
                state.ready = true;
            }
        }
        write_attachment(&ws, &state).map_err(to_worker)
    }

    async fn websocket_close(&self, ws: worker::WebSocket, code: usize, reason: String, _was_clean: bool) -> worker::Result<()> {
        let code = u16::try_from(code).ok().filter(|code| (1000..5000).contains(code) && !matches!(code, 1005 | 1006 | 1015));
        let _ = ws.close(code, Some(reason));
        Ok(())
    }

    async fn websocket_error(&self, _ws: worker::WebSocket, _error: worker::Error) -> worker::Result<()> {
        Ok(())
    }
}
