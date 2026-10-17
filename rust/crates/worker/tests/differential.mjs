// Differential test: the JS Worker (REPO/worker) and the Rust Worker (this crate) under local `wrangler dev`, seeded
// identically and compared request by request. Run after `worker-build --release`:
//   MYMIND_ROOT=<repo> MYMIND_TEST_HOSTED_DB=<library fixture> node crates/worker/tests/differential.mjs
// Local only: no Cloudflare API, no deploy, throwaway ADMIN_TOKEN and dummy Google credentials.
import { spawn } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const CRATE = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REPO = process.env.MYMIND_ROOT;
const HOSTED_DB = process.env.MYMIND_TEST_HOSTED_DB;
if (!REPO || !HOSTED_DB) throw new Error('MYMIND_ROOT and MYMIND_TEST_HOSTED_DB are required');
const WORK = path.resolve(CRATE, '../../target/diff');
const WRANGLER = path.join(REPO, 'worker/node_modules/wrangler/bin/wrangler.js');
const TOKEN = randomBytes(16).toString('hex');
const PASSWORD = `pw-${randomBytes(12).toString('hex')}`;
const LIBRARY = 'testlib';

const { setPassword, installAuthSchema } = await import(`${REPO}/lib/auth-store.mjs`);
const { openSql } = await import(`${REPO}/lib/sql-node.mjs`);
const { encodeValue, tableDigest } = await import(`${REPO}/lib/library-rows.mjs`);
const { DatabaseSync } = await import('node:sqlite');

const jsonc = (file) => JSON.parse(readFileSync(file, 'utf8').replace(/^\s*\/\/.*$/gm, ''));
const freePort = () => new Promise((resolve) => { const s = net.createServer().listen(0, '127.0.0.1', () => { const { port } = s.address(); s.close(() => resolve(port)); }); });

function stageConfig(name, config) {
  const dir = path.join(WORK, name);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const { containers, routes, build, ...rest } = config;
  writeFileSync(path.join(dir, 'wrangler.jsonc'), JSON.stringify({ ...rest, workers_dev: true }, null, 2));
  writeFileSync(path.join(dir, '.dev.vars'), `ADMIN_TOKEN=${TOKEN}\nGOOGLE_CLIENT_ID=dummy-client\nGOOGLE_CLIENT_SECRET=dummy-secret\nOWNER_EMAIL=owner@example.test\nPUBLIC_URL=http://127.0.0.1\n`);
  return dir;
}

async function start(name, config) {
  const dir = stageConfig(name, config);
  const port = await freePort();
  const child = spawn(process.execPath, [WRANGLER, 'dev', '-c', path.join(dir, 'wrangler.jsonc'), '--port', String(port), '--ip', '127.0.0.1',
    '--persist-to', path.join(dir, 'state'), '--inspector-port', String(await freePort())], {
    cwd: dir, detached: true, stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, XDG_CONFIG_HOME: path.join(WORK, 'xdg'), WRANGLER_SEND_METRICS: 'false', CI: '1', NO_PROXY: '127.0.0.1,localhost', no_proxy: '127.0.0.1,localhost' },
  });
  const log = [];
  child.stdout.on('data', (chunk) => log.push(String(chunk)));
  child.stderr.on('data', (chunk) => log.push(String(chunk)));
  const base = `http://127.0.0.1:${port}`;
  const started = Date.now();
  for (;;) {
    if (child.exitCode !== null) throw new Error(`${name} wrangler exited:\n${log.join('').slice(-2000)}`);
    if (Date.now() - started > 180000) throw new Error(`${name} did not start:\n${log.join('').slice(-2000)}`);
    try { await fetch(`${base}/robots.txt`, { redirect: 'manual' }); console.log(`${name} answered its first request ${Date.now() - started} ms after wrangler started`); break; } catch { await new Promise((r) => setTimeout(r, 500)); }
  }
  return { name, base, child, log, port };
}

const stop = (worker) => { try { process.kill(-worker.child.pid, 'SIGTERM'); } catch {} };

async function raw(worker, { method = 'GET', url, headers = {}, body }) {
  const response = await fetch(worker.base + url, { method, headers, body, redirect: 'manual' });
  const bytes = Buffer.from(await response.arrayBuffer());
  return { status: response.status, headers: response.headers, bytes };
}

