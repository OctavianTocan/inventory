//! `mymind capture-fixtures`: port of `tools/capture-fixtures.mjs`, the offline HTTP server the capture and search
//! checks run against, as a module a test can start and steer.

use std::convert::Infallible;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use mm_core::ai_tags::tag_key;
use mm_core::{Error, Result};
use mm_js::{Object, Value, json, obj, url};
use regex::Regex;
use tokio::net::TcpListener;

/// The page cases of `cases`: name, canonical URL, expected card classes and expected markup.
pub const CASES: [[&str; 4]; 12] = [
    ["XPost", "https://x.com/fixture/status/123", "x-post post", "class=\"body x\""],
    ["ThreadsPost", "https://www.threads.net/@fixture/post/123", "threads-post post", "class=\"body threads\""],
    ["SubstackNote", "https://substack.com/@fixture/note/c-123", "substack-note post", "class=\"body substack\""],
    ["YouTubeVideo", "https://www.youtube.com/watch?v=fixture", "youtube-video video", "<mymind-image"],
    ["InstagramReel", "https://www.instagram.com/reel/fixture/", "instagram-reel", "class=\"lazy main-img\""],
    ["Repository", "https://github.com/fixture/repo", "repository", "class=\"inner github\""],
    ["RedditPost", "https://www.reddit.com/r/fixtures/comments/123/post/", "reddit-post", "class=\"reddit inner\""],
    ["SoftwareApplication", "https://apps.apple.com/us/app/fixture/id123", "software-application", "distributor_app-store"],
    ["SoftwareApplicationPlay", "https://play.google.com/store/apps/details?id=fixture", "software-application", "distributor_play-store"],
    ["Product", "https://shop.invalid/fixture", "product", "class=\"price\""],
    ["Article", "https://journal.invalid/fixture", "article", "class=\"article-card-top\""],
    ["WebPage", "https://site.invalid/fixture", "web-page", "<mymind-image"],
];

/// Synthetic media bytes (`fixtureMedia`'s result).
#[derive(Clone, Debug)]
pub struct Media {
    /// 640 by 480 PNG filled with `#cc4422`.
    pub png: Vec<u8>,
    /// The same picture as WebP.
    pub webp: Vec<u8>,
    /// A one-page PDF.
    pub pdf: Vec<u8>,
    /// A 0.3 second red H.264 clip.
    pub mp4: Vec<u8>,
}

/// A request the server answered, as the JS `requests` list holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureRequest {
    /// The URL path.
    pub path: String,
    /// The parsed JSON body, or `Undefined` for an empty one.
    pub body: Value,
    /// The HTTP method.
    pub method: String,
}

/// What an `embedding` hook answers.
#[derive(Clone, Debug)]
pub struct EmbeddingReply {
    /// The JSON body to send.
    pub body: Value,
    /// The status; 200 when `None`.
    pub status: Option<u16>,
}

/// A `control.embedding` function: the parsed request body to the reply.
pub type EmbeddingHook = Arc<dyn Fn(&Value) -> EmbeddingReply + Send + Sync>;

/// The knobs of `control`.
#[derive(Clone, Default)]
pub struct Control {
    /// Answer `/chat/completions` with status 400.
    pub ai_failure: bool,
    /// Answer every POST with a redirect to a host that must never be requested.
    pub ai_redirect: bool,
    /// Milliseconds `/cover.webp` waits before answering.
    pub image_delay: u64,
    /// Milliseconds `/chat/completions` waits before answering.
    pub chat_delay: u64,
    /// The tags `/chat/completions` returns; the default six when `None`.
    pub tags: Option<Value>,
    /// Answers `embedContent` requests instead of the default vector.
    pub embedding: Option<EmbeddingHook>,
}

#[derive(Default)]
struct State {
    requests: Vec<FixtureRequest>,
    control: Control,
}

struct Reply {
    status: u16,
    content_type: Option<String>,
    location: Option<String>,
    body: Vec<u8>,
}

/// A running fixture server; dropping it stops the server.
pub struct Fixtures {
    /// The server's origin, `http://127.0.0.1:<port>`.
    pub base: String,
    state: Arc<Mutex<State>>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

fn crc_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut crc = flate2::Crc::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc.sum().to_be_bytes());
}

