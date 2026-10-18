// Snapshots data/mymind.db and data/media into library/ as a local backup that stays out of Git.
// Usage: npm run bundle.
import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { fileURLToPath } from 'node:url';

if (process.argv.includes('--help')) {
  console.log('Usage: npm run bundle\nSnapshots the local library database and media into the local backup directory.');
  process.exit(0);
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bundle = path.join(ROOT, 'library');
const source = path.join(ROOT, 'data', 'mymind.db');
if (!existsSync(source)) throw new Error('No data/mymind.db to bundle; run rust/target/release/mymind seed-db first.');

mkdirSync(bundle, { recursive: true });
const target = path.join(bundle, 'mymind.db');
rmSync(target, { force: true });
// VACUUM INTO reads one consistent snapshot even while the server is writing.
new DatabaseSync(source, { readOnly: true }).exec(`VACUUM INTO '${target.replaceAll("'", "''")}'`);
rmSync(path.join(bundle, 'media'), { recursive: true, force: true });
if (existsSync(path.join(ROOT, 'data', 'media'))) cpSync(path.join(ROOT, 'data', 'media'), path.join(bundle, 'media'), { recursive: true });

console.log('library/mymind.db written.');
