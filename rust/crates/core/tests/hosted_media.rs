//! Synthetic tests for `HostedMedia` and `BucketUploads` without reading recordings or golden fixtures.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use futures_util::future::join_all;
use futures_util::stream;
use mm_core::host::{
    AnalysisImage, Bucket, ByteSpan, ListedObject, MediaProcessor, MediaUser, ObjectInfo, PreparedMedia, RenditionOutput,
    RenditionSource, StoredObject, UploadStorage,
};
use mm_core::hosted_media::{BucketUploads, HostedMedia};
use mm_core::http::Body;
use mm_core::{Error, Value, arr, obj};
use sha2::{Digest, Sha256};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            return Poll::Ready(());
        }
        self.0 = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn invalid_key() -> Error {
    Error::status(400, "Invalid object key")
}

fn normalize(key: &str) -> Result<String, Error> {
    if key.starts_with('/') {
        return Err(invalid_key());
    }
    let mut parts: Vec<&str> = Vec::new();
    for part in key.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or_else(invalid_key)?;
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() { Err(invalid_key()) } else { Ok(parts.join("/")) }
}

#[derive(Default)]
struct MemoryBucket {
    objects: RefCell<BTreeMap<String, Vec<u8>>>,
    log: RefCell<Vec<Value>>,
}

fn bytes_stream(bytes: Vec<u8>) -> mm_core::http::ByteStream {
    Box::pin(stream::once(async move { Ok(bytes) }))
}

