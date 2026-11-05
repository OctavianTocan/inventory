//! Resumable AI backfill for cards that still lack a summary, tags or an embedding.
//!
//! Fills gaps only. A summary already stored, a summary the user edited, and every existing tag and vector stay.
//! Cards with nothing to read are left unmarked so a later edit can enrich them. A 429 stops the batch after the
//! provider's own retries; cards committed earlier in the batch stay committed.

use std::rc::Rc;

use async_trait::async_trait;
use mm_js::{Value, json, obj, text};

use crate::ai::{Ai, AiError, AiImage, AiItem, EmbedInput, EmbedKind, VocabularyEntry, document_text};
use crate::ai_tags::relevant_tag;
use crate::enrichment::ai_configured;
use crate::host::{Fetch, Media};
use crate::runtime;
use crate::search_document::analysis_item;
use crate::sql::SqlExt;
use crate::store::{GRID_HIDDEN_FLAGS, Store};
use crate::store_enrich::GapFill;
use crate::store_write::Writer;
use crate::{Error, Result};

/// Cards one hosted request enriches. Small enough for a Durable Object's subrequest budget.
pub const HOSTED_BATCH: usize = 4;
/// Largest `limit` `POST /ai/backfill` accepts.
pub const HOSTED_BATCH_MAX: usize = 20;

// Published Gemini API rates, October 2026: gemini-embedding-2 text, gemini-2.5-flash-lite input and output.
const EMBED_USD_PER_MILLION: f64 = 0.20;
const CHAT_INPUT_USD_PER_MILLION: f64 = 0.10;
const CHAT_OUTPUT_USD_PER_MILLION: f64 = 0.40;
const CHAT_PROMPT_TOKENS: usize = 250;
const CHAT_OUTPUT_TOKENS: usize = 150;
const IMAGE_TOKENS: usize = 258;
const EMBED_BATCH: usize = 100;

/// Rough size of a backfill, with no network calls.
#[derive(Clone, Debug, PartialEq)]
pub struct Estimate {
    /// Cards a run would send to the model.
    pub cards: usize,
    /// Of those, cards with an empty summary the user has not edited.
    pub missing_summary: usize,
    /// Of those, cards with no AI tag and no finished tag attempt.
    pub missing_tags: usize,
    /// Of those, cards with no stored vector.
    pub missing_embedding: usize,
    /// Visible cards that already have a summary, tags and a vector.
    pub skipped_complete: usize,
    /// Visible cards with gaps but no title, text or image.
    pub skipped_empty: usize,
    /// Rough chat input tokens, including a fixed prompt overhead.
    pub chat_input_tokens: usize,
    /// Rough chat output tokens.
    pub chat_output_tokens: usize,
    /// Rough embedding input tokens.
    pub embed_tokens: usize,
    /// Rough USD from the rates above. OpenAI tag picking is not included.
    pub estimated_usd: f64,
    /// `OPENAI_API_KEY` is set, so tag picking with OpenAI's Decisions API would run. Its price is not in
    /// `estimated_usd`.
    pub decisions: bool,
}

impl Estimate {
    /// The JSON a dry run returns.
    pub fn to_value(&self) -> Value {
        obj! {
            "cards" => self.cards as f64,
            "missingSummary" => self.missing_summary as f64,
            "missingTags" => self.missing_tags as f64,
            "missingEmbedding" => self.missing_embedding as f64,
            "skippedComplete" => self.skipped_complete as f64,
            "skippedEmpty" => self.skipped_empty as f64,
            "chatInputTokens" => self.chat_input_tokens as f64,
            "chatOutputTokens" => self.chat_output_tokens as f64,
            "embedTokens" => self.embed_tokens as f64,
            "estimatedUsd" => self.estimated_usd,
            "decisions" => self.decisions,
            "rates" => obj! {
                "embedUsdPerMillion" => EMBED_USD_PER_MILLION,
                "chatInputUsdPerMillion" => CHAT_INPUT_USD_PER_MILLION,
                "chatOutputUsdPerMillion" => CHAT_OUTPUT_USD_PER_MILLION,
            },
        }
        .into()
    }
}

