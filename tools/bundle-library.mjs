// Prepares the recordings for committing: moves replay bodies that no recording references into replay/unreferenced/
// (gitignored), so Git only carries what a clone uses, and snapshots data/mymind.db (and data/media) into library/
// as a local backup that stays out of Git. Usage: npm run bundle, then commit capture/, mirror/ and replay/.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, renameSync, rmSync } from 'node:fs';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bundle = path.join(ROOT, 'library');
const source = path.join(ROOT, 'data', 'mymind.db');
if (!existsSync(source)) throw new Error('No data/mymind.db to bundle; run node tools/seed-db.mjs first.');

mkdirSync(bundle, { recursive: true });
const target = path.join(bundle, 'mymind.db');
rmSync(target, { force: true });
// VACUUM INTO reads one consistent snapshot even while the server is writing.
new DatabaseSync(source, { readOnly: true }).exec(`VACUUM INTO '${target.replaceAll("'", "''")}'`);
rmSync(path.join(bundle, 'media'), { recursive: true, force: true });
if (existsSync(path.join(ROOT, 'data', 'media'))) cpSync(path.join(ROOT, 'data', 'media'), path.join(bundle, 'media'), { recursive: true });

const bodies = path.join(ROOT, 'replay', 'bodies');
const unreferenced = path.join(ROOT, 'replay', 'unreferenced');
const used = new Set(Object.values(JSON.parse(readFileSync(path.join(ROOT, 'replay', 'index.json'), 'utf8'))).map(entry => entry.file));
let moved = 0;
for (const file of existsSync(bodies) ? readdirSync(bodies) : []) {
  if (used.has(file)) continue;
  mkdirSync(unreferenced, { recursive: true });
  renameSync(path.join(bodies, file), path.join(unreferenced, file));
  moved++;
}
console.log(`library/mymind.db written; ${moved} unreferenced replay bodies moved to replay/unreferenced/ (safe to delete).`);
