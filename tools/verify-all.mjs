// Runs every profile x suite through tools/drive.mjs (several at once, each with its own verifier browser)
// and prints one table of how many screenshots REA found pixel-identical.
// Usage: node tools/verify-all.mjs [--parallel 3] [--port-base 9333] [--reseed] [profile-suite ...] (default: all)
// --reseed records every live pass first, then backs up and reseeds data/mymind.db from those recordings and
// restarts the replica before any replica pass, so a library that changed since the last seed still compares.
import { spawn } from 'node:child_process';
import { createWriteStream } from 'node:fs';
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DRIVE = path.join(ROOT, 'evidence', 'drive');
const MATRIX = [
  ['desktop-dark', 'ui'],
  ['iphone', 'ui'],
  ['desktop-light', 'ui'],
  ['android', 'ui'],
  ['desktop-light', 'cards'],
  ['desktop-dark', 'cards'],
  ['iphone', 'cards'],
  ['android', 'cards'],
];
const { values: opts, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    parallel: { type: 'string', default: '3' },
    'port-base': { type: 'string', default: '9333' },
    reseed: { type: 'boolean', default: false },
  },
});
const jobs = positionals.length ? MATRIX.filter(([p, s]) => positionals.includes(`${p}-${s}`)) : [...MATRIX];

const up = () => fetch('http://localhost:4321/__replay/reload', { method: 'POST' }).then((r) => r.ok, () => false);
const run = (args) => new Promise((resolve) => spawn('node', args, { cwd: ROOT, stdio: 'inherit' }).on('exit', resolve));
let server = null;
async function startServer() {
  server = spawn('node', ['server.mjs'], { cwd: ROOT, env: { ...process.env, STORE_READONLY: '1' }, stdio: 'ignore' });
  while (!(await up())) await new Promise((r) => setTimeout(r, 200));
}
if (await up()) {
  if (opts.reseed) {
    console.error('--reseed restarts the replica after seeding: stop the server on :4321 first.');
    process.exit(2);
  }
} else await startServer();
await mkdir(DRIVE, { recursive: true });

const rows = [];
const liveSides = new Map();
// Run the queued jobs on `parallel` workers; `sides` limits each drive to one pass, with its own log suffix.
async function pool(queue, sides) {
  async function worker(slot) {
    for (let job = queue.shift(); job; job = queue.shift()) {
      const [profile, suite] = job;
      const log = path.join(DRIVE, `${profile}-${suite}${sides ? `.${sides}` : ''}.log`);
      console.log(`start ${profile}-${suite}${sides ? ` (${sides})` : ''} (log: ${path.relative(ROOT, log)})`);
      const child = spawn('node', ['tools/drive.mjs', '--profile', profile, '--suite', suite, ...(sides ? ['--sides', sides] : [])], {
        cwd: ROOT,
        env: { ...process.env, VERIFIER_PORT: String(Number(opts['port-base']) + slot) },
        stdio: ['ignore', 'pipe', 'pipe'],
      });
      const out = createWriteStream(log);
      child.stdout.pipe(out);
      child.stderr.pipe(out);
      const exit = await new Promise((resolve) => child.on('exit', resolve));
      const report = JSON.parse(await readFile(path.join(DRIVE, `${profile}-${suite}`, 'report.json'), 'utf8').catch(() => '{}'));
      if (sides === 'live') {
        liveSides.set(`${profile}-${suite}`, report.sides?.live);
        console.log(`done  ${profile}-${suite} live pass (exit ${exit})`);
        continue;
      }
      if (liveSides.has(`${profile}-${suite}`)) report.sides = { live: liveSides.get(`${profile}-${suite}`), ...report.sides };
      const results = report.results ?? [];
      const row = {
        run: `${profile}-${suite}`,
        exit,
        shots: results.length,
        identical: results.filter((r) => r.status === 'identical').length,
        different: results.filter((r) => r.status !== 'identical').map((r) => r.name),
        stepErrors: Object.values(report.sides ?? {}).flatMap((s) => s.errors),
        writesKeptOffAccount: report.sides?.live?.writesAnsweredLocally?.length ?? 0,
      };
      rows.push(row);
      console.log(`done  ${row.run}: ${row.identical}/${row.shots} identical, ${row.stepErrors.length} step errors`);
    }
  }
  await Promise.all(Array.from({ length: Number(opts.parallel) }, (_, slot) => worker(slot)));
}
try {
  if (opts.reseed) {
    await pool([...jobs], 'live');
    server.kill();
    await new Promise((r) => server.once('exit', r));
    const db = path.join(ROOT, 'data', 'mymind.db');
    await copyFile(db, `${db}.bak-${Date.now()}`).catch(() => {});
    if ((await run(['tools/seed-db.mjs'])) !== 0) throw new Error('seed-db failed; the backup is next to data/mymind.db');
    await startServer();
    await pool([...jobs], 'replica');
  } else await pool([...jobs]);
} finally {
  server?.kill();
}
// Merge with rows other verify processes wrote, so parallel invocations add up to one summary.
const previous = JSON.parse(await readFile(path.join(DRIVE, 'summary.json'), 'utf8').catch(() => '[]'));
const merged = [...previous.filter((row) => !rows.some((r) => r.run === row.run)), ...rows];
await writeFile(path.join(DRIVE, 'summary.json'), JSON.stringify(merged, null, 1));
console.log('\nrun                     identical    step errors  writes kept off account');
for (const r of rows) console.log(`${r.run.padEnd(24)}${`${r.identical}/${r.shots}`.padEnd(13)}${String(r.stepErrors.length).padEnd(13)}${r.writesKeptOffAccount}`);
