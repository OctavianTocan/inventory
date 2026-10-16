import { strictEqual, deepStrictEqual, ok, rejects } from 'node:assert';
import { mkdtemp, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import sharp from 'sharp';
import { testServer } from './http-check.mjs';
import { cases, fixtureMedia, startFixtures } from './capture-fixtures.mjs';
import { openStore } from '../lib/store.mjs';
import { createWriter } from '../lib/store-write.mjs';
import { createMedia } from '../lib/local-media.mjs';
import { createEnrichment } from '../lib/enrichment.mjs';
import { detectType, sanitizePost, pageMetadata } from '../lib/capture-html.mjs';
import { fetchResource, screenshotResource } from '../lib/capture-link.mjs';
import { renderCard } from '../lib/render-card.mjs';
import { embed, describe, pickTags } from '../lib/ai.mjs';
import { tagKey } from '../lib/ai-tags.mjs';
import { checkCaptureHttp } from './check-capture-http.mjs';
import { checkCaptureQuality } from './check-capture-quality.mjs';
import { checkCaptureBrowser } from './check-capture-browser.mjs';
import { checkLinkCapture } from './check-link-lifecycle.mjs';

const inProcess = process.argv.includes('--in-process');
const dir = await mkdtemp(join(tmpdir(), 'mymind-capture-'));
const previous = Object.fromEntries(['GEMINI_API_KEY', 'TYPESAFE_API_KEY', 'AI_CHAT_API_KEY', 'GEMINI_BASE_URL', 'AI_CHAT_BASE_URL', 'TYPESAFE_BASE_URL', 'STORE_MEDIA'].map(key => [key, process.env[key]]));
delete process.env.GEMINI_API_KEY;
delete process.env.TYPESAFE_API_KEY;
delete process.env.AI_CHAT_API_KEY;
delete process.env.STORE_MEDIA;
const mediaBytes = await fixtureMedia(dir);
const fixtures = await startFixtures(mediaBytes, inProcess);
const server = await testServer();
fixtures.allowOrigin(server.base);
const hub = await server.hub();
let passed = 0, failed = 0, skipped = 0;

// Report only assertion names, never captured library data or provider credentials.
async function check(name, run) {
  try {
    if (await run() === false) { console.log(`SKIP  ${name} (requires host sockets)`); skipped++; }
    else { console.log(`PASS  ${name}`); passed++; }
  }
  catch (error) { console.log(`FAIL  ${name}: ${error.message.slice(0, 500)}\n${error.stack?.split('\n').find(line => line.includes('check-capture.mjs')) ?? ''}`); failed++; }
}

// Read the exact persisted capture state independently of the HTTP serializer.
function row(id) {
  const db = new DatabaseSync(server.dbPath, { readOnly: true });
  try {
    const statement = db.prepare('SELECT * FROM cards WHERE id = ?');
    statement.setReadBigInts(true);
    const result = statement.get(id);
    return { ...result, data: JSON.parse(result.data), extras: JSON.parse(result.extras) };
  } finally { db.close(); }
}

// Wait for one observable background completion with a bounded deadline.
async function until(read, predicate, timeout = 15000) {
  const end = Date.now() + timeout;
  do {
    const result = await read();
    if (predicate(result)) return result;
    await new Promise(resolve => setTimeout(resolve, 20));
  } while (Date.now() < end);
  throw new Error('Background completion timed out');
}

// Save a URL and prove the response precedes any fixture HTTP request.
async function save(name, body = {}) {
  const before = fixtures.requests.length;
  const result = await server.request('/objects', { method: 'POST', body: { content: `${fixtures.base}/${name}`, ...body } });
  strictEqual(result.response.status, 201);
  if (inProcess) strictEqual(fixtures.requests.length, before, '201 returned before fetch');
  return result.value.id;
}

// Wait for capture completion; AI is deliberately disabled for capture-only checks.
async function captured(id) {
  return until(() => row(id), item => !item.extras.capturePending && !(item.data.flags & 1), 45000);
}

// Exercise the actual binary API with an inclusive two-chunk range and capability token.
async function upload(bytes, type, name) {
  const auth = (await server.request('/uploads/authorize', { method: 'POST' })).value;
  const split = Math.max(1, Math.floor(bytes.length / 2));
  let uploadId;
  for (const [start, end] of [[0, split - 1], [split, bytes.length - 1]]) {
    const result = await server.request(auth.url, { method: 'PUT', body: bytes.subarray(start, end + 1), type,
      headers: { authorization: `Bearer ${auth.token}`, 'x-file-name': encodeURI(name), 'content-range': `bytes ${start}-${end}/${bytes.length}`, ...(uploadId ? { 'x-upload-id': uploadId } : {}) } });
    strictEqual(result.response.status, 200, 'chunk status');
    uploadId = result.value.blob.id;
    strictEqual(result.value.received, end + 1);
  }
  const result = await server.request('/objects', { method: 'POST', body: { type: 'Media', uploadId } });
  strictEqual(result.response.status, 201);
  return { id: result.value.id, uploadId };
}

let imageId, imageUploadId, articleId;
try {
  await checkLinkCapture({ check, server, fixtures, row });
  await checkCaptureHttp(check);
  await checkCaptureBrowser(check, fixtures, mediaBytes.png, inProcess);
  await checkCaptureQuality({ check, server, fixtures, mediaBytes, save, captured, row, until, inProcess });
  for (const [name, url, className, structure] of cases) await check(`${name}: metadata, extras, recorded renderer structure, hub update`, async () => {
    const id = await save(name);
    const item = await captured(id);
    const type = name === 'SoftwareApplicationPlay' ? 'SoftwareApplication' : name;
    strictEqual(detectType(url, name === 'Product' ? { entities: [{ '@type': 'Product' }] } : name === 'Article' ? { meta: { 'og:type': ['article'] } } : {}), type);
    strictEqual(item.data.type, type);
    strictEqual(item.data.title, type === 'Repository' ? 'repo' : 'Fixture title');
    strictEqual(item.data.description, 'Fixture description');
    strictEqual(item.data.domain, new URL(url).hostname);
    strictEqual(item.extras.linkUrl, url);
    strictEqual(item.extras.title, type === 'Repository' ? 'repo' : 'Fixture title');
    ok(item.extras.captureText.includes(['XPost', 'ThreadsPost', 'SubstackNote'].includes(type) ? 'Fixture post body' : 'Fixture body text'));
    ok(item.html.includes(`class="${className}"`));
    ok(item.html.includes(structure));
    strictEqual(item.html, renderCard(item.data, item.extras));
    ok(item.extras.cover.src.startsWith('/media/'));
    strictEqual(item.extras.cover.width, 640);
    strictEqual(item.extras.cover.height, 480);
    ok(/^#[0-9a-f]{6}$/.test(item.extras.cover.dominantColor));
    if (['XPost', 'ThreadsPost', 'SubstackNote'].includes(type)) {
      strictEqual(item.extras.post.author, 'by Fixture author');
      ok(item.extras.post.bodyHtml.includes('<strong>post</strong>'));
      ok(!/script|onclick|javascript:|<img/.test(item.extras.post.bodyHtml));
      ok(Array.isArray(item.extras.post.gallery));
    }
    if (type === 'Product') { strictEqual(item.extras.product.price, 'USD 12.50'); strictEqual(item.data.brand, 'Fixture brand'); }
    if (type === 'RedditPost') strictEqual(item.extras.reddit.subreddit, 'fixtures');
    if (type === 'SoftwareApplication') ok(item.extras.app.srcset.includes('/media/'));
    if (type === 'Article') {
      ok(item.data.flags & 2);
      const detail = await server.request(`/cards/${id}/detail`, { accept: 'text/html' });
      ok(detail.value.includes('Fixture body text'));
    }
    await until(() => hub.messages, messages => messages.some(message => message.target === 'object:updated' && message.arguments[0].id === id));
    if (type === 'Article') articleId = id;
  });

  await check('multi-image social posts use a gallery without duplicating the cover', async () => {
    const item = await captured(await save('Gallery'));
    strictEqual(item.data.type, 'XPost');
    strictEqual(item.extras.post.gallery.length, 2);
    ok(!item.extras.cover);
    ok(item.html.includes('gallery-wrap gallery-grid'));
    strictEqual([...item.html.matchAll(/<img class="lazy"/g)].length, 2);
  });

  await check('JSON-LD graphs, Twitter metadata and hostile post sanitization', async () => {
    const item = await captured(await save('JsonLd'));
    strictEqual(item.data.title, 'JSON-LD title');
    strictEqual(item.data.type, 'Article');
    strictEqual(item.data.description, 'JSON-LD description');
    strictEqual(item.data.author, 'Schema author');
    ok(item.extras.cover);
    strictEqual(pageMetadata('<meta name=twitter:title content="Twitter title"><meta name=twitter:image content="/cover.png">', fixtures.base).title, 'Twitter title');
    const clean = sanitizePost('<p onclick=x>Hi &lt;script&gt;<svg><script>x()</script></svg><a href="jav&#x61;script:alert(1)">bad</a><a href="/safe" style=x>ok</a><span>text</span></p>', fixtures.base);
    ok(!/onclick|<script|<svg|javascript:|style=/.test(clean));
    ok(clean.includes('&lt;script&gt;'));
    ok(clean.includes(`href="${fixtures.base}/safe"`));
  });

  await check('PDF URL capture produces a Document and actual page rendition', async () => {
    const item = await captured(await save('doc.pdf'));
    strictEqual(item.data.type, 'Document');
    strictEqual(item.data.blob.type, 'application/pdf');
    ok(item.html.includes('document-background'));
    const result = await server.request(`/media/${item.data.blob.path};background(fff)/page(1)/642x840-t.webp`, { accept: 'image/webp' });
    strictEqual(result.response.headers.get('content-type'), 'image/webp');
    const info = await sharp(result.bytes).metadata();
    ok(info.width > 0 && info.height > 0);
  });

  await check('blocked fetch and live-account redirect retain readable fallback cards', async () => {
    for (const name of ['blocked', 'redirect-account']) {
      const item = await captured(await save(name));
      strictEqual(item.data.type, 'Placeholder');
      ok(item.html.includes('inaccessible-guts') && item.html.includes('blocking us'));
      ok(item.data.source.url.endsWith(name));
    }
    await rejects(() => fetchResource('https://access.mymind.com/everything'), /Unsupported/);
    await rejects(() => fetchResource('https://access.mymind.com./everything'), /Unsupported/);
  });

  await check('PNG upload, original bytes, resize/crop/JPEG and media-host aliases', async () => {
    const { id, uploadId } = await upload(mediaBytes.png, 'image/png', 'fixture image.png');
    imageId = id;
    imageUploadId = uploadId;
    const item = row(id);
    strictEqual(item.data.type, 'Image');
    strictEqual(item.data.blob.width, 640);
    strictEqual(item.data.blob.height, 480);
    ok(item.html.includes('class="image"'));
    deepStrictEqual((await server.request(`/media/${item.data.blob.path}`)).bytes, mediaBytes.png);
    deepStrictEqual(await readFile(join(dirname(server.dbPath), 'media', item.data.blob.path)), mediaBytes.png);
    for (const [suffix, expected, width, height] of [[';100x80-c.webp', 'webp', 100, 80], [';100x80-t.webp', 'webp', 100, 75], [';100x80.jpeg', 'jpeg', 100, 75]]) {
      const result = await server.request(`/_mediahost/${item.data.blob.path}${suffix}`);
      strictEqual(result.response.status, 200);
      const info = await sharp(result.bytes).metadata();
      strictEqual(info.format, expected); strictEqual(info.width, width); strictEqual(info.height, height);
    }
    strictEqual((await server.request('/objects', { method: 'POST', body: { type: 'Media', uploadId } })).response.status, 200);
    strictEqual((await server.request(`/media/${item.data.blob.path};9000x9000.webp`)).response.status, 400);
  });

  await check('clipboard multipart PNG/PDF and raw, remote and uploaded image attachments', async () => {
    for (const [extension, mime, type] of [['png', 'image/png', 'Image'], ['pdf', 'application/pdf', 'Document']]) {
      const form = new FormData();
      form.set('metadata', new Blob([JSON.stringify({ source: 'clipboard', title: 'Pasted fixture' })], { type: 'application/json' }));
      form.set('blob', new Blob([mediaBytes[extension]], { type: mime }), `fixture.${extension}`);
      const request = new Request('http://localhost/objects', { method: 'POST', body: form });
      const result = await server.request('/objects', { method: 'POST', body: Buffer.from(await request.arrayBuffer()), type: request.headers.get('content-type') });
      strictEqual(result.response.status, 201);
      const item = row(result.value.id);
      strictEqual(item.data.type, type); strictEqual(item.data.title, 'Pasted fixture');
      strictEqual(item.data.source.name, 'clipboard');
    }
    for (const options of [{ body: mediaBytes.png, type: 'image/png' }, { body: { remoteUrl: `${fixtures.base}/cover.png` } }, { body: { uploadId: imageUploadId } }]) {
      const result = await server.request(`/objects/${imageId}/attachments`, { method: 'POST', ...options });
      strictEqual(result.response.status, 200);
      strictEqual(result.value.width, 640); strictEqual(result.value.height, 480);
      ok(result.value.path.startsWith('/media/'));
      deepStrictEqual((await server.request(result.value.path)).bytes, mediaBytes.png);
    }
  });

  for (const [extension, mime, type, selector] of [['mp4', 'video/mp4', 'Video', 'hover-preview'], ['pdf', 'application/pdf', 'Document', 'document-background']]) {
    await check(`${type} upload: original, renderer URLs, preview and byte ranges`, async () => {
      const bytes = mediaBytes[extension];
      const { id } = await upload(bytes, mime, `fixture.${extension}`);
      const item = row(id);
      strictEqual(item.data.type, type);
      ok(item.data.blob.width > 0 && item.data.blob.height > 0);
      ok(item.html.includes(selector));
      const range = await server.request(`/media/${item.data.blob.path}`, { headers: { range: 'bytes=2-8' } });
      strictEqual(range.response.status, 206); deepStrictEqual(range.bytes, bytes.subarray(2, 9));
      strictEqual(range.response.headers.get('content-range'), `bytes 2-8/${bytes.length}`);
      const preview = /(?:data-src|src)="(\/media\/[^" ]+\.(?:jpeg|webp))"/.exec(item.html)[1];
      const result = await server.request(preview);
      strictEqual(result.response.status, 200); ok((await sharp(result.bytes).metadata()).width > 0);
      if (type === 'Video') {
        const video = /src="(\/media\/[^" ]+\/hevc.mp4)"/.exec(item.html)[1];
        const rendered = await server.request(video);
        strictEqual(rendered.response.status, 200); strictEqual(rendered.response.headers.get('content-type'), 'video/mp4');
        strictEqual(rendered.bytes.toString('latin1', 4, 8), 'ftyp');
      }
    });
  }

  await check('upload authorization, count, incomplete file and range validation', async () => {
    strictEqual((await server.request('/uploads/authorize?count=2', { method: 'POST' })).value.length, 2);
    strictEqual((await server.request('/uploads/authorize?count=101', { method: 'POST' })).response.status, 400);
    const auth = (await server.request('/uploads/authorize', { method: 'POST' })).value;
    const id = auth.url.split('/').at(-1);
    strictEqual((await server.request('/objects', { method: 'POST', body: { type: 'Media', uploadId: id } })).response.status, 409);
    const options = { method: 'PUT', body: Buffer.from('abc'), type: 'image/png', headers: { authorization: `Bearer ${auth.token}`, 'content-range': 'bytes 0-2/6' } };
    strictEqual((await server.request(auth.url, { ...options, headers: { ...options.headers, authorization: 'Bearer invalid' } })).response.status, 401);
    strictEqual((await server.request(auth.url, options)).response.status, 200);
    strictEqual((await server.request(auth.url, options)).response.status, 409, 'overlap rejected');
    strictEqual((await server.request(auth.url, { ...options, headers: { ...options.headers, 'content-range': 'bytes 4-5/6' } })).response.status, 409);
    strictEqual((await server.request(auth.url, { ...options, headers: { ...options.headers, 'content-range': 'bytes 3-5/7' } })).response.status, 409);
    strictEqual((await server.request(auth.url, { ...options, body: Buffer.from('ab'), headers: { ...options.headers, 'content-range': 'bytes 3-5/6' } })).response.status, 400);
  });

  await check('screenshot cover and concurrent user edits merge into capture stages', async () => {
    const store = openStore(server.dbPath), media = createMedia(join(dirname(server.dbPath), 'media'));
    const writer = createWriter(store, () => {});
    let shots = 0;
    const jobs = createEnrichment({ store, writer, media, offline: true, screenshot: async () => { shots++; return mediaBytes.png; } });
    try {
      for (const [name, type] of [['NoImage', 'WebPage'], ['BrokenImage', 'WebPage'], ['ScreenshotFixture', 'Screenshot']]) {
        const { body: { id } } = writer.run(() => writer.create({ content: `${fixtures.base}/${name}`, type }));
        jobs.schedule(id, true);
        await jobs.drain();
        const item = row(id);
        strictEqual(item.data.type, type); ok(item.extras.cover.src.startsWith('/media/'));
        strictEqual(item.data.flags, 0);
        ok(!item.extras.screenshotPending && !item.extras.screenshotFailed);
        ok(item.html.includes(`class="${type === 'Screenshot' ? 'screenshot' : 'web-page'}"`));
      }
      strictEqual(shots, 3);
      fixtures.control.imageDelay = 150;
      const id = await save('WebPage?race=1');
      await until(() => row(id), item => item.data.title === 'Fixture title');
      await server.request(`/objects/${id}`, { method: 'PATCH', body: { title: 'User wins', description: 'Edited while capturing' } });
      await server.request(`/objects/${id}/tags`, { method: 'POST', body: { name: 'manual-during-capture' } });
      const item = await captured(id);
      strictEqual(item.data.title, 'User wins'); strictEqual(item.data.description, 'Edited while capturing');
      ok(item.data.tags.some(tag => tag.name === 'manual-during-capture' && tag.flags === 8));
      ok(item.html.includes('User wins')); ok(!item.html.includes('Fixture title'));
    } finally { fixtures.control.imageDelay = 0; jobs.close(); store.close(); }
  });

  await check('real Chromium screenshot of a fixture page without og:image', async () => {
    if (inProcess) return false;
    const item = await captured(await save('NoImage?chromium=1'));
    strictEqual(item.data.type, 'WebPage'); ok(item.extras.cover);
    const result = await server.request(item.extras.cover.src);
    strictEqual((await sharp(result.bytes).metadata()).width, 800);
  });

  await check('screenshot interception validates every resource redirect before following', async () => {
    for (const target of ['redirect-account', 'redirect-external']) {
      const before = fixtures.requests.length;
      let aborted = false;
      await screenshotResource({
        request: () => ({ method: () => 'GET', url: () => `${fixtures.base}/${target}` }),
        fulfill: async () => { throw new Error('Forbidden redirect was fulfilled'); },
        abort: async () => { aborted = true; },
      }, { offline: true });
      strictEqual(fixtures.requests.length, before + 1); strictEqual(aborted, true);
    }
  });

  await check('AI stubs: summary, vocabulary, PNG cover, flags, embedding, hub and detail', async () => {
    const beforeAI = fixtures.requests.length;
    Object.assign(process.env, { GEMINI_API_KEY: 'fixture-key', TYPESAFE_API_KEY: 'fixture-typesafe-key', GEMINI_BASE_URL: fixtures.base, AI_CHAT_BASE_URL: fixtures.base, TYPESAFE_BASE_URL: fixtures.base });
    const store = openStore(server.dbPath), media = createMedia(join(dirname(server.dbPath), 'media'));
    const events = [];
    const writer = createWriter(store, (target, payload) => events.push({ target, payload }));
    const jobs = createEnrichment({ store, writer, media, offline: true });
    try {
      writer.run(() => writer.addTag(imageId, 'design'));
      jobs.schedule(imageId);
      await jobs.drain();
      const item = row(imageId);
      strictEqual(item.data.summary, 'A fixture summary <safe>.');
      ok(!(item.data.flags & 1));
      strictEqual(item.data.tags.find(tag => tag.name === 'design').flags, 8, 'manual tag preserved');
      ok(!item.data.tags.some(tag => ['website', 'link', 'page', 'screenshot'].includes(tag.name)));
      ok(item.data.tags.some(tag => tagKey(tag.name) === 'image' && tag.flags === 2));
      const embedding = store.db.prepare('SELECT dimensions, vector FROM embeddings WHERE card_id = ?').get(imageId);
      strictEqual(embedding.dimensions, 768); strictEqual(embedding.vector.byteLength, 3072);
      strictEqual(Buffer.from(embedding.vector).readFloatLE(0), 1);
      ok(events.some(event => event.target === 'object:analyzed' && event.payload.id === imageId));
      const detail = await server.request(`/cards/${imageId}/detail`, { accept: 'text/html' });
      ok(detail.value.includes('<mymind-tldr><p>A fixture summary &lt;safe&gt;.</p>'));
      const embed = fixtures.requests.slice(beforeAI).find(request => request.path.endsWith(':embedContent'));
      strictEqual(embed.body.content.parts.find(part => part.inline_data).inline_data.mime_type, 'image/png');
      const jpeg = fixtures.requests.slice(beforeAI).find(request => request.path === '/v1/systemone');
      ok(Object.keys(jpeg.body.questions).length > 0);
      ok(fixtures.requests.some(request => request.path.endsWith(':batchEmbedContents')));
      jobs.schedule(articleId); await jobs.drain();
      ok(!row(articleId).data.tags.some(tag => ['image', 'screenshot'].includes(tag.name)));
      fixtures.control.aiFailure = true;
      jobs.schedule(articleId); await jobs.drain();
      strictEqual(row(articleId).data.summary, 'A fixture summary <safe>.'); ok(!(row(articleId).data.flags & 1));
      fixtures.control.aiFailure = false;
      fixtures.control.chatDelay = 150;
      const beforeChats = fixtures.requests.filter(request => request.path === '/chat/completions').length;
      jobs.schedule(articleId);
      await until(() => row(articleId), item => item.data.flags & 1);
      writer.run(() => writer.patch(articleId, { title: 'Edited during analysis' }));
      await jobs.drain();
      strictEqual(row(articleId).data.title, 'Edited during analysis');
      ok(fixtures.requests.filter(request => request.path === '/chat/completions').length >= beforeChats + 2, 'stale analysis rerun');
      const analyzedEvents = events.filter(event => event.target === 'object:analyzed').length;
      jobs.schedule(articleId);
      await until(() => row(articleId), item => item.data.flags & 1);
      writer.run(() => writer.trash(articleId));
      await jobs.drain();
      ok(row(articleId).data.deleted); ok(!(row(articleId).data.flags & 1));
      strictEqual(events.filter(event => event.target === 'object:analyzed').length, analyzedEvents);
      writer.run(() => writer.trash(articleId, true));
      jobs.schedule(articleId); await jobs.drain();
      ok(!row(articleId).data.deleted); ok(!(row(articleId).data.flags & 1));
      delete process.env.TYPESAFE_API_KEY;
      const beforeJev = fixtures.requests.filter(request => request.path === '/v1/systemone').length;
      jobs.schedule(articleId); await jobs.drain();
      strictEqual(fixtures.requests.filter(request => request.path === '/v1/systemone').length, beforeJev);
      ok(row(articleId).data.tags.some(tag => tag.name === 'design' && tag.flags === 2));
      process.env.TYPESAFE_API_KEY = 'fixture-typesafe-key';
    } finally { fixtures.control.aiFailure = false; fixtures.control.chatDelay = 0; jobs.close(); store.close(); }
  });

  await check('Gemini, chat and TypeSafe reject redirects without forwarding API POSTs', async () => {
    fixtures.control.aiRedirect = true;
    try {
      for (const request of [() => embed({ text: 'Fixture redirect' }, { kind: 'document' }), () => describe({ title: 'Fixture redirect' }), () => pickTags({ title: 'Fixture redirect' }, ['design'])]) {
        await rejects(request, error => error.status === 307);
      }
    } finally { fixtures.control.aiRedirect = false; }
  });

  await check('note save and explicit analyze route invoke background AI with missing-key fallback', async () => {
    // Socket servers inherit their environment at launch; a new isolated server receives stub configuration.
    const aiServer = inProcess ? server : await testServer();
    fixtures.allowOrigin(aiServer.base);
    const aiHub = await aiServer.hub();
    try {
      const result = await aiServer.request('/objects', { method: 'POST', body: { type: 'Note', content: 'Fixture note for AI' } });
      strictEqual(result.response.status, 201);
      const id = result.value.id;
      const read = async () => (await aiServer.request(`/objects/${id}`)).value;
      await until(read, item => item.summary === 'A fixture summary <safe>.');
      const analyzed = await aiServer.request(`/cards/${id}/analyze`, { method: 'POST', body: {} });
      strictEqual(analyzed.value.status, 'Processing');
      await until(() => aiHub.messages, messages => messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length >= 2);
      await aiServer.request(`/objects/${id}/prose`, { method: 'PUT', body: { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Edited note for AI' }] }] } });
      await until(() => aiHub.messages, messages => messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length >= 3);
    } finally { aiHub.close(); if (aiServer !== server) await aiServer.close(); }
    delete process.env.GEMINI_API_KEY;
    const skipped = await server.request(`/cards/${imageId}/analyze`, { method: 'POST' });
    if (inProcess) strictEqual(skipped.value.status, 'Skipped');
  });

  await check('STORE_READONLY creates no uploads, jobs, embeddings or AI requests', async () => {
    process.env.GEMINI_API_KEY = 'fixture-key';
    const readonly = await testServer({ readonly: true });
    fixtures.allowOrigin(readonly.base);
    const before = fixtures.requests.length;
    try {
      await readonly.request('/objects', { method: 'POST', body: { content: `${fixtures.base}/WebPage` } });
      await readonly.request('/uploads/authorize', { method: 'POST' });
      await readonly.request(`/cards/${articleId}/analyze`, { method: 'POST' });
      await new Promise(resolve => setTimeout(resolve, 50));
      strictEqual(fixtures.requests.length, before);
      const db = new DatabaseSync(readonly.dbPath, { readOnly: true });
      try { strictEqual(db.prepare('SELECT COUNT(*) n FROM uploads').get().n, 0); strictEqual(db.prepare('SELECT COUNT(*) n FROM embeddings').get().n, 0); }
      finally { db.close(); }
    } finally { await readonly.close(); }
  });
} finally {
  hub.close();
  await server.close();
  await fixtures.close();
  await rm(dir, { recursive: true, force: true });
  for (const [key, value] of Object.entries(previous)) { if (value === undefined) delete process.env[key]; else process.env[key] = value; }
}
console.log(`\n${passed}/${passed + failed} capture checks passed, ${skipped} skipped (${inProcess ? 'in-process transport' : 'real HTTP and Chromium'})`);
process.exitCode = failed ? 1 : 0;
