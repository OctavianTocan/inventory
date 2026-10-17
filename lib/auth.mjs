import { createHash, createPublicKey, randomBytes, verify } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { checkPassword, installAuthSchema, openAuthStore, passwordSubject } from './auth-store.mjs';

const SESSION_COOKIE = 'mm_session';
const STATE_COOKIE = 'mm_oauth';
const SESSION_MS = 30 * 24 * 3600 * 1000;
const LOGIN_MS = 10 * 60 * 1000;
const APP_TOKEN_MS = 90 * 24 * 3600 * 1000;
const SKEW_S = 60;
const FAILURE_WINDOW_MS = 15 * 60 * 1000;
const FAILURES_PER_USER = 10;
const FAILURES_PER_ADDRESS = 30;
const KEY_REFRESH_MS = 60_000;
// Paths a signed-out browser needs to show the sign-in page and finish signing in.
const APP_SIGNIN = /^\/app\/v1\/auth\/(nonce|google|password)$/;
const PUBLIC = [APP_SIGNIN, /^\/signin$/, /^\/google\/oauth2-(redirect|callback)$/, /^\/_static\//, /^\/_auth\//, /^\/manifest\.webmanifest$/, /^\/favicon\.ico$/, /^\/jserrors$/];
// Through the tunnel the plain sign-in page needs no assets, so mymind's stay behind sign-in.
const TUNNEL_PUBLIC = [APP_SIGNIN, /^\/signin$/, /^\/google\/oauth2-(redirect|callback)$/];

const base64url = (bytes) => Buffer.from(bytes).toString('base64url');
const sha256 = (text) => createHash('sha256').update(text).digest('hex');

// Read a request body as UTF-8 text; null when it is larger than `limit` bytes.
async function readText(req, limit) {
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > limit) return null;
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString('utf8');
}

// Read a small JSON object body; null when it is too large or not an object.
async function readJson(req, limit = 16384) {
  const text = await readText(req, limit);
  if (text === null) return null;
  try {
    const value = JSON.parse(text || '{}');
    return value && typeof value === 'object' && !Array.isArray(value) ? value : null;
  } catch { return null; }
}

// Parse the request's Cookie header into a name → value map.
function cookies(req) {
  return Object.fromEntries((req.headers.cookie ?? '').split(';').map((part) => part.trim().split('=')).filter(([name]) => name)
    .map(([name, ...value]) => [name, decodeCookie(value.join('='))]));
}

// Percent-decode a cookie value, keeping it as sent when it is not valid encoding.
function decodeCookie(value) {
  try { return decodeURIComponent(value); } catch { return value; }
}

// Serialize one Set-Cookie value with the attributes every auth cookie shares.
function cookie(name, value, { maxAge, secure, path = '/' }) {
  return `${name}=${encodeURIComponent(value)}; Path=${path}; Max-Age=${Math.floor(maxAge / 1000)}; HttpOnly; SameSite=Lax${secure ? '; Secure' : ''}`;
}

// Page loaded in the sign-in popup: hands the result to the sign-in window and closes, or navigates itself.
function popupPage(target) {
  const to = JSON.stringify(target).replaceAll('<', '\\u003c');
  return `<!doctype html><meta charset="utf-8"><title>Signing in…</title><script>
const to = ${to};
try { new BroadcastChannel('replica-auth').postMessage(to); } catch {}
try { if (opener && opener.location.origin === location.origin) { opener.location.assign(to); close(); } } catch {}
if (opener) close();
setTimeout(() => location.replace(to), 300);
</script>`;
}

const PASSWORD_ERRORS = {
  invalid: 'That username and password do not match.',
  locked: 'Too many attempts. Try again in 15 minutes.',
};

const SIGNIN_ERRORS = {
  AccountNotFound: 'That Google account does not have access.',
  Cancelled: 'Sign-in was cancelled.',
  SignInFailed: 'Sign-in did not work. Try again.',
};

const GOOGLE_LOGO = '<svg width="18" height="18" viewBox="0 0 48 48" aria-hidden="true"><path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"/><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"/><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"/><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"/></svg>';

