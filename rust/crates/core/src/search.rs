//! Port of `lib/search.mjs`: text, smart-space, suggestion and semantic search over the visible library.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::LazyLock;

use mm_js::{Value, date, locale_compare, number, obj, text, url};
use regex::Regex;

use crate::host::Fetch;
use crate::runtime;
use crate::search_index::refresh_search;
use crate::search_query::{Parsed, Scores, Term, evaluate, parse_query, type_slugs};
use crate::search_semantic::{Eligible, Match, Semantic};
use crate::sql::SqlValue;
use crate::store::{GRID_HIDDEN_FLAGS, Store};
use crate::store_query::{SpaceMatch, SpaceSearch};
use crate::{Error, Result};

const LABELS: [(&str, &str); 17] = [
    ("XPost", "X Posts"),
    ("WebPage", "Web Pages"),
    ("Repository", "Repositories"),
    ("Screenshot", "Screenshots"),
    ("Article", "Articles"),
    ("Image", "Images"),
    ("Video", "Videos"),
    ("InstagramReel", "Instagram Reels"),
    ("Note", "Notes"),
    ("YouTubeVideo", "YouTube Videos"),
    ("Product", "Products"),
    ("ThreadsPost", "Threads Posts"),
    ("SoftwareApplication", "Apps"),
    ("Document", "Documents"),
    ("Content", "Snippets"),
    ("RedditPost", "Reddit Posts"),
    ("SubstackNote", "Substack Notes"),
];
const TEXT_FIELDS: [&str; 9] = ["title", "description", "summary", "text", "body", "prose", "note", "ocr", "url"];
const JS_SPACE: &str = r"[\t\n\x{b}\x{c}\r \x{a0}\x{1680}\x{2000}-\x{200a}\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}\x{feff}]";
const DAY_NS: i128 = 86_400_000_000_000;
const NS_PER_MS: i128 = 1_000_000;

static LETTER_OR_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]").expect("pattern"));
static RANGE: LazyLock<Regex> = LazyLock::new(|| {
    let any = r"[^\n\r\x{2028}\x{2029}]";
    Regex::new(&format!(r"(?i)^\[({any}+?){JS_SPACE}+TO{JS_SPACE}+({any}+?)\]$")).expect("pattern")
});

/// The answer to a search (`{elapsed, count, query, matches}`).
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    /// Milliseconds the search took, rounded.
    pub elapsed: f64,
    /// Number of matches.
    pub count: usize,
    /// The normalized query text.
    pub query: String,
    /// Matching cards in result order.
    pub matches: Vec<Match>,
}

impl SearchResult {
    /// The object the JS returned.
    pub fn to_value(&self) -> Value {
        obj! {
            "elapsed" => self.elapsed,
            "count" => self.count as f64,
            "query" => self.query.as_str(),
            "matches" => self.matches.iter().map(Match::to_value).collect::<Vec<_>>(),
        }
        .into()
    }
}

/// How [`Search::hybrid`] blends similarity to the query into word search.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hybrid {
    /// Weight of a card's similarity to the query, added to its word score.
    pub boost: f64,
    /// Least similarity for a card that matches no word of the query.
    pub floor: f64,
    /// Most cards that match no word of the query, nearest first.
    pub take: usize,
}

impl Default for Hybrid {
    /// Boost 2 as in [`Search::search`], floor 0.6 and at most 20 cards found by meaning alone.
    fn default() -> Self {
        Self { boost: 2.0, floor: 0.6, take: 20 }
    }
}

/// One search suggestion: a card type or a tag.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// `type` or `tag`.
    pub kind: &'static str,
    /// Label shown to the user.
    pub text: String,
    /// Query that selects the suggestion.
    pub query_text: String,
    /// Number of cards.
    pub count: f64,
    /// Ranking weight.
    pub score: f64,
    /// Whether an AI produced the tag.
    pub is_ai: bool,
}

