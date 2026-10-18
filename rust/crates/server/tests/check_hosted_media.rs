//! Assertion-preserving Rust port of `tools/check-hosted-media.mjs`, using synthetic directory buckets only.

mod media_check_support;

use std::cell::Cell;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;

use async_trait::async_trait;
use futures_util::StreamExt;
use media_check_support::{
    Denied, Fixture, ask_hosted, body, collect, hosted, library, png, request, status,
};
use mm_core::host::{
    AnalysisImage, Bucket, ByteSpan, MediaInput, MediaProcessor, MediaUser, ObjectInfo,
    PreparedMedia, RenditionOutput, RenditionSource, UploadStorage,
};
use mm_core::hosted::{Hosted, HostedOptions};
use mm_core::hosted_media::{BucketUploads, HostedMedia};
use mm_core::http::{Body, Headers};
use mm_core::recorded::Recorded;
use mm_core::sql::Sql;
use mm_core::uploads::{UploadOptions, UploadRequest, Uploads};
use mm_core::{Result, Value, obj};
use mymind::bucket_fs::DirectoryBucket;
use mymind::media_processor::LocalProcessor;
use mymind::vips;

const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1";

// Return the original checks' synthetic media user.
fn user() -> MediaUser {
    MediaUser {
        account_uid: "account".into(),
        id_prefix: "prefix-".into(),
    }
}

// Count real rendition processing while retaining the production processor's behavior.
struct CountingProcessor {
    inner: LocalProcessor,
    renditions: Cell<usize>,
}

impl CountingProcessor {
    // Build a target-local processor with an observable rendition count.
    fn new(directory: &Path) -> Self {
        Self {
            inner: LocalProcessor::new(Some(directory.to_owned())),
            renditions: Cell::new(0),
        }
    }
}

#[async_trait(?Send)]
impl MediaProcessor for CountingProcessor {
    async fn prepare(
        &self,
        body: Body,
        content_type: &str,
        name: &str,
        user: &MediaUser,
        id: &str,
    ) -> Result<PreparedMedia> {
        self.inner.prepare(body, content_type, name, user, id).await
    }

    async fn rendition(
        &self,
        original: RenditionSource,
        relative: &str,
    ) -> Result<Option<RenditionOutput>> {
        self.renditions.set(self.renditions.get() + 1);
        self.inner.rendition(original, relative).await
    }

    async fn image(&self, body: Body) -> Result<AnalysisImage> {
        self.inner.image(body).await
    }
}

