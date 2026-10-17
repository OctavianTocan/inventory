//! `LocalMedia`, `LocalProcessor` and `mymind container` against frozen pre-switch media expectations. Inputs have
//! an immutable manifest; image bytes are exact and encoded-video renditions retain their ffprobe semantics.
//! The original JS generators and container run only under explicit golden recording.

mod common;

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use common::{Server, repo};
use mm_core::host::{Media, MediaInput, MediaProcessor, MediaResult, MediaUser, RenditionSource};
use mm_core::http::Body;
use mm_core::{Error, arr, obj};
use mm_js::{Value, base64, json};
use mm_test_support::golden::{Batch, TestGuard, recording, recording_mismatch, register_loopback_port};
use mymind::local_media::{LocalMedia, analysis_image};
use mymind::media_processor::LocalProcessor;
use sha2::{Digest, Sha256};

struct Fixtures {
    dir: PathBuf,
    names: Vec<String>,
    identity: String,
    inputs: Vec<PathBuf>,
}

// Return the target-local directory used by all media test scratch files.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("media-test")
}

// Locate the original media oracle retained for explicit recording.
fn driver() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/drivers/media.mjs")
}

// Describe bytes by their exact length and SHA-256 without retaining their contents.
fn digest(bytes: &[u8]) -> Value {
    obj! { "bytes" => bytes.len(), "sha256" => format!("{:x}", Sha256::digest(bytes)) }.into()
}

// Create a recording-only Node command with an isolated process environment.
fn legacy_command() -> Command {
    assert!(recording(), "the media JS oracle is recording-only");
    let mut command = Command::new("node");
    command
        .arg(driver())
        .env_clear()
        .env("MYMIND_ROOT", repo())
        .env("STORE_LOAD_ENV", "0")
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"));
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
}

// Collect regular files recursively in deterministic directory order.
fn walk_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(directory).unwrap().map(|entry| entry.unwrap()).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_type().unwrap().is_dir() {
            walk_files(&entry.path(), files);
        } else if entry.file_type().unwrap().is_file() {
            files.push(entry.path());
        }
    }
}

// Reproduce the original oracle's ordered spread of recorded library media.
fn selected_originals(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(&root.join("capture/media"), &mut files);
    files.sort();
    let mut classified = Vec::new();
    for path in files {
        if std::fs::metadata(&path).unwrap().len() > 12_000_000 {
            continue;
        }
        let mut head = [0u8; 16];
        let read = std::fs::File::open(&path).unwrap().read(&mut head).unwrap();
        let head = &head[..read];
        let kind = if head.starts_with(&[0xff, 0xd8, 0xff]) {
            Some("jpg")
        } else if head.starts_with(b"RIFF") {
            Some("webp")
        } else if head.get(1..4) == Some(b"PNG") {
            Some("png")
        } else if head.starts_with(b"%PDF-") {
            Some("pdf")
        } else if head.get(4..8) == Some(b"ftyp") {
            Some("mp4")
        } else {
            None
        };
        if let Some(kind) = kind {
            classified.push((path, kind));
        }
    }
    let mut selected = Vec::new();
    for (kind, count) in [("jpg", 18), ("webp", 14), ("png", 6), ("pdf", 3), ("mp4", 3)] {
        let matches: Vec<_> = classified.iter().filter(|(_, found)| *found == kind).collect();
        for (path, _) in matches.iter().step_by((matches.len() / count).max(1)).take(count) {
            selected.push(path.clone());
        }
    }
    selected
}

