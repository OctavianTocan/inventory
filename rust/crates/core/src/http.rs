//! Requests and responses in place of Node's `req` and `res`, shared by the native server and the Worker.

use std::any::Any;
use std::pin::Pin;

use futures_core::Stream;
use futures_util::StreamExt;
use url::Url;

use crate::{Error, Result};

/// A body delivered in chunks.
pub type ByteStream = Pin<Box<dyn BodyStream>>;

/// Any stream of body chunks, which a host can recognise again as one of its own.
pub trait BodyStream: Stream<Item = Result<Vec<u8>>> {
    /// The stream as its concrete type, for a host to hand its own unread body back to its runtime.
    ///
    /// # Returns
    /// The stream itself, as `Any`.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Stream<Item = Result<Vec<u8>>> + 'static> BodyStream for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Header names and values in the order they were set, as the JS backend's header objects held them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Headers(Vec<(String, String)>);

impl Headers {
    /// No headers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Headers from name and value pairs, a later name replacing an earlier one in its place.
    ///
    /// # Arguments
    /// * `pairs` - Names and values in order.
    pub fn from_pairs<K: Into<String>, V: Into<String>>(pairs: impl IntoIterator<Item = (K, V)>) -> Self {
        let mut headers = Self::new();
        for (name, value) in pairs {
            headers.set(name, value);
        }
        headers
    }

    /// The value of the header with exactly this name. Request header names are lower case.
    ///
    /// # Arguments
    /// * `name` - Header name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }

    /// Whether a header with exactly this name is set.
    ///
    /// # Arguments
    /// * `name` - Header name.
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Set a header as an object property is assigned: an existing name keeps its place and takes the new value.
    ///
    /// # Arguments
    /// * `name` - Header name.
    /// * `value` - Header value.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let (name, value) = (name.into(), value.into());
        match self.0.iter_mut().find(|(key, _)| *key == name) {
            Some(entry) => entry.1 = value,
            None => self.0.push((name, value)),
        }
    }

    /// Add a header after the existing ones even when its name is already set, as a header the JS code wrote as an array
    /// (`set-cookie`) is sent once per value.
    ///
    /// # Arguments
    /// * `name` - Header name.
    /// * `value` - Header value.
    pub fn append(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.push((name.into(), value.into()));
    }

    /// Remove a header, returning its value.
    ///
    /// # Arguments
    /// * `name` - Header name.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        let index = self.0.iter().position(|(key, _)| key == name)?;
        Some(self.0.remove(index).1)
    }

    /// Set every header of `other` in order, as `{...this, ...other}` does.
    ///
    /// # Arguments
    /// * `other` - Headers that win over these.
    pub fn extend(&mut self, other: &Headers) {
        for (name, value) in other.iter() {
            self.set(name, value);
        }
    }

    /// These headers with `other` spread over them.
    ///
    /// # Arguments
    /// * `other` - Headers that win over these.
    #[must_use]
    pub fn with(mut self, other: &Headers) -> Self {
        self.extend(other);
        self
    }

    /// Names and values in order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Number of headers.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no header is set.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: Into<String>, V: Into<String>, const N: usize> From<[(K, V); N]> for Headers {
    fn from(pairs: [(K, V); N]) -> Self {
        Self::from_pairs(pairs)
    }
}

/// A request or response body.
#[derive(Default)]
pub enum Body {
    /// No body.
    #[default]
    Empty,
    /// The whole body.
    Bytes(Vec<u8>),
    /// A body still arriving.
    Stream(ByteStream),
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("Body::Empty"),
            Self::Bytes(bytes) => write!(f, "Body::Bytes({} bytes)", bytes.len()),
            Self::Stream(_) => f.write_str("Body::Stream"),
        }
    }
}

