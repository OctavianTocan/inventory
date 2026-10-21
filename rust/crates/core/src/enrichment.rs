//! Port of `lib/enrichment.mjs`: link capture and hosted AI after a save, one card at a time.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;
use mm_js::{Value, json, obj};

use crate::ai::{Ai, AiImage, AiItem, VocabularyEntry};
use crate::ai_tags::relevant_tag;
use crate::backfill::{self, Estimate, MediaImages, Report};
use crate::capture_link::{CaptureOptions, capture_link};
use crate::host::{Fetch, Media, MediaUser, Resource, Screenshot};
use crate::search_document::analysis_item;
use crate::sql::SqlExt;
use crate::store::{CardRow, Store};
use crate::store_write::Writer;
use crate::{Error, Result, runtime};

const HIDDEN: i64 = 131_072 | 262_144;
const PROCESSING: i64 = 1;

/// What [`Enrichment::new`] works with (`options` of `createEnrichment`).
pub struct EnrichmentOptions {
    /// Open store; never used after [`Enrichment::close`].
    pub store: Rc<Store>,
    /// Writer with the enrichment mutations.
    pub writer: Rc<Writer>,
    /// Media handle that stores captured originals and reads analysis images.
    pub media: Rc<dyn Media>,
    /// Validated HTTP transport for captured pages.
    pub resource: Rc<dyn Resource>,
    /// Page screenshots for link capture.
    pub screenshot: Rc<dyn Screenshot>,
    /// Outbound HTTP for the hosted AI APIs.
    pub fetch: Rc<dyn Fetch>,
    /// Capture may reach loopback fixtures only, and AI runs only against loopback APIs.
    pub offline: bool,
    /// Disable every job, API call and write.
    pub readonly: bool,
}

struct Inner {
    options: EnrichmentOptions,
    pending: RefCell<IndexMap<String, bool>>,
    closed: Cell<bool>,
    running: Cell<bool>,
}

/// The post-save job queue: capture and hosted AI, serialized and coalesced per card (`createEnrichment`'s result).
#[derive(Clone)]
pub struct Enrichment(Rc<Inner>);

fn env_set(name: &str) -> bool {
    runtime::env(name).is_some_and(|value| !value.is_empty())
}

fn loopback(name: &str) -> bool {
    let Some(address) = runtime::env(name) else { return false };
    mm_js::url::parse(&address, None).is_some_and(|parsed| ["127.0.0.1", "localhost", "[::1]"].contains(&parsed.host_str().unwrap_or("")))
}

/// Whether hosted AI may run: `GEMINI_API_KEY` is set, and in offline mode every AI base URL is a loopback address.
///
/// # Arguments
/// * `offline` - Allow only loopback AI endpoints.
pub fn ai_configured(offline: bool) -> bool {
    if !env_set("GEMINI_API_KEY") {
        return false;
    }
    let typesafe = if env_set("TYPESAFE_API_KEY") { &["TYPESAFE_BASE_URL"][..] } else { &[] };
    !offline || ["GEMINI_BASE_URL", "AI_CHAT_BASE_URL"].iter().chain(typesafe).all(|name| loopback(name))
}

// A summary the user wrote, or one imported from their library, is not replaced by a later analysis.
fn summary_owned(data: &Value, extras: &Value) -> bool {
    extras.get("summaryEdited").truthy() || (data.get("summary").truthy() && extras.get("summarySource").as_str() != Some("ai"))
}

fn parse(text: &str) -> Result<Value> {
    Ok(json::parse(text)?)
}

// A card row as JS reads `row.deletedNs`: 0n is falsy.
fn deleted(row: &CardRow) -> bool {
    row.deleted_ns.is_some_and(|ns| ns != 0)
}

// Use the reader editor's current document instead of stale scraped Article prose.
fn article_content(row: &CardRow, store: &Store) -> Result<Value> {
    if row.kind != "Article" {
        return Ok(Value::Null);
    }
    match store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])? {
        Some(content) => parse(content.text("body").unwrap_or("")),
        None => Ok(parse(&row.data)?.get("prose").clone()),
    }
}