// A self-contained page in the replica's own plain style, not mymind's; it requests no favicon and asks not to be indexed.
function plainPage(body) {
  return `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex, nofollow"><link rel="icon" href="data:,"><title>Sign in</title><style>
:root { color-scheme: light dark; --bg: #f5f5f3; --card: #fff; --text: #1d1d1f; --muted: #6e6e73; --line: #d8d8dc; --button: #1d1d1f; --button-text: #fff; }
@media (prefers-color-scheme: dark) { :root { --bg: #111113; --card: #1c1c1f; --text: #f2f2f4; --muted: #a1a1a6; --line: #333338; --button: #f2f2f4; --button-text: #111113; } }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 16px; background: var(--bg); color: var(--text); font: 16px/1.4 system-ui, sans-serif; }
.card { width: 100%; max-width: 360px; display: grid; gap: 14px; padding: 32px 28px; background: var(--card); border: 1px solid var(--line); border-radius: 16px; }
h1 { margin: 0 0 6px; font-size: 22px; font-weight: 600; }
p { margin: 0; color: var(--muted); font-size: 15px; }
label { display: grid; gap: 6px; font-size: 14px; color: var(--muted); }
input { font: inherit; color: var(--text); background: transparent; padding: 11px 12px; border: 1px solid var(--line); border-radius: 10px; }
input:focus { outline: 2px solid var(--text); outline-offset: -1px; border-color: transparent; }
a { color: var(--muted); font-size: 14px; text-align: center; }
button, .button { display: flex; align-items: center; justify-content: center; gap: 10px; font: inherit; font-weight: 600; margin-top: 6px; padding: 12px; border: 0; border-radius: 10px; background: var(--button); color: var(--button-text); cursor: pointer; text-decoration: none; }
.error { color: #d93a1a; font-size: 14px; }
</style>
${body}</html>`;
}

// The sign-in page visitors through the tunnel see: Google in a popup (as on mymind's page), or a password.
function plainSigninPage(error, passwords) {
  const message = Object.hasOwn(SIGNIN_ERRORS, error ?? '') ? `<p class="error" role="alert">${SIGNIN_ERRORS[error]}</p>` : '';
  return plainPage(`<main class="card">
<h1>Sign in</h1>
<p>This library is private. Sign in with an account that has access.</p>${message}
<a class="button" id="google" href="/google/oauth2-redirect">${GOOGLE_LOGO}Continue with Google</a>
${passwords ? '<a href="/signin/password">Sign in with a password</a>\n' : ''}</main>
<script>
new BroadcastChannel('replica-auth').onmessage = (event) => location.assign(event.data);
document.getElementById('google').addEventListener('click', (event) => {
  if (window.open('/google/oauth2-redirect?popup=true', 'signin', 'popup,width=500,height=640')) event.preventDefault();
});
</script>`);
}

// Username and password form at /signin/password, for the password users in the auth database.
function passwordPage(error) {
  const message = Object.hasOwn(PASSWORD_ERRORS, error ?? '') ? `<p class="error" role="alert">${PASSWORD_ERRORS[error]}</p>` : '';
  return plainPage(`<form class="card" method="post" action="/signin/password">
<h1>Sign in</h1>${message}
<label>Username <input name="username" autocomplete="username" autocapitalize="none" spellcheck="false" required autofocus></label>
<label>Password <input name="password" type="password" autocomplete="current-password" required></label>
<button type="submit">Sign in</button>
<a href="/signin">Sign in with Google instead</a>
</form>`);
}

// Read a small urlencoded form body; null when it is too large.
async function readForm(req, limit = 4096) {
  const text = await readText(req, limit);
  return text === null ? null : new URLSearchParams(text);
}

