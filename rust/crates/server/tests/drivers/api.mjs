// lib/api.mjs's createApi over a fixture copy, driven by the request scripts of the native API's differential test.
// Time and random bytes are fixed; media, capture, screenshots and the hosted AI are table-driven fakes that the
// Rust test mirrors.
import crypto from 'node:crypto';
import fs from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

let counter = 0;
crypto.randomBytes = (size) => {
  counter += 1;
  return Buffer.alloc(size, counter % 256);
};
syncBuiltinESMExports();

const ROOT = process.env.MYMIND_ROOT;
const lib = (name) => import(pathToFileURL(path.join(ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');
const { createApi } = await lib('api.mjs');
const { localize, send } = await lib('site.mjs');
const msgpack = await import(pathToFileURL(path.join(ROOT, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);

const NOW = 1787000000000;
const ENV_NAMES = ['GEMINI_API_KEY', 'TYPESAFE_API_KEY', 'AI_CHAT_API_KEY', 'AI_CHAT_MODEL', 'GEMINI_BASE_URL', 'AI_CHAT_BASE_URL', 'TYPESAFE_BASE_URL', 'OFFLINE'];
const TABLES = ['cards', 'changes', 'annotations', 'spaces', 'space_cards', 'pins', 'shares', 'revisions', 'users', 'tag_names', 'card_content', 'uploads', 'embeddings'];
const PLACEHOLDER = /\{\{(\w+)(?::(\d+))?\}\}/g;
const LISTS = {
  card: 'SELECT id FROM cards ORDER BY id',
  live: 'SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id',
  deleted: 'SELECT id FROM cards WHERE deleted_ns IS NOT NULL ORDER BY id',
  local: "SELECT id FROM cards WHERE json_extract(extras, '$.local') = 1 ORDER BY id",
  note: "SELECT id FROM cards WHERE type = 'Note' ORDER BY id",
  article: "SELECT id FROM cards WHERE type = 'Article' ORDER BY id",
  pinned: 'SELECT id FROM cards WHERE (flags & 512) != 0 ORDER BY id',
  space: 'SELECT id FROM spaces ORDER BY id',
  query: "SELECT json_extract(query, '$.id') AS id FROM spaces WHERE query IS NOT NULL ORDER BY id",
  annotation: 'SELECT id FROM annotations ORDER BY id',
  tag: "SELECT DISTINCT name AS id FROM annotations WHERE name NOT GLOB '*[#&+%?]*' ORDER BY name",
  share: 'SELECT id FROM shares ORDER BY created_ns DESC, id',
  revision: 'SELECT id FROM revisions ORDER BY id',
  user: 'SELECT id FROM users ORDER BY id',
  account: 'SELECT account_uid AS id FROM users ORDER BY id',
};

const emb = (seed) => Array.from({ length: 768 }, (_, i) => ((i * 7 + seed * 3) % 13) / 5 - 1.1);
const CHAT = JSON.stringify({ choices: [{ message: { content: JSON.stringify({ title: 'Generated title', summary: 'A generated summary.', tags: ['alpha', 'Beta tag', 'design'] }) } }] });
const ROUTES = [[':embedContent', 'embedContent'], [':batchEmbedContents', 'batchEmbedContents'], ['/chat/completions', 'chat'], ['/v1/systemone', 'systemone']];
const normalizeUuid = (text) => text.replace(/mymind-image-[0-9a-f-]{36}/g, 'mymind-image-UUID');
const byteSum = (text) => Buffer.from(text).reduce((sum, byte) => sum + byte, 0);

const blocked = () => Object.assign(new Error('Unsupported capture destination'), { code: 'CAPTURE_BLOCKED', status: 400 });
const fetched = (text, url, mime) => ({ bytes: Buffer.from(text), url, mime, status: 200, headers: {} });

// The page or file a captured URL answers with, by the words in it.
function resource(url) {
  if (url.includes('blocked')) throw blocked();
  if (url.includes('fail')) throw new Error('Page unavailable');
  if (url.includes('img.example')) return fetched(url.includes('bad') ? 'XIMG' : 'PNGDATA', url, 'image/png');
  if (url.endsWith('.pdf')) return fetched('%PDF-1.4', url, 'application/pdf');
  if (url.includes('page')) {
    return fetched('<title>Page T | Example</title><meta property="og:image" content="https://img.example/1.png"><meta name="description" content="Described"><article><p>First paragraph.</p><p>Second.</p></article>', url, 'text/html; charset=utf-8');
  }
  return fetched('<title>Plain</title><p>Body</p>', url, 'text/html');
}

const mediaSize = (input) => (Buffer.isBuffer(input) || input instanceof Uint8Array ? input.length
  : input?.file ? fs.statSync(input.file).size : input?.upload ? -1 : -2);

export async function run(file, uploadDir, phase) {
  for (const name of ENV_NAMES) delete process.env[name];
  Object.assign(process.env, phase.env);
  Date.now = () => NOW;
  counter = 0;
  const log = [];
  const fetches = [];
  console.log = (...parts) => log.push(parts.join(' '));
  globalThis.fetch = async (url, options = {}) => {
    const route = ROUTES.find(([suffix]) => String(url).endsWith(suffix))?.[1] ?? 'other';
    const body = normalizeUuid(String(options.body ?? ''));
    fetches.push(JSON.stringify({ route, url: String(url), method: options.method ?? 'GET', headers: Object.entries(options.headers ?? {}), body }));
    let text = '{}', status = 200;
    if (route === 'embedContent') text = JSON.stringify({ embedding: { values: emb(byteSum(body) % 11) } });
    else if (route === 'batchEmbedContents') {
      const count = JSON.parse(body).requests.length;
      text = JSON.stringify({ embeddings: Array.from({ length: count }, (_, i) => ({ values: emb(i) })), usageMetadata: { totalTokenCount: 3 } });
    } else if (route === 'chat') text = CHAT;
    else status = 404;
    return { ok: status === 200, status, statusText: status === 200 ? 'OK' : 'Not Found', headers: new Headers({ 'content-type': 'application/json' }), text: async () => text, json: async () => JSON.parse(text) };
  };
  const media = {
    async prepare(input, type, rawName, who, cardId) {
      const name = String(rawName ?? '');
      const size = mediaSize(input);
      log.push(`[media] prepare ${size} ${type} ${name} ${who.accountUid} ${cardId}`);
      if (Buffer.isBuffer(input) && input[0] === 0x58) throw Object.assign(new Error('Media preview could not be generated'), { status: 422 });
      const key = `${who.accountUid}/${cardId}/${size}`;
      if (type.startsWith('image/')) {
        return { data: { type: 'Image', title: name, blob: { path: `${key}.png`, width: 40, height: 30 } }, extras: { cover: { src: `/media/${key}.png;800x600-t.webp`, width: 800, height: 600 } } };
      }
      if (type === 'application/pdf') return { data: { type: 'Document', title: name, blob: { path: `${key}.pdf` }, format: 'pdf' }, extras: {} };
      return { data: { type: 'File', title: name, blob: { path: `${key}.bin` } }, extras: {} };
    },
    async response(relative) {
      return relative.includes('gone') ? null : { file: relative, type: 'image/png', size: 7 };
    },
    async send(res, result, { method, range }) {
      res.writeHead(200, { 'content-type': result.type, 'x-media': result.file, 'x-range': range ?? '' });
      res.end(method === 'HEAD' ? '' : `MEDIA:${result.file}`);
    },
    async image(source) {
      if (!source || typeof source !== 'object') return null;
      const where = source.path ?? source.src;
      if (!where) return null;
      return { mime: 'image/png', data: Buffer.from(`IMG:${where}`) };
    },
  };
  const screenshot = async (url, options) => {
    log.push(`[screenshot] ${url} ${options.offline}`);
    if (url.includes('noshot')) throw new Error('browser failed');
    return Buffer.from('SHOT');
  };
  const capture = async (url, options) => {
    // The real transport rejects a destination that is not a string before any request.
    if (typeof url !== 'string') throw blocked();
    log.push(`[resource] ${url} ${options.offline} ${options.limit}`);
    return resource(url);
  };
  const events = [];
  const store = createStore(openSql(file));
  const api = await createApi({ root: '', store, media, uploadStorage: uploadDir, screenshot, resource: capture, readonly: phase.readonly,
    msgpack, localize, send, push: (target, payload) => events.push([target, payload]), hasReplay: (req) => req.headers['x-replay'] === '1' });
  const settle = async () => { for (let turn = 0; turn < 300; turn += 1) await new Promise((resolve) => setImmediate(resolve)); };
  await settle();
  const uploads = [], tokens = [];
  let last = 'missing';
  const shell = fs.readFileSync(path.join(ROOT, 'capture/shell.html'), 'utf8');
  const resolve = (text) => text.replace(PLACEHOLDER, (whole, kind, index = '0') => {
    if (kind === 'new') return last;
    if (kind === 'upload') return uploads[Number(index)] ?? 'missing';
    if (kind === 'token') return tokens[Number(index)] ?? 'missing';
    const sql = LISTS[kind];
    if (!sql) return whole;
    return store.db.prepare(sql).all()[Number(index)]?.id ?? 'missing';
  });
  const answers = [];
  for (const step of phase.steps) {
    if (step.op === 'shell') {
      answers.push({ shell: api.shell(shell) });
      continue;
    }
    const target = resolve(step.target);
    const headers = Object.fromEntries(Object.entries(step.headers ?? {}).map(([name, value]) => [name, resolve(value)]));
    const body = step.base64 ? Buffer.from(step.base64, 'base64') : Buffer.from(resolve(step.body ?? ''));
    const req = { method: step.method, url: target, headers, async *[Symbol.asyncIterator]() { if (body.length) yield body; } };
    let sent = null;
    const res = {
      writeHead(status, written) { sent = { status, headers: Object.entries(written).map(([name, value]) => [name, String(value)]), body: [] }; },
      end(chunk) { sent.body.push(Buffer.from(chunk ?? '')); },
    };
    let answer;
    try {
      const handled = await api.handle(req, res, new URL(target, 'http://localhost'));
      answer = handled ? { status: sent.status, headers: sent.headers, body: Buffer.concat(sent.body).toString('base64') } : { handled: false };
    } catch (error) {
      answer = { threw: true };
    }
    await settle();
    if (answer.status >= 200 && answer.status < 300 && answer.headers.some(([name, value]) => name === 'content-type' && value.includes('json'))) {
      try {
        const parsed = JSON.parse(Buffer.from(answer.body, 'base64').toString('utf8'));
        for (const item of [parsed].flat()) {
          if (typeof item?.id === 'string') last = item.id;
          if (typeof item?.url === 'string' && item.url.startsWith('/uploads/') && item.token) { uploads.push(item.url.split('/').pop()); tokens.push(item.token); }
        }
      } catch {}
    }
    answers.push(answer);
  }
  api.close();
  const state = {};
  for (const table of TABLES) {
    const columns = store.db.prepare(`PRAGMA table_info(${table})`).all().map((column) => `CASE WHEN typeof("${column.name}") = 'integer' AND abs("${column.name}") > 9007199254740991 THEN CAST("${column.name}" AS TEXT) WHEN typeof("${column.name}") = 'blob' THEN hex("${column.name}") ELSE "${column.name}" END AS "${column.name}"`);
    state[table] = store.db.prepare(`SELECT ${columns.join(', ')} FROM ${table} ORDER BY 1, 2`).all().map((row) => JSON.stringify(row));
  }
  const files = fs.existsSync(uploadDir) ? fs.readdirSync(uploadDir).sort().map((name) => [name, fs.readFileSync(path.join(uploadDir, name)).toString('base64')]) : [];
  store.db.close?.();
  return { answers, events: events.map((event) => JSON.stringify(event, (key, value) => (typeof value === 'bigint' ? `${value}n` : value))), log, fetches: fetches.sort(), state, files };
}
