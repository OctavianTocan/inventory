//! Port of `lib/store-enrich.mjs`: synchronous enrichment mutations that run inside a writer transaction.

use ai_kit::embed;
use mm_js::{Value, json, obj, text};

use crate::ai_tags::curate_tags;
use crate::sql::{SqlExt, SqlValue};
use crate::store::{IdKind, Store, User};
use crate::{Error, Result};

const SCREENSHOT_PENDING: i32 = 2_097_152;
const SCREENSHOT_FAILED: i32 = 4_194_304;
const PROCESSING: i32 = 1;

/// A card loaded for editing: the row's JSON columns decoded, as the writer's `card(id)` returns it.
#[derive(Clone, Debug, PartialEq)]
pub struct EditCard {
    /// Card id.
    pub id: String,
    /// Decoded `data` column.
    pub data: Value,
    /// Decoded `extras` column.
    pub extras: Value,
    /// Grid sort key in nanoseconds since the epoch.
    pub bumped_ns: i128,
}

/// The writer helpers the enricher runs on (`{ card, save, now, emit }`).
pub trait EnrichHooks {
    /// Load a card inside the writer's transaction.
    ///
    /// # Arguments
    /// * `id` - Card id.
    ///
    /// # Errors
    /// Status 404 `Card not found` for an unknown id, or the database's error.
    fn card(&self, id: &str) -> Result<EditCard>;

    /// Store an edited card and its rendered markup, and queue its event.
    ///
    /// # Arguments
    /// * `card` - The edited card; its modification fields are updated.
    /// * `event` - Hub event to queue for the card, or `None` for none.
    ///
    /// # Errors
    /// The writer's error.
    fn save(&self, card: &mut EditCard, event: Option<&str>) -> Result<()>;

    /// The writer's current time in nanoseconds since the epoch.
    ///
    /// # Errors
    /// The writer's error.
    fn now(&self) -> Result<i128>;

    /// Queue a hub notification for delivery after the transaction commits.
    ///
    /// # Arguments
    /// * `target` - Event name.
    /// * `payload` - Event body.
    fn emit(&self, target: &str, payload: Value);
}

/// The enrichment mutations (`createEnricher`'s result).
pub struct Enricher<'a> {
    store: &'a Store,
    user: User,
    hooks: &'a dyn EnrichHooks,
}

pub(crate) fn to_int32(value: f64) -> i32 {
    if value.is_finite() { value.trunc().rem_euclid(4_294_967_296.0) as u32 as i32 } else { 0 }
}

pub(crate) fn update_flags(data: &mut Value, change: impl FnOnce(i32) -> i32) {
    let flags = change(to_int32(data.get("flags").to_number()));
    data.set("flags", f64::from(flags));
}

pub(crate) fn strict_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::BigInt(a), Value::BigInt(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

pub(crate) fn entries(value: &Value) -> Vec<(String, Value)> {
    match value {
        Value::Object(object) => object.iter().map(|(key, item)| (key.into_owned(), item.clone())).collect(),
        Value::Array(items) => items.iter().enumerate().map(|(index, item)| (index.to_string(), item.clone())).collect(),
        _ => Vec::new(),
    }
}

fn same(a: &Value, b: &Value) -> Result<bool> {
    Ok(json::stringify(a)? == json::stringify(b)?)
}

fn member<'v>(value: &'v Value, name: &str) -> Result<&'v Value> {
    if value.is_nullish() {
        return Err(Error::invalid(format!("Cannot read properties of {} (reading '{name}')", value.to_js_string())));
    }
    Ok(value.get(name))
}

fn remove_extra(row: &mut EditCard, name: &str) {
    if let Some(extras) = row.extras.as_object_mut() {
        extras.remove(name);
    }
}

