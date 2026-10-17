//! Port of `lib/hosted-media.mjs`: a library's media kept in an object store and served with the operations of
//! `createMedia` (`lib/local-media.mjs`), leaving the image and video work to a [`MediaProcessor`].

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::rc::Rc;

use async_trait::async_trait;
use futures_util::future::{FutureExt, LocalBoxFuture, Shared};
use futures_util::{StreamExt, stream};
use mm_js::Value;
use mm_js::number::{number_to_string, string_to_number};

pub use crate::host::MediaResult;
use crate::host::{
    AnalysisImage, Bucket, ByteSpan, ListedObject, Media, MediaInput, MediaProcessor, MediaUser, PreparedMedia, RenditionSource,
    UploadStorage,
};
use crate::http::{Body, ByteStream, Headers, Response};
use crate::media_files::{Rendition, byte_range, cover as media_cover, mime, parse_rendition};
use crate::render::access::member;
use crate::{Error, Result};

type Pending = Shared<LocalBoxFuture<'static, Result<Option<MediaResult>>>>;

struct Inner {
    bucket: Rc<dyn Bucket>,
    prefix: String,
    processor: Rc<dyn MediaProcessor>,
}

/// A library's media in an object store (`createHostedMedia`).
pub struct HostedMedia {
    inner: Rc<Inner>,
    pending: Rc<RefCell<HashMap<String, Pending>>>,
}

fn safe(relative: &str) -> bool {
    !relative.starts_with('/') && !relative.split('/').any(|part| part == ".." || part == ".")
}

fn content_type(key: &str) -> String {
    let extension = key.rsplit('/').next().unwrap_or("").rsplit('.').next().unwrap_or("");
    mime(extension).unwrap_or("application/octet-stream").to_owned()
}

fn missing_body() -> Error {
    Error::invalid("Cannot read properties of null (reading 'body')")
}

fn with_status(error: Error, message: &str) -> Error {
    if matches!(error, Error::Status { .. }) { error } else { Error::status(422, message) }
}

/// One stream of several stored objects, read in order.
fn concat(bucket: Rc<dyn Bucket>, keys: Vec<String>) -> ByteStream {
    struct State {
        bucket: Rc<dyn Bucket>,
        keys: VecDeque<String>,
        current: Option<ByteStream>,
    }
    let state = State { bucket, keys: keys.into(), current: None };
    Box::pin(stream::unfold(state, |mut state| async move {
        loop {
            if let Some(current) = state.current.as_mut() {
                match current.next().await {
                    Some(Ok(chunk)) => return Some((Ok(chunk), state)),
                    Some(Err(error)) => {
                        state.current = None;
                        state.keys.clear();
                        return Some((Err(error), state));
                    }
                    None => state.current = None,
                }
            }
            let key = state.keys.pop_front()?;
            match state.bucket.read(&key, None).await {
                Ok(Some(object)) => state.current = Some(object.body),
                Ok(None) => {
                    state.keys.clear();
                    return Some((Err(Error::internal("Upload part is missing")), state));
                }
                Err(error) => {
                    state.keys.clear();
                    return Some((Err(error), state));
                }
            }
        }
    }))
}

enum Open {
    Bytes(Vec<u8>),
    Key(String),
    Parts(Vec<String>),
}

struct Source {
    size: u64,
    open: Open,
}

impl Inner {
    fn media_key(&self, relative: &str) -> String {
        format!("{}media/{relative}", self.prefix)
    }

    /// The stored original for a media path, including extensionless recorded blob URLs.
    async fn original(&self, source: &str) -> Result<Option<MediaResult>> {
        if !safe(source) {
            return Ok(None);
        }
        let key = self.media_key(source);
        if let Some(exact) = self.bucket.head(&key).await? {
            return Ok(Some(MediaResult { content_type: content_type(source), size: exact.size, key }));
        }
        let folder = self.media_key(&source[..source.rfind('/').map_or(0, |index| index + 1)]);
        let found = self
            .bucket
            .list(&format!("{key}."))
            .await?
            .into_iter()
            .find(|object| !object.key[folder.len()..].contains('/') && !object.key.contains(".page-"));
        Ok(found.map(|object| MediaResult { content_type: content_type(&object.key), size: object.size, key: object.key }))
    }

