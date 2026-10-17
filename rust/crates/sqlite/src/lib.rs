//! SQLite files behind [`mm_core::sql::Sql`], opened with the settings `openSql` (`lib/sql-node.mjs`) used.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use mm_core::sql::{Row, Sql, SqlValue};
use mm_core::{Error, Result};
use rusqlite::config::DbConfig;
use rusqlite::types::{ToSqlOutput, ValueRef};
use rusqlite::{Connection, OpenFlags, ToSql};

/// An open SQLite database.
pub struct Sqlite {
    connection: Connection,
    depth: Cell<usize>,
}

fn failure(error: rusqlite::Error) -> Error {
    Error::internal(error.to_string())
}

struct Param<'a>(&'a SqlValue);

impl ToSql for Param<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(match self.0 {
            SqlValue::Null => ValueRef::Null,
            SqlValue::Integer(number) => ValueRef::Integer(*number),
            SqlValue::Real(number) => ValueRef::Real(*number),
            SqlValue::Text(text) => ValueRef::Text(text.as_bytes()),
            SqlValue::Blob(bytes) => ValueRef::Blob(bytes),
        }))
    }
}

fn value(cell: ValueRef<'_>) -> SqlValue {
    match cell {
        ValueRef::Null => SqlValue::Null,
        ValueRef::Integer(number) => SqlValue::Integer(number),
        ValueRef::Real(number) => SqlValue::Real(number),
        ValueRef::Text(text) => SqlValue::Text(String::from_utf8_lossy(text).into_owned()),
        ValueRef::Blob(bytes) => SqlValue::Blob(bytes.to_vec()),
    }
}

impl Sqlite {
    /// Open a database file, or `:memory:`, as `openSql` did: WAL journal, foreign keys on and normal sync unless
    /// read-only, and double-quoted strings refused as node:sqlite refuses them.
    ///
    /// # Arguments
    /// * `path` - Database file or `:memory:`; parent directories are created unless read-only.
    /// * `read_only` - Open without write access.
    ///
    /// # Errors
    /// The file cannot be created or opened.
    pub fn open(path: impl AsRef<Path>, read_only: bool) -> Result<Self> {
        let path = path.as_ref();
        let memory = path == Path::new(":memory:");
        if !memory
            && !read_only
            && let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| Error::internal(error.to_string()))?;
        }
        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_NO_MUTEX
        };
        let connection = if memory { Connection::open_in_memory() } else { Connection::open_with_flags(path, flags) }.map_err(failure)?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DQS_DML, false).map_err(failure)?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DQS_DDL, false).map_err(failure)?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true).map_err(failure)?;
        connection.set_prepared_statement_cache_capacity(256);
        if !read_only {
            connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;").map_err(failure)?;
        }
        Ok(Self { connection, depth: Cell::new(0) })
    }

    /// The underlying connection, for host code that needs SQLite itself (backups, imports).
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

impl Sql for Sqlite {
    fn query(&self, sql: &str, params: &[SqlValue]) -> Result<Vec<Row>> {
        let mut rows = Vec::new();
        self.query_each(sql, params, &mut |row| {
            rows.push(row);
            Ok(true)
        })?;
        Ok(rows)
    }

    fn query_each(&self, sql: &str, params: &[SqlValue], each: &mut dyn FnMut(Row) -> Result<bool>) -> Result<()> {
        let mut statement = self.connection.prepare_cached(sql).map_err(failure)?;
        let columns: Rc<[String]> = statement.column_names().into_iter().map(str::to_owned).collect();
        let bound: Vec<Param<'_>> = params.iter().map(Param).collect();
        let mut rows = statement.query(rusqlite::params_from_iter(bound.iter())).map_err(failure)?;
        while let Some(row) = rows.next().map_err(failure)? {
            let values = (0..columns.len()).map(|index| row.get_ref(index).map(value)).collect::<rusqlite::Result<Vec<_>>>().map_err(failure)?;
            if !each(Row::new(Rc::clone(&columns), values))? {
                break;
            }
        }
        Ok(())
    }

    fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<usize> {
        let mut statement = self.connection.prepare_cached(sql).map_err(failure)?;
        for (index, param) in params.iter().enumerate() {
            statement.raw_bind_parameter(index + 1, Param(param)).map_err(failure)?;
        }
        let mut rows = statement.raw_query();
        while rows.next().map_err(failure)?.is_some() {}
        Ok(self.connection.changes() as usize)
    }

    fn exec_batch(&self, sql: &str) -> Result<()> {
        self.connection.execute_batch(sql).map_err(failure)
    }

    fn transaction_dyn(&self, work: &mut dyn FnMut() -> Result<()>) -> Result<()> {
        let depth = self.depth.get();
        let savepoint = (depth > 0).then(|| format!("nested_{depth}"));
        self.exec_batch(&savepoint.as_ref().map_or_else(|| "BEGIN IMMEDIATE".to_owned(), |name| format!("SAVEPOINT {name}")))?;
        self.depth.set(depth + 1);
        let result = work();
        self.depth.set(depth);
        match (&result, &savepoint) {
            (Ok(()), Some(name)) => self.exec_batch(&format!("RELEASE {name}"))?,
            (Ok(()), None) => self.exec_batch("COMMIT")?,
            (Err(_), Some(name)) => self.exec_batch(&format!("ROLLBACK TO {name}; RELEASE {name}"))?,
            (Err(_), None) => self.exec_batch("ROLLBACK")?,
        }
        result
    }
}