/**
 * Create sign-in for the replica, or return null when the environment does not turn it on.
 * Sign-in is on only with `AUTH=google`, which also requires `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` and
 * `OWNER_EMAIL`; only the owner's verified Google account may sign in with Google. Password users added with
 * `tools/auth-user.mjs` may also sign in at `/signin/password`. Every signed-in session reaches the existing library.
 * The native app signs in at `/app/v1/auth/*` and then sends `Authorization: Bearer <token>`, which is accepted wherever
 * the session cookie is; `APP_NONCE_MS` shortens the nonce lifetime (default 10 minutes) for tests.
 * Signed-out visitors through a Cloudflare tunnel get a plain sign-in page of the replica's own and none of mymind's
 * assets; visitors on this machine get mymind's recorded sign-in page.
 * @param {object} options
 * @param {string} options.root Repository root; sessions and password users live in `data/auth.db` (or `AUTH_DB`), sign-in pages in `capture/signin/`.
 * @param {(text:string) => string} options.localize Rewrite recorded origins to local paths.
 * @param {Record<string,string|undefined>} [options.env=process.env] Configuration source.
 * @param {number} options.port Port the server listens on, for the default public URL.
 * @param {object} [options.db] Open SQL handle to keep sign-in data in instead of the `AUTH_DB` file (the Worker's
 *   Durable Object); the caller closes it.
 * @returns {object|null} Auth handlers, or null when sign-in is off.
 * @throws {Error} If `AUTH=google` is set without the client credentials or owner email.
 */
