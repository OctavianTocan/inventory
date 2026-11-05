import { spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { PILOT_ROOT, ROOT, readContract } from './contract.mjs';

readContract();
const artifactRoot = fileURLToPath(new URL('.artifacts/', PILOT_ROOT));
const rust = path.join(artifactRoot, 'rust');
rmSync(rust, { recursive: true, force: true });
mkdirSync(rust, { recursive: true });
for (const name of ['Cargo.toml', 'Cargo.lock', 'crates', 'test-support']) {
  cpSync(fileURLToPath(new URL(`rust/${name}`, ROOT)), path.join(rust, name), {
    recursive: true,
    filter: (source) => !['node_modules', 'target', 'build', '.wrangler'].includes(path.basename(source)),
  });
}
// The Worker's AI crate builds the shared AI kit from the vendor/ai-kit submodule, at the same place beside rust/.
const kit = path.join(artifactRoot, 'vendor/ai-kit');
rmSync(kit, { recursive: true, force: true });
cpSync(fileURLToPath(new URL('vendor/ai-kit', ROOT)), kit, {
  recursive: true,
  filter: (source) => !['target', 'kotlin', 'build'].includes(path.basename(source)),
});
const result = spawnSync('worker-build', ['--release'], {
  cwd: path.join(rust, 'crates/worker'), stdio: 'inherit',
  env: { ...process.env, CARGO_TARGET_DIR: path.join(artifactRoot, 'cargo-target') },
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
