import { strictEqual, deepStrictEqual, ok, rejects } from 'node:assert/strict';
import fs from 'node:fs';
import fsPromises, { mkdtemp, mkdir, open, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { syncBuiltinESMExports } from 'node:module';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { Readable, Writable } from 'node:stream';
import sharp from 'sharp';
import { createMedia } from '../lib/local-media.mjs';
import { createUploads } from '../lib/uploads.mjs';
import { openStore } from '../lib/store.mjs';
import { createApi } from '../lib/api.mjs';

const root = await mkdtemp(join(tmpdir(), 'mymind-media-'));
const previous = Object.fromEntries(['OFFLINE', 'STORE_MEDIA', 'GEMINI_API_KEY', 'TYPESAFE_API_KEY'].map(key => [key, process.env[key]]));
process.env.OFFLINE = '1';
delete process.env.STORE_MEDIA;
delete process.env.GEMINI_API_KEY;
delete process.env.TYPESAFE_API_KEY;
const originalFetch = globalThis.fetch;
globalThis.fetch = async () => { throw new Error('Network is disabled in media checks'); };
const png = await sharp({ create: { width: 48, height: 32, channels: 3, background: '#539ac1' } }).png().toBuffer();
let passed = 0, failed = 0;

// Report only synthetic fixture check names and safe assertion details.
async function check(name, run) {
  try { await run(); passed++; console.log(`PASS  ${name}`); }
  catch (error) { failed++; console.log(`FAIL  ${name}: ${error.message}`); }
}

// Construct a controllable promise for deterministic concurrency checks.
function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}

// Seed only a synthetic user; no recordings, mirrors, dotenv files or live account are read.
function seed(file) {
  const store = openStore(file);
  store.db.prepare('INSERT INTO users VALUES (?, ?, ?, 0, ?)').run('test-user', 'test-account', 'test-', '{}');
  return store;
}

// Build a request using the same minimal readable interface as the in-process HTTP checker.
function request(auth, bytes, range, { body = Readable.from([bytes]), name = 'fixture.png', type = 'image/png' } = {}) {
  body.headers = { authorization: `Bearer ${auth.token}`, 'content-range': range, 'content-type': type, 'x-file-name': name };
  return body;
}

// Collect the web stream accepted by tools/http-check.mjs without opening a socket.
async function serve(media, descriptor, options) {
  let response;
  await media.send({
    writeHead(status, headers) { this.status = status; this.headers = headers; },
    end(body) { response = new Response(body || null, { status: this.status, headers: this.headers }); },
  }, descriptor, options);
  return { response, bytes: Buffer.from(await response.arrayBuffer()) };
}

// Run the real API routing with in-memory request and response adapters.
async function apiFixture() {
  const dir = await mkdtemp(join(root, 'api-'));
  const dbPath = join(dir, 'test.db');
  const store = seed(dbPath);
  store.close();
  const api = await createApi({ root: dir, dbPath, msgpack: {}, localize: value => value, push() {},
    send(res, status, body, headers) { res.writeHead(status, headers); res.end(body); } });
  return {
    dir,
    async request(path, { method = 'GET', body, headers = {}, type = 'application/json' } = {}) {
      const content = body === undefined ? [] : [Buffer.isBuffer(body) ? body : Buffer.from(JSON.stringify(body))];
      const req = Readable.from(content);
      Object.assign(req, { method, headers: { 'content-type': type, ...headers } });
      let response;
      const handled = await api.handle(req, {
        writeHead(status, values) { this.status = status; this.headers = values; },
        end(value) { response = new Response(value || null, { status: this.status, headers: this.headers }); },
      }, new URL(path, 'http://127.0.0.1'));
      ok(handled);
      const bytes = Buffer.from(await response.arrayBuffer());
      return { response, bytes, value: response.headers.get('content-type')?.includes('json') ? JSON.parse(bytes) : undefined };
    },
    close() { api.close(); },
  };
}

