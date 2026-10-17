//! Port of `lib/store.mjs`: the typed read API over the library database, plus id generation.
//!
//! `openStore` and `close` belong to the host; [`Store::new`] is `createStore`.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::LazyLock;

use mm_js::{Object, Value, date, json, text};
use regex::Regex;

use crate::search_index::{install_search, refresh_search};
use crate::sql::{Row, Sql, SqlExt, SqlValue};
use crate::store_schema::SCHEMA;
use crate::{Error, Result};

/// Number of grid cards the client draws before it paints the rest of the stream (`INITIAL_DRAW_COUNT`).
pub const INITIAL_DRAW_COUNT: usize = 250;

/// Card flag bits that hide a card from the grid: private, draft, stub (`GRID_HIDDEN_FLAGS`).
pub const GRID_HIDDEN_FLAGS: i64 = 8 | 131_072 | 262_144;

const MANUAL_TAG_FLAG: i64 = 8;
const NS_PER_SECOND: i128 = 1_000_000_000;
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const COUNTER_WIDTH: u32 = 9;
// Ids per batched lookup; Durable Object SQLite caps a statement at 100 bound parameters.
const LOOKUP_CHUNK: usize = 50;

static ISO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2})(?:\.([0-9]{1,9}))?Z$").expect("pattern"));

fn visible() -> String {
    format!("c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0")
}

fn ns_column(column: &str, alias: &str) -> String {
    format!("CAST({column} AS TEXT) AS {alias}")
}

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(",")
}

fn card_columns(prefix: &str) -> String {
    format!(
        "{prefix}id, {prefix}type, {prefix}flags, {prefix}data, {prefix}html, {prefix}extras, {}, {}, {}, {}",
        ns_column(&format!("{prefix}created_ns"), "created_ns"),
        ns_column(&format!("{prefix}bumped_ns"), "bumped_ns"),
        ns_column(&format!("{prefix}modified_ns"), "modified_ns"),
        ns_column(&format!("{prefix}deleted_ns"), "deleted_ns"),
    )
}

/// Convert an ISO timestamp with up to nine fractional digits to nanoseconds since the epoch (`nsFromIso`).
///
/// # Arguments
/// * `iso` - UTC timestamp such as `2026-10-15T09:35:08.050534Z`.
///
/// # Returns
/// Exact nanoseconds since the epoch.
///
/// # Errors
/// An invalid-data error when `iso` is not a UTC ISO timestamp or names an impossible date.
pub fn ns_from_iso(iso: &str) -> Result<i128> {
    let captures = ISO.captures(iso).ok_or_else(|| Error::invalid(format!("Not a UTC ISO timestamp: {iso}")))?;
    let ms = date::parse(&format!("{}Z", &captures[1]));
    if ms.is_nan() {
        return Err(Error::invalid("The number NaN cannot be converted to a BigInt"));
    }
    let fraction = text::pad_end(captures.get(2).map_or("", |found| found.as_str()), 9, "0");
    Ok((ms / 1000.0) as i128 * NS_PER_SECOND + fraction.parse::<i128>().map_err(|_| Error::internal("fraction"))?)
}

/// Format nanoseconds since the epoch as a UTC ISO timestamp (`isoFromNs`).
///
/// # Arguments
/// * `ns` - Nanoseconds since the epoch.
/// * `digits` - Fractional digits to include, 0 to 9.
///
/// # Returns
/// An ISO timestamp ending in `Z`, truncated (not rounded) to `digits`.
///
/// # Errors
/// An invalid-data error when the instant is outside the range of a JS date.
pub fn iso_from_ns(ns: i128, digits: usize) -> Result<String> {
    let seconds = (ns / NS_PER_SECOND) as f64;
    let iso = date::iso_from_ms(seconds * 1000.0).ok_or_else(|| Error::invalid("Invalid time value"))?;
    let base = text::slice(&iso, 0, Some(19));
    if digits == 0 {
        return Ok(format!("{base}Z"));
    }
    let fraction = text::pad_start(&(ns % NS_PER_SECOND).to_string(), 9, "0");
    Ok(format!("{base}.{}Z", text::slice(&fraction, 0, Some(digits as i64))))
}

fn decode_counter(source: &str) -> i128 {
    source.chars().fold(0_i128, |total, character| {
        let digit = BASE62.iter().position(|candidate| char::from(*candidate) == character).map_or(-1, |index| index as i128);
        total.saturating_mul(62).saturating_add(digit)
    })
}

