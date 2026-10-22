import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { ARTIFACT_NAME, seal, unseal } from '../scripts/state.mjs';

const KEY = 'synthetic-state-encryption-password-only';
const ORIGINAL = Buffer.from([0, 1, 127, 128, 255, 10]);
const STATE_FILE = 'mymind-alchemy-pilot/staging/nested/worker.json';

// Copy only the real state program into a temporary project and replace gh, never the network.
function fixture(t, options = {}) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'alchemy-state-cli-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const project = path.join(root, 'project/infra/alchemy');
  mkdirSync(path.join(project, 'scripts'), { recursive: true });
  mkdirSync(path.join(root, 'bin'));
  for (const name of ['state.mjs', 'contract.mjs']) copyFileSync(new URL(`../scripts/${name}`, import.meta.url), path.join(project, 'scripts', name));
  copyFileSync(new URL('./fixtures/gh-state.mjs', import.meta.url), path.join(root, 'bin/gh'));
  chmodSync(path.join(root, 'bin/gh'), 0o755);
  const artifacts = options.artifacts ?? [{ id: 1, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 42 }, created_at: '2026-10-21T12:01:00Z' }];
  const pages = [];
  for (let start = 0; start < artifacts.length || start === 0; start += 100) pages.push({ total_count: artifacts.length, artifacts: artifacts.slice(start, start + 100) });
  writeFileSync(path.join(root, 'github.json'), JSON.stringify({
    artifacts, pages: options.pages ?? pages,
    deployments: options.deployments ?? [{ payload: { run_id: '42' }, created_at: '2026-10-21T12:00:00Z' }],
    workflow: options.workflow ?? '.github/workflows/alchemy-pilot.yml',
  }));
  const encrypted = seal({ version: 1, files: options.files ?? { [STATE_FILE]: ORIGINAL.toString('base64') } }, KEY);
  const zip = spawnSync('python3', ['-I', '-c',
    'import io,sys,zipfile; out=io.BytesIO(); archive=zipfile.ZipFile(out,"w"); archive.writestr("state.enc",sys.stdin.buffer.read()); archive.close(); sys.stdout.buffer.write(out.getvalue())'],
  { input: encrypted });
  assert.equal(zip.status, 0, zip.stderr?.toString());
  writeFileSync(path.join(root, 'archive.zip'), zip.stdout);
  return {
    root, project,
    run(command, overrides = {}) {
      return spawnSync(process.execPath, [path.join(project, 'scripts/state.mjs'), command], {
        env: { PATH: `${path.join(root, 'bin')}${path.delimiter}${process.env.PATH}`,
          STATE_FIXTURE: root, GITHUB_REPOSITORY: 'synthetic/private', GITHUB_SHA: 'a'.repeat(40),
          GITHUB_RUN_ID: '43', MYMIND_ALCHEMY_STATE_KEY: KEY, ...overrides },
        encoding: 'utf8', timeout: 30000,
      });
    },
  };
}

test('the actual restore/seal CLI preserves nested state and checkpoints new writes', (t) => {
  const f = fixture(t);
  const restored = f.run('restore');
  assert.equal(restored.status, 0, restored.stderr);
  const state = path.join(f.project, '.alchemy/state', STATE_FILE);
  assert.deepEqual(readFileSync(state), ORIGINAL);
  const calls = readFileSync(path.join(f.root, 'calls'), 'utf8').trim().split('\n');
  assert.match(calls.at(-2), /artifacts\/1\/zip$/);
  assert.equal(calls.at(-1), 'POST repos/synthetic/private/deployments');
  const marker = JSON.parse(readFileSync(path.join(f.root, 'marker.json'), 'utf8'));
  assert.equal(marker.payload.run_id, '43');
  assert.equal(marker.ref, 'a'.repeat(40));
  const changed = Buffer.from('new synthetic resource state\n');
  writeFileSync(state, changed);
  const sealed = f.run('seal');
  assert.equal(sealed.status, 0, sealed.stderr);
  const archive = readFileSync(path.join(f.project, '.artifacts/state/state.enc'));
  assert.ok(!archive.includes(changed));
  const decoded = unseal(archive, KEY);
  assert.deepEqual(Buffer.from(decoded.files[STATE_FILE], 'base64'), changed);
});

test('the actual restore CLI reads later pages and selects the newer lower-ID checkpoint matching the latest marker', (t) => {
  const older = Array.from({ length: 100 }, (_, index) => ({ id: 200 - index, name: ARTIFACT_NAME, expired: false,
    workflow_run: { id: 42 }, created_at: new Date(Date.UTC(2026, 9, 21, 10, 0, index)).toISOString() }));
  const newer = { id: 1, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 43 }, created_at: '2026-10-21T12:03:00Z' };
  const f = fixture(t, { artifacts: [...older, newer], deployments: [{ payload: { run_id: '43' }, created_at: '2026-10-21T12:02:00Z' }] });
  const restored = f.run('restore', { GITHUB_RUN_ID: '44' });
  assert.equal(restored.status, 0, restored.stderr);
  assert.deepEqual(readFileSync(path.join(f.project, '.alchemy/state', STATE_FILE)), ORIGINAL);
  const calls = readFileSync(path.join(f.root, 'calls'), 'utf8');
  assert.match(calls, /artifacts\?.*--paginate --slurp/);
  assert.match(calls, /actions\/runs\/43\n/);
  assert.match(calls, /artifacts\/1\/zip\n/);
  assert.equal(JSON.parse(readFileSync(path.join(f.root, 'marker.json'), 'utf8')).payload.run_id, '44');
});

