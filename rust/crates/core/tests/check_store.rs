//! Independent recording assertions formerly run by `tools/check-store.mjs`.

#[path = "support/recordings.rs"]
mod recordings;

use std::collections::HashMap;
use std::rc::Rc;

use mm_core::render_card::render_card;
use mm_core::sql::SqlExt;
use mm_core::store::{CardRow, ChangesOptions, INITIAL_DRAW_COUNT, IdKind, Store};
use mm_core::{Value, arr, obj};
use mm_js::{date, json};
use mm_sqlite::Sqlite;
use recordings::{Recording, Recordings};
use regex::Regex;

// Seed an isolated store from committed recordings without touching mutable or golden fixture databases.
fn context() -> (Rc<Sqlite>, Store, Recordings) {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!(
            "recorded-store-{}-{}",
            mm_core::runtime::random_uuid(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
        .join("recorded.db");
    let rec = recordings::read_recordings();
    let (db, store) = recordings::seed_recorded_store(&path, &rec);
    (db, store, rec)
}

// Compare values without printing private card, tag or account contents on a failure.
fn equal(actual: &Value, expected: &Value, label: &str) {
    assert!(
        actual == expected,
        "{label} differs from independent recording"
    );
}

// Apply the original JSON round trip, including Date conversion and undefined omission.
fn plain(value: &Value) -> Value {
    json::parse(&json::to_string(value)).expect("plain JSON value")
}

// Extract the ordered persisted card identities.
fn ids(cards: &[CardRow]) -> Vec<String> {
    cards.iter().map(|card| card.id.clone()).collect()
}

// Extract the ordered identities from recorded root elements.
fn recorded_ids(cards: &[Recording]) -> Vec<String> {
    cards.iter().map(Recording::id).collect()
}

// Ignore only the CDN expiration and signature bytes the original checker blanked.
fn unsign(html: &str) -> String {
    Regex::new(r#"Expires=[0-9]+(&(?:amp;)?)Signature=[^&"\s]+"#)
        .expect("signature pattern")
        .replace_all(html, "Expires=${1}Signature=")
        .into_owned()
}

// Compare original render bytes and report only position, never recorded text.
fn render_equal(cards: &[CardRow], label: &str) {
    for (index, card) in cards.iter().enumerate() {
        let data = json::parse(&card.data).expect("persisted card JSON");
        let extras = json::parse(&card.extras).expect("persisted card extras");
        let html = render_card(&data, &extras).expect("render persisted card");
        let offset = card
            .html
            .bytes()
            .zip(html.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(card.html.len().min(html.len()));
        assert!(
            html == card.html,
            "{label} recording #{} differs at byte {offset}",
            index + 1
        );
    }
}

#[test]
fn grid_count_order_and_both_pages_match_recordings() {
    let (_, store, rec) = context();
    let grid = store.grid_cards(None, 0, None).expect("grid cards");
    let expected = recorded_ids(&rec.stream);
    assert_eq!(
        store.grid_count(None).expect("grid count"),
        rec.stream.len() as i64,
        "grid card count"
    );
    assert!(ids(&grid) == expected, "full grid order differs");
    let first = store
        .grid_cards(None, 0, Some(INITIAL_DRAW_COUNT as i64))
        .expect("first page");
    let next = store
        .grid_cards(None, INITIAL_DRAW_COUNT as i64, Some(50))
        .expect("second page");
    assert!(
        ids(&first) == expected[..expected.len().min(INITIAL_DRAW_COUNT)],
        "first page differs"
    );
    assert!(
        ids(&next)
            == expected
                .iter()
                .skip(INITIAL_DRAW_COUNT)
                .take(50)
                .cloned()
                .collect::<Vec<_>>(),
        "second page differs"
    );
}

#[test]
fn grid_bytes_and_capture_presence_match_recordings() {
    let (_, store, rec) = context();
    let grid = store.grid_cards(None, 0, None).expect("grid cards");
    let by_id: HashMap<_, _> = grid.iter().map(|row| (row.id.as_str(), row)).collect();
    for (index, recorded) in rec.stream.iter().enumerate() {
        let id = recorded.id();
        let row = by_id
            .get(id.as_str())
            .expect("recorded grid card persisted");
        assert!(
            unsign(&row.html) == unsign(&recorded.html) && row.data == recorded.json,
            "grid html/json recording #{} differs",
            index + 1
        );
    }
    for (index, recorded) in rec.capture.iter().enumerate() {
        assert!(
            store
                .card(&recorded.id(), None)
                .expect("capture card lookup")
                .is_some(),
            "capture recording #{} missing",
            index + 1
        );
    }
}

#[test]
fn trash_order_bytes_deletion_times_and_grid_disjointness_match_recordings() {
    let (_, store, rec) = context();
    let trash = store.trash_cards(None).expect("trash cards");
    let expected = recorded_ids(&rec.trash);
    assert!(ids(&trash) == expected, "trash order differs");
    for (index, (row, recorded)) in trash.iter().zip(&rec.trash).enumerate() {
        assert!(
            row.html == recorded.html && row.data == recorded.json,
            "trash recording #{} bytes differ",
            index + 1
        );
        let data = json::parse(&recorded.json).expect("trash JSON");
        let deleted = date::parse(data.get("deleted").as_str().expect("deleted ISO time")) as i128
            * 1_000_000;
        assert_eq!(
            row.deleted_ns,
            Some(deleted),
            "trash deletion time #{}",
            index + 1
        );
    }
    let grid_ids = recorded_ids(&rec.stream);
    assert!(
        expected.iter().all(|id| !grid_ids.contains(id)),
        "recorded trash card also in grid"
    );
}

#[test]
fn spaces_lists_details_and_ranked_members_match_recordings() {
    let (_, store, rec) = context();
    let spaces = Value::Array(
        store
            .spaces(None)
            .expect("spaces")
            .iter()
            .map(|space| space.to_value())
            .collect(),
    );
    equal(&plain(&spaces), &plain(&rec.spaces), "spaces list");
    for (index, (id, recorded)) in rec.space_details.iter().enumerate() {
        let mut actual = plain(
            &store
                .space(id)
                .expect("space detail")
                .map_or(Value::Null, |space| space.to_value()),
        );
        let mut recorded = recorded.clone();
        if let Some(query) = actual.get_mut("query") {
            query.remove("checked");
        }
        if let Some(query) = recorded.get_mut("query") {
            query.remove("checked");
        }
        equal(&actual, &recorded, &format!("space detail #{}", index + 1));
    }
    for (index, space) in rec
        .spaces
        .as_array()
        .expect("space response array")
        .iter()
        .enumerate()
    {
        let actual = ids(&store
            .space_cards(space.get("id").as_str().expect("space id"))
            .expect("space cards"));
        let expected = space
            .get("objects")
            .as_array()
            .expect("space members")
            .iter()
            .map(|member| member.get("id").as_str().expect("member id").to_owned())
            .collect::<Vec<_>>();
        assert!(
            actual == expected,
            "space #{} member order differs",
            index + 1
        );
    }
}

#[test]
fn all_card_tags_and_manual_and_recent_tag_responses_match_recordings() {
    let (_, store, rec) = context();
    let mut all = store.grid_cards(None, 0, None).expect("grid cards");
    all.extend(store.trash_cards(None).expect("trash cards"));
    let tags = |id: &str| {
        Value::Array(
            store
                .card_tags(id)
                .expect("card tags")
                .iter()
                .map(|tag| tag.to_value())
                .collect(),
        )
    };
    for (index, card) in all.iter().enumerate() {
        let data = json::parse(&card.data).expect("card JSON");
        let expected = if data.get("tags").is_nullish() {
            arr![]
        } else {
            data.get("tags").clone()
        };
        equal(
            &tags(&card.id),
            &expected,
            &format!("persisted card tags #{}", index + 1),
        );
    }
    for (index, recorded) in rec.stream.iter().chain(&rec.trash).enumerate() {
        let data = json::parse(&recorded.json).expect("recorded card JSON");
        let expected = if data.get("tags").is_nullish() {
            arr![]
        } else {
            data.get("tags").clone()
        };
        equal(
            &tags(&recorded.id()),
            &expected,
            &format!("recorded card tags #{}", index + 1),
        );
    }
    let manual = Value::Array(
        store
            .tags_manual(None)
            .expect("manual tags")
            .iter()
            .map(|tag| tag.to_value())
            .collect(),
    );
    equal(&manual, &rec.manual, "manual tags including Date values");
    let recent = Value::Array(
        store
            .tags_recent(10, None)
            .expect("recent tags")
            .iter()
            .map(|name| obj! { "name" => name.as_str() }.into())
            .collect(),
    );
    equal(&recent, &rec.recent, "recent tags");
}

#[test]
fn checkpoint_newer_deletions_and_complete_change_log_match_recordings() {
    let (_, store, rec) = context();
    let expected = rec
        .grid_headers
        .get("x-checkpoint")
        .as_str()
        .expect("recorded checkpoint");
    assert!(
        store.grid_checkpoint(None).expect("grid checkpoint") == expected,
        "grid checkpoint differs"
    );
    let newer = store
        .changes_since(
            expected.parse().expect("checkpoint digits"),
            &ChangesOptions::default(),
        )
        .expect("newer changes");
    assert!(
        newer.iter().all(|change| change.kind == 2),
        "non-deletion after grid checkpoint"
    );
    let count = store.grid_cards(None, 0, None).expect("grid cards").len()
        + store.trash_cards(None).expect("trash cards").len();
    let changes = store
        .changes_since(
            0,
            &ChangesOptions {
                limit: count as i64 + 1,
                ..Default::default()
            },
        )
        .expect("all changes");
    assert_eq!(changes.len(), count, "one change per card");
    assert!(
        changes.windows(2).all(|pair| pair[0].ts < pair[1].ts),
        "strictly increasing change times"
    );
    for (index, change) in changes.iter().enumerate() {
        let card = change.card.as_ref().expect("change's persisted card");
        assert_eq!(
            change.ts,
            card.modified_ns,
            "change #{} modified time",
            index + 1
        );
        assert_eq!(
            change.kind,
            if card.deleted_ns.is_none() { 1 } else { 2 },
            "change #{} type",
            index + 1
        );
    }
}

#[test]
fn quota_user_and_settings_match_recorded_headers_and_shell() {
    let (_, store, rec) = context();
    let quota = json::parse(
        rec.grid_headers
            .get("x-object-quota")
            .as_str()
            .expect("quota header"),
    )
    .expect("quota JSON");
    equal(
        &store.quota(None).expect("quota").to_value(),
        &quota,
        "object quota",
    );
    let user = store.user(None).expect("user").expect("seeded user");
    let c = &rec.config;
    equal(
        &arr![user.id.as_str(), user.account_uid.as_str(), user.flags],
        &arr![
            c.get("user").get("uid").clone(),
            c.get("account").get("uid").clone(),
            c.get("user").get("flags").clone()
        ],
        "user identity and flags",
    );
    equal(&user.settings, &obj! {
        "colorScheme" => c.get("colorScheme").clone(), "timeZoneId" => c.get("timeZoneId").clone(),
        "locale" => c.get("locale").clone(), "currency" => c.get("currency").clone(),
        "aiLevel" => c.get("aiLevel").clone(), "onboardFlags" => c.get("onboardFlags").clone(),
        "objectLimit" => c.get("quotas").get("objects").get("limit").clone(),
    }.into(), "user settings");
}

#[test]
fn new_ids_have_recorded_user_prefix_kind_alphabet_and_strict_order() {
    let (db, store, _) = context();
    let user = store.user(None).expect("user").expect("seeded user");
    for (kind, code) in [
        (IdKind::Card, "0O"),
        (IdKind::Tag, "0y"),
        (IdKind::Space, "gl"),
        (IdKind::Query, "iV"),
        (IdKind::Note, "9Z"),
    ] {
        let first = store.new_id(kind, None).expect("first id");
        let second = store.new_id(kind, None).expect("second id");
        let prefix = format!("{}{code}", user.id_prefix);
        let suffix = first.strip_prefix(&prefix).expect("user and kind prefix");
        assert!(
            suffix.len() == 9 && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric()),
            "nine base62 id digits"
        );
        assert!(second > first, "new ids strictly increase");
    }
    let existing = db
        .one(
            "SELECT MAX(id) AS id FROM cards WHERE substr(id, 12, 2) = ?",
            &["0O".into()],
        )
        .expect("existing card ids")
        .expect("maximum row");
    assert!(
        store
            .new_id(IdKind::Card, None)
            .expect("next card id")
            .as_str()
            > existing.text("id").expect("maximum existing id"),
        "new card id sorts after all recorded cards"
    );
}

#[test]
fn persisted_grid_and_trash_render_back_to_exact_recorded_bytes() {
    let (_, store, _) = context();
    render_equal(
        &store.grid_cards(None, 0, None).expect("grid cards"),
        "grid",
    );
    render_equal(&store.trash_cards(None).expect("trash cards"), "trash");
}

#[test]
fn signature_normalization_preserves_every_other_html_byte() {
    assert_eq!(
        unsign("/x?Expires=123&Signature=old&Key=a"),
        "/x?Expires=&Signature=&Key=a"
    );
    assert_eq!(
        unsign("/x?Expires=123&amp;Signature=old&amp;Key=a"),
        "/x?Expires=&amp;Signature=&amp;Key=a"
    );
    assert_ne!(
        unsign("/x?Expires=123&Signature=old&Key=a"),
        unsign("/y?Expires=456&Signature=new&Key=a")
    );
    assert_eq!(unsign("Expires=x&Signature=old"), "Expires=x&Signature=old");
}
