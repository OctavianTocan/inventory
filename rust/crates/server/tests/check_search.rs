//! Complete search assertions formerly in `tools/check-search.mjs`, with isolated Rust HTTP and hosted API fixtures.

mod stage_b_common;
mod stage_b_expectations;

use async_trait::async_trait;
use mm_core::enrichment::{Enrichment, EnrichmentOptions};
use mm_core::host::Screenshot;
use mm_core::search::Search;
use mm_core::sql::{Sql, SqlExt};
use mm_core::store::Store;
use mm_core::store_schema::SCHEMA;
use mm_core::store_write::Writer;
use mm_core::{Error, Result};
use mm_js::{Value, arr, json, obj};
use mm_sqlite::Sqlite;
use mymind::capture_http::CaptureTransport;
use mymind::fetch::NativeFetch;
use mymind::local_media::LocalMedia;
use mymind::tool_capture_fixtures::{EmbeddingReply, Fixtures, fixture_media};
use stage_b_common::*;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

// Build a predictable 768-dimensional unit vector.
fn vector(axis: usize) -> Value {
    Value::Array(
        (0..768)
            .map(|i| Value::Number(if i == axis { 1.0 } else { 0.0 }))
            .collect(),
    )
}

// Persist vectors with the production analysis mutation, including summary and automatic labels.
fn analyze(writer: &Writer, id: &str, axis: usize, summary: &str) {
    writer.run(|writer| writer.analysis(id, &obj! { "embedding" => vector(axis), "title" => "", "summary" => summary, "tags" => arr![obj! { "name" => "synthetic ai label" }] }.into())).unwrap();
}

// Percent-encode search input without building an untrusted URL string.
fn encoded(text: &str) -> String {
    url::form_urlencoded::byte_serialize(text.as_bytes()).collect()
}

// Search over the actual HTTP route, retaining parser options.
fn query(client: &mut Client, q: &str, options: &[(&str, &str)]) -> Value {
    let mut path = format!("/search?q={}", encoded(q));
    for (name, value) in options {
        path.push_str(&format!("&{}={}", encoded(name), encoded(value)));
    }
    get(client, &path)
}

// Test a query's object membership over HTTP.
fn has(client: &mut Client, q: &str, id: &str) -> bool {
    contains(query(client, q, &[]).get("matches"), id)
}

// Search direct store inputs for the capture queue and transactional trigger assertions.
fn direct_has(search: &Search, q: &str, id: &str) -> bool {
    search
        .text(q, &Value::Undefined)
        .unwrap()
        .matches
        .iter()
        .any(|item| item.id == id)
}

// Supply the same synthetic PNG for capture screenshots without launching a browser.
struct FixedScreenshot(Vec<u8>);
#[async_trait(?Send)]
impl Screenshot for FixedScreenshot {
    async fn capture(&self, url: &str, offline: bool) -> Result<Vec<u8>> {
        assert!(url.starts_with("http://127.0.0.1:") && offline);
        Ok(self.0.clone())
    }
}

