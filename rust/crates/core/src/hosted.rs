//! Port of `lib/hosted.mjs` and `createSite` of `lib/site.mjs`: one person's library served from its own database and
//! its own part of the object store, as the Cloudflare Worker and `HOSTED=1` serve it.

use std::rc::Rc;
use std::sync::LazyLock;

use async_trait::async_trait;
use mm_js::{text, uri};
use regex::Regex;

use crate::api::{Api, ApiOptions, Push};
use crate::host::{Bucket, Fetch, MediaProcessor, Resource, Screenshot};
use crate::hosted_media::{BucketUploads, HostedMedia};
use crate::http::{Body, Headers, Request, Response};
use crate::site::{LOCAL_PREFIXES, LibraryApi, Site, content_type, is_text_extension, localize};
use crate::sql::Sql;
use crate::store::Store;
use crate::{Error, Result, runtime};

static LIBRARY: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z0-9-]+$").expect("pattern"));
static EXTENSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.[^./;]*$").expect("pattern"));

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
    bucket: Rc<dyn Bucket>,
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

// A decoded object key below `folder`, or `None` for a path that climbs out of it.
fn key_for(folder: &str, relative: &str) -> Result<Option<String>> {
    let decoded = uri::decode_uri_component(relative)?;
    if decoded.starts_with('/') || decoded.split('/').any(|part| part == ".." || part == ".") {
        return Ok(None);
    }
    Ok(Some(format!("{folder}{decoded}")))
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
            random: options.random,
        })
        .await?;
        Ok(Self { api: Rc::new(api), bucket: options.bucket, offline: options.offline, upstream: options.fetch })
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

    async fn files(&self, request: &mut Request) -> Result<Response> {
        let url = request.url.clone();
        let pathname = request.pathname().to_owned();
        let search = mm_js::url::search(&url);
        if let Some(relative) = pathname.strip_prefix("/_static/") {
            return self.serve_static(relative, &search).await;
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

