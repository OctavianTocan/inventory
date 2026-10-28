//! Port of `lib/search-semantic.mjs`: cosine search over the stored 768-float embeddings.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use mm_js::{Value, locale_compare, obj, url};

use crate::ai::{Ai, AiItem, EmbedKind, cosine};
use crate::host::Fetch;
use crate::runtime;
use crate::sql::SqlExt;
use crate::store::{GRID_HIDDEN_FLAGS, Store};
use crate::{Error, Result};

/// How many recent query embeddings [`Semantic::search_cached`] keeps.
const QUERY_CACHE: usize = 64;

/// A card and its similarity score.
#[derive(Clone, Debug, PartialEq)]
pub struct Match {
    /// Card id.
    pub id: String,
    /// Relevance score.
    pub score: f64,
}

impl Match {
    /// The `{id, score}` object the JS returned.
    pub fn to_value(&self) -> Value {
        obj! { "id" => self.id.as_str(), "score" => self.score }.into()
    }
}

/// Which cards a nearest-vector search may return (the `eligible` option).
pub enum Eligible<'a> {
    /// Every card.
    Any,
    /// The ids the function returns when called, once per search.
    Set(&'a dyn Fn() -> Result<HashSet<String>>),
    /// A JS value passed through as `eligible`; any truthy value other than a function has no `has` method, so it
    /// makes the search throw as soon as one vector is considered.
    Raw(&'a Value),
}

/// Whether hosted embeddings are allowed (`embeddingsEnabled`).
///
/// # Arguments
/// * `offline` - Disable hosted API traffic; only a loopback `GEMINI_BASE_URL` stub is then allowed.
///
/// # Returns
/// `true` when `GEMINI_API_KEY` is set and the transport policy permits an embedding call.
pub fn embeddings_enabled(offline: bool) -> bool {
    if runtime::env("GEMINI_API_KEY").is_none_or(|key| key.is_empty()) {
        return false;
    }
    if !offline {
        return true;
    }
    let Some(base) = runtime::env("GEMINI_BASE_URL") else { return false };
    url::parse(&base, None).is_some_and(|parsed| matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")))
}

/// Decode a stored little-endian 768-float embedding (`decodeVector`), from the AI kit shared with Pully.
pub use ai_kit::embed::decode_vector;

/// Cosine search over a user's active vectors (`createSemantic`'s result).
pub struct Semantic {
    store: Rc<Store>,
    fetch: Option<Rc<dyn Fetch>>,
    offline: bool,
    queries: RefCell<VecDeque<(String, Rc<[f32]>)>>,
}

impl Semantic {
    /// Build the search over a store (`createSemantic`).
    ///
    /// # Arguments
    /// * `store` - Open store.
    /// * `fetch` - Outbound HTTP for query embeddings; `None` makes [`Semantic::search`] always answer empty.
    /// * `offline` - Hosted API policy, as in [`embeddings_enabled`].
    pub fn new(store: Rc<Store>, fetch: Option<Rc<dyn Fetch>>, offline: bool) -> Self {
        Self { store, fetch, offline, queries: RefCell::new(VecDeque::new()) }
    }

    fn has_vectors(&self) -> Result<bool> {
        Ok(self
            .store
            .db()
            .one(
                &format!(
                    "SELECT 1 FROM embeddings e JOIN cards c ON c.id = e.card_id
      WHERE e.user_id = ? AND c.user_id = e.user_id AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0 LIMIT 1"
                ),
                &[self.store.default_user_id()?.into()],
            )?
            .is_some())
    }

    fn vectors(&self) -> Result<Vec<(String, Option<Vec<u8>>)>> {
        Ok(self
            .store
            .db()
            .query(
                &format!(
                    "SELECT e.card_id AS id, e.vector FROM embeddings e JOIN cards c ON c.id = e.card_id
      WHERE e.user_id = ? AND c.user_id = e.user_id AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0"
                ),
                &[self.store.default_user_id()?.into()],
            )?
            .iter()
            .map(|row| (row.text("id").unwrap_or("").to_owned(), row.blob("vector").map(<[u8]>::to_vec)))
            .collect())
    }

    fn nearest(&self, vector: &[f32], take: &Value, eligible: &Eligible<'_>, exclude: Option<&str>) -> Result<Vec<Match>> {
        let take = if take.is_undefined() { &Value::Number(100.0) } else { take };
        let requested = take.to_number();
        let count = if matches!(take, Value::Number(n) if *n == -1.0) {
            usize::MAX
        } else {
            let bounded = if requested.is_finite() { requested.floor() } else { 100.0 };
            bounded.clamp(0.0, 1000.0) as usize
        };
        let allowed = match eligible {
            Eligible::Set(make) => Some(make()?),
            _ => None,
        };
        let mut found = Vec::new();
        for (id, bytes) in self.vectors()? {
            if exclude == Some(id.as_str()) {
                continue;
            }
            match eligible {
                Eligible::Set(_) if !allowed.as_ref().is_some_and(|set| set.contains(&id)) => continue,
                Eligible::Raw(raw) if raw.truthy() => return Err(Error::invalid("allowed.has is not a function")),
                _ => {}
            }
            if let Some(other) = bytes.as_deref().and_then(decode_vector) {
                found.push(Match { id, score: cosine(vector, &other) });
            }
        }
        found.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal).then_with(|| locale_compare(&a.id, &b.id)));
        found.truncate(count);
        Ok(found)
    }

    /// Cards nearest to the embedding of a query text (`search`).
    ///
    /// # Arguments
    /// * `q` - Query text.
    /// * `take` - Result count: undefined for 100, -1 for all, otherwise clamped to 0..1000.
    /// * `eligible` - Which cards may be returned.
    ///
    /// # Returns
    /// Matches by score descending; empty when embeddings are disabled, no vectors exist, the query is blank or the
    /// embedding call fails.
    ///
    /// # Errors
    /// The database's error.
    pub async fn search(&self, q: &str, take: &Value, eligible: Eligible<'_>) -> Result<Vec<Match>> {
        self.nearest_to_query(q, take, eligible, false).await
    }

    /// [`Semantic::search`] that reuses the embeddings of the last 64 distinct query texts, so paging through one
    /// query's results asks Gemini once. Failed embeddings are not kept.
    ///
    /// # Arguments
    /// * `q` - Query text, the cache key as given.
    /// * `take` - Result count, as in [`Semantic::search`].
    /// * `eligible` - Which cards may be returned.
    ///
    /// # Returns
    /// Matches by score descending, empty in the same cases as [`Semantic::search`].
    ///
    /// # Errors
    /// The database's error.
    pub async fn search_cached(&self, q: &str, take: &Value, eligible: Eligible<'_>) -> Result<Vec<Match>> {
        self.nearest_to_query(q, take, eligible, true).await
    }

    async fn nearest_to_query(&self, q: &str, take: &Value, eligible: Eligible<'_>, cached: bool) -> Result<Vec<Match>> {
        if mm_js::text::trim(q).is_empty() || !embeddings_enabled(self.offline) || !self.has_vectors()? {
            return Ok(Vec::new());
        }
        let Some(vector) = self.query_vector(q, cached).await else { return Ok(Vec::new()) };
        Ok(self.nearest(&vector, take, &eligible, None).unwrap_or_default())
    }

    // The query's embedding, from the cache when allowed; `None` without a transport or when the call fails.
    async fn query_vector(&self, q: &str, cached: bool) -> Option<Rc<[f32]>> {
        if cached {
            let mut queries = self.queries.borrow_mut();
            if let Some(at) = queries.iter().position(|(text, _)| text == q) {
                let entry = queries.remove(at)?;
                let vector = Rc::clone(&entry.1);
                queries.push_back(entry);
                return Some(vector);
            }
        }
        let fetch = self.fetch.as_deref()?;
        let item = AiItem { text: Some(q.to_owned()), ..AiItem::default() };
        let vector: Rc<[f32]> = Ai::new(fetch).embed(&item, EmbedKind::Query).await.ok()?.into();
        if cached {
            let mut queries = self.queries.borrow_mut();
            queries.push_back((q.to_owned(), Rc::clone(&vector)));
            if queries.len() > QUERY_CACHE {
                queries.pop_front();
            }
        }
        Some(vector)
    }

    /// Cards whose stored vector is nearest to a card's own (`related`).
    ///
    /// # Arguments
    /// * `id` - Card id; the card itself is excluded.
    /// * `take` - Result count, as in [`Semantic::search`].
    /// * `eligible` - Which cards may be returned.
    ///
    /// # Returns
    /// Matches by score descending; empty when the card has no usable vector.
    ///
    /// # Errors
    /// The database's error, or an invalid-data error for a non-function `eligible`.
    pub fn related(&self, id: &str, take: &Value, eligible: Eligible<'_>) -> Result<Vec<Match>> {
        let vectors = self.vectors()?;
        let Some(vector) = vectors.iter().find(|(card, _)| card == id).and_then(|(_, bytes)| bytes.as_deref()).and_then(decode_vector) else {
            return Ok(Vec::new());
        };
        self.nearest(&vector, take, &eligible, Some(id))
    }
}