impl Suggestion {
    /// The object the JS returned.
    pub fn to_value(&self) -> Value {
        obj! {
            "type" => self.kind,
            "text" => self.text.as_str(),
            "queryText" => self.query_text.as_str(),
            "count" => self.count,
            "score" => self.score,
            "isAI" => self.is_ai,
        }
        .into()
    }
}

struct Row {
    id: String,
    bumped: i128,
    created: i128,
    modified: i128,
}

/// Search over a store's visible library (`createSearch`'s result).
pub struct Search {
    store: Rc<Store>,
    user: String,
    semantic: Semantic,
}

// The query's terms without a field, the text that semantic search embeds.
fn free_words(parsed: &Parsed) -> String {
    parsed.terms.iter().filter(|term| term.field.is_empty()).map(|term| term.value.as_str()).collect::<Vec<_>>().join(" ")
}

fn visible() -> String {
    format!("c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0")
}

fn fts_term(term: &Term) -> Option<String> {
    if !LETTER_OR_NUMBER.is_match(&term.value) {
        return None;
    }
    let column = match term.field.as_str() {
        "" => String::new(),
        "tag" | "label" => "tags : ".to_owned(),
        "text" | "body" | "prose" => "body : ".to_owned(),
        other => format!("{other} : "),
    };
    Some(format!("{column}\"{}\"{}", term.value.replace('"', "\"\""), if term.prefix { "*" } else { "" }))
}

fn date_value(value: &str) -> Option<i128> {
    let bytes = value.as_bytes();
    let plain = bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| if index == 4 || index == 7 { *byte == b'-' } else { byte.is_ascii_digit() });
    let ms = date::parse(&if plain { format!("{value}T00:00:00Z") } else { value.to_owned() });
    ms.is_finite().then(|| ms as i128 * NS_PER_MS)
}

fn strip_prefix_ignore_case<'v>(value: &'v str, prefix: &str) -> Option<&'v str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then(|| &value[prefix.len()..])
}

fn parse_sort(sort: &str) -> Option<(&'static str, bool)> {
    let field = ["bumped", "created", "modified"].into_iter().find(|name| strip_prefix_ignore_case(sort, name).is_some())?;
    let rest = &sort[field.len()..];
    if rest.is_empty() {
        return Some((field, false));
    }
    let direction = rest.trim_start_matches(text::is_whitespace);
    if direction.len() == rest.len() {
        return None;
    }
    match direction.to_ascii_lowercase().as_str() {
        "asc" => Some((field, true)),
        "desc" => Some((field, false)),
        _ => None,
    }
}

fn sign(value: f64) -> Option<std::cmp::Ordering> {
    if value.is_nan() || value == 0.0 { None } else if value > 0.0 { Some(std::cmp::Ordering::Greater) } else { Some(std::cmp::Ordering::Less) }
}

fn spaced_type(kind: &str) -> String {
    let mut out = String::new();
    let mut previous = ' ';
    for character in kind.chars() {
        if previous.is_ascii_lowercase() && character.is_ascii_uppercase() {
            out.push(' ');
        }
        out.push(character);
        previous = character;
    }
    out
}

fn host_of(value: &str) -> String {
    let fallback = value.to_lowercase();
    let fallback = fallback.strip_prefix("www.").unwrap_or(&fallback).to_owned();
    let address = if value.contains("://") { value.to_owned() } else { format!("https://{value}") };
    match url::parse(&address, None) {
        Some(parsed) => {
            let host = parsed.host_str().unwrap_or("");
            host.strip_prefix("www.").unwrap_or(host).to_owned()
        }
        None => fallback,
    }
}

