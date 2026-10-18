//! Assertion-preserving Rust port of `tools/check-media.mjs`, using synthetic inputs and no sockets or network.

mod media_check_support;

use std::cell::Cell;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;

use futures_util::StreamExt;
use media_check_support::{
    Counted, Fixture, api, ask_api, body, collect, library, marker, png, status,
};
use mm_core::host::{Media, MediaInput, MediaUser};
use mm_core::http::{Body, Headers};
use mm_core::runtime;
use mm_core::sql::{Sql, SqlExt};
use mm_core::uploads::{UploadOptions, UploadRequest, Uploads};
use mm_core::{Value, obj};
use mymind::local_media::LocalMedia;
use mymind::uploads_fs::DirectoryUploads;
use mymind::vips;

// Return the original checks' synthetic media user.
fn user() -> MediaUser {
    MediaUser {
        account_uid: "account".into(),
        id_prefix: "prefix-".into(),
    }
}

// Extract a minted upload capability's id.
fn upload_id(auth: &Value) -> &str {
    auth.get("url")
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
}

// Construct the original chunk headers around an independently controlled body.
fn chunk(auth: &Value, bytes: Body, range: &str) -> UploadRequest {
    UploadRequest {
        headers: Headers::from_pairs([
            (
                "authorization",
                format!("Bearer {}", auth.get("token").as_str().unwrap()),
            ),
            ("content-range", range.to_owned()),
            ("content-type", "image/png".to_owned()),
            ("x-file-name", "fixture.png".to_owned()),
        ]),
        body: bytes,
    }
}

// Observe the kernel's file position without consuming bytes from the media file.
fn media_position(path: &Path) -> u64 {
    let path = std::fs::canonicalize(path).unwrap();
    let descriptors = std::fs::read_dir("/proc/self/fd").unwrap();
    let matches = descriptors
        .filter_map(|entry| {
            let entry = entry.unwrap();
            (std::fs::read_link(entry.path()).ok().as_ref() == Some(&path))
                .then_some(entry.file_name())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "the response retains exactly one open file"
    );
    let info = std::fs::read_to_string(Path::new("/proc/self/fdinfo").join(&matches[0])).unwrap();
    info.lines()
        .find_map(|line| line.strip_prefix("pos:\t"))
        .unwrap()
        .parse()
        .unwrap()
}

// Read process filesystem byte accounting to reject whole-original reads before or during streaming.
fn read_chars() -> u64 {
    std::fs::read_to_string("/proc/self/io")
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("rchar: "))
        .unwrap()
        .parse()
        .unwrap()
}

// Require the actual file reader to start at the range offset and stop at its inclusive end.
async fn bounded_range(
    media: &LocalMedia,
    descriptor: &mm_core::host::MediaResult,
    range: &str,
    start: u64,
    end: u64,
) -> media_check_support::Answer {
    let before = read_chars();
    let response = media.send(descriptor, "GET", Some(range)).await.unwrap();
    assert_eq!(media_position(Path::new(&descriptor.key)), start);
    let status = response.status;
    let headers = response.headers;
    let mut stream = response.body.into_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
        assert_eq!(
            media_position(Path::new(&descriptor.key)),
            start + bytes.len() as u64
        );
        assert!(start + bytes.len() as u64 <= end + 1);
    }
    assert_eq!(bytes.len() as u64, end - start + 1);
    assert!(
        read_chars() - before < 16 * 1024,
        "a tiny range cannot read the 512 MiB original"
    );
    media_check_support::Answer {
        status,
        headers,
        bytes,
    }
}

