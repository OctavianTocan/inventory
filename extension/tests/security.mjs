import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { rmSync } from 'node:fs';
import { join } from 'node:path';
import { launch } from './flows.mjs';
import { prepareExtension, scratch, startPages } from './harness.mjs';

const STUB_PORT = 4342;
const CARD = 'AAAAAAAAAAAAAAAAAAAAAA';
const BOUND_TAG = 'BBBBBBBBBBBBBBBBBBBBBB';
const OTHER = 'CCCCCCCCCCCCCCCCCCCCCC';
const SESSION = 'stub-session-value-0123456789';

/** A host that records every request and answers like the replica would for one card. */
async function startRecorder() {
  const requests = [];
  const json = (response, status, body) => {
    response.writeHead(status, { 'content-type': 'application/json' });
    response.end(JSON.stringify(body));
  };
  const server = createServer((request, response) => {
    requests.push({ method: request.method, path: request.url, auth: request.headers.authorization, cookie: request.headers.cookie });
    request.resume();
    if (request.method === 'POST' && request.url === '/objects') return json(response, 201, { id: CARD, tags: [{ id: BOUND_TAG, name: 'kept' }], note: null });
    if (request.url.startsWith('/tags/')) return json(response, 200, []);
    if (request.method === 'POST' && request.url === `/objects/${CARD}/tags`) return json(response, 201, { id: 'DDDDDDDDDDDDDDDDDDDDDD', name: 'fresh' });
    if (request.method === 'DELETE' && request.url === `/annotations/${BOUND_TAG}`) return json(response, 200, {});
    if (request.method === 'PUT' && request.url === `/objects/${CARD}/note`) return json(response, 200, {});
    return json(response, 404, { status: 404, detail: 'unexpected' });
  });
  await new Promise((resolve) => server.listen(STUB_PORT, '127.0.0.1', resolve));
  return { requests, origin: `http://localhost:${STUB_PORT}`, stop: () => server.close() };
}

const extension = prepareExtension();
const pages = await startPages(4400);
const recorder = await startRecorder();
const profile = join(scratch, 'profile-security');
rmSync(profile, { recursive: true, force: true });
const { context, worker: firstWorker } = await launch(extension, profile);

