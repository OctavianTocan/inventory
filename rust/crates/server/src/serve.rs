//! `handle` of `server.mjs`: sign-in and the tunnel refusal, then a hosted library or the
//! library API; and each listener's `/hub` WebSocket.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use async_trait::async_trait;
use mm_core::auth::Auth;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::hub::{Allows, Hub, accept_key};
use mm_core::site::{LOCAL_PREFIXES, LibraryApi, Site, content_type, is_text_extension, localize};
use mm_core::Error;

use crate::http::{Handler, Upgrade};
use crate::node_path::{extname, read_file, resolve};

fn file_inside(root: &Path, relative: &str) -> Option<PathBuf> {
    let decoded = mm_js::uri::decode_uri_component(relative).ok()?;
    if decoded.starts_with('/') || decoded.split('/').any(|part| part == ".." || part == ".") {
        return None;
    }
    let resolved = resolve(root, &decoded);
    let prefix = format!("{}/", root.to_string_lossy());
    (resolved == root || resolved.to_string_lossy().starts_with(&prefix)).then_some(resolved)
}

/// What one server answers with, shared by its listeners.
pub struct Server {
    /// Mirror directory (`mirror/static`).
    pub mirror: PathBuf,
    /// Outbound HTTP for missing website code when not offline.
    pub fetch: Rc<dyn Fetch>,
    /// Sign-in, when `AUTH=google` turned it on.
    pub auth: Option<Rc<Auth>>,
    /// The library API over `STORE_DB`, when that database exists.
    pub api: Option<Rc<dyn LibraryApi>>,
    /// The hosted library (`HOSTED=1`), which answers everything after sign-in.
    pub hosted: Option<Rc<dyn Site>>,
    /// `OFFLINE=1`.
    pub offline: bool,
}

impl Server {
    /// The hosted library's or the library API's answer, or `None` when neither serves the request.
    async fn library(&self, request: &mut Request) -> mm_core::Result<Option<Response>> {
        if let Some(hosted) = &self.hosted {
            return hosted.handle(request).await.map(Some);
        }
        match &self.api {
            Some(api) => api.handle(request).await,
            None => Ok(None),
        }
    }

    /// Claude Code and Codex (`mm_ai`) for the signed-in account, with the library's own search for the cards an AI
    /// search ranks.
    async fn answer_ai(&self, request: &mut Request) -> mm_core::Result<Response> {
        let login = mm_ai::login_for(request.header("x-mymind-email"));
        let body = request.take_body().bytes_within(64 * 1024).await?;
        let origin = request.url.origin().ascii_serialization();
        let mut headers = request.headers.clone();
        headers.remove("content-type");
        headers.remove("content-length");
        let candidates = |target: String| async move {
            let mut search = Request::new("GET", &format!("{origin}{target}")).map_err(|error| anyhow::anyhow!(error.to_js_string()))?;
            search.headers = headers;
            let response = match self.library(&mut search).await {
                Ok(Some(response)) if response.status == 200 => response,
                Ok(_) => anyhow::bail!("The library search did not answer."),
                Err(error) => anyhow::bail!(error.to_js_string()),
            };
            let bytes = response.body.bytes().await.map_err(|error| anyhow::anyhow!(error.to_js_string()))?;
            Ok(serde_json::from_slice(&bytes)?)
        };
        let reply = mm_ai::answer(&login, &request.method, &request.url, &body, candidates)
            .await
            .unwrap_or_else(|| unreachable!("answer_ai runs only for the paths mm_ai serves"));
        let headers = Headers::from([("content-type", "application/json"), ("cache-control", "no-store")]);
        Ok(Response::send(reply.status, reply.body.to_string(), &headers))
    }

    async fn serve_static(&self, relative: &str, search: &str) -> mm_core::Result<Response> {
        let Some(file) = file_inside(&self.mirror, relative) else {
            return Ok(Response::send(400, "bad path", &Headers::from([("content-type", "text/plain; charset=utf-8")])));
        };
        if let Ok(mut body) = read_file(&file).await {
            let extension = extname(&file.to_string_lossy()).to_lowercase();
            if is_text_extension(&extension) {
                body = localize(&String::from_utf8_lossy(&body)).into_bytes();
            }
            let headers = Headers::from([("content-type", content_type(&extension).unwrap_or("application/octet-stream"))]);
            return Ok(Response::send(200, body, &headers));
        }
        if self.offline {
            return Ok(Response::send(404, "not mirrored", &Headers::from([("content-type", "text/plain; charset=utf-8")])));
        }
        let request = Request::new("GET", &format!("{}{relative}{search}", LOCAL_PREFIXES[0].1))?;
        let upstream = self.fetch.fetch(request).await?;
        if !(200..300).contains(&upstream.status) {
            return Ok(Response::send(upstream.status, format!("upstream {}", upstream.status), &Headers::from([("content-type", "text/plain; charset=utf-8")])));
        }
        let mut body = upstream.body.bytes().await?;
        if let Some(parent) = file.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|error| Error::internal(error.to_string()))?;
        }
        tokio::fs::write(&file, &body).await.map_err(|error| Error::internal(error.to_string()))?;
        let extension = extname(&file.to_string_lossy()).to_lowercase();
        if is_text_extension(&extension) {
            body = localize(&String::from_utf8_lossy(&body)).into_bytes();
        }
        let headers = Headers::from([("content-type", content_type(&extension).unwrap_or("application/octet-stream"))]);
        Ok(Response::send(200, body, &headers))
    }
}