fn encode_counter(value: i128) -> String {
    let mut rest = value;
    let mut out = String::new();
    for _ in 0..COUNTER_WIDTH {
        let index = rest % 62;
        let character = if index < 0 { "undefined".to_owned() } else { char::from(BASE62[index as usize]).to_string() };
        out = character + &out;
        rest /= 62;
    }
    out
}

fn big(row: &Row, column: &str) -> Result<i128> {
    row.text(column)
        .ok_or_else(|| Error::internal(format!("{column} is not text")))?
        .parse::<i128>()
        .map_err(|_| Error::internal(format!("{column} is not an integer")))
}

fn number(row: &Row, column: &str) -> i64 {
    row.number(column).unwrap_or(0.0) as i64
}

fn string(row: &Row, column: &str) -> String {
    row.text(column).unwrap_or("").to_owned()
}

/// A card as the store reads it (`CardRow`).
#[derive(Clone, Debug, PartialEq)]
pub struct CardRow {
    /// Card id.
    pub id: String,
    /// Card type such as `XPost` or `Article` (`type`).
    pub kind: String,
    /// Card flag bitmask.
    pub flags: i64,
    /// Card JSON exactly as mymind sends it in the `json` field.
    pub data: String,
    /// Grid card markup exactly as mymind sends it in the `html` field.
    pub html: String,
    /// JSON object of source metadata that the card renderer needs besides `data`.
    pub extras: String,
    /// Creation time in nanoseconds since the epoch.
    pub created_ns: i128,
    /// Grid sort key in nanoseconds since the epoch.
    pub bumped_ns: i128,
    /// Last change in nanoseconds since the epoch.
    pub modified_ns: i128,
    /// Time moved to trash in nanoseconds, or `None` when not deleted.
    pub deleted_ns: Option<i128>,
}

impl CardRow {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: string(row, "id"),
            kind: string(row, "type"),
            flags: number(row, "flags"),
            data: string(row, "data"),
            html: string(row, "html"),
            extras: string(row, "extras"),
            created_ns: big(row, "created_ns")?,
            bumped_ns: big(row, "bumped_ns")?,
            modified_ns: big(row, "modified_ns")?,
            deleted_ns: if row.get("deleted_ns").is_null() { None } else { Some(big(row, "deleted_ns")?) },
        })
    }

    /// The object the JS `cardRow` built, nanosecond fields as BigInts.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", self.id.clone());
        object.insert("type", self.kind.clone());
        object.insert("flags", self.flags);
        object.insert("data", self.data.clone());
        object.insert("html", self.html.clone());
        object.insert("extras", self.extras.clone());
        object.insert("createdNs", Value::BigInt(self.created_ns));
        object.insert("bumpedNs", Value::BigInt(self.bumped_ns));
        object.insert("modifiedNs", Value::BigInt(self.modified_ns));
        object.insert("deletedNs", self.deleted_ns.map_or(Value::Null, Value::BigInt));
        Value::Object(object)
    }
}

/// An account record (`user()`'s result).
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    /// User id.
    pub id: String,
    /// Account uid mymind knows the user by.
    pub account_uid: String,
    /// Prefix of every id this user creates.
    pub id_prefix: String,
    /// User flag bitmask.
    pub flags: i64,
    /// User settings JSON, such as `objectLimit`.
    pub settings: Value,
}

impl User {
    /// The object the JS `user()` returned.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", self.id.clone());
        object.insert("accountUid", self.account_uid.clone());
        object.insert("idPrefix", self.id_prefix.clone());
        object.insert("flags", self.flags);
        object.insert("settings", self.settings.clone());
        Value::Object(object)
    }
}

/// The last card of the previous keyset page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
    /// Its sort key: `bumped_ns` for the grid, `deleted_ns` for the trash.
    pub ns: i128,
    /// Its id.
    pub id: String,
}

/// Object quota for the `x-object-quota` header.
#[derive(Clone, Debug, PartialEq)]
pub struct Quota {
    /// Visible card count.
    pub count: i64,
    /// The user's `settings.objectLimit`, undefined when the settings lack it.
    pub limit: Value,
}

impl Quota {
    /// The object the JS `quota()` returned.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("count", self.count);
        object.insert("limit", self.limit.clone());
        Value::Object(object)
    }
}

/// A tag on a card.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    /// Tag id.
    pub id: String,
    /// Tag name.
    pub name: String,
    /// Flags, `None` when 0 (the JS omitted the property).
    pub flags: Option<i64>,
}