impl<'a> Enricher<'a> {
    /// Bind the enrichment mutations to a store and the writer's helpers (`createEnricher`).
    ///
    /// # Arguments
    /// * `store` - Open store.
    /// * `hooks` - The writer's card, save, now and emit helpers.
    ///
    /// # Errors
    /// An invalid-data error when the store has no user.
    pub fn new(store: &'a Store, hooks: &'a dyn EnrichHooks) -> Result<Self> {
        let user = store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'id')"))?;
        Ok(Self { store, user, hooks })
    }

    /// Merge scraper-owned fields that have not changed since the preceding stage (`enrich`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `patch` - `{data, extras}` fields the scraper produced.
    /// * `baseline` - `{data, extras}` the scraper last saw; updated with every field merged.
    ///
    /// # Returns
    /// `false` when the card is deleted or its source changed, so nothing was merged.
    ///
    /// # Errors
    /// Status 404 `Card not found`, an invalid-data error for a baseline without `data` or `extras`, or the
    /// writer's or database's error.
    pub fn enrich(&self, id: &str, patch: &Value, baseline: &mut Value) -> Result<bool> {
        let hooks = self.hooks;
        let mut row = hooks.card(id)?;
        if row.data.get("deleted").truthy() || !same(row.data.get("source"), member(member(baseline, "data")?, "source")?)? {
            return Ok(false);
        }
        let patch_extras = member(patch, "extras")?;
        let screenshot_started = patch_extras.get("screenshotPending").truthy() && !row.extras.get("screenshotPending").truthy();
        let screenshot_failed = patch_extras.get("screenshotFailed").truthy() && !row.extras.get("screenshotFailed").truthy();
        let title_owned = !row.extras.get("titleEdited").truthy() && same(row.data.get("title"), member(member(baseline, "data")?, "title")?)?;
        let patch_data = patch.get("data");
        for (key, value) in entries(if patch_data.is_nullish() { &Value::Undefined } else { patch_data }) {
            if key == "title" && !title_owned {
                continue;
            }
            if same(row.data.get(&key), member(baseline, "data")?.get(&key))? {
                row.data.set(key.clone(), value.clone());
                set_member(baseline, "data", &key, value);
            }
        }
        for (key, value) in entries(patch_extras) {
            if key == "title" && !title_owned {
                continue;
            }
            if same(row.extras.get(&key), member(member(baseline, "extras")?, &key)?)? {
                row.extras.set(key.clone(), value.clone());
                set_member(baseline, "extras", &key, value);
            }
        }
        if row.data.get("type").as_str() == Some("Article") {
            update_flags(&mut row.data, |flags| flags | 2);
        }
        if screenshot_started {
            update_flags(&mut row.data, |flags| flags | SCREENSHOT_PENDING | PROCESSING);
        }
        if screenshot_failed {
            update_flags(&mut row.data, |flags| flags | SCREENSHOT_FAILED);
            remove_extra(&mut row, "capturePending");
        }
        hooks.save(&mut row, if screenshot_started || screenshot_failed { None } else { Some("object:updated") })?;
        if screenshot_started {
            hooks.emit("object:updated", obj! { "id" => id, "type" => "WebPage", "flags" => f64::from(SCREENSHOT_PENDING), "bumped" => "0001-01-01T00:00:00" }.into());
        }
        if screenshot_failed {
            if !row.extras.get("captureText").truthy() && !row.data.get("description").truthy() && !row.data.get("prose").truthy() {
                hooks.emit("ai:failed", obj! { "objectId" => id }.into());
                hooks.emit("exception", obj! { "id" => id, "type" => "InvalidObjectException", "message" => "no resource text" }.into());
            }
            hooks.emit("screenshot:failed", obj! { "objectId" => id }.into());
        }
        Ok(true)
    }

    /// Clear or set the processing state after capture, a missing key or a recoverable provider error (`processing`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `enabled` - Whether the card is still being processed.
    /// * `capture_done` - Whether capture finished, which clears the pending markers.
    ///
    /// # Errors
    /// Status 404 `Card not found`, or the writer's or database's error.
    pub fn processing(&self, id: &str, enabled: bool, capture_done: bool) -> Result<()> {
        let mut row = self.hooks.card(id)?;
        if row.extras.get("screenshotFailed").truthy() {
            return Ok(());
        }
        update_flags(&mut row.data, |flags| if enabled { flags | PROCESSING } else { flags & !PROCESSING });
        if capture_done {
            remove_extra(&mut row, "capturePending");
            if row.extras.get("screenshotPending").truthy() {
                remove_extra(&mut row, "screenshotPending");
                update_flags(&mut row.data, |flags| flags & !SCREENSHOT_PENDING);
            }
        }
        self.hooks.save(&mut row, Some("object:updated"))
    }

    /// Store one complete AI result without changing manual annotations or resurrecting trash (`analysis`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `result` - `{summary, title, tags, embedding}` from the AI provider.
    ///
    /// # Errors
    /// Status 404 `Card not found`, an invalid-data error unless `embedding` holds 768 finite numbers, or the
    /// writer's or database's error.
    pub fn analysis(&self, id: &str, result: &Value) -> Result<()> {
        let hooks = self.hooks;
        let db = self.store.db();
        let mut row = hooks.card(id)?;
        if row.data.get("deleted").truthy() {
            return Ok(());
        }
        let vector = embedding(member(result, "embedding")?)?;
        let keep_summary = result.get("preserveSummary").truthy() || row.extras.get("summaryEdited").truthy();
        if !keep_summary {
            row.data.set("summary", text::slice(&member(result, "summary")?.to_js_string(), 0, Some(600)));
            if result.get("summarySource").as_str() == Some("ai") {
                row.extras.set("summarySource", "ai");
            }
        }
        let kind = row.data.get("type").clone();
        let retitle = ["XPost", "ThreadsPost", "SubstackNote", "BlueskyPost", "MastodonPost"].iter().any(|name| kind.as_str() == Some(name))
            && !row.extras.get("titleEdited").truthy()
            && !row.extras.get("article").truthy()
            && strict_equal(row.data.get("title"), row.extras.get("capturedTitle"))
            && result.get("title").truthy();
        if retitle {
            let title = headline(&result.get("title").to_js_string());
            row.data.set("title", title.clone());
            row.extras.set("title", title.clone());
            row.extras.set("capturedTitle", title);
        } else if result.get("title").truthy() {
            set_note_name(&mut row, &result.get("title").to_js_string());
        }
        update_flags(&mut row.data, |flags| flags & !PROCESSING);
        let vocabulary: Vec<String> = db
            .query("SELECT name FROM annotations WHERE user_id = ? GROUP BY name ORDER BY COUNT(*) DESC, name", &[self.user.id.as_str().into()])?
            .iter()
            .map(|tag| tag.text("name").unwrap_or("").to_owned())
            .collect();
        db.execute("DELETE FROM annotations WHERE card_id = ? AND (flags & 2) != 0 AND (flags & 8) = 0", &[id.into()])?;
        let existing: Vec<String> = self.store.card_tags(id)?.into_iter().map(|tag| tag.name).collect();
        let mut position = db
            .one("SELECT COALESCE(MAX(position), -1) + 1 AS n FROM annotations WHERE card_id = ?", &[id.into()])?
            .and_then(|found| found.number("n"))
            .unwrap_or(0.0);
        let tags = match member(result, "tags")? {
            Value::Array(items) => items.as_slice(),
            _ => return Err(Error::invalid("tags is not iterable")),
        };
        let kind_name = match &kind {
            Value::String(name) => name.clone(),
            other => other.to_js_string(),
        };
        for tag in curate_tags(tags, &kind_name, &vocabulary, &existing)? {
            let name = text::trim(&tag.get("name").to_js_string()).to_owned();
            let tag_id = self.store.new_id(IdKind::Tag, None)?;
            db.execute(
                "INSERT INTO annotations VALUES (?, ?, ?, ?, 2, ?)",
                &[tag_id.as_str().into(), self.user.id.as_str().into(), id.into(), name.as_str().into(), position.into()],
            )?;
            position += 1.0;
            db.execute(
                "INSERT INTO tag_names (user_id, name, modified_ms) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
                &[self.user.id.as_str().into(), name.as_str().into(), ((hooks.now()? / 1_000_000) as f64).into()],
            )?;
            hooks.emit("tag:created", obj! { "id" => tag_id, "name" => name, "objectId" => id, "flags" => 2 }.into());
        }
        let bytes = embed::encode_vector(&vector);
        db.execute(
            "INSERT OR REPLACE INTO embeddings VALUES (?, ?, 768, ?)",
            &[id.into(), SqlValue::from(self.user.id.as_str()), bytes.into()],
        )?;
        hooks.save(&mut row, Some("object:analyzed"))
    }

    /// Give a note the name the model chose for it, and nothing else (the sweep through notes saved before names were
    /// generated).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `title` - The model's title for the note.
    ///
    /// # Returns
    /// Whether the card took the name: false for a deleted card, a card that is not a note, a note the user named, a
    /// note that has a name of its own, and a title that is empty once cleaned.
    ///
    /// # Errors
    /// Status 404 `Card not found`, or the writer's or database's error.
    pub fn name_note(&self, id: &str, title: &str) -> Result<bool> {
        let mut row = self.hooks.card(id)?;
        if row.data.get("deleted").truthy() || !set_note_name(&mut row, title) {
            return Ok(false);
        }
        self.hooks.save(&mut row, Some("object:updated"))?;
        Ok(true)
    }

    /// Fill only the AI fields a card still lacks. Manual tags and a summary the user wrote stay as they are.
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `fill` - Summary, tags and embedding the model produced. Empty pieces are left untouched. An embedding is
    ///   written only when the card has none. Tags are added beside names already on the card, never replacing them.
    ///
    /// # Errors
    /// Status 404 `Card not found`, an invalid embedding when one is supplied, or the writer's or database's error.
    pub fn fill_gaps(&self, id: &str, fill: &GapFill) -> Result<GapApplied> {
        let hooks = self.hooks;
        let db = self.store.db();
        let mut row = hooks.card(id)?;
        if row.data.get("deleted").truthy() {
            return Ok(GapApplied::default());
        }
        let mut applied = GapApplied::default();
        let owned = row.extras.get("summaryEdited").truthy() || row.data.get("summary").truthy();
        if let Some(summary) = fill.summary.as_ref().filter(|_| !owned) {
            row.data.set("summary", text::slice(summary, 0, Some(600)));
            row.extras.set("summarySource", "ai");
            applied.summarized = true;
        }
        update_flags(&mut row.data, |flags| flags & !PROCESSING);
        if fill.mark_tags_done {
            row.extras.set("aiTagsDone", true);
        }
        let existing = self.store.card_tags(id)?;
        let names: Vec<String> = existing.iter().map(|tag| tag.name.clone()).collect();
        if !fill.tags.is_empty() {
            let vocabulary: Vec<String> = db
                .query("SELECT name FROM annotations WHERE user_id = ? GROUP BY name ORDER BY COUNT(*) DESC, name", &[self.user.id.as_str().into()])?
                .iter()
                .map(|tag| tag.text("name").unwrap_or("").to_owned())
                .collect();
            let kind = match row.data.get("type") {
                Value::String(name) => name.clone(),
                other => other.to_js_string(),
            };
            let mut position = db
                .one("SELECT COALESCE(MAX(position), -1) + 1 AS n FROM annotations WHERE card_id = ?", &[id.into()])?
                .and_then(|found| found.number("n"))
                .unwrap_or(0.0);
            for tag in curate_tags(&fill.tags, &kind, &vocabulary, &names)? {
                let name = text::trim(&tag.get("name").to_js_string()).to_owned();
                let tag_id = self.store.new_id(IdKind::Tag, None)?;
                db.execute(
                    "INSERT INTO annotations VALUES (?, ?, ?, ?, 2, ?)",
                    &[tag_id.as_str().into(), self.user.id.as_str().into(), id.into(), name.as_str().into(), position.into()],
                )?;
                position += 1.0;
                db.execute(
                    "INSERT INTO tag_names (user_id, name, modified_ms) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
                    &[self.user.id.as_str().into(), name.as_str().into(), ((hooks.now()? / 1_000_000) as f64).into()],
                )?;
                hooks.emit("tag:created", obj! { "id" => tag_id, "name" => name, "objectId" => id, "flags" => 2 }.into());
                applied.tagged += 1;
            }
        }
        if let Some(vector) = &fill.embedding {
            let numbers = embedding(&Value::Array(vector.iter().map(|value| Value::from(*value as f64)).collect()))?;
            let missing = db.one("SELECT 1 AS n FROM embeddings WHERE card_id = ?", &[id.into()])?.is_none();
            if missing {
                let bytes = embed::encode_vector(&numbers);
                db.execute("INSERT INTO embeddings VALUES (?, ?, 768, ?)", &[id.into(), SqlValue::from(self.user.id.as_str()), bytes.into()])?;
                applied.embedded = true;
            }
        }
        if !row.extras.get("id").truthy() && !row.data.get("id").truthy() {
            row.extras.set("id", id);
        }
        let changed = applied.summarized || applied.tagged > 0 || applied.embedded || fill.mark_tags_done;
        if changed {
            hooks.save(&mut row, Some(if applied.summarized || applied.tagged > 0 || applied.embedded { "object:analyzed" } else { "object:updated" }))?;
        }
        Ok(applied)
    }
}

