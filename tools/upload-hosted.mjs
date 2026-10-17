// Fill the Cloudflare Worker (worker/) with what tools/import-hosted.mjs built in data/hosted, through its /_admin/
// routes, which exist only while the Worker has the ADMIN_TOKEN secret:
//   node tools/upload-hosted.mjs <worker url> bucket [--prefix <key prefix>] [--force]   R2 objects; same-size ones are skipped
//   node tools/upload-hosted.mjs <worker url> library <name> [--db <file>]                a library's rows, then a digest check
//   node tools/upload-hosted.mjs <worker url> user [<username>]                          the TEST_USER password user
//   node tools/upload-hosted.mjs <worker url> account <account> <library>                which library an account reaches
//   node tools/upload-hosted.mjs <worker url> summary                                     what the Worker holds
// Reads ADMIN_TOKEN (and TEST_USER_NAME, TEST_USER_PASSWORD for `user`) from the environment or .env; prints none of them.
import { openAsBlob } from 'node:fs';
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { installAuthSchema, setPassword } from '../lib/auth-store.mjs';
import { encodeValue, tableDigest } from '../lib/library-rows.mjs';
import { openSql } from '../lib/sql-node.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
try { process.loadEnvFile(path.join(ROOT, '.env')); } catch (error) { if (error.code !== 'ENOENT') throw new Error('Could not load .env'); }
const { values, positionals: [base, command, ...rest] } = parseArgs({ allowPositionals: true, options: {
  prefix: { type: 'string', default: '' }, force: { type: 'boolean', default: false }, db: { type: 'string' },
  concurrency: { type: 'string', default: '8' }, bucket: { type: 'string', default: path.join(ROOT, 'data/hosted/bucket') },
} });
if (!base || !command) {
  console.error('Usage: node tools/upload-hosted.mjs <worker url> bucket|library <name>|user [<name>]|account <account> <library>|summary');
  process.exit(2);
}
if (!process.env.ADMIN_TOKEN) throw new Error('ADMIN_TOKEN is not set');
const BATCH_BYTES = 4 * 1024 * 1024;

// Call one admin route; throws with the Worker's answer when it fails.
async function call(route, { method = 'POST', body, headers = {} } = {}) {
  const init = { method, headers: { authorization: `Bearer ${process.env.ADMIN_TOKEN}`, ...headers } };
  if (body !== undefined) {
    init.body = body instanceof Blob ? body : JSON.stringify(body);
    if (!(body instanceof Blob)) init.headers['content-type'] = 'application/json';
  }
  const response = await fetch(`${base.replace(/\/$/, '')}/_admin/${route}`, init);
  if (!response.ok && !(method === 'HEAD' && response.status === 404)) throw new Error(`${method} ${route}: ${response.status} ${(await response.text()).slice(0, 300)}`);
  return response;
}
const json = async (route, options) => (await call(route, options)).json().catch(() => null);

// Run `task` over `items`, `limit` at a time.
async function pool(items, limit, task) {
  let next = 0;
  await Promise.all(Array.from({ length: limit }, async () => { while (next < items.length) await task(items[next++]); }));
}

async function uploadBucket() {
  const root = path.resolve(values.bucket);
  const entries = await readdir(root, { recursive: true, withFileTypes: true });
  const keys = entries.filter((entry) => entry.isFile() && !entry.name.endsWith('.tmp'))
    .map((entry) => path.relative(root, path.join(entry.parentPath, entry.name)).split(path.sep).join('/'))
    .filter((key) => !key.startsWith('.headers/') && key.startsWith(values.prefix)).sort();
  const counts = { objects: keys.length, uploaded: 0, skipped: 0, bytes: 0 };
  await pool(keys, Number(values.concurrency), async (key) => {
    const file = path.join(root, key);
    const size = (await stat(file)).size;
    const route = `bucket/${key.split('/').map(encodeURIComponent).join('/')}`;
    if (!values.force) {
      const head = await call(route, { method: 'HEAD' });
      if (head.status === 200 && Number(head.headers.get('x-object-size')) === size) { counts.skipped++; return; }
    }
    const headers = await readFile(path.join(root, '.headers', `${key}.json`), 'utf8').catch(() => null);
    await call(route, { method: 'PUT', body: await openAsBlob(file), headers: headers ? { 'x-object-headers': JSON.stringify(JSON.parse(headers)) } : {} });
    counts.uploaded++;
    counts.bytes += size;
    if (counts.uploaded % 250 === 0) console.log(`${counts.uploaded} uploaded, ${counts.skipped} skipped of ${keys.length}`);
  });
  console.log(counts, await json(`bucket?prefix=${encodeURIComponent(values.prefix)}`, { method: 'GET' }));
}