// Freeze a first-run input or require exact equality with the immutable file already on disk.
fn freeze_input(path: &Path, bytes: &[u8], first_freeze: bool) {
    match std::fs::read(path) {
        Ok(existing) => assert!(
            existing == bytes,
            "frozen media input changed: {}; restore the original fixture instead of overwriting it",
            path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && first_freeze => std::fs::write(path, bytes).unwrap(),
        Err(error) => panic!("required frozen media input {}: {error}; restore the original fixture", path.display()),
    }
}

// Freeze inputs during recording and verify every immutable input before reuse.
fn fixtures() -> &'static Fixtures {
    static FIXTURES: OnceLock<Fixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| {
        let dir = fresh("fixtures");
        let root = repo();
        let frozen = root.join("rust/test-fixtures/media");
        let manifest_path = frozen.join("manifest.json");
        let first_freeze = !manifest_path.exists();
        if first_freeze {
            let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
            let recorded = golden.exists()
                && std::fs::read_dir(&golden).unwrap().any(|entry| {
                    let name = entry.unwrap().file_name();
                    let name = name.to_string_lossy();
                    name == "media.json"
                        || name == "media.json.gz"
                        || (name.starts_with("media.part-") && (name.ends_with(".jsonl") || name.ends_with(".jsonl.gz")))
                });
            assert!(
                !recorded,
                "the frozen media manifest is missing but media goldens exist; restore the exact frozen corpus instead of generating replacement inputs"
            );
        }
        // WebM container IDs vary between generations, so later recordings must reuse the frozen corpus.
        if recording() && first_freeze {
            std::fs::create_dir_all(&frozen).unwrap();
            let output = legacy_command().arg("fixtures").arg(&dir).output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let names = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
            let names = names.as_array().expect("fixture names");
            let originals = selected_originals(&root);
            let mut entries = Vec::new();
            for name in names {
                let name = name.as_str().unwrap();
                let bytes = std::fs::read(dir.join(name)).unwrap();
                let source = if name.starts_with("real-") {
                    let number: usize = name.strip_prefix("real-").unwrap().split('.').next().unwrap().parse().unwrap();
                    let path = originals
                        .get(number)
                        .unwrap_or_else(|| panic!("the original selected input for {name} is missing"));
                    let actual = std::fs::read(path).unwrap();
                    if actual != bytes {
                        let selection: Vec<Value> = originals
                            .iter()
                            .map(|source| source.strip_prefix(&root).unwrap().to_string_lossy().into_owned().into())
                            .collect();
                        let request: Value = obj! { "fixture" => name, "index" => number, "selection" => selection }.into();
                        recording_mismatch(
                            env!("CARGO_MANIFEST_DIR"),
                            "media",
                            "fixture-selection",
                            json::to_string(&request).as_bytes(),
                            &actual,
                            &bytes,
                        );
                    }
                    path.clone()
                } else {
                    let target = frozen.join(name);
                    freeze_input(&target, &bytes, first_freeze);
                    target
                };
                entries.push(Value::from(
                    obj! { "name" => name, "source" => source.strip_prefix(&root).unwrap().to_string_lossy().into_owned(), "content" => digest(&bytes) },
                ));
            }
            let manifest: Value = obj! { "version" => 1, "files" => entries }.into();
            freeze_input(&manifest_path, json::to_string(&manifest).as_bytes(), first_freeze);
        }
        let manifest_bytes = mm_test_support::golden::read_input(&manifest_path);
        let manifest = json::parse(std::str::from_utf8(&manifest_bytes).unwrap()).unwrap();
        assert_eq!(manifest.get("version").as_f64(), Some(1.0), "media input manifest version");
        let mut seen = HashSet::new();
        let mut inputs = Vec::new();
        let names: Vec<String> = manifest
            .get("files")
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let name = entry.get("name").as_str().unwrap();
                assert!(
                    Path::new(name).components().count() == 1 && seen.insert(name.to_owned()),
                    "unique flat media fixture names"
                );
                let source = entry.get("source").as_str().unwrap();
                assert!(
                    Path::new(source).is_relative() && !Path::new(source).components().any(|part| matches!(part, std::path::Component::ParentDir)),
                    "relative media input source"
                );
                let source = root.join(source);
                let bytes = mm_test_support::golden::read_input(&source);
                inputs.push(source);
                assert_eq!(
                    json::to_string(&digest(&bytes)),
                    json::to_string(entry.get("content")),
                    "media input fingerprint: {name}"
                );
                std::fs::write(dir.join(name), bytes).unwrap();
                name.to_owned()
            })
            .collect();
        assert!(names.len() >= 60, "the original media corpus must remain complete");
        for name in ["clip.mp4", "wide.mp4", "clip.webm", "audio-only.mp4"] {
            assert!(seen.contains(name), "required generated video fixture {name}");
        }
        Fixtures {
            dir,
            names,
            identity: format!("{:x}", Sha256::digest(&manifest_bytes)),
            inputs,
        }
    })
}

// Register every selected original and frozen synthetic input with its explicit length and SHA-256.
fn register_media_inputs(batch: &mut Batch, fixture: &Fixtures) {
    batch.register_input(&repo().join("rust/test-fixtures/media/manifest.json"));
    for input in &fixture.inputs {
        batch.register_input(input);
    }
}

