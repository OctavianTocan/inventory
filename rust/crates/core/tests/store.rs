//! `store`, `store_schema`, `search_index` and `search_document` against the JS store on both fixtures.

mod common;

use std::path::Path;
use std::rc::Rc;

use common::{Random, assert_same, calls, hosted_db, temp_copy, test_db};
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::{ChangesOptions, Cursor, IdKind, Store, ns_from_iso, iso_from_ns};
use mm_core::{Error, Result, Value, arr, obj};
use mm_sqlite::Sqlite;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/store.mjs");

fn open(file: &Path, drop_search: bool) -> Result<(Rc<Sqlite>, Store)> {
    let db = Rc::new(Sqlite::open(file, false)?);
    if drop_search {
        db.exec_batch("DROP TABLE IF EXISTS search_fts; DROP TABLE IF EXISTS search_documents")?;
    }
    let store = Store::new(db.clone())?;
    Ok((db, store))
}

fn values<T>(items: impl IntoIterator<Item = T>, convert: impl Fn(T) -> Value) -> Value {
    Value::Array(items.into_iter().map(convert).collect())
}

fn cards(items: Vec<mm_core::store::CardRow>) -> Value {
    values(items, |card| card.to_value())
}

fn ids(db: &dyn Sql, sql: &str, params: &[SqlValue]) -> Result<Vec<String>> {
    Ok(db.query(sql, params)?.iter().map(|row| row.text("id").unwrap_or("").to_owned()).collect())
}

fn cursor(card: Option<&Value>, key: &str) -> Result<Option<Cursor>> {
    let Some(card) = card else { return Ok(None) };
    let Value::BigInt(ns) = card.get(key) else { return Err(Error::internal("not a bigint")) };
    Ok(Some(Cursor { ns: *ns, id: card.get("id").as_str().unwrap_or("").to_owned() }))
}

