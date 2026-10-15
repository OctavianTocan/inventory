import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { launchVerifier } from './verifier-browser.mjs';

// Drives a running replica (writes enabled) through the real web UI and reports which SQLite-backed writes work.
// Usage: UI_BASE=http://localhost:4400 SERVER_LOG=/path/to/server.log VERIFIER_PORT=9340 node tools/try-writes-ui.mjs

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BASE = (process.env.UI_BASE ?? 'http://localhost:4400').replace(/\/$/, '');
const ORIGIN = new URL(BASE).origin;
const SERVER_LOG = process.env.SERVER_LOG;
const OUT = path.join(root, 'evidence/ui-writes');
const stamp = Date.now().toString(36);
const NOTE = `UI write test note ${stamp}`;
const TITLE = `UI test title ${stamp}`;
const TAG = `uitest-${stamp}`;
const PASTED_URL = `https://example.com/ui-write-test-${stamp}`;

fs.mkdirSync(OUT, { recursive: true });
for (const file of fs.readdirSync(OUT)) if (/^\d\d-.*\.png$/.test(file)) fs.rmSync(path.join(OUT, file));

const tmpProfiles = () => new Set(fs.readdirSync(os.tmpdir()).filter(name => name.startsWith('mymind-verifier-')));
const profilesBefore = tmpProfiles();
const logOffset = SERVER_LOG && fs.existsSync(SERVER_LOG) ? fs.statSync(SERVER_LOG).size : 0;

const results = [];
let current = null;
let shotCount = 0;

/** Starts one reported step; events from the page are attributed to it until the next call. */
function begin(name) {
  current = { name, checks: [], notes: [], shots: [], writes: [], console: [], failedRequests: [], http: [], abortedMedia: 0, error: null };
  results.push(current);
  return current;
}

const check = (label, pass, detail = '') => current.checks.push({ label, pass: Boolean(pass), detail: String(detail) });
const note = text => current.notes.push(text);

async function shot(page, label) {
  const file = `${String(++shotCount).padStart(2, '0')}-${label}.png`;
  await page.screenshot({ path: path.join(OUT, file) });
  current.shots.push(file);
  return file;
}

function attach(page) {
  page.on('console', message => {
    if (message.type() === 'error') current?.console.push(message.text().slice(0, 240));
  });
  page.on('pageerror', error => current?.console.push(`pageerror: ${error.message.slice(0, 240)}`));
  page.on('requestfailed', request => {
    if (!current) return;
    if (/\.mp4(\?|$)/.test(request.url())) { current.abortedMedia++; return; }
    current.failedRequests.push(`${request.method()} ${new URL(request.url()).pathname} ${request.failure()?.errorText}`);
  });
  page.on('response', async response => {
    if (!current) return;
    const request = response.request();
    const url = new URL(response.url());
    if (url.origin !== ORIGIN) return;
    if (request.method() !== 'GET') current.writes.push(`${response.status()} ${request.method()} ${url.pathname}`);
    if (response.status() >= 400) {
      const body = await response.text().catch(() => '');
      current.http.push(`${response.status()} ${request.method()} ${url.pathname} ${body.slice(0, 160)}`);
    }
  });
}

const waitFor = (page, method, pattern, timeout = 10000) =>
  page.waitForResponse(response => response.request().method() === method && pattern.test(new URL(response.url()).pathname), { timeout });

async function ready(page) {
  await page.waitForSelector('body.loaded', { timeout: 15000 });
  await page.waitForTimeout(1200);
}

const gridCard = (page, text) => page.locator('mymind-grid mymind-card', { hasText: text }).first();

/** Reports where the card containing `text` sits among the non-composer cards of the grid. */
function position(page, text) {
  return page.evaluate(marker => {
    const cards = [...document.querySelectorAll('mymind-grid mymind-card')].filter(card => !card.classList.contains('composer'));
    const target = cards.find(card => card.dataset.id === marker || card.textContent.includes(marker));
    if (!target) return { found: false, count: cards.length };
    const rect = card => card.getBoundingClientRect();
    const minTop = Math.min(...cards.map(card => rect(card).top));
    const sameRow = cards.filter(card => Math.abs(rect(card).top - minTop) < 2);
    const leftmost = sameRow.reduce((best, card) => (rect(card).left < rect(best).left ? card : best));
    return {
      found: true,
      id: target.dataset.id,
      domIndex: cards.indexOf(target),
      atTop: Math.abs(rect(target).top - minTop) < 2 && leftmost === target,
      top: Math.round(rect(target).top),
      className: target.className,
    };
  }, text);
}

