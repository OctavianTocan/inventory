import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { EventEmitter } from 'node:events';
import http from 'node:http';
import { unwatchFile } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { DatabaseSync, backup } from 'node:sqlite';
import { Readable } from 'node:stream';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { serverCommand, serverKind } from './server-command.mjs';

export const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
/** Database seeded from the recordings that verification servers and checks read; never the working library. */
export const VERIFY_DB = join(ROOT, 'data/verify.db');
export const msgpack = await import(pathToFileURL(join(ROOT, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);

/**
 * Localize recorded URL strings exactly as server.mjs does, preserving codec values.
 * @param {unknown} value Decoded recording, JSON string or plain response bytes as a string.
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

// Run the real server request callback without sockets when the caller explicitly selects it.
async function memoryTransport(dbPath, readonly) {
  const originalCreate = http.createServer;
  const previous = Object.fromEntries(['PORT', 'OFFLINE', 'STORE_DB', 'STORE_READONLY', 'STORE_LOAD_ENV'].map(key => [key, process.env[key]]));
  let server;
  http.createServer = (...args) => {
    server = originalCreate(...args);
    server.listen = () => server;
    return server;
  };
  Object.assign(process.env, { PORT: '0', OFFLINE: '1', STORE_DB: dbPath, STORE_READONLY: readonly ? '1' : '0', STORE_LOAD_ENV: '0' });
  try { await import(`../server.mjs?http-check=${encodeURIComponent(dbPath)}`); }
  finally {
    http.createServer = originalCreate;
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[key]; else process.env[key] = value;
    }
  }
  return {
    async fetch(url, options) {
      const req = Readable.from(options.body === undefined ? [] : [Buffer.from(options.body)]);
      Object.assign(req, { url: new URL(url).pathname + new URL(url).search, method: options.method,
        headers: { host: '127.0.0.1:0', ...Object.fromEntries(Object.entries(options.headers).map(([k, v]) => [k.toLowerCase(), v])) } });
      return new Promise(resolveResponse => {
        const res = {
          headersSent: false,
          writeHead(status, headers) { this.status = status; this.headers = headers; this.headersSent = true; },
          end(body) { resolveResponse(new Response(body || null, { status: this.status, headers: this.headers })); },
        };
        server.emit('request', req, res);
      });
    },
    hub() {
      const messages = [];
      const socket = new EventEmitter();
      socket.destroyed = false;
      socket.write = bytes => {
        if (bytes.toString().startsWith('HTTP/1.1')) return;
        let length = bytes[1] & 127;
        let offset = 2;
        if (length === 126) { length = bytes.readUInt16BE(2); offset = 4; }
        if (length === 127) { length = Number(bytes.readBigUInt64BE(2)); offset = 10; }
        for (const text of bytes.subarray(offset, offset + length).toString().split('\x1e').filter(Boolean)) messages.push(JSON.parse(text));
      };
      socket.end = socket.destroy = () => { socket.destroyed = true; socket.emit('close'); };
      server.emit('upgrade', { url: '/hub', headers: { 'sec-websocket-key': 'bG9jYWwtdGVzdC1zb2NrZXQ=' } }, socket, Buffer.alloc(0));
      const payload = Buffer.from('{"protocol":"json","version":1}\x1e');
      socket.emit('data', Buffer.concat([Buffer.from([0x81, 0x80 | payload.length, 0, 0, 0, 0]), payload]));
      return { messages, close: socket.end };
    },
    close() { unwatchFile(join(ROOT, 'replay/index.json')); server.emit('close'); },
  };
}

/**
 * Copy the seeded database with SQLite backup and start an isolated, offline replica.
 * Never binds port 4321; PORT=0 lets the OS allocate a free loopback port.
 * @param {object} [options] Test server configuration.
 * @param {boolean} [options.readonly=false] Start in verification mode.
 * @param {string} [options.source] Seeded database to copy; defaults to `$TEST_SOURCE_DB`, then data/verify.db.
 * @param {boolean} [options.noStore=false] Exercise the legacy path with a nonexistent database.
 * @param {Record<string,string>} [options.env={}] Extra server environment (spawned servers only).
 * @returns {Promise<object>} Base URL, database path, temporary directory, request helpers and async cleanup.
 */
export async function testServer({ readonly = false, source = process.env.TEST_SOURCE_DB ?? VERIFY_DB, noStore = false, env = {} } = {}) {
  const dir = await mkdtemp(join(tmpdir(), 'mymind-http-'));
  const dbPath = join(dir, 'test.db');
  let child;
  let memory;
  let startupError = '';
  try {
    const db = new DatabaseSync(source, { readOnly: true });
    try { await backup(db, dbPath); } finally { db.close(); }
    const serverDb = noStore ? join(dir, 'missing.db') : dbPath;
    // The in-process transport imports server.mjs, so it exists only for the Node backend.
    if (process.argv.includes('--in-process') && serverKind() === 'node') memory = await memoryTransport(serverDb, readonly);
    if (!memory) {
      const { command, args } = serverCommand();
      child = spawn(command, args, { cwd: ROOT,
        env: { ...process.env, PORT: '0', OFFLINE: '1', STORE_DB: serverDb, STORE_READONLY: readonly ? '1' : '0', STORE_LOAD_ENV: '0', ...env },
        stdio: ['ignore', 'pipe', 'pipe'] });
      child.stderr.on('data', chunk => { startupError += chunk; });
    }
    const base = memory ? 'http://127.0.0.1:0' : await new Promise((resolveReady, reject) => {
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
        const response = await (memory?.fetch ?? fetch)(base + path, { method, signal: AbortSignal.timeout(10000), headers: { accept, 'content-type': type,
          'x-authenticity-token': 'local-test-token', ...headers },
          ...(body !== undefined ? { body: typeof body === 'string' || body instanceof Uint8Array ? body : JSON.stringify(body) } : {}) });
        const bytes = Buffer.from(await response.arrayBuffer());
        const contentType = response.headers.get('content-type') ?? '';
        const value = contentType.includes('msgpack') ? [...msgpack.decodeMulti(bytes, { useBigInt64: true })]
          : contentType.includes('json') && bytes.length ? JSON.parse(bytes) : bytes.toString('utf8');
        return { response, bytes, value };
      },
      async hub() {
        if (memory) return memory.hub();
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
        memory?.close();
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