// A solid 8-bit RGB PNG.
fn solid_png(width: u32, height: u32, rgb: [u8; 3]) -> Result<Vec<u8>> {
    let mut raw = Vec::with_capacity((width as usize * 3 + 1) * height as usize);
    for _ in 0..height {
        raw.push(0);
        raw.extend(rgb.iter().copied().cycle().take(width as usize * 3));
    }
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw).map_err(|error| Error::internal(error.to_string()))?;
    let packed = encoder.finish().map_err(|error| Error::internal(error.to_string()))?;
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    crc_chunk(&mut out, b"IHDR", &header);
    crc_chunk(&mut out, b"IDAT", &packed);
    crc_chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

fn pdf_bytes() -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> /Contents 4 0 R >>",
        "<< /Length 27 >>\nstream\n1 0 0 rg 10 10 100 100 re f\nendstream",
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{body}\nendobj\n", index + 1));
    }
    let xref = pdf.len();
    let entries: String = offsets.iter().map(|offset| format!("{offset:010} 00000 n \n")).collect();
    pdf.push_str(&format!("xref\n0 5\n0000000000 65535 f \n{entries}trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"));
    pdf.into_bytes()
}

/// Create synthetic media without copying any of the user's captured library (`fixtureMedia`).
///
/// # Arguments
/// * `directory` - Existing temporary directory; the clip is written there as `fixture.mp4`.
///
/// # Returns
/// The PNG, WebP, PDF and MP4 bytes.
///
/// # Errors
/// An internal error when libvips cannot encode the WebP or `ffmpeg` fails or takes over 10 seconds.
pub fn fixture_media(directory: &Path) -> Result<Media> {
    let png = solid_png(640, 480, [0xcc, 0x44, 0x22])?;
    let plan = crate::vips::Resize { width: 640, height: 480, cover: false, flatten: false, enlarge: false, format: crate::vips::Format::Webp };
    let webp = crate::vips::render(&png, &plan, 0).map_err(Error::internal)?;
    let video = directory.join("fixture.mp4");
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i", "color=c=red:s=320x240:d=0.3", "-an", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-y"])
        .arg(&video)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| Error::internal(format!("ffmpeg: {error}")))?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| Error::internal(error.to_string()))? {
            break status;
        }
        if started.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            return Err(Error::internal("ffmpeg took longer than 10 seconds"));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    if !status.success() {
        return Err(Error::internal(format!("ffmpeg failed: {status}")));
    }
    let mp4 = std::fs::read(&video).map_err(|error| Error::internal(error.to_string()))?;
    Ok(Media { png, webp, pdf: pdf_bytes(), mp4 })
}

fn text_reply(body: impl Into<Vec<u8>>, content_type: &str, status: u16) -> Reply {
    Reply { status, content_type: Some(content_type.to_owned()), location: None, body: body.into() }
}

fn json_reply(value: &Value, status: u16) -> Reply {
    text_reply(json::to_string(value), "application/json", status)
}

fn redirect(status: u16, location: &str) -> Reply {
    Reply { status, content_type: None, location: Some(location.to_owned()), body: Vec::new() }
}

fn unit_vector() -> Value {
    Value::Array((0..768).map(|index| Value::Number(if index == 0 { 1.0 } else { 0.0 })).collect())
}

