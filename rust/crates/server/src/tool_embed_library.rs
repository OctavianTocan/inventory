//! `mymind embed-library`: the port of `tools/embed-library.mjs`, which estimates or runs resumable text-only Gemini
//! embeddings for cards without vectors.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use mm_core::ai::{Ai, EmbedInput, EmbedKind, AiItem, document_text};
use mm_core::search_semantic::embeddings_enabled;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::{GRID_HIDDEN_FLAGS, Store};
use mm_core::{Error, Result, Value};
use mm_js::{number, text};
use mm_sqlite::Sqlite;

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::node_path::resolve;
use crate::tools::embedding_item;

const TEXT_USD_PER_MILLION: f64 = 0.20;
const BATCH_SIZE: usize = 100;
const USAGE: &str = "Usage: mymind embed-library [--db path] [--limit N] [--run]";
const FAILED: &str = "Embedding backfill failed; completed batches are retained. Check flags, database and hosted API configuration.";

struct Config {
    db: PathBuf,
    limit: usize,
    run: bool,
}

struct Pending {
    id: String,
    user_id: String,
    item: AiItem,
}

// Parse explicit execution and database flags (`options`).
fn options(args: &[String]) -> Result<Config> {
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let mut config = Config { db: resolve(&cwd, "data/mymind.db"), limit: usize::MAX, run: false };
    let mut at = 0;
    while at < args.len() {
        let next = args.get(at + 1).filter(|value| !value.is_empty());
        match (args[at].as_str(), next) {
            ("--run", _) => config.run = true,
            ("--db", Some(path)) => {
                config.db = resolve(&cwd, path);
                at += 1;
            }
            ("--limit", Some(raw)) => {
                at += 1;
                let number = number::string_to_number(raw);
                if !raw.bytes().all(|byte| byte.is_ascii_digit()) || !number::is_safe_integer(number) {
                    return Err(Error::internal("--limit must be a nonnegative integer"));
                }
                config.limit = number as usize;
            }
            _ => return Err(Error::internal(USAGE)),
        }
        at += 1;
    }
    Ok(config)
}

fn pending_cards(path: &Path, limit: usize) -> Result<(Vec<Pending>, usize)> {
    let db = Sqlite::open(path, true)?;
    let rows = db.query(
        &format!(
            "SELECT c.id, c.user_id, c.type, c.data, c.extras, b.body FROM cards c
      LEFT JOIN card_content b ON b.card_id = c.id LEFT JOIN embeddings e ON e.card_id = c.id
      WHERE e.card_id IS NULL AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0 ORDER BY c.id"
        ),
        &[],
    )?;
    let mut empty = 0;
    let mut pending = Vec::new();
    for row in &rows {
        let item = embedding_item(row.text("data").unwrap_or(""), row.text("extras").unwrap_or("null"), row.text("body"))?;
        if document_text(&item).is_empty() {
            empty += 1;
        } else {
            pending.push(Pending { id: row.text("id").unwrap_or("").to_owned(), user_id: row.text("user_id").unwrap_or("").to_owned(), item });
        }
    }
    pending.truncate(limit);
    Ok((pending, empty))
}

fn reported_tokens(usage: &Value) -> Option<f64> {
    let count = [usage.get("promptTokenCount"), usage.get("totalTokenCount"), usage.get("inputTokenCount")].into_iter().find(|value| !matches!(value, Value::Undefined | Value::Null))?;
    count.as_f64().filter(|number| number.is_finite())
}

