// Measures the media container on two images (the JS one and the Rust one): image size, cold start to /health, idle
// memory, and the median time of each media operation through the Worker's media client. Writes JSON for
// docs/rust-benchmarks.md. Containers publish on 127.0.0.1 only; the two screenshots of example.com need the internet.
//
//   node tools/bench-container.mjs --js <image> --rust <image> --jpeg <file> --mp4 <file> --pdf <file>
//     [--runs 10] [--rounds 2] [--starts 5] [--out evidence/bench-container.json]
import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { mediaClient } from '../worker/media-client.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { values: opts } = parseArgs({ options: {
  js: { type: 'string' }, rust: { type: 'string' }, jpeg: { type: 'string' }, mp4: { type: 'string' }, pdf: { type: 'string' },
  runs: { type: 'string', default: '10' }, rounds: { type: 'string', default: '2' }, starts: { type: 'string', default: '5' },
  out: { type: 'string', default: 'evidence/bench-container.json' },
} });
for (const name of ['js', 'rust', 'jpeg', 'mp4', 'pdf']) if (!opts[name]) throw new Error(`--${name} is required`);
const RUNS = Number(opts.runs);
const docker = (...args) => execFileSync('docker', args, { encoding: 'utf8' }).trim();
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const mib = (usage) => { const [, value, unit] = usage.match(/^([\d.]+)\s*([KMG]i?B)/); return Number(value) * { KiB: 1 / 1024, MiB: 1, GiB: 1024 }[unit]; };

// Start a container and return its id, its port and the milliseconds until /health answered.
async function start(image) {
  const began = performance.now();
  const id = docker('run', '-d', '--rm', '-p', '127.0.0.1::8080', image);
  const port = docker('port', id, '8080/tcp').split('\n')[0].split(':').pop();
  for (;;) {
    try { if ((await fetch(`http://127.0.0.1:${port}/health`)).ok) return { id, port, ms: performance.now() - began }; } catch {}
    if (performance.now() - began > 30000) throw new Error(`${image} did not answer /health`);
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
}

async function operations(port) {
  const env = { MEDIA: { idFromName: (name) => name, get: () => ({ fetch: (url, init) => fetch(url.replace('http://media', `http://127.0.0.1:${port}`), { ...init, duplex: 'half' }) }) } };
  const client = mediaClient(env);
  const user = { accountUid: 'account', idPrefix: 'prefix-' };
  const rows = {};
  const time = async (name, run) => {
    const times = [];
    for (let i = 0; i < RUNS; i++) { const began = performance.now(); await run(); times.push(performance.now() - began); }
    rows[name] = median(times);
  };
  for (const [label, file, type] of [['jpeg', opts.jpeg, 'image/jpeg'], ['mp4', opts.mp4, 'video/mp4'], ['pdf', opts.pdf, 'application/pdf']]) {
    const bytes = await readFile(file);
    const made = await client.processor.prepare(bytes, type, 'x', user, 'card');
    await time(`prepare ${label}`, () => client.processor.prepare(bytes, type, 'x', user, 'card'));
    const cover = made.extras?.cover?.src?.replace(/^\/media\//, '');
    if (cover) await time(`cover rendition ${label}`, () => client.processor.rendition({ name: made.data.blob.path, body: bytes }, cover));
    if (type === 'image/jpeg') {
      await time('400x400 webp rendition jpeg', () => client.processor.rendition({ name: made.data.blob.path, body: bytes }, `${made.data.blob.path};400x400-t.webp`));
      await time('image() jpeg', () => client.processor.image(bytes));
    }
  }
  await time('screenshot of its own /health (offline)', () => client.screenshot('http://127.0.0.1:8080/health', { offline: true }));
  await time('screenshot example.com', () => client.screenshot('https://example.com/'));
  await time('resource example.com', () => client.resource('https://example.com/'));
  return rows;
}

const report = { at: new Date().toISOString(), runs: RUNS, images: {} };
for (const [backend, image] of [['js', opts.js], ['rust', opts.rust]]) {
  const starts = [];
  for (let i = 0; i < Number(opts.starts); i++) { const started = await start(image); starts.push(started.ms); docker('rm', '-f', started.id); }
  report.images[backend] = { image, sizeBytes: Number(docker('image', 'inspect', image, '--format', '{{.Size}}')), coldStartMs: median(starts), rounds: [] };
}
for (let round = 0; round < Number(opts.rounds); round++) {
  for (const backend of ['js', 'rust']) {
    const entry = report.images[backend];
    const { id, port } = await start(entry.image);
    try {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      const idleMib = mib(docker('stats', '--no-stream', '--format', '{{.MemUsage}}', id));
      const medians = await operations(port);
      entry.rounds.push({ idleMib, afterMib: mib(docker('stats', '--no-stream', '--format', '{{.MemUsage}}', id)), medians });
      console.log(`${backend} round ${round + 1}: idle ${idleMib.toFixed(1)} MiB`);
      for (const [name, ms] of Object.entries(medians)) console.log(`  ${name.padEnd(40)} ${ms.toFixed(1)} ms`);
    } finally {
      docker('rm', '-f', id);
    }
  }
}
for (const [backend, entry] of Object.entries(report.images)) console.log(`${backend}: ${(entry.sizeBytes / 1e9).toFixed(2)} GB, cold start ${entry.coldStartMs.toFixed(0)} ms`);
await writeFile(path.resolve(ROOT, opts.out), `${JSON.stringify(report, null, 2)}\n`);