    /// The original, or a rendition made once and then kept beside the library's media.
    async fn rendition(&self, relative: &str) -> Result<Option<MediaResult>> {
        let Some(found) = self.original(relative.split(';').next().unwrap_or("")).await? else { return Ok(None) };
        let wanted = match parse_rendition(relative)? {
            Rendition::Original { .. } => return Ok(Some(found)),
            Rendition::Derived { content_type, .. } => content_type,
        };
        let key = format!("{}renditions/{}", self.prefix, relative.replace('/', "_"));
        if let Some(cached) = self.bucket.head(&key).await? {
            return Ok(Some(MediaResult { key, size: cached.size, content_type: wanted }));
        }
        let object = self.bucket.read(&found.key, None).await?.ok_or_else(missing_body)?;
        let name = found.key[self.media_key("").len()..].to_owned();
        let original = RenditionSource { body: Body::Stream(object.body), size: object.size, name };
        let Some(made) = self.processor.rendition(original, relative).await? else { return Ok(None) };
        let length = match (&made.body, made.size) {
            (_, Some(size)) => Some(size),
            (Body::Bytes(bytes), None) => Some(bytes.len() as u64),
            _ => None,
        };
        self.bucket.write(&key, made.body, length, None).await?;
        let stored = self.bucket.head(&key).await?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'size')"))?;
        Ok(Some(MediaResult { key, size: stored.size, content_type: wanted }))
    }

    /// A body and its size for each kind of input `prepare` accepts.
    async fn source(&self, input: MediaInput) -> Result<Source> {
        match input {
            MediaInput::Bytes(bytes) => Ok(Source { size: bytes.len() as u64, open: Open::Bytes(bytes) }),
            MediaInput::Key(key) if !key.is_empty() => {
                let info = self.bucket.head(&key).await?.ok_or_else(|| Error::status(404, "Upload media is unavailable"))?;
                Ok(Source { size: info.size, open: Open::Key(key) })
            }
            MediaInput::Upload(upload) if !upload.is_empty() => {
                let mut parts = self.bucket.list(&upload).await?;
                let order = |part: &ListedObject| string_to_number(&part.key[upload.len()..]);
                parts.sort_by(|a, b| (order(a) - order(b)).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal));
                let size = parts.iter().map(|part| part.size).sum();
                Ok(Source { size, open: Open::Parts(parts.into_iter().map(|part| part.key).collect()) })
            }
            _ => Err(Error::status(415, "Unsupported media input")),
        }
    }

    async fn open(&self, source: &Source) -> Result<Body> {
        match &source.open {
            Open::Bytes(bytes) => Ok(Body::Bytes(bytes.clone())),
            Open::Key(key) => Ok(Body::Stream(self.bucket.read(key, None).await?.ok_or_else(missing_body)?.body)),
            Open::Parts(keys) => Ok(Body::Stream(concat(self.bucket.clone(), keys.clone()))),
        }
    }

    /// Identify the saved bytes, keep the original and return the card media fields.
    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let bytes = self.source(input).await?;
        let prepared = self.processor.prepare(self.open(&bytes).await?, content_type, name, user, id).await?;
        let path = member(member(&prepared.data, "blob")?, "path")?.to_js_string();
        self.bucket.write(&self.media_key(&path), self.open(&bytes).await?, Some(bytes.size), None).await?;
        Ok(prepared)
    }
}

