//! Port of `lib/app-api.mjs`: the native app's read endpoints under `/app/v1`.

use std::rc::Rc;

use mm_js::{Object, Value, base64, json, number, obj, text, uri};
use url::Url;

use crate::app_sync::{changed_after, install_app_sync};
use crate::http::{Headers, Response, search_param};
use crate::render::media::image_cover;
use crate::search::{Hybrid, Search};
use crate::sql::{SqlExt, SqlValue};
use crate::store::{CardRow, Cursor, GRID_HIDDEN_FLAGS, Store, iso_from_ns};
use crate::{Error, Result};

const DEFAULT_LIMIT: f64 = 50.0;
const MAX_LIMIT: f64 = 200.0;
const CHANGES_LIMIT: f64 = 100.0;
// Joined to the log's epoch in every changes token. A new value tells phones holding an older token to start over, so
// copies kept before the cards gained a field the phone reads (`spaceRanks`) fill again.
const CHANGES_FORMAT: &str = "+ranks";

/// The `/app/v1` endpoints over a store and its search.
pub struct AppApi {
    store: Rc<Store>,
    search: Rc<Search>,
    localize: fn(&str) -> String,
}

enum After {
    Keyset(Option<Cursor>),
    Offset(Option<usize>),
}

/// A prose document or node as plain text, its blocks one per line.
pub(crate) fn prose_text(node: &Value) -> Result<String> {
    if node.is_nullish() {
        return Err(Error::invalid("Cannot read properties of null (reading 'type')"));
    }
    if node.get("type").as_str() == Some("hardBreak") {
        return Ok("\n".to_owned());
    }
    if let Some(content) = node.get("text").as_str() {
        return Ok(content.to_owned());
    }
    let children = match node.get("content") {
        Value::Undefined | Value::Null => Vec::new(),
        Value::Array(items) => items.clone(),
        _ => return Err(Error::invalid("children.every is not a function")),
    };
    let mut inline = true;
    for child in &children {
        if child.is_nullish() {
            return Err(Error::invalid("Cannot read properties of null (reading 'type')"));
        }
        inline &= matches!(child.get("type").as_str(), Some("text" | "hardBreak"));
    }
    let parts = children.iter().map(prose_text).collect::<Result<Vec<_>>>()?;
    Ok(parts.join(if inline { "" } else { "\n" }))
}

fn keyset_cursor(ns: i128, id: &str) -> Result<String> {
    let raw = json::stringify(&Value::Array(vec![Value::String(ns.to_string()), Value::String(id.to_owned())]))?.unwrap_or_default();
    Ok(base64::encode_url(raw.as_bytes()))
}

fn offset_cursor(offset: usize) -> Result<String> {
    let raw = json::stringify(&Value::Array(vec![Value::Number(offset as f64)]))?.unwrap_or_default();
    Ok(base64::encode_url(raw.as_bytes()))
}

fn bad(message: &str) -> Error {
    Error::status(400, message)
}

fn read_cursor(source: &str, keyset: bool) -> Result<After> {
    let decoded = String::from_utf8_lossy(&base64::decode(source)).into_owned();
    let Ok(Value::Array(items)) = json::parse(&decoded) else { return Err(bad("bad-cursor")) };
    if keyset
        && items.len() == 2
        && let (Value::String(ns), Value::String(id)) = (&items[0], &items[1])
        && !ns.is_empty()
        && ns.chars().all(|c| c.is_ascii_digit())
        && let Ok(ns) = ns.parse::<i128>()
    {
        return Ok(After::Keyset(Some(Cursor { ns, id: id.clone() })));
    }
    if !keyset && items.len() == 1 && let Value::Number(offset) = items[0] && number::is_safe_integer(offset) && offset >= 0.0 {
        return Ok(After::Offset(Some(offset as usize)));
    }
    Err(bad("bad-cursor"))
}

