// Checks the hosted library end to end: signed-out visitors get only the plain sign-in page (never mymind's pages,
// assets or API), the TEST_USER password user signs in and reads the library, and /download serves the app only once
// signed in. With --browser it also signs in with headless Chromium and screenshots /everything.
// Usage: node tools/check-hosted.mjs [--url https://mymind.octaviantocan.com] [--browser] [--out evidence/hosted]
// Reads TEST_USER_NAME and TEST_USER_PASSWORD from .env; prints neither.
import { ok, strictEqual } from 'node:assert';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
try { process.loadEnvFile(join(ROOT, '.env')); } catch {}
const { values: opts } = parseArgs({ options: {
  url: { type: 'string', default: process.env.HOSTED_URL ?? 'https://mymind.octaviantocan.com' },
  browser: { type: 'boolean', default: false },
  out: { type: 'string', default: join(ROOT, 'evidence', 'hosted') },
} });
const required = ['TEST_USER_NAME', 'TEST_USER_PASSWORD'];
const missing = required.filter((name) => !process.env[name]);
if (missing.length) {
  console.error(`.env has no ${missing.join(', ')} (README, Testing the hosted library).`);
  process.exit(2);
}

const base = new URL(opts.url).origin;
const get = (path, headers = {}) => fetch(base + path, { redirect: 'manual', headers });
const failures = [];
async function check(label, fn) {
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`); }
}

let cookie = '';
await check('signed out, the sign-in page is plain, not mymind\'s', async () => {
  const res = await get('/signin');
  strictEqual(res.status, 200);
  const html = await res.text();
  ok(!/mymind/i.test(html), 'no mymind name');
  ok(html.includes('href="/google/oauth2-redirect"') && html.includes('noindex'));
});
await check('signed out, pages go to the sign-in page', async () => {
  for (const path of ['/', '/everything', '/download']) {
    const res = await get(path, { accept: 'text/html', 'sec-fetch-dest': 'document' });
    strictEqual(res.status, 302, path);
    strictEqual(res.headers.get('location'), '/signin', path);
  }
});
await check('signed out, mymind\'s assets stay closed', async () => {
  for (const path of ['/favicon.ico', '/manifest.webmanifest', '/_static/134/0.109.0/styles/main.css']) strictEqual((await get(path)).status, 401, path);
});
await check('robots are asked to stay out', async () => ok((await (await get('/robots.txt')).text()).includes('Disallow: /')));
await check('the library API stays closed without a mymind session', async () => strictEqual((await get('/cards')).status, 401));
await check('/download sends visitors without a mymind session to sign in', async () => {
  const res = await get('/download');
  strictEqual(res.status, 302);
  strictEqual(res.headers.get('location'), '/signin');
});
await check('a wrong password is refused', async () => {
  const res = await fetch(`${base}/signin/password`, { method: 'POST', redirect: 'manual',
    headers: { origin: base, 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ username: process.env.TEST_USER_NAME, password: 'not-the-password-0000' }) });
  strictEqual(res.status, 303);
  ok(res.headers.get('location').includes('error='));
});
await check('the test user signs in', async () => {
  const res = await fetch(`${base}/signin/password`, { method: 'POST', redirect: 'manual',
    headers: { origin: base, 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ username: process.env.TEST_USER_NAME, password: process.env.TEST_USER_PASSWORD }) });
  strictEqual(res.status, 303);
  strictEqual(new URL(res.headers.get('location'), base).pathname, '/everything');
  cookie = res.headers.getSetCookie().map((value) => value.split(';')[0]).join('; ');
  ok(cookie, 'a session cookie');
});
await check('the test user reads the library', async () => {
  const page = await get('/everything', { cookie, accept: 'text/html', 'sec-fetch-dest': 'document' });
  strictEqual(page.status, 200);
  const cards = await get('/cards', { cookie });
  strictEqual(cards.status, 200);
  ok((await cards.arrayBuffer()).byteLength > 1000, 'a non-empty card stream');
});
await check('/download serves the packaged app to the signed-in test user', async () => {
  const res = await get('/download', { cookie });
  strictEqual(res.status, 200);
  ok((await res.text()).includes('href="/download/'));
});

if (opts.browser) {
  await check('headless Chromium signs in and shows the library', async () => {
    const { chromium } = await import('playwright-core');
    const browser = await chromium.launch({ executablePath: process.env.CHROMIUM ?? '/usr/bin/chromium', headless: true });
    try {
      const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
      await page.goto(`${base}/signin/password`);
      await page.fill('input[name=username]', process.env.TEST_USER_NAME);
      await page.fill('input[name=password]', process.env.TEST_USER_PASSWORD);
      await Promise.all([page.waitForURL('**/everything'), page.click('button[type=submit]')]);
      await page.waitForLoadState('networkidle', { timeout: 30_000 }).catch(() => {});
      await mkdir(opts.out, { recursive: true });
      const file = join(opts.out, 'everything.png');
      await page.screenshot({ path: file });
      ok(await page.locator('mymind-card, [data-id]').count() > 0, 'cards on the page');
      console.log(`      screenshot ${file}`);
    } finally {
      await browser.close();
    }
  });
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'}`);
process.exitCode = failures.length ? 1 : 0;