impl Search {
    /// Create the search over a store with an installed search index (`createSearch`).
    ///
    /// # Arguments
    /// * `store` - Open store.
    /// * `fetch` - Outbound HTTP for query embeddings; `None` disables semantic matching.
    /// * `offline` - Allow only explicitly configured loopback embedding stubs.
    ///
    /// # Errors
    /// An internal error when the store has no user.
    pub fn new(store: Rc<Store>, fetch: Option<Rc<dyn Fetch>>, offline: bool) -> Result<Self> {
        Ok(Self { user: store.default_user_id()?, semantic: Semantic::new(store.clone(), fetch, offline), store })
    }

    fn universe(&self) -> Result<Vec<Row>> {
        let db = self.store.db();
        refresh_search(&**db)?;
        let big = |row: &crate::sql::Row, column: &str| -> Result<i128> {
            row.text(column).and_then(|digits| digits.parse().ok()).ok_or_else(|| Error::invalid(format!("Cannot convert {column} to a BigInt")))
        };
        db.query(
            &format!(
                "SELECT c.id, CAST(c.bumped_ns AS TEXT) AS bumped_ns, CAST(c.created_ns AS TEXT) AS created_ns,
      CAST(c.modified_ns AS TEXT) AS modified_ns FROM cards c WHERE {}",
                visible()
            ),
            &[self.user.as_str().into()],
        )?
        .iter()
        .map(|row| Ok(Row { id: row.text("id").unwrap_or("").to_owned(), bumped: big(row, "bumped_ns")?, created: big(row, "created_ns")?, modified: big(row, "modified_ns")? }))
        .collect()
    }

    fn select_ids(&self, condition: &str, mut args: Vec<SqlValue>) -> Result<Vec<String>> {
        args.insert(0, self.user.as_str().into());
        Ok(self
            .store
            .db()
            .query(&format!("SELECT c.id FROM cards c WHERE {} AND ({condition})", visible()), &args)?
            .iter()
            .map(|row| row.text("id").unwrap_or("").to_owned())
            .collect())
    }