fn fixture_page(name: &str, base: &str) -> Reply {
    let item = CASES.iter().find(|item| item[0] == name).or_else(|| (name == "Gallery").then_some(&CASES[0]));
    let canonical = item.map_or_else(|| format!("{base}/{name}"), |item| item[1].to_owned());
    let structured = match name {
        "Product" => Some(obj! {
            "@type" => "Product", "name" => "Fixture title", "description" => "Fixture description",
            "brand" => obj! { "name" => "Fixture brand" }, "offers" => obj! { "priceCurrency" => "USD", "price" => "12.50" }
        }),
        "JsonLd" => Some(obj! {
            "@type" => "Article", "headline" => "JSON-LD title", "description" => "JSON-LD description",
            "image" => format!("{base}/cover.png"), "author" => obj! { "name" => "Schema author" }
        }),
        _ => None,
    };
    let title = match name {
        "Repository" => "GitHub - fixture/repo: Fixture description",
        "SoftwareApplication" => "Fixture title on the App Store",
        "SoftwareApplicationPlay" => "Fixture title - Apps on Google Play",
        _ => "Fixture title",
    };
    let meta = if item.is_some() || name == "Twitter" {
        let kind = if name == "Twitter" { "twitter" } else { "og" };
        format!("<meta property=\"{kind}:title\" content=\"{title}\"><meta property=\"og:description\" content=\"Fixture description\"><meta property=\"og:site_name\" content=\"Fixture site\">")
    } else {
        String::new()
    };
    let article = if name == "Article" { "<meta property=\"og:type\" content=\"article\">" } else { "" };
    let cover = if ["NoImage", "BrokenImage", "JsonLd"].contains(&name) { "" } else { "<meta property=\"og:image\" content=\"/cover.webp\">" };
    let gallery = if name == "Gallery" { "<meta property=\"og:image\" content=\"/cover.png\">" } else { "" };
    let broken = if name == "BrokenImage" { "<meta property=\"og:image\" content=\"/blocked\">" } else { "" };
    let script = structured.map_or_else(String::new, |structured| {
        let graph = obj! { "@context" => "https://schema.org", "@graph" => Value::Array(vec![structured.into()]) };
        format!("<script type=\"application/ld+json\">{}</script>", json::to_string(&graph.into()))
    });
    let oembed = if item.is_some() { "<link rel=\"alternate\" type=\"application/json+oembed\" href=\"/embed\">" } else { "" };
    text_reply(
        format!(
            "<!doctype html><title>Fallback title</title><link rel=\"canonical\" href=\"{canonical}\">\n      {meta}\n      {article}\n      {cover}\n      {gallery}\n      {broken}\n      {script}\n      {oembed}<main>Fixture body text</main>"
        ),
        "text/html",
        200,
    )
}

