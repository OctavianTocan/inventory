//! Outbound HTTP for sign-in and link capture: `mm_core::host::Fetch` over the Workers `fetch`.

use async_trait::async_trait;
use mm_core::Result;
use mm_core::host::Fetch;
use mm_core::http::{Request, Response};

use crate::http::response_into;
use crate::util::{js_error, worker_error};

/// The Workers runtime's `fetch`.
pub struct WorkerFetch;

async fn send(request: Request, redirect: web_sys::RequestRedirect) -> Result<Response> {
    let init = web_sys::RequestInit::new();
    init.set_method(&request.method);
    init.set_redirect(redirect);
    let headers = web_sys::Headers::new().map_err(js_error)?;
    for (name, value) in request.headers.iter() {
        headers.append(name, value).map_err(js_error)?;
    }
    init.set_headers(&headers);
    let bytes = request.body.bytes().await?;
    if !bytes.is_empty() {
        init.set_body(&js_sys::Uint8Array::from(bytes.as_slice()));
    }
    let outgoing = web_sys::Request::new_with_str_and_init(request.url.as_str(), &init).map_err(js_error)?;
    let answer = worker::Fetch::Request(worker::Request::from(outgoing)).send().await.map_err(worker_error)?;
    let raw: web_sys::Response = answer.into();
    response_into(&raw)
}

#[async_trait(?Send)]
impl Fetch for WorkerFetch {
    async fn fetch(&self, request: Request) -> Result<Response> {
        send(request, web_sys::RequestRedirect::Follow).await
    }

    async fn fetch_unfollowed(&self, request: Request) -> Result<Response> {
        send(request, web_sys::RequestRedirect::Manual).await
    }
}
