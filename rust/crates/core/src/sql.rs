//! The synchronous SQL interface the store shares between SQLite files (`mm-sqlite`) and Durable Object storage.
//!
//! Values bind as the JS backend bound them, so stored bytes stay the same: every JS number as REAL (node:sqlite and
//! Durable Objects both bind doubles; an INTEGER column converts it back), and every `BigInt`, which neither platform
//! binds, as decimal text.

use std::rc::Rc;

use mm_js::{Object, Value};

use crate::Result;

/// One SQLite value, as bound or as read.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    Null,
    /// Read from an INTEGER value. Durable Objects read integers as JS numbers, so code must treat
    /// `Integer` and `Real` alike; nothing binds `Integer`.
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SqlValue {
    /// A `BigInt` parameter, bound as its decimal text.
    ///
    /// # Arguments
    /// * `value` - The integer, such as nanoseconds since the epoch.
    pub fn big(value: i128) -> Self {
        Self::Text(value.to_string())
    }

    /// Whether the value is NULL.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// The number an INTEGER or REAL value holds.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(number) => Some(*number as f64),
            Self::Real(number) => Some(*number),
            _ => None,
        }
    }

    /// The integer an INTEGER value, or a REAL value with no fraction, holds.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(number) => Some(*number),
            Self::Real(number) if number.fract() == 0.0 && number.abs() < 9.3e18 => Some(*number as i64),
            _ => None,
        }
    }

    /// The text a TEXT value holds.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The bytes a BLOB value holds.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Blob(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// The value as the JS store read it: a number, a string, a `Uint8Array` or `null`.
    pub fn to_value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Integer(number) => Value::Number(*number as f64),
            Self::Real(number) => Value::Number(*number),
            Self::Text(text) => Value::String(text.clone()),
            Self::Blob(bytes) => Value::Bytes(bytes.clone()),
        }
    }
}

impl From<f64> for SqlValue {
    fn from(number: f64) -> Self {
        Self::Real(number)
    }
}

macro_rules! from_number {
    ($($kind:ty),*) => {$(
        impl From<$kind> for SqlValue {
            /// A JS number parameter, bound as REAL as the JS store bound it.
            fn from(number: $kind) -> Self {
                Self::Real(number as f64)
            }
        }
    )*};
}
from_number!(i32, i64, u32, u64, usize);

impl From<&str> for SqlValue {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for SqlValue {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&String> for SqlValue {
    fn from(text: &String) -> Self {
        Self::Text(text.clone())
    }
}

impl From<Vec<u8>> for SqlValue {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Blob(bytes)
    }
}

impl From<&[u8]> for SqlValue {
    fn from(bytes: &[u8]) -> Self {
        Self::Blob(bytes.to_vec())
    }
}

impl<T: Into<SqlValue>> From<Option<T>> for SqlValue {
    /// `None` binds NULL.
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Null, Into::into)
    }
}

/// Parameters for a statement: `params![id, SqlValue::big(ns), name]`.
#[macro_export]
macro_rules! params {
    ($($value:expr),* $(,)?) => { &[$($crate::sql::SqlValue::from($value)),*][..] };
}

/// One result row, with its columns by name in select order.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    columns: Rc<[String]>,
    values: Vec<SqlValue>,
}

static NULL: SqlValue = SqlValue::Null;

impl Row {
    /// A row from its column names and values, in select order.
    ///
    /// # Arguments
    /// * `columns` - Column names, shared by every row of one result.
    /// * `values` - One value per column.
    pub fn new(columns: Rc<[String]>, values: Vec<SqlValue>) -> Self {
        debug_assert_eq!(columns.len(), values.len());
        Self { columns, values }
    }

    /// The named column's value; NULL for a column the row does not have.
    ///
    /// # Arguments
    /// * `column` - Column name or alias. With duplicate names the last one wins, as in a JS row object.
    pub fn get(&self, column: &str) -> &SqlValue {
        self.columns.iter().rposition(|name| name == column).map_or(&NULL, |index| &self.values[index])
    }

    /// The named column as text, or `None` when it is not TEXT.
    ///
    /// # Arguments
    /// * `column` - Column name or alias.
    pub fn text(&self, column: &str) -> Option<&str> {
        self.get(column).as_str()
    }

    /// The named column as a number, or `None` when it is not INTEGER or REAL.
    ///
    /// # Arguments
    /// * `column` - Column name or alias.
    pub fn number(&self, column: &str) -> Option<f64> {
        self.get(column).as_f64()
    }