/// Pieces of an AI result that [`Enricher::fill_gaps`] may write.
pub struct GapFill {
    /// Summary to store when the card has none and the user has not edited one.
    pub summary: Option<String>,
    /// Candidate tag records (`name`, and optional `p` and `source`).
    pub tags: Vec<Value>,
    /// Document vector to store when the card has none.
    pub embedding: Option<Vec<f32>>,
    /// Record that tag generation was attempted, including when it added nothing.
    pub mark_tags_done: bool,
}

/// What [`Enricher::fill_gaps`] wrote.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GapApplied {
    /// A summary was written.
    pub summarized: bool,
    /// AI tags inserted.
    pub tagged: usize,
    /// An embedding was inserted.
    pub embedded: bool,
}

fn set_member(root: &mut Value, group: &str, key: &str, value: Value) {
    if let Some(target) = root.get_mut(group) {
        target.set(key, value);
    }
}

/// Card types the app shows as a note, which the model names while the user has not.
pub const NAMED_NOTE_TYPES: [&str; 2] = ["Note", "Content"];

fn blank(value: &Value) -> bool {
    value.is_nullish() || text::trim(&value.to_js_string()).is_empty()
}

/// Whether the model may name this card: a note the user has not named, with no name of its own or only one the
/// model gave it, so a later analysis of edited words may rename it.
///
/// # Arguments
/// * `data` - The card's `data`.
/// * `extras` - The card's `extras`.
pub fn note_needs_name(data: &Value, extras: &Value) -> bool {
    NAMED_NOTE_TYPES.iter().any(|name| data.get("type").as_str() == Some(name))
        && !extras.get("titleEdited").truthy()
        && (extras.get("titleSource").as_str() == Some("ai") || (blank(data.get("title")) && blank(extras.get("title"))))
}

