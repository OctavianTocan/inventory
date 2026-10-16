import { isoFromNs } from './store.mjs';

/**
 * Build smart-space refresh for use in writer.run(), or as a nonpersisting verification preview.
 * @param {object} store Open store; evaluation and writes stay on its connection.
 * @param {object} search Search service with synchronous filters().
 * @param {object} hooks Writer's now, save, saveSpace and card transaction helpers.
 * @returns {Function} checkQuery(queryId, persist=true), returning elapsed, added and removed.
 */
export function createQueryChecker(store, search, { now, save, saveSpace, card }) {
  const { db } = store;
  return (queryId, persist = true) => {
    const started = performance.now();
    const row = db.prepare("SELECT id, query FROM spaces WHERE user_id = ? AND json_extract(query, '$.id') = ?").get(store.defaultUserId(), queryId);
    if (!row) throw Object.assign(new Error('Query not found'), { status: 404 });
    const query = JSON.parse(row.query);
    const existing = db.prepare('SELECT card_id, query_id, excluded FROM space_cards WHERE space_id = ?').all(row.id);
    const excluded = new Set(existing.filter(m => m.excluded).map(m => m.card_id));
    const manual = new Set(existing.filter(m => !m.query_id).map(m => m.card_id));
    const previous = new Set(existing.filter(m => m.query_id && !m.excluded).map(m => m.card_id));
    const current = search.filters(query.filters).filter(m => !excluded.has(m.id) && !manual.has(m.id));
    const ids = new Set(current.map(m => m.id));
    const added = current.filter(m => !previous.has(m.id)).map(m => ({ ...m, queryId }));
    const removed = [...previous].filter(id => !ids.has(id));
    if (persist) {
      db.prepare('DELETE FROM space_cards WHERE space_id = ? AND query_id = ? AND excluded = 0').run(row.id, queryId);
      let rank = db.prepare('SELECT COALESCE(MAX(rank), -1) + 1 AS n FROM space_cards WHERE space_id = ?').get(row.id).n;
      for (const member of current) db.prepare('INSERT INTO space_cards VALUES (?, ?, ?, ?, ?, ?, 0)')
        .run(row.id, member.id, store.defaultUserId(), rank++, member.score, queryId);
      query.checked = isoFromNs(now());
      db.prepare('UPDATE spaces SET query = ? WHERE id = ?').run(JSON.stringify(query), row.id);
      db.prepare('DELETE FROM api_metadata WHERE path = ?').run(`/spaces/${row.id}`);
      if (added.length || removed.length) {
        saveSpace(row.id);
        for (const id of [...added.map(m => m.id), ...removed]) save(card(id));
      }
    }
    return { elapsed: Math.round(performance.now() - started), added, removed };
  };
}
