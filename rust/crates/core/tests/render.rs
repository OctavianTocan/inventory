//! The render modules against the JS backend on every fixture card plus generated edge cases.

mod common;

use common::{assert_same, calls, hosted_db, test_db};
use mm_core::render::iphone::iphone_card;
use mm_core::render::media::{fit, image_cover, render_media, render_video};
use mm_core::render::prose::render_prose;
use mm_core::render_card::render_card;
use mm_core::sql::Sql;
use mm_core::{Value, arr, obj};
use mm_js::json;
use mm_sqlite::Sqlite;

struct Card {
    id: String,
    data: Value,
    extras: Value,
    html: String,
    raw: String,
}

fn cards() -> Vec<Card> {
    let mut out = Vec::new();
    for path in [test_db(), hosted_db()] {
        let db = Sqlite::open(path, true).unwrap();
        for row in db.query("SELECT id, data, html, extras FROM cards ORDER BY rowid", &[]).unwrap() {
            let raw = row.text("data").unwrap().to_string();
            out.push(Card {
                id: row.text("id").unwrap().to_string(),
                data: json::parse(&raw).unwrap(),
                extras: json::parse(row.text("extras").unwrap()).unwrap(),
                html: row.text("html").unwrap().to_string(),
                raw,
            });
        }
    }
    out
}

fn same<T: Into<Value>>(rust: mm_core::Result<T>, js: &common::Answer, context: &str) {
    assert_same(&rust.map(Into::into), js, context);
}

#[test]
fn iphone_matches_node() {
    let _goldens = common::test_guard();
    let cards = cards();
    let js = calls("module:render/iphone", &cards.iter().map(|card| ("iphoneCard", vec![Value::from(card.html.as_str()), Value::from(card.raw.as_str())])).collect::<Vec<_>>());
    for (card, js) in cards.iter().zip(&js) {
        same(iphone_card(&card.html, &card.raw), js, &format!("iphoneCard {}", card.id));
    }
    assert!(cards.len() > 4000);
}

#[test]
fn media_helpers_match_node() {
    let _goldens = common::test_guard();
    let cards = cards();
    let mut inputs: Vec<Value> = Vec::new();
    for card in &cards {
        for value in [card.data.get("blob"), card.extras.get("cover")] {
            inputs.push(value.clone());
        }
    }
    inputs.extend([
        Value::Null, Value::Undefined, obj! {}.into(), obj! { "width" => 0, "height" => 0 }.into(),
        obj! { "width" => "640", "height" => "480", "src" => "/a\"b", "type" => "video" }.into(),
        obj! { "width" => 1000.5, "height" => 3, "src" => "/x", "svg" => true, "imageWidth" => 30, "imageHeight" => 20 }.into(),
        obj! { "width" => 4, "height" => 7, "src" => "/x", "dominantColor" => "#fff", "type" => "image" }.into(),
        obj! { "path" => "p.png", "width" => 100, "height" => 0 }.into(),
        obj! { "path" => "p.mp4", "width" => 1921, "height" => 1081, "duration" => 3.5 }.into(),
        obj! { "path" => "p.mp4", "width" => -4, "height" => 5 }.into(),
        obj! { "width" => Value::Null, "height" => "x" }.into(),
    ]);
    let mut requests = Vec::new();
    for input in &inputs {
        requests.push(("renderMedia", vec![input.clone()]));
        requests.push(("imageCover", vec![input.clone()]));
        requests.push(("renderVideo", vec![input.clone()]));
        for (width, height, even) in [(800.0, 1000.0, false), (642.0, 840.0, true), (0.0, 10.0, false), (-5.0, 5.0, true)] {
            requests.push(("fit", vec![input.clone(), width.into(), height.into(), even.into()]));
        }
    }
    let js = calls("module:render/media", &requests);
    let mut at = 0;
    for (index, input) in inputs.iter().enumerate() {
        let context = format!("input {index} {}", common::text(input));
        same(Ok(render_media(input)), &js[at], &format!("renderMedia {context}"));
        same(image_cover(input), &js[at + 1], &format!("imageCover {context}"));
        same(render_video(input), &js[at + 2], &format!("renderVideo {context}"));
        at += 3;
        for (width, height, even) in [(800.0, 1000.0, false), (642.0, 840.0, true), (0.0, 10.0, false), (-5.0, 5.0, true)] {
            let size = fit(input, width, height, even).map(|size| obj! { "width" => size.width, "height" => size.height });
            same(size, &js[at], &format!("fit {context} {width} {height} {even}"));
            at += 1;
        }
    }
}

fn node(kind: &str, extra: Vec<(&str, Value)>) -> Value {
    let mut object = obj! { "type" => kind };
    for (key, value) in extra {
        object.insert(key, value);
    }
    object.into()
}

