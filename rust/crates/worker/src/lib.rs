//! The Cloudflare Worker that hosts the libraries (`worker/index.mjs`): every request is checked by the `Auth` object,
//! a signed-in one goes to its account's `Library` object, and `/_admin/` exists only while `ADMIN_TOKEN` is set.
//! Claude Code and Codex (`/app/v1/ai/`) run in each account's own `AgentHome` container.

mod auth_object;
mod bucket_r2;
mod fetch;
mod http;
mod library_object;
pub mod media_client;
mod media_object;
mod runtime;
mod sql_do;
mod util;

use mm_core::host::Bucket;
use mm_core::http::{Body, Headers, Response};
use mm_core::{Error, Result};
use mm_js::{Value, json, obj};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use wasm_bindgen::JsValue;
use worker::{Env, event};

// Every account's Claude Code and Codex container is an `AgentHome` of the AI kit shared with Pully.
pub use ai_kit::home::AgentHome;
pub use auth_object::Auth;
pub use library_object::Library;
pub use media_object::Media;

use auth_object::CALL;
use bucket_r2::R2Bucket;
use http::{headers_of, response_from, response_into, text};
use util::{js_error, worker_error};

fn signin_body(path: &str) -> bool {
    matches!(path, "/signin/password" | "/app/v1/auth/nonce" | "/app/v1/auth/google" | "/app/v1/auth/password")
}

fn is_admin(headers: &Headers, token: Option<&String>) -> bool {
    let Some(token) = token.filter(|token| !token.is_empty()) else { return false };
    let given = headers.get("authorization").and_then(|value| value.strip_prefix("Bearer ")).filter(|value| !value.is_empty() && !value.contains(char::is_whitespace));
    given.is_some_and(|given| Sha256::digest(given.as_bytes()).ct_eq(&Sha256::digest(token.as_bytes())).into())
}

fn object_key(path: &str) -> Option<String> {
    let key = percent_decode(path)?;
    (!key.is_empty() && !key.starts_with('/') && !key.split('/').any(|part| matches!(part, "" | "." | ".."))).then_some(key)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn json_response(status: u16, value: &Value) -> Result<Response> {
    Ok(Response::new(status, Headers::from([("content-type", "application/json")]), json::stringify(value)?.unwrap_or_default()))
}

fn empty(status: u16) -> Response {
    Response::new(status, Headers::new(), Body::Empty)
}

fn forward(url: &str, method: &str, headers: &Headers, body: Option<web_sys::ReadableStream>) -> Result<web_sys::Request> {
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    init.set_redirect(web_sys::RequestRedirect::Manual);
    let out = web_sys::Headers::new().map_err(js_error)?;
    for (name, value) in headers.iter() {
        out.append(name, value).map_err(js_error)?;
    }
    init.set_headers(&out);
    if let Some(body) = body {
        init.set_body(&body);
    }
    web_sys::Request::new_with_str_and_init(url, &init).map_err(js_error)
}

async fn send(stub: worker::Stub, request: web_sys::Request) -> Result<Response> {
    let response = stub.fetch_with_request(worker::Request::from(request)).await.map_err(worker_error)?;
    let response: web_sys::Response = response.into();
    response_into(&response)
}

fn auth_stub(env: &Env) -> Result<worker::Stub> {
    env.durable_object("AUTH_OBJECT").map_err(worker_error)?.get_by_name("auth").map_err(worker_error)
}

async fn auth_call(env: &Env, call: &str, body: Option<String>) -> Result<Response> {
    let method = if body.is_some() { "POST" } else { "GET" };
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    let headers = web_sys::Headers::new().map_err(js_error)?;
    headers.append(CALL, call).map_err(js_error)?;
    init.set_headers(&headers);
    if let Some(body) = body {
        init.set_body(&JsValue::from_str(&body));
    }
    let request = web_sys::Request::new_with_str_and_init(&format!("http://auth/{call}"), &init).map_err(js_error)?;
    send(auth_stub(env)?, request).await
}

async fn body_text(request: &web_sys::Request) -> Result<String> {
    Ok(String::from_utf8_lossy(&http::request_from(request)?.body.bytes().await?).into_owned())
}

async fn admin(request: &web_sys::Request, env: &Env, url: &url::Url) -> Result<Response> {
    let segments = url.path().split('/').skip(2).collect::<Vec<_>>();
    let (area, rest) = (segments.first().copied().unwrap_or_default(), segments.get(1..).unwrap_or_default());
    let method = request.method();
    let bucket = R2Bucket(env.bucket("BUCKET").map_err(worker_error)?);
    if area == "bucket" && rest.is_empty() && method == "GET" {
        let prefix = url.query_pairs().find(|(name, _)| name == "prefix").map(|(_, value)| value.into_owned()).unwrap_or_default();
        let listed = bucket.list(&prefix).await?;
        return json_response(200, &obj! { "objects" => listed.len(), "bytes" => listed.iter().map(|object| object.size as f64).sum::<f64>() }.into());
    }
    if area == "bucket" {
        let Some(key) = object_key(&rest.join("/")) else { return Ok(text(400, "Invalid object key", &Headers::new())) };
        if method == "HEAD" {
            return Ok(match bucket.head(&key).await? {
                Some(object) => Response::new(200, Headers::from([("x-object-size", object.size.to_string())]), Body::Empty),
                None => empty(404),
            });
        }
        if method != "PUT" {
            return Ok(text(405, "Method not allowed", &Headers::new()));
        }
        let given = request.headers().get("x-object-headers").ok().flatten().filter(|value| !value.is_empty());
        let stored = match &given {
            Some(given) => {
                let parsed = json::parse(given)?;
                let map = parsed.as_object().ok_or_else(|| Error::internal("x-object-headers must be an object"))?;
                Some(Headers::from_pairs(map.iter().map(|(name, value)| (name.into_owned(), value.to_js_string()))))
            }
            None => None,
        };
        let length = request.headers().get("content-length").ok().flatten().and_then(|value| value.parse().ok());
        let body = http::request_from(request)?.body;
        bucket.write(&key, body, length, stored.as_ref()).await?;
        return Ok(empty(204));
    }
    if area == "library" && rest.len() == 2 && !rest[0].is_empty() && rest[0].bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
        let headers = Headers::from([("x-mymind-library", rest[0]), ("x-mymind-admin", rest[1]), ("content-type", "application/json")]);
        let stub = env.durable_object("LIBRARY").map_err(worker_error)?.get_by_name(rest[0]).map_err(worker_error)?;
        return send(stub, forward(url.as_str(), &method, &headers, request.body())?).await;
    }
    if area == "auth" && rest.is_empty() && method == "GET" {
        return auth_call(env, "summary", None).await;
    }
    if area == "auth" && rest.first() == Some(&"user") && method == "POST" {
        auth_call(env, "putUser", Some(body_text(request).await?)).await?;
        return Ok(empty(204));
    }
    if area == "auth" && rest.first() == Some(&"library") && method == "POST" {
        auth_call(env, "setLibrary", Some(body_text(request).await?)).await?;
        return Ok(empty(204));
    }
    Ok(text(404, "Not found", &Headers::new()))
}

