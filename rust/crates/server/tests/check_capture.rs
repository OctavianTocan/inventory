//! Executable ports of the original capture, capture-quality and recorded save-lifecycle assertions.

mod capture_support;

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use capture_support::{Harness, Saved, ScratchTemp, Shot, doc, flags};
use mm_core::ai::{Ai, AiImage, AiItem, EmbedKind, VocabularyEntry};
use mm_core::ai_tags::{curate_tags, tag_key};
use mm_core::capture_html::{
    capture_title, detect_type, page_metadata, plain_text, profile_image, sanitize_post,
};
use mm_core::host::{Resource, ResourceOptions, Screenshot};
use mm_core::http::{Body, Request};
use mm_core::render::encoding::escape_html;
use mm_core::render_card::render_card;
use mm_core::site::LibraryApi;
use mm_core::sql::SqlExt;
use mm_core::{Value, obj};
use mm_js::json;
use mymind::screenshot::ChromiumScreenshot;
use mymind::tool_capture_fixtures::CASES;
use regex::Regex;

static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

macro_rules! verify {
    ($source:literal, $condition:expr) => {
        assert!($condition, "original assertion {}", $source)
    };
}
macro_rules! same {
    ($source:literal, $actual:expr, $expected:expr) => {
        assert_eq!($actual, $expected, "original assertion {}", $source)
    };
}

// Read a field that the original suite treated as a string.
fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).as_str().unwrap_or("")
}

// Read the names of stored tags without changing their spelling or order.
fn tag_names(tags: &Value) -> Vec<String> {
    tags.as_array()
        .unwrap()
        .iter()
        .map(|tag| text(tag, "name").to_owned())
        .collect()
}

// Run the original inclusive two-chunk upload and consume its capability through the real API.
async fn upload(h: &Harness, bytes: &[u8], mime: &str, name: &str) -> (String, String) {
    let auth = h
        .request("POST", "/uploads/authorize", b"", &[])
        .await
        .value();
    let split = (bytes.len() / 2).max(1);
    let mut upload_id = String::new();
    for (start, end) in [(0, split - 1), (split, bytes.len() - 1)] {
        let bearer = format!("Bearer {}", text(&auth, "token"));
        let range = format!("bytes {start}-{end}/{}", bytes.len());
        let encoded = mm_js::uri::encode_uri(name);
        let mut headers = vec![
            ("content-type", mime),
            ("authorization", bearer.as_str()),
            ("x-file-name", encoded.as_str()),
            ("content-range", range.as_str()),
        ];
        if !upload_id.is_empty() {
            headers.push(("x-upload-id", upload_id.as_str()));
        }
        let result = h
            .request("PUT", text(&auth, "url"), &bytes[start..=end], &headers)
            .await;
        same!("check-capture.mjs:90", result.status, 200);
        let value = result.value();
        same!(
            "check-capture.mjs:92",
            value.get("received").as_f64(),
            Some((end + 1) as f64)
        );
        upload_id = text(value.get("blob"), "id").to_owned();
    }
    let created = h
        .json(
            "POST",
            "/objects",
            obj! { "type" => "Media", "uploadId" => upload_id.as_str() }.into(),
        )
        .await;
    same!("check-capture.mjs:95", created.status, 201);
    (text(&created.value(), "id").to_owned(), upload_id)
}

// Compare persisted lifecycle snapshots with the immutable original recording after synthetic substitutions.
fn lifecycle_stage(row: &Saved, change: &Value, id: &str) {
    let expected_data = json::parse(text(change, "cardData")).unwrap();
    same!(
        "check-link-lifecycle.mjs:91",
        row.data.get("type"),
        expected_data.get("type")
    );
    same!(
        "check-link-lifecycle.mjs:92",
        row.data.get("flags"),
        expected_data.get("flags")
    );
    let mut expected = text(change, "cardHtml").to_owned();
    for (from, to) in [
        (text(change.get("object"), "id"), id),
        (
            text(expected_data.get("source"), "url"),
            text(row.data.get("source"), "url"),
        ),
        (text(&expected_data, "domain"), text(&row.data, "domain")),
    ] {
        expected = expected.replace(&escape_html(from), &escape_html(to));
    }
    verify!("check-link-lifecycle.mjs:96", row.html == expected);
}

// Preserve the three-stage snapshots and all five real SignalR invocation payloads in order.
async fn lifecycle(h: &Harness) {
    h.env(true, false);
    let recorded = json::parse(include_str!("../../core/tests/fixtures/link_save.json")).unwrap();
    let changes = recorded.get("changes").as_array().unwrap();
    let expected_events = recorded.get("events").as_array().unwrap();
    let before = h.fixtures.requests().len();
    let event_start = h.wire.messages().len();
    let snapshot_start = h.snapshots.borrow().len();
    let shots = Rc::new(Shot {
        png: h.media_bytes.png.clone(),
        calls: Cell::new(0),
        fail: true,
    });
    let jobs = h.jobs(shots.clone());
    let id = h.create(obj! { "content" => format!("{}/failed.apk", h.fixtures.base) }.into());
    {
        let snapshots = h.snapshots.borrow();
        verify!(
            "check-link-lifecycle.mjs:89",
            snapshots.get(snapshot_start).is_some()
        );
        lifecycle_stage(&snapshots[snapshot_start].2, &changes[0], &id);
    }
    jobs.schedule(&id, true);
    jobs.drain().await;
    {
        let snapshots = h.snapshots.borrow();
        let updated = snapshots[snapshot_start..]
            .iter()
            .find(|(target, _, _)| target == "object:updated");
        verify!("check-link-lifecycle.mjs:89", updated.is_some());
        lifecycle_stage(&updated.unwrap().2, &changes[1], &id);
        let failed = snapshots[snapshot_start..]
            .iter()
            .find(|(target, _, _)| target == "screenshot:failed");
        verify!("check-link-lifecycle.mjs:89", failed.is_some());
        lifecycle_stage(&failed.unwrap().2, &changes[2], &id);
    }
    let actual = h.wire.messages()[event_start..].to_vec();
    let replacement = text(changes[0].get("object"), "id");
    let expected: Vec<Value> = expected_events
        .iter()
        .map(|event| json::parse(&json::to_string(event).replace(replacement, &id)).unwrap())
        .collect();
    let failures = |events: &[Value]| {
        events
            .iter()
            .filter(|event| ["ai:failed", "exception"].contains(&text(event, "target")))
            .cloned()
            .collect::<Vec<_>>()
    };
    verify!(
        "check-link-lifecycle.mjs:106",
        json::to_string(&Value::Array(failures(&actual)))
            == json::to_string(&Value::Array(failures(&expected)))
    );
    same!("check-link-lifecycle.mjs:110", shots.calls.get(), 1);
    same!(
        "check-link-lifecycle.mjs:111",
        h.fixtures.requests()[before..]
            .iter()
            .filter(|request| request.path.contains("completions")
                || request.path.contains("embedContent"))
            .count(),
        0
    );
    for (actual, expected) in actual.iter().zip(&expected) {
        verify!(
            "check-link-lifecycle.mjs:115",
            json::to_string(actual) == json::to_string(expected)
        );
    }
    same!(
        "check-link-lifecycle.mjs:119",
        actual
            .iter()
            .map(|event| text(event, "target"))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|event| text(event, "target"))
            .collect::<Vec<_>>()
    );
    let requests = h.fixtures.requests().len();
    jobs.schedule(&id, true);
    jobs.drain().await;
    same!(
        "check-link-lifecycle.mjs:122",
        h.fixtures.requests().len(),
        requests
    );
    same!(
        "check-link-lifecycle.mjs:123",
        h.wire.messages().len() - event_start,
        expected_events.len()
    );
    lifecycle_stage(&h.row(&id), &changes[2], &id);
    let other =
        h.create(obj! { "content" => format!("{}/NoImage?failed=1", h.fixtures.base) }.into());
    jobs.schedule(&other, true);
    jobs.drain().await;
    let saved = h.row(&other);
    same!(
        "check-link-lifecycle.mjs:130",
        text(&saved.data, "type"),
        "WebPage"
    );
    same!(
        "check-link-lifecycle.mjs:130",
        saved.data.get("flags").as_f64(),
        Some(0.0)
    );
    verify!(
        "check-link-lifecycle.mjs:131",
        saved.html.contains("inaccessible-guts")
    );
    verify!(
        "check-link-lifecycle.mjs:132",
        !saved.extras.get("screenshotPending").truthy()
            && !saved.extras.get("screenshotFailed").truthy()
    );
    same!(
        "check-link-lifecycle.mjs:133",
        text(&saved.data, "summary"),
        "A fixture summary <safe>."
    );
    verify!(
        "check-link-lifecycle.mjs:134",
        !h.wire
            .messages()
            .iter()
            .any(|event| ["ai:failed", "exception", "screenshot:failed"]
                .contains(&text(event, "target"))
                && [
                    event.get("arguments").at(0).get("id"),
                    event.get("arguments").at(0).get("objectId")
                ]
                .iter()
                .any(|id| id.as_str() == Some(other.as_str())))
    );
    jobs.close();
    h.env(false, false);
}

