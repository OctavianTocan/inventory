import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');

const attempt = (work) => {
  try { return work(); } catch (error) { return { threw: error.constructor.name }; }
};

const SEARCH_TABLES = ['search_fts', 'search_documents'];

function open(file, { dropSearch = false } = {}) {
  const db = openSql(file);
  if (dropSearch) for (const table of SEARCH_TABLES) db.exec(`DROP TABLE IF EXISTS ${table}`);
  return createStore(db);
}

export function dump(file) {
  const store = open(file);
  const users = store.db.prepare('SELECT id FROM users ORDER BY rowid').all().map((row) => row.id);
  const out = { defaultUserId: store.defaultUserId(), missingUser: store.user('nope'), missingQuota: attempt(() => store.quota('nope')), users: [] };
  for (const userId of users) {
    const all = store.gridCards({ userId });
    const trash = store.trashCards(userId);
    const page = (cards) => cards.at(-1) ?? null;
    const first = store.gridCardsAfter({ userId, limit: 40 });
    const trashFirst = store.trashCardsAfter({ userId, limit: 7 });
    const changes = store.changesSince(0, { userId, limit: 100000 });
    const middle = changes[Math.floor(changes.length / 2)]?.ts ?? 0n;
    const spaces = store.spaces(userId);
    const allIds = store.db.prepare('SELECT id FROM cards WHERE user_id = ? ORDER BY id').all(userId).map((row) => row.id);
    out.users.push({
      id: userId,
      user: store.user(userId),
      quota: store.quota(userId),
      gridCount: store.gridCount(userId),
      checkpoint: store.checkpoint(userId),
      gridCheckpoint: store.gridCheckpoint(userId),
      grid: all,
      window: store.gridCards({ userId, offset: 10, limit: 20 }),
      gridFirst: first,
      gridNext: store.gridCardsAfter({ userId, after: page(first) && { ns: page(first).bumpedNs, id: page(first).id }, limit: 40 }),
      gridLast: store.gridCardsAfter({ userId, after: { ns: all.at(-1)?.bumpedNs ?? 0n, id: all.at(-1)?.id ?? '' }, limit: 5 }),
      trash,
      trashFirst,
      trashNext: store.trashCardsAfter({ userId, after: page(trashFirst) && { ns: page(trashFirst).deletedNs, id: page(trashFirst).id }, limit: 7 }),
      spaces,
      spaceCards: spaces.map((space) => store.spaceCards(space.id)),
      space: spaces.map((space) => store.space(space.id)),
      tagsManual: store.tagsManual(userId),
      tagsRecent: [store.tagsRecent(10, userId), store.tagsRecent(-2, userId), store.tagsRecent(0, userId), store.tagsRecent(3, userId), store.tagsRecent(undefined, userId)],
      changes: [changes.length, changes.slice(0, 30), store.changesSince(middle, { userId, limit: 25 }), store.changesSince(0, { userId, limit: 100000, coalesce: true }).slice(-30), store.changesSince(middle, { userId, coalesce: true })],
      cardsByIds: store.cardsByIds([...allIds.slice(0, 5), 'missing', ...allIds.slice(-3), allIds[0]], userId),
      cards: allIds.map((id) => store.card(id, userId)),
      tags: allIds.map((id) => store.cardTags(id)),
      missingCard: store.card('missing', userId),
    });
  }
  return out;
}

export function newIds(file) {
  const store = open(file);
  const users = store.db.prepare('SELECT id FROM users ORDER BY rowid').all().map((row) => row.id);
  const out = { unknownKind: attempt(() => store.newId('bogus')), unknownUser: attempt(() => store.newId('card', 'nope')), ids: [] };
  for (const userId of users) {
    for (const kind of ['card', 'tag', 'space', 'query', 'note']) {
      out.ids.push([userId, kind, [store.newId(kind, userId), store.newId(kind, userId), store.newId(kind, userId)]]);
    }
  }
  out.defaultCard = store.newId('card');
  return out;
}

export function searchState(file, dropSearch) {
  const store = open(file, { dropSearch });
  const { db } = store;
  const master = db.prepare("SELECT type, name, tbl_name, sql FROM sqlite_master WHERE name LIKE 'search%' ORDER BY name").all();
  const docs = db.prepare('SELECT id, fields FROM search_documents ORDER BY id').all();
  const dirty = db.prepare('SELECT COUNT(*) AS n FROM search_dirty').get().n;
  const fts = db.prepare("SELECT rowid FROM search_fts WHERE search_fts MATCH 'the' ORDER BY rowid").all();
  const total = db.prepare('SELECT COUNT(*) AS n FROM search_fts').get().n;
  return { master, docs, dirty, fts, total };
}

export function transactionCases(file) {
  const store = open(file);
  const { db } = store;
  const [{ id }] = db.prepare('SELECT id FROM cards ORDER BY id LIMIT 1').all();
  const snapshot = () => ({
    data: db.prepare('SELECT data FROM cards WHERE id = ?').get(id).data,
    doc: db.prepare('SELECT fields FROM search_documents WHERE id = ?').get(id)?.fields ?? null,
    dirty: db.prepare('SELECT COUNT(*) AS n FROM search_dirty').get().n,
  });
  const before = snapshot();
  const edit = (title) => db.prepare("UPDATE cards SET data = json_set(data, '$.title', ?) WHERE id = ?").run(title, id);
  const committed = store.transaction(() => { edit('Zebra committed'); return 'done'; });
  const afterCommit = snapshot();
  const thrown = attempt(() => store.transaction(() => { edit('Rolled back'); throw new RangeError('boom'); }));
  const afterRollback = snapshot();
  const nested = store.transaction(() => { edit('Outer'); return attempt(() => store.transaction(() => { edit('Inner'); throw new TypeError('inner'); })); });
  return { before, committed, afterCommit, thrown, afterRollback, nested, afterNested: snapshot() };
}