const admin = (worker, route, init = {}) => raw(worker, { ...init, url: `/_admin/${route}`, headers: { authorization: `Bearer ${TOKEN}`, ...init.headers } });
const adminJson = async (worker, route, body) => {
  const response = await admin(worker, route, { method: 'POST', body: JSON.stringify(body), headers: { 'content-type': 'application/json' } });
  if (response.status >= 300) throw new Error(`${worker.name} ${route}: ${response.status} ${response.bytes.toString().slice(0, 200)}`);
  return response.bytes.length ? JSON.parse(response.bytes.toString()) : null;
};

function tablesOf(db) {
  const names = db.prepare("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' AND name != 'search_dirty'").all();
  const tables = names.map(({ name, sql }) => {
    const columns = db.prepare('SELECT name, type, pk, hidden FROM pragma_table_xinfo(?)').all(name);
    const keys = columns.filter((column) => column.pk);
    const alias = keys.length === 1 && keys[0].type.toUpperCase() === 'INTEGER';
    const rowid = !/WITHOUT\s+ROWID/i.test(sql) && !alias;
    const parents = [...new Set(db.prepare('SELECT "table" AS parent FROM pragma_foreign_key_list(?)').all(name).map((row) => row.parent))].filter((parent) => parent !== name);
    return { table: name, columns: [...(rowid ? ['rowid'] : []), ...columns.filter((column) => column.hidden === 0).map((column) => column.name)], rowid, parents };
  });
  const ordered = [];
  const visit = (table) => { if (ordered.includes(table)) return; for (const parent of table.parents) visit(tables.find((c) => c.table === parent)); ordered.push(table); };
  tables.forEach(visit);
  return ordered;
}

async function seed(worker, user) {
  const manifest = { library: 'testlib', files: [{ name: 'mymind-linux-x86_64.tar.gz', label: 'Linux x86_64', bytes: 6000, built: '2026-10-01T00:00:00Z', sha256: 'ab'.repeat(32) }, { name: 'with space & "q".tar.gz', label: 'Odd', bytes: 3, built: '2026-10-02T00:00:00Z', sha256: 'cd'.repeat(32) }] };
  const put = (key, body, headers = {}) => admin(worker, `bucket/${key.split('/').map(encodeURIComponent).join('/')}`, { method: 'PUT', body, headers });
  const results = [
    await put('downloads/manifest.json', JSON.stringify(manifest), { 'x-object-headers': JSON.stringify({ 'content-type': 'application/json' }) }),
    await put('downloads/mymind-linux-x86_64.tar.gz', Buffer.alloc(6000, 7)),
    await put('downloads/with space & "q".tar.gz', Buffer.from('abc')),
    await put('static/a.txt', 'hello', { 'x-object-headers': JSON.stringify({ 'content-type': 'text/plain' }) }),
  ];
  if (results.some((r) => r.status !== 204)) throw new Error(`${worker.name} bucket seed failed`);
  await adminJson(worker, 'auth/user', user);
  await adminJson(worker, 'auth/user', { ...user, username: 'nolib' });
  await adminJson(worker, 'auth/library', { account: 'password:tester', library: LIBRARY });
  const db = new DatabaseSync(HOSTED_DB, { readOnly: true });
  const tables = tablesOf(db);
  const reset = await adminJson(worker, `library/${LIBRARY}/reset`, {});
  for (const { table } of tables) if (!(table in reset.tables)) throw new Error(`${worker.name}: no table ${table}`);
  for (const { table, columns, rowid } of tables) {
    const statement = db.prepare(`SELECT ${columns.map((c) => `"${c}"`).join(', ')} FROM "${table}"${rowid ? ' ORDER BY rowid' : ''}`);
    statement.setReadBigInts(true);
    const rows = [...statement.iterate()].map((row) => columns.map((column) => encodeValue(row[column])));
    for (let i = 0; i < rows.length; i += 200) await adminJson(worker, `library/${LIBRARY}/rows`, { table, columns, rows: rows.slice(i, i + 200) });
  }
  const dirty = db.prepare('SELECT id FROM search_dirty').all().map((row) => [row.id]);
  const sequences = db.prepare('SELECT name, CAST(seq AS TEXT) AS seq FROM sqlite_sequence').all();
  await adminJson(worker, `library/${LIBRARY}/finish`, { dirty, sequences });
  const checked = [...tables, { table: 'search_dirty', columns: ['id'] }];
  db.close();
  const local = openSql(HOSTED_DB, { readOnly: true });
  const remote = await adminJson(worker, `library/${LIBRARY}/digest`, { tables: checked.map(({ table, columns }) => ({ table, columns })) });
  for (const { table, columns } of checked) {
    const mine = tableDigest(local, table, columns);
    if (mine.rows !== remote[table].rows || mine.sha256 !== remote[table].sha256) throw new Error(`${worker.name}: digest of ${table} differs from the fixture`);
  }
  local.close();
  return tables.length;
}

