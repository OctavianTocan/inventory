//! Node's `http` server over hyper: incoming requests become [`mm_core::http::Request`] with the headers Node's
//! `req.headers` held, and a [`mm_core::http::Response`] goes out as `res.writeHead` and `res.end` sent it.

use std::net::SocketAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use futures_util::future::LocalBoxFuture;
use http_body_util::BodyStream;
use hyper::body::{Frame, Incoming, SizeHint};
use hyper::header::{HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use mm_core::Error;
use mm_core::http::{Body, ByteStream, Headers, Request, Response};
use tokio::net::TcpListener;
use url::Url;

/// A response body hyper sends; unlike hyper's own boxed bodies it may hold values that stay on this thread.
pub enum OutgoingBody {
    /// No body.
    Empty,
    /// The whole body, until it is sent.
    Full(Option<Bytes>),
    /// A body still being produced.
    Stream(ByteStream),
}

impl hyper::body::Body for OutgoingBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Error>>> {
        match self.get_mut() {
            Self::Empty => Poll::Ready(None),
            Self::Full(bytes) => Poll::Ready(bytes.take().map(|bytes| Ok(Frame::data(bytes)))),
            Self::Stream(stream) => stream.as_mut().poll_next(cx).map(|chunk| chunk.map(|data| data.map(|data| Frame::data(Bytes::from(data))))),
        }
    }

    fn is_end_stream(&self) -> bool {
        matches!(self, Self::Empty | Self::Full(None))
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty | Self::Full(None) => SizeHint::with_exact(0),
            Self::Full(Some(bytes)) => SizeHint::with_exact(bytes.len() as u64),
            Self::Stream(_) => SizeHint::default(),
        }
    }
}

/// Headers whose repeats Node drops, keeping the first (`matchKnownFields` in `_http_incoming.js`).
const SINGLE: [&str; 18] = [
    "age", "authorization", "content-length", "content-type", "etag", "expires", "from", "host", "if-modified-since",
    "if-unmodified-since", "last-modified", "location", "max-forwards", "proxy-authorization", "referer", "retry-after",
    "server", "user-agent",
];

/// What runs on a connection once its upgrade was answered.
pub type UpgradeTask = Box<dyn FnOnce(TokioIo<Upgraded>) -> LocalBoxFuture<'static, ()>>;

/// What a server does with a request that asks to upgrade its connection (`server.on('upgrade')`).
pub enum Upgrade {
    /// No upgrade listener: answer it as any other request.
    Ignore,
    /// Close the connection without an answer (`socket.destroy()`).
    Refuse,
    /// Send this `101` response, then run the task on the upgraded connection.
    Accept(Response, UpgradeTask),
}

/// What answers the server's requests.
#[async_trait(?Send)]
pub trait Handler: 'static {
    /// Answer one request.
    ///
    /// # Arguments
    /// * `request` - The request, its body not yet read.
    ///
    /// # Returns
    /// The response; failures are answered inside it, as the JS servers caught them.
    async fn handle(&self, request: Request) -> Response;

    /// Take a request with `Connection: upgrade` and an `Upgrade` header, which Node hands to `upgrade` listeners
    /// instead of the request handler.
    ///
    /// # Arguments
    /// * `request` - The request.
    fn upgrade(&self, request: &Request) -> Upgrade {
        let _ = request;
        Upgrade::Ignore
    }

    /// The base the server resolves `req.url` against (`new URL(req.url, base)`), or `None` for
    /// `http://${req.headers.host}`.
    fn base(&self) -> Option<&str> {
        None
    }
}

// Whether Node's parser flags the request as an upgrade: an `Upgrade` header and an `upgrade` token in `Connection`.
fn asks_upgrade(headers: &hyper::HeaderMap) -> bool {
    headers.contains_key(hyper::header::UPGRADE)
        && headers.get_all(hyper::header::CONNECTION).iter().any(|value| {
            value.to_str().is_ok_and(|value| value.split(',').any(|token| token.trim().eq_ignore_ascii_case("upgrade")))
        })
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

/// `req.headers` of a request with these header lines: lower-case names in the order each first appeared, repeats
/// joined with `, ` (`; ` for `cookie`) or dropped for the headers Node keeps once, and values read as Latin-1.
/// `set-cookie`, an array in Node, is joined with `, `.
///
/// # Arguments
/// * `lines` - Header names and raw values in the order received.
pub fn node_headers<'a>(lines: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Headers {
    let mut joined: Vec<(String, String)> = Vec::new();
    for (name, value) in lines {
        let name = name.to_ascii_lowercase();
        let value = latin1(value);
        match joined.iter_mut().find(|(seen, _)| *seen == name) {
            None => joined.push((name, value)),
            Some(_) if SINGLE.contains(&name.as_str()) => {}
            Some((_, existing)) => {
                existing.push_str(if name == "cookie" { "; " } else { ", " });
                existing.push_str(&value);
            }
        }
    }
    Headers::from_pairs(joined)
}

/// `new URL(req.url, `http://${req.headers.host}`)`, or `None` where that throws.
///
/// # Arguments
/// * `target` - The request target as sent.
/// * `host` - The Host header; a missing one reads as `undefined`, as in the template string.
pub fn request_url(target: &str, host: Option<&str>) -> Option<Url> {
    mm_js::url::parse(target, Some(&format!("http://{}", host.unwrap_or("undefined"))))
}