/// What one backfill invocation finished. Earlier cards are already committed when this returns.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// Cards whose gaps were written, including a tag attempt that added nothing.
    pub processed: usize,
    /// Summaries written.
    pub summarized: usize,
    /// AI tags inserted.
    pub tagged: usize,
    /// Vectors inserted.
    pub embedded: usize,
    /// Selected cards left unchanged because their text changed or they were already filled.
    pub skipped: usize,
    /// Cards whose model call failed for a reason other than a final 429.
    pub failed: usize,
    /// Cards still waiting after this invocation.
    pub remaining: usize,
    /// Milliseconds to wait before retrying, after a final 429. `None` when the batch was not rate-limited.
    pub retry_after_ms: Option<u64>,
    /// The model error that stopped the batch. Cards counted in `processed` are already stored.
    pub error: Option<String>,
}

impl Report {
    /// The JSON a backfill request returns.
    pub fn to_value(&self) -> Value {
        let retry = self.retry_after_ms.map_or(Value::Null, |ms| Value::from(ms as f64));
        let error = self.error.as_deref().map_or(Value::Null, Value::from);
        obj! {
            "processed" => self.processed as f64,
            "summarized" => self.summarized as f64,
            "tagged" => self.tagged as f64,
            "embedded" => self.embedded as f64,
            "skipped" => self.skipped as f64,
            "failed" => self.failed as f64,
            "remaining" => self.remaining as f64,
            "retryAfterMs" => retry,
            "error" => error,
        }
        .into()
    }
}

/// Loads the cover image a card's analysis would send.
#[async_trait(?Send)]
pub trait Images {
    /// The image at `source`, or `None` when there is none or it cannot be read.
    async fn load(&self, source: &Value) -> Option<AiImage>;
}

/// [`Images`] over a library [`Media`] handle. A read failure becomes `None`.
pub struct MediaImages(pub Rc<dyn Media>);

#[async_trait(?Send)]
impl Images for MediaImages {
    async fn load(&self, source: &Value) -> Option<AiImage> {
        match self.0.image(source).await {
            Ok(Some(image)) => Some(AiImage { mime: image.mime, data: image.data }),
            _ => None,
        }
    }
}

struct Work {
    id: String,
    item: AiItem,
    source: Value,
    fingerprint: String,
    needs_summary: bool,
    needs_tags: bool,
    needs_embedding: bool,
    has_image: bool,
}

enum Stop {
    Rate(u64),
    Failed(String),
}

/// Count cards and estimate tokens and USD. Does not call a model or write.
///
/// # Arguments
/// * `store` - Open library.
///
/// # Errors
/// The database's error, or malformed card JSON.
pub fn estimate(store: &Store) -> Result<Estimate> {
    Ok(classify(store)?.0)
}

/// Fill gaps for up to `limit` cards, committing each card (or embedding batch) before the next.
///
/// # Arguments
/// * `store` - Open library. Read again after each model call so an edit during the call is not overwritten.
/// * `writer` - Writer for the gap updates.
/// * `fetch` - Outbound HTTP for Gemini and, when configured, OpenAI.
/// * `images` - Cover images. A missing image does not fail the card.
/// * `limit` - Maximum cards to attempt. Zero writes nothing.
///
/// # Errors
/// `GEMINI_API_KEY` is missing or offline mode blocks the endpoints, or the database fails. A model error is
/// returned in [`Report::error`] so earlier commits survive.
pub async fn run(store: &Store, writer: &Writer, fetch: &dyn Fetch, images: &dyn Images, limit: usize) -> Result<Report> {
    let offline = runtime::env("OFFLINE").as_deref() == Some("1");
    if !ai_configured(offline) {
        return Err(Error::status(503, "GEMINI_API_KEY is not set"));
    }
    let mut report = Report::default();
    let mut pending = classify(store)?.1;
    pending.truncate(limit);
    let ai = Ai::new(fetch);
    let mut index = 0;
    while index < pending.len() {
        if pending[index].full() {
            match one_card(store, writer, &ai, images, &pending[index]).await? {
                Ok(step) => apply(&mut report, step),
                Err(stop) => {
                    finish(store, &mut report, stop)?;
                    return Ok(report);
                }
            }
            index += 1;
            continue;
        }
        let end = (index + 1..pending.len()).find(|at| pending[*at].full() || pending[*at].has_image).unwrap_or(pending.len());
        let batch = &pending[index..end];
        for chunk in batch.chunks(EMBED_BATCH) {
            match embed_chunk(store, writer, &ai, chunk).await? {
                Ok(step) => apply(&mut report, step),
                Err(stop) => {
                    finish(store, &mut report, stop)?;
                    return Ok(report);
                }
            }
        }
        index = end;
    }
    report.remaining = estimate(store)?.cards;
    Ok(report)
}

