import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

export const ROOT = new URL('../../../', import.meta.url);
export const PILOT_ROOT = new URL('../', import.meta.url);

/** Read the existing Wrangler deployment contract without reading environment files.
 * @returns {object} The production Worker configuration.
 */
export function readProductionContract() {
  return JSON.parse(readFileSync(new URL('../../../rust/crates/worker/wrangler.jsonc', import.meta.url), 'utf8').replace(/^\s*\/\/.*$/gm, ''));
}

/** Validate that a deployment remains isolated to the staging pilot.
 * @param {object} pilot Proposed pilot configuration.
 * @param {object} production Existing production contract.
 * @returns {void}
 */
export function assertIsolation(pilot, production) {
  assert.equal(pilot.stage, 'staging', 'staging is the only allowed stage');
  assert.equal(pilot.stack, 'mymind-alchemy-pilot', 'staging stack is fixed');
  for (const [key, suffix] of Object.entries({ name: 'worker', probe_name: 'probe', container_name: 'media' })) {
    assert.equal(pilot[key], `mymind-alchemy-staging-${suffix}`, 'staging resource names are fixed');
    assert.notEqual(pilot[key], production.name, 'production resources cannot be adopted');
  }
  assert.equal(pilot.workers_dev, false, 'public ingress must stay disabled');
  assert.equal(pilot.preview_urls, false, 'preview ingress must stay disabled');
  assert.deepEqual(pilot.routes, [], 'route ingress must stay disabled');
  for (const key of ['domain', 'domains', 'access', 'preview', 'transferredFrom', 'scriptName', 'adopt']) {
    assert.equal(pilot[key], undefined, `${key} is outside the pilot contract`);
  }
  assert.deepEqual(pilot.vars, {
    AUTH: 'google', PUBLIC_URL: 'https://mymind-alchemy-staging.invalid',
    OWNER_EMAIL: 'owner@example.test', ALLOWED_GOOGLE_DOMAIN: '',
  }, 'sign-in stays enabled and uses a non-routable synthetic origin');
  assert.equal(pilot.main, '.artifacts/rust/crates/worker/build/index.js', 'only the staged Rust build can be uploaded');
  assert.deepEqual(pilot.durable_objects, production.durable_objects, 'Rust namespace binding/class mapping drifted');
  assert.deepEqual(production.migrations?.flatMap((migration) => migration.new_sqlite_classes ?? []).sort(),
    ['Auth', 'Library', 'Media'], 'Alchemy must create the same three SQLite classes');
  assert.deepEqual(pilot.containers, [{
    class_name: 'Media', image: 'rust/container/Dockerfile',
    instance_type: production.containers[0].instance_type, max_instances: production.containers[0].max_instances,
  }], 'Media must keep the existing Rust container contract');
  assert.equal(pilot.containers[0].max_instances, 1, 'pilot container capacity is bounded');
  assert.deepEqual(pilot.r2_buckets, [{ binding: 'BUCKET', bucket_name: 'mymind-alchemy-staging-fixtures' }], 'staging R2 name and binding are fixed');
  assert.ok(!production.r2_buckets.some((bucket) => bucket.bucket_name === pilot.r2_buckets[0].bucket_name), 'production R2 cannot be reused');
}

/** Load and validate the pilot against the real Rust deployment contract.
 * @returns {{pilot: any, production: any}} Validated deployment inputs.
 */
export function readContract() {
  const production = readProductionContract();
  const pilot = JSON.parse(readFileSync(new URL('pilot.json', PILOT_ROOT), 'utf8'));
  assertIsolation(pilot, production);
  return { pilot, production };
}
