//! Outbound HTTP for the native server, as Node's global `fetch` sends it: redirects followed (or not, for
//! [`Fetch::fetch_unfollowed`]), compressed bodies decoded, and failures named by their system error code.

use std::sync::Once;

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::{Error, Result};

/// Node's `fetch` over reqwest.
pub struct NativeFetch {
    following: reqwest::Client,
    unfollowed: reqwest::Client,
}

fn client(redirect: reqwest::redirect::Policy) -> reqwest::Client {
    reqwest::Client::builder().redirect(redirect).build().expect("the HTTP client starts with its TLS roots")
}

// The `cause.code` Node's fetch rejects with, which the AI errors show; `TypeError` when there is none.
fn failure(error: &reqwest::Error) -> Error {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(cause) = source {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            use std::io::ErrorKind::{ConnectionAborted, ConnectionRefused, ConnectionReset, TimedOut};
            let code = match io.kind() {
                ConnectionRefused => "ECONNREFUSED",
                ConnectionReset | ConnectionAborted => "ECONNRESET",
                TimedOut => "ETIMEDOUT",
                _ => "TypeError",
            };
            return Error::internal(code);
        }
        if cause.to_string().contains("dns error") {
            return Error::internal("ENOTFOUND");
        }
        source = cause.source();
    }
    Error::internal(if error.is_timeout() { "ETIMEDOUT" } else { "TypeError" })
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

impl NativeFetch {
    /// A client with the system's trusted roots.
    ///
    /// # Panics
    /// When TLS cannot be set up.
    pub fn new() -> Self {
        static PROVIDER: Once = Once::new();
        PROVIDER.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
        Self { following: client(reqwest::redirect::Policy::limited(20)), unfollowed: client(reqwest::redirect::Policy::none()) }
    }

    async fn send(&self, client: &reqwest::Client, request: Request) -> Result<Response> {
        let method = reqwest::Method::from_bytes(request.method.as_bytes()).map_err(|_| Error::invalid(format!("'{}' is not a valid HTTP method.", request.method)))?;
        let mut outgoing = client.request(method, request.url.as_str());
        for (name, value) in request.headers.iter() {
            outgoing = outgoing.header(name, value);
        }
        let body = request.body.bytes().await?;
        if !body.is_empty() {
            outgoing = outgoing.body(body);
        }
        let response = outgoing.send().await.map_err(|error| failure(&error))?;
        let mut headers: Vec<(String, String)> = Vec::new();
        for (name, value) in response.headers() {
            let value = latin1(value.as_bytes());
            match headers.iter_mut().find(|(seen, _)| seen == name.as_str()) {
                Some((_, existing)) => {
                    existing.push_str(", ");
                    existing.push_str(&value);
                }
                None => headers.push((name.as_str().to_owned(), value)),
            }
        }
        let status = response.status().as_u16();
        let stream = response.bytes_stream().map(|chunk| chunk.map(|bytes| bytes.to_vec()).map_err(|error| failure(&error)));
        Ok(Response::new(status, Headers::from_pairs(headers), Body::Stream(Box::pin(stream))))
    }
}

impl Default for NativeFetch {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl Fetch for NativeFetch {
    async fn fetch(&self, request: Request) -> Result<Response> {
        self.send(&self.following, request).await
    }

    async fn fetch_unfollowed(&self, request: Request) -> Result<Response> {
        self.send(&self.unfollowed, request).await
    }
}
