//! Port of `lib/library-rows.mjs`: lossless row transport and table digests.

use std::sync::LazyLock;

use mm_js::{Value, base64, json, number_to_string};
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::sql::{Sql, SqlExt, SqlValue};
use crate::{Error, Result};

static IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("pattern"));

fn quote(name: &str) -> Result<String> {
    if !IDENTIFIER.is_match(name) {
        let shown = json::stringify(&Value::from(name)).ok().flatten().unwrap_or_default();
        return Err(Error::status(400, format!("Invalid SQL name {shown}")));
    }
    Ok(format!("\"{name}\""))
}

/// Encode one SQLite value for JSON transport, keeping its storage class (`encodeValue`).
///
/// # Arguments
/// * `value` - Column value read from SQLite.
///
/// # Returns
/// `{i: "<digits>"}` for an integer, `{b: "<base64>"}` for a blob, and text, reals and null unchanged.
///
/// # Errors
/// An internal error for a real that JSON cannot carry (infinite or NaN).
pub fn encode_value(value: &SqlValue) -> Result<Value> {
    let mut object = mm_js::Object::new();
    match value {
        SqlValue::Integer(number) => object.insert("i", number.to_string()),
        SqlValue::Blob(bytes) => object.insert("b", base64::encode(bytes)),
        SqlValue::Real(number) if !number.is_finite() => return Err(Error::internal("Cannot transport an infinite real")),
        SqlValue::Real(number) => return Ok(Value::Number(*number)),
        SqlValue::Text(text) => return Ok(Value::String(text.clone())),
        SqlValue::Null => return Ok(Value::Null),
    }
    Ok(Value::Object(object))
}

fn param(value: &Value) -> Result<SqlValue> {
    match value {
        Value::String(text) => Ok(SqlValue::Text(text.clone())),
        Value::Number(number) => Ok(SqlValue::Real(*number)),
        Value::Bool(flag) => Ok(SqlValue::Integer(i64::from(*flag))),
        other => Err(Error::internal(format!("SQLite3 can only bind numbers, strings, bigints, buffers, and null, not {}", other.type_of()))),
    }
}

/// Insert rows produced by [`encode_value`] into a table (`insertRows`).
///
/// # Arguments
/// * `db` - SQL handle.
/// * `table` - Table name.
/// * `columns` - Column names, including `rowid` when the row ids must be kept.
/// * `rows` - Encoded rows, one value per column.
///
/// # Returns
/// The number of rows inserted.
///
/// # Errors
/// Status 400 for a name that is not a plain SQL identifier, or the database's error.
pub fn insert_rows(db: &dyn Sql, table: &str, columns: &[String], rows: &[Vec<Value>]) -> Result<usize> {
    let quoted = columns.iter().map(|name| quote(name)).collect::<Result<Vec<_>>>()?;
    let into = format!("INSERT INTO {} ({}) VALUES ", quote(table)?, quoted.join(", "));
    db.transaction(|| {
        for row in rows {
            let mut params = Vec::new();
            let mut slots = Vec::new();
            for value in row {
                match value {
                    Value::Null => slots.push("NULL"),
                    Value::Object(object) if object.get("i").is_some() => {
                        params.push(param(object.get("i").unwrap_or(&Value::Undefined))?);
                        slots.push("CAST(? AS INTEGER)");
                    }
                    Value::Object(object) if object.get("b").is_some() => {
                        let text = object.get("b").and_then(Value::as_str).ok_or_else(|| Error::invalid("The first argument must be of type string"))?;
                        params.push(SqlValue::Blob(base64::decode(text)));
                        slots.push("?");
                    }
                    other => {
                        params.push(param(other)?);
                        slots.push("?");
                    }
                }
            }
            db.execute(&format!("{into}({})", slots.join(", ")), &params)?;
        }
        Ok(())
    })?;
    Ok(rows.len())
}

/// Row count and order-independent digest of a table's contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableDigest {
    /// Rows in the table.
    pub rows: usize,
    /// Hex SHA-256 over the sorted per-row digests.
    pub sha256: String,
}

impl TableDigest {
    /// The `{rows, sha256}` object the JS returned.
    pub fn to_value(&self) -> Value {
        let mut object = mm_js::Object::new();
        object.insert("rows", self.rows as f64);
        object.insert("sha256", self.sha256.clone());
        Value::Object(object)
    }
}

/// Digest every row of a table, equal on any two databases with the same rows and storage classes (`tableDigest`).
///
/// # Arguments
/// * `db` - SQL handle.
/// * `table` - Table name.
/// * `columns` - Column names to cover, including `rowid` to cover the row ids.
///
/// # Returns
/// Row count and digest.
///
/// # Errors
/// Status 400 for a name that is not a plain SQL identifier, or the database's error.
pub fn table_digest(db: &dyn Sql, table: &str, columns: &[String]) -> Result<TableDigest> {
    let mut select = Vec::new();
    for (index, name) in columns.iter().enumerate() {
        let name = quote(name)?;
        select.push(format!("typeof({name}) AS t{index}, CASE WHEN typeof({name}) = 'integer' THEN CAST({name} AS TEXT) ELSE {name} END AS v{index}"));
    }
    let mut hashes = Vec::new();
    db.query_each(&format!("SELECT {} FROM {}", select.join(", "), quote(table)?), &[], &mut |row| {
        let mut hash = Sha256::new();
        for index in 0..columns.len() {
            let bytes: Vec<u8> = match row.get(&format!("v{index}")) {
                SqlValue::Blob(bytes) => bytes.clone(),
                SqlValue::Null => Vec::new(),
                SqlValue::Text(text) => text.split('\0').next().unwrap_or_default().as_bytes().to_vec(),
                SqlValue::Real(number) => number_to_string(*number).into_bytes(),
                SqlValue::Integer(number) => number.to_string().into_bytes(),
            };
            hash.update(format!("{}\0{}\0", row.text(&format!("t{index}")).unwrap_or(""), bytes.len()).as_bytes());
            hash.update(&bytes);
        }
        hashes.push(base64::hex(&hash.finalize()));
        Ok(true)
    })?;
    hashes.sort();
    Ok(TableDigest { rows: hashes.len(), sha256: base64::hex(&Sha256::digest(hashes.join("\n").as_bytes())) })
}
