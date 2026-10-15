// Replays a captured mymind session on localhost: the real shell and front-end modules,
// with every API call answered from capture/ instead of mymind's servers.
import http from 'node:http';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.dirname(fileURLToPath(import.meta.url));
const CAPTURE = path.join(ROOT, 'capture');
const MIRROR = path.join(ROOT, 'mirror', 'static');
const PORT = Number(process.env.PORT ?? 4321);
const OFFLINE = process.env.OFFLINE === '1';

const STATIC_ORIGIN = 'https://static.accelerator.net/';
const MEDIA_HOST = 'https://media.mymind.host/';
const APP_ORIGIN = 'https://access.mymind.com/';

const TYPES = {
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.webp': 'image/webp',
  '.avif': 'image/avif',
  '.woff2': 'font/woff2',
  '.woff': 'font/woff',
  '.ttf': 'font/ttf',
  '.mp4': 'video/mp4',
  '.wasm': 'application/wasm',
};
const TEXT_EXTENSIONS = new Set(['.js', '.mjs', '.css', '.html', '.json', '.svg']);
const IMAGE_PLACEHOLDER = `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><rect width="100%" height="100%" fill="#e9e7e3"/></svg>`;

const localize = (text) =>
  text.replaceAll(STATIC_ORIGIN, '/_static/').replaceAll(MEDIA_HOST, '/_mediahost/').replaceAll(APP_ORIGIN, '/');

