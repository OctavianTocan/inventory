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