// Retain metadata, persisted rendering, stored covers, reader text and delivered hub updates for every case.
async fn capture_cases(h: &Harness) -> String {
    let color = Regex::new("^#[0-9a-f]{6}$").unwrap();
    let unsafe_post = Regex::new("script|onclick|javascript:|<img").unwrap();
    let mut article = String::new();
    for [name, url, class_name, structure] in CASES {
        let id = h.save(name).await;
        let item = h.captured(&id).await;
        let kind = if name == "SoftwareApplicationPlay" {
            "SoftwareApplication"
        } else {
            name
        };
        let metadata = if name == "Product" {
            obj! { "entities" => vec![Value::from(obj! { "@type" => "Product" })] }.into()
        } else if name == "Article" {
            obj! { "meta" => obj! { "og:type" => vec![Value::from("article")] } }.into()
        } else {
            obj! {}.into()
        };
        same!(
            "check-capture.mjs:109",
            detect_type(url, &metadata, "").unwrap(),
            kind
        );
        same!("check-capture.mjs:110", text(&item.data, "type"), kind);
        let title = if kind == "Repository" {
            "repo"
        } else {
            "Fixture title"
        };
        same!("check-capture.mjs:111", text(&item.data, "title"), title);
        same!(
            "check-capture.mjs:112",
            text(&item.data, "description"),
            "Fixture description"
        );
        same!(
            "check-capture.mjs:113",
            text(&item.data, "domain"),
            mm_js::url::parse(url, None).unwrap().host_str().unwrap()
        );
        same!("check-capture.mjs:114", text(&item.extras, "linkUrl"), url);
        same!("check-capture.mjs:115", text(&item.extras, "title"), title);
        verify!(
            "check-capture.mjs:116",
            text(&item.extras, "captureText").contains(
                if ["XPost", "ThreadsPost", "SubstackNote"].contains(&kind) {
                    "Fixture post body"
                } else {
                    "Fixture body text"
                }
            )
        );
        verify!(
            "check-capture.mjs:117",
            item.html.contains(&format!("class=\"{class_name}\""))
        );
        verify!("check-capture.mjs:118", item.html.contains(structure));
        same!(
            "check-capture.mjs:119",
            item.html.as_str(),
            render_card(&item.data, &item.extras).unwrap()
        );
        let cover = item.extras.get("cover");
        verify!(
            "check-capture.mjs:120",
            text(cover, "src").starts_with("/media/")
        );
        same!(
            "check-capture.mjs:121",
            cover.get("width").as_f64(),
            Some(640.0)
        );
        same!(
            "check-capture.mjs:122",
            cover.get("height").as_f64(),
            Some(480.0)
        );
        verify!(
            "check-capture.mjs:123",
            color.is_match(text(cover, "dominantColor"))
        );
        if ["XPost", "ThreadsPost", "SubstackNote"].contains(&kind) {
            let post = item.extras.get("post");
            same!(
                "check-capture.mjs:125",
                text(post, "author"),
                "by Fixture author"
            );
            verify!(
                "check-capture.mjs:126",
                text(post, "bodyHtml").contains("<strong>post</strong>")
            );
            verify!(
                "check-capture.mjs:127",
                !unsafe_post.is_match(text(post, "bodyHtml"))
            );
            verify!("check-capture.mjs:128", post.get("gallery").is_array());
        }
        if kind == "Product" {
            same!(
                "check-capture.mjs:130",
                text(item.extras.get("product"), "price"),
                "USD 12.50"
            );
            same!(
                "check-capture.mjs:130",
                text(&item.data, "brand"),
                "Fixture brand"
            );
        }
        if kind == "RedditPost" {
            same!(
                "check-capture.mjs:131",
                text(item.extras.get("reddit"), "subreddit"),
                "fixtures"
            );
        }
        if kind == "SoftwareApplication" {
            verify!(
                "check-capture.mjs:132",
                text(item.extras.get("app"), "srcset").contains("/media/")
            );
        }
        if kind == "Article" {
            verify!("check-capture.mjs:134", flags(&item.data) & 2 != 0);
            let detail = h
                .request(
                    "GET",
                    &format!("/cards/{id}/detail"),
                    b"",
                    &[("accept", "text/html")],
                )
                .await;
            verify!(
                "check-capture.mjs:136",
                detail.text().contains("Fixture body text")
            );
            article = id.clone();
        }
        h.until(|| {
            h.wire.messages().iter().any(|message| {
                text(message, "target") == "object:updated"
                    && message.get("arguments").at(0).get("id").as_str() == Some(id.as_str())
            })
        })
        .await;
        verify!(
            "check-capture.mjs:138",
            h.wire
                .messages()
                .iter()
                .any(|message| text(message, "target") == "object:updated"
                    && message.get("arguments").at(0).get("id").as_str() == Some(id.as_str()))
        );
    }
    let item = h.captured(&h.save("Gallery").await).await;
    same!("check-capture.mjs:144", text(&item.data, "type"), "XPost");
    same!(
        "check-capture.mjs:145",
        item.extras
            .get("post")
            .get("gallery")
            .as_array()
            .unwrap()
            .len(),
        2
    );
    verify!("check-capture.mjs:146", !item.extras.get("cover").truthy());
    verify!(
        "check-capture.mjs:147",
        item.html.contains("gallery-wrap gallery-grid")
    );
    same!(
        "check-capture.mjs:148",
        item.html.matches("<img class=\"lazy\"").count(),
        2
    );
    let item = h.captured(&h.save("JsonLd").await).await;
    same!(
        "check-capture.mjs:153",
        text(&item.data, "title"),
        "JSON-LD title"
    );
    same!("check-capture.mjs:154", text(&item.data, "type"), "Article");
    same!(
        "check-capture.mjs:155",
        text(&item.data, "description"),
        "JSON-LD description"
    );
    same!(
        "check-capture.mjs:156",
        text(&item.data, "author"),
        "Schema author"
    );
    verify!("check-capture.mjs:157", item.extras.get("cover").truthy());
    same!("check-capture.mjs:158", page_metadata("<meta name=twitter:title content=\"Twitter title\"><meta name=twitter:image content=\"/cover.png\">", &h.fixtures.base).unwrap().title.as_str(), Some("Twitter title"));
    let clean = sanitize_post(
        "<p onclick=x>Hi &lt;script&gt;<svg><script>x()</script></svg><a href=\"jav&#x61;script:alert(1)\">bad</a><a href=\"/safe\" style=x>ok</a><span>text</span></p>",
        &h.fixtures.base,
    );
    verify!(
        "check-capture.mjs:160",
        !Regex::new("onclick|<script|<svg|javascript:|style=")
            .unwrap()
            .is_match(&clean)
    );
    verify!("check-capture.mjs:161", clean.contains("&lt;script&gt;"));
    verify!(
        "check-capture.mjs:162",
        clean.contains(&format!("href=\"{}/safe\"", h.fixtures.base))
    );
    let item = h.captured(&h.save("doc.pdf").await).await;
    same!(
        "check-capture.mjs:167",
        text(&item.data, "type"),
        "Document"
    );
    same!(
        "check-capture.mjs:168",
        text(item.data.get("blob"), "type"),
        "application/pdf"
    );
    verify!(
        "check-capture.mjs:169",
        item.html.contains("document-background")
    );
    let result = h
        .request(
            "GET",
            &format!(
                "/media/{};background(fff)/page(1)/642x840-t.webp",
                text(item.data.get("blob"), "path")
            ),
            b"",
            &[("accept", "image/webp")],
        )
        .await;
    same!(
        "check-capture.mjs:171",
        result.headers.get("content-type"),
        Some("image/webp")
    );
    let info = mymind::vips::metadata(&result.bytes, 0).unwrap();
    verify!("check-capture.mjs:173", info.width > 0 && info.height > 0);
    for name in ["blocked", "redirect-account"] {
        let item = h.captured(&h.save(name).await).await;
        same!(
            "check-capture.mjs:179",
            text(&item.data, "type"),
            "Placeholder"
        );
        verify!(
            "check-capture.mjs:180",
            item.html.contains("inaccessible-guts") && item.html.contains("blocking us")
        );
        verify!(
            "check-capture.mjs:181",
            text(item.data.get("source"), "url").ends_with(name)
        );
    }
    for (line, url) in [
        (183, "https://access.mymind.com/everything"),
        (184, "https://access.mymind.com./everything"),
    ] {
        let error = h
            .resource
            .fetch(
                url,
                ResourceOptions {
                    offline: false,
                    limit: None,
                },
            )
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("Unsupported"),
            "check-capture.mjs:{line}"
        );
    }
    article
}

