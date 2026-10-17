import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { insertRows, tableDigest } = await lib('library-rows.mjs');

const attempt = (work) => {
  try { return work(); } catch (error) { return { threw: error.message, status: error.status ?? null }; }
};

export function digest(file, table, columns) {
  return tableDigest(openSql(file, { readOnly: true }), table, columns);
}

export function roundTrip(createSql, table, columns, rows) {
  const db = openSql(':memory:');
  db.exec(createSql);
  const inserted = attempt(() => insertRows(db, table, columns, rows));
  const names = columns.filter((name) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(name));
  return { inserted, count: db.prepare(`SELECT COUNT(*) AS n FROM "${table.replace(/"/g, '')}"`).get().n, digest: attempt(() => tableDigest(db, table, names)) };
}