// Recreate one target-local scratch directory for a test phase.
fn fresh(name: &str) -> PathBuf {
    let dir = scratch().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// Bind an ordered media scenario to goldens and run the original oracle only while recording.
fn oracle(namespace: &str, fixture: &Fixtures, root: &Path, ops: Vec<Value>) -> (Batch, Option<Vec<Value>>) {
    let clean = |value: &Value| {
        json::to_string(value)
            .replace(fixture.dir.to_str().unwrap(), "$FIXTURES")
            .replace(scratch().to_str().unwrap(), "$SCRATCH")
            .replace(repo().to_str().unwrap(), "$REPO")
    };
    let scenario = format!("{:x}", Sha256::digest(clean(&Value::Array(ops.clone())).as_bytes()));
    let requests: Vec<Vec<u8>> = ops
        .iter()
        .enumerate()
        .map(|(index, op)| {
            json::to_string(&Value::from(
                obj! { "fixture" => fixture.identity.as_str(), "scenario" => scenario.as_str(), "index" => index, "op" => clean(op) },
            ))
            .into_bytes()
        })
        .collect();
    let mut batch = Batch::new(env!("CARGO_MANIFEST_DIR"), "media", namespace, &requests);
    register_media_inputs(&mut batch, fixture);
    let expected = recording().then(|| {
        let job = scratch().join(format!("job-{}.json", root.file_name().unwrap().to_string_lossy()));
        let job_value: Value = obj! { "root" => root.to_string_lossy().into_owned(), "ops" => Value::Array(ops) }.into();
        std::fs::write(&job, json::to_string(&job_value)).unwrap();
        let output = legacy_command().arg("run").arg(&job).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let values = json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap().as_array().unwrap().clone();
        assert_eq!(values.len(), requests.len(), "media oracle answer count");
        values
    });
    (batch, expected)
}

// Compare one canonical Rust value with its frozen or newly recorded oracle answer.
fn compare(batch: &mut Batch, index: usize, actual: Value, expected: Option<Value>) {
    let actual = json::to_string(&actual);
    let expected = expected.map(|value| json::to_string(&value));
    batch.check(index, actual.as_bytes(), expected.as_deref().map(str::as_bytes));
}

// Account for every successful seed operation in the ordered scenario.
fn seed_checks(batch: &mut Batch, expected: Option<&[Value]>, count: usize) {
    for index in 0..count {
        compare(batch, index, Value::Null, expected.map(|values| values[index].clone()));
    }
}

// Build the original oracle operation that copies one input into its media store.
fn seed_op(fixture: &Fixtures, name: &str) -> Value {
    obj! { "op" => "seed", "from" => fixture.dir.join(name).to_string_lossy().into_owned(), "to" => format!("acct/card/{name}") }.into()
}

// Copy one verified input into the Rust media store.
fn seed_rust(fixture: &Fixtures, root: &Path, name: &str) {
    std::fs::create_dir_all(root.join("acct/card")).unwrap();
    std::fs::copy(fixture.dir.join(name), root.join("acct/card").join(name)).unwrap();
}

// Project media failures into the original oracle's comparable error shape.
fn error_value(error: &Error) -> Value {
    match error {
        Error::Status { status, message, .. } => obj! { "error" => obj! { "status" => *status, "message" => message.as_str() } }.into(),
        _ => obj! { "error" => obj! { "status" => Value::Null, "message" => "unexpected" } }.into(),
    }
}

// Return the synthetic account identity used by media preparation tests.
fn user() -> MediaUser {
    MediaUser {
        account_uid: "acct".to_owned(),
        id_prefix: "card_".to_owned(),
    }
}

// Classify a fixture extension for rendition case generation.
fn kind(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap() {
        "pdf" => "pdf",
        "mp4" | "webm" => "video",
        _ => "image",
    }
}

// Read the video codec, dimensions and pixel format while requiring a valid stream.
fn probe(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,codec_name,pix_fmt",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success(), "ffprobe failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "ffprobe must identify a video stream");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// Replace only the generated blob identifier in serialized preparation metadata.
fn normalized(value: &Value) -> String {
    let text = json::to_string(value);
    match value.get("data").get("blob").get("id").as_str() {
        Some(id) => text.replace(id, "BLOBID"),
        None => text,
    }
}

#[test]
fn prepare_matches() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let js_root = fresh("prepare-js");
    let rs_root = fresh("prepare-rs");
    let cases: Vec<(usize, &String, bool)> = fixture.names.iter().enumerate().map(|(index, name)| (index, name, index % 4 == 1)).collect();
    let ops = cases
        .iter()
        .map(|(index, name, bytes)| {
            let user: Value = obj! { "accountUid" => "acct", "idPrefix" => "card_" }.into();
            obj! {
                "op" => "prepare", "file" => fixture.dir.join(name).to_string_lossy().into_owned(), "bytes" => *bytes,
                "name" => name.as_str(), "user" => user, "id" => format!("card_{index}")
            }
            .into()
        })
        .collect();
    let (mut golden, expected) = oracle("prepare", fixture, &js_root, ops);
    let mut stored = 0;
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for (index, name, bytes) in &cases {
            let path = fixture.dir.join(name);
            let input = if *bytes {
                MediaInput::Bytes(std::fs::read(&path).unwrap())
            } else {
                MediaInput::File(path.to_string_lossy().into_owned())
            };
            let got = match media.prepare(input, "application/octet-stream", name, &user(), &format!("card_{index}")).await {
                Ok(made) => made.to_value(),
                Err(error) => error_value(&error),
            };
            stored += usize::from(got.get("data").is_object());
            compare(
                &mut golden,
                *index,
                normalized(&got).into(),
                expected.as_ref().map(|values| normalized(&values[*index]).into()),
            );
        }
    });
    println!("prepared {stored} of {} fixtures", cases.len());
    assert!(cases.len() >= 60 && stored >= 40);
    golden.finish();
    guard.finish();
}

// List the original rendition variants exercised for one fixture.
fn suffixes(name: &str) -> Vec<&'static str> {
    let mut list = vec![";800x600-t.webp"];
    match kind(name) {
        "pdf" => list.extend([
            ";background(fff)/page(1)/642x840-t.webp",
            ";background(fff)/page(2)/642x840-t.webp",
            ";background(fff)/page(9)/642x840-t.webp",
            ";300x213.jpeg",
        ]),
        "video" => list.extend([";960x540.mp4", ";600x338.mp4", ";320x180/hevc.mp4", ";800x450-t.webp", ";300x213.jpeg"]),
        _ if name.starts_with("real-") || name == "plain.jpg" || name.starts_with("exif-") || name.ends_with(".png") || name.ends_with(".webp") => {
            list.extend([
                ";800x350-c.webp",
                ";300x213.webp",
                ";800x800.jpeg",
                ";800x1000-t.webp",
                ";640x480-t.png",
                ";1200x900-t.webp",
                ";background(fff)/page(1)/642x840-t.webp",
                ";original.jpg",
            ])
        }
        _ => {}
    }
    list
}

