// Records the logged-in mymind app (shell, card stream, side endpoints, card details, media)
// from an open Chrome tab, so server.mjs can replay it offline.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { connectToPage } from './cdp.mjs';

const { values: opts } = parseArgs({
  options: {
    cdp: { type: 'string', default: 'http://127.0.0.1:9222' },
    out: { type: 'string', default: 'capture' },
    'media-cards': { type: 'string', default: '300' },
    'detail-cards': { type: 'string', default: '30' },
    videos: { type: 'boolean', default: false },
  },
});

const ORIGIN = 'https://access.mymind.com';
const MEDIA_HOST = 'https://media.mymind.host/';
const OUT = path.resolve(opts.out);

const GETS = [
  { path: '/spaces', accept: 'application/json' },
  { path: '/news/unread', accept: 'application/json' },
  { path: '/achievements/unseen', accept: 'application/json' },
  { path: '/tags/recent?take=10', accept: 'application/json' },
  { path: '/search/suggestions', accept: 'application/json' },
  { path: '/ui', accept: 'application/json' },
  { path: '/components/NewSpaceModal', accept: '*/*' },
];

function pageFetch(requests) {
  const token = (globalThis._ && _.authenticityToken) || document.getElementById('authenticityToken')?.content;
  const toBase64 = (buffer) => {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    return btoa(binary);
  };
  const queue = [...requests];
  const results = [];
  const worker = async () => {
    for (let req = queue.shift(); req; req = queue.shift()) {
      try {
        const res = await fetch(req.url, {
          credentials: 'include',
          headers: {
            Accept: req.accept ?? '*/*',
            'x-authenticity-token': token,
            'Viewport-Width': String(innerWidth),
            'Viewport-Height': String(innerHeight),
            DPR: String(devicePixelRatio),
            ...req.headers,
          },
        });
        const headers = Object.fromEntries(
          [...res.headers].filter(
            ([k]) => (k.startsWith('x-') || k === 'content-type') && !/^x-(amz-|cache$|authenticity-token$)/.test(k),
          ),
        );
        results.push({ url: req.url, status: res.status, headers, base64: toBase64(await res.arrayBuffer()) });
      } catch (error) {
        results.push({ url: req.url, error: String(error) });
      }
    }
  };
  return Promise.all(Array.from({ length: 6 }, worker)).then(() => results);
}

async function pageLoadCards() {
  const base = [...document.scripts].find((s) => s.src.includes('/scripts/base.js')).src;
  const { decodeMultiStream } = await import(base.replace('base.js', 'vendor/msgpack.min.js'));
  const token = (globalThis._ && _.authenticityToken) || document.getElementById('authenticityToken')?.content;
  const res = await fetch('/cards', {
    headers: {
      Accept: 'application/msgpack',
      'x-authenticity-token': token,
      'Viewport-Width': String(innerWidth),
      'Viewport-Height': String(innerHeight),
      DPR: String(devicePixelRatio),
    },
  });
  const items = [];
  for await (const item of decodeMultiStream(res.body)) items.push(item);
  globalThis.__mmCapture = items;
  const shell = await (await fetch('/everything', { headers: { Accept: 'text/html' } })).text();
  const staticUrls = performance
    .getEntriesByType('resource')
    .map((e) => e.name)
    .filter((u) => u.startsWith('https://static.accelerator.net/'));
  return {
    count: items.length,
    checkpoint: res.headers.get('x-checkpoint'),
    viewport: { width: innerWidth, height: innerHeight, dpr: devicePixelRatio, screen: [screen.width, screen.height] },
    shell,
    staticUrls,
  };
}

function pageCardSlice({ start, end }) {
  return globalThis.__mmCapture.slice(start, end);
}

const toFileName = (p) => p.replace(/^\//, '').replace(/[^a-z0-9.-]+/gi, '_');

async function writeInside(root, relative, data) {
  const target = path.resolve(root, relative);
  if (!target.startsWith(root + path.sep)) throw new Error(`Refusing to write outside ${root}: ${relative}`);
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, data);
}