fn vector_bytes(vector: &[f32]) -> Vec<u8> {
    let mut bytes = vec![0u8; 3072];
    for (index, value) in vector.iter().take(768).enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

async fn embed_pending(path: &Path, pending: &[Pending], fetch: &NativeFetch) -> Result<()> {
    let store = Store::new(Rc::new(Sqlite::open(path, false)?))?;
    let db = Rc::clone(store.db());
    let (mut embedded, mut reported, mut usage_reported) = (0usize, 0f64, false);
    let ai = Ai::new(fetch);
    for start in (0..pending.len()).step_by(BATCH_SIZE) {
        let mut batch = Vec::new();
        for row in &pending[start..(start + BATCH_SIZE).min(pending.len())] {
            if db.one("SELECT 1 FROM embeddings WHERE card_id = ?", &[row.id.as_str().into()])?.is_none() {
                batch.push(row);
            }
        }
        if batch.is_empty() {
            continue;
        }
        let inputs: Vec<EmbedInput> = batch.iter().map(|row| EmbedInput::Item(&row.item)).collect();
        let mut on_usage = |usage: &Value| {
            if let Some(count) = reported_tokens(usage) {
                reported += count;
                usage_reported = true;
            }
        };
        let vectors = ai.embed_texts(&inputs, EmbedKind::Document, Some(&mut on_usage)).await.map_err(|error| Error::internal(error.to_string()))?;
        store.transaction(|| {
            for (row, vector) in batch.iter().zip(&vectors) {
                let Some(current) = store.card(&row.id, Some(&row.user_id))? else { continue };
                let content = db.one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])?;
                let extras = if current.extras.is_empty() { "null" } else { current.extras.as_str() };
                let now = embedding_item(&current.data, extras, content.as_ref().and_then(|content| content.text("body")))?;
                if document_text(&now) != document_text(&row.item) {
                    continue;
                }
                embedded += db.execute(
                    &format!(
                        "INSERT INTO embeddings (card_id, user_id, dimensions, vector)
          SELECT id, user_id, 768, ? FROM cards WHERE id = ? AND user_id = ? AND deleted_ns IS NULL AND (flags & {GRID_HIDDEN_FLAGS}) = 0
          ON CONFLICT(card_id) DO NOTHING"
                    ),
                    &[SqlValue::from(vector_bytes(vector)), row.id.as_str().into(), row.user_id.as_str().into()],
                )?;
            }
            Ok(())
        })?;
        let tokens = if usage_reported { number::number_to_string(reported) } else { "not reported".to_owned() };
        println!("Embedded {embedded}/{}; API input tokens: {tokens}", pending.len());
    }
    Ok(())
}

async fn work(args: &[String]) -> Result<()> {
    let config = options(args)?;
    let (pending, empty) = pending_cards(&config.db, config.limit)?;
    let tokens: usize = pending.iter().map(|row| text::len(&document_text(&row.item)).div_ceil(4)).sum();
    let cost = tokens as f64 / 1_000_000.0 * TEXT_USD_PER_MILLION;
    println!("Cards without vectors: {}; skipped without text: {empty}", pending.len());
    println!("Estimated text input: {tokens} tokens (4 characters/token); estimated cost: ${} USD at ${}/1M", number::to_fixed(cost, 6), number::number_to_string(TEXT_USD_PER_MILLION));
    if !config.run {
        println!("Estimate only. Add --run to embed; existing vectors are always skipped.");
        return Ok(());
    }
    if std::env::var("STORE_READONLY").is_ok_and(|value| value == "1") {
        return Err(Error::internal("Backfill is disabled by STORE_READONLY=1"));
    }
    if pending.is_empty() {
        return Ok(());
    }
    if !embeddings_enabled(std::env::var("OFFLINE").is_ok_and(|value| value == "1")) {
        return Err(Error::internal("GEMINI_API_KEY and an allowed Gemini endpoint are required"));
    }
    embed_pending(&config.db, &pending, &NativeFetch::new()).await
}

/// Estimate or run the vector backfill (`embedLibrary`, as the command line runs it).
///
/// # Arguments
/// * `root` - The repository root. `.env` there fills keys the process has not set. The default database still follows the current directory.
/// * `args` - The arguments after the subcommand name: `[--db path] [--limit N] [--run]`.
///
/// # Returns
/// The exit status: 0, or 1 after printing the single failure line.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    let file = match env_file::load(&env_file::path(root)) {
        Ok(Loaded::Vars(vars)) => vars,
        _ => HashMap::new(),
    };
    mm_core::runtime::set_env(env_file::merged(std::env::vars(), file));
    match work(args).await {
        Ok(()) => 0,
        Err(_) => {
            eprintln!("{FAILED}");
            1
        }
    }
}
