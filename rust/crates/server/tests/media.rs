//! Synthetic tests for `LocalMedia`, `LocalProcessor`, and `mymind container` without reading recordings or golden fixtures.

mod media_check_support;

use std::path::Path;

use media_check_support::{Fixture, collect, png};
use mm_core::host::{Media, MediaInput, MediaProcessor, MediaUser, RenditionSource};
use mm_core::http::Body;
use mm_core::obj;
use mymind::local_media::{LocalMedia, analysis_image};
use mymind::media_processor::LocalProcessor;

fn user() -> MediaUser {
    MediaUser {
        account_uid: "acct".to_owned(),
        id_prefix: "card-".to_owned(),
    }
}

#[test]
fn prepare_renditions_send_and_analysis_work_on_synthetic_inputs() {
    let fixture = Fixture::new("media-synthetic");
    let dir = fixture.directory("store");
    let media = LocalMedia::new(&dir);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let png_bytes = png();
        let prepared = media
            .prepare(MediaInput::Bytes(png_bytes.clone()), "image/png", "sample.png", &user(), "card-123")
            .await
            .unwrap();
        let blob = prepared.data.get("blob");
        let rel = blob.get("path").as_str().unwrap().to_owned();
        assert!(Path::new(&dir).join(&rel).is_file());
        assert_eq!(blob.get("type").as_str(), Some("image/png"));

        // Original and rendition lookup.
        let orig = media.response(&rel).await.unwrap().unwrap();
        assert_eq!(orig.content_type, "image/png");
        assert_eq!(orig.size as usize, png_bytes.len());

        let rend_rel = format!("{rel};40x30-c.webp");
        let rend = media.response(&rend_rel).await.unwrap().unwrap();
        assert_eq!(rend.content_type, "image/webp");
        assert!(rend.size > 0);
        assert!(dir.join(".renditions").join(rend_rel.replace('/', "_")).is_file());

        // Range send and HEAD send.
        let full = collect(media.send(&orig, "GET", None).await.unwrap()).await;
        assert_eq!(full.status, 200);
        assert_eq!(full.bytes, png_bytes);

        let range = collect(media.send(&orig, "GET", Some("bytes=0-7")).await.unwrap()).await;
        assert_eq!(range.status, 206);
        assert_eq!(range.bytes, &png_bytes[..8]);

        let head = collect(media.send(&orig, "HEAD", None).await.unwrap()).await;
        assert_eq!(head.status, 200);
        assert!(head.bytes.is_empty());

        // Missing original returns None.
        assert!(media.response("acct/missing/file.png").await.unwrap().is_none());
        assert!(media.response("acct/missing/file.png;100x100.webp").await.unwrap().is_none());

        // Analysis image downscales to PNG.
        let img = media.image(&obj! { "path" => rel.as_str() }.into()).await.unwrap().unwrap();
        assert_eq!(img.mime, "image/png");
        assert!(!img.data.is_empty());

        let direct = analysis_image(png_bytes).await.unwrap();
        assert_eq!(direct.mime, "image/png");
    });
}

#[test]
fn processor_matches_local_media_operations() {
    let fixture = Fixture::new("processor-synthetic");
    let processor = LocalProcessor::new(Some(fixture.root.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let png_bytes = png();
        let prepared = processor
            .prepare(Body::Bytes(png_bytes.clone()), "image/png", "proc.png", &user(), "card-456")
            .await
            .unwrap();
        assert_eq!(prepared.data.get("type").as_str(), Some("Image"));

        let out = processor
            .rendition(
                RenditionSource {
                    body: Body::Bytes(png_bytes.clone()),
                    size: png_bytes.len() as u64,
                    name: "acct/456/proc.png".to_owned(),
                },
                "acct/456/proc.png;20x20-t.webp",
            )
            .await
            .unwrap()
            .unwrap();
        let rend_bytes = out.body.bytes().await.unwrap();
        assert!(!rend_bytes.is_empty());

        let analyzed = processor.image(Body::Bytes(png_bytes)).await.unwrap();
        assert_eq!(analyzed.mime, "image/png");
    });
}
