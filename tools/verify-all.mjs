// Runs every profile x suite through tools/drive.mjs (several at once, each with its own verifier browser)
// and prints one table of how many screenshots REA found pixel-identical.
// Usage: node tools/verify-all.mjs [--parallel 2] [--port-base 9333] [--reseed | --replica-only]
//        [--server rust|hosted-rust|hosted-worker] [run ...]
// Runs are `profile-suite` names (default: the whole matrix); patch runs such as `iphone-cards@media` work too.
// The replica it starts reads data/verify.db, a database seeded from the recordings (created on first use), never
// the working library in data/mymind.db.
// --reseed records every live pass first, then reseeds data/verify.db from those recordings and restarts the
// replica before any replica pass, so a library that changed since the last seed still compares.
// --replica-only re-runs only the replica passes against each run's recorded live pass, e.g. after a code change.
// The replica it starts runs with OFFLINE=1, which REA's coverage needs to prove it renders without mymind.
import { spawn } from 'node:child_process';
import { createWriteStream, existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { serverCommand, toolCommand } from './server-command.mjs';
import { assertReplicaServer } from './server-identity.mjs';
import { verificationMode } from './digests.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DRIVE = path.join(ROOT, 'evidence', 'drive');
const VERIFY_DB = path.join(ROOT, 'data', 'verify.db');
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
    parallel: { type: 'string', default: '2' },
    'port-base': { type: 'string', default: '9333' },
    reseed: { type: 'boolean', default: false },
    'replica-only': { type: 'boolean', default: false },
    server: { type: 'string', default: 'rust' },
  },
});
const mode = verificationMode(opts.server);
if (opts.reseed && mode !== 'rust') throw new Error('--reseed requires --server rust; hosted verification uses an already running signed-in proxy');
const PROFILES = [...new Set(MATRIX.map(([profile]) => profile))];
const parseRun = (run) => {
  const profile = PROFILES.find((p) => run.startsWith(`${p}-`));
  if (!profile) throw new Error(`Unknown run ${run}; runs look like desktop-light-ui or iphone-cards@media`);
  return [profile, run.slice(profile.length + 1)];
};
const jobs = positionals.length ? positionals.map(parseRun) : [...MATRIX];

const up = () => fetch('http://localhost:4321/__replay/reload', { method: 'POST' }).then((r) => (r.ok ? r.json() : null), () => null);
let server = null;
// Seed and start only the direct native backend; hosted modes require an existing identified proxy.
async function startServer() {
  const seed = toolCommand('seed-db', ['--db', VERIFY_DB]);
  const seeded = existsSync(VERIFY_DB) || await new Promise((done) => spawn(seed.command, seed.args, { cwd: ROOT, stdio: 'inherit' }).on('exit', (code) => done(code === 0)));
  if (!seeded) throw new Error('seed-db failed');
  const { command, args } = serverCommand();
  server = spawn(command, args, { cwd: ROOT, env: { ...process.env, OFFLINE: '1', STORE_READONLY: '1', STORE_DB: VERIFY_DB }, stdio: 'ignore' });
  while (!(await up())) await new Promise((r) => setTimeout(r, 200));
  await assertReplicaServer(4321, mode);
}
const proxy = mode === 'rust' ? null : await assertReplicaServer(4321, mode);
const running = mode === 'hosted-worker' ? { offline: false, store: null, proxyTarget: proxy.target } : await up();
if (running) {
  await assertReplicaServer(4321, mode);
  if (opts.reseed) {
    console.error('--reseed restarts the replica after seeding: stop the server on :4321 first.');
    process.exit(2);
  }
  if (!running.offline) console.error('warning: the server on :4321 is not OFFLINE=1, so coverage cannot prove offline rendering.');
  if (running.store !== VERIFY_DB) console.error('warning: the server on :4321 does not read data/verify.db (STORE_DB), so it compares your working library.');
} else if (mode === 'rust') await startServer();
else throw new Error('The selected hosted proxy did not answer replica readiness; start it and its backend before verifying');
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
      const child = spawn('node', ['tools/drive.mjs', '--profile', profile, '--suite', suite, '--server', mode, ...(sides ? ['--sides', sides] : [])], {
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
    const seed = toolCommand('seed-db', ['--db', VERIFY_DB]);
    const seeded = await new Promise((resolve) => {
      const child = spawn(seed.command, seed.args, { cwd: ROOT, stdio: 'inherit' });
      child.once('error', () => resolve(false));
      child.once('exit', code => resolve(code === 0));
    });
    if (!seeded) throw new Error('seed-db failed');
    await startServer();
    await pool([...jobs], 'replica');
  } else await pool([...jobs], opts['replica-only'] ? 'replica' : undefined);
} finally {
  server?.kill();
}
// Merge with rows other verify processes wrote, so parallel invocations add up to one summary.
const previous = JSON.parse(await readFile(path.join(DRIVE, 'summary.json'), 'utf8').catch(() => '[]'));
const merged = [...previous.filter((row) => !rows.some((r) => r.run === row.run)), ...rows];
await writeFile(path.join(DRIVE, 'summary.json'), JSON.stringify(merged, null, 1));
console.log('\nrun                     identical    step errors  writes kept off account');
for (const r of rows) console.log(`${r.run.padEnd(24)}${`${r.identical}/${r.shots}`.padEnd(13)}${String(r.stepErrors.length).padEnd(13)}${r.writesKeptOffAccount}`);
