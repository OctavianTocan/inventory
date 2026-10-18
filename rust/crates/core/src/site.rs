//! Port of the helpers of `lib/site.mjs` that the native server and the hosted site share.

use async_trait::async_trait;
use url::Url;

use crate::http::{Headers, Request, Response};
use crate::media_files::byte_range;
use crate::{Object, Result, Value};

/// mymind's own origin; recorded keys and bodies use it, the replica serves it from `/` (`APP_ORIGIN`).
pub const APP_ORIGIN: &str = "https://access.mymind.com/";

/// Local path prefixes the replica serves mymind's other origins under, with those origins (`LOCAL_PREFIXES`).
pub const LOCAL_PREFIXES: [(&str, &str); 3] = [
    ("/_static/", "https://static.accelerator.net/"),
    ("/_mediahost/", "https://media.mymind.host/"),
    ("/_carbon/", "https://carbon-media.accelerator.net/"),
];

/// The library API as the servers call it (`createApi`): it answers the routes it owns and leaves the rest to the
/// recordings.
#[async_trait(?Send)]
pub trait LibraryApi {
    /// Answer a request the library API owns, or `None` to leave it to the recordings.
    ///
    /// # Arguments
    /// * `request` - The request; its body is read only when the API answers.
    ///
    /// # Errors
    /// What the server answers with a 500.
    async fn handle(&self, request: &mut Request) -> Result<Option<Response>>;

    /// Render the page shell from the library database (`api.shell`).
    ///
    /// # Errors
    /// What the server answers with a 500.
    fn shell(&self) -> Result<String>;
}

/// A server for one hosted library that answers every request it is given (`createHosted().handle`).
#[async_trait(?Send)]
pub trait Site {
    /// Answer a request.
    ///
    /// # Arguments
    /// * `request` - The request.
    ///
    /// # Errors
    /// What the server answers with a 500.
    async fn handle(&self, request: &mut Request) -> Result<Response>;
}

/// The content type of mirrored and captured files with this extension (`TYPES[extension]`).
///
/// # Arguments
/// * `extension` - Lower-case extension with its dot, such as `.js`.
pub fn content_type(extension: &str) -> Option<&'static str> {
    Some(match extension {
        ".js" | ".mjs" => "text/javascript; charset=utf-8",
        ".css" => "text/css; charset=utf-8",
        ".html" => "text/html; charset=utf-8",
        ".json" => "application/json; charset=utf-8",
        ".svg" => "image/svg+xml",
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".avif" => "image/avif",
        ".woff2" => "font/woff2",
        ".woff" => "font/woff",
        ".ttf" => "font/ttf",
        ".mp4" => "video/mp4",
        ".wasm" => "application/wasm",
        _ => return None,
    })
}

/// Whether mirrored files with this extension are text with mymind's origins to localize (`TEXT_EXTENSIONS.has`).
///
/// # Arguments
/// * `extension` - Lower-case extension with its dot.
pub fn is_text_extension(extension: &str) -> bool {
    matches!(extension, ".js" | ".mjs" | ".css" | ".html" | ".json" | ".svg")
}

/// Rewrite mymind's origins in text to local paths (`localize`).
///
/// # Arguments
/// * `text` - Input text.
pub fn localize(text: &str) -> String {
    let mut out = text.to_owned();
    for (local, origin) in LOCAL_PREFIXES {
        if out.contains(origin) {
            out = out.replace(origin, local);
        }
    }
    if out.contains(APP_ORIGIN) { out.replace(APP_ORIGIN, "/") } else { out }
}

/// Localize every string in a decoded recording, keeping binary values as they are (`localizeValue`). Like the JS,
/// which copies every non-binary object through `Object.entries`, a date becomes an empty object and an extension
/// value an object of its `type` and `data`.
///
/// # Arguments
/// * `value` - Decoded MessagePack item or JSON value.
pub fn localize_value(value: &Value) -> Value {
    match value {
        Value::String(text) => Value::String(localize(text)),
        Value::Array(items) => Value::Array(items.iter().map(localize_value).collect()),
        Value::Object(object) => Value::Object(object.iter().map(|(key, item)| (key.into_owned(), localize_value(item))).collect()),
        Value::Date(_) => Value::Object(Object::default()),
        Value::Ext(kind, data) => crate::obj! { "type" => f64::from(*kind), "data" => Value::Bytes(data.clone()) }.into(),
        other => other.clone(),
    }
}

/// The request's Range header when it asks for one span of bytes (`singleRange`, `/^bytes=\d*-\d*$/`); other forms
/// get the whole body.
///
/// # Arguments
/// * `headers` - Request headers.
pub fn single_range(headers: &Headers) -> Option<&str> {
    let range = headers.get("range")?;
    let (first, last) = range.strip_prefix("bytes=")?.split_once('-')?;
    [first, last].iter().all(|part| part.bytes().all(|byte| byte.is_ascii_digit())).then_some(range)
}

/// A whole body, or the part the request's `Range` header asks for, as `sendRange` sends it: video and audio
/// elements stall on a plain 200.
///
/// # Arguments
/// * `request_headers` - Headers of the request; only `range` is read.
/// * `status` - Status of the whole body; only a 200 is split.
/// * `body` - Whole body.
/// * `headers` - Response headers.
pub fn send_range(request_headers: &Headers, status: u16, body: Vec<u8>, headers: &Headers) -> Response {
    let size = body.len();
    let part = byte_range(single_range(request_headers), size as f64);
    if status != 200 || part.status == 200 {
        return Response::send(status, body, headers);
    }
    if part.status == 416 {
        return Response::send(416, "", &headers.clone().with(&Headers::from([("content-range", format!("bytes */{size}"))])));
    }
    let (start, end) = (part.start as usize, part.end as usize);
    let range = Headers::from([("accept-ranges", "bytes".to_owned()), ("content-range", format!("bytes {start}-{end}/{size}"))]);
    Response::send(206, body[start..=end].to_vec(), &headers.clone().with(&range))
}

/// The recording key of a request, in the shape the old recorder used
/// (`replayKeyFor`).
///
/// # Arguments
/// * `method` - Request method.
/// * `headers` - Request headers; `sec-fetch-dest` is read.
/// * `url` - Parsed replica URL.
///
/// # Returns
/// `<KIND> <live URL>`, where KIND is `DOC` for a top-level document; media URLs drop their query.
pub fn replay_key_for(method: &str, headers: &Headers, url: &Url) -> String {
    let pathname = url.path();
    let local = LOCAL_PREFIXES.iter().find(|(prefix, _)| pathname.starts_with(prefix));
    let live = match local {
        Some((prefix, origin)) => format!("{origin}{}", &pathname[prefix.len()..]),
        None => format!("{APP_ORIGIN}{}", pathname.get(1..).unwrap_or("")),
    };
    let path_only = local.is_some_and(|(prefix, _)| *prefix != "/_static/") || pathname.starts_with("/media/");
    let kind = if method == "GET" && headers.get("sec-fetch-dest") == Some("document") { "DOC" } else { method };
    let search = if path_only { String::new() } else { mm_js::url::search(url) };
    format!("{kind} {live}{search}")
}