fn apply(report: &mut Report, step: Step) {
    report.processed += step.processed;
    report.summarized += step.summarized;
    report.tagged += step.tagged;
    report.embedded += step.embedded;
    report.skipped += step.skipped;
}

fn finish(store: &Store, report: &mut Report, stop: Stop) -> Result<()> {
    match stop {
        Stop::Rate(ms) => report.retry_after_ms = Some(ms),
        Stop::Failed(message) => {
            report.failed += 1;
            report.error = Some(message);
        }
    }
    report.remaining = estimate(store)?.cards;
    Ok(())
}

struct Step {
    processed: usize,
    summarized: usize,
    tagged: usize,
    embedded: usize,
    skipped: usize,
}

fn rate_limit(error: &AiError) -> Option<u64> {
    (error.status == Some(429)).then(|| error.retry_after_ms.unwrap_or(5_000))
}

fn stop_of(error: AiError) -> Stop {
    match rate_limit(&error) {
        Some(ms) => Stop::Rate(ms),
        None => Stop::Failed(error.message),
    }
}

async fn one_card(store: &Store, writer: &Writer, ai: &Ai<'_>, images: &dyn Images, work: &Work) -> Result<std::result::Result<Step, Stop>> {
    let Some(before) = fingerprint_of(store, &work.id)? else {
        return Ok(Ok(skipped()));
    };
    if before != work.fingerprint {
        return Ok(Ok(skipped()));
    }
    let mut item = work.item.clone();
    if work.has_image {
        item.image = images.load(&work.source).await;
    }
    if work.full() {
        let kind = item.kind.clone().unwrap_or_default();
        let vocabulary = vocabulary(store, &kind)?;
        let result = match ai.analyze(&item, &vocabulary).await {
            Ok(result) => result,
            Err(error) => return Ok(Err(stop_of(error))),
        };
        if fingerprint_of(store, &work.id)?.as_deref() != Some(before.as_str()) {
            return Ok(Ok(skipped()));
        }
        let value = result.to_value();
        let tags = match value.get("tags") {
            Value::Array(items) => items.clone(),
            _ => Vec::new(),
        };
        let applied = writer.run(|writer| {
            writer.fill_gaps(
                &work.id,
                &GapFill {
                    summary: work.needs_summary.then(|| value.get("summary").to_js_string()),
                    tags: if work.needs_tags { tags } else { Vec::new() },
                    embedding: work.needs_embedding.then(|| result.embedding.clone()),
                    mark_tags_done: work.needs_tags,
                },
            )
        })?;
        return Ok(Ok(counted(work.needs_tags, applied)));
    }
    if item.image.is_none() && document_text(&item).is_empty() {
        return Ok(Ok(skipped()));
    }
    let vector = match ai.embed(&item, EmbedKind::Document).await {
        Ok(vector) => vector,
        Err(error) => return Ok(Err(stop_of(error))),
    };
    if fingerprint_of(store, &work.id)?.as_deref() != Some(before.as_str()) {
        return Ok(Ok(skipped()));
    }
    let applied = writer.run(|writer| {
        writer.fill_gaps(&work.id, &GapFill { summary: None, tags: Vec::new(), embedding: Some(vector), mark_tags_done: false })
    })?;
    Ok(Ok(counted(false, applied)))
}

