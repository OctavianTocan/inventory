//! Usage analytics (`docs/analytics.md`): the one row layout every event is written in, the sink a library's host
//! installs to write them to Workers Analytics Engine, a library's daily totals, and the owner's bounded SQL questions.

use mm_js::{Object, Value, obj};

use crate::sql::{Sql, SqlExt};
use crate::{Error, Result};

/// The layout version written in `double1`. A new column is only ever added after the last one; nothing moves.
pub const LAYOUT_VERSION: f64 = 1.0;

/// Longest text kept in one column, in characters. Longer text is cut.
pub const TEXT_LIMIT: usize = 200;

/// Most rows an owner question returns.
pub const ROW_LIMIT: usize = 200;

/// The Analytics Engine dataset every event is written to (`wrangler.jsonc`).
pub const DATASET: &str = "thymind_events";

/// One thing that happened. Empty text and zero numbers mean "not given".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Event {
    /// What happened, for example `request`, `change`, `search` or `signin`.
    pub name: String,
    /// The screen it happened on (phone events).
    pub screen: String,
    /// The card it concerns.
    pub card_id: String,
    /// That card's kind.
    pub card_kind: String,
    /// The words searched for.
    pub words: String,
    /// More about it: the route, the change name, the feature or the error code.
    pub detail: String,
    /// Who sent it: `android`, `mcp`, `extension` or `web`.
    pub client: String,
    /// How many times it happened, when one event stands for several.
    pub count: f64,
    /// How many results it had.
    pub results: f64,
    /// How long it took, in milliseconds.
    pub duration_ms: f64,
    /// The HTTP status of the answer.
    pub status: f64,
}

impl Event {
    /// A new event with only its name.
    ///
    /// # Arguments
    /// * `name` - What happened.
    ///
    /// # Returns
    /// The event, every other field empty.
    pub fn named(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Self::default()
        }
    }

    /// The same event with `detail` set.
    ///
    /// # Arguments
    /// * `detail` - More about what happened.
    ///
    /// # Returns
    /// The changed event.
    pub fn detail(mut self, detail: &str) -> Self {
        detail.clone_into(&mut self.detail);
        self
    }

    /// The same event with `card_id` set.
    ///
    /// # Arguments
    /// * `card` - The card's id.
    ///
    /// # Returns
    /// The changed event.
    pub fn card(mut self, card: &str) -> Self {
        card.clone_into(&mut self.card_id);
        self
    }
}

/// One Analytics Engine data point, in the order `writeDataPoint` takes it.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    /// `index1`: a short key for the library ([`index_of`]), so sampling keeps each account apart.
    pub index: String,
    /// `blob1`…`blob10`.
    pub blobs: Vec<String>,
    /// `double1`…`double6`.
    pub doubles: Vec<f64>,
}

/// Where each field of an [`Event`] lands. Positions never move; new ones are added at the end.
///
/// | Column | Meaning |
/// |---|---|
/// | index1 | library key ([`index_of`]) |
/// | blob1 | event name |
/// | blob2 | app version |
/// | blob3 | screen |
/// | blob4 | card id |
/// | blob5 | card kind |
/// | blob6 | search words |
/// | blob7 | detail |
/// | blob8 | batch id |
/// | blob9 | client |
/// | blob10 | account email |
/// | double1 | layout version |
/// | double2 | count |
/// | double3 | result count |
/// | double4 | duration in ms |
/// | double5 | HTTP status |
pub const LAYOUT: &str = "index1 = library key (first 32 hex of the SHA-256 of the library name); blob1 = event name; blob2 = app version; blob3 = screen; \
blob4 = card id; blob5 = card kind; blob6 = search words; blob7 = detail (route, change name, feature or error); \
blob8 = batch id; blob9 = client (android, mcp, extension, web); blob10 = account email; double1 = layout version; \
double2 = count; double3 = result count; double4 = duration in ms; double5 = HTTP status";

/// Who an event is about and where it came from, the same for every event of one request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Origin {
    /// The library name.
    pub library: String,
    /// The account's email.
    pub email: String,
    /// The app version, when the client sent one.
    pub version: String,
    /// The batch the phone sent it in.
    pub batch: String,
}

/// The `index1` key of a library: Analytics Engine allows 96 bytes and library names can be longer.
///
/// # Arguments
/// * `library` - The library name.
///
/// # Returns
/// The first 32 hex digits of the SHA-256 of the name.
pub fn index_of(library: &str) -> String {
    use sha2::{Digest, Sha256};
    mm_js::base64::hex(&Sha256::digest(library.as_bytes()))[..32].to_owned()
}

fn cut(text: &str) -> String {
    text.chars().take(TEXT_LIMIT).collect()
}