async fn download(request: &web_sys::Request, env: &Env, url: &url::Url, user: &str) -> Result<Response> {
    let method = request.method();
    if method != "GET" && method != "HEAD" {
        return Ok(text(405, "", &Headers::from([("allow", "GET, HEAD")])));
    }
    let bucket = R2Bucket(env.bucket("BUCKET").map_err(worker_error)?);
    let manifest = match bucket.bytes("downloads/manifest.json").await? {
        Some(bytes) => json::parse(&String::from_utf8_lossy(&bytes))?,
        None => obj! { "files" => Vec::<Value>::new() }.into(),
    };
    if url.path() == "/download" || url.path() == "/download/" {
        let page = mm_core::download_page::download_page(&manifest, user)?;
        return Ok(Response::new(200, Headers::from([("content-type", "text/html; charset=utf-8"), ("cache-control", "no-store")]), page));
    }
    let name = percent_decode(&url.path()["/download/".len()..]);
    if name.as_deref() == Some(mm_core::download_page::ANDROID_RELEASE)
        && let Some(release) = mm_core::download_page::android_release(&manifest)
    {
        let headers = Headers::from([("content-type", "application/json; charset=utf-8"), ("cache-control", "no-store")]);
        let body = json::stringify(&release)?.unwrap_or_default();
        return Ok(Response::new(200, headers, if method == "HEAD" { Body::Empty } else { body.into() }));
    }
    let file = name.as_ref().and_then(|name| manifest.get("files").as_array().into_iter().flatten().find(|entry| entry.get("name").as_str() == Some(name)));
    let Some(file) = file else { return Ok(text(404, "Not found.\n", &Headers::new())) };
    let name = file.get("name").to_js_string();
    let kind = mm_core::download_page::content_type(file);
    let key = format!("downloads/{name}");
    let (size, body) = if method == "HEAD" {
        match bucket.head(&key).await? {
            Some(object) => (object.size, Body::Empty),
            None => return Ok(text(404, "Not found.\n", &Headers::new())),
        }
    } else {
        match bucket.read(&key, None).await? {
            Some(object) => (object.size, Body::Stream(object.body)),
            None => return Ok(text(404, "Not found.\n", &Headers::new())),
        }
    };
    let headers = Headers::from([
        ("content-type", kind.to_owned()),
        ("content-length", size.to_string()),
        ("content-disposition", format!("attachment; filename=\"{name}\"")),
        ("cache-control", "no-store".to_owned()),
    ]);
    Ok(Response::new(200, headers, body))
}

