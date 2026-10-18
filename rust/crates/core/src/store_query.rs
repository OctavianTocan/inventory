//! Port of `lib/store-query.mjs`: smart-space (saved query) refresh.

use std::collections::HashSet;

use mm_js::{Object, Value, json, number};

use crate::runtime;
use crate::sql::{SqlExt, SqlValue};
use crate::store::{Store, iso_from_ns};
use crate::{Error, Result};

/// A card that matches a saved query's filters, as the search service reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct SpaceMatch {
    /// Card id.
    pub id: String,
    /// Relevance score.
    pub score: f64,
}

/// A match that a refresh added to a space.
#[derive(Clone, Debug, PartialEq)]
pub struct AddedMatch {
    /// The match.
    pub found: SpaceMatch,
    /// The saved query that found it.
    pub query_id: String,
}

/// The search service a query refresh evaluates filters with (`search.filters`).
pub trait SpaceSearch {
    /// Cards matching a saved query's `filters`.
    ///
    /// # Arguments
    /// * `filters` - The `filters` of the stored query JSON; undefined when the query has none.
    ///
    /// # Errors
    /// The search service's error.
    fn filters(&self, filters: &Value) -> Result<Vec<SpaceMatch>>;
}

/// The writer helpers a persisting refresh uses (`{ now, save, saveSpace, card }`).
pub trait QueryWriter {
    /// What `card` returns and `save` takes.
    type Card;

    /// The writer's current time in nanoseconds since the epoch.
    ///
    /// # Errors
    /// The writer's error.
    fn now(&self) -> Result<i128>;

    /// Load a card inside the writer's transaction.
    ///
    /// # Arguments
    /// * `id` - Card id.
    ///
    /// # Errors
    /// The writer's error.
    fn card(&self, id: &str) -> Result<Self::Card>;

    /// Record a card as changed.
    ///
    /// # Arguments
    /// * `card` - A card from [`QueryWriter::card`].
    ///
    /// # Errors
    /// The writer's error.
    fn save(&self, card: Self::Card) -> Result<()>;

    /// Record a space as changed.
    ///
    /// # Arguments
    /// * `id` - Space id.
    ///
    /// # Errors
    /// The writer's error.
    fn save_space(&self, id: &str) -> Result<()>;
}

/// What a refresh found (`checkQuery`'s result).
#[derive(Clone, Debug, PartialEq)]
pub struct QueryCheck {
    /// Milliseconds the refresh took, rounded.
    pub elapsed: f64,
    /// Matches that were not in the space, each tagged with the query.
    pub added: Vec<AddedMatch>,
    /// Ids that were in the space through this query and no longer match.
    pub removed: Vec<String>,
}

impl QueryCheck {
    /// The `{elapsed, added, removed}` object the JS returned.
    pub fn to_value(&self) -> Value {
        let added = self
            .added
            .iter()
            .map(|item| {
                let mut object = Object::new();
                object.insert("id", item.found.id.clone());
                object.insert("score", item.found.score);
                object.insert("queryId", item.query_id.clone());
                Value::Object(object)
            })
            .collect();
        let mut object = Object::new();
        object.insert("elapsed", self.elapsed);
        object.insert("added", Value::Array(added));
        object.insert("removed", Value::Array(self.removed.iter().cloned().map(Value::from).collect()));
        Value::Object(object)
    }
}

/// Refresh a smart space, or preview the refresh without persisting (`createQueryChecker`'s `checkQuery`).
///
/// # Arguments
/// * `store` - Open store; evaluation and writes stay on its connection.
/// * `search` - Search service evaluating the query's filters.
/// * `writer` - Writer helpers used when persisting.
/// * `query_id` - Id of the saved query.
/// * `persist` - Write the new membership and save the changes; `false` only computes them.
///
/// # Returns
/// The time taken, the matches added and the ids removed.
///
/// # Errors
/// Status 404 `Query not found` for an unknown query id, or the database's, search's or writer's error.
pub fn check_query<W: QueryWriter>(store: &Store, search: &dyn SpaceSearch, writer: &W, query_id: &str, persist: bool) -> Result<QueryCheck> {
    let started = runtime::now_ms();
    let db = store.db();
    let user = store.default_user_id()?;
    let row = db
        .one("SELECT id, query FROM spaces WHERE user_id = ? AND json_extract(query, '$.id') = ?", &[user.clone().into(), query_id.into()])?
        .ok_or_else(|| Error::status(404, "Query not found"))?;
    let space_id = row.text("id").unwrap_or("").to_owned();
    let mut query = json::parse(row.text("query").unwrap_or("")).map_err(Error::from)?;
    let existing = db.query("SELECT card_id, query_id, excluded FROM space_cards WHERE space_id = ?", &[space_id.as_str().into()])?;
    let truthy = |value: &SqlValue| !matches!(value, SqlValue::Null) && value.as_f64() != Some(0.0) && value.as_str() != Some("");
    let member = |row: &crate::sql::Row| row.text("card_id").unwrap_or("").to_owned();
    let excluded: HashSet<String> = existing.iter().filter(|row| truthy(row.get("excluded"))).map(member).collect();
    let manual: HashSet<String> = existing.iter().filter(|row| !truthy(row.get("query_id"))).map(member).collect();
    let mut previous: Vec<String> = Vec::new();
    for row in existing.iter().filter(|row| truthy(row.get("query_id")) && !truthy(row.get("excluded"))) {
        let id = member(row);
        if !previous.contains(&id) {
            previous.push(id);
        }
    }
    let current: Vec<SpaceMatch> =
        search.filters(&query.get("filters").clone())?.into_iter().filter(|found| !excluded.contains(&found.id) && !manual.contains(&found.id)).collect();
    let ids: HashSet<&str> = current.iter().map(|found| found.id.as_str()).collect();
    let added: Vec<AddedMatch> = current
        .iter()
        .filter(|found| !previous.contains(&found.id))
        .map(|found| AddedMatch { found: found.clone(), query_id: query_id.to_owned() })
        .collect();
    let removed: Vec<String> = previous.iter().filter(|id| !ids.contains(id.as_str())).cloned().collect();
    if persist {
        db.execute("DELETE FROM space_cards WHERE space_id = ? AND query_id = ? AND excluded = 0", &[space_id.as_str().into(), query_id.into()])?;
        let mut rank = db
            .one("SELECT COALESCE(MAX(rank), -1) + 1 AS n FROM space_cards WHERE space_id = ?", &[space_id.as_str().into()])?
            .and_then(|row| row.number("n"))
            .unwrap_or(0.0);
        for found in &current {
            db.execute(
                "INSERT INTO space_cards VALUES (?, ?, ?, ?, ?, ?, 0)",
                &[space_id.as_str().into(), found.id.as_str().into(), user.as_str().into(), rank.into(), found.score.into(), query_id.into()],
            )?;
            rank += 1.0;
        }
        let checked = iso_from_ns(writer.now()?, 0)?;
        if let Value::Object(object) = &mut query {
            object.insert("checked", checked);
        }
        let text = json::stringify(&query).map_err(|error| Error::internal(error.0))?.unwrap_or_default();
        db.execute("UPDATE spaces SET query = ? WHERE id = ?", &[text.into(), space_id.as_str().into()])?;
        if !added.is_empty() || !removed.is_empty() {
            writer.save_space(&space_id)?;
            for id in added.iter().map(|item| item.found.id.as_str()).chain(removed.iter().map(String::as_str)) {
                writer.save(writer.card(id)?)?;
            }
        }
    }
    Ok(QueryCheck { elapsed: number::round(runtime::now_ms() - started), added, removed })
}