// Detect intervening edits before applying an AI result generated from older content.
fn signature(row: &CardRow, store: &Store) -> Result<String> {
    let (data, extras) = (parse(&row.data)?, parse(&row.extras)?);
    let parts = [data.get("title"), data.get("description"), data.get("prose"), data.get("source"), extras.get("captureText"), extras.get("cover")]
        .into_iter()
        .cloned()
        .chain([article_content(row, store)?])
        .collect();
    Ok(json::stringify(&Value::Array(parts))?.unwrap_or_default())
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Undefined | Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => Some(other.to_js_string()),
    }
}

fn first_present<'a>(values: &[&'a Value]) -> &'a Value {
    values.iter().copied().find(|value| !value.is_nullish()).unwrap_or(&mm_js::UNDEFINED)
}

impl Enrichment {
    /// Create the queue (`createEnrichment`).
    ///
    /// # Arguments
    /// * `options` - Services and the offline and read-only policy.
    pub fn new(options: EnrichmentOptions) -> Self {
        Self(Rc::new(Inner { options, pending: RefCell::new(IndexMap::new()), closed: Cell::new(false), running: Cell::new(false) }))
    }

    /// Whether hosted AI runs (`enabled`): `GEMINI_API_KEY` is set, writes are on, and offline every AI base URL is a
    /// loopback address.
    pub fn enabled(&self) -> bool {
        let options = &self.0.options;
        !options.readonly && ai_configured(options.offline)
    }

    /// How many stored cards still need a summary, tags or an embedding, and a rough token cost.
    ///
    /// # Errors
    /// The database's error, or malformed card JSON.
    pub fn estimate(&self) -> Result<Estimate> {
        backfill::estimate(&self.0.options.store)
    }

    /// Fill gaps on up to `limit` stored cards. Hand-written summaries and tags are left in place.
    ///
    /// # Arguments
    /// * `limit` - Maximum cards to attempt on this call.
    ///
    /// # Errors
    /// The library is read-only, hosted AI is not configured, or [`backfill::run`] fails on the database.
    pub async fn backfill(&self, limit: usize) -> Result<Report> {
        let options = &self.0.options;
        if options.readonly {
            return Err(Error::status(403, "Backfill is disabled while the library is read-only"));
        }
        let images = MediaImages(Rc::clone(&options.media));
        backfill::run(&options.store, &options.writer, &*options.fetch, &images, limit).await
    }

    /// Queue a card for enrichment on a later turn of the event loop, after the save's response (`schedule`).
    ///
    /// # Arguments
    /// * `id` - Card id; a card already queued keeps its place.
    /// * `capture` - Capture the card's link first; a queued capture is never dropped.
    pub fn schedule(&self, id: &str, capture: bool) {
        let inner = &self.0;
        if inner.closed.get() || inner.options.readonly {
            return;
        }
        let queued = inner.pending.borrow().get(id).copied().unwrap_or(false);
        inner.pending.borrow_mut().insert(id.to_owned(), capture || queued);
        if inner.running.replace(true) {
            return;
        }
        let queue = self.clone();
        runtime::defer(async move {
            runtime::sleep(0).await;
            while !queue.0.closed.get() {
                let Some((id, capture)) = queue.0.pending.borrow_mut().shift_remove_index(0) else { break };
                if queue.process_card(&id, capture).await.is_err() && !queue.0.closed.get() {
                    runtime::log("[capture] enrichment failed; saved card retained");
                }
            }
            queue.0.running.set(false);
        });
    }

    /// Wait until the queue is empty (`drain`), for tests and shutdown.
    pub async fn drain(&self) {
        while self.0.running.get() {
            runtime::sleep(1).await;
        }
    }

    /// Stop new jobs and every write of a running one (`close`).
    pub fn close(&self) {
        self.0.closed.set(true);
        self.0.pending.borrow_mut().clear();
    }

    fn processing(&self, id: &str, enabled: bool, capture_done: bool) -> Result<()> {
        self.0.options.writer.run(|writer| writer.processing(id, enabled, capture_done))
    }

