import { strictEqual, deepStrictEqual, ok } from 'node:assert';
import { join, dirname } from 'node:path';
import { openStore } from '../lib/store.mjs';
import { createWriter } from '../lib/store-write.mjs';
import { createMedia } from '../lib/local-media.mjs';
import { createEnrichment } from '../lib/enrichment.mjs';
import { captureTitle, profileImage, plainText, sanitizePost } from '../lib/capture-html.mjs';
import { curateTags, tagKey } from '../lib/ai-tags.mjs';
import { analyze, describe, pickTags } from '../lib/ai.mjs';
import { testServer } from './http-check.mjs';

// Build synthetic reader content without using any captured prose.
function doc(text) { return { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text }] }] }; }

/**
 * Run capture-quality and Article edit/AI regressions against isolated offline fixtures.
 * @param {object} context Parent checker operations and synthetic media/server handles.
 * @param {Function} context.check Assertion runner.
 * @param {object} context.server Isolated HTTP server.
 * @param {object} context.fixtures Local fixture handlers and AI stubs.
 * @param {object} context.mediaBytes Synthetic image/video/PDF byte buffers.
 * @param {Function} context.save Save a fixture URL.
 * @param {Function} context.captured Await background capture completion.
 * @param {Function} context.row Read a test database card.
 * @param {Function} context.until Await a bounded observable transition.
 * @param {boolean} context.inProcess Use memory transports when true.
 * @returns {Promise<void>} Every quality regression has completed and temporary state is restored.
 */
