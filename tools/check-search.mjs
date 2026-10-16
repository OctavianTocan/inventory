import { strictEqual, deepStrictEqual, ok, rejects } from 'node:assert/strict';
import { mkdtemp, rm, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { DatabaseSync } from 'node:sqlite';
import sharp from 'sharp';
import { ROOT, testServer, localize } from './http-check.mjs';
import { startFixtures } from './capture-fixtures.mjs';
import { openStore } from '../lib/store.mjs';
import { createWriter } from '../lib/store-write.mjs';
import { createSearch } from '../lib/search.mjs';
import { createMedia } from '../lib/local-media.mjs';
import { createEnrichment } from '../lib/enrichment.mjs';
import { documentText } from '../lib/ai.mjs';
import { analysisItem } from '../lib/search-document.mjs';
import { embedLibrary } from './embed-library.mjs';
import { compareRecordings } from './search-recordings.mjs';

const inProcess = process.argv.includes('--in-process');
const dir = await mkdtemp(join(tmpdir(), 'mymind-search-'));
const previous = Object.fromEntries(['GEMINI_API_KEY', 'GEMINI_BASE_URL', 'AI_CHAT_BASE_URL', 'TYPESAFE_API_KEY', 'OFFLINE', 'STORE_READONLY'].map(k => [k, process.env[k]]));
delete process.env.GEMINI_API_KEY; delete process.env.TYPESAFE_API_KEY; delete process.env.AI_CHAT_BASE_URL;
process.env.OFFLINE = '1'; delete process.env.STORE_READONLY;
const originalFetch = globalThis.fetch;
let server, fixtures, store;
let passed = 0, failed = 0;

// Print bounded synthetic assertion failures; recording checks report aggregates only.
async function check(name, run) {
  try { await run(); console.log(`PASS  ${name}`); passed++; }
  catch (error) { console.log(`FAIL  ${name}: ${error.message.split('\n')[0]} ${error.stack?.split('\n').find(line => line.includes('check-search.mjs:'))?.trim() ?? ''}`); failed++; }
}

// Construct a tiny editor document with visible text.
const doc = text => ({ type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text }] }] });

// Build a unit vector on one axis for predictable cosine assertions.
const vector = (axis, value = 1) => Float32Array.from({ length: 768 }, (_, i) => i === axis ? value : 0);

// Persist known vectors through the same AI storage mutation as the production pipeline.
function analyze(writer, id, axis, summary = 'synthetic analysis') {
  writer.run(() => writer.analysis(id, { embedding: vector(axis), title: '', summary, tags: [{ name: 'synthetic ai label' }] }));
}

