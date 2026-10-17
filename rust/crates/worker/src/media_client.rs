//! The media container's operations for a library on Cloudflare, sent through the `Media` Durable Object
//! (`worker/media-client.mjs`).

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::{
    AnalysisImage, FetchedResource, MediaProcessor, MediaUser, PreparedMedia, RenditionOutput, RenditionSource, Resource, ResourceOptions, Screenshot,
};
use mm_core::http::{Body, Headers};
use mm_core::{Error, Result, obj};
use mm_js::{Value, base64, json};
use wasm_bindgen::JsValue;

use crate::util::{js_error, worker_error};

/// The container's operations, reached through the `MEDIA` binding.
pub struct MediaClient {
    namespace: worker::ObjectNamespace,
}

impl MediaClient {
    /// A client for the Worker's `MEDIA` Durable Object.
    ///
    /// # Arguments
    /// * `env` - The Worker's environment.
    ///
    /// # Errors
    /// An internal error when the `MEDIA` binding is missing.
    pub fn new(env: &worker::Env) -> Result<Self> {
        Ok(Self { namespace: env.durable_object("MEDIA").map_err(worker_error)? })
    }

    async fn call(&self, operation: &str, args: Value, body: Body) -> Result<worker::Response> {
        let stub = self.namespace.get_by_name("media").map_err(worker_error)?;
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        let headers = web_sys::Headers::new().map_err(js_error)?;
        let encoded = base64::encode_url(json::stringify(&args)?.unwrap_or_default().as_bytes());
        headers.append("x-media-args", &encoded).map_err(js_error)?;
        init.set_headers(&headers);
        match body {
            Body::Empty => {}
            Body::Bytes(bytes) => init.set_body(&js_sys::Uint8Array::from(bytes.as_slice())),
            Body::Stream(stream) => init.set_body(&readable(stream)),
        }
        let request = web_sys::Request::new_with_str_and_init(&format!("http://media/{operation}"), &init).map_err(js_error)?;
        let mut response = stub.fetch_with_request(worker::Request::from(request)).await.map_err(worker_error)?;
        let status = response.status_code();
        if (200..300).contains(&status) {
            return Ok(response);
        }
        let failure = json::parse(&response.text().await.unwrap_or_default()).unwrap_or_default();
        let message = failure.get("error").as_str().map_or_else(|| format!("Media container answered {status}"), str::to_owned);
        let mut error = match failure.get("status").as_f64() {
            Some(code) if code != 0.0 => Error::status(code as u16, message),
            _ => Error::internal(message),
        };
        if let Some(code) = failure.get("code").as_str() {
            error = error.with_code(if code == mm_core::capture_http::CAPTURE_BLOCKED { mm_core::capture_http::CAPTURE_BLOCKED } else { Box::leak(code.to_owned().into_boxed_str()) });
        }
        Err(error)
    }
}

fn readable(stream: mm_core::http::ByteStream) -> web_sys::ReadableStream {
    let chunks = stream.map(|chunk| match chunk {
        Ok(bytes) => Ok(JsValue::from(js_sys::Uint8Array::from(bytes.as_slice()))),
        Err(error) => Err(JsValue::from_str(&error.to_string())),
    });
    wasm_streams::ReadableStream::from_stream(chunks).into_raw().unchecked_into()
}

use wasm_bindgen::JsCast;

#[async_trait(?Send)]
impl MediaProcessor for MediaClient {
    async fn prepare(&self, body: Body, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let args = obj! {
            "type" => content_type,
            "name" => name,
            "user" => obj! { "accountUid" => user.account_uid.as_str(), "idPrefix" => user.id_prefix.as_str() },
            "id" => id,
        };
        let mut response = self.call("prepare", args.into(), body).await?;
        let prepared = json::parse(&response.text().await.map_err(worker_error)?)?;
        Ok(PreparedMedia { data: prepared.get("data").clone(), extras: prepared.get("extras").clone() })
    }

    async fn rendition(&self, original: RenditionSource, relative: &str) -> Result<Option<RenditionOutput>> {
        let mut response = self.call("rendition", obj! { "name" => original.name.as_str(), "relative" => relative }.into(), original.body).await?;
        if response.status_code() == 204 {
            return Ok(None);
        }
        let bytes = response.bytes().await.map_err(worker_error)?;
        let size = bytes.len() as u64;
        Ok(Some(RenditionOutput { body: Body::Bytes(bytes), size: Some(size) }))
    }

    async fn image(&self, body: Body) -> Result<AnalysisImage> {
        let mut response = self.call("image", obj! {}.into(), body).await?;
        Ok(AnalysisImage { mime: "image/png".into(), data: response.bytes().await.map_err(worker_error)? })
    }
}

#[async_trait(?Send)]
impl Resource for MediaClient {
    async fn fetch(&self, url: &str, options: ResourceOptions) -> Result<FetchedResource> {
        let mut args = obj! { "url" => url, "offline" => options.offline };
        if let Some(limit) = options.limit {
            args.insert("limit", limit as f64);
        }
        let mut response = self.call("resource", args.into(), Body::Empty).await?;
        let header = response.headers().get("x-resource").map_err(worker_error)?.unwrap_or_default();
        let meta = json::parse(&String::from_utf8_lossy(&base64::decode(&header)))?;
        let headers = meta.get("headers").as_object().map(|map| Headers::from_pairs(map.iter().map(|(name, value)| (name.into_owned(), value.to_js_string())))).unwrap_or_default();
        Ok(FetchedResource {
            bytes: response.bytes().await.map_err(worker_error)?,
            url: meta.get("url").as_str().unwrap_or_default().to_owned(),
            mime: meta.get("mime").as_str().unwrap_or_default().to_owned(),
            status: meta.get("status").as_f64().unwrap_or(0.0) as u16,
            headers,
        })
    }
}

#[async_trait(?Send)]
impl Screenshot for MediaClient {
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        let mut response = self.call("screenshot", obj! { "url" => url, "offline" => offline }.into(), Body::Empty).await?;
        response.bytes().await.map_err(worker_error)
    }
}