try {
  await check('sparse original HEAD and byte ranges use stat and bounded filesystem streams', async () => {
    const dir = join(root, 'sparse');
    await mkdir(dir);
    const file = await open(join(dir, 'original.mp4'), 'w');
    const length = 512 * 1024 * 1024;
    try { await file.truncate(length); await file.write(Buffer.from('0123456789'), 0, 10, 0); }
    finally { await file.close(); }
    const media = createMedia(dir);
    const savedRead = fsPromises.readFile, savedStream = fs.createReadStream;
    const streams = [];
    fsPromises.readFile = async () => { throw new Error('Whole-file read is forbidden'); };
    fs.createReadStream = (path, options) => { streams.push(options); return savedStream(path, options); };
    syncBuiltinESMExports();
    try {
      const descriptor = await media.response('original.mp4');
      strictEqual(descriptor.size, length); strictEqual(descriptor.bytes, undefined);
      const head = await serve(media, descriptor, { method: 'HEAD' });
      strictEqual(head.bytes.length, 0); strictEqual(head.response.headers.get('content-length'), String(length));
      strictEqual(streams.length, 0);
      const slice = await serve(media, descriptor, { range: 'bytes=2-8' });
      strictEqual(slice.response.status, 206); strictEqual(slice.bytes.toString(), '2345678');
      strictEqual(slice.response.headers.get('content-range'), `bytes 2-8/${length}`);
      deepStrictEqual(streams, [{ start: 2, end: 8 }]);
      const tail = await serve(media, descriptor, { range: 'bytes=-3' });
      strictEqual(tail.bytes.length, 3); deepStrictEqual(streams.at(-1), { start: length - 3, end: length - 1 });
      for (const range of ['bytes=-', 'bytes=-0', `bytes=${length}-`, 'bytes=8-2', 'bytes=0-9007199254740992', 'bytes=0-1,3-4']) {
        const before = streams.length;
        const result = await serve(media, descriptor, { range });
        strictEqual(result.response.status, 416); strictEqual(result.bytes.length, 0);
        strictEqual(streams.length, before);
      }
      await serve(media, descriptor, { method: 'HEAD', range: 'bytes=1-3' });
      strictEqual(streams.length, 2);
    } finally { fsPromises.readFile = savedRead; fs.createReadStream = savedStream; syncBuiltinESMExports(); }
  });

  await check('native Writable streaming supports backpressure and exact range length', async () => {
    const media = createMedia(join(root, 'sparse'));
    const descriptor = await media.response('original.mp4');
    const chunks = [];
    const res = new Writable({ highWaterMark: 1, write(chunk, encoding, done) { chunks.push(chunk); setImmediate(done); } });
    res.writeHead = (status, headers) => { strictEqual(status, 206); strictEqual(headers['content-length'], '8'); };
    await media.send(res, descriptor, { range: 'bytes=1-8' });
    strictEqual(Buffer.concat(chunks).toString(), '12345678');
  });

  await check('file-backed PNG processing, extensionless originals and cached renditions stay file-backed', async () => {
    const dir = join(root, 'images');
    await mkdir(dir);
    const input = join(dir, 'input.png');
    await writeFile(input, png);
    const media = createMedia(dir);
    const prepared = await media.prepare({ file: input }, 'image/png', 'image.png', { accountUid: 'account', idPrefix: 'prefix-' }, 'prefix-card');
    const blob = prepared.data.blob;
    strictEqual(blob.width, 48); strictEqual(blob.height, 32);
    const original = await media.response(blob.path);
    deepStrictEqual((await serve(media, original)).bytes, png);
    strictEqual((await media.response(blob.path.replace(/\.png$/, ''))).file, original.file);
    for (const suffix of [';12x12-c.webp', ';12x12-t.jpg', ';12x12.png']) {
      const result = await media.response(blob.path + suffix);
      strictEqual(result.bytes, undefined);
      const info = await sharp(result.file).metadata();
      strictEqual(info.width, 12); strictEqual(info.height, suffix.includes('-c') ? 12 : 8);
      const cached = await media.response(blob.path + suffix);
      strictEqual(cached.file, result.file); strictEqual(cached.size, result.size);
      const head = await serve(media, cached, { method: 'HEAD' });
      strictEqual(head.bytes.length, 0); strictEqual(head.response.headers.get('content-length'), String(result.size));
    }
    const image = await media.image(blob);
    strictEqual((await sharp(image.data).metadata()).format, 'png');
  });

  await check('video cache publishes only successful unique encodes and cleans failed intermediates', async () => {
    const dir = join(root, 'video');
    await mkdir(dir);
    await writeFile(join(dir, 'original.mp4'), Buffer.from('synthetic original'));
    const started = deferred(), gate = deferred();
    let fail = true, calls = 0;
    const outputs = [];
    const media = createMedia(dir, { run: async (command, args) => {
      strictEqual(command, 'ffmpeg'); calls++;
      const output = args.at(-1); outputs.push(output);
      await writeFile(output, fail ? 'partial' : 'complete encoded video');
      if (fail) { started.resolve(); await gate.promise; throw new Error('Synthetic encoder failure'); }
    } });
    const relative = 'original.mp4;64x64/hevc.mp4';
    const cache = join(dir, '.renditions', relative.replaceAll('/', '_'));
    const first = media.response(relative);
    await started.promise;
    strictEqual(await stat(cache).then(() => true, () => false), false);
    ok(outputs[0].startsWith(cache + '.')); ok(outputs[0].endsWith('.tmp'));
    const second = media.response(relative);
    gate.resolve();
    await rejects(first, { status: 422 }); await rejects(second, { status: 422 });
    strictEqual(calls, 1); deepStrictEqual(await readdir(dirname(cache)), []);
    fail = false;
    const result = await media.response(relative);
    strictEqual((await readFile(result.file)).toString(), 'complete encoded video');
    ok(outputs[0] !== outputs[1]);
    await media.response(relative); strictEqual(calls, 2);
    const slice = await serve(media, result, { range: 'bytes=0-7' });
    strictEqual(slice.bytes.toString(), 'complete');
  });

  await check('PDF/video preview processors receive file paths and publish image previews atomically', async () => {
    const dir = join(root, 'previews');
    const calls = [];
    const media = createMedia(dir, { run: async (command, args) => {
      calls.push(command);
      if (command === 'ffprobe') return { stdout: JSON.stringify({ streams: [{ width: 48, height: 32 }], format: { duration: '1' } }) };
      const target = command === 'pdftoppm' ? args.at(-1) + '.png' : args.at(-1);
      await writeFile(target, png);
      const source = args[args.indexOf('-i') + 1];
      if (command === 'ffmpeg') strictEqual(typeof source, 'string');
    } });
    const user = { accountUid: 'account', idPrefix: 'prefix-' };
    const doc = await media.prepare(Buffer.from('%PDF-1.5 synthetic'), 'application/pdf', 'doc.pdf', user, 'prefix-document');
    const video = await media.prepare(Buffer.from('0000ftypisom'), 'video/mp4', 'video.mp4', user, 'prefix-video');
    strictEqual(doc.data.type, 'Document'); strictEqual(video.data.type, 'Video');
    strictEqual(video.data.blob.duration, 1); deepStrictEqual(calls, ['pdftoppm', 'ffprobe', 'ffmpeg']);
    const result = await media.response(doc.data.blob.path + ';background(fff)/page(1)/20x20-t.webp');
    strictEqual((await sharp(result.file).metadata()).width, 20);
  });

  await check('expired uploads and orphan files are swept while active reservations remain protected', async () => {
    const store = seed(join(root, 'sweep.db'));
    const directory = join(root, 'uploads-sweep');
    const uploads = createUploads(store, directory, {});
    try {
      const auth = await uploads.authorize(null), id = auth.url.split('/').at(-1);
      await uploads.put(id, request(auth, Buffer.from('abc'), 'bytes 0-2/3'));
      const orphan = 'a'.repeat(22);
      await writeFile(join(directory, orphan), 'orphan');
      store.db.prepare('UPDATE uploads SET expires_ms = 0 WHERE id = ?').run(id);
      await uploads.authorize(null);
      strictEqual(store.db.prepare('SELECT id FROM uploads WHERE id = ?').get(id), undefined);
      strictEqual(await stat(join(directory, id)).then(() => true, () => false), false);
      strictEqual(await stat(join(directory, orphan)).then(() => true, () => false), false);
      await rejects(uploads.prepare(id), { status: 404 });
      const partial = await uploads.authorize(null), partialId = partial.url.split('/').at(-1);
      await writeFile(join(directory, partialId), 'uncommitted bytes from an interrupted chunk');
      await uploads.sweep();
      strictEqual((await stat(join(directory, partialId))).size, 0, 'crashed uncommitted bytes do not escape the aggregate cap');
    } finally { store.close(); }
  });

  await check('aggregate temporary byte cap includes concurrent reservations and stored partial chunks', async () => {
    const store = seed(join(root, 'budget.db'));
    const directory = join(root, 'uploads-budget');
    const uploads = createUploads(store, directory, {}, { maxBytes: 10 });
    try {
      const [a, b, c] = await uploads.authorize('3');
      const started = deferred(), gate = deferred();
      const slow = Readable.from((async function* () { started.resolve(); await gate.promise; yield Buffer.from('123456'); })());
      const running = uploads.put(a.url.split('/').at(-1), request(a, null, 'bytes 0-5/6', { body: slow }));
      await started.promise;
      const activeId = a.url.split('/').at(-1);
      store.db.prepare('UPDATE uploads SET expires_ms = 0 WHERE id = ?').run(activeId);
      await uploads.sweep();
      ok(store.db.prepare('SELECT id FROM uploads WHERE id = ?').get(activeId));
      store.db.prepare('UPDATE uploads SET expires_ms = ? WHERE id = ?').run(Date.now() + 60000, activeId);
      let consumed = false;
      const blocked = Readable.from((async function* () { consumed = true; yield Buffer.from('abcdef'); })());
      await rejects(uploads.put(b.url.split('/').at(-1), request(b, null, 'bytes 0-5/6', { body: blocked })), { status: 413 });
      strictEqual(consumed, false);
      gate.resolve(); await running;
      await uploads.put(b.url.split('/').at(-1), request(b, Buffer.from('abcd'), 'bytes 0-3/7'));
      await rejects(uploads.put(c.url.split('/').at(-1), request(c, Buffer.from('x'), 'bytes 0-0/1')), { status: 413 });
      strictEqual(store.db.prepare('SELECT SUM(received) n FROM uploads').get().n, 10);
    } finally { store.close(); }
  });

  await check('failed chunks roll back file length and release their storage reservation', async () => {
    const store = seed(join(root, 'rollback.db'));
    const directory = join(root, 'uploads-rollback');
    const uploads = createUploads(store, directory, {}, { maxBytes: 6 });
    try {
      const auth = await uploads.authorize(null), id = auth.url.split('/').at(-1);
      await uploads.put(id, request(auth, Buffer.from('abc'), 'bytes 0-2/6'));
      await rejects(uploads.put(id, request(auth, Buffer.from('de'), 'bytes 3-5/6')), { status: 400 });
      strictEqual((await stat(join(directory, id))).size, 3);
      await uploads.put(id, request(auth, Buffer.from('def'), 'bytes 3-5/6'));
      strictEqual((await readFile(join(directory, id))).toString(), 'abcdef');
    } finally { store.close(); }
  });

  await check('API claims remove temp originals while duplicate objects and uploaded attachments retain permanent sources', async () => {
    const fixture = await apiFixture();
    try {
      async function upload() {
        const auth = (await fixture.request('/uploads/authorize', { method: 'POST' })).value;
        const id = auth.url.split('/').at(-1);
        const result = await fixture.request(auth.url, { method: 'PUT', body: png, type: 'image/png',
          headers: { authorization: `Bearer ${auth.token}`, 'content-range': `bytes 0-${png.length - 1}/${png.length}` } });
        strictEqual(result.response.status, 200);
        return id;
      }
      const uploadId = await upload();
      const saved = await fixture.request('/objects', { method: 'POST', body: { type: 'Media', uploadId } });
      strictEqual(saved.response.status, 201);
      strictEqual(await stat(join(fixture.dir, 'uploads', uploadId)).then(() => true, () => false), false);
      const duplicate = await fixture.request('/objects', { method: 'POST', body: { type: 'Media', uploadId } });
      strictEqual(duplicate.response.status, 200); deepStrictEqual(duplicate.value, saved.value);
      const card = (await fixture.request('/objects/' + saved.value.id)).value;
      const original = '/media/' + card.blob.path;
      deepStrictEqual((await fixture.request(original)).bytes, png);
      const head = await fixture.request(original, { method: 'HEAD' });
      strictEqual(head.bytes.length, 0); strictEqual(head.response.headers.get('content-length'), String(png.length));
      const alias = await fixture.request('/_mediahost/' + card.blob.path, { headers: { range: 'bytes=2-8' } });
      strictEqual(alias.response.status, 206); deepStrictEqual(alias.bytes, png.subarray(2, 9));
      const note = (await fixture.request('/objects', { method: 'POST', body: { content: 'Synthetic note' } })).value.id;
      for (const id of [uploadId, await upload()]) {
        const attached = await fixture.request(`/objects/${note}/attachments`, { method: 'POST', body: { uploadId: id } });
        strictEqual(attached.response.status, 200); deepStrictEqual((await fixture.request(attached.value.path)).bytes, png);
        strictEqual(await stat(join(fixture.dir, 'uploads', id)).then(() => true, () => false), false);
        const reused = await fixture.request(`/objects/${note}/attachments`, { method: 'POST', body: { uploadId: id } });
        strictEqual(reused.response.status, 200); deepStrictEqual((await fixture.request(reused.value.path)).bytes, png);
      }
    } finally { fixture.close(); }
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
