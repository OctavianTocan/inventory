//! Port of `lib/enrichment.mjs`: link capture and hosted AI after a save, one card at a time, and the ThumbHash of each
//! card's gallery picture (`thumb_hash`).

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::future::poll_fn;
use std::rc::Rc;
use std::task::{Poll, Waker};

use futures_util::future::join;
use indexmap::IndexMap;
use mm_js::{Value, json, obj};

use crate::ai::{Ai, AiImage, AiItem, SpaceOption, TRANSCRIBE_MAX_BYTES, VocabularyEntry, needs_picture};
use crate::ai_tags::relevant_tag;
use crate::backfill::{self, Estimate, MediaImages, Report};
use crate::capture_link::{CaptureOptions, capture_link};
use crate::capture_more::capture_more;
use crate::host::{Fetch, Media, MediaUser, Resource, Screenshot};
use crate::search_document::analysis_item;
use crate::search_semantic::{Eligible, Semantic};
use crate::sql::{SqlExt, SqlValue};
use crate::store::{CardRow, Store};
use crate::store_enrich::{NAMED_NOTE_TYPES, note_needs_name};
use crate::store_write::Writer;
use crate::{Error, Result, runtime, thumb_hash};

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
    // How many more pictures without a ThumbHash the sweep may hash, whether it found none left, and the cards whose
    // picture it could not hash for now.
    sweep: Cell<usize>,
    swept: Cell<bool>,
    unhashed: RefCell<HashSet<String>>,
    // The same for notes saved before the model named notes.
    naming: Cell<usize>,
    named: Cell<bool>,
    unnamed: RefCell<HashSet<String>>,
    // New cards already checked for being a repeat save, so the early and the later analysis check once.
    repeat_checked: RefCell<HashSet<String>>,
}

/// The post-save job queue: capture and hosted AI, serialized and coalesced per card (`createEnrichment`'s result).
#[derive(Clone)]
pub struct Enrichment(Rc<Inner>);

// A repeat save is looked for only among cards this alike by embedding, at most this many, and merged only when the
// Decisions API is this sure, for cards saved this recently.
const REPEAT_MIN_SIMILARITY: f64 = 0.8;
const REPEAT_CANDIDATES: usize = 5;
const REPEAT_MIN_P: f64 = 0.9;
const REPEAT_WINDOW_MS: f64 = 30.0 * 60.0 * 1000.0;

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
    let decisions = if env_set("OPENAI_API_KEY") { &["OPENAI_BASE_URL"][..] } else { &[] };
    !offline || ["GEMINI_BASE_URL", "AI_CHAT_BASE_URL"].iter().chain(decisions).all(|name| loopback(name))
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
    signature_of(row, store, true)
}

// The card's words without its picture: an analysis started before the picture arrived still holds when only the
// picture changed since.
fn words_signature(row: &CardRow, store: &Store) -> Result<String> {
    signature_of(row, store, false)
}

fn signature_of(row: &CardRow, store: &Store, with_cover: bool) -> Result<String> {
    let (data, extras) = (parse(&row.data)?, parse(&row.extras)?);
    let cover = if with_cover { extras.get("cover").clone() } else { Value::Undefined };
    let parts = [data.get("title"), data.get("description"), data.get("prose"), data.get("source"), extras.get("captureText")]
        .into_iter()
        .cloned()
        .chain([cover, article_content(row, store)?])
        .collect();
    Ok(json::stringify(&Value::Array(parts))?.unwrap_or_default())
}

// Set once by a capture, awaited by the analysis that starts as soon as a link's words are read.
#[derive(Default)]
struct Signal {
    set: Cell<bool>,
    waker: RefCell<Option<Waker>>,
}

impl Signal {
    fn fire(&self) {
        self.set.set(true);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    async fn wait(&self) {
        poll_fn(|context| {
            if self.set.get() {
                Poll::Ready(())
            } else {
                *self.waker.borrow_mut() = Some(context.waker().clone());
                Poll::Pending
            }
        })
        .await;
    }
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Undefined | Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => Some(other.to_js_string()),
    }
}