// Represent response metadata with exact image bytes or video rendition probe fields.
fn response_content(label: &str, content_type: &str, path: &Path) -> Value {
    let bytes = if content_type == "video/mp4" && label.contains(';') {
        obj! { "probe" => probe(path) }.into()
    } else {
        digest(&std::fs::read(path).unwrap())
    };
    obj! { "result" => obj! { "type" => content_type }, "content" => bytes }.into()
}

// Read the original oracle's stored output into the canonical response shape.
fn expected_response(label: &str, expected: &Value, root: &Path) -> Value {
    if let Some(content_type) = expected.get("type").as_str() {
        response_content(label, content_type, &root.join(expected.get("file").as_str().unwrap()))
    } else {
        expected.clone()
    }
}

// Project a Rust media response or error into the canonical response shape.
fn actual_response(label: &str, got: &Result<Option<MediaResult>, Error>) -> Value {
    match got {
        Ok(None) => Value::Null,
        Ok(Some(found)) => response_content(label, &found.content_type, Path::new(&found.key)),
        Err(error) => error_value(error),
    }
}

#[test]
fn renditions_match() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let js_root = fresh("rendition-js");
    let rs_root = fresh("rendition-rs");
    let mut ops: Vec<Value> = fixture.names.iter().map(|name| seed_op(fixture, name)).collect();
    let mut cases = Vec::new();
    for name in &fixture.names {
        seed_rust(fixture, &rs_root, name);
        for suffix in suffixes(name) {
            let relative = format!("acct/card/{name}{suffix}");
            ops.push(obj! { "op" => "response", "relative" => relative.as_str() }.into());
            cases.push(relative);
        }
    }
    let (mut golden, results) = oracle("renditions", fixture, &js_root, ops);
    seed_checks(&mut golden, results.as_deref(), fixture.names.len());
    let mut produced = 0;
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for (index, relative) in cases.iter().enumerate() {
            let got = media.response(relative).await;
            produced += usize::from(matches!(&got, Ok(Some(_))));
            let offset = fixture.names.len() + index;
            compare(
                &mut golden,
                offset,
                actual_response(relative, &got),
                results.as_ref().map(|values| expected_response(relative, &values[offset], &js_root)),
            );
        }
    });
    println!("{produced} of {} renditions were made by both", cases.len());
    assert!(produced >= 300);
    golden.finish();
    guard.finish();
}

// Collect a response body so send assertions compare its complete contents.
async fn drain(body: Body) -> Vec<u8> {
    body.bytes().await.unwrap()
}

#[test]
fn send_matches() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let js_root = fresh("send-js");
    let rs_root = fresh("send-rs");
    let files = ["plain.jpg", "empty.bin", "clip.mp4"];
    let ranges: [Option<&str>; 14] = [
        None,
        Some("bytes=0-0"),
        Some("bytes=5-"),
        Some("bytes=-10"),
        Some("bytes=0-999999999"),
        Some("bytes=999999999-"),
        Some("bytes=abc"),
        Some("bytes=0-1,3-4"),
        Some("bytes=-0"),
        Some("bytes=5-2"),
        Some("items=0-1"),
        Some("bytes=-999999999"),
        Some("bytes=1-"),
        Some("bytes=-"),
    ];
    let mut ops: Vec<Value> = files.iter().map(|name| seed_op(fixture, name)).collect();
    let mut cases = Vec::new();
    for name in files {
        seed_rust(fixture, &rs_root, name);
        for range in ranges {
            for method in ["GET", "HEAD"] {
                let relative = format!("acct/card/{name}");
                ops.push(
                    obj! { "op" => "send", "relative" => relative.as_str(), "method" => method, "range" => range.map_or(Value::Null, Value::from) }.into(),
                );
                cases.push((relative, method, range));
            }
        }
    }
    let (mut golden, results) = oracle("send", fixture, &js_root, ops);
    seed_checks(&mut golden, results.as_deref(), files.len());
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for (index, (relative, method, range)) in cases.iter().enumerate() {
            let found = media.response(relative).await.unwrap().unwrap();
            let sent = media.send(&found, method, *range).await.unwrap();
            let headers: Vec<Value> = sent.headers.iter().map(|(name, value)| arr![name, value]).collect();
            let body = base64::encode(&drain(sent.body).await);
            let got: Value = obj! { "status" => sent.status, "headers" => Value::Array(headers), "body" => body }.into();
            let offset = files.len() + index;
            compare(&mut golden, offset, got, results.as_ref().map(|values| values[offset].clone()));
        }
    });
    golden.finish();
    guard.finish();
}

