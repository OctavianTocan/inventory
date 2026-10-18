//! Synthetic fixtures shared by the native and hosted media regression checks.

#![allow(dead_code)]

use std::cell::Cell;
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use async_trait::async_trait;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use mm_core::api::{Api, ApiOptions};
use mm_core::host::{
    Fetch, FetchedResource, Media, Resource, ResourceOptions, Screenshot, UploadStorage,
};
use mm_core::hosted::{Hosted, HostedOptions};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime;
use mm_core::site::{LibraryApi, Site};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::{Error, Result, Value};
use mm_js::json;
use mm_sqlite::Sqlite;
use mymind::bucket_fs::DirectoryBucket;
use mymind::media_processor::LocalProcessor;

/// Test-owned files and a process-local media command environment.
pub struct Fixture {
    pub root: PathBuf,
    old_path: Option<std::ffi::OsString>,
    old_env_file: Option<std::ffi::OsString>,
}

// Quote an arbitrary filesystem path as one POSIX shell word.
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

impl Fixture {
    /// Create target-local synthetic media inputs and denied external services.
    ///
    /// # Arguments
    /// * `name` - Unique suite name within this integration test executable.
    ///
    /// # Returns
    /// An owning fixture that removes its files and restores process overrides on drop.
    pub fn new(name: &str) -> Self {
        let root =
            Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::write(root.join("fixture.png"), png()).unwrap();
        let script = format!(
            "#!/bin/sh\nroot={}\ncommand=${{0##*/}}\noutput=\nsource=\nprevious=\nprintf '%s' \"$command\" >> \"$root/commands\"\nfor argument do\n  printf '\\t%s' \"$argument\" >> \"$root/commands\"\n  if [ \"$previous\" = '-i' ]; then source=$argument; fi\n  previous=$argument\n  output=$argument\ndone\nprintf '\\n' >> \"$root/commands\"\nif [ \"$command\" = ffprobe ]; then\n  printf '%s' '{{\"streams\":[{{\"width\":48,\"height\":32}}],\"format\":{{\"duration\":\"1\"}}}}'\n  exit 0\nfi\nif [ \"$command\" = pdftoppm ]; then output=$output.png; fi\ncase \"$output\" in\n  *.tmp)\n    if [ -f \"$root/fail\" ]; then\n      printf partial > \"$output\"\n      : > \"$root/started\"\n      while [ ! -f \"$root/release\" ]; do /bin/sleep 0.01; done\n      exit 1\n    fi\n    printf 'complete encoded video' > \"$output\";;\n  *.mp4) printf 'encoded video' > \"$output\";;\n  *) /bin/cp \"$root/fixture.png\" \"$output\";;\nesac\n",
            quote(&root)
        );
        for program in ["ffmpeg", "ffprobe", "pdftoppm"] {
            let path = root.join("bin").join(program);
            std::fs::write(&path, &script).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let old_path = std::env::var_os("PATH");
        let old_env_file = std::env::var_os("MYMIND_ENV_FILE");
        let mut paths = vec![root.join("bin")];
        if let Some(path) = &old_path {
            paths.extend(std::env::split_paths(path));
        }
        let path = std::env::join_paths(paths).unwrap();
        let env_file = root.join("absent-environment-file");
        assert!(!env_file.exists());
        // Each executable has one test; overrides are installed before its runtime starts.
        unsafe {
            std::env::set_var("PATH", path);
            std::env::set_var("MYMIND_ENV_FILE", &env_file);
        }
        runtime::set_env(HashMap::from([
            ("OFFLINE".into(), "1".into()),
            (
                "MYMIND_ENV_FILE".into(),
                env_file.to_string_lossy().into_owned(),
            ),
        ]));
        Self {
            root,
            old_path,
            old_env_file,
        }
    }

    /// Return an empty, uniquely named directory inside this fixture.
    ///
    /// # Arguments
    /// * `name` - Relative directory name.
    pub fn directory(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// Read the exact ordered synthetic media command log.
    pub fn commands(&self) -> Vec<Vec<String>> {
        std::fs::read_to_string(self.root.join("commands"))
            .unwrap_or_default()
            .lines()
            .map(|line| line.split('\t').map(str::to_owned).collect())
            .collect()
    }

    /// Clear command observations between independent scenarios.
    pub fn clear_commands(&self) {
        let _ = std::fs::remove_file(self.root.join("commands"));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // The suite runtime and its child processes have finished before fixture teardown.
        unsafe {
            match &self.old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match &self.old_env_file {
                Some(value) => std::env::set_var("MYMIND_ENV_FILE", value),
                None => std::env::remove_var("MYMIND_ENV_FILE"),
            }
        }
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

// Calculate the PNG chunk checksum without another dependency.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320u32 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

// Append one correctly framed PNG chunk.
fn chunk(output: &mut Vec<u8>, kind: &[u8; 4], bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    let start = output.len();
    output.extend_from_slice(kind);
    output.extend_from_slice(bytes);
    output.extend_from_slice(&crc32(&output[start..]).to_be_bytes());
}

/// Generate the original checks' 48 by 32 RGB image with background `#539ac1`.
pub fn png() -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::from(48u32.to_be_bytes());
    header.extend_from_slice(&32u32.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut bytes, b"IHDR", &header);
    let mut row = vec![0];
    for _ in 0..48 {
        row.extend_from_slice(&[0x53, 0x9a, 0xc1]);
    }
    let mut raw = Vec::new();
    for _ in 0..32 {
        raw.extend_from_slice(&row);
    }
    let mut compressed = ZlibEncoder::new(Vec::new(), Compression::default());
    compressed.write_all(&raw).unwrap();
    chunk(&mut bytes, b"IDAT", &compressed.finish().unwrap());
    chunk(&mut bytes, b"IEND", &[]);
    bytes
}

/// Transport that fails the test before any network or browser operation.
pub struct Denied;

#[async_trait(?Send)]
impl Fetch for Denied {
    async fn fetch(&self, _request: Request) -> Result<Response> {
        panic!("external fetch is forbidden in media checks")
    }
}

#[async_trait(?Send)]
impl Resource for Denied {
    async fn fetch(&self, _url: &str, _options: ResourceOptions) -> Result<FetchedResource> {
        panic!("resource fetching is forbidden in media checks")
    }
}

#[async_trait(?Send)]
impl Screenshot for Denied {
    async fn capture(&self, _url: &str, _offline: bool) -> Result<Vec<u8>> {
        panic!("browser capture is forbidden in media checks")
    }
}

/// Seed an otherwise empty SQLite library with only the synthetic user.
///
/// # Arguments
/// * `path` - Target-local database location, or `:memory:`.
///
/// # Returns
/// Its open database and store handles.
pub fn library(path: &Path) -> (Rc<Sqlite>, Rc<Store>) {
    let db = Rc::new(Sqlite::open(path, false).unwrap());
    let store = Rc::new(Store::new(db.clone()).unwrap());
    db.execute(
        "INSERT INTO users VALUES (?, ?, ?, 0, ?)",
        &[
            "test-user".into(),
            "test-account".into(),
            "test-".into(),
            "{}".into(),
        ],
    )
    .unwrap();
    (db, store)
}

/// Build the real API with exclusively local media and denied external services.
///
/// # Arguments
/// * `store` - Synthetic library store.
/// * `media` - Media backend under test.
/// * `upload_storage` - Temporary chunk backend under test.
///
/// # Returns
/// The API with offline background work.
pub async fn api(
    store: Rc<Store>,
    media: Rc<dyn Media>,
    upload_storage: Rc<dyn UploadStorage>,
) -> Api {
    Api::new(ApiOptions {
        store,
        media,
        upload_storage,
        screenshot: Rc::new(Denied),
        resource: Rc::new(Denied),
        fetch: Rc::new(Denied),
        readonly: false,
        localize: |value| value.to_owned(),
        push: Box::new(|_, _| {}),
        random: runtime::random_bytes,
    })
    .await
    .unwrap()
}

/// Build the real hosted library with a synthetic database and directory bucket.
///
/// # Arguments
/// * `db` - Synthetic library database.
/// * `bucket` - Library object storage.
/// * `directory` - Target-local processor scratch root.
/// * `name` - Library namespace.
///
/// # Returns
/// The hosted server with all external services denied.
pub async fn hosted(
    db: Rc<Sqlite>,
    bucket: Rc<DirectoryBucket>,
    directory: &Path,
    name: &str,
) -> Hosted {
    Hosted::new(HostedOptions {
        db,
        bucket,
        library: name.to_owned(),
        processor: Rc::new(LocalProcessor::new(Some(directory.to_owned()))),
        push: Box::new(|_, _| {}),
        readonly: false,
        offline: true,
        screenshot: Rc::new(Denied),
        resource: Rc::new(Denied),
        fetch: Rc::new(Denied),
        random: runtime::random_bytes,
    })
    .await
    .unwrap()
}

/// A collected HTTP answer retaining exact status, headers and bytes.
pub struct Answer {
    pub status: u16,
    pub headers: Headers,
    pub bytes: Vec<u8>,
}

impl Answer {
    /// Decode an answer known to contain JSON.
    pub fn value(&self) -> Value {
        json::parse(std::str::from_utf8(&self.bytes).unwrap()).unwrap()
    }

    /// Decode an answer known to contain UTF-8 text.
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap()
    }
}

/// Drain a response while preserving its status and headers.
///
/// # Arguments
/// * `response` - Response returned by the production media or request handler.
pub async fn collect(response: Response) -> Answer {
    Answer {
        status: response.status,
        headers: response.headers,
        bytes: response.body.bytes().await.unwrap(),
    }
}

/// Construct an in-memory request with a loopback URL.
///
/// # Arguments
/// * `method` - HTTP method.
/// * `path` - Relative path below the synthetic loopback origin.
/// * `body` - Request body.
/// * `headers` - Explicit request headers.
pub fn request(method: &str, path: &str, body: Body, headers: &[(&str, &str)]) -> Request {
    let mut request = Request::new(method, &format!("http://127.0.0.1{path}")).unwrap();
    request.headers = Headers::from_pairs(headers.iter().copied());
    if !request.headers.has("content-type") {
        request.headers.set("content-type", "application/json");
    }
    request.body = body;
    request
}

/// Send one request through the real library API and require that it owns the route.
///
/// # Arguments
/// * `api` - API under test.
/// * `method` - HTTP method.
/// * `path` - Relative path.
/// * `body` - Request body.
/// * `headers` - Request headers.
pub async fn ask_api(
    api: &Api,
    method: &str,
    path: &str,
    body: Body,
    headers: &[(&str, &str)],
) -> Answer {
    let response = api
        .handle(&mut request(method, path, body, headers))
        .await
        .unwrap();
    assert!(response.is_some(), "API must own {method} {path}");
    collect(response.unwrap()).await
}

/// Send one request through the real hosted site.
///
/// # Arguments
/// * `hosted` - Site under test.
/// * `method` - HTTP method.
/// * `path` - Relative path.
/// * `body` - Request body.
/// * `headers` - Request headers.
pub async fn ask_hosted(
    hosted: &Hosted,
    method: &str,
    path: &str,
    body: Body,
    headers: &[(&str, &str)],
) -> Answer {
    collect(
        hosted
            .handle(&mut request(method, path, body, headers))
            .await
            .unwrap(),
    )
    .await
}

/// Build a JSON body preserving JavaScript property order.
///
/// # Arguments
/// * `value` - JSON request value.
pub fn body(value: Value) -> Body {
    Body::Bytes(json::to_string(&value).into_bytes())
}

/// Assert a failure's explicit HTTP status without requiring its success value to implement `Debug`.
///
/// # Arguments
/// * `result` - Operation expected to fail.
/// * `wanted` - Required status.
pub fn status<T>(result: Result<T>, wanted: u16) {
    match result {
        Err(Error::Status { status, .. }) => assert_eq!(status, wanted),
        Err(other) => panic!("expected HTTP {wanted}, got {other}"),
        Ok(_) => panic!("expected HTTP {wanted}, operation succeeded"),
    }
}

/// Wait a bounded amount of time for a fake command's synchronization file.
///
/// # Arguments
/// * `path` - Target-local marker created by a child process.
pub async fn marker(path: &Path) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("synthetic media command did not reach its gate");
}

/// Stream polling counter used to prove reservations reject a body before consuming it.
pub struct Counted {
    pub polls: Rc<Cell<usize>>,
    pub body: Option<Vec<u8>>,
}

impl futures_util::Stream for Counted {
    type Item = Result<Vec<u8>>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.polls.set(self.polls.get() + 1);
        std::task::Poll::Ready(self.body.take().map(Ok))
    }
}