// Name a note from the model's title, without quotes or a closing full stop; false when it may not be named.
fn set_note_name(row: &mut EditCard, title: &str) -> bool {
    if !note_needs_name(&row.data, &row.extras) {
        return false;
    }
    let name = headline(title);
    let name = text::trim(name.trim_matches(|c| matches!(c, '"' | '\'' | '“' | '”' | '‘' | '’')).trim_end_matches('.')).to_owned();
    if name.is_empty() {
        return false;
    }
    row.data.set("title", name);
    row.extras.set("titleSource", "ai");
    true
}

fn headline(title: &str) -> String {
    let trimmed = text::trim(title);
    let mut words: Vec<&str> = Vec::new();
    let mut start = None;
    for (index, character) in trimmed.char_indices() {
        match (text::is_whitespace(character), start) {
            (false, None) => start = Some(index),
            (true, Some(begin)) => {
                words.push(&trimmed[begin..index]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start {
        words.push(&trimmed[begin..]);
    }
    if words.is_empty() {
        words.push("");
    }
    words.truncate(8);
    words.join(" ")
}

fn embedding(value: &Value) -> Result<Vec<f32>> {
    let numbers: Vec<f32> = match value {
        Value::Array(items) => items.iter().map(|item| item.to_number() as f32).collect(),
        Value::Bytes(bytes) => bytes.iter().map(|byte| f32::from(*byte)).collect(),
        _ => return Err(Error::invalid("Invalid embedding")),
    };
    if numbers.len() != embed::DIMENSIONS || !numbers.iter().all(|number| number.is_finite()) {
        return Err(Error::invalid("Invalid embedding"));
    }
    Ok(numbers)
}

