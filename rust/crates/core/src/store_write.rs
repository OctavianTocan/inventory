//! Port of `lib/store-write.mjs`: the mutations of the library (cards, tags, pins, spaces, settings, shares).

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::LazyLock;

use mm_js::{Object, Value, base64, date, json, obj, text, url};
use regex::Regex;

use crate::capture_html::source_url;
use crate::render_card::render_card;
use crate::runtime;
use crate::search::Search;
use crate::sql::{SqlExt, SqlValue};
use crate::store::{IdKind, Store, User, iso_from_ns, ns_from_iso};
use crate::store_enrich::{EditCard, EnrichHooks, Enricher, strict_equal, to_int32, update_flags};
use crate::store_query::{QueryCheck, QueryWriter, check_query};
use crate::{Error, Result};

const PRIVACY_TAGS: [&str; 3] = ["private", "nsfw", "hidden"];
const MS_NS: i128 = 1_000_000;
const DAY_NS: i128 = 86_400_000_000_000;
const JS_SPACE: &str = r"\t\n\x{b}\x{c}\r \x{a0}\x{1680}\x{2000}-\x{200a}\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}\x{feff}";

static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"^https?://[^{JS_SPACE}]+$")).expect("pattern"));

fn fail(status: u16, detail: &str) -> Error {
    Error::status(status, detail)
}

fn has(body: &Value, key: &str) -> Result<bool> {
    match body {
        Value::Undefined | Value::Null => Err(Error::invalid("Cannot convert undefined or null to object")),
        Value::Object(object) => Ok(object.contains_key(key)),
        _ => Ok(false),
    }
}

fn field<'v>(body: &'v Value, key: &str) -> Result<&'v Value> {
    if body.is_nullish() { Err(Error::invalid(format!("Cannot read properties of null (reading '{key}')"))) } else { Ok(body.get(key)) }
}

fn bind(value: &Value) -> Result<SqlValue> {
    Ok(match value {
        Value::String(text) => SqlValue::Text(text.clone()),
        Value::Number(number) => SqlValue::Real(*number),
        Value::Bool(flag) => SqlValue::Integer(i64::from(*flag)),
        Value::Null => SqlValue::Null,
        Value::BigInt(number) => SqlValue::big(*number),
        _ => return Err(Error::invalid("Provided value cannot be bound to SQLite parameter")),
    })
}

fn stringify(value: &Value) -> Result<String> {
    json::stringify(value)?.ok_or_else(|| Error::invalid("Provided value cannot be bound to SQLite parameter"))
}

// A prose document with one paragraph per line of a text.
pub(crate) fn text_document(content: &Value) -> Result<Value> {
    let Some(source) = content.as_str() else { return Err(Error::invalid("text.split is not a function")) };
    let lines = source.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<Value> = lines
        .split('\n')
        .map(|line| {
            let mut paragraph = obj! { "type" => "paragraph" };
            if !line.is_empty() {
                paragraph.insert("content", vec![Value::from(obj! { "type" => "text", "text" => line })]);
            }
            paragraph.into()
        })
        .collect();
    Ok(obj! { "type" => "doc", "content" => paragraphs }.into())
}

fn document_body(body: &Value) -> Result<&Value> {
    let content = body.get("content");
    if !body.truthy() || body.get("type").as_str() != Some("doc") || (!content.is_undefined() && !content.is_array()) {
        return Err(fail(400, "Expected a prose document"));
    }
    Ok(body)
}

