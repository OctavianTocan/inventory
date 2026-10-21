import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

test('the pinned deployment CLI loads its Node peers without cloud credentials', () => {
  const auth = mkdtempSync(path.join(os.tmpdir(), 'alchemy-cli-fixture-'));
  try {
    const result = spawnSync(process.execPath, ['./node_modules/alchemy/bin/cli.js', 'deploy', '--help'], {
      cwd: fileURLToPath(new URL('../', import.meta.url)),
      env: { PATH: process.env.PATH, ALCHEMY_HOME: auth, NO_TRACK: '1' },
      encoding: 'utf8', timeout: 30000,
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /alchemy deploy/);
    assert.match(result.stdout, /--config/);
    assert.match(result.stdout, /--env-file/);
  } finally {
    rmSync(auth, { recursive: true, force: true });
  }
});