/// Lay `event` out as the data point written for it.
///
/// # Arguments
/// * `event` - What happened.
/// * `origin` - Whose it is and where it came from.
///
/// # Returns
/// The point, every text cut to [`TEXT_LIMIT`] characters.
pub fn point(event: &Event, origin: &Origin) -> Point {
    let blobs = [
        &event.name,
        &origin.version,
        &event.screen,
        &event.card_id,
        &event.card_kind,
        &event.words,
        &event.detail,
        &origin.batch,
        &event.client,
        &origin.email,
    ];
    Point {
        index: index_of(&origin.library),
        blobs: blobs.into_iter().map(|text| cut(text)).collect(),
        doubles: vec![
            LAYOUT_VERSION,
            event.count,
            event.results,
            event.duration_ms,
            event.status,
        ],
    }
}

/// Where a library sends its events (`Store::set_analytics`): the host writes each one to Analytics Engine.
pub type Sink = Box<dyn Fn(&Event)>;

/// Which client sent a request, from its user agent.
///
/// # Arguments
/// * `user_agent` - The request's `user-agent` header.
///
/// # Returns
/// `mcp` for ThyMind's MCP server, `android` for the app, `extension` for the browser extension, otherwise `web`.
pub fn client_of(user_agent: &str) -> &'static str {
    let agent = user_agent.to_ascii_lowercase();
    if agent.contains("thymind-mcp") {
        "mcp"
    } else if agent.contains("okhttp")
        || agent.contains("thymind-android")
        || agent.contains("ktor")
    {
        "android"
    } else if agent.contains("thymind-extension") {
        "extension"
    } else {
        "web"
    }
}

/// The route family of an API path, so one kind of request counts as one detail however its ids vary.
///
/// # Arguments
/// * `path` - The request's path.
///
/// # Returns
/// The path with every segment after the route name dropped (`/app/v1/cards/abc` gives `app/v1/cards`), or `None`
/// for paths that are not API requests (pages, media, the hub).
pub fn route_of(path: &str) -> Option<String> {
    let parts: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    match parts.as_slice() {
        ["app", "v1", name, ..] => Some(format!("app/v1/{name}")),
        ["mcp", ..] => None,
        [first, ..]
            if [
                "api", "objects", "spaces", "tags", "ai", "search", "upload", "uploads",
            ]
            .contains(first) =>
        {
            Some((*first).to_owned())
        }
        _ => None,
    }
}

/// Check that an owner question is one read-only `SELECT` and give it a row cap.
///
/// # Arguments
/// * `sql` - The question as Analytics Engine SQL.
///
/// # Returns
/// The statement without a trailing `;`, with `LIMIT 200` added when it has no `LIMIT`.
///
/// # Errors
/// 400 when it is empty, is not a `SELECT` (or `WITH … SELECT`), holds more than one statement, or asks for more
/// than [`ROW_LIMIT`] rows.
pub fn bounded_select(sql: &str) -> Result<String> {
    let statement = sql.trim().trim_end_matches(';').trim();
    let lower = statement.to_ascii_lowercase();
    if statement.is_empty() || !(lower.starts_with("select") || lower.starts_with("with")) {
        return Err(Error::status(400, "Only one SELECT question is allowed."));
    }
    if statement.contains(';') {
        return Err(Error::status(400, "Only one statement is allowed."));
    }
    let words: Vec<&str> = lower.split_whitespace().collect();
    match words.iter().rposition(|word| *word == "limit") {
        Some(at) => {
            let rows = words
                .get(at + 1)
                .and_then(|word| word.trim_end_matches(')').parse::<usize>().ok());
            if rows.is_none_or(|rows| rows > ROW_LIMIT) {
                return Err(Error::status(
                    400,
                    format!("LIMIT must be a number up to {ROW_LIMIT}."),
                ));
            }
            Ok(statement.to_owned())
        }
        None => Ok(format!("{statement} LIMIT {ROW_LIMIT}")),
    }
}

/// Shorten an Analytics Engine SQL answer for an agent: at most [`ROW_LIMIT`] rows, every text cut to
/// [`TEXT_LIMIT`] characters.
///
/// # Arguments
/// * `answer` - The SQL API's JSON answer (`{meta, data, rows, …}`).
///
/// # Returns
/// `{columns, rows, total, cut}`: the column names, the kept rows, how many rows the answer had and whether any were
/// dropped.
pub fn shorten(answer: &Value) -> Value {
    let columns: Vec<Value> = answer
        .get("meta")
        .as_array()
        .into_iter()
        .flatten()
        .map(|column| column.get("name").clone())
        .collect();
    let data = answer.get("data").as_array().cloned().unwrap_or_default();
    let rows: Vec<Value> = data
        .iter()
        .take(ROW_LIMIT)
        .map(|row| match row {
            Value::Object(fields) => {
                let mut kept = Object::new();
                for (name, value) in fields.iter() {
                    kept.insert(
                        name.into_owned(),
                        match value {
                            Value::String(text) => Value::from(cut(text)),
                            other => other.clone(),
                        },
                    );
                }
                Value::Object(kept)
            }
            other => other.clone(),
        })
        .collect();
    obj! { "columns" => columns, "total" => data.len() as f64, "cut" => data.len() > ROW_LIMIT, "rows" => rows }.into()
}

fn count(db: &dyn Sql, sql: &str) -> Result<f64> {
    Ok(db
        .one(sql, &[])?
        .and_then(|row| row.number("n"))
        .unwrap_or(0.0))
}

