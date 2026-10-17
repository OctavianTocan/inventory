//! Port of `lib/hosted.mjs` and `createSite` of `lib/site.mjs`: one person's library served from its own database and
//! its own part of the object store, as the Cloudflare Worker and `HOSTED=1` serve it.

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::LazyLock;

use async_trait::async_trait;
use mm_js::{Value, text, uri};
use regex::Regex;

use crate::api::{Api, ApiOptions, Push};
use crate::device::device_variant;
use crate::host::{Bucket, ByteSpan, Fetch, ListedObject, MediaProcessor, ObjectInfo, Resource, Screenshot};
use crate::hosted_media::{BucketUploads, HostedMedia};
use crate::http::{Body, Headers, Request, Response};
use crate::media_files::byte_range;
use crate::recorded::Recorded;
use crate::site::{IMAGE_PLACEHOLDER, LOCAL_PREFIXES, LibraryApi, Site, content_type, is_text_extension, localize, replay_key_for, send_range, single_range};
use crate::sql::Sql;
use crate::store::Store;
use crate::{Error, Result, runtime};

static LIBRARY: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z0-9-]+$").expect("pattern"));
static EXTENSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.[^./;]*$").expect("pattern"));
static AREA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)x(\d+)").expect("pattern"));
static VIDEO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\.(mp4|mov|webm)$").expect("pattern"));
static DETAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^/cards/([^/]+)/detail$").expect("pattern"));
static NOT_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_.-]").expect("pattern"));

/// What [`Hosted::new`] serves a library with (`options` of `createHosted`).
pub struct HostedOptions {
    /// SQL handle of the library database; the caller closes it.
    pub db: Rc<dyn Sql>,
    /// Object store holding mymind's website code (`static/`, shared) and every library's own files.
    pub bucket: Rc<dyn Bucket>,
    /// Library name; nothing outside `libraries/<library>/` and `static/` is read.
    pub library: String,
    /// Media processor (the local one, or the media container's client).
    pub processor: Rc<dyn MediaProcessor>,
    /// Broadcaster for committed changes.
    pub push: Push,
    /// Acknowledge writes without persisting them.
    pub readonly: bool,
    /// Never fetch website code that is missing from the store.
    pub offline: bool,
    /// Page screenshots for link capture.
    pub screenshot: Rc<dyn Screenshot>,
    /// Validated HTTP transport for link capture and remote attachments.
    pub resource: Rc<dyn Resource>,
    /// Outbound HTTP for the hosted AI APIs and missing website code.
    pub fetch: Rc<dyn Fetch>,
    /// Random bytes for ids and upload tokens.
    pub random: fn(usize) -> Vec<u8>,
}

/// One library's server (`createHosted`'s result): the library API first, then its recordings and stored files.
pub struct Hosted {
    api: Rc<Api>,
    recorded: Rc<Recorded>,
    bucket: Rc<dyn Bucket>,
    capture: String,
    offline: bool,
    upstream: Rc<dyn Fetch>,
}

fn send(status: u16, body: impl Into<Body>, headers: &[(&str, &str)]) -> Response {
    Response::send(status, body, &Headers::from_pairs(headers.iter().copied()))
}

fn extension(key: &str) -> String {
    EXTENSION.find(key).map_or_else(String::new, |found| found.as_str().to_lowercase())
}

fn text_type(key: &str) -> &'static str {
    content_type(&extension(key)).unwrap_or("application/octet-stream")
}

fn desktop(request: &Request) -> bool {
    device_variant(request.header("user-agent").unwrap_or("")).is_empty()
}

fn document(request: &Request) -> bool {
    request.method == "GET" && request.header("sec-fetch-dest") == Some("document")
}

// A JS value as `String(value)` writes it into a header.
fn header_text(value: &Value) -> String {
    match value {
        Value::Object(_) => "[object Object]".to_owned(),
        other => other.to_js_string(),
    }
}

fn recorded_headers(headers: &Value) -> Headers {
    match headers {
        Value::Object(headers) => Headers::from_pairs(headers.iter().map(|(name, value)| (name.into_owned(), header_text(value)))),
        _ => Headers::new(),
    }
}

// A decoded object key below `folder`, or `None` for a path that climbs out of it.
fn key_for(folder: &str, relative: &str) -> Result<Option<String>> {
    let decoded = uri::decode_uri_component(relative)?;
    if decoded.starts_with('/') || decoded.split('/').any(|part| part == ".." || part == ".") {
        return Ok(None);
    }
    Ok(Some(format!("{folder}{decoded}")))
}

// `name.match(/(\d+)x(\d+)/)?.slice(1).reduce((a, b) => a * b) ?? 0`.
fn area(name: &str) -> f64 {
    AREA.captures(name).map_or(0.0, |found| found[1].parse::<f64>().unwrap_or(f64::INFINITY) * found[2].parse::<f64>().unwrap_or(f64::INFINITY))
}

