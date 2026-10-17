//! The adapter binds, reads and nests transactions as `openSql` did.

use mm_core::params;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::{Error, Value};
use mm_sqlite::Sqlite;

#[test]
fn numbers_bind_as_real_and_big_integers_as_text() {
    let db = Sqlite::open(":memory:", false).unwrap();
    let row = db.one("SELECT typeof(?) a, typeof(?) b, json_object('x', ?) c, ? || '' d", params![8, SqlValue::big(8), 8, 8.5]).unwrap().unwrap();
    assert_eq!(row.text("a"), Some("real"));
    assert_eq!(row.text("b"), Some("text"));
    assert_eq!(row.text("c"), Some("{\"x\":8.0}"));
    assert_eq!(row.text("d"), Some("8.5"));
    db.exec_batch("CREATE TABLE t (i INTEGER, x TEXT, b BLOB)").unwrap();
    assert_eq!(db.execute("INSERT INTO t VALUES (?, ?, ?)", params![8, 8, vec![1u8, 2]]).unwrap(), 1);
    let row = db.one("SELECT typeof(i) ti, i, x, b FROM t", &[]).unwrap().unwrap();
    assert_eq!(row.text("ti"), Some("integer"));
    assert_eq!(row.text("x"), Some("8.0"));
    assert_eq!(row.get("i").to_value(), Value::Number(8.0));
    assert_eq!(row.blob("b"), Some(&[1u8, 2][..]));
    assert!(db.one("SELECT \"nope\" FROM t", &[]).is_err(), "double-quoted strings are refused as node:sqlite refuses them");
}

#[test]
fn transactions_nest_and_roll_back_only_the_inner_work() {
    let db = Sqlite::open(":memory:", false).unwrap();
    db.exec_batch("CREATE TABLE t (n)").unwrap();
    db.transaction(|| {
        db.execute("INSERT INTO t VALUES (1)", &[])?;
        let inner: Result<(), Error> = db.transaction(|| {
            db.execute("INSERT INTO t VALUES (2)", &[])?;
            Err(Error::internal("inner"))
        });
        assert!(inner.is_err());
        db.execute("INSERT INTO t VALUES (3)", &[])
    })
    .unwrap();
    let rows = db.query("SELECT n FROM t ORDER BY n", &[]).unwrap();
    assert_eq!(rows.iter().map(|row| row.number("n").unwrap()).collect::<Vec<_>>(), [1.0, 3.0]);
    let failed: Result<(), Error> = db.transaction(|| {
        db.execute("INSERT INTO t VALUES (4)", &[])?;
        Err(Error::internal("outer"))
    });
    assert!(failed.is_err());
    assert_eq!(db.query("SELECT n FROM t", &[]).unwrap().len(), 2);
}

#[test]
fn bundled_sqlite_has_full_text_search() {
    let db = Sqlite::open(":memory:", false).unwrap();
    db.exec_batch("CREATE VIRTUAL TABLE f USING fts5(body)").unwrap();
    let version = db.one("SELECT sqlite_version() v", &[]).unwrap().unwrap();
    eprintln!("SQLite {}", version.text("v").unwrap());
}
