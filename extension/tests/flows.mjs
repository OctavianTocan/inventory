import { mkdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from 'playwright-core';
import { freshLibrary, prepareExtension, repo, signIn, startPages, startReplica, startStub } from './harness.mjs';

const REPLICA_PORT = 4341;
const PAGES_PORT = 4400;
const CDP_PORT = 9333;

/**
 * Starts one task-owned Chromium with an extension loaded.
 * @param {string} extension - Unpacked extension folder.
 * @param {string} profile - User data directory.
 * @returns {Promise<{context: import('playwright-core').BrowserContext, worker: import('playwright-core').Worker}>} The browser context and the extension's service worker.
 */
export async function launch(extension, profile) {
  const context = await chromium.launchPersistentContext(profile, {
    executablePath: '/usr/bin/chromium',
    headless: false,
    viewport: { width: 1280, height: 800 },
    deviceScaleFactor: 1,
    args: [
      '--headless=new',
      '--no-sandbox',
      '--host-resolver-rules=MAP access.mymind.com 0.0.0.0',
      `--remote-debugging-port=${CDP_PORT}`,
      `--disable-extensions-except=${extension}`,
      `--load-extension=${extension}`,
    ],
  });
  const worker = context.serviceWorkers()[0] ?? (await context.waitForEvent('serviceworker'));
  return { context, worker };
}

async function toast(page, selector = 'mymind-notification') {
  const locator = page.locator(selector).first();
  await locator.waitFor({ state: 'attached', timeout: 15000 });
  return locator;
}

async function closeExtras(context, keep) {
  for (const page of context.pages()) if (!keep.includes(page)) await page.close().catch(() => {});
}

/**
 * Runs the save flows against an extension and screenshots the in-page UI.
 * @param {{label: string, extension: string, outDir: string, setup?: (worker: import('playwright-core').Worker, origin: string) => Promise<void>}} options - Which extension to drive and where screenshots go; `setup` points it at the replica.
 * @returns {Promise<Record<string, any>>} What the replica held after each flow.
 */
export async function runFlows({ label, extension, outDir, setup }) {
  mkdirSync(outDir, { recursive: true });
  const shot = (page, name) => page.screenshot({ path: join(outDir, `${label}-${name}.png`), animations: 'disabled', caret: 'hide' });
  const pages = await startPages(PAGES_PORT);
  const replica = await startReplica(REPLICA_PORT, freshLibrary(label));
  const profile = join(repo, 'evidence', 'extension', `profile-${label}`);
  rmSync(profile, { recursive: true, force: true });
  const { context, worker } = await launch(extension, profile);
  const facts = {};
  try {
    if (setup) await setup(worker, replica.origin);
    const session = await signIn(replica.origin);
    await context.addCookies([{ name: 'mm_session', value: session, url: replica.origin, httpOnly: true }]);
    const control = await context.newPage();
    const id = new URL(worker.url()).host;
    await control.goto(`chrome-extension://${id}/manifest.json`);
    const page = await context.newPage();
    await page.goto(`${pages.origin}/article.html`);
    await page.bringToFront();
    await closeExtras(context, [page, control]);
    const tabId = () => worker.evaluate((url) => chrome.tabs.query({}).then((tabs) => tabs.find((tab) => tab.url === url).id), page.url());
    const send = async (message) => control.evaluate((payload) => chrome.runtime.sendMessage(payload), message);
    const api = async (path) => (await fetch(replica.origin + path, { headers: { Authorization: `Bearer ${session}`, Accept: 'application/json' } })).json();

    const ui = page.frameLocator('iframe.mymind-drawer-frame');
    const savePageRequest = send({ action: 'savePageOfTab', tabId: await tabId() });
    await toast(page);
    await page.waitForTimeout(150);
    await shot(page, '1-saving');
    await savePageRequest;
    await page.locator('mymind-notification.success').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '2-saved');

    await page.mouse.move(1075, 75);
    await page.waitForTimeout(600);
    await shot(page, '3-hover');

    const input = ui.locator('.mymind-notification-tags input');
    await input.click();
    await input.pressSequentially('de', { delay: 40 });
    await page.waitForTimeout(500);
    await shot(page, '4-tag-suggestions');
    await input.press('Enter');
    await page.waitForTimeout(600);
    await shot(page, '5-tag-added');
    await input.pressSequentially('zebra', { delay: 40 });
    await input.press('Enter');
    await page.waitForTimeout(600);
    await shot(page, '6-second-tag');
    await ui.locator('.mymind-notification-tag .remove-tag-icon').last().click();
    await page.waitForTimeout(500);
    await shot(page, '7-tag-removed');

    await ui.locator('.mymind-note .ProseMirror').click();
    await page.keyboard.type('A note written in the toast.', { delay: 30 });
    await page.waitForTimeout(400);
    await shot(page, '8-note-typed');
    await page.waitForTimeout(1800);
    await shot(page, '9-note-saving');
    await page.waitForTimeout(1500);

    const { matches } = await api('/search?q=localhost');
    for (const { id } of matches.slice(0, 6)) {
      const object = await api(`/objects/${id}`);
      if (object.source?.url === page.url() && !facts.page) facts.page = { id, keys: Object.keys(object), tags: object.tags, note: object.note, prose: object.prose };
    }

    await page.keyboard.press('Escape');
    await page.waitForTimeout(800);
    await page.mouse.move(300, 500);
    await page.waitForTimeout(1000);

    await send({ action: 'saveObject', tabId: await tabId(), media: { url: `${pages.origin}/cat.png`, type: 'image' }, page: { url: page.url() } });
    await page.locator('mymind-notification.success').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '10-image-saved');
    await page.mouse.move(300, 500);
    await page.waitForTimeout(5500);

    await page.evaluate(() => {
      const range = document.createRange();
      range.selectNodeContents(document.getElementById('para'));
      getSelection().removeAllRanges();
      getSelection().addRange(range);
    });
    const selectionText = await page.evaluate(() => getSelection().toString());
    await send({ action: 'saveSelectionOfTab', tabId: await tabId(), context: { selectionText, pageUrl: page.url() } });
    await page.locator('mymind-notification.success').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '11-selection-saved');
    await page.mouse.move(300, 500);
    await page.waitForTimeout(5500);

    await send({ action: 'saveObject', tabId: await tabId(), media: { url: 'blob:http://localhost/x', type: 'video' }, page: { url: page.url() } });
    await page.locator('mymind-notification.error').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '12-blob-error');
    await page.waitForTimeout(5500);

    await context.addCookies([{ name: 'mm_session', value: 'not-a-real-session', url: replica.origin, httpOnly: true }]);
    await send({ action: 'savePageOfTab', tabId: await tabId() });
    await page.locator('mymind-notification.error').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '13-invalid-session');
    await page.waitForTimeout(5500);

    await context.clearCookies({ name: 'mm_session' });
    await page.locator('mymind-notification').first().waitFor({ timeout: 15000 });
    await page.locator('mymind-notification.error').waitFor({ timeout: 15000 });
    await page.waitForTimeout(700);
    await shot(page, '14-signed-out');
    await page.waitForTimeout(5500);

    replica.stop();
    await new Promise((resolve) => setTimeout(resolve, 500));
    await context.addCookies([{ name: 'mm_session', value: 'any', url: replica.origin, httpOnly: true }]);
    const failures = [
      ['15-server-500', { status: 500, body: { type: 'about:blank', title: 'Server error', status: 500, detail: 'The library is on fire.' } }],
      ['16-forbidden-403', { status: 403, body: { status: 403, detail: 'no' } }],
      ['17-upgrade-402', { status: 402, body: { status: 402, detail: 'Payment Required' } }],
    ];
    for (const [name, reply] of failures) {
      const stub = await startStub(REPLICA_PORT, reply);
      await send({ action: 'savePageOfTab', tabId: await tabId() });
      await page.locator('mymind-notification').first().waitFor({ timeout: 15000 });
      await page.waitForTimeout(1500);
      await shot(page, name);
      stub.stop();
      await page.waitForTimeout(5500);
    }
    return facts;
  } finally {
    await context.close().catch(() => {});
    replica.stop();
    pages.stop();
  }
}

if (process.argv[1] === new URL(import.meta.url).pathname) {
  const [label, hostSetup] = process.argv.slice(2);
  const extension = prepareExtension();
  const out = join(repo, 'evidence', 'extension', 'shots');
  const setup = hostSetup === 'host'
    ? async (worker, origin) => {
        await worker.evaluate((host) => chrome.storage.local.set({ host }), origin);
        await new Promise((resolve) => setTimeout(resolve, 800));
      }
    : undefined;
  console.log(JSON.stringify(await runFlows({ label, extension, outDir: out, setup })));
}
