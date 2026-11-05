//! Complete HTTP mutation assertions formerly in `tools/check-writes.mjs`, executed against only the Rust server.

mod stage_b_common;
mod stage_b_expectations;

use mm_core::sql::{Sql, SqlExt};
use mm_js::{Value, arr, json, obj};
use mm_sqlite::Sqlite;
use stage_b_common::*;
use std::path::Path;
use std::time::{Duration, Instant};

// Localize persisted inputs independently of the endpoint being tested.
fn localized(text: &str) -> String {
    text.replace("https://static.accelerator.net/", "/_static/")
        .replace("https://media.mymind.host/", "/_mediahost/")
        .replace("https://carbon-media.accelerator.net/", "/_carbon/")
        .replace("https://access.mymind.com/", "/")
}

// Wait for the offline fallback before comparing separate reads of the same persisted URL card.
fn settle_offline_capture(db: &Path, id: &str) {
    let started = Instant::now();
    loop {
        let stored = Sqlite::open(db, true)
            .unwrap()
            .one("SELECT extras FROM cards WHERE id=?", &[id.into()])
            .unwrap()
            .expect("URL card persisted");
        let extras = json::parse(stored.text("extras").unwrap()).unwrap();
        if !extras.get("capturePending").truthy() {
            assert!(
                extras.get("captureFailed").truthy(),
                "offline URL reached its readable failure fallback"
            );
            return;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "offline URL capture finishes before independent snapshot reads"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

// Verify storage, an independent render, the stream and JSON content negotiation for every card read.
fn card(client: &mut Client, db: &Path, id: &str) -> (Value, String, String) {
    let answer = request(
        client,
        "GET",
        &format!("/cards/{id}"),
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    status(&answer, 200);
    let packed = value(&answer);
    let packed = packed.as_array().unwrap();
    assert_eq!(packed.len(), 1, "one card packet");
    let stored = Sqlite::open(db, true)
        .unwrap()
        .one(
            "SELECT data,html,extras FROM cards WHERE id=?",
            &[id.into()],
        )
        .unwrap()
        .expect("card persisted");
    let data = stored.text("data").unwrap();
    let html = stored.text("html").unwrap();
    let extras = stored.text("extras").unwrap();
    stage_b_expectations::assert_html(
        &json::parse(data).unwrap(),
        &json::parse(extras).unwrap(),
        html,
    );
    same(
        &packed[0],
        &obj! { "html" => localized(html), "json" => localized(data) }.into(),
        "card stream equals persisted inputs",
    );
    let model = json::parse(packed[0].get("json").as_str().unwrap()).unwrap();
    same(
        &get(client, &format!("/cards/{id}")),
        &model,
        &format!(
            "GET /cards/<id> JSON content negotiation for {}",
            model.get("type").as_str().unwrap_or("unknown type")
        ),
    );
    (model, localized(html), localized(data))
}

// Read a transport checkpoint without depending on any recorded grid bytes.
fn checkpoint(client: &mut Client) -> String {
    let answer = request(
        client,
        "GET",
        "/cards",
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    status(&answer, 200);
    let checkpoint = header(&answer, "x-checkpoint").expect("checkpoint");
    assert!(checkpoint.bytes().all(|byte| byte.is_ascii_digit()));
    checkpoint.into()
}

// Require poll metadata, exact uint64 checkpoints, timestamp extensions and the current rendered card.
fn changed(
    client: &mut Client,
    db: &Path,
    since: &str,
    id: &str,
    kind: f64,
    object_type: f64,
) -> Value {
    let answer = request(
        client,
        "GET",
        &format!("/changes?since={since}&render=true"),
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    status(&answer, 200);
    let packed = value(&answer);
    let changes = packed.as_array().unwrap();
    assert_eq!(
        header(&answer, "x-count")
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        changes.len()
    );
    assert!(header(&answer, "x-object-quota").is_some());
    let item = changes
        .iter()
        .rev()
        .find(|change| {
            change.get("object").get("id").as_str() == Some(id)
                && change.get("objectType").as_f64() == Some(object_type)
        })
        .expect("change returned");
    assert_eq!(item.get("type").as_f64(), Some(kind));
    assert!(
        matches!(item.get("timestamp"), Value::BigInt(timestamp) if *timestamp > since.parse::<i128>().unwrap()),
        "strictly newer uint64 checkpoint"
    );
    if object_type == 9.0 {
        let (data, html, json) = card(client, db, id);
        assert!(
            item.get("cardHtml").as_str() == Some(html.as_str()),
            "poll HTML current"
        );
        assert!(
            item.get("cardData").as_str() == Some(json.as_str()),
            "poll data current"
        );
        let expected_ms =
            stage_b_expectations::milliseconds(data.get("modified").as_str().unwrap());
        assert!(
            matches!(item.get("object").get("modified"), Value::Date(ms) if *ms == expected_ms),
            "MessagePack timestamp extension preserves modified milliseconds"
        );
    }
    item.clone()
}

// Read a successful HTML detail or shell without treating it as JSON.
fn html(client: &mut Client, path: &str, document: bool) -> String {
    let mut sent = Sent::new("GET", path, &[("accept", "text/html")]);
    if document {
        sent.headers
            .push(("sec-fetch-dest".into(), "document".into()));
    }
    let answer = client.send(&sent);
    status(&answer, 200);
    String::from_utf8(answer.body).unwrap()
}

// Check a grid's HTML membership while preserving the stream packet boundaries.
fn grid_has(client: &mut Client, path: &str, id: &str) -> bool {
    let answer = request(
        client,
        "GET",
        path,
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    status(&answer, 200);
    value(&answer)
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.get("html").as_str().unwrap().contains(id))
}

// Parse the current shell configuration with a bounded literal script delimiter.
fn config(shell: &str) -> Value {
    let body = shell
        .split_once("<script id=\"config\" type=\"application/json\">")
        .expect("config script")
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    json::parse(body).unwrap()
}

// Save a prose document with its media type while retaining the shared write status and quota assertions.
fn prose_put(client: &mut Client, path: &str, body: &Value) {
    let answer = request(
        client,
        "PUT",
        path,
        body,
        "application/json",
        "application/prose+json",
    );
    status(&answer, 200);
    assert!(
        header(&answer, "x-object-quota").is_some(),
        "write quota header"
    );
}

// Wait for the server's start-up ThumbHash sweep to finish, so its card updates cannot move a checkpoint the test
// expects to stay put (on a busy machine the sweep outlasts the first writes).
fn pictures_hashed(db: &Path) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let waiting = Sqlite::open(db, true)
            .unwrap()
            .one(
                "SELECT count(*) AS n FROM cards WHERE deleted_ns IS NULL AND (
                   (json_extract(extras, '$.cover.src') LIKE '/media/%' AND json_extract(extras, '$.cover.thumbHash') IS NULL)
                   OR (json_extract(extras, '$.cover') IS NULL AND json_extract(data, '$.blob.path') IS NOT NULL
                       AND json_extract(data, '$.blob.thumbHash') IS NULL))",
                &[],
            )
            .unwrap()
            .unwrap()
            .int("n")
            .unwrap();
        if waiting == 0 {
            return;
        }
        assert!(Instant::now() < deadline, "{waiting} pictures still lack a ThumbHash after 60 s");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn every_original_write_assertion_and_committed_hub_event() {
    let root = root("writes");
    let db = library(&root);
    let server = start(&root, &db, false, &[]);
    pictures_hashed(&db);
    let mut client = Client::new(server.port);
    let hub = Hub::connect(server.port);
    let initial_quota = get(&mut client, "/quota");
    let since = checkpoint(&mut client);
    let note = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "type" => "Note", "title" => "HTTP note", "prose" => doc("First paragraph") }.into(),
        201,
    ));
    assert!(note.len() == 22 && note.bytes().all(|byte| byte.is_ascii_alphanumeric()));
    let (data, rendered, _) = card(&mut client, &db, &note);
    assert_eq!(data.get("title").as_str(), Some("HTTP note"));
    assert!(rendered.contains("First paragraph"));
    let grid = request(
        &mut client,
        "GET",
        "/cards",
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    assert!(
        value(&grid)
            .at(0)
            .get("html")
            .as_str()
            .unwrap()
            .contains(&note),
        "new note first"
    );
    assert_eq!(
        json::parse(header(&grid, "x-object-quota").unwrap())
            .unwrap()
            .get("count")
            .as_f64()
            .expect("numeric field"),
        initial_quota.get("count").as_f64().expect("numeric field") + 1.0
    );
    changed(&mut client, &db, &since, &note, 1.0, 9.0);

    let text = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "content" => "Line one\n\n<script>local text</script>" }.into(),
        201,
    ));
    let (data, rendered, _) = card(&mut client, &db, &text);
    assert_eq!(data.get("type").as_str(), Some("Note"));
    assert_eq!(
        data.get("prose").get("content").as_array().unwrap().len(),
        3
    );
    assert!(rendered.contains("&lt;script&gt;") && !rendered.contains("<script>"));
    let url = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "content" => "https://example.invalid/local" }.into(),
        201,
    ));
    settle_offline_capture(&db, &url);
    let (data, _, _) = card(&mut client, &db, &url);
    assert!(matches!(
        data.get("type").as_str(),
        Some("WebPage" | "Placeholder")
    ));
    assert_eq!(
        data.get("source").get("url").as_str(),
        Some("https://example.invalid/local")
    );
    assert_eq!(data.get("domain").as_str(), Some("example.invalid"));
    assert_eq!(
        data.get("flags").as_f64().expect("numeric field") as i64 & !1,
        0
    );
    let since = checkpoint(&mut client);
    assert_eq!(
        id(&write(
            &mut client,
            "POST",
            "/objects",
            obj! { "content" => "https://example.invalid/local" }.into(),
            200
        )),
        url
    );
    changed(&mut client, &db, &since, &url, 1.0, 9.0);

    let before = card(&mut client, &db, &note).0;
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{note}"),
        obj! { "title" => "Changed & escaped", "description" => "Local description", "flags" => 0 }
            .into(),
        200,
    );
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{note}/entity"),
        obj! { "author" => obj! { "name" => "Local author" } }.into(),
        200,
    );
    let (data, rendered, _) = card(&mut client, &db, &note);
    assert_eq!(data.get("title").as_str(), Some("Changed & escaped"));
    assert_eq!(data.get("description").as_str(), Some("Local description"));
    assert_eq!(
        data.get("author").get("name").as_str(),
        Some("Local author")
    );
    same(
        data.get("bumped"),
        before.get("bumped"),
        "edit preserves bumped",
    );
    assert!(rendered.contains("Changed &amp; escaped"));
    changed(&mut client, &db, &since, &note, 1.0, 9.0);
    let since = checkpoint(&mut client);
    let attachment = write(
        &mut client,
        "PUT",
        &format!("/objects/{note}/note"),
        doc("Attached note"),
        200,
    );
    let edited = write(
        &mut client,
        "PUT",
        &format!("/objects/{note}/note"),
        doc("Edited attached note"),
        200,
    );
    same(attachment.get("id"), edited.get("id"), "stable note id");
    same(
        card(&mut client, &db, &note).0.get("note").get("prose"),
        &doc("Edited attached note"),
        "attached note",
    );
    changed(&mut client, &db, &since, &note, 1.0, 9.0);

    let since = checkpoint(&mut client);
    write(
        &mut client,
        "PUT",
        &format!("/objects/{note}/prose"),
        doc("Edited prose"),
        200,
    );
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{note}/prose"),
        doc("Patched prose"),
        200,
    );
    assert!(card(&mut client, &db, &note).1.contains("Patched prose"));
    let result = request(
        &mut client,
        "PUT",
        &format!("/objects/{note}/content"),
        &doc("Reader edit"),
        "application/json",
        "application/prose+json",
    );
    status(&result, 200);
    assert!(header(&result, "x-object-quota").is_some());
    same(
        &get(&mut client, &format!("/objects/{note}/content")),
        &doc("Reader edit"),
        "reader read",
    );
    assert!(card(&mut client, &db, &note).1.contains("Reader edit"));
    write(
        &mut client,
        "PUT",
        &format!("/objects/{note}/prose"),
        doc("Composer edit"),
        200,
    );
    same(
        &get(&mut client, &format!("/objects/{note}/content")),
        &doc("Composer edit"),
        "composer sync",
    );
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{note}"),
        obj! { "prose" => doc("Native model edit") }.into(),
        200,
    );
    same(
        &get(&mut client, &format!("/objects/{note}/content")),
        &doc("Native model edit"),
        "native sync",
    );
    changed(&mut client, &db, &since, &note, 1.0, 9.0);

    let draft = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "type" => "Note", "isDraft" => true, "prose" => doc("Draft") }.into(),
        201,
    ));
    let stub = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "type" => "Note", "isStub" => true, "title" => "Wiki idea" }.into(),
        201,
    ));
    assert!(!grid_has(&mut client, "/cards", &draft) && !grid_has(&mut client, "/cards", &stub));
    let since = checkpoint(&mut client);
    assert_eq!(
        id(&write(
            &mut client,
            "POST",
            &format!("/objects/{draft}/commit"),
            obj! { "prose" => doc("Committed note") }.into(),
            201
        )),
        draft
    );
    assert_eq!(
        card(&mut client, &db, &draft)
            .0
            .get("flags")
            .as_f64()
            .expect("numeric field") as i64
            & 131072,
        0
    );
    assert!(grid_has(&mut client, "/cards", &draft));
    changed(&mut client, &db, &since, &draft, 1.0, 9.0);
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{draft}/commit"),
        obj! { "prose" => doc("Committed again") }.into(),
        200,
    );
    prose_put(
        &mut client,
        &format!("/objects/{stub}/content"),
        &doc("Written wiki note"),
    );
    assert_eq!(
        card(&mut client, &db, &stub)
            .0
            .get("flags")
            .as_f64()
            .expect("numeric field") as i64
            & 262144,
        0
    );

    let since = checkpoint(&mut client);
    let tag = write(
        &mut client,
        "POST",
        &format!("/objects/{note}/tags"),
        obj! { "name" => "local-http-tag" }.into(),
        200,
    );
    let tag_id = id(&tag);
    assert_eq!(tag.get("flags").as_f64().expect("numeric field"), 8.0);
    assert_eq!(
        id(&write(
            &mut client,
            "POST",
            &format!("/objects/{note}/tags"),
            obj! { "name" => "local-http-tag" }.into(),
            200
        )),
        tag_id
    );
    same(
        &get(&mut client, &format!("/objects/{note}/tags")),
        &arr![tag.clone()],
        "tag idempotency",
    );
    let manual = request(
        &mut client,
        "GET",
        "/tags/manual",
        &Value::Undefined,
        "application/msgpack",
        "application/json",
    );
    status(&manual, 200);
    assert!(
        value(&manual)
            .at(0)
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.get("name") == tag.get("name")
                && value.get("count").as_f64().expect("numeric field") == 1.0)
    );
    let manual_json = get(&mut client, "/tags/manual");
    let manual_packets = value(&manual);
    assert!(
        manual_packets
            .at(0)
            .as_array()
            .unwrap()
            .iter()
            .all(|tag| matches!(tag.get("modified"), Value::Date(_))),
        "manual tag modified keeps its MessagePack Date extension"
    );
    let manual_plain = json::parse(&json::to_string(manual_packets.at(0))).unwrap();
    same(
        &manual_json,
        &manual_plain,
        "intentional JSON tags/manual negotiation with ISO dates",
    );
    for path in ["/tags/recent?take=10", "/tags/suggest?q=local-http"] {
        assert!(
            get(&mut client, path)
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.get("name") == tag.get("name"))
        );
    }
    assert!(contains(
        get(&mut client, "/tags/detail?name=local-http-tag").get("objects"),
        &note
    ));
    changed(&mut client, &db, &since, &note, 1.0, 9.0);
    let highlight = id(&write(
        &mut client,
        "POST",
        &format!("/objects/{note}/tags"),
        obj! { "name" => "highlight" }.into(),
        200,
    ));
    let (data, rendered, _) = card(&mut client, &db, &note);
    assert!(rendered.contains("note content highlighted"));
    assert!(
        html(&mut client, &format!("/cards/{note}/detail"), false)
            .contains("class=\"body inner highlighted\"")
    );
    assert!(
        data.get("tags")
            .as_array()
            .unwrap()
            .iter()
            .find(|tag| tag.get("id").as_str() == Some(&highlight))
            .unwrap()
            .as_object()
            .unwrap()
            .get("flags")
            .is_none()
    );
    write(
        &mut client,
        "DELETE",
        &format!("/annotations/{highlight}"),
        Value::Undefined,
        200,
    );
    assert!(
        !card(&mut client, &db, &note)
            .1
            .contains("content highlighted")
    );
    assert!(
        !html(&mut client, &format!("/cards/{note}/detail"), false)
            .contains("class=\"body inner highlighted\"")
    );
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "DELETE",
        &format!("/annotations/{tag_id}"),
        Value::Undefined,
        200,
    );
    assert!(
        get(&mut client, &format!("/objects/{note}/tags"))
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !get(&mut client, "/tags/recent?take=100")
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag.get("name").as_str() == Some("local-http-tag"))
    );
    changed(&mut client, &db, &since, &note, 1.0, 9.0);
    let private = id(&write(
        &mut client,
        "POST",
        &format!("/objects/{note}/tags"),
        obj! { "name" => "private" }.into(),
        200,
    ));
    let hidden = id(&write(
        &mut client,
        "POST",
        &format!("/objects/{note}/tags"),
        obj! { "name" => "hidden" }.into(),
        200,
    ));
    assert_ne!(
        card(&mut client, &db, &note)
            .0
            .get("flags")
            .as_f64()
            .expect("numeric field") as i64
            & 8,
        0
    );
    assert!(!grid_has(&mut client, "/cards", &note));
    write(
        &mut client,
        "DELETE",
        &format!("/annotations/{private}"),
        Value::Undefined,
        200,
    );
    assert_ne!(
        card(&mut client, &db, &note)
            .0
            .get("flags")
            .as_f64()
            .expect("numeric field") as i64
            & 8,
        0
    );
    write(
        &mut client,
        "DELETE",
        &format!("/annotations/{hidden}"),
        Value::Undefined,
        200,
    );
    assert_eq!(
        card(&mut client, &db, &note)
            .0
            .get("flags")
            .as_f64()
            .expect("numeric field") as i64
            & 8,
        0
    );

    let since = checkpoint(&mut client);
    let space = write(
        &mut client,
        "POST",
        "/spaces",
        obj! { "name" => "Local HTTP space", "color" => "#a1b2c3", "position" => 99 }.into(),
        200,
    );
    let space_id = id(&space);
    assert!(space.get("objects").as_array().unwrap().is_empty());
    write(
        &mut client,
        "PATCH",
        &format!("/spaces/{space_id}"),
        obj! { "name" => "Renamed HTTP space", "color" => "#112233" }.into(),
        200,
    );
    let space = get(&mut client, &format!("/spaces/{space_id}"));
    assert_eq!(space.get("name").as_str(), Some("Renamed HTTP space"));
    assert_eq!(space.get("color").as_str(), Some("#112233"));
    assert!(contains(&get(&mut client, "/spaces"), &space_id));
    changed(&mut client, &db, &since, &space_id, 1.0, 190.0);
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "PUT",
        &format!("/spaces/{space_id}/objects/{note}"),
        obj! {}.into(),
        200,
    );
    assert!(contains(
        get(&mut client, &format!("/spaces/{space_id}")).get("objects"),
        &note
    ));
    assert!(contains(
        changed(&mut client, &db, &since, &note, 1.0, 9.0)
            .get("object")
            .get("spaces"),
        &space_id
    ));
    write(
        &mut client,
        "DELETE",
        &format!("/spaces/{space_id}/objects/{note}"),
        Value::Undefined,
        200,
    );
    assert!(!contains(
        get(&mut client, &format!("/spaces/{space_id}")).get("objects"),
        &note
    ));
    write(
        &mut client,
        "POST",
        "/objects",
        obj! { "content" => "https://example.invalid/local", "spaceId" => space_id.clone() }.into(),
        200,
    );
    assert!(contains(
        get(&mut client, &format!("/spaces/{space_id}")).get("objects"),
        &url
    ));
    let smart = write(&mut client, "POST", "/spaces", obj! { "name" => "Local smart", "color" => "#123456", "query" => obj! { "filters" => arr!["type:note AND title:\"Changed & escaped\""], "matches" => arr![] } }.into(), 200);
    let smart_id = id(&smart);
    let query_id = id(smart.get("query"));
    same(
        smart.get("objects").at(0).get("queryId"),
        smart.get("query").get("id"),
        "smart query membership",
    );
    assert_eq!(
        smart.get("objects").at(0).get("id").as_str(),
        Some(note.as_str())
    );
    let checked = get(&mut client, &format!("/queries/{query_id}/check"));
    assert!(
        checked
            .get("elapsed")
            .as_f64()
            .expect("numeric field")
            .is_finite()
    );
    same(checked.get("added"), &arr![], "smart added");
    same(checked.get("removed"), &arr![], "smart removed");
    write(
        &mut client,
        "POST",
        &format!("/queries/{query_id}/objects/{note}/exclude"),
        obj! {}.into(),
        200,
    );
    assert!(
        get(&mut client, &format!("/spaces/{smart_id}"))
            .get("objects")
            .as_array()
            .unwrap()
            .is_empty()
    );
    write(
        &mut client,
        "POST",
        "/spaces/reorder",
        arr![smart_id.clone(), space_id.clone()],
        200,
    );
    let spaces = get(&mut client, "/spaces");
    let position = |id: &str| {
        spaces
            .as_array()
            .unwrap()
            .iter()
            .find(|space| space.get("id").as_str() == Some(id))
            .unwrap()
            .get("position")
            .as_f64()
            .expect("numeric field")
    };
    assert!(position(&smart_id) < position(&space_id));
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "DELETE",
        &format!("/spaces/{smart_id}"),
        Value::Undefined,
        200,
    );
    assert!(!contains(&get(&mut client, "/spaces"), &smart_id));
    changed(&mut client, &db, &since, &smart_id, 2.0, 190.0);

    for (action, bit, set) in [
        ("pin", 512, true),
        ("unpin", 512, false),
        ("complete", 65536, true),
        ("uncomplete", 65536, false),
    ] {
        write(
            &mut client,
            "POST",
            &format!("/objects/{note}/{action}"),
            obj! {}.into(),
            200,
        );
        assert_eq!(
            card(&mut client, &db, &note)
                .0
                .get("flags")
                .as_f64()
                .expect("numeric field") as i64
                & bit
                != 0,
            set
        );
    }
    write(
        &mut client,
        "POST",
        "/objects/reorder-pins",
        arr![text.clone(), note.clone()],
        200,
    );
    assert_eq!(
        get(&mut client, &format!("/objects/{text}"))
            .get("position")
            .as_f64()
            .expect("numeric field"),
        0.0
    );
    assert_eq!(
        get(&mut client, &format!("/objects/{note}"))
            .get("position")
            .as_f64()
            .expect("numeric field"),
        1.0
    );
    let pinned = client.send(&Sent::new(
        "GET",
        &format!("/cards/{note}?template=PinnedCard"),
        &[("accept", "text/html"), ("x-partial", "PinnedCard")],
    ));
    status(&pinned, 200);
    assert_eq!(header(&pinned, "x-partial"), Some("PinnedCard"));
    let pinned = String::from_utf8(pinned.body).unwrap();
    assert!(pinned.contains("class=\"tom__item draggable\"") && pinned.contains(&note));
    let shell = html(&mut client, "/everything", true);
    let list = shell
        .split_once("class=\"tom__list sortable\"")
        .unwrap()
        .1
        .split_once("</mymind-top-of-mind>")
        .unwrap()
        .0;
    assert!(
        list.find(&format!("data-id=\"{text}\"")).unwrap()
            < list.find(&format!("data-id=\"{note}\"")).unwrap()
    );
    write(
        &mut client,
        "POST",
        "/batch/tag",
        obj! { "objectIds" => arr![note.clone(), text.clone()], "tag" => "http-bulk" }.into(),
        200,
    );
    assert!(
        get(&mut client, &format!("/objects/{text}/tags"))
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag.get("name").as_str() == Some("http-bulk"))
    );
    write(
        &mut client,
        "POST",
        "/batch/add-to-space",
        obj! { "objectIds" => arr![note.clone(), text.clone()], "spaceId" => space_id.clone() }
            .into(),
        200,
    );
    let members = get(&mut client, &format!("/spaces/{space_id}"));
    assert!(contains(members.get("objects"), &note) && contains(members.get("objects"), &text));

    let before = get(&mut client, "/quota");
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "DELETE",
        &format!("/objects/{note}"),
        Value::Undefined,
        200,
    );
    let (data, rendered, _) = card(&mut client, &db, &note);
    assert!(
        data.get("deleted").truthy()
            && rendered.contains("deleted")
            && rendered.contains("card:restore")
    );
    assert!(grid_has(&mut client, "/cards/trash", &note));
    assert_eq!(
        get(&mut client, "/quota")
            .get("count")
            .as_f64()
            .expect("numeric field"),
        before.get("count").as_f64().expect("numeric field") - 1.0
    );
    changed(&mut client, &db, &since, &note, 2.0, 9.0);
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "POST",
        &format!("/objects/{note}/restore"),
        Value::Undefined,
        200,
    );
    let (data, rendered, _) = card(&mut client, &db, &note);
    assert!(!data.get("deleted").truthy() && rendered.contains("card:open"));
    same(
        get(&mut client, "/quota").get("count"),
        before.get("count"),
        "restored quota",
    );
    changed(&mut client, &db, &since, &note, 1.0, 9.0);
    write(
        &mut client,
        "POST",
        "/batch/delete",
        obj! { "objectIds" => arr![text.clone(), draft.clone()] }.into(),
        200,
    );
    assert!(
        card(&mut client, &db, &text).0.get("deleted").truthy()
            && card(&mut client, &db, &draft).0.get("deleted").truthy()
    );
    let revisions = get(&mut client, &format!("/objects/{note}/revisions"));
    assert!(revisions.as_array().unwrap().len() >= 3 && revisions.at(0).get("isCurrent").truthy());
    let old = revisions.as_array().unwrap().last().unwrap();
    write(
        &mut client,
        "POST",
        &format!("/objects/{note}/revisions/{}/restore", id(old)),
        obj! {}.into(),
        200,
    );
    same(
        &get(&mut client, &format!("/objects/{note}/content")),
        old.get("content").get("body"),
        "revision reader",
    );
    same(
        card(&mut client, &db, &note).0.get("prose"),
        old.get("content").get("body"),
        "revision card",
    );
    let share = write(
        &mut client,
        "POST",
        &format!("/cards/{note}/shares"),
        obj! {}.into(),
        200,
    );
    let share_id = id(&share);
    assert_eq!(
        share.get("url").as_str(),
        Some(format!("/shares/{share_id}").as_str())
    );
    let share_url = share.get("url").as_str().unwrap();
    assert!(html(&mut client, share_url, true).contains(&note));
    assert!(html(&mut client, &format!("/cards/{note}/detail"), false).contains(share_url));
    write(
        &mut client,
        "DELETE",
        &format!("/cards/{note}/shares/{share_id}"),
        Value::Undefined,
        200,
    );
    status(
        &request(
            &mut client,
            "GET",
            share_url,
            &Value::Undefined,
            "text/html",
            "application/json",
        ),
        410,
    );
    let detail = client.send(&Sent::new(
        "GET",
        &format!("/cards/{note}/detail"),
        &[("accept", "text/html"), ("x-partial", "CardDetail")],
    ));
    status(&detail, 200);
    let detail = String::from_utf8(detail.body).unwrap();
    assert!(
        detail.contains("Changed &amp; escaped")
            && detail.contains(&format!("/objects/{note}/prose"))
            && detail.contains(&format!("/objects/{note}/note"))
            && detail.contains("Edited attached note")
            && detail.contains("http-bulk")
    );

    let sql = Sqlite::open(&db, true).unwrap();
    let article = sql
        .one(
            "SELECT id FROM cards WHERE type='Article' AND deleted_ns IS NULL LIMIT 1",
            &[],
        )
        .unwrap()
        .expect("Article fixture")
        .text("id")
        .unwrap()
        .to_owned();
    let image = sql
        .one(
            "SELECT id FROM cards WHERE type='Image' AND deleted_ns IS NULL LIMIT 1",
            &[],
        )
        .unwrap()
        .expect("Image fixture")
        .text("id")
        .unwrap()
        .to_owned();
    drop(sql);
    prose_put(
        &mut client,
        &format!("/objects/{article}/content"),
        &doc("Locally edited article"),
    );
    let detail = html(&mut client, &format!("/cards/{article}/detail"), false);
    assert!(
        detail.contains("data-type=\"card\"")
            && detail.contains(&format!("/objects/{article}/content"))
            && detail.contains("Locally edited article")
    );
    let revisions = get(&mut client, &format!("/objects/{article}/revisions"));
    assert!(!revisions.as_array().unwrap().is_empty());
    write(
        &mut client,
        "POST",
        &format!(
            "/objects/{article}/revisions/{}/restore",
            id(revisions.at(0))
        ),
        obj! {}.into(),
        200,
    );
    same(
        &get(&mut client, &format!("/objects/{article}/content")),
        revisions.at(0).get("content").get("body"),
        "article restore",
    );
    let share = write(
        &mut client,
        "POST",
        &format!("/cards/{image}/shares"),
        obj! {}.into(),
        200,
    );
    let page = html(&mut client, share.get("url").as_str().unwrap(), false);
    assert!(
        regex::Regex::new(r#"<img\b[^>]* src=\""#)
            .unwrap()
            .is_match(&page)
    );
    assert!(
        !regex::Regex::new(r#"<img\b[^>]* data-src="#)
            .unwrap()
            .is_match(&page)
    );
    write(
        &mut client,
        "DELETE",
        &format!("/cards/{image}/shares/{}", id(&share)),
        Value::Undefined,
        200,
    );
    let since = checkpoint(&mut client);
    write(
        &mut client,
        "POST",
        &format!("/cards/{url}/remove-ai-thumbnail"),
        obj! {}.into(),
        200,
    );
    changed(&mut client, &db, &since, &url, 1.0, 9.0);
    let converted = request(
        &mut client,
        "POST",
        "/convert",
        &"# Heading\n**bold**\n- item\n> quote\n```\ncode\n```".into(),
        "application/prose+json",
        "text/markdown",
    );
    status(&converted, 200);
    assert!(header(&converted, "x-object-quota").is_some());
    let converted = value(&converted);
    assert_eq!(converted.get("type").as_str(), Some("doc"));
    assert_eq!(
        converted.get("content").at(0).get("type").as_str(),
        Some("heading")
    );
    let mut nodes = converted.get("content").as_array().unwrap().clone();
    nodes.extend([
        obj! { "type" => "taskList", "content" => arr![obj! { "type" => "taskItem", "attrs" => obj! { "checked" => true }, "content" => arr![doc("task").get("content").at(0).clone()] }] }.into(),
        obj! { "type" => "table", "content" => arr![obj! { "type" => "tableRow", "content" => arr![obj! { "type" => "tableCell", "content" => arr![doc("cell").get("content").at(0).clone()] }] }] }.into(),
        obj! { "type" => "paragraph", "content" => arr![obj! { "type" => "wikiLink", "attrs" => obj! { "target" => "Local idea", "text" => "Local idea", "targetId" => stub.clone() } }, obj! { "type" => "text", "text" => "marked", "marks" => arr![obj! { "type" => "highlight" }, obj! { "type" => "subscript" }, obj! { "type" => "superscript" }] }] }.into(),
    ]);
    write(
        &mut client,
        "PUT",
        &format!("/objects/{note}/prose"),
        obj! { "type" => "doc", "content" => nodes }.into(),
        200,
    );
    let rendered = card(&mut client, &db, &note).1;
    assert!(
        rendered.contains("wiki-link")
            && rendered.contains("<table>")
            && rendered.contains("data-checked=\"true\"")
            && rendered.contains("<mark>")
    );

    let uid = Sqlite::open(&db, true)
        .unwrap()
        .one("SELECT id FROM users LIMIT 1", &[])
        .unwrap()
        .unwrap()
        .text("id")
        .unwrap()
        .to_owned();
    let settings = write(&mut client, "PATCH", &format!("/users/{uid}"), obj! { "colorScheme" => "dark", "locale" => "en-GB", "timeZoneId" => "Europe/London", "soundEnabled" => false }.into(), 200);
    same(
        &settings,
        &obj! { "status" => 200 }.into(),
        "settings acknowledgement",
    );
    write(
        &mut client,
        "POST",
        "/onboard/flags/4096/set",
        Value::Undefined,
        200,
    );
    let shell = html(&mut client, "/everything", true);
    let current = config(&shell);
    assert_eq!(current.get("colorScheme").as_str(), Some("dark"));
    assert_eq!(current.get("locale").as_str(), Some("en-GB"));
    assert_eq!(current.get("timeZoneId").as_str(), Some("Europe/London"));
    assert_eq!(current.get("soundEnabled"), &Value::Bool(false));
    same(
        get(&mut client, "/account.json")
            .get("quotas")
            .get("objects")
            .get("count"),
        get(&mut client, "/quota").get("count"),
        "account quota",
    );
    assert_ne!(
        current.get("onboardFlags").as_f64().expect("numeric field") as i64 & 4096,
        0
    );
    write(
        &mut client,
        "POST",
        "/onboard/flags/4096/unset",
        Value::Undefined,
        200,
    );
    let shell = html(&mut client, "/everything", true);
    assert_eq!(
        config(&shell)
            .get("onboardFlags")
            .as_f64()
            .expect("numeric field") as i64
            & 4096,
        0
    );
    assert!(shell.contains("<mymind-top-of-mind class=\"tom \">"));
    write(
        &mut client,
        "POST",
        "/onboard/flags/128/set",
        Value::Undefined,
        200,
    );
    assert!(
        html(&mut client, "/everything", true)
            .contains("<mymind-top-of-mind class=\"tom introduced\">")
    );
    let introduction = client.send(&Sent::new(
        "GET",
        "/components/TomIntroductionModal",
        &[("accept", "*/*")],
    ));
    assert!(
        String::from_utf8(introduction.body)
            .unwrap()
            .contains("data-name=\"tom-introduction-modal\"")
    );

    let since = checkpoint(&mut client);
    let short = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "content" => "Short-lived note" }.into(),
        201,
    ));
    write(
        &mut client,
        "PATCH",
        &format!("/objects/{short}"),
        obj! { "title" => "Edited before delete" }.into(),
        200,
    );
    write(
        &mut client,
        "DELETE",
        &format!("/objects/{short}"),
        Value::Undefined,
        200,
    );
    write(
        &mut client,
        "POST",
        &format!("/objects/{short}/tags"),
        obj! { "name" => "stale-tab-edit" }.into(),
        200,
    );
    for (restore, expected) in [(false, 2.0), (true, 1.0)] {
        if restore {
            write(
                &mut client,
                "POST",
                &format!("/objects/{short}/restore"),
                Value::Undefined,
                200,
            );
        }
        let changes = request(
            &mut client,
            "GET",
            &format!("/changes?since={since}&render=true"),
            &Value::Undefined,
            "application/msgpack",
            "application/json",
        );
        let changes = value(&changes);
        let items = changes
            .as_array()
            .unwrap()
            .iter()
            .filter(|change| change.get("object").get("id").as_str() == Some(short.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1, "coalesced batched change");
        assert_eq!(
            items[0].get("type").as_f64().expect("numeric field"),
            expected
        );
    }
    let since = checkpoint(&mut client);
    let other_draft = id(&write(
        &mut client,
        "POST",
        "/objects",
        obj! { "type" => "Note", "isDraft" => true, "prose" => doc("Other tab draft") }.into(),
        201,
    ));
    assert_eq!(
        changed(&mut client, &db, &since, &other_draft, 2.0, 9.0)
            .get("type")
            .as_f64()
            .expect("numeric field"),
        2.0
    );
    write(
        &mut client,
        "POST",
        "/serendipity/complete",
        obj! {}.into(),
        200,
    );
    write(
        &mut client,
        "POST",
        "/promotions/local-promotion/dismiss",
        obj! {}.into(),
        200,
    );
    same(
        &get(&mut client, "/achievements/local-achievement?dismiss=true"),
        &obj! {}.into(),
        "achievement acknowledgement",
    );
    write(
        &mut client,
        "POST",
        "/jserrors",
        obj! { "message" => "local check" }.into(),
        200,
    );
    let settings = Sqlite::open(&db, true)
        .unwrap()
        .one("SELECT settings FROM users LIMIT 1", &[])
        .unwrap()
        .unwrap();
    let settings = json::parse(settings.text("settings").unwrap()).unwrap();
    for (key, item) in [
        ("completedFlows", "serendipity"),
        ("dismissedPromotions", "local-promotion"),
        ("dismissedAchievements", "local-achievement"),
    ] {
        assert!(
            settings
                .get(key)
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(item))
        );
    }
    let since = checkpoint(&mut client);
    let before = card(&mut client, &db, &note).2;
    let bad = request(
        &mut client,
        "POST",
        "/batch/tag",
        &obj! { "objectIds" => arr![note.clone(), "missing"], "tag" => "must-roll-back" }.into(),
        "application/json",
        "application/json",
    );
    status(&bad, 404);
    assert_eq!(
        header(&bad, "content-type"),
        Some("application/problem+json")
    );
    assert!(
        card(&mut client, &db, &note).2 == before,
        "bulk rollback preserves card"
    );
    assert_eq!(checkpoint(&mut client), since);
    status(
        &request(
            &mut client,
            "POST",
            "/objects",
            &"{".into(),
            "application/json",
            "application/json",
        ),
        400,
    );
    status(
        &request(
            &mut client,
            "POST",
            "/objects",
            &obj! { "type" => "Media", "uploadId" => "pending" }.into(),
            "application/json",
            "application/json",
        ),
        404,
    );
    let messages = hub.messages(Some("account:updated"));
    for target in [
        "object:created",
        "object:updated",
        "object:bumped",
        "object:deleted",
        "tag:created",
        "tag:deleted",
        "space:created",
        "space:updated",
        "space:deleted",
        "account:updated",
    ] {
        assert!(
            messages.iter().any(
                |message| message.get("type").as_f64().expect("numeric field") == 1.0
                    && message.get("target").as_str() == Some(target)
                    && message
                        .get("arguments")
                        .as_array()
                        .is_some_and(|args| args.len() == 1)
            ),
            "committed hub target {target}"
        );
    }
    assert!(messages.iter().any(|message| {
        message.get("target").as_str() == Some("tag:created")
            && message.get("arguments").at(0).get("objectId").as_str() == Some(note.as_str())
            && message
                .get("arguments")
                .at(0)
                .get("flags")
                .as_f64()
                .expect("numeric field")
                == 8.0
    }));
    assert!(
        !messages.iter().any(
            |message| message.get("arguments").at(0).get("name").as_str() == Some("must-roll-back")
        ),
        "rollback emits no hub event"
    );
    readonly_assertions(&root, &db, &note);
    assert!(
        server.logged(&["[error]"]).is_empty(),
        "native requests logged no unhandled errors"
    );
}

