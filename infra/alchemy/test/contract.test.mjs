import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { assertIsolation, readContract, readProductionContract } from '../scripts/contract.mjs';
import { ARTIFACT_NAME, assertCheckpoint, seal, selectArtifact, unseal } from '../scripts/state.mjs';

const production = readProductionContract();

test('the existing production configuration cannot be used as the staging pilot', () => {
  assert.throws(() => assertIsolation(production, production), /staging|ingress/);
});

test('staging matches the real Rust bindings while rejecting public ingress and production resources', () => {
  const { pilot } = readContract();
  for (const change of [
    { name: production.name }, { routes: production.routes }, { workers_dev: true }, { preview_urls: true },
    { r2_buckets: production.r2_buckets }, { vars: { ...pilot.vars, AUTH: 'none' } }, { stage: 'production' },
    { durable_objects: { bindings: [{ name: 'MEDIA', class_name: 'Library' }] } },
    { containers: [{ ...pilot.containers[0], image: 'other/Dockerfile' }] }, { scriptName: production.name },
  ]) assert.throws(() => assertIsolation({ ...structuredClone(pilot), ...change }, production));
  const runtime = readFileSync(new URL('../../../rust/crates/worker/src/lib.rs', import.meta.url), 'utf8');
  for (const { class_name } of pilot.durable_objects.bindings) assert.match(runtime, new RegExp(`pub use .*${class_name}`));
  const media = readFileSync(new URL('../../../rust/crates/worker/src/media_object.rs', import.meta.url), 'utf8');
  assert.match(media, /const PORT: u16 = 8080/);
  const docker = readFileSync(new URL('../../../rust/container/Dockerfile', import.meta.url), 'utf8');
  assert.match(docker, /EXPOSE 8080/);
  assert.match(docker, /CMD \["mymind", "container"\]/);
});

test('CI state recovery fails closed and authenticates encrypted archives', () => {
  assert.throws(() => selectArtifact([], false), /missing/);
  assert.equal(selectArtifact([], true), undefined);
  const older = { id: 1, name: ARTIFACT_NAME, expired: false, created_at: '2026-10-21T12:01:00Z' };
  assert.throws(() => selectArtifact([older, { ...older, id: 2, expired: true, created_at: '2026-10-21T12:02:00Z' }], false), /expired/);
  assert.throws(() => selectArtifact([older], true), /forbidden/);
  assert.equal(selectArtifact([older], false).id, 1);
  const secret = 'synthetic-encryption-fixture-only';
  const contents = { files: { 'worker.json': 'synthetic state' } };
  const encrypted = seal(contents, secret);
  assert.ok(!encrypted.includes(Buffer.from('synthetic state')));
  assert.deepEqual(unseal(encrypted, secret), contents);
  encrypted[encrypted.length - 1] ^= 1;
  assert.throws(() => unseal(encrypted, secret));
});

test('an interrupted operation cannot fall back to an older checkpoint', () => {
  const marker = { payload: { run_id: '42' }, created_at: '2026-10-21T12:00:00Z' };
  const checkpoint = { workflow_run: { id: 42 }, created_at: '2026-10-21T12:01:00Z' };
  assert.doesNotThrow(() => assertCheckpoint(undefined, undefined));
  assert.doesNotThrow(() => assertCheckpoint(checkpoint, marker));
  assert.throws(() => assertCheckpoint(undefined, marker), /recover/);
  assert.throws(() => assertCheckpoint(checkpoint, undefined), /recover/);
  assert.throws(() => assertCheckpoint(checkpoint, { ...marker, payload: { run_id: '43' } }), /recover/);
  assert.throws(() => assertCheckpoint(checkpoint, { ...marker, created_at: '2026-10-21T12:02:00Z' }), /recover/);
});

test('a newer checkpoint with a lower artifact ID remains authoritative', () => {
  const older = { id: 2, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 42 }, created_at: '2026-10-21T12:01:00Z' };
  const newer = { id: 1, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 43 }, created_at: '2026-10-21T12:03:00Z' };
  const selected = selectArtifact([older, newer], false);
  assert.equal(selected, newer);
  assert.doesNotThrow(() => assertCheckpoint(selected, { payload: { run_id: '43' }, created_at: '2026-10-21T12:02:00Z' }));
  assert.throws(() => selectArtifact([older, { ...newer, expired: true }], false), /expired/);
  assert.throws(() => selectArtifact([older, newer], true), /forbidden/);
  assert.throws(() => assertCheckpoint(newer, { payload: { run_id: '42' }, created_at: '2026-10-21T12:00:00Z' }), /authoritative checkpoint/);
});

test('checkpoint chronology rejects malformed dates and ambiguous newest timestamps', () => {
  const checkpoint = { id: 1, name: ARTIFACT_NAME, expired: false, workflow_run: { id: 42 }, created_at: '2026-10-21T12:01:00Z' };
  for (const created_at of [undefined, null, 0, 'not-a-date', '2026-10-21', '2026-02-30T12:01:00Z', '2026-10-21T24:00:00Z']) {
    assert.throws(() => selectArtifact([checkpoint, { ...checkpoint, id: 2, created_at }], false), /timestamp is invalid/);
  }
  for (const duplicate of [
    { ...checkpoint, id: 2 },
    { ...checkpoint, id: 2, created_at: '2026-10-21T13:01:00+01:00' },
    { ...checkpoint, workflow_run: { id: 43 } },
  ]) assert.throws(() => selectArtifact([checkpoint, duplicate], false), /ambiguous timestamps/);
  assert.equal(selectArtifact([{ ...checkpoint, created_at: '2026-10-21T12:01:00.123Z' }, checkpoint], false).created_at, '2026-10-21T12:01:00.123Z');
});
