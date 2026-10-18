import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:http';
import { cpSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
export const repo = resolve(here, '..', '..');
export const scratch = join(repo, 'evidence', 'extension', 'scratch');
export const extensionDir = join(repo, 'extension');
export const USER = 'exttest';
export const PASSWORD = 'scratch-password-for-extension-tests-1';

const binary = join(repo, 'rust', 'target', 'release', 'mymind');

/**
 * Creates the scratch library and the scratch password user when they are missing.
 * @throws {Error} When the replica binary is not built or a tool fails.
 */
export function ensureScratch() {
  if (!existsSync(binary)) throw new Error('build the replica first: cd rust && cargo build --release -p mymind');
  mkdirSync(scratch, { recursive: true });
  const env = { PATH: process.env.PATH, HOME: process.env.HOME, MYMIND_ROOT: repo, STORE_LOAD_ENV: '0' };
  const base = join(scratch, 'lib.base.db');
  if (!existsSync(base)) {
    const seeded = spawnSync(binary, ['seed-db', '--db', base], { cwd: repo, env, encoding: 'utf8' });
    if (seeded.status !== 0) throw new Error(`seed-db failed: ${seeded.stderr || seeded.stdout}`);
  }
  const auth = join(scratch, 'auth.db');
  if (!existsSync(auth)) {
    const added = spawnSync(binary, ['auth-user', 'add', USER, '--db', auth], { cwd: repo, env, input: `${PASSWORD}\n`, encoding: 'utf8' });
    if (added.status !== 0) throw new Error(`auth-user failed: ${added.stderr || added.stdout}`);
  }
}

/**
 * Copies the extension to the scratch folder with the loopback hosts granted in the manifest.
 * Optional host permissions need a user prompt that a headless browser cannot answer; the shipped manifest keeps them optional.
 * @returns {string} The folder to load as the unpacked extension.
 * @throws {Error} When the extension vendor script is missing.
 */
export function prepareExtension() {
  if (!existsSync(join(extensionDir, 'vendor', 'tiptap.min.js'))) {
    throw new Error('extension vendor script is missing');
  }
  const target = join(scratch, 'ext-under-test');
  rmSync(target, { recursive: true, force: true });
  cpSync(extensionDir, target, { recursive: true, filter: (source) => !source.includes(`${extensionDir}/tests`) });
  const manifestPath = join(target, 'manifest.json');
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  manifest.host_permissions.push('http://localhost/*', 'http://127.0.0.1/*');
  writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
  return target;
}

/**
 * Starts the replica binary against the scratch library and scratch auth database.
 * @param {number} port - Port to listen on.
 * @param {string} [library] - Library database file; defaults to the scratch library.
 * @returns {Promise<{origin: string, stop: () => void}>} The server once it answers.
 */
export async function startReplica(port, library = join(scratch, 'lib.db')) {
  const env = {
    PATH: process.env.PATH,
    HOME: process.env.HOME,
    STORE_LOAD_ENV: '0',
    MYMIND_ROOT: repo,
    PORT: String(port),
    STORE_DB: library,
    AUTH_DB: join(scratch, 'auth.db'),
    AUTH: 'google',
    GOOGLE_CLIENT_ID: 'dummy',
    GOOGLE_CLIENT_SECRET: 'dummy',
    OWNER_EMAIL: 'owner@example.invalid',
    OFFLINE: '1',
  };
  const child = spawn('nice', ['-n', '19', join(repo, 'rust/target/release/mymind')], { env, stdio: 'ignore' });
  const origin = `http://localhost:${port}`;
  for (let attempt = 0; attempt < 60; attempt++) {
    try {
      if ((await fetch(`${origin}/signin`)).ok) return { origin, stop: () => child.kill() };
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  child.kill();
  throw new Error('replica did not start');
}

/**
 * Signs the scratch user in over HTTP.
 * @param {string} origin - Replica origin.
 * @returns {Promise<string>} The `mm_session` cookie value.
 */
export async function signIn(origin) {
  const response = await fetch(`${origin}/signin/password`, {
    method: 'POST',
    redirect: 'manual',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ username: USER, password: PASSWORD }),
  });
  const cookie = response.headers.get('set-cookie')?.match(/mm_session=([^;]+)/);
  if (!cookie) throw new Error(`sign-in failed: ${response.status}`);
  return cookie[1];
}

/**
 * Serves `extension/tests/pages` for the save-flow pages.
 * @param {number} port - Port to listen on.
 * @returns {Promise<{origin: string, stop: () => void}>} The running server.
 */
export async function startPages(port) {
  const dir = join(extensionDir, 'tests', 'pages');
  const types = { '.html': 'text/html', '.png': 'image/png' };
  const server = createServer((request, response) => {
    const name = request.url.split('?')[0].replace(/^\/+/, '') || 'article.html';
    try {
      const body = readFileSync(join(dir, name));
      response.writeHead(200, { 'content-type': types[name.slice(name.lastIndexOf('.'))] ?? 'application/octet-stream' });
      response.end(body);
    } catch {
      response.writeHead(404).end();
    }
  });
  await new Promise((resolve) => server.listen(port, '127.0.0.1', resolve));
  return { origin: `http://localhost:${port}`, stop: () => server.close() };
}

/**
 * Gives a run its own copy of the seeded library so runs never see each other's cards.
 * @param {string} name - Run name, used as the folder under the scratch directory.
 * @returns {string} Path of the fresh library database.
 */
export function freshLibrary(name) {
  const dir = join(scratch, `run-${name}`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  ensureScratch();
  copyFileSync(join(scratch, 'lib.base.db'), join(dir, 'lib.db'));
  return join(dir, 'lib.db');
}

/**
 * Starts a stand-in that answers every write with a fixed failure.
 * @param {number} port - Port to listen on.
 * @param {{status: number, body: object}} reply - Status and problem+json body for `POST /objects`.
 * @returns {Promise<{stop: () => void}>} The running stub.
 */
export async function startStub(port, reply) {
  const server = createServer((request, response) => {
    if (request.method === 'POST' && request.url === '/objects') {
      response.writeHead(reply.status, { 'content-type': 'application/problem+json' });
      response.end(JSON.stringify(reply.body));
    } else {
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end('[]');
    }
  });
  await new Promise((resolve) => server.listen(port, '127.0.0.1', resolve));
  return { stop: () => server.close() };
}