impl Hosted {
    /// Open a library's server (`createHosted`).
    ///
    /// # Arguments
    /// * `options` - Database, object store, library name and services.
    ///
    /// # Errors
    /// An internal error for a library name other than lower-case letters, digits and dashes; the store's, recordings'
    /// and API's errors.
    pub async fn new(options: HostedOptions) -> Result<Self> {
        if !LIBRARY.is_match(&options.library) {
            return Err(Error::internal("Error: Library names are lower-case letters, digits and dashes"));
        }
        let prefix = format!("libraries/{}/", options.library);
        let recorded = Rc::new(Recorded::new(Rc::clone(&options.db))?);
        let replays = Rc::clone(&recorded);
        let api = Api::new(ApiOptions {
            store: Rc::new(Store::new(options.db)?),
            media: Rc::new(HostedMedia::new(Rc::clone(&options.bucket), &prefix, options.processor)),
            upload_storage: Rc::new(BucketUploads::new(Rc::clone(&options.bucket), &prefix)),
            screenshot: options.screenshot,
            resource: options.resource,
            fetch: Rc::clone(&options.fetch),
            readonly: options.readonly,
            localize,
            push: options.push,
            has_replay: Box::new(move |request| replays.has(request, &replay_key_for(&request.method, &request.headers, &request.url))),
            random: options.random,
        })
        .await?;
        Ok(Self { api: Rc::new(api), recorded, bucket: options.bucket, capture: format!("{prefix}capture/"), offline: options.offline, upstream: options.fetch })
    }

    /// The library API, whose `shell` builds page shells.
    pub fn api(&self) -> &Rc<Api> {
        &self.api
    }

    /// Stop the library's background work (`close`).
    pub fn close(&self) {
        self.api.close();
    }

    // mymind's website code, fetched once from its static origin when it was never mirrored.
    async fn serve_static(&self, relative: &str, search: &str) -> Result<Response> {
        let Some(key) = key_for("static/", relative)? else { return Ok(send(400, "bad path", &[])) };
        if let Some(object) = self.bucket.read(&key, None).await? {
            let mut headers = Headers::from([("cache-control", "no-store"), ("content-type", text_type(&key))]);
            if let Some(stored) = &object.headers {
                headers.extend(stored);
            }
            headers.set("content-length", object.size.to_string());
            return Ok(Response::new(200, headers, Body::Stream(object.body)));
        }
        if self.offline {
            return Ok(send(404, "not mirrored", &[]));
        }
        let response = self.upstream.fetch(Request::new("GET", &format!("{}{relative}{search}", LOCAL_PREFIXES[0].1))?).await?;
        if !(200..300).contains(&response.status) {
            return Ok(send(response.status, format!("upstream {}", response.status), &[]));
        }
        let mut body = response.body.bytes().await?;
        if is_text_extension(&extension(&key)) {
            body = localize(&text::decode_utf8(&body)).into_bytes();
        }
        let length = body.len() as u64;
        self.bucket.write(&key, Body::Bytes(body.clone()), Some(length), None).await?;
        Ok(send(200, body, &[("content-type", text_type(&key))]))
    }

    // The largest captured rendition of the same transformed media (e.g. the 1x of a srcset's 2x), or `None`.
    async fn sibling(&self, key: &str) -> Result<Option<ListedObject>> {
        let stem = &key[..key.rfind(['/', ';']).map_or(0, |at| at + 1)];
        if !stem.contains(';') {
            return Ok(None);
        }
        let folder = &stem[..stem.rfind('/').map_or(0, |at| at + 1)];
        let mut names: Vec<ListedObject> = self.bucket.list(stem).await?.into_iter().filter(|object| !object.key[folder.len()..].contains('/')).collect();
        names.sort_by(|a, b| area(&b.key[folder.len()..]).partial_cmp(&area(&a.key[folder.len()..])).unwrap_or(Ordering::Equal));
        Ok(names.into_iter().next())
    }

    // Stream a stored object, or the span of it a Range header asks for.
    async fn serve_object(&self, request: &Request, key: &str, found: &ObjectInfo) -> Result<Response> {
        let size = found.size;
        let mut headers = Headers::from([("content-type", text_type(key))]);
        if let Some(stored) = &found.headers {
            headers.extend(stored);
        }
        let span = byte_range(single_range(&request.headers), size as f64);
        if span.status == 416 {
            headers.set("accept-ranges", "bytes");
            headers.set("content-range", format!("bytes */{size}"));
            return Ok(Response::send(416, "", &headers));
        }
        let length = (span.end - span.start + 1.0).max(0.0) as u64;
        let object = if length > 0 && request.method != "HEAD" {
            self.bucket.read(key, Some(ByteSpan { offset: span.start as u64, length })).await?
        } else {
            None
        };
        let mut all = Headers::from([("cache-control", "no-store")]);
        all.extend(&headers);
        all.set("content-length", length.to_string());
        if span.status == 206 {
            all.set("accept-ranges", "bytes");
            all.set("content-range", format!("bytes {}-{}/{size}", span.start, span.end));
        }
        Ok(Response::new(span.status, all, object.map_or(Body::Empty, |object| Body::Stream(object.body))))
    }