export async function checkCaptureQuality({ check, server, fixtures, mediaBytes, save, captured, row, until, inProcess }) {
  await check('B1: repo/app names and matching site suffixes produce recorded-style titles', async () => {
    const cases = [
      ['GitHub - owner/yap: Description', 'Repository', 'https://github.com/owner/yap', 'GitHub', 'yap'],
      ['GitHub - owner/transitions.dev: Description', 'Repository', 'https://github.com/owner/transitions.dev', 'GitHub', 'transitions.dev'],
      ['App Name - Apps on Google Play', 'SoftwareApplication', 'https://play.google.com/store/apps/details?id=test', '', 'App Name'],
      ['App Name on the App Store', 'SoftwareApplication', 'https://apps.apple.com/app/id1', '', 'App Name'],
      ['Mind map - Wikipedia', 'Article', 'https://en.wikipedia.org/wiki/Mind_map', '', 'Mind map'],
      ['Topic | Journal', 'Article', 'https://journal.invalid/topic', 'Journal', 'Topic'],
      ['Topic — Journal', 'WebPage', 'https://journal.invalid/topic', 'Journal', 'Topic'],
      ['Topic - A related subject', 'WebPage', 'https://journal.invalid/topic', 'Journal', 'Topic - A related subject'],
    ];
    for (const [title, type, canonical, siteName, expected] of cases) strictEqual(captureTitle(title, type, { canonical, siteName, entities: [] }), expected);
  });

  let postId, articleId;
  await check('B2: X oEmbed keeps post text, decodes punctuation, removes attribution and rejects avatars', async () => {
    postId = await save('XQuality');
    const item = await captured(postId);
    strictEqual(item.data.type, 'XPost');
    strictEqual(item.data.author, 'by Jack Example');
    ok(item.extras.post.bodyHtml.includes('&#x2014; hello &amp; welcome'));
    ok(!/March 21|@jack|&amp;mdash;|&mdash;/.test(item.extras.post.bodyHtml));
    strictEqual(item.extras.captureText, 'just setting up my twttr — hello & welcome');
    strictEqual(plainText(sanitizePost('<p>Caf&eacute; &mdash; &copy; &#x110000;</p>', fixtures.base)), 'Café — © �');
    strictEqual(plainText('&constructor;'), '&constructor;');
    ok(!item.extras.cover); deepStrictEqual(item.extras.post.gallery, []);
    ok(!fixtures.requests.some(request => request.path.includes('profile_images')));
    for (const url of ['https://pbs.twimg.com/profile_images/1/photo.jpg', 'https://cdn.invalid/profile-pictures/1.jpg', 'https://avatars.invalid/u/1', 'https://lh3.googleusercontent.com/a/avatar', 'https://cdn.invalid/v/t51.2885-19/1.jpg']) ok(profileImage(url));
    ok(!profileImage('https://pbs.twimg.com/media/post-image.jpg'));
  });

  await check('B3/B5: capture stores main paragraphs and description, pending grid/detail never show failure', async () => {
    const before = fixtures.requests.length;
    articleId = await save('ArticleQuality');
    if (inProcess) {
      const pending = row(articleId);
      ok(pending.html.includes('web-page processing'));
      ok(pending.html.includes('<div class="processing-guts"><div class="spinner"></div><div class="text">127.0.0.1</div>'));
      ok(!/inaccessible-guts|blocking us/.test(pending.html));
      strictEqual(fixtures.requests.length, before);
      const detail = await server.request(`/cards/${articleId}/detail`, { accept: 'text/html' });
      ok(detail.value.includes('<mymind-tldr class="processing"><div class="processing-content">'));
    }
    const item = await captured(articleId);
    strictEqual(item.data.title, 'Mind map');
    ok(item.extras.captureText.includes('A mind map organizes ideas visually.'));
    ok(item.extras.captureText.includes('Mind maps connect concepts using branches'));
    ok(item.extras.captureText.includes('Use central topics and connections to plan a project.'));
    ok(!item.extras.captureText.includes('Unrelated navigation'));
    ok(!item.html.includes('processing-guts'));
    const failure = await captured(await save('blocked?quality=1'));
    ok(failure.html.includes('blocking us')); ok(!failure.html.includes('processing-guts'));
  });

  await check('B4: normalized vocabulary, plurals, type-only tags, manual duplicates and ten-tag limit', async () => {
    const tags = ['Design-Systems', 'design_system', ' DESIGN   SYSTEMS ', 'cats', 'cat'].map(name => ({ name }));
    deepStrictEqual(curateTags(tags, 'Article', ['Design Systems', 'Cat']).map(tag => tag.name), ['Design Systems', 'Cat']);
    strictEqual(curateTags(tags, 'Article', [], ['design systems', 'cat']).length, 0);
    for (const [type, redundant] of Object.entries({ YouTubeVideo: ['video', 'videos'], Video: ['videos'], Document: ['pdf', 'document', 'files'],
      WebPage: ['web_page', 'webpage', 'web-pages'], SoftwareApplication: ['app', 'applications', 'mobile-apps'],
      XPost: ['tweets', 'posts'], ThreadsPost: ['post'], SubstackNote: ['post'] })) {
      deepStrictEqual(curateTags([...redundant, 'screenshot', 'architecture'].map(name => ({ name })), type).map(tag => tag.name), ['architecture']);
    }
    deepStrictEqual(curateTags([{ name: 'screenshots' }], 'Screenshot').map(tag => tag.name), ['screenshots']);
    strictEqual(curateTags(Array.from({ length: 15 }, (_, i) => ({ name: `topic ${i}` })), 'Article').length, 10);
    const store = openStore(server.dbPath), writer = createWriter(store, () => {});
    try {
      const id = writer.run(() => writer.create({ type: 'Note', content: 'Tag fixture' })).body.id;
      writer.run(() => writer.addTag(id, 'manual-system'));
      writer.run(() => writer.analysis(id, { summary: 'Fixture', embedding: new Float32Array(768), tags: [{ name: 'manual_systems' }, ...Array.from({ length: 14 }, (_, i) => ({ name: `fixture ${i}` }))] }));
      const stored = store.cardTags(id);
      strictEqual(stored.filter(tag => tagKey(tag.name) === 'manual system').length, 1);
      strictEqual(stored.find(tag => tagKey(tag.name) === 'manual system').flags, 8);
      strictEqual(stored.filter(tag => tag.flags === 2).length, 10);
    } finally { store.close(); }
  });

  Object.assign(process.env, { GEMINI_API_KEY: 'fixture-key', TYPESAFE_API_KEY: 'fixture-typesafe-key', GEMINI_BASE_URL: fixtures.base, AI_CHAT_BASE_URL: fixtures.base, TYPESAFE_BASE_URL: fixtures.base });
  await check('B3: long text controls summaries/tags; images inform only visual or text-sparse cards', async () => {
    const item = { type: 'Article', title: 'Mind map', text: row(articleId).extras.captureText, image: { mime: 'image/png', data: mediaBytes.png } };
    let start = fixtures.requests.length;
    await describe(item); await pickTags(item, ['design']);
    const requests = fixtures.requests.slice(start);
    const chat = requests.find(request => request.path === '/chat/completions').body;
    ok(chat.messages[0].content.includes('derive the summary and tags from the supplied text'));
    ok(chat.messages[1].content.some(part => part.text?.includes('Mind maps connect concepts')));
    ok(!chat.messages[1].content.some(part => part.type === 'image_url'));
    ok(!requests.find(request => request.path === '/v1/systemone').body.assets);
    for (const extra of [{ type: 'Image' }, { type: 'Screenshot' }, { type: 'Video' }, { text: 'Short caption' }]) {
      start = fixtures.requests.length;
      await describe({ ...item, ...extra });
      ok(fixtures.requests.slice(start).find(request => request.path === '/chat/completions').body.messages[1].content.some(part => part.type === 'image_url'));
    }
    delete process.env.TYPESAFE_API_KEY;
    fixtures.control.tags = ['Design-Systems', 'design_system', 'DESIGN SYSTEMS'];
    try {
      const result = await analyze({ type: 'Article', title: 'Fixture', text: 'A fixture on interface systems' }, { vocabulary: ['Design Systems'] });
      deepStrictEqual(result.tags.map(tag => tag.name), ['Design Systems']);
      strictEqual(result.title.split(' ').length, 8);
    } finally { fixtures.control.tags = null; process.env.TYPESAFE_API_KEY = 'fixture-typesafe-key'; }
  });

  await check('B1/A6: AI headlines respect user edits and Article analysis reruns on effective content edits', async () => {
    const store = openStore(server.dbPath), writer = createWriter(store, () => {}), media = createMedia(join(dirname(server.dbPath), 'media'));
    const jobs = createEnrichment({ store, writer, media, offline: true });
    try {
      jobs.schedule(postId); await jobs.drain();
      strictEqual(row(postId).data.title, 'Upgrade Codex Memory Models for Free Now Today');
      writer.run(() => writer.patch(postId, { title: 'My chosen headline' }));
      jobs.schedule(postId); await jobs.drain();
      strictEqual(row(postId).data.title, 'My chosen headline');
      fixtures.control.chatDelay = 100;
      jobs.schedule(postId);
      await until(() => row(postId), item => item.data.flags & 1);
      writer.run(() => writer.patch(postId, { title: 'Edited during the pending analysis' }));
      await jobs.drain();
      strictEqual(row(postId).data.title, 'Edited during the pending analysis');
      fixtures.control.chatDelay = 0;
      const ownTitle = writer.run(() => writer.create({ content: `${fixtures.base}/XQuality?own-title=1`, title: 'Title chosen before capture' })).body.id;
      jobs.schedule(ownTitle, true); await jobs.drain();
      strictEqual(row(ownTitle).data.title, 'Title chosen before capture');
      writer.run(() => writer.prose(articleId, doc('First edited reader content about software architecture. '.repeat(6)), 'content'));
      fixtures.control.chatDelay = 100;
      const start = fixtures.requests.length;
      jobs.schedule(articleId);
      await until(() => fixtures.requests.slice(start), requests => requests.some(request => request.path === '/chat/completions'));
      const detail = await server.request(`/cards/${articleId}/detail`, { accept: 'text/html' });
      ok(detail.value.includes('<mymind-tldr class="processing">'));
      writer.run(() => writer.prose(articleId, doc('Latest edited reader content about databases. '.repeat(6)), 'content'));
      await jobs.drain();
      const chats = fixtures.requests.slice(start).filter(request => request.path === '/chat/completions');
      ok(chats.length >= 2, 'stale reader-content analysis reruns');
      const latest = chats.at(-1).body.messages[1].content[0].text;
      ok(latest.includes('Latest edited reader content about databases.'));
      ok(!latest.includes('First edited reader content') && !latest.includes('Mind maps connect'));
      strictEqual(row(articleId).data.title, 'Mind map', 'Article title is never replaced by an AI headline');
    } finally { fixtures.control.chatDelay = 0; jobs.close(); store.close(); }
  });

  await check('A6: Article content HTTP edits schedule fresh analysis', async () => {
    const aiServer = inProcess ? server : await testServer();
    fixtures.allowOrigin(aiServer.base);
    const hub = await aiServer.hub();
    try {
      const saved = await aiServer.request('/objects', { method: 'POST', body: { content: `${fixtures.base}/ArticleQuality?api-content=1` } });
      const id = saved.value.id;
      await until(() => hub.messages, messages => messages.some(message => message.target === 'object:analyzed' && message.arguments[0].id === id));
      const before = hub.messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length;
      const result = await aiServer.request(`/objects/${id}/content`, { method: 'PUT', type: 'application/prose+json', body: doc('Edited via the Article content HTTP route. '.repeat(6)) });
      strictEqual(result.response.status, 200);
      await until(() => hub.messages, messages => messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length > before);
      const latest = fixtures.requests.filter(request => request.path === '/chat/completions').at(-1).body.messages[1].content[0].text;
      ok(latest.includes('Edited via the Article content HTTP route.'));
      const revisions = (await aiServer.request(`/objects/${id}/revisions`)).value;
      const analyzed = hub.messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length;
      strictEqual((await aiServer.request(`/objects/${id}/revisions/${revisions.at(-1).id}/restore`, { method: 'POST' })).response.status, 200);
      await until(() => hub.messages, messages => messages.filter(message => message.target === 'object:analyzed' && message.arguments[0].id === id).length > analyzed);
      ok(fixtures.requests.filter(request => request.path === '/chat/completions').at(-1).body.messages[1].content[0].text.includes('Mind maps connect concepts'));
    } finally { hub.close(); if (aiServer !== server) await aiServer.close(); }
  });
  delete process.env.GEMINI_API_KEY;
  delete process.env.TYPESAFE_API_KEY;
}