const cookieOf = (response) => (response.headers.getSetCookie?.() ?? []).map((c) => c.split(';')[0]).find((c) => /=./.test(c));

async function signIn(worker, username) {
  const response = await raw(worker, { method: 'POST', url: '/signin/password', headers: { 'content-type': 'application/x-www-form-urlencoded' }, body: new URLSearchParams({ username, password: PASSWORD }).toString() });
  return cookieOf(response);
}

async function appToken(worker) {
  const response = await raw(worker, { method: 'POST', url: '/app/v1/auth/password', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ username: 'tester', password: PASSWORD, device: 'diff' }) });
  const body = JSON.parse(response.bytes.toString());
  return { token: body.token ?? body.accessToken ?? body.access_token, keys: Object.keys(body) };
}

const DROP = new Set(['date', 'content-length', 'transfer-encoding', 'connection', 'keep-alive', 'content-encoding', 'vary', 'cf-ray', 'server']);
function normalize(response, masks = []) {
  const headers = {};
  for (const [name, value] of response.headers) if (!DROP.has(name)) headers[name] = name === 'set-cookie' ? value.replace(/^([^=]+)=[^;]*/, '$1=<v>') : value;
  if (headers.location) headers.location = headers.location.replace(/(state|nonce|code_challenge)=[^&]*/g, '$1=<r>');
  const setCookies = (response.headers.getSetCookie?.() ?? []).map((c) => c.replace(/^([^=]+)=[^;]*/, '$1=<v>').replace(/Expires=[^;]*/i, 'Expires=<d>'));
  if (setCookies.length) headers['set-cookie'] = setCookies;
  let body = response.bytes.toString('utf8');
  const binary = !/^(text\/|application\/json)/.test(response.headers.get('content-type') ?? '') && response.bytes.length;
  if (binary) body = `sha256:${createHash('sha256').update(response.bytes).digest('hex')} (${response.bytes.length} bytes)`;
  if (response.status === 500) { delete headers['content-type']; body = '<runtime error page>'; }
  for (const key of masks) body = body.replace(new RegExp(`("${key}":)("[^"]*"|[0-9.]+)`, 'g'), '$1"<masked>"');
  return { status: response.status, headers, body };
}

function websocket(worker, { cookie, key = true, messages = [], upgrade = true, wait = 600 }) {
  return new Promise((resolve) => {
    const headers = { connection: upgrade ? 'Upgrade' : 'close', ...(upgrade ? { upgrade: 'websocket', 'sec-websocket-version': '13' } : {}) };
    if (key) headers['sec-websocket-key'] = randomBytes(16).toString('base64');
    if (cookie) headers.cookie = cookie;
    const request = http.request({ host: '127.0.0.1', port: worker.port, path: '/hub', headers });
    const out = { status: null, frames: [] };
    const finish = () => resolve(out);
    request.on('response', (response) => { out.status = response.statusCode; response.resume(); response.on('end', finish); });
    request.on('upgrade', (response, socket) => {
      out.status = response.statusCode;
      let pending = Buffer.alloc(0);
      socket.on('data', (chunk) => {
        pending = Buffer.concat([pending, chunk]);
        for (;;) {
          if (pending.length < 2) return;
          let length = pending[1] & 127, offset = 2;
          if (length === 126) { if (pending.length < 4) return; length = pending.readUInt16BE(2); offset = 4; }
          else if (length === 127) { if (pending.length < 10) return; length = Number(pending.readBigUInt64BE(2)); offset = 10; }
          if (pending.length < offset + length) return;
          const opcode = pending[0] & 15, payload = pending.subarray(offset, offset + length);
          out.frames.push(opcode === 8 ? `close ${payload.readUInt16BE(0)} ${payload.subarray(2).toString()}` : `${opcode === 1 ? 'text' : `op${opcode}`} ${JSON.stringify(payload.toString())}`);
          pending = pending.subarray(offset + length);
        }
      });
      socket.on('close', finish);
      socket.on('error', finish);
      (async () => {
        for (const message of messages) {
          const payload = Buffer.from(message);
          const head = payload.length < 126 ? Buffer.from([0x81, 0x80 | payload.length]) : payload.length < 65536
            ? Buffer.from([0x81, 0xfe, payload.length >> 8, payload.length & 255]) : (() => { const b = Buffer.alloc(10); b[0] = 0x81; b[1] = 0xff; b.writeBigUInt64BE(BigInt(payload.length), 2); return b; })();
          socket.write(Buffer.concat([head, Buffer.alloc(4), payload]));
          await new Promise((r) => setTimeout(r, 150));
        }
        setTimeout(() => { socket.destroy(); finish(); }, wait);
      })();
    });
    request.on('error', () => finish());
    request.end();
  });
}