// The tables to copy, parents before children, with the columns that hold data and whether row ids are kept apart.
function tablesOf(db) {
  const names = db.prepare("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' AND name != 'search_dirty'").all();
  const tables = names.map(({ name, sql }) => {
    const columns = db.prepare('SELECT name, type, pk, hidden FROM pragma_table_xinfo(?)').all(name);
    const keys = columns.filter((column) => column.pk);
    const alias = keys.length === 1 && keys[0].type.toUpperCase() === 'INTEGER';
    const rowid = !/WITHOUT\s+ROWID/i.test(sql) && !alias;
    const parents = [...new Set(db.prepare('SELECT "table" AS parent FROM pragma_foreign_key_list(?)').all(name).map((row) => row.parent))].filter((parent) => parent !== name);
    return { table: name, columns: [...(rowid ? ['rowid'] : []), ...columns.filter((column) => column.hidden === 0).map((column) => column.name)], rowid, parents };
  });
  const ordered = [];
  const visit = (table, path = []) => {
    if (ordered.includes(table)) return;
    if (path.includes(table)) throw new Error(`Foreign key cycle through ${table.table}`);
    for (const parent of table.parents) visit(tables.find((candidate) => candidate.table === parent), [...path, table]);
    ordered.push(table);
  };
  for (const table of tables) visit(table);
  return ordered;
}

async function uploadLibrary(name) {
  if (!/^[a-z0-9-]+$/.test(name ?? '')) throw new Error('Library names are lower-case letters, digits and dashes');
  const file = path.resolve(values.db ?? path.join(ROOT, 'data/hosted', `${name}.db`));
  const db = new DatabaseSync(file, { readOnly: true });
  const tables = tablesOf(db);
  const reset = await json(`library/${name}/reset`, { body: {} });
  const missing = tables.filter(({ table }) => !(table in reset.tables)).map(({ table }) => table);
  if (missing.length) throw new Error(`The Worker's schema has no table ${missing.join(', ')}`);
  for (const { table, columns, rowid } of tables) {
    const statement = db.prepare(`SELECT ${columns.map((column) => `"${column}"`).join(', ')} FROM "${table}"${rowid ? ' ORDER BY rowid' : ''}`);
    statement.setReadBigInts(true);
    let batch = [], bytes = 0, sent = 0;
    const flush = async () => {
      if (!batch.length) return;
      sent += (await json(`library/${name}/rows`, { body: { table, columns, rows: batch } })).inserted;
      batch = [];
      bytes = 0;
    };
    for (const row of statement.iterate()) {
      const encoded = columns.map((column) => encodeValue(row[column]));
      const size = JSON.stringify(encoded).length;
      if (bytes + size > BATCH_BYTES) await flush();
      batch.push(encoded);
      bytes += size;
    }
    await flush();
    console.log(`${table}: ${sent} rows`);
  }
  const dirty = db.prepare('SELECT id FROM search_dirty').all().map((row) => [row.id]);
  const sequences = db.prepare('SELECT name, CAST(seq AS TEXT) AS seq FROM sqlite_sequence').all();
  await json(`library/${name}/finish`, { body: { dirty, sequences } });
  db.close();
  const checked = [...tables, { table: 'search_dirty', columns: ['id'] }];
  const local = openSql(file, { readOnly: true });
  const remote = await json(`library/${name}/digest`, { body: { tables: checked.map(({ table, columns }) => ({ table, columns })) } });
  let same = true;
  for (const { table, columns } of checked) {
    const mine = tableDigest(local, table, columns);
    const match = mine.rows === remote[table].rows && mine.sha256 === remote[table].sha256;
    same &&= match;
    console.log(`${match ? 'same' : 'DIFFERENT'} ${table}: ${mine.rows} rows here, ${remote[table].rows} on the Worker`);
  }
  local.close();
  if (!same) process.exitCode = 1;
  console.log(same ? `library ${name}: every table matches` : `library ${name}: tables differ`);
}

async function uploadUser(username = process.env.TEST_USER_NAME) {
  if (!username || !process.env.TEST_USER_PASSWORD || username !== process.env.TEST_USER_NAME) throw new Error('Only the TEST_USER password user from .env can be uploaded');
  // The password is hashed here; only the salt and hash leave the machine.
  const db = installAuthSchema(openSql(':memory:'));
  const name = await setPassword(db, username, process.env.TEST_USER_PASSWORD);
  const row = db.prepare('SELECT username, salt, hash, created_ms FROM users WHERE username = ?').get(name);
  db.close();
  await call('auth/user', { body: { username: row.username, salt: Buffer.from(row.salt).toString('base64'), hash: Buffer.from(row.hash).toString('base64'), created_ms: row.created_ms } });
  console.log(`password user ${row.username} stored`);
}

if (command === 'bucket') await uploadBucket();
else if (command === 'library') await uploadLibrary(rest[0]);
else if (command === 'user') await uploadUser(rest[0]);
else if (command === 'account') {
  await call('auth/library', { body: { account: rest[0], library: rest[1] } });
  console.log(`${rest[0]} reaches library ${rest[1]}`);
} else if (command === 'summary') {
  const summary = await json('auth', { method: 'GET' });
  console.log({ users: summary.users, libraries: summary.libraries.map(({ library }) => library), sessions: summary.sessions, appTokens: summary.appTokens },
    await json('bucket', { method: 'GET' }));
} else {
  console.error(`Unknown command ${command}`);
  process.exit(2);
}