// Capture readonly stderr directly in a file so shutdown cannot race a detached log-reader thread.
fn readonly_server(root: &Path, file: &Path, errors: &Path) -> Server {
    let native = command(root);
    let mut wrapper = std::process::Command::new("/bin/sh");
    wrapper
        .env_clear()
        .envs(
            native
                .get_envs()
                .filter_map(|(key, value)| value.map(|value| (key, value))),
        )
        .current_dir(root)
        .arg("-c")
        .arg("exec \"$1\" serve 2>\"$2\"")
        .arg("readonly-logger")
        .arg(native.get_program())
        .arg(errors)
        .env("STORE_DB", file)
        .env("STORE_READONLY", "1");
    Server::start(wrapper, "mymind replica on http://localhost:")
}

// Prove acknowledged readonly writes preserve every table and do not broadcast mutations.
fn readonly_assertions(root: &Path, source: &Path, note: &str) {
    let file = root.join("readonly.db");
    Sqlite::open(source, false)
        .unwrap()
        .exec_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    std::fs::copy(source, &file).unwrap();
    let errors = root.join("readonly.stderr");
    let server = readonly_server(root, &file, &errors);
    let mut client = Client::new(server.port);
    let socket = Hub::connect(server.port);
    let before = snapshot(&file);
    for (method, path, body) in [
        (
            "POST",
            "/objects".into(),
            obj! { "content" => "Ignored note" }.into(),
        ),
        (
            "PATCH",
            format!("/objects/{note}"),
            obj! { "title" => "Ignored" }.into(),
        ),
        ("DELETE", format!("/objects/{note}"), Value::Undefined),
        (
            "POST",
            format!("/objects/{note}/tags"),
            obj! { "name" => "ignored" }.into(),
        ),
        (
            "POST",
            "/spaces".into(),
            obj! { "name" => "Ignored" }.into(),
        ),
        ("POST", "/onboard/flags/4096/set".into(), Value::Undefined),
    ] {
        let answer = request(
            &mut client,
            method,
            &path,
            &body,
            "application/json",
            "application/json",
        );
        assert!(
            (200..300).contains(&answer.status.get(9..12).unwrap().parse::<u16>().unwrap()),
            "readonly write acknowledged"
        );
    }
    assert!(
        snapshot(&file) == before,
        "readonly preserves every stored typed row"
    );
    assert!(
        !socket.messages(None).iter().any(|message| message
            .get("type")
            .as_f64()
            .expect("numeric field")
            == 1.0)
    );
    assert!(grid_has(&mut client, "/cards", note));
    let hydrate = request(
        &mut client,
        "POST",
        "/cards/grid",
        &arr![note],
        "application/msgpack",
        "application/json",
    );
    assert_eq!(value(&hydrate).as_array().unwrap().len(), 1);
    drop(socket);
    drop(client);
    drop(server);
    let stderr = std::fs::read_to_string(errors).unwrap();
    assert!(
        stderr.lines().all(|line| !line.starts_with("[error]")),
        "readonly requests logged no unhandled errors"
    );
}