#[test]
fn errors_and_images_match() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let js_root = fresh("errors-js");
    let rs_root = fresh("errors-rs");
    let names = [
        "plain.jpg",
        "garbage.bin",
        "pages.pdf",
        "exif-6.jpg",
        "alpha.png",
        "truncated.jpg",
        "image.tiff",
        "image.avif",
        "image.svg",
        "huge.png",
    ];
    let mut ops: Vec<Value> = names.iter().map(|name| seed_op(fixture, name)).collect();
    let relatives = [
        "acct/card/missing.jpg",
        "../escape.jpg",
        "acct/../../x.jpg",
        "nodir/x.jpg;800x600-t.webp",
        "acct/card/plain.jpg;bogus",
        "acct/card/plain.jpg;0x0.webp",
        "acct/card/plain.jpg;5000x10.webp",
        "acct/card/plain.jpg;original.jpg",
        "acct/card/garbage.bin;800x600-t.webp",
        "acct/card/plain;800x600-t.webp",
        "acct/card/exif-6;300x213.webp",
        "acct/card/pages.pdf;background(fff)/page(9)/642x840-t.webp",
        "acct/card/plain.jpg;800x600-t.gif",
        "acct/card/truncated.jpg;800x600-t.webp",
        "acct/card/image.tiff;300x200-t.webp",
        "acct/card/image.avif;300x200-t.jpeg",
        "acct/card/image.svg;300x200-t.webp",
        "acct/card/huge.png;300x200-t.webp",
        "acct/card/alpha.png;300x200-t.jpeg",
    ];
    for relative in relatives {
        ops.push(obj! { "op" => "response", "relative" => relative }.into());
    }
    let out = |name: &str| fresh(&format!("errors-out-{name}"));
    let images: Vec<(&str, Value)> = vec![
        ("path", obj! { "path" => "acct/card/plain.jpg" }.into()),
        ("src", obj! { "src" => "/media/acct/card/exif-6.jpg;800x1000-t.webp" }.into()),
        ("pdf", obj! { "path" => "acct/card/pages.pdf" }.into()),
        ("missing", obj! { "path" => "acct/card/nothing.jpg" }.into()),
        ("falsy", Value::Null),
        ("garbage", obj! { "path" => "acct/card/garbage.bin" }.into()),
    ];
    let image_dirs: Vec<PathBuf> = images.iter().map(|(name, _)| out(name)).collect();
    for ((_, value), dir) in images.iter().zip(&image_dirs) {
        ops.push(obj! { "op" => "image", "value" => value.clone(), "out" => dir.join("js.png").to_string_lossy().into_owned() }.into());
    }
    let (mut golden, results) = oracle("errors-and-images", fixture, &js_root, ops);
    seed_checks(&mut golden, results.as_deref(), names.len());
    mymind::runtime::run(async {
        let media = LocalMedia::new(&rs_root);
        for name in names {
            seed_rust(fixture, &rs_root, name);
        }
        for (index, relative) in relatives.iter().enumerate() {
            let got = media.response(relative).await;
            let offset = names.len() + index;
            compare(
                &mut golden,
                offset,
                actual_response(relative, &got),
                results.as_ref().map(|values| expected_response(relative, &values[offset], &js_root)),
            );
        }
        let concurrent = futures_util::future::join_all((0..8).map(|_| media.response("acct/card/alpha.png;777x333-t.webp"))).await;
        let keys: Vec<_> = concurrent.iter().map(|made| made.as_ref().unwrap().as_ref().unwrap().key.clone()).collect();
        assert!(
            keys.iter().all(|key| key == &keys[0]) && Path::new(&keys[0]).is_file(),
            "concurrent requests agree on the stored file"
        );
        for (index, ((label, value), dir)) in images.iter().zip(&image_dirs).enumerate() {
            let got = media.image(value).await;
            let actual: Value = match &got {
                Ok(None) => Value::Null,
                Ok(Some(made)) => obj! { "mime" => made.mime.as_str(), "content" => digest(&made.data) }.into(),
                Err(_) => obj! { "error" => true }.into(),
            };
            let offset = names.len() + relatives.len() + index;
            let expected = results.as_ref().map(|values| {
                let expected = &values[offset];
                if expected.get("error").is_object() {
                    obj! { "error" => true }.into()
                } else if expected.get("mime").is_string() {
                    obj! { "mime" => expected.get("mime").clone(), "content" => digest(&std::fs::read(dir.join("js.png")).unwrap()) }.into()
                } else {
                    expected.clone()
                }
            });
            compare(&mut golden, offset, actual, expected);
            println!("image {label} compared");
        }
    });
    golden.finish();
    guard.finish();
}

#[test]
fn analysis_matches() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let js_root = fresh("analysis-js");
    let out = fresh("analysis-out");
    let ops = fixture
        .names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            obj! { "op" => "analysis", "file" => fixture.dir.join(name).to_string_lossy().into_owned(), "out" => out.join(format!("{index}.png")).to_string_lossy().into_owned() }.into()
        })
        .collect();
    let (mut golden, results) = oracle("analysis", fixture, &js_root, ops);
    let mut converted = 0;
    mymind::runtime::run(async {
        for (index, name) in fixture.names.iter().enumerate() {
            let got = analysis_image(std::fs::read(fixture.dir.join(name)).unwrap()).await;
            let actual: Value = match got {
                Ok(made) => {
                    converted += 1;
                    obj! { "mime" => made.mime, "content" => digest(&made.data) }.into()
                }
                Err(_) => obj! { "error" => true }.into(),
            };
            let expected = results.as_ref().map(|values| {
                if values[index].get("error").is_object() {
                    obj! { "error" => true }.into()
                } else {
                    obj! { "mime" => values[index].get("mime").clone(), "content" => digest(&std::fs::read(out.join(format!("{index}.png"))).unwrap()) }.into()
                }
            });
            let mime = actual.get("mime").as_str().map(str::to_owned);
            compare(&mut golden, index, actual, expected);
            if let Some(mime) = mime {
                assert_eq!(mime, "image/png", "analysis {name}");
            }
        }
    });
    println!("{converted} fixtures converted by both");
    assert!(converted >= 40);
    golden.finish();
    guard.finish();
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

