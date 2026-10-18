//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use common::{assert_same, calls, hosted_db, temp_copy, test_db};
use mm_core::sql::Sql;
use mm_core::store::Store;
use mm_core::store_query::{QueryCheck, QueryWriter, SpaceMatch, SpaceSearch, check_query};
use mm_core::{Error, Result, Value, arr, obj};
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:store-query";
const PREPS: [&str; 3] = ["", "UPDATE space_cards SET excluded = 1 WHERE rowid % 7 = 0", "UPDATE space_cards SET query_id = NULL WHERE rowid % 5 = 0"];
const QUERIES: [&str; 3] = ["00000002UnEiVcPRujmLEk", "00000002UnEiVcgCiecNqM", "nope"];

struct Search {
    ids: Vec<String>,
    seed: u64,
    seen: RefCell<Vec<Value>>,
}

fn sum(id: &str) -> u64 {
    id.encode_utf16().map(u64::from).sum()
}

impl SpaceSearch for Search {
    fn filters(&self, filters: &Value) -> Result<Vec<SpaceMatch>> {
        self.seen.borrow_mut().push(if matches!(filters, Value::Undefined) { Value::Null } else { filters.clone() });
        Ok(self
            .ids
            .iter()
            .filter(|id| (sum(id) + self.seed) % 4 < 2)
            .map(|id| SpaceMatch { id: id.clone(), score: ((sum(id) + self.seed) % 100) as f64 / 100.0 })
            .collect())
    }
}

struct Writer {
    log: RefCell<Vec<Value>>,
}

impl QueryWriter for Writer {
    type Card = String;

    fn now(&self) -> Result<i128> {
        Ok(1_787_000_000_123_456_789)
    }

    fn card(&self, id: &str) -> Result<String> {
        self.log.borrow_mut().push(arr!["card", id]);
        Ok(id.to_owned())
    }

    fn save(&self, card: String) -> Result<()> {
        self.log.borrow_mut().push(arr!["save", card]);
        Ok(())
    }

    fn save_space(&self, id: &str) -> Result<()> {
        self.log.borrow_mut().push(arr!["space", id]);
        Ok(())
    }
}

fn rows(db: &dyn Sql, sql: &str) -> Result<Value> {
    Ok(Value::Array(db.query(sql, &[])?.iter().map(|row| Value::Object(row.to_object())).collect()))
}

fn run(file: &Path, query_id: &str, persist: bool, seed: u64, prep: usize) -> Result<Value> {
    let db = Rc::new(Sqlite::open(file, false)?);
    let store = Store::new(db.clone())?;
    if !PREPS[prep].is_empty() {
        db.exec_batch(PREPS[prep])?;
    }
    db.exec_batch("INSERT OR IGNORE INTO api_metadata (path, status, headers, attributes) SELECT '/spaces/' || id, 200, '{}', '{}' FROM spaces")?;
    let ids = db
        .query("SELECT id FROM cards WHERE user_id = ? ORDER BY id", &[store.default_user_id()?.into()])?
        .iter()
        .map(|row| row.text("id").unwrap_or("").to_owned())
        .collect();
    let search = Search { ids, seed, seen: RefCell::new(Vec::new()) };
    let writer = Writer { log: RefCell::new(Vec::new()) };
    let result = match check_query(&store, &search, &writer, query_id, persist) {
        Ok(found) => {
            assert!(found.elapsed >= 0.0 && found.elapsed.fract() == 0.0, "elapsed {}", found.elapsed);
            QueryCheck { elapsed: 0.0, ..found }.to_value()
        }
        Err(Error::Status { status, message, .. }) => obj! { "threw" => message, "status" => status }.into(),
        Err(other) => return Err(other),
    };
    Ok(obj! {
        "result" => result,
        "log" => writer.log.into_inner(),
        "seen" => search.seen.into_inner(),
        "members" => rows(&*db, "SELECT space_id, card_id, rank, score, query_id, excluded FROM space_cards ORDER BY space_id, card_id")?,
        "queries" => rows(&*db, "SELECT id, query FROM spaces WHERE query IS NOT NULL ORDER BY id")?,
        "metadata" => rows(&*db, "SELECT path FROM api_metadata ORDER BY path")?,
    }
    .into())
}

#[test]
fn check_query_matches_node() {
    let _goldens = common::test_guard();
    let mut cases = 0;
    for fixture in [test_db(), hosted_db()] {
        for query_id in QUERIES {
            for persist in [false, true] {
                for seed in [0, 2] {
                    for prep in 0..PREPS.len() {
                        let js = calls(DRIVER, &[("check", vec![Value::from(temp_copy(&fixture).to_str().unwrap()), query_id.into(), persist.into(), seed.into(), prep.into()])]).remove(0);
                        assert_same(&run(&temp_copy(&fixture), query_id, persist, seed, prep), &js, &format!("check({query_id}, persist={persist}, seed={seed}, prep={prep})"));
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 72);
}
