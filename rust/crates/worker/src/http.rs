//! Workers requests and responses as mm-core's (`worker/http.mjs`).

use std::cell::Cell;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, StreamExt};
use mm_core::http::{Body, ByteStream, Headers, Request, Response};
use mm_core::{Error, Result};

use crate::util::{js_error, worker_error};

/// Statuses whose responses carry no body.
const EMPTY: [u16; 4] = [101, 204, 205, 304];

/// A JavaScript `ReadableStream` read as Rust chunks only once something polls it, so a body nothing read goes back
/// to the runtime as the same stream instead of being copied through wasm memory.
struct JsBody {
    raw: Cell<Option<web_sys::ReadableStream>>,
    chunks: Option<worker::ByteStream>,
}

impl Stream for JsBody {
    type Item = Result<Vec<u8>>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.chunks.is_none() {
            let Some(raw) = self.raw.take() else { return Poll::Ready(None) };
            self.chunks = Some(worker::ByteStream::from(raw));
        }
        let chunks = self.chunks.as_mut().expect("chunks set above");
        chunks.poll_next_unpin(context).map(|item| item.map(|chunk| chunk.map_err(|error| Error::internal(error.to_string()))))
    }
}

/// The body of a JavaScript `ReadableStream` as mm-core's byte stream.
pub fn byte_stream(stream: web_sys::ReadableStream) -> ByteStream {
    Box::pin(JsBody { raw: Cell::new(Some(stream)), chunks: None })
}

/// The JavaScript stream behind a byte stream made by [`byte_stream`] that nothing has read yet.
pub fn unread(stream: &ByteStream) -> Option<web_sys::ReadableStream> {
    mm_core::http::BodyStream::as_any(stream.as_ref().get_ref()).downcast_ref::<JsBody>()?.raw.take()
}

/// The headers of a JavaScript `Headers`, names in lower case.
pub fn headers_of(headers: &web_sys::Headers) -> Result<Headers> {
    let mut out = Headers::new();
    for entry in js_sys::try_iter(headers.as_ref()).map_err(js_error)?.into_iter().flatten() {
        let pair = js_sys::Array::from(&entry.map_err(js_error)?);
        if let (Some(name), Some(value)) = (pair.get(0).as_string(), pair.get(1).as_string()) {
            out.append(name.to_lowercase(), value);
        }
    }
    Ok(out)
}

/// A request from the Workers runtime as mm-core handles it (`nodePair`'s `req`).
///
/// # Arguments
/// * `request` - The incoming request; its body is read lazily, not copied.
///
/// # Errors
/// An invalid-URL or header error.
pub fn request_from(request: &web_sys::Request) -> Result<Request> {
    let mut out = Request::new(&request.method(), &request.url())?;
    out.headers = headers_of(&request.headers())?;
    out.remote_address = out.headers.get("cf-connecting-ip").unwrap_or("127.0.0.1").to_owned();
    out.body = match request.body() {
        Some(stream) if out.method != "GET" && out.method != "HEAD" => Body::Stream(byte_stream(stream)),
        _ => Body::Empty,
    };
    Ok(out)
}

/// A response for the Workers runtime from mm-core's (`nodePair`'s `res`).
///
/// # Arguments
/// * `response` - The answer; its stream body is passed on as it is read.
///
/// # Errors
/// A header the runtime refuses.
pub fn response_from(response: Response) -> Result<worker::Response> {
    let headers = worker::Headers::new();
    for (name, value) in response.headers.iter() {
        headers.append(name, value).map_err(worker_error)?;
    }
    let builder = worker::Response::builder().with_status(response.status).with_headers(headers);
    let empty = EMPTY.contains(&response.status);
    Ok(match response.body {
        _ if empty => builder.empty(),
        Body::Empty => builder.empty(),
        Body::Bytes(bytes) => builder.fixed(bytes),
        Body::Stream(stream) => match unread(&stream) {
            Some(raw) => builder.stream(raw),
            None => builder.from_stream(into_chunks_of(stream)).map_err(worker_error)?,
        },
    })
}

/// A response from the Workers runtime as mm-core's, with its body as a stream.
///
/// # Arguments
/// * `response` - A `fetch` or Durable Object answer.
///
/// # Errors
/// A header error.
pub fn response_into(response: &web_sys::Response) -> Result<Response> {
    let body = match response.body() {
        Some(stream) if !EMPTY.contains(&response.status()) => Body::Stream(byte_stream(stream)),
        _ => Body::Empty,
    };
    Ok(Response::new(response.status(), headers_of(&response.headers())?, body))
}

/// Text with a status, as the Worker's `text` helper answers it.
pub fn text(status: u16, body: &str, headers: &Headers) -> Response {
    let mut all = Headers::from([("content-type", "text/plain; charset=utf-8"), ("cache-control", "no-store")]);
    all.extend(headers);
    Response::new(status, all, body)
}

fn into_chunks_of(stream: ByteStream) -> impl futures_util::Stream<Item = std::result::Result<Vec<u8>, worker::Error>> {
    stream.map(|chunk| chunk.map_err(|error| worker::Error::RustError(error.to_string())))
}
