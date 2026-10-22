//! The app's change log: every card whose row, tags, text, spaces or pin changed, newest last, for
//! `GET /app/v1/changes`.

use crate::runtime;
use crate::sql::{Sql, SqlExt, SqlValue};
use crate::Result;

// Tables whose rows belong to a card through `card_id`; a write to any of them changes that card in the app.
const CARD_TABLES: [&str; 4] = ["annotations", "card_content", "space_cards", "pins"];

fn touch(id: &str) -> String {
    format!("DELETE FROM app_sync WHERE card_id = {id}; INSERT INTO app_sync (card_id) VALUES ({id});")
}

fn epoch(db: &dyn Sql) -> Result<Option<String>> {
    if db.one("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_sync_epoch'", &[])?.is_none() {
        return Ok(None);
    }
    Ok(db.one("SELECT epoch FROM app_sync_epoch WHERE id = 1", &[])?.and_then(|row| row.text("epoch").map(str::to_owned)))
}

/// Start the change log on first use and return its epoch.
///
/// The first call creates the log, the triggers that keep it, and an entry for every existing card, oldest first, so
/// a phone that starts from nothing reads the whole library through the same feed. Later calls only read the epoch.
///
/// # Arguments
/// * `db` - Writable database with the store schema.
///
/// # Returns
/// The log's epoch: it changes whenever the log is created again (a library replaced by an import starts a new one),
/// which tells a phone holding an older position to start over.
///
/// # Errors
/// The database's error, such as a read-only database on first use.
pub fn install_app_sync(db: &dyn Sql) -> Result<String> {
    if let Some(found) = epoch(db)? {
        return Ok(found);
    }
    db.transaction(|| {
        if let Some(found) = epoch(db)? {
            return Ok(found);
        }
        let noise: String = runtime::random_bytes(4).iter().map(|byte| format!("{byte:02x}")).collect();
        let started = format!("{}-{noise}", runtime::now_ms() as i64);
        let mut sql = String::from(
            "CREATE TABLE app_sync (seq INTEGER PRIMARY KEY AUTOINCREMENT, card_id TEXT NOT NULL UNIQUE) STRICT;
    CREATE TABLE app_sync_epoch (id INTEGER PRIMARY KEY CHECK (id = 1), epoch TEXT NOT NULL) STRICT;",
        );
        sql.push_str(&format!(
            "CREATE TRIGGER app_sync_cards_insert AFTER INSERT ON cards BEGIN {} END;
    CREATE TRIGGER app_sync_cards_update AFTER UPDATE ON cards BEGIN {} END;
    CREATE TRIGGER app_sync_cards_delete AFTER DELETE ON cards BEGIN {} END;",
            touch("new.id"),
            touch("new.id"),
            touch("old.id")
        ));
        for table in CARD_TABLES {
            sql.push_str(&format!(
                "CREATE TRIGGER app_sync_{table}_insert AFTER INSERT ON {table} BEGIN {} END;
    CREATE TRIGGER app_sync_{table}_update AFTER UPDATE ON {table} BEGIN {}
      DELETE FROM app_sync WHERE card_id = old.card_id AND old.card_id <> new.card_id;
      INSERT INTO app_sync (card_id) SELECT old.card_id WHERE old.card_id <> new.card_id; END;
    CREATE TRIGGER app_sync_{table}_delete AFTER DELETE ON {table} BEGIN {} END;",
                touch("new.card_id"),
                touch("new.card_id"),
                touch("old.card_id")
            ));
        }
        sql.push_str("INSERT INTO app_sync (card_id) SELECT id FROM cards ORDER BY bumped_ns, id;");
        db.exec_batch(&sql)?;
        db.execute("INSERT INTO app_sync_epoch (id, epoch) VALUES (1, ?)", &[SqlValue::from(started.as_str())])?;
        Ok(started)
    })
}

/// One entry of the change log.
pub struct Logged {
    /// Position in the log; later changes have larger numbers.
    pub seq: i64,
    /// The changed card, which may since have been trashed or deleted.
    pub card_id: String,
}

/// The cards changed after a position in the log, oldest change first, each listed once at its latest change.
///
/// # Arguments
/// * `db` - Database where [`install_app_sync`] has run.
/// * `after` - Log position already seen; 0 reads the whole log.
/// * `limit` - Most entries to return.
///
/// # Errors
/// The database's error.
pub fn changed_after(db: &dyn Sql, after: i64, limit: usize) -> Result<Vec<Logged>> {
    let rows = db.query("SELECT seq, card_id FROM app_sync WHERE seq > ? ORDER BY seq LIMIT ?", &[after.into(), (limit as f64).into()])?;
    Ok(rows.iter().map(|row| Logged { seq: row.int("seq").unwrap_or(0), card_id: row.text("card_id").unwrap_or("").to_owned() }).collect())
}
