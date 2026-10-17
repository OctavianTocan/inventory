//! Port of `lib/search-index.mjs`: the FTS5 index over `search_documents` and the queue that keeps it current.

use crate::search_document::search_document;
use crate::sql::{Sql, SqlExt, SqlValue};
use crate::Result;

const FIELDS: [&str; 8] = ["title", "description", "summary", "tags", "url", "body", "note", "ocr"];

fn names() -> String {
    FIELDS.join(", ")
}

fn values(prefix: &str) -> String {
    FIELDS.iter().map(|name| format!("{prefix}.{name}")).collect::<Vec<_>>().join(", ")
}

fn dirty(id: &str) -> String {
    format!("INSERT OR IGNORE INTO search_dirty (id) VALUES ({id});")
}

fn current(db: &dyn Sql) -> Result<bool> {
    Ok(db.one("SELECT 1 FROM sqlite_master WHERE name = 'search_dirty'", &[])?.is_some()
        && db.one("SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND sql LIKE '%search_document(%'", &[])?.is_none())
}

/// Install FTS5 and change tracking, indexing existing rows on the first open (`installSearch`).
///
/// # Arguments
/// * `db` - Writable database that already has the base schema.
///
/// # Errors
/// The database's error.
pub fn install_search(db: &dyn Sql) -> Result<()> {
    if current(db)? {
        return refresh_search(db);
    }
    db.transaction(|| {
        if current(db)? {
            return Ok(());
        }
        let fresh = db.one("SELECT 1 FROM sqlite_master WHERE name = 'search_fts'", &[])?.is_none();
        if fresh {
            let generated = FIELDS
                .iter()
                .map(|name| format!("{name} TEXT GENERATED ALWAYS AS (json_extract(fields, '$.{name}')) STORED"))
                .collect::<Vec<_>>()
                .join(",\n");
            let (names, new, old) = (names(), values("new"), values("old"));
            db.exec_batch(&format!(
                "CREATE TABLE search_documents (
      id TEXT NOT NULL UNIQUE REFERENCES cards(id) ON DELETE CASCADE,
      fields TEXT NOT NULL CHECK(json_valid(fields)),
      {generated}
    );
    CREATE VIRTUAL TABLE search_fts USING fts5({names}, content='search_documents', content_rowid='rowid',
      tokenize='unicode61 remove_diacritics 2', prefix='2 3 4 5 6');
    CREATE TRIGGER search_documents_insert AFTER INSERT ON search_documents BEGIN
      INSERT INTO search_fts(rowid, {names}) VALUES (new.rowid, {new}); END;
    CREATE TRIGGER search_documents_delete AFTER DELETE ON search_documents BEGIN
      INSERT INTO search_fts(search_fts, rowid, {names}) VALUES ('delete', old.rowid, {old}); END;
    CREATE TRIGGER search_documents_update AFTER UPDATE ON search_documents BEGIN
      INSERT INTO search_fts(search_fts, rowid, {names}) VALUES ('delete', old.rowid, {old});
      INSERT INTO search_fts(rowid, {names}) VALUES (new.rowid, {new}); END;"
            ))?;
        }
        for row in db.query("SELECT name FROM sqlite_master WHERE type = 'trigger' AND sql LIKE '%search_document(%'", &[])? {
            db.exec_batch(&format!("DROP TRIGGER {}", row.text("name").unwrap_or("")))?;
        }
        db.exec_batch(&format!(
            "CREATE TABLE IF NOT EXISTS search_dirty (id TEXT PRIMARY KEY) STRICT;
    CREATE TRIGGER IF NOT EXISTS search_cards_insert AFTER INSERT ON cards BEGIN {} END;
    CREATE TRIGGER IF NOT EXISTS search_cards_update AFTER UPDATE OF data, extras ON cards BEGIN {} END;
    CREATE TRIGGER IF NOT EXISTS search_cards_delete AFTER DELETE ON cards BEGIN DELETE FROM search_documents WHERE id = old.id; END;",
            dirty("new.id"),
            dirty("new.id")
        ))?;
        for table in ["annotations", "card_content"] {
            for action in ["INSERT", "UPDATE", "DELETE"] {
                let ids: Vec<&str> = match action {
                    "UPDATE" => vec!["old.card_id", "new.card_id"],
                    "DELETE" => vec!["old.card_id"],
                    _ => vec!["new.card_id"],
                };
                let body = ids.iter().map(|id| dirty(id)).collect::<Vec<_>>().join("\n");
                db.exec_batch(&format!(
                    "CREATE TRIGGER IF NOT EXISTS search_{table}_{} AFTER {action} ON {table}
          BEGIN {body} END;",
                    action.to_lowercase()
                ))?;
            }
        }
        if fresh {
            db.exec_batch("INSERT OR IGNORE INTO search_dirty (id) SELECT id FROM cards")?;
        }
        Ok(())
    })?;
    refresh_search(db)
}

/// Reindex the cards changed since the last refresh (`refreshSearch`).
///
/// # Arguments
/// * `db` - Writable database with the search index installed.
///
/// # Errors
/// The database's error, or the error of a card whose stored JSON cannot be indexed.
pub fn refresh_search(db: &dyn Sql) -> Result<()> {
    if db.one("SELECT 1 FROM search_dirty LIMIT 1", &[])?.is_none() {
        return Ok(());
    }
    db.transaction(|| {
        for dirty_row in db.query("SELECT id FROM search_dirty", &[])? {
            let id = dirty_row.get("id").clone();
            let row = db.one(
                "SELECT c.data, c.extras, (SELECT body FROM card_content WHERE card_id = c.id) AS content,
        (SELECT group_concat(name, char(10)) FROM annotations WHERE card_id = c.id) AS tags FROM cards c WHERE c.id = ?",
                std::slice::from_ref(&id),
            )?;
            if let Some(row) = row {
                let fields = search_document(row.text("data").unwrap_or("null"), row.text("extras").unwrap_or("null"), row.text("content"), row.text("tags"))?;
                db.execute(
                    "INSERT INTO search_documents (id, fields) VALUES (?, ?)
          ON CONFLICT(id) DO UPDATE SET fields = excluded.fields WHERE fields IS NOT excluded.fields",
                    &[id.clone(), SqlValue::from(fields)],
                )?;
            } else {
                db.execute("DELETE FROM search_documents WHERE id = ?", std::slice::from_ref(&id))?;
            }
            db.execute("DELETE FROM search_dirty WHERE id = ?", std::slice::from_ref(&id))?;
        }
        Ok(())
    })
}