/// A hyper request as the library code's [`Request`], or `None` when its URL does not parse (the servers answer 400
/// `Bad request`).
///
/// # Arguments
/// * `request` - The request hyper parsed.
/// * `remote` - The client's address.
/// * `local_port` - The port the connection arrived on.
/// * `base` - What the target is resolved against, or `None` for the Host header.
pub fn incoming(request: hyper::Request<Incoming>, remote: SocketAddr, local_port: u16, base: Option<&str>) -> Option<Request> {
    let (parts, body) = request.into_parts();
    let headers = node_headers(parts.headers.iter().map(|(name, value)| (name.as_str(), value.as_bytes())));
    let target = parts.uri.to_string();
    let url = match base {
        Some(base) => mm_js::url::parse(&target, Some(base))?,
        None => request_url(&target, headers.get("host"))?,
    };
    let body = if hyper::body::Body::is_end_stream(&body) {
        Body::Empty
    } else {
        Body::Stream(Box::pin(BodyStream::new(body).filter_map(|frame| async move {
            match frame {
                Ok(frame) => frame.into_data().ok().map(|data| Ok(data.to_vec())),
                Err(error) => Some(Err(Error::internal(error.to_string()))),
            }
        })))
    };
    Some(Request { method: parts.method.as_str().to_owned(), target, url, headers, remote_address: remote.ip().to_string(), local_port, body })
}

fn send_failure(message: String) -> hyper::Response<OutgoingBody> {
    let mut response = hyper::Response::new(OutgoingBody::Full(Some(Bytes::from(message))));
    *response.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
    response.headers_mut().insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

/// A [`Response`] as hyper sends it. A header Node would refuse to write becomes the 500 the servers answered with
/// the `TypeError`'s text.
///
/// # Arguments
/// * `response` - What the handler answered.
pub fn outgoing(response: Response) -> hyper::Response<OutgoingBody> {
    let body = match response.body {
        Body::Empty => OutgoingBody::Empty,
        Body::Bytes(bytes) => OutgoingBody::Full(Some(Bytes::from(bytes))),
        Body::Stream(stream) => OutgoingBody::Stream(stream),
    };
    let mut out = hyper::Response::new(body);
    let Ok(status) = hyper::StatusCode::from_u16(response.status) else {
        return send_failure(format!("RangeError [ERR_HTTP_INVALID_STATUS_CODE]: Invalid status code: {}", response.status));
    };
    *out.status_mut() = status;
    for (name, value) in response.headers.iter() {
        let Ok(header) = HeaderName::from_bytes(name.as_bytes()) else {
            return send_failure(format!("TypeError [ERR_INVALID_HTTP_TOKEN]: Header name must be a valid HTTP token [\"{name}\"]"));
        };
        let bytes: Option<Vec<u8>> = value.chars().map(|c| u8::try_from(u32::from(c)).ok()).collect();
        let Some(value) = bytes.and_then(|bytes| HeaderValue::from_bytes(&bytes).ok()) else {
            return send_failure(format!("TypeError [ERR_INVALID_CHAR]: Invalid character in header content [\"{name}\"]"));
        };
        out.headers_mut().append(header, value);
    }
    out
}

/// The connection closes without an answer.
#[derive(Debug)]
struct Refused;

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("upgrade refused")
    }
}

impl std::error::Error for Refused {}

async fn respond(handler: &dyn Handler, mut request: hyper::Request<Incoming>, remote: SocketAddr, local_port: u16) -> Result<hyper::Response<OutgoingBody>, Refused> {
    let upgrading = asks_upgrade(request.headers()).then(|| hyper::upgrade::on(&mut request));
    let Some(request) = incoming(request, remote, local_port, handler.base()) else {
        return if upgrading.is_some() { Err(Refused) } else { Ok(outgoing(Response::send(400, "Bad request", &Headers::new()))) };
    };
    if let Some(upgraded) = upgrading {
        match handler.upgrade(&request) {
            Upgrade::Ignore => {}
            Upgrade::Refuse => return Err(Refused),
            Upgrade::Accept(response, task) => {
                tokio::task::spawn_local(async move {
                    if let Ok(connection) = upgraded.await {
                        task(TokioIo::new(connection)).await;
                    }
                });
                return Ok(outgoing(response));
            }
        }
    }
    let head = request.method == "HEAD";
    let mut response = outgoing(handler.handle(request).await);
    if head {
        // Node writes the head before it knows the body, so a HEAD answer has no content-length.
        *response.body_mut() = OutgoingBody::Stream(Box::pin(futures_util::stream::empty()));
    }
    Ok(response)
}

/// Serve HTTP/1.1 connections from `listener`, each connection a task on the current `LocalSet`.
///
/// # Arguments
/// * `listener` - Bound listener.
/// * `handler` - What answers every request.
///
pub async fn serve(listener: TcpListener, handler: Rc<dyn Handler>) {
    loop {
        // Node keeps listening when one accept fails (EMFILE, a reset before accept).
        let Ok((stream, remote)) = listener.accept().await else {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            continue;
        };
        // Node's HTTP server turns Nagle's algorithm off on every connection (`noDelay: true`).
        let _ = stream.set_nodelay(true);
        let Ok(local_port) = stream.local_addr().map(|address| address.port()) else { continue };
        let handler = Rc::clone(&handler);
        tokio::task::spawn_local(async move {
            let service = service_fn(move |request| {
                let handler = Rc::clone(&handler);
                async move { respond(handler.as_ref(), request, remote, local_port).await }
            });
            let _ = http1::Builder::new().half_close(true).serve_connection(TokioIo::new(stream), service).with_upgrades().await;
        });
    }
}