async function openCard(page, text) {
  const card = gridCard(page, text);
  await card.waitFor({ state: 'visible', timeout: 10000 });
  await card.hover();
  await card.locator('.hover').first().click();
  await page.waitForSelector('mymind-lightbox mymind-card-detail', { timeout: 10000 });
  await page.waitForTimeout(1000);
}

async function closeDetail(page) {
  await page.evaluate(() => document.activeElement?.blur());
  await page.keyboard.press('Escape');
  await page.waitForSelector('mymind-card-detail', { state: 'detached', timeout: 5000 }).catch(() => {});
  await page.waitForTimeout(600);
}

async function addTag(page, name) {
  const posted = waitFor(page, 'POST', /^\/objects\/[^/]+\/tags$/);
  await page.locator('mymind-card-detail .add-tag-button').click();
  const input = page.locator('mymind-card-detail .add-tag-form input.input');
  await input.waitFor({ state: 'visible', timeout: 5000 });
  await input.fill(name);
  await input.press('Enter');
  return posted;
}

async function step1(page) {
  begin('1 create note');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  const saved = waitFor(page, 'POST', /^\/objects$/, 15000);
  await page.keyboard.press('n');
  await page.waitForTimeout(500);
  await page.keyboard.type(NOTE);
  await page.waitForTimeout(300);
  await shot(page, 'composer-typed');
  await page.keyboard.press('Control+Enter');
  const response = await saved;
  const body = await response.json().catch(() => ({}));
  check('POST /objects returns 201', response.status() === 201, `status ${response.status()}`);
  check('response carries a card id', Boolean(body.id), JSON.stringify(body).slice(0, 120));
  await gridCard(page, NOTE).waitFor({ state: 'visible', timeout: 10000 }).catch(() => {});
  await page.waitForTimeout(1500);
  let where = await position(page, NOTE);
  check('card appears in grid without reload', where.found, JSON.stringify(where));
  check('card is at the top of the grid', where.atTop, `top=${where.top} domIndex=${where.domIndex}`);
  await shot(page, 'note-created');
  await page.reload();
  await ready(page);
  where = await position(page, NOTE);
  check('card present after reload', where.found, JSON.stringify(where));
  check('card is at the top after reload', where.atTop && where.domIndex === 0, `top=${where.top} domIndex=${where.domIndex}`);
  await shot(page, 'note-after-reload');
}