try {
  const seed = join(dir, 'synthetic.db');
  const initial = openStore(seed);
  initial.db.prepare('INSERT INTO users VALUES (?, ?, ?, 0, ?)').run('search-user', 'search-account', 'fixture0000', JSON.stringify({ objectLimit: 10000 }));
  initial.close();
  const png = await sharp({ create: { width: 32, height: 32, channels: 3, background: '#678abc' } }).png().toBuffer();
  fixtures = await startFixtures({ png, webp: png }, inProcess);
  server = await testServer({ source: seed }); fixtures.allowOrigin(server.base);
  store = openStore(server.dbPath);
  let writer = createWriter(store, () => {}), search = createSearch(store, { offline: true });
  const get = async path => {
    const result = await server.request(path);
    strictEqual(result.response.status, 200); return result.value;
  };
  const write = async (method, path, body) => {
    const result = await server.request(path, { method, body }); ok(result.response.ok); return result.value;
  };
  const query = async (q, options = {}) => get('/search?' + new URLSearchParams({ q, ...options }));
  const has = async (q, id, options) => (await query(q, options)).matches.some(m => m.id === id);
  let first, second, link;

  await check('search shape, FTS migration and prefix vs exact word semantics', async () => {
    first = (await write('POST', '/objects', { title: 'Orchard alpha', prose: doc('silver birch blue orchard') })).id;
    second = (await write('POST', '/objects', { title: 'Orchard beta', prose: doc('birch silver blueberry') })).id;
    const result = await query('blue', { wildcard: 'true' });
    deepStrictEqual(Object.keys(result).sort(), ['count', 'elapsed', 'matches', 'query']);
    strictEqual(result.query, 'blue*'); strictEqual(result.count, 2);
    ok(await has('blue', first)); ok(!await has('blue', second)); ok(await has('blue*', second));
    ok(result.matches.every(m => Object.keys(m).length === 2 && Number.isFinite(m.score)));
    strictEqual((await query('design', { wildcard: 'true' })).count, 0, 'writable requests must override recorded searches');
    store.db.exec("INSERT INTO search_fts(search_fts, rank) VALUES('integrity-check', 1)");
  });

  await check('type aliases, tags/label/#, multiword suggestions and site/domain boundaries', async () => {
    await write('POST', `/objects/${first}/tags`, { name: 'trial design' });
    ok(await has('type:note AND tag:"trial design"', first));
    for (const q of ['tag:trial design', 'label:trial design', '#trial design']) ok(await has(q, first));
    ok(!await has('tag:trial', first)); ok(await has('tag:trial*', first));
    link = writer.run(() => writer.create({ content: 'https://sub.example.invalid/path' })).body.id;
    ok(search.text('type:placeholder').matches.some(m => m.id === link));
    const pending = store.card(link);
    writer.run(() => writer.enrich(link, { data: { type: 'WebPage' } }, { data: JSON.parse(pending.data), extras: JSON.parse(pending.extras) }));
    ok(await has('site:example.invalid', link)); ok(await has('domain:sub.example.invalid', link));
    ok(!await has('site:ample.invalid', link)); ok(await has('site:example', link));
    ok(search.text('type:web-page').matches.some(m => m.id === link));
    await write('PATCH', `/objects/${link}`, { format: 'pdf' });
    ok(await has('format:pdf AND action:read', link)); ok(!await has('action:watch', link));
  });

  await check('quotes, negation, boolean groups, escaped punctuation, dates and stable sort', async () => {
    ok(await has('"silver birch"', first)); ok(!await has('"silver birch"', second));
    ok(await has('“silver birch”', first));
    ok(await has('birch -blueberry', first)); ok(!await has('birch -blueberry', second));
    ok(await has('(title:alpha OR title:beta) AND -tag:"trial design"', second));
    ok(!await has('(title:alpha OR title:beta) AND -tag:"trial design"', first));
    ok(await has('+silver AND NOT blueberry', first));
    ok(await has('after:2000-01-01 before:2100-01-01', first));
    ok(await has('created:[2000-01-01 TO 2100-01-01]', first));
    ok(!(await query('title:"a\" OR 1=1 --"')).matches.length);
    const result = await query('orchard', { sortBy: 'bumped asc' }); strictEqual(result.matches[0].id, first);
    await write('POST', `/objects/${first}/complete`, {}); ok(await has('completed:true', first));
    ok(!await has('completed:false', first));
    const bad = await server.request('/search?q=' + 'a'.repeat(8200)); strictEqual(bad.response.status, 400);
  });

  await check('FTS synchronizes edit, note, reader content, annotation removal, trash and restore', async () => {
    await write('PATCH', `/objects/${first}`, { title: 'Cypress fixture', description: 'violet lantern', ocr: 'amber typography' });
    ok(await has('cypress', first)); ok(!await has('title:alpha', first));
    ok(await has('violet', first)); ok(await has('typography', first));
    await write('PUT', `/objects/${first}/note`, doc('cardnotezz')); ok(await has('cardnotezz', first));
    await write('PUT', `/objects/${first}/content`, doc('readernewzz')); ok(await has('readernewzz', first)); ok(!await has('birch', first));
    const tag = store.cardTags(first).find(t => t.name === 'trial design');
    await write('DELETE', `/annotations/${tag.id}`); ok(!await has('trial', first));
    await write('DELETE', `/objects/${first}`); ok(!await has('cypress', first));
    await write('POST', `/objects/${first}/restore`, {}); ok(await has('cypress', first));
    await write('POST', `/objects/${first}/tags`, { name: 'private' }); ok(!await has('cypress', first));
    const privacy = store.cardTags(first).find(t => t.name === 'private');
    await write('DELETE', `/annotations/${privacy.id}`); ok(await has('cypress', first));
  });

  await check('actual capture queue and AI result update body, summary and automatic-tag FTS', async () => {
    const captureId = writer.run(() => writer.create({ content: `${fixtures.base}/ArticleQuality` })).body.id;
    const jobs = createEnrichment({ store, writer, media: createMedia(join(dir, 'media')), offline: true, screenshot: async () => png });
    try { jobs.schedule(captureId, true); await jobs.drain(); }
    finally { jobs.close(); }
    ok(search.text('"Mind maps"').matches.some(m => m.id === captureId));
    ok(search.text('body:knowledge').matches.some(m => m.id === captureId));
    writer.run(() => writer.prose(captureId, doc('replacementreaderzz'), 'content'));
    ok(search.text('replacementreaderzz').matches.some(m => m.id === captureId));
    ok(!search.text('body:knowledge').matches.some(m => m.id === captureId));
    analyze(writer, first, 0, 'summaryonlyzz');
    ok(await has('summaryonlyzz', first)); ok(await has('synthetic', first));
    const before = search.text('rollbackzz').count;
    try { writer.run(() => { writer.addTag(first, 'rollbackzz'); throw new Error('rollback'); }); } catch {}
    strictEqual(search.text('rollbackzz').count, before);
  });

  await check('suggestions count active types and casefold tags; draft/stub/trash are excluded', async () => {
    await write('POST', `/objects/${first}/tags`, { name: 'trial label' });
    await write('POST', `/objects/${second}/tags`, { name: 'Trial Label' });
    const tags = await get('/search/suggestions?q=trial');
    strictEqual(tags.length, 1); strictEqual(tags[0].count, 2); strictEqual(tags[0].score, 4); strictEqual(tags[0].isAI, false);
    const types = await get('/search/suggestions'); const notes = types.find(s => s.queryText === 'type:note');
    strictEqual(notes.count, 2); strictEqual(notes.score, 6);
    await write('POST', '/objects', { isDraft: true, content: 'draftonlyzz' });
    await write('POST', '/objects', { isStub: true, content: 'stubonlyzz' });
    strictEqual((await query('draftonlyzz OR stubonlyzz')).count, 0);
    await write('DELETE', `/objects/${second}`); strictEqual((await get('/search/suggestions?q=trial'))[0].count, 1);
    await write('POST', `/objects/${second}/restore`, {});
  });

  await check('smart creation evaluates filters; query checks persist added/removed and exclusions', async () => {
    const space = await write('POST', '/spaces', { name: 'Synthetic smart', query: { filters: ['tag:trial label', '-title:beta'], matches: [{ id: link, score: 99 }] } });
    deepStrictEqual(space.objects.map(m => m.id), [first]); strictEqual(space.objects[0].queryId, space.query.id);
    await write('PATCH', `/objects/${second}`, { title: 'eligible now' });
    let result = await get(`/queries/${space.query.id}/check`);
    deepStrictEqual(result.added.map(m => m.id), [second]); deepStrictEqual(result.removed, []);
    strictEqual((await get(`/spaces/${space.id}`)).objects.length, 2);
    await write('DELETE', `/objects/${first}`);
    result = await get(`/queries/${space.query.id}/check`); deepStrictEqual(result.removed, [first]);
    await write('POST', `/objects/${first}/restore`, {}); await get(`/queries/${space.query.id}/check`);
    await write('POST', `/queries/${space.query.id}/objects/${first}/exclude`, {});
    result = await get(`/queries/${space.query.id}/check`); deepStrictEqual(result.added, []);
    ok(!(await get(`/spaces/${space.id}`)).objects.some(m => m.id === first));
    await write('PUT', `/spaces/${space.id}/objects/${link}`, {});
    store.db.prepare('INSERT OR REPLACE INTO api_metadata VALUES (?,200,?,?)')
      .run(`/spaces/${space.id}`, '{}', JSON.stringify({ checked: '2000-01-01T00:00:00Z' }));
    result = await get(`/queries/${space.query.id}/check`); deepStrictEqual(result.added, []);
    const updated = await get(`/spaces/${space.id}`);
    ok(updated.objects.some(m => m.id === link && !m.queryId)); ok(updated.query.checked !== '2000-01-01T00:00:00Z');
    await write('DELETE', `/objects/${second}`);
    const preview = await testServer({ readonly: true, source: server.dbPath }); fixtures.allowOrigin(preview.base);
    try {
      const checked = await preview.request(`/queries/${space.query.id}/check`); deepStrictEqual(checked.value.removed, [second]);
      const unchanged = await preview.request(`/spaces/${space.id}`); ok(unchanged.value.objects.some(m => m.id === second));
      strictEqual(unchanged.value.query.checked, updated.query.checked);
    } finally { await preview.close(); }
    await write('POST', `/objects/${second}/restore`, {});
  });

  await check('semantic requests degrade without a key/offline stub; related excludes self/hidden cards', async () => {
    analyze(writer, second, 1);
    deepStrictEqual((await get('/embeddings/search?q=forest&modelId=2&take=2')).matches, []);
    const text = await query('cypress'), blended = await query('cypress', { semantic: 'true' });
    deepStrictEqual(text.matches, blended.matches);
    const related = await get(`/embeddings/related/${first}?take=1`); strictEqual(related.matches.length, 1); strictEqual(related.matches[0].id, second);
    deepStrictEqual((await get('/embeddings/related/missing?take=100')).matches, []);
    await write('DELETE', `/objects/${second}`); strictEqual((await get(`/embeddings/related/${first}`)).count, 0);
    await write('POST', `/objects/${second}/restore`, {});
    process.env.GEMINI_API_KEY = 'synthetic-test-key'; delete process.env.GEMINI_BASE_URL;
    deepStrictEqual((await get('/embeddings/search?q=forest')).matches, []);
  });

  await check('Gemini query vectors, cosine order, take/filter/modelId and semanticBoost blending', async () => {
    process.env.GEMINI_BASE_URL = fixtures.base;
    if (!inProcess) {
      const previous = server;
      const replacement = await testServer({ source: server.dbPath });
      store.close(); await previous.close();
      server = replacement; fixtures.allowOrigin(server.base);
      store = openStore(server.dbPath); writer = createWriter(store, () => {}); search = createSearch(store, { offline: true });
    }
    const result = await get('/embeddings/search?q=forest&modelId=136434018&take=2');
    strictEqual(result.matches[0].id, first); strictEqual(result.matches[0].score, 1); strictEqual(result.matches[1].score, 0);
    strictEqual((await get('/embeddings/search?q=forest&take=0')).count, 0);
    const filtered = await get('/embeddings/search?q=forest&filter=title:cypress'); deepStrictEqual(filtered.matches.map(m => m.id), [first]);
    const low = await query('cypress', { semantic: 'true', semanticBoost: '0' });
    deepStrictEqual(low.matches, (await query('cypress')).matches);
    const high = await query('cypress', { semantic: 'true', semanticBoost: '10' });
    ok(high.matches.find(m => m.id === first).score > low.matches.find(m => m.id === first).score);
    strictEqual(high.matches[0].id, first);
    ok(!(await query('forest -title:cypress', { semantic: 'true' })).matches.some(m => m.id === first));
    const call = fixtures.requests.find(r => r.path.endsWith(':embedContent') && r.body.content.parts[0].text.includes('query: forest'));
    ok(call); strictEqual(call.body.output_dimensionality, 768);
    analyze(writer, second, 0);
    deepStrictEqual((await query('forest title:cypress', { semantic: 'true' })).matches.map(m => m.id), [first]);
  });

  await check('semantic awaits respect intervening trash/filter edits and provider failures degrade safely', async () => {
    const response = () => ({ body: { embedding: { values: [...vector(0)] } } });
    try {
      fixtures.control.embedding = async () => { writer.run(() => writer.trash(first)); return response(); };
      ok(!(await query('cypress', { semantic: 'true' })).matches.some(m => m.id === first));
      writer.run(() => writer.trash(first, true));
      fixtures.control.embedding = async () => { writer.run(() => writer.patch(first, { title: 'changed during query' })); return response(); };
      strictEqual((await get('/embeddings/search?q=forest&filter=title:cypress')).count, 0);
      writer.run(() => writer.patch(first, { title: 'Cypress fixture' }));
      fixtures.control.embedding = async () => ({ status: 400, body: {} });
      deepStrictEqual((await get('/embeddings/search?q=forest')).matches, []);
      deepStrictEqual((await query('cypress', { semantic: 'true' })).matches, (await query('cypress')).matches);
    } finally { delete fixtures.control.embedding; }
  });

  await check('backfill dry-run, limit, exact save prompt, retries, transactions, usage and resume', async () => {
    const file = join(dir, 'backfill.db'), db = openStore(file);
    db.db.prepare('INSERT INTO users VALUES (?, ?, ?, 0, ?)').run('search-user', 'search-account', 'fixture0000', '{}');
    const w = createWriter(db, () => {});
    const ids = Array.from({ length: 3 }, (_, i) => w.run(() => w.create({ title: `Backfill ${i}`, content: 'synthetic body' })).body.id);
    const row = db.card(ids[0]); const prompt = documentText(analysisItem(row)); db.close();
    const logs = [];
    const estimate = await embedLibrary(['--db', file, '--limit', '2'], { log: line => logs.push(line) });
    strictEqual(estimate.cards, 2); strictEqual(estimate.embedded, 0); ok(estimate.estimatedTokens > 0 && estimate.estimatedCost > 0);
    const fetchBefore = globalThis.fetch;
    let requests = 0, input;
    globalThis.fetch = async () => Response.json({ embeddings: [{ values: [...vector(0)] }, { values: [1] }] });
    try {
      await rejects(() => embedLibrary(['--db', file, '--limit', '2', '--run'], { log() {} }));
      const unchanged = new DatabaseSync(file, { readOnly: true });
      try { strictEqual(unchanged.prepare('SELECT COUNT(*) AS n FROM embeddings').get().n, 0); } finally { unchanged.close(); }
    } finally { globalThis.fetch = fetchBefore; }
    globalThis.fetch = async (url, options) => {
      if (!String(url).startsWith(fixtures.base)) throw new Error('Backfill attempted a nonfixture request');
      requests++; input = JSON.parse(options.body);
      if (requests <= 2) return new Response('{}', { status: requests === 1 ? 429 : 503, headers: { 'retry-after': '0' } });
      return Response.json({ embeddings: input.requests.map(() => ({ values: [...vector(0)] })), usageMetadata: { promptTokenCount: 42 } });
    };
    try {
      const result = await embedLibrary(['--db', file, '--limit', '2', '--run'], { log: line => logs.push(line) });
      strictEqual(requests, 3); strictEqual(result.embedded, 2); strictEqual(result.reportedTokens, 42);
      strictEqual(input.requests[0].content.parts[0].text, prompt);
      const resumed = await embedLibrary(['--db', file, '--run'], { log: line => logs.push(line) }); strictEqual(resumed.cards, 1); strictEqual(resumed.embedded, 1);
      const complete = await embedLibrary(['--db', file, '--run'], { log: line => logs.push(line) }); strictEqual(complete.cards, 0);
      ok(!logs.join('\n').includes('synthetic-test-key'));
    } finally { globalThis.fetch = fetchBefore; }
    const reopened = openStore(file);
    try { strictEqual(reopened.db.prepare('SELECT COUNT(*) AS n FROM embeddings').get().n, 3); }
    finally { reopened.close(); }
  });

  await check('existing-row migration, annotation/content triggers and index survive reopen', async () => {
    const file = join(dir, 'legacy.db');
    const db = new DatabaseSync(file); const { SCHEMA } = await import('../lib/store-schema.mjs'); db.exec(SCHEMA);
    db.prepare('INSERT INTO users VALUES (?, ?, ?, 0, ?)').run('legacy', 'legacy', 'fixture0000', '{}');
    db.prepare('INSERT INTO cards(id,user_id,type,data,html,created_ns,bumped_ns,modified_ns) VALUES(?,?,?, ?,?,0,0,0)')
      .run('legacy-card', 'legacy', 'Note', JSON.stringify({ title: 'migrationzz', type: 'Note', flags: 0 }), ''); db.close();
    let migrated = openStore(file);
    try {
      strictEqual(createSearch(migrated).text('migrationzz').count, 1);
      migrated.db.prepare('INSERT INTO annotations VALUES(?,?,?,?,0,0)').run('legacy-tag', 'legacy', 'legacy-card', 'annotationzz');
      strictEqual(createSearch(migrated).text('annotationzz').count, 1);
      migrated.db.prepare('INSERT INTO card_content VALUES(?,?)').run('legacy-card', JSON.stringify(doc('currentreaderzz')));
      strictEqual(createSearch(migrated).text('currentreaderzz').count, 1);
      migrated.db.prepare('DELETE FROM annotations WHERE id = ?').run('legacy-tag'); strictEqual(createSearch(migrated).text('annotationzz').count, 0);
    } finally { migrated.close(); }
    migrated = openStore(file);
    try { strictEqual(createSearch(migrated).text('currentreaderzz').count, 1); migrated.db.prepare('DELETE FROM cards WHERE id = ?').run('legacy-card'); strictEqual(createSearch(migrated).text('currentreaderzz').count, 0); }
    finally { migrated.close(); }
  });

  await check('recorded comparison and exact verification replay; unrecorded reads use SQLite', async () => {
    const recorded = await testServer({ readonly: true }); fixtures.allowOrigin(recorded.base);
    let copy;
    try {
      copy = openStore(recorded.dbPath);
      const comparison = await compareRecordings(copy, ROOT);
      console.table(comparison.map(({ q, live, ours, overlap, liveOnly, storeOnly }) => ({ q, live, ours, overlap, liveOnly, storeOnly })));
      const floors = { de: 1585, des: 933, desi: 789, desig: 786, design: 786 };
      for (const row of comparison) ok(row.overlap >= floors[row.q], `${row.q} overlap regressed`);
      console.log('Live match field evidence (overlapping counts; fields can match the same card):');
      console.table(comparison.map(({ q, fields }) => ({ q, ...fields })));
      const idx = JSON.parse(await readFile(join(ROOT, 'replay/index.json'), 'utf8'));
      for (const path of ['/search?q=design&wildcard=true', '/search/suggestions', '/search/suggestions?q=design',
        ...Object.keys(idx).filter(k => /^GET https:\/\/access.mymind.com\/queries\/[^/]+\/check$/.test(k)).map(k => new URL(k.slice(4)).pathname)]) {
        const entry = idx['GET https://access.mymind.com' + path];
        const result = await recorded.request(path);
        ok(result.bytes.equals(Buffer.from(localize(await readFile(join(ROOT, 'replay/bodies', entry.file), 'utf8')))), 'recorded bytes differ');
      }
      const unrecorded = await recorded.request('/search?q=unrecordedzz&wildcard=false');
      strictEqual(unrecorded.value.count, 0); deepStrictEqual(unrecorded.value.matches, []);
      const suggest = await recorded.request('/search/suggestions?q=unrecordedzz'); deepStrictEqual(suggest.value, []);
      const writable = createSearch(copy);
      const types = writable.suggestions(), tags = writable.suggestions('design');
      const liveTypes = JSON.parse(await readFile(join(ROOT, 'replay/bodies', idx['GET https://access.mymind.com/search/suggestions'].file), 'utf8'));
      deepStrictEqual(types, liveTypes);
      const liveTags = JSON.parse(await readFile(join(ROOT, 'replay/bodies', idx['GET https://access.mymind.com/search/suggestions?q=design'].file), 'utf8'));
      for (const tag of liveTags) ok(tags.some(t => t.queryText.toLowerCase() === tag.queryText.toLowerCase() && t.count === tag.count), 'recorded tag count differs');
      const estimate = await embedLibrary(['--db', recorded.dbPath], { log: line => console.log('SEED  ' + line) }); ok(estimate.cards > 0);
    } finally { copy?.close(); await recorded.close(); }
  });
} finally {
  store?.close(); await server?.close(); await fixtures?.close(); globalThis.fetch = originalFetch;
  for (const [key, value] of Object.entries(previous)) { if (value === undefined) delete process.env[key]; else process.env[key] = value; }
  await rm(dir, { recursive: true, force: true });
}
console.log(`\n${passed}/${passed + failed} search checks passed (${inProcess ? 'in-process transport' : 'isolated HTTP'})`);
process.exitCode = failed ? 1 : 0;
