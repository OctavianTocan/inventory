// Proves Google sign-in against a local stand-in for Google (its own keys, token endpoint and JWKS): the gate,
// the PKCE/state/nonce checks, every ID-token rejection, owner-only access, sessions, sign-out and the hub.
import { ok, strictEqual } from 'node:assert';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
import http from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { testServer } from './http-check.mjs';

const OWNER = 'owner@example.com';
const CLIENT_ID = 'test-client.apps.googleusercontent.com';
const ISSUER = 'https://accounts.google.test';
const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
const { privateKey: strangerKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
const codes = new Map();

// Sign claims as an RS256 JWT with the stand-in's key (or another one, to forge it).
function idToken(claims, key = privateKey) {
  const encode = (value) => Buffer.from(JSON.stringify(value)).toString('base64url');
  const body = `${encode({ alg: 'RS256', kid: 'test-key', typ: 'JWT' })}.${encode(claims)}`;
  return `${body}.${sign('RSA-SHA256', Buffer.from(body), key).toString('base64url')}`;
}

const idp = http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://idp');
  if (url.pathname === '/certs') {
    res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'max-age=60' });
    return res.end(JSON.stringify({ keys: [{ ...publicKey.export({ format: 'jwk' }), kid: 'test-key', alg: 'RS256', use: 'sig' }] }));
  }
  if (url.pathname === '/token' && req.method === 'POST') {
    let text = '';
    for await (const chunk of req) text += chunk;
    const form = new URLSearchParams(text);
    const grant = codes.get(form.get('code'));
    codes.delete(form.get('code'));
    const challenge = createHash('sha256').update(form.get('code_verifier') ?? '').digest('base64url');
    if (!grant || challenge !== grant.challenge || form.get('client_id') !== CLIENT_ID || form.get('client_secret') !== 'test-secret' || form.get('redirect_uri') !== grant.redirectUri) {
      res.writeHead(400, { 'content-type': 'application/json' });
      return res.end('{"error":"invalid_grant"}');
    }
    res.writeHead(200, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ access_token: 'unused', id_token: grant.token }));
  }
  res.writeHead(404);
  res.end();
});
idp.listen(0, '127.0.0.1');
await new Promise((ready) => idp.once('listening', ready));
const IDP = `http://127.0.0.1:${idp.address().port}`;

const authDir = await mkdtemp(join(tmpdir(), 'mymind-auth-'));
const server = await testServer({ readonly: true, env: {
  AUTH: 'google', GOOGLE_CLIENT_ID: CLIENT_ID, GOOGLE_CLIENT_SECRET: 'test-secret', OWNER_EMAIL: OWNER, AUTH_DB: join(authDir, 'auth.db'),
  GOOGLE_AUTH_URL: `${IDP}/authorize`, GOOGLE_TOKEN_URL: `${IDP}/token`, GOOGLE_CERTS_URL: `${IDP}/certs`, GOOGLE_ISSUER: ISSUER,
} });
const get = (path, headers = {}) => fetch(server.base + path, { redirect: 'manual', headers, signal: AbortSignal.timeout(10000) });
const setCookie = (res, name) => res.headers.getSetCookie().map((c) => c.split(';')[0]).find((c) => c.startsWith(`${name}=`));

// Start a sign-in like mymind's Google button does; returns what Google would see plus the browser's state cookie.
async function begin() {
  const res = await get('/google/oauth2-redirect?mode=popup&createAccount=false&popup=true');
  strictEqual(res.status, 302);
  const location = new URL(res.headers.get('location'));
  strictEqual(location.origin + location.pathname, `${IDP}/authorize`);
  const params = Object.fromEntries(location.searchParams);
  strictEqual(params.code_challenge_method, 'S256');
  strictEqual(params.client_id, CLIENT_ID);
  return { ...params, cookie: setCookie(res, 'mm_oauth') };
}

// Finish a sign-in whose ID token carries `claims` (merged over valid ones); returns the callback response.
async function finish(start, claims = {}, { key, cookie = start.cookie, state = start.state } = {}) {
  const now = Math.floor(Date.now() / 1000);
  const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1234567890', email: OWNER, email_verified: true, nonce: start.nonce, iat: now, exp: now + 3600, ...claims }, key);
  const code = `code-${Math.random()}`;
  codes.set(code, { token, challenge: start.code_challenge, redirectUri: start.redirect_uri });
  return get(`/google/oauth2-callback?code=${code}&state=${state}`, cookie ? { cookie } : {});
}

// Open a hub WebSocket upgrade; resolves true when the server switches protocols.
function hubOpens(cookie) {
  return new Promise((resolve) => {
    const req = http.request(`${server.base}/hub`, { headers: { connection: 'Upgrade', upgrade: 'websocket', 'sec-websocket-version': '13', 'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==', ...(cookie ? { cookie } : {}) } });
    req.on('upgrade', (_res, socket) => { socket.destroy(); resolve(true); });
    req.on('error', () => resolve(false));
    req.on('close', () => resolve(false));
    req.end();
  });
}

