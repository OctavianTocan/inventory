//! Outbound HTTP for the native server, as Node's global `fetch` sends it: redirects followed (or not, for
//! [`Fetch::fetch_unfollowed`]), compressed bodies decoded, and failures named by their system error code.

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Once;

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::{Error, Result};
use tokio::io::AsyncReadExt;

const FILE_CHUNK: usize = 64 * 1024;

#[derive(Clone, PartialEq, Eq)]
struct FileStamp {
    length: u64,
    modified_nsec: i64,
}

#[derive(Clone)]
struct FileSource {
    path: PathBuf,
    stamp: FileStamp,
    #[cfg(test)]
    read: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
}

// Node 24.12.0 synthetic controls show size and mtime_nsec comparison, ignoring seconds, inode and ctime.
fn file_stamp(metadata: &std::fs::Metadata) -> FileStamp {
    FileStamp {
        length: metadata.len(),
        modified_nsec: metadata.mtime_nsec(),
    }
}

// A changed size or fractional mtime cannot supply the original Blob's data.
fn changed_file() -> std::io::Error {
    std::io::Error::other("The blob could not be read")
}

impl FileSource {
    // Record the Blob's size and fractional mtime without opening its per-stream read descriptor yet.
    async fn snapshot(path: &Path) -> std::io::Result<Self> {
        Ok(Self { path: path.to_owned(), stamp: file_stamp(&tokio::fs::metadata(path).await?), #[cfg(test)] read: None })
    }

    // Once a stream opens the descriptor, Node checks that descriptor even after rename or unlink.
    async fn check(&self, file: &tokio::fs::File) -> std::io::Result<()> {
        let descriptor = file.metadata().await.map_err(|_| changed_file())?;
        if file_stamp(&descriptor) != self.stamp {
            return Err(changed_file());
        }
        Ok(())
    }

    // Each new Blob stream reopens the pathname and checks its original size and fractional mtime.
    async fn open(&self) -> std::io::Result<tokio::fs::File> {
        let file = tokio::fs::File::open(&self.path).await.map_err(|_| changed_file())?;
        self.check(&file).await?;
        Ok(file)
    }
}

/// Node's `fetch` over reqwest.
pub struct NativeFetch {
    following: reqwest::Client,
    unfollowed: reqwest::Client,
}

fn client(redirect: reqwest::redirect::Policy) -> reqwest::Client {
    reqwest::Client::builder().redirect(redirect).build().expect("the HTTP client starts with its TLS roots")
}

// The `cause.code` Node's fetch rejects with, which the AI errors show; `TypeError` when there is none.
fn failure(error: &reqwest::Error) -> Error {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(cause) = source {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            use std::io::ErrorKind::{ConnectionAborted, ConnectionRefused, ConnectionReset, TimedOut};
            let code = match io.kind() {
                ConnectionRefused => "ECONNREFUSED",
                ConnectionReset | ConnectionAborted => "ECONNRESET",
                TimedOut => "ETIMEDOUT",
                _ => "TypeError",
            };
            return Error::internal(code);
        }
        if cause.to_string().contains("dns error") {
            return Error::internal("ENOTFOUND");
        }
        source = cause.source();
    }
    Error::internal(if error.is_timeout() { "ETIMEDOUT" } else { "TypeError" })
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

// A disk body stays bounded while reqwest waits for the receiving socket to accept its next chunk.
fn file_chunks(source: FileSource) -> impl futures_util::Stream<Item = std::io::Result<Vec<u8>>> + Send {
    futures_util::stream::try_unfold((None, source.stamp.length, source), |(file, left, source)| async move {
        if left == 0 {
            return Ok(None);
        }
        let mut file = match file {
            Some(file) => {
                source.check(&file).await?;
                file
            }
            None => source.open().await?,
        };
        let mut chunk = vec![0; FILE_CHUNK.min(usize::try_from(left).unwrap_or(FILE_CHUNK))];
        let read = file.read(&mut chunk).await?;
        #[cfg(test)]
        if let Some(count) = &source.read {
            count.fetch_add(read as u64, std::sync::atomic::Ordering::SeqCst);
        }
        if read == 0 {
            return Err(changed_file());
        }
        chunk.truncate(read);
        Ok(Some((chunk, (Some(file), left - read as u64, source))))
    })
}

// Preserve the same response decoding for buffered requests and file uploads.
fn response(response: reqwest::Response) -> Response {
    let mut headers: Vec<(String, String)> = Vec::new();
    for (name, value) in response.headers() {
        let value = latin1(value.as_bytes());
        match headers.iter_mut().find(|(seen, _)| seen == name.as_str()) {
            Some((_, existing)) => {
                existing.push_str(", ");
                existing.push_str(&value);
            }
            None => headers.push((name.as_str().to_owned(), value)),
        }
    }
    let status = response.status().as_u16();
    let stream = response.bytes_stream().map(|chunk| chunk.map(|bytes| bytes.to_vec()).map_err(|error| failure(&error)));
    Response::new(status, Headers::from_pairs(headers), Body::Stream(Box::pin(stream)))
}

// The original method, URL and headers, without consuming a request's body.
fn outgoing(client: &reqwest::Client, request: &Request) -> Result<reqwest::RequestBuilder> {
    let method = reqwest::Method::from_bytes(request.method.as_bytes()).map_err(|_| Error::invalid(format!("'{}' is not a valid HTTP method.", request.method)))?;
    let mut outgoing = client.request(method, request.url.as_str());
    for (name, value) in request.headers.iter() {
        outgoing = outgoing.header(name, value);
    }
    Ok(outgoing)
}

impl NativeFetch {
    /// A client with the system's trusted roots.
    ///
    /// # Panics
    /// When TLS cannot be set up.
    pub fn new() -> Self {
        static PROVIDER: Once = Once::new();
        PROVIDER.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
        Self { following: client(reqwest::redirect::Policy::limited(20)), unfollowed: client(reqwest::redirect::Policy::none()) }
    }

    /// Send a file as the request body without accumulating it in memory (`openAsBlob`).
    ///
    /// # Arguments
    /// * `request` - Method, URL and headers; its body is replaced by the file and its content length.
    /// * `path` - Backing file read lazily in chunks of at most 64 KiB, checking its initial size and mtime nanosecond component as Node 24.12.0 does.
    ///
    /// # Errors
    /// The file cannot be opened or inspected, its size or fractional mtime changes during reading or replay, or the HTTP request fails.
    pub async fn fetch_file(&self, request: Request, path: &Path) -> Result<Response> {
        let source = FileSource::snapshot(path).await.map_err(|error| Error::internal(error.to_string()))?;
        self.fetch_source(request, &source).await
    }

    // Replay the file Blob against its original size and fractional mtime, with backpressure on every hop.
    async fn fetch_source(&self, request: Request, source: &FileSource) -> Result<Response> {
        let mut request = request.head();
        let (mut file_body, mut redirects) = (true, 0);
        loop {
            let mut outgoing = outgoing(&self.unfollowed, &request)?;
            if file_body {
                outgoing = outgoing.header(reqwest::header::CONTENT_LENGTH, source.stamp.length).body(reqwest::Body::wrap_stream(file_chunks(source.clone())));
            }
            let received = outgoing.send().await.map_err(|error| failure(&error))?;
            let status = received.status().as_u16();
            let next = received.headers().get(reqwest::header::LOCATION).and_then(|location| location.to_str().ok()).and_then(|location| mm_js::url::parse(location, Some(request.url.as_str())));
            let Some(next) = next.filter(|_| matches!(status, 301 | 302 | 303 | 307 | 308)) else { return Ok(response(received)); };
            if redirects == 20 || !matches!(next.scheme(), "http" | "https") {
                return Err(Error::internal("TypeError"));
            }
            redirects += 1;
            // A streamed reqwest body cannot be cloned for redirects; reopen the file at each retained-body hop.
            if ((status == 301 || status == 302) && request.method == "POST") || status == 303 {
                if request.method != "HEAD" {
                    request.method = "GET".to_owned();
                }
                file_body = false;
                for header in ["content-type", "content-length", "content-encoding", "transfer-encoding"] {
                    request.headers.remove(header);
                }
            }
            if next.origin() != request.url.origin() {
                for header in ["authorization", "cookie", "cookie2", "proxy-authorization", "www-authenticate"] {
                    request.headers.remove(header);
                }
            }
            if next.scheme() != "http" || request.url.scheme() != "https" {
                let mut referer = request.url.clone();
                let _ = referer.set_username("");
                let _ = referer.set_password(None);
                referer.set_fragment(None);
                request.headers.set("referer", referer.as_str());
            }
            request.url = next;
        }
    }

    async fn send(&self, client: &reqwest::Client, request: Request) -> Result<Response> {
        let mut outgoing = outgoing(client, &request)?;
        let body = request.body.bytes().await?;
        if !body.is_empty() {
            outgoing = outgoing.body(body);
        }
        Ok(response(outgoing.send().await.map_err(|error| failure(&error))?))
    }
}

impl Default for NativeFetch {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl Fetch for NativeFetch {
    async fn fetch(&self, request: Request) -> Result<Response> {
        self.send(&self.following, request).await
    }

    async fn fetch_unfollowed(&self, request: Request) -> Result<Response> {
        self.send(&self.unfollowed, request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, Write};
    use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
    use tokio::io::AsyncWriteExt;

    // Isolate each synthetic upload file on disk below the Cargo target directory.
    fn scratch(label: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
        std::fs::create_dir_all(&root).unwrap();
        root.join(format!("fetch-{label}-{}", std::process::id()))
    }

    #[tokio::test]
    async fn file_chunks_are_lazy_and_bounded() {
        let path = scratch("chunks");
        let mut output = std::fs::File::create(&path).unwrap();
        let length = FILE_CHUNK as u64 * 3 + 17;
        output.set_len(length).unwrap();
        output.write_all(b"immutable backing file").unwrap();
        let read = Arc::new(AtomicU64::new(0));
        let mut source = FileSource::snapshot(&path).await.unwrap();
        source.read = Some(read.clone());
        let stream = file_chunks(source);
        futures_util::pin_mut!(stream);
        assert_eq!(read.load(Ordering::SeqCst), 0);
        let first = stream.next().await.unwrap().unwrap();
        assert!(first.starts_with(b"immutable backing file"));
        assert_eq!(first.len(), FILE_CHUNK);
        assert_eq!(read.load(Ordering::SeqCst), FILE_CHUNK as u64);
        let mut total = first.len();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            assert!(!chunk.is_empty() && chunk.len() <= FILE_CHUNK);
            total += chunk.len();
        }
        assert_eq!(total as u64, length);
        assert_eq!(read.load(Ordering::SeqCst), length);
        drop(output);
        std::fs::remove_file(path).unwrap();
    }

    // Reproduce the Node Blob controls, preserving nanosecond mtime only when the case requests it.
    fn alter_file(path: &Path, change: &str) {
        match change {
            "rewrite" | "rewrite-preserve-mtime" | "rewrite-whole-second" => {
                let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
                let modified = file.metadata().unwrap().modified().unwrap();
                let offset = (FILE_CHUNK as u64).min(file.metadata().unwrap().len() - 1);
                file.seek(std::io::SeekFrom::Start(offset)).unwrap();
                file.write_all(b"x").unwrap();
                let modified = if change.ends_with("preserve-mtime") { modified } else if change.ends_with("whole-second") { modified + std::time::Duration::from_secs(1) } else { modified + std::time::Duration::from_nanos(1) };
                file.set_times(std::fs::FileTimes::new().set_modified(modified)).unwrap();
            }
            "resize" => {
                let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
                file.set_len(file.metadata().unwrap().len() + 1).unwrap();
            }
            "replace" | "replace-preserve-mtime" | "replace-whole-second" => {
                let replacement = path.with_extension("replacement");
                let original = std::fs::metadata(path).unwrap();
                let mut file = std::fs::File::create(&replacement).unwrap();
                file.set_len(original.len()).unwrap();
                file.write_all(b"new").unwrap();
                let modified = original.modified().unwrap();
                let modified = if change.ends_with("preserve-mtime") { modified } else if change.ends_with("whole-second") { modified + std::time::Duration::from_secs(1) } else { modified + std::time::Duration::from_nanos(1) };
                file.set_times(std::fs::FileTimes::new().set_modified(modified)).unwrap();
                std::fs::rename(replacement, path).unwrap();
            }
            "chmod" => {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            "unlink" => std::fs::remove_file(path).unwrap(),
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn file_chunks_match_node_blob_changes_before_and_during_reads() {
        let path = scratch("changed-chunks");
        for change in ["chmod", "rewrite", "rewrite-preserve-mtime", "rewrite-whole-second", "resize", "replace", "replace-preserve-mtime", "replace-whole-second", "unlink"] {
            for after_first_read in [false, true] {
                std::fs::File::create(&path).unwrap().set_len(FILE_CHUNK as u64 * 3).unwrap();
                let source = FileSource::snapshot(&path).await.unwrap();
                let stream = file_chunks(source);
                futures_util::pin_mut!(stream);
                if after_first_read {
                    assert_eq!(stream.next().await.unwrap().unwrap().len(), FILE_CHUNK);
                }
                alter_file(&path, change);
                let accepted = change == "chmod" || change.ends_with("preserve-mtime") || change.ends_with("whole-second") || (after_first_read && matches!(change, "replace" | "unlink"));
                if accepted {
                    let next = stream.next().await.unwrap().unwrap();
                    let expected = if after_first_read && change.starts_with("rewrite") { b'x' } else if !after_first_read && change.starts_with("replace") { b'n' } else { 0 };
                    assert_eq!(next[0], expected, "{change}, after first read: {after_first_read}");
                    let mut rewritten = next.contains(&b'x');
                    let mut bytes = next.len() + if after_first_read { FILE_CHUNK } else { 0 };
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        rewritten |= chunk.contains(&b'x');
                        bytes += chunk.len();
                    }
                    assert_eq!(bytes, FILE_CHUNK * 3);
                    assert_eq!(rewritten, change.starts_with("rewrite"));
                } else {
                    assert_eq!(stream.next().await.unwrap().unwrap_err().to_string(), "The blob could not be read", "{change}, after first read: {after_first_read}");
                    assert!(stream.next().await.is_none());
                }
                if path.exists() {
                    std::fs::remove_file(&path).unwrap();
                }
            }
        }
    }

    #[tokio::test]
    async fn fetch_file_sends_disk_chunks_before_reading_the_tail() {
        let path = scratch("streaming");
        let length = 32 * 1024 * 1024;
        std::fs::File::create(&path).unwrap().set_len(length).unwrap();
        let read = Arc::new(AtomicU64::new(0));
        let mut source = FileSource::snapshot(&path).await.unwrap();
        source.read = Some(read.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        mm_test_support::golden::register_loopback_port(addr.port());
        let server_read = read.clone();
        let server = async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0; 8192];
            let mut initial = Vec::new();
            let header_end = loop {
                let read = socket.read(&mut buffer).await.unwrap();
                assert_ne!(read, 0);
                initial.extend_from_slice(&buffer[..read]);
                if let Some(at) = initial.windows(4).position(|window| window == b"\r\n\r\n") {
                    break at + 4;
                }
            };
            let headers = String::from_utf8_lossy(&initial[..header_end]).to_lowercase();
            assert!(headers.contains(&format!("content-length: {length}\r\n")));
            assert!(headers.contains("x-object-headers: fixture\r\n"));
            let mut received = initial.len() - header_end;
            if received == 0 {
                received = socket.read(&mut buffer).await.unwrap();
                assert_ne!(received, 0);
            }
            // The peer sees a real disk prefix while the same upload still has unread file data.
            let read_at_prefix = server_read.load(Ordering::SeqCst);
            assert!(read_at_prefix > 0 && read_at_prefix < length, "read {read_at_prefix} of {length} before the first HTTP prefix");
            while received < length as usize {
                let read = socket.read(&mut buffer).await.unwrap();
                assert_ne!(read, 0);
                assert!(buffer[..read].iter().all(|byte| *byte == 0));
                received += read;
            }
            assert_eq!(received, length as usize);
            assert_eq!(server_read.load(Ordering::SeqCst), length);
            socket.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\nX-Upload: disk\r\nConnection: close\r\n\r\nok").await.unwrap();
        };
        let client = async {
            let mut request = Request::new("PUT", &format!("http://{addr}/object")).unwrap();
            request.headers.set("x-object-headers", "fixture");
            let response = NativeFetch::new().fetch_source(request, &source).await.unwrap();
            assert_eq!(response.status, 201);
            assert_eq!(response.headers.get("x-upload"), Some("disk"));
            assert_eq!(response.body.bytes().await.unwrap(), b"ok");
        };
        tokio::time::timeout(std::time::Duration::from_secs(30), async { tokio::join!(server, client) }).await.unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn fetch_file_aborts_when_the_backing_file_changes_during_http() {
        let path = scratch("changed-http");
        std::fs::File::create(&path).unwrap().set_len(32 * 1024 * 1024).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        mm_test_support::golden::register_loopback_port(addr.port());
        let server_path = path.clone();
        let server = async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let (_, mut prefix) = read_head(&mut socket).await;
            if prefix.is_empty() {
                prefix.resize(8192, 0);
                assert_ne!(socket.read(&mut prefix).await.unwrap(), 0);
            }
            alter_file(&server_path, "rewrite");
            let mut buffer = [0; 8192];
            while matches!(socket.read(&mut buffer).await, Ok(read) if read != 0) {}
        };
        let client = async {
            let request = Request::new("PUT", &format!("http://{addr}/object")).unwrap();
            let error = NativeFetch::new().fetch_file(request, &path).await.unwrap_err();
            assert_eq!(error.to_string(), "TypeError");
        };
        tokio::time::timeout(std::time::Duration::from_secs(30), async { tokio::join!(server, client) }).await.unwrap();
        std::fs::remove_file(path).unwrap();
    }

    // Read the HTTP headers and any immediately received body prefix without draining the upload.
    async fn read_head(socket: &mut tokio::net::TcpStream) -> (String, Vec<u8>) {
        let mut bytes = Vec::new();
        let mut buffer = [0; 8192];
        let header_end = loop {
            let read = socket.read(&mut buffer).await.unwrap();
            assert_ne!(read, 0);
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(at) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break at + 4;
            }
        };
        (String::from_utf8_lossy(&bytes[..header_end]).to_lowercase(), bytes[header_end..].to_vec())
    }

    // Read one small redirect-test request with its declared body.
    async fn read_request(socket: &mut tokio::net::TcpStream) -> (String, Vec<u8>) {
        let (headers, mut bytes) = read_head(socket).await;
        let mut buffer = [0; 8192];
        let length: usize = headers.lines().find_map(|line| line.strip_prefix("content-length: ")).map_or(0, |length| length.parse().unwrap());
        while bytes.len() < length {
            let read = socket.read(&mut buffer).await.unwrap();
            assert_ne!(read, 0);
            bytes.extend_from_slice(&buffer[..read]);
        }
        (headers, bytes)
    }

    #[tokio::test]
    async fn fetch_file_replays_redirects_and_strips_cross_origin_credentials() {
        let path = scratch("redirects");
        let body = b"replayed from disk";
        std::fs::write(&path, body).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (addr, destination_addr) = (listener.local_addr().unwrap(), destination.local_addr().unwrap());
        mm_test_support::golden::register_loopback_port(addr.port());
        mm_test_support::golden::register_loopback_port(destination_addr.port());
        let server = async move {
            for (status, location) in [(307, "/step2".to_owned()), (308, "/step3".to_owned()), (303, format!("http://{destination_addr}/done"))] {
                let (mut socket, _) = listener.accept().await.unwrap();
                let (headers, received) = read_request(&mut socket).await;
                assert!(headers.starts_with("put "));
                assert!(headers.contains("authorization: fixture\r\n"));
                assert_eq!(received, body);
                socket.write_all(format!("HTTP/1.1 {status} Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
            }
            let (mut socket, _) = destination.accept().await.unwrap();
            let (headers, received) = read_request(&mut socket).await;
            assert!(headers.starts_with("get /done "));
            assert!(!headers.contains("authorization:"));
            assert!(!headers.contains("content-type:"));
            assert!(received.is_empty());
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok").await.unwrap();
        };
        let client = async {
            let mut request = Request::new("PUT", &format!("http://{addr}/start")).unwrap();
            request.headers.set("authorization", "fixture");
            request.headers.set("content-type", "fixture");
            let response = NativeFetch::new().fetch_file(request, &path).await.unwrap();
            assert_eq!(response.status, 200);
            assert_eq!(response.body.bytes().await.unwrap(), b"ok");
        };
        tokio::time::timeout(std::time::Duration::from_secs(30), async { tokio::join!(server, client) }).await.unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn fetch_file_matches_node_blob_changes_on_retained_body_redirects() {
        let path = scratch("changed-redirect");
        for (status, change, accepted) in [(307, "rewrite", false), (308, "replace", false), (307, "rewrite-preserve-mtime", true), (308, "replace-preserve-mtime", true), (307, "rewrite-whole-second", true), (308, "replace-whole-second", true), (307, "chmod", true)] {
            std::fs::write(&path, b"immutable backing file").unwrap();
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            mm_test_support::golden::register_loopback_port(addr.port());
            let server_path = path.clone();
            let done = Arc::new(tokio::sync::Notify::new());
            let server_done = done.clone();
            let server = async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let (_, body) = read_request(&mut socket).await;
                assert_eq!(body, b"immutable backing file");
                alter_file(&server_path, change);
                socket.write_all(format!("HTTP/1.1 {status} Redirect\r\nLocation: /retry\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
                drop(socket);
                if accepted {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let (_, body) = read_request(&mut socket).await;
                    assert_eq!(body.len(), b"immutable backing file".len());
                    if change.starts_with("replace") {
                        assert!(body.starts_with(b"new"));
                    } else if change.starts_with("rewrite") {
                        assert!(body.ends_with(b"x"));
                    } else {
                        assert_eq!(body, b"immutable backing file");
                    }
                    socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.unwrap();
                } else {
                    tokio::select! {
                        retry = listener.accept() => {
                            let (mut socket, _) = retry.unwrap();
                            let mut buffer = [0; 8192];
                            while matches!(socket.read(&mut buffer).await, Ok(read) if read != 0) {}
                        }
                        _ = server_done.notified() => {}
                    }
                }
            };
            let client = async {
                let request = Request::new("PUT", &format!("http://{addr}/object")).unwrap();
                let result = NativeFetch::new().fetch_file(request, &path).await;
                done.notify_one();
                if accepted {
                    assert_eq!(result.unwrap().status, 200);
                } else {
                    assert_eq!(result.unwrap_err().to_string(), "TypeError", "{status}, {change}");
                }
            };
            tokio::time::timeout(std::time::Duration::from_secs(30), async { tokio::join!(server, client) }).await.unwrap();
            std::fs::remove_file(&path).unwrap();
        }
    }
}
