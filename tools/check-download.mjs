// Proves deploy/download-server.mjs serves the app only to the owner's valid Cloudflare Access token, against a
// stand-in for Cloudflare's signing keys; it never contacts Cloudflare.
import { ok, strictEqual } from 'node:assert';
import { spawn } from 'node:child_process';
import { generateKeyPairSync, sign } from 'node:crypto';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const TEAM = 'team.cloudflareaccess.test';
const AUD = 'test-audience';
const OWNER = 'owner@example.com';
const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
const { privateKey: strangerKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });

// An RS256 Access token carrying `claims` over valid ones, signed with the stand-in's key or `key`.
function token(claims = {}, key = privateKey) {
  const now = Math.floor(Date.now() / 1000);
  const encode = (value) => Buffer.from(JSON.stringify(value)).toString('base64url');
  const body = `${encode({ alg: 'RS256', kid: 'test-key', typ: 'JWT' })}.${encode({ iss: `https://${TEAM}`, aud: [AUD], email: OWNER, iat: now, nbf: now, exp: now + 3600, type: 'app', ...claims })}`;
  return `${body}.${sign('RSA-SHA256', Buffer.from(body), key).toString('base64url')}`;
}

const certs = http.createServer((_req, res) => {
  res.writeHead(200, { 'content-type': 'application/json' });
  res.end(JSON.stringify({ keys: [{ ...publicKey.export({ format: 'jwk' }), kid: 'test-key', alg: 'RS256' }] }));
}).listen(0, '127.0.0.1');
await new Promise((ready) => certs.once('listening', ready));

const spare = http.createServer().listen(0, '127.0.0.1');
await new Promise((ready) => spare.once('listening', ready));
const port = spare.address().port;
await new Promise((closed) => spare.close(closed));

const dir = await mkdtemp(join(tmpdir(), 'mymind-download-'));
const app = Buffer.from('packaged app');
await writeFile(join(dir, 'mymind-linux-x86_64.tar.gz'), app);
await writeFile(join(dir, 'manifest.json'), JSON.stringify({ library: 'https://mymind.example.com', files: [
  { name: 'mymind-linux-x86_64.tar.gz', label: 'Linux (x86_64)', bytes: app.length, sha256: 'abc', built: new Date().toISOString() },
] }));
const server = spawn(process.execPath, ['deploy/download-server.mjs'], {
  env: { PATH: process.env.PATH, CF_ACCESS_TEAM_DOMAIN: TEAM, CF_ACCESS_AUD: AUD, OWNER_EMAIL: OWNER, DOWNLOAD_PORT: String(port), DOWNLOAD_DIR: dir,
    CF_ACCESS_CERTS_URL: `http://127.0.0.1:${certs.address().port}/cdn-cgi/access/certs` },
  stdio: ['ignore', 'pipe', 'inherit'],
});
await new Promise((ready) => server.stdout.once('data', ready));

const base = `http://127.0.0.1:${port}`;
const get = (path, jwt, init = {}) => fetch(base + path, { ...init, headers: jwt ? { 'cf-access-jwt-assertion': jwt } : {} });
const failures = [];
async function check(label, fn) {
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`); }
}

try {
  await check('without an Access token the page is refused', async () => strictEqual((await get('/download')).status, 403));
  await check('without an Access token the file is refused', async () => strictEqual((await get('/download/mymind-linux-x86_64.tar.gz')).status, 403));
  await check('a token signed by another key is refused', async () => strictEqual((await get('/download', token({}, strangerKey))).status, 403));
  await check('another application\'s token is refused', async () => strictEqual((await get('/download', token({ aud: ['other'] }))).status, 403));
  await check('another team\'s token is refused', async () => strictEqual((await get('/download', token({ iss: 'https://evil.cloudflareaccess.com' }))).status, 403));
  await check('an expired token is refused', async () => strictEqual((await get('/download', token({ exp: Math.floor(Date.now() / 1000) - 60 }))).status, 403));
  await check('another person\'s token is refused', async () => strictEqual((await get('/download', token({ email: 'someone@example.com' }))).status, 403));
  await check('a token without a signature is refused', async () => strictEqual((await get('/download', token().replace(/\.[^.]+$/, '.'))).status, 403));
  await check('the owner gets the download page', async () => {
    const res = await get('/download', token());
    strictEqual(res.status, 200);
    const html = await res.text();
    ok(html.includes('href="/download/mymind-linux-x86_64.tar.gz"') && html.includes(OWNER));
  });
  await check('the owner downloads the app', async () => {
    const res = await get('/download/mymind-linux-x86_64.tar.gz', token());
    strictEqual(res.status, 200);
    ok(res.headers.get('content-disposition').includes('attachment'));
    ok(Buffer.from(await res.arrayBuffer()).equals(app));
  });
  await check('only packaged files are served', async () => {
    strictEqual((await get('/download/manifest.json', token())).status, 404);
    strictEqual((await get('/download/..%2Fdownload-server.mjs', token())).status, 404);
  });
  await check('writes are refused', async () => strictEqual((await get('/download', token(), { method: 'POST' })).status, 405));
} finally {
  server.kill();
  certs.close();
  await rm(dir, { recursive: true, force: true });
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'}`);
process.exitCode = failures.length ? 1 : 0;
