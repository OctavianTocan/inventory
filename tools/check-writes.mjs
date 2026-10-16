import { strictEqual, deepStrictEqual, ok } from 'node:assert';
import { DatabaseSync } from 'node:sqlite';
import { renderCard } from '../lib/render-card.mjs';
import { nsFromIso } from '../lib/store.mjs';
import { testServer, localize } from './http-check.mjs';

const server = await testServer();
let hub;
try { hub = await server.hub(); }
catch (error) { await server.close(); throw error; }
let passed = 0;
let failed = 0;
const doc = text => ({ type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text }] }] });
const read = async path => {
  const result = await server.request(path);
  strictEqual(result.response.status, 200, `GET ${path} status`);
  return result.value;
};
const write = async (method, path, body, status = 200, options = {}) => {
  const result = await server.request(path, { method, body, ...options });
  strictEqual(result.response.status, status, `${method} ${path} status`);
  ok(result.response.headers.has('x-object-quota'), 'write quota header');
  return result.value;
};

// Check a fresh card over HTTP, then independently render its persisted SQLite inputs.
async function card(id) {
  const result = await server.request(`/cards/${id}`, { accept: 'application/msgpack' });
  strictEqual(result.response.status, 200);
  strictEqual(result.value.length, 1);
  const db = new DatabaseSync(server.dbPath, { readOnly: true });
  let row;
  try { row = db.prepare('SELECT data, html, extras FROM cards WHERE id = ?').get(id); }
  finally { db.close(); }
  ok(row, 'card persisted');
  strictEqual(row.html, renderCard(JSON.parse(row.data), JSON.parse(row.extras)), 'persisted render');
  deepStrictEqual(result.value[0], { html: localize(row.html), json: localize(row.data) });
  const data = JSON.parse(result.value[0].json);
  deepStrictEqual(await read(`/cards/${id}`), data, 'JSON content negotiation');
  return { ...result.value[0], data };
}

// Read a checkpoint without downloading the whole grid on every mutation.
async function checkpoint() {
  const result = await server.request('/cards', { accept: 'application/msgpack' });
  return result.response.headers.get('x-checkpoint');
}

// Verify that polling describes the same newly rendered state as the card endpoint.
async function changed(since, id, type = 1, objectType = 9) {
  const result = await server.request(`/changes?since=${since}&render=true`, { accept: 'application/msgpack' });
  strictEqual(result.response.status, 200);
  strictEqual(Number(result.response.headers.get('x-count')), result.value.length);
  ok(result.response.headers.has('x-object-quota'));
  const item = result.value.filter(change => change.object.id === id && change.objectType === objectType).at(-1);
  ok(item, 'change returned');
  strictEqual(item.type, type);
  strictEqual(typeof item.timestamp, 'bigint', 'uint64 checkpoint');
  ok(item.timestamp > BigInt(since), 'strictly newer checkpoint');
  if (objectType === 9) {
    const current = await card(id);
    strictEqual(item.cardHtml, current.html);
    strictEqual(item.cardData, current.json);
    ok(item.object.modified instanceof Date, 'msgpack timestamp extension');
    strictEqual(item.object.modified.getTime(), Number(nsFromIso(current.data.modified) / 1_000_000n));
  }
  return item;
}

// Print bounded failures without disclosing entire card or settings payloads.
async function check(name, run) {
  try { await run(); console.log(`PASS  ${name}`); passed++; }
  catch (error) { console.log(`FAIL  ${name}: ${error.message.split('\n')[0]}`); failed++; }
}

