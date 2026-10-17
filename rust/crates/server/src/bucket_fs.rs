//! A directory as an object store (`directoryBucket`, `lib/bucket-fs.mjs`): keys are `/`-separated paths below the
//! root, stored response headers sit in a parallel `.headers/` tree as R2 keeps them in object metadata.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::{Bucket, ByteSpan, ListedObject, ObjectInfo, StoredObject};
use mm_core::http::{Body, ByteStream, Headers};
use mm_core::{Error, Object, Result, Value};
use mm_js::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::node_path::{null_bytes_error, resolve};

const HEADERS: &str = ".headers/";
const CHUNK: usize = 64 * 1024;

/// The objects below one directory.
pub struct DirectoryBucket {
    base: PathBuf,
}

fn failure(error: std::io::Error) -> Error {
    Error::internal(error.to_string())
}

// Whether an error means the object is not there, as the JS treats ENOENT, ENOTDIR and EISDIR.
fn is_missing(error: &std::io::Error) -> bool {
    use std::io::ErrorKind::{IsADirectory, NotADirectory, NotFound};
    matches!(error.kind(), NotFound | NotADirectory | IsADirectory)
}

fn missing<T>(result: std::io::Result<T>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if is_missing(&error) => Ok(None),
        Err(error) => Err(failure(error)),
    }
}

fn stored_headers(text: &str) -> Result<Headers> {
    let Value::Object(object) = json::parse(text)? else { return Ok(Headers::new()) };
    Ok(Headers::from_pairs(object.iter().map(|(name, value)| {
        let value = match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => mm_js::number_to_string(*number),
            other => json::to_string(other),
        };
        (name.into_owned(), value)
    })))
}

// A file's bytes from its current position, at most `length` of them, as a stream.
pub(crate) fn file_stream(file: tokio::fs::File, length: u64) -> ByteStream {
    Box::pin(futures_util::stream::unfold((file, length), |(mut file, left)| async move {
        if left == 0 {
            return None;
        }
        let mut chunk = vec![0; CHUNK.min(usize::try_from(left).unwrap_or(CHUNK))];
        match file.read(&mut chunk).await {
            Ok(0) => None,
            Ok(read) => {
                chunk.truncate(read);
                Some((Ok(chunk), (file, left - read as u64)))
            }
            Err(error) => Some((Err(failure(error)), (file, 0))),
        }
    }))
}

impl DirectoryBucket {
    /// A store over `root`, which is created on the first write.
    ///
    /// # Arguments
    /// * `root` - Directory holding the objects, resolved against the current directory.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Self { base: resolve(&cwd, &root.as_ref().to_string_lossy()) }
    }

    fn file(&self, key: &str) -> Result<PathBuf> {
        let target = resolve(&self.base, key);
        if !target.starts_with(&self.base) || target == self.base {
            return Err(Error::status(400, "Invalid object key"));
        }
        // The first file system call on the path throws this `TypeError`.
        if key.contains('\0') {
            return Err(Error::invalid(null_bytes_error(&target)));
        }
        Ok(target)
    }

    fn headers_file(&self, key: &str) -> Result<PathBuf> {
        self.file(key)?;
        self.file(&format!("{HEADERS}{key}.json"))
    }
}

#[async_trait(?Send)]
impl Bucket for DirectoryBucket {
    async fn head(&self, key: &str) -> Result<Option<ObjectInfo>> {
        let Some(info) = missing(tokio::fs::metadata(self.file(key)?).await)? else { return Ok(None) };
        if !info.is_file() {
            return Ok(None);
        }
        let headers = missing(tokio::fs::read_to_string(self.headers_file(key)?).await)?;
        Ok(Some(ObjectInfo { size: info.len(), headers: headers.filter(|text| !text.is_empty()).map(|text| stored_headers(&text)).transpose()? }))
    }

    async fn read(&self, key: &str, range: Option<ByteSpan>) -> Result<Option<StoredObject>> {
        let Some(info) = self.head(key).await? else { return Ok(None) };
        let empty = |info: ObjectInfo| StoredObject { size: info.size, headers: info.headers, body: Body::Empty.into_stream() };
        if range.is_some_and(|range| range.length == 0) {
            return Ok(Some(empty(info)));
        }
        let Some(mut file) = missing(tokio::fs::File::open(self.file(key)?).await)? else { return Ok(Some(empty(info))) };
        let (offset, length) = range.map_or((0, u64::MAX), |range| (range.offset, range.length));
        if offset > 0 {
            file.seek(SeekFrom::Start(offset)).await.map_err(failure)?;
        }
        Ok(Some(StoredObject { size: info.size, headers: info.headers, body: file_stream(file, length) }))
    }

    async fn bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        missing(tokio::fs::read(self.file(key)?).await)
    }

    async fn write(&self, key: &str, body: Body, _length: Option<u64>, headers: Option<&Headers>) -> Result<()> {
        let target = self.file(key)?;
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(failure)?;
        }
        let mut temp = target.clone().into_os_string();
        temp.push(format!(".{}.tmp", mm_js::base64::hex(&mm_core::runtime::random_bytes(6))));
        let temp = PathBuf::from(temp);
        let written = async {
            let mut file = tokio::fs::File::create(&temp).await.map_err(failure)?;
            let mut stream = body.into_stream();
            while let Some(chunk) = stream.next().await {
                file.write_all(&chunk?).await.map_err(failure)?;
            }
            file.flush().await.map_err(failure)?;
            drop(file);
            tokio::fs::rename(&temp, &target).await.map_err(failure)
        }
        .await;
        let _ = tokio::fs::remove_file(&temp).await;
        written?;
        let headers_file = self.headers_file(key)?;
        match headers {
            None => {
                missing(tokio::fs::remove_file(&headers_file).await)?;
            }
            Some(headers) => {
                if let Some(parent) = headers_file.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(failure)?;
                }
                let object: Object = headers.iter().map(|(name, value)| (name.to_owned(), Value::from(value))).collect();
                tokio::fs::write(&headers_file, json::to_string(&Value::Object(object))).await.map_err(failure)?;
            }
        }
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<()> {
        // `rm(path, { force: true })` ignores only a missing path.
        for path in [self.file(key)?, self.headers_file(key)?] {
            match tokio::fs::remove_file(&path).await {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(failure(error)),
                _ => {}
            }
        }
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<ListedObject>> {
        let folder = prefix.rfind('/').map_or("", |slash| &prefix[..slash]);
        let directory = if folder.is_empty() { self.base.clone() } else { self.file(folder)? };
        let mut objects = Vec::new();
        let mut pending = vec![directory];
        while let Some(directory) = pending.pop() {
            let Some(mut entries) = missing(tokio::fs::read_dir(&directory).await)? else { continue };
            while let Some(entry) = entries.next_entry().await.map_err(failure)? {
                let kind = entry.file_type().await.map_err(failure)?;
                let path = entry.path();
                if kind.is_dir() {
                    pending.push(path);
                    continue;
                }
                if !kind.is_file() || entry.file_name().to_string_lossy().ends_with(".tmp") {
                    continue;
                }
                let Ok(relative) = path.strip_prefix(&self.base) else { continue };
                let key = relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
                if key.starts_with(prefix) && !key.starts_with(HEADERS) {
                    let size = tokio::fs::metadata(&path).await.map_err(failure)?.len();
                    objects.push(ListedObject { key, size });
                }
            }
        }
        objects.sort_by(|a, b| mm_js::text::compare(&a.key, &b.key));
        Ok(objects)
    }
}
