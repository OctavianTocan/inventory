// Restores data/ from the local snapshot in library/ (written by npm run bundle) when data/mymind.db is missing; does
// nothing without a snapshot. Runs as npm's postinstall; the desktop app also runs it before starting the server.
import { copyFileSync, cpSync, existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.argv.includes('--help')) {
  console.log('Usage: npm run postinstall\nRestores the local library snapshot only when data/mymind.db is absent.');
  process.exit(0);
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bundle = path.join(ROOT, 'library');
const data = path.join(ROOT, 'data');

if (!existsSync(path.join(data, 'mymind.db')) && existsSync(path.join(bundle, 'mymind.db'))) {
  mkdirSync(data, { recursive: true });
  copyFileSync(path.join(bundle, 'mymind.db'), path.join(data, 'mymind.db'));
  if (existsSync(path.join(bundle, 'media'))) cpSync(path.join(bundle, 'media'), path.join(data, 'media'), { recursive: true });
  console.log('Installed the bundled library into data/mymind.db.');
}