async function step2(page) {
  begin('2 title, tag, highlight');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  await openCard(page, NOTE);
  await shot(page, 'detail-open');
  const id = await page.locator('mymind-card-sidebar').getAttribute('data-object-id');

  const titled = waitFor(page, 'PATCH', /^\/objects\/[^/]+$/);
  const title = page.locator('mymind-card-detail input.title');
  await title.click();
  await title.fill(TITLE);
  await title.press('Tab');
  const patch = await titled.catch(error => error);
  check('title PATCH answered 200', patch.status?.() === 200, patch.status ? `status ${patch.status()} body ${patch.request().postData()}` : patch.message);

  const tagged = await addTag(page, TAG).catch(error => error);
  check('tag POST answered', tagged.status && tagged.status() < 300, tagged.status ? `status ${tagged.status()} ${(await tagged.text()).slice(0, 100)}` : tagged.message);
  await page.waitForTimeout(500);
  const tagTexts = await page.locator('mymind-card-detail mymind-tag-list mymind-tag .value').allTextContents();
  check('tag chip shown in detail', tagTexts.some(text => text.includes(TAG)), tagTexts.join(' | '));

  const highlightTag = await addTag(page, 'highlight').catch(error => error);
  check('highlight tag POST answered', highlightTag.status && highlightTag.status() < 300, highlightTag.status ? `status ${highlightTag.status()}` : highlightTag.message);
  const highlighted = await page.evaluate(() => document.querySelector('mymind-card-detail .body.inner')?.classList.contains('highlighted'));
  check('detail body gets .highlighted', highlighted);

  const prose = page.locator('mymind-card-detail .card-detail-prose .ProseMirror p').first();
  await prose.click({ clickCount: 3 });
  const toggle = page.locator('.toggle-highlight').first();
  const toggleVisible = await toggle.waitFor({ state: 'visible', timeout: 4000 }).then(() => true, () => false);
  if (toggleVisible) {
    const put = waitFor(page, 'PUT', /^\/objects\/[^/]+\/prose$/, 8000);
    await toggle.click();
    const proseResponse = await put.catch(error => error);
    check('prose highlight mark saved (PUT prose)', proseResponse.status && proseResponse.status() < 300, proseResponse.status ? `status ${proseResponse.status()}` : proseResponse.message);
    const marks = await page.locator('mymind-card-detail .card-detail-prose mark').count();
    check('highlight mark in prose', marks > 0, `${marks} mark elements`);
  } else {
    note('bubble menu .toggle-highlight never appeared after selecting text');
    check('prose highlight toggle available', false, 'toggle not visible');
  }
  await shot(page, 'detail-edited');
  await closeDetail(page);

  const where = await position(page, TITLE);
  check('grid card shows new title', where.found, JSON.stringify(where));
  check('grid card has .highlighted class', /highlighted/.test(where.className ?? ''), where.className ?? '');
  await shot(page, 'grid-after-edit');

  await page.reload();
  await ready(page);
  await openCard(page, TITLE).catch(() => openCard(page, NOTE));
  const titleValue = await page.locator('mymind-card-detail input.title').inputValue();
  const tagsAfter = await page.locator('mymind-card-detail mymind-tag-list mymind-tag .value').allTextContents();
  check('title persisted after reload', titleValue === TITLE, titleValue);
  check('tag persisted after reload', tagsAfter.some(text => text.includes(TAG)), tagsAfter.join(' | '));
  check('highlight persisted after reload', await page.evaluate(() => document.querySelector('mymind-card-detail .body.inner')?.classList.contains('highlighted')));
  note(`card ${id}`);
  await shot(page, 'detail-after-reload');
  await closeDetail(page);
}

async function step3(page) {
  begin('3 add to space');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  const card = gridCard(page, NOTE);
  await card.waitFor({ state: 'visible', timeout: 10000 });
  await card.hover();
  await card.locator('.hover').first().click({ button: 'right' });
  const entry = page.locator('mymind-card-menu .action.add-to-space').first();
  const menuVisible = await entry.waitFor({ state: 'visible', timeout: 5000 }).then(() => true, () => false);
  check('context menu shows "Add to space"', menuVisible);
  await shot(page, 'context-menu');
  if (!menuVisible) return;
  await entry.click();
  const items = page.locator('mymind-space-chooser li');
  await items.first().waitFor({ state: 'visible', timeout: 5000 }).catch(() => {});
  const names = await items.allTextContents();
  check('space chooser lists spaces', names.length > 0, `${names.length} spaces`);
  await shot(page, 'space-chooser');
  const added = waitFor(page, 'PUT', /^\/spaces\/[^/]+\/objects\/[^/]+$/, 8000);
  await items.first().click();
  const response = await added.catch(error => error);
  check('PUT /spaces/:id/objects/:id answered', response.status && response.status() < 300, response.status ? `status ${response.status()}` : response.message);
  if (!response.url) return;
  const spaceId = new URL(response.url()).pathname.split('/')[2];
  note(`space ${spaceId} (${names[0]?.trim().replace(/\s+/g, ' ')})`);
  await page.waitForTimeout(800);
  await shot(page, 'space-added');
  const chip = page.locator(`a[href*="${spaceId}"]`).first();
  if (await chip.count()) await chip.click();
  else { note('no chip for the space on /everything; opened by URL'); await page.goto(`${BASE}/spaces/${spaceId}`); }
  await ready(page);
  const where = await position(page, NOTE);
  check('card listed inside the space', where.found, `url ${new URL(page.url()).pathname} ${JSON.stringify(where)}`);
  await shot(page, 'space-opened');
  await page.reload();
  await ready(page);
  const reloaded = await position(page, NOTE);
  check('card still in the space after reload', reloaded.found, JSON.stringify(reloaded));
}

