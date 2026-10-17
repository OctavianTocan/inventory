//! What a host gives the library code: an object store, outbound HTTP, media processing and upload storage.

use async_trait::async_trait;

use crate::{Result, Value};
use crate::http::{Body, ByteStream, Headers, Request, Response};

/// What an object store knows about one object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectInfo {
    /// Size in bytes.
    pub size: u64,
    /// Response headers stored with the object, or `None` when it has none.
    pub headers: Option<Headers>,
}

/// A stored object being read.
pub struct StoredObject {
    /// Size of the whole object in bytes.
    pub size: u64,
    /// Response headers stored with the object.
    pub headers: Option<Headers>,
    /// The whole object, or the span that was asked for.
    pub body: ByteStream,
}

/// A span of an object's bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteSpan {
    /// First byte.
    pub offset: u64,
    /// Number of bytes; zero reads nothing.
    pub length: u64,
}

/// One object in a listing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedObject {
    /// Object key.
    pub key: String,
    /// Size in bytes.
    pub size: u64,
}

/// An object store with the operations the hosted server uses: a directory (`lib/bucket-fs.mjs`) or R2
/// (`worker/bucket-r2.mjs`). Keys are `/`-separated paths.
#[async_trait(?Send)]
pub trait Bucket {
    /// What the store knows about an object, or `None` when there is no such object.
    ///
    /// # Arguments
    /// * `key` - Object key.
    ///
    /// # Errors
    /// Status 400 for a key that leaves the store, or the store's error.
    async fn head(&self, key: &str) -> Result<Option<ObjectInfo>>;

    /// An object's bytes, all of them or the span asked for, or `None` when there is no such object.
    ///
    /// # Arguments
    /// * `key` - Object key.
    /// * `range` - Span to read; `None` reads the whole object.
    ///
    /// # Errors
    /// As [`Bucket::head`].
    async fn read(&self, key: &str, range: Option<ByteSpan>) -> Result<Option<StoredObject>>;

    /// An object's whole contents, or `None` when there is no such object.
    ///
    /// # Arguments
    /// * `key` - Object key.
    ///
    /// # Errors
    /// As [`Bucket::head`].
    async fn bytes(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Store an object atomically, replacing any object with the same key.
    ///
    /// # Arguments
    /// * `key` - Object key.
    /// * `body` - Contents.
    /// * `length` - Size in bytes when known; R2 needs it for a streamed body.
    /// * `headers` - Response headers to keep with the object; `None` removes stored ones.
    ///
    /// # Errors
    /// As [`Bucket::head`], or the body's error.
    async fn write(&self, key: &str, body: Body, length: Option<u64>, headers: Option<&Headers>) -> Result<()>;

    /// Delete an object; deleting a missing object succeeds.
    ///
    /// # Arguments
    /// * `key` - Object key.
    ///
    /// # Errors
    /// As [`Bucket::head`].
    async fn remove(&self, key: &str) -> Result<()>;

    /// Every object whose key starts with `prefix`, sorted by key in byte order.
    ///
    /// # Arguments
    /// * `prefix` - Key prefix; empty lists everything.
    ///
    /// # Errors
    /// The store's error.
    async fn list(&self, prefix: &str) -> Result<Vec<ListedObject>>;
}

/// Outbound HTTP, as the global `fetch` the JS backend called (AI services, Google sign-in, mymind's static origin).
#[async_trait(?Send)]
pub trait Fetch {
    /// Send a request and resolve its response once the headers arrive; redirects are followed.
    ///
    /// # Arguments
    /// * `request` - Method, absolute URL, headers and body.
    ///
    /// # Errors
    /// A network failure, as `fetch` rejects with a `TypeError`.
    async fn fetch(&self, request: Request) -> Result<Response>;