fn questions_answer(body: &Value) -> std::result::Result<Reply, ()> {
    static TAG: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| Regex::new(r#"tag "([^"]+)""#).unwrap());
    let questions = body.get("questions");
    let entries: Vec<(String, &Value)> = match questions {
        Value::Object(object) => object.iter().map(|(key, question)| (key.into_owned(), question)).collect(),
        Value::Array(items) => items.iter().enumerate().map(|(index, question)| (index.to_string(), question)).collect(),
        _ => return Err(()),
    };
    let mut answers = Object::default();
    for (key, question) in entries {
        if question.is_nullish() {
            return Err(());
        }
        let instructions = question.get("instructions").to_js_string();
        let wanted = TAG.captures(&instructions).map(|found| tag_key(&found[1])).unwrap_or_default();
        let noul = if wanted == "design" || wanted == "image" { 0.95 } else { 0.1 };
        answers.insert(key, obj! { "type" => "noul", "noul" => noul });
    }
    Ok(json_reply(&obj! { "answers" => Value::Object(answers) }.into(), 200))
}

async fn answer(state: &Arc<Mutex<State>>, media: &Media, base: &str, method: &str, target: &str, text: &str) -> std::result::Result<Reply, ()> {
    let url = url::parse(target, Some(base)).ok_or(())?;
    let path = url.path().to_owned();
    let body = if text.is_empty() { Value::Undefined } else { json::parse(text).map_err(|_| ())? };
    let control = {
        let mut state = state.lock().unwrap();
        state.requests.push(FixtureRequest { path: path.clone(), body: body.clone(), method: method.to_owned() });
        state.control.clone()
    };
    if control.ai_redirect && method == "POST" {
        return Ok(redirect(307, "https://redirect.invalid/should-never-be-requested"));
    }
    match path.as_str() {
        "/cover.webp" => {
            if control.image_delay > 0 {
                tokio::time::sleep(Duration::from_millis(control.image_delay)).await;
            }
            return Ok(text_reply(media.webp.clone(), "image/webp", 200));
        }
        "/cover.png" | "/avatar.png" => return Ok(text_reply(media.png.clone(), "image/png", 200)),
        "/doc.pdf" => return Ok(text_reply(media.pdf.clone(), "application/pdf", 200)),
        "/failed.apk" => return Ok(text_reply(vec![0x50, 0x4b, 3, 4, 0, 0], "application/octet-stream", 200)),
        "/blocked" => return Ok(text_reply("blocked", "text/plain", 403)),
        "/redirect-account" => return Ok(redirect(302, "https://access.mymind.com/everything")),
        "/redirect-external" => return Ok(redirect(302, "https://external.invalid/image.png")),
        "/x-embed" => {
            return Ok(json_reply(
                &obj! {
                    "author_name" => "Jack Example",
                    "thumbnail_url" => format!("{base}/profile_images/avatar.png"),
                    "html" => "<blockquote class=\"twitter-tweet\"><p>just setting up my twttr &mdash; hello &amp; welcome</p>&mdash; jack (@jack) <a href=\"https://x.com/jack/status/20\">March 21, 2006</a></blockquote>"
                }
                .into(),
                200,
            ));
        }
        "/embed" => {
            return Ok(json_reply(
                &obj! {
                    "title" => "Fixture title", "author_name" => "Fixture author", "type" => "rich", "thumbnail_url" => format!("{base}/cover.png"),
                    "html" => "<blockquote><p>Fixture <strong>post</strong><br>body <a href=\"javascript:alert(1)\" onclick=\"bad()\">bad link</a><a href=\"https://example.invalid/safe\">safe</a></p></blockquote><script>alert(1)</script><img src=x onerror=bad()>"
                }
                .into(),
                200,
            ));
        }
        _ => {}
    }
    if path.contains("batchEmbedContents") {
        let Value::Array(requests) = body.get("requests") else { return Err(()) };
        let embeddings = requests.iter().map(|_| obj! { "values" => unit_vector() }.into()).collect();
        return Ok(json_reply(&obj! { "embeddings" => Value::Array(embeddings) }.into(), 200));
    }
    if path.contains("embedContent") {
        if let Some(hook) = &control.embedding {
            let reply = hook(&body);
            return Ok(json_reply(&reply.body, reply.status.unwrap_or(200)));
        }
        return Ok(json_reply(&obj! { "embedding" => obj! { "values" => unit_vector() } }.into(), 200));
    }
    if path == "/chat/completions" {
        if control.chat_delay > 0 {
            tokio::time::sleep(Duration::from_millis(control.chat_delay)).await;
        }
        if control.ai_failure {
            return Ok(text_reply("{}", "application/json", 400));
        }
        let default_tags = ["design", "website", "link", "page", "image", "screenshot"].map(Value::from).to_vec();
        let tags = control.tags.filter(|tags| !tags.is_nullish()).unwrap_or(Value::Array(default_tags));
        let content = obj! { "title" => "Upgrade Codex Memory Models for Free Now Today Extra", "summary" => "A fixture summary <safe>.", "tags" => tags };
        let message = obj! { "message" => obj! { "content" => json::to_string(&content.into()) } };
        return Ok(json_reply(&obj! { "choices" => Value::Array(vec![message.into()]) }.into(), 200));
    }
    if path == "/v1/systemone" {
        return questions_answer(&body);
    }
    let name = path.get(1..).unwrap_or("");
    if name == "XQuality" {
        return Ok(text_reply(
            "<title>Jack on X</title><link rel=\"canonical\" href=\"https://x.com/jack/status/20\"><meta property=\"og:image\" content=\"/profile_images/avatar.png\"><link rel=\"alternate\" type=\"application/json+oembed\" href=\"/x-embed\">",
            "text/html",
            200,
        ));
    }
    if name == "ArticleQuality" {
        let paragraph = "Mind maps connect concepts using branches and help organize knowledge. ".repeat(8);
        return Ok(text_reply(
            format!(
                "<title>Mind map - Wikipedia</title><meta property=\"og:site_name\" content=\"Wikipedia\"><meta name=\"description\" content=\"A mind map organizes ideas visually.\"><meta property=\"og:type\" content=\"article\"><meta property=\"og:image\" content=\"/cover.png\"><main><nav>Unrelated navigation</nav><p>{paragraph}</p><p>Use central topics and connections to plan a project.</p></main>"
            ),
            "text/html",
            200,
        ));
    }
    Ok(fixture_page(name, base))
}

async fn serve_one(state: Arc<Mutex<State>>, media: Arc<Media>, base: Arc<String>, request: hyper::Request<Incoming>) -> std::result::Result<hyper::Response<Full<Bytes>>, Infallible> {
    let method = request.method().as_str().to_owned();
    let target = request.uri().path_and_query().map_or("/", |target| target.as_str()).to_owned();
    let bytes = request.into_body().collect().await.map(|collected| collected.to_bytes()).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let reply = answer(&state, &media, &base, &method, &target, &text).await.unwrap_or(Reply { status: 500, content_type: None, location: None, body: Vec::new() });
    let mut response = hyper::Response::builder().status(reply.status);
    if let Some(kind) = &reply.content_type {
        response = response.header("content-type", kind);
    }
    if let Some(location) = &reply.location {
        response = response.header("location", location);
    }
    Ok(response.body(Full::new(Bytes::from(reply.body))).unwrap())
}

impl Fixtures {
    /// Start the server on 127.0.0.1 at a free port, on its own thread and runtime (`startFixtures` with a real listener).
    ///
    /// # Arguments
    /// * `media` - The bytes `/cover.webp`, `/cover.png`, `/avatar.png` and `/doc.pdf` answer with.
    ///
    /// # Returns
    /// The running server.
    ///
    /// # Errors
    /// An internal error when the listener or the runtime cannot start.
    pub fn start(media: Media) -> Result<Self> {
        Self::start_on(media, 0)
    }

    /// [`Fixtures::start`] on a chosen port; 0 picks a free one.
    ///
    /// # Arguments
    /// * `media` - As for [`Fixtures::start`].
    /// * `port` - The TCP port to listen on.
    ///
    /// # Errors
    /// As [`Fixtures::start`].
    pub fn start_on(media: Media, port: u16) -> Result<Self> {
        let state = Arc::new(Mutex::new(State::default()));
        let (ready, started) = std::sync::mpsc::channel::<std::result::Result<u16, String>>();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let thread_state = state.clone();
        let thread = std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(runtime) => runtime,
                Err(error) => return drop(ready.send(Err(error.to_string()))),
            };
            runtime.block_on(async move {
                let listener = match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await {
                    Ok(listener) => listener,
                    Err(error) => return drop(ready.send(Err(error.to_string()))),
                };
                let port = listener.local_addr().map(|address| address.port()).unwrap_or(0);
                let (base, media) = (Arc::new(format!("http://127.0.0.1:{port}")), Arc::new(media));
                let _ = ready.send(Ok(port));
                let mut stopped = stopped;
                loop {
                    let accepted = tokio::select! {
                        _ = &mut stopped => break,
                        accepted = listener.accept() => accepted,
                    };
                    let Ok((stream, _)) = accepted else { continue };
                    let (state, media, base) = (thread_state.clone(), media.clone(), base.clone());
                    tokio::spawn(async move {
                        let service = service_fn(move |request| serve_one(state.clone(), media.clone(), base.clone(), request));
                        let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
                    });
                }
            });
        });
        match started.recv() {
            Ok(Ok(port)) => Ok(Self { base: format!("http://127.0.0.1:{port}"), state, stop: Some(stop), thread: Some(thread) }),
            Ok(Err(message)) => Err(Error::internal(message)),
            Err(_) => Err(Error::internal("the fixture server thread ended before it listened")),
        }
    }

    /// The requests answered so far, oldest first (`requests`).
    pub fn requests(&self) -> Vec<FixtureRequest> {
        self.state.lock().unwrap().requests.clone()
    }

    /// Change the knobs (`control`).
    ///
    /// # Arguments
    /// * `change` - Edits the current knobs in place.
    pub fn control(&self, change: impl FnOnce(&mut Control)) {
        change(&mut self.state.lock().unwrap().control);
    }

    /// Stop the server and wait for its thread (`close`).
    pub fn close(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Run `mymind capture-fixtures`: make the media in a temporary directory, serve on `PORT` (or a free port), print
/// `capture fixtures on <base>` and serve until killed.
///
/// # Arguments
/// * `root` - Project root; unused, the fixtures need none.
/// * `args` - The arguments after the subcommand name; unused.
///
/// # Returns
/// The exit status: 1 after `Error: <message>` was printed to standard error when the server cannot start; it
/// does not return otherwise.
pub async fn run(_root: &Path, _args: &[String]) -> u8 {
    let started = (|| {
        let directory = std::env::temp_dir().join(format!("mymind-fixtures-{}", std::process::id()));
        std::fs::create_dir_all(&directory).map_err(|error| Error::internal(error.to_string()))?;
        let media = fixture_media(&directory);
        let _ = std::fs::remove_dir_all(&directory);
        let port = match std::env::var("PORT") {
            Ok(port) if !port.is_empty() => port.parse::<u16>().map_err(|_| Error::internal(format!("Invalid PORT {port}")))?,
            _ => 0,
        };
        Fixtures::start_on(media?, port)
    })();
    match started {
        Ok(fixtures) => {
            println!("capture fixtures on {}", fixtures.base);
            let _ = std::io::stdout().flush();
            std::future::pending::<()>().await;
            0
        }
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}
