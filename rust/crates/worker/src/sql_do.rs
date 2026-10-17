//! `mm_core::sql::Sql` over a Durable Object's SQLite storage (`worker/sql-do.mjs`).

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, ArrayBuffer, Function, Uint8Array};
use mm_core::sql::{Row, Sql, SqlValue};
use mm_core::{Error, Result};
use wasm_bindgen::closure::ScopedClosure;
use wasm_bindgen::{JsCast, JsValue};

use crate::util::{call, get, js_error};

/// A Durable Object's `ctx.storage.sql`.
pub struct DoSql {
    storage: JsValue,
    sql: JsValue,
    exec: Function,
}

impl DoSql {
    /// Open the SQLite storage of the object whose state is `state` (`openDurableSql`).
    ///
    /// # Arguments
    /// * `state` - The Durable Object's state.
    ///
    /// # Errors
    /// An internal error when the object has no SQLite storage.
    pub fn open(state: &worker::State) -> Result<Rc<Self>> {
        let storage = get(state.as_raw().as_ref(), "storage")?;
        let sql = get(&storage, "sql")?;
        let exec = get(&sql, "exec")?.dyn_into().map_err(|_| Error::internal("storage.sql.exec is missing"))?;
        Ok(Rc::new(Self { storage, sql, exec }))
    }

    fn run(&self, sql: &str, params: &[SqlValue]) -> Result<JsValue> {
        let args = Array::new();
        args.push(&sql.into());
        for value in params {
            args.push(&bind(value));
        }
        self.exec.apply(&self.sql, &args).map_err(js_error)
    }

    fn consume(cursor: &JsValue) {
        let _ = Array::from(cursor);
    }
}

fn bind(value: &SqlValue) -> JsValue {
    match value {
        SqlValue::Null => JsValue::NULL,
        SqlValue::Integer(number) => JsValue::from_f64(*number as f64),
        SqlValue::Real(number) => JsValue::from_f64(*number),
        SqlValue::Text(text) => JsValue::from_str(text),
        SqlValue::Blob(bytes) => Uint8Array::from(bytes.as_slice()).into(),
    }
}

fn read(value: &JsValue) -> SqlValue {
    if value.is_null() || value.is_undefined() {
        return SqlValue::Null;
    }
    if let Some(number) = value.as_f64() {
        let whole = number.fract() == 0.0 && number.abs() < 9_007_199_254_740_992.0;
        return if whole { SqlValue::Integer(number as i64) } else { SqlValue::Real(number) };
    }
    if let Some(text) = value.as_string() {
        return SqlValue::Text(text);
    }
    if let Some(buffer) = value.dyn_ref::<ArrayBuffer>() {
        return SqlValue::Blob(Uint8Array::new(buffer).to_vec());
    }
    if let Some(bytes) = value.dyn_ref::<Uint8Array>() {
        return SqlValue::Blob(bytes.to_vec());
    }
    SqlValue::Null
}

fn columns(cursor: &JsValue) -> Result<Rc<[String]>> {
    let names = Array::from(&get(cursor, "columnNames")?);
    Ok(names.iter().map(|name| name.as_string().unwrap_or_default()).collect::<Vec<_>>().into())
}

fn row(columns: &Rc<[String]>, raw: &JsValue) -> Row {
    Row::new(columns.clone(), Array::from(raw).iter().map(|value| read(&value)).collect())
}

impl Sql for DoSql {
    fn query(&self, sql: &str, params: &[SqlValue]) -> Result<Vec<Row>> {
        let cursor = self.run(sql, params)?;
        let names = columns(&cursor)?;
        let raw = call(&cursor, "raw", &[])?;
        Ok(Array::from(&raw).iter().map(|item| row(&names, &item)).collect())
    }

    fn query_each(&self, sql: &str, params: &[SqlValue], each: &mut dyn FnMut(Row) -> Result<bool>) -> Result<()> {
        let cursor = self.run(sql, params)?;
        let names = columns(&cursor)?;
        let raw = call(&cursor, "raw", &[])?;
        for item in js_sys::try_iter(&raw).map_err(js_error)?.into_iter().flatten() {
            if !each(row(&names, &item.map_err(js_error)?))? {
                break;
            }
        }
        Ok(())
    }

    fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<usize> {
        Self::consume(&self.run(sql, params)?);
        let changes = self.query("SELECT changes() AS n", &[])?;
        Ok(changes.first().and_then(|row| row.number("n")).unwrap_or(0.0) as usize)
    }

    fn exec_batch(&self, sql: &str) -> Result<()> {
        Self::consume(&self.run(sql, &[])?);
        Ok(())
    }

    fn transaction_dyn(&self, work: &mut dyn FnMut() -> Result<()>) -> Result<()> {
        let failure: RefCell<Option<Error>> = RefCell::new(None);
        let mut body = || -> std::result::Result<JsValue, JsValue> {
            work().map(|()| JsValue::UNDEFINED).map_err(|error| {
                *failure.borrow_mut() = Some(error);
                JsValue::from_str("rollback")
            })
        };
        let closure = ScopedClosure::borrow_mut(&mut body);
        let outcome = call(&self.storage, "transactionSync", &[closure.as_ref().clone()]);
        drop(closure);
        drop(body);
        match failure.into_inner() {
            Some(error) => Err(error),
            None => outcome.map(|_| ()),
        }
    }
}