#[async_trait(?Send)]
impl Bucket for MemoryBucket {
    async fn head(&self, key: &str) -> mm_core::Result<Option<ObjectInfo>> {
        self.log.borrow_mut().push(arr!["head", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        Ok(self.objects.borrow().get(&key).map(|bytes| ObjectInfo { size: bytes.len() as u64, headers: None }))
    }

    async fn read(&self, key: &str, range: Option<ByteSpan>) -> mm_core::Result<Option<StoredObject>> {
        let span = range.map_or(Value::Null, |span| arr![span.offset as f64, span.length as f64]);
        self.log.borrow_mut().push(arr!["read", key, span]);
        YieldNow(false).await;
        let key = normalize(key)?;
        let Some(bytes) = self.objects.borrow().get(&key).cloned() else { return Ok(None) };
        let size = bytes.len() as u64;
        let body = match range {
            Some(span) if span.length == 0 => Vec::new(),
            Some(span) => {
                let start = (span.offset as usize).min(bytes.len());
                let end = (span.offset.saturating_add(span.length) as usize).min(bytes.len());
                bytes[start..end].to_vec()
            }
            None => bytes,
        };
        Ok(Some(StoredObject { size, headers: None, body: bytes_stream(body) }))
    }

    async fn bytes(&self, key: &str) -> mm_core::Result<Option<Vec<u8>>> {
        self.log.borrow_mut().push(arr!["bytes", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        Ok(self.objects.borrow().get(&key).cloned())
    }

    async fn write(&self, key: &str, body: Body, length: Option<u64>, headers: Option<&mm_core::http::Headers>) -> mm_core::Result<()> {
        self.log.borrow_mut().push(arr!["write", key, length.map(|length| length as f64), headers.is_some()]);
        YieldNow(false).await;
        let key = normalize(key)?;
        let bytes = body.bytes().await?;
        self.objects.borrow_mut().insert(key, bytes);
        Ok(())
    }

    async fn remove(&self, key: &str) -> mm_core::Result<()> {
        self.log.borrow_mut().push(arr!["remove", key]);
        YieldNow(false).await;
        let key = normalize(key)?;
        self.objects.borrow_mut().remove(&key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> mm_core::Result<Vec<ListedObject>> {
        self.log.borrow_mut().push(arr!["list", prefix]);
        YieldNow(false).await;
        if let Some(index) = prefix.rfind('/').filter(|index| *index > 0) {
            normalize(&prefix[..index])?;
        }
        Ok(self
            .objects
            .borrow()
            .iter()
            .filter(|(key, _)| key.starts_with(prefix) && !key.starts_with(".headers/") && !key.ends_with(".tmp"))
            .map(|(key, bytes)| ListedObject { key: key.clone(), size: bytes.len() as u64 })
            .collect())
    }
}

struct FakeProcessor {
    log: Rc<RefCell<Vec<Value>>>,
}

fn extension(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => "png",
        "application/pdf" => "pdf",
        "video/mp4" => "mp4",
        _ => "bin",
    }
}

#[async_trait(?Send)]
impl MediaProcessor for FakeProcessor {
    async fn prepare(&self, body: Body, content_type: &str, name: &str, user: &MediaUser, id: &str) -> mm_core::Result<PreparedMedia> {
        let bytes = body.bytes().await?;
        let digest = sha(&bytes);
        let who = obj! { "accountUid" => user.account_uid.as_str(), "idPrefix" => user.id_prefix.as_str() };
        self.log.borrow_mut().push(arr![
            "prepare",
            obj! { "length" => bytes.len() as f64, "sha" => digest.as_str(), "type" => content_type, "name" => name, "user" => who, "id" => id }
        ]);
        match name {
            "fail-plain" => return Err(Error::internal("boom")),
            "fail-415" => return Err(Error::status(415, "Unsupported media")),
            "noblob" => return Ok(PreparedMedia { data: obj! { "type" => "Image" }.into(), extras: obj! {}.into() }),
            "nopath" => return Ok(PreparedMedia { data: obj! { "type" => "Image", "blob" => obj! {} }.into(), extras: obj! {}.into() }),
            _ => {}
        }
        let file = match name {
            "travpath" => "../escape.bin".to_owned(),
            "escape" => "../../../../escape.bin".to_owned(),
            _ => format!("{}/{}/{}.{}", user.account_uid, id.get(user.id_prefix.len()..).unwrap_or(""), &digest[..8], extension(content_type)),
        };
        let blob = obj! { "id" => id, "path" => file.as_str(), "type" => content_type, "size" => bytes.len() as f64, "name" => name };
        let cover = obj! { "src" => format!("/media/{file};800x450-t.webp"), "width" => 800.0, "height" => 450.0 };
        Ok(PreparedMedia {
            data: obj! { "type" => "Image", "title" => name, "blob" => blob }.into(),
            extras: obj! { "cover" => cover }.into(),
        })
    }

    async fn rendition(&self, original: RenditionSource, relative: &str) -> mm_core::Result<Option<RenditionOutput>> {
        let size = original.size;
        let name = original.name.clone();
        let bytes = original.body.bytes().await?;
        let digest = sha(&bytes);
        self.log.borrow_mut().push(arr![
            "rendition",
            obj! { "name" => name, "size" => size as f64, "sha" => digest.as_str(), "relative" => relative }
        ]);
        if relative.contains("null") {
            return Ok(None);
        }
        if relative.contains("boom") {
            return Err(Error::internal("rendition boom"));
        }
        if relative.contains("teapot") {
            return Err(Error::status(418, "teapot"));
        }
        let body = format!("R:{relative}:{digest}").into_bytes();
        if relative.contains("nosize") {
            return Ok(Some(RenditionOutput { body: Body::Bytes(body), size: None }));
        }
        if relative.contains("streamy") {
            return Ok(Some(RenditionOutput { body: Body::Stream(bytes_stream(body)), size: None }));
        }
        let length = body.len() as u64;
        Ok(Some(RenditionOutput { body: Body::Bytes(body), size: Some(length) }))
    }

    async fn image(&self, body: Body) -> mm_core::Result<AnalysisImage> {
        let bytes = body.bytes().await?;
        let digest = sha(&bytes);
        self.log.borrow_mut().push(arr!["image", obj! { "length" => bytes.len() as f64, "sha" => digest.as_str() }]);
        Ok(AnalysisImage { mime: "image/png".to_owned(), data: format!("I:{digest}").into_bytes() })
    }
}

fn user() -> MediaUser {
    MediaUser { account_uid: "acct1".to_owned(), id_prefix: "card_".to_owned() }
}

const SYNTHETIC: [(&str, &str); 9] = [
    ("synth/empty.bin", ""),
    ("synth/a b/c%20d.png", "png-bytes-1"),
    ("synth/noext", "n"),
    ("synth/d/pic.page-1.webp", "p1"),
    ("synth/e/pic.page-1.webp", "p"),
    ("synth/e/pic.png", "real"),
    ("synth/f/m.gif", "g"),
    ("synth/f/m.jpg", "j"),
    ("synth/tiny.txt", "abc"),
];

fn setup(prefix: &str) -> (Rc<MemoryBucket>, Rc<RefCell<Vec<Value>>>, HostedMedia, BucketUploads) {
    let bucket = Rc::new(MemoryBucket::default());
    for (relative, body) in SYNTHETIC {
        bucket
            .objects
            .borrow_mut()
            .insert(format!("{prefix}media/{relative}"), body.as_bytes().to_vec());
    }
    let log = Rc::new(RefCell::new(Vec::new()));
    let processor: Rc<dyn MediaProcessor> = Rc::new(FakeProcessor { log: Rc::clone(&log) });
    let hosted = HostedMedia::new(bucket.clone(), prefix, processor);
    let uploads = BucketUploads::new(bucket.clone(), prefix);
    (bucket, log, hosted, uploads)
}

#[test]
fn objects_ranges_and_missing_keys() {
    futures_executor::block_on(async {
        for prefix in ["", "libraries/owner/"] {
            let (_bucket, _log, hosted, _uploads) = setup(prefix);

            // Exact match and extensionless lookup (ignoring .page-1 files).
            let tiny = hosted.response("synth/tiny.txt").await.unwrap().unwrap();
            assert_eq!(tiny.key, format!("{prefix}media/synth/tiny.txt"));
            assert_eq!(tiny.size, 3);

            assert!(hosted.response("synth/d/pic").await.unwrap().is_none(), "page-1 preview is not treated as an original");
            let pic = hosted.response("synth/e/pic").await.unwrap().unwrap();
            assert_eq!(pic.key, format!("{prefix}media/synth/e/pic.png"));
            assert_eq!(pic.content_type, "image/png");

            // Traversal and missing keys return None.
            for bad in ["synth/missing.png", "/abs", "../x", "a/../b", "synth/./tiny.txt", "..", "."] {
                assert!(hosted.response(bad).await.unwrap().is_none(), "expected None for {bad}");
            }

            // Byte ranges on send.
            let full = hosted.send(&tiny, "GET", None).await.unwrap();
            assert_eq!(full.status, 200);
            assert_eq!(full.body.bytes().await.unwrap(), b"abc");

            let slice = hosted.send(&tiny, "GET", Some("bytes=1-2")).await.unwrap();
            assert_eq!(slice.status, 206);
            assert_eq!(slice.headers.get("content-range"), Some("bytes 1-2/3"));
            assert_eq!(slice.body.bytes().await.unwrap(), b"bc");

            let head = hosted.send(&tiny, "HEAD", None).await.unwrap();
            assert_eq!(head.status, 200);
            assert!(head.body.bytes().await.unwrap().is_empty());

            let out_of_range = hosted.send(&tiny, "GET", Some("bytes=99-")).await.unwrap();
            assert_eq!(out_of_range.status, 416);
        }
    });
}

#[test]
fn renditions_cache_and_processor_failures() {
    futures_executor::block_on(async {
        let prefix = "libraries/owner/";
        let (bucket, log, hosted, _uploads) = setup(prefix);

        let first = hosted.response("synth/e/pic;800x450.webp").await.unwrap().unwrap();
        assert_eq!(first.key, "libraries/owner/renditions/synth_e_pic;800x450.webp");
        assert_eq!(first.content_type, "image/webp");
        assert_eq!(log.borrow().len(), 1, "processor called once");

        // Second lookup serves cached rendition from bucket without calling processor.
        let second = hosted.response("synth/e/pic;800x450.webp").await.unwrap().unwrap();
        assert_eq!(second.key, first.key);
        assert_eq!(log.borrow().len(), 1, "cached rendition reused");
        assert!(bucket.objects.borrow().contains_key("libraries/owner/renditions/synth_e_pic;800x450.webp"));

        // Concurrent requests for the same uncached rendition coalesce into one processor call.
        log.borrow_mut().clear();
        let results = join_all((0..4).map(|_| hosted.response("synth/e/pic;100x100.png"))).await;
        for res in results {
            assert!(res.unwrap().is_some());
        }
        assert_eq!(log.borrow().len(), 1, "concurrent rendition coalesced");

        // Invalid rendition suffixes fail with 400.
        for bad in ["synth/e/pic;bad", "synth/e/pic;0x10.webp", "synth/e/pic;4097x10.webp", "synth/e/pic;10x10.gif"] {
            let err = hosted.response(bad).await.unwrap_err();
            assert!(err.to_string().contains("Invalid media rendition"), "unexpected error for {bad}: {err}");
        }

        // Processor returning None or error surfaces cleanly.
        assert!(hosted.response("synth/e/pic;null/10x10.webp").await.unwrap().is_none());
        let boom = hosted.response("synth/e/pic;boom/10x10.webp").await.unwrap_err();
        assert!(boom.to_string().contains("Media rendition unavailable"));
    });
}

#[test]
fn prepare_image_cover_and_uploads() {
    futures_executor::block_on(async {
        let prefix = "lib/x/";
        let (_bucket, _log, hosted, uploads) = setup(prefix);

        // Chunked uploads write, size, truncate, and prepare into permanent media.
        uploads.write("u1", 0, Body::Bytes(b"hello ".to_vec()), Some(6)).await.unwrap();
        uploads.write("u1", 6, Body::Bytes(b"world".to_vec()), Some(5)).await.unwrap();
        assert_eq!(uploads.size("u1").await.unwrap(), Some(11));
        assert_eq!(uploads.names().await.unwrap(), vec!["u1".to_owned()]);

        let prepared = hosted.prepare(uploads.input("u1"), "image/png", "joined.png", &user(), "card_joined").await.unwrap();
        let path = prepared.data.get("blob").get("path").as_str().unwrap().to_owned();
        let stored = hosted.response(&path).await.unwrap().unwrap();
        assert_eq!(stored.size, 11);

        let img = hosted.image(&obj! { "path" => path.as_str() }.into()).await.unwrap().unwrap();
        assert_eq!(img.mime, "image/png");

        let cover = HostedMedia::cover(&obj! { "path" => path.as_str(), "width" => 1600.0, "height" => 1000.0 }.into()).unwrap();
        assert_eq!(cover.get("width").as_f64(), Some(800.0));
        assert_eq!(cover.get("height").as_f64(), Some(500.0));

        uploads.truncate("u1", 6).await.unwrap();
        assert_eq!(uploads.size("u1").await.unwrap(), Some(6));
        uploads.remove("u1").await.unwrap();
        assert_eq!(uploads.size("u1").await.unwrap(), None);
    });
}
