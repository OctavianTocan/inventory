// Content digests that tie a verification run to the exact code it verified, so REA's coverage check
// rejects results once the replica or the normalization changes.
import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
// Hash ordered relative file names and their exact bytes.
const digestOf = async (files) => {
  const hash = createHash('sha256');
  for (const file of files) hash.update(file).update('\0').update(await readFile(path.join(ROOT, file))).update('\0');
  return hash.digest('hex');
};

// List request-serving Rust source files without generated build output.
const rustSources = async (dir) => (await readdir(path.join(ROOT, dir), { recursive: true }))
  .filter(name => name.endsWith('.rs')).map(name => `${dir}/${name}`);

/**
 * Hash the native request-serving Rust sources, manifests and build inputs.
 * @returns {Promise<string>} SHA-256 bound to the native backend and replay helper.
 */
export async function replicaDigest() {
  const names = ['js', 'msgpack', 'core', 'sqlite', 'server'];
  const crates = await Promise.all(names.map(name => rustSources(`rust/crates/${name}/src`)));
  return digestOf([
    'rust/Cargo.toml', 'rust/Cargo.lock', 'rust/.cargo/config.toml',
    ...names.map(name => `rust/crates/${name}/Cargo.toml`),
    'rust/crates/server/build.rs', 'rust/fetch-libvips.sh', 'tools/replay-store.mjs', 'tools/lib/device.mjs',
    ...crates.flat(),
  ].sort());
}

/**
 * Hash the Worker request-serving sources, bindings and native media-container build inputs.
 * @returns {Promise<string>} SHA-256 bound to the Worker backend.
 */
export async function workerDigest() {
  const names = ['js', 'msgpack', 'core', 'worker', 'sqlite', 'server'];
  const crates = await Promise.all(names.map(name => rustSources(`rust/crates/${name}/src`)));
  return digestOf([
    'rust/Cargo.toml', 'rust/Cargo.lock', 'rust/.cargo/config.toml',
    ...names.map(name => `rust/crates/${name}/Cargo.toml`),
    'rust/crates/server/build.rs', 'rust/crates/worker/wrangler.jsonc',
    'rust/crates/worker/package.json', 'rust/crates/worker/package-lock.json',
    'rust/container/Dockerfile', 'rust/container/Dockerfile.dockerignore', 'rust/fetch-libvips.sh',
    ...crates.flat(),
  ].sort());
}

/**
 * Validate an explicitly selected verification transport and backend.
 * @param {string} mode Command-line server mode; omitted callers use the native Rust listener.
 * @returns {'rust'|'hosted-rust'|'hosted-worker'} The validated mode.
 * @throws {Error} If the mode does not identify a supported verification backend.
 */
export function verificationMode(mode = 'rust') {
  if (!['rust', 'hosted-rust', 'hosted-worker'].includes(mode)) throw new Error(`Unknown verification server ${mode}; use rust, hosted-rust or hosted-worker`);
  return mode;
}

/**
 * Hash the selected backend and, for hosted modes, the signed-in proxy that serves it to the verifier.
 * @param {string} mode Explicit verification server mode.
 * @returns {Promise<string>} SHA-256 bound to the selected backend and transport sources.
 * @throws {Error} If the mode is unsupported or a build input cannot be read.
 */
export async function verificationDigest(mode = 'rust') {
  verificationMode(mode);
  const backend = await (mode === 'hosted-worker' ? workerDigest() : replicaDigest());
  if (mode === 'rust') return backend;
  const proxy = await digestOf(['tools/hosted-proxy.mjs', 'tools/server-identity.mjs', 'tools/digests.mjs']);
  return createHash('sha256').update(mode).update('\0').update(backend).update('\0').update(proxy).digest('hex');
}

/** @returns {Promise<string>} SHA-256 of everything that makes both sides deterministic (the normalization). */
export async function normalizationDigest() {
  const files = await digestOf(['tools/page-determinism.js', 'tools/side.mjs', 'tools/verifier-browser.mjs', 'tools/server-command.mjs', 'tools/server-identity.mjs']);
  // Browser flags changed from the environment are part of the normalization too.
  const flags = [process.env.VERIFIER_EXTRA_FLAGS, process.env.VERIFIER_DROP_FLAGS].filter(Boolean).join('|');
  return flags ? createHash('sha256').update(files).update(flags).digest('hex') : files;
}
