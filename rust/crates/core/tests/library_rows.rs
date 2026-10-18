//! Compare Rust with frozen pre-switch expectations and immutable synthetic fixtures.

mod common;

use std::rc::Rc;

use common::{Random, assert_same, calls, hosted_db, temp_copy, test_db};
use mm_core::library_rows::{encode_value, insert_rows, table_digest};
use mm_core::sql::{Sql, SqlValue};
use mm_core::{Error, Result, Value, arr, obj};
use mm_sqlite::Sqlite;

// Logical identity for a frozen pre-switch comparison.
const DRIVER: &str = "driver:core:library-rows";

fn columns_of(db: &dyn Sql, table: &str) -> Vec<String> {
    db.query(&format!("PRAGMA table_info({table})"), &[]).unwrap().iter().map(|row| row.text("name").unwrap().to_owned()).collect()
}

fn names(columns: &[String]) -> Value {
    Value::Array(columns.iter().map(|name| Value::from(name.as_str())).collect())
}

#[test]
fn digests_match_node_on_every_fixture_table() {
    let _goldens = common::test_guard();
    let mut cases = 0;
    for fixture in [test_db(), hosted_db()] {
        let copy = temp_copy(&fixture);
        let db = Sqlite::open(&copy, true).unwrap();
        let tables: Vec<String> = db
            .query("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%' AND name NOT LIKE 'sqlite_%' ORDER BY name", &[])
            .unwrap()
            .iter()
            .map(|row| row.text("name").unwrap().to_owned())
            .collect();
        for table in tables {
            let plain = columns_of(&db, &table);
            let mut with_rowid = vec!["rowid".to_owned()];
            with_rowid.extend(plain.clone());
            for columns in [plain, with_rowid] {
                let js = calls(DRIVER, &[("digest", vec![copy.to_str().unwrap().into(), table.as_str().into(), names(&columns)])]).remove(0);
                assert_same(&table_digest(&db, &table, &columns).map(|digest| digest.to_value()), &js, &format!("tableDigest({table}, {columns:?})"));
                cases += 1;
            }
        }
    }
    assert!(cases > 40);
}

#[test]
fn digest_names_are_validated() {
    let _goldens = common::test_guard();
    let db = Sqlite::open(":memory:", false).unwrap();
    db.exec_batch("CREATE TABLE t (a, b)").unwrap();
    let cases: Vec<(&str, Vec<&str>)> = vec![("t", vec!["a x"]), ("t x", vec!["a"]), ("", vec!["a"]), ("t", vec![""]), ("t", vec!["1a"]), ("t", vec!["a\n"]), ("t", vec!["a\"b"]), ("t", vec!["é"]), ("t", vec!["a", "b"]), ("t", vec![])];
    let path = temp_copy(&test_db());
    for (table, columns) in cases {
        let columns: Vec<String> = columns.into_iter().map(str::to_owned).collect();
        let js = calls(DRIVER, &[("digest", vec![path.to_str().unwrap().into(), table.into(), names(&columns)])]).remove(0);
        let rust = table_digest(&Sqlite::open(&path, true).unwrap(), table, &columns).map(|digest| digest.to_value());
        if matches!(rust, Err(Error::Status { .. })) {
            assert_same(&rust, &js, &format!("tableDigest({table:?}, {columns:?})"));
        } else {
            common::assert_throws(&rust, &js, &format!("tableDigest({table:?}, {columns:?})"));
        }
    }
}

#[test]
fn encode_value_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(3);
    let mut cases: Vec<SqlValue> = vec![
        SqlValue::Null, SqlValue::Integer(0), SqlValue::Integer(-1), SqlValue::Integer(i64::MAX), SqlValue::Integer(i64::MIN), SqlValue::Integer(9_007_199_254_740_993),
        SqlValue::Real(0.0), SqlValue::Real(-0.0), SqlValue::Real(1.5), SqlValue::Real(1e300), SqlValue::Real(f64::INFINITY), SqlValue::Real(f64::NEG_INFINITY), SqlValue::Real(f64::NAN),
        SqlValue::Text(String::new()), SqlValue::Text("é😀\n".into()), SqlValue::Blob(vec![]), SqlValue::Blob(vec![0, 255, 1]),
    ];
    for _ in 0..300 {
        cases.push(SqlValue::Integer(random.next() as i64));
        cases.push(SqlValue::Blob((0..random.below(40)).map(|_| random.below(256) as u8).collect()));
    }
    for value in cases.iter().filter(|value| matches!(value, SqlValue::Real(number) if !number.is_finite())) {
        assert!(encode_value(value).is_err(), "encodeValue({value:?}) must throw like JS");
    }
    cases.retain(|value| !matches!(value, SqlValue::Real(number) if !number.is_finite()));
    let js = calls(
        "module:library-rows",
        &cases
            .iter()
            .map(|value| {
                let argument = match value {
                    SqlValue::Integer(number) => Value::BigInt(i128::from(*number)),
                    other => other.to_value(),
                };
                ("encodeValue", vec![argument])
            })
            .collect::<Vec<_>>(),
    );
    for (value, answer) in cases.iter().zip(&js) {
        assert_same(&encode_value(value), answer, &format!("encodeValue({value:?})"));
    }
}