// What one step of the naming sweep did.
enum Naming {
    Next,
    Failed,
    Done,
}

// Notes without a name, newest first: visible or private, never drafts, stubs or trash.
fn unnamed_notes(store: &Store, limit: usize) -> Result<Vec<String>> {
    let types = NAMED_NOTE_TYPES.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>().join(", ");
    let rows = store.db().query(
        &format!(
            "SELECT id, data, extras FROM cards WHERE user_id = ? AND deleted_ns IS NULL AND (flags & {HIDDEN}) = 0 AND type IN ({types})
               AND trim(COALESCE(json_extract(data, '$.title'), '')) = '' AND trim(COALESCE(json_extract(extras, '$.title'), '')) = ''
               AND COALESCE(json_extract(extras, '$.titleEdited'), 0) = 0
             ORDER BY bumped_ns DESC LIMIT ?"
        ),
        &[store.default_user_id()?.into(), (limit as i64).into()],
    )?;
    let mut ids = Vec::new();
    for row in &rows {
        if note_needs_name(&parse(row.text("data").unwrap_or(""))?, &parse(row.text("extras").unwrap_or(""))?) {
            ids.extend(row.text("id").map(str::to_owned));
        }
    }
    Ok(ids)
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
        Self(Rc::new(Inner {
            options,
            pending: RefCell::new(IndexMap::new()),
            closed: Cell::new(false),
            running: Cell::new(false),
            sweep: Cell::new(0),
            swept: Cell::new(false),
            unhashed: RefCell::default(),
            naming: Cell::new(0),
            named: Cell::new(false),
            unnamed: RefCell::default(),
            repeat_checked: RefCell::default(),
        }))
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
        self.run();
    }

    /// Give the gallery pictures of up to `limit` visible cards a ThumbHash, newest first, one card at a time whenever
    /// no saved card waits for enrichment. New cards get theirs after capture; this sweep is for the ones saved before.
    /// Once a sweep finds none left, later calls do nothing until the queue is made anew, and so does a picture that
    /// failed for a reason that may pass, such as an unreachable media container.
    ///
    /// # Arguments
    /// * `limit` - Most pictures this call hashes, so a host can spread the work over its requests.
    pub fn hash_pictures(&self, limit: usize) {
        let inner = &self.0;
        if inner.closed.get() || inner.options.readonly || inner.swept.get() {
            return;
        }
        inner.sweep.set(inner.sweep.get().max(limit));
        self.run();
    }

    /// Name up to `limit` notes that have no name, newest first, one at a time whenever no saved card waits for
    /// enrichment and no picture waits for its ThumbHash. New notes get theirs from their analysis; this sweep is for
    /// the ones saved before. Each takes one call to the chat model and changes only the note's title. A note the
    /// model could not name is left for the next queue, and the failure ends this call's naming so a provider that is
    /// down or rate-limiting is not asked again at once. Once a sweep finds none left, later calls do nothing until the
    /// queue is made anew. Nothing runs without hosted AI ([`Enrichment::enabled`]).
    ///
    /// # Arguments
    /// * `limit` - Most notes this call names, so a host can spread the work over its requests.
    pub fn name_notes(&self, limit: usize) {
        let inner = &self.0;
        if inner.closed.get() || inner.named.get() || !self.enabled() {
            return;
        }
        inner.naming.set(inner.naming.get().max(limit));
        self.run();
    }

    // Work through the queue on a later turn of the event loop, unless that is under way.
    fn run(&self) {
        if self.0.running.replace(true) {
            return;
        }
        let queue = self.clone();
        runtime::defer(async move {
            runtime::sleep(0).await;
            while !queue.0.closed.get() {
                let next = queue.0.pending.borrow_mut().shift_remove_index(0);
                match next {
                    Some((id, capture)) => {
                        if queue.process_card(&id, capture).await.is_err() && !queue.0.closed.get() {
                            runtime::log("[capture] enrichment failed; saved card retained");
                        }
                    }
                    None if queue.0.sweep.get() > 0 => {
                        queue.0.sweep.set(queue.0.sweep.get() - 1);
                        if !queue.hash_next().await {
                            queue.0.sweep.set(0);
                            queue.0.swept.set(true);
                        }
                    }
                    None if queue.0.naming.get() > 0 => {
                        queue.0.naming.set(queue.0.naming.get() - 1);
                        match queue.name_next().await {
                            Naming::Next => {}
                            Naming::Failed => queue.0.naming.set(0),
                            Naming::Done => {
                                queue.0.naming.set(0);
                                queue.0.named.set(true);
                            }
                        }
                    }
                    None => break,
                }
            }
            queue.0.running.set(false);
        });
    }

    // Hash the newest picture still without a ThumbHash; false when none is left.
    async fn hash_next(&self) -> bool {
        let inner = &self.0;
        let skipped = inner.unhashed.borrow().len();
        let Ok(waiting) = thumb_hash::missing(&inner.options.store, skipped + 1) else { return false };
        let Some(id) = waiting.into_iter().find(|id| !inner.unhashed.borrow().contains(id)) else { return false };
        if !self.hash_picture(&id).await {
            inner.unhashed.borrow_mut().insert(id);
        }
        true
    }

    // Name the newest note still without a name.
    async fn name_next(&self) -> Naming {
        let inner = &self.0;
        let skipped = inner.unnamed.borrow().len();
        let Ok(waiting) = unnamed_notes(&inner.options.store, skipped + 1) else { return Naming::Done };
        let Some(id) = waiting.into_iter().find(|id| !inner.unnamed.borrow().contains(id)) else { return Naming::Done };
        match self.name(&id).await {
            Ok(true) => Naming::Next,
            Ok(false) => {
                inner.unnamed.borrow_mut().insert(id);
                Naming::Next
            }
            Err(_) => {
                inner.unnamed.borrow_mut().insert(id);
                if !inner.closed.get() {
                    runtime::log("[ai] naming a note failed; note retained");
                }
                Naming::Failed
            }
        }
    }

    // Ask the model for one note's name and store it unless the note changed meanwhile; false when the note has no
    // words to name it by or took no name.
    async fn name(&self, id: &str) -> Result<bool> {
        let options = &self.0.options;
        let store = &options.store;
        let Some(row) = store.card(id, None)? else { return Ok(false) };
        if deleted(&row) || row.flags & HIDDEN != 0 {
            return Ok(false);
        }
        let item = analysis_item(&row.data, &row.extras, None)?;
        if text_of(&item.title).is_some_and(|title| !title.trim().is_empty()) || item.text.trim().is_empty() {
            return Ok(false);
        }
        let before = signature(&row, store)?;
        let item = AiItem { title: None, kind: text_of(&item.kind), text: Some(item.text), url: text_of(&item.url), domain: text_of(&item.domain), image: None };
        let description = Ai::new(&*options.fetch).describe(&item).await?;
        if self.0.closed.get() {
            return Ok(false);
        }
        let Some(current) = store.card(id, None)? else { return Ok(false) };
        if signature(&current, store)? != before {
            return Ok(false);
        }
        options.writer.run(|writer| writer.name_note(id, &description.title))
    }

    // Whether the card's gallery picture took a ThumbHash; a failure leaves the card as it was.
    async fn hash_picture(&self, id: &str) -> bool {
        let options = &self.0.options;
        match thumb_hash::fill(&options.store, &options.writer, &*options.media, id).await {
            Ok(hashed) => hashed,
            Err(_) => {
                runtime::log("[media] picture preview failed; card retained");
                false
            }
        }
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
        // A link's analysis starts as soon as its words are read, beside the rest of the capture (pictures,
        // screenshot, comments), so its tags reach the share sheet while that is still open.
        let mut early = None;
        if capture {
            let words = Signal::default();
            let over = Cell::new(false);
            let captured = async {
                let result = self.capture(&row, &words).await;
                over.set(true);
                words.fire();
                result
            };
            let (result, judged) = join(captured, self.analyze_early(id, &words, &over)).await;
            result?;
            early = judged;
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
        self.hash_picture(id).await;
        if inner.closed.get() {
            return Ok(());
        }
        self.transcribe(id).await;
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
        if early.is_some() && early == Some(words_signature(&row, store)?) {
            if row.flags & PROCESSING != 0 {
                self.processing(id, false, false)?;
            }
            self.merge_repeat(id).await;
            return Ok(());
        }
        self.processing(id, true, false)?;
        let row = store.card(id, None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'data')"))?;
        let before = signature(&row, store)?;
        if self.analyze(&row, &before).await.is_err() && !inner.closed.get() {
            runtime::log("[ai] analysis failed; saved card retained");
            self.processing(id, false, false)?;
        } else {
            self.merge_repeat(id).await;
        }
        Ok(())
    }

    // Write what a saved video or audio file says into `extras.transcript` once, so tags, summary and search read it. A video with
    // no sound keeps an empty text, so it is not sent again; a failed call leaves the card as it was.
    async fn transcribe(&self, id: &str) {
        let options = &self.0.options;
        if options.readonly || !env_set("OPENAI_API_KEY") || (options.offline && !loopback("OPENAI_BASE_URL")) {
            return;
        }
        let Ok(Some(row)) = options.store.card(id, None) else { return };
        let (Ok(data), Ok(extras)) = (parse(&row.data), parse(&row.extras)) else { return };
        if deleted(&row) || !matches!(data.get("type").as_str(), Some("Video" | "Audio")) || !extras.get("transcript").is_nullish() {
            return;
        }
        let transcript = match options.media.audio(data.get("blob")).await {
            Ok(None) => Value::from(obj! { "text" => "" }),
            Ok(Some(audio)) if audio.len() > TRANSCRIBE_MAX_BYTES => {
                runtime::log("[ai] transcription skipped: the sound is over 25 MB");
                return;
            }
            Ok(Some(audio)) => match Ai::new(&*options.fetch).transcribe(&audio, "audio.ogg").await {
                Ok(heard) => obj! { "text" => heard.text, "language" => heard.language }.into(),
                Err(error) => {
                    runtime::log(&format!("[ai] transcription failed: {}", error.message));
                    return;
                }
            },
            Err(error) => {
                runtime::log(&format!("[ai] audio extraction failed: {error}"));
                return;
            }
        };
        if self.0.closed.get() {
            return;
        }
        let mut baseline = Value::from(obj! { "data" => data, "extras" => extras });
        let patch = Value::from(obj! { "data" => obj! {}, "extras" => obj! { "transcript" => transcript } });
        if options.writer.run(|writer| writer.enrich(id, &patch, &mut baseline)).is_err() {
            runtime::log("[ai] transcript could not be saved");
        }
    }

    // Analyse a link once capture has written its words, unless the card needs its picture to be understood. Returns
    // the words' signature when the result was stored, so the analysis after capture can be skipped while the words
    // stay the same; None when nothing was stored and that analysis must run.
    async fn analyze_early(&self, id: &str, words: &Signal, over: &Cell<bool>) -> Option<String> {
        words.wait().await;
        let inner = &self.0;
        if over.get() || inner.closed.get() || !self.enabled() {
            return None;
        }
        let store = &inner.options.store;
        let row = store.card(id, None).ok()??;
        if deleted(&row) || row.flags & HIDDEN != 0 {
            return None;
        }
        let item = analysis_item(&row.data, &row.extras, Some(&article_content(&row, store).ok()?)).ok()?;
        let peek = AiItem { kind: text_of(&item.kind), text: Some(item.text), ..AiItem::default() };
        if needs_picture(&peek) {
            return None;
        }
        let before = words_signature(&row, store).ok()?;
        let value = self.analysis_of(&row).await.ok()?;
        if inner.closed.get() {
            return None;
        }
        let current = store.card(id, None).ok()??;
        if deleted(&current) || words_signature(&current, store).ok()? != before {
            return None;
        }
        options_write(&inner.options, id, &value).ok()?;
        Some(before)
    }

    async fn capture(&self, row: &CardRow, words: &Signal) -> Result<()> {
        let options = &self.0.options;
        let baseline = RefCell::new(Value::from(obj! { "data" => parse(&row.data)?, "extras" => parse(&row.extras)? }));
        let user = options.store.user(None)?.ok_or_else(|| Error::invalid("Cannot read properties of undefined (reading 'accountUid')"))?;
        let user = MediaUser { account_uid: user.account_uid, id_prefix: user.id_prefix };
        let update = |patch: Value| -> Result<()> {
            if !self.0.closed.get() {
                options.writer.run(|writer| writer.enrich(&row.id, &patch, &mut baseline.borrow_mut()))?;
                if patch.get("data").get("type").truthy() && !patch.get("extras").get("captureFailed").truthy() {
                    words.fire();
                }
            }
            Ok(())
        };
        let capture = CaptureOptions { id: &row.id, user: &user, media: &*options.media, update: &update, offline: options.offline };
        capture_link(&|| baseline.borrow().get("data").clone(), &capture, &*options.resource, &*options.screenshot).await?;
        if self.0.closed.get() {
            return Ok(());
        }
        let captured = baseline.borrow().clone();
        capture_more(&captured, &capture, &*options.resource).await
    }

    async fn analyze(&self, row: &CardRow, before: &str) -> Result<()> {
        let options = &self.0.options;
        let store = &options.store;
        let value = self.analysis_of(row).await?;
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
        options_write(options, &row.id, &value)
    }

    // A card as the hosted models read it: its words and its picture.
    async fn ai_item(&self, row: &CardRow) -> Result<AiItem> {
        let options = &self.0.options;
        let (data, extras) = (parse(&row.data)?, parse(&row.extras)?);
        // An audio file has no picture, so its blob is never sent to be drawn.
        let blob = if data.get("type").as_str() == Some("Audio") { &Value::Undefined } else { data.get("blob") };
        let source = first_present(&[extras.get("cover"), extras.get("post").get("gallery").at(0), blob]);
        let image = options.media.image(source).await.ok().flatten();
        let item = analysis_item(&row.data, &row.extras, Some(&article_content(row, &options.store)?))?;
        Ok(AiItem {
            title: text_of(&item.title),
            kind: text_of(&item.kind),
            text: Some(item.text),
            url: text_of(&item.url),
            domain: text_of(&item.domain),
            image: image.map(|image| AiImage { mime: image.mime, data: image.data }),
        })
    }

    // Fold a card saved in the last 30 minutes into the earlier card it repeats, when OpenAI is at least 90% sure
    // they are the same thing; each card is checked once. A failure leaves both cards as they are.
    async fn merge_repeat(&self, id: &str) {
        let inner = &self.0;
        if !env_set("OPENAI_API_KEY") || !inner.repeat_checked.borrow_mut().insert(id.to_owned()) {
            return;
        }
        if let Err(error) = self.try_merge_repeat(id).await {
            runtime::log(&format!("[ai] repeat check for {id} failed: {error}"));
        }
    }

    async fn try_merge_repeat(&self, id: &str) -> Result<()> {
        let options = &self.0.options;
        let store = &options.store;
        let Some(row) = store.card(id, None)? else { return Ok(()) };
        if deleted(&row) || row.flags & HIDDEN != 0 || runtime::now_ms() - row.created_ns as f64 / 1e6 > REPEAT_WINDOW_MS {
            return Ok(());
        }
        let user = store.default_user_id()?;
        let older: HashSet<String> = store
            .db()
            .query("SELECT id FROM cards WHERE user_id = ? AND deleted_ns IS NULL AND created_ns < ?", &[user.into(), SqlValue::big(row.created_ns)])?
            .iter()
            .filter_map(|found| found.text("id").map(str::to_owned))
            .collect();
        let eligible = || Ok(older.clone());
        let semantic = Semantic::new(store.clone(), None, options.offline);
        let near: Vec<String> = semantic
            .related(id, &Value::Number(REPEAT_CANDIDATES as f64), Eligible::Set(&eligible))?
            .into_iter()
            .filter(|found| found.score >= REPEAT_MIN_SIMILARITY)
            .map(|found| found.id)
            .collect();
        if near.is_empty() {
            return Ok(());
        }
        let item = self.ai_item(&row).await?;
        let mut earlier = Vec::new();
        for other in &near {
            if let Some(other) = store.card(other, None)? {
                earlier.push(self.ai_item(&other).await?);
            }
        }
        let same = Ai::new(&*options.fetch).same_as(&item, &earlier).await.map_err(|error| Error::internal(error.message))?;
        let best = same.iter().enumerate().filter(|(_, p)| **p >= REPEAT_MIN_P).max_by(|a, b| a.1.total_cmp(b.1));
        if let Some((index, p)) = best
            && !self.0.closed.get()
        {
            let target = &near[index];
            if options.writer.run(|writer| writer.merge_into(id, target))? {
                runtime::log(&format!("[ai] merged repeat save {id} into {target} (p {p})"));
            }
        }
        Ok(())
    }

    // Ask the hosted models for the card's summary, tags and embedding, ready for the analysis write.
    async fn analysis_of(&self, row: &CardRow) -> Result<Value> {
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
        let item = self.ai_item(row).await?;
        let ai = Ai::new(&*options.fetch);
        let spaces = if env_set("OPENAI_API_KEY") { space_options(store, &row.id)? } else { Vec::new() };
        let (result, suggested) = join(ai.analyze(&item, &vocabulary), async {
            if spaces.is_empty() { Ok(None) } else { ai.suggest_space(&item, &spaces).await }
        })
        .await;
        let mut value = result?.to_value();
        // A failed suggestion only leaves the card without one; the phone falls back to its own guess.
        match suggested {
            Ok(Some(space)) => value.set("suggestedSpace", obj! { "id" => space.id, "p" => space.p }),
            Ok(None) => {}
            Err(error) => runtime::log(&format!("space suggestion for {} failed: {}", row.id, error.message)),
        }
        // A summary already stored, and not one this backfill marked as its own, is left in place. The result
        // otherwise matches the historical analysis write, including cards that receive their first summary.
        if summary_owned(&data, &extras) {
            value.set("preserveSummary", true);
        }
        Ok(value)
    }
}