async function step4(page) {
  begin('4 delete, trash, restore');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  await openCard(page, NOTE);
  const deleted = waitFor(page, 'DELETE', /^\/objects\/[^/]+$/, 8000);
  await page.locator('mymind-card-detail .delete-action').click();
  const response = await deleted.catch(error => error);
  check('DELETE /objects/:id answered', response.status && response.status() < 300, response.status ? `status ${response.status()}` : response.message);
  await page.waitForTimeout(1200);
  await shot(page, 'after-delete');
  if (await page.locator('mymind-card-detail').count()) await closeDetail(page);
  await page.reload();
  await ready(page);
  check('card gone from /everything', !(await position(page, NOTE)).found);

  await page.goto(`${BASE}/trash`);
  await ready(page);
  let where = await position(page, NOTE);
  check('card listed in /trash', where.found, `${JSON.stringify(where)} url ${new URL(page.url()).pathname}`);
  await shot(page, 'trash');
  if (!where.found) return;
  const card = gridCard(page, NOTE);
  const restored = waitFor(page, 'POST', /^\/objects\/[^/]+\/restore$/, 8000);
  await card.hover();
  await card.locator('.hover').first().click();
  const restore = await restored.catch(error => error);
  check('POST /objects/:id/restore answered', restore.status && restore.status() < 300, restore.status ? `status ${restore.status()}` : restore.message);
  await page.waitForTimeout(1200);
  check('card removed from the trash grid', !(await position(page, NOTE)).found);
  await shot(page, 'after-restore');
  const exit = page.locator('.exit').first();
  if (await exit.count()) await exit.click({ timeout: 3000 });
  else note('no .exit control in the trash header');
  await page.waitForTimeout(1500);
  where = await position(page, NOTE);
  note(`in-page grid right after exit, before any sync: found=${where.found}`);
  await page.evaluate(() => window.dispatchEvent(new Event('focus')));
  await page.waitForTimeout(2500);
  where = await position(page, NOTE);
  check('card back in /everything after the focus sync', where.found, JSON.stringify(where));
  await shot(page, 'restored-after-sync');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  where = await position(page, NOTE);
  check('card back in /everything after a fresh load', where.found, JSON.stringify(where));
  note(`restored card atTop=${where.atTop} domIndex=${where.domIndex}`);
  await shot(page, 'restored-in-grid');
}

async function step5(page) {
  begin('5 save a URL');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  await page.evaluate(() => document.activeElement?.blur());
  const posted = waitFor(page, 'POST', /^\/objects$/, 10000);
  await page.evaluate(url => {
    const data = new DataTransfer();
    data.setData('text/plain', url);
    document.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
  }, PASTED_URL);
  const response = await posted.catch(error => error);
  check('POST /objects answered', response.status && response.status() < 300, response.status ? `status ${response.status()} req ${response.request().postData()}` : response.message);
  const id = response.json ? (await response.json().catch(() => ({}))).id : undefined;
  await page.waitForTimeout(2500);
  const describe = () => page.evaluate(marker => {
    const card = [...document.querySelectorAll('mymind-grid mymind-card')].find(item => item.dataset.id === marker);
    return card ? { className: card.className, text: card.textContent.replace(/\s+/g, ' ').trim().slice(0, 140) } : null;
  }, id);
  const live = await describe();
  note(`new card before reload: ${JSON.stringify(live)}`);
  check('placeholder card appears without reload', live && /example\.com/.test(live.text), JSON.stringify(live));
  check('placeholder card is at the top', (await position(page, id)).atTop, JSON.stringify(await position(page, id)));
  await shot(page, 'url-pasted');
  await page.reload();
  await ready(page);
  const after = await describe();
  note(`new card after reload: ${JSON.stringify(after)}`);
  check('URL card persists after reload', after && /example\.com/.test(after.text), JSON.stringify(after));
  check('URL card is at the top after reload', (await position(page, id)).atTop);
  await shot(page, 'url-after-reload');
}

