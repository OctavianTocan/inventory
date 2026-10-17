import crypto from 'node:crypto';
import { syncBuiltinESMExports } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');

let counter = 0;
crypto.randomBytes = (size) => {
  counter += 1;
  return Buffer.alloc(size, counter);
};
syncBuiltinESMExports();
const { createWriter } = await lib('store-write.mjs');

const TABLES = ['embeddings', 'cards', 'changes', 'annotations', 'spaces', 'space_cards', 'pins', 'shares', 'revisions', 'users', 'tag_names', 'card_content', 'api_metadata'];
const SKIP = new Set(['html']);

function resolve(db, arg) {
  if (typeof arg !== 'string' || !arg.startsWith('@')) return arg;
  const [kind, index = '0'] = arg.slice(1).split(':');
  if (kind === 'baseline') {
    const row = db.prepare('SELECT data, extras FROM cards ORDER BY id').all()[Number(index)];
    return { data: JSON.parse(row.data), extras: JSON.parse(row.extras) };
  }
  const sql = {
    card: 'SELECT id FROM cards ORDER BY id',
    annotation: 'SELECT id FROM annotations ORDER BY id',
    space: 'SELECT id FROM spaces ORDER BY id',
    revision: 'SELECT id FROM revisions ORDER BY id',
    share: 'SELECT id FROM shares ORDER BY id',
    query: "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id",
  }[kind];
  return db.prepare(sql).all()[Number(index)]?.id ?? 'missing';
}

export function scenario(file, steps) {
  const store = createStore(openSql(file));
  const { db } = store;
  const realNow = Date.now;
  Date.now = () => 1787000000000;
  counter = 0;
  const events = [];
  const writer = createWriter(store, (target, payload) => events.push([target, payload]));
  const results = [];
  try {
    for (const [name, ...raw] of steps) {
      const args = raw.map((arg) => resolve(db, arg));
      try {
        const result = writer.run(() => writer[name](...args));
        if (result && typeof result === 'object' && 'elapsed' in result) {
          if (!Number.isInteger(result.elapsed) || result.elapsed < 0) throw new Error('elapsed');
          result.elapsed = 0;
        }
        results.push({ ok: result === undefined ? null : result });
      } catch (error) {
        results.push(error.status ? { status: error.status, message: error.message } : { error: error.constructor.name });
      }
    }
  } finally {
    Date.now = realNow;
  }
  const state = {};
  for (const table of TABLES) {
    const columns = db.prepare(`PRAGMA table_info(${table})`).all().map((column) => `CASE WHEN typeof("${column.name}") = 'integer' AND abs("${column.name}") > 9007199254740991 THEN CAST("${column.name}" AS TEXT) WHEN typeof("${column.name}") = 'blob' THEN hex("${column.name}") ELSE "${column.name}" END AS "${column.name}"`);
    state[table] = db.prepare(`SELECT ${columns.join(', ')} FROM ${table} ORDER BY 1, 2`).all().map((row) => {
      for (const key of SKIP) delete row[key];
      return row;
    });
  }
  return { results, events, state };
}