    // Process a saved card without holding a transaction across any I/O or hosted API.
    async fn process_card(&self, id: &str, capture: bool) -> Result<()> {
        let inner = &self.0;
        let options = &inner.options;
        let store = &options.store;
        let Some(row) = store.card(id, None)? else { return Ok(()) };
        if deleted(&row) || row.flags & HIDDEN != 0 || parse(&row.extras)?.get("screenshotFailed").truthy() {
            return Ok(());
        }
        if capture {
            self.capture(&row).await?;
            if inner.closed.get() {
                return Ok(());
            }
            let current = store.card(id, None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'extras')"))?;
            if parse(&current.extras)?.get("screenshotFailed").truthy() {
                return Ok(());
            }
            self.processing(id, false, true)?;
        }
        if inner.closed.get() {
            return Ok(());
        }
        let Some(row) = store.card(id, None)? else { return Ok(()) };
        if deleted(&row) {
            return Ok(());
        }
        let saved = parse(&row.data)?;
        // A blocked or failed capture leaves only the URL, from which the model invents a summary.
        if parse(&row.extras)?.get("captureFailed").truthy() && !saved.get("description").truthy() && !saved.get("prose").truthy() {
            if row.flags & PROCESSING != 0 {
                self.processing(id, false, false)?;
            }
            return Ok(());
        }
        if !self.enabled() {
            runtime::log(if env_set("GEMINI_API_KEY") { "[ai] skipped: offline hosted services disabled" } else { "[ai] skipped: GEMINI_API_KEY is not set" });
            if row.flags & PROCESSING != 0 {
                self.processing(id, false, false)?;
            }
            return Ok(());
        }
        self.processing(id, true, false)?;
        let row = store.card(id, None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'data')"))?;
        let before = signature(&row, store)?;
        if self.analyze(&row, &before).await.is_err() && !inner.closed.get() {
            runtime::log("[ai] analysis failed; saved card retained");
            self.processing(id, false, false)?;
        }
        Ok(())
    }

    async fn capture(&self, row: &CardRow) -> Result<()> {
        let options = &self.0.options;
        let baseline = RefCell::new(Value::from(obj! { "data" => parse(&row.data)?, "extras" => parse(&row.extras)? }));
        let user = options.store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'accountUid')"))?;
        let user = MediaUser { account_uid: user.account_uid, id_prefix: user.id_prefix };
        let update = |patch: Value| -> Result<()> {
            if !self.0.closed.get() {
                options.writer.run(|writer| writer.enrich(&row.id, &patch, &mut baseline.borrow_mut()))?;
            }
            Ok(())
        };
        let capture = CaptureOptions { id: &row.id, user: &user, media: &*options.media, update: &update, offline: options.offline };
        capture_link(&|| baseline.borrow().get("data").clone(), &capture, &*options.resource, &*options.screenshot).await
    }

    async fn analyze(&self, row: &CardRow, before: &str) -> Result<()> {
        let options = &self.0.options;
        let store = &options.store;
        let (data, extras) = (parse(&row.data)?, parse(&row.extras)?);
        let kind = data.get("type").to_js_string();
        let names = store.db().query(
            "SELECT name, COUNT(*) AS n FROM annotations WHERE user_id = ? GROUP BY name ORDER BY n DESC, name",
            &[store.default_user_id()?.into()],
        )?;
        let mut vocabulary = Vec::new();
        for name in names.iter().map(|tag| tag.text("name").unwrap_or("").to_owned()) {
            if relevant_tag(&name, &kind)? {
                vocabulary.push(VocabularyEntry { name, embedding: None });
            }
        }
        let source = first_present(&[extras.get("cover"), extras.get("post").get("gallery").at(0), data.get("blob")]);
        let image = options.media.image(source).await.ok().flatten();
        let item = analysis_item(&row.data, &row.extras, Some(&article_content(row, store)?))?;
        let item = AiItem {
            title: text_of(&item.title),
            kind: text_of(&item.kind),
            text: Some(item.text),
            url: text_of(&item.url),
            domain: text_of(&item.domain),
            image: image.map(|image| AiImage { mime: image.mime, data: image.data }),
        };
        let result = Ai::new(&*options.fetch).analyze(&item, &vocabulary).await?;
        if self.0.closed.get() {
            return Ok(());
        }
        let Some(current) = store.card(&row.id, None)? else { return Ok(()) };
        if deleted(&current) {
            return Ok(());
        }
        if signature(&current, store)? != before {
            self.schedule(&row.id, false);
            return Ok(());
        }
        let mut value = result.to_value();
        // A summary already stored, and not one this backfill marked as its own, is left in place. The result
        // otherwise matches the historical analysis write, including cards that receive their first summary.
        if summary_owned(&data, &extras) {
            value.set("preserveSummary", true);
        }
        options.writer.run(|writer| writer.analysis(&row.id, &value))
    }
}