let worker = firstWorker;
try {
  assert.equal(await worker.evaluate(() => typeof chrome.declarativeNetRequest), 'undefined', 'the extension rewrites no requests');
  await worker.evaluate((host) => chrome.storage.local.set({ host }), recorder.origin);
  await context.addCookies([{ name: 'mm_session', value: SESSION, url: recorder.origin, httpOnly: true }]);
  const control = await context.newPage();
  const id = new URL(worker.url()).host;
  await control.goto(`chrome-extension://${id}/manifest.json`);
  const page = await context.newPage();
  await page.goto(`${pages.origin}/article.html`);
  const tabId = (target) => worker.evaluate((url) => chrome.tabs.query({}).then((tabs) => tabs.find((tab) => tab.url === url).id), target.url());
  const pageTab = await tabId(page);
  const say = (target, message) => target.evaluate((payload) => chrome.runtime.sendMessage(payload), message);

  await say(control, { action: 'savePageOfTab', tabId: pageTab });
  await page.locator('mymind-notification.success').waitFor({ timeout: 15000 });
  const frame = await (await page.locator('iframe.mymind-drawer-frame').elementHandle()).contentFrame();
  assert.ok(frame, 'the drawer is an extension-origin frame');
  await frame.locator('.mymind-notification-tag').first().waitFor();

  const seen = await page.evaluate(() => {
    const root = document.getElementById('mymind-shadow');
    const iframe = root.shadowRoot.querySelector('iframe');
    let crossOrigin = false;
    try {
      void iframe.contentDocument.body;
    } catch {
      crossOrigin = true;
    }
    return {
      crossOrigin,
      tagsInPage: Boolean(root.shadowRoot.querySelector('.mymind-notification-tags, .mymind-notification-tag, .mymind-note')),
      suggestion: document.body.innerHTML.includes('kept'),
      shadowText: root.shadowRoot.textContent.includes('kept'),
    };
  });
  assert.deepEqual(seen, { crossOrigin: true, tagsInPage: false, suggestion: false, shadowText: false }, 'the saved page cannot read tags, suggestions or notes');

  await page.evaluate((card) => {
    const forged = document.createElement('span');
    forged.dataset.id = `../objects/${card}`;
    forged.className = 'remove-tag-icon';
    forged.setAttribute('on-click', 'removeTag');
    document.body.append(forged);
    forged.click();
    window.postMessage({ mymindDrawer: true, type: 'close' }, '*');
    document.getElementById('mymind-shadow').shadowRoot.querySelector('iframe').contentWindow.postMessage({ mymindDrawer: true, type: 'close' }, '*');
  }, CARD);
  await page.waitForTimeout(500);
  assert.ok(await page.locator('mymind-notification').count(), 'forged messages from the page do not close or drive the toast');

  const before = recorder.requests.length;
  const asDrawer = (message) => say(frame, message);
  const forbidden = (result, label) => assert.equal(result?.error?.status, 403, `${label} is refused`);
  forbidden(await asDrawer({ action: 'deleteTag', id: `../objects/${CARD}` }), 'a traversal id');
  forbidden(await asDrawer({ action: 'deleteTag', id: OTHER }), 'an annotation the server never issued for this card');
  forbidden(await asDrawer({ action: 'deleteTag', id: `${BOUND_TAG}/../../objects/${CARD}` }), 'an issued id with a suffix');
  forbidden(await asDrawer({ action: 'deleteTag', id: [BOUND_TAG] }), 'a non-string id');
  forbidden(await asDrawer({ action: 'savePageOfTab', tabId: pageTab }), 'a test seam from the drawer');
  forbidden(await asDrawer({ action: 'saveObject', tabId: pageTab, url: 'http://localhost/' }), 'a save by tab id from the drawer');
  forbidden(await asDrawer({ action: 'openNewTab', url: 'javascript:alert(1)' }), 'openNewTab from the drawer');
  forbidden(await asDrawer({ action: 'constructor' }), 'an inherited property as an action');
  assert.equal((await asDrawer({ action: 'addTag', name: 'x'.repeat(101) })).error.status, 400, 'an oversize tag');
  assert.equal((await asDrawer({ action: 'saveObjectNote', prose: 'plain string' })).error.status, 400, 'a malformed note');
  assert.equal(recorder.requests.length, before, 'none of those reached the server');

  await worker.evaluate(() => { globalThis.restartProbe = true; });
  const browser = await context.browser().newBrowserCDPSession();
  const { targetInfos } = await browser.send('Target.getTargets');
  const stopped = targetInfos.find((target) => target.type === 'service_worker' && target.url.startsWith(`chrome-extension://${id}/`));
  assert.ok(stopped, 'the service worker target exists');
  const attached = await browser.send('Target.attachToTarget', { targetId: stopped.targetId, flatten: true });
  await browser.send('ServiceWorker.enable', {}, attached.sessionId).catch(() => {});
  await browser.send('ServiceWorker.stopAllWorkers', {}, attached.sessionId).catch(() => {});
  await browser.send('Target.closeTarget', { targetId: stopped.targetId }).catch(() => {});
  await page.waitForTimeout(1000);
  await say(control, { action: 'wake' });
  await page.waitForTimeout(500);
  const woken = context.serviceWorkers().filter((candidate) => candidate.url().startsWith(`chrome-extension://${id}/`)).at(-1);
  assert.equal(await woken.evaluate(() => globalThis.restartProbe).catch(() => undefined), undefined, 'the service worker was restarted');

  const added = await asDrawer({ action: 'addTag', name: 'fresh', objectId: `../objects/${OTHER}` });
  assert.equal(added.name, 'fresh', 'a tag is added to the bound card and the message id is ignored');
  assert.equal((await asDrawer({ action: 'deleteTag', id: BOUND_TAG })).status, 200, 'the issued annotation can be removed');
  assert.equal((await asDrawer({ action: 'saveObjectNote', card: { id: OTHER }, prose: { type: 'doc', content: [] } })).status, 200);
  assert.ok(recorder.requests.some((r) => r.method === 'PUT' && r.path === `/objects/${CARD}/note`), 'the note reached the library after the restart');

  const second = await context.newPage();
  await second.goto(`${pages.origin}/article.html`);
  await second.evaluate((src) => {
    const frame = document.createElement('iframe');
    frame.src = src;
    document.body.append(frame);
  }, `chrome-extension://${id}/drawer.html`);
  const stranger = await (await second.locator('iframe').elementHandle()).contentFrame();
  await stranger.waitForLoadState();
  forbidden(await say(stranger, { action: 'addTag', name: 'fresh' }), 'a drawer frame in a tab without a saved card');
  forbidden(await say(stranger, { action: 'drawerInit' }), 'drawerInit in a tab without a saved card');

  const top = await context.newPage();
  await top.goto(`chrome-extension://${id}/drawer.html`);
  forbidden(await say(top, { action: 'drawerInit' }), 'drawer.html opened as a top-level page');
  forbidden(await say(top, { action: 'savePageOfTab', tabId: pageTab }), 'a test seam from a top-level drawer.html');

  const expected = new Set([
    'POST /objects',
    `POST /objects/${CARD}/tags`,
    `DELETE /annotations/${BOUND_TAG}`,
    `PUT /objects/${CARD}/note`,
    'GET /tags/manual',
    'GET /tags/recent?take=5',
    'POST /browser-extension/activate',
  ]);
  for (const { method, path, auth, cookie } of recorder.requests) {
    assert.ok(expected.has(`${method} ${path}`), `unexpected request ${method} ${path}`);
    assert.ok(!/\.\.|%2e/i.test(path), 'no traversal reaches the server');
    assert.equal(auth, `Bearer ${SESSION}`);
    assert.equal(cookie, undefined, 'the browser never attaches cookies to extension requests');
  }

  await asDrawer({ action: 'getRecentTags' });
  const cache = await control.evaluate(() => chrome.storage.local.get('tagCache'));
  assert.ok(cache.tagCache.scope.startsWith(`${recorder.origin}|`), 'the cache is keyed by host and account');
  assert.ok(!JSON.stringify(cache).includes(SESSION), 'the raw session is never stored');
  await context.addCookies([{ name: 'mm_session', value: 'a-different-account', url: recorder.origin, httpOnly: true }]);
  await page.waitForTimeout(500);
  assert.equal((await control.evaluate(() => chrome.storage.local.get('tagCache'))).tagCache, undefined, 'a changed session clears the cache');
  forbidden(await asDrawer({ action: 'deleteTag', id: BOUND_TAG }), 'a card bound to the previous session');
  console.log('security tests passed');
} finally {
  await context.close().catch(() => {});
  recorder.stop();
  pages.stop();
}
