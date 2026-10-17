// The Cloudflare Worker that hosts the libraries. Every request is checked by the Auth Durable Object (lib/auth.mjs);
// a signed-in one goes to its account's Library Durable Object, which serves that library from its own SQLite
// storage and R2. /_admin/ (tools/upload-hosted.mjs) exists only while the ADMIN_TOKEN secret is set.
import { createHash, timingSafeEqual } from 'node:crypto';
import { downloadPage } from '../lib/download-page.mjs';

export { Auth } from './auth-object.mjs';
export { Library } from './library-object.mjs';
export { Media } from './media-object.mjs';

// Sign-in endpoints that read the request body; the Auth object always answers them itself.
const SIGNIN_BODY = /^\/(signin\/password|app\/v1\/auth\/(nonce|google|password))$/;
const text = (status, body, headers = {}) => new Response(body, { status, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store', ...headers } });

// Whether the request carries the admin token; false whenever no token is configured.
function isAdmin(request, env) {
  if (!env.ADMIN_TOKEN) return false;
  const given = /^Bearer (\S+)$/.exec(request.headers.get('authorization') ?? '')?.[1];
  const digest = (value) => createHash('sha256').update(value).digest();
  return Boolean(given) && timingSafeEqual(digest(given), digest(env.ADMIN_TOKEN));
}

// An object key from a URL path, or null for one that is absolute or climbs.
function objectKey(path) {
  let key;
  try { key = decodeURIComponent(path); } catch { return null; }
  return key && !key.startsWith('/') && !key.split('/').some((part) => part === '' || part === '.' || part === '..') ? key : null;
}

// Import routes: R2 objects, library rows and sign-in rows.
async function admin(request, env, url) {
  const [, area, ...rest] = url.pathname.split('/').slice(1);
  if (area === 'bucket' && !rest.length && request.method === 'GET') {
    let objects = 0, bytes = 0, cursor;
    do {
      const page = await env.BUCKET.list({ prefix: url.searchParams.get('prefix') ?? undefined, cursor });
      for (const object of page.objects) { objects++; bytes += object.size; }
      cursor = page.truncated ? page.cursor : undefined;
    } while (cursor);
    return Response.json({ objects, bytes });
  }
  if (area === 'bucket') {
    const key = objectKey(rest.join('/'));
    if (!key) return text(400, 'Invalid object key');
    if (request.method === 'HEAD') {
      const object = await env.BUCKET.head(key);
      return new Response(null, { status: object ? 200 : 404, headers: object ? { 'x-object-size': String(object.size) } : {} });
    }
    if (request.method !== 'PUT') return text(405, 'Method not allowed');
    const headers = request.headers.get('x-object-headers');
    if (headers) JSON.parse(headers);
    await env.BUCKET.put(key, request.body, headers ? { customMetadata: { headers } } : {});
    return new Response(null, { status: 204 });
  }
  if (area === 'library' && rest.length === 2 && /^[a-z0-9-]+$/.test(rest[0])) {
    const headers = new Headers({ 'x-mymind-library': rest[0], 'x-mymind-admin': rest[1], 'content-type': 'application/json' });
    return env.LIBRARY.get(env.LIBRARY.idFromName(rest[0])).fetch(new Request(request.url, { method: request.method, headers, body: request.body, redirect: 'manual' }));
  }
  const auth = env.AUTH_OBJECT.get(env.AUTH_OBJECT.idFromName('auth'));
  if (area === 'auth' && !rest.length && request.method === 'GET') return Response.json(await auth.summary());
  if (area === 'auth' && rest[0] === 'user' && request.method === 'POST') {
    const user = await request.json();
    await auth.putUser({ ...user, salt: Buffer.from(user.salt, 'base64'), hash: Buffer.from(user.hash, 'base64') });
    return new Response(null, { status: 204 });
  }
  if (area === 'auth' && rest[0] === 'library' && request.method === 'POST') {
    const { account, library } = await request.json();
    await auth.setLibrary(account, library);
    return new Response(null, { status: 204 });
  }
  return text(404, 'Not found');
}

// The packaged desktop app, for any signed-in visitor.
async function download(request, env, url, user) {
  if (request.method !== 'GET' && request.method !== 'HEAD') return text(405, '', { allow: 'GET, HEAD' });
  const manifest = await env.BUCKET.get('downloads/manifest.json').then((object) => object?.json()) ?? { files: [] };
  if (url.pathname === '/download' || url.pathname === '/download/') {
    return new Response(downloadPage(manifest, user), { headers: { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' } });
  }
  let name;
  try { name = decodeURIComponent(url.pathname.slice('/download/'.length)); } catch { name = null; }
  const file = manifest.files.find((entry) => entry.name === name);
  const object = file && (request.method === 'HEAD' ? await env.BUCKET.head(`downloads/${file.name}`) : await env.BUCKET.get(`downloads/${file.name}`));
  if (!object) return text(404, 'Not found.\n');
  return new Response(request.method === 'HEAD' ? null : object.body, { headers: { 'content-type': 'application/gzip', 'content-length': String(object.size),
    'content-disposition': `attachment; filename="${file.name}"`, 'cache-control': 'no-store' } });
}

export default {
  async fetch(request, env) {
    if (env.AUTH !== 'google') return text(503, 'This server has no sign-in configured.');
    const url = new URL(request.url);
    const headers = new Headers(request.headers);
    // Only this Worker names libraries and sessions to the Durable Objects.
    for (const name of [...headers.keys()]) if (name.startsWith('x-mymind-')) headers.delete(name);
    if (!headers.has('cf-connecting-ip')) headers.set('cf-connecting-ip', '127.0.0.1');
    if (url.pathname.startsWith('/_admin/') && isAdmin(request, env)) return admin(request, env, url);

    const auth = env.AUTH_OBJECT.get(env.AUTH_OBJECT.idFromName('auth'));
    const withBody = request.method === 'POST' && SIGNIN_BODY.test(url.pathname);
    const bare = new Headers(headers);
    for (const name of ['content-length', 'transfer-encoding']) bare.delete(name);
    // A redirect from sign-in goes back to the browser, never followed here.
    const gate = await auth.fetch(withBody ? new Request(request, { headers, redirect: 'manual' }) : new Request(request.url, { method: request.method, headers: bare, redirect: 'manual' }));
    const library = gate.headers.get('x-mymind-library');
    const listing = url.pathname === '/download' || url.pathname === '/download/';
    // As the tunnel's download server did, the download page sends every signed-out visitor to sign in.
    if (!library && listing && gate.status === 401) return new Response(null, { status: 302, headers: { location: '/signin', 'cache-control': 'no-store' } });
    if (!library) return gate;
    if (withBody) return text(500, 'Sign-in did not answer');
    if (url.pathname === '/download' || url.pathname.startsWith('/download/')) return download(request, env, url, gate.headers.get('x-mymind-email'));

    headers.set('x-mymind-library', library);
    headers.set('x-mymind-credential', gate.headers.get('x-mymind-credential'));
    return env.LIBRARY.get(env.LIBRARY.idFromName(library)).fetch(new Request(request, { headers, redirect: 'manual' }));
  },
};
