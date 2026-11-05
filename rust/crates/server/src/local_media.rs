//! Port of `lib/local-media.mjs`: originals, previews and renditions on disk, made with libvips, ffmpeg, ffprobe and
//! pdftoppm.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::rc::Rc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::FutureExt;
use futures_util::future::{LocalBoxFuture, Shared};
use mm_core::host::{AnalysisImage, Media, MediaInput, MediaResult, MediaUser, PreparedMedia};
use mm_core::http::{Body, Headers, Response};
use mm_core::media_files::{Rendition, audio_format, byte_range, cover, mime, parse_rendition};
use mm_core::runtime::random_bytes;
use mm_core::{Error, Result};
use mm_js::{Object, Value, base64, json, text};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use tokio::process::Command;

use crate::bucket_fs::file_stream;
use crate::node_path::{extname, resolve};
use crate::vips::{self, Format, Resize};

const MAX_PIXELS: u64 = 80_000_000;
const MAX_OUTPUT: usize = 1024 * 1024;

type Pending = Shared<LocalBoxFuture<'static, Result<Option<MediaResult>>>>;

/// Downscale an image to the PNG the AI analysis reads (`analysisImage`).
///
/// # Arguments
/// * `data` - The image's bytes.
///
/// # Returns
/// A PNG at most 1400 pixels on each side, upright and never enlarged.
///
/// # Errors
/// An internal error when the image cannot be decoded.
pub async fn analysis_image(data: Vec<u8>) -> Result<AnalysisImage> {
    let plan = Resize { width: 1400, height: 1400, cover: false, flatten: false, enlarge: false, format: Format::Png };
    let data = blocking(move || vips::render(&data, &plan, MAX_PIXELS)).await?;
    Ok(AnalysisImage { mime: "image/png".to_owned(), data })
}

/// A library's media on disk below one root (`createMedia`).
/// Pull the first sound track of a video file out as mono 16 kHz Ogg Opus at 24 kbit/s, small enough to send for
/// transcription (about 11 MB an hour).
///
/// # Arguments
/// * `file` - Path of the video.
///
/// # Returns
/// The audio, or `None` when the video has no sound track.
///
/// # Errors
/// An internal error when ffprobe or ffmpeg fails or runs out of time.
pub async fn extract_audio(file: &str) -> Result<Option<Vec<u8>>> {
    let streams = run("ffprobe", &args(&["-v", "error", "-select_streams", "a", "-show_entries", "stream=index", "-of", "csv=p=0", file]), 30000).await?;
    if streams.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }
    let target = format!("{file}.{}.ogg", token());
    let arguments = args(&["-v", "error", "-y", "-i", file, "-map", "0:a:0", "-vn", "-ac", "1", "-ar", "16000", "-c:a", "libopus", "-b:a", "24k", &target]);
    let made = async {
        run("ffmpeg", &arguments, 600_000).await?;
        tokio::fs::read(&target).await.map_err(io)
    }
    .await;
    let _ = tokio::fs::remove_file(&target).await;
    made.map(Some)
}

pub struct LocalMedia {
    inner: Rc<Inner>,
}

struct Inner {
    root: PathBuf,
    pending: RefCell<HashMap<String, Pending>>,
}

impl LocalMedia {
    /// Media stored below `root`.
    ///
    /// # Arguments
    /// * `root` - Absolute media directory; originals sit at `<account>/<card>/<blob>.<ext>` below it and renditions
    ///   in `.renditions`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = resolve(Path::new("/"), &root.into().to_string_lossy());
        Self { inner: Rc::new(Inner { root, pending: RefCell::new(HashMap::new()) }) }
    }

    /// The media directory.
    pub fn root(&self) -> &Path {
        &self.inner.root
    }
}

fn io(error: std::io::Error) -> Error {
    Error::internal(error.to_string())
}

fn token() -> String {
    base64::hex(&random_bytes(6))
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> vips::Outcome<T> + Send + 'static) -> Result<T> {
    vips::init();
    match tokio::task::spawn_blocking(work).await {
        Ok(done) => done.map_err(Error::internal),
        Err(error) => Err(Error::internal(error.to_string())),
    }
}

async fn is_file(path: &str) -> bool {
    tokio::fs::metadata(path).await.is_ok_and(|info| info.is_file())
}