    fn predicate(&self, term: &Term) -> Result<Scores> {
        let (field, value, prefix) = (term.field.as_str(), term.value.as_str(), term.prefix);
        let mut sql = "0".to_owned();
        let mut args: Vec<SqlValue> = Vec::new();
        if field == "tag" || field == "label" {
            let compare = if prefix { "substr(lower(a.name), 1, length(?)) = lower(?)" } else { "lower(a.name) = lower(?)" };
            sql = format!("EXISTS (SELECT 1 FROM annotations a WHERE a.card_id = c.id AND {compare})");
            args = if prefix { vec![value.into(), value.into()] } else { vec![value.into()] };
        } else if field == "type" {
            let wanted = text::to_lower(value);
            let mut types = Vec::new();
            for row in self.store.db().query("SELECT DISTINCT type FROM cards WHERE user_id = ?", &[self.user.as_str().into()])? {
                let kind = row.text("type").unwrap_or("").to_owned();
                if type_slugs(&kind)?.iter().any(|slug| if prefix { slug.starts_with(&wanted) } else { *slug == wanted }) {
                    types.push(kind);
                }
            }
            sql = format!("c.type IN ({})", if types.is_empty() { "NULL".to_owned() } else { vec!["?"; types.len()].join(",") });
            args = types.into_iter().map(SqlValue::from).collect();
        } else if field == "site" || field == "domain" {
            let host = host_of(value);
            let domain = "lower(COALESCE(json_extract(c.data, '$.domain'), ''))";
            sql = format!("{domain} = ? OR {domain} LIKE ? ESCAPE '\\'");
            let escaped: String = host.chars().flat_map(|c| if matches!(c, '\\' | '%' | '_') { vec!['\\', c] } else { vec![c] }).collect();
            args = vec![host.as_str().into(), format!("%.{escaped}").into()];
            if !host.contains('.') {
                sql.push_str(&format!(" OR instr('.' || {domain} || '.', ?) > 0"));
                args.push(format!(".{host}.").into());
            }
        } else if field == "format" {
            sql = "lower(COALESCE(json_extract(c.data, '$.format'), '')) = lower(?)".to_owned();
            args = vec![value.into()];
        } else if field == "completed" {
            sql = format!("(c.flags & 65536) {} 0", if value == "true" { "!=" } else { "=" });
        } else if field == "action" {
            let group: &[&str] = match text::to_lower(value).as_str() {
                "read" => &["Article", "Book", "Document", "WebPage", "Content"],
                "watch" => &["Video", "YouTubeVideo", "VimeoVideo", "InstagramReel", "Movie", "TVEpisode", "TVSeries", "TedTalk"],
                "listen" => &["Audio", "Podcast", "PodcastEpisode", "MusicRecording", "MusicAlbum", "MusicPlaylist"],
                "play" => &["Game", "VideoGame"],
                "constructor" | "__proto__" => return Err(Error::invalid("args.map is not a function")),
                _ => &[],
            };
            sql = format!("c.type IN ({})", if group.is_empty() { "NULL".to_owned() } else { vec!["?"; group.len()].join(",") });
            args = group.iter().map(|name| SqlValue::from(*name)).collect();
        } else if ["before", "after", "date", "created", "bumped", "modified"].contains(&field) {
            let column = if field == "bumped" || field == "modified" { format!("c.{field}_ns") } else { "c.created_ns".to_owned() };
            if let Some(range) = RANGE.captures(value) {
                let start = if &range[1] == "*" { Some(0) } else { date_value(&range[1]) };
                let end = if &range[2] == "*" { Some(i128::from(i64::MAX)) } else { date_value(&range[2]) };
                if let (Some(start), Some(end)) = (start, end) {
                    sql = format!("{column} >= ? AND {column} <= ?");
                    args = vec![SqlValue::big(start), SqlValue::big(end)];
                }
            } else {
                let op = [">=", "<=", ">", "<"].into_iter().find(|op| value.starts_with(op));
                if let Some(date) = date_value(&value[op.map_or(0, str::len)..]) {
                    let default = if field == "before" { "<" } else if field == "after" { ">" } else { ">=" };
                    sql = format!("{column} {} ?", op.unwrap_or(default));
                    args = vec![SqlValue::big(date)];
                    if op.is_none() && field != "before" && field != "after" {
                        sql.push_str(&format!(" AND {column} < ?"));
                        args.push(SqlValue::big(date + DAY_NS));
                    }
                }
            }
        }
        Ok(self.select_ids(&sql, args)?.into_iter().map(|id| (id, term.boost)).collect())
    }

    fn text_matches(&self, term: &Term) -> Result<Scores> {
        let Some(expression) = fts_term(term) else { return Ok(Scores::new()) };
        let rows = self.store.db().query(
            &format!(
                "SELECT d.id, bm25(search_fts, 4, 2, 1, 4, 1, 2, 2, 2) AS rank
          FROM search_fts JOIN search_documents d ON d.rowid = search_fts.rowid
          JOIN cards c ON c.id = d.id WHERE search_fts MATCH ? AND {}",
                visible()
            ),
            &[expression.as_str().into(), self.user.as_str().into()],
        )?;
        Ok(rows.iter().map(|row| (row.text("id").unwrap_or("").to_owned(), (1.0 - row.number("rank").unwrap_or(0.0)) * term.boost)).collect())
    }

    fn matches(&self, parsed: &crate::search_query::Parsed, rows: &[Row], candidates: bool) -> Result<Scores> {
        let all: Scores = rows.iter().map(|row| (row.id.clone(), 1.0)).collect();
        evaluate(
            &parsed.tree,
            &parsed.terms,
            &all,
            &mut |term, negative| {
                if candidates && !negative && term.field.is_empty() {
                    return Ok(all.clone());
                }
                if !term.field.is_empty() && !TEXT_FIELDS.contains(&term.field.as_str()) { self.predicate(term) } else { self.text_matches(term) }
            },
            false,
        )
    }

