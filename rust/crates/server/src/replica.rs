//! The replay replica of `server.mjs`: mymind's website code from `mirror/`, recorded answers from `capture/` and
//! `replay/`, and the fallbacks for what was never recorded.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use mm_core::device::device_variant;
use mm_core::host::Fetch;
use mm_core::http::{Headers, Request, Response};
use mm_core::site::{LOCAL_PREFIXES, LibraryApi, content_type, is_text_extension, localize, localize_value, replay_key_for, send_range};
use mm_core::{Error, Object, Result, Value};
use mm_js::json;
use url::Url;

use crate::node_path::{extname, fs_error, read_file, resolve};

struct Card {
    html: String,
    json: String,
}

/// The recordings one replica serves, loaded at startup.
pub struct Replica {
    capture: PathBuf,
    mirror: PathBuf,
    replay: PathBuf,
    offline: bool,
    meta: Value,
    shell: String,
    cards: Vec<Card>,
    cards_by_id: HashMap<String, usize>,
    responses: Object,
    details: Object,
    index: RefCell<Rc<Object>>,
    bodies: RefCell<HashMap<String, Rc<Vec<u8>>>>,
    fetch: Rc<dyn Fetch>,
}

fn js_failure(text: String) -> Error {
    Error::internal(text)
}

async fn read_json(path: &Path) -> Result<Value> {
    let bytes = read_file(path).await.map_err(js_failure)?;
    json::parse(&String::from_utf8_lossy(&bytes)).map_err(Error::from)
}

fn object(value: Value) -> Object {
    match value {
        Value::Object(object) => object,
        _ => Object::default(),
    }
}

fn send(status: u16, body: impl Into<mm_core::http::Body>, headers: &[(&str, &str)]) -> Response {
    Response::send(status, body, &Headers::from_pairs(headers.iter().copied()))
}

// The first `data-id="…"` value of a card's HTML (`/data-id="([^"]+)"/`).
fn card_id(html: &str) -> Option<&str> {
    let mut rest = html;
    while let Some(start) = rest.find("data-id=\"") {
        rest = &rest[start + "data-id=\"".len()..];
        let end = rest.find('"')?;
        if end > 0 {
            return Some(&rest[..end]);
        }
    }
    None
}

// A JS value as `String(value)` writes it into a header.
fn header_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => mm_js::number_to_string(*number),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".to_owned(),
        _ => "[object Object]".to_owned(),
    }
}

fn entry_headers(entry: &Value) -> Headers {
    match entry.get("headers") {
        Value::Object(headers) => Headers::from_pairs(headers.iter().map(|(name, value)| (name.into_owned(), header_text(value)))),
        _ => Headers::new(),
    }
}

fn entry_status(entry: &Value) -> u16 {
    entry.get("status").as_f64().map_or(200, |status| status as u16)
}

// Encode a string with the historical recorded MessagePack contract.
fn pack_string(out: &mut Vec<u8>, text: &str) {
    let length = text.len();
    if (32..0x100).contains(&length) {
        out.extend_from_slice(&[0xd9, length as u8]);
    } else if length <= 31 {
        out.push(0xa0 | length as u8);
    } else if length < 0x10000 {
        out.extend_from_slice(&[0xda, (length >> 8) as u8, length as u8]);
    } else {
        out.push(0xdb);
        out.extend_from_slice(&(length as u32).to_be_bytes());
    }
    out.extend_from_slice(text.as_bytes());
}

// Encode one `{html, json}` MessagePack map per card.
fn pack_cards<'a>(cards: impl IntoIterator<Item = &'a Card>) -> Vec<u8> {
    let mut out = Vec::new();
    for card in cards {
        out.push(0x82);
        pack_string(&mut out, "html");
        pack_string(&mut out, &card.html);
        pack_string(&mut out, "json");
        pack_string(&mut out, &card.json);
    }
    out
}

