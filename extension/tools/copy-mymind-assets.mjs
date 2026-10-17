import { cpSync, existsSync, mkdirSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const target = resolve(here, '..', 'mymind');
const extensionId = 'nmgcefdhjpjefhgcpocffdlibknajbmj';
const fallback = join(homedir(), '.config/chromium/Default/Extensions', extensionId);

/**
 * Picks the folder that holds mymind's installed extension files.
 * @param {string | undefined} given - A folder from the command line or `MYMIND_EXTENSION_SOURCE`.
 * @returns {string} A folder with `images/`, `icons/`, `fonts/` and `vendor/`.
 * @throws {Error} When no usable folder exists.
 */
function findSource(given) {
  if (given) return resolve(given);
  if (!existsSync(fallback)) throw new Error(`No source given and ${fallback} does not exist.`);
  const versions = readdirSync(fallback).sort();
  if (!versions.length) throw new Error(`No version folder under ${fallback}.`);
  return join(fallback, versions[versions.length - 1]);
}

const source = findSource(process.argv[2] ?? process.env.MYMIND_EXTENSION_SOURCE);
mkdirSync(target, { recursive: true });
for (const folder of ['images', 'icons', 'fonts', 'vendor']) {
  const from = join(source, folder);
  if (!existsSync(from)) throw new Error(`Missing ${from}`);
  cpSync(from, join(target, folder), { recursive: true });
}
console.log(`Copied mymind assets from ${source} to ${target}`);
