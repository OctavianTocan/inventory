// A dedicated Chromium for comparisons: the same browser as the agent one, but rendering offscreen (Ozone's
// headless platform, so no windows and an unchanged user agent) with a renderer that draws the same pixels
// every time: software raster, no partial re-raster, no deferred image decode.
import { spawn } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright-core';

const BINARY = process.env.VERIFIER_CHROMIUM ?? '/usr/lib/chromium/chromium';
const PORT = Number(process.env.VERIFIER_PORT ?? 9333);
const FLAGS = [
  ...(process.env.VERIFIER_EXTRA_FLAGS ?? '').split(' ').filter(Boolean),
  '--disable-gpu',
  // WebGL (serendipity's background) on the CPU rasterizer, which is deterministic too.
  '--use-angle=swiftshader',
  '--enable-unsafe-swiftshader',
  '--disable-partial-raster',
  // Wheel and keyboard scrolling animate on real time otherwise.
  '--disable-smooth-scrolling',
  '--disable-checker-imaging',
  // Animations run on the main thread, so a frozen animation is rastered at exactly its frozen state.
  '--disable-threaded-animation',
  '--disable-threaded-scrolling',
  '--run-all-compositor-stages-before-draw',
  '--force-color-profile=srgb',
  '--force-device-scale-factor=1',
  '--disable-backgrounding-occluded-windows',
  '--disable-renderer-backgrounding',
  '--disable-background-timer-throttling',
  // Responses are handed to pages by DevTools, which leaves them without an address space; Chrome would then
  // treat the replica on localhost as a public site and block its WebSocket to localhost.
  '--disable-features=LocalNetworkAccessChecks,BlockInsecurePrivateNetworkRequests',
  '--disable-extensions',
  '--disable-sync',
  '--password-store=basic',
  '--no-first-run',
  '--no-default-browser-check',
  '--window-size=1400,940',
];

/**
 * Starts the verifier Chromium and connects to it.
 * @returns {Promise<{ browser: import('playwright-core').Browser, close: () => Promise<void> }>}
 *   `close` stops the process and deletes its throwaway profile.
 * @throws {Error} When the browser does not expose DevTools within 20 seconds.
 */
export async function launchVerifier() {
  const profile = await mkdtemp(path.join(os.tmpdir(), 'mymind-verifier-'));
  const child = spawn(
    BINARY,
    [process.env.VERIFIER_OZONE ? `--ozone-platform=${process.env.VERIFIER_OZONE}` : '--ozone-platform=headless', `--user-data-dir=${profile}`, '--remote-debugging-address=127.0.0.1', `--remote-debugging-port=${PORT}`, '--class=verify-chromium', ...FLAGS.filter((flag) => !(process.env.VERIFIER_DROP_FLAGS ?? '').split(' ').includes(flag)), 'about:blank'],
    { stdio: 'ignore', detached: false },
  );
  const endpoint = `http://127.0.0.1:${PORT}`;
  for (let i = 0; i < 100; i++) {
    if (await fetch(`${endpoint}/json/version`).then((r) => r.ok, () => false)) {
      const browser = await chromium.connectOverCDP(endpoint);
      return {
        browser,
        async close() {
          child.kill();
          await new Promise((resolve) => child.once('exit', resolve));
          await rm(profile, { recursive: true, force: true });
        },
      };
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  child.kill();
  throw new Error(`verifier Chromium did not open DevTools on ${endpoint}`);
}
