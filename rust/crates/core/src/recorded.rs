//! Port of `lib/recorded.mjs`: a library's recorded mymind responses, kept in its own database.

use std::rc::Rc;

use mm_js::{Value, json};

use crate::device::device_variant;
use crate::http::Request;
use crate::sql::{Sql, SqlExt, SqlValue};
use crate::{Error, Result};

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS recorded (
  key TEXT PRIMARY KEY,
  status INTEGER NOT NULL,
  headers TEXT NOT NULL CHECK(json_valid(headers)),
  body BLOB NOT NULL
) STRICT";

/// One recorded response.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedEntry {
    /// HTTP status.
    pub status: i64,
    /// Response headers as the stored JSON object.
    pub headers: Value,
    /// Response body, already localized.
    pub body: Vec<u8>,
}

/// The `recorded` table (`createRecorded`'s handle). Keys have the replay format
/// `[run:<run>|][<device>|]<KIND> https://access.mymind.com<path>[?search]`.
pub struct Recorded {
    db: Rc<dyn Sql>,
}

impl Recorded {
    /// Create the table when missing (`createRecorded`).
    ///
    /// # Arguments
    /// * `db` - SQL handle.
    ///
    /// # Errors
    /// The database's error.
    pub fn new(db: Rc<dyn Sql>) -> Result<Self> {
        db.exec_batch(SCHEMA)?;
        Ok(Self { db })
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.db.one("SELECT 1 FROM recorded WHERE key = ?", &[key.into()])?.is_some())
    }

    fn resolve(&self, req: &Request, key: &str) -> Result<String> {
        let mut key = key.to_owned();
        let variant = device_variant(req.header("user-agent").unwrap_or(""));
        if !variant.is_empty() && self.exists(&format!("{variant}|{key}"))? {
            key = format!("{variant}|{key}");
        }
        match req.header("x-replay-run").filter(|run| !run.is_empty()) {
            Some(run) if self.exists(&format!("run:{run}|{key}"))? => Ok(format!("run:{run}|{key}")),
            _ => Ok(key),
        }
    }

    /// The response recorded for a request's key: a phone's own recording first, then a verification run's with `x-replay-run` (`entry`).
    ///
    /// # Arguments
    /// * `req` - The request, for its user agent and `x-replay-run` header.
    /// * `key` - Base replay key.
    ///
    /// # Returns
    /// The recording, or `None`.
    ///
    /// # Errors
    /// The database's error, or an internal error for malformed stored headers.
    pub fn entry(&self, req: &Request, key: &str) -> Result<Option<RecordedEntry>> {
        let resolved = self.resolve(req, key)?;
        let Some(row) = self.db.one("SELECT status, headers, body FROM recorded WHERE key = ?", &[resolved.into()])? else { return Ok(None) };
        let headers = json::parse(row.text("headers").unwrap_or("")).map_err(Error::from)?;
        Ok(Some(RecordedEntry { status: row.int("status").unwrap_or(0), headers, body: row.blob("body").unwrap_or(&[]).to_vec() }))
    }

    /// Whether a recording exists for a request's key (`has`).
    ///
    /// # Arguments
    /// * `req` - The request, for its user agent and `x-replay-run` header.
    /// * `key` - Base replay key.
    ///
    /// # Errors
    /// The database's error.
    pub fn has(&self, req: &Request, key: &str) -> Result<bool> {
        let resolved = self.resolve(req, key)?;
        self.exists(&resolved)
    }

    /// Store a recording unless the key is taken (`put`).
    ///
    /// # Arguments
    /// * `key` - Full replay key.
    /// * `status` - HTTP status.
    /// * `headers` - Response headers as a JSON object.
    /// * `body` - Response body.
    ///
    /// # Errors
    /// The database's error, or an internal error when `headers` cannot be stringified.
    pub fn put(&self, key: &str, status: i64, headers: &Value, body: &[u8]) -> Result<()> {
        let text = json::stringify(headers).map_err(|error| Error::internal(error.0))?.unwrap_or_default();
        self.db.execute(
            "INSERT OR IGNORE INTO recorded (key, status, headers, body) VALUES (?, ?, ?, ?)",
            &[key.into(), status.into(), text.into(), SqlValue::Blob(body.to_vec())],
        )?;
        Ok(())
    }

    /// Number of recordings stored (`count`).
    ///
    /// # Errors
    /// The database's error.
    pub fn count(&self) -> Result<i64> {
        Ok(self.db.one("SELECT COUNT(*) AS n FROM recorded", &[])?.and_then(|row| row.int("n")).unwrap_or(0))
    }
}