fn dump(file: &Path) -> Result<Value> {
    let (db, store) = open(file, false)?;
    let users = ids(&*db, "SELECT id FROM users ORDER BY rowid", &[])?;
    let missing_quota: Value = match store.quota(Some("nope")) {
        Err(Error::Invalid(_)) => Value::from(obj! { "threw" => "TypeError" }),
        other => return Err(Error::internal(format!("quota: {other:?}"))),
    };
    let mut out = obj! {
        "defaultUserId" => store.default_user_id()?,
        "missingUser" => store.user(Some("nope"))?.map_or(Value::Null, |user| user.to_value()),
        "missingQuota" => missing_quota,
    };
    let mut entries = Vec::new();
    for user_id in &users {
        let user = Some(user_id.as_str());
        let all = store.grid_cards(user, 0, None)?;
        let trash = store.trash_cards(user)?;
        let first = store.grid_cards_after(user, None, 40)?;
        let first_value = first.last().map(|card| card.to_value());
        let next_after = cursor(first_value.as_ref(), "bumpedNs")?;
        let trash_first = store.trash_cards_after(user, None, 7)?;
        let trash_last = trash_first.last().map(|card| card.to_value());
        let trash_after = cursor(trash_last.as_ref(), "deletedNs")?;
        let last = all.last().map(|card| Cursor { ns: card.bumped_ns, id: card.id.clone() }).unwrap_or(Cursor { ns: 0, id: String::new() });
        let changes = store.changes_since(0, &ChangesOptions { user_id: user, limit: 100_000, coalesce: false })?;
        let middle = changes.get(changes.len() / 2).map_or(0, |change| change.ts);
        let spaces = store.spaces(user)?;
        let all_ids = ids(&*db, "SELECT id FROM cards WHERE user_id = ? ORDER BY id", &[user_id.as_str().into()])?;
        let mut by_ids: Vec<String> = all_ids.iter().take(5).cloned().collect();
        by_ids.push("missing".into());
        by_ids.extend(all_ids.iter().skip(all_ids.len().saturating_sub(3)).cloned());
        by_ids.extend(all_ids.first().cloned());
        let coalesced = store.changes_since(0, &ChangesOptions { user_id: user, limit: 100_000, coalesce: true })?;
        let change_list = |items: &[mm_core::store::Change]| values(items.iter(), |change| change.to_value());
        let recent = |take: i64| -> Result<Value> { Ok(values(store.tags_recent(take, user)?, |name| obj! { "name" => name }.into())) };
        let entry = obj! {
            "id" => user_id.as_str(),
            "user" => store.user(user)?.map_or(Value::Null, |item| item.to_value()),
            "quota" => store.quota(user)?.to_value(),
            "gridCount" => store.grid_count(user)?,
            "checkpoint" => store.checkpoint(user)?,
            "gridCheckpoint" => store.grid_checkpoint(user)?,
            "grid" => cards(all.clone()),
            "window" => cards(store.grid_cards(user, 10, Some(20))?),
            "gridFirst" => cards(first),
            "gridNext" => cards(store.grid_cards_after(user, next_after.as_ref(), 40)?),
            "gridLast" => cards(store.grid_cards_after(user, Some(&last), 5)?),
            "trash" => cards(trash),
            "trashFirst" => cards(trash_first),
            "trashNext" => cards(store.trash_cards_after(user, trash_after.as_ref(), 7)?),
            "spaces" => values(spaces.iter(), |space| space.to_value()),
            "spaceCards" => { let mut lists = Vec::new(); for space in &spaces { lists.push(cards(store.space_cards(&space.id)?)); } Value::Array(lists) },
            "space" => { let mut lists = Vec::new(); for space in &spaces { lists.push(store.space(&space.id)?.map_or(Value::Null, |item| item.to_value())); } Value::Array(lists) },
            "tagsManual" => values(store.tags_manual(user)?, |tag| tag.to_value()),
            "tagsRecent" => Value::Array(vec![recent(10)?, recent(-2)?, recent(0)?, recent(3)?, recent(10)?]),
            "changes" => Value::Array(vec![
                changes.len().into(),
                change_list(&changes[..changes.len().min(30)]),
                change_list(&store.changes_since(middle, &ChangesOptions { user_id: user, limit: 25, coalesce: false })?),
                change_list(&coalesced[coalesced.len().saturating_sub(30)..]),
                change_list(&store.changes_since(middle, &ChangesOptions { user_id: user, limit: 1000, coalesce: true })?),
            ]),
            "cardsByIds" => cards(store.cards_by_ids(&by_ids, user)?),
            "cards" => { let mut list = Vec::new(); for id in &all_ids { list.push(store.card(id, user)?.map_or(Value::Null, |card| card.to_value())); } Value::Array(list) },
            "tags" => { let mut list = Vec::new(); for id in &all_ids { list.push(values(store.card_tags(id)?, |tag| tag.to_value())); } Value::Array(list) },
            "missingCard" => store.card("missing", user)?.map_or(Value::Null, |card| card.to_value()),
        };
        entries.push(Value::Object(entry));
    }
    out.insert("users", Value::Array(entries));
    Ok(Value::Object(out))
}

fn check_dump(fixture: &Path) {
    let js = calls(DRIVER, &[("dump", vec![Value::from(temp_copy(fixture).to_str().unwrap())])]).remove(0);
    assert_same(&dump(&temp_copy(fixture)), &js, &format!("dump({})", fixture.display()));
}

#[test]
fn reads_match_node_on_owner_fixture() {
    check_dump(&test_db());
}

#[test]
fn reads_match_node_on_hosted_fixture() {
    check_dump(&hosted_db());
}

fn new_ids(file: &Path) -> Result<Value> {
    let (db, store) = open(file, false)?;
    let users = ids(&*db, "SELECT id FROM users ORDER BY rowid", &[])?;
    let unknown_kind = match IdKind::parse("bogus") {
        Err(Error::Invalid(_)) => Value::from(obj! { "threw" => "RangeError" }),
        other => return Err(Error::internal(format!("{other:?}"))),
    };
    let unknown_user = match store.new_id(IdKind::Card, Some("nope")) {
        Err(Error::Invalid(_)) => Value::from(obj! { "threw" => "TypeError" }),
        other => return Err(Error::internal(format!("{other:?}"))),
    };
    let mut list = Vec::new();
    for user in &users {
        for kind in ["card", "tag", "space", "query", "note"] {
            let parsed = IdKind::parse(kind)?;
            let made = [store.new_id(parsed, Some(user))?, store.new_id(parsed, Some(user))?, store.new_id(parsed, Some(user))?];
            list.push(arr![user.as_str(), kind, values(made, Value::from)]);
        }
    }
    Ok(obj! {
        "unknownKind" => unknown_kind,
        "unknownUser" => unknown_user,
        "ids" => Value::Array(list),
        "defaultCard" => store.new_id(IdKind::Card, None)?,
    }
    .into())
}

