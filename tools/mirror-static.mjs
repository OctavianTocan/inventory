// Downloads every static asset the live page loaded into mirror/static, so `OFFLINE=1 npm run serve` works.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MIRROR = path.join(ROOT, 'mirror', 'static');
const urls = JSON.parse(await readFile(path.join(ROOT, 'capture', 'static-urls.json'), 'utf8'));

let mirrored = 0;
for (const url of urls) {
  const target = path.resolve(MIRROR, decodeURIComponent(new URL(url).pathname.slice(1)));
  if (!target.startsWith(MIRROR + path.sep)) continue;
  const res = await fetch(url);
  if (!res.ok) {
    console.warn(`${res.status} ${url}`);
    continue;
  }
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, Buffer.from(await res.arrayBuffer()));
  mirrored++;
}
console.log(`Mirrored ${mirrored}/${urls.length} assets into ${MIRROR}`);
