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
import { downloadPage } from '../lib/download-page.mjs';

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

// The download page for the packages in the manifest.
async function page(user) {
  return downloadPage(JSON.parse(await readFile(path.join(dir, 'manifest.json'), 'utf8').catch(() => '{"files":[]}')), user);
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
  let name;
  try { name = decodeURIComponent(pathname.slice('/download/'.length)); } catch { name = null; }
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
