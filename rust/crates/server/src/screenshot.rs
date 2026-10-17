//! Port of `screenshotPage` and `screenshotResource` of `lib/capture-link.mjs`: headless Chromium driven over the Chrome
//! DevTools Protocol on a pipe, with every request answered by the capture transport.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::rc::{Rc, Weak};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::future::try_join_all;
use mm_core::capture_http::{capture_destination, is_blocked};
use mm_core::host::{ResourceOptions, Screenshot};
use mm_core::runtime::random_bytes;
use mm_core::{Error, Result, Value};
use mm_js::{arr, base64, json, obj};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::pipe::{Receiver, Sender};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, Notify, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::capture_http::{Budget, CaptureTransport};

// Playwright's own launch switches, plus one Chromium switch: one renderer for every frame, so the page's Fetch
// interception also sees cross-origin iframe requests (Playwright attaches to each iframe target instead).
const ARGS: [&str; 47] = [
    "--disable-field-trial-config", "--disable-background-networking", "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows", "--disable-back-forward-cache", "--disable-breakpad",
    "--disable-client-side-phishing-detection", "--disable-component-extensions-with-background-pages",
    "--disable-component-update", "--no-default-browser-check", "--disable-default-apps", "--disable-dev-shm-usage",
    "--disable-extensions",
    "--disable-features=AvoidUnnecessaryBeforeUnloadCheckSync,BoundaryEventDispatchTracksNodeRemoval,\
DestroyProfileOnBrowserClose,DialMediaRouteProvider,GlobalMediaControls,HttpsUpgrades,LensOverlay,MediaRouter,PaintHolding,\
ThirdPartyStoragePartitioning,Translate,AutoDeElevate,RenderDocument,OptimizationHints,IsolateOrigins,site-per-process",
    "--enable-features=CDPScreenshotNewSurface", "--allow-pre-commit-input", "--disable-hang-monitor",
    "--disable-ipc-flooding-protection", "--disable-popup-blocking", "--disable-prompt-on-repost",
    "--disable-renderer-backgrounding", "--force-color-profile=srgb", "--metrics-recording-only", "--no-first-run",
    "--password-store=basic", "--use-mock-keychain", "--no-service-autorun", "--export-tagged-pdf",
    "--disable-search-engine-choice-screen", "--unsafely-disable-devtools-self-xss-warnings",
    "--edge-skip-compat-layer-relaunch", "--enable-automation", "--disable-infobars", "--disable-sync",
    "--enable-unsafe-swiftshader", "--headless", "--hide-scrollbars", "--mute-audio",
    "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
    "--no-sandbox", "--proxy-server=http://capture-denied.invalid:9", "--proxy-bypass-list=<-loopback>", "--disable-quic",
    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp", "--host-resolver-rules=MAP * ~NOTFOUND",
    "--remote-debugging-pipe", "--no-startup-window",
];
const INIT_SCRIPT: &str = "(() => { for (const name of ['RTCPeerConnection', 'webkitRTCPeerConnection', 'RTCDataChannel', \
    'WebTransport']) { try { Object.defineProperty(globalThis, name, { value: undefined, configurable: false, writable: false }); } catch {} } })()";
// What Playwright adds to a context with `serviceWorkers: 'block'`.
const BLOCK_SERVICE_WORKERS: &str = "\nif (navigator.serviceWorker) navigator.serviceWorker.register = async () => { console.warn('Service Worker \
    registration blocked by Playwright'); };\n";
// Playwright's `caret: 'hide'` before a screenshot, run in every frame.
const HIDE_CARET: &str = "(() => { const roots = []; const collect = (root) => { roots.push(root); const walker = \
    document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT); do { const node = walker.currentNode; const shadow = node instanceof \
    Element ? node.shadowRoot : null; if (shadow) collect(shadow); } while (walker.nextNode()); }; collect(document); for (const \
    root of roots) root.querySelectorAll('input,textarea,[contenteditable]').forEach((element) => \
    element.style.setProperty('caret-color', 'transparent', 'important')); })()";
const RESOURCE_LIMIT: u64 = 25 * 1024 * 1024;
const BUDGET: i64 = 100 * 1024 * 1024;