impl Body {
    /// The whole body, read to its end.
    ///
    /// # Errors
    /// The stream's error.
    pub async fn bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Empty => Ok(Vec::new()),
            Self::Bytes(bytes) => Ok(bytes),
            Self::Stream(mut stream) => {
                let mut out = Vec::new();
                while let Some(chunk) = stream.next().await {
                    out.extend_from_slice(&chunk?);
                }
                Ok(out)
            }
        }
    }

    /// The whole body, read to its end unless it grows past `limit` bytes.
    ///
    /// # Arguments
    /// * `limit` - Largest accepted size in bytes.
    ///
    /// # Errors
    /// Status 413 `Request body too large` past the limit, or the stream's error.
    pub async fn bytes_within(self, limit: usize) -> Result<Vec<u8>> {
        let too_large = || Error::status(413, "Request body too large");
        match self {
            Self::Empty => Ok(Vec::new()),
            Self::Bytes(bytes) if bytes.len() > limit => Err(too_large()),
            Self::Bytes(bytes) => Ok(bytes),
            Self::Stream(mut stream) => {
                let mut out = Vec::new();
                while let Some(chunk) = stream.next().await {
                    out.extend_from_slice(&chunk?);
                    if out.len() > limit {
                        return Err(too_large());
                    }
                }
                Ok(out)
            }
        }
    }

    /// The body as a stream of chunks.
    pub fn into_stream(self) -> ByteStream {
        match self {
            Self::Empty => Box::pin(futures_util::stream::empty()),
            Self::Bytes(bytes) => Box::pin(futures_util::stream::once(async move { Ok(bytes) })),
            Self::Stream(stream) => stream,
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<&[u8]> for Body {
    fn from(bytes: &[u8]) -> Self {
        Self::Bytes(bytes.to_vec())
    }
}

impl From<String> for Body {
    fn from(text: String) -> Self {
        Self::Bytes(text.into_bytes())
    }
}

impl From<&str> for Body {
    fn from(text: &str) -> Self {
        Self::Bytes(text.as_bytes().to_vec())
    }
}

/// An incoming request, as Node's `req` presented it.
#[derive(Debug)]
pub struct Request {
    /// Upper-case method.
    pub method: String,
    /// The request target as sent (`req.url`).
    pub target: String,
    /// The full URL, its origin taken from the Host header (`new URL(req.url, `http://${host}`)`).
    pub url: Url,
    /// Headers with lower-case names; repeated headers joined as Node joins them.
    pub headers: Headers,
    /// Address of the connecting client (`req.socket.remoteAddress`).
    pub remote_address: String,
    /// Port the server accepted the connection on (`req.socket.localPort`).
    pub local_port: u16,
    /// The body, read once.
    pub body: Body,
}

impl Request {
    /// A request with no body from a test or another object of the same Worker.
    ///
    /// # Arguments
    /// * `method` - Upper-case method.
    /// * `url` - Absolute URL.
    ///
    /// # Errors
    /// An `url` that does not parse.
    pub fn new(method: &str, url: &str) -> Result<Self> {
        let url = mm_js::url::parse(url, None).ok_or_else(|| Error::invalid(format!("Invalid URL: {url}")))?;
        let local_port = url.port_or_known_default().unwrap_or(80);
        let target = format!("{}{}", url.path(), search(&url));
        Ok(Self { method: method.to_owned(), target, url, headers: Headers::new(), remote_address: "127.0.0.1".into(), local_port, body: Body::Empty })
    }

    /// The same request without its body, for code that answers after the body was taken.
    pub fn head(&self) -> Self {
        Self {
            method: self.method.clone(),
            target: self.target.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            remote_address: self.remote_address.clone(),
            local_port: self.local_port,
            body: Body::Empty,
        }
    }

    /// A request header by lower-case name.
    ///
    /// # Arguments
    /// * `name` - Lower-case header name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    /// `url.pathname`.
    pub fn pathname(&self) -> &str {
        self.url.path()
    }

    /// `url.search`: the query with its `?`, or empty when there is none or it is empty.
    pub fn search(&self) -> String {
        search(&self.url)
    }

    /// The body, leaving an empty one in its place.
    pub fn take_body(&mut self) -> Body {
        std::mem::take(&mut self.body)
    }
}

/// `url.search` of a URL: its query with the `?`, or empty when the query is missing or empty.
///
/// # Arguments
/// * `url` - Parsed URL.
pub fn search(url: &Url) -> String {
    match url.query() {
        Some(query) if !query.is_empty() => format!("?{query}"),
        _ => String::new(),
    }
}

/// `url.searchParams.get(name)`: the first value of a query parameter, decoded as `URLSearchParams` decodes it.
///
/// # Arguments
/// * `url` - Parsed URL.
/// * `name` - Parameter name.
pub fn search_param(url: &Url, name: &str) -> Option<String> {
    url.query_pairs().find(|(key, _)| key == name).map(|(_, value)| value.into_owned())
}

/// A response, as the JS backend wrote it with `writeHead` and `end`.
#[derive(Debug)]
pub struct Response {
    /// HTTP status.
    pub status: u16,
    /// Headers in the order written; names as the code wrote them.
    pub headers: Headers,
    /// The body; ignored for statuses that carry none.
    pub body: Body,
}

impl Response {
    /// A response with these parts.
    ///
    /// # Arguments
    /// * `status` - HTTP status.
    /// * `headers` - Headers in order.
    /// * `body` - Body.
    pub fn new(status: u16, headers: Headers, body: impl Into<Body>) -> Self {
        Self { status, headers, body: body.into() }
    }

    /// `send(res, status, body, headers)` (`lib/site.mjs`): a whole body that is never cached, with
    /// `cache-control: no-store` first unless `headers` sets it.
    ///
    /// # Arguments
    /// * `status` - HTTP status.
    /// * `body` - Response body.
    /// * `headers` - Extra headers, spread over `cache-control: no-store`.
    pub fn send(status: u16, body: impl Into<Body>, headers: &Headers) -> Self {
        let mut all = Headers::from([("cache-control", "no-store")]);
        all.extend(headers);
        Self::new(status, all, body)
    }

    /// The value of a response header with exactly this name.
    ///
    /// # Arguments
    /// * `name` - Header name as written.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }
}
