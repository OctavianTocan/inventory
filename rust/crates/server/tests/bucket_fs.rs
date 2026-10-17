//! Frozen directory-bucket expectations retain all generated operation answers and the complete final file tree.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};

use mm_core::host::{Bucket, ByteSpan};
use mm_core::http::{Body, Headers};
use mm_core::{Error, Value, obj};
use mm_js::base64;
use mm_test_support::golden::Batch;
use mymind::bucket_fs::DirectoryBucket;

const DRIVER: &str = "rust/crates/server/tests/drivers/bucket.mjs";

// Represent bucket headers with the same object order as the reference snapshots.
fn headers_value(headers: Option<&Headers>) -> Value {
    headers.map_or(Value::Null, |headers| Value::Object(headers.iter().map(|(name, value)| (name.to_owned(), Value::from(value))).collect()))
}

// Execute one generated bucket operation without hiding errors or body bytes.
async fn apply(bucket: &DirectoryBucket, op: &Value) -> Result<Value, Error> {
    let key = op.get("key").as_str().unwrap_or("");
    Ok(match op.get("op").as_str().unwrap() {
        "write" => {
            let bytes = base64::decode(op.get("body").as_str().unwrap());
            let body = if op.get("stream").truthy() {
                let (first, rest) = bytes.split_at(bytes.len().min(3));
                let chunks = vec![Ok(first.to_vec()), Ok(rest.to_vec())];
                Body::Stream(Box::pin(futures_util::stream::iter(chunks)))
            } else {
                Body::Bytes(bytes)
            };
            let headers = op.get("headers").as_object().map(|object| Headers::from_pairs(object.iter().map(|(name, value)| (name.into_owned(), value.as_str().unwrap().to_owned()))));
            bucket.write(key, body, op.get("length").as_f64().map(|length| length as u64), headers.as_ref()).await?;
            Value::Null
        }
        "head" => match bucket.head(key).await? {
            None => Value::Null,
            Some(info) => obj! { "size" => info.size as f64, "headers" => headers_value(info.headers.as_ref()) }.into(),
        },
        "read" => {
            let range = op.get("range").as_object().map(|range| ByteSpan { offset: range.get("offset").unwrap().as_f64().unwrap() as u64, length: range.get("length").unwrap().as_f64().unwrap() as u64 });
            match bucket.read(key, range).await? {
                None => Value::Null,
                Some(object) => {
                    let body = Body::Stream(object.body).bytes().await?;
                    obj! { "size" => object.size as f64, "headers" => headers_value(object.headers.as_ref()), "body" => base64::encode(&body) }.into()
                }
            }
        }
        "bytes" => bucket.bytes(key).await?.map_or(Value::Null, |bytes| base64::encode(&bytes).into()),
        "remove" => {
            bucket.remove(key).await?;
            Value::Null
        }
        "list" => Value::Array(
            bucket.list(op.get("prefix").as_str().unwrap()).await?.into_iter().map(|object| obj! { "key" => object.key, "size" => object.size as f64 }.into()).collect(),
        ),
        other => panic!("unknown op {other}"),
    })
}

// Preserve the original status and invalid-input error distinctions.
fn answer(result: Result<Value, Error>) -> Value {
    match result {
        Ok(value) => obj! { "ok" => value }.into(),
        Err(error) => obj! { "error" => obj! { "status" => error.http_status().map_or(Value::Null, |status| f64::from(status).into()), "invalid" => matches!(error, Error::Invalid(_)) } }.into(),
    }
}

// Every file below `root` with its contents, keyed by relative path, skipping nothing.
fn tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push((path.strip_prefix(root).unwrap().to_string_lossy().into_owned(), std::fs::read(&path).unwrap()));
            }
        }
    }
    files.sort();
    files
}

// Give each backend an empty directory below the test target.
fn temp_root(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("bucket-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    root.join("bucket")
}

#[test]
fn directory_bucket_matches_node() {
    let keys = [
        "a", "a/b.txt", "a/b/c", "libraries/owner/media/x;100x100.webp", "libraries/owner/media/x;50x50.webp", "../escape", "/abs",
        "", ".headers/x", "a/./b", "a/../a/b.txt", "é/ü.bin", "😀", "\u{e000}", "x.tmp", "dir/", "a/b.txt/c", "a\0b", "static/134/app.js",
    ];
    let headers = [Value::Null, obj! { "content-type" => "image/webp" }.into(), obj! { "content-type" => "text/html", "x-magic" => "1" }.into()];
    let prefixes = ["", "a", "a/", "libraries/", "libraries/owner/media/x;", "../", "é", "x", "\u{e000}", "😀", ".headers/", "/"];
    let mut random = common::Random(0xb0c4_e7f5);
    let mut ops: Vec<Value> = Vec::new();
    for _ in 0..600 {
        let key = keys[random.below(keys.len() as u64) as usize];
        let op: Value = match random.below(7) {
            0 | 1 => {
                let length = random.below(40) as usize;
                let body: Vec<u8> = (0..length).map(|_| random.below(256) as u8).collect();
                obj! { "op" => "write", "key" => key, "body" => base64::encode(&body), "stream" => random.below(2) == 0, "headers" => headers[random.below(3) as usize].clone() }.into()
            }
            2 => obj! { "op" => "head", "key" => key }.into(),
            3 => {
                let range: Value = if random.below(3) == 0 { Value::Null } else { obj! { "offset" => random.below(30) as f64, "length" => random.below(30) as f64 }.into() };
                obj! { "op" => "read", "key" => key, "range" => range }.into()
            }
            4 => obj! { "op" => "bytes", "key" => key }.into(),
            5 => obj! { "op" => "remove", "key" => key }.into(),
            _ => obj! { "op" => "list", "prefix" => prefixes[random.below(prefixes.len() as u64) as usize] }.into(),
        };
        ops.push(op);
    }
    let js_root = temp_root("js");
    let rust_root = temp_root("rust");
    assert_eq!(ops.len(), 600, "seeded bucket operation count");
    let expected = common::call(DRIVER, "run", vec![js_root.to_string_lossy().as_ref().into(), Value::Array(ops.clone())]);
    let bucket = DirectoryBucket::new(&rust_root);
    let rust: Vec<Value> = mymind::runtime::run(async {
        let mut out = Vec::new();
        for op in &ops {
            out.push(answer(apply(&bucket, op).await));
        }
        out
    });
    assert_eq!(rust.len(), ops.len(), "bucket answer count");
    common::assert_value(&Value::Array(rust), &expected, "all bucket operation answers");
    let request = common::text(&Value::Array(ops)).into_bytes();
    let mut files = Batch::new(env!("CARGO_MANIFEST_DIR"), "bucket_fs", "directory-tree", &[request]);
    let snapshot = |root: &Path| {
        let files = tree(root).into_iter().map(|(path, bytes)| Value::Array(vec![path.into(), base64::encode(&bytes).into()])).collect();
        common::text(&Value::Array(files)).into_bytes()
    };
    let rust_files = snapshot(&rust_root);
    let oracle_files = common::recording().then(|| snapshot(&js_root));
    files.check(0, &rust_files, oracle_files.as_deref());
    files.finish();
    for root in [js_root, rust_root] {
        let _ = std::fs::remove_dir_all(root.parent().unwrap());
    }
}
