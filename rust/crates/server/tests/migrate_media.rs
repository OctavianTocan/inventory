//! Tests for `mymind migrate-media` (U1): local mode and bucket mode on a synthetic library.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::Store;
use mm_js::{Value, json};
use mm_sqlite::Sqlite;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("migrate-media-test").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

fn build_synthetic_library(root: &Path) -> PathBuf {
    let db_path = root.join("data/mymind.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let db = std::rc::Rc::new(Sqlite::open(&db_path, false).unwrap());
    let _store = Store::new(db.clone()).unwrap();
    db.execute(
        "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES ('u1', 'acct', '0000', 0, '{}')",
        &[],
    )
    .unwrap();

    // Card 1: Image card with renditions only (no original file).
    db.execute(
        "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
         VALUES ('0000c1', 'u1', 'Image', 0, ?, ?, ?, 100, 100, 100)",
        &[
            SqlValue::Text(r#"{"type":"Image","title":"One","blob":{"path":"acct/c1/b1","type":"image/webp","width":800,"height":600}}"#.into()),
            SqlValue::Text(r#"<mymind-card data-id="0000c1"><img data-src="/media/acct/c1/b1;400x300.webp"/></mymind-card>"#.into()),
            SqlValue::Text(r#"{"id":"0000c1","cover":{"src":"/media/acct/c1/b1;400x300.webp","width":400,"height":300}}"#.into()),
        ],
    )
    .unwrap();
    write_file(&root.join("capture/media/acct/c1/b1;400x300.webp"), b"small-rendition-400x300");
    write_file(&root.join("capture/media/acct/c1/b1;800x600.webp"), b"larger-rendition-800x600-bytes");
    // Card 2: SoftwareApplication card with signed media.mymind.host links in html and extras.
    db.execute(
        "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
         VALUES ('0000c2', 'u1', 'SoftwareApplication', 0, ?, ?, ?, 200, 200, 200)",
        &[
            SqlValue::Text(r#"{"type":"SoftwareApplication","title":"App"}"#.into()),
            SqlValue::Text(r#"<mymind-card data-id="0000c2"><img srcset="https://media.mymind.host/acct/c2/b2;250x250.webp?Expires=1792141200&amp;Signature=sig1 1x, https://media.mymind.host/acct/c2/b2;500x500.webp?Expires=1792141200&amp;Signature=sig2 2x"/></mymind-card>"#.into()),
            SqlValue::Text(r#"{"id":"0000c2","app":{"srcset":"https://media.mymind.host/acct/c2/b2;250x250.webp?Expires=1792141200&Signature=sig1 1x, https://media.mymind.host/acct/c2/b2;500x500.webp?Expires=1792141200&Signature=sig2 2x","width":250,"height":250}}"#.into()),
        ],
    )
    .unwrap();
    write_file(&root.join("capture/mediahost/acct/c2/b2;250x250.webp"), b"app-icon-250");
    write_file(&root.join("capture/mediahost/acct/c2/b2;500x500.webp"), b"app-icon-500-largest");

    // Card 3: Image card whose media file is completely missing from capture/.
    db.execute(
        "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
         VALUES ('0000c3', 'u1', 'Image', 0, ?, ?, ?, 300, 300, 300)",
        &[
            SqlValue::Text(r#"{"type":"Image","title":"Missing","blob":{"path":"acct/c3/missing","type":"image/png","width":100,"height":100}}"#.into()),
            SqlValue::Text(r#"<mymind-card data-id="0000c3"><img data-src="/media/acct/c3/missing;100x100.webp"/></mymind-card>"#.into()),
            SqlValue::Text(r#"{"id":"0000c3","cover":{"src":"/media/acct/c3/missing;100x100.webp","width":100,"height":100}}"#.into()),
        ],
    )
    .unwrap();

    db_path
}

#[test]
fn local_migration_copies_media_rewrites_links_reports_missing_and_is_idempotent() {
    let root = scratch("local");
    let db_path = build_synthetic_library(&root);
    let media_dir = root.join("data/media");
    let report_path = root.join("data/migrate-media-report.json");

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let args = vec![
        "local".to_owned(),
        "--db".to_owned(),
        db_path.to_string_lossy().into_owned(),
        "--media".to_owned(),
        media_dir.to_string_lossy().into_owned(),
        "--report".to_owned(),
        report_path.to_string_lossy().into_owned(),
    ];
    let status = runtime.block_on(mymind::tool_migrate_media::run(&root, &args));
    assert_eq!(status, 0);

    // Card 1: largest rendition picked as original, both renditions in .renditions.
    assert_eq!(std::fs::read(media_dir.join("acct/c1/b1.webp")).unwrap(), b"larger-rendition-800x600-bytes");
    assert_eq!(std::fs::read(media_dir.join(".renditions/acct_c1_b1;400x300.webp")).unwrap(), b"small-rendition-400x300");
    assert_eq!(std::fs::read(media_dir.join(".renditions/acct_c1_b1;800x600.webp")).unwrap(), b"larger-rendition-800x600-bytes");

    // Card 2: largest rendition picked as original, both renditions in .renditions, links rewritten in DB.
    assert_eq!(std::fs::read(media_dir.join("acct/c2/b2.webp")).unwrap(), b"app-icon-500-largest");
    assert_eq!(std::fs::read(media_dir.join(".renditions/acct_c2_b2;250x250.webp")).unwrap(), b"app-icon-250");
    assert_eq!(std::fs::read(media_dir.join(".renditions/acct_c2_b2;500x500.webp")).unwrap(), b"app-icon-500-largest");

    let db = Sqlite::open(&db_path, true).unwrap();
    let row = db.one("SELECT html, extras FROM cards WHERE id = '0000c2'", &[]).unwrap().unwrap();
    let html = row.text("html").unwrap();
    let extras = row.text("extras").unwrap();
    assert!(!html.contains("media.mymind.host"), "html still contains media.mymind.host: {html}");
    assert!(html.contains("/media/acct/c2/b2;250x250.webp 1x, /media/acct/c2/b2;500x500.webp 2x"), "html was not rewritten: {html}");
    assert!(!extras.contains("media.mymind.host"), "extras still contains media.mymind.host: {extras}");
    assert!(extras.contains("/media/acct/c2/b2;250x250.webp 1x, /media/acct/c2/b2;500x500.webp 2x"), "extras was not rewritten: {extras}");

    // Report lists Card 3 as missing.
    let report: Value = json::parse(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report.get("rewrittenCards").as_f64(), Some(1.0));
    assert_eq!(report.get("migratedOriginals").as_f64(), Some(2.0));
    assert_eq!(report.get("migratedRenditions").as_f64(), Some(4.0));
    let missing = report.get("missing").as_array().unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].get("cardId").as_str(), Some("0000c3"));
    assert_eq!(missing[0].get("path").as_str(), Some("acct/c3/missing"));

    // Second run changes nothing.
    let status2 = runtime.block_on(mymind::tool_migrate_media::run(&root, &args));
    assert_eq!(status2, 0);
    let report2: Value = json::parse(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report2.get("rewrittenCards").as_f64(), Some(0.0));
    assert_eq!(report2.get("migratedOriginals").as_f64(), Some(0.0));
    assert_eq!(report2.get("migratedRenditions").as_f64(), Some(0.0));
    assert_eq!(report2.get("missing").as_array().unwrap().len(), 1);
}

#[test]
fn bucket_migration_uploads_objects_and_skips_existing_on_second_run() {
    let root = scratch("bucket");
    let db_path = build_synthetic_library(&root);
    let media_dir = root.join("data/media");
    let report_path = root.join("data/bucket-report.json");
    write_file(&root.join(".env"), b"ADMIN_TOKEN=test-admin-secret\n");

    let put_count = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let put_clone = Arc::clone(&put_count);

    let server = std::thread::spawn(move || {
        let mut store: HashMap<String, Vec<u8>> = HashMap::new();
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            let mut header_end = None;
            let mut content_length = 0usize;
            loop {
                let n = stream.read(&mut chunk).unwrap_or(0);
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if header_end.is_none()
                    && let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n")
                {
                    header_end = Some(pos + 4);
                    let headers_str = String::from_utf8_lossy(&buf[..pos]);
                    for line in headers_str.lines() {
                        if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = rest.trim().parse().unwrap_or(0);
                        }
                    }
                }
                if let Some(end) = header_end
                    && buf.len() >= end + content_length
                {
                    break;
                }
            }
            let Some(end) = header_end else { continue };
            let head_str = String::from_utf8_lossy(&buf[..end - 4]);
            let first_line = head_str.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let (method, path) = (parts[0], parts[1]);
            if method == "POST" && path == "/__stop" {
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                break;
            }
            let authorized = head_str.lines().any(|line| line.eq_ignore_ascii_case("authorization: Bearer test-admin-secret"));
            if !authorized {
                let _ = stream.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                continue;
            }
            let key = path.strip_prefix("/_admin/bucket/").unwrap_or("").replace("%3B", ";").replace("%28", "(").replace("%29", ")");
            match method {
                "HEAD" => {
                    if let Some(bytes) = store.get(&key) {
                        let reply = format!("HTTP/1.1 200 OK\r\nx-object-size: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", bytes.len());
                        let _ = stream.write_all(reply.as_bytes());
                    } else {
                        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    }
                }
                "PUT" => {
                    let body = buf[end..end + content_length].to_vec();
                    store.insert(key, body);
                    put_clone.fetch_add(1, Ordering::SeqCst);
                    let _ = stream.write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                }
                _ => {
                    let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                }
            }
        }
        store
    });

    let url = format!("http://127.0.0.1:{port}");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let args = vec![
        "bucket".to_owned(),
        url.clone(),
        "--library".to_owned(),
        "owner".to_owned(),
        "--db".to_owned(),
        db_path.to_string_lossy().into_owned(),
        "--media".to_owned(),
        media_dir.to_string_lossy().into_owned(),
        "--report".to_owned(),
        report_path.to_string_lossy().into_owned(),
    ];
    let status = runtime.block_on(mymind::tool_migrate_media::run(&root, &args));
    assert_eq!(status, 0);
    assert_eq!(put_count.load(Ordering::SeqCst), 6);

    // Second run uploads nothing because HEAD reports matching sizes.
    let status2 = runtime.block_on(mymind::tool_migrate_media::run(&root, &args));
    assert_eq!(status2, 0);
    assert_eq!(put_count.load(Ordering::SeqCst), 6);

    if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
        let _ = stream.write_all(b"POST /__stop HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n");
    }
    let map = server.join().unwrap();
    assert_eq!(map.get("libraries/owner/media/acct/c1/b1.webp").map(Vec::as_slice), Some(&b"larger-rendition-800x600-bytes"[..]));
    assert_eq!(map.get("libraries/owner/renditions/acct_c1_b1;400x300.webp").map(Vec::as_slice), Some(&b"small-rendition-400x300"[..]));
    assert_eq!(map.get("libraries/owner/media/acct/c2/b2.webp").map(Vec::as_slice), Some(&b"app-icon-500-largest"[..]));
    assert_eq!(map.get("libraries/owner/renditions/acct_c2_b2;500x500.webp").map(Vec::as_slice), Some(&b"app-icon-500-largest"[..]));
}
