import { createHash, createPublicKey, randomBytes, verify } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { checkPassword, openAuthStore, passwordSubject } from './auth-store.mjs';

const SESSION_COOKIE = 'mm_session';
const STATE_COOKIE = 'mm_oauth';
const SESSION_MS = 30 * 24 * 3600 * 1000;
const LOGIN_MS = 10 * 60 * 1000;
const SKEW_S = 60;
const FAILURE_WINDOW_MS = 15 * 60 * 1000;
const FAILURES_PER_USER = 10;
const FAILURES_PER_ADDRESS = 30;
// Paths a signed-out browser needs to show the sign-in page and finish signing in.
const PUBLIC = [/^\/signin$/, /^\/google\/oauth2-(redirect|callback)$/, /^\/_static\//, /^\/_auth\//, /^\/manifest\.webmanifest$/, /^\/favicon\.ico$/, /^\/jserrors$/];

const base64url = (bytes) => Buffer.from(bytes).toString('base64url');
const sha256 = (text) => createHash('sha256').update(text).digest('hex');

// Parse the request's Cookie header into a name → value map.
function cookies(req) {
  return Object.fromEntries((req.headers.cookie ?? '').split(';').map((part) => part.trim().split('=')).filter(([name]) => name)
    .map(([name, ...value]) => [name, decodeURIComponent(value.join('='))]));
}

// Serialize one Set-Cookie value with the attributes every auth cookie shares.
function cookie(name, value, { maxAge, secure, path = '/' }) {
  return `${name}=${encodeURIComponent(value)}; Path=${path}; Max-Age=${Math.floor(maxAge / 1000)}; HttpOnly; SameSite=Lax${secure ? '; Secure' : ''}`;
}

// Page loaded in the sign-in popup: hands the result to the sign-in window and closes, or navigates itself.
function popupPage(target) {
  const to = JSON.stringify(target).replaceAll('<', '\\u003c');
  return `<!doctype html><meta charset="utf-8"><title>mymind</title><script>
const to = ${to};
try { new BroadcastChannel('mymind-auth').postMessage(to); } catch {}
try { if (opener && opener.location.origin === location.origin) { opener.location.assign(to); close(); } } catch {}
if (opener) close();
setTimeout(() => location.replace(to), 300);
</script>`;
}

const PASSWORD_ERRORS = {
  invalid: 'That username and password do not match.',
  locked: 'Too many attempts. Try again in 15 minutes.',
};

// Username and password form at /signin/password, for the password users in the auth database.
function passwordPage(error) {
  const message = Object.hasOwn(PASSWORD_ERRORS, error ?? '') ? `<p class="error" role="alert">${PASSWORD_ERRORS[error]}</p>` : '';
  return `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Sign in – mymind</title><style>
:root { color-scheme: light dark; --bg: #f5f5f3; --card: #fff; --text: #1d1d1f; --muted: #6e6e73; --line: #d8d8dc; --accent: #ff5924; }
@media (prefers-color-scheme: dark) { :root { --bg: #111113; --card: #1c1c1f; --text: #f2f2f4; --muted: #a1a1a6; --line: #333338; } }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 16px; background: var(--bg); color: var(--text); font: 16px/1.4 system-ui, sans-serif; }
form { width: 100%; max-width: 360px; display: grid; gap: 14px; padding: 32px 28px; background: var(--card); border: 1px solid var(--line); border-radius: 16px; }
h1 { margin: 0 0 6px; font-size: 22px; font-weight: 600; }
label { display: grid; gap: 6px; font-size: 14px; color: var(--muted); }
input { font: inherit; color: var(--text); background: transparent; padding: 11px 12px; border: 1px solid var(--line); border-radius: 10px; }
input:focus { outline: 2px solid var(--accent); outline-offset: -1px; border-color: transparent; }
button { font: inherit; font-weight: 600; margin-top: 6px; padding: 12px; border: 0; border-radius: 10px; background: var(--accent); color: #fff; cursor: pointer; }
.error { margin: 0; color: #d93a1a; font-size: 14px; }
a { color: var(--muted); font-size: 14px; text-align: center; }
</style>
<form method="post" action="/signin/password">
<h1>Sign in to mymind</h1>${message}
<label>Username <input name="username" autocomplete="username" autocapitalize="none" spellcheck="false" required autofocus></label>
<label>Password <input name="password" type="password" autocomplete="current-password" required></label>
<button type="submit">Sign in</button>
<a href="/signin">Sign in with Google instead</a>
</form></html>`;
}

// Read a small urlencoded form body; null when it is too large.
async function readForm(req, limit = 4096) {
  let text = '';
  for await (const chunk of req) {
    text += chunk;
    if (text.length > limit) return null;
  }
  return new URLSearchParams(text);
}

/**
 * Create sign-in for the replica, or return null when the environment does not turn it on.
 * Sign-in is on only with `AUTH=google`, which also requires `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` and
 * `OWNER_EMAIL`; only the owner's verified Google account may sign in with Google. Password users added with
 * `tools/auth-user.mjs` may also sign in at `/signin/password`. Every signed-in session reaches the existing library.
 * @param {object} options
 * @param {string} options.root Repository root; sessions and password users live in `data/auth.db` (or `AUTH_DB`), sign-in pages in `capture/signin/`.
 * @param {(text:string) => string} options.localize Rewrite recorded origins to local paths.
 * @param {Record<string,string|undefined>} [options.env=process.env] Configuration source.
 * @param {number} options.port Port the server listens on, for the default public URL.
 * @returns {object|null} Auth handlers, or null when sign-in is off.
 * @throws {Error} If `AUTH=google` is set without the client credentials or owner email.
 */
export function createAuth({ root, localize, env = process.env, port }) {
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

  const db = openAuthStore(env.AUTH_DB ?? join(root, 'data', 'auth.db'));
  db.prepare('DELETE FROM sessions WHERE expires_ms < ?').run(Date.now());
  const logins = new Map();
  const failures = new Map();
  let keys = { fetched: 0, maxAge: 0, byKid: new Map() };

  // Fetch Google's signing keys, honouring their cache lifetime; refetch once for an unknown key id.
  async function signingKey(kid) {
    const stale = Date.now() - keys.fetched > keys.maxAge;
    if (stale || !keys.byKid.has(kid)) {
      const res = await fetch(google.certs);
      if (!res.ok) throw new Error(`certs ${res.status}`);
      const maxAge = Number(/max-age=(\d+)/.exec(res.headers.get('cache-control') ?? '')?.[1] ?? 3600) * 1000;
      keys = { fetched: Date.now(), maxAge, byKid: new Map((await res.json()).keys.map((jwk) => [jwk.kid, createPublicKey({ key: jwk, format: 'jwk' })])) };
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
  function session(req) {
    const id = cookies(req)[SESSION_COOKIE];
    if (!id) return null;
    const row = db.prepare('SELECT hash, sub, email, expires_ms, used_ms FROM sessions WHERE hash = ?').get(sha256(id));
    if (!row || row.expires_ms < Date.now()) return null;
    if (Date.now() - row.used_ms > 3600_000) db.prepare('UPDATE sessions SET used_ms = ? WHERE hash = ?').run(Date.now(), row.hash);
    return row;
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

  // Failed password attempts for `key` within the window, forgetting older ones.
  function recentFailures(key) {
    const since = Date.now() - FAILURE_WINDOW_MS;
    const times = (failures.get(key) ?? []).filter((time) => time > since);
    if (times.length) failures.set(key, times); else failures.delete(key);
    return times.length;
  }

  // Finish a sign-in attempt: a new session id for the owner, or the problem mymind's page names.
  async function callback(url, req) {
    const state = url.searchParams.get('state') ?? '';
    const login = logins.get(state);
    logins.delete(state);
    if (!login || cookies(req)[STATE_COOKIE] !== state || Date.now() - login.created > LOGIN_MS) throw new Error('unknown sign-in attempt');
    if (url.searchParams.get('error')) return { problem: 'Cancelled' };
    const claims = await verifyIdToken(await exchange(url.searchParams.get('code') ?? '', login.verifier), login.nonce);
    if (claims.email_verified !== true || String(claims.email).toLowerCase() !== owner) return { problem: 'AccountNotFound' };
    return { id: startSession(claims.sub, owner) };
  }

  // Check a password form post; a new session id, or the error the form shows.
  async function passwordSignIn(req) {
    const form = await readForm(req);
    const username = String(form?.get('username') ?? '').trim().toLowerCase();
    const keys = [`user:${username}`, `address:${req.headers['cf-connecting-ip'] ?? req.socket.remoteAddress}`];
    if (recentFailures(keys[0]) >= FAILURES_PER_USER || recentFailures(keys[1]) >= FAILURES_PER_ADDRESS) return { error: 'locked' };
    const name = form && await checkPassword(db, username, form.get('password'));
    if (!name) {
      for (const key of keys) failures.set(key, [...(failures.get(key) ?? []), Date.now()]);
      if (failures.size > 10_000) failures.delete(failures.keys().next().value);
      return { error: 'invalid' };
    }
    failures.delete(keys[0]);
    return { id: startSession(passwordSubject(name), name) };
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
      if (req.method !== 'GET' && req.headers.origin && req.headers.origin !== new URL(publicUrl).origin && req.headers.origin !== `http://${req.headers.host}`) {
        res.writeHead(403); res.end('cross-origin write refused'); return true;
      }
      if (pathname === '/signin' && req.method === 'GET') return signedIn ? redirect('/everything') : html(await signinPage(url.searchParams.get('error')));
      if (pathname === '/signin/password') {
        if (!db.prepare('SELECT 1 FROM users LIMIT 1').get()) { res.writeHead(404); res.end(); return true; }
        if (req.method === 'GET') return signedIn ? redirect('/everything') : html(passwordPage(url.searchParams.get('error')));
        if (req.method !== 'POST') { res.writeHead(405, { allow: 'GET, POST' }); res.end(); return true; }
        const result = await passwordSignIn(req);
        const seeOther = (location, extra = {}) => { res.writeHead(303, { location, 'cache-control': 'no-store', ...extra }); res.end(); return true; };
        if (result.error) return seeOther(`/signin/password?error=${result.error}`);
        return seeOther('/everything', { 'set-cookie': cookie(SESSION_COOKIE, result.id, { maxAge: SESSION_MS, secure }) });
      }
      if (pathname === '/_auth/signin.js') {
        res.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
        res.end(`new BroadcastChannel('mymind-auth').onmessage = (event) => location.assign(event.data);
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
        for (const [key, login] of logins) if (Date.now() - login.created > LOGIN_MS) logins.delete(key);
        logins.set(state, { verifier, nonce, created: Date.now() });
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
      if (signedIn || PUBLIC.some((pattern) => pattern.test(pathname))) return false;
      if (document) return redirect('/signin');
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

    /** Close the session database. */
    close() {
      db.close();
    },
  };
}
