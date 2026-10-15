// Replays a captured mymind session on localhost: the real shell and front-end modules,
// with every API call answered from capture/ instead of mymind's servers.
import { createHash } from 'node:crypto';
import http from 'node:http';
import { watchFile } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { deviceVariant } from './tools/replay-store.mjs';

const ROOT = path.dirname(fileURLToPath(import.meta.url));
const CAPTURE = path.join(ROOT, 'capture');
const MIRROR = path.join(ROOT, 'mirror', 'static');
const REPLAY = path.join(ROOT, 'replay');
const PORT = Number(process.env.PORT ?? 4321);
const OFFLINE = process.env.OFFLINE === '1';

const STATIC_ORIGIN = 'https://static.accelerator.net/';
const MEDIA_HOST = 'https://media.mymind.host/';
const CARBON_HOST = 'https://carbon-media.accelerator.net/';
const APP_ORIGIN = 'https://access.mymind.com/';
const LOCAL_PREFIXES = [
  ['/_static/', STATIC_ORIGIN],
  ['/_mediahost/', MEDIA_HOST],
  ['/_carbon/', CARBON_HOST],
];

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
  LOCAL_PREFIXES.reduce((out, [local, origin]) => out.replaceAll(origin, local), text).replaceAll(APP_ORIGIN, '/');

