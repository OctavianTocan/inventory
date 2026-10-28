//! The app's push (`POST /app/v1/push`): changes made on a phone, each applied once and in the order the phone made
//! them, as in the Replicache and Zero push protocol.

use std::borrow::Cow;

use mm_js::{Object, Value, json, number, obj, text as js_text};

use crate::app_api::prose_text;
use crate::sql::{Sql, SqlExt};
use crate::store::Store;
use crate::store_write::{Writer, text_document};
use crate::{Error, Result};

/// Most changes one push may carry.
pub const PUSH_LIMIT: usize = 100;

const TABLES: &str = "CREATE TABLE IF NOT EXISTS app_clients (client_id TEXT PRIMARY KEY, last_change INTEGER NOT NULL) STRICT;
CREATE TABLE IF NOT EXISTS app_aliases (phone_id TEXT PRIMARY KEY, card_id TEXT NOT NULL) STRICT";

/// What a push did.
pub struct Pushed {
    /// The phone's answer: `{lastChangeId, refused, tagIds, cardIds}`, as [`push`] describes.
    pub answer: Value,
    /// The cards the push created, for the caller to schedule their enrichment as after `POST /objects`.
    pub created: Vec<String>,
}

// What one change made that the phone needs to know.
enum Made {
    Tag(String),
    Card { id: String, created: bool },
}

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

// An id a phone made: 32 lowercase hex digits, a UUID without its dashes. Letters and digits only, like the library's
// own ids, and its letters stop at `f`, so it can never hold the kind code (`gl`, `0O`, …) that those ids carry and
// `Store::new_id` counts from.
fn phone_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

// The change's card: a card the phone made that turned out to be a link already saved stands for that saved card.
fn card<'a>(db: &dyn Sql, args: &'a Value) -> Result<Cow<'a, str>> {
    let id = text(args, "cardId")?;
    if !phone_id(id) {
        return Ok(id.into());
    }
    let saved = db.one("SELECT card_id FROM app_aliases WHERE phone_id = ?", &[id.into()])?.and_then(|row| row.text("card_id").map(str::to_owned));
    Ok(saved.map_or(id.into(), Into::into))
}

fn has_note(store: &Store, id: &str) -> Result<bool> {
    let Some(row) = store.card(id, None)? else { return Ok(false) };
    let note = json::parse(&row.data)?.get("note").get("prose").clone();
    Ok(note.truthy() && !js_text::trim(&prose_text(&note)?).is_empty())
}

// Saves a link or text under the id the phone made. A link already saved is bumped instead, as `POST /objects` does,
// and the phone's id becomes another name for it so the phone's later changes reach it; it gets the note only when it
// has none, so a link saved before keeps its own.
fn create_card(store: &Store, writer: &Writer, args: &Value) -> Result<Made> {
    let id = text(args, "cardId")?;
    if !phone_id(id) {
        return Err(bad("bad-change"));
    }
    let body = obj! { "content" => text(args, "content")?, "source" => args.get("source").clone() }.into();
    let made = writer.create_as(&body, &Value::Undefined, Some(id))?;
    let kept = made.get("body").get("id").to_js_string();
    let created = made.get("status").as_f64() == Some(201.0);
    if !created {
        store.db().execute("INSERT INTO app_aliases (phone_id, card_id) VALUES (?, ?)", &[id.into(), kept.as_str().into()])?;
    }
    let note = args.get("note");
    if note.as_str().is_some_and(|words| !js_text::trim(words).is_empty()) && (created || !has_note(store, &kept)?) {
        writer.prose(&kept, &text_document(note)?, "note")?;
    }
    Ok(Made::Card { id: kept, created })
}

// One change through the same writer calls as the web routes it stands for.
fn apply(store: &Store, writer: &Writer, name: &str, args: &Value) -> Result<Option<Made>> {
    let db = store.db();
    match name {
        "pin" => {
            let pinned = args.get("pinned").as_bool().ok_or_else(|| bad("bad-change"))?;
            writer.flag(&card(&**db, args)?, 512, pinned, &Value::Undefined)?;
        }
        "trash" => {
            writer.trash(&card(&**db, args)?, false)?;
        }
        "note" => {
            let note = args.get("text");
            if !note.is_string() {
                return Err(bad("bad-change"));
            }
            writer.prose(&card(&**db, args)?, &text_document(note)?, "note")?;
        }
        "add-tag" => return Ok(writer.add_tag(&card(&**db, args)?, args.get("name"))?.get("id").as_str().map(|id| Made::Tag(id.to_owned()))),
        "remove-tag" => {
            writer.remove_tag(text(args, "tagId")?)?;
        }
        "add-to-space" | "remove-from-space" => {
            writer.membership(text(args, "spaceId")?, &card(&**db, args)?, name == "remove-from-space")?;
        }
        "create-space" => {
            let id = text(args, "spaceId")?;
            if !phone_id(id) {
                return Err(bad("bad-change"));
            }
            writer.create_space_as(&obj! { "name" => args.get("name").clone(), "color" => args.get("color").clone() }.into(), Some(id))?;
        }
        "delete-space" => {
            writer.delete_space(text(args, "spaceId")?)?;
        }
        "create-card" => return create_card(store, writer, args).map(Some),
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
/// (`tagId`), `add-to-space` and `remove-from-space` (`cardId`, `spaceId`), `create-space` (`spaceId`, 32 lowercase
/// hex digits the phone made, `name`, `color`), `delete-space` (`spaceId`) and `create-card` (`cardId` made the same
/// way, `content`, `source`, `note`).
///
/// # Arguments
/// * `store` - The library the writer writes to.
/// * `writer` - The library's writer.
/// * `body` - The request body.
///
/// # Returns
/// The answer `{lastChangeId, refused, tagIds, cardIds}`: the highest change id now applied for the phone, the ids of
/// the changes turned down, and by change id the id of each tag an `add-tag` change added and the id the library keeps
/// for each card a `create-card` change saved (the phone's own, or that of a link already saved); and the cards the
/// push created.
///
/// # Errors
/// Status 400 for a body without a client id or a list of at most [`PUSH_LIMIT`] changes, each with a positive whole
/// `id`; the database's error, which leaves the change it hit and the ones after it unapplied.
pub fn push(store: &Store, writer: &Writer, body: &Value) -> Result<Pushed> {
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
    db.exec_batch(TABLES)?;
    let mut last = last_change(&**db, client)?;
    let (mut turned_down, mut tags, mut cards, mut created) = (Vec::new(), Object::new(), Object::new(), Vec::new());
    for (id, name, args) in parsed {
        if id <= last {
            continue;
        }
        match writer.run(|writer| {
            let made = apply(store, writer, name, args)?;
            record(&**db, client, id)?;
            Ok(made)
        }) {
            Ok(Some(Made::Tag(tag))) => {
                tags.insert(id.to_string(), tag);
            }
            Ok(Some(Made::Card { id: card, created: new })) => {
                if new {
                    created.push(card.clone());
                }
                cards.insert(id.to_string(), card);
            }
            Ok(None) => {}
            Err(error) if refused(&error) => {
                record(&**db, client, id)?;
                turned_down.push(Value::from(id as f64));
            }
            Err(error) => return Err(error),
        }
        last = id;
    }
    let answer = obj! { "lastChangeId" => last as f64, "refused" => Value::Array(turned_down), "tagIds" => tags, "cardIds" => cards }.into();
    Ok(Pushed { answer, created })
}
