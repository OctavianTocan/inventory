//! Chunked uploads kept as one growing file per upload in a private directory (`directoryStorage` of
//! `lib/uploads.mjs`).

use std::io::SeekFrom;
use std::path::PathBuf;

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::{MediaInput, MediaResult, UploadStorage};
use mm_core::http::Body;
use mm_core::{Error, Result};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::node_path::resolve;

/// The upload files below one directory.
pub struct DirectoryUploads {
    directory: PathBuf,
}

fn failure(error: std::io::Error) -> Error {
    Error::internal(error.to_string())
}

fn absent<T>(result: std::io::Result<T>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failure(error)),
    }
}

impl DirectoryUploads {
    /// The storage for chunks below `directory`, which is created on the first write.
    ///
    /// # Arguments
    /// * `directory` - Private temporary upload directory.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self { directory: directory.into() }
    }

    fn file(&self, id: &str) -> PathBuf {
        self.directory.join(id)
    }
}

#[async_trait(?Send)]
impl UploadStorage for DirectoryUploads {
    fn key(&self) -> String {
        resolve(&std::env::current_dir().unwrap_or_default(), &self.directory.to_string_lossy()).to_string_lossy().into_owned()
    }

    async fn names(&self) -> Result<Vec<String>> {
        let Some(mut entries) = absent(tokio::fs::read_dir(&self.directory).await)? else { return Ok(Vec::new()) };
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(failure)? {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
        Ok(names)
    }

    async fn size(&self, id: &str) -> Result<Option<u64>> {
        Ok(absent(tokio::fs::metadata(self.file(id)).await)?.map(|metadata| metadata.len()))
    }

    async fn truncate(&self, id: &str, size: u64) -> Result<()> {
        let file = tokio::fs::OpenOptions::new().write(true).open(self.file(id)).await.map_err(failure)?;
        file.set_len(size).await.map_err(failure)
    }

    async fn remove(&self, id: &str) -> Result<()> {
        absent(tokio::fs::remove_file(self.file(id)).await).map(|_| ())
    }

    fn input(&self, id: &str) -> MediaInput {
        MediaInput::File(self.file(id).to_string_lossy().into_owned())
    }

    fn claimed_input(&self, found: &MediaResult) -> MediaInput {
        MediaInput::File(found.key.clone())
    }

    async fn write(&self, id: &str, start: u64, chunks: Body, _length: Option<u64>) -> Result<()> {
        tokio::fs::create_dir_all(&self.directory).await.map_err(failure)?;
        let mut options = tokio::fs::OpenOptions::new();
        options.write(true);
        if start == 0 {
            options.create(true).truncate(true);
        }
        let mut file = options.open(self.file(id)).await.map_err(failure)?;
        let mut position = start;
        let mut stream = chunks.into_stream();
        let written: Result<()> = async {
            file.seek(SeekFrom::Start(start)).await.map_err(failure)?;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await.map_err(failure)?;
                position += chunk.len() as u64;
            }
            file.flush().await.map_err(failure)
        }
        .await;
        if written.is_err() {
            let _ = file.set_len(start).await;
        }
        written
    }
}