fn limit_of(url: &Url, default: f64) -> Result<usize> {
    let limit = search_param(url, "limit").map_or(default, |raw| number::string_to_number(&raw));
    if limit.fract() != 0.0 || !limit.is_finite() || !(1.0..=MAX_LIMIT).contains(&limit) {
        return Err(bad("bad-limit"));
    }
    Ok(limit as usize)
}

fn paging(url: &Url, keyset: bool) -> Result<(usize, After)> {
    let limit = limit_of(url, DEFAULT_LIMIT)?;
    let after = match search_param(url, "cursor") {
        None if keyset => After::Keyset(None),
        None => After::Offset(None),
        Some(cursor) => read_cursor(&cursor, keyset)?,
    };
    Ok((limit, after))
}

fn changes_token(epoch: &str, seq: i64) -> Result<String> {
    let raw = json::stringify(&Value::Array(vec![Value::String(epoch.to_owned()), Value::String(seq.to_string())]))?.unwrap_or_default();
    Ok(base64::encode_url(raw.as_bytes()))
}

// The epoch and log position a `since` token names.
fn read_changes_token(source: &str) -> Result<(String, i64)> {
    let decoded = String::from_utf8_lossy(&base64::decode(source)).into_owned();
    if let Ok(Value::Array(items)) = json::parse(&decoded)
        && let [Value::String(epoch), Value::String(seq)] = items.as_slice()
        && !seq.is_empty()
        && seq.chars().all(|c| c.is_ascii_digit())
        && let Ok(seq) = seq.parse::<i64>()
    {
        return Ok((epoch.clone(), seq));
    }
    Err(bad("bad-cursor"))
}

fn present(entries: Vec<(&str, Value)>) -> Object {
    entries.into_iter().map(|(key, value)| (key.to_owned(), value)).collect()
}

fn or_iso(value: &Value, ns: i128) -> Result<Value> {
    Ok(if value.is_nullish() { Value::String(iso_from_ns(ns, 0)?) } else { value.clone() })
}

fn first(a: &Value, b: &Value) -> Value {
    if a.is_nullish() { b.clone() } else { a.clone() }
}

impl AppApi {
    /// Create the endpoints (`createAppApi`).
    ///
    /// # Arguments
    /// * `store` - Open store.
    /// * `search` - Search service whose `search(q, {})` orders the matches, or `hybrid` for `/search?semantic=1`.
    /// * `localize` - Rewrites recorded origins to local paths in every JSON body.
    pub fn new(store: Rc<Store>, search: Rc<Search>, localize: fn(&str) -> String) -> Self {
        Self { store, search, localize }
    }

    fn json(&self, status: u16, body: &Value) -> Result<Response> {
        let text = json::stringify(body)?.unwrap_or_default();
        Ok(Response::send(status, (self.localize)(&text).into_bytes(), &Headers::from([("content-type", "application/json; charset=utf-8")])))
    }

    fn problem(&self, status: u16, error: &str) -> Result<Response> {
        self.json(status, &obj! { "error" => error }.into())
    }