// Require bucket headers to stay outside listings and reject all tested escaping keys.
async fn directory_bucket(fixture: &Fixture) {
    let bucket = DirectoryBucket::new(fixture.directory("bucket-basic"));
    let headers = Headers::from([("content-type", "text/javascript"), ("x-raw", "1")]);
    bucket
        .write(
            "static/a.js",
            Body::Bytes(b"one".to_vec()),
            Some(3),
            Some(&headers),
        )
        .await
        .unwrap();
    bucket
        .write("static/b.js", Body::Bytes(b"two".to_vec()), None, None)
        .await
        .unwrap();
    assert_eq!(
        bucket.head("static/a.js").await.unwrap(),
        Some(ObjectInfo {
            size: 3,
            headers: Some(headers)
        })
    );
    assert_eq!(
        bucket.head("static/b.js").await.unwrap(),
        Some(ObjectInfo {
            size: 3,
            headers: None
        })
    );
    assert_eq!(
        bucket
            .list("")
            .await
            .unwrap()
            .iter()
            .map(|object| object.key.as_str())
            .collect::<Vec<_>>(),
        ["static/a.js", "static/b.js"]
    );
    let ranged = bucket
        .read(
            "static/a.js",
            Some(ByteSpan {
                offset: 1,
                length: 2,
            }),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(Body::Stream(ranged.body).bytes().await.unwrap(), b"ne");
    bucket
        .write("static/a.js", Body::Bytes(b"uno".to_vec()), None, None)
        .await
        .unwrap();
    assert!(
        bucket
            .head("static/a.js")
            .await
            .unwrap()
            .unwrap()
            .headers
            .is_none()
    );
    bucket
        .write(
            "static/a.js",
            Body::Bytes(b"one".to_vec()),
            Some(3),
            Some(&Headers::from([("content-type", "text/javascript")])),
        )
        .await
        .unwrap();
    bucket.remove("static/a.js").await.unwrap();
    assert!(bucket.head("static/a.js").await.unwrap().is_none());
    assert!(
        bucket
            .head(".headers/static/a.js.json")
            .await
            .unwrap()
            .is_none()
    );
    for key in ["../escape", "static/../../escape"] {
        status(
            bucket
                .write(key, Body::Bytes(b"x".to_vec()), None, None)
                .await,
            400,
        );
    }
    assert!(bucket.head("missing/file").await.unwrap().is_none());
    assert!(bucket.read("missing/file", None).await.unwrap().is_none());
}

// Originals, aliases and byte ranges must stay inside the requested library's namespace.
async fn originals_and_isolation(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-media")));
    let processor = Rc::new(CountingProcessor::new(&fixture.root));
    let media = HostedMedia::new(bucket.clone(), "libraries/one/", processor.clone());
    let bytes = png();
    let prepared = media
        .prepare(
            MediaInput::Bytes(bytes.clone()),
            "image/png",
            "image.png",
            &user(),
            "prefix-card",
        )
        .await
        .unwrap();
    let blob = prepared.data.get("blob");
    assert_eq!(blob.get("width").as_f64(), Some(48.0));
    assert_eq!(blob.get("height").as_f64(), Some(32.0));
    let path = blob.get("path").as_str().unwrap();
    let key = format!("libraries/one/media/{path}");
    assert!(bucket.head(&key).await.unwrap().is_some());
    assert_eq!(
        bucket
            .list("libraries/")
            .await
            .unwrap()
            .iter()
            .map(|object| object.key.clone())
            .collect::<Vec<_>>(),
        std::slice::from_ref(&key)
    );
    let original = media.response(path).await.unwrap().unwrap();
    assert_eq!(original.key, key);
    assert_eq!(original.size, bytes.len() as u64);
    assert_eq!(
        media
            .response(path.strip_suffix(".png").unwrap())
            .await
            .unwrap()
            .unwrap()
            .key,
        original.key
    );
    let whole = collect(media.send(&original, "GET", None).await.unwrap()).await;
    assert_eq!(whole.status, 200);
    assert_eq!(whole.bytes, bytes);
    let head = collect(media.send(&original, "HEAD", None).await.unwrap()).await;
    assert!(head.bytes.is_empty());
    assert_eq!(
        head.headers.get("content-length"),
        Some(bytes.len().to_string().as_str())
    );
    let slice = collect(
        media
            .send(&original, "GET", Some("bytes=2-8"))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(slice.status, 206);
    assert_eq!(slice.bytes, bytes[2..9]);
    assert_eq!(
        slice.headers.get("content-range"),
        Some(format!("bytes 2-8/{}", bytes.len()).as_str())
    );
    let outside = collect(
        media
            .send(&original, "GET", Some(&format!("bytes={}-", bytes.len())))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(outside.status, 416);
    assert!(outside.bytes.is_empty());
    assert!(
        HostedMedia::new(bucket, "libraries/two/", processor)
            .response(path)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        media
            .response(&format!("../one/media/{path}"))
            .await
            .unwrap()
            .is_none()
    );
}

// Stream ranges into a deliberately paused consumer and prove later chunks are pulled lazily.
async fn streaming_backpressure(fixture: &Fixture) {
    let dir = fixture.directory("bucket-stream");
    let bucket = Rc::new(DirectoryBucket::new(&dir));
    let media = HostedMedia::new(
        bucket.clone(),
        "",
        Rc::new(CountingProcessor::new(&fixture.root)),
    );
    bucket
        .write(
            "media/clip.mp4",
            Body::Bytes(b"0123456789".to_vec()),
            None,
            None,
        )
        .await
        .unwrap();
    let original = media.response("clip.mp4").await.unwrap().unwrap();
    let response = media
        .send(&original, "GET", Some("bytes=1-8"))
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.headers.get("content-length"), Some("8"));
    assert!(matches!(&response.body, Body::Stream(_)));
    let mut stream = response.body.into_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
        tokio::task::yield_now().await;
    }
    assert_eq!(bytes, b"12345678");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(dir.join("media/clip.mp4"))
        .unwrap();
    file.set_len(512 * 1024).unwrap();
    let original = media.response("clip.mp4").await.unwrap().unwrap();
    let response = media.send(&original, "GET", None).await.unwrap();
    let mut stream = response.body.into_stream();
    let first = stream.next().await.unwrap().unwrap();
    assert!(!first.is_empty() && first.len() < original.size as usize);
    tokio::task::yield_now().await;
    file.seek(SeekFrom::Start(400 * 1024)).unwrap();
    file.write_all(b"lazy-read-proof").unwrap();
    file.flush().unwrap();
    let mut observed = first;
    while let Some(chunk) = stream.next().await {
        observed.extend_from_slice(&chunk.unwrap());
        tokio::task::yield_now().await;
    }
    assert_eq!(&observed[400 * 1024..400 * 1024 + 15], b"lazy-read-proof");
}

// Coalesce concurrent renditions, retain exact image dimensions, and reuse each cached object.
async fn renditions(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-renditions")));
    let processor = Rc::new(CountingProcessor::new(&fixture.root));
    let media = HostedMedia::new(bucket.clone(), "libraries/one/", processor.clone());
    let prepared = media
        .prepare(
            MediaInput::Bytes(png()),
            "image/png",
            "image.png",
            &user(),
            "prefix-card",
        )
        .await
        .unwrap();
    let blob = prepared.data.get("blob");
    let path = blob.get("path").as_str().unwrap();
    for suffix in [";12x12-c.webp", ";12x12-t.jpg"] {
        let relative = format!("{path}{suffix}");
        let (first, second) =
            futures_util::join!(media.response(&relative), media.response(&relative));
        let first = first.unwrap().unwrap();
        assert_eq!(Some(first.clone()), second.unwrap());
        assert!(first.key.starts_with("libraries/one/renditions/"));
        let bytes = Body::Stream(bucket.read(&first.key, None).await.unwrap().unwrap().body)
            .bytes()
            .await
            .unwrap();
        let info = vips::metadata(&bytes, 0).unwrap();
        assert_eq!(info.width, 12);
        assert_eq!(info.height, if suffix.contains("-c") { 12 } else { 8 });
        assert_eq!(media.response(&relative).await.unwrap(), Some(first));
    }
    assert_eq!(processor.renditions.get(), 2);
    assert_eq!(
        vips::metadata(&media.image(blob).await.unwrap().unwrap().data, 0)
            .unwrap()
            .format,
        "png"
    );
    status(media.response(&format!("{path};bogus")).await, 400);
}

// Keep PDF and video preview processing on the real hosted processor path.
async fn previews(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-previews")));
    let media = HostedMedia::new(
        bucket.clone(),
        "",
        Rc::new(CountingProcessor::new(&fixture.root)),
    );
    let doc = media
        .prepare(
            MediaInput::Bytes(b"%PDF-1.5 synthetic".to_vec()),
            "application/pdf",
            "doc.pdf",
            &user(),
            "prefix-document",
        )
        .await
        .unwrap();
    let video = media
        .prepare(
            MediaInput::Bytes(b"0000ftypisom".to_vec()),
            "video/mp4",
            "video.mp4",
            &user(),
            "prefix-video",
        )
        .await
        .unwrap();
    assert_eq!(doc.data.get("type").as_str(), Some("Document"));
    assert_eq!(video.data.get("type").as_str(), Some("Video"));
    assert_eq!(video.data.get("blob").get("duration").as_f64(), Some(1.0));
    let page = media
        .response(&format!(
            "{};background(fff)/page(1)/20x20-t.webp",
            doc.data.get("blob").get("path").as_str().unwrap()
        ))
        .await
        .unwrap()
        .unwrap();
    let bytes = Body::Stream(bucket.read(&page.key, None).await.unwrap().unwrap().body)
        .bytes()
        .await
        .unwrap();
    assert_eq!(vips::metadata(&bytes, 0).unwrap().width, 20);
}

// Bucket chunks must roll back individually, sweep orphans, and concatenate the committed original exactly.
async fn bucket_uploads(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-uploads")));
    let (_, store) = library(&fixture.root.join("uploads.db"));
    let storage = Rc::new(BucketUploads::new(bucket.clone(), "libraries/one/"));
    let media = Rc::new(HostedMedia::new(
        bucket.clone(),
        "libraries/one/",
        Rc::new(CountingProcessor::new(&fixture.root)),
    ));
    let uploads = Uploads::new(
        store,
        storage.clone(),
        media.clone(),
        UploadOptions::default(),
    )
    .unwrap();
    let auth = uploads.authorize(&Value::Null).await.unwrap();
    let id = auth
        .get("url")
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let bytes = png();
    let put = |bytes: Vec<u8>, range: String| UploadRequest {
        body: Body::Stream(Box::pin(futures_util::stream::iter([Ok(bytes)]))),
        headers: Headers::from_pairs([
            (
                "authorization",
                format!("Bearer {}", auth.get("token").as_str().unwrap()),
            ),
            ("content-range", range),
            ("content-type", "image/png".into()),
            ("x-file-name", "fixture.png".into()),
        ]),
    };
    uploads
        .put(
            id,
            put(bytes[..10].to_vec(), format!("bytes 0-9/{}", bytes.len())),
        )
        .await
        .unwrap();
    status(
        uploads
            .put(
                id,
                put(
                    bytes[10..12].to_vec(),
                    format!("bytes 10-{}/{}", bytes.len() - 1, bytes.len()),
                ),
            )
            .await,
        400,
    );
    assert_eq!(storage.size(id).await.unwrap(), Some(10));
    uploads
        .put(
            id,
            put(
                bytes[10..].to_vec(),
                format!("bytes 10-{}/{}", bytes.len() - 1, bytes.len()),
            ),
        )
        .await
        .unwrap();
    assert_eq!(storage.size(id).await.unwrap(), Some(bytes.len() as u64));
    assert_eq!(
        bucket
            .list(&format!("libraries/one/uploads/{id}/"))
            .await
            .unwrap()
            .len(),
        2
    );
    let orphan = format!("libraries/one/uploads/{}/", "a".repeat(22));
    bucket
        .write(
            &format!("{orphan}000000000000"),
            Body::Bytes(b"orphan".to_vec()),
            None,
            None,
        )
        .await
        .unwrap();
    uploads.sweep().await.unwrap();
    assert!(bucket.list(&orphan).await.unwrap().is_empty());
    let prepared = uploads.prepare(id).await.unwrap();
    let original = media
        .response(
            prepared
                .get("data")
                .get("blob")
                .get("path")
                .as_str()
                .unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Body::Stream(
            bucket
                .read(&original.key, None)
                .await
                .unwrap()
                .unwrap()
                .body
        )
        .bytes()
        .await
        .unwrap(),
        bytes
    );
}

// Recordings keep the first copy and resolve device before verification-run variants.
fn recordings(fixture: &Fixture) {
    let (db, _) = library(&fixture.root.join("recorded.db"));
    let recorded = Recorded::new(db).unwrap();
    let key = "GET https://access.mymind.com/thing";
    recorded
        .put(
            key,
            200,
            &obj! { "content-type" => "text/plain" }.into(),
            b"plain",
        )
        .unwrap();
    recorded.put(key, 200, &obj! {}.into(), b"second").unwrap();
    recorded
        .put(&format!("iphone|{key}"), 200, &obj! {}.into(), b"phone")
        .unwrap();
    recorded
        .put(
            &format!("run:r1|iphone|{key}"),
            200,
            &obj! {}.into(),
            b"phone run",
        )
        .unwrap();
    recorded
        .put(&format!("run:r1|{key}"), 200, &obj! {}.into(), b"run")
        .unwrap();
    for (headers, expected) in [
        (vec![], "plain"),
        (vec![("user-agent", IPHONE)], "phone"),
        (
            vec![("user-agent", IPHONE), ("x-replay-run", "r1")],
            "phone run",
        ),
        (vec![("x-replay-run", "r1")], "run"),
        (vec![("x-replay-run", "r2")], "plain"),
    ] {
        assert_eq!(
            recorded
                .entry(&request("GET", "/thing", Body::Empty, &headers), key)
                .unwrap()
                .unwrap()
                .body,
            expected.as_bytes()
        );
    }
    assert!(
        recorded
            .entry(
                &request("GET", "/other", Body::Empty, &[]),
                "GET https://access.mymind.com/other"
            )
            .unwrap()
            .is_none()
    );
    assert_eq!(recorded.count().unwrap(), 4);
}

// Missing capture prefixes are unrepresentable: Hosted derives its private capture field from a required valid library.
async fn capture_prefix_is_required_by_construction(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-prefix")));
    let (db, _) = library(&fixture.root.join("prefix.db"));
    for name in ["", "../two", "one/capture"] {
        let result = Hosted::new(HostedOptions {
            db: db.clone(),
            bucket: bucket.clone(),
            library: name.to_owned(),
            processor: Rc::new(LocalProcessor::new(Some(fixture.root.clone()))),
            push: Box::new(|_, _| {}),
            readonly: false,
            offline: true,
            screenshot: Rc::new(Denied),
            resource: Rc::new(Denied),
            fetch: Rc::new(Denied),
            random: mm_core::runtime::random_bytes,
        })
        .await;
        let error = result
            .err()
            .expect("a library without a valid capture namespace cannot be constructed");
        assert!(
            error
                .to_string()
                .contains("Library names are lower-case letters, digits and dashes"),
            "check-hosted-media.mjs:229 missing capture prefix is prevented at Hosted construction: {name:?}"
        );
    }
    assert!(bucket.list("").await.unwrap().is_empty());
}

// Preserve recording precedence, desktop and phone shells, fallbacks, ranges and library boundaries.
async fn site_fallbacks(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-site")));
    let (db, _) = library(&fixture.root.join("site.db"));
    db.execute(
        "UPDATE users SET settings = ?",
        &[r#"{"onboardFlags":128}"#.into()],
    )
    .unwrap();
    let recorded = Recorded::new(db.clone()).unwrap();
    recorded
        .put(
            "GET https://access.mymind.com/news/unread",
            200,
            &obj! { "content-type" => "application/json" }.into(),
            b"[1]",
        )
        .unwrap();
    // The real API's observable desktop shell mutation replaces the JS harness's `[desktop]` marker.
    let html = r#"<p>everything</p><mymind-top-of-mind class="tom ">"#;
    let desktop = r#"<p>everything</p><mymind-top-of-mind class="tom introduced">"#;
    recorded
        .put(
            "DOC https://access.mymind.com/everything",
            200,
            &obj! { "content-type" => "text/html" }.into(),
            html.as_bytes(),
        )
        .unwrap();
    recorded
        .put(
            "shell",
            200,
            &obj! { "content-type" => "text/html" }.into(),
            b"<p>shell</p>",
        )
        .unwrap();
    bucket
        .write(
            "static/app.js",
            Body::Bytes(b"code".to_vec()),
            Some(4),
            Some(&Headers::from([
                ("content-type", "text/javascript"),
                ("x-raw", "1"),
            ])),
        )
        .await
        .unwrap();
    for (key, bytes) in [
        ("static/plain.css", &b"css"[..]),
        ("libraries/one/media/a/b.webp", &b"wide"[..]),
        ("libraries/one/renditions/a_b;50x50.webp", &b"wide"[..]),
        ("libraries/one/renditions/a_b;10x10.webp", &b"small"[..]),
        ("libraries/one/media/a/clip.mp4", &b"0123456789"[..]),
        ("libraries/two/media/a/other.webp", &b"other library"[..]),
        ("media/a/shared.webp", &b"shared"[..]),
    ] {
        bucket
            .write(key, Body::Bytes(bytes.to_vec()), None, None)
            .await
            .unwrap();
    }
    let site = hosted(db, bucket, &fixture.root, "one").await;
    assert_eq!(
        ask_hosted(&site, "GET", "/news/unread", Body::Empty, &[])
            .await
            .text(),
        "[1]"
    );
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/everything",
            Body::Empty,
            &[("sec-fetch-dest", "document")]
        )
        .await
        .text(),
        desktop
    );
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/everything",
            Body::Empty,
            &[("sec-fetch-dest", "document"), ("user-agent", IPHONE)]
        )
        .await
        .text(),
        html
    );
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/anything/else",
            Body::Empty,
            &[("sec-fetch-dest", "document")]
        )
        .await
        .text(),
        desktop
    );
    let code = ask_hosted(&site, "GET", "/_static/app.js", Body::Empty, &[]).await;
    assert_eq!(code.text(), "code");
    assert_eq!(code.headers.get("content-type"), Some("text/javascript"));
    assert_eq!(code.headers.get("x-raw"), Some("1"));
    assert_eq!(
        ask_hosted(&site, "GET", "/_static/plain.css", Body::Empty, &[])
            .await
            .headers
            .get("content-type"),
        Some("text/css; charset=utf-8")
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/_static/missing.js", Body::Empty, &[])
            .await
            .status,
        404
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/_static/..%2F..%2Fetc", Body::Empty, &[])
            .await
            .status,
        400
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/media/a/b;50x50.webp", Body::Empty, &[])
            .await
            .text(),
        "wide"
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/media/a/b;10x10.webp", Body::Empty, &[])
            .await
            .text(),
        "small"
    );
    let range = ask_hosted(
        &site,
        "GET",
        "/media/a/clip.mp4",
        Body::Empty,
        &[("range", "bytes=2-5")],
    )
    .await;
    assert_eq!(range.status, 206);
    assert_eq!(range.text(), "2345");
    assert_eq!(range.headers.get("content-range"), Some("bytes 2-5/10"));
    let past = ask_hosted(
        &site,
        "GET",
        "/media/a/clip.mp4",
        Body::Empty,
        &[("range", "bytes=99-")],
    )
    .await;
    assert_eq!(past.status, 416);
    assert_eq!(past.headers.get("content-range"), Some("bytes */10"));
    assert!(past.bytes.is_empty());
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/media/a/clip.mp4",
            Body::Empty,
            &[("range", "bytes=5-3")]
        )
        .await
        .status,
        416
    );
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/media/a/clip.mp4",
            Body::Empty,
            &[("range", "bytes=0-1,4-5")]
        )
        .await
        .status,
        416
    );
    let head = ask_hosted(&site, "HEAD", "/media/a/clip.mp4", Body::Empty, &[]).await;
    assert_eq!(head.status, 200);
    assert_eq!(head.headers.get("content-length"), Some("10"));
    assert!(head.bytes.is_empty());
    for path in [
        "/media/a/other.webp",
        "/media/a/shared.webp",
        "/media/..%2F..%2Ftwo%2Fmedia%2Fa%2Fother.webp",
    ] {
        assert_eq!(
            ask_hosted(&site, "GET", path, Body::Empty, &[])
                .await
                .status,
            404
        );
    }
    let recorded_range = ask_hosted(
        &site,
        "GET",
        "/news/unread",
        Body::Empty,
        &[("range", "bytes=9-")],
    )
    .await;
    assert_eq!(recorded_range.status, 416);
    assert_eq!(
        recorded_range.headers.get("content-range"),
        Some("bytes */3")
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/media/none/x.webp", Body::Empty, &[])
            .await
            .status,
        404
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/media/none/x.mp4", Body::Empty, &[])
            .await
            .status,
        404
    );
    assert!(
        ask_hosted(&site, "GET", "/cards/abc/detail", Body::Empty, &[])
            .await
            .text()
            .contains("Detail view not captured for abc.")
    );
    assert_eq!(
        ask_hosted(&site, "POST", "/unknown", body(obj! {}.into()), &[])
            .await
            .text(),
        "{}"
    );
    assert_eq!(
        ask_hosted(
            &site,
            "GET",
            "/unknown",
            Body::Empty,
            &[("accept", "application/json")]
        )
        .await
        .status,
        200
    );
    assert_eq!(
        ask_hosted(&site, "GET", "/unknown", Body::Empty, &[])
            .await
            .status,
        404
    );
    site.api().jobs().drain().await;
    site.close();
}