// Prove sparse originals stay file-backed and every valid or invalid range is bounded.
async fn sparse_originals_and_backpressure(fixture: &Fixture) {
    let dir = fixture.directory("sparse");
    let path = dir.join("original.mp4");
    let mut file = std::fs::File::create(&path).unwrap();
    let length = 512 * 1024 * 1024u64;
    file.set_len(length).unwrap();
    file.write_all(b"0123456789").unwrap();
    drop(file);
    let setup_reads = read_chars();
    let media = LocalMedia::new(&dir);
    let descriptor = media.response("original.mp4").await.unwrap().unwrap();
    assert_eq!(descriptor.size, length);
    assert_eq!(Path::new(&descriptor.key), path);
    let head = media.send(&descriptor, "HEAD", None).await.unwrap();
    assert!(matches!(&head.body, Body::Empty));
    let head = collect(head).await;
    assert!(head.bytes.is_empty());
    assert_eq!(
        head.headers.get("content-length"),
        Some(length.to_string().as_str())
    );
    assert!(
        read_chars() - setup_reads < 16 * 1024,
        "descriptor lookup and HEAD cannot read the 512 MiB original"
    );
    let slice = bounded_range(&media, &descriptor, "bytes=2-8", 2, 8).await;
    assert_eq!(slice.status, 206);
    assert_eq!(slice.bytes, b"2345678");
    assert_eq!(
        slice.headers.get("content-range"),
        Some(format!("bytes 2-8/{length}").as_str())
    );
    let tail = bounded_range(&media, &descriptor, "bytes=-3", length - 3, length - 1).await;
    assert_eq!(tail.bytes, [0, 0, 0]);
    assert_eq!(
        tail.headers.get("content-range"),
        Some(format!("bytes {}-{}/{length}", length - 3, length - 1).as_str())
    );
    // Removing the file makes an accidental open on invalid ranges or HEAD fail deterministically.
    std::fs::remove_file(&path).unwrap();
    for range in [
        "bytes=-".to_owned(),
        "bytes=-0".to_owned(),
        format!("bytes={length}-"),
        "bytes=8-2".into(),
        "bytes=0-9007199254740992".into(),
        "bytes=0-1,3-4".into(),
    ] {
        let response = media.send(&descriptor, "GET", Some(&range)).await.unwrap();
        assert!(matches!(&response.body, Body::Empty));
        let result = collect(response).await;
        assert_eq!(result.status, 416, "{range}");
        assert!(result.bytes.is_empty(), "{range}");
    }
    let head = media
        .send(&descriptor, "HEAD", Some("bytes=1-3"))
        .await
        .unwrap();
    assert!(matches!(head.body, Body::Empty));
    std::fs::write(&path, b"0123456789").unwrap();
    let descriptor = media.response("original.mp4").await.unwrap().unwrap();
    let response = media
        .send(&descriptor, "GET", Some("bytes=1-8"))
        .await
        .unwrap();
    assert_eq!(response.status, 206);
    assert_eq!(response.headers.get("content-length"), Some("8"));
    let mut stream = response.body.into_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
        tokio::task::yield_now().await;
    }
    assert_eq!(bytes, b"12345678");
    // A paused consumer must observe later disk writes beyond the first bounded read.
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    file.set_len(512 * 1024).unwrap();
    let descriptor = media.response("original.mp4").await.unwrap().unwrap();
    let response = media.send(&descriptor, "GET", None).await.unwrap();
    let mut stream = response.body.into_stream();
    let first = stream.next().await.unwrap().unwrap();
    assert!(!first.is_empty() && first.len() < descriptor.size as usize);
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