    /// Send a request without following redirects (`redirect: 'manual'`): a redirect resolves as the 3xx response
    /// itself. Hosts that cannot tell the two apart may follow them.
    ///
    /// # Arguments
    /// * `request` - Method, absolute URL, headers and body.
    ///
    /// # Errors
    /// A network failure.
    async fn fetch_unfollowed(&self, request: Request) -> Result<Response> {
        self.fetch(request).await
    }
}

/// What `prepare` reads a new media file from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaInput {
    /// The bytes themselves.
    Bytes(Vec<u8>),
    /// A stored object (a card's original), by key.
    Key(String),
    /// Chunks of a finished upload: the key prefix `UploadStorage::input` names, holding one object per chunk.
    Upload(String),
    /// A file on the host's disk (`{ file }`); only the directory-backed media reads it.
    File(String),
}

/// The account a prepared file is stored for (`user` of `createMedia().prepare`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaUser {
    /// Account directory name below the media root (`accountUid`).
    pub account_uid: String,
    /// Prefix of the library's card ids (`idPrefix`).
    pub id_prefix: String,
}

/// The card media fields a processor derives from an uploaded file.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedMedia {
    /// Card `data` fields (`type`, `title`, `blob`, and `format` for a document); `blob.path` is where the original is
    /// stored.
    pub data: Value,
    /// Card `extras` fields (`cover`).
    pub extras: Value,
}

impl PreparedMedia {
    /// The `{ data, extras }` object `createMedia().prepare` resolves.
    pub fn to_value(&self) -> Value {
        crate::obj! { "data" => self.data.clone(), "extras" => self.extras.clone() }.into()
    }
}

/// The stored original a rendition is made from.
pub struct RenditionSource {
    /// The original's bytes.
    pub body: Body,
    /// Size of the original in bytes.
    pub size: u64,
    /// The original's path below the library's media folder, such as `<account>/<card>/<blob>.jpg`.
    pub name: String,
}

/// A rendition a processor made.
pub struct RenditionOutput {
    /// The rendition's bytes.
    pub body: Body,
    /// Size in bytes when the processor knows it; otherwise the length of a bytes body.
    pub size: Option<u64>,
}

/// An image downscaled for the AI analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisImage {
    /// Content type (`image/png`).
    pub mime: String,
    /// The image bytes.
    pub data: Vec<u8>,
}

/// The image, video and document work behind the hosted media: sharp, ffmpeg and pdftoppm on a native host, the media
/// container on Cloudflare (`lib/media-processor.mjs`, `worker/media-client.mjs`).
#[async_trait(?Send)]
pub trait MediaProcessor {
    /// Identify a new file and derive the card media fields; the caller stores the original at `data.blob.path`.
    ///
    /// # Arguments
    /// * `body` - The file's bytes.
    /// * `content_type` - The type the client claimed; the processor sniffs the real one.
    /// * `name` - Original file name.
    /// * `user` - Account the media is stored for.
    /// * `id` - Id of the card being made.
    ///
    /// # Errors
    /// Status 415 for unsupported media, or the processor's error.
    async fn prepare(&self, body: Body, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia>;

    /// Make the rendition `relative` names of a stored original.
    ///
    /// # Arguments
    /// * `original` - The original and where it is stored.
    /// * `relative` - Media path with its `;` rendition suffix.
    ///
    /// # Returns
    /// The rendition, or `None` when `relative` is not a rendition of the original.
    ///
    /// # Errors
    /// The processor's error.
    async fn rendition(&self, original: RenditionSource, relative: &str) -> Result<Option<RenditionOutput>>;

    /// Downscale an image to the PNG the AI analysis reads.
    ///
    /// # Arguments
    /// * `body` - The image's bytes.
    ///
    /// # Errors
    /// The processor's error.
    async fn image(&self, body: Body) -> Result<AnalysisImage>;
}

/// Where chunked uploads keep their chunks: a private directory (`directoryStorage` of `lib/uploads.mjs`) or an object
/// store ([`crate::hosted_media::BucketUploads`]). Upload ids are 22 hex characters.
#[async_trait(?Send)]
pub trait UploadStorage {
    /// Name that identifies this storage, so uploads of one storage share one coordinator.
    fn key(&self) -> String;

