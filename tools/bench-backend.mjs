// Measures the Rust backend on immutable fixture copies: start time, latency, CPU time and memory per
// heavy route, in the replica's library mode and in hosted mode. Writes JSON for docs/rust-benchmarks.md.
//
// Compare with docs/rust-benchmarks.md for historical pre-switch measurements.
//   node tools/bench-backend.mjs [--baseline <executable>] [--rust rust/target/release/mymind] [--runs 30] [--out evidence/bench.json]
import { execFileSync, spawn } from 'node:child_process';
import { cp, mkdir, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { values: opts } = parseArgs({ options: {
  rust: { type: 'string', default: 'rust/target/release/mymind' },
  baseline: { type: 'string' },
  runs: { type: 'string', default: '30' },
  starts: { type: 'string', default: '5' },
  out: { type: 'string', default: 'evidence/bench-backend.json' },
} });
const RUNS = Number(opts.runs);
const FIXTURES = path.join(ROOT, 'rust/test-fixtures');
const WORK = path.join(ROOT, 'rust/target/tmp', `bench-${process.pid}`);
const TICK_MS = 10;
const READY = /mymind replica on http:\/\/localhost:(\d+)\//;
const BACKENDS = { rust: [path.resolve(ROOT, opts.rust), ['serve']] };
if (opts.baseline) BACKENDS.baseline = [path.resolve(ROOT, opts.baseline), ['serve']];

const ids = (file, sql) => { const db = new DatabaseSync(file, { readOnly: true }); try { return db.prepare(sql).all().map((row) => row.id); } finally { db.close(); } };

// A fresh copy of the fixture a mode serves, and the variables that point a server at it.
async function library(mode, name) {
  const directory = path.join(WORK, `${mode}-${name}`);
  await mkdir(directory, { recursive: true });
  if (mode === 'library') {
    await cp(path.join(FIXTURES, 'mymind.db'), path.join(directory, 'mymind.db'));
    return { file: path.join(directory, 'mymind.db'), env: { STORE_DB: path.join(directory, 'mymind.db'), STORE_READONLY: '1' } };
  }
  await cp(path.join(FIXTURES, 'hosted-test.db'), path.join(directory, 'test.db'));
  execFileSync('cp', ['-a', '--reflink=auto', path.join(FIXTURES, 'hosted-bucket'), path.join(directory, 'bucket')]);
  return { file: path.join(directory, 'test.db'), env: { HOSTED: '1', HOSTED_DIR: directory, HOSTED_LIBRARY: 'test', STORE_READONLY: '1' } };
}

function start(backend, env) {
  const [command, args] = BACKENDS[backend];
  const began = performance.now();
  const child = spawn(command, args, { cwd: ROOT, stdio: ['ignore', 'pipe', 'pipe'],
    env: { PATH: process.env.PATH, HOME: process.env.HOME, MYMIND_ROOT: ROOT, STORE_LOAD_ENV: '0', MYMIND_ENV_FILE: path.join(WORK, 'absent-env'), OFFLINE: '1', PORT: '0', ...env } });
  return new Promise((resolve, reject) => {
    let output = '';
    const timer = setTimeout(() => reject(new Error(`${backend} did not start: ${output}`)), 60000);
    child.stderr.on('data', (chunk) => { output += chunk; });
    child.stdout.on('data', (chunk) => {
      output += chunk;
      const port = output.match(READY)?.[1];
      if (port) { clearTimeout(timer); resolve({ child, base: `http://127.0.0.1:${port}`, startMs: performance.now() - began }); }
    });
    child.once('exit', (code) => { clearTimeout(timer); reject(new Error(`${backend} exited ${code}: ${output}`)); });
  });
}

async function stop(child) {
  child.removeAllListeners('exit');
  const exited = new Promise((resolve) => child.once('exit', resolve));
  child.kill('SIGTERM');
  await exited;
}

// User plus system CPU time of a process and its threads, in milliseconds.
async function cpuMs(pid) {
  const stat = await readFile(`/proc/${pid}/stat`, 'utf8');
  const fields = stat.slice(stat.lastIndexOf(')') + 2).split(' ');
  return (Number(fields[11]) + Number(fields[12])) * TICK_MS;
}

async function memory(pid) {
  const status = await readFile(`/proc/${pid}/status`, 'utf8');
  const kb = (name) => Number(status.match(new RegExp(`^${name}:\\s+(\\d+) kB`, 'm'))?.[1] ?? 0);
  return { rssMb: kb('VmRSS') / 1024, peakMb: kb('VmHWM') / 1024 };
}

const quantile = (sorted, q) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];