impl HostedMedia {
    /// Serve a library's media from an object store (`createHostedMedia`).
    ///
    /// # Arguments
    /// * `bucket` - Object store; originals live below `<prefix>media/` and cached renditions below
    ///   `<prefix>renditions/`.
    /// * `prefix` - Key prefix of this library, such as `libraries/owner/`; empty for none.
    /// * `processor` - Media processor for new files, renditions and analysis images.
    pub fn new(bucket: Rc<dyn Bucket>, prefix: &str, processor: Rc<dyn MediaProcessor>) -> Self {
        Self { inner: Rc::new(Inner { bucket, prefix: prefix.to_owned(), processor }), pending: Rc::default() }
    }

    /// Fit intrinsic dimensions to the cover bounding box (`cover`, 800 by 1000).
    ///
    /// # Arguments
    /// * `blob` - Stored original's metadata.
    ///
    /// # Errors
    /// `blob` is null or undefined.
    pub fn cover(blob: &Value) -> Result<Value> {
        media_cover(blob, 800.0, 1000.0)
    }

    /// Identify uploaded bytes, store the original and return the card media fields (`prepare`).
    ///
    /// # Arguments
    /// * `input` - The bytes, a stored object, or a finished chunked upload.
    /// * `content_type` - The type the client claimed.
    /// * `name` - Original file name.
    /// * `user` - Account the media is stored for.
    /// * `id` - Id of the card being made.
    ///
    /// # Errors
    /// Status 404 for a missing stored object, 415 for an unsupported input, the processor's status errors, and 422
    /// `Media preview could not be generated` for any other failure.
    pub async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        self.inner.prepare(input, content_type, name, user, id).await.map_err(|error| with_status(error, "Media preview could not be generated"))
    }

    /// The response for a stored object or a span of it (`send`).
    ///
    /// # Arguments
    /// * `result` - What [`HostedMedia::response`] found.
    /// * `method` - Request method; `HEAD` sends no body.
    /// * `range` - The request's `Range` header.
    ///
    /// # Returns
    /// A 200, 206 or 416 response whose body streams from the store.
    ///
    /// # Errors
    /// The store's error.
    pub async fn send(&self, result: &MediaResult, method: &str, range: Option<&str>) -> Result<Response> {
        let mut headers = Headers::from([("cache-control", "no-store"), ("content-type", result.content_type.as_str()), ("accept-ranges", "bytes")]);
        let span = byte_range(range, result.size as f64);
        if span.status == 416 {
            headers.set("content-range", format!("bytes */{}", result.size));
            headers.set("content-length", "0");
            return Ok(Response::new(416, headers, Body::Empty));
        }
        if span.status == 206 {
            headers.set("content-range", format!("bytes {}-{}/{}", number_to_string(span.start), number_to_string(span.end), result.size));
        }
        let length = (span.end - span.start + 1.0).max(0.0);
        headers.set("content-length", number_to_string(length));
        let object = if method == "HEAD" || result.size == 0 {
            None
        } else {
            self.inner.bucket.read(&result.key, Some(ByteSpan { offset: span.start as u64, length: length as u64 })).await?
        };
        Ok(Response::new(span.status, headers, object.map_or(Body::Empty, |object| Body::Stream(object.body))))
    }

    /// The stored original or rendition a media path names (`response`); concurrent calls for one path share one
    /// rendition.
    ///
    /// # Arguments
    /// * `relative` - Path below `/media/`, decoded, with an optional `;` rendition suffix.
    ///
    /// # Returns
    /// `None` when the library has no such original, or the processor has no such rendition.
    ///
    /// # Errors
    /// Status 400 `Invalid media rendition` for a malformed suffix, the processor's status errors, and 422
    /// `Media rendition unavailable` for any other failure.
    pub fn response(&self, relative: &str) -> impl Future<Output = Result<Option<MediaResult>>> + use<> {
        let mut pending = self.pending.borrow_mut();
        pending
            .entry(relative.to_owned())
            .or_insert_with(|| {
                let (inner, table, relative) = (self.inner.clone(), self.pending.clone(), relative.to_owned());
                async move {
                    let outcome = inner.rendition(&relative).await.map_err(|error| with_status(error, "Media rendition unavailable"));
                    table.borrow_mut().remove(&relative);
                    outcome
                }
                .boxed_local()
                .shared()
            })
            .clone()
    }

    /// The analysis image of a card's blob or cover (`image`).
    ///
    /// # Arguments
    /// * `blob_or_cover` - A stored `blob` (`path`) or a `cover` (`src`); a falsy value has no image.
    ///
    /// # Returns
    /// `None` when there is no such media.
    ///
    /// # Errors
    /// As [`HostedMedia::response`], or the processor's error.
    pub async fn image(&self, blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        if !blob_or_cover.truthy() {
            return Ok(None);
        }
        let relative = match blob_or_cover.get("src") {
            Value::Null | Value::Undefined => format!("{};800x1000-t.webp", blob_or_cover.get("path").to_js_string()),
            Value::String(src) => src.strip_prefix("/media/").unwrap_or(src).to_owned(),
            _ => return Err(Error::invalid("blobOrCover.src?.replace is not a function")),
        };
        let Some(result) = self.response(&relative).await? else { return Ok(None) };
        let object = self.inner.bucket.read(&result.key, None).await?.ok_or_else(missing_body)?;
        Ok(Some(self.inner.processor.image(Body::Stream(object.body)).await?))
    }
}