let noteId, textId, urlId, draftId, stubId, spaceId, smartId;
let highlightId, tagId, shareId;
try {
  const initialQuota = await read('/quota');
  await check('create note: 201, card HTML, grid, quota, changes', async () => {
    const since = await checkpoint();
    noteId = (await write('POST', '/objects', { type: 'Note', title: 'HTTP note', prose: doc('First paragraph') }, 201)).id;
    ok(/^[0-9A-Za-z]{22}$/.test(noteId));
    const result = await card(noteId);
    strictEqual(result.data.title, 'HTTP note');
    ok(result.html.includes('First paragraph'));
    const grid = await server.request('/cards', { accept: 'application/msgpack' });
    ok(grid.value[0].html.includes(noteId), 'new note first in grid');
    strictEqual(JSON.parse(grid.response.headers.get('x-object-quota')).count, initialQuota.count + 1);
    await changed(since, noteId);
  });
  await check('create plain text: line breaks and escaped HTML', async () => {
    textId = (await write('POST', '/objects', { content: 'Line one\n\n<script>local text</script>' }, 201)).id;
    const result = await card(textId);
    strictEqual(result.data.type, 'Note');
    strictEqual(result.data.prose.content.length, 3);
    ok(result.html.includes('&lt;script&gt;'));
    ok(!result.html.includes('<script>'));
  });
  await check('URL create and duplicate bump with readable offline fallback', async () => {
    urlId = (await write('POST', '/objects', { content: 'https://example.invalid/local' }, 201)).id;
    const created = await card(urlId);
    ok(['WebPage', 'Placeholder'].includes(created.data.type));
    strictEqual(created.data.source.url, 'https://example.invalid/local');
    strictEqual(created.data.domain, 'example.invalid');
    strictEqual(created.data.flags & ~1, 0);
    const since = await checkpoint();
    const result = await write('POST', '/objects', { content: 'https://example.invalid/local' });
    strictEqual(result.id, urlId);
    await changed(since, urlId);
  });
  await check('title, description, full-model flags and entity edits', async () => {
    const before = (await card(noteId)).data;
    const since = await checkpoint();
    await write('PATCH', `/objects/${noteId}`, { title: 'Changed & escaped', description: 'Local description', flags: 0 });
    await write('PATCH', `/objects/${noteId}/entity`, { author: { name: 'Local author' } });
    const result = await card(noteId);
    strictEqual(result.data.title, 'Changed & escaped');
    strictEqual(result.data.description, 'Local description');
    strictEqual(result.data.author.name, 'Local author');
    strictEqual(result.data.bumped, before.bumped);
    ok(result.html.includes('Changed &amp; escaped'));
    await changed(since, noteId);
  });
  await check('card note save and stable note id', async () => {
    const since = await checkpoint();
    const first = await write('PUT', `/objects/${noteId}/note`, doc('Attached note'));
    const second = await write('PUT', `/objects/${noteId}/note`, doc('Edited attached note'));
    strictEqual(first.id, second.id);
    deepStrictEqual((await card(noteId)).data.note.prose, doc('Edited attached note'));
    await changed(since, noteId);
  });
  await check('prose PUT/PATCH and reader content remain synchronized', async () => {
    const since = await checkpoint();
    await write('PUT', `/objects/${noteId}/prose`, doc('Edited prose'));
    await write('PATCH', `/objects/${noteId}/prose`, doc('Patched prose'));
    ok((await card(noteId)).html.includes('Patched prose'));
    await write('PUT', `/objects/${noteId}/content`, doc('Reader edit'), 200, { type: 'application/prose+json' });
    deepStrictEqual(await read(`/objects/${noteId}/content`), doc('Reader edit'));
    ok((await card(noteId)).html.includes('Reader edit'));
    await write('PUT', `/objects/${noteId}/prose`, doc('Composer edit'));
    deepStrictEqual(await read(`/objects/${noteId}/content`), doc('Composer edit'));
    await write('PATCH', `/objects/${noteId}`, { prose: doc('Native model edit') });
    deepStrictEqual(await read(`/objects/${noteId}/content`), doc('Native model edit'));
    await changed(since, noteId);
  });
  await check('draft and stub grid visibility; draft commit', async () => {
    draftId = (await write('POST', '/objects', { type: 'Note', isDraft: true, prose: doc('Draft') }, 201)).id;
    stubId = (await write('POST', '/objects', { type: 'Note', isStub: true, title: 'Wiki idea' }, 201)).id;
    const grid = (await server.request('/cards', { accept: 'application/msgpack' })).value;
    ok(!grid.some(value => value.html.includes(draftId) || value.html.includes(stubId)));
    const since = await checkpoint();
    strictEqual((await write('POST', `/objects/${draftId}/commit`, { prose: doc('Committed note') }, 201)).id, draftId);
    ok(!((await card(draftId)).data.flags & 131072));
    ok((await server.request('/cards', { accept: 'application/msgpack' })).value.some(value => value.html.includes(draftId)));
    await changed(since, draftId);
    await write('PATCH', `/objects/${draftId}/commit`, { prose: doc('Committed again') });
    await write('PUT', `/objects/${stubId}/content`, doc('Written wiki note'), 200, { type: 'application/prose+json' });
    ok(!((await card(stubId)).data.flags & 262144));
  });
  await check('annotations add/idempotency, manual/recent/suggest/detail reads', async () => {
    const since = await checkpoint();
    const tag = await write('POST', `/objects/${noteId}/tags`, { name: 'local-http-tag' });
    tagId = tag.id;
    strictEqual(tag.flags, 8);
    strictEqual((await write('POST', `/objects/${noteId}/tags`, { name: 'local-http-tag' })).id, tagId);
    deepStrictEqual(await read(`/objects/${noteId}/tags`), [tag]);
    ok((await server.request('/tags/manual', { accept: 'application/msgpack' })).value[0].some(value => value.name === tag.name && value.count === 1));
    ok((await read('/tags/recent?take=10')).some(value => value.name === tag.name));
    ok((await read('/tags/suggest?q=local-http')).some(value => value.name === tag.name));
    ok((await read('/tags/detail?name=local-http-tag')).objects.some(value => value.id === noteId));
    await changed(since, noteId);
  });
  await check('highlight annotation changes rendered class; removal', async () => {
    highlightId = (await write('POST', `/objects/${noteId}/tags`, { name: 'highlight' })).id;
    let result = await card(noteId);
    ok(result.html.includes('note content highlighted'));
    ok((await server.request(`/cards/${noteId}/detail`, { accept: 'text/html' })).value.includes('class="body inner highlighted"'), 'detail body highlighted on reopen');
    ok(!Object.hasOwn(result.data.tags.find(tag => tag.id === highlightId), 'flags'));
    await write('DELETE', `/annotations/${highlightId}`);
    result = await card(noteId);
    ok(!result.html.includes('content highlighted'));
    ok(!(await server.request(`/cards/${noteId}/detail`, { accept: 'text/html' })).value.includes('class="body inner highlighted"'), 'detail highlight removed');
  });
  await check('annotation removal changes card and tag counts', async () => {
    const since = await checkpoint();
    await write('DELETE', `/annotations/${tagId}`);
    strictEqual((await read(`/objects/${noteId}/tags`)).length, 0);
    ok(!(await read('/tags/recent?take=100')).some(value => value.name === 'local-http-tag'));
    await changed(since, noteId);
  });
  await check('privacy annotations hide and restore cards persistently', async () => {
    const privateTag = await write('POST', `/objects/${noteId}/tags`, { name: 'private' });
    const hiddenTag = await write('POST', `/objects/${noteId}/tags`, { name: 'hidden' });
    ok((await card(noteId)).data.flags & 8);
    ok(!(await server.request('/cards', { accept: 'application/msgpack' })).value.some(value => value.html.includes(noteId)));
    await write('DELETE', `/annotations/${privateTag.id}`);
    ok((await card(noteId)).data.flags & 8);
    await write('DELETE', `/annotations/${hiddenTag.id}`);
    ok(!((await card(noteId)).data.flags & 8));
  });
  await check('spaces create, rename, recolor and list', async () => {
    const since = await checkpoint();
    const result = await write('POST', '/spaces', { name: 'Local HTTP space', color: '#a1b2c3', position: 99 });
    spaceId = result.id;
    deepStrictEqual(result.objects, []);
    await write('PATCH', `/spaces/${spaceId}`, { name: 'Renamed HTTP space', color: '#112233' });
    const space = await read(`/spaces/${spaceId}`);
    strictEqual(space.name, 'Renamed HTTP space');
    strictEqual(space.color, '#112233');
    ok((await read('/spaces')).some(value => value.id === spaceId));
    await changed(since, spaceId, 1, 190);
  });
  await check('space membership add/remove and URL bump membership', async () => {
    const since = await checkpoint();
    await write('PUT', `/spaces/${spaceId}/objects/${noteId}`, {});
    ok((await read(`/spaces/${spaceId}`)).objects.some(value => value.id === noteId));
    ok((await changed(since, noteId)).object.spaces.some(value => value.id === spaceId));
    await write('DELETE', `/spaces/${spaceId}/objects/${noteId}`);
    ok(!(await read(`/spaces/${spaceId}`)).objects.some(value => value.id === noteId));
    await write('POST', '/objects', { content: 'https://example.invalid/local', spaceId });
    ok((await read(`/spaces/${spaceId}`)).objects.some(value => value.id === urlId));
  });
  await check('smart space evaluates filters and preserves query exclusions', async () => {
    const space = await write('POST', '/spaces', { name: 'Local smart', color: '#123456',
      query: { filters: [`type:note AND title:"${(await card(noteId)).data.title}"`], matches: [] } });
    smartId = space.id;
    strictEqual(space.objects[0].queryId, space.query.id);
    strictEqual(space.objects[0].id, noteId);
    const checked = await read(`/queries/${space.query.id}/check`);
    ok(Number.isFinite(checked.elapsed));
    deepStrictEqual(checked.added, []); deepStrictEqual(checked.removed, []);
    await write('POST', `/queries/${space.query.id}/objects/${noteId}/exclude`, {});
    strictEqual((await read(`/spaces/${smartId}`)).objects.length, 0);
  });
  await check('space reorder and delete', async () => {
    await write('POST', '/spaces/reorder', [smartId, spaceId]);
    const spaces = await read('/spaces');
    ok(spaces.find(value => value.id === smartId).position < spaces.find(value => value.id === spaceId).position);
    const since = await checkpoint();
    await write('DELETE', `/spaces/${smartId}`);
    ok(!(await read('/spaces')).some(value => value.id === smartId));
    await changed(since, smartId, 2, 190);
  });
  await check('pin/unpin, completed flags and pin order', async () => {
    await write('POST', `/objects/${noteId}/pin`, {});
    ok((await card(noteId)).data.flags & 512);
    await write('POST', `/objects/${noteId}/unpin`, {});
    ok(!((await card(noteId)).data.flags & 512));
    await write('POST', `/objects/${noteId}/complete`, {});
    ok((await card(noteId)).data.flags & 65536);
    await write('POST', `/objects/${noteId}/uncomplete`, {});
    ok(!((await card(noteId)).data.flags & 65536));
    await write('POST', '/objects/reorder-pins', [textId, noteId]);
    strictEqual((await read(`/objects/${textId}`)).position, 0);
    strictEqual((await read(`/objects/${noteId}`)).position, 1);
    const pinned = await server.request(`/cards/${noteId}?template=PinnedCard`, { accept: 'text/html', headers: { 'x-partial': 'PinnedCard' } });
    strictEqual(pinned.response.status, 200);
    strictEqual(pinned.response.headers.get('x-partial'), 'PinnedCard');
    ok(pinned.value.includes('class="tom__item draggable"') && pinned.value.includes(noteId));
    const shell = (await server.request('/everything', { headers: { 'sec-fetch-dest': 'document' }, accept: 'text/html' })).value;
    const listStart = shell.indexOf('class="tom__list sortable"');
    ok(listStart >= 0);
    const list = shell.slice(listStart, shell.indexOf('</mymind-top-of-mind>', listStart));
    ok(list.indexOf(`data-id="${textId}"`) < list.indexOf(`data-id="${noteId}"`));
  });
  await check('bulk tag and add to space', async () => {
    await write('POST', '/batch/tag', { objectIds: [noteId, textId], tag: 'http-bulk' });
    ok((await read(`/objects/${textId}/tags`)).some(value => value.name === 'http-bulk'));
    await write('POST', '/batch/add-to-space', { objectIds: [noteId, textId], spaceId });
    const members = (await read(`/spaces/${spaceId}`)).objects.map(value => value.id);
    ok(members.includes(noteId) && members.includes(textId));
  });
  await check('trash, deleted rendering, restore and quotas', async () => {
    const before = await read('/quota');
    let since = await checkpoint();
    await write('DELETE', `/objects/${noteId}`);
    let result = await card(noteId);
    ok(result.data.deleted && result.html.includes('deleted') && result.html.includes('card:restore'));
    ok((await server.request('/cards/trash', { accept: 'application/msgpack' })).value.some(value => value.html.includes(noteId)));
    strictEqual((await read('/quota')).count, before.count - 1);
    await changed(since, noteId, 2);
    since = await checkpoint();
    await write('POST', `/objects/${noteId}/restore`);
    result = await card(noteId);
    ok(!result.data.deleted && result.html.includes('card:open'));
    strictEqual((await read('/quota')).count, before.count);
    await changed(since, noteId);
  });
  await check('bulk trash', async () => {
    await write('POST', '/batch/delete', { objectIds: [textId, draftId] });
    ok((await card(textId)).data.deleted && (await card(draftId)).data.deleted);
  });
  await check('revisions read and restore', async () => {
    const revisions = await read(`/objects/${noteId}/revisions`);
    ok(revisions.length >= 3 && revisions[0].isCurrent);
    const old = revisions.at(-1);
    await write('POST', `/objects/${noteId}/revisions/${old.id}/restore`, {});
    deepStrictEqual(await read(`/objects/${noteId}/content`), old.content.body);
    deepStrictEqual((await card(noteId)).data.prose, old.content.body);
  });
  await check('shares create and revoke', async () => {
    const share = await write('POST', `/cards/${noteId}/shares`, {});
    shareId = share.id;
    strictEqual(share.url, `/shares/${shareId}`);
    const page = await server.request(share.url, { headers: { 'sec-fetch-dest': 'document' }, accept: 'text/html' });
    strictEqual(page.response.status, 200);
    ok(page.value.includes(noteId));
    const detail = await server.request(`/cards/${noteId}/detail`, { accept: 'text/html' });
    strictEqual(detail.response.status, 200);
    ok(detail.value.includes(share.url));
    await write('DELETE', `/cards/${noteId}/shares/${shareId}`);
    strictEqual((await server.request(share.url, { accept: 'text/html' })).response.status, 410);
  });
  await check('new and edited details contain current editor actions and values', async () => {
    const detail = await server.request(`/cards/${noteId}/detail`, { accept: 'text/html', headers: { 'x-partial': 'CardDetail' } });
    strictEqual(detail.response.status, 200);
    ok(detail.value.includes('Changed &amp; escaped'));
    ok(detail.value.includes(`/objects/${noteId}/prose`) && detail.value.includes(`/objects/${noteId}/note`));
    ok(detail.value.includes('Edited attached note'));
    ok(detail.value.includes('http-bulk'));
  });
  await check('article content edits retain detail editor and revisions', async () => {
    const db = new DatabaseSync(server.dbPath, { readOnly: true });
    let id;
    try { id = db.prepare("SELECT id FROM cards WHERE type = 'Article' AND deleted_ns IS NULL LIMIT 1").get().id; }
    finally { db.close(); }
    await write('PUT', `/objects/${id}/content`, doc('Locally edited article'), 200, { type: 'application/prose+json' });
    const detail = await server.request(`/cards/${id}/detail`, { accept: 'text/html' });
    strictEqual(detail.response.status, 200);
    ok(detail.value.includes('data-type="card"') && detail.value.includes(`/objects/${id}/content`));
    ok(detail.value.includes('Locally edited article'));
    const revisions = await read(`/objects/${id}/revisions`);
    ok(revisions.length >= 1);
    await write('POST', `/objects/${id}/revisions/${revisions[0].id}/restore`, {});
    deepStrictEqual(await read(`/objects/${id}/content`), revisions[0].content.body);
  });
  await check('shared media uses eager image attributes', async () => {
    const db = new DatabaseSync(server.dbPath, { readOnly: true });
    let id;
    try { id = db.prepare("SELECT id FROM cards WHERE type = 'Image' AND deleted_ns IS NULL LIMIT 1").get().id; }
    finally { db.close(); }
    const share = await write('POST', `/cards/${id}/shares`, {});
    const page = await server.request(share.url, { accept: 'text/html' });
    strictEqual(page.response.status, 200);
    ok(/<img\b[^>]* src="/.test(page.value));
    ok(!/<img\b[^>]* data-src=/.test(page.value));
    await write('DELETE', `/cards/${id}/shares/${share.id}`);
  });
  await check('remove thumbnail without AI', async () => {
    const since = await checkpoint();
    await write('POST', `/cards/${urlId}/remove-ai-thumbnail`, {});
    await changed(since, urlId);
  });
  await check('markdown conversion and editor document nodes', async () => {
    const result = await write('POST', '/convert', '# Heading\n**bold**\n- item\n> quote\n```\ncode\n```', 200, { type: 'text/markdown', accept: 'application/prose+json' });
    strictEqual(result.type, 'doc');
    strictEqual(result.content[0].type, 'heading');
    const rich = { type: 'doc', content: [...result.content,
      { type: 'taskList', content: [{ type: 'taskItem', attrs: { checked: true }, content: [doc('task').content[0]] }] },
      { type: 'table', content: [{ type: 'tableRow', content: [{ type: 'tableCell', content: [doc('cell').content[0]] }] }] },
      { type: 'paragraph', content: [{ type: 'wikiLink', attrs: { target: 'Local idea', text: 'Local idea', targetId: stubId } },
        { type: 'text', text: 'marked', marks: [{ type: 'highlight' }, { type: 'subscript' }, { type: 'superscript' }] }] }] };
    await write('PUT', `/objects/${noteId}/prose`, rich);
    const rendered = await card(noteId);
    ok(rendered.html.includes('wiki-link') && rendered.html.includes('<table>') && rendered.html.includes('data-checked="true"'));
    ok(rendered.html.includes('<mark>'));
  });
  await check('settings PATCH and onboarding echoed in next shell', async () => {
    const db = new DatabaseSync(server.dbPath, { readOnly: true });
    let uid;
    try { uid = db.prepare('SELECT id FROM users LIMIT 1').get().id; } finally { db.close(); }
    const result = await write('PATCH', `/users/${uid}`, { colorScheme: 'dark', locale: 'en-GB', timeZoneId: 'Europe/London', soundEnabled: false });
    deepStrictEqual(result, { status: 200 });
    await write('POST', '/onboard/flags/4096/set');
    let shell = (await server.request('/everything', { headers: { 'sec-fetch-dest': 'document' }, accept: 'text/html' })).value;
    let config = JSON.parse(shell.match(/<script id="config" type="application\/json">([\s\S]*?)<\/script>/)[1]);
    strictEqual(config.colorScheme, 'dark'); strictEqual(config.locale, 'en-GB');
    strictEqual(config.timeZoneId, 'Europe/London'); strictEqual(config.soundEnabled, false);
    strictEqual((await read('/account.json')).quotas.objects.count, (await read('/quota')).count);
    ok(config.onboardFlags & 4096);
    await write('POST', '/onboard/flags/4096/unset');
    shell = (await server.request('/everything', { headers: { 'sec-fetch-dest': 'document' }, accept: 'text/html' })).value;
    config = JSON.parse(shell.match(/<script id="config" type="application\/json">([\s\S]*?)<\/script>/)[1]);
    ok(!(config.onboardFlags & 4096));
  });
  await check('one final change per card prevents delete and restore races', async () => {
    const since = await checkpoint();
    const id = (await write('POST', '/objects', { content: 'Short-lived note' }, 201)).id;
    await write('PATCH', `/objects/${id}`, { title: 'Edited before delete' });
    await write('DELETE', `/objects/${id}`);
    await write('POST', `/objects/${id}/tags`, { name: 'stale-tab-edit' });
    const changes = (await server.request(`/changes?since=${since}&render=true`, { accept: 'application/msgpack' })).value;
    const items = changes.filter(item => item.object.id === id);
    strictEqual(items.length, 1);
    strictEqual(items[0].type, 2);
    await write('POST', `/objects/${id}/restore`);
    const restored = (await server.request(`/changes?since=${since}&render=true`, { accept: 'application/msgpack' })).value.filter(item => item.object.id === id);
    strictEqual(restored.length, 1);
    strictEqual(restored[0].type, 1);
    const beforeDraft = await checkpoint();
    const hidden = (await write('POST', '/objects', { type: 'Note', isDraft: true, prose: doc('Other tab draft') }, 201)).id;
    strictEqual((await changed(beforeDraft, hidden, 2)).type, 2);
  });
  await check('local completion and dismissal acknowledgments', async () => {
    await write('POST', '/serendipity/complete', {});
    await write('POST', '/promotions/local-promotion/dismiss', {});
    deepStrictEqual(await read('/achievements/local-achievement?dismiss=true'), {});
    await write('POST', '/jserrors', { message: 'local check' });
    const db = new DatabaseSync(server.dbPath, { readOnly: true });
    try {
      const settings = JSON.parse(db.prepare('SELECT settings FROM users LIMIT 1').get().settings);
      ok(settings.completedFlows.includes('serendipity'));
      ok(settings.dismissedPromotions.includes('local-promotion'));
      ok(settings.dismissedAchievements.includes('local-achievement'));
    } finally { db.close(); }
  });
  await check('invalid writes fail and bulk operations roll back', async () => {
    const since = await checkpoint();
    const before = (await card(noteId)).json;
    const bad = await server.request('/batch/tag', { method: 'POST', body: { objectIds: [noteId, 'missing'], tag: 'must-roll-back' } });
    strictEqual(bad.response.status, 404);
    strictEqual(bad.response.headers.get('content-type'), 'application/problem+json');
    strictEqual((await card(noteId)).json, before);
    strictEqual(await checkpoint(), since);
    const invalid = await server.request('/objects', { method: 'POST', body: '{' });
    strictEqual(invalid.response.status, 400);
    const media = await server.request('/objects', { method: 'POST', body: { type: 'Media', uploadId: 'pending' } });
    strictEqual(media.response.status, 404);
  });
  await check('SignalR committed events and payloads reach connected hub', async () => {
    for (let i = 0; i < 100 && !hub.messages.some(message => message.target === 'account:updated'); i++) {
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    for (const target of ['object:created', 'object:updated', 'object:bumped', 'object:deleted', 'tag:created', 'tag:deleted', 'space:created', 'space:updated', 'space:deleted', 'account:updated']) {
      ok(hub.messages.some(message => message.type === 1 && message.target === target && message.arguments.length === 1), target);
    }
    ok(hub.messages.some(message => message.target === 'tag:created' && message.arguments[0].objectId === noteId && message.arguments[0].flags === 8));
    ok(!hub.messages.some(message => message.arguments?.[0]?.name === 'must-roll-back'), 'rollback emitted no hub event');
  });
  await check('STORE_READONLY acknowledges writes and preserves every stored row', async () => {
    const readonly = await testServer({ readonly: true, source: server.dbPath });
    const socket = await readonly.hub();
    const snapshot = () => {
      const db = new DatabaseSync(readonly.dbPath, { readOnly: true });
      try {
        const tables = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name").all();
        return tables.map(({ name }) => {
          const columns = db.prepare(`PRAGMA table_info("${name}")`).all();
          const statement = db.prepare(`SELECT * FROM "${name}" ORDER BY ${columns.map(c => '"' + c.name + '"').join(',')}`);
          statement.setReadBigInts(true);
          return [name, statement.all()];
        });
      } finally { db.close(); }
    };
    try {
      const before = snapshot();
      for (const [method, path, body] of [
        ['POST', '/objects', { content: 'Ignored note' }], ['PATCH', `/objects/${noteId}`, { title: 'Ignored' }],
        ['DELETE', `/objects/${noteId}`], ['POST', `/objects/${noteId}/tags`, { name: 'ignored' }],
        ['POST', '/spaces', { name: 'Ignored' }], ['POST', '/onboard/flags/4096/set'],
      ]) ok((await readonly.request(path, { method, body })).response.ok);
      deepStrictEqual(snapshot(), before);
      ok(!socket.messages.some(message => message.type === 1));
      const grid = await readonly.request('/cards', { accept: 'application/msgpack' });
      ok(grid.value.some(value => value.html.includes(noteId)));
      const hydrate = await readonly.request('/cards/grid', { method: 'POST', body: [noteId], accept: 'application/msgpack' });
      strictEqual(hydrate.value.length, 1);
    } finally { socket.close(); await readonly.close(); }
  });
} finally {
  hub.close();
  await server.close();
}
console.log(`\n${passed}/${passed + failed} write checks passed${process.argv.includes('--in-process') ? ' (in-process transport)' : ''}`);
process.exitCode = failed ? 1 : 0;