fn has_content(node: &Value) -> Result<bool> {
    if node.is_nullish() {
        return Err(Error::invalid("Cannot read properties of null (reading 'text')"));
    }
    match node.get("text") {
        Value::Undefined | Value::Null => {}
        Value::String(source) if !text::trim(source).is_empty() => return Ok(true),
        Value::String(_) => {}
        _ => return Err(Error::invalid("node.text?.trim is not a function")),
    }
    if matches!(node.get("type").as_str(), Some("image" | "wikiLink")) {
        return Ok(true);
    }
    match node.get("content") {
        Value::Undefined | Value::Null => Ok(false),
        Value::Array(items) => {
            for item in items {
                if has_content(item)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Err(Error::invalid("node.content?.some is not a function")),
    }
}

fn clear_bit(data: &mut Value, bit: i32) {
    update_flags(data, |flags| flags & !bit);
}

fn set_bit(data: &mut Value, bit: i32) {
    update_flags(data, |flags| flags | bit);
}

fn id_text(value: &Value) -> String {
    value.to_js_string()
}

type Push = Box<dyn Fn(&str, &Value)>;

/// The mutations over an open store (`createWriter`'s result). Call mutations only inside [`Writer::run`].
pub struct Writer {
    store: Rc<Store>,
    user: User,
    search: Search,
    clock: Cell<i128>,
    events: RefCell<Option<Vec<(String, Value)>>>,
    push: Push,
    random: fn(usize) -> Vec<u8>,
}

impl Writer {
    /// Create the writer (`createWriter`).
    ///
    /// # Arguments
    /// * `store` - Open store.
    /// * `push` - Broadcasts a committed hub event: `(target, payload)`.
    ///
    /// # Errors
    /// An internal error when the store has no user, or the database's error.
    pub fn new(store: Rc<Store>, push: impl Fn(&str, &Value) + 'static) -> Result<Self> {
        let user = store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'id')"))?;
        let clock = store.checkpoint(None)?.parse::<i128>().map_err(|_| Error::invalid("Cannot convert checkpoint to a BigInt"))?;
        Ok(Self {
            search: Search::new(store.clone(), None, false)?,
            store,
            user,
            clock: Cell::new(clock),
            events: RefCell::new(None),
            push: Box::new(push),
            random: runtime::random_bytes,
        })
    }

    /// Replace the random source behind revision and share ids.
    ///
    /// # Arguments
    /// * `random` - Returns the requested number of random bytes.
    pub fn with_random(mut self, random: fn(usize) -> Vec<u8>) -> Self {
        self.random = random;
        self
    }

    fn tick(&self) -> Result<i128> {
        let checkpoint = self.store.checkpoint(None)?.parse::<i128>().map_err(|_| Error::invalid("Cannot convert checkpoint to a BigInt"))?;
        if checkpoint > self.clock.get() {
            self.clock.set(checkpoint);
        }
        let wall = (runtime::now_ms() as i128) * MS_NS;
        self.clock.set(if wall > self.clock.get() { wall } else { self.clock.get() + MS_NS });
        Ok(self.clock.get())
    }

    fn queue(&self, target: &str, payload: Value) {
        if let Some(events) = self.events.borrow_mut().as_mut() {
            events.push((target.to_owned(), payload));
        }
    }

    fn change(&self, id: &str, object_type: i64, kind: i64, ts: i128) -> Result<()> {
        self.store.db().execute(
            "INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, ?, ?, ?)",
            &[self.user.id.as_str().into(), SqlValue::big(ts), (object_type as f64).into(), id.into(), (kind as f64).into()],
        )?;
        Ok(())
    }

    fn load(&self, id: &str) -> Result<EditCard> {
        let row = self.store.card(id, None)?.ok_or_else(|| fail(404, "Card not found"))?;
        Ok(EditCard { id: row.id, data: json::parse(&row.data)?, extras: json::parse(&row.extras)?, bumped_ns: row.bumped_ns })
    }

    fn space(&self, id: &str) -> Result<crate::store::Space> {
        let found = self.store.space(id)?;
        let owned = self.store.db().one("SELECT id FROM spaces WHERE id = ? AND user_id = ?", &[id.into(), self.user.id.as_str().into()])?;
        match found {
            Some(space) if owned.is_some() => Ok(space),
            _ => Err(fail(404, "Space not found")),
        }
    }

    fn store_card(&self, card: &mut EditCard, event: Option<&str>, kind: Option<i64>) -> Result<()> {
        let ts = self.tick()?;
        card.data.set("modified", iso_from_ns(ts, 6)?);
        card.extras.set("local", true);
        let tags = self.store.card_tags(&card.id)?;
        if tags.is_empty() {
            card.data.remove("tags");
        } else {
            card.data.set("tags", tags.iter().map(|tag| tag.to_value()).collect::<Vec<_>>());
        }
        let html = render_card(&card.data, &card.extras)?;
        let deleted = card.data.get("deleted").truthy();
        let deleted_ns = if deleted { SqlValue::big(ns_from_iso(&card.data.get("deleted").to_js_string())?) } else { SqlValue::Null };
        self.store.db().execute(
            "UPDATE cards SET type = ?, flags = ?, data = ?, html = ?, extras = ?,
      bumped_ns = ?, modified_ns = ?, deleted_ns = ? WHERE id = ? AND user_id = ?",
            &[
                bind(card.data.get("type"))?,
                bind(card.data.get("flags"))?,
                stringify(&card.data)?.into(),
                html.into(),
                stringify(&card.extras)?.into(),
                SqlValue::big(card.bumped_ns),
                SqlValue::big(ts),
                deleted_ns,
                card.id.as_str().into(),
                self.user.id.as_str().into(),
            ],
        )?;
        self.change(&card.id, 9, kind.unwrap_or(if deleted { 2 } else { 1 }), ts)?;
        if let Some(event) = event {
            self.queue(event, obj! { "id" => card.id.as_str() }.into());
        }
        Ok(())
    }

    fn save_default(&self, card: &mut EditCard) -> Result<()> {
        self.store_card(card, Some("object:updated"), None)
    }

    fn store_space(&self, id: &str, event: &str, kind: i64) -> Result<()> {
        let ts = self.tick()?;
        self.change(id, 190, kind, ts)?;
        self.queue(event, obj! { "id" => id }.into());
        Ok(())
    }

    fn revision(&self, id: &str, prose: &Value) -> Result<()> {
        let revision = base64::hex(&(self.random)(16));
        let created = iso_from_ns(self.tick()?, 6)?;
        self.store.db().execute("INSERT INTO revisions VALUES (?, ?, ?, ?)", &[revision.into(), id.into(), created.into(), stringify(prose)?.into()])?;
        Ok(())
    }

    fn has_content_row(&self, id: &str) -> Result<bool> {
        Ok(self.store.db().one("SELECT card_id FROM card_content WHERE card_id = ?", &[id.into()])?.is_some())
    }

    fn preserve_body(&self, card: &EditCard) -> Result<()> {
        let db = self.store.db();
        if db.one("SELECT id FROM revisions WHERE card_id = ? LIMIT 1", &[card.id.as_str().into()])?.is_some() {
            return Ok(());
        }
        let before = match db.one("SELECT body FROM card_content WHERE card_id = ?", &[card.id.as_str().into()])? {
            Some(content) => json::parse(content.text("body").unwrap_or(""))?,
            None => card.data.get("prose").clone(),
        };
        if before.truthy() { self.revision(&card.id, &before) } else { Ok(()) }
    }

    fn add_to_space(&self, space_id: &str, id: &str) -> Result<()> {
        self.space(space_id)?;
        self.load(id)?;
        let db = self.store.db();
        let rank = db
            .one("SELECT COALESCE(MAX(rank), -1) + 1 AS n FROM space_cards WHERE space_id = ?", &[space_id.into()])?
            .and_then(|row| row.number("n"))
            .unwrap_or(0.0);
        db.execute(
            "INSERT INTO space_cards (space_id, card_id, user_id, rank, score, excluded)
      VALUES (?, ?, ?, ?, 1, 0) ON CONFLICT(space_id, card_id) DO UPDATE SET excluded = 0",
            &[space_id.into(), id.into(), self.user.id.as_str().into(), rank.into()],
        )?;
        self.store_space(space_id, "space:updated", 1)?;
        let mut card = self.load(id)?;
        self.save_default(&mut card)
    }

    fn space_id_of(value: &Value) -> Result<String> {
        value.as_str().map(str::to_owned).ok_or_else(|| Error::invalid("Space id must be a string"))
    }

    fn write_settings(&self, settings: &Value) -> Result<()> {
        self.store.db().execute("UPDATE users SET settings = ? WHERE id = ?", &[stringify(settings)?.into(), self.user.id.as_str().into()])?;
        Ok(())
    }

    fn current_user(&self) -> Result<User> {
        self.store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'settings')"))
    }

    /// Run a group of mutations in one transaction, then broadcast the events they queued (`run`).
    ///
    /// # Arguments
    /// * `work` - The mutations, called with this writer.
    ///
    /// # Returns
    /// What `work` returned.
    ///
    /// # Errors
    /// The work's or the database's error; nothing is broadcast then.
    pub fn run<T>(&self, work: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        *self.events.borrow_mut() = Some(Vec::new());
        let result = self.store.transaction(|| work(self))?;
        let committed = self.events.borrow_mut().take().unwrap_or_default();
        for (target, payload) in &committed {
            (self.push)(target, payload);
        }
        Ok(result)
    }

    /// Run the saved-query refresh (`checkQuery`).
    ///
    /// # Arguments
    /// * `query_id` - Id of the saved query.
    /// * `persist` - Write the new membership; `false` only computes it.
    ///
    /// # Errors
    /// Status 404 `Query not found`, or the database's or search's error.
    pub fn check_query(&self, query_id: &str, persist: bool) -> Result<QueryCheck> {
        check_query(&self.store, &self.search, self, query_id, persist)
    }

    /// Apply a client-side enrichment patch to a card (`enrich`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `patch` - Fields to merge.
    /// * `baseline` - Previously applied fields, updated in place.
    ///
    /// # Returns
    /// Whether the card changed.
    ///
    /// # Errors
    /// The enricher's error.
    pub fn enrich(&self, id: &str, patch: &Value, baseline: &mut Value) -> Result<bool> {
        Enricher::new(&self.store, self)?.enrich(id, patch, baseline)
    }

    /// Set the processing state of a card (`processing`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `enabled` - Whether processing is on.
    /// * `capture_done` - Whether the capture finished.
    ///
    /// # Errors
    /// The enricher's error.
    pub fn processing(&self, id: &str, enabled: bool, capture_done: bool) -> Result<()> {
        Enricher::new(&self.store, self)?.processing(id, enabled, capture_done)
    }

    /// Store the AI analysis of a card (`analysis`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `result` - The analysis `{title, tags, embedding, ...}`. `preserveSummary` keeps the card's summary.
    ///
    /// # Errors
    /// The enricher's error.
    pub fn analysis(&self, id: &str, result: &Value) -> Result<()> {
        Enricher::new(&self.store, self)?.analysis(id, result)
    }

    /// Give a note the model's name for it (`Enricher::name_note`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `title` - The model's title.
    ///
    /// # Returns
    /// Whether the note took the name.
    ///
    /// # Errors
    /// The enricher's error.
    pub fn name_note(&self, id: &str, title: &str) -> Result<bool> {
        Enricher::new(&self.store, self)?.name_note(id, title)
    }

    /// Fill missing AI fields without replacing a summary or tags already on the card.
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `fill` - The pieces to add.
    ///
    /// # Errors
    /// The enricher's error.
    pub fn fill_gaps(&self, id: &str, fill: &crate::store_enrich::GapFill) -> Result<crate::store_enrich::GapApplied> {
        Enricher::new(&self.store, self)?.fill_gaps(id, fill)
    }

    /// Save a note, plain text or URL, bumping an existing URL when present (`create`).
    /// Explicit Content keeps selected text as prose and its string source as `source.url`.
    ///
    /// # Arguments
    /// * `body` - Request body: `content`, `type`, `prose`, `title`, `source`, `spaceId`, `uploadId`, `isDraft`, `isStub`.
    /// * `media` - Completed upload `{id, data, extras}`, or `Value::Undefined`.
    ///
    /// # Returns
    /// `{status, body: {id}}`.
    ///
    /// # Errors
    /// Status 400 for a missing upload, an invalid URL or an invalid document; the database's error.
    pub fn create(&self, body: &Value, media: &Value) -> Result<Value> {
        self.create_as(body, media, None)
    }

    /// Create a card as [`Writer::create`] does, under an id the caller chose, such as one a phone made offline.
    ///
    /// # Arguments
    /// * `body` - As for [`Writer::create`].
    /// * `media` - As for [`Writer::create`].
    /// * `id` - The new card's id, or `None` for the upload's id or the next id of the library's own.
    ///
    /// # Returns
    /// `{status, body: {id}}`: status 201 for a new card, 200 with the existing card's id for a link already saved.
    ///
    /// # Errors
    /// Status 409 `Card already exists` for a chosen id in use; otherwise as [`Writer::create`].
    pub fn create_as(&self, body: &Value, media: &Value, id: Option<&str>) -> Result<Value> {
        if (field(body, "uploadId")?.truthy() || body.get("type").as_str() == Some("Media")) && !media.truthy() {
            return Err(fail(400, "A completed upload is required"));
        }
        let db = self.store.db();
        let content = body.get("content");
        let mut parsed = None;
        let is_content = body.get("type").as_str() == Some("Content");
        if !is_content && let Some(source) = content.as_str().map(text::trim).filter(|trimmed| LINK.is_match(trimmed)) {
            parsed = Some(url::parse(source, None).ok_or_else(|| fail(400, "Invalid URL"))?);
        }
        if let Some(address) = &parsed {
            let rows = db.query(
                "SELECT id, json_extract(data, '$.source.url') AS url FROM cards
          WHERE user_id = ? AND deleted_ns IS NULL AND json_extract(data, '$.source.url') IS NOT NULL
          ORDER BY bumped_ns DESC",
                &[self.user.id.as_str().into()],
            )?;
            let existing = rows.iter().find(|row| row.text("url").and_then(|href| url::parse(href, None)).is_some_and(|known| known.as_str() == address.as_str()));
            if let Some(existing) = existing {
                let mut row = self.load(existing.text("id").unwrap_or(""))?;
                row.bumped_ns = self.tick()?;
                row.data.set("bumped", iso_from_ns(row.bumped_ns, 0)?);
                self.store_card(&mut row, Some("object:bumped"), None)?;
                if body.get("spaceId").truthy() {
                    self.add_to_space(&Self::space_id_of(body.get("spaceId"))?, &row.id)?;
                }
                return Ok(obj! { "status" => 200.0, "body" => obj! { "id" => row.id.as_str() } }.into());
            }
        }
        let id = match (id, media.get("id")) {
            (Some(chosen), _) if db.one("SELECT 1 FROM cards WHERE id = ?", &[chosen.into()])?.is_some() => return Err(fail(409, "Card already exists")),
            (Some(chosen), _) => chosen.to_owned(),
            (None, Value::Undefined | Value::Null) => self.store.new_id(IdKind::Card, None)?,
            (None, other) => other.to_js_string(),
        };
        let ts = self.tick()?;
        let screenshot = body.get("type").as_str() == Some("Screenshot");
        let kind = if parsed.is_some() { if screenshot { "Screenshot" } else { "Placeholder" } } else if is_content { "Content" } else { "Note" };
        let mut data: Value = obj! {
            "type" => kind,
            "flags" => if parsed.is_some() && screenshot { 1.0 } else { 0.0 },
            "modified" => iso_from_ns(ts, 6)?,
            "bumped" => iso_from_ns(ts, 0)?,
            "created" => iso_from_ns(ts, 0)?,
        }
        .into();
        if let Some(address) = &parsed {
            if screenshot {
                data.set("title", url::host(address));
            }
            let hostname = address.host_str().unwrap_or("").to_owned();
            data.set("domain", hostname.as_str());
            data.set("siteName", hostname.as_str());
            let trimmed = text::trim(content.as_str().unwrap_or(""));
            let name = match body.get("source") {
                Value::Undefined | Value::Null => Value::from("clipboard"),
                other => other.clone(),
            };
            let link = source_url(&Value::from(trimmed), trimmed).unwrap_or_else(|| address.as_str().to_owned());
            data.set("source", obj! { "name" => name, "url" => link });
        } else if media.truthy() {
            if let Value::Object(assigned) = media.get("data") {
                for (key, item) in assigned.iter() {
                    data.set(key.into_owned(), item.clone());
                }
            }
        } else {
            let prose = match body.get("prose") {
                Value::Undefined | Value::Null => text_document(&if content.is_nullish() { Value::from("") } else { content.clone() })?,
                other => other.clone(),
            };
            document_body(&prose)?;
            data.set("prose", prose);
            if is_content && let Some(source) = body.get("source").as_str().filter(|source| !source.is_empty()) {
                data.set("source", obj! { "url" => source });
            }
        }
        if media.truthy() && body.get("source").truthy() {
            data.set("source", obj! { "name" => body.get("source").to_js_string() });
        }
        if !body.get("title").is_undefined() {
            data.set("title", body.get("title").to_js_string());
        }
        if body.get("isDraft").truthy() {
            set_bit(&mut data, 131_072);
        }
        if body.get("isStub").truthy() {
            set_bit(&mut data, 262_144);
        }
        let mut extras: Value = obj! { "id" => id.as_str(), "local" => true }.into();
        if parsed.is_some() {
            extras.set("capturePending", true);
        }
        if !body.get("title").is_undefined() {
            extras.set("titleEdited", true);
        }
        if let Value::Object(spread) = media.get("extras") {
            for (key, item) in spread.iter() {
                extras.set(key.into_owned(), item.clone());
            }
        }
        let html = render_card(&data, &extras)?;
        db.execute(
            "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                id.as_str().into(),
                self.user.id.as_str().into(),
                bind(data.get("type"))?,
                bind(data.get("flags"))?,
                stringify(&data)?.into(),
                html.into(),
                stringify(&extras)?.into(),
                SqlValue::big(ts),
                SqlValue::big(ts),
                SqlValue::big(ts),
            ],
        )?;
        self.change(&id, 9, 1, ts)?;
        self.queue("object:created", obj! { "id" => id.as_str() }.into());
        if data.get("prose").truthy() {
            self.revision(&id, data.get("prose"))?;
        }
        if body.get("spaceId").truthy() {
            self.add_to_space(&Self::space_id_of(body.get("spaceId"))?, &id)?;
        }
        Ok(obj! { "status" => 201.0, "body" => obj! { "id" => id.as_str() } }.into())
    }

    /// Apply editable card fields without changing the card's bump time (`patch`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `body` - Fields among `title`, `summary`, `description`, `flags`, `prose`, `source`, `siteName`, `domain`, `ocr`, `author`, `brand`, `format`, `objectNames`. A `summary` is kept as the user's own.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown card; status 400 for invalid flags or prose; the database's error.
    pub fn patch(&self, id: &str, body: &Value) -> Result<Value> {
        let mut row = self.load(id)?;
        if has(body, "flags")? {
            let flags = body.get("flags");
            let valid = matches!(flags, Value::Number(number) if number.is_finite() && number.fract() == 0.0 && *number >= 0.0);
            if !valid {
                return Err(fail(400, "Invalid card flags"));
            }
        }
        if has(body, "prose")? {
            document_body(body.get("prose"))?;
            self.preserve_body(&row)?;
            self.revision(id, body.get("prose"))?;
            if self.has_content_row(id)? {
                self.store.db().execute("UPDATE card_content SET body = ? WHERE card_id = ?", &[stringify(body.get("prose"))?.into(), id.into()])?;
            }
        }
        for key in ["title", "summary", "description", "flags", "prose", "source", "siteName", "domain", "ocr", "author", "brand", "format", "objectNames"] {
            if has(body, key)? {
                row.data.set(key, body.get(key).clone());
            }
        }
        if has(body, "summary")? {
            row.extras.set("summaryEdited", true);
            row.extras.remove("summarySource");
        }
        if has(body, "title")? {
            row.extras.set("titleEdited", true);
            row.extras.remove("title");
            row.extras.remove("titleSource");
            if row.extras.get("reddit").truthy()
                && let Some(reddit) = row.extras.get_mut("reddit")
            {
                reddit.set("title", body.get("title").clone());
            }
        }
        if has(body, "description")?
            && row.extras.get("post").truthy()
            && let Some(post) = row.extras.get_mut("post")
        {
            post.remove("bodyHtml");
        }
        if has(body, "author")? && row.extras.get("post").truthy() {
            let author = body.get("author");
            let name = if author.get("name").is_nullish() { author.clone() } else { author.get("name").clone() };
            if let Some(post) = row.extras.get_mut("post") {
                post.set("author", name);
            }
        }
        if body.get("prose").truthy() && has_content(body.get("prose"))? {
            clear_bit(&mut row.data, 262_144);
        }
        self.save_default(&mut row)?;
        Ok(Value::Object(Object::new()))
    }

    /// Save a document in its editor resource and refresh the card (`prose`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `body` - Prose document `{type: 'doc', content}`.
    /// * `kind` - `prose`, `note`, `content` or `commit`.
    ///
    /// # Returns
    /// `{id}`; for a note, the note's id.
    ///
    /// # Errors
    /// Status 400 for an invalid document; status 404 for an unknown card; the database's error.
    pub fn prose(&self, id: &str, body: &Value, kind: &str) -> Result<Value> {
        document_body(body)?;
        let mut row = self.load(id)?;
        if kind != "note" && has_content(body)? {
            clear_bit(&mut row.data, 262_144);
        }
        let mut result: Value = obj! { "id" => id }.into();
        if kind == "note" {
            let note_id = match row.data.get("note").get("id") {
                Value::Undefined | Value::Null => Value::from(self.store.new_id(IdKind::Note, None)?),
                other => other.clone(),
            };
            let modified = iso_from_ns(self.tick()?, 6)?;
            row.data.set("note", obj! { "id" => note_id.clone(), "prose" => body.clone(), "modified" => modified });
            result = obj! { "id" => note_id }.into();
        } else if kind == "content" {
            self.preserve_body(&row)?;
            self.store.db().execute("INSERT OR REPLACE INTO card_content VALUES (?, ?)", &[id.into(), stringify(body)?.into()])?;
            if matches!(row.data.get("type").as_str(), Some("Note" | "Content")) {
                row.data.set("prose", body.clone());
            }
            self.revision(id, body)?;
        } else {
            self.preserve_body(&row)?;
            row.data.set("prose", body.clone());
            if self.has_content_row(id)? {
                self.store.db().execute("UPDATE card_content SET body = ? WHERE card_id = ?", &[stringify(body)?.into(), id.into()])?;
            }
            self.revision(id, body)?;
            if kind == "commit" {
                clear_bit(&mut row.data, 131_072);
                row.bumped_ns = self.tick()?;
                row.data.set("bumped", iso_from_ns(row.bumped_ns, 0)?);
            }
        }
        self.save_default(&mut row)?;
        Ok(result)
    }

    /// Publish a draft with the creation status used by the composer (`commit`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `body` - Prose document.
    ///
    /// # Returns
    /// `{status, body}`: 201 when the card was a draft, otherwise 200.
    ///
    /// # Errors
    /// The errors of [`Writer::prose`].
    pub fn commit(&self, id: &str, body: &Value) -> Result<Value> {
        let was_draft = to_int32(self.load(id)?.data.get("flags").to_number()) & 131_072 != 0;
        let saved = self.prose(id, body, "commit")?;
        Ok(obj! { "status" => if was_draft { 201.0 } else { 200.0 }, "body" => saved }.into())
    }

    /// Create a per-card annotation unless that name already exists (`addTag`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `name` - Tag name; a leading `#` is dropped.
    ///
    /// # Returns
    /// The tag `{id, name, flags?}`.
    ///
    /// # Errors
    /// Status 404 for an unknown card; status 400 `Tag name is required`; the database's error.
    pub fn add_tag(&self, id: &str, name: &Value) -> Result<Value> {
        let mut row = self.load(id)?;
        let Some(given) = name.as_str().filter(|given| !text::trim(given).is_empty()) else { return Err(fail(400, "Tag name is required")) };
        let trimmed = text::trim(given);
        let name = text::trim(trimmed.strip_prefix('#').unwrap_or(trimmed)).to_owned();
        if name.is_empty() {
            return Err(fail(400, "Tag name is required"));
        }
        if let Some(existing) = self.store.card_tags(id)?.into_iter().find(|tag| tag.name == name) {
            return Ok(existing.to_value());
        }
        let tag_id = self.store.new_id(IdKind::Tag, None)?;
        let highlight = name == "highlight";
        let mut tag: Value = obj! { "id" => tag_id.as_str(), "name" => name.as_str() }.into();
        if !highlight {
            tag.set("flags", 8.0);
        }
        let db = self.store.db();
        let position = db
            .one("SELECT COALESCE(MAX(position), -1) + 1 AS n FROM annotations WHERE card_id = ?", &[id.into()])?
            .and_then(|found| found.number("n"))
            .unwrap_or(0.0);
        db.execute(
            "INSERT INTO annotations VALUES (?, ?, ?, ?, ?, ?)",
            &[tag_id.as_str().into(), self.user.id.as_str().into(), id.into(), name.as_str().into(), if highlight { 0.0 } else { 8.0 }.into(), position.into()],
        )?;
        if PRIVACY_TAGS.contains(&name.as_str()) {
            set_bit(&mut row.data, 8);
        }
        let modified = self.tick()? / MS_NS;
        db.execute(
            "INSERT INTO tag_names (user_id, name, modified_ms) VALUES (?, ?, ?)
        ON CONFLICT(user_id, name) DO UPDATE SET modified_ms = excluded.modified_ms",
            &[self.user.id.as_str().into(), name.as_str().into(), (modified as f64).into()],
        )?;
        self.save_default(&mut row)?;
        let mut event = tag.clone();
        event.set("objectId", id);
        event.set("flags", if highlight { 0.0 } else { 8.0 });
        self.queue("tag:created", event);
        Ok(tag)
    }

    /// Remove only the annotation with this per-card id (`removeTag`).
    ///
    /// # Arguments
    /// * `id` - Annotation id.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 `Annotation not found`; the database's error.
    pub fn remove_tag(&self, id: &str) -> Result<Value> {
        let db = self.store.db();
        let tag = db
            .one("SELECT card_id, name FROM annotations WHERE id = ? AND user_id = ?", &[id.into(), self.user.id.as_str().into()])?
            .ok_or_else(|| fail(404, "Annotation not found"))?;
        let (card_id, name) = (tag.text("card_id").unwrap_or("").to_owned(), tag.text("name").unwrap_or("").to_owned());
        db.execute("DELETE FROM annotations WHERE id = ?", &[id.into()])?;
        let mut row = self.load(&card_id)?;
        if PRIVACY_TAGS.contains(&name.as_str()) && !self.store.card_tags(&row.id)?.iter().any(|item| PRIVACY_TAGS.contains(&item.name.as_str())) {
            clear_bit(&mut row.data, 8);
        }
        self.save_default(&mut row)?;
        self.queue("tag:deleted", obj! { "objectId" => card_id.as_str(), "name" => name.as_str() }.into());
        Ok(Value::Object(Object::new()))
    }

    /// Toggle a client-visible card flag and maintain pin membership (`flag`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `bit` - Flag bit; 512 is the pin.
    /// * `enabled` - Whether the bit is set.
    /// * `position` - One-based pin position; `Value::Undefined` means 1.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn flag(&self, id: &str, bit: i32, enabled: bool, position: &Value) -> Result<Value> {
        let mut row = self.load(id)?;
        update_flags(&mut row.data, |flags| if enabled { flags | bit } else { flags & !bit });
        if bit == 512 {
            let mut settings = self.current_user()?.settings;
            settings.set("pinsChanged", true);
            self.write_settings(&settings)?;
            let db = self.store.db();
            db.execute("DELETE FROM pins WHERE card_id = ?", &[id.into()])?;
            if enabled {
                let wanted = if position.is_undefined() { 1.0 } else { position.to_number() };
                let integer = matches!(position, Value::Undefined) || matches!(position, Value::Number(_)) && wanted.is_finite() && wanted.fract() == 0.0;
                let rank = (if integer { wanted - 1.0 } else { 0.0 }).max(0.0);
                db.execute("UPDATE pins SET position = position + 1 WHERE position >= ?", &[rank.into()])?;
                db.execute("INSERT INTO pins VALUES (?, ?)", &[id.into(), rank.into()])?;
                row.extras.set("pinPosition", rank);
            } else {
                row.extras.remove("pinPosition");
            }
        }
        self.save_default(&mut row)?;
        Ok(Value::Object(Object::new()))
    }

    /// Persist the requested pin order (`reorderPins`).
    ///
    /// # Arguments
    /// * `ids` - Array of card ids in order.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 400 unless `ids` is an array; status 404 for an unknown card; the database's error.
    pub fn reorder_pins(&self, ids: &Value) -> Result<Value> {
        let Some(list) = ids.as_array() else { return Err(fail(400, "Expected an array of card ids")) };
        let mut settings = self.current_user()?.settings;
        settings.set("pinsChanged", true);
        self.write_settings(&settings)?;
        for (position, id) in list.iter().enumerate() {
            let id = id_text(id);
            let mut row = self.load(&id)?;
            set_bit(&mut row.data, 512);
            row.extras.set("pinPosition", position as f64);
            self.store.db().execute("INSERT OR REPLACE INTO pins VALUES (?, ?)", &[id.as_str().into(), (position as f64).into()])?;
            self.save_default(&mut row)?;
        }
        Ok(Value::Object(Object::new()))
    }

    /// Move a card into the trash or restore it (`trash`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `restore` - Restore instead of trashing.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn trash(&self, id: &str, restore: bool) -> Result<Value> {
        let mut row = self.load(id)?;
        if restore {
            row.data.remove("deleted");
            row.extras.remove("mergedInto");
        } else {
            row.data.set("deleted", iso_from_ns(self.tick()?, 0)?);
            clear_bit(&mut row.data, 1);
        }
        self.store_card(&mut row, Some(if restore { "object:updated" } else { "object:deleted" }), Some(if restore { 1 } else { 2 }))?;
        Ok(Value::Object(Object::new()))
    }

    /// Fold a repeat save into the earlier card it duplicates: the earlier card takes the repeat's own tags, its note
    /// (after its own, when both have one) and its Spaces, and is bumped to the top as a fresh save of a link is; the
    /// repeat goes to the trash, restorable, marked `extras.mergedInto` with the earlier card's id.
    ///
    /// # Arguments
    /// * `id` - The repeat save.
    /// * `earlier` - The earlier card it duplicates.
    ///
    /// # Returns
    /// `false`, changing nothing, when either card is gone or trashed or they are the same card.
    ///
    /// # Errors
    /// The database's error.
    pub fn merge_into(&self, id: &str, earlier: &str) -> Result<bool> {
        let alive = |card: &str| -> Result<bool> { Ok(self.store.card(card, None)?.is_some_and(|row| row.deleted_ns.is_none())) };
        if id == earlier || !alive(id)? || !alive(earlier)? {
            return Ok(false);
        }
        let repeat = self.load(id)?;
        let mut target = self.load(earlier)?;
        let note = repeat.data.get("note").get("prose");
        if note.truthy() && has_content(note)? {
            let kept = target.data.get("note").get("prose");
            let prose = if kept.truthy() && has_content(kept)? {
                let mut content = kept.get("content").as_array().cloned().unwrap_or_default();
                content.extend(note.get("content").as_array().cloned().unwrap_or_default());
                obj! { "type" => "doc", "content" => Value::Array(content) }.into()
            } else {
                note.clone()
            };
            let note_id = match target.data.get("note").get("id") {
                Value::Undefined | Value::Null => Value::from(self.store.new_id(IdKind::Note, None)?),
                other => other.clone(),
            };
            target.data.set("note", obj! { "id" => note_id, "prose" => prose, "modified" => iso_from_ns(self.tick()?, 6)? });
        }
        target.bumped_ns = self.tick()?;
        target.data.set("bumped", iso_from_ns(target.bumped_ns, 0)?);
        self.store_card(&mut target, Some("object:bumped"), None)?;
        let db = self.store.db();
        let own_tags = db.query("SELECT name FROM annotations WHERE card_id = ? AND (flags & 2) = 0 ORDER BY position", &[id.into()])?;
        for tag in &own_tags {
            self.add_tag(earlier, &Value::from(tag.text("name").unwrap_or("")))?;
        }
        let spaces = db.query("SELECT space_id FROM space_cards WHERE card_id = ? AND excluded = 0 ORDER BY rank", &[id.into()])?;
        for space in &spaces {
            self.add_to_space(space.text("space_id").unwrap_or(""), earlier)?;
        }
        let mut repeat = self.load(id)?;
        repeat.extras.set("mergedInto", earlier);
        repeat.data.set("deleted", iso_from_ns(self.tick()?, 0)?);
        clear_bit(&mut repeat.data, 1);
        self.store_card(&mut repeat, Some("object:deleted"), Some(2))?;
        Ok(true)
    }

    /// Create a manual space or evaluate smart filters against the library (`createSpace`).
    ///
    /// # Arguments
    /// * `body` - `{name, color?, position?, query?: {filters}}`.
    ///
    /// # Returns
    /// The new space as `GET /spaces/<id>` returns it.
    ///
    /// # Errors
    /// Status 400 `Space name is required` or the filters' error; the database's error.
    pub fn create_space(&self, body: &Value) -> Result<Value> {
        self.create_space_as(body, None)
    }

    /// Create a space as [`Writer::create_space`] does, under an id the caller chose, such as one a phone made offline.
    ///
    /// # Arguments
    /// * `body` - As for [`Writer::create_space`].
    /// * `id` - The new space's id, or `None` for the next id of the library's own.
    ///
    /// # Returns
    /// The new space as `GET /spaces/<id>` returns it.
    ///
    /// # Errors
    /// Status 409 `Space already exists` for a chosen id in use; otherwise as [`Writer::create_space`].
    pub fn create_space_as(&self, body: &Value, id: Option<&str>) -> Result<Value> {
        let name = field(body, "name")?;
        let Some(label) = name.as_str().filter(|label| !text::trim(label).is_empty()) else { return Err(fail(400, "Space name is required")) };
        let id = match id {
            Some(chosen) if self.store.db().one("SELECT 1 FROM spaces WHERE id = ?", &[chosen.into()])?.is_some() => return Err(fail(409, "Space already exists")),
            Some(chosen) => chosen.to_owned(),
            None => self.store.new_id(IdKind::Space, None)?,
        };
        let created = iso_from_ns(self.tick()?, 6)?;
        let query = if body.get("query").truthy() {
            let filters = body.get("query").get("filters");
            let filters = if filters.is_nullish() { Value::Array(Vec::new()) } else { filters.clone() };
            let query_id = self.store.new_id(IdKind::Query, None)?;
            Some(Value::from(obj! { "id" => query_id, "filters" => filters, "checked" => iso_from_ns(self.tick()?, 0)? }))
        } else {
            None
        };
        let db = self.store.db();
        let position = match body.get("position") {
            Value::Undefined | Value::Null => match &query {
                Some(_) => Value::Number(-(date::parse(&created) / 1000.0).floor()),
                None => Value::Number(db.one("SELECT COALESCE(MAX(position), 0) + 1 AS n FROM spaces WHERE user_id = ?", &[self.user.id.as_str().into()])?.and_then(|row| row.number("n")).unwrap_or(0.0)),
            },
            other => other.clone(),
        };
        let color = match body.get("color") {
            Value::Undefined | Value::Null => Value::from("#e8c4a0"),
            other => other.clone(),
        };
        db.execute(
            "INSERT INTO spaces VALUES (?, ?, ?, ?, ?, ?, ?)",
            &[
                id.as_str().into(),
                self.user.id.as_str().into(),
                label.into(),
                bind(&color)?,
                bind(&position)?,
                created.into(),
                match &query {
                    Some(query) => stringify(query)?.into(),
                    None => SqlValue::Null,
                },
            ],
        )?;
        if let Some(query) = &query {
            for (rank, member) in self.search.filters(query.get("filters"))?.iter().enumerate() {
                self.load(&member.id)?;
                db.execute(
                    "INSERT INTO space_cards VALUES (?, ?, ?, ?, ?, ?, 0)",
                    &[id.as_str().into(), member.id.as_str().into(), self.user.id.as_str().into(), (rank as f64).into(), member.score.into(), query.get("id").to_js_string().into()],
                )?;
                let mut card = self.load(&member.id)?;
                self.save_default(&mut card)?;
            }
        }
        self.store_space(&id, "space:created", 1)?;
        Ok(self.store.space(&id)?.map_or(Value::Null, |space| space.to_value()))
    }

    /// Rename or recolor a space (`patchSpace`).
    ///
    /// # Arguments
    /// * `id` - Space id.
    /// * `body` - `{name?, color?}`.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 `Space not found`; the database's error.
    pub fn patch_space(&self, id: &str, body: &Value) -> Result<Value> {
        let row = self.space(id)?;
        let name = if field(body, "name")?.is_nullish() { Value::from(row.name.as_str()) } else { body.get("name").clone() };
        let color = if body.get("color").is_nullish() { Value::from(row.color.as_str()) } else { body.get("color").clone() };
        self.store.db().execute("UPDATE spaces SET name = ?, color = ? WHERE id = ?", &[bind(&name)?, bind(&color)?, id.into()])?;
        self.store_space(id, "space:updated", 1)?;
        Ok(Value::Object(Object::new()))
    }

    /// Assign ascending positions to the given spaces (`reorderSpaces`).
    ///
    /// # Arguments
    /// * `ids` - Array of space ids in order.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 400 unless `ids` is an array; status 404 for an unknown space; the database's error.
    pub fn reorder_spaces(&self, ids: &Value) -> Result<Value> {
        let Some(list) = ids.as_array() else { return Err(fail(400, "Expected an array of space ids")) };
        for (position, id) in list.iter().enumerate() {
            let id = id_text(id);
            self.space(&id)?;
            self.store.db().execute("UPDATE spaces SET position = ? WHERE id = ?", &[((position + 1) as f64).into(), id.as_str().into()])?;
            self.store_space(&id, "space:updated", 1)?;
        }
        Ok(Value::Object(Object::new()))
    }

    /// Delete a space and keep its cards (`deleteSpace`).
    ///
    /// # Arguments
    /// * `id` - Space id.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 `Space not found`; the database's error.
    pub fn delete_space(&self, id: &str) -> Result<Value> {
        let row = self.space(id)?;
        self.store.db().execute("DELETE FROM spaces WHERE id = ?", &[id.into()])?;
        self.store_space(id, "space:deleted", 2)?;
        for member in &row.objects {
            let mut card = self.load(&member.id)?;
            self.save_default(&mut card)?;
        }
        Ok(Value::Object(Object::new()))
    }

    /// Add or remove a manual space membership (`membership`).
    ///
    /// # Arguments
    /// * `space_id` - Space id.
    /// * `id` - Card id.
    /// * `remove` - Remove instead of add.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown space or card; the database's error.
    pub fn membership(&self, space_id: &str, id: &str, remove: bool) -> Result<Value> {
        if remove {
            self.space(space_id)?;
            self.load(id)?;
            self.store.db().execute("DELETE FROM space_cards WHERE space_id = ? AND card_id = ?", &[space_id.into(), id.into()])?;
            self.store_space(space_id, "space:updated", 1)?;
            let mut card = self.load(id)?;
            self.save_default(&mut card)?;
        } else {
            self.add_to_space(space_id, id)?;
        }
        Ok(Value::Object(Object::new()))
    }

    /// Retain a smart-space exclusion (`exclude`).
    ///
    /// # Arguments
    /// * `query_id` - Saved query id.
    /// * `id` - Card id.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 `Query not found` or `Card not found`; the database's error.
    pub fn exclude(&self, query_id: &str, id: &str) -> Result<Value> {
        let db = self.store.db();
        let row = db
            .one("SELECT id FROM spaces WHERE user_id = ? AND json_extract(query, '$.id') = ?", &[self.user.id.as_str().into(), query_id.into()])?
            .ok_or_else(|| fail(404, "Query not found"))?;
        let space_id = row.text("id").unwrap_or("").to_owned();
        self.load(id)?;
        db.execute("UPDATE space_cards SET excluded = 1 WHERE space_id = ? AND card_id = ?", &[space_id.as_str().into(), id.into()])?;
        self.store_space(&space_id, "space:updated", 1)?;
        let mut card = self.load(id)?;
        self.save_default(&mut card)?;
        Ok(Value::Object(Object::new()))
    }

    /// Persist supported settings (`settings`).
    ///
    /// # Arguments
    /// * `body` - Any of `colorScheme`, `locale`, `timeZoneId`, `soundEnabled`, `onboardFlags`.
    ///
    /// # Returns
    /// `{status: 200}`.
    ///
    /// # Errors
    /// The database's error.
    pub fn settings(&self, body: &Value) -> Result<Value> {
        let mut current = self.current_user()?;
        for key in ["colorScheme", "locale", "timeZoneId", "soundEnabled", "onboardFlags"] {
            if has(body, key)? {
                current.settings.set(key, body.get(key).clone());
            }
        }
        if has(body, "soundEnabled")? {
            current.flags = if body.get("soundEnabled").truthy() { current.flags & !64 } else { current.flags | 64 };
        }
        self.store.db().execute(
            "UPDATE users SET settings = ?, flags = ? WHERE id = ?",
            &[stringify(&current.settings)?.into(), (current.flags as f64).into(), self.user.id.as_str().into()],
        )?;
        self.queue("account:updated", obj! { "id" => self.user.account_uid.as_str() }.into());
        Ok(obj! { "status" => 200.0 }.into())
    }

    /// Retain a dismissal or completion id in the user settings (`remember`).
    ///
    /// # Arguments
    /// * `key` - Settings key holding the list.
    /// * `id` - Value to add.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// The database's error.
    pub fn remember(&self, key: &str, id: &Value) -> Result<Value> {
        let mut current = self.current_user()?;
        let mut items: Vec<Value> = match current.settings.get(key) {
            Value::Undefined | Value::Null => Vec::new(),
            Value::Array(existing) => existing.clone(),
            _ => return Err(Error::invalid("object is not iterable")),
        };
        items.push(id.clone());
        let mut unique: Vec<Value> = Vec::new();
        for item in items {
            let primitive = !matches!(item, Value::Object(_) | Value::Array(_));
            if !(primitive && unique.iter().any(|seen| strict_equal(seen, &item))) {
                unique.push(item);
            }
        }
        current.settings.set(key, unique);
        self.write_settings(&current.settings)?;
        Ok(Value::Object(Object::new()))
    }

    /// Set or clear an onboarding bit (`onboard`).
    ///
    /// # Arguments
    /// * `value` - The bit.
    /// * `enabled` - Whether it is set.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// The database's error.
    pub fn onboard(&self, value: i32, enabled: bool) -> Result<Value> {
        let current = self.current_user()?;
        let flags = match current.settings.get("onboardFlags") {
            Value::Undefined | Value::Null => 0,
            other => to_int32(other.to_number()),
        };
        let next = if enabled { flags | value } else { flags & !value };
        self.settings(&obj! { "onboardFlags" => f64::from(next) }.into())?;
        Ok(Value::Object(Object::new()))
    }

    /// Create a local share that expires in 24 hours (`share`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    ///
    /// # Returns
    /// `{id, url}`.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn share(&self, id: &str) -> Result<Value> {
        self.load(id)?;
        let share_id = base64::hex(&(self.random)(16));
        let ts = self.tick()?;
        self.store.db().execute("INSERT INTO shares VALUES (?, ?, ?, ?)", &[share_id.as_str().into(), id.into(), SqlValue::big(ts), SqlValue::big(ts + DAY_NS)])?;
        let mut card = self.load(id)?;
        self.save_default(&mut card)?;
        Ok(obj! { "id" => share_id.as_str(), "url" => format!("/shares/{share_id}") }.into())
    }

    /// Remove a share of a card (`revokeShare`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `share_id` - Share id.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn revoke_share(&self, id: &str, share_id: &str) -> Result<Value> {
        self.load(id)?;
        self.store.db().execute("DELETE FROM shares WHERE id = ? AND card_id = ?", &[share_id.into(), id.into()])?;
        let mut card = self.load(id)?;
        self.save_default(&mut card)?;
        Ok(Value::Object(Object::new()))
    }

    /// Restore a stored revision to the card's editor resource (`restoreRevision`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `revision_id` - Revision id.
    ///
    /// # Returns
    /// The result of [`Writer::prose`].
    ///
    /// # Errors
    /// Status 404 `Revision not found` or for an unknown card; the database's error.
    pub fn restore_revision(&self, id: &str, revision_id: &str) -> Result<Value> {
        self.load(id)?;
        let row = self
            .store
            .db()
            .one("SELECT body FROM revisions WHERE id = ? AND card_id = ?", &[revision_id.into(), id.into()])?
            .ok_or_else(|| fail(404, "Revision not found"))?;
        let kind = if self.has_content_row(id)? { "content" } else { "prose" };
        self.prose(id, &json::parse(row.text("body").unwrap_or(""))?, kind)
    }

    /// Remove a card's generated cover metadata (`removeThumbnail`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    ///
    /// # Returns
    /// An empty object.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn remove_thumbnail(&self, id: &str) -> Result<Value> {
        let mut row = self.load(id)?;
        row.extras.remove("cover");
        clear_bit(&mut row.data, 8_388_608);
        self.save_default(&mut row)?;
        Ok(Value::Object(Object::new()))
    }

    /// Keep the ThumbHash of a card's gallery picture as its `thumbHash`, unless the picture changed while the hash was
    /// read.
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `source` - The picture the hash was read from: the cover's `src`, or the blob's `path` of a card without a
    ///   cover.
    /// * `hash` - The hash in base64.
    ///
    /// # Returns
    /// Whether the card kept it.
    ///
    /// # Errors
    /// Status 404 for an unknown card; the database's error.
    pub fn picture_hash(&self, id: &str, source: &str, hash: &str) -> Result<bool> {
        let mut row = self.load(id)?;
        let (holder, key) = if row.extras.get("cover").get("src").as_str() == Some(source) {
            (&mut row.extras, "cover")
        } else if !row.extras.get("cover").truthy() && row.data.get("blob").get("path").as_str() == Some(source) {
            (&mut row.data, "blob")
        } else {
            return Ok(false);
        };
        let mut picture = holder.get(key).clone();
        picture.set("thumbHash", hash);
        holder.set(key, picture);
        self.save_default(&mut row)?;
        Ok(true)
    }
}

impl EnrichHooks for Writer {
    fn card(&self, id: &str) -> Result<EditCard> {
        self.load(id)
    }

    fn save(&self, card: &mut EditCard, event: Option<&str>) -> Result<()> {
        self.store_card(card, event, None)
    }

    fn now(&self) -> Result<i128> {
        self.tick()
    }

    fn emit(&self, target: &str, payload: Value) {
        self.queue(target, payload);
    }
}

impl QueryWriter for Writer {
    type Card = EditCard;

    fn now(&self) -> Result<i128> {
        self.tick()
    }

    fn card(&self, id: &str) -> Result<EditCard> {
        self.load(id)
    }

    fn save(&self, mut card: EditCard) -> Result<()> {
        self.save_default(&mut card)
    }

    fn save_space(&self, id: &str) -> Result<()> {
        self.store_space(id, "space:updated", 1)
    }
}