// Preserve exact originals, disk persistence, real renditions, multipart, attachments and capability validation.
async fn media(h: &Harness) -> (String, String) {
    let preview_url = Regex::new(r#"(?:data-src|src)="(/media/[^" ]+\.(?:jpeg|webp))""#).unwrap();
    let video_url = Regex::new(r#"src="(/media/[^" ]+/hevc.mp4)""#).unwrap();
    let (image_id, image_upload) =
        upload(h, &h.media_bytes.png, "image/png", "fixture image.png").await;
    let item = h.row(&image_id);
    same!("check-capture.mjs:192", text(&item.data, "type"), "Image");
    let blob = item.data.get("blob");
    same!(
        "check-capture.mjs:193",
        blob.get("width").as_f64(),
        Some(640.0)
    );
    same!(
        "check-capture.mjs:194",
        blob.get("height").as_f64(),
        Some(480.0)
    );
    verify!(
        "check-capture.mjs:195",
        item.html.contains("class=\"image\"")
    );
    same!(
        "check-capture.mjs:196",
        h.request("GET", &format!("/media/{}", text(blob, "path")), b"", &[])
            .await
            .bytes,
        h.media_bytes.png
    );
    same!(
        "check-capture.mjs:197",
        std::fs::read(h.dir.join("media").join(text(blob, "path"))).unwrap(),
        h.media_bytes.png
    );
    for (suffix, format, width, height) in [
        (";100x80-c.webp", "webp", 100, 80),
        (";100x80-t.webp", "webp", 100, 75),
        (";100x80.jpeg", "jpeg", 100, 75),
    ] {
        let result = h
            .request(
                "GET",
                &format!("/media/{}{suffix}", text(blob, "path")),
                b"",
                &[],
            )
            .await;
        same!("check-capture.mjs:200", result.status, 200);
        let info = mymind::vips::metadata(&result.bytes, 0).unwrap();
        same!(
            "check-capture.mjs:202",
            (info.format, info.width, info.height),
            (format, width, height)
        );
    }
    same!(
        "check-capture.mjs:204",
        h.json(
            "POST",
            "/objects",
            obj! { "type" => "Media", "uploadId" => image_upload.as_str() }.into()
        )
        .await
        .status,
        200
    );
    same!(
        "check-capture.mjs:205",
        h.request(
            "GET",
            &format!("/media/{};9000x9000.webp", text(blob, "path")),
            b"",
            &[]
        )
        .await
        .status,
        400
    );
    for (extension, mime, kind, bytes) in [
        ("png", "image/png", "Image", &h.media_bytes.png),
        ("pdf", "application/pdf", "Document", &h.media_bytes.pdf),
    ] {
        let boundary = "capture-synthetic-boundary";
        let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"; filename=\"blob\"\r\nContent-Type: application/json\r\n\r\n{{\"source\":\"clipboard\",\"title\":\"Pasted fixture\"}}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"blob\"; filename=\"fixture.{extension}\"\r\nContent-Type: {mime}\r\n\r\n").into_bytes();
        body.extend(bytes);
        body.extend(format!("\r\n--{boundary}--\r\n").as_bytes());
        let result = h
            .request(
                "POST",
                "/objects",
                &body,
                &[(
                    "content-type",
                    &format!("multipart/form-data; boundary={boundary}"),
                )],
            )
            .await;
        same!("check-capture.mjs:215", result.status, 201);
        let saved = h.row(text(&result.value(), "id"));
        same!(
            "check-capture.mjs:217",
            (text(&saved.data, "type"), text(&saved.data, "title")),
            (kind, "Pasted fixture")
        );
        same!(
            "check-capture.mjs:218",
            text(saved.data.get("source"), "name"),
            "clipboard"
        );
    }
    for (body, mime) in [
        (h.media_bytes.png.clone(), "image/png"),
        (
            json::to_string(
                &obj! { "remoteUrl" => format!("{}/cover.png", h.fixtures.base) }.into(),
            )
            .into_bytes(),
            "application/json",
        ),
        (
            json::to_string(&obj! { "uploadId" => image_upload.as_str() }.into()).into_bytes(),
            "application/json",
        ),
    ] {
        let result = h
            .request(
                "POST",
                &format!("/objects/{image_id}/attachments"),
                &body,
                &[("content-type", mime)],
            )
            .await;
        same!("check-capture.mjs:222", result.status, 200);
        let value = result.value();
        same!(
            "check-capture.mjs:223",
            (value.get("width").as_f64(), value.get("height").as_f64()),
            (Some(640.0), Some(480.0))
        );
        verify!(
            "check-capture.mjs:224",
            text(&value, "path").starts_with("/media/")
        );
        same!(
            "check-capture.mjs:225",
            h.request("GET", text(&value, "path"), b"", &[]).await.bytes,
            h.media_bytes.png
        );
    }
    for (extension, mime, kind, selector, bytes) in [
        (
            "mp4",
            "video/mp4",
            "Video",
            "hover-preview",
            &h.media_bytes.mp4,
        ),
        (
            "pdf",
            "application/pdf",
            "Document",
            "document-background",
            &h.media_bytes.pdf,
        ),
    ] {
        let (id, _) = upload(h, bytes, mime, &format!("fixture.{extension}")).await;
        let saved = h.row(&id);
        same!("check-capture.mjs:234", text(&saved.data, "type"), kind);
        let blob = saved.data.get("blob");
        verify!(
            "check-capture.mjs:235",
            blob.get("width").to_number() > 0.0 && blob.get("height").to_number() > 0.0
        );
        verify!("check-capture.mjs:236", saved.html.contains(selector));
        let range = h
            .request(
                "GET",
                &format!("/media/{}", text(blob, "path")),
                b"",
                &[("range", "bytes=2-8")],
            )
            .await;
        same!("check-capture.mjs:238", range.status, 206);
        same!(
            "check-capture.mjs:238",
            range.bytes.as_slice(),
            &bytes[2..9]
        );
        same!(
            "check-capture.mjs:239",
            range.headers.get("content-range"),
            Some(format!("bytes 2-8/{}", bytes.len()).as_str())
        );
        let preview = preview_url
            .captures(&saved.html)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_owned();
        let result = h.request("GET", &preview, b"", &[]).await;
        same!("check-capture.mjs:242", result.status, 200);
        verify!(
            "check-capture.mjs:242",
            mymind::vips::metadata(&result.bytes, 0).unwrap().width > 0
        );
        if kind == "Video" {
            let video = video_url
                .captures(&saved.html)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_owned();
            let rendered = h.request("GET", &video, b"", &[]).await;
            same!("check-capture.mjs:246", rendered.status, 200);
            same!(
                "check-capture.mjs:246",
                rendered.headers.get("content-type"),
                Some("video/mp4")
            );
            same!("check-capture.mjs:247", &rendered.bytes[4..8], b"ftyp");
        }
    }
    same!(
        "check-capture.mjs:253",
        h.request("POST", "/uploads/authorize?count=2", b"", &[])
            .await
            .value()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    same!(
        "check-capture.mjs:254",
        h.request("POST", "/uploads/authorize?count=101", b"", &[])
            .await
            .status,
        400
    );
    let auth = h
        .request("POST", "/uploads/authorize", b"", &[])
        .await
        .value();
    let upload_url = text(&auth, "url");
    let id = upload_url.rsplit('/').next().unwrap();
    same!(
        "check-capture.mjs:257",
        h.json(
            "POST",
            "/objects",
            obj! { "type" => "Media", "uploadId" => id }.into()
        )
        .await
        .status,
        409
    );
    let bearer = format!("Bearer {}", text(&auth, "token"));
    // Build upload headers for one synthetic chunk.
    fn chunk<'a>(range: &'a str, token: &'a str) -> [(&'static str, &'a str); 3] {
        [
            ("content-type", "image/png"),
            ("authorization", token),
            ("content-range", range),
        ]
    }
    same!(
        "check-capture.mjs:259",
        h.request(
            "PUT",
            upload_url,
            b"abc",
            &chunk("bytes 0-2/6", "Bearer invalid")
        )
        .await
        .status,
        401
    );
    same!(
        "check-capture.mjs:260",
        h.request("PUT", upload_url, b"abc", &chunk("bytes 0-2/6", &bearer))
            .await
            .status,
        200
    );
    same!(
        "check-capture.mjs:261",
        h.request("PUT", upload_url, b"abc", &chunk("bytes 0-2/6", &bearer))
            .await
            .status,
        409
    );
    same!(
        "check-capture.mjs:262",
        h.request("PUT", upload_url, b"abc", &chunk("bytes 4-5/6", &bearer))
            .await
            .status,
        409
    );
    same!(
        "check-capture.mjs:263",
        h.request("PUT", upload_url, b"abc", &chunk("bytes 3-5/7", &bearer))
            .await
            .status,
        409
    );
    same!(
        "check-capture.mjs:264",
        h.request("PUT", upload_url, b"ab", &chunk("bytes 3-5/6", &bearer))
            .await
            .status,
        400
    );
    (image_id, image_upload)
}

// Preserve B1-B5, provider image selection, protected headlines and reader-edit reanalysis.
async fn quality(h: &Harness) {
    for (title, kind, canonical, site_name, expected) in [
        (
            "GitHub - owner/yap: Description",
            "Repository",
            "https://github.com/owner/yap",
            "GitHub",
            "yap",
        ),
        (
            "GitHub - owner/transitions.dev: Description",
            "Repository",
            "https://github.com/owner/transitions.dev",
            "GitHub",
            "transitions.dev",
        ),
        (
            "App Name - Apps on Google Play",
            "SoftwareApplication",
            "https://play.google.com/store/apps/details?id=test",
            "",
            "App Name",
        ),
        (
            "App Name on the App Store",
            "SoftwareApplication",
            "https://apps.apple.com/app/id1",
            "",
            "App Name",
        ),
        (
            "Mind map - Wikipedia",
            "Article",
            "https://en.wikipedia.org/wiki/Mind_map",
            "",
            "Mind map",
        ),
        (
            "Topic | Journal",
            "Article",
            "https://journal.invalid/topic",
            "Journal",
            "Topic",
        ),
        (
            "Topic — Journal",
            "WebPage",
            "https://journal.invalid/topic",
            "Journal",
            "Topic",
        ),
        (
            "Topic - A related subject",
            "WebPage",
            "https://journal.invalid/topic",
            "Journal",
            "Topic - A related subject",
        ),
    ] {
        same!("check-capture-quality.mjs:41", capture_title(title, kind, &obj! { "canonical" => canonical, "siteName" => site_name, "entities" => Vec::<Value>::new() }.into()).unwrap(), expected);
    }
    let post = h.save("XQuality").await;
    let item = h.captured(&post).await;
    same!(
        "check-capture-quality.mjs:48",
        text(&item.data, "type"),
        "XPost"
    );
    same!(
        "check-capture-quality.mjs:49",
        text(&item.data, "author"),
        "by Jack Example"
    );
    let body = text(item.extras.get("post"), "bodyHtml");
    verify!(
        "check-capture-quality.mjs:50",
        body.contains("&#x2014; hello &amp; welcome")
    );
    verify!(
        "check-capture-quality.mjs:51",
        !Regex::new("March 21|@jack|&amp;mdash;|&mdash;")
            .unwrap()
            .is_match(body)
    );
    same!(
        "check-capture-quality.mjs:52",
        text(&item.extras, "captureText"),
        "just setting up my twttr — hello & welcome"
    );
    same!(
        "check-capture-quality.mjs:53",
        plain_text(&sanitize_post(
            "<p>Caf&eacute; &mdash; &copy; &#x110000;</p>",
            &h.fixtures.base
        )),
        "Café — © �"
    );
    same!(
        "check-capture-quality.mjs:54",
        plain_text("&constructor;"),
        "&constructor;"
    );
    verify!(
        "check-capture-quality.mjs:55",
        !item.extras.get("cover").truthy()
    );
    same!(
        "check-capture-quality.mjs:55",
        item.extras.get("post").get("gallery"),
        &Value::Array(Vec::new())
    );
    verify!(
        "check-capture-quality.mjs:56",
        !h.fixtures
            .requests()
            .iter()
            .any(|request| request.path.contains("profile_images"))
    );
    for url in [
        "https://pbs.twimg.com/profile_images/1/photo.jpg",
        "https://cdn.invalid/profile-pictures/1.jpg",
        "https://avatars.invalid/u/1",
        "https://lh3.googleusercontent.com/a/avatar",
        "https://cdn.invalid/v/t51.2885-19/1.jpg",
    ] {
        verify!("check-capture-quality.mjs:57", profile_image(url).unwrap());
    }
    verify!(
        "check-capture-quality.mjs:58",
        !profile_image("https://pbs.twimg.com/media/post-image.jpg").unwrap()
    );
    let before = h.fixtures.requests().len();
    let mut request = Request::new("POST", &format!("{}/objects", h.base)).unwrap();
    request.headers.set("content-type", "application/json");
    request.body = Body::from(json::to_string(
        &obj! { "content" => format!("{}/ArticleQuality", h.fixtures.base) }.into(),
    ));
    let result = h.api.handle(&mut request).await.unwrap().unwrap();
    same!("check-capture.mjs:72", result.status, 201);
    same!("check-capture.mjs:73", h.fixtures.requests().len(), before);
    let article = text(
        &json::parse(std::str::from_utf8(&result.body.bytes().await.unwrap()).unwrap()).unwrap(),
        "id",
    )
    .to_owned();
    let pending = h.row(&article);
    same!(
        "check-capture-quality.mjs:66",
        text(&pending.data, "type"),
        "Placeholder"
    );
    same!(
        "check-capture-quality.mjs:67",
        pending.data.get("flags").as_f64(),
        Some(0.0)
    );
    verify!(
        "check-capture-quality.mjs:68",
        pending.html.contains("placeholder processing")
    );
    verify!(
        "check-capture-quality.mjs:69",
        pending.html.contains("head_spin.png")
    );
    verify!(
        "check-capture-quality.mjs:70",
        !Regex::new("inaccessible-guts|blocking us")
            .unwrap()
            .is_match(&pending.html)
    );
    same!(
        "check-capture-quality.mjs:71",
        h.fixtures.requests().len(),
        before
    );
    let mut detail = Request::new("GET", &format!("{}/cards/{article}/detail", h.base)).unwrap();
    detail.headers.set("accept", "text/html");
    let detail = h
        .api
        .handle(&mut detail)
        .await
        .unwrap()
        .unwrap()
        .body
        .bytes()
        .await
        .unwrap();
    verify!(
        "check-capture-quality.mjs:73",
        !String::from_utf8(detail).unwrap().contains("<mymind-tldr")
    );
    let item = h.captured(&article).await;
    same!(
        "check-capture-quality.mjs:76",
        text(&item.data, "title"),
        "Mind map"
    );
    verify!(
        "check-capture-quality.mjs:77",
        text(&item.extras, "captureText").contains("A mind map organizes ideas visually.")
    );
    verify!(
        "check-capture-quality.mjs:78",
        text(&item.extras, "captureText").contains("Mind maps connect concepts using branches")
    );
    verify!(
        "check-capture-quality.mjs:79",
        text(&item.extras, "captureText")
            .contains("Use central topics and connections to plan a project.")
    );
    verify!(
        "check-capture-quality.mjs:80",
        !text(&item.extras, "captureText").contains("Unrelated navigation")
    );
    verify!(
        "check-capture-quality.mjs:81",
        !item.html.contains("processing-guts")
    );
    let failed = h.captured(&h.save("blocked?quality=1").await).await;
    verify!(
        "check-capture-quality.mjs:83",
        failed.html.contains("blocking us")
    );
    verify!(
        "check-capture-quality.mjs:83",
        !failed.html.contains("processing-guts")
    );
    let tags: Vec<Value> = [
        "Design-Systems",
        "design_system",
        " DESIGN   SYSTEMS ",
        "cats",
        "cat",
    ]
    .iter()
    .map(|name| obj! { "name" => *name }.into())
    .collect();
    same!(
        "check-capture-quality.mjs:88",
        curate_tags(
            &tags,
            "Article",
            &["Design Systems".into(), "Cat".into()],
            &[]
        )
        .unwrap()
        .iter()
        .map(|tag| text(tag, "name"))
        .collect::<Vec<_>>(),
        ["Design Systems", "Cat"]
    );
    same!(
        "check-capture-quality.mjs:89",
        curate_tags(
            &tags,
            "Article",
            &[],
            &["design systems".into(), "cat".into()]
        )
        .unwrap()
        .len(),
        0
    );
    for (kind, redundant) in [
        ("YouTubeVideo", vec!["video", "videos"]),
        ("Video", vec!["videos"]),
        ("Document", vec!["pdf", "document", "files"]),
        ("WebPage", vec!["web_page", "webpage", "web-pages"]),
        (
            "SoftwareApplication",
            vec!["app", "applications", "mobile-apps"],
        ),
        ("XPost", vec!["tweets", "posts"]),
        ("ThreadsPost", vec!["post"]),
        ("SubstackNote", vec!["post"]),
    ] {
        let tags: Vec<Value> = redundant
            .into_iter()
            .chain(["screenshot", "architecture"])
            .map(|name| obj! { "name" => name }.into())
            .collect();
        same!(
            "check-capture-quality.mjs:93",
            curate_tags(&tags, kind, &[], &[])
                .unwrap()
                .iter()
                .map(|tag| text(tag, "name"))
                .collect::<Vec<_>>(),
            ["architecture"]
        );
    }
    same!(
        "check-capture-quality.mjs:95",
        curate_tags(
            &[obj! { "name" => "screenshots" }.into()],
            "Screenshot",
            &[],
            &[]
        )
        .unwrap()
        .iter()
        .map(|tag| text(tag, "name"))
        .collect::<Vec<_>>(),
        ["screenshots"]
    );
    let tags: Vec<Value> = (0..15)
        .map(|i| obj! { "name" => format!("topic {i}") }.into())
        .collect();
    same!(
        "check-capture-quality.mjs:96",
        curate_tags(&tags, "Article", &[], &[]).unwrap().len(),
        10
    );
    let id = h.create(obj! { "type" => "Note", "content" => "Tag fixture" }.into());
    h.writer
        .run(|writer| writer.add_tag(&id, &"manual-system".into()))
        .unwrap();
    let tags: Vec<Value> = std::iter::once(obj! { "name" => "manual_systems" }.into())
        .chain((0..14).map(|i| obj! { "name" => format!("fixture {i}") }.into()))
        .collect();
    h.writer.run(|writer| writer.analysis(&id, &obj! { "summary" => "Fixture", "embedding" => Value::Array(vec![Value::from(0.0); 768]), "tags" => tags }.into())).unwrap();
    let stored = h.store.card_tags(&id).unwrap();
    same!(
        "check-capture-quality.mjs:103",
        stored
            .iter()
            .filter(|tag| tag_key(&tag.name) == "manual system")
            .count(),
        1
    );
    same!(
        "check-capture-quality.mjs:104",
        stored
            .iter()
            .find(|tag| tag_key(&tag.name) == "manual system")
            .unwrap()
            .flags,
        Some(8)
    );
    same!(
        "check-capture-quality.mjs:105",
        stored.iter().filter(|tag| tag.flags == Some(2)).count(),
        10
    );
    h.env(true, true);
    let ai = Ai::new(&*h.fetch);
    let item = AiItem {
        kind: Some("Article".into()),
        title: Some("Mind map".into()),
        text: Some(text(&h.row(&article).extras, "captureText").to_owned()),
        image: Some(AiImage {
            mime: "image/png".into(),
            data: h.media_bytes.png.clone(),
        }),
        ..AiItem::default()
    };
    let start = h.fixtures.requests().len();
    ai.describe(&item).await.unwrap();
    ai.pick_tags(&item, &["design".into()]).await.unwrap();
    let requests = h.fixtures.requests()[start..].to_vec();
    let chat = &requests
        .iter()
        .find(|request| request.path == "/chat/completions")
        .unwrap()
        .body;
    verify!(
        "check-capture-quality.mjs:116",
        text(chat.get("messages").at(0), "content")
            .contains("derive the summary and tags from the supplied text")
    );
    verify!(
        "check-capture-quality.mjs:117",
        chat.get("messages")
            .at(1)
            .get("content")
            .as_array()
            .unwrap()
            .iter()
            .any(|part| text(part, "text").contains("Mind maps connect concepts"))
    );
    verify!(
        "check-capture-quality.mjs:118",
        !chat
            .get("messages")
            .at(1)
            .get("content")
            .as_array()
            .unwrap()
            .iter()
            .any(|part| text(part, "type") == "image_url")
    );
    verify!(
        "check-capture-quality.mjs:119",
        !requests
            .iter()
            .find(|request| request.path == "/v1/systemone")
            .unwrap()
            .body
            .get("assets")
            .truthy()
    );
    for (kind, text_override) in [
        (Some("Image"), None),
        (Some("Screenshot"), None),
        (Some("Video"), None),
        (None, Some("Short caption")),
    ] {
        let start = h.fixtures.requests().len();
        let mut item = item.clone();
        if let Some(kind) = kind {
            item.kind = Some(kind.into());
        }
        if let Some(text) = text_override {
            item.text = Some(text.into());
        }
        ai.describe(&item).await.unwrap();
        let requests = h.fixtures.requests();
        let chat = &requests[start..]
            .iter()
            .find(|request| request.path == "/chat/completions")
            .unwrap()
            .body;
        verify!(
            "check-capture-quality.mjs:123",
            chat.get("messages")
                .at(1)
                .get("content")
                .as_array()
                .unwrap()
                .iter()
                .any(|part| text(part, "type") == "image_url")
        );
    }
    h.env(true, false);
    h.fixtures.control(|control| {
        control.tags = Some(Value::Array(
            ["Design-Systems", "design_system", "DESIGN SYSTEMS"]
                .into_iter()
                .map(Value::from)
                .collect(),
        ))
    });
    let result = ai
        .analyze(
            &AiItem {
                kind: Some("Article".into()),
                title: Some("Fixture".into()),
                text: Some("A fixture on interface systems".into()),
                ..AiItem::default()
            },
            &[VocabularyEntry::from("Design Systems")],
        )
        .await
        .unwrap();
    same!(
        "check-capture-quality.mjs:129",
        result
            .tags
            .iter()
            .map(|tag| tag.name.as_str())
            .collect::<Vec<_>>(),
        ["Design Systems"]
    );
    same!(
        "check-capture-quality.mjs:130",
        result.title.split(' ').count(),
        8
    );
    h.fixtures.control(|control| control.tags = None);
    h.env(true, true);
    let jobs = h.jobs(h.shot.clone());
    jobs.schedule(&post, false);
    jobs.drain().await;
    same!(
        "check-capture-quality.mjs:139",
        text(&h.row(&post).data, "title"),
        "Upgrade Codex Memory Models for Free Now Today"
    );
    h.writer
        .run(|writer| writer.patch(&post, &obj! { "title" => "My chosen headline" }.into()))
        .unwrap();
    jobs.schedule(&post, false);
    jobs.drain().await;
    same!(
        "check-capture-quality.mjs:142",
        text(&h.row(&post).data, "title"),
        "My chosen headline"
    );
    h.fixtures.control(|control| control.chat_delay = 100);
    jobs.schedule(&post, false);
    h.until(|| flags(&h.row(&post).data) & 1 != 0).await;
    h.writer
        .run(|writer| {
            writer.patch(
                &post,
                &obj! { "title" => "Edited during the pending analysis" }.into(),
            )
        })
        .unwrap();
    jobs.drain().await;
    same!(
        "check-capture-quality.mjs:148",
        text(&h.row(&post).data, "title"),
        "Edited during the pending analysis"
    );
    h.fixtures.control(|control| control.chat_delay = 0);
    let own_title = h.create(obj! { "content" => format!("{}/XQuality?own-title=1", h.fixtures.base), "title" => "Title chosen before capture" }.into());
    jobs.schedule(&own_title, true);
    jobs.drain().await;
    same!(
        "check-capture-quality.mjs:152",
        text(&h.row(&own_title).data, "title"),
        "Title chosen before capture"
    );
    h.writer
        .run(|writer| {
            writer.prose(
                &article,
                &doc(&"First edited reader content about software architecture. ".repeat(6)),
                "content",
            )
        })
        .unwrap();
    h.fixtures.control(|control| control.chat_delay = 100);
    let start = h.fixtures.requests().len();
    jobs.schedule(&article, false);
    h.until(|| {
        h.fixtures.requests()[start..]
            .iter()
            .any(|request| request.path == "/chat/completions")
    })
    .await;
    let detail = h
        .request(
            "GET",
            &format!("/cards/{article}/detail"),
            b"",
            &[("accept", "text/html")],
        )
        .await;
    same!(
        "check-capture-quality.mjs:159",
        detail
            .text()
            .contains("<mymind-tldr data-watchable=\"false\"><p>"),
        h.row(&article).data.get("summary").truthy()
    );
    h.writer
        .run(|writer| {
            writer.prose(
                &article,
                &doc(&"Latest edited reader content about databases. ".repeat(6)),
                "content",
            )
        })
        .unwrap();
    jobs.drain().await;
    let chats: Vec<_> = h.fixtures.requests()[start..]
        .iter()
        .filter(|request| request.path == "/chat/completions")
        .cloned()
        .collect();
    verify!("check-capture-quality.mjs:163", chats.len() >= 2);
    let latest = text(
        chats
            .last()
            .unwrap()
            .body
            .get("messages")
            .at(1)
            .get("content")
            .at(0),
        "text",
    );
    verify!(
        "check-capture-quality.mjs:165",
        latest.contains("Latest edited reader content about databases.")
    );
    verify!(
        "check-capture-quality.mjs:166",
        !latest.contains("First edited reader content") && !latest.contains("Mind maps connect")
    );
    same!(
        "check-capture-quality.mjs:167",
        text(&h.row(&article).data, "title"),
        "Mind map"
    );
    h.fixtures.control(|control| control.chat_delay = 0);
    jobs.close();
    let api_article = h.save("ArticleQuality?api-content=1").await;
    h.until(|| analyzed(h, &api_article) > 0).await;
    let before = analyzed(h, &api_article);
    let result = h
        .request(
            "PUT",
            &format!("/objects/{api_article}/content"),
            json::to_string(&doc(
                &"Edited via the Article content HTTP route. ".repeat(6)
            ))
            .as_bytes(),
            &[("content-type", "application/prose+json")],
        )
        .await;
    same!("check-capture-quality.mjs:181", result.status, 200);
    h.until(|| analyzed(h, &api_article) > before).await;
    let requests = h.fixtures.requests();
    let latest = &requests
        .iter()
        .rev()
        .find(|request| request.path == "/chat/completions")
        .unwrap()
        .body;
    verify!(
        "check-capture-quality.mjs:184",
        text(latest.get("messages").at(1).get("content").at(0), "text")
            .contains("Edited via the Article content HTTP route.")
    );
    let revisions = h
        .request(
            "GET",
            &format!("/objects/{api_article}/revisions"),
            b"",
            &[],
        )
        .await
        .value();
    let revision = text(revisions.as_array().unwrap().last().unwrap(), "id");
    let before = analyzed(h, &api_article);
    same!(
        "check-capture-quality.mjs:187",
        h.request(
            "POST",
            &format!("/objects/{api_article}/revisions/{revision}/restore"),
            b"",
            &[]
        )
        .await
        .status,
        200
    );
    h.until(|| analyzed(h, &api_article) > before).await;
    let requests = h.fixtures.requests();
    let latest = &requests
        .iter()
        .rev()
        .find(|request| request.path == "/chat/completions")
        .unwrap()
        .body;
    verify!(
        "check-capture-quality.mjs:189",
        text(latest.get("messages").at(1).get("content").at(0), "text")
            .contains("Mind maps connect concepts")
    );
    h.env(false, false);
}

// Count delivered analyses for one id using decoded real SignalR invocation frames.
fn analyzed(h: &Harness, id: &str) -> usize {
    h.wire
        .messages()
        .iter()
        .filter(|event| {
            text(event, "target") == "object:analyzed"
                && event.get("arguments").at(0).get("id").as_str() == Some(id)
        })
        .count()
}

// Preserve injected and real screenshots, concurrent capture edits and blocked redirect interception.
async fn screenshot_and_edits(h: &Harness) {
    let shots = Rc::new(Shot {
        png: h.media_bytes.png.clone(),
        calls: Cell::new(0),
        fail: false,
    });
    let jobs = h.jobs(shots.clone());
    for (name, kind) in [
        ("NoImage", "WebPage"),
        ("BrokenImage", "WebPage"),
        ("ScreenshotFixture", "Screenshot"),
    ] {
        let id = h.create(
            obj! { "content" => format!("{}/{name}", h.fixtures.base), "type" => kind }.into(),
        );
        jobs.schedule(&id, true);
        jobs.drain().await;
        let item = h.row(&id);
        same!("check-capture.mjs:278", text(&item.data, "type"), kind);
        verify!(
            "check-capture.mjs:278",
            text(item.extras.get("cover"), "src").starts_with("/media/")
        );
        same!(
            "check-capture.mjs:279",
            item.data.get("flags").as_f64(),
            Some(0.0)
        );
        verify!(
            "check-capture.mjs:280",
            !item.extras.get("screenshotPending").truthy()
                && !item.extras.get("screenshotFailed").truthy()
        );
        verify!(
            "check-capture.mjs:281",
            item.html.contains(&format!(
                "class=\"{}\"",
                if kind == "Screenshot" {
                    "screenshot"
                } else {
                    "web-page"
                }
            ))
        );
    }
    same!("check-capture.mjs:283", shots.calls.get(), 3);
    h.fixtures.control(|control| control.image_delay = 150);
    let id = h.save("WebPage?race=1").await;
    h.until(|| text(&h.row(&id).data, "title") == "Fixture title")
        .await;
    h.json(
        "PATCH",
        &format!("/objects/{id}"),
        obj! { "title" => "User wins", "description" => "Edited while capturing" }.into(),
    )
    .await;
    h.json(
        "POST",
        &format!("/objects/{id}/tags"),
        obj! { "name" => "manual-during-capture" }.into(),
    )
    .await;
    let item = h.captured(&id).await;
    same!(
        "check-capture.mjs:290",
        text(&item.data, "title"),
        "User wins"
    );
    same!(
        "check-capture.mjs:290",
        text(&item.data, "description"),
        "Edited while capturing"
    );
    verify!(
        "check-capture.mjs:291",
        item.data
            .get("tags")
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| text(tag, "name") == "manual-during-capture"
                && tag.get("flags").as_f64() == Some(8.0))
    );
    verify!("check-capture.mjs:292", item.html.contains("User wins"));
    verify!(
        "check-capture.mjs:292",
        !item.html.contains("Fixture title")
    );
    h.fixtures.control(|control| control.image_delay = 0);
    jobs.close();
}