// `fileInside(root, relative)`: the decoded path below `root`, or `None` when it climbs out.
fn file_inside(root: &Path, relative: &str) -> Result<Option<PathBuf>> {
    let decoded = mm_js::uri::decode_uri_component(relative).map_err(|error| js_failure(format!("URIError: {error}")))?;
    let target = resolve(root, &decoded);
    Ok((target.starts_with(root) && target != root).then_some(target))
}

/// How a replica is set up.
pub struct Options {
    /// The repository root, holding `capture/`, `mirror/` and `replay/`.
    pub root: PathBuf,
    /// Never fetch missing website code from mymind's static origin (`OFFLINE=1`).
    pub offline: bool,
    /// Serve a hosted library instead, so no recording is read (`HOSTED=1`).
    pub hosted: bool,
}

impl Replica {
    /// Read the immutable recordings at server startup.
    ///
    /// # Arguments
    /// * `options` - Root and flags.
    /// * `fetch` - Outbound HTTP for website code that was never mirrored.
    ///
    /// # Errors
    /// A recording file that is missing or not JSON.
    pub async fn load(options: &Options, fetch: Rc<dyn Fetch>) -> Result<Self> {
        let capture = options.root.join("capture");
        let mut replica = Self {
            mirror: options.root.join("mirror").join("static"),
            replay: options.root.join("replay"),
            offline: options.offline,
            meta: Value::Object(Object::default()),
            shell: String::new(),
            cards: Vec::new(),
            cards_by_id: HashMap::new(),
            responses: Object::default(),
            details: Object::default(),
            index: RefCell::new(Rc::new(Object::default())),
            bodies: RefCell::new(HashMap::new()),
            fetch,
            capture,
        };
        if options.hosted {
            return Ok(replica);
        }
        replica.meta = read_json(&replica.capture.join("meta.json")).await?;
        replica.shell = mm_core::api::SHELL_TEMPLATE.to_owned();
        let Value::Array(cards) = read_json(&replica.capture.join("cards.json")).await? else {
            return Err(js_failure("TypeError: cards.map is not a function".into()));
        };
        for card in cards {
            let (Some(html), Some(json)) = (card.get("html").as_str(), card.get("json").as_str()) else {
                return Err(js_failure("TypeError: text.replaceAll is not a function".into()));
            };
            replica.cards.push(Card { html: localize(html), json: localize(json) });
        }
        for (index, card) in replica.cards.iter().enumerate() {
            if let Some(id) = card_id(&card.html) {
                replica.cards_by_id.insert(id.to_owned(), index);
            }
        }
        replica.responses = object(read_json(&replica.capture.join("responses.json")).await?);
        replica.details = object(read_json(&replica.capture.join("details.json")).await?);
        replica.reload_replay().await?;
        Ok(replica)
    }

    /// The number of cards, detail views and replayed responses, for the startup lines.
    pub fn counts(&self) -> (usize, usize, usize) {
        (self.cards.len(), self.details.len(), self.index.borrow().len())
    }

    /// `meta.capturedAt` as the startup line prints it.
    pub fn captured_at(&self) -> String {
        match self.meta.get("capturedAt") {
            Value::Undefined => "undefined".to_owned(),
            other => header_text(other),
        }
    }