fn text_node(text: &str, marks: Vec<Value>) -> Value {
    node("text", vec![("text", text.into()), ("marks", Value::Array(marks))])
}

fn prose_cases() -> Vec<Value> {
    let long = "x".repeat(600);
    let marks = ["bold", "italic", "underline", "strike", "code", "highlight", "subscript", "superscript", "comment"];
    let mut cases = vec![
        obj! { "type" => "doc" }.into(),
        node("doc", vec![("content", arr![])]),
        node("doc", vec![("content", arr![node("paragraph", vec![]), node("horizontalRule", vec![])])]),
        node("doc", vec![("content", arr![node("paragraph", vec![("content", arr![text_node("a<b>&\"'é😀", marks.iter().map(|m| obj! { "type" => *m }.into()).collect())])])])]),
        node("doc", vec![("content", arr![node("paragraph", vec![("content", arr![text_node("link", vec![node("link", vec![("attrs", obj! { "href" => "/a?b=\"c\"" }.into())])]), node("hardBreak", vec![]), text_node("z", vec![node("link", vec![])])])])])]),
        node("doc", vec![("content", arr![node("heading", vec![("attrs", obj! { "level" => 3 }.into()), ("content", arr![text_node("h", vec![])])]), node("heading", vec![])])]),
        node("doc", vec![("content", arr![node("bulletList", vec![("content", arr![node("listItem", vec![("content", arr![node("paragraph", vec![])])])])]), node("orderedList", vec![]), node("blockquote", vec![]), node("codeBlock", vec![("content", arr![text_node("let a", vec![])])])])]),
        node("doc", vec![("content", arr![node("wikiLink", vec![("attrs", obj! { "id" => "i", "target" => "t", "targetId" => "ti", "targetType" => "tt", "text" => "tx<" }.into())]), node("wikiLink", vec![("attrs", obj! { "target" => "only" }.into())]), node("wikiLink", vec![])])]),
        node("doc", vec![("content", arr![node("taskList", vec![("content", arr![node("taskItem", vec![("attrs", obj! { "checked" => true }.into()), ("content", arr![node("paragraph", vec![])])]), node("taskItem", vec![])])])])]),
        node("doc", vec![("content", arr![node("table", vec![("content", arr![node("tableRow", vec![("content", arr![node("tableHeader", vec![("attrs", obj! { "colspan" => 2, "rowspan" => "3" }.into())]), node("tableCell", vec![])])])])])])]),
        node("doc", vec![("content", arr![node("image", vec![("attrs", obj! { "src" => "/i.png" }.into())]), node("image", vec![])])]),
        node("doc", vec![("content", arr![node("mystery", vec![])])]),
        node("doc", vec![("content", arr![node("paragraph", vec![("content", arr![node("text", vec![])])])])]),
        node("doc", vec![("content", arr![node("paragraph", vec![("content", arr![node("text", vec![("text", 5.into()), ("marks", arr![Value::Null])])])])])]),
        Value::Null,
        Value::Undefined,
        "doc".into(),
    ];
    let paragraph = |text: &str| node("paragraph", vec![("content", arr![text_node(text, vec![])])]);
    cases.push(node("doc", vec![("content", Value::Array((0..3).map(|_| paragraph(&long)).collect()))]));
    cases.push(node("doc", vec![("content", Value::Array((0..25).map(|_| paragraph("line")).collect()))]));
    cases.push(node("doc", vec![("content", Value::Array((0..11).map(|_| paragraph("line")).collect()))]));
    cases.push(node("doc", vec![("content", Value::Array(vec![paragraph(&"y".repeat(1001)), paragraph("after")]))]));
    cases.push(node("doc", vec![("content", Value::Array(vec![paragraph(&"y".repeat(500)), paragraph("a"), paragraph(&"y".repeat(500)), paragraph("b")]))]));
    cases
}

#[test]
fn prose_matches_node() {
    let _goldens = common::test_guard();
    let mut docs: Vec<(String, Value)> = cards().into_iter().filter(|card| !card.data.get("prose").is_undefined()).map(|card| (card.id, card.data.get("prose").clone())).collect();
    assert!(docs.len() > 30);
    docs.extend(prose_cases().into_iter().enumerate().map(|(index, doc)| (format!("case {index}"), doc)));
    let js = calls("module:render/prose", &docs.iter().map(|(_, doc)| ("renderProse", vec![doc.clone()])).collect::<Vec<_>>());
    for ((name, doc), js) in docs.iter().zip(&js) {
        same(render_prose(doc), js, &format!("renderProse {name}"));
    }
}

