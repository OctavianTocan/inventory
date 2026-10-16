// Proves the native app's API (docs/app-api.md) against a local stand-in for Google: nonces, Google and password
// sign-in, bearer tokens on API, media and hub, sign-out, user removal, the tunnel's reach, and the typed card reads.
import { deepStrictEqual, ok, strictEqual } from 'node:assert';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
import { EventEmitter } from 'node:events';
import http from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { openAuthStore, removeUser, setPassword } from '../lib/auth-store.mjs';
import { createHub } from '../lib/hub.mjs';
import { imageCover } from '../lib/render/media.mjs';
import { msgpack, testServer } from './http-check.mjs';

const OWNER = 'owner@example.com';
const CLIENT_ID = 'test-client.apps.googleusercontent.com';
const ISSUER = 'https://accounts.google.test';
const NONCE_MS = 1500;
const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
const { privateKey: strangerKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });

// Sign claims as an RS256 JWT with the stand-in's key (or another one, or another key id, to forge it).
function idToken(claims, key = privateKey, kid = 'test-key') {
  const encode = (value) => Buffer.from(JSON.stringify(value)).toString('base64url');
  const body = `${encode({ alg: 'RS256', kid, typ: 'JWT' })}.${encode(claims)}`;
  return `${body}.${sign('RSA-SHA256', Buffer.from(body), key).toString('base64url')}`;
}

let certFetches = 0;
const idp = http.createServer((req, res) => {
  if (req.url !== '/certs') { res.writeHead(404); return res.end(); }
  certFetches += 1;
  res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'max-age=60' });
  res.end(JSON.stringify({ keys: [{ ...publicKey.export({ format: 'jwk' }), kid: 'test-key', alg: 'RS256', use: 'sig' }] }));
});
idp.listen(0, '127.0.0.1');
await new Promise((ready) => idp.once('listening', ready));
const IDP = `http://127.0.0.1:${idp.address().port}`;

const spare = http.createServer().listen(0, '127.0.0.1');
await new Promise((ready) => spare.once('listening', ready));
const LOCAL = `http://127.0.0.1:${spare.address().port}`;
await new Promise((closed) => spare.close(closed));

const authDir = await mkdtemp(join(tmpdir(), 'mymind-app-api-'));
const authPath = join(authDir, 'auth.db');
const server = await testServer({ readonly: true, env: {
  LOCAL_PORT: LOCAL.split(':').pop(), AUTH: 'google', GOOGLE_CLIENT_ID: CLIENT_ID, GOOGLE_CLIENT_SECRET: 'test-secret', OWNER_EMAIL: OWNER, AUTH_DB: authPath,
  GOOGLE_AUTH_URL: `${IDP}/authorize`, GOOGLE_TOKEN_URL: `${IDP}/token`, GOOGLE_CERTS_URL: `${IDP}/certs`, GOOGLE_ISSUER: ISSUER, APP_NONCE_MS: String(NONCE_MS),
} });