    /// Whether a recorded detail view exists for this card id (`Object.hasOwn(details, id)`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    pub fn has_detail(&self, id: &str) -> bool {
        self.details.get(id).is_some()
    }

    /// Read `replay/index.json` again and forget every cached body (`reloadReplay`).
    ///
    /// # Returns
    /// The number of replayed responses.
    ///
    /// # Errors
    /// An index that is not JSON; a missing index reads as empty.
    pub async fn reload_replay(&self) -> Result<usize> {
        let text = match read_file(&self.replay.join("index.json")).await {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => "{}".to_owned(),
        };
        let index = object(json::parse(&text).map_err(Error::from)?);
        let count = index.len();
        *self.index.borrow_mut() = Rc::new(index);
        self.bodies.borrow_mut().clear();
        Ok(count)
    }

    /// Reload the replay index whenever `replay/index.json` changes, checking every second as `watchFile` does.
    pub async fn watch(self: Rc<Self>) {
        let file = self.replay.join("index.json");
        let stamp = |file: &Path| {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata(file).ok().map(|info| (info.mtime(), info.mtime_nsec(), info.ctime(), info.ctime_nsec(), info.size(), info.ino(), info.mode()))
        };
        let mut last = stamp(&file);
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let now = stamp(&file);
            if now != last {
                last = now;
                match self.reload_replay().await {
                    Ok(count) => mm_core::runtime::log(&format!("replay/ reloaded: {count} responses")),
                    Err(error) => eprintln!("replay/ reload failed: {error}"),
                }
            }
        }
    }

    /// The recording for this request (`replayedFor`): the phone variant first for phone requests, and a verification
    /// run's own recording when the request names its run.
    ///
    /// # Arguments
    /// * `request` - Method and headers.
    /// * `url` - The URL to look up, which may differ from the request's.
    pub fn replayed_for(&self, request: &Request, url: &Url) -> Option<Value> {
        let index = Rc::clone(&self.index.borrow());
        let found = |key: &str| index.get(key).filter(|entry| entry.truthy()).cloned();
        let mut key = replay_key_for(&request.method, &request.headers, url);
        let variant = device_variant(request.header("user-agent").unwrap_or(""));
        if !variant.is_empty() && found(&format!("{variant}|{key}")).is_some() {
            key = format!("{variant}|{key}");
        }
        if let Some(run) = request.header("x-replay-run").filter(|run| !run.is_empty())
            && let Some(entry) = found(&format!("run:{run}|{key}"))
        {
            return Some(entry);
        }
        found(&key)
    }

    // A replayed body with mymind's origins localized, cached until the next reload (`replayBody`).
    async fn replay_body(&self, entry: &Value) -> Result<Rc<Vec<u8>>> {
        let file = header_text(entry.get("file"));
        if let Some(body) = self.bodies.borrow().get(&file) {
            return Ok(Rc::clone(body));
        }
        let mut body = read_file(&self.replay.join("bodies").join(&file)).await.map_err(js_failure)?;
        let kind = match entry.get("headers").get("content-type") {
            Value::Undefined | Value::Null => String::new(),
            other => header_text(other),
        };
        if kind.contains("msgpack") {
            let items = mm_msgpack::decode_multi(&body, mm_msgpack::DecodeOptions::default()).map_err(|error| js_failure(format!("RangeError: {error}")))?;
            let mut out = Vec::new();
            for item in items {
                mm_msgpack::encode_into(&mut out, &localize_value(&item), mm_msgpack::EncodeOptions::default()).map_err(|error| js_failure(format!("Error: {error}")))?;
            }
            body = out;
        } else if ["text", "javascript", "json", "svg", "xml"].iter().any(|part| kind.contains(part)) {
            body = localize(&String::from_utf8_lossy(&body)).into_bytes();
        }
        let body = Rc::new(body);
        self.bodies.borrow_mut().insert(file, Rc::clone(&body));
        Ok(body)
    }

    // A mirrored or captured file, localized when asked and it is text, and split by Range when `range` is given.
    async fn serve_file(&self, file: &Path, localize_text: bool, range: Option<&Headers>) -> Result<Response> {
        let path = file.to_string_lossy();
        let extension = extname(&path).to_lowercase();
        let mut body = read_file(file).await.map_err(js_failure)?;
        if localize_text && is_text_extension(&extension) {
            body = localize(&String::from_utf8_lossy(&body)).into_bytes();
        }
        let headers = Headers::from([("content-type", content_type(&extension).unwrap_or("application/octet-stream"))]);
        Ok(match range {
            Some(request) => send_range(request, 200, body, &headers),
            None => Response::send(200, body, &headers),
        })
    }

    // mymind's website code, fetched once from its static origin when it was never mirrored (`serveStatic`).
    async fn serve_static(&self, relative: &str, search: &str) -> Result<Response> {
        let Some(file) = file_inside(&self.mirror, relative)? else { return Ok(send(400, "bad path", &[])) };
        match self.serve_file(&file, true, None).await {
            Ok(response) => return Ok(response),
            Err(_) if self.offline => return Ok(send(404, "not mirrored", &[])),
            Err(_) => {}
        }
        let request = Request::new("GET", &format!("{}{relative}{search}", LOCAL_PREFIXES[0].1)).map_err(|_| js_failure("TypeError: Failed to parse URL".into()))?;
        let upstream = self.fetch.fetch(request).await.map_err(|_| js_failure("TypeError: fetch failed".into()))?;
        if !(200..300).contains(&upstream.status) {
            return Ok(send(upstream.status, format!("upstream {}", upstream.status), &[]));
        }
        let body = upstream.body.bytes().await.map_err(|_| js_failure("TypeError: terminated".into()))?;
        if let Some(parent) = file.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|error| js_failure(fs_error(&error, "mkdir", Some(parent))))?;
        }
        tokio::fs::write(&file, &body).await.map_err(|error| js_failure(fs_error(&error, "open", Some(&file))))?;
        self.serve_file(&file, true, None).await
    }

    // A captured response of `capture/responses.json` (`serveRecorded`).
    async fn serve_recorded(&self, entry: &Value) -> Result<Response> {
        let body = read_file(&self.capture.join(header_text(entry.get("file")))).await.map_err(js_failure)?;
        Ok(Response::send(entry_status(entry), localize(&String::from_utf8_lossy(&body)), &entry_headers(entry)))
    }

    // A captured detail view, or the card's grid HTML with a note when none was captured (`serveDetail`).
    async fn serve_detail(&self, id: &str) -> Result<Response> {
        let mut name = String::new();
        let mut in_run = false;
        for character in id.chars() {
            let allowed = character.is_ascii_alphanumeric() || character == '.' || character == '-';
            if allowed {
                name.push(character);
            } else if !in_run {
                name.push('_');
            }
            in_run = !allowed;
        }
        let file = file_inside(&self.capture.join("details"), &format!("{name}.html"))?;
        if let Some(file) = file
            && let Ok(bytes) = read_file(&file).await
        {
            let mut headers = Headers::from([("content-type", "text/html; charset=utf-8")]);
            let magic = self.details.get(id).map_or(&Value::Undefined, |detail| detail.get("headers").get("x-magic"));
            if magic.truthy() {
                headers.set("x-magic", header_text(magic));
            }
            return Ok(Response::send(200, localize(&String::from_utf8_lossy(&bytes)), &headers));
        }
        let html = self.cards_by_id.get(id).map_or("", |&index| self.cards[index].html.as_str());
        let body = format!(
            "<div style=\"padding:48px;max-width:720px;margin:auto\">{html}<p style=\"opacity:.6\">Detail view not captured for {id}. Re-run capture with a higher --detail-cards.</p></div>"
        );
        Ok(send(200, body, &[("content-type", "text/html; charset=utf-8")]))
    }

    // `POST /cards/grid`: the recorded cards a JSON list of ids names, in its order.
    async fn grid(&self, request: &mut Request) -> Result<Response> {
        let body = request.take_body().bytes().await.map_err(|error| js_failure(format!("Error: {error}")))?;
        let text = String::from_utf8_lossy(&body);
        let ids = json::parse(if text.is_empty() { "[]" } else { &text }).map_err(Error::from)?;
        let ids = match ids {
            Value::Array(ids) => ids,
            Value::Null => return Err(js_failure("TypeError: Cannot read properties of null (reading 'map')".into())),
            _ => return Err(js_failure("TypeError: ids.map is not a function".into())),
        };
        let cards = ids.iter().filter_map(|id| id.as_str().and_then(|id| self.cards_by_id.get(id)).map(|&index| &self.cards[index]));
        Ok(send(200, pack_cards(cards), &[("content-type", "application/msgpack")]))
    }

    // The document shell from the API when the server has one.
    fn shell_for(api: Option<&dyn LibraryApi>) -> Result<Option<String>> {
        let Some(api) = api else { return Ok(None) };
        api.shell().map(Some)
    }

    /// Answer a request from the recordings, after the sign-in gate and the library API passed it on: the part of
    /// Serve replayed responses through the final fallback miss.
    ///
    /// # Arguments
    /// * `request` - The request.
    /// * `api` - The library API, for the desktop document shell.
    ///
    /// # Errors
    /// What the JS server answered with a 500, its text being `String(error)`.
    pub async fn handle(&self, request: &mut Request, api: Option<&dyn LibraryApi>) -> Result<Response> {
        let url = request.url.clone();
        let pathname = url.path().to_owned();
        let search = mm_js::url::search(&url);
        let key = format!("{} {pathname}{search}", request.method);
        let document = request.header("sec-fetch-dest") == Some("document");
        if request.method == "GET" && document {
            let body = Self::shell_for(api)?.unwrap_or_else(|| self.shell.clone()).into_bytes();
            return Ok(send(200, body, &[("content-type", "text/html; charset=utf-8")]));
        }
        let local = LOCAL_PREFIXES.iter().any(|(prefix, _)| pathname.starts_with(prefix)) || pathname.starts_with("/media/");
        if !local && let Some(entry) = self.replayed_for(request, &url) {
            let body = self.replay_body(&entry).await?;
            let headers = entry_headers(&entry);
            let status = entry_status(&entry);
            return Ok(send_range(&request.headers, status, body.as_ref().clone(), &headers));
        }
        if let Some(relative) = pathname.strip_prefix("/_static/") {
            return self.serve_static(relative, &search).await;
        }
        if key == "GET /cards" {
            let checkpoint = match self.meta.get("checkpoint") {
                Value::Undefined | Value::Null => "0".to_owned(),
                other => header_text(other),
            };
            return Ok(send(200, pack_cards(&self.cards), &[("content-type", "application/msgpack"), ("x-checkpoint", &checkpoint)]));
        }
        if request.method == "POST" && pathname == "/cards/grid" {
            return self.grid(request).await;
        }
        let card = pathname.strip_prefix("/cards/").filter(|_| request.method == "GET");
        if let Some(id) = card.and_then(|rest| rest.strip_suffix("/detail")).filter(|id| !id.is_empty() && !id.contains('/')) {
            let id = mm_js::uri::decode_uri_component(id).map_err(|error| js_failure(format!("URIError: {error}")))?;
            return self.serve_detail(&id).await;
        }
        if let Some(id) = card.filter(|id| !id.is_empty() && !id.contains('/'))
            && let Some(&index) = self.cards_by_id.get(id)
            && !url.query_pairs().any(|(name, _)| name == "template")
        {
            return Ok(send(200, pack_cards([&self.cards[index]]), &[("content-type", "application/msgpack")]));
        }
        if pathname == "/changes" {
            return Ok(send(200, "", &[("content-type", "application/msgpack"), ("x-count", "0")]));
        }
        if let Some(entry) = self.responses.get(&key) {
            return self.serve_recorded(entry).await;
        }
        if request.method != "GET" {
            if pathname != "/jserrors" {
                mm_core::runtime::log(&format!("[write ignored] {key}"));
            }
            return Ok(send(200, "{}", &[("content-type", "application/json")]));
        }
        let accept = request.header("accept").unwrap_or("-");
        mm_core::runtime::log(&format!("[miss] {key} (accept: {accept})"));
        let wants_json = request.header("accept").unwrap_or("").contains("json");
        Ok(if wants_json { send(200, "{}", &[("content-type", "application/json")]) } else { send(404, "", &[("content-type", "text/plain")]) })
    }
}