async fn drain(pipe: Option<impl AsyncRead + Unpin>) -> (Vec<u8>, bool) {
    let Some(mut pipe) = pipe else { return (Vec::new(), false) };
    let (mut kept, mut over, mut chunk) = (Vec::new(), false, [0u8; 8192]);
    while let Ok(read) = pipe.read(&mut chunk).await {
        if read == 0 {
            break;
        }
        if kept.len() + read > MAX_OUTPUT {
            over = true;
        } else {
            kept.extend_from_slice(&chunk[..read]);
        }
    }
    (kept, over)
}

async fn run(program: &str, args: &[String], timeout: u64) -> Result<Vec<u8>> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(io)?;
    let (stdout, stderr) = (child.stdout.take(), child.stderr.take());
    let finished = tokio::time::timeout(Duration::from_millis(timeout), async {
        let ((out, out_over), (_, err_over), status) = tokio::join!(drain(stdout), drain(stderr), child.wait());
        (out, out_over || err_over, status)
    })
    .await;
    match finished {
        Ok((out, over, Ok(status))) if status.success() && !over => Ok(out),
        Ok(_) => Err(Error::internal(format!("{program} failed"))),
        Err(_) => {
            let _ = child.kill().await;
            Err(Error::internal(format!("{program} timed out")))
        }
    }
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

impl Inner {
    /// Resolve only descendants of the media directory.
    fn inside(&self, relative: &str) -> Option<String> {
        let file = resolve(&self.root, relative).to_string_lossy().into_owned();
        let prefix = format!("{}/", self.root.to_string_lossy().trim_end_matches('/'));
        file.starts_with(&prefix).then_some(file)
    }

    async fn image_info(&self, data: Vec<u8>) -> Result<(i32, i32, String)> {
        blocking(move || {
            let info = vips::metadata(&data, MAX_PIXELS)?;
            let color = vips::dominant(&data, MAX_PIXELS)?;
            let swapped = (5..=8).contains(&info.orientation);
            Ok(if swapped { (info.height, info.width, color) } else { (info.width, info.height, color) })
        })
        .await
    }

    async fn preview(&self, file: &str, content_type: &str, page: u32) -> Result<String> {
        let target = format!("{file}.page-{page}.png");
        if is_file(&target).await {
            return Ok(target);
        }
        let temp = format!("{target}.{}.png", token());
        let made = async {
            if content_type == "application/pdf" {
                let page = page.to_string();
                let stem = &temp[..temp.len() - 4];
                run("pdftoppm", &args(&["-f", &page, "-l", &page, "-singlefile", "-scale-to", "1600", "-png", file, stem]), 30000).await?;
            } else {
                let arguments = args(&["-v", "error", "-y", "-i", file, "-frames:v", "1", "-vf", "scale=1600:1600:force_original_aspect_ratio=decrease", &temp]);
                run("ffmpeg", &arguments, 30000).await?;
            }
            tokio::fs::rename(&temp, &target).await.map_err(io)
        }
        .await;
        let _ = tokio::fs::remove_file(&temp).await;
        made.map(|()| target)
    }