#[test]
fn complete_synthetic_search_mutation_capture_and_semantic_scenarios() {
    mymind::runtime::run(async {
        mm_core::runtime::set_env(HashMap::new());
        let root = root("search");
        let file = root.join("synthetic.db");
        synthetic(&file);
        let media = fixture_media(&root).unwrap();
        let png = media.png.clone();
        let fixtures = Fixtures::start(media).unwrap();
        let mut server = start(&root, &file, false, &[]);
        let mut client = Client::new(server.port);
        let store = Rc::new(Store::new(Rc::new(Sqlite::open(&file, false).unwrap())).unwrap());
        let writer = Rc::new(Writer::new(store.clone(), |_, _| {}).unwrap());
        let search = Search::new(store.clone(), None, true).unwrap();
        let first = id(&write(
            &mut client,
            "POST",
            "/objects",
            obj! { "title" => "Orchard alpha", "prose" => doc("silver birch blue orchard") }.into(),
            201,
        ));
        let second = id(&write(
            &mut client,
            "POST",
            "/objects",
            obj! { "title" => "Orchard beta", "prose" => doc("birch silver blueberry") }.into(),
            201,
        ));
        let result = query(&mut client, "blue", &[("wildcard", "true")]);
        let mut keys = result
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, _)| key.into_owned())
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, ["count", "elapsed", "matches", "query"]);
        assert_eq!(result.get("query").as_str(), Some("blue*"));
        assert_eq!(result.get("count").as_f64().expect("numeric field"), 2.0);
        assert!(
            has(&mut client, "blue", &first)
                && !has(&mut client, "blue", &second)
                && has(&mut client, "blue*", &second)
        );
        assert!(
            result
                .get("matches")
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item.as_object().unwrap().len() == 2
                    && item
                        .get("score")
                        .as_f64()
                        .expect("numeric field")
                        .is_finite())
        );
        assert_eq!(
            query(&mut client, "design", &[("wildcard", "true")])
                .get("count")
                .as_f64()
                .expect("numeric field"),
            0.0,
            "writable search overrides recordings"
        );
        store
            .db()
            .exec_batch("INSERT INTO search_fts(search_fts,rank) VALUES('integrity-check',1)")
            .unwrap();

        write(
            &mut client,
            "POST",
            &format!("/objects/{first}/tags"),
            obj! { "name" => "trial design" }.into(),
            200,
        );
        for q in [
            "type:note AND tag:\"trial design\"",
            "tag:trial design",
            "label:trial design",
            "#trial design",
            "tag:trial*",
        ] {
            assert!(has(&mut client, q, &first), "tag spelling {q}");
        }
        assert!(!has(&mut client, "tag:trial", &first));
        let link = id(writer
            .run(|writer| {
                writer.create(
                    &obj! { "content" => "https://sub.example.invalid/path" }.into(),
                    &Value::Undefined,
                )
            })
            .unwrap()
            .get("body"));
        assert!(direct_has(&search, "type:placeholder", &link));
        let pending = store.card(&link, None).unwrap().unwrap();
        let mut baseline: Value = obj! { "data" => json::parse(&pending.data).unwrap(), "extras" => json::parse(&pending.extras).unwrap() }.into();
        writer
            .run(|writer| {
                writer.enrich(
                    &link,
                    &obj! { "data" => obj! { "type" => "WebPage" } }.into(),
                    &mut baseline,
                )
            })
            .unwrap();
        for q in [
            "site:example.invalid",
            "domain:sub.example.invalid",
            "site:example",
        ] {
            assert!(has(&mut client, q, &link));
        }
        assert!(!has(&mut client, "site:ample.invalid", &link));
        assert!(direct_has(&search, "type:web-page", &link));
        write(
            &mut client,
            "PATCH",
            &format!("/objects/{link}"),
            obj! { "format" => "pdf" }.into(),
            200,
        );
        assert!(
            has(&mut client, "format:pdf AND action:read", &link)
                && !has(&mut client, "action:watch", &link)
        );
        for q in [
            "\"silver birch\"",
            "“silver birch”",
            "birch -blueberry",
            "+silver AND NOT blueberry",
            "after:2000-01-01 before:2100-01-01",
            "created:[2000-01-01 TO 2100-01-01]",
        ] {
            assert!(has(&mut client, q, &first), "query {q}");
        }
        assert!(
            !has(&mut client, "\"silver birch\"", &second)
                && !has(&mut client, "birch -blueberry", &second)
        );
        assert!(
            has(
                &mut client,
                "(title:alpha OR title:beta) AND -tag:\"trial design\"",
                &second
            ) && !has(
                &mut client,
                "(title:alpha OR title:beta) AND -tag:\"trial design\"",
                &first
            )
        );
        assert!(
            query(&mut client, "title:\"a\" OR 1=1 --\"", &[])
                .get("matches")
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            query(&mut client, "orchard", &[("sortBy", "bumped asc")])
                .get("matches")
                .at(0)
                .get("id")
                .as_str(),
            Some(first.as_str())
        );
        write(
            &mut client,
            "POST",
            &format!("/objects/{first}/complete"),
            obj! {}.into(),
            200,
        );
        assert!(
            has(&mut client, "completed:true", &first)
                && !has(&mut client, "completed:false", &first)
        );
        status(
            &request(
                &mut client,
                "GET",
                &format!("/search?q={}", "a".repeat(8200)),
                &Value::Undefined,
                "application/json",
                "application/json",
            ),
            400,
        );

        write(&mut client, "PATCH", &format!("/objects/{first}"), obj! { "title" => "Cypress fixture", "description" => "violet lantern", "ocr" => "amber typography" }.into(), 200);
        assert!(
            has(&mut client, "cypress", &first)
                && !has(&mut client, "title:alpha", &first)
                && has(&mut client, "violet", &first)
                && has(&mut client, "typography", &first)
        );
        write(
            &mut client,
            "PUT",
            &format!("/objects/{first}/note"),
            doc("cardnotezz"),
            200,
        );
        assert!(has(&mut client, "cardnotezz", &first));
        write(
            &mut client,
            "PUT",
            &format!("/objects/{first}/content"),
            doc("readernewzz"),
            200,
        );
        assert!(has(&mut client, "readernewzz", &first) && !has(&mut client, "birch", &first));
        let tag = store
            .card_tags(&first)
            .unwrap()
            .into_iter()
            .find(|tag| tag.name == "trial design")
            .unwrap();
        write(
            &mut client,
            "DELETE",
            &format!("/annotations/{}", tag.id),
            Value::Undefined,
            200,
        );
        assert!(!has(&mut client, "trial", &first));
        write(
            &mut client,
            "DELETE",
            &format!("/objects/{first}"),
            Value::Undefined,
            200,
        );
        assert!(!has(&mut client, "cypress", &first));
        write(
            &mut client,
            "POST",
            &format!("/objects/{first}/restore"),
            obj! {}.into(),
            200,
        );
        assert!(has(&mut client, "cypress", &first));
        let privacy = id(&write(
            &mut client,
            "POST",
            &format!("/objects/{first}/tags"),
            obj! { "name" => "private" }.into(),
            200,
        ));
        assert!(!has(&mut client, "cypress", &first));
        write(
            &mut client,
            "DELETE",
            &format!("/annotations/{privacy}"),
            Value::Undefined,
            200,
        );
        assert!(has(&mut client, "cypress", &first));

        let capture = id(writer
            .run(|writer| {
                writer.create(
                    &obj! { "content" => format!("{}/ArticleQuality", fixtures.base) }.into(),
                    &Value::Undefined,
                )
            })
            .unwrap()
            .get("body"));
        let jobs = Enrichment::new(EnrichmentOptions {
            store: store.clone(),
            writer: writer.clone(),
            media: Rc::new(LocalMedia::new(root.join("media"))),
            resource: Rc::new(CaptureTransport::new()),
            screenshot: Rc::new(FixedScreenshot(png)),
            fetch: Rc::new(NativeFetch::new()),
            offline: true,
            readonly: false,
        });
        jobs.schedule(&capture, true);
        jobs.drain().await;
        jobs.close();
        assert!(
            direct_has(&search, "\"Mind maps\"", &capture)
                && direct_has(&search, "body:knowledge", &capture)
        );
        writer
            .run(|writer| writer.prose(&capture, &doc("replacementreaderzz"), "content"))
            .unwrap();
        assert!(
            direct_has(&search, "replacementreaderzz", &capture)
                && !direct_has(&search, "body:knowledge", &capture)
        );
        analyze(&writer, &first, 0, "summaryonlyzz");
        assert!(has(&mut client, "summaryonlyzz", &first) && has(&mut client, "synthetic", &first));
        let before = search.text("rollbackzz", &Value::Undefined).unwrap().count;
        let rollback: Result<()> = writer.run(|writer| {
            writer.add_tag(&first, &"rollbackzz".into())?;
            Err(Error::internal("rollback"))
        });
        assert!(rollback.is_err());
        assert_eq!(
            search.text("rollbackzz", &Value::Undefined).unwrap().count,
            before
        );

        write(
            &mut client,
            "POST",
            &format!("/objects/{first}/tags"),
            obj! { "name" => "trial label" }.into(),
            200,
        );
        write(
            &mut client,
            "POST",
            &format!("/objects/{second}/tags"),
            obj! { "name" => "Trial Label" }.into(),
            200,
        );
        let tags = get(&mut client, "/search/suggestions?q=trial");
        assert_eq!(tags.as_array().unwrap().len(), 1);
        assert_eq!(
            tags.at(0).get("count").as_f64().expect("numeric field"),
            2.0
        );
        assert_eq!(
            tags.at(0).get("score").as_f64().expect("numeric field"),
            4.0
        );
        assert_eq!(tags.at(0).get("isAI"), &Value::Bool(false));
        let types = get(&mut client, "/search/suggestions");
        let notes = types
            .as_array()
            .unwrap()
            .iter()
            .find(|suggestion| suggestion.get("queryText").as_str() == Some("type:note"))
            .unwrap();
        assert_eq!(notes.get("count").as_f64().expect("numeric field"), 2.0);
        assert_eq!(notes.get("score").as_f64().expect("numeric field"), 6.0);
        write(
            &mut client,
            "POST",
            "/objects",
            obj! { "isDraft" => true, "content" => "draftonlyzz" }.into(),
            201,
        );
        write(
            &mut client,
            "POST",
            "/objects",
            obj! { "isStub" => true, "content" => "stubonlyzz" }.into(),
            201,
        );
        assert_eq!(
            query(&mut client, "draftonlyzz OR stubonlyzz", &[])
                .get("count")
                .as_f64()
                .expect("numeric field"),
            0.0
        );
        write(
            &mut client,
            "DELETE",
            &format!("/objects/{second}"),
            Value::Undefined,
            200,
        );
        assert_eq!(
            get(&mut client, "/search/suggestions?q=trial")
                .at(0)
                .get("count")
                .as_f64()
                .expect("numeric field"),
            1.0
        );
        write(
            &mut client,
            "POST",
            &format!("/objects/{second}/restore"),
            obj! {}.into(),
            200,
        );

        let space = write(&mut client, "POST", "/spaces", obj! { "name" => "Synthetic smart", "query" => obj! { "filters" => arr!["tag:trial label", "-title:beta"], "matches" => arr![obj! { "id" => link.clone(), "score" => 99 }] } }.into(), 200);
        let space_id = id(&space);
        let query_id = id(space.get("query"));
        assert_eq!(space.get("objects").as_array().unwrap().len(), 1);
        assert_eq!(
            space.get("objects").at(0).get("id").as_str(),
            Some(first.as_str())
        );
        same(
            space.get("objects").at(0).get("queryId"),
            space.get("query").get("id"),
            "smart evaluates filters",
        );
        write(
            &mut client,
            "PATCH",
            &format!("/objects/{second}"),
            obj! { "title" => "eligible now" }.into(),
            200,
        );
        let result = get(&mut client, &format!("/queries/{query_id}/check"));
        assert_eq!(result.get("added").as_array().unwrap().len(), 1);
        assert_eq!(
            result.get("added").at(0).get("id").as_str(),
            Some(second.as_str())
        );
        same(result.get("removed"), &arr![], "smart no removal");
        assert_eq!(
            get(&mut client, &format!("/spaces/{space_id}"))
                .get("objects")
                .as_array()
                .unwrap()
                .len(),
            2
        );
        write(
            &mut client,
            "DELETE",
            &format!("/objects/{first}"),
            Value::Undefined,
            200,
        );
        same(
            get(&mut client, &format!("/queries/{query_id}/check")).get("removed"),
            &arr![first.clone()],
            "smart removes trashed",
        );
        write(
            &mut client,
            "POST",
            &format!("/objects/{first}/restore"),
            obj! {}.into(),
            200,
        );
        get(&mut client, &format!("/queries/{query_id}/check"));
        write(
            &mut client,
            "POST",
            &format!("/queries/{query_id}/objects/{first}/exclude"),
            obj! {}.into(),
            200,
        );
        same(
            get(&mut client, &format!("/queries/{query_id}/check")).get("added"),
            &arr![],
            "exclusion persists",
        );
        assert!(!contains(
            get(&mut client, &format!("/spaces/{space_id}")).get("objects"),
            &first
        ));
        write(
            &mut client,
            "PUT",
            &format!("/spaces/{space_id}/objects/{link}"),
            obj! {}.into(),
            200,
        );
        store
            .db()
            .execute(
                "INSERT OR REPLACE INTO api_metadata VALUES (?,200,?,?)",
                &[
                    format!("/spaces/{space_id}").into(),
                    "{}".into(),
                    r#"{"checked":"2000-01-01T00:00:00Z"}"#.into(),
                ],
            )
            .unwrap();
        same(
            get(&mut client, &format!("/queries/{query_id}/check")).get("added"),
            &arr![],
            "manual membership preserved",
        );
        let updated = get(&mut client, &format!("/spaces/{space_id}"));
        assert!(
            updated
                .get("objects")
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item.get("id").as_str() == Some(link.as_str())
                    && !item.get("queryId").truthy())
        );
        assert_ne!(
            updated.get("query").get("checked").as_str(),
            Some("2000-01-01T00:00:00Z")
        );
        write(
            &mut client,
            "DELETE",
            &format!("/objects/{second}"),
            Value::Undefined,
            200,
        );
        let readonly = root.join("preview.db");
        store
            .db()
            .exec_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .unwrap();
        std::fs::copy(&file, &readonly).unwrap();
        let preview = start(&root, &readonly, true, &[]);
        let mut preview_client = Client::new(preview.port);
        same(
            get(&mut preview_client, &format!("/queries/{query_id}/check")).get("removed"),
            &arr![second.clone()],
            "readonly query computes removal",
        );
        let unchanged = get(&mut preview_client, &format!("/spaces/{space_id}"));
        assert!(contains(unchanged.get("objects"), &second));
        same(
            unchanged.get("query").get("checked"),
            updated.get("query").get("checked"),
            "readonly query does not persist checked",
        );
        drop(preview);
        write(
            &mut client,
            "POST",
            &format!("/objects/{second}/restore"),
            obj! {}.into(),
            200,
        );

        analyze(&writer, &second, 1, "synthetic analysis");
        same(
            get(&mut client, "/embeddings/search?q=forest&modelId=2&take=2").get("matches"),
            &arr![],
            "no key semantic degradation",
        );
        same(
            query(&mut client, "cypress", &[("semantic", "true")]).get("matches"),
            query(&mut client, "cypress", &[]).get("matches"),
            "no key blend text fallback",
        );
        let related = get(&mut client, &format!("/embeddings/related/{first}?take=1"));
        assert_eq!(related.get("matches").as_array().unwrap().len(), 1);
        assert_eq!(
            related.get("matches").at(0).get("id").as_str(),
            Some(second.as_str())
        );
        same(
            get(&mut client, "/embeddings/related/missing?take=100").get("matches"),
            &arr![],
            "missing related empty",
        );
        write(
            &mut client,
            "DELETE",
            &format!("/objects/{second}"),
            Value::Undefined,
            200,
        );
        assert_eq!(
            get(&mut client, &format!("/embeddings/related/{first}"))
                .get("count")
                .as_f64()
                .expect("numeric field"),
            0.0
        );
        write(
            &mut client,
            "POST",
            &format!("/objects/{second}/restore"),
            obj! {}.into(),
            200,
        );
        drop(client);
        drop(server);
        server = start(
            &root,
            &file,
            false,
            &[("GEMINI_API_KEY", "synthetic-test-key")],
        );
        client = Client::new(server.port);
        same(
            get(&mut client, "/embeddings/search?q=forest").get("matches"),
            &arr![],
            "offline key without loopback provider degrades",
        );
        drop(client);
        drop(server);
        server = start(
            &root,
            &file,
            false,
            &[
                ("GEMINI_API_KEY", "synthetic-test-key"),
                ("GEMINI_BASE_URL", &fixtures.base),
            ],
        );
        client = Client::new(server.port);
        let result = get(
            &mut client,
            "/embeddings/search?q=forest&modelId=136434018&take=2",
        );
        assert_eq!(
            result.get("matches").at(0).get("id").as_str(),
            Some(first.as_str())
        );
        assert_eq!(
            result
                .get("matches")
                .at(0)
                .get("score")
                .as_f64()
                .expect("numeric field"),
            1.0
        );
        assert_eq!(
            result
                .get("matches")
                .at(1)
                .get("score")
                .as_f64()
                .expect("numeric field"),
            0.0
        );
        assert_eq!(
            get(&mut client, "/embeddings/search?q=forest&take=0")
                .get("count")
                .as_f64()
                .expect("numeric field"),
            0.0
        );
        let filtered = get(
            &mut client,
            "/embeddings/search?q=forest&filter=title:cypress",
        );
        assert_eq!(filtered.get("matches").as_array().unwrap().len(), 1);
        assert_eq!(
            filtered.get("matches").at(0).get("id").as_str(),
            Some(first.as_str())
        );
        let low = query(
            &mut client,
            "cypress",
            &[("semantic", "true"), ("semanticBoost", "0")],
        );
        same(
            low.get("matches"),
            query(&mut client, "cypress", &[]).get("matches"),
            "zero boost unchanged",
        );
        let high = query(
            &mut client,
            "cypress",
            &[("semantic", "true"), ("semanticBoost", "10")],
        );
        let score = |value: &Value| {
            value
                .get("matches")
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item.get("id").as_str() == Some(first.as_str()))
                .unwrap()
                .get("score")
                .as_f64()
                .expect("numeric field")
        };
        assert!(score(&high) > score(&low));
        assert_eq!(
            high.get("matches").at(0).get("id").as_str(),
            Some(first.as_str())
        );
        assert!(!contains(
            query(
                &mut client,
                "forest -title:cypress",
                &[("semantic", "true")]
            )
            .get("matches"),
            &first
        ));
        let call = fixtures
            .requests()
            .into_iter()
            .find(|call| {
                call.path.ends_with(":embedContent")
                    && call
                        .body
                        .get("content")
                        .get("parts")
                        .at(0)
                        .get("text")
                        .as_str()
                        .is_some_and(|text| text.contains("query: forest"))
            })
            .expect("Gemini request");
        assert_eq!(
            call.body
                .get("output_dimensionality")
                .as_f64()
                .expect("numeric field"),
            768.0
        );
        analyze(&writer, &second, 0, "synthetic analysis");
        let scoped = query(&mut client, "forest title:cypress", &[("semantic", "true")]);
        assert_eq!(scoped.get("matches").as_array().unwrap().len(), 1);
        assert_eq!(
            scoped.get("matches").at(0).get("id").as_str(),
            Some(first.as_str())
        );

        let edit_file = file.clone();
        let edit_id = first.clone();
        fixtures.control(|control| {
            control.embedding = Some(Arc::new(move |_| {
                let store =
                    Rc::new(Store::new(Rc::new(Sqlite::open(&edit_file, false).unwrap())).unwrap());
                Writer::new(store, |_, _| {})
                    .unwrap()
                    .run(|writer| writer.trash(&edit_id, false))
                    .unwrap();
                EmbeddingReply {
                    status: None,
                    body: obj! { "embedding" => obj! { "values" => vector(0) } }.into(),
                }
            }))
        });
        assert!(!contains(
            query(&mut client, "cypress", &[("semantic", "true")]).get("matches"),
            &first
        ));
        writer.run(|writer| writer.trash(&first, true)).unwrap();
        let edit_file = file.clone();
        let edit_id = first.clone();
        fixtures.control(|control| {
            control.embedding = Some(Arc::new(move |_| {
                let store =
                    Rc::new(Store::new(Rc::new(Sqlite::open(&edit_file, false).unwrap())).unwrap());
                Writer::new(store, |_, _| {})
                    .unwrap()
                    .run(|writer| {
                        writer.patch(&edit_id, &obj! { "title" => "changed during query" }.into())
                    })
                    .unwrap();
                EmbeddingReply {
                    status: None,
                    body: obj! { "embedding" => obj! { "values" => vector(0) } }.into(),
                }
            }))
        });
        assert_eq!(
            get(
                &mut client,
                "/embeddings/search?q=forest&filter=title:cypress"
            )
            .get("count")
            .as_f64()
            .expect("numeric field"),
            0.0
        );
        writer
            .run(|writer| writer.patch(&first, &obj! { "title" => "Cypress fixture" }.into()))
            .unwrap();
        fixtures.control(|control| {
            control.embedding = Some(Arc::new(|_| EmbeddingReply {
                status: Some(400),
                body: obj! {}.into(),
            }))
        });
        same(
            get(&mut client, "/embeddings/search?q=forest").get("matches"),
            &arr![],
            "provider failure empty semantic matches",
        );
        same(
            query(&mut client, "cypress", &[("semantic", "true")]).get("matches"),
            query(&mut client, "cypress", &[]).get("matches"),
            "provider failure text fallback",
        );
        fixtures.control(|control| control.embedding = None);
    });
}