// Preserve file-input preparation, extensionless source reuse and all three cached image renditions.
async fn file_backed_images(fixture: &Fixture) {
    let dir = fixture.directory("images");
    let bytes = png();
    let input = dir.join("input.png");
    std::fs::write(&input, &bytes).unwrap();
    let media = LocalMedia::new(&dir);
    let prepared = media
        .prepare(
            MediaInput::File(input.to_string_lossy().into_owned()),
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
    let original = media.response(path).await.unwrap().unwrap();
    assert_eq!(
        collect(media.send(&original, "GET", None).await.unwrap())
            .await
            .bytes,
        bytes
    );
    assert_eq!(
        media
            .response(path.strip_suffix(".png").unwrap())
            .await
            .unwrap()
            .unwrap()
            .key,
        original.key
    );
    for suffix in [";12x12-c.webp", ";12x12-t.jpg", ";12x12.png"] {
        let result = media
            .response(&format!("{path}{suffix}"))
            .await
            .unwrap()
            .unwrap();
        assert!(Path::new(&result.key).is_file());
        let info = vips::metadata(&std::fs::read(&result.key).unwrap(), 0).unwrap();
        assert_eq!(info.width, 12);
        assert_eq!(info.height, if suffix.contains("-c") { 12 } else { 8 });
        let cached = media
            .response(&format!("{path}{suffix}"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cached.key, result.key);
        assert_eq!(cached.size, result.size);
        let head = collect(media.send(&cached, "HEAD", None).await.unwrap()).await;
        assert!(head.bytes.is_empty());
        assert_eq!(
            head.headers.get("content-length"),
            Some(result.size.to_string().as_str())
        );
    }
    assert_eq!(
        vips::metadata(&media.image(blob).await.unwrap().unwrap().data, 0)
            .unwrap()
            .format,
        "png"
    );
}

// Gate failed encodes to prove coalescing, atomic publication, cleanup, unique retries and cache reuse.
async fn video_cache(fixture: &Fixture) {
    fixture.clear_commands();
    let dir = fixture.directory("video");
    std::fs::write(dir.join("original.mp4"), b"synthetic original").unwrap();
    std::fs::write(fixture.root.join("fail"), b"").unwrap();
    let media = LocalMedia::new(&dir);
    let relative = "original.mp4;64x64/hevc.mp4";
    let cache = dir.join(".renditions").join(relative.replace('/', "_"));
    let first = media.response(relative);
    let concurrent = async {
        marker(&fixture.root.join("started")).await;
        assert!(!cache.exists());
        let commands = fixture.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0][0], "ffmpeg");
        let output = commands[0].last().unwrap();
        assert!(output.starts_with(&format!("{}.", cache.display())));
        assert!(output.ends_with(".tmp"));
        let second = media.response(relative);
        tokio::pin!(second);
        assert!(futures_util::poll!(&mut second).is_pending());
        std::fs::write(fixture.root.join("release"), b"").unwrap();
        status(second.await, 422);
    };
    let (first, ()) = futures_util::join!(first, concurrent);
    status(first, 422);
    assert_eq!(fixture.commands().len(), 1);
    assert_eq!(
        std::fs::read_dir(cache.parent().unwrap()).unwrap().count(),
        0
    );
    std::fs::remove_file(fixture.root.join("fail")).unwrap();
    let result = media.response(relative).await.unwrap().unwrap();
    assert_eq!(
        std::fs::read(&result.key).unwrap(),
        b"complete encoded video"
    );
    let commands = fixture.commands();
    assert_eq!(commands.len(), 2);
    assert_ne!(commands[0].last(), commands[1].last());
    media.response(relative).await.unwrap().unwrap();
    assert_eq!(fixture.commands().len(), 2);
    assert_eq!(
        collect(media.send(&result, "GET", Some("bytes=0-7")).await.unwrap())
            .await
            .bytes,
        b"complete"
    );
}

// Exercise PDF and video preparation through real processors supplied with file-path command arguments.
async fn previews(fixture: &Fixture) {
    fixture.clear_commands();
    let dir = fixture.directory("previews");
    let media = LocalMedia::new(&dir);
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
    let commands = fixture.commands();
    assert_eq!(
        commands
            .iter()
            .map(|command| command[0].as_str())
            .collect::<Vec<_>>(),
        ["pdftoppm", "ffprobe", "ffmpeg"]
    );
    let ffmpeg = &commands[2];
    let input = &ffmpeg[ffmpeg.iter().position(|argument| argument == "-i").unwrap() + 1];
    assert!(Path::new(input).is_file());
    let relative = format!(
        "{};background(fff)/page(1)/20x20-t.webp",
        doc.data.get("blob").get("path").as_str().unwrap()
    );
    let result = media.response(&relative).await.unwrap().unwrap();
    assert_eq!(
        vips::metadata(&std::fs::read(&result.key).unwrap(), 0)
            .unwrap()
            .width,
        20
    );
    for entry in std::fs::read_dir(&dir).unwrap() {
        assert!(
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        );
    }
}

// Expiry must remove rows and temporary bytes, while interrupted uncommitted bytes are truncated.
async fn expiry_and_orphans(fixture: &Fixture) {
    let (db, store) = library(&fixture.root.join("sweep.db"));
    let directory = fixture.directory("uploads-sweep");
    let uploads = Uploads::new(
        store,
        Rc::new(DirectoryUploads::new(&directory)),
        Rc::new(LocalMedia::new(fixture.directory("sweep-media"))),
        UploadOptions::default(),
    )
    .unwrap();
    let auth = uploads.authorize(&Value::Null).await.unwrap();
    let id = upload_id(&auth);
    uploads
        .put(
            id,
            chunk(&auth, Body::Bytes(b"abc".to_vec()), "bytes 0-2/3"),
        )
        .await
        .unwrap();
    let orphan = "a".repeat(22);
    std::fs::write(directory.join(&orphan), b"orphan").unwrap();
    db.execute(
        "UPDATE uploads SET expires_ms = 0 WHERE id = ?",
        &[id.into()],
    )
    .unwrap();
    uploads.authorize(&Value::Null).await.unwrap();
    assert!(
        db.one("SELECT id FROM uploads WHERE id = ?", &[id.into()])
            .unwrap()
            .is_none()
    );
    assert!(!directory.join(id).exists());
    assert!(!directory.join(&orphan).exists());
    status(uploads.prepare(id).await, 404);
    let partial = uploads.authorize(&Value::Null).await.unwrap();
    let partial_id = upload_id(&partial);
    std::fs::write(
        directory.join(partial_id),
        b"uncommitted bytes from an interrupted chunk",
    )
    .unwrap();
    uploads.sweep().await.unwrap();
    assert_eq!(
        std::fs::metadata(directory.join(partial_id)).unwrap().len(),
        0
    );
}

