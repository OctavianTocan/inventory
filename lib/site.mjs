import { deviceVariant } from './device.mjs';
import { sendStream } from './hosted-media.mjs';
import { byteRange } from './media-files.mjs';

/** mymind's own origin; recorded keys and bodies use it, the replica serves it from `/`. */
export const APP_ORIGIN = 'https://access.mymind.com/';
/** Local path prefixes the replica serves mymind's other origins under. */
export const LOCAL_PREFIXES = [
  ['/_static/', 'https://static.accelerator.net/'],
  ['/_mediahost/', 'https://media.mymind.host/'],
  ['/_carbon/', 'https://carbon-media.accelerator.net/'],
];
/** Content types by file extension for mirrored and captured files. */
export const TYPES = {
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
/** Extensions whose mirrored files are text with mymind's origins to localize. */
export const TEXT_EXTENSIONS = new Set(['.js', '.mjs', '.css', '.html', '.json', '.svg']);
/** Grey image served for card media that was never captured. */
export const IMAGE_PLACEHOLDER = `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><rect width="100%" height="100%" fill="#e9e7e3"/></svg>`;

/**
 * Rewrite mymind's origins in recorded text to the replica's local paths.
 * @param {string} text Recorded text.
 * @returns {string} The text with every mymind origin replaced.
 */
export const localize = (text) =>
  LOCAL_PREFIXES.reduce((out, [local, origin]) => out.replaceAll(origin, local), text).replaceAll(APP_ORIGIN, '/');

/**
 * Localize every string in a decoded recording, keeping binary and codec values as they are.
 * @param {unknown} value Decoded MessagePack item or JSON value.
 * @returns {unknown} A copy with mymind's origins replaced in every string.
 */
export const localizeValue = (value) =>
  typeof value === 'string'
    ? localize(value)
    : Array.isArray(value)
      ? value.map(localizeValue)
      : value && typeof value === 'object' && !(value instanceof Uint8Array)
        ? Object.fromEntries(Object.entries(value).map(([k, v]) => [k, localizeValue(v)]))
        : value;

/**
 * Send a whole response body that is never cached.
 * @param {object} res Node response, or an adapter with `writeHead` and `end`.
 * @param {number} status HTTP status.
 * @param {string|Uint8Array} body Response body.
 * @param {object} [headers] Extra response headers.
 * @returns {void}
 */
export function send(res, status, body, headers = {}) {
  res.writeHead(status, { 'cache-control': 'no-store', ...headers });
  res.end(body);
}

/**
 * Send a body, or the part a `Range` header asks for: video and audio elements stall on a plain 200.
 * @param {object} req Request whose `range` header is read.
 * @param {object} res Response.
 * @param {number} status Status of the whole body; only a 200 is split.
 * @param {Uint8Array} body Whole body.
 * @param {object} headers Response headers.
 * @returns {void}
 */
export function sendRange(req, res, status, body, headers) {
  const { status: part, start, end } = byteRange(singleRange(req), body.length);
  if (status !== 200 || part === 200) return send(res, status, body, headers);
  if (part === 416) return send(res, 416, '', { ...headers, 'content-range': `bytes */${body.length}` });
  send(res, 206, body.subarray(start, end + 1), {
    ...headers,
    'accept-ranges': 'bytes',
    'content-range': `bytes ${start}-${end}/${body.length}`,
  });
}

// The request's Range header when it asks for one span of bytes; other forms get the whole body.
const singleRange = (req) => (/^bytes=\d*-\d*$/.test(req.headers.range ?? '') ? req.headers.range : undefined);

/**
 * The recording key of a request to the replica, in the shape `tools/replay-store.mjs` records live requests.
 * @param {object} req Request with `method` and `headers`.
 * @param {URL} url Parsed replica URL.
 * @returns {string} `<KIND> <live URL>`, where KIND is DOC for a top-level document; media URLs drop their query.
 */
export function replayKeyFor(req, url) {
  const [local, origin] = LOCAL_PREFIXES.find(([prefix]) => url.pathname.startsWith(prefix)) ?? [];
  const live = local ? origin + url.pathname.slice(local.length) : APP_ORIGIN + url.pathname.slice(1);
  const pathOnly = (local && local !== '/_static/') || url.pathname.startsWith('/media/');
  const kind = req.method === 'GET' && req.headers['sec-fetch-dest'] === 'document' ? 'DOC' : req.method;
  return `${kind} ${live}${pathOnly ? '' : url.search}`;
}

const extension = (key) => /\.[^./;]*$/.exec(key)?.[0].toLowerCase() ?? '';

/**
 * Answer the requests the library API leaves over from recordings and files a hosted server keeps in its database
 * and object store, in the order `server.mjs` answers them from `replay/`, `mirror/` and `capture/`.
 * @param {object} options
 * @param {object} options.api Library API (`createApi`); asked first.
 * @param {object} options.recorded The library's recordings (`createRecorded`).
 * @param {object} options.bucket Object store holding `static/` (mymind's website code, shared by every library) and
 *   this library's captured card media.
 * @param {string} options.capture Key prefix of this library's captured card media (`libraries/<name>/capture/`), with
 *   `media/`, `mediahost/` and `carbon/` below it; no other library's media is ever read.
 * @param {boolean} [options.offline=false] Never fetch missing website code from mymind's static origin.
 * @param {Function} [options.upstream=fetch] Fetch for missing website code.
 * @returns {{handle: (req:object, res:object, url:URL) => Promise<void>}} Answers every request it is given.
 */
export function createSite({ api, recorded, bucket, capture, offline = false, upstream = (...args) => fetch(...args) }) {
  if (!/^libraries\/[a-z0-9-]+\/capture\/$/.test(capture ?? '')) throw new Error('capture must be libraries/<name>/capture/');
  const desktop = (req) => !deviceVariant(req.headers['user-agent']);
  const textType = (key) => TYPES[extension(key)] ?? 'application/octet-stream';

  // A decoded object key below `folder`, or null for a path that climbs out of it.
  function keyFor(folder, relative) {
    const decoded = decodeURIComponent(relative);
    if (decoded.startsWith('/') || decoded.split('/').some((part) => part === '..' || part === '.')) return null;
    return `${folder}${decoded}`;
  }

  // mymind's website code, fetched once from its static origin when it was never mirrored.
  async function serveStatic(res, relative, search) {
    const key = keyFor('static/', relative);
    if (!key) return send(res, 400, 'bad path');
    const object = await bucket.read(key);
    if (object) {
      res.writeHead(200, { 'cache-control': 'no-store', 'content-type': textType(key), ...object.headers, 'content-length': String(object.size) });
      return sendStream(res, object.body);
    }
    if (offline) return send(res, 404, 'not mirrored');
    const response = await upstream(LOCAL_PREFIXES[0][1] + relative + search);
    if (!response.ok) return send(res, response.status, `upstream ${response.status}`);
    let body = new Uint8Array(await response.arrayBuffer());
    if (TEXT_EXTENSIONS.has(extension(key))) body = new TextEncoder().encode(localize(new TextDecoder().decode(body)));
    await bucket.write(key, body, body.length);
    return send(res, 200, body, { 'content-type': textType(key) });
  }

  // The largest captured rendition of the same transformed media (e.g. the 1x of a srcset's 2x), or null.
  async function sibling(key) {
    const stem = key.replace(/[^/;]*$/, '');
    if (!stem.includes(';')) return null;
    const folder = stem.slice(0, stem.lastIndexOf('/') + 1);
    const area = (name) => name.match(/(\d+)x(\d+)/)?.slice(1).reduce((a, b) => a * b) ?? 0;
    const names = (await bucket.list(stem)).filter((object) => !object.key.slice(folder.length).includes('/'));
    return names.sort((a, b) => area(b.key.slice(folder.length)) - area(a.key.slice(folder.length)))[0] ?? null;
  }

  // Stream a stored object (`found` is its `head`), or the span of it a Range header asks for.
  async function serveObject(req, res, key, { size, headers: stored }) {
    const headers = { 'content-type': textType(key), ...stored };
    const { status, start, end } = byteRange(singleRange(req), size);
    if (status === 416) return send(res, 416, '', { ...headers, 'accept-ranges': 'bytes', 'content-range': `bytes */${size}` });
    const length = Math.max(0, end - start + 1);
    const object = length && req.method !== 'HEAD' ? await bucket.read(key, { offset: start, length }) : null;
    res.writeHead(status, { 'cache-control': 'no-store', ...headers, 'content-length': String(length),
      ...(status === 206 ? { 'accept-ranges': 'bytes', 'content-range': `bytes ${start}-${end}/${size}` } : {}) });
    if (!object) return res.end();
    await sendStream(res, object.body);
  }

  // Captured card media, falling back to another size of the same image, then to a placeholder.
  async function serveMedia(req, res, folder, relative) {
    const key = keyFor(folder, relative);
    const found = key && await bucket.head(key);
    if (found) return serveObject(req, res, key, found);
    const near = key && await sibling(key);
    if (near) return serveObject(req, res, near.key, await bucket.head(near.key));
    console.log(`[placeholder] /${folder.slice(capture.length)}${relative}`);
    if (/\.(mp4|mov|webm)$/i.test(relative)) return send(res, 404, 'video not captured');
    send(res, 200, IMAGE_PLACEHOLDER, { 'content-type': 'image/svg+xml' });
  }

  return {
    async handle(req, res, url) {
      const { pathname, search } = url;
      if (await api.handle(req, res, url)) return;
      const local = LOCAL_PREFIXES.some(([prefix]) => pathname.startsWith(prefix)) || pathname.startsWith('/media/');
      const entry = local ? null : recorded.entry(req, replayKeyFor(req, url));
      if (entry) {
        let body = entry.body;
        if (desktop(req) && req.headers['sec-fetch-dest'] === 'document') body = api.shell(new TextDecoder().decode(body));
        return sendRange(req, res, entry.status, typeof body === 'string' ? new TextEncoder().encode(body) : body, entry.headers);
      }
      if (pathname.startsWith('/_static/')) return serveStatic(res, pathname.slice('/_static/'.length), search);
      if (pathname.startsWith('/media/')) return serveMedia(req, res, `${capture}media/`, pathname.slice('/media/'.length));
      if (pathname.startsWith('/_mediahost/')) {
        const relative = pathname.slice('/_mediahost/'.length);
        const hosted = keyFor(`${capture}mediahost/`, relative);
        return serveMedia(req, res, hosted && await bucket.head(hosted) ? `${capture}mediahost/` : `${capture}media/`, relative);
      }
      if (pathname.startsWith('/_carbon/')) return serveMedia(req, res, `${capture}carbon/`, pathname.slice('/_carbon/'.length));
      if (req.method === 'GET' && req.headers['sec-fetch-dest'] === 'document') {
        const shell = recorded.entry(req, replayKeyFor(req, new URL('/everything', url))) ?? recorded.entry(req, 'shell');
        const html = new TextDecoder().decode(shell?.body ?? new Uint8Array());
        return send(res, 200, desktop(req) ? api.shell(html) : html, { 'content-type': 'text/html; charset=utf-8' });
      }
      const detail = req.method === 'GET' && /^\/cards\/([^/]+)\/detail$/.exec(pathname);
      if (detail) {
        return send(res, 200, `<div style="padding:48px;max-width:720px;margin:auto"><p style="opacity:.6">Detail view not captured for ${decodeURIComponent(detail[1]).replace(/[^\w.-]/g, '')}.</p></div>`,
          { 'content-type': 'text/html; charset=utf-8' });
      }
      if (req.method !== 'GET') {
        if (pathname !== '/jserrors') console.log(`[write ignored] ${req.method} ${pathname}${search}`);
        return send(res, 200, '{}', { 'content-type': 'application/json' });
      }
      console.log(`[miss] ${req.method} ${pathname}${search} (accept: ${req.headers.accept ?? '-'})`);
      const wantsJson = (req.headers.accept ?? '').includes('json');
      return send(res, wantsJson ? 200 : 404, wantsJson ? '{}' : '', { 'content-type': wantsJson ? 'application/json' : 'text/plain' });
    },
  };
}