/// What one library holds today, counts only.
///
/// # Arguments
/// * `db` - The library's database.
///
/// # Returns
/// `{cards, trashed, notes, spaces, tags, pinned, shared, uploadedBytes, kinds: {<type>: n}, lastSavedNs}`. Cards and
/// kinds leave out trashed cards.
///
/// # Errors
/// Any database failure.
pub fn library_totals(db: &dyn Sql) -> Result<Value> {
    let mut kinds = Object::new();
    for row in db.query("SELECT type, COUNT(*) AS n FROM cards WHERE deleted_ns IS NULL GROUP BY type ORDER BY type", &[])? {
        kinds.insert(row.text("type").unwrap_or_default().to_owned(), row.number("n").unwrap_or(0.0));
    }
    let last = db
        .one("SELECT MAX(created_ns) AS n FROM cards", &[])?
        .and_then(|row| row.number("n"));
    Ok(obj! {
        "cards" => count(db, "SELECT COUNT(*) AS n FROM cards WHERE deleted_ns IS NULL")?,
        "trashed" => count(db, "SELECT COUNT(*) AS n FROM cards WHERE deleted_ns IS NOT NULL")?,
        "notes" => count(db, "SELECT COUNT(*) AS n FROM cards WHERE deleted_ns IS NULL AND json_type(data, '$.note.prose') IS NOT NULL")?,
        "spaces" => count(db, "SELECT COUNT(*) AS n FROM spaces")?,
        "tags" => count(db, "SELECT COUNT(*) AS n FROM tag_names")?,
        "pinned" => count(db, "SELECT COUNT(*) AS n FROM pins")?,
        "shared" => count(db, "SELECT COUNT(*) AS n FROM shares")?,
        "uploadedBytes" => count(db, "SELECT COALESCE(SUM(size), 0) AS n FROM uploads WHERE done = 1")?,
        "kinds" => kinds,
        "lastSavedNs" => last.map_or(Value::Null, Value::from),
    }
    .into())
}

const DAILY: &str = "CREATE TABLE IF NOT EXISTS analytics_daily (day TEXT NOT NULL, library TEXT NOT NULL, account TEXT NOT NULL,
  totals TEXT NOT NULL CHECK (json_valid(totals)), PRIMARY KEY (day, library)) STRICT";

/// Keep one library's totals for one day. Writing the same day again replaces it, so a retried run does no harm.
///
/// # Arguments
/// * `db` - The sign-in database (the `Auth` object), which keeps these rows with no end date.
/// * `day` - The UTC day, `YYYY-MM-DD`.
/// * `library` - The library name.
/// * `account` - The account it belongs to.
/// * `totals` - [`library_totals`] for it, or `{error}` when it could not be read.
///
/// # Errors
/// Any database failure, or totals that are not JSON.
pub fn put_daily(
    db: &dyn Sql,
    day: &str,
    library: &str,
    account: &str,
    totals: &Value,
) -> Result<()> {
    db.exec_batch(DAILY)?;
    let totals = mm_js::json::stringify(totals)?.unwrap_or_else(|| "{}".to_owned());
    db.execute(
        "INSERT INTO analytics_daily (day, library, account, totals) VALUES (?, ?, ?, ?)
         ON CONFLICT (day, library) DO UPDATE SET account = excluded.account, totals = excluded.totals",
        &[day.into(), library.into(), account.into(), totals.into()],
    )?;
    Ok(())
}

/// The kept daily totals, newest day first.
///
/// # Arguments
/// * `db` - The sign-in database.
/// * `since` - The first UTC day to include, `YYYY-MM-DD`.
/// * `library` - Only this library, or every library when empty.
///
/// # Returns
/// `{days: [{day, library, account, totals}]}`, at most 1000 rows.
///
/// # Errors
/// Any database failure.
pub fn daily(db: &dyn Sql, since: &str, library: &str) -> Result<Value> {
    db.exec_batch(DAILY)?;
    let rows = db.query(
        "SELECT day, library, account, totals FROM analytics_daily WHERE day >= ? AND (? = '' OR library = ?) ORDER BY day DESC, account LIMIT 1000",
        &[since.into(), library.into(), library.into()],
    )?;
    let mut days = Vec::with_capacity(rows.len());
    for row in rows {
        days.push(
            obj! {
                "day" => row.text("day").unwrap_or_default(),
                "library" => row.text("library").unwrap_or_default(),
                "account" => row.text("account").unwrap_or_default(),
                "totals" => mm_js::json::parse(row.text("totals").unwrap_or("{}"))?,
            }
            .into(),
        );
    }
    Ok(obj! { "days" => days }.into())
}

/// Whether analytics is on: the `ANALYTICS` variable turns it off with `off`.
///
/// # Arguments
/// * `value` - The variable's value, if set.
///
/// # Returns
/// `false` only for `off`, `0` or `false`.
pub fn enabled(value: Option<&str>) -> bool {
    !matches!(value.map(str::trim), Some("off" | "0" | "false"))
}