// Send one loopback container request and normalize HTTP framing headers.
fn post(port: u16, method: &str, target: &str, headers: &[(&str, String)], body: &[u8]) -> Reply {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(120))).unwrap();
    let mut head = format!(
        "{method} {target} HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\ncontent-length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(body).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let split = raw.windows(4).position(|window| window == b"\r\n\r\n").unwrap();
    let text = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = text.lines();
    let status = lines.next().unwrap()[9..12].parse().unwrap();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let mut body = raw[split + 4..].to_vec();
    if headers.iter().any(|(name, value)| name == "transfer-encoding" && value.contains("chunked")) {
        let (mut out, mut rest) = (Vec::new(), &body[..]);
        loop {
            let end = rest.windows(2).position(|window| window == b"\r\n").unwrap();
            let size = usize::from_str_radix(std::str::from_utf8(&rest[..end]).unwrap().split(';').next().unwrap().trim(), 16).unwrap();
            if size == 0 {
                break;
            }
            out.extend_from_slice(&rest[end + 2..end + 2 + size]);
            rest = &rest[end + 2 + size + 2..];
        }
        body = out;
    }
    let headers = headers
        .into_iter()
        .filter(|(name, _)| !["date", "connection", "keep-alive", "transfer-encoding", "content-length"].contains(&name.as_str()))
        .collect();
    Reply { status, headers, body }
}

// Start a container process and wait for its ready message.
fn spawn_container(command: Command) -> Server {
    Server::start(command, "media container on :")
}

// Configure an isolated container process on a registered dynamic loopback port.
fn container_command(program: &str, arguments: &[&Path]) -> Command {
    let mut command = Command::new(program);
    let free = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    register_loopback_port(free);
    command
        .args(arguments)
        .current_dir(repo())
        .env_clear()
        .env("PORT", free.to_string())
        .env("TMPDIR", scratch())
        .env("STORE_LOAD_ENV", "0");
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
}

// Encode one container operation's arguments in its required request header.
fn media_args(value: Value) -> (&'static str, String) {
    ("x-media-args", base64::encode_url(json::to_string(&value).as_bytes()))
}

// Serve a fixed image and missing-resource response from a loopback fixture.
fn resource_server(image: Vec<u8>) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    register_loopback_port(port);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (mut stream, image) = (stream, image.clone());
            std::thread::spawn(move || {
                let mut request = [0u8; 2048];
                let read = stream.read(&mut request).unwrap_or(0);
                let first = String::from_utf8_lossy(&request[..read]).lines().next().unwrap_or("").to_owned();
                let (status, kind, body) = if first.contains("/image.png") {
                    ("200 OK", "image/png", image)
                } else {
                    ("404 Not Found", "text/plain", b"nope".to_vec())
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nx-fixture: yes\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            });
        }
    });
    port
}

struct ContainerCase {
    label: String,
    method: String,
    target: String,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
    video: bool,
}

// Normalize canonical encoded text with its original alphabet, retaining every other spelling exactly.
fn encoded_header(value: &str, port: u16) -> Value {
    let bytes = base64::decode(value);
    if let Ok(text) = std::str::from_utf8(&bytes) {
        let normalized = text.replace(&format!(":{port}"), ":RESOURCE_PORT");
        // These headers use base64url; shared spellings remain URL-encoded across port length changes.
        if base64::encode_url(&bytes) == value {
            return obj! { "encoding" => "base64url", "value" => base64::encode_url(normalized.as_bytes()) }.into();
        }
        if base64::encode(&bytes) == value {
            return obj! { "encoding" => "base64", "value" => base64::encode(normalized.as_bytes()) }.into();
        }
    }
    obj! { "encoding" => "raw", "value" => value }.into()
}

// Preserve header names, order and ordinary values while normalizing the selected encoded header.
fn container_headers(headers: &[(&str, String)], port: u16, encoded: &str) -> Value {
    Value::Array(
        headers
            .iter()
            .map(|(name, value)| {
                let value = if *name == encoded {
                    encoded_header(value, port)
                } else {
                    value.clone().into()
                };
                arr![*name, value]
            })
            .collect(),
    )
}