async function step6(page) {
  begin('6 color scheme');
  await page.goto(`${BASE}/everything`);
  await ready(page);
  const initial = await page.evaluate(() => mymind.colorScheme);
  const target = initial === 'dark' ? 'light' : 'dark';
  note(`initial scheme ${initial}, switching to ${target}`);
  await shot(page, 'scheme-before');
  await page.locator('.color-scheme-toggle').click();
  await page.waitForSelector('.color-scheme-selector.visible', { timeout: 5000 }).catch(() => {});
  const patched = waitFor(page, 'PATCH', /^\/users\//, 8000);
  await page.locator(`.color-scheme-selector .${target}`).click();
  const response = await patched.catch(error => error);
  check('PATCH /users/:uid answered', response.status && response.status() < 300, response.status ? `status ${response.status()} req ${response.request().postData()}` : response.message);
  await page.waitForTimeout(1000);
  await shot(page, 'scheme-switched');
  const live = await page.evaluate(() => ({ scheme: mymind.colorScheme, body: document.body.className }));
  check('scheme applied live', live.scheme === target, JSON.stringify(live));
  await page.reload();
  await ready(page);
  const reloaded = await page.evaluate(() => ({ scheme: mymind.colorScheme, body: document.body.className, current: document.querySelector('.color-scheme-selector .current')?.textContent?.trim() }));
  check('scheme persisted after reload', reloaded.scheme === target, JSON.stringify(reloaded));
  const shell = await page.evaluate(() => {
    try { return JSON.parse(document.querySelector('script#config').textContent).colorScheme ?? null; }
    catch { return null; }
  });
  check('server shell echoes the scheme', shell === target, `shell colorScheme=${shell}`);
  await shot(page, 'scheme-after-reload');
}

function serverLines() {
  if (!SERVER_LOG || !fs.existsSync(SERVER_LOG)) return { error: 'SERVER_LOG not set or missing', lines: [], placeholders: 0 };
  const text = fs.readFileSync(SERVER_LOG).subarray(logOffset).toString('utf8');
  const lines = text.split('\n').filter(Boolean);
  return {
    lines: [...new Set(lines.filter(line => /\[miss\]|\[error\]|\[write ignored\]|error|exception|ENOENT/i.test(line)))],
    placeholders: lines.filter(line => line.includes('[placeholder]')).length,
  };
}

const { browser, close } = await launchVerifier();
let exitCode = 0;
try {
  const context = browser.contexts()[0] ?? await browser.newContext();
  const page = await context.newPage();
  await page.setViewportSize({ width: 1280, height: 720 });
  attach(page);
  for (const fn of [step1, step2, step3, step4, step5, step6]) {
    try { await fn(page); }
    catch (error) {
      current.error = error.message.split('\n')[0];
      await shot(page, 'error').catch(() => {});
    }
  }
} finally {
  try { await close(); } catch (error) { console.error(`browser close: ${error.code ?? error.message}`); }
  for (const name of tmpProfiles()) {
    if (!profilesBefore.has(name)) fs.rmSync(path.join(os.tmpdir(), name), { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
  }
}

for (const result of results) {
  const failed = result.error || result.checks.some(item => !item.pass);
  if (failed) exitCode = 1;
  console.log(`\n== ${result.name}: ${failed ? 'BROKEN' : 'ok'}${result.error ? ` (exception: ${result.error})` : ''}`);
  for (const item of result.checks) console.log(`  ${item.pass ? 'pass' : 'FAIL'} ${item.label}${item.detail ? ` -- ${item.detail}` : ''}`);
  for (const text of result.notes) console.log(`  note: ${text}`);
  console.log(`  writes: ${result.writes.join(', ') || 'none'}`);
  if (result.console.length) console.log(`  console errors: ${[...new Set(result.console)].join(' || ')}`);
  if (result.failedRequests.length) console.log(`  failed requests: ${[...new Set(result.failedRequests)].join(' || ')}`);
  if (result.http.length) console.log(`  http >= 400: ${[...new Set(result.http)].join(' || ')}`);
  if (result.abortedMedia) console.log(`  aborted .mp4 loads (uncaptured video, benign): ${result.abortedMedia}`);
  console.log(`  shots: ${result.shots.join(', ')}`);
}
const server = serverLines();
console.log(`\n== server log (${server.placeholders} [placeholder] lines omitted)`);
console.log(server.error ?? (server.lines.length ? server.lines.map(line => `  ${line}`).join('\n') : '  no [miss] or error lines'));
process.exitCode = exitCode;
