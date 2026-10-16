// Records mymind's signed-out sign-in page (and the error variants server.mjs shows) from a cookie-less context of
// the agent Chromium, plus every static asset those pages load, so the replica can serve them when accounts are on.
import { existsSync } from 'node:fs';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { chromium } from 'playwright-core';

const { values: opts } = parseArgs({ options: { cdp: { type: 'string', default: 'http://127.0.0.1:9222' } } });

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ORIGIN = 'https://access.mymind.com';
const STATIC_ORIGIN = 'https://static.accelerator.net/';
const PAGES = { signin: '', AccountNotFound: '?error=AccountNotFound&identityProvider=Google' };

const browser = await chromium.connectOverCDP(opts.cdp);
// A new context carries no cookies, so mymind answers as it does for anyone signed out.
const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
const page = await context.newPage();
const assets = new Set();
page.on('response', (res) => res.url().startsWith(STATIC_ORIGIN) && assets.add(res.url().split('?')[0]));

await mkdir(path.join(ROOT, 'capture', 'signin'), { recursive: true });
for (const [name, query] of Object.entries(PAGES)) {
  const res = await page.goto(`${ORIGIN}/signin${query}`, { waitUntil: 'networkidle' });
  if (res.status() !== 200 || !page.url().startsWith(`${ORIGIN}/signin`)) throw new Error(`Expected the signed-out sign-in page, got ${res.status()} ${page.url()}`);
  // The anonymous session's token and nonce are per request; the server fills in its own.
  const html = (await res.text())
    .replace(/(<meta id="authenticityToken" content=")[^"]*"/, '$1"')
    .replace(/(<meta id="nonce" nonce=")[^"]*"/, '$1"');
  await writeFile(path.join(ROOT, 'capture', 'signin', `${name}.html`), html);
  await page.waitForTimeout(3000);
}

let saved = 0;
for (const url of assets) {
  const file = path.join(ROOT, 'mirror', 'static', url.slice(STATIC_ORIGIN.length));
  if (existsSync(file)) continue;
  // A fresh GET returns whole files where the page streamed ranges (video, audio).
  const res = await context.request.get(url);
  if (!res.ok()) continue;
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, await res.body());
  saved++;
}
console.log(`Sign-in pages: ${Object.keys(PAGES).length}, static assets: ${assets.size} seen, ${saved} newly mirrored`);
await context.close();
await browser.close().catch(() => {});
process.exit(0);