    /// The ids of every upload that has stored chunks.
    ///
    /// # Errors
    /// The store's error.
    async fn names(&self) -> Result<Vec<String>>;

    /// Bytes stored for an upload, or `None` when it has no chunks.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    ///
    /// # Errors
    /// The store's error.
    async fn size(&self, id: &str) -> Result<Option<u64>>;

    /// Drop everything stored at or after byte `size`.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    /// * `size` - Bytes to keep.
    ///
    /// # Errors
    /// The store's error.
    async fn truncate(&self, id: &str, size: u64) -> Result<()>;

    /// Delete everything stored for an upload.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    ///
    /// # Errors
    /// The store's error.
    async fn remove(&self, id: &str) -> Result<()>;

    /// What `HostedMedia::prepare` reads the finished upload from.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    fn input(&self, id: &str) -> MediaInput;

    /// How a claimed upload's permanent original is read: the object key for an object store, a file path for a
    /// directory.
    ///
    /// # Arguments
    /// * `found` - What [`Media::response`] found for the original.
    fn claimed_input(&self, found: &MediaResult) -> MediaInput {
        MediaInput::Key(found.key.clone())
    }

    /// Store a chunk at byte offset `start`.
    ///
    /// # Arguments
    /// * `id` - Upload id.
    /// * `start` - Offset of the chunk's first byte.
    /// * `chunks` - The chunk's bytes.
    /// * `length` - The chunk's size in bytes.
    ///
    /// # Errors
    /// The store's error or the body's.
    async fn write(&self, id: &str, start: u64, chunks: Body, length: Option<u64>) -> Result<()>;
}

/// A stored original or rendition, as [`Media::response`] finds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaResult {
    /// Where the bytes are: an object key in the store, or a file path for media on disk.
    pub key: String,
    /// Size in bytes.
    pub size: u64,
    /// Content type to serve it with.
    pub content_type: String,
}

impl MediaResult {
    /// The `{ key, size, type }` object `response` resolves.
    pub fn to_value(&self) -> Value {
        crate::obj! { "key" => self.key.as_str(), "size" => self.size as f64, "type" => self.content_type.as_str() }.into()
    }
}

/// A library's media handle, on disk (`createMedia`, `lib/local-media.mjs`) or in an object store
/// (`createHostedMedia`): what uploads, writes, enrichment and the API call.
#[async_trait(?Send)]
pub trait Media {
    /// Fit a stored original's intrinsic dimensions to the 800 by 1000 cover box (`cover`).
    ///
    /// # Arguments
    /// * `blob` - Stored original's metadata.
    ///
    /// # Errors
    /// `blob` is null or undefined.
    fn cover(&self, blob: &Value) -> Result<Value>;