const failures = [];
async function check(label, fn) {
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.split('\n')[0]}`); }
}
const rejected = async (res, error = 'SignInFailed') => {
  strictEqual(res.status, 200);
  ok(!setCookie(res, 'mm_session'), 'no session cookie');
  ok((await res.text()).includes(`/signin?error=${error}&identityProvider=Google`), `points back to the ${error} page`);
};

let session;
try {
  await check('signed-out page load redirects to the sign-in page', async () => {
    const res = await get('/everything', { accept: 'text/html', 'sec-fetch-dest': 'document' });
    strictEqual(res.status, 302);
    strictEqual(res.headers.get('location'), '/signin');
  });
  await check('signed-out API request is refused', async () => strictEqual((await get('/cards', { accept: 'application/msgpack' })).status, 401));
  await check('sign-in page is mymind\'s, wired to this server', async () => {
    const res = await get('/signin', { accept: 'text/html' });
    strictEqual(res.status, 200);
    const html = await res.text();
    ok(html.includes('signin-button-google') && html.includes('/_auth/signin.js') && !html.includes('appleid.cdn-apple.com'));
  });
  await check('sign-in error page shows mymind\'s message', async () => ok((await (await get('/signin?error=AccountNotFound&identityProvider=Google')).text()).includes('could not find an account')));
  await check('signed-out hub upgrade is refused', async () => strictEqual(await hubOpens(), false));
  await check('forged ID token is rejected', async () => rejected(await finish(await begin(), {}, { key: strangerKey })));
  await check('wrong nonce is rejected', async () => rejected(await finish(await begin(), { nonce: 'other' })));
  await check('wrong audience is rejected', async () => rejected(await finish(await begin(), { aud: 'someone-else' })));
  await check('wrong issuer is rejected', async () => rejected(await finish(await begin(), { iss: 'https://evil.test' })));
  await check('expired ID token is rejected', async () => rejected(await finish(await begin(), { exp: Math.floor(Date.now() / 1000) - 3600 })));
  await check('callback without the browser\'s state cookie is rejected', async () => rejected(await finish(await begin(), {}, { cookie: null })));
  await check('unknown state is rejected', async () => rejected(await finish(await begin(), {}, { state: 'made-up' })));
  await check('another Google account is not let in', async () => rejected(await finish(await begin(), { email: 'someone@example.com' }), 'AccountNotFound'));
  await check('unverified owner email is not let in', async () => rejected(await finish(await begin(), { email_verified: false }), 'AccountNotFound'));
  await check('owner signs in', async () => {
    const res = await finish(await begin());
    strictEqual(res.status, 200);
    session = setCookie(res, 'mm_session');
    ok(session, 'session cookie set');
    ok(res.headers.getSetCookie().some((c) => c.startsWith('mm_session=') && /HttpOnly/.test(c) && /SameSite=Lax/.test(c)));
    ok((await res.text()).includes('"/everything"'));
  });
  await check('a used state cannot be replayed', async () => {
    const start = await begin();
    await finish(start);
    await rejected(await finish(start));
  });
  await check('signed-in owner gets the library', async () => {
    strictEqual((await get('/everything', { accept: 'text/html', 'sec-fetch-dest': 'document', cookie: session })).status, 200);
    strictEqual((await get('/cards', { accept: 'application/msgpack', cookie: session })).status, 200);
  });
  await check('signed-in owner skips the sign-in page', async () => strictEqual((await get('/signin', { cookie: session })).headers.get('location'), '/everything'));
  await check('signed-in hub upgrade connects', async () => strictEqual(await hubOpens(session), true));
  await check('cross-origin write is refused', async () => {
    const res = await fetch(`${server.base}/cards/x/tags`, { method: 'POST', headers: { cookie: session, origin: 'https://evil.test' }, body: '{}' });
    strictEqual(res.status, 403);
  });
  await check('sign-out ends the session', async () => {
    const res = await get('/signout', { cookie: session });
    strictEqual(res.headers.get('location'), '/signin');
    strictEqual((await get('/cards', { accept: 'application/msgpack', cookie: session })).status, 401);
  });
} finally {
  await server.close();
  idp.close();
  await rm(authDir, { recursive: true, force: true });
}

const open = await testServer({ readonly: true });
try {
  await check('without sign-in, tunnelled requests are refused', async () => strictEqual((await fetch(`${open.base}/everything`, { headers: { 'cf-connecting-ip': '203.0.113.9' } })).status, 403));
  await check('without sign-in, local requests work as before', async () => strictEqual((await fetch(`${open.base}/everything`, { headers: { accept: 'text/html', 'sec-fetch-dest': 'document' } })).status, 200));
} finally {
  await open.close();
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'}`);
process.exitCode = failures.length ? 1 : 0;
