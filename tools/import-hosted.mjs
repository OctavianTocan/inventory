// Build what the hosted server reads (HOSTED=1 locally, the Worker on Cloudflare):
//   node tools/import-hosted.mjs bucket                      mymind's website code, static/ in the object store
//   node tools/import-hosted.mjs library <name> --db <file> [--recordings [--runs]] [--media <dir>]
//                                                            data/hosted/<name>.db with the library, and
//                                                            libraries/<name>/ in the store for its own media
// --recordings adds this repository's recordings of the owner's library: recorded answers, the page shell, read
// metadata, and captured card media under libraries/<name>/capture/. Only the owner's and the test library take them.
// --runs also keeps verification-run recordings (the test library). Output goes to data/hosted (or --out).
import { copyFile, link, mkdir, readdir, readFile, rm, stat } from 'node:fs/promises';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { directoryBucket } from '../lib/bucket-fs.mjs';
import { openSql } from '../lib/sql-node.mjs';
import { createStore } from '../lib/store.mjs';
import { importReadMetadata } from '../lib/read-metadata.mjs';
import { createRecorded } from '../lib/recorded.mjs';
import { APP_ORIGIN, LOCAL_PREFIXES, TEXT_EXTENSIONS, localize, localizeValue } from '../lib/site.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
// A Durable Object row holds at most 2 MB; the larger recordings are card lists the API always answers.
const MAX_BODY = 1_900_000;
const { values, positionals } = parseArgs({ allowPositionals: true, options: {
  db: { type: 'string' }, media: { type: 'string' }, recordings: { type: 'boolean', default: false }, runs: { type: 'boolean', default: false },
  out: { type: 'string', default: path.join(ROOT, 'data/hosted') },
} });
const [command, name] = positionals;
const out = path.resolve(values.out);
const bucket = path.join(out, 'bucket');
const objects = directoryBucket(bucket);
const index = JSON.parse(await readFile(path.join(ROOT, 'replay/index.json'), 'utf8'));
const msgpack = await import(pathToFileURL(path.join(ROOT, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);

// A recorded body as server.mjs serves it: MessagePack items and text with mymind's origins localized.
async function replayBody(entry) {
  const body = await readFile(path.join(ROOT, 'replay/bodies', entry.file));
  const type = entry.headers['content-type'] ?? '';
  if (type.includes('msgpack')) return Buffer.concat([...msgpack.decodeMulti(body)].map((item) => msgpack.encode(localizeValue(item))));
  if (/text|javascript|json|svg|xml/.test(type)) return Buffer.from(localize(body.toString('utf8')));
  return body;
}

// Every file below `folder`, as paths relative to it.
async function files(folder) {
  const entries = await readdir(folder, { recursive: true, withFileTypes: true }).catch(() => []);
  return entries.filter((entry) => entry.isFile()).map((entry) => path.relative(folder, path.join(entry.parentPath, entry.name)));
}

// Place `source` at `key` in the store, as a hard link when nothing needs rewriting.
async function place(key, source, { replace = true } = {}) {
  const target = path.join(bucket, key);
  if (!replace && await stat(target).then(() => true, () => false)) return false;
  await mkdir(path.dirname(target), { recursive: true });
  await rm(target, { force: true });
  await link(source, target).catch(() => copyFile(source, target));
  return true;
}

// Write `body` at `key` in the store, with the response headers mymind sent for it.
async function put(key, body, { replace = true, headers } = {}) {
  if (!replace && await objects.head(key)) return false;
  await objects.write(key, body, body.length, headers);
  return true;
}

// Split a replay key into its prefixes, method and live URL; null for keys of other shapes.
function parseKey(key) {
  const match = /^(run:[^|]+\|)?((?:iphone|ipad|android-phone|android-tablet)\|)?(\w+) (https:\/\/[^/]+\/.*)$/.exec(key);
  return match && { run: Boolean(match[1]), variant: Boolean(match[2]), kind: match[3], url: new URL(match[4]) };
}

// Store the recorded bodies of the URLs below each prefix of `folders` (URL prefix to key folder). Recordings answer
// before mirrored and captured files, so the plain ones replace them; device and run recordings of the same path carry
// the same bytes and only fill gaps.
async function importReplayed(folders) {
  let count = 0;
  const keys = Object.keys(index).sort((a, b) => Number(Boolean(parseKey(a)?.run || parseKey(a)?.variant)) - Number(Boolean(parseKey(b)?.run || parseKey(b)?.variant)));
  for (const key of keys) {
    const parsed = parseKey(key);
    if (!parsed || !['GET', 'DOC'].includes(parsed.kind) || index[key].status !== 200) continue;
    const [from, folder] = [...folders].find(([prefix]) => parsed.url.href.startsWith(prefix)) ?? [];
    if (!folder) continue;
    const relative = decodeURIComponent(parsed.url.pathname.slice(new URL(from).pathname.length));
    if (relative.split('/').some((part) => part === '..' || part === '.' || part === '')) continue;
    const options = { replace: !(parsed.run || parsed.variant), headers: index[key].headers };
    if (await put(`${folder}${relative}`, await replayBody(index[key]), options)) count++;
  }
  return count;
}

async function importBucket() {
  const counts = { static: 0, replayed: 0 };
  for (const folder of ['static', '.headers/static']) await rm(path.join(bucket, folder), { recursive: true, force: true });
  for (const file of await files(path.join(ROOT, 'mirror/static'))) {
    const source = path.join(ROOT, 'mirror/static', file);
    if (TEXT_EXTENSIONS.has(path.extname(file).toLowerCase())) await put(`static/${file}`, Buffer.from(localize(await readFile(source, 'utf8'))));
    else await place(`static/${file}`, source);
    counts.static++;
  }
  counts.replayed = await importReplayed(new Map([[LOCAL_PREFIXES[0][1], 'static/']]));
  console.log(counts);
}

// The owner's captured card media, below `capture` in the store.
async function importCaptured(capture) {
  const counts = { media: 0, mediahost: 0, carbon: 0, replayed: 0 };
  for (const folder of [capture, `.headers/${capture}`]) await rm(path.join(bucket, folder), { recursive: true, force: true });
  for (const folder of ['media', 'mediahost', 'carbon']) {
    for (const file of await files(path.join(ROOT, 'capture', folder))) {
      await place(`${capture}${folder}/${file}`, path.join(ROOT, 'capture', folder, file));
      counts[folder]++;
    }
  }
  counts.replayed = await importReplayed(new Map([[`${APP_ORIGIN}media/`, `${capture}media/`],
    [LOCAL_PREFIXES[1][1], `${capture}mediahost/`], [LOCAL_PREFIXES[2][1], `${capture}carbon/`]]));
  return counts;
}

// The owner's recorded answers, detail views and page shell, as rows of the `recorded` table.
async function recordedRows() {
  const counts = { replay: 0, skipped: 0, details: 0, responses: 0 };
  const rows = [];
  for (const [key, entry] of Object.entries(index)) {
    const parsed = parseKey(key);
    if (!parsed || `${parsed.url.origin}/` !== APP_ORIGIN || parsed.url.pathname.startsWith('/media/')) continue;
    if (parsed.run && !values.runs) continue;
    const body = await replayBody(entry);
    if (body.length > MAX_BODY) { counts.skipped++; continue; }
    rows.push([key, entry.status, entry.headers, body]);
    counts.replay++;
  }
  const details = JSON.parse(await readFile(path.join(ROOT, 'capture/details.json'), 'utf8'));
  for (const [id, entry] of Object.entries(details)) {
    const file = path.join(ROOT, 'capture/details', `${id.replace(/[^a-z0-9.-]+/gi, '_')}.html`);
    const html = await readFile(file, 'utf8').catch(() => null);
    if (html === null) continue;
    const headers = { 'content-type': 'text/html; charset=utf-8', ...(entry.headers?.['x-magic'] ? { 'x-magic': entry.headers['x-magic'] } : {}) };
    rows.push([`GET ${APP_ORIGIN}cards/${id}/detail`, 200, headers, Buffer.from(localize(html))]);
    counts.details++;
  }
  const responses = JSON.parse(await readFile(path.join(ROOT, 'capture/responses.json'), 'utf8'));
  for (const [key, entry] of Object.entries(responses)) {
    const [method, local] = [key.slice(0, key.indexOf(' ')), key.slice(key.indexOf(' ') + 1)];
    rows.push([`${method} ${APP_ORIGIN}${local.slice(1)}`, entry.status, entry.headers,
      Buffer.from(localize(await readFile(path.join(ROOT, 'capture', entry.file), 'utf8')))]);
    counts.responses++;
  }
  rows.push(['shell', 200, { 'content-type': 'text/html; charset=utf-8' }, Buffer.from(localize(await readFile(path.join(ROOT, 'capture/shell.html'), 'utf8')))]);
  return { rows, counts };
}

async function importLibrary(library) {
  if (!/^[a-z0-9-]+$/.test(library ?? '')) throw new Error('Library names are lower-case letters, digits and dashes');
  if (!values.db) throw new Error('--db is required');
  if (values.runs && !values.recordings) throw new Error('--runs needs --recordings');
  await mkdir(out, { recursive: true });
  const target = path.join(out, `${library}.db`);
  for (const suffix of ['', '-wal', '-shm']) await rm(target + suffix, { force: true });
  const source = openSql(path.resolve(values.db), { readOnly: true });
  source.exec(`VACUUM INTO '${target.replaceAll("'", "''")}'`);
  source.close();
  const db = openSql(target);
  const store = createStore(db);
  const recorded = createRecorded(db);
  db.transaction(() => db.exec('DELETE FROM recorded'));
  const prefix = `libraries/${library}/`;
  let counts = {};
  if (values.recordings) {
    await importReadMetadata(store, ROOT);
    const { rows, counts: recordedCounts } = await recordedRows();
    db.transaction(() => { for (const row of rows) recorded.put(...row); });
    counts = { ...recordedCounts, captured: await importCaptured(`${prefix}capture/`) };
  }
  let media = 0;
  if (values.media) {
    const folder = path.resolve(values.media);
    for (const file of await files(folder)) {
      if (/\.page-\d+\.png$/.test(file) || file.endsWith('.tmp')) continue;
      const key = file.startsWith(`.renditions${path.sep}`) ? `${prefix}renditions/${file.slice('.renditions/'.length)}` : `${prefix}media/${file.split(path.sep).join('/')}`;
      await place(key, path.join(folder, file));
      media++;
    }
  }
  console.log({ library, database: target, cards: db.prepare('SELECT COUNT(*) AS n FROM cards').get().n, recorded: recorded.count(), ...counts, media });
  db.close();
}

if (command === 'bucket') await importBucket();
else if (command === 'library') await importLibrary(name);
else {
  console.error('Usage: node tools/import-hosted.mjs bucket | library <name> --db <file> [--recordings [--runs]] [--media <dir>] [--out <dir>]');
  process.exit(2);
}
