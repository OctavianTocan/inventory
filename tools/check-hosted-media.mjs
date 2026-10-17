// In-process checks of the hosted server's storage: object store, hosted media and uploads, recordings and the site
// fallbacks, and a whole hosted library answering API requests. Synthetic fixtures only; no network or sockets.
import { deepStrictEqual, ok, rejects, strictEqual, throws } from 'node:assert/strict';
import { mkdtemp, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable, Writable } from 'node:stream';
import sharp from 'sharp';
import { directoryBucket } from '../lib/bucket-fs.mjs';
import { bucketUploads, createHostedMedia } from '../lib/hosted-media.mjs';
import { createHosted } from '../lib/hosted.mjs';
import { createProcessor } from '../lib/media-processor.mjs';
import { createRecorded } from '../lib/recorded.mjs';
import { IMAGE_PLACEHOLDER, createSite } from '../lib/site.mjs';
import { openSql } from '../lib/sql-node.mjs';
import { createStore } from '../lib/store.mjs';
import { createUploads } from '../lib/uploads.mjs';

const root = await mkdtemp(join(tmpdir(), 'mymind-hosted-'));
const previous = Object.fromEntries(['OFFLINE', 'GEMINI_API_KEY', 'TYPESAFE_API_KEY'].map(key => [key, process.env[key]]));
process.env.OFFLINE = '1';
delete process.env.GEMINI_API_KEY;
delete process.env.TYPESAFE_API_KEY;
const originalFetch = globalThis.fetch;
globalThis.fetch = async () => { throw new Error('Network is disabled in hosted checks'); };
const png = await sharp({ create: { width: 48, height: 32, channels: 3, background: '#539ac1' } }).png().toBuffer();
const user = { accountUid: 'account', idPrefix: 'prefix-' };
const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
let passed = 0, failed = 0;

// Report only synthetic fixture check names and safe assertion details.
async function check(name, run) {
  try { await run(); passed++; console.log(`PASS  ${name}`); }
  catch (error) { failed++; console.log(`FAIL  ${name}: ${error.message}`); }
}

// A response adapter whose `end` takes bytes, a string or a web stream, as the Worker's does.
function capture() {
  let done;
  const finished = new Promise(resolve => { done = resolve; });
  const res = {
    headersSent: false,
    writeHead(status, headers) { this.status = status; this.headers = headers; this.headersSent = true; },
    end(body) { done(new Response(body || null, { status: this.status, headers: this.headers })); },
  };
  return { res, response: finished };
}

// Ask a handler one request and collect status, headers and body bytes.
async function ask(handle, path, { method = 'GET', headers = {}, body } = {}) {
  const req = Readable.from(body === undefined ? [] : [Buffer.isBuffer(body) ? body : Buffer.from(JSON.stringify(body))]);
  Object.assign(req, { method, headers: { 'content-type': 'application/json', ...headers } });
  const { res, response } = capture();
  const handled = await handle(req, res, new URL(path, 'http://127.0.0.1'));
  if (handled === false) return null;
  const result = await response;
  return { status: result.status, headers: Object.fromEntries(result.headers), bytes: Buffer.from(await result.arrayBuffer()) };
}

// Seed a library database with only a synthetic user.
function library(file) {
  const db = openSql(file);
  createStore(db).db.prepare('INSERT INTO users VALUES (?, ?, ?, 0, ?)').run('test-user', 'test-account', 'test-', '{}');
  return db;
}

// A processor that counts rendition work and fakes the PDF and video tools.
function countingProcessor() {
  const counts = { rendition: 0 };
  const processor = createProcessor({ directory: root, run: async (command, args) => {
    if (command === 'ffprobe') return { stdout: JSON.stringify({ streams: [{ width: 48, height: 32 }], format: { duration: '1' } }) };
    const { writeFile } = await import('node:fs/promises');
    await writeFile(command === 'pdftoppm' ? args.at(-1) + '.png' : args.at(-1), command === 'ffmpeg' && args.at(-1).endsWith('.mp4') ? 'encoded video' : png);
  } });
  return { counts, processor: { ...processor, rendition: (...args) => { counts.rendition++; return processor.rendition(...args); } } };
}

