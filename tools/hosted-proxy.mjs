// A signed-in front for a server with sign-in (the Worker under `wrangler dev`, or mymind serve with AUTH=google), so
// tools written for the open replica (tools/drive.mjs, tools/compare-hosted.mjs) run against it unchanged:
//   node tools/hosted-proxy.mjs <target url> [--port 4321] [--backend rust|worker]
// Pass --server hosted-rust or --server hosted-worker to drive/verify-all to identify this proxy explicitly.
// Signs in once as TEST_USER_NAME / TEST_USER_PASSWORD from .env and adds that session cookie to every request and
// /hub WebSocket that has none of its own. Listens on 127.0.0.1 only.
import http from 'node:http';
import https from 'node:https';
import net from 'node:net';
import path from 'node:path';
import tls from 'node:tls';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { verificationDigest, verificationMode } from './digests.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
if (process.env.STORE_LOAD_ENV !== '0') {
  try { process.loadEnvFile(path.join(ROOT, '.env')); } catch (error) { if (error.code !== 'ENOENT') throw new Error('Could not load .env'); }
}
const { values, positionals: [target] } = parseArgs({ allowPositionals: true, options: {
  port: { type: 'string', default: '4321' }, backend: { type: 'string', default: 'rust' },
} });
if (!target) {
  console.error('Usage: node tools/hosted-proxy.mjs <target url> [--port 4321] [--backend rust|worker]');
  process.exit(2);
}
if (!process.env.TEST_USER_NAME || !process.env.TEST_USER_PASSWORD) throw new Error('TEST_USER_NAME and TEST_USER_PASSWORD must be set');
const upstream = new URL(target);
const mode = verificationMode(`hosted-${values.backend}`);
const identity = { proxy: 'mymind-hosted-proxy', server: mode, digest: await verificationDigest(mode), target: upstream.origin };
const secure = upstream.protocol === 'https:';
const SESSION = 'mm_session';
const HOP = new Set(['connection', 'keep-alive', 'proxy-connection', 'transfer-encoding', 'upgrade', 'te', 'trailer']);
let cookie = null;
let signingIn = null;

// Sign in as the test user and keep the session cookie.
async function signIn() {
  const response = await fetch(new URL('/signin/password', upstream), {
    method: 'POST', redirect: 'manual', headers: { 'content-type': 'application/x-www-form-urlencoded', origin: upstream.origin },
    body: new URLSearchParams({ username: process.env.TEST_USER_NAME, password: process.env.TEST_USER_PASSWORD }),
  });
  const session = response.headers.getSetCookie().map((line) => line.split(';')[0]).find((pair) => pair.startsWith(`${SESSION}=`) && pair.length > SESSION.length + 1);
  if (!session) throw new Error(`Test user sign-in failed: ${response.status} ${response.headers.get('location') ?? ''}`);
  cookie = session;
}
// Coalesce concurrent sign-ins while keeping credentials and the resulting cookie private to this process.
const ensureSession = () => (cookie ? Promise.resolve() : (signingIn ??= signIn().finally(() => { signingIn = null; })));

// The request headers to send upstream: the client's, addressed to the target, with the test session added when it
// brought none.
function upstreamHeaders(headers) {
  const out = {};
  for (const [name, value] of Object.entries(headers)) if (!HOP.has(name)) out[name] = value;
  out.host = upstream.host;
  // The target refuses writes from other origins, and to it the proxy's own origin is one.
  if (/^http:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/.test(out.origin ?? '')) out.origin = upstream.origin;
  const cookies = (out.cookie ?? '').split(/;\s*/).filter(Boolean);
  if (!cookies.some((pair) => pair.startsWith(`${SESSION}=`))) out.cookie = [...cookies, cookie].join('; ');
  return out;
}

// Send one buffered request upstream and resolve with its response.
function forward(req, body) {
  return new Promise((resolve, reject) => {
    const headers = upstreamHeaders(req.headers);
    if (body.length || req.method === 'POST' || req.method === 'PUT' || req.method === 'PATCH') headers['content-length'] = String(body.length);
    const request = (secure ? https : http).request({ host: upstream.hostname, port: upstream.port || (secure ? 443 : 80), method: req.method, path: req.url,
      headers, servername: upstream.hostname }, resolve);
    request.on('error', reject);
    request.end(body);
  });
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === 'GET' && req.url === '/__mymind/verification') {
      res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'no-store' });
      res.end(JSON.stringify(identity));
      return;
    }
    const chunks = [];
    for await (const chunk of req) chunks.push(chunk);
    const body = Buffer.concat(chunks);
    await ensureSession();
    let response = await forward(req, body);
    // The session can end under a run (a sign-out test, an expiry); sign in again once and retry.
    if (response.statusCode === 401 && !/(^|;\s*)mm_session=/.test(req.headers.cookie ?? '')) {
      response.resume();
      cookie = null;
      await ensureSession();
      response = await forward(req, body);
    }
    const headers = { ...response.headers };
    for (const name of HOP) delete headers[name];
    res.writeHead(response.statusCode, response.statusMessage, headers);
    response.pipe(res);
  } catch (error) {
    console.error(`[proxy] ${req.method} ${req.url}`, error.message);
    if (!res.headersSent) res.writeHead(502, { 'content-type': 'text/plain' });
    res.end(String(error.message));
  }
});

// Pass WebSocket upgrades (the /hub) through as raw bytes once the upgrade request has the session cookie.
server.on('upgrade', async (req, socket, head) => {
  socket.on('error', () => {});
  try { await ensureSession(); } catch (error) { console.error('[proxy] upgrade', error.message); return socket.destroy(); }
  const port = Number(upstream.port || (secure ? 443 : 80));
  const remote = secure ? tls.connect({ host: upstream.hostname, port, servername: upstream.hostname }) : net.connect({ host: upstream.hostname, port });
  remote.on('error', () => socket.destroy());
  remote.once(secure ? 'secureConnect' : 'connect', () => {
    const headers = upstreamHeaders(req.headers);
    headers.connection = 'Upgrade';
    headers.upgrade = req.headers.upgrade;
    remote.write(`${req.method} ${req.url} HTTP/1.1\r\n${Object.entries(headers).map(([name, value]) => `${name}: ${value}`).join('\r\n')}\r\n\r\n`);
    if (head.length) remote.write(head);
    socket.pipe(remote).pipe(socket);
  });
  socket.on('close', () => remote.destroy());
});

await ensureSession();
server.listen(Number(values.port), '127.0.0.1', () => console.log(`signed-in proxy for ${upstream.origin} on http://localhost:${values.port}`));