// Canonicalize container status, headers and exact body bytes or video probe fields.
fn container_reply(reply: &Reply, port: u16, video: bool) -> Value {
    let headers: Vec<(&str, String)> = reply.headers.iter().map(|(name, value)| (name.as_str(), value.clone())).collect();
    let body = if video && reply.status == 200 {
        let path = scratch().join("container-probe.mp4");
        std::fs::write(&path, &reply.body).unwrap();
        obj! { "probe" => probe(&path) }.into()
    } else if reply.headers.iter().any(|(name, value)| name == "content-type" && value == "application/json") {
        if let Ok(text) = std::str::from_utf8(&reply.body) {
            let id = json::parse(text)
                .ok()
                .and_then(|value| value.get("data").get("blob").get("id").as_str().map(str::to_owned));
            let text = id.as_deref().map_or_else(|| text.to_owned(), |id| text.replace(id, "BLOBID"));
            digest(text.replace(&format!(":{port}"), ":RESOURCE_PORT").as_bytes())
        } else {
            digest(&reply.body)
        }
    } else {
        digest(&reply.body)
    };
    obj! { "status" => reply.status, "headers" => container_headers(&headers, port, "x-resource"), "body" => body }.into()
}

#[test]
fn container_normalization_preserves_wire_bytes() {
    let resource = "http://127.0.0.1:12345/";
    let url = base64::encode_url(resource.as_bytes());
    let standard = base64::encode(resource.as_bytes());
    assert_ne!(url, standard, "the fixture distinguishes padded standard encoding from base64url");
    let normalized = encoded_header(&url, 12345);
    assert_eq!(normalized.get("encoding").as_str(), Some("base64url"));
    assert_eq!(encoded_header(&standard, 12345).get("encoding").as_str(), Some("base64"));
    assert_ne!(json::to_string(&normalized), json::to_string(&encoded_header(&standard, 12345)));
    let other_port = base64::encode_url(b"http://127.0.0.1:1/");
    assert_eq!(json::to_string(&normalized), json::to_string(&encoded_header(&other_port, 1)));
    for raw in [format!("{url}!"), base64::encode_url(&[0xff])] {
        let value = encoded_header(&raw, 12345);
        assert_eq!(value.get("encoding").as_str(), Some("raw"));
        assert_eq!(value.get("value").as_str(), Some(raw.as_str()));
    }
    let headers = [("X-Raw", resource.to_owned()), ("x-resource", url)];
    let actual = container_headers(&headers, 12345, "x-resource");
    let expected = Value::Array(vec![arr!["X-Raw", resource], arr!["x-resource", normalized]]);
    assert_eq!(json::to_string(&actual), json::to_string(&expected));

    let body = r#"{ "data" : { "blob" : { "id" : "abc" } }, "source" : "http://127.0.0.1:12345/" }"#;
    let reply = |bytes: &[u8]| Reply {
        status: 200,
        headers: vec![("content-type".into(), "application/json".into())],
        body: bytes.to_vec(),
    };
    let expected = digest(body.replace("abc", "BLOBID").replace(":12345", ":RESOURCE_PORT").as_bytes());
    assert_eq!(
        json::to_string(container_reply(&reply(body.as_bytes()), 12345, false).get("body")),
        json::to_string(&expected)
    );
    for invalid in [b"{broken".as_slice(), &[0xff]] {
        let actual = container_reply(&reply(invalid), 12345, false);
        assert_eq!(json::to_string(actual.get("body")), json::to_string(&digest(invalid)));
    }
}