fn round_trip(create: &str, table: &str, columns: &[String], rows: &[Vec<Value>]) -> Result<Value> {
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(":memory:", false)?);
    db.exec_batch(create)?;
    let inserted = match insert_rows(&*db, table, columns, rows) {
        Ok(count) => Value::from(count),
        Err(Error::Status { status, message, .. }) => obj! { "threw" => message, "status" => status }.into(),
        Err(other) => return Err(other),
    };
    let count = db.query(&format!("SELECT COUNT(*) AS n FROM \"{}\"", table.replace('"', "")), &[])?[0].get("n").to_value();
    let valid: Vec<String> = columns.iter().filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !name.starts_with(|c: char| c.is_ascii_digit())).cloned().collect();
    let digest = match table_digest(&*db, table, &valid) {
        Ok(digest) => digest.to_value(),
        Err(Error::Status { status, message, .. }) => obj! { "threw" => message, "status" => status }.into(),
        Err(other) => return Err(other),
    };
    Ok(obj! { "inserted" => inserted, "count" => count, "digest" => digest }.into())
}

#[test]
fn insert_rows_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(9);
    let create = "CREATE TABLE t (a INTEGER, b REAL, c TEXT, d BLOB, e)";
    let columns: Vec<String> = ["a", "b", "c", "d", "e"].iter().map(|name| (*name).to_owned()).collect();
    let integer = |digits: &str| Value::from(obj! { "i" => digits });
    let blob = |text: &str| Value::from(obj! { "b" => text });
    let mut batches: Vec<Vec<Vec<Value>>> = vec![
        vec![],
        vec![vec![integer("0"), 1.5.into(), "x".into(), blob("AAEC"), Value::Null]],
        vec![vec![integer("9223372036854775807"), Value::Null, "".into(), blob(""), integer("-9223372036854775808")]],
        vec![vec![integer("5"), Value::Null, Value::Null, Value::Null, "\u{1f600}".into()], vec![integer("6"), 2.0.into(), "é".into(), blob("/+8="), 3.5.into()]],
        vec![vec![integer("12345678901234567890"), Value::Null, Value::Null, Value::Null, Value::Null]],
        vec![vec![integer("abc"), Value::Null, Value::Null, Value::Null, Value::Null]],
    ];
    for _ in 0..40 {
        let rows = (0..random.below(6) + 1)
            .map(|_| {
                vec![
                    integer(&(random.next() as i64).to_string()),
                    if random.below(3) == 0 { Value::Null } else { Value::from((random.below(100000) as f64) / 7.0) },
                    Value::from(random.text(&["a", "b", "é", " ", "\u{0}x", "😀"], 5)),
                    blob(&mm_js::base64::encode(&(0..random.below(20)).map(|_| random.below(256) as u8).collect::<Vec<_>>())),
                    match random.below(4) {
                        0 => integer(&(random.below(1000) as i64 - 500).to_string()),
                        1 => Value::from(random.text(&["z", "y"], 3)),
                        2 => Value::Null,
                        _ => Value::from(random.below(1000) as f64 / 8.0),
                    },
                ]
            })
            .collect();
        batches.push(rows);
    }
    for rows in &batches {
        let js = calls(DRIVER, &[("roundTrip", vec![create.into(), "t".into(), names(&columns), Value::Array(rows.iter().map(|row| Value::Array(row.clone())).collect())])]).remove(0);
        assert_same(&round_trip(create, "t", &columns, rows), &js, &format!("insertRows({rows:?})"));
    }
}

#[test]
fn insert_rows_validates_names_like_node() {
    let _goldens = common::test_guard();
    let create = "CREATE TABLE t (a, b)";
    let row = vec![Value::from(1), Value::from(2)];
    for (table, columns) in [("t", vec!["a", "b"]), ("t x", vec!["a", "b"]), ("t", vec!["a", "b c"]), ("t", vec!["a\"", "b"]), ("1t", vec!["a", "b"]), ("t", vec!["a", "b\n"]), ("", vec!["a", "b"]), ("t", vec!["é", "b"])] {
        let columns: Vec<String> = columns.into_iter().map(str::to_owned).collect();
        let rows = vec![row.clone()];
        let js = calls(DRIVER, &[("roundTrip", vec![create.into(), table.into(), names(&columns), arr![Value::Array(row.clone())]])]).remove(0);
        let rust = round_trip(create, table, &columns, &rows);
        assert_same(&rust, &js, &format!("insertRows({table:?}, {columns:?})"));
    }
}