    fn ordered(scores: &Scores, rows: &[Row], sort_by: &Value) -> Vec<Match> {
        let metadata: HashMap<&str, &Row> = rows.iter().map(|row| (row.id.as_str(), row)).collect();
        let sort_text = if sort_by.is_undefined() { "score".to_owned() } else { sort_by.to_js_string() };
        let sort = parse_sort(&sort_text);
        let key = |row: &Row| match sort.map_or("bumped", |(field, _)| field) {
            "created" => row.created,
            "modified" => row.modified,
            _ => row.bumped,
        };
        let mut items: Vec<(Match, i128)> = scores
            .iter()
            .filter_map(|(id, score)| metadata.get(id.as_str()).map(|row| (Match { id: id.clone(), score: *score }, key(row))))
            .collect();
        items.sort_by(|(a, x), (b, y)| {
            let mut date_order = if x > y { -1.0 } else if x < y { 1.0 } else { 0.0 };
            if sort.is_some_and(|(_, ascending)| ascending) {
                date_order = -date_order;
            }
            let primary = if sort.is_some() {
                date_order
            } else {
                let difference = b.score - a.score;
                if difference != 0.0 && !difference.is_nan() { difference } else { date_order }
            };
            sign(primary).unwrap_or_else(|| locale_compare(&b.id, &a.id))
        });
        items.into_iter().map(|(item, _)| item).collect()
    }

    fn result(started: f64, query: String, matches: Vec<Match>) -> SearchResult {
        SearchResult { elapsed: number::round(runtime::now_ms() - started), count: matches.len(), query, matches }
    }

    /// Synchronous text search, also the smart-space evaluator (`text`).
    ///
    /// # Arguments
    /// * `q` - Query text.
    /// * `options` - Object with `wildcard` (only `true` counts) and `sortBy`.
    ///
    /// # Errors
    /// Status 400 for an over-long query, or the database's error.
    pub fn text(&self, q: &str, options: &Value) -> Result<SearchResult> {
        let started = runtime::now_ms();
        let rows = self.universe()?;
        let parsed = parse_query(q, matches!(options.get("wildcard"), Value::Bool(true)))?;
        let found = Self::ordered(&self.matches(&parsed, &rows, false)?, &rows, options.get("sortBy"));
        Ok(Self::result(started, parsed.query, found))
    }

    /// Text search blended with semantic candidates (`search`).
    ///
    /// # Arguments
    /// * `q` - Query text; `similar:<id>` delegates to [`Search::related`].
    /// * `options` - Object with `wildcard`, `semantic` (only `true` counts), `semanticBoost` and `sortBy`.
    ///
    /// # Errors
    /// Status 400 for an over-long query, or the database's error.
    pub async fn search(&self, q: &str, options: &Value) -> Result<SearchResult> {
        let boost = if matches!(options.get("semantic"), Value::Bool(true)) {
            let requested = options.get("semanticBoost").to_number();
            if requested.is_finite() { requested.clamp(0.0, 100.0) } else { 2.0 }
        } else {
            0.0
        };
        self.blend(q, options, boost, None).await
    }

    /// The app's search: words, plus cards close in meaning to the query's free words. A card that matches a word
    /// gains `boost` times its similarity; a card that matches none joins only above `floor`, at most `take` of them.
    /// Query embeddings are cached, so paging asks Gemini once per query.
    ///
    /// # Arguments
    /// * `q` - Query text in mymind's syntax; `similar:<id>` delegates to [`Search::related`].
    /// * `tuning` - The blend.
    ///
    /// # Returns
    /// Matches by score descending; word matches only when embeddings are disabled or the embedding call fails.
    ///
    /// # Errors
    /// Status 400 for an over-long query, or the database's error.
    pub async fn hybrid(&self, q: &str, tuning: &Hybrid) -> Result<SearchResult> {
        self.blend(q, &Value::Undefined, tuning.boost.clamp(0.0, 100.0), Some(tuning)).await
    }

