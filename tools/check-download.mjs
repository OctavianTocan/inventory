// Proves deploy/download-server.mjs serves the app only to requests with a session from the library's sign-in, against
// a scratch sign-in database.
import { ok, strictEqual } from 'node:assert';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { openAuthStore } from '../lib/auth-store.mjs';
import { toolCommand } from './server-command.mjs';

const spare = http.createServer().listen(0, '127.0.0.1');
await new Promise((ready) => spare.once('listening', ready));
const port = spare.address().port;
await new Promise((closed) => spare.close(closed));

const dir = await mkdtemp(join(tmpdir(), 'mymind-download-'));
const SESSION = 'live-session-id';
const EXPIRED = 'expired-session-id';
const db = openAuthStore(join(dir, 'auth.db'));
const insert = db.prepare('INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)');
const hash = (id) => createHash('sha256').update(id).digest('hex');
insert.run(hash(SESSION), 'password:tester', 'tester', Date.now(), Date.now() + 3_600_000, Date.now());
insert.run(hash(EXPIRED), 'password:tester', 'tester', Date.now() - 7_200_000, Date.now() - 3_600_000, Date.now() - 7_200_000);
db.close();
const app = Buffer.from('packaged app');
await writeFile(join(dir, 'mymind-linux-x86_64.tar.gz'), app);
await writeFile(join(dir, 'manifest.json'), JSON.stringify({ library: 'https://mymind.example.com', files: [
  { name: 'mymind-linux-x86_64.tar.gz', label: 'Linux (x86_64)', bytes: app.length, sha256: 'abc', built: new Date().toISOString() },
] }));
const command = toolCommand('download-server', []);
const server = spawn(command.command, command.args, {
  env: { PATH: process.env.PATH, DOWNLOAD_PORT: String(port), DOWNLOAD_DIR: dir, AUTH_DB: join(dir, 'auth.db') },
  stdio: ['ignore', 'pipe', 'inherit'],
});
await new Promise((ready) => server.stdout.once('data', ready));

const base = `http://127.0.0.1:${port}`;
const get = (path, session = SESSION, init = {}) => fetch(base + path, { redirect: 'manual', ...init, headers: {
  ...(session && { cookie: `theme=dark; mm_session=${session}` }) } });
const failures = [];
async function check(label, fn) {
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`); }
}

try {
  await check('signed out, the page sends you to sign in', async () => {
    const res = await get('/download', null);
    strictEqual(res.status, 302);
    strictEqual(res.headers.get('location'), '/signin');
  });
  await check('signed out, the file is refused', async () => strictEqual((await get('/download/mymind-linux-x86_64.tar.gz', null)).status, 401));
  await check('an expired or unknown session is refused', async () => {
    strictEqual((await get('/download', EXPIRED)).status, 302);
    strictEqual((await get('/download/mymind-linux-x86_64.tar.gz', 'made-up')).status, 401);
  });
  await check('a signed-in user gets the download page', async () => {
    const res = await get('/download');
    strictEqual(res.status, 200);
    const html = await res.text();
    ok(html.includes('href="/download/mymind-linux-x86_64.tar.gz"') && html.includes('Signed in as tester'));
  });
  await check('a signed-in user downloads the app', async () => {
    const res = await get('/download/mymind-linux-x86_64.tar.gz');
    strictEqual(res.status, 200);
    ok(res.headers.get('content-disposition').includes('attachment'));
    ok(Buffer.from(await res.arrayBuffer()).equals(app));
  });
  await check('only packaged files are served', async () => {
    strictEqual((await get('/download/manifest.json')).status, 404);
    strictEqual((await get('/download/..%2Fdownload-server.mjs')).status, 404);
  });
  await check('writes are refused', async () => strictEqual((await get('/download', SESSION, { method: 'POST' })).status, 405));
} finally {
  server.kill();
  await rm(dir, { recursive: true, force: true });
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'}`);
process.exitCode = failures.length ? 1 : 0;
