import { createHash, createPublicKey, randomBytes, verify } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { DatabaseSync } from 'node:sqlite';

const SESSION_COOKIE = 'mm_session';
const STATE_COOKIE = 'mm_oauth';
const SESSION_MS = 30 * 24 * 3600 * 1000;
const LOGIN_MS = 10 * 60 * 1000;
const SKEW_S = 60;
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

/**
 * Create Google sign-in for the replica, or return null when the environment does not turn it on.
 * Sign-in is on only with `AUTH=google`, which also requires `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` and
 * `OWNER_EMAIL`; only the owner's verified Google account may sign in, and it reaches the existing library.
 * @param {object} options
 * @param {string} options.root Repository root; sessions live in `data/auth.db` (or `AUTH_DB`), sign-in pages in `capture/signin/`.
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

  const dbPath = env.AUTH_DB ?? join(root, 'data', 'auth.db');
  mkdirSync(dirname(dbPath), { recursive: true });
  const db = new DatabaseSync(dbPath);
  db.exec(`CREATE TABLE IF NOT EXISTS sessions (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL,
    created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT`);
  db.prepare('DELETE FROM sessions WHERE expires_ms < ?').run(Date.now());
  const logins = new Map();
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

  // Finish a sign-in attempt: a new session id for the owner, or the problem mymind's page names.
  async function callback(url, req) {
    const state = url.searchParams.get('state') ?? '';
    const login = logins.get(state);
    logins.delete(state);
    if (!login || cookies(req)[STATE_COOKIE] !== state || Date.now() - login.created > LOGIN_MS) throw new Error('unknown sign-in attempt');
    if (url.searchParams.get('error')) return { problem: 'Cancelled' };
    const claims = await verifyIdToken(await exchange(url.searchParams.get('code') ?? '', login.verifier), login.nonce);
    if (claims.email_verified !== true || String(claims.email).toLowerCase() !== owner) return { problem: 'AccountNotFound' };
    const id = base64url(randomBytes(32));
    const now = Date.now();
    db.prepare('INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)')
      .run(sha256(id), claims.sub, owner, now, now + SESSION_MS, now);
    return { id };
  }

  return {
    /**
     * Answer sign-in, sign-out and gate every other request that has no valid session.
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
     * @returns {boolean} True for a signed-in owner session.
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