const TUNNELLED = { 'cf-connecting-ip': '203.0.113.9' };
const bearer = (token) => ({ authorization: `Bearer ${token}` });
const call = (base, path, { method = 'GET', body, headers = {} } = {}) => fetch(base + path, { method, redirect: 'manual', signal: AbortSignal.timeout(10000),
  headers: { ...(body === undefined ? {} : { 'content-type': 'application/json' }), ...headers }, ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
const get = (path, headers) => call(server.base, path, { headers });
const post = (path, body, headers) => call(server.base, path, { method: 'POST', body: body ?? {}, headers });
const local = (value) => (value === undefined ? undefined : JSON.parse(JSON.stringify(value).replaceAll('https://media.mymind.host/', '/_mediahost/')));
const sha256 = (text) => createHash('sha256').update(text).digest('hex');

const nonce = async (headers) => (await (await post('/app/v1/auth/nonce', {}, headers)).json()).nonce;

// Sign in with Google through the app endpoint; `claims` merge over valid ones.
async function googleSignIn(claims = {}, { key, device = 'Test phone', headers } = {}) {
  const issued = await nonce(headers);
  const now = Math.floor(Date.now() / 1000);
  const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1234567890', email: OWNER, email_verified: true, nonce: issued, iat: now, exp: now + 3600, ...claims }, key);
  return post('/app/v1/auth/google', { idToken: token, nonce: issued, device }, headers);
}

// Open a hub WebSocket upgrade; resolves true when the server switches protocols.
function hubOpens(headers = {}, base = server.base) {
  return new Promise((resolve) => {
    const req = http.request(`${base}/hub`, { headers: { connection: 'Upgrade', upgrade: 'websocket', 'sec-websocket-version': '13', 'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==', ...headers } });
    req.on('upgrade', (_res, socket) => { socket.destroy(); resolve(true); });
    req.on('error', () => resolve(false));
    req.on('close', () => resolve(false));
    req.end();
  });
}

// Send a raw request with an exact path and body split into `parts`; resolves the status, or null when the connection drops.
function rawRequest(path, { method = 'GET', headers = {}, parts = [] } = {}) {
  return new Promise((resolve) => {
    const size = parts.reduce((sum, part) => sum + part.length, 0);
    const req = http.request(`${server.base}/`, { method, path, headers: { ...(parts.length ? { 'content-length': size } : {}), ...headers } }, (res) => { res.resume(); resolve(res.statusCode); });
    req.on('error', () => resolve(null));
    const [first, ...rest] = parts;
    if (first) req.write(first);
    setTimeout(() => { for (const part of rest) req.write(part); req.end(); }, 100);
  });
}

// Every card the web streams at `path`, as ids in order.
async function webIds(base, path = '/cards') {
  const res = await fetch(base + path, { headers: { accept: 'application/msgpack' } });
  return [...msgpack.decodeMulti(Buffer.from(await res.arrayBuffer()))].map((item) => /data-id="([^"]+)"/.exec(item.html)[1]);
}

// Walk a paged list to its end; returns the cards and the number of pages.
async function walk(base, path, limit, headers = {}) {
  const cards = [];
  let cursor = null;
  let pages = 0;
  do {
    const res = await call(base, `${path}${path.includes('?') ? '&' : '?'}limit=${limit}${cursor ? `&cursor=${cursor}` : ''}`, { headers });
    strictEqual(res.status, 200);
    const page = await res.json();
    ok(page.cards.length <= limit);
    cards.push(...page.cards);
    cursor = page.next;
    pages += 1;
    ok(pages < 5000, 'terminates');
  } while (cursor);
  return { cards, pages };
}

const failures = [];
let total = 0;
async function check(label, fn) {
  total += 1;
  try { await fn(); console.log(`PASS  ${label}`); }
  catch (error) { failures.push(label); console.log(`FAIL  ${label}: ${error.message.slice(0, 700)}`); }
}
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));
const PASSWORD = 'correct horse battery staple';
const form = (headers) => ({ 'content-type': 'application/x-www-form-urlencoded', ...headers });
const webPassword = (username, password, headers) => fetch(`${server.base}/signin/password`, { method: 'POST', redirect: 'manual', headers: form(headers), body: new URLSearchParams({ username, password }) });

