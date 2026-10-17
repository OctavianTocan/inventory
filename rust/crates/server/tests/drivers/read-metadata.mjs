// lib/read-metadata.mjs's importReadMetadata over a fixture copy and a recordings root, for the native port's test.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');
const { importReadMetadata } = await lib('read-metadata.mjs');

export async function run(file, root) {
  const db = openSql(file);
  const store = createStore(db);
  let threw = false;
  try { await importReadMetadata(store, root); } catch { threw = true; }
  const dump = (sql) => db.prepare(sql).all().map((row) => JSON.stringify(row));
  const out = { threw, metadata: dump('SELECT * FROM api_metadata ORDER BY 1'), content: dump('SELECT * FROM card_content ORDER BY 1') };
  db.close?.();
  return out;
}