async fn embed_chunk(store: &Store, writer: &Writer, ai: &Ai<'_>, chunk: &[Work]) -> Result<std::result::Result<Step, Stop>> {
    let mut ready = Vec::new();
    let mut skipped_count = 0;
    for work in chunk {
        match fingerprint_of(store, &work.id)? {
            Some(now) if now == work.fingerprint && !document_text(&work.item).is_empty() => ready.push(work),
            _ => skipped_count += 1,
        }
    }
    if ready.is_empty() {
        return Ok(Ok(Step { skipped: skipped_count, ..Step::zero() }));
    }
    let inputs: Vec<EmbedInput<'_>> = ready.iter().map(|work| EmbedInput::Item(&work.item)).collect();
    let vectors = match ai.embed_texts(&inputs, EmbedKind::Document, None).await {
        Ok(vectors) => vectors,
        Err(error) => return Ok(Err(stop_of(error))),
    };
    let mut step = Step { skipped: skipped_count, ..Step::zero() };
    for (work, vector) in ready.iter().zip(vectors) {
        if fingerprint_of(store, &work.id)?.as_deref() != Some(work.fingerprint.as_str()) {
            step.skipped += 1;
            continue;
        }
        let applied = writer.run(|writer| {
            writer.fill_gaps(&work.id, &GapFill { summary: None, tags: Vec::new(), embedding: Some(vector), mark_tags_done: false })
        })?;
        let counted = counted(false, applied);
        step.processed += counted.processed;
        step.embedded += counted.embedded;
        step.skipped += counted.skipped;
    }
    Ok(Ok(step))
}

fn skipped() -> Step {
    Step { skipped: 1, ..Step::zero() }
}

fn counted(mark_tags: bool, applied: crate::store_enrich::GapApplied) -> Step {
    let wrote = mark_tags || applied.summarized || applied.tagged > 0 || applied.embedded;
    Step {
        processed: usize::from(wrote),
        summarized: usize::from(applied.summarized),
        tagged: applied.tagged,
        embedded: usize::from(applied.embedded),
        skipped: usize::from(!wrote),
    }
}

impl Step {
    fn zero() -> Self {
        Self { processed: 0, summarized: 0, tagged: 0, embedded: 0, skipped: 0 }
    }
}

impl Work {
    fn full(&self) -> bool {
        self.needs_summary || self.needs_tags
    }
}

fn classify(store: &Store) -> Result<(Estimate, Vec<Work>)> {
    let rows = store.db().query(
        &format!(
            "SELECT c.id, c.data, c.extras, b.body,
                    CASE WHEN e.card_id IS NULL THEN 0 ELSE 1 END AS embedded,
                    CASE WHEN EXISTS (SELECT 1 FROM annotations a WHERE a.card_id = c.id AND (a.flags & 2) != 0) THEN 1 ELSE 0 END AS ai_tag
             FROM cards c
             LEFT JOIN card_content b ON b.card_id = c.id
             LEFT JOIN embeddings e ON e.card_id = c.id
             WHERE c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0
             ORDER BY c.id"
        ),
        &[],
    )?;
    let mut estimate = Estimate {
        decisions: runtime::env("OPENAI_API_KEY").is_some_and(|key| !key.is_empty()),
        ..Estimate::empty()
    };
    let mut pending = Vec::new();
    for row in &rows {
        let data = row.text("data").unwrap_or("");
        let extras = row.text("extras").unwrap_or("null");
        let (item, source) = item_of(data, extras, row.text("body"))?;
        let (data_value, extras_value) = (json::parse(data)?, json::parse(extras).unwrap_or(Value::Null));
        let summary_edited = extras_value.get("summaryEdited").truthy();
        let needs_summary = !summary_edited && !data_value.get("summary").truthy();
        let needs_tags = row.int("ai_tag") != Some(1) && !extras_value.get("aiTagsDone").truthy();
        let needs_embedding = row.int("embedded") != Some(1);
        if !needs_summary && !needs_tags && !needs_embedding {
            estimate.skipped_complete += 1;
            continue;
        }
        let has_image = source.truthy();
        if document_text(&item).is_empty() && !has_image {
            estimate.skipped_empty += 1;
            continue;
        }
        estimate.cards += 1;
        estimate.missing_summary += usize::from(needs_summary);
        estimate.missing_tags += usize::from(needs_tags);
        estimate.missing_embedding += usize::from(needs_embedding);
        let chats = needs_summary || needs_tags;
        let embed_tokens = embed_tokens(&item, has_image);
        if chats {
            let body = text::len(item.text.as_deref().unwrap_or("")).min(8_000) / 4;
            estimate.chat_input_tokens += body + CHAT_PROMPT_TOKENS + if has_image { IMAGE_TOKENS } else { 0 };
            estimate.chat_output_tokens += CHAT_OUTPUT_TOKENS;
            estimate.embed_tokens += embed_tokens;
        } else if needs_embedding {
            estimate.embed_tokens += embed_tokens;
        }
        pending.push(Work {
            id: row.text("id").unwrap_or("").to_owned(),
            fingerprint: fingerprint(&item, &source),
            item,
            source,
            needs_summary,
            needs_tags,
            needs_embedding,
            has_image,
        });
    }
    estimate.estimated_usd = estimate.chat_input_tokens as f64 / 1_000_000.0 * CHAT_INPUT_USD_PER_MILLION
        + estimate.chat_output_tokens as f64 / 1_000_000.0 * CHAT_OUTPUT_USD_PER_MILLION
        + estimate.embed_tokens as f64 / 1_000_000.0 * EMBED_USD_PER_MILLION;
    Ok((estimate, pending))
}

