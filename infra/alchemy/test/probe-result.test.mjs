import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import test from 'node:test';

test('the actual result reader preserves R2 key separators and consumes private evidence', () => {
  const temporary = mkdtempSync(path.join(os.tmpdir(), 'alchemy-result-fixture-'));
  try {
    const loader = path.join(temporary, 'fetch.mjs');
    writeFileSync(loader, `
      import assert from 'node:assert/strict';
      globalThis.fetch = async (input, options) => {
        const url = new URL(input);
        assert.equal(url.origin, 'https://api.cloudflare.com');
        assert.equal(url.pathname, '/client/v4/accounts/synthetic-account/r2/buckets/mymind-alchemy-staging-fixtures/objects/pilot/results/synthetic-run.json');
        assert.equal(options.headers.authorization, 'Bearer synthetic-fixture');
        return Response.json({ run: 'synthetic-run', passed: true, attempts: 1 });
      };
    `);
    const result = spawnSync(process.execPath, ['--import', pathToFileURL(loader).href,
      fileURLToPath(new URL('../scripts/probe-result.mjs', import.meta.url))], {
      env: { PATH: process.env.PATH, MYMIND_ALCHEMY_RUN: 'synthetic-run',
        CLOUDFLARE_ACCOUNT_ID: 'synthetic-account', CLOUDFLARE_API_TOKEN: 'synthetic-fixture' },
      encoding: 'utf8', timeout: 30000,
    });
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), { run: 'synthetic-run', passed: true, attempts: 1 });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

// Run the real reader with synthetic fetch results and no polling delay.
function runReader(fetchBody) {
  const temporary = mkdtempSync(path.join(os.tmpdir(), 'alchemy-result-retry-'));
  try {
    const loader = path.join(temporary, 'fetch.mjs');
    writeFileSync(loader, `
      import assert from 'node:assert/strict';
      import timers from 'node:timers/promises';
      import { syncBuiltinESMExports } from 'node:module';
      timers.setTimeout = async () => {};
      syncBuiltinESMExports();
      let requests = 0;
      let now = 0;
      Date.now = () => now;
      globalThis.fetch = async () => {
        requests += 1;
        ${fetchBody}
      };
    `);
    return spawnSync(process.execPath, ['--import', pathToFileURL(loader).href,
      fileURLToPath(new URL('../scripts/probe-result.mjs', import.meta.url))], {
      env: { PATH: process.env.PATH, MYMIND_ALCHEMY_RUN: 'synthetic-run',
        CLOUDFLARE_ACCOUNT_ID: 'synthetic-account', CLOUDFLARE_API_TOKEN: 'synthetic-fixture' },
      encoding: 'utf8', timeout: 5000,
    });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

test('a timed-out R2 request does not abort the private result polling window', () => {
  const result = runReader(`
    if (requests === 1) throw new DOMException('Synthetic request timed out', 'TimeoutError');
    assert.equal(requests, 2);
    return Response.json({ run: 'synthetic-run', passed: true, attempts: 1 });
  `);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), { run: 'synthetic-run', passed: true, attempts: 1 });
});

test('a timed-out R2 response body is retried within the polling window', () => {
  const result = runReader(`
    if (requests === 1) return new Response(new ReadableStream({
      start(controller) { controller.error(new DOMException('Synthetic body timed out', 'TimeoutError')); },
    }));
    assert.equal(requests, 2);
    return Response.json({ run: 'synthetic-run', passed: true, attempts: 1 });
  `);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), { run: 'synthetic-run', passed: true, attempts: 1 });
});

test('R2 authentication failures still abort without retrying', () => {
  const result = runReader(`
    assert.equal(requests, 1, 'Authentication failures must not be retried');
    return new Response(null, { status: 403 });
  `);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /Private R2 result read failed with HTTP 403/);
});

test('request timeouts do not extend the private result polling deadline', () => {
  const result = runReader(`
    assert.equal(requests, 1);
    now = 11 * 60_000;
    throw new DOMException('Synthetic request timed out', 'TimeoutError');
  `);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /No successful private smoke result within 11 minutes/);
});
