//! Port of `lib/media-processor.mjs`: media whose bytes are stored elsewhere, processed with [`LocalMedia`] in a
//! temporary directory per call.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures_util::StreamExt;
use mm_core::host::{AnalysisImage, Media, MediaInput, MediaProcessor, MediaUser, PreparedMedia, RenditionOutput, RenditionSource};
use mm_core::http::Body;
use mm_core::runtime::random_bytes;
use mm_core::{Error, Result};
use mm_js::base64;
use tokio::io::AsyncWriteExt;

use crate::local_media::{LocalMedia, analysis_image, extract_audio};
use crate::node_path::resolve;

/// Processes media in a fresh directory per call (`createProcessor`).
pub struct LocalProcessor {
    directory: PathBuf,
}

fn io(error: std::io::Error) -> Error {
    Error::internal(error.to_string())
}

async fn save(file: &Path, body: Body) -> Result<()> {
    if let Some(parent) = file.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(io)?;
    }
    match body {
        Body::Bytes(bytes) => tokio::fs::write(file, bytes).await.map_err(io),
        other => {
            let mut out = tokio::fs::File::create(file).await.map_err(io)?;
            let mut stream = other.into_stream();
            while let Some(chunk) = stream.next().await {
                out.write_all(&chunk?).await.map_err(io)?;
            }
            out.flush().await.map_err(io)
        }
    }
}

impl LocalProcessor {
    /// A processor whose per-call directories are made below `directory`.
    ///
    /// # Arguments
    /// * `directory` - Parent of the temporary directories; the system temporary directory when `None`.
    pub fn new(directory: Option<PathBuf>) -> Self {
        Self { directory: directory.unwrap_or_else(std::env::temp_dir) }
    }

    async fn within<T>(&self, task: impl AsyncFnOnce(&Path) -> Result<T>) -> Result<T> {
        let folder = self.directory.join(format!("mymind-media-{}", base64::hex(&random_bytes(3))));
        tokio::fs::create_dir_all(&folder).await.map_err(io)?;
        let done = task(&folder).await;
        let _ = tokio::fs::remove_dir_all(&folder).await;
        done
    }
}

#[async_trait(?Send)]
impl MediaProcessor for LocalProcessor {
    /// Identify a new file and derive the card media fields (`prepare`).
    ///
    /// # Arguments
    /// * `body` - The file's bytes.
    /// * `content_type` - The type the client claimed.
    /// * `name` - Original file name.
    /// * `user` - Account the media is stored for.
    /// * `id` - Id of the card being made.
    ///
    /// # Errors
    /// As [`Media::prepare`].
    async fn prepare(&self, body: Body, content_type: &str, name: &str, user: &MediaUser, id: &str) -> Result<PreparedMedia> {
        self.within(async |folder| {
            let input = folder.join("input");
            save(&input, body).await?;
            let media = LocalMedia::new(folder.join("media"));
            media.prepare(MediaInput::File(input.to_string_lossy().into_owned()), content_type, name, user, id).await
        })
        .await
    }

    /// Make the rendition `relative` names of a stored original (`rendition`).
    ///
    /// # Arguments
    /// * `original` - The original and where it is stored.
    /// * `relative` - Media path with its `;` rendition suffix.
    ///
    /// # Returns
    /// The rendition, or `None` when `relative` is not a rendition of the original.
    ///
    /// # Errors
    /// Status 400 `Invalid media path` when the original's name leaves the media folder, otherwise as
    /// [`Media::response`].
    async fn rendition(&self, original: RenditionSource, relative: &str) -> Result<Option<RenditionOutput>> {
        self.within(async |folder| {
            let root = folder.join("media");
            let target = resolve(&root, &original.name);
            if !target.to_string_lossy().starts_with(&format!("{}/", root.to_string_lossy())) {
                return Err(Error::status(400, "Invalid media path"));
            }
            save(&target, original.body).await?;
            let media = LocalMedia::new(&root);
            let Some(result) = media.response(relative).await? else { return Ok(None) };
            let bytes = tokio::fs::read(&result.key).await.map_err(io)?;
            Ok(Some(RenditionOutput { size: Some(bytes.len() as u64), body: Body::Bytes(bytes) }))
        })
        .await
    }

    /// Downscale an image to the PNG the AI analysis reads (`image`).
    ///
    /// # Arguments
    /// * `body` - The image's bytes.
    ///
    /// # Errors
    /// An internal error when the image cannot be decoded.
    async fn image(&self, body: Body) -> Result<AnalysisImage> {
        analysis_image(body.bytes().await?).await
    }

    /// Pull the sound out of a video (`audio`).
    ///
    /// # Arguments
    /// * `body` - The video's bytes.
    ///
    /// # Returns
    /// The mono Ogg Opus audio, or `None` when the video has no sound.
    ///
    /// # Errors
    /// As [`extract_audio`].
    async fn audio(&self, body: Body) -> Result<Option<Vec<u8>>> {
        self.within(async |folder| {
            let input = folder.join("input");
            save(&input, body).await?;
            extract_audio(&input.to_string_lossy()).await
        })
        .await
    }
}