#[test]
fn new_id_matches_node() {
    for fixture in [test_db(), hosted_db()] {
        let js = calls(DRIVER, &[("newIds", vec![Value::from(temp_copy(&fixture).to_str().unwrap())])]).remove(0);
        assert_same(&new_ids(&temp_copy(&fixture)), &js, "newIds");
    }
}

fn search_state(file: &Path, drop_search: bool) -> Result<Value> {
    let (db, _store) = open(file, drop_search)?;
    let master = values(
        db.query("SELECT type, name, tbl_name, sql FROM sqlite_master WHERE name LIKE 'search%' ORDER BY name", &[])?,
        |row| Value::Object(row.to_object()),
    );
    let docs = values(db.query("SELECT id, fields FROM search_documents ORDER BY id", &[])?, |row| Value::Object(row.to_object()));
    let count = |sql: &str| -> Result<Value> { Ok(db.one(sql, &[])?.map_or(Value::Null, |row| row.get("n").to_value())) };
    let fts = values(db.query("SELECT rowid FROM search_fts WHERE search_fts MATCH 'the' ORDER BY rowid", &[])?, |row| Value::Object(row.to_object()));
    Ok(obj! { "master" => master, "docs" => docs, "dirty" => count("SELECT COUNT(*) AS n FROM search_dirty")?, "fts" => fts, "total" => count("SELECT COUNT(*) AS n FROM search_fts")? }.into())
}

#[test]
fn search_index_matches_node() {
    for fixture in [test_db(), hosted_db()] {
        for drop_search in [false, true] {
            let js = calls(DRIVER, &[("searchState", vec![Value::from(temp_copy(&fixture).to_str().unwrap()), Value::from(drop_search)])]).remove(0);
            assert_same(&search_state(&temp_copy(&fixture), drop_search), &js, &format!("searchState({}, drop={drop_search})", fixture.display()));
        }
    }
}

fn transaction_cases(file: &Path) -> Result<Value> {
    let (db, store) = open(file, false)?;
    let id = ids(&*db, "SELECT id FROM cards ORDER BY id LIMIT 1", &[])?.remove(0);
    let snapshot = || -> Result<Value> {
        let data = db.one("SELECT data FROM cards WHERE id = ?", &[id.as_str().into()])?.and_then(|row| row.text("data").map(str::to_owned));
        let doc = db.one("SELECT fields FROM search_documents WHERE id = ?", &[id.as_str().into()])?.and_then(|row| row.text("fields").map(str::to_owned));
        let dirty = db.one("SELECT COUNT(*) AS n FROM search_dirty", &[])?.map_or(Value::Null, |row| row.get("n").to_value());
        Ok(obj! { "data" => data.map_or(Value::Null, Value::from), "doc" => doc.map_or(Value::Null, Value::from), "dirty" => dirty }.into())
    };
    let edit = |title: &str| db.execute("UPDATE cards SET data = json_set(data, '$.title', ?) WHERE id = ?", &[title.into(), id.as_str().into()]);
    let before = snapshot()?;
    let committed = store.transaction(|| {
        edit("Zebra committed")?;
        Ok("done")
    })?;
    let after_commit = snapshot()?;
    let thrown = store.transaction(|| -> Result<()> {
        edit("Rolled back")?;
        Err(Error::invalid("boom"))
    });
    let thrown = match thrown {
        Err(Error::Invalid(_)) => Value::from(obj! { "threw" => "RangeError" }),
        other => return Err(Error::internal(format!("{other:?}"))),
    };
    let after_rollback = snapshot()?;
    let nested = store.transaction(|| {
        edit("Outer")?;
        let inner = store.transaction(|| -> Result<()> {
            edit("Inner")?;
            Err(Error::invalid("inner"))
        });
        Ok(match inner {
            Err(Error::Invalid(_)) => Value::from(obj! { "threw" => "TypeError" }),
            other => return Err(Error::internal(format!("{other:?}"))),
        })
    })?;
    Ok(obj! {
        "before" => before,
        "committed" => committed,
        "afterCommit" => after_commit,
        "thrown" => thrown,
        "afterRollback" => after_rollback,
        "nested" => nested,
        "afterNested" => snapshot()?,
    }
    .into())
}

