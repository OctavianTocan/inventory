// Runs one suite on live mymind, records every response it sees into replay/, runs the same suite on the
// replica, and has REA compare every screenshot pair. Both sides run with virtual time in fresh contexts of
// a deterministic verifier Chromium, signed in with the session of the agent Chromium on :9222, so any
// difference comes from the servers.
// Usage: node tools/drive.mjs [--profile desktop-light|desktop-dark|iphone|android] [--suite ui|cards|...]
//        [--sides live,replica] [--no-record] [--server rust|hosted-rust|hosted-worker]
import { execFile, spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs, promisify } from 'node:util';
import { chromium, devices } from 'playwright-core';
import { normalizationDigest, verificationDigest, verificationMode } from './digests.mjs';
import { serverCommand } from './server-command.mjs';
import { assertReplicaServer } from './server-identity.mjs';
import { reaRoot } from './rea-root.mjs';
import { openStore } from './replay-store.mjs';
import { Side } from './side.mjs';
import { SUITES } from './suites.mjs';
import { launchVerifier } from './verifier-browser.mjs';

if (process.argv.includes('--help')) {
  console.log('Usage: npm run drive -- [--profile desktop-light|desktop-dark|iphone|android] [--suite NAME] [--sides live,replica] [--no-record] [--server rust|hosted-rust|hosted-worker]\nRequires the signed-in browser on CDP :9222.');
  process.exit(0);
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CDP = 'http://127.0.0.1:9222';
const LIVE = 'https://access.mymind.com';
const REPLICA = 'http://localhost:4321';
const READ_ONLY_POSTS = new Set(['/cards/grid']);
const run = promisify(execFile);

const { values: opts } = parseArgs({
  options: {
    profile: { type: 'string', default: 'desktop-light' },
    suite: { type: 'string', default: 'ui' },
    sides: { type: 'string', default: 'live,replica' },
    'no-record': { type: 'boolean', default: false },
    server: { type: 'string', default: 'rust' },
  },
});
const mode = verificationMode(opts.server);

const phone = (name) => {
  const { defaultBrowserType, ...descriptor } = devices[name];
  return descriptor;
};
const PROFILES = {
  'desktop-light': { viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1, colorScheme: 'light' },
  'desktop-dark': { viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1, colorScheme: 'dark' },
  iphone: { ...phone('iPhone 15 Pro'), colorScheme: 'light' },
  android: { ...phone('Pixel 7'), colorScheme: 'light' },
};
const profile = PROFILES[opts.profile];
const steps = SUITES[`${opts.profile.split('-')[0]}-${opts.suite}`] ?? SUITES[opts.suite];
if (!profile || !steps) throw new Error(`Unknown profile ${opts.profile} or suite ${opts.suite}`);

const OUT = path.join(ROOT, 'evidence', 'drive', `${opts.profile}-${opts.suite}`);

// When evidence/drive/claims/ exists, each run is done once: whichever verify process gets here first claims it
// (a claim made by hand skips a run that is already verified).
const CLAIMS = path.join(ROOT, 'evidence', 'drive', 'claims');
if (existsSync(CLAIMS)) {
  try {
    const claim = `${opts.profile}-${opts.suite}${opts.sides === 'live,replica' ? '' : `.${opts.sides}`}`;
    await writeFile(path.join(CLAIMS, claim), String(process.pid), { flag: 'wx' });
  } catch {
    console.log(`${opts.profile}-${opts.suite} (${opts.sides}) is already claimed; skipping`);
    process.exit(0);
  }
}
const sides = opts.sides.split(',');


// Reuse only the explicitly selected listener, and start a native server only in direct Rust mode.
async function ensureReplica() {
  if (mode !== 'rust') { await assertReplicaServer(Number(new URL(REPLICA).port), mode); return null; }
  const up = await fetch(`${REPLICA}/__replay/reload`, { method: 'POST' }).then((r) => r.ok, () => false);
  if (up) { await assertReplicaServer(Number(new URL(REPLICA).port || 80), mode); return null; }
  const { command, args } = serverCommand();
  const child = spawn(command, args, { cwd: ROOT, stdio: ['ignore', 'inherit', 'inherit'] });
  for (let i = 0; i < 50; i++) {
    await new Promise((r) => setTimeout(r, 200));
    if (await fetch(`${REPLICA}/__replay/reload`, { method: 'POST' }).then((r) => r.ok, () => false)) {
      await assertReplicaServer(Number(new URL(REPLICA).port || 80), mode);
      return child;
    }
  }
  throw new Error('replica server did not start');
}

// Hover targets: every visible element that one of mymind's own `:hover` rules applies to.
async function hoverSelectors() {
  const files = [];
  const walk = async (dir) => {
    for (const entry of await readdir(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) await walk(full);
      else if (entry.name.endsWith('.css')) files.push(full);
    }
  };
  await walk(path.join(ROOT, 'mirror', 'static'));
  const selectors = new Set();
  for (const file of files) {
    const css = (await readFile(file, 'utf8')).replace(/\/\*[\s\S]*?\*\//g, '');
    for (const [, list] of css.matchAll(/([^{}@;]+)\{/g)) {
      for (const selector of list.split(',')) {
        const at = selector.indexOf(':hover');
        if (at < 0 || /:(is|not|where|has)\(/.test(selector.slice(0, at))) continue;
        selectors.add(selector.slice(0, at).trim());
      }
    }
  }
  return [...selectors].filter((s) => s && !/[>+~]$/.test(s));
}

function findHoverTargets(selectors) {
  const seen = new Set();
  const points = [];
  for (const selector of selectors) {
    let elements;
    try {
      elements = document.querySelectorAll(selector);
    } catch {
      continue;
    }
    for (const el of elements) {
      if (seen.has(el)) continue;
      const r = el.getBoundingClientRect();
      if (r.width < 4 || r.height < 4 || r.right <= 0 || r.bottom <= 0 || r.left >= innerWidth || r.top >= innerHeight) continue;
      const x = Math.round(Math.min(Math.max(r.left + r.width / 2, 1), innerWidth - 2));
      const y = Math.round(Math.min(Math.max(r.top + r.height / 2, 1), innerHeight - 2));
      const hit = document.elementFromPoint(x, y);
      if (!hit || !(hit === el || el.contains(hit))) continue;
      seen.add(el);
      points.push({ selector, x, y });
      break;
    }
  }
  return points.sort((a, b) => a.y - b.y || a.x - b.x).slice(0, 60);
}

async function storageFor(browser) {
  const defaults = browser.contexts()[0];
  const state = await defaults.storageState();
  const cookies = state.cookies.filter((c) => /(^|\.)access\.mymind\.com$/.test(c.domain.replace(/^\./, '')));
  const liveTab = defaults.pages().find((p) => p.url().startsWith(LIVE));
  const localStorage = liveTab ? await liveTab.evaluate(() => Object.entries(window.localStorage).map(([name, value]) => ({ name, value }))) : [];
  return {
    live: { cookies, origins: [{ origin: LIVE, localStorage }] },
    replica: { cookies: [], origins: [{ origin: REPLICA, localStorage }] },
  };
}

async function runSide(side, plan) {
  const errors = [];
  const shoot = (name) => side.shot(path.join(OUT, side.name, name));
  const mouse = side.page.mouse;
  for (const step of steps) {
    const label = step.shot ?? step.film ?? step.hovers ?? JSON.stringify(step).slice(0, 80);
    try {
      if (step.goto !== undefined) await side.goto(step.goto, step.wait ?? 2000);
      else if (step.click) {
        const [x, y] = await side.point(step.click);
        await mouse.click(x, y);
        await side.settle(step.wait ?? 1500);
      } else if (step.press) {
        await side.page.keyboard.press(step.press);
        await side.settle(step.wait ?? 1000);
      } else if (step.type) {
        const [x, y] = await side.point(step.type[0]);
        if (profile.hasTouch) await side.page.touchscreen.tap(x, y);
        else await mouse.click(x, y);
        await side.settle(200);
        for (const char of step.type[1]) {
          await side.page.keyboard.type(char);
          await side.settle(80);
        }
        await side.settle(step.wait ?? 1500);
      } else if (step.scroll) {
        await mouse.move(640, 400);
        await mouse.wheel(0, step.scroll);
        await side.settle(step.wait ?? 1000);
      } else if (step.drag) {
        const [x1, y1] = await side.point(step.drag[0]);
        const [x2, y2] = await side.point(step.drag[1]);
        await mouse.move(x1, y1);
        await side.settle(100);
        await mouse.down();
        await side.settle(100);
        for (let i = 1; i <= 10; i++) {
          await mouse.move(x1 + ((x2 - x1) * i) / 10, y1 + ((y2 - y1) * i) / 10);
          await side.settle(32, 16);
        }
        await side.settle(step.wait ?? 0);
      } else if (step.drop) {
        await mouse.up();
        await side.settle(step.wait ?? 1000);
      } else if (step.tap) {
        const [x, y] = await side.point(step.tap, { reveal: step.reveal });
        await side.page.touchscreen.tap(x, y);
        await side.settle(step.wait ?? 1500);
      } else if (step.swipe) {
        const [selector, dx, dy] = step.swipe;
        const [x, y] = await side.point(selector);
        const touch = (type, px, py) =>
          side.cdp.send('Input.dispatchTouchEvent', { type, touchPoints: type === 'touchEnd' ? [] : [{ x: px, y: py }] });
        await touch('touchStart', x, y);
        for (let i = 1; i <= 8; i++) {
          await touch('touchMove', x + (dx * i) / 8, y + (dy * i) / 8);
          await side.settle(16, 16);
        }
        await touch('touchEnd', x + dx, y + dy);
        await side.settle(step.wait ?? 1500);
      } else if (step.shot) await shoot(step.shot);
      else if (step.film) {
        for (let t = 0; t <= step.ms; t += step.every) {
          await shoot(`${step.film}_${String(t).padStart(4, '0')}`);
          if (t < step.ms) await side.settle(step.every, Math.min(step.every, 16));
        }
      } else if (step.hovers) {
        plan.hovers[step.hovers] ??= await side.page.evaluate(findHoverTargets, plan.hoverSelectors);
        for (const [i, point] of plan.hovers[step.hovers].entries()) {
          await mouse.move(point.x, point.y);
          await side.settle(600);
          await shoot(`hover_${step.hovers}_${String(i).padStart(2, '0')}`);
        }
        await mouse.move(...side.restPoint);
        await side.settle(600);
      } else if (step.wait !== undefined) await side.settle(step.wait);
    } catch (error) {
      errors.push(`${label}: ${error.message.split('\n')[0]}`);
      console.log(`  ${side.name} ✗ ${label}: ${error.message.split('\n')[0]}`);
    }
  }
  return errors;
}

async function compareAll() {
  const rea = path.join(reaRoot(), 'scripts', 'rea.mjs');
  const names = (await readdir(path.join(OUT, 'live'))).filter((f) => f.endsWith('.artifact.json')).map((f) => f.slice(0, -'.artifact.json'.length));
  const replicaFiles = new Set(await readdir(path.join(OUT, 'replica')).catch(() => []));
  const results = [];
  const evidenceRecords = [];
  const queue = [...names];
  await Promise.all(
    Array.from({ length: 6 }, async () => {
      for (let name = queue.shift(); name; name = queue.shift()) {
        if (!replicaFiles.has(`${name}.artifact.json`)) {
          results.push({ name, status: 'missing', ratio: 1 });
          continue;
        }
        const { stdout } = await run('node', [rea, 'compare-web-screenshots', path.join(OUT, 'live', `${name}.artifact.json`), path.join(OUT, 'replica', `${name}.artifact.json`), '--channel-threshold', '0', '--format', 'json', '--full-output'], { maxBuffer: 1 << 26 });
        const evidence = JSON.parse(stdout).data;
        const result = evidence.normalized_result;
        evidenceRecords.push(evidence);
        results.push({ name, status: result.status, ratio: result.changed_ratio ?? 0, maxDelta: result.maximum_channel_delta, evidenceId: evidence.evidence_id });
      }
    }),
  );
  results.sort((a, b) => b.ratio - a.ratio || a.name.localeCompare(b.name));
  await writeFile(path.join(OUT, 'rea-evidence.json'), JSON.stringify(evidenceRecords));
  await rm(path.join(OUT, 'diffs'), { recursive: true, force: true });
  await mkdir(path.join(OUT, 'diffs'), { recursive: true });
  for (const r of results.filter((r) => r.status !== 'identical' && r.status !== 'missing')) {
    const [a, b] = ['live', 'replica'].map((s) => path.join(OUT, s, `${r.name}.png`));
    const diff = path.join(OUT, 'diffs', `${r.name}.png`);
    await run('magick', ['compare', '-metric', 'AE', a, b, '-compose', 'src', path.join(os.tmpdir(), 'diff.png')]).catch(() => {});
    await run('magick', [a, b, path.join(os.tmpdir(), 'diff.png'), '+append', diff]).catch(() => {});
  }
  return results;
}

const server = await ensureReplica();
const storage = await storageFor(await chromium.connectOverCDP(CDP));
const verifier = await launchVerifier();
const browser = verifier.browser;
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => verifier.close().finally(() => process.exit(130)));
const startTime = Math.floor(Date.now() / 60000) * 60000;
const planFile = path.join(OUT, 'plan.json');
const plan = sides.includes('live')
  ? { startTime, hovers: {}, hoverSelectors: await hoverSelectors() }
  : JSON.parse(await readFile(planFile, 'utf8'));
const contextFor = (name) => ({ ...profile, storageState: storage[name], timezoneId: Intl.DateTimeFormat().resolvedOptions().timeZone, locale: 'en-US', reducedMotion: 'no-preference' });
const report = { profile: opts.profile, suite: opts.suite, startedAt: new Date().toISOString(), normalizationDigest: await normalizationDigest(), sides: {} };

try {
  for (const name of sides) {
    await rm(path.join(OUT, name), { recursive: true, force: true });
    const store = name === 'live' && !opts['no-record'] ? await openStore() : undefined;
    const side = await Side.open(browser, {
      name,
      origin: name === 'live' ? LIVE : REPLICA,
      contextOptions: contextFor(name),
      startTime: plan.startTime,
      store,
      runId: `${opts.profile}-${opts.suite}`,
      mirror: REPLICA,
      // Live never receives writes: they are answered exactly as the replica answers them.
      firewall:
        name === 'live'
          ? (request) => {
              const url = new URL(request.url);
              return url.origin === LIVE && !['GET', 'HEAD', 'OPTIONS'].includes(request.method) && !READ_ONLY_POSTS.has(url.pathname);
            }
          : undefined,
    });
    console.log(`${name}: running ${steps.length} steps`);
    let errors, media = 0;
    try {
      errors = await runSide(side, plan);
    } finally {
      // Save what live sent even when the pass dies, or stale recordings from older library states stay in replay/.
      if (store) {
        media = await side.recordMedia().catch(() => 0);
        await store.save();
        if (mode !== 'hosted-worker') await fetch(`${REPLICA}/__replay/reload`, { method: 'POST' }).catch(() => {});
      }
    }
    if (name === 'replica') {
      const identity = await assertReplicaServer(Number(new URL(REPLICA).port), mode);
      report.replica = mode === 'hosted-worker'
        ? { offline: false, store: null, proxyTarget: identity.target }
        : await fetch(`${REPLICA}/__replay/reload`, { method: 'POST' }).then((r) => r.json());
      Object.assign(report.replica, { digest: await verificationDigest(mode), server: mode, at: new Date().toISOString() });
    }
    report.sides[name] = { errors, writesAnsweredLocally: side.writes, notes: side.log, mediaRecorded: media };
    console.log(`${name}: ${errors.length} step errors, ${side.writes.length} writes kept off the account, ${side.log.length} notes`);
    await side.close();
  }
  await mkdir(OUT, { recursive: true });
  await writeFile(planFile, JSON.stringify(plan, null, 1));
  if (sides.includes('replica')) {
    report.results = await compareAll();
    const identical = report.results.filter((r) => r.status === 'identical').length;
    for (const r of report.results.filter((r) => r.status !== 'identical')) console.log(`${(r.ratio * 100).toFixed(3).padStart(8)}%  ${r.status.padEnd(10)} ${r.name}`);
    console.log(`\n${identical}/${report.results.length} screenshots pixel-identical (REA, channel threshold 0)`);
  }
  await writeFile(path.join(OUT, 'report.json'), JSON.stringify(report, null, 1));
} finally {
  // Never close the agent browser: closing a connectOverCDP browser can take the user's Chromium down with it.
  await verifier.close();
  server?.kill();
  setTimeout(() => process.exit(process.exitCode ?? 0), 200);
}