export function createAuth({ root, localize, env = process.env, port, db: injected }) {
  if (env.AUTH !== 'google') return null;
  const clientId = env.GOOGLE_CLIENT_ID;
  const clientSecret = env.GOOGLE_CLIENT_SECRET;
  const owner = env.OWNER_EMAIL?.trim().toLowerCase();
  if (!clientId || !clientSecret || !owner) throw new Error('AUTH=google needs GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET and OWNER_EMAIL');
  const publicUrl = (env.PUBLIC_URL ?? `http://localhost:${port}`).replace(/\/$/, '');
  const secure = publicUrl.startsWith('https://');
  const redirectUri = `${publicUrl}/google/oauth2-callback`;
  const google = {
    authorize: env.GOOGLE_AUTH_URL ?? 'https://accounts.google.com/o/oauth2/v2/auth',
    token: env.GOOGLE_TOKEN_URL ?? 'https://oauth2.googleapis.com/token',
    certs: env.GOOGLE_CERTS_URL ?? 'https://www.googleapis.com/oauth2/v3/certs',
    issuers: (env.GOOGLE_ISSUER ?? 'https://accounts.google.com,accounts.google.com').split(','),
  };

  const db = injected ? installAuthSchema(injected) : openAuthStore(env.AUTH_DB ?? join(root, 'data', 'auth.db'));
  db.prepare('DELETE FROM sessions WHERE expires_ms < ?').run(Date.now());
  db.prepare('DELETE FROM app_tokens WHERE expires_ms < ?').run(Date.now());
  const nonceMs = Number(env.APP_NONCE_MS ?? LOGIN_MS);
  let keys = { fetched: 0, maxAge: 0, byKid: new Map() };
  let keysAttempted = 0;
  let keysLoading = null;

  // Fetch Google's signing keys into the cache.
  async function fetchKeys() {
    keysAttempted = Date.now();
    const res = await fetch(google.certs, { signal: AbortSignal.timeout(10_000) });
    if (!res.ok) throw new Error(`certs ${res.status}`);
    const maxAge = Number(/max-age=(\d+)/.exec(res.headers.get('cache-control') ?? '')?.[1] ?? 3600) * 1000;
    keys = { fetched: Date.now(), maxAge, byKid: new Map((await res.json()).keys.map((jwk) => [jwk.kid, createPublicKey({ key: jwk, format: 'jwk' })])) };
  }

  // Google's signing key `kid`, honouring the keys' cache lifetime; anyone can send an unknown key id, so it refetches at most once a minute.
  async function signingKey(kid) {
    const stale = Date.now() - keys.fetched > keys.maxAge;
    if (stale || (!keys.byKid.has(kid) && Date.now() - keysAttempted > KEY_REFRESH_MS)) {
      keysLoading ??= fetchKeys().finally(() => { keysLoading = null; });
      await keysLoading;
    }
    const key = keys.byKid.get(kid);
    if (!key) throw new Error('unknown signing key');
    return key;
  }

  // Verify an ID token's RS256 signature and claims; returns its claims.
  async function verifyIdToken(token, nonce) {
    const [header, payload, signature] = token.split('.');
    const { alg, kid } = JSON.parse(Buffer.from(header, 'base64url'));
    if (alg !== 'RS256') throw new Error('unexpected token algorithm');
    if (!verify('RSA-SHA256', Buffer.from(`${header}.${payload}`), await signingKey(kid), Buffer.from(signature, 'base64url'))) throw new Error('bad token signature');
    const claims = JSON.parse(Buffer.from(payload, 'base64url'));
    const now = Date.now() / 1000;
    if (!google.issuers.includes(claims.iss)) throw new Error('unexpected issuer');
    if (claims.aud !== clientId) throw new Error('unexpected audience');
    if (!(claims.exp > now - SKEW_S) || !(claims.iat < now + SKEW_S)) throw new Error('token expired');
    if (claims.nonce !== nonce) throw new Error('nonce mismatch');
    return claims;
  }

  // Exchange the authorization code for tokens at Google's token endpoint.
  async function exchange(code, verifier) {
    const res = await fetch(google.token, {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded', accept: 'application/json' },
      body: new URLSearchParams({ code, client_id: clientId, client_secret: clientSecret, redirect_uri: redirectUri, grant_type: 'authorization_code', code_verifier: verifier }),
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok || typeof body.id_token !== 'string') throw new Error(`token exchange ${res.status}`);
    return body.id_token;
  }

  // The session row for this request's cookie, or null; refreshes its last-use time at most hourly.
  function cookieSession(req) {
    const id = cookies(req)[SESSION_COOKIE];
    if (!id) return null;
    const row = db.prepare('SELECT hash, sub, email, expires_ms, used_ms FROM sessions WHERE hash = ?').get(sha256(id));
    if (!row || row.expires_ms < Date.now()) return null;
    if (Date.now() - row.used_ms > 3600_000) db.prepare('UPDATE sessions SET used_ms = ? WHERE hash = ?').run(Date.now(), row.hash);
    return row;
  }

  // The bearer token on this request, if any.
  const bearer = (req) => /^Bearer +(\S+)$/i.exec(req.headers.authorization ?? '')?.[1] ?? null;

  // The live app token row for this request's bearer token, or null; extends its expiry at most hourly.
  function appSession(req) {
    const token = bearer(req);
    if (!token) return null;
    const row = db.prepare('SELECT hash, sub, email, expires_ms, used_ms FROM app_tokens WHERE hash = ?').get(sha256(token));
    if (!row || row.expires_ms < Date.now()) return null;
    if (Date.now() - row.used_ms > 3600_000) db.prepare('UPDATE app_tokens SET used_ms = ?, expires_ms = ? WHERE hash = ?').run(Date.now(), Date.now() + APP_TOKEN_MS, row.hash);
    return row;
  }

  // The signed-in identity for this request: a session cookie, else an app bearer token.
  const session = (req) => cookieSession(req) ?? appSession(req);

  // The email a verified Google ID token may sign in as, or null; the one rule for the web callback and the app.
  const googleAccess = (claims) => (claims.email_verified === true && String(claims.email).toLowerCase() === owner ? owner : null);

  // Store a new app token and return the sign-in answer, which is the only time the token itself is known.
  function startAppToken(sub, email, kind, device) {
    const token = base64url(randomBytes(32));
    const now = Date.now();
    db.prepare('INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)')
      .run(sha256(token), sub, email, kind, String(device ?? '').slice(0, 100), now, now + APP_TOKEN_MS, now);
    return { token, expiresAt: new Date(now + APP_TOKEN_MS).toISOString(), user: { email, kind } };
  }

  // mymind's recorded sign-in page (or its error variant), wired to this server's Google sign-in.
  async function signinPage(error) {
    const name = error === 'AccountNotFound' ? 'AccountNotFound' : 'signin';
    const html = localize(await readFile(join(root, 'capture', 'signin', `${name}.html`), 'utf8'))
      .replace('https://appleid.cdn-apple.com/appleauth/static/jsapi/appleid/1/en_US/appleid.auth.js', '/_auth/appleid.js')
      .replace('</body>', '<script src="/_auth/signin.js"></script></body>');
    return html;
  }

  // Store a new session for `sub` and return its id, which only the browser's cookie holds.
  function startSession(sub, email) {
    const id = base64url(randomBytes(32));
    const now = Date.now();
    db.prepare('INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)')
      .run(sha256(id), sub, email, now, now + SESSION_MS, now);
    return id;
  }

  // Failed password attempts for `key` within the window.
  const recentFailures = (key) => db.prepare('SELECT COUNT(*) AS n FROM failures WHERE key = ? AND at_ms > ?').get(key, Date.now() - FAILURE_WINDOW_MS).n;

  // Finish a sign-in attempt: a new session id for the owner, or the problem mymind's page names.
  async function callback(url, req) {
    const state = url.searchParams.get('state') ?? '';
    const login = db.prepare('SELECT verifier, nonce, created_ms FROM logins WHERE state = ?').get(state);
    db.prepare('DELETE FROM logins WHERE state = ?').run(state);
    if (!login || cookies(req)[STATE_COOKIE] !== state || Date.now() - login.created_ms > LOGIN_MS) throw new Error('unknown sign-in attempt');
    if (url.searchParams.get('error')) return { problem: 'Cancelled' };
    const claims = await verifyIdToken(await exchange(url.searchParams.get('code') ?? '', login.verifier), login.nonce);
    const email = googleAccess(claims);
    if (!email) return { problem: 'AccountNotFound' };
    return { id: startSession(claims.sub, email) };
  }

  // Check a username and password against the lockout state shared by the web form and the app: the stored name, or the error.
  async function verifyPassword(req, rawUsername, password) {
    const username = String(rawUsername ?? '').trim().toLowerCase();
    const keys = [`user:${username}`, `address:${req.headers['cf-connecting-ip'] ?? req.socket.remoteAddress}`];
    db.prepare('DELETE FROM failures WHERE at_ms <= ?').run(Date.now() - FAILURE_WINDOW_MS);
    if (recentFailures(keys[0]) >= FAILURES_PER_USER || recentFailures(keys[1]) >= FAILURES_PER_ADDRESS) return { error: 'locked' };
    // Count the attempt as failed while the hash runs, so parallel guesses cannot pass the limit.
    const attempt = base64url(randomBytes(12));
    for (const key of keys) db.prepare('INSERT INTO failures (key, attempt, at_ms) VALUES (?, ?, ?)').run(key, attempt, Date.now());
    const name = password !== undefined && await checkPassword(db, username, password);
    if (!name) return { error: 'invalid' };
    db.prepare('DELETE FROM failures WHERE key = ?').run(keys[0]);
    db.prepare('DELETE FROM failures WHERE key = ? AND attempt = ?').run(keys[1], attempt);
    return { name };
  }

  // Check a password form post; a new session id, or the error the form shows.
  async function passwordSignIn(req) {
    const form = await readForm(req);
    const result = await verifyPassword(req, form?.get('username'), form ? form.get('password') : undefined);
    return result.error ? result : { id: startSession(passwordSubject(result.name), result.name) };
  }

  // Answer the app's sign-in POSTs (`/app/v1/auth/{nonce,google,password}`) as JSON.
  async function appSignIn(req, res, step) {
    const reply = (status, body) => { res.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' }); res.end(JSON.stringify(body)); return true; };
    if (req.method !== 'POST') { res.writeHead(405, { allow: 'POST' }); res.end(); return true; }
    if (step === 'nonce') {
      db.prepare('DELETE FROM app_nonces WHERE created_ms < ?').run(Date.now() - nonceMs);
      db.prepare('DELETE FROM app_nonces WHERE nonce IN (SELECT nonce FROM app_nonces ORDER BY created_ms DESC LIMIT -1 OFFSET 9999)').run();
      const nonce = base64url(randomBytes(24));
      db.prepare('INSERT INTO app_nonces (nonce, created_ms) VALUES (?, ?)').run(nonce, Date.now());
      return reply(200, { nonce, expiresAt: new Date(Date.now() + nonceMs).toISOString() });
    }
    const body = await readJson(req);
    if (step === 'google') {
      const nonce = String(body?.nonce ?? '');
      const created = db.prepare('SELECT created_ms FROM app_nonces WHERE nonce = ?').get(nonce)?.created_ms;
      db.prepare('DELETE FROM app_nonces WHERE nonce = ?').run(nonce);
      if (created === undefined || Date.now() - created > nonceMs || typeof body.idToken !== 'string') return reply(400, { error: 'invalid-token' });
      let claims;
      try { claims = await verifyIdToken(body.idToken, nonce); } catch { return reply(400, { error: 'invalid-token' }); }
      if (claims.email_verified !== true) return reply(400, { error: 'invalid-token' });
      const email = googleAccess(claims);
      return email ? reply(200, startAppToken(claims.sub, email, 'google', body.device)) : reply(401, { error: 'not-allowed' });
    }
    const result = body ? await verifyPassword(req, body.username, typeof body.password === 'string' ? body.password : undefined) : { error: 'invalid' };
    if (result.error) return reply(result.error === 'locked' ? 429 : 401, { error: result.error });
    return reply(200, startAppToken(passwordSubject(result.name), result.name, 'password', body.device));
  }

  return {
    /**
     * Answer sign-in (Google and password), sign-out and gate every other request that has no valid session.
     * @param {import('node:http').IncomingMessage} req Incoming request.
     * @param {import('node:http').ServerResponse} res Response to write when this returns true.
     * @param {URL} url Parsed request URL.
     * @returns {Promise<boolean>} True when the request was answered here.
     */
    async handle(req, res, url) {
      const { pathname } = url;
      const signedIn = session(req);
      const document = req.method === 'GET' && (req.headers['sec-fetch-dest'] === 'document' || req.headers.accept?.includes('text/html'));
      const redirect = (location, extra = {}) => { res.writeHead(302, { location, 'cache-control': 'no-store', ...extra }); res.end(); return true; };
      const html = (body, extra = {}) => { res.writeHead(200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store', ...extra }); res.end(body); return true; };
      const tunnelled = Boolean(req.headers['cf-connecting-ip']);
      const passwords = () => Boolean(db.prepare('SELECT 1 FROM users LIMIT 1').get());
      if (req.method !== 'GET' && req.headers.origin && req.headers.origin !== new URL(publicUrl).origin && req.headers.origin !== `http://${req.headers.host}`) {
        res.writeHead(403); res.end('cross-origin write refused'); return true;
      }
      const appStep = APP_SIGNIN.exec(pathname)?.[1];
      if (appStep) return appSignIn(req, res, appStep);
      if (pathname === '/robots.txt' && req.method === 'GET') {
        res.writeHead(200, { 'content-type': 'text/plain; charset=utf-8' });
        res.end('User-agent: *\nDisallow: /\n');
        return true;
      }
      if (pathname === '/signin' && req.method === 'GET') {
        if (signedIn) return redirect('/everything');
        const error = url.searchParams.get('error');
        return html(tunnelled ? plainSigninPage(error, passwords()) : await signinPage(error));
      }
      if (pathname === '/signin/password') {
        if (!passwords()) { res.writeHead(404); res.end(); return true; }
        if (req.method === 'GET') return signedIn ? redirect('/everything') : html(passwordPage(url.searchParams.get('error')));
        if (req.method !== 'POST') { res.writeHead(405, { allow: 'GET, POST' }); res.end(); return true; }
        const result = await passwordSignIn(req);
        const seeOther = (location, extra = {}) => { res.writeHead(303, { location, 'cache-control': 'no-store', ...extra }); res.end(); return true; };
        if (result.error) return seeOther(`/signin/password?error=${result.error}`);
        return seeOther('/everything', { 'set-cookie': cookie(SESSION_COOKIE, result.id, { maxAge: SESSION_MS, secure }) });
      }
      if (pathname === '/_auth/signin.js') {
        res.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
        res.end(`new BroadcastChannel('replica-auth').onmessage = (event) => location.assign(event.data);
document.addEventListener('click', (event) => { if (event.target.closest?.('#appleid-signin')) { event.preventDefault(); event.stopImmediatePropagation(); } }, true);\n`);
        return true;
      }
      if (pathname === '/_auth/appleid.js') {
        res.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
        res.end('window.AppleID = { auth: { init() {} } };\n');
        return true;
      }
      if (pathname === '/google/oauth2-redirect' && req.method === 'GET') {
        const state = base64url(randomBytes(24));
        const verifier = base64url(randomBytes(32));
        const nonce = base64url(randomBytes(24));
        db.prepare('DELETE FROM logins WHERE created_ms < ?').run(Date.now() - LOGIN_MS);
        db.prepare('INSERT INTO logins (state, verifier, nonce, created_ms) VALUES (?, ?, ?, ?)').run(state, verifier, nonce, Date.now());
        const query = new URLSearchParams({ client_id: clientId, redirect_uri: redirectUri, response_type: 'code', scope: 'openid email',
          state, nonce, code_challenge: base64url(createHash('sha256').update(verifier).digest()), code_challenge_method: 'S256', prompt: 'select_account' });
        return redirect(`${google.authorize}?${query}`, { 'set-cookie': cookie(STATE_COOKIE, state, { maxAge: LOGIN_MS, secure, path: '/google' }) });
      }
      if (pathname === '/google/oauth2-callback' && req.method === 'GET') {
        const clearState = cookie(STATE_COOKIE, '', { maxAge: 0, secure, path: '/google' });
        try {
          const result = await callback(url, req);
          if (result.problem) return html(popupPage(`/signin?error=${result.problem}&identityProvider=Google`), { 'set-cookie': clearState });
          return html(popupPage('/everything'), { 'set-cookie': [clearState, cookie(SESSION_COOKIE, result.id, { maxAge: SESSION_MS, secure })] });
        } catch {
          return html(popupPage('/signin?error=SignInFailed&identityProvider=Google'), { 'set-cookie': clearState });
        }
      }
      if (pathname === '/signout' && req.method === 'GET') {
        const id = cookies(req)[SESSION_COOKIE];
        if (id) db.prepare('DELETE FROM sessions WHERE hash = ?').run(sha256(id));
        return redirect('/signin', { 'set-cookie': cookie(SESSION_COOKIE, '', { maxAge: 0, secure }) });
      }
      if (pathname === '/app/v1/auth/signout' && req.method === 'POST' && bearer(req) && appSession(req)) {
        db.prepare('DELETE FROM app_tokens WHERE hash = ?').run(sha256(bearer(req)));
        res.writeHead(204, { 'cache-control': 'no-store' });
        res.end();
        return true;
      }
      if (pathname === '/app/v1/me' && req.method === 'GET' && signedIn) {
        res.writeHead(200, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
        res.end(JSON.stringify({ email: signedIn.email, kind: signedIn.sub.startsWith('password:') ? 'password' : 'google' }));
        return true;
      }
      if (signedIn && pathname !== '/app/v1/auth/signout') return false;
      if ((tunnelled ? TUNNEL_PUBLIC : PUBLIC).some((pattern) => pattern.test(pathname))) return false;
      if (document && !bearer(req)) return redirect('/signin');
      res.writeHead(401, { 'content-type': 'application/json', 'cache-control': 'no-store' });
      res.end('{"type":"about:blank","status":401,"title":"Unauthorized"}');
      return true;
    },

    /**
     * Whether a request (including a WebSocket upgrade) carries a valid session.
     * @param {import('node:http').IncomingMessage} req Incoming request.
     * @returns {boolean} True for a signed-in session.
     */
    allows(req) {
      return Boolean(session(req));
    },

    /**
     * Who a request is signed in as.
     * @param {import('node:http').IncomingMessage} req Incoming request.
     * @returns {{sub: string, email: string, credential: string}|null} The session's or app token's identity, with
     *   `credential`, a name for that session to check later with `active`; null when signed out.
     */
    identity(req) {
      const row = session(req);
      return row ? { sub: row.sub, email: row.email, credential: row.hash } : null;
    },

    /**
     * Who a session or app token named by `identity` is still signed in as.
     * @param {string} credential The `credential` of an earlier `identity`.
     * @returns {{sub: string, email: string}|null} Its identity, or null once it was signed out or has expired.
     */
    active(credential) {
      const row = db.prepare('SELECT sub, email, expires_ms FROM sessions WHERE hash = ? UNION ALL SELECT sub, email, expires_ms FROM app_tokens WHERE hash = ?')
        .get(String(credential), String(credential));
      return row && row.expires_ms >= Date.now() ? { sub: row.sub, email: row.email } : null;
    },

    /** Close the session database, unless the caller gave it. */
    close() {
      if (!injected) db.close();
    },
  };
}