const users = openAuthStore(authPath);
const library = new DatabaseSync(server.dbPath, { readOnly: true });
let owner;
try {
  await check('nonce is random, base64url and expires in the future', async () => {
    const res = await post('/app/v1/auth/nonce');
    strictEqual(res.status, 200);
    strictEqual(res.headers.get('content-type'), 'application/json; charset=utf-8');
    strictEqual(res.headers.get('cache-control'), 'no-store');
    const body = await res.json();
    ok(/^[A-Za-z0-9_-]{20,}$/.test(body.nonce) && body.nonce !== await nonce());
    ok(Date.parse(body.expiresAt) > Date.now());
  });
  await check('sign-in endpoints take POST only and refuse cross-origin posts', async () => {
    strictEqual((await get('/app/v1/auth/nonce')).status, 405);
    strictEqual((await post('/app/v1/auth/nonce', {}, { origin: 'https://evil.test' })).status, 403);
  });
  await check('owner signs in with Google and gets a token', async () => {
    const res = await googleSignIn();
    strictEqual(res.status, 200);
    const body = await res.json();
    ok(/^[A-Za-z0-9_-]{43}$/.test(body.token), '256 bits as base64url');
    deepStrictEqual(body.user, { email: OWNER, kind: 'google' });
    const days = (Date.parse(body.expiresAt) - Date.now()) / 86_400_000;
    ok(days > 89 && days <= 90, `expires in ${days.toFixed(2)} days`);
    owner = body.token;
  });
  await check('the token is stored only as its SHA-256, with the device label', async () => {
    const rows = users.prepare('SELECT * FROM app_tokens').all();
    strictEqual(rows.length, 1);
    strictEqual(rows[0].hash, sha256(owner));
    strictEqual(rows[0].device, 'Test phone');
    strictEqual(rows[0].kind, 'google');
    ok(!JSON.stringify(rows).includes(owner));
  });
  await check('a nonce works once', async () => {
    const issued = await nonce();
    const now = Math.floor(Date.now() / 1000);
    const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1', email: OWNER, email_verified: true, nonce: issued, iat: now, exp: now + 3600 });
    strictEqual((await post('/app/v1/auth/google', { idToken: token, nonce: issued, device: 'a' })).status, 200);
    const replay = await post('/app/v1/auth/google', { idToken: token, nonce: issued, device: 'a' });
    strictEqual(replay.status, 400);
    deepStrictEqual(await replay.json(), { error: 'invalid-token' });
  });
  await check('a nonce the server never issued is refused', async () => {
    const now = Math.floor(Date.now() / 1000);
    const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1', email: OWNER, email_verified: true, nonce: 'made-up', iat: now, exp: now + 3600 });
    strictEqual((await post('/app/v1/auth/google', { idToken: token, nonce: 'made-up' })).status, 400);
  });
  await check('a nonce expires', async () => {
    const issued = await nonce();
    await sleep(NONCE_MS + 300);
    const now = Math.floor(Date.now() / 1000);
    const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1', email: OWNER, email_verified: true, nonce: issued, iat: now, exp: now + 3600 });
    const res = await post('/app/v1/auth/google', { idToken: token, nonce: issued });
    strictEqual(res.status, 400);
    deepStrictEqual(await res.json(), { error: 'invalid-token' });
  });
  await check('a verified Google account without access gets 401 not-allowed', async () => {
    const res = await googleSignIn({ email: 'someone@example.com' });
    strictEqual(res.status, 401);
    deepStrictEqual(await res.json(), { error: 'not-allowed' });
  });
  const invalid = [
    ['forged signature', {}, { key: strangerKey }],
    ['wrong audience', { aud: 'someone-else' }],
    ['wrong issuer', { iss: 'https://evil.test' }],
    ['expired token', { exp: Math.floor(Date.now() / 1000) - 3600 }],
    ['nonce that differs from the issued one', { nonce: 'other' }],
    ['unverified email', { email_verified: false }],
  ];
  for (const [label, claims, options] of invalid) {
    await check(`${label} gets 400 invalid-token`, async () => {
      const res = await googleSignIn(claims, options);
      strictEqual(res.status, 400);
      deepStrictEqual(await res.json(), { error: 'invalid-token' });
    });
  }
  await check('unknown key ids refetch Google\'s keys at most once a minute', async () => {
    const before = certFetches;
    const forged = async (_, i) => {
      const issued = await nonce();
      const now = Math.floor(Date.now() / 1000);
      const token = idToken({ iss: ISSUER, aud: CLIENT_ID, sub: '1', email: OWNER, email_verified: true, nonce: issued, iat: now, exp: now + 3600 }, privateKey, `unknown-${i}`);
      return (await post('/app/v1/auth/google', { idToken: token, nonce: issued })).status;
    };
    deepStrictEqual(await Promise.all(Array.from({ length: 8 }, forged)), Array(8).fill(400));
    for (let i = 8; i < 12; i++) strictEqual(await forged(null, i), 400);
    ok(certFetches - before <= 1, `${certFetches - before} key fetches for 12 forged tokens`);
    strictEqual((await googleSignIn()).status, 200, 'the real key still works');
  });
  await check('malformed sign-in bodies get 400 invalid-token', async () => {
    strictEqual((await post('/app/v1/auth/google', { nonce: await nonce() })).status, 400);
    const raw = await fetch(`${server.base}/app/v1/auth/google`, { method: 'POST', body: 'not json', headers: { 'content-type': 'application/json' } });
    strictEqual(raw.status, 400);
  });
  await check('no token, or a dead one, gets the 401 JSON', async () => {
    for (const headers of [{}, bearer('nope'), { ...bearer('nope'), accept: 'text/html', 'sec-fetch-dest': 'document' }]) {
      const res = await get('/app/v1/cards', headers);
      strictEqual(res.status, 401);
      strictEqual(res.headers.get('content-type'), 'application/json');
    }
  });
  await check('the bearer token opens API routes', async () => {
    const res = await get('/app/v1/cards?limit=1', bearer(owner));
    strictEqual(res.status, 200);
    strictEqual(res.headers.get('content-type'), 'application/json; charset=utf-8');
    strictEqual(res.headers.get('cache-control'), 'no-store');
    strictEqual((await get('/cards', { ...bearer(owner), accept: 'application/msgpack' })).status, 200);
    deepStrictEqual(await (await get('/app/v1/me', bearer(owner))).json(), { email: OWNER, kind: 'google' });
  });
  await check('the bearer token opens media routes', async () => {
    strictEqual((await get('/media/not-a-real-file.jpg')).status, 401);
    ok((await get('/media/not-a-real-file.jpg', bearer(owner))).status !== 401);
  });
  await check('the bearer token opens the hub', async () => {
    strictEqual(await hubOpens(), false);
    strictEqual(await hubOpens(bearer('nope')), false);
    strictEqual(await hubOpens(bearer(owner)), true);
  });
  await check('a bearer token after several spaces is accepted', async () => {
    strictEqual((await get('/app/v1/cards?limit=1', { authorization: `Bearer   ${owner}` })).status, 200);
  });
  await check('a malformed cookie neither crashes the server nor hides a valid token', async () => {
    strictEqual(await hubOpens({ cookie: 'unrelated=%' }), false);
    strictEqual(await hubOpens({ cookie: 'unrelated=%', ...bearer(owner) }), true);
    strictEqual((await get('/app/v1/cards?limit=1', { cookie: 'unrelated=%; mm_session=%E0%A4%A', ...bearer(owner) })).status, 200);
    strictEqual((await get('/app/v1/cards?limit=1', { cookie: 'mm_session=%' })).status, 401);
  });
  await check('an unparsable request path gets 400 and the server keeps running', async () => {
    strictEqual(await rawRequest('//'), 400);
    strictEqual(await rawRequest('//', { headers: { connection: 'Upgrade', upgrade: 'websocket', 'sec-websocket-version': '13', 'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==' } }), null);
    strictEqual((await get('/app/v1/cards?limit=1', bearer(owner))).status, 200);
  });
  await check('the hub disconnects a connection once its session ends', async () => {
    let allowed = true;
    const fakeServer = new EventEmitter(), socket = new EventEmitter(), sent = [];
    const push = createHub(fakeServer, () => allowed);
    socket.destroyed = false;
    socket.write = (bytes) => { sent.push(bytes.toString()); };
    socket.destroy = socket.end = () => { socket.destroyed = true; socket.emit('close'); };
    try {
      fakeServer.emit('upgrade', { url: '/hub', headers: { 'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==' } }, socket, Buffer.alloc(0));
      const handshake = Buffer.from('{"protocol":"json","version":1}\x1e');
      socket.emit('data', Buffer.concat([Buffer.from([0x81, 0x80 | handshake.length, 0, 0, 0, 0]), handshake]));
      push('object:updated', { id: 'while-signed-in' });
      ok(sent.some((text) => text.includes('while-signed-in')));
      allowed = false;
      push('object:updated', { id: 'after-sign-out' });
      ok(socket.destroyed, 'socket closed');
      ok(!sent.some((text) => text.includes('after-sign-out')), 'no event after sign-out');
    } finally {
      socket.end();
    }
  });
  await check('a token expires 90 days after last use and is refreshed at most hourly', async () => {
    const hash = sha256(owner);
    const set = users.prepare('UPDATE app_tokens SET used_ms = ?, expires_ms = ? WHERE hash = ?');
    const read = () => ({ ...users.prepare('SELECT used_ms, expires_ms FROM app_tokens WHERE hash = ?').get(hash) });
    const before = read();
    await get('/app/v1/cards?limit=1', bearer(owner));
    deepStrictEqual(read(), before, 'a fresh token is left alone');
    set.run(Date.now() - 7_200_000, Date.now() + 86_400_000, hash);
    await get('/app/v1/cards?limit=1', bearer(owner));
    ok(read().expires_ms - Date.now() > 89 * 86_400_000, 'expiry pushed out');
    set.run(Date.now() - 100_000_000, Date.now() - 1000, hash);
    strictEqual((await get('/app/v1/cards?limit=1', bearer(owner))).status, 401);
    set.run(Date.now(), Date.now() + 90 * 86_400_000, hash);
    strictEqual((await get('/app/v1/cards?limit=1', bearer(owner))).status, 200);
  });
  await check('a bad session cookie still gets 401', async () => {
    strictEqual((await get('/cards', { accept: 'application/msgpack', cookie: 'mm_session=nope' })).status, 401);
  });
  await check('sign-out kills the token', async () => {
    const token = (await (await googleSignIn()).json()).token;
    strictEqual((await get('/app/v1/cards?limit=1', bearer(token))).status, 200);
    strictEqual((await post('/app/v1/auth/signout', {}, bearer(token))).status, 204);
    strictEqual((await get('/app/v1/cards?limit=1', bearer(token))).status, 401);
    strictEqual(await hubOpens(bearer(token)), false);
    strictEqual((await post('/app/v1/auth/signout')).status, 401);
    strictEqual((await get('/app/v1/cards?limit=1', bearer(owner))).status, 200, 'other tokens survive');
  });

  await setPassword(users, 'Tester', PASSWORD);
  let passwordToken;
  await check('password user signs in, in any case', async () => {
    const res = await post('/app/v1/auth/password', { username: 'TESTER', password: PASSWORD, device: 'Pixel' });
    strictEqual(res.status, 200);
    const body = await res.json();
    deepStrictEqual(body.user, { email: 'tester', kind: 'password' });
    passwordToken = body.token;
    deepStrictEqual(await (await get('/app/v1/me', bearer(passwordToken))).json(), { email: 'tester', kind: 'password' });
    strictEqual(await hubOpens(bearer(passwordToken)), true);
  });
  await check('wrong password and unknown user get 401 invalid', async () => {
    for (const body of [{ username: 'tester', password: 'not the password at all' }, { username: 'nobody', password: PASSWORD }, {}]) {
      const res = await post('/app/v1/auth/password', body);
      strictEqual(res.status, 401);
      deepStrictEqual(await res.json(), { error: 'invalid' });
    }
  });
  await check('failures at the app lock the web form, and the other way round', async () => {
    await setPassword(users, 'applock', PASSWORD);
    await setPassword(users, 'weblock', PASSWORD);
    const appSide = { 'cf-connecting-ip': '203.0.113.10' };
    const webSide = { 'cf-connecting-ip': '203.0.113.11' };
    for (let i = 0; i < 10; i++) strictEqual((await post('/app/v1/auth/password', { username: 'applock', password: `wrong guess number ${i}` }, appSide)).status, 401);
    const locked = await post('/app/v1/auth/password', { username: 'applock', password: PASSWORD }, appSide);
    strictEqual(locked.status, 429);
    deepStrictEqual(await locked.json(), { error: 'locked' });
    strictEqual((await webPassword('applock', PASSWORD, appSide)).headers.get('location'), '/signin/password?error=locked');
    for (let i = 0; i < 10; i++) strictEqual((await webPassword('weblock', `wrong guess number ${i}`, webSide)).headers.get('location'), '/signin/password?error=invalid');
    strictEqual((await post('/app/v1/auth/password', { username: 'weblock', password: PASSWORD }, webSide)).status, 429);
  });
  await check('parallel wrong passwords stop at the lockout limit', async () => {
    await setPassword(users, 'parallel', PASSWORD);
    const from = { 'cf-connecting-ip': '203.0.113.12' };
    const statuses = await Promise.all(Array.from({ length: 20 }, (_, i) => post('/app/v1/auth/password', { username: 'parallel', password: `parallel guess ${i}` }, from).then((res) => res.status)));
    strictEqual(statuses.filter((status) => status === 401).length, 10, statuses.join(' '));
    strictEqual(statuses.filter((status) => status === 429).length, 10);
  });
  await check('a password split mid-character across body chunks still signs in', async () => {
    const unicode = 'pässwörd é ünïcödé';
    await setPassword(users, 'unicode', unicode);
    const body = Buffer.from(JSON.stringify({ username: 'unicode', password: unicode }));
    const cut = body.indexOf(Buffer.from('é')) + 1;
    strictEqual(await rawRequest('/app/v1/auth/password', { method: 'POST', headers: { 'content-type': 'application/json' }, parts: [body.subarray(0, cut), body.subarray(cut)] }), 200);
  });
  await check('removing a password user ends their app tokens', async () => {
    strictEqual((await get('/app/v1/cards?limit=1', bearer(passwordToken))).status, 200);
    ok(removeUser(users, 'tester'));
    strictEqual((await get('/app/v1/cards?limit=1', bearer(passwordToken))).status, 401);
    strictEqual(users.prepare('SELECT COUNT(*) AS n FROM app_tokens WHERE sub = ?').get('password:tester').n, 0);
    strictEqual((await post('/app/v1/auth/password', { username: 'tester', password: PASSWORD })).status, 401);
  });
  await check('a plain tunnel visitor reaches only the sign-in POSTs', async () => {
    strictEqual((await post('/app/v1/auth/nonce', {}, TUNNELLED)).status, 200);
    strictEqual((await googleSignIn({}, { headers: TUNNELLED })).status, 200);
    strictEqual((await post('/app/v1/auth/password', { username: 'x', password: 'y' }, TUNNELLED)).status, 401);
    for (const path of ['/app/v1/cards', '/app/v1/cards/x', '/app/v1/trash', '/app/v1/spaces', '/app/v1/tags', '/app/v1/search?q=a', '/app/v1/me', '/cards', '/media/x.jpg', '/_static/134/0.109.0/styles/main.css']) {
      strictEqual((await get(path, TUNNELLED)).status, 401, path);
    }
    strictEqual((await post('/app/v1/auth/signout', {}, TUNNELLED)).status, 401);
    strictEqual((await get('/app/v1/cards?limit=1', { ...TUNNELLED, ...bearer(owner) })).status, 200, 'a token works through the tunnel');
  });

  const headers = bearer(owner);
  const grid = await webIds(LOCAL);
  await check('card pages are stable, match the web grid order and filter, and cover every card once', async () => {
    ok(grid.length > 100, 'the library has cards');
    for (const limit of [200, 37]) {
      const { cards, pages } = await walk(server.base, '/app/v1/cards', limit, headers);
      deepStrictEqual(cards.map((card) => card.id), grid);
      strictEqual(pages, Math.ceil(grid.length / limit));
    }
    const first = await (await get('/app/v1/cards?limit=5', headers)).json();
    strictEqual(first.cards.length, 5);
    ok(typeof first.next === 'string');
    const again = await (await get(`/app/v1/cards?limit=5&cursor=${first.next}`, headers)).json();
    deepStrictEqual(again.cards.map((c) => c.id), grid.slice(5, 10));
  });
  await check('limits and cursors outside the contract get 400', async () => {
    for (const query of ['limit=0', 'limit=201', 'limit=abc', 'cursor=garbage', 'cursor=W10']) strictEqual((await get(`/app/v1/cards?${query}`, headers)).status, 400, query);
    deepStrictEqual(await (await get('/app/v1/cards?limit=0', headers)).json(), { error: 'bad-limit' });
  });
  await check('card detail matches the list and unknown ids get 404 not-found', async () => {
    const [listed] = (await (await get('/app/v1/cards?limit=1', headers)).json()).cards;
    deepStrictEqual(await (await get(`/app/v1/cards/${listed.id}`, headers)).json(), listed);
    const missing = await get('/app/v1/cards/nope', headers);
    strictEqual(missing.status, 404);
    deepStrictEqual(await missing.json(), { error: 'not-found' });
  });
  await check('card JSON follows the mapping rules on every stored type', async () => {
    const seen = new Set();
    let checked = 0;
    for (const type of library.prepare('SELECT DISTINCT type FROM cards').all().map((row) => row.type)) {
      for (const row of library.prepare('SELECT id, data, extras, flags FROM cards WHERE type = ? AND deleted_ns IS NULL ORDER BY id LIMIT 4').all(type)) {
        const data = JSON.parse(row.data);
        const extras = JSON.parse(row.extras);
        const card = await (await get(`/app/v1/cards/${row.id}`, headers)).json();
        seen.add(type);
        checked += 1;
        strictEqual(card.id, row.id);
        strictEqual(card.type, type);
        strictEqual(card.flags, Number(row.flags));
        for (const field of ['created', 'modified', 'bumped']) ok(/^\d{4}-\d\d-\d\dT[\d:.]+Z$/.test(card[field]), field);
        ok(!('deleted' in card));
        strictEqual(card.title, extras.title ?? data.title);
        strictEqual(card.url, extras.linkUrl ?? data.source?.url);
        strictEqual(card.description, data.description);
        strictEqual(card.domain, data.domain);
        strictEqual(card.siteName, data.siteName);
        deepStrictEqual(card.cover, local(extras.cover ?? (type === 'Image' && data.blob ? imageCover(data.blob) : undefined)));
        deepStrictEqual(card.post, local(extras.post));
        deepStrictEqual(card.app, local(extras.app));
        deepStrictEqual(card.product, local(extras.product));
        deepStrictEqual(card.reddit, local(extras.reddit));
        deepStrictEqual(card.prose, data.prose);
        deepStrictEqual((card.tags ?? []).map((tag) => tag.name), library.prepare('SELECT name FROM annotations WHERE card_id = ? ORDER BY position').all(row.id).map((r) => r.name));
        if (data.blob) deepStrictEqual(Object.keys(card.blob).sort(), ['dominantColor', 'height', 'path', 'type', 'width'].filter((key) => data.blob[key] !== undefined).sort());
        else ok(!('blob' in card));
        strictEqual(card.state, 'ready');
        ok(!JSON.stringify(card).includes('null'), 'absent values are omitted, never null');
        ok(!JSON.stringify(card).includes('https://static.accelerator.net/'), 'origins are localized');
      }
    }
    ok(seen.size >= 5 && ['XPost', 'WebPage', 'Image'].every((type) => seen.has(type)), `types covered: ${seen.size}`);
    ok(checked >= 20);
  });
  await check('a card with a note or space membership maps those fields', async () => {
    const noted = library.prepare("SELECT id FROM cards WHERE json_extract(data, '$.note.prose') IS NOT NULL AND deleted_ns IS NULL LIMIT 1").get();
    if (noted) strictEqual(typeof (await (await get(`/app/v1/cards/${noted.id}`, headers)).json()).note, 'string');
    const member = library.prepare('SELECT card_id FROM space_cards WHERE excluded = 0 LIMIT 1').get();
    if (member) {
      const card = await (await get(`/app/v1/cards/${member.card_id}`, headers)).json();
      ok(Array.isArray(card.spaces) && card.spaces.length > 0 && card.spaces.every((id) => typeof id === 'string'));
    }
    ok(noted || member, 'the library has a note or a space member');
  });
  await check('spaces list matches the web, and a space\'s cards follow its order', async () => {
    const webSpaces = await (await fetch(`${LOCAL}/spaces`, { headers: { accept: 'application/json' } })).json();
    const { spaces } = await (await get('/app/v1/spaces', headers)).json();
    deepStrictEqual(spaces.map((space) => space.id), webSpaces.map((space) => space.id));
    for (const space of spaces) {
      deepStrictEqual(Object.keys(space).sort(), ['color', 'count', 'id', 'name', 'position', 'smart']);
      strictEqual(typeof space.smart, 'boolean');
    }
    const full = spaces.find((space) => space.count > 3);
    ok(full, 'a space with cards');
    const expected = webSpaces.find((space) => space.id === full.id).objects.map((o) => o.id);
    const listed = (await walk(server.base, `/app/v1/spaces/${full.id}/cards`, 3, headers)).cards.map((card) => card.id);
    const visible = new Set(grid);
    deepStrictEqual(listed, expected.filter((id) => visible.has(id)));
    strictEqual((await get('/app/v1/spaces/nope/cards', headers)).status, 404);
  });
  await check('tags match the web tag list', async () => {
    const res = await fetch(`${LOCAL}/tags/manual`, { headers: { accept: 'application/msgpack' } });
    const [webTags] = [...msgpack.decodeMulti(Buffer.from(await res.arrayBuffer()), { useBigInt64: true })];
    const { tags } = await (await get('/app/v1/tags', headers)).json();
    deepStrictEqual(tags.map((tag) => tag.name), webTags.map((tag) => tag.name));
    deepStrictEqual(tags.map((tag) => tag.count), webTags.map((tag) => Number(tag.count)));
    ok(tags.length > 0 && tags.every((tag) => Object.keys(tag).length === 2));
  });
  await check('search pages match the web search order and stay stable across page sizes', async () => {
    const q = 'type:image';
    const web = (await (await fetch(`${LOCAL}/search?q=${encodeURIComponent(q)}`, { headers: { accept: 'application/json' } })).json()).matches.map((m) => m.id);
    ok(web.length > 5);
    for (const limit of [200, 7]) {
      const { cards } = await walk(server.base, `/app/v1/search?q=${encodeURIComponent(q)}`, limit, headers);
      deepStrictEqual(cards.map((card) => card.id), web);
    }
    strictEqual((await get('/app/v1/search', headers)).status, 400);
    deepStrictEqual((await (await get('/app/v1/search?q=zzzqqqnomatchzzz', headers)).json()).cards, []);
  });
  await check('trash answers the list shape, and web write routes still need the token', async () => {
    const trash = await (await get('/app/v1/trash', headers)).json();
    ok(Array.isArray(trash.cards) && trash.next === null);
    strictEqual((await call(server.base, '/objects', { method: 'POST', body: { type: 'Note' } })).status, 401);
  });
  await check('LOCAL_PORT serves the reads without sign-in and refuses the tunnel', async () => {
    strictEqual((await call(LOCAL, '/app/v1/cards?limit=2')).status, 200);
    strictEqual((await call(LOCAL, '/app/v1/cards?limit=2', { headers: TUNNELLED })).status, 403);
  });
} finally {
  library.close();
  users.close();
  await server.close();
  idp.close();
  await rm(authDir, { recursive: true, force: true });
}