impl Estimate {
    fn empty() -> Self {
        Self {
            cards: 0,
            missing_summary: 0,
            missing_tags: 0,
            missing_embedding: 0,
            skipped_complete: 0,
            skipped_empty: 0,
            chat_input_tokens: 0,
            chat_output_tokens: 0,
            embed_tokens: 0,
            estimated_usd: 0.0,
            decisions: false,
        }
    }
}

fn embed_tokens(item: &AiItem, has_image: bool) -> usize {
    let text_tokens = text::len(&document_text(item)) / 4;
    text_tokens + if has_image { IMAGE_TOKENS } else { 0 }
}

fn vocabulary(store: &Store, kind: &str) -> Result<Vec<VocabularyEntry>> {
    let names = store.db().query(
        "SELECT name, COUNT(*) AS n FROM annotations WHERE user_id = ? GROUP BY name ORDER BY n DESC, name",
        &[store.default_user_id()?.into()],
    )?;
    let mut vocabulary = Vec::new();
    for name in names.iter().map(|tag| tag.text("name").unwrap_or("").to_owned()) {
        if relevant_tag(&name, kind)? {
            vocabulary.push(VocabularyEntry { name, embedding: None });
        }
    }
    Ok(vocabulary)
}

fn item_of(data: &str, extras: &str, body: Option<&str>) -> Result<(AiItem, Value)> {
    let content = match body.filter(|body| !body.is_empty()) {
        Some(body) => Some(json::parse(body)?),
        None => None,
    };
    let parsed = analysis_item(data, extras, content.as_ref())?;
    let data_value = json::parse(data)?;
    let extras_value = json::parse(extras).unwrap_or(Value::Null);
    let source = image_source(&data_value, &extras_value);
    let text_of = |value: &Value| match value {
        Value::Undefined | Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => Some(other.to_js_string()),
    };
    let item = AiItem {
        title: text_of(&parsed.title),
        kind: text_of(&parsed.kind),
        text: Some(parsed.text),
        url: text_of(&parsed.url),
        domain: text_of(&parsed.domain),
        image: None,
    };
    Ok((item, source))
}

fn image_source(data: &Value, extras: &Value) -> Value {
    let gallery = extras.get("post").get("gallery").at(0).clone();
    [extras.get("cover").clone(), gallery, data.get("blob").clone()].into_iter().find(|value| !value.is_nullish()).unwrap_or(Value::Null)
}

fn fingerprint(item: &AiItem, source: &Value) -> String {
    format!("{}\n{}", document_text(item), json::stringify(source).ok().flatten().unwrap_or_default())
}

fn fingerprint_of(store: &Store, id: &str) -> Result<Option<String>> {
    let Some(row) = store.card(id, None)? else { return Ok(None) };
    if row.deleted_ns.is_some_and(|ns| ns != 0) {
        return Ok(None);
    }
    let body = store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[id.into()])?;
    let (item, source) = item_of(&row.data, if row.extras.is_empty() { "null" } else { &row.extras }, body.as_ref().and_then(|row| row.text("body")))?;
    Ok(Some(fingerprint(&item, &source)))
}