// Retain strict real-Chromium cover and redirect interception assertions independently of non-browser coverage.
async fn native_screenshots(h: &Harness) {
    let executable = PathBuf::from(
        std::env::var_os("VERIFIER_CHROMIUM")
            .unwrap_or_else(|| "/usr/lib/chromium/chromium".into()),
    );
    assert!(
        executable.exists(),
        "original native Chromium capture checks require an executable"
    );
    let host = Rc::new(ChromiumScreenshot::new(executable, h.resource.clone()));
    let jobs = h.jobs(host.clone());
    let id =
        h.create(obj! { "content" => format!("{}/NoImage?chromium=1", h.fixtures.base) }.into());
    jobs.schedule(&id, true);
    jobs.drain().await;
    let item = h.row(&id);
    same!("check-capture.mjs:299", text(&item.data, "type"), "WebPage");
    verify!("check-capture.mjs:299", item.extras.get("cover").truthy());
    let result = h
        .request("GET", text(item.extras.get("cover"), "src"), b"", &[])
        .await;
    same!(
        "check-capture.mjs:301",
        mymind::vips::metadata(&result.bytes, 0).unwrap().width,
        800
    );
    jobs.close();
    for target in ["redirect-account", "redirect-external"] {
        let before = h.fixtures.requests().len();
        let error = host
            .capture(&format!("{}/{target}", h.fixtures.base), true)
            .await
            .unwrap_err();
        same!(
            "check-capture.mjs:313",
            h.fixtures.requests().len(),
            before + 1
        );
        verify!(
            "check-capture.mjs:313",
            mm_core::capture_http::is_blocked(&error)
        );
    }
}