#[test]
fn backfill_estimate_atomic_validation_retry_usage_and_resume() {
    let root = root("search-backfill");
    let file = root.join("backfill.db");
    synthetic(&file);
    let store = Rc::new(Store::new(Rc::new(Sqlite::open(&file, false).unwrap())).unwrap());
    let writer = Writer::new(store.clone(), |_, _| {}).unwrap();
    let _ids = (0..3)
        .map(|i| {
            id(writer
                .run(|writer| {
                    writer.create(
                        &obj! { "title" => format!("Backfill {i}"), "content" => "synthetic body" }
                            .into(),
                        &Value::Undefined,
                    )
                })
                .unwrap()
                .get("body"))
        })
        .collect::<Vec<_>>();
    let prompt = stage_b_expectations::backfill_prompt();
    let api = FakeApi::start(|_, _| {
        (200, obj! { "embeddings" => arr![obj! { "values" => vector(0) }, obj! { "values" => arr![1] }] }.into())
    });
    let mut rejected = command(&root);
    fixture_connections(&mut rejected, &root, &api.base);
    let rejected = rejected
        .arg("embed-library")
        .arg("--db")
        .arg(&file)
        .arg("--run")
        .env("GEMINI_API_KEY", "synthetic-test-key")
        .env(
            "GEMINI_BASE_URL",
            format!(
                "http://127.0.0.1:{}",
                if url::Url::parse(&api.base).unwrap().port().unwrap() == 65535 {
                    1
                } else {
                    url::Url::parse(&api.base).unwrap().port().unwrap() + 1
                }
            ),
        )
        .output()
        .unwrap();
    assert_eq!(
        rejected.status.code(),
        Some(86),
        "nonfixture backfill connections are rejected before connecting: {}",
        String::from_utf8_lossy(&rejected.stderr).replace("synthetic-test-key", "[redacted]")
    );
    assert!(
        api.calls().is_empty(),
        "rejected destination sends no request"
    );
    let estimate = backfill(&root, &file, &["--limit", "2"], &api.base);
    assert!(estimate.status.success());
    let estimate = String::from_utf8(estimate.stdout).unwrap();
    assert!(estimate.contains("Cards without vectors: 2;") && estimate.contains("Estimate only."));
    let captures =
        regex::Regex::new(r"Estimated text input: (\d+) tokens.*estimated cost: \$(\d+\.\d+)")
            .unwrap();
    let estimate = captures.captures(&estimate).unwrap();
    assert!(estimate[1].parse::<usize>().unwrap() > 0 && estimate[2].parse::<f64>().unwrap() > 0.0);
    assert!(api.calls().is_empty(), "dry run never embeds");
    let failed = backfill(&root, &file, &["--limit", "2", "--run"], &api.base);
    assert!(
        !failed.status.success(),
        "invalid vector rejects complete batch"
    );
    assert_eq!(
        store
            .db()
            .one("SELECT COUNT(*) AS n FROM embeddings", &[])
            .unwrap()
            .unwrap()
            .int("n"),
        Some(0)
    );
    drop(api);
    let api = FakeApi::start(|index, input| {
        if index <= 2 {
            return (if index == 1 { 429 } else { 503 }, obj! {}.into());
        }
        (200, obj! { "embeddings" => Value::Array(input.get("requests").as_array().unwrap().iter().map(|_| obj! { "values" => vector(0) }.into()).collect()), "usageMetadata" => obj! { "promptTokenCount" => 42 } }.into())
    });
    let run = backfill(&root, &file, &["--limit", "2", "--run"], &api.base);
    assert!(run.status.success(), "backfill retries succeed");
    let output = String::from_utf8(run.stdout).unwrap();
    assert!(output.contains("Embedded 2/2; API input tokens: 42"));
    let requests = api.calls();
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests[2]
            .get("requests")
            .at(0)
            .get("content")
            .get("parts")
            .at(0)
            .get("text")
            .as_str(),
        Some(prompt.as_str()),
        "same exact save prompt"
    );
    let resumed = backfill(&root, &file, &["--run"], &api.base);
    assert!(resumed.status.success());
    let resumed = String::from_utf8(resumed.stdout).unwrap();
    assert!(resumed.contains("Cards without vectors: 1;") && resumed.contains("Embedded 1/1;"));
    let complete = backfill(&root, &file, &["--run"], &api.base);
    assert!(complete.status.success());
    let complete = String::from_utf8(complete.stdout).unwrap();
    assert!(complete.contains("Cards without vectors: 0;"));
    for text in [&output, &resumed, &complete] {
        assert!(
            !text.contains("synthetic-test-key"),
            "no provider secret in logs"
        );
    }
    drop(writer);
    drop(store);
    let reopened = Sqlite::open(&file, true).unwrap();
    assert_eq!(
        reopened
            .one("SELECT COUNT(*) AS n FROM embeddings", &[])
            .unwrap()
            .unwrap()
            .int("n"),
        Some(3)
    );
}