// Perform the actual hosted authorize and PUT flow and require a successful chunk response.
async fn upload(site: &mm_core::hosted::Hosted, bytes: &[u8]) -> String {
    let auth = ask_hosted(site, "POST", "/uploads/authorize", Body::Empty, &[])
        .await
        .value();
    let token = format!("Bearer {}", auth.get("token").as_str().unwrap());
    let range = format!("bytes 0-{}/{}", bytes.len() - 1, bytes.len());
    let result = ask_hosted(
        site,
        "PUT",
        auth.get("url").as_str().unwrap(),
        Body::Bytes(bytes.to_vec()),
        &[
            ("content-type", "image/png"),
            ("authorization", &token),
            ("content-range", &range),
        ],
    )
    .await;
    assert_eq!(result.status, 200);
    auth.get("url")
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .to_owned()
}

// A full hosted library must claim uploads and serve originals, ranges, renditions and both attachment sources.
async fn hosted_uploads_and_attachments(fixture: &Fixture) {
    let bucket = Rc::new(DirectoryBucket::new(fixture.directory("bucket-library")));
    let (db, _) = library(&fixture.root.join("library.db"));
    let site = hosted(db, bucket.clone(), &fixture.root, "one").await;
    let bytes = png();
    let upload_id = upload(&site, &bytes).await;
    let saved = ask_hosted(
        &site,
        "POST",
        "/objects",
        body(obj! { "type" => "Media", "uploadId" => upload_id.as_str() }.into()),
        &[],
    )
    .await;
    assert_eq!(saved.status, 201);
    assert!(
        bucket
            .list(&format!("libraries/one/uploads/{upload_id}/"))
            .await
            .unwrap()
            .is_empty()
    );
    let card = ask_hosted(
        &site,
        "GET",
        &format!("/objects/{}", saved.value().get("id").as_str().unwrap()),
        Body::Empty,
        &[],
    )
    .await
    .value();
    let path = card.get("blob").get("path").as_str().unwrap();
    assert_eq!(
        ask_hosted(&site, "GET", &format!("/media/{path}"), Body::Empty, &[])
            .await
            .bytes,
        bytes
    );
    let head = ask_hosted(&site, "HEAD", &format!("/media/{path}"), Body::Empty, &[]).await;
    assert!(head.bytes.is_empty());
    assert_eq!(
        head.headers.get("content-length"),
        Some(bytes.len().to_string().as_str())
    );
    let alias = ask_hosted(
        &site,
        "GET",
        &format!("/media/{path}"),
        Body::Empty,
        &[("range", "bytes=2-8")],
    )
    .await;
    assert_eq!(alias.status, 206);
    assert_eq!(alias.bytes, bytes[2..9]);
    let thumbnail = ask_hosted(
        &site,
        "GET",
        &format!("/media/{path};12x12-c.webp"),
        Body::Empty,
        &[],
    )
    .await;
    assert_eq!(vips::metadata(&thumbnail.bytes, 0).unwrap().width, 12);
    let note = ask_hosted(
        &site,
        "POST",
        "/objects",
        body(obj! { "content" => "Synthetic note" }.into()),
        &[],
    )
    .await
    .value();
    let note = note.get("id").as_str().unwrap();
    for id in [upload_id, upload(&site, &bytes).await] {
        let attached = ask_hosted(
            &site,
            "POST",
            &format!("/objects/{note}/attachments"),
            body(obj! { "uploadId" => id }.into()),
            &[],
        )
        .await;
        assert_eq!(attached.status, 200);
        assert_eq!(
            ask_hosted(
                &site,
                "GET",
                attached.value().get("path").as_str().unwrap(),
                Body::Empty,
                &[]
            )
            .await
            .bytes,
            bytes
        );
    }
    let keys = bucket.list("").await.unwrap();
    assert!(
        keys.iter()
            .all(|object| object.key.starts_with("libraries/one/")),
        "every object stays in the library"
    );
    assert!(
        keys.iter()
            .any(|object| object.key.starts_with("libraries/one/renditions/"))
    );
    assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("mymind-media-")
    }));
    site.api().jobs().drain().await;
    site.close();
}

// Run the original hosted scenarios serially within one isolated synthetic environment.
#[test]
fn check_hosted_media() {
    let fixture = Fixture::new("check-hosted-media");
    mymind::runtime::run(async {
        directory_bucket(&fixture).await;
        originals_and_isolation(&fixture).await;
        streaming_backpressure(&fixture).await;
        renditions(&fixture).await;
        previews(&fixture).await;
        bucket_uploads(&fixture).await;
        recordings(&fixture);
        capture_prefix_is_required_by_construction(&fixture).await;
        site_fallbacks(&fixture).await;
        hosted_uploads_and_attachments(&fixture).await;
    });
}