    /// The named column as an integer, or `None` when it holds no whole number.
    ///
    /// # Arguments
    /// * `column` - Column name or alias.
    pub fn int(&self, column: &str) -> Option<i64> {
        self.get(column).as_i64()
    }

    /// The named column as bytes, or `None` when it is not a BLOB.
    ///
    /// # Arguments
    /// * `column` - Column name or alias.
    pub fn blob(&self, column: &str) -> Option<&[u8]> {
        self.get(column).as_bytes()
    }

    /// Column names in select order.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Values in select order.
    pub fn values(&self) -> &[SqlValue] {
        &self.values
    }

    /// The row as the JS store read it: an object of its columns, the last of duplicate names winning.
    pub fn to_object(&self) -> Object {
        let mut object = Object::new();
        for (name, value) in self.columns.iter().zip(&self.values) {
            object.insert(name.as_str(), value.to_value());
        }
        object
    }
}

/// A SQL database the store runs synchronous statements on, as `openSql` (`rust/crates/sqlite/src/lib.rs`) and the Durable Object
/// adapter (`rust/crates/worker/src/sql_do.rs`) provide it.
pub trait Sql {
    /// Run one statement and collect its rows.
    ///
    /// # Arguments
    /// * `sql` - One statement with `?` placeholders.
    /// * `params` - One value per placeholder.
    ///
    /// # Errors
    /// The database's error, which the API answers with a 500.
    fn query(&self, sql: &str, params: &[SqlValue]) -> Result<Vec<Row>>;

    /// Run one statement and hand each row to `each` as it is read, for results too big to collect.
    ///
    /// # Arguments
    /// * `sql` - One statement with `?` placeholders.
    /// * `params` - One value per placeholder.
    /// * `each` - Called with every row until it returns `false`; an error stops the query and is returned.
    ///
    /// # Errors
    /// The database's error, or the first one `each` returns.
    fn query_each(&self, sql: &str, params: &[SqlValue], each: &mut dyn FnMut(Row) -> Result<bool>) -> Result<()> {
        for row in self.query(sql, params)? {
            if !each(row)? {
                break;
            }
        }
        Ok(())
    }

    /// Run one statement for its effect.
    ///
    /// # Arguments
    /// * `sql` - One statement with `?` placeholders.
    /// * `params` - One value per placeholder.
    ///
    /// # Returns
    /// The number of rows it inserted, changed or deleted (`run().changes`).
    ///
    /// # Errors
    /// The database's error.
    fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<usize>;

    /// Run statements without parameters, separated by semicolons (`exec`).
    ///
    /// # Arguments
    /// * `sql` - The statements.
    ///
    /// # Errors
    /// The database's error; statements before the failing one stay applied.
    fn exec_batch(&self, sql: &str) -> Result<()>;

    /// Run `work` in a transaction that commits when it succeeds and rolls back when it fails. Transactions nest: an
    /// inner failure rolls back only the inner work. Use [`SqlExt::transaction`] to return a value.
    ///
    /// # Arguments
    /// * `work` - The statements to run atomically.
    ///
    /// # Errors
    /// The error `work` returned, after rolling back, or the database's error.
    fn transaction_dyn(&self, work: &mut dyn FnMut() -> Result<()>) -> Result<()>;
}

/// Conveniences over every [`Sql`] database.
pub trait SqlExt: Sql {
    /// The first row of a query, as a statement's `get` returns it, or `None` when there is none.
    ///
    /// # Arguments
    /// * `sql` - One statement with `?` placeholders.
    /// * `params` - One value per placeholder.
    ///
    /// # Errors
    /// The database's error.
    fn one(&self, sql: &str, params: &[SqlValue]) -> Result<Option<Row>> {
        let mut first = None;
        self.query_each(sql, params, &mut |row| {
            first = Some(row);
            Ok(false)
        })?;
        Ok(first)
    }

    /// Run `work` in a nested transaction and return what it returns.
    ///
    /// # Arguments
    /// * `work` - The statements to run atomically.
    ///
    /// # Errors
    /// As [`Sql::transaction_dyn`].
    fn transaction<T>(&self, work: impl FnOnce() -> Result<T>) -> Result<T> {
        let mut work = Some(work);
        let mut result = None;
        self.transaction_dyn(&mut || {
            let value = (work.take().expect("a transaction runs its work once"))()?;
            result = Some(value);
            Ok(())
        })?;
        Ok(result.expect("a committed transaction ran its work"))
    }
}

impl<T: Sql + ?Sized> SqlExt for T {}
