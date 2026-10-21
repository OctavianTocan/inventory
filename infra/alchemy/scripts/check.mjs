import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { PILOT_ROOT, readContract } from './contract.mjs';

const { pilot } = readContract();
if (!process.argv.includes('--contract-only')) {
  const entry = new URL(pilot.main, PILOT_ROOT);
  assert.ok(existsSync(entry), 'Run npm run build:worker before checking deploy readiness');
  const source = readFileSync(entry, 'utf8');
  for (const { class_name } of pilot.durable_objects.bindings) {
    assert.match(source, new RegExp(`\\b${class_name}\\b`), `prebuilt entry is missing ${class_name}; runtime smoke must confirm its export`);
  }
  const walk = (dir) => readdirSync(dir, { withFileTypes: true }).flatMap((item) => item.isDirectory() ? walk(path.join(dir, item.name)) : [path.join(dir, item.name)]);
  const modules = walk(path.dirname(fileURLToPath(entry)));
  const wasm = modules.filter((file) => file.endsWith('.wasm'));
  assert.ok(wasm.length > 0, 'Rust Worker Wasm module is missing');
  for (const file of wasm) assert.deepEqual([...readFileSync(file).subarray(0, 4)], [0, 97, 115, 109], 'invalid Wasm artifact');
}
console.log('Staging isolation and Rust artifact checks passed. No cloud API was called.');
