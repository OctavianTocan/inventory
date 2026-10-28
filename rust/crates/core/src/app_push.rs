//! The app's push (`POST /app/v1/push`): changes made on a phone, each applied once and in the order the phone made
//! them, as in the Replicache and Zero push protocol.

use mm_js::{Object, Value, number, obj};

use crate::sql::{Sql, SqlExt};
use crate::store::Store;
use crate::store_write::{Writer, text_document};
use crate::{Error, Result};

/// Most changes one push may carry.
pub const PUSH_LIMIT: usize = 100;

const CLIENTS: &str = "CREATE TABLE IF NOT EXISTS app_clients (client_id TEXT PRIMARY KEY, last_change INTEGER NOT NULL) STRICT";

fn bad(message: &str) -> Error {
    Error::status(400, message)
}

fn text<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key).as_str().filter(|value| !value.is_empty()).ok_or_else(|| bad("bad-change"))
}

// The highest change id applied for a phone, 0 before its first push.
fn last_change(db: &dyn Sql, client: &str) -> Result<i64> {
    Ok(db.one("SELECT last_change FROM app_clients WHERE client_id = ?", &[client.into()])?.and_then(|row| row.int("last_change")).unwrap_or(0))
}

fn record(db: &dyn Sql, client: &str, id: i64) -> Result<()> {
    db.execute(
        "INSERT INTO app_clients (client_id, last_change) VALUES (?, ?) ON CONFLICT(client_id) DO UPDATE SET last_change = excluded.last_change",
        &[client.into(), id.into()],
    )?;
    Ok(())
}

// One change through the same writer calls as the web routes it stands for; an added tag answers its id.
fn apply(writer: &Writer, name: &str, args: &Value) -> Result<Option<String>> {
    match name {
        "pin" => {
            let pinned = args.get("pinned").as_bool().ok_or_else(|| bad("bad-change"))?;
            writer.flag(text(args, "cardId")?, 512, pinned, &Value::Undefined)?;
        }
        "trash" => {
            writer.trash(text(args, "cardId")?, false)?;
        }
        "note" => {
            let note = args.get("text");
            if !note.is_string() {
                return Err(bad("bad-change"));
            }
            writer.prose(text(args, "cardId")?, &text_document(note)?, "note")?;
        }
        "add-tag" => return Ok(writer.add_tag(text(args, "cardId")?, args.get("name"))?.get("id").as_str().map(str::to_owned)),
        "remove-tag" => {
            writer.remove_tag(text(args, "tagId")?)?;
        }
        "add-to-space" | "remove-from-space" => {
            writer.membership(text(args, "spaceId")?, text(args, "cardId")?, name == "remove-from-space")?;
        }
        _ => return Err(bad("unknown-change")),
    }
    Ok(None)
}

// A change the library turned down, such as one for a card that is gone; any other failure stops the push.
fn refused(error: &Error) -> bool {
    match error {
        Error::Status { status, .. } => (400..500).contains(status),
        Error::Invalid(_) => true,
        Error::Internal(_) => false,
    }
}

/// Apply a phone's changes (`POST /app/v1/push` with `{clientId, changes: [{id, name, args}]}`).
///
/// Each change runs in its own transaction with the record of the last change id applied for that phone, so a change
/// sent again, as after an answer lost on the way, is skipped instead of applied twice. A change the library turns
/// down (a card or space that is gone, a malformed change, a name this server does not know) is recorded as handled
/// and listed in `refused`; the phone takes the server's version. Names: `pin` (`cardId`, `pinned`), `trash`
/// (`cardId`), `note` (`cardId`, `text`, one paragraph per line), `add-tag` (`cardId`, `name`), `remove-tag`
/// (`tagId`), `add-to-space` and `remove-from-space` (`cardId`, `spaceId`).
///
/// # Arguments
/// * `store` - The library the writer writes to.
/// * `writer` - The library's writer.
/// * `body` - The request body.
///
/// # Returns
/// `{lastChangeId, refused, tagIds}`: the highest change id now applied for the phone, the ids of the changes turned
/// down, and the id of each tag an `add-tag` change of this push added, by change id.
///
/// # Errors
/// Status 400 for a body without a client id or a list of at most [`PUSH_LIMIT`] changes, each with a positive whole
/// `id`; the database's error, which leaves the change it hit and the ones after it unapplied.
pub fn push(store: &Store, writer: &Writer, body: &Value) -> Result<Value> {
    let client = body.get("clientId").as_str().filter(|id| !id.is_empty() && id.len() <= 64).ok_or_else(|| bad("bad-client"))?;
    let changes = body.get("changes").as_array().ok_or_else(|| bad("bad-changes"))?;
    if changes.len() > PUSH_LIMIT {
        return Err(bad("too-many-changes"));
    }
    let mut parsed = Vec::with_capacity(changes.len());
    for change in changes {
        let id = change.get("id").as_f64().filter(|id| *id >= 1.0 && number::is_safe_integer(*id)).ok_or_else(|| bad("bad-change"))?;
        parsed.push((id as i64, change.get("name").as_str().unwrap_or(""), change.get("args")));
    }
    let db = store.db();
    db.exec_batch(CLIENTS)?;
    let mut last = last_change(&**db, client)?;
    let (mut turned_down, mut tags) = (Vec::new(), Object::new());
    for (id, name, args) in parsed {
        if id <= last {
            continue;
        }
        match writer.run(|writer| {
            let tag = apply(writer, name, args)?;
            record(&**db, client, id)?;
            Ok(tag)
        }) {
            Ok(tag) => {
                if let Some(tag) = tag {
                    tags.insert(id.to_string(), tag);
                }
            }
            Err(error) if refused(&error) => {
                record(&**db, client, id)?;
                turned_down.push(Value::from(id as f64));
            }
            Err(error) => return Err(error),
        }
        last = id;
    }
    Ok(obj! { "lastChangeId" => last as f64, "refused" => Value::Array(turned_down), "tagIds" => tags }.into())
}