    /// Every card with a vector, by similarity to the query's free words alone: the half of [`Search::hybrid`] that
    /// is not word search, sharing its query cache.
    ///
    /// # Arguments
    /// * `q` - Query text in mymind's syntax; field terms such as `type:` are ignored.
    ///
    /// # Returns
    /// Matches by similarity descending; empty when the query has no free words, embeddings are disabled or the
    /// embedding call fails.
    ///
    /// # Errors
    /// Status 400 for an over-long query, or the database's error.
    pub async fn meaning(&self, q: &str) -> Result<SearchResult> {
        let started = runtime::now_ms();
        let parsed = parse_query(q, false)?;
        let found = self.semantic.search_cached(&free_words(&parsed), &Value::Number(-1.0), Eligible::Any).await?;
        Ok(Self::result(started, parsed.query, found))
    }

    // Word scores plus `boost` times similarity; `join` limits the cards that match no word and caches the query.
    async fn blend(&self, q: &str, options: &Value, boost: f64, join: Option<&Hybrid>) -> Result<SearchResult> {
        if let Some(id) = q.strip_prefix("similar:") {
            return self.related(text::trim(id), options);
        }
        let started = runtime::now_ms();
        let parsed = parse_query(q, matches!(options.get("wildcard"), Value::Bool(true)))?;
        let mut rows = self.universe()?;
        let mut scores = self.matches(&parsed, &rows, false)?;
        let words = free_words(&parsed);
        if boost > 0.0 && !words.is_empty() {
            let similar = match join {
                Some(_) => self.semantic.search_cached(&words, &Value::Number(-1.0), Eligible::Any).await?,
                None => self.semantic.search(&words, &Value::Number(-1.0), Eligible::Any).await?,
            };
            rows = self.universe()?;
            scores = self.matches(&parsed, &rows, false)?;
            let eligible = self.matches(&parsed, &rows, true)?;
            let mut joined = 0;
            for found in similar {
                if !eligible.contains_key(&found.id) || found.score <= 0.0 {
                    continue;
                }
                let current = scores.get(&found.id).copied();
                if let (None, Some(limits)) = (current, join) {
                    if joined >= limits.take || found.score < limits.floor {
                        continue;
                    }
                    joined += 1;
                }
                scores.insert(found.id, current.unwrap_or(0.0) + boost * found.score);
            }
        }
        Ok(Self::result(started, parsed.query, Self::ordered(&scores, &rows, options.get("sortBy"))))
    }

    /// Cards matching the intersection of client query bubbles (`filters`).
    ///
    /// # Arguments
    /// * `filters` - Array of query strings.
    ///
    /// # Errors
    /// Status 400 `Expected query filters` unless `filters` is an array of strings, or the search's error.
    pub fn filters(&self, filters: &Value) -> Result<Vec<Match>> {
        let Some(items) = filters.as_array().filter(|items| items.iter().all(Value::is_string)) else {
            return Err(Error::status(400, "Expected query filters"));
        };
        let query = items.iter().map(|item| format!("({})", item.to_js_string())).collect::<Vec<_>>().join(" AND ");
        Ok(self.text(if items.is_empty() { "" } else { &query }, &Value::Undefined)?.matches)
    }

