import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');
const { createQueryChecker } = await lib('store-query.mjs');

const PREPS = [
  '',
  'UPDATE space_cards SET excluded = 1 WHERE rowid % 7 = 0',
  'UPDATE space_cards SET query_id = NULL WHERE rowid % 5 = 0',
];

export function check(file, queryId, persist, seed, prep) {
  const store = createStore(openSql(file));
  const { db } = store;
  if (PREPS[prep]) db.exec(PREPS[prep]);
  db.exec("INSERT OR IGNORE INTO api_metadata (path, status, headers, attributes) SELECT '/spaces/' || id, 200, '{}', '{}' FROM spaces");
  const ids = db.prepare('SELECT id FROM cards WHERE user_id = ? ORDER BY id').all(store.defaultUserId()).map((row) => row.id);
  const sum = (id) => [...id].reduce((n, char) => n + char.charCodeAt(0), 0);
  const seen = [];
  const search = {
    filters(filters) {
      seen.push(filters === undefined ? null : filters);
      return ids.filter((id) => (sum(id) + seed) % 4 < 2).map((id) => ({ id, score: ((sum(id) + seed) % 100) / 100 }));
    },
  };
  const log = [];
  const run = createQueryChecker(store, search, {
    now: () => 1787000000123456789n,
    save: (card) => log.push(['save', card.id]),
    saveSpace: (id) => log.push(['space', id]),
    card: (id) => { log.push(['card', id]); return { id }; },
  });
  let result;
  try {
    result = run(queryId, persist);
    if (!Number.isInteger(result.elapsed) || result.elapsed < 0) throw new Error('elapsed');
    result.elapsed = 0;
  } catch (error) {
    result = { threw: error.message, status: error.status };
  }
  return {
    result,
    log,
    seen,
    members: db.prepare('SELECT space_id, card_id, rank, score, query_id, excluded FROM space_cards ORDER BY space_id, card_id').all(),
    queries: db.prepare('SELECT id, query FROM spaces WHERE query IS NOT NULL ORDER BY id').all(),
    metadata: db.prepare('SELECT path FROM api_metadata ORDER BY path').all(),
  };
}
