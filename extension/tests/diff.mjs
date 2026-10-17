import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import sharp from 'sharp';
import { repo } from './harness.mjs';

/** Compares the ours-* and ref-* screenshots pairwise and writes diff-* images; prints the differing pixel count per pair. */
const dir = join(repo, 'evidence', 'extension', 'shots');
for (const name of readdirSync(dir).filter((file) => file.startsWith('ours-') && !file.includes('dbg')).sort()) {
  const other = name.replace('ours-', 'ref-');
  const a = await sharp(join(dir, name)).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  const b = await sharp(join(dir, other)).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  const { width, height } = a.info;
  const out = Buffer.alloc(width * height * 4, 255);
  let count = 0;
  let minX = width, minY = height, maxX = 0, maxY = 0;
  for (let i = 0; i < width * height; i++) {
    const delta = Math.abs(a.data[i * 4] - b.data[i * 4]) + Math.abs(a.data[i * 4 + 1] - b.data[i * 4 + 1]) + Math.abs(a.data[i * 4 + 2] - b.data[i * 4 + 2]);
    if (delta > 12) {
      count++;
      const x = i % width, y = Math.floor(i / width);
      minX = Math.min(minX, x); maxX = Math.max(maxX, x); minY = Math.min(minY, y); maxY = Math.max(maxY, y);
      out[i * 4] = 255; out[i * 4 + 1] = 0; out[i * 4 + 2] = 0;
    } else out[i * 4] = out[i * 4 + 1] = out[i * 4 + 2] = 255 - Math.round(a.data[i * 4] * 0.1);
  }
  await sharp(out, { raw: { width, height, channels: 4 } }).png().toFile(join(dir, name.replace('ours-', 'diff-')));
  console.log(name.padEnd(34), String(count).padStart(6), count ? `box ${minX},${minY}-${maxX},${maxY}` : '');
}