// Preserve hosted AI payloads, independent persisted vectors, races, deletion and HTTP scheduling.
async fn ai(h: &Harness, image: &str, article: &str) {
    let before_ai = h.fixtures.requests().len();
    h.env(true, true);
    let jobs = h.jobs(h.shot.clone());
    h.writer
        .run(|writer| writer.add_tag(image, &"design".into()))
        .unwrap();
    jobs.schedule(image, false);
    jobs.drain().await;
    let item = h.row(image);
    same!(
        "check-capture.mjs:329",
        text(&item.data, "summary"),
        "A fixture summary <safe>."
    );
    verify!("check-capture.mjs:330", flags(&item.data) & 1 == 0);
    let tags = item.data.get("tags").as_array().unwrap();
    same!(
        "check-capture.mjs:331",
        tags.iter()
            .find(|tag| text(tag, "name") == "design")
            .unwrap()
            .get("flags")
            .as_f64(),
        Some(8.0)
    );
    verify!(
        "check-capture.mjs:332",
        !tags
            .iter()
            .any(|tag| ["website", "link", "page", "screenshot"].contains(&text(tag, "name")))
    );
    verify!(
        "check-capture.mjs:333",
        tags.iter()
            .any(|tag| tag_key(text(tag, "name")) == "image"
                && tag.get("flags").as_f64() == Some(2.0))
    );
    let vector = h
        .store
        .db()
        .one(
            "SELECT dimensions, vector FROM embeddings WHERE card_id = ?",
            &[image.into()],
        )
        .unwrap()
        .unwrap();
    same!("check-capture.mjs:335", vector.int("dimensions"), Some(768));
    same!(
        "check-capture.mjs:335",
        vector.blob("vector").unwrap().len(),
        3072
    );
    same!(
        "check-capture.mjs:336",
        f32::from_le_bytes(vector.blob("vector").unwrap()[..4].try_into().unwrap()),
        1.0
    );
    verify!("check-capture.mjs:337", analyzed(h, image) > 0);
    let detail = h
        .request(
            "GET",
            &format!("/cards/{image}/detail"),
            b"",
            &[("accept", "text/html")],
        )
        .await;
    verify!(
        "check-capture.mjs:339",
        detail.text().contains(
            "<mymind-tldr data-watchable=\"false\"><p>A fixture summary &lt;safe&gt;.</p>"
        )
    );
    let requests = h.fixtures.requests();
    let embed = &requests[before_ai..]
        .iter()
        .find(|request| request.path.ends_with(":embedContent"))
        .unwrap()
        .body;
    let inline = embed
        .get("content")
        .get("parts")
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part.get("inline_data").truthy())
        .unwrap()
        .get("inline_data");
    same!(
        "check-capture.mjs:341",
        text(inline, "mime_type"),
        "image/png"
    );
    let jev = &requests[before_ai..]
        .iter()
        .find(|request| request.path == "/v1/systemone")
        .unwrap()
        .body;
    verify!(
        "check-capture.mjs:343",
        !jev.get("questions").as_object().unwrap().is_empty()
    );
    verify!(
        "check-capture.mjs:344",
        requests
            .iter()
            .any(|request| request.path.ends_with(":batchEmbedContents"))
    );
    jobs.schedule(article, false);
    jobs.drain().await;
    verify!(
        "check-capture.mjs:346",
        !tag_names(h.row(article).data.get("tags"))
            .iter()
            .any(|name| ["image", "screenshot"].contains(&name.as_str()))
    );
    h.fixtures.control(|control| control.ai_failure = true);
    jobs.schedule(article, false);
    jobs.drain().await;
    same!(
        "check-capture.mjs:349",
        text(&h.row(article).data, "summary"),
        "A fixture summary <safe>."
    );
    verify!(
        "check-capture.mjs:349",
        flags(&h.row(article).data) & 1 == 0
    );
    h.fixtures.control(|control| {
        control.ai_failure = false;
        control.chat_delay = 150;
    });
    let before = h
        .fixtures
        .requests()
        .iter()
        .filter(|request| request.path == "/chat/completions")
        .count();
    jobs.schedule(article, false);
    h.until(|| flags(&h.row(article).data) & 1 != 0).await;
    h.writer
        .run(|writer| {
            writer.patch(
                article,
                &obj! { "title" => "Edited during analysis" }.into(),
            )
        })
        .unwrap();
    jobs.drain().await;
    same!(
        "check-capture.mjs:357",
        text(&h.row(article).data, "title"),
        "Edited during analysis"
    );
    verify!(
        "check-capture.mjs:358",
        h.fixtures
            .requests()
            .iter()
            .filter(|request| request.path == "/chat/completions")
            .count()
            >= before + 2
    );
    let analyzed_events = h
        .wire
        .messages()
        .iter()
        .filter(|event| text(event, "target") == "object:analyzed")
        .count();
    jobs.schedule(article, false);
    h.until(|| flags(&h.row(article).data) & 1 != 0).await;
    h.writer.run(|writer| writer.trash(article, false)).unwrap();
    jobs.drain().await;
    verify!(
        "check-capture.mjs:364",
        h.row(article).data.get("deleted").truthy()
    );
    verify!(
        "check-capture.mjs:364",
        flags(&h.row(article).data) & 1 == 0
    );
    same!(
        "check-capture.mjs:365",
        h.wire
            .messages()
            .iter()
            .filter(|event| text(event, "target") == "object:analyzed")
            .count(),
        analyzed_events
    );
    h.writer.run(|writer| writer.trash(article, true)).unwrap();
    jobs.schedule(article, false);
    jobs.drain().await;
    verify!(
        "check-capture.mjs:368",
        !h.row(article).data.get("deleted").truthy()
    );
    verify!(
        "check-capture.mjs:368",
        flags(&h.row(article).data) & 1 == 0
    );
    h.env(true, false);
    let before = h
        .fixtures
        .requests()
        .iter()
        .filter(|request| request.path == "/v1/systemone")
        .count();
    jobs.schedule(article, false);
    jobs.drain().await;
    same!(
        "check-capture.mjs:372",
        h.fixtures
            .requests()
            .iter()
            .filter(|request| request.path == "/v1/systemone")
            .count(),
        before
    );
    verify!(
        "check-capture.mjs:373",
        h.row(article)
            .data
            .get("tags")
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| text(tag, "name") == "design" && tag.get("flags").as_f64() == Some(2.0))
    );
    h.fixtures.control(|control| control.chat_delay = 0);
    h.env(true, true);
    jobs.close();
    h.fixtures.control(|control| control.ai_redirect = true);
    let ai = Ai::new(&*h.fetch);
    let item = AiItem {
        title: Some("Fixture redirect".into()),
        text: Some("Fixture redirect".into()),
        ..AiItem::default()
    };
    same!(
        "check-capture.mjs:382",
        ai.embed(&item, EmbedKind::Document)
            .await
            .unwrap_err()
            .status,
        Some(307)
    );
    same!(
        "check-capture.mjs:382",
        ai.describe(&item).await.unwrap_err().status,
        Some(307)
    );
    same!(
        "check-capture.mjs:382",
        ai.pick_tags(&item, &["design".into()])
            .await
            .unwrap_err()
            .status,
        Some(307)
    );
    h.fixtures.control(|control| control.ai_redirect = false);
    let result = h
        .json(
            "POST",
            "/objects",
            obj! { "type" => "Note", "content" => "Fixture note for AI" }.into(),
        )
        .await;
    same!("check-capture.mjs:394", result.status, 201);
    let id = text(&result.value(), "id").to_owned();
    h.until(|| text(&h.row(&id).data, "summary") == "A fixture summary <safe>.")
        .await;
    verify!(
        "check-capture.mjs:397",
        text(&h.row(&id).data, "summary") == "A fixture summary <safe>."
    );
    let result = h
        .json("POST", &format!("/cards/{id}/analyze"), obj! {}.into())
        .await;
    same!(
        "check-capture.mjs:399",
        text(&result.value(), "status"),
        "Processing"
    );
    h.until(|| analyzed(h, &id) >= 2).await;
    verify!("check-capture.mjs:400", analyzed(h, &id) >= 2);
    h.json(
        "PUT",
        &format!("/objects/{id}/prose"),
        doc("Edited note for AI"),
    )
    .await;
    h.until(|| analyzed(h, &id) >= 3).await;
    verify!("check-capture.mjs:402", analyzed(h, &id) >= 3);
    h.env(false, true);
    let result = h
        .request("POST", &format!("/cards/{image}/analyze"), b"", &[])
        .await;
    same!(
        "check-capture.mjs:406",
        text(&result.value(), "status"),
        "Skipped"
    );
}