type Handler = Rc<dyn Fn(Value)>;
type Reply = oneshot::Sender<Result<Value>>;

/// Link-capture screenshots with a headless Chromium (`screenshotPage`).
pub struct ChromiumScreenshot {
    executable: PathBuf,
    transport: Rc<CaptureTransport>,
}

impl ChromiumScreenshot {
    /// A screenshot host that launches `executable` for every capture.
    ///
    /// # Arguments
    /// * `executable` - Chromium binary, normally `VERIFIER_CHROMIUM` or `/usr/lib/chromium/chromium`.
    /// * `transport` - The only network path the browser's requests are answered through.
    pub fn new(executable: PathBuf, transport: Rc<CaptureTransport>) -> Self {
        Self { executable, transport }
    }
}

fn fail(message: impl Into<String>) -> Error {
    Error::internal(message)
}

fn io(error: std::io::Error) -> Error {
    Error::internal(error.to_string())
}

/// One DevTools connection over the browser's pipes.
struct Cdp {
    writer: Mutex<Sender>,
    next: Cell<u64>,
    pending: RefCell<HashMap<u64, (String, Reply)>>,
    closed: Cell<bool>,
    handler: RefCell<Option<Handler>>,
}

impl Cdp {
    /// Send a command and wait for its reply.
    async fn send(&self, method: &str, params: Value, session: Option<&str>) -> Result<Value> {
        if self.closed.get() {
            return Err(fail("Browser closed"));
        }
        let id = self.next.get() + 1;
        self.next.set(id);
        let mut message = obj! { "id" => id as f64, "method" => method, "params" => params };
        if let Some(session) = session {
            message.insert("sessionId", session);
        }
        let (reply, answer) = oneshot::channel();
        self.pending.borrow_mut().insert(id, (method.to_string(), reply));
        let mut bytes = json::to_string(&message.into()).into_bytes();
        bytes.push(0);
        if let Err(error) = self.writer.lock().await.write_all(&bytes).await {
            self.pending.borrow_mut().remove(&id);
            return Err(io(error));
        }
        answer.await.unwrap_or_else(|_| Err(fail("Browser closed")))
    }

    /// Route one parsed message from the browser to its command or to the event handler.
    fn dispatch(&self, message: Value) {
        let id = message.get("id").as_f64();
        let Some(id) = id else {
            let handler = self.handler.borrow().clone();
            if let Some(handler) = handler {
                handler(message);
            }
            return;
        };
        let Some((method, reply)) = self.pending.borrow_mut().remove(&(id as u64)) else { return };
        let error = message.get("error");
        let outcome = if error.is_object() {
            Err(fail(format!("Protocol error ({method}): {}", error.get("message").as_str().unwrap_or(""))))
        } else {
            Ok(message.get("result").clone())
        };
        let _ = reply.send(outcome);
    }

    fn close(&self) {
        self.closed.set(true);
        let pending: Vec<_> = self.pending.borrow_mut().drain().collect();
        for (_, (_, reply)) in pending {
            let _ = reply.send(Err(fail("Browser closed")));
        }
        self.handler.borrow_mut().take();
    }
}

/// Read NUL-terminated JSON messages from the browser until the pipe closes.
async fn read_messages(cdp: Rc<Cdp>, mut input: Receiver) {
    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        match input.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => buffer.extend_from_slice(&chunk[..count]),
        }
        while let Some(end) = buffer.iter().position(|byte| *byte == 0) {
            let line: Vec<u8> = buffer.drain(..=end).collect();
            if let Ok(message) = json::parse(&String::from_utf8_lossy(&line[..end])) {
                cdp.dispatch(message);
            }
        }
    }
    cdp.close();
}

/// A launched Chromium; dropping it kills the process and removes its profile.
struct Browser {
    cdp: Rc<Cdp>,
    child: Child,
    directory: PathBuf,
    reader: JoinHandle<()>,
}