const checkpoint = { id: 1, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 42 }, created_at: '2026-10-21T12:01:00Z' };
const hundred = Array.from({ length: 100 }, (_, index) => ({ ...checkpoint, id: index + 101 }));
for (const [label, options, message] of [
  ['newest expired lower-ID checkpoint', { artifacts: [{ ...checkpoint, id: 2 }, { ...checkpoint, expired: true, created_at: '2026-10-21T12:03:00Z' }] }, /expired/],
  ['newest expired checkpoint on a later page', { artifacts: [...hundred, { ...checkpoint, expired: true, created_at: '2026-10-21T12:03:00Z' }] }, /expired/],
  ['malformed checkpoint timestamp', { artifacts: [{ ...checkpoint, created_at: 'invalid' }] }, /timestamp is invalid/],
  ['ambiguous newest timestamp', { artifacts: [checkpoint, { ...checkpoint, id: 2, workflow_run: { id: 43 } }] }, /ambiguous timestamps/],
  ['newest checkpoint from a different operation', { artifacts: [{ ...checkpoint, id: 2 }, { ...checkpoint, workflow_run: { id: 43 }, created_at: '2026-10-21T12:03:00Z' }] }, /authoritative checkpoint/],
  ['missing page', { pages: [{ total_count: 101, artifacts: [checkpoint] }] }, /pagination/],
  ['changed page totals', { pages: [{ total_count: 101, artifacts: hundred }, { total_count: 102, artifacts: [checkpoint] }] }, /pagination/],
  ['short nonterminal page', { pages: [{ total_count: 101, artifacts: hundred.slice(0, 99) }, { total_count: 101, artifacts: [checkpoint, { ...checkpoint, id: 2 }] }] }, /pagination/],
  ['duplicated artifact record', { pages: [{ total_count: 2, artifacts: [checkpoint, checkpoint] }] }, /pagination/],
  ['malformed page', { pages: [{ total_count: 0, artifacts: null }] }, /pagination/],
  ['missing slurp pages', { pages: [] }, /pagination/],
  ['unslurped response', { pages: { total_count: 1, artifacts: [checkpoint] } }, /pagination/],
]) test(`the actual restore CLI rejects ${label} without downloading or marking an operation`, (t) => {
  const f = fixture(t, options);
  const result = f.run('restore');
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, message);
  assert.equal(existsSync(path.join(f.root, 'marker.json')), false);
  assert.equal(existsSync(path.join(f.project, '.alchemy/state')), false);
  assert.ok(!readFileSync(path.join(f.root, 'calls'), 'utf8').includes('/zip'));
});

for (const [label, options, environment, message] of [
  ['wrong workflow', { workflow: '.github/workflows/untrusted.yml' }, {}, /different workflow/],
  ['wrong key', {}, { MYMIND_ALCHEMY_STATE_KEY: 'wrong-synthetic-state-encryption-password' }, /authenticate|decrypt/i],
  ['unsafe archive path', { files: { '../escaped.json': Buffer.from('unsafe fixture').toString('base64') } }, {}, /Unsafe path/],
  ['interrupted later operation', { deployments: [{ payload: { run_id: '43' }, created_at: '2026-10-21T12:02:00Z' }] }, {}, /authoritative checkpoint/],
  ['missing state after initialization', { artifacts: [] }, { MYMIND_ALCHEMY_INITIALIZE: 'true' }, /recover/],
]) {
  test(`the actual restore CLI rejects ${label} before marking a new operation`, (t) => {
    const f = fixture(t, options);
    const result = f.run('restore', environment);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, message);
    assert.equal(existsSync(path.join(f.root, 'marker.json')), false);
    assert.equal(existsSync(path.join(f.project, '.alchemy/escaped.json')), false);
    assert.equal(existsSync(path.join(f.project, '.artifacts/state/state.enc')), false);
  });
}

test('the actual restore CLI permits an explicit first initialization with no history', (t) => {
  const f = fixture(t, { artifacts: [], deployments: [] });
  const result = f.run('restore', { MYMIND_ALCHEMY_INITIALIZE: 'true' });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(existsSync(path.join(f.project, '.alchemy/state')), true);
  assert.equal(existsSync(path.join(f.root, 'marker.json')), true);
  assert.ok(!readFileSync(path.join(f.root, 'calls'), 'utf8').includes('/zip'));
});