// Prove read-only mode performs no uploads, embeddings, jobs or outbound fixture calls.
async fn readonly(h: &Harness) {
    let readonly = Harness::new("readonly", true).await;
    readonly.env(true, true);
    let count = || {
        ["uploads", "embeddings"].map(|table| {
            readonly
                .store
                .db()
                .one(&format!("SELECT COUNT(*) n FROM {table}"), &[])
                .unwrap()
                .unwrap()
                .int("n")
                .unwrap()
        })
    };
    let seeded = count();
    let before = readonly.fixtures.requests().len();
    readonly
        .json(
            "POST",
            "/objects",
            obj! { "content" => format!("{}/WebPage", readonly.fixtures.base) }.into(),
        )
        .await;
    readonly
        .request("POST", "/uploads/authorize", b"", &[])
        .await;
    readonly
        .request("POST", "/cards/nonexistent/analyze", b"", &[])
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    same!(
        "check-capture.mjs:425",
        readonly.fixtures.requests().len(),
        before
    );
    same!("check-capture.mjs:426", count(), seeded);
    h.env(false, false);
}

#[test]
fn original_capture_media_ai_and_readonly_assertions() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    mymind::runtime::run(async {
        let h = Harness::new("complete", false).await;
        let article = capture_cases(&h).await;
        let (image, _) = media(&h).await;
        screenshot_and_edits(&h).await;
        ai(&h, &image, &article).await;
        readonly(&h).await;
    });
}

#[test]
fn original_capture_quality_assertions() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    mymind::runtime::run(async {
        let h = Harness::new("quality", false).await;
        quality(&h).await;
    });
}

#[test]
fn original_recorded_link_lifecycle_assertions() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    mymind::runtime::run(async {
        let h = Harness::new("lifecycle", false).await;
        lifecycle(&h).await;
    });
}

#[test]
fn original_native_screenshot_and_interception_assertions() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _scratch = ScratchTemp::new();
    mymind::runtime::run(async {
        let h = Harness::new("native", false).await;
        native_screenshots(&h).await;
    });
}
