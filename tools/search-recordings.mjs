import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createSearch } from '../lib/search.mjs';

/**
 * Compare store results with the five recorded prefix searches without exposing card text or ids.
 * @param {object} store Open temporary store copy, never the shared seeded database.
 * @param {string} root Project directory containing read-only replay recordings.
 * @returns {Promise<object[]>} Per-query counts, overlap and field evidence for the offline report.
 */
export async function compareRecordings(store, root) {
  const index = JSON.parse(await readFile(join(root, 'replay/index.json'), 'utf8'));
  const search = createSearch(store);
  const rows = [];
  for (const q of ['de', 'des', 'desi', 'desig', 'design']) {
    const entry = index[`GET https://access.mymind.com/search?q=${q}&wildcard=true`];
    if (!entry) throw new Error(`Missing recorded ${q} prefix search`);
    const live = JSON.parse(await readFile(join(root, 'replay/bodies', entry.file), 'utf8'));
    const liveIds = new Set(live.matches.map(m => m.id));
    const ours = search.text(q, { wildcard: true });
    const overlap = ours.matches.filter(m => liveIds.has(m.id)).length;
    const fields = {};
    for (const field of ['title', 'description', 'summary', 'tags', 'url', 'body', 'note', 'ocr']) {
      fields[field] = store.db.prepare(`SELECT d.id FROM search_fts JOIN search_documents d ON d.rowid = search_fts.rowid
        JOIN cards c ON c.id = d.id WHERE search_fts MATCH ? AND c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & 393224) = 0`)
        .all(`${field}: "${q}"*`, store.defaultUserId()).filter(row => liveIds.has(row.id)).length;
    }
    rows.push({ q, live: live.count, ours: ours.count, overlap, liveOnly: live.count - overlap,
      storeOnly: ours.count - overlap, fields, reason: 'Live-only text is absent from indexed fields; URL/body prefixes add store-only matches.' });
  }
  return rows;
}