    async fn prepare(&self, input: MediaInput, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        let (bytes, source) = match input {
            MediaInput::Bytes(bytes) => (Some(bytes), None),
            MediaInput::File(path) => (None, Some(path)),
            _ => return Err(Error::internal("unsupported media input")),
        };
        let (header, length) = match (&bytes, &source) {
            (Some(bytes), _) => (bytes[..bytes.len().min(16)].to_vec(), bytes.len() as u64),
            (None, Some(path)) => {
                let mut handle = tokio::fs::File::open(path).await.map_err(io)?;
                let length = handle.metadata().await.map_err(io)?.len();
                let mut header = vec![0; 16];
                let mut filled = 0;
                while filled < 16 {
                    let read = handle.read(&mut header[filled..]).await.map_err(io)?;
                    if read == 0 {
                        break;
                    }
                    filled += read;
                }
                header.truncate(filled);
                (header, length)
            }
            _ => unreachable!("one input is set"),
        };
        let unsupported = || Error::status(415, "Unsupported or invalid media");
        let mut content_type: String;
        let ftyp = header.get(4..8) == Some(b"ftyp");
        let (mut kind, mut extension) = if header.starts_with(b"%PDF-") {
            content_type = "application/pdf".to_owned();
            ("Document", "pdf".to_owned())
        } else if let Some(format) = audio_format(&header) {
            content_type = mime(format).unwrap_or_default().to_owned();
            ("Audio", format.to_owned())
        } else if ftyp || header.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
            let extension = if ftyp { "mp4" } else { "webm" };
            content_type = mime(extension).unwrap_or_default().to_owned();
            ("Video", extension.to_owned())
        } else {
            let data = match (&bytes, &source) {
                (Some(bytes), _) => bytes.clone(),
                (None, Some(path)) => tokio::fs::read(path).await.map_err(|_| unsupported())?,
                _ => unreachable!("one input is set"),
            };
            let info = blocking(move || vips::metadata(&data, MAX_PIXELS)).await.map_err(|_| unsupported())?;
            let extension = if info.format == "jpeg" { "jpg" } else { info.format };
            if mime(extension).is_none() || extension == "svg" {
                return Err(Error::status(415, "Unsupported image format"));
            }
            content_type = mime(extension).unwrap_or_default().to_owned();
            ("Image", extension.to_owned())
        };
        let blob_id = base64::hex(&random_bytes(11));
        let card = text::slice(id, text::len(&user.id_prefix) as i64, None);
        let mut relative = format!("{}/{}/{blob_id}.{extension}", user.account_uid, card);
        let mut file = self.inside(&relative).ok_or_else(|| Error::internal("media path outside the root"))?;
        tokio::fs::create_dir_all(Path::new(&file).parent().unwrap_or(&self.root)).await.map_err(io)?;
        match (&bytes, &source) {
            (Some(bytes), _) => tokio::fs::write(&file, bytes).await.map_err(io)?,
            (None, Some(path)) => {
                tokio::fs::copy(path, &file).await.map_err(io)?;
            }
            _ => unreachable!("one input is set"),
        }
        let probed = if matches!(kind, "Video" | "Audio") {
            let entries = if kind == "Video" { "stream=width,height:format=duration" } else { "format=duration" };
            let output = run("ffprobe", &args(&["-v", "error", "-select_streams", "v:0", "-show_entries", entries, "-of", "json", &file]), 15000).await?;
            json::parse(&String::from_utf8_lossy(&output)).map_err(|error| Error::internal(error.message().to_owned()))?
        } else {
            Value::Undefined
        };
        // An MP4 or WebM file with no picture is a recording, such as a phone's .m4a voice memo.
        if kind == "Video" && probed.get("streams").at(0).as_object().is_none() {
            kind = "Audio";
            extension = if ftyp { "m4a" } else { "weba" }.to_owned();
            content_type = mime(&extension).unwrap_or_default().to_owned();
            let renamed = format!("{}/{}/{blob_id}.{extension}", user.account_uid, card);
            let target = self.inside(&renamed).ok_or_else(|| Error::internal("media path outside the root"))?;
            tokio::fs::rename(&file, &target).await.map_err(io)?;
            (relative, file) = (renamed, target);
        }
        let mut blob = Object::new();
        blob.insert("id", blob_id);
        blob.insert("path", relative);
        blob.insert("type", content_type.clone());
        blob.insert("size", length as f64);
        blob.insert("name", text::slice(name, 0, Some(255)));
        if kind == "Image" {
            let (width, height, color) = self.image_info(tokio::fs::read(&file).await.map_err(io)?).await?;
            blob.insert("width", width);
            blob.insert("height", height);
            blob.insert("dominantColor", color);
        } else if kind == "Audio" {
            let duration = probed.get("format").get("duration").to_number();
            blob.insert("duration", if duration.is_nan() { 0.0 } else { duration });
        } else {
            if kind == "Video" {
                if let Some(stream) = probed.get("streams").at(0).as_object() {
                    for (key, value) in stream.iter() {
                        blob.insert(key.into_owned(), value.clone());
                    }
                }
                let duration = probed.get("format").get("duration").to_number();
                blob.insert("duration", if duration.is_nan() { 0.0 } else { duration });
            }
            let page = self.preview(&file, &content_type, 1).await?;
            let (width, height, color) = self.image_info(tokio::fs::read(&page).await.map_err(io)?).await?;
            if kind == "Document" {
                blob.insert("width", width);
                blob.insert("height", height);
            }
            blob.insert("dominantColor", color);
        }
        let usable = |key: &str| blob.get(key).is_some_and(|value| value.to_number() > 0.0);
        if kind != "Audio" && !(usable("width") && usable("height")) {
            return Err(Error::status(415, "Media has no usable dimensions"));
        }
        let blob: Value = blob.into();
        let mut data = Object::new();
        data.insert("type", kind);
        data.insert("title", blob.get("name").clone());
        data.insert("blob", blob.clone());
        if kind == "Document" {
            data.insert("format", "PDF");
        }
        let mut extras = Object::new();
        if kind != "Audio" {
            extras.insert("cover", cover(&blob, 800.0, 1000.0)?);
        }
        Ok(PreparedMedia { data: data.into(), extras: extras.into() })
    }

    async fn original(&self, relative: &str) -> Result<Option<(String, u64)>> {
        let Some(exact) = self.inside(relative) else { return Ok(None) };
        if let Ok(info) = tokio::fs::metadata(&exact).await
            && info.is_file()
        {
            return Ok(Some((exact, info.len())));
        }
        let directory = Path::new(&exact).parent().unwrap_or(&self.root).to_path_buf();
        let Ok(mut entries) = tokio::fs::read_dir(&directory).await else { return Ok(None) };
        let mut names = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        let stem = format!("{}.", relative.rsplit('/').next().unwrap_or(""));
        let Some(name) = names.into_iter().find(|name| name.starts_with(&stem) && !name.contains(".page-")) else { return Ok(None) };
        let file = directory.join(name).to_string_lossy().into_owned();
        let info = tokio::fs::metadata(&file).await.map_err(io)?;
        Ok(info.is_file().then_some((file, info.len())))
    }

    async fn rendition(&self, relative: &str) -> Result<Option<MediaResult>> {
        let Some((file, size)) = self.original(relative.split(';').next().unwrap_or("")).await? else { return Ok(None) };
        let wanted = parse_rendition(relative)?;
        let content_type = mime(extname(&file).trim_start_matches('.')).unwrap_or("application/octet-stream");
        let Rendition::Derived { width, height, mode, format, page, flatten, content_type: output_type, .. } = wanted else {
            return Ok(Some(MediaResult { key: file, size, content_type: content_type.to_owned() }));
        };
        let cache = self
            .inside(&format!(".renditions/{}", relative.replace('/', "_")))
            .ok_or_else(|| Error::internal("rendition path outside the root"))?;
        if let Ok(info) = tokio::fs::metadata(&cache).await
            && info.is_file()
        {
            return Ok(Some(MediaResult { key: cache, size: info.len(), content_type: output_type }));
        }
        tokio::fs::create_dir_all(Path::new(&cache).parent().unwrap_or(&self.root)).await.map_err(io)?;
        let temp = format!("{cache}.{}.tmp", token());
        let made = async {
            if output_type == "video/mp4" {
                let scale = format!("scale={width}:{height}:force_original_aspect_ratio=decrease");
                let arguments = args(&[
                    "-v", "error", "-y", "-i", &file, "-vf", &scale, "-an", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
                    "-f", "mp4", &temp,
                ]);
                run("ffmpeg", &arguments, 60000).await?;
            } else {
                let input = if content_type == "application/pdf" || content_type.starts_with("video/") {
                    self.preview(&file, content_type, page).await?
                } else {
                    file.clone()
                };
                let data = tokio::fs::read(&input).await.map_err(io)?;
                let format = match format.as_str() {
                    "webp" => Format::Webp,
                    "png" => Format::Png,
                    _ => Format::Jpeg,
                };
                let plan = Resize { width: width as i32, height: height as i32, cover: mode == Some('c'), flatten, enlarge: true, format };
                let encoded = blocking(move || vips::render(&data, &plan, MAX_PIXELS)).await?;
                tokio::fs::write(&temp, encoded).await.map_err(io)?;
            }
            tokio::fs::rename(&temp, &cache).await.map_err(io)?;
            Ok::<u64, Error>(tokio::fs::metadata(&cache).await.map_err(io)?.len())
        }
        .await;
        let _ = tokio::fs::remove_file(&temp).await;
        Ok(Some(MediaResult { key: cache, size: made?, content_type: output_type }))
    }
}

