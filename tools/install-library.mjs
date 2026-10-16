// Copies the library bundled in library/ into data/ on a fresh checkout. An existing data/mymind.db is never touched.
// Runs as npm's postinstall; the desktop app also runs it before starting the server.
import { copyFileSync, cpSync, existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bundle = path.join(ROOT, 'library');
const data = path.join(ROOT, 'data');

if (!existsSync(path.join(data, 'mymind.db')) && existsSync(path.join(bundle, 'mymind.db'))) {
  mkdirSync(data, { recursive: true });
  copyFileSync(path.join(bundle, 'mymind.db'), path.join(data, 'mymind.db'));
  if (existsSync(path.join(bundle, 'media'))) cpSync(path.join(bundle, 'media'), path.join(data, 'media'), { recursive: true });
  console.log('Installed the bundled library into data/mymind.db.');
}