const HS = '{"protocol":"json","version":1}\u001e';
const sessionsFor = { none: () => ({}), user: (s) => ({ cookie: s.cookie }), nolib: (s) => ({ cookie: s.nolibCookie }), bearer: (s) => ({ authorization: `Bearer ${s.token}` }), badbearer: () => ({ authorization: 'Bearer nope' }) };

function cases() {
  const form = (data) => ({ headers: { 'content-type': 'application/x-www-form-urlencoded' }, body: new URLSearchParams(data).toString() });
  const json = (data) => ({ headers: { 'content-type': 'application/json' }, body: JSON.stringify(data) });
  const out = [];
  const add = (name, request, extra = {}) => out.push({ name, request, ...extra });
  for (const [path, method] of [['/signin', 'GET'], ['/signin?error=SignInFailed', 'GET'], ['/signin/password', 'GET'], ['/signin/password?error=invalid', 'GET'], ['/robots.txt', 'GET'], ['/_auth/signin.js', 'GET'],
    ['/signout', 'GET'], ['/everything', 'GET'], ['/api/x', 'GET'], ['/google/oauth2-redirect', 'GET'], ['/google/oauth2-callback', 'GET'], ['/signin/password', 'DELETE'], ['/app/v1/me', 'GET']]) add(`signed-out ${method} ${path}`, { method, url: path });
  add('password sign-in wrong password', { method: 'POST', url: '/signin/password', ...form({ username: 'tester', password: 'wrong-wrong-wrong-1' }) });
  add('password sign-in unknown user', { method: 'POST', url: '/signin/password', ...form({ username: 'ghost', password: PASSWORD }) });
  add('password sign-in no body', { method: 'POST', url: '/signin/password' });
  add('password sign-in ok (no library)', { method: 'POST', url: '/signin/password', ...form({ username: 'nolib', password: PASSWORD }) });
  add('password sign-in ok', { method: 'POST', url: '/signin/password', ...form({ username: 'tester', password: PASSWORD }) });
  add('signed-in GET /signin redirects', { url: '/signin', session: 'user' });
  add('signed-in GET /signin/password redirects', { url: '/signin/password', session: 'user' });
  add('sign-out', { url: '/signout', session: 'user' }, { last: true });
  add('app nonce', { method: 'POST', url: '/app/v1/auth/nonce', ...json({}) }, { masks: ['nonce', 'expiresAt'] });
  add('app google bad token', { method: 'POST', url: '/app/v1/auth/google', ...json({ nonce: 'x', idToken: 'y' }) });
  add('app password wrong', { method: 'POST', url: '/app/v1/auth/password', ...json({ username: 'tester', password: 'wrong-wrong-wrong-1' }) });
  add('app password invalid json', { method: 'POST', url: '/app/v1/auth/password', headers: { 'content-type': 'application/json' }, body: '{nope' });
  add('app password ok', { method: 'POST', url: '/app/v1/auth/password', ...json({ username: 'tester', password: PASSWORD, device: 'diff2' }) }, { masks: ['token', 'accessToken', 'access_token', 'expiresAt', 'expires_at', 'id'] });
  add('app GET nonce', { url: '/app/v1/auth/nonce' });
  add('app me with token', { url: '/app/v1/me', session: 'bearer' });
  add('app me with cookie', { url: '/app/v1/me', session: 'user' });
  add('app me with bad token', { url: '/app/v1/me', session: 'badbearer' });
  add('app signout bad token', { method: 'POST', url: '/app/v1/auth/signout', session: 'badbearer' });
  add('no-library account hits site', { url: '/everything', session: 'nolib' });
  add('no-library account app me', { url: '/app/v1/me', session: 'nolib' });
  add('admin summary right token', { url: '/_admin/auth', admin: 'right' }, { masks: ['created_ms'] });
  add('admin summary wrong token', { url: '/_admin/auth', admin: 'wrong' });
  add('admin summary no token', { url: '/_admin/auth' });
  add('admin summary basic scheme', { url: '/_admin/auth', headers: { authorization: `Basic ${TOKEN}` } });
  add('admin unknown area', { url: '/_admin/nothing', admin: 'right' });
  add('admin bare', { url: '/_admin/', admin: 'right' });
  add('admin bucket totals', { url: '/_admin/bucket', admin: 'right' });
  add('admin bucket prefix', { url: '/_admin/bucket?prefix=downloads%2F', admin: 'right' });
  add('admin bucket prefix none', { url: '/_admin/bucket?prefix=zzz', admin: 'right' });
  add('admin bucket head', { method: 'HEAD', url: '/_admin/bucket/static/a.txt', admin: 'right' });
  add('admin bucket head missing', { method: 'HEAD', url: '/_admin/bucket/static/missing.txt', admin: 'right' });
  add('admin bucket bad key', { method: 'HEAD', url: '/_admin/bucket/a/../b', admin: 'right' });
  add('admin bucket empty segment', { method: 'HEAD', url: '/_admin/bucket/a//b', admin: 'right' });
  add('admin bucket bad encoding', { method: 'HEAD', url: '/_admin/bucket/%E0%A4%A', admin: 'right' });
  add('admin bucket GET key', { url: '/_admin/bucket/static/a.txt', admin: 'right' });
  add('admin bucket DELETE', { method: 'DELETE', url: '/_admin/bucket/static/a.txt', admin: 'right' });
  add('admin bucket put', { method: 'PUT', url: '/_admin/bucket/tmp/put.txt', body: 'payload', headers: { 'x-object-headers': '{"content-type":"text/x-test"}' }, admin: 'right' });
  add('admin bucket head after put', { method: 'HEAD', url: '/_admin/bucket/tmp/put.txt', admin: 'right' });
  add('admin bucket put bad headers json', { method: 'PUT', url: '/_admin/bucket/tmp/bad.txt', body: 'x', headers: { 'x-object-headers': '{nope' }, admin: 'right' });
  add('admin library counts', { method: 'POST', url: `/_admin/library/${LIBRARY}/counts`, body: '{}', admin: 'right' });
  add('admin library unknown step', { method: 'POST', url: `/_admin/library/${LIBRARY}/nope`, body: '{}', admin: 'right' });
  add('admin library bad name', { method: 'POST', url: '/_admin/library/Bad_Name/counts', body: '{}', admin: 'right' });
  add('admin library digest', { method: 'POST', url: `/_admin/library/${LIBRARY}/digest`, body: JSON.stringify({ tables: [{ table: 'cards', columns: ['id'] }] }), admin: 'right' });
  add('admin library digest bad table', { method: 'POST', url: `/_admin/library/${LIBRARY}/digest`, body: JSON.stringify({ tables: [{ table: 'nope', columns: ['id'] }] }), admin: 'right' });
  add('admin library rows bad table', { method: 'POST', url: `/_admin/library/${LIBRARY}/rows`, body: JSON.stringify({ table: 'nope', columns: ['id'], rows: [[1]] }), admin: 'right' });
  add('admin library counts wrong token', { method: 'POST', url: `/_admin/library/${LIBRARY}/counts`, body: '{}', admin: 'wrong' });
  add('admin auth user bad json', { method: 'POST', url: '/_admin/auth/user', body: '{nope', admin: 'right' });
  add('admin auth library missing fields', { method: 'POST', url: '/_admin/auth/library', body: '{}', admin: 'right' });
  add('admin auth summary after', { url: '/_admin/auth', admin: 'right' }, { masks: ['created_ms'] });
  add('download listing signed out', { url: '/download' });
  add('download slash signed out', { url: '/download/' });
  add('download file signed out', { url: '/download/mymind-linux-x86_64.tar.gz' });
  add('download listing', { url: '/download', session: 'user' });
  add('download listing slash', { url: '/download/', session: 'user' });
  add('download listing bearer', { url: '/download', session: 'bearer' });
  add('download file', { url: '/download/mymind-linux-x86_64.tar.gz', session: 'user' });
  add('download file HEAD', { method: 'HEAD', url: '/download/mymind-linux-x86_64.tar.gz', session: 'user' });
  add('download odd name', { url: `/download/${encodeURIComponent('with space & "q".tar.gz')}`, session: 'user' });
  add('download missing', { url: '/download/nope.tar.gz', session: 'user' });
  add('download not in manifest', { url: '/download/manifest.json', session: 'user' });
  add('download bad encoding', { url: '/download/%E0%A4%A', session: 'user' });
  add('download POST', { method: 'POST', url: '/download', session: 'user' });
  add('download POST file', { method: 'POST', url: '/download/x', session: 'user' });
  add('download prefix lookalike', { url: '/downloads', session: 'user' });
  for (const [path, method] of [['/everything', 'GET'], ['/api/cards', 'GET'], ['/app/v1/cards', 'GET'], ['/static/x.css', 'GET'], ['/', 'GET'], ['/everything', 'POST']]) {
    add(`library site ${method} ${path}`, { method, url: path, session: 'user' });
  }
  const fixture = new DatabaseSync(HOSTED_DB, { readOnly: true });
  const cards = fixture.prepare('SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 3').all().map((row) => row.id);
  const spaces = fixture.prepare('SELECT id FROM spaces ORDER BY id LIMIT 2').all().map((row) => row.id);
  fixture.close();
  const asJson = { accept: 'application/json' };
  const reads = ['/cards', '/cards/trash', '/spaces', '/tags/manual', '/tags/recent', '/changes?since=0', '/account.json', '/quota',
    '/search?q=design', '/search/suggestions?q=d', '/tags/suggest?q=a', '/app/v1/me', '/app/v1/cards?limit=5', '/app/v1/spaces', '/objects/nope',
    ...cards.flatMap((id) => [`/cards/${id}`, `/cards/${id}/detail`, `/objects/${id}`, `/objects/${id}/content`, `/objects/${id}/tags`, `/embeddings/related/${id}`, `/app/v1/cards/${id}`]),
    ...spaces.map((id) => `/spaces/${id}`)];
  for (const url of reads) {
    // `elapsed` is how long the search took.
    add(`library read ${url}`, { url, session: 'user', headers: asJson }, { masks: ['elapsed'] });
    add(`library read ${url} (MessagePack)`, { url, session: 'user' }, { masks: ['elapsed'] });
  }
  add('library read /cards bearer', { url: '/cards', session: 'bearer', headers: asJson });
  add('library grid', { method: 'POST', url: '/cards/grid', session: 'user', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cards) });
  return out;
}

