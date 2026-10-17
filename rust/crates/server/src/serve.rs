//! `handle` of `server.mjs`: sign-in, the tunnel refusal and the framed-media redirect, then a hosted library, or the
//! library API in front of the replay replica; and each listener's `/hub` WebSocket.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use async_trait::async_trait;
use mm_core::auth::{Auth, SigninPages};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::hub::{Allows, Hub, accept_key};
use mm_core::site::{LibraryApi, Site};
use mm_core::{Error, Value, obj};
use mm_js::json;

use crate::http::{Handler, Upgrade};
use crate::node_path::read_file;
use crate::replica::Replica;

/// mymind's recorded sign-in pages from `capture/signin/`.
pub struct FilePages {
    directory: PathBuf,
}

impl FilePages {
    /// The pages of the repository at `root`.
    ///
    /// # Arguments
    /// * `root` - Repository root.
    pub fn new(root: &Path) -> Self {
        Self { directory: root.join("capture").join("signin") }
    }
}

#[async_trait(?Send)]
impl SigninPages for FilePages {
    async fn page(&self, name: &str) -> mm_core::Result<String> {
        let bytes = read_file(&self.directory.join(format!("{name}.html"))).await.map_err(Error::Internal)?;
        Ok(mm_core::site::localize(&String::from_utf8_lossy(&bytes)))
    }
}

/// What one server answers with, shared by its listeners.
pub struct Server {
    /// The recordings.
    pub replica: Rc<Replica>,
    /// Sign-in, when `AUTH=google` turned it on.
    pub auth: Option<Rc<Auth>>,
    /// The library API over `STORE_DB`, when that database exists.
    pub api: Option<Rc<dyn LibraryApi>>,
    /// The hosted library (`HOSTED=1`), which answers everything after sign-in.
    pub hosted: Option<Rc<dyn Site>>,
    /// `OFFLINE=1`.
    pub offline: bool,
    /// `STORE_DB` resolved against the working directory, as `/__replay/reload` reports it.
    pub store: Option<PathBuf>,
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
        let gate = server.auth.as_ref().filter(|_| self.gated);
        if let Some(gate) = gate
            && let Some(response) = gate.handle(request).await?
        {
            return Ok(response);
        }
        if gate.is_none() && tunnelled(request) {
            return Ok(Response::send(403, "This server has no sign-in configured.", &Headers::new()));
        }
        let pathname = request.pathname().to_owned();
        let dest = request.header("sec-fetch-dest");
        if request.method == "GET"
            && pathname.starts_with("/media/")
            && matches!(dest, Some("iframe" | "frame"))
            && request.header("host").is_some_and(|host| host.starts_with("localhost"))
        {
            let location = format!("http://127.0.0.1:{}/_mediahost/{}{}", request.local_port, &pathname["/media/".len()..], request.search());
            return Ok(Response::send(302, "", &Headers::from([("location", location)])));
        }
        if let Some(hosted) = &server.hosted {
            return hosted.handle(request).await;
        }
        if request.method == "POST" && pathname == "/__replay/reload" {
            let store = server.store.as_ref().map_or(Value::Null, |path| Value::from(path.to_string_lossy().into_owned()));
            let body = obj! { "responses" => server.replica.reload_replay().await? as f64, "offline" => server.offline, "store" => store };
            return Ok(Response::send(200, json::to_string(&body.into()), &Headers::from([("content-type", "application/json")])));
        }
        if let Some(api) = &server.api
            && let Some(response) = api.handle(request).await?
        {
            return Ok(response);
        }
        server.replica.handle(request, server.api.as_deref()).await
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