impl Tag {
    /// The object `cardTags` built.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", self.id.clone());
        object.insert("name", self.name.clone());
        if let Some(flags) = self.flags {
            object.insert("flags", flags);
        }
        Value::Object(object)
    }
}

/// A card's membership in a space.
#[derive(Clone, Debug, PartialEq)]
pub struct SpaceObject {
    /// Card id.
    pub id: String,
    /// Relevance score.
    pub score: f64,
    /// The saved query that put the card there, when one did.
    pub query_id: Option<String>,
}

/// A space as `GET /spaces` returns it.
#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    /// Space id.
    pub id: String,
    /// Space name.
    pub name: String,
    /// Space color.
    pub color: String,
    /// Sort position.
    pub position: i64,
    /// Member cards in rank order, excluded ones left out.
    pub objects: Vec<SpaceObject>,
    /// Creation time as stored.
    pub created: String,
    /// The saved query JSON, when the space has one.
    pub query: Option<Value>,
}

impl Space {
    /// The object the JS `space()` returned.
    pub fn to_value(&self) -> Value {
        let objects = self
            .objects
            .iter()
            .map(|member| {
                let mut object = Object::new();
                object.insert("id", member.id.clone());
                object.insert("score", member.score);
                if let Some(query_id) = &member.query_id {
                    object.insert("queryId", query_id.clone());
                }
                Value::Object(object)
            })
            .collect();
        let mut object = Object::new();
        object.insert("id", self.id.clone());
        object.insert("name", self.name.clone());
        object.insert("color", self.color.clone());
        object.insert("position", self.position);
        object.insert("objects", Value::Array(objects));
        object.insert("created", self.created.clone());
        if let Some(query) = &self.query {
            object.insert("query", query.clone());
        }
        Value::Object(object)
    }
}

/// A manual tag with its use count.
#[derive(Clone, Debug, PartialEq)]
pub struct ManualTag {
    /// Tag name.
    pub name: String,
    /// Cards that carry it.
    pub count: i64,
    /// Always 0.
    pub flags: i64,
    /// Last modification as the `Date` the JS built, in milliseconds since the epoch (NaN when out of range).
    pub modified: f64,
}

impl ManualTag {
    /// The object `tagsManual` built, `modified` as a Date.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", self.name.clone());
        object.insert("count", self.count);
        object.insert("flags", self.flags);
        object.insert("modified", Value::Date(self.modified));
        Value::Object(object)
    }
}

/// One entry of the change log.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    /// Log sequence number.
    pub seq: i64,
    /// Time of the change in nanoseconds since the epoch.
    pub ts: i128,
    /// 1 created or updated, 2 deleted (`type`).
    pub kind: i64,
    /// 9 is a card.
    pub object_type: i64,
    /// Id of the changed object.
    pub object_id: String,
    /// The card's current row when the object is a card that exists.
    pub card: Option<CardRow>,
}

impl Change {
    /// The object `changesSince` built.
    pub fn to_value(&self) -> Value {
        let mut object = Object::new();
        object.insert("seq", self.seq);
        object.insert("ts", Value::BigInt(self.ts));
        object.insert("type", self.kind);
        object.insert("objectType", self.object_type);
        object.insert("objectId", self.object_id.clone());
        object.insert("card", self.card.as_ref().map_or(Value::Null, CardRow::to_value));
        Value::Object(object)
    }
}

/// Options of [`Store::changes_since`].
#[derive(Clone, Debug)]
pub struct ChangesOptions<'a> {
    /// User id; the first user when `None`.
    pub user_id: Option<&'a str>,
    /// Maximum changes to return.
    pub limit: i64,
    /// Return only the newest change for each object.
    pub coalesce: bool,
}

impl Default for ChangesOptions<'_> {
    fn default() -> Self {
        Self { user_id: None, limit: 1000, coalesce: false }
    }
}

/// What an id is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdKind {
    Card,
    Tag,
    Space,
    Query,
    Note,
}

