// The JS side of the media differential tests (tests/media.rs). Usage:
//   node media.mjs fixtures <empty directory>   writes the fixture files and prints their names as JSON
//   node media.mjs run <job.json>               runs the job's operations against lib/local-media.mjs
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { copyFileSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { Writable } from 'node:stream';
import { pathToFileURL } from 'node:url';

const repo = process.env.MYMIND_ROOT;
const [command, argument] = process.argv.slice(2);

async function fixtures(dir) {
  const sharp = createRequire(join(repo, 'package.json'))('sharp');
  const names = [];
  const add = (name, data) => { writeFileSync(join(dir, name), data); names.push(name); };
  const quadrants = (width, height, channels) => {
    const raw = Buffer.alloc(width * height * channels);
    for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
      const at = (y * width + x) * channels;
      const colour = [x * 255 / width, y * 255 / height, (x < width / 2) === (y < height / 2) ? 220 : 40, 200 - (x + y) % 120];
      for (let band = 0; band < channels; band++) raw[at + band] = colour[band];
    }
    return raw;
  };
  const make = (width, height, channels) => sharp(quadrants(width, height, channels), { raw: { width, height, channels } });
  const tasks = [];
  for (let orientation = 1; orientation <= 8; orientation++) {
    tasks.push(async () => add(`exif-${orientation}.jpg`, await make(120, 80, 3).jpeg().withMetadata({ orientation }).toBuffer()));
  }
  tasks.push(
    async () => add('plain.jpg', await make(1600, 1200, 3).jpeg({ quality: 90 }).toBuffer()),
    async () => add('alpha.png', await make(200, 150, 4).png().toBuffer()),
    async () => add('grey.png', await make(200, 150, 1).png().toBuffer()),
    async () => add('palette.png', await make(200, 150, 3).png({ palette: true }).toBuffer()),
    async () => add('deep.png', await sharp(Buffer.from(new Uint16Array(160 * 100 * 3).map((_, index) => index * 7 % 65535).buffer), { raw: { width: 160, height: 100, channels: 3, depth: 'ushort' } }).png().toBuffer()),
    async () => add('anim.gif', await make(160, 100, 3).gif().toBuffer()),
    async () => add('alpha.webp', await make(300, 200, 4).webp().toBuffer()),
    async () => add('photo.webp', await make(900, 600, 3).webp({ quality: 70 }).toBuffer()),
    async () => add('image.avif', await make(128, 96, 3).avif().toBuffer()),
    async () => add('image.tiff', await make(128, 96, 3).tiff().toBuffer()),
    async () => add('tall.jpg', await make(300, 2400, 3).jpeg().toBuffer()),
    async () => add('tiny.png', await make(1, 1, 3).png().toBuffer()),
  );
  for (const task of tasks) await task();
  const jpeg = readFileSync(join(dir, 'plain.jpg'));
  add('image.svg', '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30" fill="red"/></svg>');
  add('truncated.jpg', jpeg.subarray(0, Math.floor(jpeg.length * 0.6)));
  add('garbage.bin', Buffer.from(Array.from({ length: 600 }, (_, index) => (index * 131 + 17) % 256)));
  add('empty.bin', Buffer.alloc(0));
  const header = (width, height) => {
    const ihdr = Buffer.alloc(13);
    ihdr.writeUInt32BE(width, 0); ihdr.writeUInt32BE(height, 4); ihdr.set([8, 2, 0, 0, 0], 8);
    const crc = (buffer) => { let c, value = 0xffffffff; for (const byte of buffer) { c = (value ^ byte) & 0xff; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; value = (value >>> 8) ^ c; } return (value ^ 0xffffffff) >>> 0; };
    const length = Buffer.alloc(4); length.writeUInt32BE(13);
    const body = Buffer.concat([Buffer.from('IHDR'), ihdr]);
    const sum = Buffer.alloc(4); sum.writeUInt32BE(crc(body));
    return Buffer.concat([Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), length, body, sum]);
  };
  add('zero-width.png', header(0, 50));
  add('huge.png', header(20000, 20000));
  const pages = 3;
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>', `<< /Type /Pages /Kids [${Array.from({ length: pages }, (_, index) => `${3 + index * 2} 0 R`).join(' ')}] /Count ${pages} >>`];
  for (let index = 0; index < pages; index++) {
    const stream = `${0.2 + index * 0.3} 0.4 0.8 rg 40 40 ${200 + index * 50} 300 re f BT /F1 36 Tf 60 400 Td (Page ${index + 1}) Tj ET`;
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${300 + index * 40} 500] /Contents ${4 + index * 2} 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>`);
    objects.push(`<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`);
  }
  let pdf = '%PDF-1.4\n';
  const offsets = objects.map((object, index) => { const at = pdf.length; pdf += `${index + 1} 0 obj\n${object}\nendobj\n`; return at; });
  const xref = pdf.length;
  pdf += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n${offsets.map((at) => `${String(at).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  add('pages.pdf', pdf);
  const ffmpeg = (name, args) => {
    try { execFileSync('ffmpeg', ['-v', 'error', '-y', ...args, join(dir, name)]); names.push(name); } catch { /* the encoder is missing */ }
  };
  ffmpeg('clip.mp4', ['-f', 'lavfi', '-i', 'testsrc2=size=320x180:rate=10:duration=2', '-pix_fmt', 'yuv420p', '-c:v', 'libx264', '-movflags', '+faststart']);
  ffmpeg('wide.mp4', ['-f', 'lavfi', '-i', 'testsrc=size=642x360:rate=8:duration=1', '-f', 'lavfi', '-i', 'sine=duration=1', '-pix_fmt', 'yuv420p', '-c:v', 'libx264', '-c:a', 'aac', '-shortest']);
  ffmpeg('clip.webm', ['-f', 'lavfi', '-i', 'testsrc2=size=256x144:rate=10:duration=1', '-c:v', 'libvpx', '-b:v', '300k']);
  ffmpeg('audio-only.mp4', ['-f', 'lavfi', '-i', 'sine=duration=1', '-c:a', 'aac']);
  const real = [];
  const walk = (folder) => { for (const entry of readdirSync(folder, { withFileTypes: true })) (entry.isDirectory() ? walk : (path) => real.push(path))(join(folder, entry.name)); };
  walk(join(repo, 'capture/media'));
  real.sort();
  const kinds = { jpg: [], webp: [], png: [], pdf: [], mp4: [] };
  for (const path of real) {
    const size = statSync(path).size;
    if (size > 12_000_000) continue;
    const head = readFileSync(path).subarray(0, 16);
    const kind = head.subarray(0, 3).equals(Buffer.from([0xff, 0xd8, 0xff])) ? 'jpg' : head.toString('latin1', 0, 4) === 'RIFF' ? 'webp'
      : head.toString('latin1', 1, 4) === 'PNG' ? 'png' : head.toString('latin1', 0, 5) === '%PDF-' ? 'pdf' : head.toString('latin1', 4, 8) === 'ftyp' ? 'mp4' : null;
    if (kind) kinds[kind].push(path);
  }
  const spread = (list, count) => list.filter((_, index) => index % Math.max(1, Math.floor(list.length / count)) === 0).slice(0, count);
  let number = 0;
  for (const [kind, count] of [['jpg', 18], ['webp', 14], ['png', 6], ['pdf', 3], ['mp4', 3]]) {
    for (const path of spread(kinds[kind], count)) {
      const name = `real-${String(number++).padStart(2, '0')}.${kind}`;
      copyFileSync(path, join(dir, name));
      names.push(name);
    }
  }
  return names;
}