#[test]
fn legacy_row_migration_triggers_reopen_and_recorded_search_floors() {
    let root = root("search-migration");
    let file = root.join("legacy.db");
    let db = Sqlite::open(&file, false).unwrap();
    db.exec_batch(SCHEMA).unwrap();
    db.execute(
        "INSERT INTO users VALUES(?,?,?,0,?)",
        &[
            "legacy".into(),
            "legacy".into(),
            "fixture0000".into(),
            "{}".into(),
        ],
    )
    .unwrap();
    db.execute("INSERT INTO cards(id,user_id,type,data,html,created_ns,bumped_ns,modified_ns) VALUES(?,?,?,?,?,0,0,0)", &["legacy-card".into(), "legacy".into(), "Note".into(), r#"{"title":"migrationzz","type":"Note","flags":0}"#.into(), "".into()]).unwrap();
    drop(db);
    {
        let store = Rc::new(Store::new(Rc::new(Sqlite::open(&file, false).unwrap())).unwrap());
        let search = Search::new(store.clone(), None, false).unwrap();
        assert_eq!(
            search.text("migrationzz", &Value::Undefined).unwrap().count,
            1
        );
        store
            .db()
            .execute(
                "INSERT INTO annotations VALUES(?,?,?,?,0,0)",
                &[
                    "legacy-tag".into(),
                    "legacy".into(),
                    "legacy-card".into(),
                    "annotationzz".into(),
                ],
            )
            .unwrap();
        assert_eq!(
            search
                .text("annotationzz", &Value::Undefined)
                .unwrap()
                .count,
            1
        );
        store
            .db()
            .execute(
                "INSERT INTO card_content VALUES(?,?)",
                &[
                    "legacy-card".into(),
                    json::to_string(&doc("currentreaderzz")).into(),
                ],
            )
            .unwrap();
        assert_eq!(
            search
                .text("currentreaderzz", &Value::Undefined)
                .unwrap()
                .count,
            1
        );
        store
            .db()
            .execute("DELETE FROM annotations WHERE id=?", &["legacy-tag".into()])
            .unwrap();
        assert_eq!(
            search
                .text("annotationzz", &Value::Undefined)
                .unwrap()
                .count,
            0
        );
    }
    {
        let store = Rc::new(Store::new(Rc::new(Sqlite::open(&file, false).unwrap())).unwrap());
        let search = Search::new(store.clone(), None, false).unwrap();
        assert_eq!(
            search
                .text("currentreaderzz", &Value::Undefined)
                .unwrap()
                .count,
            1
        );
        store
            .db()
            .execute("DELETE FROM cards WHERE id=?", &["legacy-card".into()])
            .unwrap();
        assert_eq!(
            search
                .text("currentreaderzz", &Value::Undefined)
                .unwrap()
                .count,
            0
        );
    }
    recorded_assertions(&root);
}

// Preserve the five original real-library overlap floors and exact replay fallback checks.
fn recorded_assertions(root: &Path) {
    let file = library(root);
    let server = start(root, &file, true, &[]);
    let mut client = Client::new(server.port);
    let store = Rc::new(Store::new(Rc::new(Sqlite::open(&file, false).unwrap())).unwrap());
    let comparison = mymind::tool_search_recordings::compare_recordings(&store, root).unwrap();
    assert_eq!(comparison.len(), 5);
    for (row, (query, floor)) in comparison.iter().zip([
        ("de", 1585),
        ("des", 933),
        ("desi", 789),
        ("desig", 786),
        ("design", 786),
    ]) {
        assert_eq!(row.get("q").as_str(), Some(query));
        assert!(
            row.get("overlap").as_f64().expect("numeric field") >= floor as f64,
            "recorded prefix {query} overlap floor {floor}"
        );
    }
    let index =
        json::parse(&std::fs::read_to_string(root.join("replay/index.json")).unwrap()).unwrap();
    let mut paths = vec![
        "/search?q=design&wildcard=true".to_owned(),
        "/search/suggestions".to_owned(),
        "/search/suggestions?q=design".to_owned(),
    ];
    let prefix = "GET https://access.mymind.com";
    let checks = regex::Regex::new(r"^/queries/[^/]+/check$").unwrap();
    for (key, _) in index.as_object().unwrap().iter() {
        if let Some(path) = key.strip_prefix(prefix)
            && checks.is_match(path)
        {
            paths.push(path.into());
        }
    }
    assert!(paths.len() > 3, "recorded query checks are present");
    for path in paths {
        let entry = index.get(&format!("{prefix}{path}"));
        let body = std::fs::read_to_string(
            root.join("replay/bodies")
                .join(entry.get("file").as_str().unwrap()),
        )
        .unwrap();
        let localized = body
            .replace("https://static.accelerator.net/", "/_static/")
            .replace("https://media.mymind.host/", "/_mediahost/")
            .replace("https://carbon-media.accelerator.net/", "/_carbon/")
            .replace("https://access.mymind.com/", "/");
        let answer = request(
            &mut client,
            "GET",
            &path,
            &Value::Undefined,
            "application/json",
            "application/json",
        );
        assert!(
            answer.body == localized.as_bytes(),
            "recorded search/check bytes identical"
        );
    }
    let unrecorded = get(&mut client, "/search?q=unrecordedzz&wildcard=false");
    assert_eq!(
        unrecorded.get("count").as_f64().expect("numeric field"),
        0.0
    );
    same(
        unrecorded.get("matches"),
        &arr![],
        "unrecorded SQL search empty",
    );
    same(
        &get(&mut client, "/search/suggestions?q=unrecordedzz"),
        &arr![],
        "unrecorded suggestions empty",
    );
    let search = Search::new(store, None, false).unwrap();
    let read = |path: &str| {
        json::parse(
            &std::fs::read_to_string(
                root.join("replay/bodies").join(
                    index
                        .get(&format!("{prefix}{path}"))
                        .get("file")
                        .as_str()
                        .unwrap(),
                ),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let types = Value::Array(
        search
            .suggestions("")
            .unwrap()
            .iter()
            .map(|item| item.to_value())
            .collect(),
    );
    same(
        &types,
        &read("/search/suggestions"),
        "recorded type counts exact",
    );
    let tags = search.suggestions("design").unwrap();
    for live in read("/search/suggestions?q=design").as_array().unwrap() {
        assert!(
            tags.iter().any(|tag| tag.query_text.to_lowercase()
                == live.get("queryText").as_str().unwrap().to_lowercase()
                && tag.count == live.get("count").as_f64().expect("numeric field")),
            "recorded tag count"
        );
    }
    let estimate = backfill(root, &file, &[], "http://127.0.0.1:1");
    assert!(estimate.status.success());
    let stdout = String::from_utf8(estimate.stdout).unwrap();
    let captures =
        regex::Regex::new(r"Cards without vectors: (\d+); skipped without text: (\d+)").unwrap();
    let counts = captures.captures(&stdout).unwrap();
    assert!(
        counts[1].parse::<usize>().unwrap() + counts[2].parse::<usize>().unwrap() > 0,
        "seed estimate includes cards or empty skips"
    );
}