#[test]
fn container_matches() {
    let guard = TestGuard::new(env!("CARGO_MANIFEST_DIR"), "media");
    let fixture = fixtures();
    let root = repo();
    let js = recording().then(|| {
        spawn_container(container_command(
            "node",
            &[Path::new("--no-warnings"), &root.join("worker/container/server.mjs")],
        ))
    });
    let rust = spawn_container(container_command(env!("CARGO_BIN_EXE_mymind"), &[Path::new("container")]));
    let image_port = resource_server(std::fs::read(fixture.dir.join("alpha.png")).unwrap());
    let mut cases = Vec::new();
    let mut add = |label: &str, method: &str, target: &str, headers: Vec<(&'static str, String)>, body: Vec<u8>, video: bool| {
        cases.push(ContainerCase {
            label: label.to_owned(),
            method: method.to_owned(),
            target: target.to_owned(),
            headers,
            body,
            video,
        });
    };
    add("health", "GET", "/health", vec![], vec![], false);
    add("health post", "POST", "/health", vec![], vec![], false);
    add("get prepare", "GET", "/prepare", vec![], vec![], false);
    add("unknown", "POST", "/nope", vec![], vec![], false);
    add("inherited", "POST", "/constructor", vec![], vec![], false);
    for name in [
        "plain.jpg",
        "exif-6.jpg",
        "alpha.png",
        "pages.pdf",
        "clip.mp4",
        "image.svg",
        "garbage.bin",
        "huge.png",
        "real-00.jpg",
        "anim.gif",
        "truncated.jpg",
        "audio-only.mp4",
    ] {
        let args =
            obj! { "type" => "application/octet-stream", "name" => name, "user" => obj! { "accountUid" => "acct", "idPrefix" => "card_" }, "id" => "card_9" };
        add(
            &format!("prepare {name}"),
            "POST",
            "/prepare",
            vec![media_args(args.into())],
            std::fs::read(fixture.dir.join(name)).unwrap(),
            false,
        );
    }
    for name in ["plain.jpg", "exif-6.jpg", "pages.pdf", "clip.mp4", "real-01.jpg"] {
        let suffix = match kind(name) {
            "pdf" => ";background(fff)/page(1)/642x840-t.webp",
            "video" => ";320x180.mp4",
            _ => ";300x213.webp",
        };
        let relative = format!("acct/card/{name}{suffix}");
        let args = media_args(obj! { "name" => format!("acct/card/{name}"), "relative" => relative.as_str() }.into());
        add(
            &format!("rendition {name}"),
            "POST",
            "/rendition",
            vec![args],
            std::fs::read(fixture.dir.join(name)).unwrap(),
            suffix.ends_with("mp4"),
        );
    }
    let missing = media_args(obj! { "name" => "acct/card/other.jpg", "relative" => "acct/card/plain.jpg;300x213.webp" }.into());
    add(
        "rendition without original",
        "POST",
        "/rendition",
        vec![missing],
        std::fs::read(fixture.dir.join("plain.jpg")).unwrap(),
        false,
    );
    let escape = media_args(obj! { "name" => "../x.jpg", "relative" => "acct/card/plain.jpg;300x213.webp" }.into());
    add("rendition outside", "POST", "/rendition", vec![escape], b"x".to_vec(), false);
    let bad = media_args(obj! { "name" => "acct/card/a.jpg", "relative" => "acct/card/a.jpg;bogus" }.into());
    add(
        "rendition bad suffix",
        "POST",
        "/rendition",
        vec![bad],
        std::fs::read(fixture.dir.join("plain.jpg")).unwrap(),
        false,
    );
    for name in ["plain.jpg", "exif-3.jpg", "alpha.png", "anim.gif", "garbage.bin"] {
        add(
            &format!("image {name}"),
            "POST",
            "/image",
            vec![],
            std::fs::read(fixture.dir.join(name)).unwrap(),
            false,
        );
    }
    add("bad args", "POST", "/image", vec![("x-media-args", "!!!".to_owned())], vec![], false);
    add(
        "bad json",
        "POST",
        "/image",
        vec![("x-media-args", base64::encode_url(b"{nope"))],
        vec![],
        false,
    );
    for (label, path, offline, limit) in [
        ("png", "/image.png", true, Value::Null),
        ("missing", "/missing", true, Value::Null),
        ("online", "/image.png", false, Value::Null),
        ("limit", "/image.png", true, Value::from(10)),
    ] {
        let mut args = obj! { "url" => format!("http://127.0.0.1:{image_port}{path}"), "offline" => offline };
        if !limit.is_nullish() {
            args.insert("limit", limit);
        }
        add(&format!("resource {label}"), "POST", "/resource", vec![media_args(args.into())], vec![], false);
    }
    add(
        "screenshot of a private address",
        "POST",
        "/screenshot",
        vec![media_args(obj! { "url" => "http://127.0.0.1:1/" }.into())],
        vec![],
        false,
    );
    let requests: Vec<Vec<u8>> = cases
        .iter()
        .map(|case| {
            let headers = container_headers(&case.headers, image_port, "x-media-args");
            json::to_string(&Value::from(
                obj! { "fixture" => fixture.identity.as_str(), "label" => case.label.as_str(), "method" => case.method.as_str(),
                "target" => case.target.as_str(), "headers" => headers, "body" => digest(&case.body), "video" => case.video },
            ))
            .into_bytes()
        })
        .collect();
    let mut golden = Batch::new(env!("CARGO_MANIFEST_DIR"), "media", "container", &requests);
    register_media_inputs(&mut golden, fixture);
    assert_eq!(cases.len(), 37, "the complete container case corpus");
    for (index, case) in cases.iter().enumerate() {
        let actual = post(rust.port, &case.method, &case.target, &case.headers, &case.body);
        let expected = js
            .as_ref()
            .map(|server| post(server.port, &case.method, &case.target, &case.headers, &case.body));
        compare(
            &mut golden,
            index,
            container_reply(&actual, image_port, case.video),
            expected.as_ref().map(|reply| container_reply(reply, image_port, case.video)),
        );
    }
    golden.finish();
    guard.finish();
    println!("container: {} comparisons", cases.len());
}

#[test]
fn processor_matches_library_functions() {
    let fixture = fixtures();
    let directory = fresh("processor");
    let processor = LocalProcessor::new(Some(directory.clone()));
    mymind::runtime::run(async {
        let body = Body::Bytes(std::fs::read(fixture.dir.join("plain.jpg")).unwrap());
        let made = processor.prepare(body, "image/jpeg", "plain.jpg", &user(), "card_1").await.unwrap();
        assert_eq!(made.data.get("type").as_str(), Some("Image"));
        let original = RenditionSource {
            body: Body::Bytes(std::fs::read(fixture.dir.join("plain.jpg")).unwrap()),
            size: 0,
            name: "acct/card/plain.jpg".to_owned(),
        };
        assert!(processor.rendition(original, "acct/card/plain.jpg;300x213.webp").await.unwrap().is_some());
        let outside = RenditionSource {
            body: Body::Empty,
            size: 0,
            name: "../plain.jpg".to_owned(),
        };
        let failure = processor.rendition(outside, "acct/card/plain.jpg;300x213.webp").await.err().unwrap();
        assert_eq!(failure, Error::status(400, "Invalid media path"));
    });
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0, "the temporary directories are removed");
}
