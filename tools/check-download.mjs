// Proves Rust download-server serves the app only to requests with a session or app token from the library's sign-in, against
// a scratch sign-in database.
import { deepStrictEqual, ok, strictEqual } from 'node:assert';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { once } from 'node:events';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { authFixture } from './lib/auth-fixture.mjs';
import { toolCommand } from './server-command.mjs';

const spare = http.createServer().listen(0, '127.0.0.1');
await new Promise((ready) => spare.once('listening', ready));
const port = spare.address().port;
await new Promise((closed) => spare.close(closed));

const dir = await mkdtemp(join(tmpdir(), 'mymind-download-'));
const SESSION = 'live-session-id';
const EXPIRED = 'expired-session-id';
const TOKEN = 'synthetic-live-app-token';
const EXPIRED_TOKEN = 'synthetic-expired-app-token';
const fixture = await authFixture(join(dir, 'auth.db'));
const db = fixture.db;
const insert = db.prepare('INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)');
const hash = (id) => createHash('sha256').update(id).digest('hex');
insert.run(hash(SESSION), 'password:tester', 'tester', Date.now(), Date.now() + 3_600_000, Date.now());
insert.run(hash(EXPIRED), 'password:tester', 'tester', Date.now() - 7_200_000, Date.now() - 3_600_000, Date.now() - 7_200_000);
const insertToken = db.prepare('INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)');
insertToken.run(hash(TOKEN), 'password:tester', 'tester', 'password', 'Synthetic Android', Date.now(), Date.now() + 3_600_000, Date.now());
insertToken.run(hash(EXPIRED_TOKEN), 'password:tester', 'tester', 'password', 'Synthetic Android', Date.now() - 7_200_000, Date.now() - 3_600_000, Date.now() - 7_200_000);
const app = Buffer.from('packaged app');
const apk = Buffer.from('synthetic Android package');
const built = new Date().toISOString();
const android = { name: 'mymind-android.apk', platform: 'android', label: 'Android', bytes: apk.length, sha256: hash(apk), built,
  versionName: '1.2.3', versionCode: 123, notes: ['Synthetic release'], history: [{ versionName: '1.2.3', versionCode: 123, date: built, notes: ['Synthetic release'] }] };
await writeFile(join(dir, 'mymind-linux-x86_64.tar.gz'), app);
await writeFile(join(dir, android.name), apk);
await writeFile(join(dir, 'manifest.json'), JSON.stringify({ library: 'https://mymind.example.com', files: [
  { name: 'mymind-linux-x86_64.tar.gz', label: 'Linux (x86_64)', bytes: app.length, sha256: 'abc', built }, android,
] }));
const command = toolCommand('download-server', []);
const server = spawn(command.command, command.args, {
  env: { PATH: process.env.PATH, MYMIND_ENV_FILE: join(dir, 'absent.env'), DOWNLOAD_PORT: String(port), DOWNLOAD_DIR: dir, AUTH_DB: join(dir, 'auth.db') },
  stdio: ['ignore', 'pipe', 'inherit'],
});
await new Promise((ready, fail) => {
  const timer = setTimeout(() => fail(new Error('Rust download-server did not start within 10 seconds')), 10000);
  server.stdout.once('data', () => { clearTimeout(timer); ready(); });
  server.once('error', error => { clearTimeout(timer); fail(error); });
  server.once('exit', code => { clearTimeout(timer); fail(new Error(`Rust download-server exited before ready (${code})`)); });
});

const base = `http://127.0.0.1:${port}`;
const get = (path, session = SESSION, init = {}) => fetch(base + path, { redirect: 'manual', signal: AbortSignal.timeout(10000), ...init, headers: {
  ...(session && { cookie: `theme=dark; mm_session=${session}` }), ...init.headers } });
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
  await check('an app bearer token downloads the Android package and release description', async () => {
    const headers = { authorization: `Bearer ${TOKEN}` };
    const release = await get('/download/android.json', null, { headers });
    strictEqual(release.status, 200);
    strictEqual(release.headers.get('cache-control'), 'no-store');
    const { versionName, versionCode, bytes, sha256, built: released, notes, history } = android;
    deepStrictEqual(await release.json(), { versionName, versionCode, url: `/download/${android.name}`, bytes, sha256, built: released, notes, history });
    const res = await get(`/download/${android.name}`, null, { headers });
    strictEqual(res.status, 200);
    strictEqual(res.headers.get('content-type'), 'application/vnd.android.package-archive');
    ok(Buffer.from(await res.arrayBuffer()).equals(apk));
  });
  await check('expired, unknown and revoked app bearer tokens cannot download', async () => {
    for (const token of [EXPIRED_TOKEN, 'unknown-app-token']) {
      strictEqual((await get('/download/android.json', null, { headers: { authorization: `Bearer ${token}` } })).status, 401);
      strictEqual((await get(`/download/${android.name}`, null, { headers: { authorization: `Bearer ${token}` } })).status, 401);
    }
    db.prepare('DELETE FROM app_tokens WHERE hash = ?').run(hash(TOKEN));
    strictEqual((await get(`/download/${android.name}`, null, { headers: { authorization: `Bearer ${TOKEN}` } })).status, 401);
  });
  await check('only packaged files are served', async () => {
    strictEqual((await get('/download/manifest.json')).status, 404);
    strictEqual((await get('/download/..%2Fdownload-server.mjs')).status, 404);
  });
  await check('writes are refused', async () => strictEqual((await get('/download', SESSION, { method: 'POST' })).status, 405));
} finally {
  if (server.exitCode === null && server.signalCode === null) {
    const exited = once(server, 'exit');
    server.kill();
    await exited;
  }
  fixture.close();
  await rm(dir, { recursive: true, force: true });
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'}`);
process.exitCode = failures.length ? 1 : 0;