#[async_trait(?Send)]
impl Media for LocalMedia {
    /// Fit a stored original's intrinsic dimensions to the 800 by 1000 cover box (`cover`).
    ///
    /// # Arguments
    /// * `blob` - Stored original's metadata.
    ///
    /// # Errors
    /// `blob` is null or undefined.
    fn cover(&self, blob: &Value) -> Result<Value> {
        cover(blob, 800.0, 1000.0)
    }

    /// Identify a new file, store the original below the root and return the card media fields (`prepare`).
    ///
    /// # Arguments
    /// * `input` - The bytes or a file on disk.
    /// * `content_type` - The type the client claimed.
    /// * `name` - Original file name.
    /// * `user` - Account the media is stored for.
    /// * `id` - Id of the card being made.
    ///
    /// # Errors
    /// Status 415 for unsupported media and 422 `Media preview could not be generated` for any other failure.
    async fn prepare(&self, input: MediaInput, _content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        self.inner.prepare(input, name, user, id).await.map_err(|error| match error {
            Error::Status { .. } => error,
            _ => Error::status(422, "Media preview could not be generated"),
        })
    }

    /// The response for a found original or rendition, or the span of it a `Range` header asks for (`send`).
    ///
    /// # Arguments
    /// * `result` - What [`Media::response`] found; `key` is the file.
    /// * `method` - Request method; `HEAD` sends no body.
    /// * `range` - The request's `Range` header.
    ///
    /// # Errors
    /// The file's I/O error.
    async fn send(&self, result: &MediaResult, method: &str, range: Option<&str>) -> Result<Response> {
        let mut headers = Headers::new();
        headers.set("cache-control", "no-store");
        headers.set("content-type", result.content_type.as_str());
        headers.set("accept-ranges", "bytes");
        let span = byte_range(range, result.size as f64);
        if span.status == 416 {
            headers.set("content-range", format!("bytes */{}", result.size));
            headers.set("content-length", "0");
            return Ok(Response::new(416, headers, Body::Empty));
        }
        let (start, end) = (span.start as i64, span.end as i64);
        if span.status == 206 {
            headers.set("content-range", format!("bytes {start}-{end}/{}", result.size));
        }
        let length = (end - start + 1).max(0) as u64;
        headers.set("content-length", length.to_string());
        if method == "HEAD" || result.size == 0 {
            return Ok(Response::new(span.status, headers, Body::Empty));
        }
        let mut file = tokio::fs::File::open(&result.key).await.map_err(io)?;
        file.seek(SeekFrom::Start(start as u64)).await.map_err(io)?;
        Ok(Response::new(span.status, headers, Body::Stream(file_stream(file, length))))
    }

