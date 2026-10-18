// Proves every read endpoint on a synthetic library database without recordings.
import { deepStrictEqual, ok, strictEqual } from 'node:assert';
import { testServer } from './http-check.mjs';

const server = await testServer({ readonly: true });
let passed = 0;
let failed = 0;

async function check(label, fn) {
  try {
    await fn();
    console.log(`PASS  ${label}`);
    passed++;
  } catch (error) {
    console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`);
    failed++;
  }
}

try {
  let cards = [];
  await check('GET /cards streams msgpack grid with quota and checkpoint headers', async () => {
    const result = await server.request('/cards', { accept: 'application/msgpack' });
    strictEqual(result.response.status, 200);
    ok(result.response.headers.get('content-type')?.includes('application/msgpack'));
    ok(result.response.headers.get('x-object-quota'));
    ok(result.response.headers.get('x-checkpoint'));
    strictEqual(result.response.headers.get('access-control-expose-headers'), 'x-object-quota, x-checkpoint');
    cards = result.value;
    ok(cards.length >= 100);
  });

  await check('GET /cards on iPhone renders phone card markup', async () => {
    const result = await server.request('/cards', { accept: 'application/msgpack', headers: { 'user-agent': 'iPhone' } });
    strictEqual(result.response.status, 200);
    strictEqual(result.value.length, cards.length);
    ok(result.value[0].html.includes('data-id='));
  });

  await check('GET /cards/trash streams trashed cards', async () => {
    const result = await server.request('/cards/trash', { accept: 'application/msgpack' });
    strictEqual(result.response.status, 200);
    ok(result.value.length >= 2);
  });

  await check('POST /cards/grid returns requested cards in requested order', async () => {
    const selected = [cards.at(-1), cards[0], cards[10]];
    const ids = selected.map(card => card.html.match(/data-id="([^"]+)"/)[1]);
    const result = await server.request('/cards/grid', { method: 'POST', body: ids, accept: 'application/msgpack' });
    strictEqual(result.response.status, 200);
    deepStrictEqual(result.value, selected);
  });

  await check('page shell renders for every page route', async () => {
    for (const route of ['/everything', '/spaces', '/serendipity', '/trash', '/account', '/achievements', '/bookshelf']) {
      const result = await server.request(route, { accept: 'text/html', headers: { 'sec-fetch-dest': 'document' } });
      strictEqual(result.response.status, 200, route);
      ok(result.value.includes('/_static/134/0.109.0/scripts/app.js'), route);
      ok(result.value.includes('<script id="config" type="application/json">'), route);
    }
  });
  await check('card detail, reader, PinnedCard, markdown and object endpoints work from the database', async () => {
    const articleId = '00000002UnE0O000000003';
    const detail = await server.request(`/cards/${articleId}/detail`, { accept: 'text/html' });
    strictEqual(detail.response.status, 200);
    ok(detail.value.includes(articleId));

    const reader = await server.request(`/cards/${articleId}/reader`, { accept: 'text/html' });
    strictEqual(reader.response.status, 200);
    ok(reader.value.includes(`data-action="/objects/${articleId}/content"`));

    const pinned = await server.request(`/cards/${articleId}?template=PinnedCard`, { accept: 'text/html' });
    strictEqual(pinned.response.status, 200);
    ok(pinned.value.includes(`data-id="${articleId}"`));

    const md = await server.request(`/cards/${articleId}/markdown`, { accept: 'text/markdown' });
    strictEqual(md.response.status, 200);
    ok(md.value.length > 0);

    const obj = await server.request(`/objects/${articleId}`);
    strictEqual(obj.response.status, 200);
    strictEqual(obj.value.id, articleId);

    const tags = await server.request(`/objects/${articleId}/tags`);
    strictEqual(tags.response.status, 200);
    ok(Array.isArray(tags.value));

    const content = await server.request(`/objects/${articleId}/content`);
    strictEqual(content.response.status, 200);
    strictEqual(content.value.type, 'doc');
  });

  await check('spaces, tags, changes, quota, search and UI routes work from the database', async () => {
    const spaces = await server.request('/spaces');
    strictEqual(spaces.response.status, 200);
    ok(spaces.value.length >= 2);

    const space = await server.request(`/spaces/${spaces.value[0].id}`);
    strictEqual(space.response.status, 200);
    strictEqual(space.value.id, spaces.value[0].id);

    const manual = await server.request('/tags/manual', { accept: 'application/msgpack' });
    strictEqual(manual.response.status, 200);
    ok(manual.value[0].length > 0);

    const recent = await server.request('/tags/recent?take=10');
    strictEqual(recent.response.status, 200);
    ok(Array.isArray(recent.value));

    const changes = await server.request('/changes?since=0', { accept: 'application/msgpack' });
    strictEqual(changes.response.status, 200);
    ok(Number(changes.response.headers.get('x-count')) >= 100);

    const quota = await server.request('/quotas/objects');
    strictEqual(quota.response.status, 200);
    ok(quota.value.count >= 100);

    const account = await server.request('/account.json');
    strictEqual(account.response.status, 200);
    ok(account.value.quotas?.objects?.count >= 100);

    const search = await server.request('/search?q=type:image');
    strictEqual(search.response.status, 200);
    ok(search.value.matches.length > 5);

    const serendipity = await server.request('/serendipity', { accept: 'text/html', headers: { 'x-partial': '1' } });
    strictEqual(serendipity.response.status, 200);
    ok(serendipity.value.includes('serendipity-outro'));
  });
} finally {
  await server.close();
}

console.log(`\n${passed}/${passed + failed} read checks passed (Rust loopback transport)`);
process.exitCode = failed ? 1 : 0;