impl IdKind {
    /// The kind a JS caller named.
    ///
    /// # Arguments
    /// * `name` - `card`, `tag`, `space`, `query` or `note`.
    ///
    /// # Errors
    /// An invalid-data error for any other name.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "card" => Ok(Self::Card),
            "tag" => Ok(Self::Tag),
            "space" => Ok(Self::Space),
            "query" => Ok(Self::Query),
            "note" => Ok(Self::Note),
            _ => Err(Error::invalid(format!("Unknown id kind: {name}"))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Tag => "tag",
            Self::Space => "space",
            Self::Query => "query",
            Self::Note => "note",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Card => "0O",
            Self::Tag => "0y",
            Self::Space => "gl",
            Self::Query => "iV",
            Self::Note => "9Z",
        }
    }

    fn source(self) -> (&'static str, &'static str) {
        match self {
            Self::Card => ("cards", "id"),
            Self::Tag => ("annotations", "id"),
            Self::Space => ("spaces", "id"),
            Self::Query => ("spaces", "json_extract(query, '$.id')"),
            Self::Note => ("cards", "json_extract(data, '$.note.id')"),
        }
    }
}

/// The library database with its schema and search index installed (`createStore`'s handle).
pub struct Store {
    db: Rc<dyn Sql>,
    default_user: RefCell<Option<String>>,
    counters: RefCell<HashMap<String, i128>>,
}

impl Store {
    /// Install the schema and the search index on an open database (`createStore`).
    ///
    /// # Arguments
    /// * `db` - Writable database from the host.
    ///
    /// # Errors
    /// The database's error.
    pub fn new(db: Rc<dyn Sql>) -> Result<Self> {
        db.exec_batch(SCHEMA)?;
        install_search(&*db)?;
        Ok(Self { db, default_user: RefCell::new(None), counters: RefCell::new(HashMap::new()) })
    }

    /// The SQL handle writers use (`store.db`).
    pub fn db(&self) -> &Rc<dyn Sql> {
        &self.db
    }

    /// Run `work` in one write transaction, then reindex changed cards; roll back if it fails (`transaction`).
    ///
    /// # Arguments
    /// * `work` - Work that mutates the database.
    ///
    /// # Returns
    /// What `work` returns.
    ///
    /// # Errors
    /// The error `work` returned, after the rollback, or the database's error.
    pub fn transaction<T>(&self, work: impl FnOnce() -> Result<T>) -> Result<T> {
        self.db.transaction(|| {
            let result = work()?;
            refresh_search(&*self.db)?;
            Ok(result)
        })
    }

    fn resolve_user(&self, user_id: Option<&str>) -> Result<String> {
        if let Some(id) = user_id.filter(|id| !id.is_empty()) {
            return Ok(id.to_owned());
        }
        if let Some(id) = self.default_user.borrow().as_ref() {
            return Ok(id.clone());
        }
        let found = self.db.one("SELECT id FROM users ORDER BY rowid LIMIT 1", &[])?.and_then(|row| row.text("id").map(str::to_owned));
        let id = found.filter(|id| !id.is_empty()).ok_or_else(|| Error::internal("The store has no user; seed it first"))?;
        *self.default_user.borrow_mut() = Some(id.clone());
        Ok(id)
    }

    fn cards(&self, sql: &str, params: &[SqlValue]) -> Result<Vec<CardRow>> {
        self.db.query(sql, params)?.iter().map(CardRow::from_row).collect()
    }

    /// Id of the first (default) user (`defaultUserId`).
    ///
    /// # Errors
    /// An internal error when the store has no user.
    pub fn default_user_id(&self) -> Result<String> {
        self.resolve_user(None)
    }

    /// Account record and settings for a user (`user`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// The user, or `None` if unknown.
    ///
    /// # Errors
    /// An internal error when no user can be resolved.
    pub fn user(&self, user_id: Option<&str>) -> Result<Option<User>> {
        let id = self.resolve_user(user_id)?;
        let Some(row) = self.db.one("SELECT * FROM users WHERE id = ?", &[SqlValue::from(id)])? else { return Ok(None) };
        let settings = json::parse(row.text("settings").unwrap_or("null")).map_err(Error::from)?;
        Ok(Some(User {
            id: string(&row, "id"),
            account_uid: string(&row, "account_uid"),
            id_prefix: string(&row, "id_prefix"),
            flags: number(&row, "flags"),
            settings,
        }))
    }