// Preserve aggregate quota reservations during active streams and reject blocked chunks before polling them.
async fn aggregate_reservations(fixture: &Fixture) {
    let (db, store) = library(&fixture.root.join("budget.db"));
    let directory = fixture.directory("uploads-budget");
    let uploads = Uploads::new(
        store,
        Rc::new(DirectoryUploads::new(&directory)),
        Rc::new(LocalMedia::new(fixture.directory("budget-media"))),
        UploadOptions {
            max_bytes: 10,
            ..UploadOptions::default()
        },
    )
    .unwrap();
    let auths = uploads.authorize(&Value::from("3")).await.unwrap();
    let auths = auths.as_array().unwrap();
    let (a, b, c) = (&auths[0], &auths[1], &auths[2]);
    let (started, wait_started) = tokio::sync::oneshot::channel();
    let (release, gate) = tokio::sync::oneshot::channel();
    let slow = Body::Stream(Box::pin(futures_util::stream::once(async move {
        started.send(()).unwrap();
        gate.await.unwrap();
        Ok(b"123456".to_vec())
    })));
    let running = uploads.put(upload_id(a), chunk(a, slow, "bytes 0-5/6"));
    let control = async {
        wait_started.await.unwrap();
        db.execute(
            "UPDATE uploads SET expires_ms = 0 WHERE id = ?",
            &[upload_id(a).into()],
        )
        .unwrap();
        uploads.sweep().await.unwrap();
        assert!(
            db.one(
                "SELECT id FROM uploads WHERE id = ?",
                &[upload_id(a).into()]
            )
            .unwrap()
            .is_some()
        );
        db.execute(
            "UPDATE uploads SET expires_ms = ? WHERE id = ?",
            &[(runtime::now_ms() + 60_000.0).into(), upload_id(a).into()],
        )
        .unwrap();
        let polls = Rc::new(Cell::new(0));
        let blocked = Body::Stream(Box::pin(Counted {
            polls: polls.clone(),
            body: Some(b"abcdef".to_vec()),
        }));
        status(
            uploads
                .put(upload_id(b), chunk(b, blocked, "bytes 0-5/6"))
                .await,
            413,
        );
        assert_eq!(polls.get(), 0);
        release.send(()).unwrap();
    };
    let (result, ()) = futures_util::join!(running, control);
    result.unwrap();
    uploads
        .put(
            upload_id(b),
            chunk(b, Body::Bytes(b"abcd".to_vec()), "bytes 0-3/7"),
        )
        .await
        .unwrap();
    status(
        uploads
            .put(
                upload_id(c),
                chunk(c, Body::Bytes(b"x".to_vec()), "bytes 0-0/1"),
            )
            .await,
        413,
    );
    assert_eq!(
        db.one("SELECT SUM(received) n FROM uploads", &[])
            .unwrap()
            .unwrap()
            .number("n"),
        Some(10.0)
    );
}

// Failed chunks must restore the committed file length and release capacity for a correct retry.
async fn chunk_rollback(fixture: &Fixture) {
    let (_, store) = library(&fixture.root.join("rollback.db"));
    let directory = fixture.directory("uploads-rollback");
    let uploads = Uploads::new(
        store,
        Rc::new(DirectoryUploads::new(&directory)),
        Rc::new(LocalMedia::new(fixture.directory("rollback-media"))),
        UploadOptions {
            max_bytes: 6,
            ..UploadOptions::default()
        },
    )
    .unwrap();
    let auth = uploads.authorize(&Value::Null).await.unwrap();
    let id = upload_id(&auth);
    uploads
        .put(
            id,
            chunk(&auth, Body::Bytes(b"abc".to_vec()), "bytes 0-2/6"),
        )
        .await
        .unwrap();
    status(
        uploads
            .put(id, chunk(&auth, Body::Bytes(b"de".to_vec()), "bytes 3-5/6"))
            .await,
        400,
    );
    assert_eq!(std::fs::metadata(directory.join(id)).unwrap().len(), 3);
    uploads
        .put(
            id,
            chunk(&auth, Body::Bytes(b"def".to_vec()), "bytes 3-5/6"),
        )
        .await
        .unwrap();
    assert_eq!(std::fs::read(directory.join(id)).unwrap(), b"abcdef");
}