    // Captured card media, falling back to another size of the same image, then to a placeholder.
    async fn serve_media(&self, request: &Request, folder: &str, relative: &str) -> Result<Response> {
        let key = key_for(folder, relative)?;
        if let Some(key) = &key {
            if let Some(found) = self.bucket.head(key).await? {
                return self.serve_object(request, key, &found).await;
            }
            if let Some(near) = self.sibling(key).await? {
                let found = self.bucket.head(&near.key).await?.ok_or_else(|| Error::invalid("Cannot destructure property 'size' of 'object null' as it is null."))?;
                return self.serve_object(request, &near.key, &found).await;
            }
        }
        runtime::log(&format!("[placeholder] /{}{relative}", &folder[self.capture.len()..]));
        if VIDEO.is_match(relative) {
            return Ok(send(404, "video not captured", &[]));
        }
        Ok(send(200, IMAGE_PLACEHOLDER, &[("content-type", "image/svg+xml")]))
    }

    async fn files(&self, request: &mut Request) -> Result<Response> {
        let url = request.url.clone();
        let pathname = request.pathname().to_owned();
        let search = mm_js::url::search(&url);
        let local = LOCAL_PREFIXES.iter().any(|(prefix, _)| pathname.starts_with(prefix)) || pathname.starts_with("/media/");
        let entry = if local { None } else { self.recorded.entry(request, &replay_key_for(&request.method, &request.headers, &url))? };
        if let Some(entry) = entry {
            let mut body = entry.body;
            if desktop(request) && request.header("sec-fetch-dest") == Some("document") {
                body = self.api.shell(&text::decode_utf8(&body))?.into_bytes();
            }
            return Ok(send_range(&request.headers, entry.status as u16, body, &recorded_headers(&entry.headers)));
        }
        if let Some(relative) = pathname.strip_prefix("/_static/") {
            return self.serve_static(relative, &search).await;
        }
        if let Some(relative) = pathname.strip_prefix("/media/") {
            return self.serve_media(request, &format!("{}media/", self.capture), relative).await;
        }
        if let Some(relative) = pathname.strip_prefix("/_mediahost/") {
            let hosted = format!("{}mediahost/", self.capture);
            let stored = match key_for(&hosted, relative)? {
                Some(key) => self.bucket.head(&key).await?.is_some(),
                None => false,
            };
            let folder = if stored { hosted } else { format!("{}media/", self.capture) };
            return self.serve_media(request, &folder, relative).await;
        }
        if let Some(relative) = pathname.strip_prefix("/_carbon/") {
            return self.serve_media(request, &format!("{}carbon/", self.capture), relative).await;
        }
        if document(request) {
            let everything = url.join("/everything").map_err(|error| Error::internal(error.to_string()))?;
            let shell = match self.recorded.entry(request, &replay_key_for(&request.method, &request.headers, &everything))? {
                Some(entry) => Some(entry),
                None => self.recorded.entry(request, "shell")?,
            };
            let html = text::decode_utf8(shell.as_ref().map_or(&[][..], |entry| &entry.body));
            let body = if desktop(request) { self.api.shell(&html)? } else { html };
            return Ok(send(200, body, &[("content-type", "text/html; charset=utf-8")]));
        }
        if request.method == "GET"
            && let Some(found) = DETAIL.captures(&pathname)
        {
            let id = NOT_WORD.replace_all(&uri::decode_uri_component(&found[1])?, "").into_owned();
            let body = format!(r#"<div style="padding:48px;max-width:720px;margin:auto"><p style="opacity:.6">Detail view not captured for {id}.</p></div>"#);
            return Ok(send(200, body, &[("content-type", "text/html; charset=utf-8")]));
        }
        if request.method != "GET" {
            if pathname != "/jserrors" {
                runtime::log(&format!("[write ignored] {} {pathname}{search}", request.method));
            }
            return Ok(send(200, "{}", &[("content-type", "application/json")]));
        }
        runtime::log(&format!("[miss] {} {pathname}{search} (accept: {})", request.method, request.header("accept").unwrap_or("-")));
        Ok(if request.header("accept").unwrap_or("").contains("json") { send(200, "{}", &[("content-type", "application/json")]) } else { send(404, "", &[("content-type", "text/plain")]) })
    }
}

#[async_trait(?Send)]
impl Site for Hosted {
    async fn handle(&self, request: &mut Request) -> Result<Response> {
        if let Some(response) = self.api.handle(request).await? {
            return Ok(response);
        }
        self.files(request).await
    }
}