/// One listening port of a [`Server`]: the main port behind sign-in, or `LOCAL_PORT` without it.
pub struct Listener {
    server: Rc<Server>,
    gated: bool,
    hub: Rc<Hub>,
}

fn tunnelled(request: &Request) -> bool {
    request.header("cf-connecting-ip").is_some_and(|ip| !ip.is_empty())
}

impl Listener {
    /// A listener of `server` with its own hub.
    ///
    /// # Arguments
    /// * `server` - What it answers with.
    /// * `gated` - Whether requests pass the server's sign-in first; the `LOCAL_PORT` listener has none.
    pub fn new(server: Rc<Server>, gated: bool) -> Self {
        let allows: Allows = match server.auth.clone().filter(|_| gated) {
            Some(auth) => Rc::new(move |request: &Request| auth.allows_upgrade(request)),
            None => Rc::new(|request: &Request| Ok(!tunnelled(request))),
        };
        Self { server, gated, hub: Hub::new(allows) }
    }

    /// This listener's hub, for the server's broadcasts.
    pub fn hub(&self) -> Rc<Hub> {
        Rc::clone(&self.hub)
    }

    async fn answer(&self, request: &mut Request) -> mm_core::Result<Response> {
        let server = &self.server;
        // The client cannot choose this header. Sign-in stamps the session email the library uses for the owner check.
        request.headers.remove("x-mymind-email");
        let gate = server.auth.as_ref().filter(|_| self.gated);
        if let Some(gate) = gate {
            if let Some(response) = gate.handle(request).await? {
                return Ok(response);
            }
            if let Some(identity) = gate.identity(request)? {
                request.headers.set("x-mymind-email", &identity.email);
            }
        }
        if gate.is_none() && tunnelled(request) {
            return Ok(Response::send(403, "This server has no sign-in configured.", &Headers::new()));
        }
        let pathname = request.pathname().to_owned();
        if mm_ai::serves(&pathname) {
            return server.answer_ai(request).await;
        }
        if let Some(response) = server.library(request).await? {
            return Ok(response);
        }
        let search = mm_js::url::search(&request.url);
        if let Some(relative) = pathname.strip_prefix("/_static/") {
            return server.serve_static(relative, &search).await;
        }
        if request.method == "GET" && request.header("sec-fetch-dest") == Some("document") {
            let body = match &server.api {
                Some(api) => api.shell()?,
                None => mm_core::api::SHELL_TEMPLATE.to_owned(),
            };
            return Ok(Response::send(200, body, &Headers::from([("content-type", "text/html; charset=utf-8")])));
        }
        if request.method != "GET" {
            if pathname != "/jserrors" {
                mm_core::runtime::log(&format!("[write ignored] {} {pathname}{search}", request.method));
            }
            return Ok(Response::send(200, "{}", &Headers::from([("content-type", "application/json")])));
        }
        let accept = request.header("accept").unwrap_or("-");
        mm_core::runtime::log(&format!("[miss] GET {pathname}{search} (accept: {accept})"));
        Ok(if accept.contains("json") {
            Response::send(200, "{}", &Headers::from([("content-type", "application/json")]))
        } else {
            Response::send(404, "", &Headers::from([("content-type", "text/plain")]))
        })
    }
}

#[async_trait(?Send)]
impl Handler for Listener {
    async fn handle(&self, mut request: Request) -> Response {
        let key = format!("{} {}{}", request.method, request.pathname(), request.search());
        match self.answer(&mut request).await {
            Ok(response) => response,
            Err(error) => {
                let text = error.to_js_string();
                eprintln!("[error] {key} {text}");
                Response::send(500, text, &Headers::new())
            }
        }
    }

    fn upgrade(&self, request: &Request) -> Upgrade {
        if !self.hub.admits(request) {
            return Upgrade::Refuse;
        }
        let accept = accept_key(request.header("sec-websocket-key").unwrap_or_default());
        let headers = Headers::from([("upgrade", "websocket".to_owned()), ("connection", "Upgrade".to_owned()), ("sec-websocket-accept", accept)]);
        let (hub, head) = (Rc::clone(&self.hub), request.head());
        Upgrade::Accept(Response::new(101, headers, Body::Empty), Box::new(move |io| Box::pin(crate::socket::run(hub, head, io))))
    }
}
