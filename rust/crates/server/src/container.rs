//! Port of `worker/container/server.mjs`: the media operations of the Worker's Media Durable Object, each a POST whose
//! arguments are base64url JSON in `x-media-args` and whose body is the input bytes.

use std::path::PathBuf;
use std::rc::Rc;

use async_trait::async_trait;
use mm_core::host::{MediaProcessor, MediaUser, RenditionSource, ResourceOptions, Screenshot};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::{Error, Object, Result, Value};
use mm_js::{base64, json};

use crate::capture_http::CaptureTransport;
use crate::http::Handler;
use crate::media_processor::LocalProcessor;
use crate::screenshot::ChromiumScreenshot;

/// The media container: `prepare`, `rendition`, `image`, `resource` and `screenshot` over HTTP.
pub struct Container {
    processor: LocalProcessor,
    transport: Rc<CaptureTransport>,
    screenshot: ChromiumScreenshot,
}

struct Failure {
    status: Option<u16>,
    message: String,
    code: Option<&'static str>,
    name: &'static str,
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        match error {
            Error::Status { status, message, code } if (400..600).contains(&status) => Self { status: Some(status), message, code, name: "Error" },
            other => Self { status: None, message: other.to_string(), code: None, name: "Error" },
        }
    }
}

enum Answer {
    Json(String),
    Bytes(Vec<u8>, Headers),
    NoContent,
}

fn octet(extra: Option<(&str, String)>) -> Headers {
    let mut headers = Headers::from([("content-type", "application/octet-stream")]);
    if let Some((name, value)) = extra {
        headers.set(name, value);
    }
    headers
}

fn text(args: &Value, key: &str) -> String {
    let value = args.get(key);
    if value.is_nullish() { String::new() } else { value.to_js_string() }
}

impl Container {
    /// A container that processes media below the system temporary directory and screenshots pages with the
    /// Chromium at `VERIFIER_CHROMIUM` (default `/usr/lib/chromium/chromium`).
    pub fn new() -> Self {
        let transport = Rc::new(CaptureTransport::new());
        let chromium = std::env::var_os("VERIFIER_CHROMIUM").map_or_else(|| PathBuf::from("/usr/lib/chromium/chromium"), PathBuf::from);
        Self { processor: LocalProcessor::new(None), screenshot: ChromiumScreenshot::new(chromium, Rc::clone(&transport)), transport }
    }

    async fn run(&self, operation: &str, args: &Value, request: &mut Request) -> Result<Answer> {
        let body = request.take_body();
        match operation {
            "prepare" => {
                let user = MediaUser { account_uid: text(args.get("user"), "accountUid"), id_prefix: text(args.get("user"), "idPrefix") };
                let made = self.processor.prepare(body, &text(args, "type"), &text(args, "name"), &user, &text(args, "id")).await?;
                Ok(Answer::Json(json::to_string(&made.to_value())))
            }
            "rendition" => {
                let size = request.header("content-length").and_then(|length| length.parse().ok()).unwrap_or(0);
                let original = RenditionSource { body, size, name: text(args, "name") };
                match self.processor.rendition(original, &text(args, "relative")).await? {
                    Some(made) => Ok(Answer::Bytes(made.body.bytes().await?, octet(None))),
                    None => Ok(Answer::NoContent),
                }
            }
            "image" => Ok(Answer::Bytes(self.processor.image(body).await?.data, octet(None))),
            "audio" => match self.processor.audio(body).await? {
                Some(audio) => Ok(Answer::Bytes(audio, octet(None))),
                None => Ok(Answer::NoContent),
            },
            "resource" => {
                let limit = args.get("limit");
                let options = ResourceOptions { offline: args.get("offline").truthy(), limit: (!limit.is_nullish()).then(|| limit.to_number() as u64) };
                let mut meta = self.transport.fetch_resource(&text(args, "url"), options, None).await?.to_value();
                let bytes = match meta.as_object_mut().and_then(|object| object.remove("bytes")) {
                    Some(Value::Bytes(bytes)) => bytes,
                    _ => Vec::new(),
                };
                let header = base64::encode_url(json::to_string(&meta).as_bytes());
                Ok(Answer::Bytes(bytes, octet(Some(("x-resource", header)))))
            }
            _ => Ok(Answer::Bytes(self.screenshot.capture(&text(args, "url"), args.get("offline").truthy()).await?, octet(None))),
        }
    }

    async fn answer(&self, operation: &str, request: &mut Request) -> std::result::Result<Answer, Failure> {
        let encoded = request.header("x-media-args").unwrap_or("e30").to_owned();
        let args = json::parse(&String::from_utf8_lossy(&base64::decode(&encoded)))
            .map_err(|error| Failure { status: None, message: error.message().to_owned(), code: None, name: "SyntaxError" })?;
        Ok(self.run(operation, &args, request).await?)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl Handler for Container {
    async fn handle(&self, mut request: Request) -> Response {
        let pathname = request.pathname().to_owned();
        if pathname == "/health" {
            return Response::new(200, Headers::new(), "ok");
        }
        let operation = pathname.strip_prefix('/').unwrap_or("");
        if request.method != "POST" || !["prepare", "rendition", "image", "audio", "resource", "screenshot"].contains(&operation) {
            return Response::new(404, Headers::new(), Body::Empty);
        }
        match self.answer(operation, &mut request).await {
            Ok(Answer::Json(text)) => Response::new(200, Headers::from([("content-type", "application/json")]), text),
            Ok(Answer::Bytes(bytes, headers)) => Response::new(200, headers, bytes),
            Ok(Answer::NoContent) => Response::new(204, octet(None), Body::Empty),
            Err(failure) => {
                let status = failure.status.unwrap_or(500);
                eprintln!("[media] {pathname} failed: {status} {}", failure.code.unwrap_or(failure.name));
                let mut body = Object::new();
                body.insert("error", if failure.status.is_some() { failure.message.as_str() } else { "Media operation failed" });
                if let Some(status) = failure.status {
                    body.insert("status", status);
                }
                if let Some(code) = failure.code {
                    body.insert("code", code);
                }
                Response::new(status, Headers::from([("content-type", "application/json")]), json::to_string(&body.into()))
            }
        }
    }

    fn base(&self) -> Option<&str> {
        Some("http://container")
    }
}