const meta = JSON.parse(await readFile(path.join(CAPTURE, 'meta.json'), 'utf8'));
const shell = localize(await readFile(path.join(CAPTURE, 'shell.html'), 'utf8'));
const cards = JSON.parse(await readFile(path.join(CAPTURE, 'cards.json'), 'utf8')).map((card) => ({
  html: localize(card.html),
  json: localize(card.json),
}));
const cardsById = new Map(cards.map((card) => [card.html.match(/data-id="([^"]+)"/)?.[1], card]));
const responses = JSON.parse(await readFile(path.join(CAPTURE, 'responses.json'), 'utf8'));
const details = JSON.parse(await readFile(path.join(CAPTURE, 'details.json'), 'utf8'));
let replayIndex = {};
const msgpack = await import(pathToFileURL(path.join(MIRROR, '134/0.109.0/scripts/vendor/msgpack.min.js')).href);
const replayBodies = new Map();
async function reloadReplay() {
  replayIndex = JSON.parse(await readFile(path.join(REPLAY, 'index.json'), 'utf8').catch(() => '{}'));
  replayBodies.clear();
  return Object.keys(replayIndex).length;
}
await reloadReplay();
watchFile(path.join(REPLAY, 'index.json'), { interval: 1000 }, async () => console.log(`replay/ reloaded: ${await reloadReplay()} responses`));

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

// Video and audio elements stream with Range requests and stall on a plain 200.
function sendRange(req, res, status, body, headers) {
  const range = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range ?? '');
  if (!range || status !== 200) return send(res, status, body, headers);
  const start = range[1] ? Number(range[1]) : Math.max(0, body.length - Number(range[2]));
  const end = range[1] && range[2] ? Math.min(Number(range[2]), body.length - 1) : body.length - 1;
  send(res, 206, body.subarray(start, end + 1), {
    ...headers,
    'accept-ranges': 'bytes',
    'content-range': `bytes ${start}-${end}/${body.length}`,
  });
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
  console.log(`[placeholder] ${root.slice(CAPTURE.length)}/${relative}`);
  if (/\.(mp4|mov|webm)$/i.test(relative)) return send(res, 404, 'video not captured');
  send(res, 200, IMAGE_PLACEHOLDER, { 'content-type': 'image/svg+xml' });
}

async function serveRecorded(res, key) {
  const entry = responses[key];
  const body = localize((await readFile(path.join(CAPTURE, entry.file))).toString('utf8'));
  send(res, entry.status, body, entry.headers);
}

// Same key shape as tools/replay-store.mjs, from the replica-side URL.
function replayKeyFor(req, url) {
  const [local, origin] = LOCAL_PREFIXES.find(([prefix]) => url.pathname.startsWith(prefix)) ?? [];
  const live = local ? origin + url.pathname.slice(local.length) : APP_ORIGIN + url.pathname.slice(1);
  const pathOnly = (local && local !== '/_static/') || url.pathname.startsWith('/media/');
  const kind = req.method === 'GET' && req.headers['sec-fetch-dest'] === 'document' ? 'DOC' : req.method;
  return `${kind} ${live}${pathOnly ? '' : url.search}`;
}

// mymind renders different HTML per device, so phone recordings take precedence for phone requests.
function replayedFor(req, url) {
  let key = replayKeyFor(req, url);
  const variant = deviceVariant(req.headers['user-agent']);
  if (variant) key = replayIndex[`${variant}|${key}`] ? `${variant}|${key}` : key;
  // A verification run asks for the responses its own live pass recorded.
  const run = req.headers['x-replay-run'];
  return (run && replayIndex[`run:${run}|${key}`]) || replayIndex[key];
}

const localizeValue = (value) =>
  typeof value === 'string'
    ? localize(value)
    : Array.isArray(value)
      ? value.map(localizeValue)
      : value && typeof value === 'object' && !(value instanceof Uint8Array)
        ? Object.fromEntries(Object.entries(value).map(([k, v]) => [k, localizeValue(v)]))
        : value;

async function replayBody(entry) {
  if (replayBodies.has(entry.file)) return replayBodies.get(entry.file);
  let body = await readFile(path.join(REPLAY, 'bodies', entry.file));
  const type = entry.headers['content-type'] ?? '';
  if (type.includes('msgpack')) {
    body = Buffer.concat([...msgpack.decodeMulti(body)].map((item) => msgpack.encode(localizeValue(item))));
  } else if (/text|javascript|json|svg|xml/.test(type)) {
    body = Buffer.from(localize(body.toString('utf8')));
  }
  replayBodies.set(entry.file, body);
  return body;
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
    // mymind redirects framed media (PDFs) to its media host, so the viewer runs in a cross-site frame. Mirror
    // that with a cross-site loopback address: frame rendering differs between same-site and cross-site frames.
    const dest = req.headers['sec-fetch-dest'];
    if (req.method === 'GET' && pathname.startsWith('/media/') && (dest === 'iframe' || dest === 'frame') && req.headers.host?.startsWith('localhost')) {
      return send(res, 302, '', { location: `http://127.0.0.1:${PORT}/_mediahost/${pathname.slice('/media/'.length)}${search}` });
    }
    if (req.method === 'POST' && pathname === '/__replay/reload') {
      return send(res, 200, JSON.stringify({ responses: await reloadReplay(), offline: OFFLINE }), { 'content-type': 'application/json' });
    }
    const replayed = replayedFor(req, url);
    if (replayed) return sendRange(req, res, replayed.status, await replayBody(replayed), replayed.headers);

    if (pathname.startsWith('/_static/')) return await serveStatic(res, pathname.slice('/_static/'.length), search);
    if (pathname.startsWith('/media/')) return await serveMedia(res, path.join(CAPTURE, 'media'), pathname.slice('/media/'.length));
    if (pathname.startsWith('/_mediahost/')) {
      const relative = pathname.slice('/_mediahost/'.length);
      const hosted = fileInside(path.join(CAPTURE, 'mediahost'), relative);
      const root = hosted && (await readFile(hosted).then(() => true, () => false)) ? 'mediahost' : 'media';
      return await serveMedia(res, path.join(CAPTURE, root), relative);
    }
    if (pathname.startsWith('/_carbon/')) return await serveMedia(res, path.join(CAPTURE, 'carbon'), pathname.slice('/_carbon/'.length));

    if (req.method === 'GET' && req.headers['sec-fetch-dest'] === 'document') {
      const replayedShell = replayedFor(req, new URL('/everything', url));
      const body = replayedShell ? await replayBody(replayedShell) : shell;
      return send(res, 200, body, { 'content-type': 'text/html; charset=utf-8' });
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

// Realtime hub: a SignalR JSON-protocol endpoint that completes the handshake and keeps the socket
// alive, so the app connects exactly as it does against mymind (no pushes are ever sent).
server.on('upgrade', (req, socket) => {
  if (!req.url.startsWith('/hub')) return socket.destroy();
  const accept = createHash('sha1').update(`${req.headers['sec-websocket-key']}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`).digest('base64');
  socket.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
  const frame = (opcode, payload) => {
    const head = payload.length < 126 ? Buffer.from([0x80 | opcode, payload.length]) : Buffer.from([0x80 | opcode, 126, payload.length >> 8, payload.length & 0xff]);
    socket.write(Buffer.concat([head, payload]));
  };
  let buffer = Buffer.alloc(0);
  socket.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      if (buffer.length < 2) return;
      let length = buffer[1] & 0x7f;
      let offset = 2;
      if (length === 126) {
        if (buffer.length < 4) return;
        length = buffer.readUInt16BE(2);
        offset = 4;
      }
      const mask = buffer.subarray(offset, offset + 4);
      offset += 4;
      if (buffer.length < offset + length) return;
      const payload = Buffer.from(buffer.subarray(offset, offset + length)).map((byte, i) => byte ^ mask[i % 4]);
      const opcode = buffer[0] & 0x0f;
      buffer = buffer.subarray(offset + length);
      if (opcode === 0x8) return socket.end();
      if (opcode === 0x9) frame(0xa, payload);
      if (opcode === 0x1 && payload.toString().includes('"protocol"')) frame(0x1, Buffer.from('{}\x1e'));
    }
  });
  const ping = setInterval(() => frame(0x1, Buffer.from('{"type":6}\x1e')), 15000);
  socket.on('close', () => clearInterval(ping));
  socket.on('error', () => clearInterval(ping));
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`mymind replica on http://localhost:${PORT}/everything`);
  console.log(`${cards.length} cards, ${Object.keys(details).length} detail views, captured ${meta.capturedAt}${OFFLINE ? ', offline' : ''}`);
  console.log(`${Object.keys(replayIndex).length} replayed responses from REA scenarios`);
});