    /// Visible grid cards, newest `bumped` first (`gridCards`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    /// * `offset` - Cards to skip.
    /// * `limit` - Maximum cards to return; `None` for all.
    ///
    /// # Errors
    /// The database's error.
    pub fn grid_cards(&self, user_id: Option<&str>, offset: i64, limit: Option<i64>) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        self.cards(
            &format!("SELECT {} FROM cards c WHERE {} ORDER BY c.bumped_ns DESC, c.id DESC LIMIT ? OFFSET ?", card_columns(""), visible()),
            &[user.into(), limit.unwrap_or(-1).into(), offset.into()],
        )
    }

    /// One keyset page of the grid, in the order of [`Store::grid_cards`] (`gridCardsAfter`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    /// * `after` - Last card of the previous page; `None` for the first page.
    /// * `limit` - Maximum cards to return.
    ///
    /// # Errors
    /// The database's error.
    pub fn grid_cards_after(&self, user_id: Option<&str>, after: Option<&Cursor>, limit: i64) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        let (ns, id) = cursor_params(after);
        self.cards(
            &format!(
                "SELECT {} FROM cards c WHERE {} AND (? IS NULL OR c.bumped_ns < ? OR (c.bumped_ns = ? AND c.id < ?))
        ORDER BY c.bumped_ns DESC, c.id DESC LIMIT ?",
                card_columns(""),
                visible()
            ),
            &[user.into(), ns.clone(), ns.clone(), ns, id, limit.into()],
        )
    }

    /// One keyset page of the trash, in the order of [`Store::trash_cards`] (`trashCardsAfter`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    /// * `after` - Last card of the previous page, `ns` being its deletion time; `None` for the first page.
    /// * `limit` - Maximum cards to return.
    ///
    /// # Errors
    /// The database's error.
    pub fn trash_cards_after(&self, user_id: Option<&str>, after: Option<&Cursor>, limit: i64) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        let (ns, id) = cursor_params(after);
        self.cards(
            &format!(
                "SELECT {} FROM cards WHERE user_id = ? AND deleted_ns IS NOT NULL AND (? IS NULL OR deleted_ns < ? OR (deleted_ns = ? AND id < ?))
        ORDER BY cards.deleted_ns DESC, cards.id DESC LIMIT ?",
                card_columns("")
            ),
            &[user.into(), ns.clone(), ns.clone(), ns, id, limit.into()],
        )
    }

    /// Number of visible grid cards (`gridCount`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Errors
    /// The database's error.
    pub fn grid_count(&self, user_id: Option<&str>) -> Result<i64> {
        let user = self.resolve_user(user_id)?;
        let row = self.db.one(&format!("SELECT COUNT(*) AS n FROM cards c WHERE {}", visible()), &[user.into()])?;
        Ok(row.map_or(0, |row| number(&row, "n")))
    }

    /// Object quota for the `x-object-quota` header (`quota`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Errors
    /// An invalid-data error when the user does not exist.
    pub fn quota(&self, user_id: Option<&str>) -> Result<Quota> {
        let user = self.user(user_id)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'id')"))?;
        Ok(Quota { count: self.grid_count(Some(&user.id))?, limit: user.settings.get("objectLimit").clone() })
    }

    /// One card by id, deleted ones included (`card`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// The card, or `None` if it does not exist.
    ///
    /// # Errors
    /// The database's error.
    pub fn card(&self, id: &str, user_id: Option<&str>) -> Result<Option<CardRow>> {
        let user = self.resolve_user(user_id)?;
        Ok(self.cards(&format!("SELECT {} FROM cards WHERE id = ? AND user_id = ?", card_columns("")), &[id.into(), user.into()])?.into_iter().next())
    }

    /// Cards for `POST /cards/grid`, in the order requested (`cardsByIds`).
    ///
    /// # Arguments
    /// * `ids` - Card ids.
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// The cards that exist, deleted ones included; unknown ids are skipped.
    ///
    /// # Errors
    /// The database's error.
    pub fn cards_by_ids(&self, ids: &[String], user_id: Option<&str>) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        let mut found = Vec::new();
        for id in ids {
            found.extend(self.cards(&format!("SELECT {} FROM cards WHERE id = ? AND user_id = ?", card_columns("")), &[id.into(), user.clone().into()])?);
        }
        Ok(found)
    }

    /// Top of Mind: pinned cards that are not deleted or hidden from the grid, in pin order.
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// The cards in pin order; cards without a stored position follow, most recently bumped first.
    ///
    /// # Errors
    /// The database's error.
    pub fn pinned_cards(&self, user_id: Option<&str>) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        self.cards(
            &format!(
                "SELECT {} FROM cards c LEFT JOIN pins p ON p.card_id = c.id
          WHERE c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & 512) != 0 AND (c.flags & 393224) = 0
          ORDER BY COALESCE(p.position, 2147483647), c.bumped_ns DESC, c.id DESC",
                card_columns("c.")
            ),
            &[user.into()],
        )
    }

    /// Trashed cards, most recently deleted first (`trashCards`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Errors
    /// The database's error.
    pub fn trash_cards(&self, user_id: Option<&str>) -> Result<Vec<CardRow>> {
        let user = self.resolve_user(user_id)?;
        self.cards(
            &format!("SELECT {} FROM cards WHERE user_id = ? AND deleted_ns IS NOT NULL ORDER BY cards.deleted_ns DESC, cards.id DESC", card_columns("")),
            &[user.into()],
        )
    }

    /// A card's tags in display order (`cardTags`).
    ///
    /// # Arguments
    /// * `card_id` - Card id.
    ///
    /// # Errors
    /// The database's error.
    pub fn card_tags(&self, card_id: &str) -> Result<Vec<Tag>> {
        let rows = self.db.query("SELECT id, name, flags FROM annotations WHERE card_id = ? ORDER BY position", &[card_id.into()])?;
        Ok(rows
            .iter()
            .map(|row| Tag { id: string(row, "id"), name: string(row, "name"), flags: Some(number(row, "flags")).filter(|flags| *flags != 0) })
            .collect())
    }

    /// Spaces as `GET /spaces` returns them (`spaces`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// Spaces sorted by position.
    ///
    /// # Errors
    /// The database's error.
    pub fn spaces(&self, user_id: Option<&str>) -> Result<Vec<Space>> {
        let user = self.resolve_user(user_id)?;
        let mut spaces = Vec::new();
        for row in self.db.query("SELECT id FROM spaces WHERE user_id = ? ORDER BY position, id", &[user.into()])? {
            spaces.extend(self.space(row.text("id").unwrap_or(""))?);
        }
        Ok(spaces)
    }

    /// One space as `GET /spaces/<id>` returns it (`space`).
    ///
    /// # Arguments
    /// * `id` - Space id.
    ///
    /// # Returns
    /// The space, or `None` if unknown.
    ///
    /// # Errors
    /// The database's error, or an internal error for a malformed stored query.
    pub fn space(&self, id: &str) -> Result<Option<Space>> {
        let Some(row) = self.db.one("SELECT * FROM spaces WHERE id = ?", &[id.into()])? else { return Ok(None) };
        let objects = self
            .db
            .query("SELECT card_id, score, query_id FROM space_cards WHERE space_id = ? AND excluded = 0 ORDER BY rank", &[id.into()])?
            .iter()
            .map(|member| SpaceObject {
                id: string(member, "card_id"),
                score: member.number("score").unwrap_or(0.0),
                query_id: member.text("query_id").filter(|query_id| !query_id.is_empty()).map(str::to_owned),
            })
            .collect();
        let query = match row.text("query").filter(|query| !query.is_empty()) {
            Some(query) => Some(json::parse(query).map_err(Error::from)?),
            None => None,
        };
        Ok(Some(Space {
            id: string(&row, "id"),
            name: string(&row, "name"),
            color: string(&row, "color"),
            position: number(&row, "position"),
            objects,
            created: string(&row, "created"),
            query,
        }))
    }

    /// Cards in a space, for hydrating its grid (`spaceCards`).
    ///
    /// # Arguments
    /// * `space_id` - Space id.
    ///
    /// # Returns
    /// Member cards in space order, trashed ones included.
    ///
    /// # Errors
    /// The database's error.
    pub fn space_cards(&self, space_id: &str) -> Result<Vec<CardRow>> {
        self.cards(
            &format!(
                "SELECT {} FROM space_cards s JOIN cards c ON c.id = s.card_id WHERE s.space_id = ? AND s.excluded = 0 ORDER BY s.rank",
                card_columns("c.")
            ),
            &[space_id.into()],
        )
    }

    /// Manual tags as `GET /tags/manual` returns them (`tagsManual`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// One entry per manual tag name, most-used first.
    ///
    /// # Errors
    /// The database's error.
    pub fn tags_manual(&self, user_id: Option<&str>) -> Result<Vec<ManualTag>> {
        let user = self.resolve_user(user_id)?;
        let rows = self.db.query(
            &format!(
                "
        SELECT a.name AS name, COUNT(*) AS count,
               COALESCE(MAX(t.modified_ms), MAX(c.modified_ns) / 1000000) AS modified
        FROM annotations a
        JOIN cards c ON c.id = a.card_id AND c.deleted_ns IS NULL
        LEFT JOIN tag_names t ON t.user_id = a.user_id AND t.name = a.name
        WHERE a.user_id = ? AND (a.flags & {MANUAL_TAG_FLAG}) != 0
        GROUP BY a.name
        ORDER BY count DESC, COALESCE(MIN(t.manual_rank), 2147483647), a.name"
            ),
            &[user.into()],
        )?;
        Ok(rows
            .iter()
            .map(|row| {
                let modified = row.number("modified").unwrap_or(0.0);
                ManualTag {
                    name: string(row, "name"),
                    count: number(row, "count"),
                    flags: 0,
                    modified: if modified.abs() > 8.64e15 { f64::NAN } else { modified.trunc() },
                }
            })
            .collect())
    }

    /// Recently used manual tag names, most recently modified first (`tagsRecent`).
    ///
    /// # Arguments
    /// * `take` - Maximum names to return (`slice(0, take)`, so a negative count drops from the end).
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Errors
    /// The database's error.
    pub fn tags_recent(&self, take: i64, user_id: Option<&str>) -> Result<Vec<String>> {
        let tags = self.tags_manual(user_id)?;
        let mut sorted: Vec<ManualTag> = Vec::with_capacity(tags.len());
        for tag in tags {
            let mut at = sorted.len();
            while at > 0 && (tag.modified - sorted[at - 1].modified) > 0.0 {
                at -= 1;
            }
            sorted.insert(at, tag);
        }
        let length = sorted.len() as i64;
        let end = if take < 0 { (length + take).max(0) } else { take.min(length) };
        Ok(sorted.into_iter().take(end as usize).map(|tag| tag.name).collect())
    }

    fn max_ts(&self, sql: &str, user_id: Option<&str>) -> Result<String> {
        let user = self.resolve_user(user_id)?;
        let row = self.db.one(sql, &[user.into()])?;
        Ok(row.and_then(|row| row.text("ts").map(str::to_owned)).unwrap_or_else(|| "0".to_owned()))
    }

    /// Newest change timestamp, the `x-checkpoint` header value (`checkpoint`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// Nanoseconds since the epoch as decimal text, `"0"` when there are no changes.
    ///
    /// # Errors
    /// The database's error.
    pub fn checkpoint(&self, user_id: Option<&str>) -> Result<String> {
        self.max_ts(&format!("SELECT {} FROM changes WHERE user_id = ?", ns_column("MAX(ts_ns)", "ts")), user_id)
    }

    /// Newest non-deletion change, the grid's `x-checkpoint` header (`gridCheckpoint`).
    ///
    /// # Arguments
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// Nanoseconds since the epoch as decimal text, `"0"` when there are no such changes.
    ///
    /// # Errors
    /// The database's error.
    pub fn grid_checkpoint(&self, user_id: Option<&str>) -> Result<String> {
        self.max_ts(&format!("SELECT {} FROM changes WHERE user_id = ? AND type <> 2", ns_column("MAX(ts_ns)", "ts")), user_id)
    }

    /// Changes newer than a checkpoint, for `GET /changes?since=` (`changesSince`).
    ///
    /// # Arguments
    /// * `since` - Checkpoint in nanoseconds; only strictly newer changes are returned.
    /// * `options` - User, limit and coalescing.
    ///
    /// # Returns
    /// Changes oldest first, each with its card's current row when the object is a card that exists.
    ///
    /// # Errors
    /// The database's error.
    pub fn changes_since(&self, since: i128, options: &ChangesOptions<'_>) -> Result<Vec<Change>> {
        let user = self.resolve_user(options.user_id)?;
        let mut params: Vec<SqlValue> = vec![user.clone().into(), SqlValue::big(since)];
        let latest = if options.coalesce {
            params.push(user.clone().into());
            " AND seq IN (SELECT MAX(newer.seq) FROM changes newer WHERE newer.user_id = ?
        GROUP BY newer.object_type, newer.object_id)"
        } else {
            ""
        };
        params.push(options.limit.into());
        let rows = self.db.query(
            &format!(
                "SELECT seq, {}, type, object_type, object_id FROM changes
        WHERE user_id = ? AND ts_ns > ?{latest} ORDER BY changes.ts_ns, seq LIMIT ?",
                ns_column("ts_ns", "ts_ns")
            ),
            &params,
        )?;
        let objects: Vec<(i64, String)> = rows.iter().map(|row| (number(row, "object_type"), string(row, "object_id"))).collect();
        let card_ids: Vec<&str> = objects.iter().filter(|(kind, _)| *kind == 9).map(|(_, id)| id.as_str()).collect();
        let cards = self.cards_in(&card_ids, &user)?;
        let mut changes = Vec::with_capacity(rows.len());
        for (row, (object_type, object_id)) in rows.iter().zip(objects) {
            let card = if object_type == 9 { cards.get(&object_id).cloned() } else { None };
            changes.push(Change { seq: number(row, "seq"), ts: big(row, "ts_ns")?, kind: number(row, "type"), object_type, object_id, card });
        }
        Ok(changes)
    }

    /// Spaces that hold each card, in rank order.
    ///
    /// # Arguments
    /// * `card_ids` - Cards to look up; repeated ids are allowed.
    ///
    /// # Returns
    /// For every card in at least one space, the ids of the spaces that include it, lowest rank first.
    ///
    /// # Errors
    /// The database's error.
    pub fn card_spaces(&self, card_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        let mut seen = HashSet::new();
        let unique: Vec<&str> = card_ids.iter().copied().filter(|id| seen.insert(*id)).collect();
        let mut found: HashMap<String, Vec<String>> = HashMap::new();
        for chunk in unique.chunks(LOOKUP_CHUNK) {
            let params: Vec<SqlValue> = chunk.iter().map(|id| SqlValue::from(*id)).collect();
            let rows = self.db.query(
                &format!("SELECT card_id, space_id FROM space_cards WHERE card_id IN ({}) AND excluded = 0 ORDER BY rank, rowid", placeholders(chunk.len())),
                &params,
            )?;
            for row in &rows {
                found.entry(string(row, "card_id")).or_default().push(string(row, "space_id"));
            }
        }
        Ok(found)
    }

    /// The user's cards among `ids`, keyed by id, in as few statements as the parameter limit allows.
    fn cards_in(&self, ids: &[&str], user: &str) -> Result<HashMap<String, CardRow>> {
        let mut found = HashMap::with_capacity(ids.len());
        for chunk in ids.chunks(LOOKUP_CHUNK) {
            let mut params = vec![SqlValue::from(user)];
            params.extend(chunk.iter().map(|id| SqlValue::from(*id)));
            let sql = format!("SELECT {} FROM cards WHERE user_id = ? AND id IN ({})", card_columns(""), placeholders(chunk.len()));
            for card in self.cards(&sql, &params)? {
                found.insert(card.id.clone(), card);
            }
        }
        Ok(found)
    }

    /// Generate the next unused id of a kind: account prefix, 2-character kind code, 9 base62 characters (`newId`).
    ///
    /// # Arguments
    /// * `kind` - What the id is for.
    /// * `user_id` - User id; the first user when `None`.
    ///
    /// # Returns
    /// A 22-character id that sorts after every existing id of that kind.
    ///
    /// # Errors
    /// An invalid-data error when the user does not exist or the counter is exhausted.
    pub fn new_id(&self, kind: IdKind, user_id: Option<&str>) -> Result<String> {
        let user = self.user(user_id)?.ok_or_else(|| Error::invalid("Cannot read properties of null (reading 'id')"))?;
        let key = format!("{}:{}", user.id, kind.name());
        let (table, column) = kind.source();
        let prefix_length = text::len(&user.id_prefix);
        let row = self.db.one(
            &format!("SELECT MAX({column}) AS id FROM {table} WHERE user_id = ? AND substr({column}, ?, 2) = ?"),
            &[user.id.clone().into(), (prefix_length + 1).into(), kind.code().into()],
        )?;
        let persisted = match row.as_ref().and_then(|row| row.text("id")).filter(|id| !id.is_empty()) {
            Some(id) => decode_counter(&text::slice(id, prefix_length as i64 + 2, None)),
            None => 0,
        };
        let mut counters = self.counters.borrow_mut();
        let current = counters.entry(key).or_insert(persisted);
        if persisted > *current {
            *current = persisted;
        }
        let next = current.saturating_add(1);
        if next >= 62_i128.pow(COUNTER_WIDTH) {
            return Err(Error::invalid(format!("Id counter exhausted for {}", kind.name())));
        }
        *current = next;
        Ok(format!("{}{}{}", user.id_prefix, kind.code(), encode_counter(next)))
    }
}

fn cursor_params(after: Option<&Cursor>) -> (SqlValue, SqlValue) {
    after.map_or((SqlValue::Null, SqlValue::Null), |cursor| (SqlValue::big(cursor.ns), SqlValue::from(cursor.id.as_str())))
}