// The user's own Spaces the card is not in yet, in their order, each with up to three titles of its newest cards.
fn space_options(store: &Store, card_id: &str) -> Result<Vec<SpaceOption>> {
    let db = store.db();
    let rows = db.query(
        "SELECT id, name FROM spaces WHERE user_id = ? AND query IS NULL
           AND id NOT IN (SELECT space_id FROM space_cards WHERE card_id = ? AND excluded = 0)
         ORDER BY position, id",
        &[store.default_user_id()?.into(), card_id.into()],
    )?;
    let mut spaces = Vec::new();
    for row in &rows {
        let id = row.text("id").unwrap_or("").to_owned();
        let examples = db
            .query(
                "SELECT json_extract(c.data, '$.title') AS title FROM space_cards s JOIN cards c ON c.id = s.card_id
                 WHERE s.space_id = ? AND s.excluded = 0 AND c.deleted_ns IS NULL AND c.id != ?
                   AND trim(COALESCE(json_extract(c.data, '$.title'), '')) != ''
                 ORDER BY c.created_ns DESC LIMIT 3",
                &[id.as_str().into(), card_id.into()],
            )?
            .iter()
            .filter_map(|found| found.text("title").map(str::to_owned))
            .collect();
        spaces.push(SpaceOption { id, name: row.text("name").unwrap_or("").to_owned(), examples });
    }
    Ok(spaces)
}

fn options_write(options: &EnrichmentOptions, id: &str, value: &Value) -> Result<()> {
    options.writer.run(|writer| writer.analysis(id, value))
}