async function run(worker, state, item) {
  const { request } = item;
  const headers = { ...request.headers, ...sessionsFor[request.session ?? 'none'](state) };
  if (request.admin) headers.authorization = `Bearer ${request.admin === 'right' ? TOKEN : 'wrong-token'}`;
  const response = await raw(worker, { method: request.method, url: request.url, headers, body: request.body });
  if (process.env.DEBUG_CASE && process.env.DEBUG_CASE === item.name) console.log(worker.name, Object.keys(headers), response.status, [...response.headers], response.bytes.toString().slice(0, 300));
  return normalize(response, item.masks);
}

const diff = (a, b) => {
  const lines = [];
  if (a.status !== b.status) lines.push(`  status js=${a.status} rust=${b.status}`);
  for (const name of new Set([...Object.keys(a.headers), ...Object.keys(b.headers)])) {
    if (JSON.stringify(a.headers[name]) !== JSON.stringify(b.headers[name])) lines.push(`  header ${name}: js=${JSON.stringify(a.headers[name])} rust=${JSON.stringify(b.headers[name])}`);
  }
  if (a.body !== b.body) lines.push(`  body js=${JSON.stringify(a.body.slice(0, 160))} rust=${JSON.stringify(b.body.slice(0, 160))}`);
  return lines;
};