class Capture extends Writable {
  constructor() { super(); this.chunks = []; }
  writeHead(status, headers) { this.status = status; this.headers = Object.entries(headers); }
  _write(chunk, encoding, done) { this.chunks.push(chunk); done(); }
}

async function run(jobFile) {
  const job = JSON.parse(readFileSync(jobFile, 'utf8'));
  const { createMedia, analysisImage } = await import(pathToFileURL(join(repo, 'lib/local-media.mjs')));
  const media = createMedia(job.root);
  const strip = (file) => (file === null ? null : file.slice(job.root.length + 1));
  const results = [];
  for (const op of job.ops) {
    try {
      if (op.op === 'seed') {
        mkdirSync(dirname(join(job.root, op.to)), { recursive: true });
        copyFileSync(op.from, join(job.root, op.to));
        results.push(null);
      } else if (op.op === 'prepare') {
        const input = op.bytes ? readFileSync(op.file) : { file: op.file };
        results.push(await media.prepare(input, 'application/octet-stream', op.name, op.user, op.id));
      } else if (op.op === 'response') {
        const found = await media.response(op.relative);
        results.push(found ? { file: strip(found.file), size: found.size, type: found.type } : null);
      } else if (op.op === 'send') {
        const found = await media.response(op.relative);
        const res = new Capture();
        await media.send(res, found, { method: op.method, range: op.range ?? undefined });
        await new Promise((resolve) => res.writableFinished ? resolve() : res.once('finish', resolve));
        results.push({ status: res.status, headers: res.headers, body: Buffer.concat(res.chunks).toString('base64') });
      } else if (op.op === 'analysis') {
        const made = await analysisImage(op.file);
        writeFileSync(op.out, made.data);
        results.push({ mime: made.mime });
      } else if (op.op === 'image') {
        const made = await media.image(op.value);
        if (made) writeFileSync(op.out, made.data);
        results.push(made ? { mime: made.mime } : null);
      }
    } catch (error) {
      results.push({ error: { status: error.status ?? null, message: error.status ? error.message : 'unexpected' } });
    }
  }
  return results;
}

if (command === 'fixtures') console.log(JSON.stringify(await fixtures(argument)));
else console.log(JSON.stringify(await run(argument)));