    fn card_json(&self, row: &CardRow) -> Result<Value> {
        let data = json::parse(&row.data)?;
        let extras = json::parse(&row.extras)?;
        let db = self.store.db();
        let cover = if extras.get("cover").is_nullish() {
            if data.get("type").as_str() == Some("Image") && data.get("blob").truthy() { image_cover(data.get("blob"))? } else { Value::Undefined }
        } else {
            extras.get("cover").clone()
        };
        let content = db.query("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])?;
        let prose = match content.first() {
            Some(found) => json::parse(found.text("body").unwrap_or(""))?,
            None => data.get("prose").clone(),
        };
        let members = db.query("SELECT space_id, rank FROM space_cards WHERE card_id = ? AND excluded = 0 ORDER BY rank", &[SqlValue::from(row.id.as_str())])?;
        let spaces: Vec<Value> = members.iter().map(|found| Value::String(found.text("space_id").unwrap_or("").to_owned())).collect();
        let space_ranks: Object =
            members.iter().map(|found| (found.text("space_id").unwrap_or("").to_owned(), Value::Number(found.number("rank").unwrap_or(0.0)))).collect();
        let tags: Vec<Value> = self.store.card_tags(&row.id)?.iter().map(|tag| tag.to_value()).collect();
        let note = if data.get("note").get("prose").truthy() {
            let trimmed = text::trim(&prose_text(data.get("note").get("prose"))?).to_owned();
            if trimmed.is_empty() { Value::Undefined } else { Value::String(trimmed) }
        } else {
            Value::Undefined
        };
        let blob = data.get("blob");
        let blob = if blob.truthy() {
            Value::Object(present(["path", "type", "width", "height", "duration", "dominantColor", "thumbHash"].map(|key| (key, blob.get(key).clone())).to_vec()))
        } else {
            blob.clone()
        };
        let state = if extras.get("captureFailed").truthy() {
            "failed"
        } else if extras.get("screenshotPending").truthy() {
            "screenshot"
        } else if extras.get("capturePending").truthy() {
            "saving"
        } else {
            "ready"
        };
        let author = if data.get("author").is_string() { data.get("author").clone() } else { Value::Undefined };
        let mut product = extras.get("product").clone();
        if data.get("brand").is_string() {
            if product.is_nullish() {
                product = Value::Object(obj! {});
            }
            product.set("brand", data.get("brand").clone());
        }
        let deleted = match row.deleted_ns {
            None => Value::Undefined,
            Some(ns) => or_iso(data.get("deleted"), ns)?,
        };
        let card = Value::Object(present(vec![
            ("id", row.id.as_str().into()),
            ("type", row.kind.as_str().into()),
            ("flags", Value::Number(row.flags as f64)),
            ("created", or_iso(data.get("created"), row.created_ns)?),
            ("modified", or_iso(data.get("modified"), row.modified_ns)?),
            ("bumped", or_iso(data.get("bumped"), row.bumped_ns)?),
            ("deleted", deleted),
            ("title", first(extras.get("title"), data.get("title"))),
            ("description", data.get("description").clone()),
            ("summary", if extras.get("post").get("thread").truthy() { data.get("summary").clone() } else { Value::Undefined }),
            ("url", first(extras.get("linkUrl"), data.get("source").get("url"))),
            ("domain", data.get("domain").clone()),
            ("siteName", data.get("siteName").clone()),
            ("author", author),
            ("note", note),
            ("tags", if tags.is_empty() { Value::Undefined } else { Value::Array(tags) }),
            ("spaces", if spaces.is_empty() { Value::Undefined } else { Value::Array(spaces) }),
            ("spaceRanks", if space_ranks.is_empty() { Value::Undefined } else { Value::Object(space_ranks) }),
            ("suggestedSpace", extras.get("suggestedSpace").get("id").clone()),
            ("mergedInto", extras.get("mergedInto").clone()),
            ("pinPosition", extras.get("pinPosition").clone()),
            ("cover", cover),
            ("blob", blob),
            ("prose", prose),
            ("post", extras.get("post").clone()),
            ("app", extras.get("app").clone()),
            ("product", product),
            ("reddit", extras.get("reddit").clone()),
            ("videoLength", extras.get("videoLength").clone()),
            ("transcript", if extras.get("transcript").get("text").truthy() { extras.get("transcript").get("text").clone() } else { Value::Undefined }),
            ("article", extras.get("article").clone()),
            ("state", state.into()),
        ]));
        Ok(json::parse(&json::stringify(&card)?.unwrap_or_default())?)
    }

    fn cards_json(&self, rows: &[CardRow]) -> Result<Value> {
        Ok(Value::Array(rows.iter().map(|row| self.card_json(row)).collect::<Result<_>>()?))
    }

    fn keyset_page(&self, url: &Url, trash: bool) -> Result<Response> {
        let (limit, After::Keyset(after)) = paging(url, true)? else { unreachable!() };
        let fetch = limit as i64 + 1;
        let rows = if trash { self.store.trash_cards_after(None, after.as_ref(), fetch)? } else { self.store.grid_cards_after(None, after.as_ref(), fetch)? };
        let page = &rows[..rows.len().min(limit)];
        let next = match page.last() {
            Some(last) if rows.len() > limit => Value::String(keyset_cursor(if trash { last.deleted_ns.unwrap_or(0) } else { last.bumped_ns }, &last.id)?),
            _ => Value::Null,
        };
        self.json(200, &obj! { "cards" => self.cards_json(page)?, "next" => next }.into())
    }

    fn offset_page(&self, url: &Url, rows: &[CardRow]) -> Result<Response> {
        let (limit, After::Offset(offset)) = paging(url, false)? else { unreachable!() };
        let start = offset.unwrap_or(0);
        let page = rows.get(start..).map_or(&rows[..0], |tail| &tail[..tail.len().min(limit)]);
        let next = if start + limit < rows.len() { Value::String(offset_cursor(start + limit)?) } else { Value::Null };
        self.json(200, &obj! { "cards" => self.cards_json(page)?, "next" => next }.into())
    }

    fn spaces_json(&self) -> Result<Value> {
        let spaces = self.store.spaces(None)?;
        Ok(Value::Array(
            spaces
                .iter()
                .map(|space| {
                    obj! {
                        "id" => space.id.as_str(),
                        "name" => space.name.as_str(),
                        "color" => space.color.as_str(),
                        "position" => space.position as f64,
                        "smart" => space.query.as_ref().is_some_and(Value::truthy),
                        "count" => space.objects.len() as f64,
                    }
                    .into()
                })
                .collect(),
        ))
    }

    // One page of the change log: the changed cards as `/cards` sends them, with `pinPosition` read from the pin order
    // Top of Mind uses, and the ids of cards trashed or deleted since. The last page also carries the spaces.
    fn changes_page(&self, url: &Url) -> Result<Response> {
        let limit = limit_of(url, CHANGES_LIMIT)?;
        let since = search_param(url, "since").map(|token| read_changes_token(&token)).transpose()?;
        let db = self.store.db();
        let epoch = format!("{}{CHANGES_FORMAT}", install_app_sync(&**db)?);
        let (reset, after) = match since {
            Some((from, seq)) if from == epoch => (false, seq),
            Some(_) => (true, 0),
            None => (false, 0),
        };
        let mut logged = changed_after(&**db, after, limit + 1)?;
        let more = logged.len() > limit;
        logged.truncate(limit);
        let ids: Vec<String> = logged.iter().map(|entry| entry.card_id.clone()).collect();
        let mut rows: std::collections::HashMap<String, CardRow> = self.store.cards_by_ids(&ids, None)?.into_iter().map(|row| (row.id.clone(), row)).collect();
        let (mut cards, mut removed) = (Vec::new(), Vec::new());
        for id in ids {
            match rows.remove(&id).filter(|row| row.deleted_ns.is_none()) {
                Some(row) => {
                    let mut card = self.card_json(&row)?;
                    let pin = db.one("SELECT position FROM pins WHERE card_id = ?", &[id.as_str().into()])?.and_then(|found| found.number("position"));
                    match pin {
                        Some(position) => card.set("pinPosition", position),
                        None => {
                            card.remove("pinPosition");
                        }
                    }
                    cards.push(card);
                }
                None => removed.push(Value::String(id)),
            }
        }
        let last = logged.last().map_or(after, |entry| entry.seq);
        let mut body = obj! {
            "cards" => Value::Array(cards),
            "removed" => Value::Array(removed),
            "next" => changes_token(&epoch, last)?,
            "more" => more,
            "reset" => reset,
        };
        if !more {
            body.insert("spaces", self.spaces_json()?);
        }
        self.json(200, &body.into())
    }

    /// Answer a request under `/app/v1` (`handle`).
    ///
    /// # Arguments
    /// * `method` - HTTP method.
    /// * `url` - Absolute request URL.
    ///
    /// # Returns
    /// `Some(response)` for `GET /app/v1/cards`, `/cards/<id>`, `/trash`, `/spaces`, `/spaces/<id>/cards`, `/tags`,
    /// `/search`, `/top-of-mind` (pinned cards in pin order, paged like a space) and `/changes` (the cards changed
    /// since a `since` token, starting the change log on first use), a 405 for other methods on those, and `None`
    /// for any other request.
    ///
    /// # Errors
    /// Any failure other than a 400, which is answered as a problem body.
    pub async fn handle(&self, method: &str, url: &Url) -> Result<Option<Response>> {
        let Some(path) = url.path().strip_prefix("/app/v1/").map(|rest| format!("/{rest}")) else { return Ok(None) };
        match self.route(method, url, &path).await {
            Err(error) if error.http_status() == Some(400) => {
                let message = error.to_string();
                let plain = !message.is_empty() && message.chars().all(|c| c.is_ascii_lowercase() || c == '-');
                self.problem(400, if plain { &message } else { "bad-request" }).map(Some)
            }
            other => other,
        }
    }

    async fn route(&self, method: &str, url: &Url, path: &str) -> Result<Option<Response>> {
        let head = path[1..].split('/').next().unwrap_or("");
        if !["cards", "trash", "spaces", "tags", "search", "top-of-mind", "changes"].contains(&head) {
            return Ok(None);
        }
        if method != "GET" {
            return Ok(Some(Response::send(405, Vec::new(), &Headers::from([("allow", "GET")]))));
        }
        let parts: Vec<&str> = path[1..].split('/').collect();
        let response = match parts.as_slice() {
            ["cards"] => self.keyset_page(url, false)?,
            ["trash"] => self.keyset_page(url, true)?,
            ["top-of-mind"] => self.offset_page(url, &self.store.pinned_cards(None)?)?,
            ["changes"] => self.changes_page(url)?,
            ["cards", id] if !id.is_empty() => match self.store.card(&uri::decode_uri_component(id)?, None)? {
                Some(row) => self.json(200, &self.card_json(&row)?)?,
                None => self.problem(404, "not-found")?,
            },
            ["spaces"] => self.json(200, &obj! { "spaces" => self.spaces_json()? }.into())?,
            ["spaces", id, "cards"] if !id.is_empty() => {
                let id = uri::decode_uri_component(id)?;
                if self.store.space(&id)?.is_some() {
                    let rows: Vec<CardRow> = self.store.space_cards(&id)?.into_iter().filter(|row| row.deleted_ns.is_none() && row.flags & GRID_HIDDEN_FLAGS == 0).collect();
                    self.offset_page(url, &rows)?
                } else {
                    self.problem(404, "not-found")?
                }
            }
            ["tags"] => {
                let list: Vec<Value> = self.store.tags_manual(None)?.iter().map(|tag| obj! { "name" => tag.name.as_str(), "count" => tag.count as f64 }.into()).collect();
                self.json(200, &obj! { "tags" => list }.into())?
            }
            ["search"] => match search_param(url, "q").filter(|q| !text::trim(q).is_empty()) {
                None => self.problem(400, "missing-query")?,
                Some(q) => {
                    let found = if search_param(url, "semantic").as_deref() == Some("1") {
                        self.search.hybrid(&q, &Hybrid::default()).await?
                    } else {
                        self.search.search(&q, &Value::Object(Object::new())).await?
                    };
                    let ids: Vec<String> = found.matches.into_iter().map(|found| found.id).collect();
                    self.offset_page(url, &self.store.cards_by_ids(&ids, None)?)?
                }
            },
            _ => return Ok(None),
        };
        Ok(Some(response))
    }
}