/// Claude Code and Codex for the signed-in account (`mm_ai`), its sign-ins kept by its `AgentHome`, with its library's
/// own search for the cards an AI search ranks.
async fn ai(request: &web_sys::Request, env: &Env, url: &url::Url, headers: &Headers, library: worker::Stub) -> Result<Response> {
    ai_kit::home::install(env, "AGENT_HOME").map_err(worker_error)?;
    ai_kit::rt::env::set(runtime::bindings(env));
    let login = mm_ai::login_for(headers.get("x-mymind-email"));
    let body = http::request_from(request)?.body.bytes_within(64 * 1024).await?;
    let origin = url.origin().ascii_serialization();
    let mut search = headers.clone();
    for name in ["content-type", "content-length", "transfer-encoding"] {
        search.remove(name);
    }
    let candidates = |target: String| async move {
        let failed = |error: Error| anyhow::anyhow!(error.to_string());
        let response = send(library, forward(&format!("{origin}{target}"), "GET", &search, None).map_err(failed)?).await.map_err(failed)?;
        anyhow::ensure!(response.status == 200, "The library search did not answer.");
        Ok(serde_json::from_slice(&response.body.bytes().await.map_err(failed)?)?)
    };
    let reply = mm_ai::answer(&login, &request.method(), url, &body, candidates)
        .await
        .unwrap_or_else(|| unreachable!("ai runs only for the paths mm_ai serves"));
    Ok(Response::new(reply.status, Headers::from([("content-type", "application/json"), ("cache-control", "no-store")]), reply.body.to_string()))
}

/// An answer built here, or a Durable Object's own response, kept whole because a WebSocket upgrade cannot be copied.
enum Reply {
    Built(Response),
    Passed(worker::Response),
}

async fn route(request: web_sys::Request, env: Env) -> Result<Reply> {
    runtime::configure(&env);
    let vars = runtime::bindings(&env);
    if vars.get("AUTH").map(String::as_str) != Some("google") {
        return Ok(Reply::Built(text(503, "This server has no sign-in configured.", &Headers::new())));
    }
    let url = url::Url::parse(&request.url()).map_err(|error| Error::internal(error.to_string()))?;
    let mut headers = headers_of(&request.headers())?;
    let names = headers.iter().map(|(name, _)| name.to_owned()).filter(|name| name.starts_with("x-mymind-")).collect::<Vec<_>>();
    for name in names {
        headers.remove(&name);
    }
    if headers.get("cf-connecting-ip").is_none() {
        headers.set("cf-connecting-ip", "127.0.0.1");
    }
    if url.path().starts_with("/_admin/") && is_admin(&headers, vars.get("ADMIN_TOKEN")) {
        return admin(&request, &env, &url).await.map(Reply::Built);
    }

    let method = request.method();
    let with_body = method == "POST" && signin_body(url.path());
    let mut gate_headers = headers.clone();
    if !with_body {
        gate_headers.remove("content-length");
        gate_headers.remove("transfer-encoding");
    }
    let gate = send(auth_stub(&env)?, forward(url.as_str(), &method, &gate_headers, if with_body { request.body() } else { None })?).await?;
    let Some(library) = gate.headers.get("x-mymind-library").map(str::to_owned) else {
        let listing = url.path() == "/download" || url.path() == "/download/";
        if listing && gate.status == 401 {
            return Ok(Reply::Built(Response::new(302, Headers::from([("location", "/signin"), ("cache-control", "no-store")]), Body::Empty)));
        }
        return Ok(Reply::Built(gate));
    };
    if with_body {
        return Ok(Reply::Built(text(500, "Sign-in did not answer", &Headers::new())));
    }
    if url.path() == "/download" || url.path().starts_with("/download/") {
        return download(&request, &env, &url, gate.headers.get("x-mymind-email").unwrap_or_default()).await.map(Reply::Built);
    }
    headers.set("x-mymind-library", &library);
    headers.set("x-mymind-credential", gate.headers.get("x-mymind-credential").unwrap_or_default());
    headers.set("x-mymind-email", gate.headers.get("x-mymind-email").unwrap_or_default());
    headers.set("x-mymind-sub", gate.headers.get("x-mymind-sub").unwrap_or_default());
    let stub = env.durable_object("LIBRARY").map_err(worker_error)?.get_by_name(&library).map_err(worker_error)?;
    if mm_ai::serves(url.path()) {
        return ai(&request, &env, &url, &headers, stub).await.map(Reply::Built);
    }
    let forwarded = forward(url.as_str(), &method, &headers, request.body())?;
    Ok(Reply::Passed(stub.fetch_with_request(worker::Request::from(forwarded)).await.map_err(worker_error)?))
}

/// Names the AI kit's volumes and containers before any object answers, since an `AgentHome` checks logins against them.
#[event(start)]
fn start() {
    mm_ai::configure();
}

/// Route a Worker request through sign-in and its assigned library.
///
/// # Arguments
/// * `request` - The incoming request.
/// * `env` - The Worker's bindings, variables and secrets.
///
/// # Errors
/// A failure outside any handler, which the runtime reports as a 500.
#[event(fetch)]
pub async fn fetch(request: web_sys::Request, env: Env, _context: worker::Context) -> worker::Result<worker::Response> {
    match route(request, env).await {
        Ok(Reply::Built(response)) => response_from(response).map_err(|error| worker::Error::RustError(error.to_string())),
        Ok(Reply::Passed(response)) => Ok(response),
        Err(error) => Err(worker::Error::RustError(error.to_string())),
    }
}
