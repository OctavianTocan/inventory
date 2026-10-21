import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

for (const [label, stage, armed] of [
  ['declares private staging resources and the real Rust bindings', 'staging', '0'],
  ['arms only the private probe when explicitly requested', 'staging', '1'],
  ['rejects a production stage before declaring resources', 'production', '0'],
]) {
  test(`the actual stack ${label}`, () => {
    const result = spawnSync(process.execPath, [fileURLToPath(new URL('./fixtures/check-stack.mjs', import.meta.url))], {
      env: { PATH: process.env.PATH, STACK_FIXTURE_STAGE: stage, MYMIND_ALCHEMY_PROBE: armed },
      encoding: 'utf8', timeout: 30000,
    });
    assert.equal(result.status, 0, result.stderr);
  });
}