    /// The original or rendition a media path names, making the rendition once however many callers ask (`response`).
    ///
    /// # Arguments
    /// * `relative` - Path below `/media/`, decoded, with an optional `;` rendition suffix.
    ///
    /// # Returns
    /// `None` when there is no such original or no such rendition.
    ///
    /// # Errors
    /// Status 400 for a malformed suffix, and 422 `Media rendition unavailable` for a failed rendition.
    async fn response(&self, relative: &str) -> Result<Option<MediaResult>> {
        let existing = self.inner.pending.borrow().get(relative).cloned();
        let shared = match existing {
            Some(shared) => shared,
            None => {
                let inner = Rc::clone(&self.inner);
                let key = relative.to_owned();
                let job: LocalBoxFuture<'static, Result<Option<MediaResult>>> = Box::pin(async move {
                    let made = inner.rendition(&key).await.map_err(|error| match error {
                        Error::Status { .. } => error,
                        _ => Error::status(422, "Media rendition unavailable"),
                    });
                    inner.pending.borrow_mut().remove(&key);
                    made
                });
                let shared = job.shared();
                self.inner.pending.borrow_mut().insert(relative.to_owned(), shared.clone());
                shared
            }
        };
        shared.await
    }

    /// The analysis image of a card's blob or cover (`image`).
    ///
    /// # Arguments
    /// * `blob_or_cover` - A stored `blob` (`path`) or a `cover` (`src`); a falsy value has no image.
    ///
    /// # Returns
    /// `None` when there is no such media.
    ///
    /// # Errors
    /// As [`Media::response`], or an internal error when the image cannot be decoded.
    /// The sound of a stored video (`audio`).
    ///
    /// # Arguments
    /// * `blob` - The card's stored `blob` (`path`); a falsy value has no audio.
    ///
    /// # Returns
    /// `None` when there is no such original or it has no sound.
    ///
    /// # Errors
    /// As [`extract_audio`].
    async fn audio(&self, blob: &Value) -> Result<Option<Vec<u8>>> {
        let path = blob.get("path");
        if !blob.truthy() || !path.truthy() {
            return Ok(None);
        }
        let Some(result) = self.response(&path.to_js_string()).await? else { return Ok(None) };
        extract_audio(&result.key).await
    }

    async fn image(&self, blob_or_cover: &Value) -> Result<Option<AnalysisImage>> {
        if !blob_or_cover.truthy() {
            return Ok(None);
        }
        let src = blob_or_cover.get("src");
        let relative = if src.is_nullish() {
            format!("{};800x1000-t.webp", blob_or_cover.get("path").to_js_string())
        } else {
            let src = src.to_js_string();
            src.strip_prefix("/media/").map_or(src.clone(), str::to_owned)
        };
        let Some(result) = self.response(&relative).await? else { return Ok(None) };
        let data = tokio::fs::read(&result.key).await.map_err(io)?;
        analysis_image(data).await.map(Some)
    }
}