#[async_trait(?Send)]
impl Media for HostedMedia {
    fn cover(&self, blob: &Value) -> Result<Value> {
        HostedMedia::cover(blob)
    }

    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        HostedMedia::prepare(self, input, content_type, name, user, id).await
    }

    async fn send(&self, result: &MediaResult, method: &str, range: Option<&str>) -> Result<Response> {
        HostedMedia::send(self, result, method, range).await
    }

    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        HostedMedia::response(self, relative).await
    }

    async fn image(&self, blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        HostedMedia::image(self, blob_or_cover).await
    }
}

/// Chunked uploads kept in an object store, one object per chunk (`bucketUploads`).
pub struct BucketUploads {
    bucket: Rc<dyn Bucket>,
    prefix: String,
    folder: String,
}

impl BucketUploads {
    /// Chunk storage below `<prefix>uploads/<id>/<offset>`.
    ///
    /// # Arguments
    /// * `bucket` - Object store.
    /// * `prefix` - Key prefix of this library; empty for none.
    pub fn new(bucket: Rc<dyn Bucket>, prefix: &str) -> Self {
        Self { bucket, prefix: prefix.to_owned(), folder: format!("{prefix}uploads/") }
    }

    async fn parts(&self, id: &str) -> Result<Vec<ListedObject>> {
        self.bucket.list(&format!("{}{id}/", self.folder)).await
    }
}

#[async_trait(?Send)]
impl UploadStorage for BucketUploads {
    fn key(&self) -> String {
        format!("bucket:{}", self.prefix)
    }

    async fn names(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = Vec::new();
        for object in self.bucket.list(&self.folder).await? {
            let name = object.key[self.folder.len()..].split('/').next().unwrap_or("").to_owned();
            if !names.contains(&name) {
                names.push(name);
            }
        }
        Ok(names)
    }

    async fn size(&self, id: &str) -> Result<Option<u64>> {
        let found = self.parts(id).await?;
        Ok((!found.is_empty()).then(|| found.iter().map(|part| part.size).sum()))
    }

    async fn truncate(&self, id: &str, size: u64) -> Result<()> {
        for part in self.parts(id).await? {
            if string_to_number(part.key.rsplit('/').next().unwrap_or("")) >= size as f64 {
                self.bucket.remove(&part.key).await?;
            }
        }
        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<()> {
        for part in self.parts(id).await? {
            self.bucket.remove(&part.key).await?;
        }
        Ok(())
    }

    fn input(&self, id: &str) -> MediaInput {
        MediaInput::Upload(format!("{}{id}/", self.folder))
    }

    async fn write(&self, id: &str, start: u64, chunks: Body, length: Option<u64>) -> Result<()> {
        self.bucket.write(&format!("{}{id}/{start:012}", self.folder), chunks, length, None).await
    }
}