    /// Identify a new file, store the original and return the card media fields (`prepare`).
    ///
    /// # Arguments
    /// * `input` - The bytes, a stored object, a finished chunked upload, or a file on disk.
    /// * `content_type` - The type the client claimed.
    /// * `name` - Original file name.
    /// * `user` - Account the media is stored for.
    /// * `id` - Id of the card being made.
    ///
    /// # Errors
    /// Status errors for missing or unsupported input, and 422 `Media preview could not be generated` otherwise.
    async fn prepare(&self, input: MediaInput, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia>;

    /// The response for a found original or rendition, or the span of it a `Range` header asks for (`send`).
    ///
    /// # Arguments
    /// * `result` - What [`Media::response`] found.
    /// * `method` - Request method; `HEAD` sends no body.
    /// * `range` - The request's `Range` header.
    ///
    /// # Errors
    /// The storage's error.
    async fn send(&self, result: &MediaResult, method: &str, range: Option<&str>) -> Result<Response>;

    /// The original or rendition a media path names, making the rendition once (`response`).
    ///
    /// # Arguments
    /// * `relative` - Path below `/media/`, decoded, with an optional `;` rendition suffix.
    ///
    /// # Returns
    /// `None` when the library has no such original or no such rendition.
    ///
    /// # Errors
    /// Status 400 for a malformed suffix, and 422 `Media rendition unavailable` for a failed rendition.
    async fn response(&self, relative: &str) -> Result<Option<MediaResult>>;

    /// The analysis image of a card's blob or cover (`image`).
    ///
    /// # Arguments
    /// * `blob_or_cover` - A stored `blob` (`path`) or a `cover` (`src`); a falsy value has no image.
    ///
    /// # Returns
    /// `None` when there is no such media.
    ///
    /// # Errors
    /// As [`Media::response`], or the processor's error.
    async fn image(&self, blob_or_cover: &Value) -> Result<Option<AnalysisImage>>;
}

/// What [`Resource::fetch`] was asked for (`{ offline, limit }` of `fetchResource`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceOptions {
    /// Allow loopback fixture hosts and addresses only.
    pub offline: bool,
    /// Largest accepted size in bytes, on the wire and decoded; `None` is 8 MiB.
    pub limit: Option<u64>,
}

impl ResourceOptions {
    /// The size limit in bytes (`limit = 8 * 1024 * 1024`).
    pub fn limit_bytes(&self) -> u64 {
        self.limit.unwrap_or(8 * 1024 * 1024)
    }
}

/// A fetched resource: the object `fetchResource` resolves, `{ bytes, url, mime, status, headers }`.
#[derive(Clone, Debug, PartialEq)]
pub struct FetchedResource {
    /// The decoded body.
    pub bytes: Vec<u8>,
    /// The final URL after redirects.
    pub url: String,
    /// The `content-type` header, or empty.
    pub mime: String,
    /// The final response status.
    pub status: u16,
    /// The response headers without `content-length`, `content-encoding`, `transfer-encoding` and `set-cookie`.
    pub headers: Headers,
}

impl FetchedResource {
    /// The object `fetchResource` resolved, with its key order.
    pub fn to_value(&self) -> Value {
        crate::obj! {
            "bytes" => Value::Bytes(self.bytes.clone()),
            "url" => self.url.as_str(),
            "mime" => self.mime.as_str(),
            "status" => f64::from(self.status),
            "headers" => Value::Object(self.headers.iter().map(|(name, value)| (name.to_owned(), Value::from(value))).collect()),
        }
        .into()
    }
}

/// The validated HTTP transport behind link capture (`fetchResource` of `lib/capture-http.mjs`): the native transport
/// on a native host, the media container's `resource` operation on Cloudflare.
#[async_trait(?Send)]
pub trait Resource {
    /// Fetch a public URL with the capture destination policy applied to the URL, every DNS answer and every redirect.
    ///
    /// # Arguments
    /// * `url` - Absolute HTTP(S) URL.
    /// * `options` - Loopback-only mode and the size limit.
    ///
    /// # Errors
    /// A `CAPTURE_BLOCKED` status 400 error for a rejected destination (see
    /// [`crate::capture_http::is_blocked`]); any other error for unavailable, oversized or undecodable resources.
    async fn fetch(&self, url: &str, options: ResourceOptions) -> Result<FetchedResource>;
}

/// Page screenshots behind link capture (`screenshotPage` of `lib/capture-link.mjs`): headless Chromium on a native
/// host, the media container's `screenshot` operation on Cloudflare.
#[async_trait(?Send)]
pub trait Screenshot {
    /// Render a public page in a fresh 1400 by 940 browser and screenshot it.
    ///
    /// # Arguments
    /// * `url` - Absolute HTTP(S) page URL.
    /// * `offline` - Restrict every browser resource to loopback.
    ///
    /// # Returns
    /// The PNG bytes.
    ///
    /// # Errors
    /// A `CAPTURE_BLOCKED` error when the page or one of its resources was rejected, or the browser's error.
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>>;
}