fn nonblocking(fd: &OwnedFd) -> Result<()> {
    let raw = fd.as_raw_fd();
    // SAFETY: `raw` is a descriptor this process owns for the duration of both calls.
    unsafe {
        let flags = libc::fcntl(raw, libc::F_GETFL);
        if flags < 0 || libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(io(std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

impl Browser {
    async fn launch(executable: &Path) -> Result<Self> {
        let directory = std::env::temp_dir().join(format!("mymind-chromium-{}", base64::hex(&random_bytes(6))));
        tokio::fs::create_dir_all(&directory).await.map_err(io)?;
        match Self::start(executable, &directory) {
            Ok(browser) => Ok(browser),
            Err(error) => {
                let _ = std::fs::remove_dir_all(&directory);
                Err(error)
            }
        }
    }

    fn start(executable: &Path, directory: &Path) -> Result<Self> {
        let (commands_in, commands_out) = std::io::pipe().map_err(io)?;
        let (replies_in, replies_out) = std::io::pipe().map_err(io)?;
        let (child_read, child_write) = (commands_in.as_raw_fd(), replies_out.as_raw_fd());
        let mut command = Command::new(executable);
        command.args(ARGS).arg(format!("--user-data-dir={}", directory.display()));
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true);
        // SAFETY: the closure only calls async-signal-safe libc functions between fork and exec.
        unsafe {
            command.pre_exec(move || {
                // Duplicating above fd 4 first keeps dup2 from clobbering a source that already sits on 3 or 4.
                let read = libc::fcntl(child_read, libc::F_DUPFD_CLOEXEC, 10);
                let write = libc::fcntl(child_write, libc::F_DUPFD_CLOEXEC, 10);
                if read < 0 || write < 0 || libc::dup2(read, 3) < 0 || libc::dup2(write, 4) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn().map_err(io)?;
        drop((commands_in, replies_out));
        let sender_fd = OwnedFd::from(commands_out);
        let receiver_fd = OwnedFd::from(replies_in);
        nonblocking(&sender_fd)?;
        nonblocking(&receiver_fd)?;
        let writer = Sender::from_owned_fd(sender_fd).map_err(io)?;
        let input = Receiver::from_owned_fd(receiver_fd).map_err(io)?;
        let cdp = Rc::new(Cdp {
            writer: Mutex::new(writer),
            next: Cell::new(0),
            pending: RefCell::new(HashMap::new()),
            closed: Cell::new(false),
            handler: RefCell::new(None),
        });
        let reader = tokio::task::spawn_local(read_messages(cdp.clone(), input));
        Ok(Self { cdp, child, directory: directory.to_path_buf(), reader })
    }

    /// Ask the browser to quit, then make sure it is gone.
    async fn shut_down(mut self) {
        let _ = timeout(Duration::from_secs(2), self.cdp.send("Browser.close", obj! {}.into(), None)).await;
        if timeout(Duration::from_secs(3), self.child.wait()).await.is_err() {
            let _ = self.child.start_kill();
            let _ = self.child.wait().await;
        }
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        self.reader.abort();
        self.cdp.close();
        let _ = self.child.start_kill();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// What the event handler shares with the capture: the page's DOMContentLoaded loads and the latest blocked error.
struct Page {
    session: String,
    loaded: RefCell<Vec<(String, String)>>,
    notify: Notify,
    blocked: RefCell<Option<Error>>,
}

/// Everything one intercepted request needs.
struct Interception {
    cdp: Weak<Cdp>,
    page: Rc<Page>,
    transport: Rc<CaptureTransport>,
    budget: Budget,
    offline: bool,
}

impl Interception {
    fn handle(&self, event: Value) {
        let params = event.get("params");
        match event.get("method").as_str() {
            Some("Page.lifecycleEvent") if params.get("name").as_str() == Some("DOMContentLoaded") => {
                let frame = params.get("frameId").as_str().unwrap_or("").to_string();
                let loader = params.get("loaderId").as_str().unwrap_or("").to_string();
                self.page.loaded.borrow_mut().push((frame, loader));
                self.page.notify.notify_one();
            }
            Some("Fetch.requestPaused") => {
                let (Some(cdp), Some(request)) = (self.cdp.upgrade(), params.get("requestId").as_str()) else { return };
                let url = params.get("request").get("url").as_str().unwrap_or("").to_string();
                let method = params.get("request").get("method").as_str().unwrap_or("").to_string();
                let origin = params.get("request").get("headers").as_object().and_then(|headers| {
                    headers.iter().find(|(name, _)| name.eq_ignore_ascii_case("origin")).and_then(|(_, value)| value.as_str().map(str::to_string))
                });
                let task = Self {
                    cdp: Rc::downgrade(&cdp),
                    page: self.page.clone(),
                    transport: self.transport.clone(),
                    budget: self.budget.clone(),
                    offline: self.offline,
                };
                let request = request.to_string();
                tokio::task::spawn_local(async move { task.answer(&cdp, &request, &url, &method, origin.as_deref()).await });
            }
            _ => {}
        }
    }

    /// Answer one paused request from the capture transport (`screenshotResource`), as `route.fulfill` and `route.abort` do.
    async fn answer(&self, cdp: &Cdp, request: &str, url: &str, method: &str, origin: Option<&str>) {
        let session = Some(self.page.session.as_str());
        let params = |extra: Value| {
            let mut all = obj! { "requestId" => request };
            all.extend(extra.as_object().expect("an object"));
            Value::from(all)
        };
        let local = ["data:", "blob:", "about:"].iter().any(|scheme| url.starts_with(scheme));
        if method == "GET" && local {
            let _ = cdp.send("Fetch.continueRequest", params(obj! {}.into()), session).await;
            return;
        }
        if method != "GET" {
            let _ = cdp.send("Fetch.failRequest", params(obj! { "errorReason" => "Failed" }.into()), session).await;
            return;
        }
        let options = ResourceOptions { offline: self.offline, limit: Some(RESOURCE_LIMIT) };
        match self.transport.fetch_resource(url, options, Some(self.budget.clone())).await {
            Ok(response) => {
                let mut headers: Vec<Value> = response
                    .headers
                    .iter()
                    .map(|(name, value)| obj! { "name" => name.to_lowercase(), "value" => value }.into())
                    .collect();
                if !response.bytes.is_empty() && !response.headers.has("content-length") {
                    headers.push(obj! { "name" => "content-length", "value" => response.bytes.len().to_string() }.into());
                }
                // Playwright allows a cross-origin fetch of a fulfilled response the page made with an Origin (web
                // fonts, for one); the capture transport sends none, so servers leave the CORS headers out.
                let allowed = headers.iter().any(|header| header.get("name").as_str() == Some("access-control-allow-origin"));
                if let (Some(origin), false) = (origin, allowed) {
                    let target = mm_js::url::parse(url, None);
                    if target.as_ref().is_some_and(|target| target.scheme().starts_with("http") && mm_js::url::origin(target) != origin.trim()) {
                        for (name, value) in [("access-control-allow-origin", origin), ("access-control-allow-credentials", "true"), ("vary", "Origin")] {
                            headers.push(obj! { "name" => name, "value" => value }.into());
                        }
                    }
                }
                let fulfil = obj! {
                    "responseCode" => response.status,
                    "responsePhrase" => status_text(response.status),
                    "responseHeaders" => Value::Array(headers),
                    "body" => base64::encode(&response.bytes),
                };
                let _ = cdp.send("Fetch.fulfillRequest", params(fulfil.into()), session).await;
            }
            Err(error) => {
                if is_blocked(&error) {
                    *self.page.blocked.borrow_mut() = Some(error);
                }
                let _ = cdp.send("Fetch.failRequest", params(obj! { "errorReason" => "Failed" }.into()), session).await;
            }
        }
    }
}

/// Playwright's reason phrase for a fulfilled status.
fn status_text(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        306 => "Switch Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}

fn string_field(value: &Value, key: &str) -> Result<String> {
    value.get(key).as_str().map(str::to_string).ok_or_else(|| fail(format!("Protocol reply lacks {key}")))
}

/// Wait until the navigation's DOMContentLoaded was seen.
async fn dom_content_loaded(page: &Page, frame: &str, loader: &str) {
    loop {
        if page.loaded.borrow().iter().any(|(f, l)| f == frame && (l == loader || loader.is_empty())) {
            return;
        }
        page.notify.notified().await;
    }
}

impl ChromiumScreenshot {
    async fn shoot(&self, cdp: &Rc<Cdp>, url: &str, offline: bool) -> Result<Vec<u8>> {
        let context = cdp.send("Target.createBrowserContext", obj! { "disposeOnDetach" => true }.into(), None).await?;
        let context = string_field(&context, "browserContextId")?;
        let target = cdp.send("Target.createTarget", obj! { "url" => "about:blank", "browserContextId" => context }.into(), None).await?;
        let target = string_field(&target, "targetId")?;
        let attached = cdp.send("Target.attachToTarget", obj! { "targetId" => target, "flatten" => true }.into(), None).await?;
        let page = Rc::new(Page {
            session: string_field(&attached, "sessionId")?,
            loaded: RefCell::new(Vec::new()),
            notify: Notify::new(),
            blocked: RefCell::new(None),
        });
        let interception = Rc::new(Interception {
            cdp: Rc::downgrade(cdp),
            page: page.clone(),
            transport: self.transport.clone(),
            budget: Rc::new(Cell::new(BUDGET)),
            offline,
        });
        *cdp.handler.borrow_mut() = Some(Rc::new(move |event| interception.handle(event)));
        let session = Some(page.session.as_str());
        let window = cdp.send("Browser.getWindowForTarget", obj! {}.into(), session).await?;
        let fonts = obj! {
            "standard" => "Times New Roman", "fixed" => "Monospace", "serif" => "Times New Roman", "sansSerif" => "Arial",
            "cursive" => "Comic Sans MS", "fantasy" => "Impact",
        };
        let features = [("prefers-color-scheme", "light"), ("prefers-reduced-motion", "no-preference"), ("forced-colors", "none"), ("prefers-contrast", "no-preference")]
            .map(|(name, value)| Value::from(obj! { "name" => name, "value" => value }));
        // Playwright's setup of a new page in a context with a 1400 by 940 viewport, a route on every URL, blocked
        // service workers and one init script; sent together, as Playwright does, and run by the browser in order.
        let setup: [(&str, Value); 13] = [
            ("Page.enable", obj! {}.into()),
            ("Page.setLifecycleEventsEnabled", obj! { "enabled" => true }.into()),
            ("Network.enable", obj! {}.into()),
            ("Network.setBypassServiceWorker", obj! { "bypass" => true }.into()),
            ("Network.setCacheDisabled", obj! { "cacheDisabled" => true }.into()),
            ("Fetch.enable", obj! { "patterns" => arr![obj! { "urlPattern" => "*", "requestStage" => "Request" }] }.into()),
            ("Emulation.setFocusEmulationEnabled", obj! { "enabled" => true }.into()),
            ("Browser.setWindowBounds", obj! { "windowId" => window.get("windowId").clone(), "bounds" => obj! { "width" => 1400, "height" => 940 } }.into()),
            ("Emulation.setDeviceMetricsOverride", obj! {
                "width" => 1400, "height" => 940, "screenWidth" => 1400, "screenHeight" => 940, "deviceScaleFactor" => 1, "mobile" => false,
                "screenOrientation" => obj! { "angle" => 0, "type" => "landscapePrimary" }, "dontSetVisibleSize" => false,
            }.into()),
            ("Page.setFontFamilies", obj! { "fontFamilies" => fonts }.into()),
            ("Emulation.setEmulatedMedia", obj! { "media" => "", "features" => Value::Array(features.to_vec()) }.into()),
            ("Page.addScriptToEvaluateOnNewDocument", obj! { "source" => BLOCK_SERVICE_WORKERS, "runImmediately" => true }.into()),
            ("Page.addScriptToEvaluateOnNewDocument", obj! { "source" => INIT_SCRIPT, "runImmediately" => true }.into()),
        ];
        try_join_all(setup.into_iter().map(|(method, params)| cdp.send(method, params, session))).await?;
        let blocked_or = |error: Error| page.blocked.borrow().clone().unwrap_or(error);
        if let Err(error) = goto(cdp, &page, url).await {
            return Err(blocked_or(error));
        }
        let fonts = obj! { "expression" => "document.fonts.ready.then(() => undefined)", "awaitPromise" => true };
        let _ = timeout(Duration::from_secs(2), cdp.send("Runtime.evaluate", fonts.clone().into(), session)).await;
        if let Some(error) = page.blocked.borrow().clone() {
            return Err(error);
        }
        let shot = timeout(Duration::from_secs(15), screenshot(cdp, &page, fonts.into()))
            .await
            .map_err(|_| fail("page.screenshot: Timeout 15000ms exceeded."))??;
        if let Some(error) = page.blocked.borrow().clone() {
            return Err(error);
        }
        Ok(shot)
    }
}

/// `page.screenshot({ type: 'png' })`: the caret hidden in every frame, fonts loaded, then the visual viewport.
async fn screenshot(cdp: &Cdp, page: &Page, fonts: Value) -> Result<Vec<u8>> {
    let session = Some(page.session.as_str());
    let tree = cdp.send("Page.getFrameTree", obj! {}.into(), session).await?;
    let mut frames = Vec::new();
    let mut pending = vec![tree.get("frameTree").clone()];
    while let Some(node) = pending.pop() {
        frames.extend(node.get("frame").get("id").as_str().map(str::to_owned));
        if let Value::Array(children) = node.get("childFrames") {
            pending.extend(children.iter().cloned());
        }
    }
    // Playwright's utility world; a frame that is gone or not ready is skipped, as its evaluation in all frames does.
    let hide = frames.iter().map(|frame| async move {
        let world = obj! { "frameId" => frame.as_str(), "worldName" => "__mymind_utility_world__", "grantUniveralAccess" => true };
        let Ok(world) = cdp.send("Page.createIsolatedWorld", world.into(), session).await else { return };
        let context = world.get("executionContextId").clone();
        let _ = cdp.send("Runtime.evaluate", obj! { "expression" => HIDE_CARET, "contextId" => context }.into(), session).await;
    });
    futures_util::future::join_all(hide).await;
    let _ = cdp.send("Runtime.evaluate", fonts, session).await;
    let metrics = cdp.send("Page.getLayoutMetrics", obj! {}.into(), session).await?;
    let viewport = metrics.get("visualViewport");
    let number = |key: &str| viewport.get(key).as_f64().unwrap_or(0.0);
    let scale = if viewport.get("scale").as_f64().is_some() { number("scale") } else { 1.0 };
    let clip = obj! {
        "x" => number("pageX"), "y" => number("pageY"),
        "width" => (1400.0 / scale + 1e-3).floor(), "height" => (940.0 / scale + 1e-3).floor(), "scale" => scale,
    };
    let shot = obj! { "format" => "png", "clip" => clip, "captureBeyondViewport" => false };
    let shot = cdp.send("Page.captureScreenshot", shot.into(), session).await?;
    Ok(base64::decode(shot.get("data").as_str().unwrap_or("")))
}

/// `page.goto(url, { waitUntil: 'domcontentloaded', timeout: 20000 })`.
async fn goto(cdp: &Cdp, page: &Page, url: &str) -> Result<()> {
    let deadline = || fail("page.goto: Timeout 20000ms exceeded.");
    let navigation = async {
        let reply = cdp.send("Page.navigate", obj! { "url" => url }.into(), Some(&page.session)).await?;
        if let Some(text) = reply.get("errorText").as_str() {
            return Err(fail(format!("page.goto: {text} at {url}")));
        }
        let frame = string_field(&reply, "frameId")?;
        let loader = reply.get("loaderId").as_str().unwrap_or("").to_string();
        dom_content_loaded(page, &frame, &loader).await;
        Ok(())
    };
    timeout(Duration::from_secs(20), navigation).await.map_err(|_| deadline())?
}

#[async_trait(?Send)]
impl Screenshot for ChromiumScreenshot {
    /// Render a public page in a fresh 1400 by 940 Chromium and screenshot it (`screenshotPage`).
    ///
    /// # Arguments
    /// * `url` - Absolute HTTP(S) page URL.
    /// * `offline` - Restrict every browser resource to loopback.
    ///
    /// # Returns
    /// The PNG bytes.
    ///
    /// # Errors
    /// A `CAPTURE_BLOCKED` error when the page or one of its resources was rejected, or the browser's error.
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        capture_destination(url, offline)?;
        let browser = Browser::launch(&self.executable).await?;
        let result = self.shoot(&browser.cdp, url, offline).await;
        browser.shut_down().await;
        result
    }
}