fn variants(card: &Card) -> Vec<(String, Value, Value)> {
    let set = |base: &Value, key: &str, value: Value| {
        let mut copy = base.clone();
        if let Some(object) = copy.as_object_mut() {
            object.insert(key, value);
        }
        copy
    };
    let (data, extras) = (&card.data, &card.extras);
    let mut out = vec![
        ("base".to_string(), data.clone(), extras.clone()),
        ("deleted".into(), set(data, "deleted", true.into()), extras.clone()),
        ("pending".into(), data.clone(), set(extras, "capturePending", true.into())),
        ("failed".into(), data.clone(), set(&set(extras, "capturePending", true.into()), "captureFailed", true.into())),
        ("screenshot".into(), data.clone(), set(extras, "screenshotPending", true.into())),
        ("link".into(), data.clone(), set(extras, "linkUrl", "https://x.test/a?b=\"c\"&d".into())),
        ("title".into(), data.clone(), set(extras, "title", "T <i>é</i>".into())),
        ("no-cover".into(), data.clone(), set(extras, "cover", Value::Undefined)),
        ("highlight".into(), set(data, "tags", arr![obj! { "name" => "highlight" }]), extras.clone()),
        ("flags".into(), set(data, "flags", 8_388_608.into()), set(extras, "cover", obj! { "src" => "/r.jpg" }.into())),
        ("thumb".into(), data.clone(), set(extras, "cover", obj! { "src" => "/r.jpg", "isThumbnail" => false }.into())),
        ("post".into(), set(data, "description", "a\r\nb\rc\n<d>".into()), set(extras, "post", obj! { "hasVideo" => true, "author" => "Au <x>", "gallery" => arr![obj! { "src" => "/g1.jpg", "width" => 1, "height" => 2 }, obj! { "srcset" => "a 1x, b 2x", "width" => 3, "height" => 4 }, obj! { "src" => "/g3" }] }.into())),
        ("postbody".into(), data.clone(), set(extras, "post", obj! { "bodyHtml" => "<p>raw</p>" }.into())),
        ("noid".into(), data.clone(), obj! {}.into()),
        ("app".into(), data.clone(), set(extras, "app", obj! { "distributor" => "play\"", "srcset" => "a\"b 1x" }.into())),
        ("product".into(), data.clone(), set(extras, "product", obj! { "price" => "$5 <b>" }.into())),
        ("reddit".into(), data.clone(), set(extras, "reddit", obj! { "title" => "RT", "subreddit" => "r/x" }.into())),
        ("undefined-extras".into(), data.clone(), Value::Undefined),
        ("null-extras".into(), data.clone(), Value::Null),
        ("bad-type".into(), set(data, "type", "constructor".into()), extras.clone()),
        ("other-type".into(), set(data, "type", "Mystery".into()), extras.clone()),
        ("bad-desc".into(), set(data, "description", 5.into()), set(extras, "post", Value::Null)),
        ("bad-gallery".into(), data.clone(), set(extras, "post", obj! { "gallery" => "abc" }.into())),
        ("bad-tags".into(), set(data, "tags", "x".into()), extras.clone()),
        ("null-tag".into(), set(data, "tags", arr![Value::Null]), extras.clone()),
    ];
    for kind in ["XPost", "ThreadsPost", "SubstackNote", "Repository", "WebPage", "Article", "Placeholder", "RedditPost", "SoftwareApplication", "Product", "InstagramReel", "Document", "Image", "Screenshot", "Video", "YouTubeVideo", "Note", "Content"] {
        out.push((format!("as-{kind}"), set(data, "type", kind.into()), extras.clone()));
        out.push((format!("pending-{kind}"), set(data, "type", kind.into()), set(extras, "capturePending", true.into())));
        out.push((format!("shot-{kind}"), set(data, "type", kind.into()), set(extras, "screenshotPending", true.into())));
    }
    out
}

#[test]
fn render_card_matches_node() {
    let _goldens = common::test_guard();
    let cards = cards();
    let mut requests: Vec<(String, Value, Value)> = Vec::new();
    for (index, card) in cards.iter().enumerate() {
        if index % 7 == 0 || index < 30 {
            for (name, data, extras) in variants(card) {
                requests.push((format!("{} {name}", card.id), data, extras));
            }
        } else {
            requests.push((format!("{} base", card.id), card.data.clone(), card.extras.clone()));
        }
    }
    let js = calls("module:render-card", &requests.iter().map(|(_, data, extras)| ("renderCard", vec![data.clone(), extras.clone()])).collect::<Vec<_>>());
    for ((name, data, extras), js) in requests.iter().zip(&js) {
        same(render_card(data, extras), js, &format!("renderCard {name}"));
    }
    println!("renderCard cases: {}", requests.len());
}