// Upload through the API, requiring the native request adapters to preserve successful chunk status.
async fn upload(api: &mm_core::api::Api, bytes: &[u8]) -> String {
    let auth = ask_api(api, "POST", "/uploads/authorize", Body::Empty, &[])
        .await
        .value();
    let token = format!("Bearer {}", auth.get("token").as_str().unwrap());
    let range = format!("bytes 0-{}/{}", bytes.len() - 1, bytes.len());
    let result = ask_api(
        api,
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
    upload_id(&auth).to_owned()
}

// Claims must remove temporary originals and leave permanent sources reusable for cards and attachments.
async fn api_claims(fixture: &Fixture) {
    let dir = fixture.directory("api");
    let (_, store) = library(&dir.join("test.db"));
    let uploads = dir.join("uploads");
    let api = api(
        store,
        Rc::new(LocalMedia::new(dir.join("media"))),
        Rc::new(DirectoryUploads::new(&uploads)),
    )
    .await;
    let bytes = png();
    let upload_id = upload(&api, &bytes).await;
    let saved = ask_api(
        &api,
        "POST",
        "/objects",
        body(obj! { "type" => "Media", "uploadId" => upload_id.as_str() }.into()),
        &[],
    )
    .await;
    assert_eq!(saved.status, 201);
    assert!(!uploads.join(&upload_id).exists());
    let duplicate = ask_api(
        &api,
        "POST",
        "/objects",
        body(obj! { "type" => "Media", "uploadId" => upload_id.as_str() }.into()),
        &[],
    )
    .await;
    assert_eq!(duplicate.status, 200);
    assert_eq!(duplicate.value(), saved.value());
    let card = ask_api(
        &api,
        "GET",
        &format!("/objects/{}", saved.value().get("id").as_str().unwrap()),
        Body::Empty,
        &[],
    )
    .await
    .value();
    let path = card.get("blob").get("path").as_str().unwrap();
    let original = format!("/media/{path}");
    assert_eq!(
        ask_api(&api, "GET", &original, Body::Empty, &[])
            .await
            .bytes,
        bytes
    );
    let head = ask_api(&api, "HEAD", &original, Body::Empty, &[]).await;
    assert!(head.bytes.is_empty());
    assert_eq!(
        head.headers.get("content-length"),
        Some(bytes.len().to_string().as_str())
    );
    let alias = ask_api(
        &api,
        "GET",
        &format!("/media/{path}"),
        Body::Empty,
        &[("range", "bytes=2-8")],
    )
    .await;
    assert_eq!(alias.status, 206);
    assert_eq!(alias.bytes, bytes[2..9]);
    let note = ask_api(
        &api,
        "POST",
        "/objects",
        body(obj! { "content" => "Synthetic note" }.into()),
        &[],
    )
    .await
    .value();
    let note = note.get("id").as_str().unwrap();
    for id in [upload_id, upload(&api, &bytes).await] {
        let attached = ask_api(
            &api,
            "POST",
            &format!("/objects/{note}/attachments"),
            body(obj! { "uploadId" => id.as_str() }.into()),
            &[],
        )
        .await;
        assert_eq!(attached.status, 200);
        assert_eq!(
            ask_api(
                &api,
                "GET",
                attached.value().get("path").as_str().unwrap(),
                Body::Empty,
                &[]
            )
            .await
            .bytes,
            bytes
        );
        assert!(!uploads.join(&id).exists());
        let reused = ask_api(
            &api,
            "POST",
            &format!("/objects/{note}/attachments"),
            body(obj! { "uploadId" => id.as_str() }.into()),
            &[],
        )
        .await;
        assert_eq!(reused.status, 200);
        assert_eq!(
            ask_api(
                &api,
                "GET",
                reused.value().get("path").as_str().unwrap(),
                Body::Empty,
                &[]
            )
            .await
            .bytes,
            bytes
        );
    }
    api.jobs().drain().await;
    api.close();
}

// Run all original scenarios serially so the fake command environment is never shared with another test.
#[test]
fn check_media() {
    let fixture = Fixture::new("check-media");
    mymind::runtime::run(async {
        sparse_originals_and_backpressure(&fixture).await;
        file_backed_images(&fixture).await;
        video_cache(&fixture).await;
        previews(&fixture).await;
        expiry_and_orphans(&fixture).await;
        aggregate_reservations(&fixture).await;
        chunk_rollback(&fixture).await;
        api_claims(&fixture).await;
    });
}