function mediaUrlsIn(html, includeVideos) {
  const found = new Set();
  const decoded = html.replaceAll('&amp;', '&');
  for (const [url] of decoded.matchAll(/(?:https:\/\/access\.mymind\.com)?\/media\/[^"'\s)<>,]+/g)) found.add(url.replace(ORIGIN, ''));
  for (const [url] of decoded.matchAll(/https:\/\/media\.mymind\.host\/[^"'\s)<>,]+/g)) found.add(url);
  return [...found].filter((u) => includeVideos || !/\.(mp4|mov|webm)(\?|$)/i.test(u));
}

async function fetchInBatches(page, requests, label) {
  const all = [];
  for (let i = 0; i < requests.length; i += 40) {
    all.push(...(await page.evaluate(pageFetch, requests.slice(i, i + 40))));
    process.stdout.write(`\r${label}: ${Math.min(i + 40, requests.length)}/${requests.length}`);
  }
  if (requests.length) process.stdout.write('\n');
  return all;
}

// Media redirects cross-origin without CORS, so page fetch() fails; DevTools' loader carries the tab's cookies instead.
async function loadThroughDevTools(page, frameId, url) {
  const { resource } = await page.send('Network.loadNetworkResource', {
    frameId,
    url,
    options: { disableCache: false, includeCredentials: true },
  });
  if (!resource.success || resource.httpStatusCode !== 200 || !resource.stream) return null;
  const chunks = [];
  for (;;) {
    const read = await page.send('IO.read', { handle: resource.stream, size: 1 << 20 });
    chunks.push(Buffer.from(read.data, read.base64Encoded ? 'base64' : 'utf8'));
    if (read.eof) break;
  }
  await page.send('IO.close', { handle: resource.stream });
  return Buffer.concat(chunks);
}

const page = await connectToPage(opts.cdp, ORIGIN);
console.log(`Attached to ${page.url}`);
await mkdir(OUT, { recursive: true });

const loaded = await page.evaluate(pageLoadCards);
const cards = [];
for (let start = 0; start < loaded.count; start += 250) {
  cards.push(...(await page.evaluate(pageCardSlice, { start, end: start + 250 })));
}
await writeFile(path.join(OUT, 'shell.html'), loaded.shell);
await writeFile(path.join(OUT, 'cards.json'), JSON.stringify(cards));
await writeFile(path.join(OUT, 'static-urls.json'), JSON.stringify([...new Set(loaded.staticUrls)], null, 2));
console.log(`Cards: ${cards.length}, shell: ${loaded.shell.length} chars`);

const responses = {};
for (const res of await page.evaluate(pageFetch, GETS.map((g) => ({ url: g.path, accept: g.accept })))) {
  if (res.error) {
    console.warn(`  ${res.url}: ${res.error}`);
    continue;
  }
  const file = `responses/${toFileName(res.url)}.body`;
  await writeInside(OUT, file, Buffer.from(res.base64, 'base64'));
  responses[`GET ${res.url}`] = { status: res.status, headers: res.headers, file };
}
await writeFile(path.join(OUT, 'responses.json'), JSON.stringify(responses, null, 2));
console.log(`Side endpoints: ${Object.keys(responses).length}`);

const detailIds = cards
  .slice(0, Number(opts['detail-cards']))
  .map((c) => c.html.match(/data-id="([^"]+)"/)?.[1])
  .filter(Boolean);
const detailRequests = detailIds.map((id) => ({
  url: `/cards/${id}/detail`,
  accept: 'text/html,image/webp',
  headers: { 'x-viewport-width': String(loaded.viewport.screen[0]), 'x-viewport-height': String(loaded.viewport.screen[1]), 'x-partial': '1' },
}));
const details = {};
let detailHtml = '';
for (const res of await fetchInBatches(page, detailRequests, 'Card details')) {
  if (res.error || res.status !== 200) continue;
  const id = res.url.split('/')[2];
  const html = Buffer.from(res.base64, 'base64');
  detailHtml += html.toString();
  await writeInside(OUT, `details/${toFileName(id)}.html`, html);
  details[id] = { headers: res.headers };
}
await writeFile(path.join(OUT, 'details.json'), JSON.stringify(details, null, 2));

const mediaSource = [
  loaded.shell,
  detailHtml,
  ...cards.slice(0, Number(opts['media-cards'])).map((c) => c.html),
  ...(await Promise.all(Object.values(responses).map((r) => readFile(path.join(OUT, r.file), 'utf8')))),
].join('\n');
const mediaUrls = mediaUrlsIn(mediaSource, opts.videos);
const { frameTree } = await page.send('Page.getFrameTree');
const mediaQueue = [...mediaUrls];
let saved = 0;
let done = 0;
await Promise.all(
  Array.from({ length: 6 }, async () => {
    for (let url = mediaQueue.shift(); url; url = mediaQueue.shift()) {
      const body = await loadThroughDevTools(page, frameTree.frame.id, url.startsWith('/') ? ORIGIN + url : url).catch(() => null);
      if (body) {
        const relative = url.startsWith(MEDIA_HOST)
          ? `mediahost/${decodeURIComponent(url.slice(MEDIA_HOST.length))}`
          : `media/${decodeURIComponent(url.slice('/media/'.length))}`;
        await writeInside(OUT, relative, body);
        saved++;
      }
      process.stdout.write(`\rMedia: ${++done}/${mediaUrls.length}`);
    }
  }),
);
if (mediaUrls.length) process.stdout.write('\n');

await writeFile(
  path.join(OUT, 'meta.json'),
  JSON.stringify(
    { capturedAt: new Date().toISOString(), origin: ORIGIN, checkpoint: loaded.checkpoint, viewport: loaded.viewport, cards: cards.length, details: Object.keys(details).length, media: saved },
    null,
    2,
  ),
);
console.log(`Details: ${Object.keys(details).length}, media saved: ${saved}/${mediaUrls.length}. Wrote ${OUT}`);
page.close();
