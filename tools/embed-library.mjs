import { DatabaseSync } from 'node:sqlite';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { documentText, embedTexts } from '../lib/ai.mjs';
import { analysisItem } from '../lib/search-document.mjs';
import { embeddingsEnabled } from '../lib/search-semantic.mjs';
import { openStore, GRID_HIDDEN_FLAGS } from '../lib/store.mjs';

// Gemini Developer API standard text input, checked 2026-10-15:
// https://ai.google.dev/gemini-api/docs/pricing#gemini-embedding-2 ($0.20 / 1M; batchEmbedContents is synchronous).
const TEXT_USD_PER_MILLION = 0.20;
const BATCH_SIZE = 100;

// Parse explicit execution and database flags; a dry run never opens a writable connection or reads .env.
function options(args) {
  const result = { db: resolve('data/mymind.db'), limit: Infinity, run: false };
  for (let at = 0; at < args.length; at++) {
    if (args[at] === '--run') result.run = true;
    else if (args[at] === '--db' && args[at + 1]) result.db = resolve(args[++at]);
    else if (args[at] === '--limit' && args[at + 1]) {
      const raw = args[++at];
      if (!/^\d+$/.test(raw) || !Number.isSafeInteger(Number(raw))) throw new Error('--limit must be a nonnegative integer');
      result.limit = Number(raw);
    } else throw new Error('Usage: node tools/embed-library.mjs [--db path] [--limit N] [--run]');
  }
  return result;
}

/**
 * Estimate or run resumable text-only Gemini embeddings for cards without vectors.
 * @param {string[]} [args] CLI arguments; --run explicitly enables hosted calls and vector writes.
 * @param {{log?:Function}} [dependencies] Optional progress logger for in-process checks.
 * @returns {Promise<object>} Counts, estimate and API-reported token totals, without private document text.
 */
export async function embedLibrary(args = process.argv.slice(2), { log = console.log } = {}) {
  const config = options(args);
  const db = new DatabaseSync(config.db, { readOnly: true });
  let pending, empty = 0;
  try {
    pending = db.prepare(`SELECT c.id, c.user_id, c.type, c.data, c.extras, b.body FROM cards c
      LEFT JOIN card_content b ON b.card_id = c.id LEFT JOIN embeddings e ON e.card_id = c.id
      WHERE e.card_id IS NULL AND c.deleted_ns IS NULL AND (c.flags & ${GRID_HIDDEN_FLAGS}) = 0 ORDER BY c.id`)
      .all().flatMap(row => {
        const item = analysisItem(row, row.body ? JSON.parse(row.body) : null);
        if (!documentText(item)) item.text = JSON.parse(row.data).summary ?? '';
        if (!documentText(item)) { empty++; return []; }
        return [{ id: row.id, userId: row.user_id, item }];
      }).slice(0, config.limit);
  } finally { db.close(); }
  const tokens = pending.reduce((sum, row) => sum + Math.ceil(documentText(row.item).length / 4), 0);
  const result = { cards: pending.length, skippedEmpty: empty, estimatedTokens: tokens,
    estimatedCost: tokens / 1_000_000 * TEXT_USD_PER_MILLION, embedded: 0, reportedTokens: 0, usageReported: false };
  log(`Cards without vectors: ${result.cards}; skipped without text: ${empty}`);
  log(`Estimated text input: ${tokens} tokens (4 characters/token); estimated cost: $${result.estimatedCost.toFixed(6)} USD at $${TEXT_USD_PER_MILLION}/1M`);
  if (!config.run) { log('Estimate only. Add --run to embed; existing vectors are always skipped.'); return result; }
  if (process.env.STORE_READONLY === '1') throw new Error('Backfill is disabled by STORE_READONLY=1');
  if (!pending.length) return result;
  if (!embeddingsEnabled(process.env.OFFLINE === '1')) throw new Error('GEMINI_API_KEY and an allowed Gemini endpoint are required');
  const store = openStore(config.db);
  try {
    for (let start = 0; start < pending.length; start += BATCH_SIZE) {
      const batch = pending.slice(start, start + BATCH_SIZE).filter(row => !store.db.prepare('SELECT 1 FROM embeddings WHERE card_id = ?').get(row.id));
      if (!batch.length) continue;
      const vectors = await embedTexts(batch.map(row => row.item), { kind: 'document', onUsage: usage => {
        const count = usage?.promptTokenCount ?? usage?.totalTokenCount ?? usage?.inputTokenCount;
        if (Number.isFinite(count)) { result.reportedTokens += count; result.usageReported = true; }
      } });
      store.transaction(() => {
        const insert = store.db.prepare(`INSERT INTO embeddings (card_id, user_id, dimensions, vector)
          SELECT id, user_id, 768, ? FROM cards WHERE id = ? AND user_id = ? AND deleted_ns IS NULL AND (flags & ${GRID_HIDDEN_FLAGS}) = 0
          ON CONFLICT(card_id) DO NOTHING`);
        batch.forEach((row, i) => {
          const current = store.card(row.id, row.userId);
          if (!current) return;
          const content = store.db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
          const item = analysisItem(current, content ? JSON.parse(content.body) : null);
          if (!documentText(item)) item.text = JSON.parse(current.data).summary ?? '';
          if (documentText(item) !== documentText(row.item)) return;
          const bytes = Buffer.alloc(3072);
          vectors[i].forEach((value, index) => bytes.writeFloatLE(value, index * 4));
          result.embedded += Number(insert.run(bytes, row.id, row.userId).changes);
        });
      });
      log(`Embedded ${result.embedded}/${result.cards}; API input tokens: ${result.usageReported ? result.reportedTokens : 'not reported'}`);
    }
  } finally { store.close(); }
  return result;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try { await embedLibrary(); }
  catch { console.error('Embedding backfill failed; completed batches are retained. Check flags, database and hosted API configuration.'); process.exitCode = 1; }
}
