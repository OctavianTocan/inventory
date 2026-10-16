// Serves the packaged desktop app at /download to anyone signed in to the hosted library (Google or password): each
// request needs a session from the library's sign-in, so the files stay private even if the tunnel's routing changes.
// Signed-out visitors are sent to the sign-in page.
// Env: DOWNLOAD_PORT (default 4332), DOWNLOAD_DIR (default downloads/), AUTH_DB (default data/auth.db, shared with the
// library server).
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { readFile, stat } from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { openAuthStore } from '../lib/auth-store.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const port = Number(process.env.DOWNLOAD_PORT ?? 4332);
const dir = path.resolve(root, process.env.DOWNLOAD_DIR ?? 'downloads');
const auth = openAuthStore(process.env.AUTH_DB ?? path.join(root, 'data', 'auth.db'));
auth.exec('PRAGMA busy_timeout = 2000');

// The email of the unexpired mymind session in the request's cookie, or null. Same cookie and stored hash as
// lib/auth.mjs.
function mymindUser(req) {
  const id = /(?:^|;\s*)mm_session=([^;]+)/.exec(req.headers.cookie ?? '')?.[1];
  if (!id) return null;
  const row = auth.prepare('SELECT email, expires_ms FROM sessions WHERE hash = ?').get(createHash('sha256').update(id).digest('hex'));
  return row && row.expires_ms > Date.now() ? row.email : null;
}

const escape = (text) => String(text).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const size = (bytes) => `${(bytes / 1_048_576).toFixed(1)} MB`;

// The download page: every packaged file with its size and checksum, and how to install it.
async function page(user) {
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
<p>Signed in as ${escape(user)} · <a href="/signout">Sign out</a></p>
</main></body></html>`;
}

http.createServer(async (req, res) => {
  const { pathname } = new URL(req.url, 'http://download');
  if (req.method !== 'GET' && req.method !== 'HEAD') {
    res.writeHead(405, { allow: 'GET, HEAD' });
    return res.end();
  }
  const listing = pathname === '/download' || pathname === '/download/';
  const user = mymindUser(req);
  if (!user) {
    console.warn(`refused ${req.method} ${req.url}: no session`);
    if (listing) res.writeHead(302, { location: '/signin', 'cache-control': 'no-store' });
    else res.writeHead(401, { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' });
    return res.end(listing ? undefined : 'Sign in to download the app.\n');
  }
  if (listing) {
    res.writeHead(200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' });
    return res.end(await page(user));
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
}).listen(port, '127.0.0.1', () => console.log(`downloads for signed-in users on http://127.0.0.1:${port}/download`));
