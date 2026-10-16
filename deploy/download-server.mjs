// Serves the packaged desktop app at /download, only to requests that carry a valid token of the hosted site's
// Cloudflare Access application, so whoever its policy lets in can download and the files stay behind Access even if
// the tunnel's routing changes.
// Env: CF_ACCESS_TEAM_DOMAIN (e.g. you.cloudflareaccess.com), CF_ACCESS_AUD (the Access application's audience tag),
// DOWNLOAD_PORT (default 4332), DOWNLOAD_DIR (default downloads/), CF_ACCESS_CERTS_URL (default the team's).
import { createPublicKey, verify } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { readFile, stat } from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { CF_ACCESS_TEAM_DOMAIN: team, CF_ACCESS_AUD: audience } = process.env;
const port = Number(process.env.DOWNLOAD_PORT ?? 4332);
const dir = path.resolve(root, process.env.DOWNLOAD_DIR ?? 'downloads');
if (!team || !audience) {
  console.error('CF_ACCESS_TEAM_DOMAIN and CF_ACCESS_AUD are required.');
  process.exit(1);
}
const issuer = `https://${team}`;
const certs = process.env.CF_ACCESS_CERTS_URL ?? `${issuer}/cdn-cgi/access/certs`;
let keys = { fetched: 0, byKid: new Map() };

// Cloudflare Access's public key for `kid`.
async function signingKey(kid) {
  if (!keys.byKid.has(kid) || Date.now() - keys.fetched > 3_600_000) {
    const res = await fetch(certs);
    if (!res.ok) throw new Error(`certs ${res.status}`);
    const { keys: jwks } = await res.json();
    keys = { fetched: Date.now(), byKid: new Map(jwks.map((jwk) => [jwk.kid, createPublicKey({ key: jwk, format: 'jwk' })])) };
  }
  const key = keys.byKid.get(kid);
  if (!key) throw new Error('unknown signing key');
  return key;
}

// Who a verified Access token of this application names (a person's email or a service token's client ID), or why
// the token is refused.
async function accessIdentity(token) {
  if (!token) return { refused: 'no Access token' };
  try {
    const [header, payload, signature] = token.split('.');
    const { alg, kid } = JSON.parse(Buffer.from(header, 'base64url'));
    if (alg !== 'RS256') return { refused: `algorithm ${alg}` };
    if (!verify('RSA-SHA256', Buffer.from(`${header}.${payload}`), await signingKey(kid), Buffer.from(signature, 'base64url'))) {
      return { refused: 'bad signature' };
    }
    const claims = JSON.parse(Buffer.from(payload, 'base64url'));
    const now = Date.now() / 1000;
    if (claims.iss !== issuer) return { refused: `issuer ${claims.iss}` };
    if (![claims.aud].flat().includes(audience)) return { refused: 'another application' };
    if (!(claims.exp > now) || (claims.nbf ?? 0) > now + 60) return { refused: 'expired' };
    const identity = claims.email || claims.common_name;
    return identity ? { identity } : { refused: 'no email or service token' };
  } catch (error) {
    return { refused: error.message };
  }
}

const escape = (text) => String(text).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const size = (bytes) => `${(bytes / 1_048_576).toFixed(1)} MB`;

// The download page: every packaged file with its size and checksum, and how to install it.
async function page(identity) {
  const manifest = JSON.parse(await readFile(path.join(dir, 'manifest.json'), 'utf8').catch(() => '{"files":[]}'));
  const rows = manifest.files.map((file) => `<li><a href="/download/${encodeURIComponent(file.name)}">${escape(file.label)}</a>
    <span>${size(file.bytes)} · built ${escape(file.built.slice(0, 10))}</span><code>sha256 ${escape(file.sha256)}</code></li>`).join('');
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>mymind app</title><style>
:root{color-scheme:light dark;--bg:#f7f8fa;--fg:#1d2433;--muted:#6b7280;--accent:#ff5924;--card:#fff}
@media (prefers-color-scheme:dark){:root{--bg:#14161b;--fg:#e8eaf0;--muted:#9aa1ad;--card:#1d2027}}
body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}
main{max-width:640px;margin:0 auto;padding:48px 16px}h1{font:600 2rem Georgia,serif;margin:0 0 8px}
p,span{color:var(--muted)}ul{list-style:none;padding:0}li{background:var(--card);border-radius:14px;padding:16px;margin:12px 0}
a{color:var(--accent);font-weight:600}li span,li code{display:block;font-size:.85rem;overflow-wrap:anywhere}
pre{background:var(--card);border-radius:14px;padding:16px;overflow-x:auto;font-size:.85rem}
</style></head><body><main><h1>mymind app</h1>
<p>Opens your library at ${escape(manifest.library ?? 'the hosted address')}; sign in with Google the first time.</p>
<ul>${rows || '<li>No app has been packaged yet. Run <code>deploy/package-app.sh</code> on the host.</li>'}</ul>
<pre>tar -xzf mymind-linux-x86_64.tar.gz
mymind/install.sh        # ~/.local/bin/mymind and a launcher entry</pre>
<p>Needs WebKitGTK 4.1 and GStreamer's good plugins (Arch: <code>webkit2gtk-4.1 gst-plugins-good</code>).</p>
<p>Signed in to Cloudflare Access as ${escape(identity)} · <a href="/cdn-cgi/access/logout">Sign out</a></p>
</main></body></html>`;
}

http.createServer(async (req, res) => {
  const { identity, refused } = await accessIdentity(req.headers['cf-access-jwt-assertion']);
  if (!identity) {
    console.warn(`refused ${req.method} ${req.url}: ${refused}`);
    res.writeHead(403, { 'content-type': 'text/plain; charset=utf-8' });
    return res.end('Sign in through Cloudflare Access to download the app.\n');
  }
  const { pathname } = new URL(req.url, 'http://download');
  if (req.method !== 'GET' && req.method !== 'HEAD') {
    res.writeHead(405, { allow: 'GET, HEAD' });
    return res.end();
  }
  if (pathname === '/download' || pathname === '/download/') {
    res.writeHead(200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' });
    return res.end(await page(identity));
  }
  const name = decodeURIComponent(pathname.slice('/download/'.length));
  const manifest = JSON.parse(await readFile(path.join(dir, 'manifest.json'), 'utf8').catch(() => '{"files":[]}'));
  const file = manifest.files.find((entry) => entry.name === name);
  const info = file && (await stat(path.join(dir, file.name)).catch(() => null));
  if (!info) {
    res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    return res.end('Not found.\n');
  }
  res.writeHead(200, {
    'content-type': 'application/gzip',
    'content-length': info.size,
    'content-disposition': `attachment; filename="${file.name}"`,
    'cache-control': 'no-store',
  });
  if (req.method === 'HEAD') return res.end();
  createReadStream(path.join(dir, file.name)).pipe(res);
}).listen(port, '127.0.0.1', () => console.log(`downloads behind Cloudflare Access on http://127.0.0.1:${port}/download`));