const meta = JSON.parse(await readFile(path.join(CAPTURE, 'meta.json'), 'utf8'));
const shell = localize(await readFile(path.join(CAPTURE, 'shell.html'), 'utf8'));
const cards = JSON.parse(await readFile(path.join(CAPTURE, 'cards.json'), 'utf8')).map((card) => ({
  html: localize(card.html),
  json: localize(card.json),
}));
const cardsById = new Map(cards.map((card) => [card.html.match(/data-id="([^"]+)"/)?.[1], card]));
const responses = JSON.parse(await readFile(path.join(CAPTURE, 'responses.json'), 'utf8'));
const details = JSON.parse(await readFile(path.join(CAPTURE, 'details.json'), 'utf8'));

function encodeMsgpack(value, parts = []) {
  const header = (fix, b16, b32, n, fixMax) => {
    if (n <= fixMax) parts.push(Buffer.from([fix | n]));
    else if (n < 0x10000) parts.push(Buffer.from([b16, n >> 8, n & 0xff]));
    else {
      const h = Buffer.alloc(5);
      h[0] = b32;
      h.writeUInt32BE(n, 1);
      parts.push(h);
    }
  };
  if (value === null || value === undefined) parts.push(Buffer.from([0xc0]));
  else if (typeof value === 'boolean') parts.push(Buffer.from([value ? 0xc3 : 0xc2]));
  else if (typeof value === 'number') {
    if (Number.isInteger(value) && value >= 0 && value < 0x80) parts.push(Buffer.from([value]));
    else {
      const b = Buffer.alloc(9);
      b[0] = 0xcb;
      b.writeDoubleBE(value, 1);
      parts.push(b);
    }
  } else if (typeof value === 'string') {
    const bytes = Buffer.from(value, 'utf8');
    if (bytes.length >= 32 && bytes.length < 0x100) parts.push(Buffer.from([0xd9, bytes.length]));
    else header(0xa0, 0xda, 0xdb, bytes.length, 31);
    parts.push(bytes);
  } else if (Array.isArray(value)) {
    header(0x90, 0xdc, 0xdd, value.length, 15);
    for (const item of value) encodeMsgpack(item, parts);
  } else {
    const entries = Object.entries(value);
    header(0x80, 0xde, 0xdf, entries.length, 15);
    for (const [k, v] of entries) {
      encodeMsgpack(k, parts);
      encodeMsgpack(v, parts);
    }
  }
  return parts;
}

const msgpackStream = (items) => Buffer.concat(items.flatMap((item) => encodeMsgpack(item)));

function fileInside(root, relative) {
  const target = path.resolve(root, decodeURIComponent(relative));
  return target.startsWith(root + path.sep) ? target : null;
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  return Buffer.concat(chunks).toString('utf8');
}

function send(res, status, body, headers = {}) {
  res.writeHead(status, { 'cache-control': 'no-store', ...headers });
  res.end(body);
}

async function serveFile(res, file, { localizeText = false } = {}) {
  const ext = path.extname(file).toLowerCase();
  let body = await readFile(file);
  if (localizeText && TEXT_EXTENSIONS.has(ext)) body = localize(body.toString('utf8'));
  send(res, 200, body, { 'content-type': TYPES[ext] ?? 'application/octet-stream' });
}

async function serveStatic(res, relative, search) {
  const file = fileInside(MIRROR, relative);
  if (!file) return send(res, 400, 'bad path');
  try {
    return await serveFile(res, file, { localizeText: true });
  } catch {
    if (OFFLINE) return send(res, 404, 'not mirrored');
  }
  const upstream = await fetch(STATIC_ORIGIN + relative + search);
  if (!upstream.ok) return send(res, upstream.status, `upstream ${upstream.status}`);
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, Buffer.from(await upstream.arrayBuffer()));
  return serveFile(res, file, { localizeText: true });
}

async function serveMedia(res, root, relative) {
  const file = fileInside(root, relative);
  try {
    if (file) return await serveFile(res, file);
  } catch {}
  if (/\.(mp4|mov|webm)$/i.test(relative)) return send(res, 404, 'video not captured');
  send(res, 200, IMAGE_PLACEHOLDER, { 'content-type': 'image/svg+xml' });
}

async function serveRecorded(res, key) {
  const entry = responses[key];
  const body = localize((await readFile(path.join(CAPTURE, entry.file))).toString('utf8'));
  send(res, entry.status, body, entry.headers);
}

async function serveDetail(res, id) {
  const file = fileInside(path.join(CAPTURE, 'details'), `${id.replace(/[^a-z0-9.-]+/gi, '_')}.html`);
  try {
    const html = localize(await readFile(file, 'utf8'));
    const headers = { 'content-type': 'text/html; charset=utf-8' };
    if (details[id]?.headers?.['x-magic']) headers['x-magic'] = details[id].headers['x-magic'];
    return send(res, 200, html, headers);
  } catch {}
  const card = cardsById.get(id);
  send(res, 200, `<div style="padding:48px;max-width:720px;margin:auto">${card?.html ?? ''}<p style="opacity:.6">Detail view not captured for ${id}. Re-run capture with a higher --detail-cards.</p></div>`, {
    'content-type': 'text/html; charset=utf-8',
  });
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  const { pathname, search } = url;
  const key = `${req.method} ${pathname}${search}`;
  try {
    if (pathname.startsWith('/_static/')) return await serveStatic(res, pathname.slice('/_static/'.length), search);
    if (pathname.startsWith('/media/')) return await serveMedia(res, path.join(CAPTURE, 'media'), pathname.slice('/media/'.length));
    if (pathname.startsWith('/_mediahost/')) return await serveMedia(res, path.join(CAPTURE, 'mediahost'), pathname.slice('/_mediahost/'.length));

    if (req.method === 'GET' && req.headers['sec-fetch-dest'] === 'document') {
      return send(res, 200, shell, { 'content-type': 'text/html; charset=utf-8' });
    }

    if (key === 'GET /cards') {
      return send(res, 200, msgpackStream(cards), { 'content-type': 'application/msgpack', 'x-checkpoint': meta.checkpoint ?? '0' });
    }
    if (req.method === 'POST' && pathname === '/cards/grid') {
      const ids = JSON.parse((await readBody(req)) || '[]');
      return send(res, 200, msgpackStream(ids.map((id) => cardsById.get(id)).filter(Boolean)), { 'content-type': 'application/msgpack' });
    }
    const detail = req.method === 'GET' && pathname.match(/^\/cards\/([^/]+)\/detail$/);
    if (detail) return await serveDetail(res, decodeURIComponent(detail[1]));
    const single = req.method === 'GET' && pathname.match(/^\/cards\/([^/]+)$/);
    if (single && cardsById.has(single[1]) && !url.searchParams.has('template')) {
      return send(res, 200, msgpackStream([cardsById.get(single[1])]), { 'content-type': 'application/msgpack' });
    }
    if (pathname === '/changes') return send(res, 200, '', { 'content-type': 'application/msgpack', 'x-count': '0' });
    if (pathname.startsWith('/hub')) return send(res, 404, 'realtime hub is not replayed');
    if (key in responses) return await serveRecorded(res, key);

    if (req.method !== 'GET') {
      if (pathname !== '/jserrors') console.log(`[write ignored] ${key}`);
      return send(res, 200, '{}', { 'content-type': 'application/json' });
    }
    console.log(`[miss] ${key} (accept: ${req.headers.accept ?? '-'})`);
    const wantsJson = (req.headers.accept ?? '').includes('json');
    return send(res, wantsJson ? 200 : 404, wantsJson ? '{}' : '', { 'content-type': wantsJson ? 'application/json' : 'text/plain' });
  } catch (error) {
    console.error(`[error] ${key}`, error);
    if (!res.headersSent) send(res, 500, String(error));
  }
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`mymind replica on http://localhost:${PORT}/everything`);
  console.log(`${cards.length} cards, ${Object.keys(details).length} detail views, captured ${meta.capturedAt}${OFFLINE ? ', offline' : ''}`);
});