try {
  await check('directory bucket keeps headers beside objects, out of listings, and rejects escaping keys', async () => {
    const bucket = directoryBucket(join(root, 'bucket-basic'));
    await bucket.write('static/a.js', 'one', 3, { 'content-type': 'text/javascript', 'x-raw': '1' });
    await bucket.write('static/b.js', Buffer.from('two'));
    deepStrictEqual(await bucket.head('static/a.js'), { size: 3, headers: { 'content-type': 'text/javascript', 'x-raw': '1' } });
    deepStrictEqual(await bucket.head('static/b.js'), { size: 3, headers: undefined });
    deepStrictEqual((await bucket.list('')).map(object => object.key), ['static/a.js', 'static/b.js']);
    const ranged = await bucket.read('static/a.js', { offset: 1, length: 2 });
    strictEqual(Buffer.from(await new Response(ranged.body).arrayBuffer()).toString(), 'ne');
    await bucket.write('static/a.js', 'uno');
    strictEqual((await bucket.head('static/a.js')).headers, undefined);
    await bucket.write('static/a.js', 'one', 3, { 'content-type': 'text/javascript' });
    await bucket.remove('static/a.js');
    strictEqual(await bucket.head('static/a.js'), null);
    strictEqual(await bucket.head('.headers/static/a.js.json'), null);
    for (const key of ['../escape', 'static/../../escape']) await rejects(bucket.write(key, 'x'), { status: 400 });
    strictEqual(await bucket.head('missing/file'), null);
    strictEqual(await bucket.read('missing/file'), null);
  });

  await check('hosted media stores originals per library and serves whole files, HEAD and byte ranges', async () => {
    const bucket = directoryBucket(join(root, 'bucket-media'));
    const { processor } = countingProcessor();
    const media = createHostedMedia({ bucket, prefix: 'libraries/one/', processor });
    const prepared = await media.prepare(png, 'image/png', 'image.png', user, 'prefix-card');
    const blob = prepared.data.blob;
    strictEqual(blob.width, 48); strictEqual(blob.height, 32);
    ok(await bucket.head(`libraries/one/media/${blob.path}`));
    deepStrictEqual((await bucket.list('libraries/')).map(object => object.key), [`libraries/one/media/${blob.path}`]);
    const original = await media.response(blob.path);
    strictEqual(original.key, `libraries/one/media/${blob.path}`); strictEqual(original.size, png.length);
    strictEqual((await media.response(blob.path.replace(/\.png$/, ''))).key, original.key);
    const whole = await ask((req, res) => media.send(res, original), '/');
    strictEqual(whole.status, 200); deepStrictEqual(whole.bytes, png);
    const head = await ask((req, res) => media.send(res, original, { method: 'HEAD' }), '/');
    strictEqual(head.bytes.length, 0); strictEqual(head.headers['content-length'], String(png.length));
    const slice = await ask((req, res) => media.send(res, original, { range: 'bytes=2-8' }), '/');
    strictEqual(slice.status, 206); deepStrictEqual(slice.bytes, png.subarray(2, 9));
    strictEqual(slice.headers['content-range'], `bytes 2-8/${png.length}`);
    const outside = await ask((req, res) => media.send(res, original, { range: `bytes=${png.length}-` }), '/');
    strictEqual(outside.status, 416); strictEqual(outside.bytes.length, 0);
    strictEqual(await createHostedMedia({ bucket, prefix: 'libraries/two/', processor }).response(blob.path), null);
    strictEqual(await media.response('../one/media/' + blob.path), null);
  });

  await check('a Node response receives streamed media with backpressure', async () => {
    const bucket = directoryBucket(join(root, 'bucket-stream'));
    const media = createHostedMedia({ bucket, processor: countingProcessor().processor });
    await bucket.write('media/clip.mp4', Buffer.from('0123456789'));
    const chunks = [];
    const res = new Writable({ highWaterMark: 1, write(chunk, encoding, done) { chunks.push(chunk); setImmediate(done); } });
    res.writeHead = (status, headers) => { strictEqual(status, 206); strictEqual(headers['content-length'], '8'); };
    await media.send(res, await media.response('clip.mp4'), { range: 'bytes=1-8' });
    strictEqual(Buffer.concat(chunks).toString(), '12345678');
  });

  await check('renditions are made once, kept in the library, and shared by concurrent requests', async () => {
    const bucket = directoryBucket(join(root, 'bucket-renditions'));
    const { counts, processor } = countingProcessor();
    const media = createHostedMedia({ bucket, prefix: 'libraries/one/', processor });
    const { data: { blob } } = await media.prepare(png, 'image/png', 'image.png', user, 'prefix-card');
    for (const suffix of [';12x12-c.webp', ';12x12-t.jpg']) {
      const [first, second] = await Promise.all([media.response(blob.path + suffix), media.response(blob.path + suffix)]);
      deepStrictEqual(first, second);
      ok(first.key.startsWith('libraries/one/renditions/'));
      const bytes = Buffer.from(await new Response((await bucket.read(first.key)).body).arrayBuffer());
      const info = await sharp(bytes).metadata();
      strictEqual(info.width, 12); strictEqual(info.height, suffix.includes('-c') ? 12 : 8);
      deepStrictEqual(await media.response(blob.path + suffix), first);
    }
    strictEqual(counts.rendition, 2);
    strictEqual((await sharp((await media.image(blob)).data).metadata()).format, 'png');
    await rejects(media.response(blob.path + ';bogus'), { status: 400 });
  });

  await check('PDF and video uploads get previews through the processor', async () => {
    const bucket = directoryBucket(join(root, 'bucket-previews'));
    const media = createHostedMedia({ bucket, processor: countingProcessor().processor });
    const doc = await media.prepare(Buffer.from('%PDF-1.5 synthetic'), 'application/pdf', 'doc.pdf', user, 'prefix-document');
    const video = await media.prepare(Buffer.from('0000ftypisom'), 'video/mp4', 'video.mp4', user, 'prefix-video');
    strictEqual(doc.data.type, 'Document'); strictEqual(video.data.type, 'Video'); strictEqual(video.data.blob.duration, 1);
    const page = await media.response(doc.data.blob.path + ';background(fff)/page(1)/20x20-t.webp');
    const bytes = Buffer.from(await new Response((await bucket.read(page.key)).body).arrayBuffer());
    strictEqual((await sharp(bytes).metadata()).width, 20);
  });

  await check('bucket uploads keep chunks as objects, drop failed chunks and sweep orphans', async () => {
    const bucket = directoryBucket(join(root, 'bucket-uploads'));
    const db = library(join(root, 'uploads.db'));
    const store = createStore(db);
    const storage = bucketUploads(bucket, 'libraries/one/');
    const media = createHostedMedia({ bucket, prefix: 'libraries/one/', processor: countingProcessor().processor });
    const uploads = createUploads(store, storage, media);
    try {
      const auth = await uploads.authorize(null), id = auth.url.split('/').at(-1);
      const put = (bytes, range) => {
        const body = Readable.from([bytes]);
        body.headers = { authorization: `Bearer ${auth.token}`, 'content-range': range, 'content-type': 'image/png', 'x-file-name': 'fixture.png' };
        return uploads.put(id, body);
      };
      await put(png.subarray(0, 10), `bytes 0-9/${png.length}`);
      await rejects(put(png.subarray(10, 12), `bytes 10-${png.length - 1}/${png.length}`), { status: 400 });
      strictEqual(await storage.size(id), 10);
      await put(png.subarray(10), `bytes 10-${png.length - 1}/${png.length}`);
      strictEqual(await storage.size(id), png.length);
      strictEqual((await bucket.list(`libraries/one/uploads/${id}/`)).length, 2);
      await bucket.write(`libraries/one/uploads/${'a'.repeat(22)}/000000000000`, 'orphan');
      await uploads.sweep();
      strictEqual((await bucket.list(`libraries/one/uploads/${'a'.repeat(22)}/`)).length, 0);
      const prepared = await uploads.prepare(id);
      const original = await media.response(prepared.data.blob.path);
      deepStrictEqual(Buffer.from(await new Response((await bucket.read(original.key)).body).arrayBuffer()), png);
    } finally { db.close(); }
  });

  await check('recordings follow the device, then the verification run, and keep the first copy of a key', async () => {
    const db = openSql(':memory:');
    const recorded = createRecorded(db);
    const key = 'GET https://access.mymind.com/thing';
    recorded.put(key, 200, { 'content-type': 'text/plain' }, Buffer.from('plain'));
    recorded.put(key, 200, {}, Buffer.from('second'));
    recorded.put(`iphone|${key}`, 200, {}, Buffer.from('phone'));
    recorded.put(`run:r1|iphone|${key}`, 200, {}, Buffer.from('phone run'));
    recorded.put(`run:r1|${key}`, 200, {}, Buffer.from('run'));
    const body = (headers) => Buffer.from(recorded.entry({ headers }, key).body).toString();
    strictEqual(body({}), 'plain');
    strictEqual(body({ 'user-agent': IPHONE }), 'phone');
    strictEqual(body({ 'user-agent': IPHONE, 'x-replay-run': 'r1' }), 'phone run');
    strictEqual(body({ 'x-replay-run': 'r1' }), 'run');
    strictEqual(body({ 'x-replay-run': 'r2' }), 'plain');
    strictEqual(recorded.entry({ headers: {} }, 'GET https://access.mymind.com/other'), null);
    strictEqual(recorded.count(), 4);
    db.close();
  });

  await check('the site serves recordings, stored headers and media fallbacks in the replica order', async () => {
    const bucket = directoryBucket(join(root, 'bucket-site'));
    const db = openSql(':memory:');
    const recorded = createRecorded(db);
    recorded.put('GET https://access.mymind.com/news/unread', 200, { 'content-type': 'application/json' }, Buffer.from('[1]'));
    recorded.put('DOC https://access.mymind.com/everything', 200, { 'content-type': 'text/html' }, Buffer.from('<p>everything</p>'));
    recorded.put('shell', 200, { 'content-type': 'text/html' }, Buffer.from('<p>shell</p>'));
    await bucket.write('static/app.js', 'code', 4, { 'content-type': 'text/javascript', 'x-raw': '1' });
    await bucket.write('static/plain.css', 'css');
    await bucket.write('libraries/one/capture/media/a/b;100x100.webp', Buffer.from('wide'));
    await bucket.write('libraries/one/capture/media/a/b;10x10.webp', Buffer.from('small'));
    await bucket.write('libraries/one/capture/media/a/clip.mp4', Buffer.from('0123456789'));
    await bucket.write('libraries/two/capture/media/a/other.webp', Buffer.from('other library'));
    await bucket.write('capture/media/a/shared.webp', Buffer.from('shared'));
    const api = { handle: async () => false, shell: html => `[desktop]${html}` };
    throws(() => createSite({ api, recorded, bucket, offline: true }), /capture must be/);
    const site = createSite({ api, recorded, bucket, capture: 'libraries/one/capture/', offline: true });
    const handle = (req, res, url) => site.handle(req, res, url);
    const text = result => result.bytes.toString();
    strictEqual(text(await ask(handle, '/news/unread')), '[1]');
    const page = await ask(handle, '/everything', { headers: { 'sec-fetch-dest': 'document' } });
    strictEqual(text(page), '[desktop]<p>everything</p>');
    strictEqual(text(await ask(handle, '/everything', { headers: { 'sec-fetch-dest': 'document', 'user-agent': IPHONE } })), '<p>everything</p>');
    strictEqual(text(await ask(handle, '/anything/else', { headers: { 'sec-fetch-dest': 'document' } })), '[desktop]<p>everything</p>');
    const code = await ask(handle, '/_static/app.js');
    strictEqual(text(code), 'code'); strictEqual(code.headers['content-type'], 'text/javascript'); strictEqual(code.headers['x-raw'], '1');
    strictEqual((await ask(handle, '/_static/plain.css')).headers['content-type'], 'text/css; charset=utf-8');
    strictEqual((await ask(handle, '/_static/missing.js')).status, 404);
    strictEqual((await ask(handle, '/_static/..%2F..%2Fetc')).status, 400);
    strictEqual(text(await ask(handle, '/media/a/b;50x50.webp')), 'wide');
    strictEqual(text(await ask(handle, '/_mediahost/a/b;10x10.webp')), 'small');
    const range = await ask(handle, '/media/a/clip.mp4', { headers: { range: 'bytes=2-5' } });
    strictEqual(range.status, 206); strictEqual(text(range), '2345'); strictEqual(range.headers['content-range'], 'bytes 2-5/10');
    const past = await ask(handle, '/media/a/clip.mp4', { headers: { range: 'bytes=99-' } });
    strictEqual(past.status, 416); strictEqual(past.headers['content-range'], 'bytes */10'); strictEqual(past.bytes.length, 0);
    strictEqual((await ask(handle, '/media/a/clip.mp4', { headers: { range: 'bytes=5-3' } })).status, 416);
    strictEqual((await ask(handle, '/media/a/clip.mp4', { headers: { range: 'bytes=0-1,4-5' } })).status, 200);
    const head = await ask(handle, '/media/a/clip.mp4', { method: 'HEAD' });
    strictEqual(head.status, 200); strictEqual(head.headers['content-length'], '10'); strictEqual(head.bytes.length, 0);
    strictEqual(text(await ask(handle, '/media/a/other.webp')), IMAGE_PLACEHOLDER);
    strictEqual(text(await ask(handle, '/media/a/shared.webp')), IMAGE_PLACEHOLDER);
    strictEqual(text(await ask(handle, '/media/..%2F..%2Ftwo%2Fcapture%2Fmedia%2Fa%2Fother.webp')), IMAGE_PLACEHOLDER);
    const recordedRange = await ask(handle, '/news/unread', { headers: { range: 'bytes=9-' } });
    strictEqual(recordedRange.status, 416); strictEqual(recordedRange.headers['content-range'], 'bytes */3');
    strictEqual(text(await ask(handle, '/media/none/x.webp')), IMAGE_PLACEHOLDER);
    strictEqual((await ask(handle, '/media/none/x.mp4')).status, 404);
    ok(text(await ask(handle, '/cards/abc/detail')).includes('Detail view not captured for abc.'));
    strictEqual(text(await ask(handle, '/unknown', { method: 'POST', body: {} })), '{}');
    strictEqual((await ask(handle, '/unknown', { headers: { accept: 'application/json' } })).status, 200);
    strictEqual((await ask(handle, '/unknown')).status, 404);
    db.close();
  });

  await check('a hosted library saves an upload as a card and serves its media, attachments and claims', async () => {
    const bucket = directoryBucket(join(root, 'bucket-library'));
    const db = library(join(root, 'library.db'));
    const hosted = await createHosted({ db, bucket, library: 'one', processor: countingProcessor().processor, msgpack: {}, push() {} });
    try {
      const request = async (path, options) => {
        const result = await ask(hosted.handle, path, options);
        return { ...result, value: result.headers['content-type']?.includes('json') && result.bytes.length ? JSON.parse(result.bytes) : undefined };
      };
      async function upload() {
        const auth = (await request('/uploads/authorize', { method: 'POST' })).value;
        const result = await request(auth.url, { method: 'PUT', body: png, headers: { 'content-type': 'image/png',
          authorization: `Bearer ${auth.token}`, 'content-range': `bytes 0-${png.length - 1}/${png.length}` } });
        strictEqual(result.status, 200);
        return auth.url.split('/').at(-1);
      }
      const uploadId = await upload();
      const saved = await request('/objects', { method: 'POST', body: { type: 'Media', uploadId } });
      strictEqual(saved.status, 201);
      strictEqual((await bucket.list(`libraries/one/uploads/${uploadId}/`)).length, 0);
      const card = (await request('/objects/' + saved.value.id)).value;
      deepStrictEqual((await request('/media/' + card.blob.path)).bytes, png);
      const head = await request('/media/' + card.blob.path, { method: 'HEAD' });
      strictEqual(head.bytes.length, 0); strictEqual(head.headers['content-length'], String(png.length));
      const alias = await request('/_mediahost/' + card.blob.path, { headers: { range: 'bytes=2-8' } });
      strictEqual(alias.status, 206); deepStrictEqual(alias.bytes, png.subarray(2, 9));
      const thumbnail = await request(`/media/${card.blob.path};12x12-c.webp`);
      strictEqual((await sharp(thumbnail.bytes).metadata()).width, 12);
      const note = (await request('/objects', { method: 'POST', body: { content: 'Synthetic note' } })).value.id;
      for (const id of [uploadId, await upload()]) {
        const attached = await request(`/objects/${note}/attachments`, { method: 'POST', body: { uploadId: id } });
        strictEqual(attached.status, 200); deepStrictEqual((await request(attached.value.path)).bytes, png);
      }
      const keys = (await bucket.list('')).map(object => object.key);
      ok(keys.every(key => key.startsWith('libraries/one/')), 'every object stays in the library');
      ok(keys.some(key => key.startsWith('libraries/one/renditions/')));
      deepStrictEqual((await readdir(root)).filter(name => name.startsWith('mymind-media-')), []);
    } finally { hosted.close(); db.close(); }
  });
} finally {
  globalThis.fetch = originalFetch;
  for (const [key, value] of Object.entries(previous)) {
    if (value === undefined) delete process.env[key]; else process.env[key] = value;
  }
  await rm(root, { recursive: true, force: true });
}
console.log(`\n${passed} passed, ${failed} failed (in-process; no sockets or Chromium)`);
process.exitCode = failed ? 1 : 0;