mkdirSync(WORK, { recursive: true });
const jsConfig = jsonc(path.join(REPO, 'worker/wrangler.jsonc'));
const abs = (p) => path.resolve(REPO, 'worker', p);
jsConfig.main = abs(jsConfig.main);
jsConfig.alias = Object.fromEntries(Object.entries(jsConfig.alias).map(([k, v]) => [k, abs(v)]));
const rustConfig = jsonc(path.join(CRATE, 'wrangler.jsonc'));
rustConfig.main = path.resolve(CRATE, rustConfig.main);

const workers = [];
let failures = 0, compared = 0;
try {
  const user = await (async () => {
    const db = installAuthSchema(openSql(':memory:'));
    await setPassword(db, 'tester', PASSWORD);
    const row = db.prepare('SELECT username, salt, hash, created_ms FROM users WHERE username = ?').get('tester');
    db.close();
    return { username: row.username, salt: Buffer.from(row.salt).toString('base64'), hash: Buffer.from(row.hash).toString('base64'), created_ms: row.created_ms };
  })();
  const js = await start('js', jsConfig); workers.push(js);
  const rust = await start('rust', rustConfig); workers.push(rust);
  console.log(`js ${js.base}, rust ${rust.base}`);
  const tables = await Promise.all([seed(js, user), seed(rust, user)]);
  console.log(`seeded both: ${tables[0]} / ${tables[1]} library tables, digests match the fixture`);

  const states = [];
  for (const worker of [js, rust]) {
    const state = { cookie: await signIn(worker, 'tester'), nolibCookie: await signIn(worker, 'nolib') };
    const app = await appToken(worker);
    state.token = app.token;
    if (!state.cookie || !state.nolibCookie || !state.token) throw new Error(`${worker.name} could not sign in (app keys: ${app.keys})`);
    states.push(state);
  }
  for (const item of cases()) {
    if (item.last) continue;
    const [a, b] = [await run(js, states[0], item), await run(rust, states[1], item)];
    compared++;
    if (process.env.VERBOSE) console.log(`${a.status}/${b.status} ${item.name}`);
    const lines = diff(a, b);
    if (lines.length) { failures++; console.log(`DIFF ${item.name}\n${lines.join('\n')}`); }
  }
  for (const item of cases().filter((c) => c.last)) {
    const [a, b] = [await run(js, states[0], item), await run(rust, states[1], item)];
    compared++;
    const lines = diff(a, b);
    if (lines.length) { failures++; console.log(`DIFF ${item.name}\n${lines.join('\n')}`); }
    const after = await Promise.all([run(js, states[0], { request: { url: '/app/v1/me', session: 'user' } }), run(rust, states[1], { request: { url: '/app/v1/me', session: 'user' } })]);
    compared++;
    const more = diff(...after);
    if (more.length) { failures++; console.log(`DIFF after sign-out\n${more.join('\n')}`); }
  }

  const fresh = [await signIn(js, 'tester'), await signIn(rust, 'tester')];
  const big = `${'x'.repeat(1048577)}\u001e`;
  const hubCases = [
    ['hub signed out', (c) => ({ cookie: undefined })],
    ['hub no key', (c) => ({ cookie: c, key: false })],
    ['hub handshake + ping', (c) => ({ cookie: c, messages: [HS, '{"type":6}\u001e', '{"type":6}\u001e'] })],
    ['hub handshake split records', (c) => ({ cookie: c, messages: [HS.slice(0, 10), HS.slice(10)] })],
    ['hub invalid record', (c) => ({ cookie: c, messages: [HS, 'not json\u001e'] })],
    ['hub too large', (c) => ({ cookie: c, messages: [HS, big] })],
  ];
  for (const [name, make] of hubCases) {
    const [a, b] = [await websocket(js, make(fresh[0])), await websocket(rust, make(fresh[1]))];
    compared++;
    if (JSON.stringify(a) !== JSON.stringify(b)) { failures++; console.log(`DIFF ${name}\n  js=${JSON.stringify(a)}\n  rust=${JSON.stringify(b)}`); }
  }
  console.log(`\ncompared ${compared} requests, ${failures} differ`);
  if (failures && process.env.VERBOSE) console.log(`rust log tail:\n${workers[1].log.join('').slice(-3500)}`);
} finally {
  workers.forEach(stop);
}
process.exit(failures ? 1 : 0);
