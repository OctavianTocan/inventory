// Checks the hosted library end to end through Cloudflare Access: signed-out requests go to the Access login, the
// Access service token gets through, the TEST_USER password user signs in and reads the library, and /download serves
// the app. With --browser it also signs in with headless Chromium and screenshots /everything.
// Usage: node tools/check-hosted.mjs [--url https://mymind.octaviantocan.com] [--browser] [--out evidence/hosted]
// Reads CF_ACCESS_CLIENT_ID, CF_ACCESS_CLIENT_SECRET, TEST_USER_NAME and TEST_USER_PASSWORD from .env; prints none.
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
const required = ['CF_ACCESS_CLIENT_ID', 'CF_ACCESS_CLIENT_SECRET', 'TEST_USER_NAME', 'TEST_USER_PASSWORD'];
const missing = required.filter((name) => !process.env[name]);
if (missing.length) {
  console.error(`.env has no ${missing.join(', ')} (README, Testing the hosted library).`);
  process.exit(2);
}

const base = new URL(opts.url).origin;
const access = { 'cf-access-client-id': process.env.CF_ACCESS_CLIENT_ID, 'cf-access-client-secret': process.env.CF_ACCESS_CLIENT_SECRET };
const get = (path, headers = {}) => fetch(base + path, { redirect: 'manual', headers: { ...access, ...headers } });
const failures = [];
async function check(label, fn) {
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`); }
}

let cookie = '';
await check('signed out, every page goes to the Cloudflare Access login', async () => {
  for (const path of ['/', '/everything', '/cards', '/download']) {
    const res = await fetch(base + path, { redirect: 'manual' });
    strictEqual(res.status, 302, path);
    ok(new URL(res.headers.get('location')).hostname.endsWith('.cloudflareaccess.com'), path);
  }
});
await check('a wrong service token secret is refused', async () => {
  const res = await get('/signin', { 'cf-access-client-secret': 'wrong' });
  ok(res.status !== 200, `status ${res.status}`);
});
await check('the service token reaches mymind\'s sign-in page', async () => {
  const res = await get('/signin');
  strictEqual(res.status, 200);
  ok((await res.text()).includes('Access <br>your mind'));
});
await check('the library API stays closed without a mymind session', async () => strictEqual((await get('/cards')).status, 401));
await check('a wrong password is refused', async () => {
  const res = await fetch(`${base}/signin/password`, { method: 'POST', redirect: 'manual',
    headers: { ...access, origin: base, 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ username: process.env.TEST_USER_NAME, password: 'not-the-password-0000' }) });
  strictEqual(res.status, 303);
  ok(res.headers.get('location').includes('error='));
});
await check('the test user signs in', async () => {
  const res = await fetch(`${base}/signin/password`, { method: 'POST', redirect: 'manual',
    headers: { ...access, origin: base, 'content-type': 'application/x-www-form-urlencoded' },
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
await check('/download serves the packaged app', async () => {
  const res = await get('/download');
  strictEqual(res.status, 200);
  ok((await res.text()).includes('href="/download/'));
});

if (opts.browser) {
  await check('headless Chromium signs in and shows the library', async () => {
    const { chromium } = await import('playwright-core');
    const browser = await chromium.launch({ executablePath: process.env.CHROMIUM ?? '/usr/bin/chromium', headless: true });
    try {
      const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
      // Only the hosted origin gets the service token; fonts, media and Google never see it.
      await page.route(`${base}/**`, (route) => route.continue({ headers: { ...route.request().headers(), ...access } }));
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
