import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { DatabaseSync, backup } from 'node:sqlite';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { serverCommand, toolCommand } from './server-command.mjs';

export const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
/** Synthetic verification database seeded by `mymind seed-db`; never the working library. */
export const VERIFY_DB = join(ROOT, 'rust/target/stage-b/tmp/synthetic-verify.db');
export const msgpack = await import(pathToFileURL(join(ROOT, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);

/**
 * Localize URL strings as the Rust server does, preserving codec values.
 * @param {unknown} value Decoded value, JSON string or plain response bytes as a string.
 * @returns {unknown} Copy with remote origins replaced by local paths.
 */
export function localize(value) {
  if (typeof value === 'string') return value
    .replaceAll('https://static.accelerator.net/', '/_static/')
    .replaceAll('https://media.mymind.host/', '/_mediahost/')
    .replaceAll('https://carbon-media.accelerator.net/', '/_carbon/')
    .replaceAll('https://access.mymind.com/', '/');
  if (Array.isArray(value)) return value.map(localize);
  if (value && typeof value === 'object' && !(value instanceof Date) && !(value instanceof Uint8Array)) {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, localize(item)]));
  }
  return value;
}

/**
 * Copy the seeded database with SQLite backup and start an isolated, offline server.
 * Never binds port 4321; PORT=0 lets the OS allocate a free loopback port.
 * @param {object} [options] Test server configuration.
 * @param {boolean} [options.readonly=false] Start in verification mode.
 * @param {string} [options.source] Seeded database to copy; defaults to `$TEST_SOURCE_DB`, then synthetic VERIFY_DB.
 * @param {boolean} [options.noStore=false] Exercise the path with a nonexistent database.
 * @param {Record<string,string>} [options.env={}] Extra server environment (spawned servers only).
 * @returns {Promise<object>} Base URL, database path, temporary directory, request helpers and async cleanup.
 */
export async function testServer({ readonly = false, source = process.env.TEST_SOURCE_DB ?? VERIFY_DB, noStore = false, env = {} } = {}) {
  if (process.argv.includes('--in-process')) throw new Error('--in-process is unavailable for Rust. Use: cd rust && cargo test --offline -p mymind --test check_writes -- --test-threads=2');
  const temporary = join(ROOT, 'rust/target/stage-b/tmp');
  await mkdir(temporary, { recursive: true });
  if (source === VERIFY_DB && !existsSync(VERIFY_DB)) {
    const seed = toolCommand('seed-db', ['--db', VERIFY_DB]);
    const seeded = spawnSync(seed.command, seed.args, { cwd: ROOT, encoding: 'utf8' });
    if (seeded.status !== 0) throw new Error(`seed-db failed: ${seeded.stderr || seeded.stdout}`);
  }
  const dir = await mkdtemp(join(temporary, 'mymind-http-'));
  const dbPath = join(dir, 'test.db');
  let child;
  let startupError = '';
  try {
    const db = new DatabaseSync(source, { readOnly: true });
    try { await backup(db, dbPath); } finally { db.close(); }
    const serverDb = noStore ? join(dir, 'missing.db') : dbPath;
    const defaultMedia = source === VERIFY_DB ? join(temporary, 'media') : undefined;
    const { command, args } = serverCommand();
    child = spawn(command, args, { cwd: ROOT,
        env: {
          ...process.env,
          PORT: '0',
          OFFLINE: '1',
          STORE_DB: serverDb,
          STORE_READONLY: readonly ? '1' : '0',
          STORE_LOAD_ENV: '0',
          MYMIND_ENV_FILE: join(dir, 'absent-env'),
          ...(defaultMedia ? { STORE_MEDIA: defaultMedia } : {}),
          ...env,
        },
        stdio: ['ignore', 'pipe', 'pipe'] });
    child.stderr.on('data', chunk => { startupError += chunk; });
    const base = await new Promise((resolveReady, reject) => {
      const timer = setTimeout(() => reject(new Error('Test server did not start within 10 seconds')), 10000);
      let output = '';
      child.stdout.on('data', chunk => {
        output += chunk;
        const match = output.match(/http:\/\/localhost:(\d+)\/everything/);
        if (match) { clearTimeout(timer); resolveReady(`http://127.0.0.1:${match[1]}`); }
      });
      child.once('error', error => { clearTimeout(timer); reject(error); });
      child.once('exit', code => { clearTimeout(timer); reject(new Error(`Test server exited before ready (${code}): ${startupError}`)); });
    });
    const errors = [];
    child?.stderr.on('data', chunk => { if (!chunk.toString().includes('ExperimentalWarning')) errors.push(chunk.toString()); });
    return {
      base, dbPath, dir,
      async request(path, { method = 'GET', body, accept = 'application/json', type = 'application/json', headers = {} } = {}) {
        const response = await fetch(base + path, { method, signal: AbortSignal.timeout(10000), headers: { accept, 'content-type': type,
          'x-authenticity-token': 'local-test-token', ...headers },
          ...(body !== undefined ? { body: typeof body === 'string' || body instanceof Uint8Array ? body : JSON.stringify(body) } : {}) });
        const bytes = Buffer.from(await response.arrayBuffer());
        const contentType = response.headers.get('content-type') ?? '';
        const value = contentType.includes('msgpack') ? [...msgpack.decodeMulti(bytes, { useBigInt64: true })]
          : contentType.includes('json') && bytes.length ? JSON.parse(bytes) : bytes.toString('utf8');
        return { response, bytes, value };
      },
      async hub() {
        const socket = new WebSocket(base.replace('http:', 'ws:') + '/hub');
        const messages = [];
        try {
          await new Promise((ready, reject) => {
            let connected = false;
            const timer = setTimeout(() => reject(new Error('Hub handshake timed out after 10 seconds')), 10000);
            const fail = () => { clearTimeout(timer); if (!connected) reject(new Error('Hub closed before completing its handshake')); };
            socket.addEventListener('open', () => socket.send('{"protocol":"json","version":1}\x1e'));
            socket.addEventListener('error', fail, { once: true });
            socket.addEventListener('close', fail, { once: true });
            socket.addEventListener('message', event => {
              try {
                for (const text of String(event.data).split('\x1e').filter(Boolean)) {
                  const message = JSON.parse(text);
                  messages.push(message);
                  if (!message.type) { clearTimeout(timer); connected = true; ready(); }
                }
              } catch { clearTimeout(timer); reject(new Error('Invalid hub JSON response')); }
            });
          });
        } catch (error) { socket.close(); throw error; }
        return { messages, close: () => socket.close() };
      },
      async close() {
        if (child && child.exitCode === null && child.signalCode === null) {
          const exited = once(child, 'exit');
          child.kill();
          await exited;
        }
        await rm(dir, { recursive: true, force: true });
        if (errors.some(error => error.includes('[error]'))) throw new Error('Test server logged an unhandled request error');
      },
    };
  } catch (error) {
    child?.kill();
    await rm(dir, { recursive: true, force: true });
    throw error;
  }
}