async function send(base, request) {
  const response = await fetch(base + request.target, { method: request.method ?? 'GET', headers: request.headers, body: request.body });
  const bytes = (await response.arrayBuffer()).byteLength;
  return { status: response.status, bytes };
}

// The largest media file of the hosted test library, as its `/media/` path.
async function largestMedia() {
  const base = path.join(FIXTURES, 'hosted-bucket/libraries/test/source/media');
  let best = { size: 0, name: '' };
  for (const name of await readdir(base, { recursive: true })) {
    const info = await stat(path.join(base, name));
    if (info.isFile() && info.size > best.size) best = { size: info.size, name };
  }
  return `/media/${best.name}`;
}

async function routes(mode, file) {
  const cards = ids(file, 'SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 4');
  const json = { accept: 'application/json' };
  const list = [
    { name: 'GET /cards', target: '/cards', headers: json },
    { name: 'GET /cards (MessagePack)', target: '/cards', headers: {} },
    { name: 'GET /changes?since=0', target: '/changes?since=0', headers: json },
    { name: 'GET /cards/:id/detail', target: `/cards/${cards[0]}/detail`, headers: json },
    { name: 'GET /search?q=design', target: '/search?q=design', headers: json },
    { name: 'GET /spaces', target: '/spaces', headers: json },
    { name: 'GET /tags/manual', target: '/tags/manual', headers: json },
    { name: 'GET /embeddings/related/:id', target: `/embeddings/related/${cards[0]}`, headers: json },
    { name: 'POST /cards/grid', target: '/cards/grid', method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cards) },
    { name: 'GET /everything (page)', target: '/everything', headers: { 'sec-fetch-dest': 'document' } },
  ];
  if (mode === 'hosted') {
    list.push({ name: 'GET /_static/<589 KB script>', target: '/_static/134/0.109.0/scripts/vendor/lottie.js', headers: {} });
    list.push({ name: 'GET /media/<largest>', target: await largestMedia(), headers: {} });
  }
  return list;
}

async function measure(backend, mode) {
  const starts = [];
  for (let i = 0; i < Number(opts.starts); i++) {
    const { env } = await library(mode, `${backend}-start-${i}`);
    const server = await start(backend, env);
    starts.push(server.startMs);
    await stop(server.child);
  }
  const { file, env } = await library(mode, backend);
  const server = await start(backend, env);
  const pid = server.child.pid;
  const idle = await memory(pid);
  const results = [];
  for (const request of await routes(mode, file)) {
    const first = await send(server.base, request);
    for (let i = 0; i < 3; i++) await send(server.base, request);
    const times = [];
    const cpuBefore = await cpuMs(pid);
    const window = performance.now();
    // CPU time counts in 10 ms ticks, so fast routes repeat for at least two seconds.
    while (times.length < RUNS || (performance.now() - window < 2000 && times.length < 5000)) {
      const began = performance.now();
      await send(server.base, request);
      times.push(performance.now() - began);
    }
    const cpu = (await cpuMs(pid)) - cpuBefore;
    times.sort((a, b) => a - b);
    results.push({ route: request.name, status: first.status, bytes: first.bytes, requests: times.length, medianMs: quantile(times, 0.5), p95Ms: quantile(times, 0.95), cpuMsPerRequest: cpu / times.length });
  }
  const end = await memory(pid);
  await stop(server.child);
  starts.sort((a, b) => a - b);
  return { backend, mode, startMs: quantile(starts, 0.5), idleRssMb: idle.rssMb, peakRssMb: end.peakMb, routes: results };
}

try {
  const report = { at: new Date().toISOString(), minimumRuns: RUNS, rust: opts.rust, results: [] };
  for (const mode of ['library', 'hosted']) {
    for (const backend of Object.keys(BACKENDS)) {
      const result = await measure(backend, mode);
      report.results.push(result);
      console.log(`${mode} ${backend}: start ${result.startMs.toFixed(0)} ms, idle ${result.idleRssMb.toFixed(0)} MB, peak ${result.peakRssMb.toFixed(0)} MB`);
      for (const route of result.routes) {
        console.log(`  ${route.route.padEnd(32)} ${route.status} ${String(route.bytes).padStart(9)} B  median ${route.medianMs.toFixed(1)} ms  p95 ${route.p95Ms.toFixed(1)} ms  cpu ${route.cpuMsPerRequest.toFixed(1)} ms`);
      }
    }
  }
  await mkdir(path.dirname(path.resolve(ROOT, opts.out)), { recursive: true });
  await writeFile(path.resolve(ROOT, opts.out), `${JSON.stringify(report, null, 2)}\n`);
} finally {
  await rm(WORK, { recursive: true, force: true });
}