#[test]
fn transaction_matches_node() {
    for fixture in [test_db(), hosted_db()] {
        let js = calls(DRIVER, &[("transactionCases", vec![Value::from(temp_copy(&fixture).to_str().unwrap())])]).remove(0);
        assert_same(&transaction_cases(&temp_copy(&fixture)), &js, "transactionCases");
    }
}

#[test]
fn nanosecond_helpers_match_node() {
    let mut random = Random(77);
    let mut isos: Vec<String> = [
        "2026-10-15T09:35:08.050534Z", "2026-10-15T09:35:08Z", "1970-01-01T00:00:00Z", "1969-12-31T23:59:59.5Z", "0001-01-01T00:00:00Z", "9999-12-31T23:59:59.999999999Z",
        "2026-13-15T09:35:08Z", "2026-02-30T09:35:08Z", "2026-10-15T24:00:00Z", "2026-10-15 09:35:08Z", "2026-10-15T09:35:08.Z", "2026-10-15T09:35:08.1234567890Z", "2026-10-15T09:35:08+00:00",
        "2026-10-15T09:35:08z", "", "x", "2026-10-15T09:35:08.050534Z\n", "275760-09-13T00:00:00Z",
    ]
    .iter()
    .map(|text| (*text).to_owned())
    .collect();
    for _ in 0..1500 {
        let fraction = match random.below(3) {
            0 => String::new(),
            _ => {
                let length = random.below(9) as usize + 1;
                format!(".{}", random.text(&["0", "1", "5", "9", "7"], length))
            }
        };
        isos.push(format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{fraction}Z", random.below(10000), random.below(14), random.below(32), random.below(25), random.below(61), random.below(61)));
    }
    let js = calls("lib/store.mjs", &isos.iter().map(|iso| ("nsFromIso", vec![Value::from(iso.as_str())])).collect::<Vec<_>>());
    let mut parsed = Vec::new();
    for (iso, answer) in isos.iter().zip(&js) {
        let result = ns_from_iso(iso);
        assert_same(&result.clone().map(Value::BigInt), answer, &format!("nsFromIso({iso:?})"));
        parsed.extend(result.ok());
    }
    let mut instants = parsed;
    instants.extend([0, -1, -999_999_999, -1_000_000_000, -1_500_000_000, 1, 1_999_999_999, 8_640_000_000_000_000_000_000_000, 8_640_000_000_000_001_000_000_000, -8_640_000_000_000_000_000_000_000, -8_640_000_000_000_001_000_000_000]);
    for _ in 0..1000 {
        instants.push(i128::from(random.next() as i64) * 1000 + i128::from(random.below(1000)));
    }
    let mut cases = Vec::new();
    for ns in &instants {
        for digits in [0usize, 1, 3, 6, 9] {
            cases.push((*ns, digits));
        }
    }
    let js = calls("lib/store.mjs", &cases.iter().map(|(ns, digits)| ("isoFromNs", vec![Value::BigInt(*ns), Value::from(*digits)])).collect::<Vec<_>>());
    for ((ns, digits), answer) in cases.iter().zip(&js) {
        assert_same(&iso_from_ns(*ns, *digits).map(Value::from), answer, &format!("isoFromNs({ns}, {digits})"));
    }
}

#[test]
fn constants_are_the_js_values() {
    assert_eq!(mm_core::store::INITIAL_DRAW_COUNT, 250);
    assert_eq!(mm_core::store::GRID_HIDDEN_FLAGS, 8 | 131_072 | 262_144);
}