    /// Type suggestions for a blank query, otherwise tag completions (`suggestions`).
    ///
    /// # Arguments
    /// * `q` - Query text.
    ///
    /// # Returns
    /// Suggestions by score descending, then text.
    ///
    /// # Errors
    /// The database's error.
    pub fn suggestions(&self, q: &str) -> Result<Vec<Suggestion>> {
        let db = self.store.db();
        let mut out = Vec::new();
        if text::trim(q).is_empty() {
            let rows = db.query(&format!("SELECT c.type, COUNT(*) AS count FROM cards c WHERE {} AND c.type != 'Placeholder' GROUP BY c.type", visible()), &[self.user.as_str().into()])?;
            for row in &rows {
                let kind = row.text("type").unwrap_or("");
                let count = row.number("count").unwrap_or(0.0);
                let label = LABELS.iter().find(|(name, _)| *name == kind).map_or_else(|| format!("{}s", spaced_type(kind)), |(_, label)| (*label).to_owned());
                let slugs = type_slugs(kind)?;
                out.push(Suggestion { kind: "type", text: label, query_text: format!("type:{}", slugs.first().map_or("undefined", String::as_str)), count, score: count * 3.0, is_ai: false });
            }
        } else {
            let trimmed = text::trim(q);
            let stripped = ["tag:", "label:", "#"].iter().find_map(|prefix| strip_prefix_ignore_case(trimmed, prefix)).unwrap_or(trimmed);
            let prefix = text::to_lower(stripped);
            let rows = db.query(
                &format!(
                    "SELECT a.name, COUNT(DISTINCT a.card_id) AS count, MAX((a.flags & 8) != 0) AS manual
        FROM annotations a JOIN cards c ON c.id = a.card_id WHERE {}
        AND substr(lower(a.name), 1, length(?)) = ? GROUP BY lower(a.name)",
                    visible()
                ),
                &[self.user.as_str().into(), prefix.as_str().into(), prefix.as_str().into()],
            )?;
            for row in &rows {
                let name = row.text("name").unwrap_or("");
                let count = row.number("count").unwrap_or(0.0);
                let manual = row.number("manual").unwrap_or(0.0) != 0.0;
                out.push(Suggestion { kind: "tag", text: name.to_owned(), query_text: format!("tag:{name}"), count, score: count * if manual { 2.0 } else { 1.0 }, is_ai: !manual });
            }
        }
        out.sort_by(|a, b| sign(b.score - a.score).unwrap_or_else(|| locale_compare(&a.text, &b.text)));
        Ok(out)
    }

    /// Semantic search under a legacy model id (`embeddings`).
    ///
    /// # Arguments
    /// * `q` - Query text.
    /// * `options` - Object with `take` and `filter` (a text query that limits the candidates).
    ///
    /// # Errors
    /// Status 400 for an over-long `filter`, or the database's error.
    pub async fn embeddings(&self, q: &str, options: &Value) -> Result<SearchResult> {
        let started = runtime::now_ms();
        let filter = options.get("filter");
        if filter.truthy() {
            parse_query(&filter.to_js_string(), false)?;
        }
        let allowed = || -> Result<HashSet<String>> { Ok(self.text(&filter.to_js_string(), &Value::Undefined)?.matches.into_iter().map(|found| found.id).collect()) };
        let eligible = if filter.truthy() { Eligible::Set(&allowed) } else { Eligible::Any };
        let found = self.semantic.search(q, options.get("take"), eligible).await?;
        Ok(Self::result(started, q.to_owned(), found))
    }

    /// Cards whose stored vector is nearest to a card's (`related`).
    ///
    /// # Arguments
    /// * `id` - Card id.
    /// * `options` - Object with `take` and `eligible`.
    ///
    /// # Errors
    /// The database's error, or an invalid-data error for a truthy `eligible`.
    pub fn related(&self, id: &str, options: &Value) -> Result<SearchResult> {
        let started = runtime::now_ms();
        let found = self.semantic.related(id, options.get("take"), Eligible::Raw(options.get("eligible")))?;
        Ok(Self::result(started, format!("similar:{id}"), found))
    }
}

impl SpaceSearch for Search {
    fn filters(&self, filters: &Value) -> Result<Vec<SpaceMatch>> {
        Ok(Search::filters(self, filters)?.into_iter().map(|found| SpaceMatch { id: found.id, score: found.score }).collect())
    }
}