const open = await testServer({ readonly: false });
try {
  await check('without sign-in the reads work and pagination survives writes', async () => {
    const before = (await walk(open.base, '/app/v1/cards', 200)).cards.map((c) => c.id);
    const first = await (await call(open.base, '/app/v1/cards?limit=10')).json();
    const created = await open.request('/objects', { method: 'POST', body: { type: 'Note', title: 'App API note', prose: { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'hello' }] }] } } });
    strictEqual(created.response.status, 201);
    const second = await (await call(open.base, `/app/v1/cards?limit=10&cursor=${first.next}`)).json();
    deepStrictEqual([...first.cards, ...second.cards].map((c) => c.id), before.slice(0, 20), 'a new card does not shift later pages');
    const newest = (await (await call(open.base, '/app/v1/cards?limit=1')).json()).cards[0];
    strictEqual(newest.id, created.value.id);
    strictEqual(newest.type, 'Note');
    strictEqual(typeof newest.prose, 'object');
  });
  await check('trash lists deleted cards, most recent first, outside the grid', async () => {
    const [a, b] = (await (await call(open.base, '/app/v1/cards?limit=3')).json()).cards;
    strictEqual((await open.request(`/objects/${a.id}`, { method: 'DELETE' })).response.status, 200);
    await sleep(1100);
    strictEqual((await open.request(`/objects/${b.id}`, { method: 'DELETE' })).response.status, 200);
    const trash = (await walk(open.base, '/app/v1/trash', 1)).cards;
    deepStrictEqual(trash.slice(0, 2).map((c) => c.id), [b.id, a.id]);
    ok(trash.every((c) => typeof c.deleted === 'string'));
    deepStrictEqual(trash.map((c) => c.id), await webIds(open.base, '/cards/trash'));
    ok(!(await walk(open.base, '/app/v1/cards', 200)).cards.some((c) => [a.id, b.id].includes(c.id)));
    strictEqual((await (await call(open.base, `/app/v1/cards/${a.id}`)).json()).deleted, trash[1].deleted);
  });
} finally {
  await open.close();
}

console.log(`\n${failures.length ? `${failures.length} failed` : 'all passed'} (${total - failures.length}/${total})`);
process.exitCode = failures.length ? 1 : 0;
