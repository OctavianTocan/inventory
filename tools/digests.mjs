// Content digests that tie a verification run to the exact code it verified, so REA's coverage check
// rejects results once the replica or the normalization changes.
import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { serverKind } from './server-command.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const digestOf = async (files) => {
  const hash = createHash('sha256');
  for (const file of files) hash.update(file).update('\0').update(await readFile(path.join(ROOT, file))).update('\0');
  return hash.digest('hex');
};

const rustSources = async (dir) => (await readdir(path.join(ROOT, dir), { recursive: true }))
  .filter(name => name.endsWith('.rs')).map(name => `${dir}/${name}`);

/**
 * SHA-256 of the code of the backend being verified: the Rust crates that answer requests when `MYMIND_SERVER` is
 * set, else `server.mjs` and its modules.
 * @returns {Promise<string>} The digest.
 */
export async function replicaDigest() {
  if (serverKind() === 'rust') {
    const crates = await Promise.all(['js', 'msgpack', 'core', 'sqlite', 'server'].map(name => rustSources(`rust/crates/${name}/src`)));
    return digestOf(['rust/Cargo.toml', 'rust/Cargo.lock', ...crates.flat().sort()]);
  }
  const modules = (await Promise.all(['lib', 'lib/render'].map(async (dir) =>
    (await readdir(path.join(ROOT, dir))).filter(name => name.endsWith('.mjs')).map(name => `${dir}/${name}`)))).flat().sort();
  return digestOf(['server.mjs', 'tools/replay-store.mjs', ...modules]);
}

/** @returns {Promise<string>} SHA-256 of everything that makes both sides deterministic (the normalization). */
export async function normalizationDigest() {
  const files = await digestOf(['tools/page-determinism.js', 'tools/side.mjs', 'tools/verifier-browser.mjs']);
  // Browser flags changed from the environment are part of the normalization too.
  const flags = [process.env.VERIFIER_EXTRA_FLAGS, process.env.VERIFIER_DROP_FLAGS].filter(Boolean).join('|');
  return flags ? createHash('sha256').update(files).update(flags).digest('hex') : files;
}
