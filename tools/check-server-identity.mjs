// Loopback-only ownership and hosted-backend regression checks; uses synthetic recordings and credentials.
// Usage: MYMIND_SERVER=rust/target/debug/mymind node tools/check-server-identity.mjs
import { strictEqual, notStrictEqual, rejects, throws, ok } from 'node:assert';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { normalizationDigest, replicaDigest, verificationDigest, verificationMode, workerDigest } from './digests.mjs';
import { serverCommand } from './server-command.mjs';
import { assertReplicaServer, assertRustServer } from './server-identity.mjs';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const { command, args } = serverCommand();
const scratch = path.join(ROOT, 'rust/target');
await mkdir(scratch, { recursive: true });
const directory = await mkdtemp(path.join(scratch, 'server-identity-'));
const children = [];
const servers = [];
const environment = { PATH: process.env.PATH, STORE_LOAD_ENV: '0' };

// Bind a synthetic HTTP fixture only on loopback and remember it for unconditional cleanup.
async function listen(handler) {
  const server = http.createServer(handler);
  servers.push(server);
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  return server.address().port;
}

// Reserve a loopback port for a spawned executable without reading any real server configuration.
async function unusedPort() {
  const server = http.createServer();
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return port;
}

// Start one isolated child and wait for a successful response, reporting only synthetic child output on failure.
async function start(executable, arguments_, port, env, native = false) {
  const child = spawn(executable, arguments_, { cwd: ROOT, env: { ...environment, ...env }, stdio: ['ignore', 'pipe', 'pipe'] });
  children.push(child);
  let output = '';
  child.stdout.on('data', chunk => { output += chunk; });
  child.stderr.on('data', chunk => { output += chunk; });
  let error;
  child.once('error', failure => { error = failure; });
  for (let attempt = 0; attempt < 100; attempt++) {
    if (error || child.exitCode !== null) throw new Error(`Synthetic server did not start: ${error?.message ?? output}`);
    const route = native ? '/__replay/reload' : '/__mymind/verification';
    const ready = await fetch(`http://127.0.0.1:${port}${route}`, { method: native ? 'POST' : 'GET', signal: AbortSignal.timeout(500) }).then(response => response.ok, () => false);
    if (ready) return child;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error(`Synthetic server readiness timed out: ${output}`);
}

// Stop the direct child and wait until its listener has been released.
async function stop(child) {
  if (child.exitCode !== null || child.signalCode !== null || child.pid === undefined) return;
  const exited = once(child, 'exit');
  child.kill();
  await exited;
}

try {
  strictEqual(verificationMode(), 'rust');
  throws(() => verificationMode('worker'), /Unknown verification server/);
  throws(() => verificationMode('hosted-js'), /Unknown verification server/);
  const native = await replicaDigest(), worker = await workerDigest();
  const hostedRust = await verificationDigest('hosted-rust'), hostedWorker = await verificationDigest('hosted-worker');
  strictEqual(await verificationDigest(), native);
  notStrictEqual(hostedRust, native);
  notStrictEqual(hostedWorker, worker);
  notStrictEqual(hostedWorker, hostedRust);
  ok(/^[a-f0-9]{64}$/.test(await normalizationDigest()));
  await rejects(() => assertRustServer(0), /valid local TCP port/);

  const requests = [];
  const upstream = await listen(async (req, res) => {
    let body = '';
    for await (const chunk of req) body += chunk;
    requests.push([req.method, req.url, req.headers.cookie]);
    if (req.url === '/signin/password') {
      strictEqual(body, 'username=synthetic-user&password=synthetic-password');
      res.writeHead(302, { 'set-cookie': 'mm_session=synthetic-session; HttpOnly', location: '/' });
      res.end();
    } else {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ proxy: 'mymind-hosted-proxy', server: 'hosted-worker', digest: hostedWorker, synthetic: true }));
    }
  });
  await rejects(() => assertRustServer(upstream), /Cannot verify port/);
  await rejects(() => assertReplicaServer(upstream, 'hosted-worker'), /Cannot verify port/);

  for (const [backend, mode, digest] of [['rust', 'hosted-rust', hostedRust], ['worker', 'hosted-worker', hostedWorker]]) {
    const port = await unusedPort();
    const proxy = await start(process.execPath, ['tools/hosted-proxy.mjs', `http://127.0.0.1:${upstream}`, '--port', String(port), '--backend', backend], port, {
      TEST_USER_NAME: 'synthetic-user', TEST_USER_PASSWORD: 'synthetic-password',
    });
    const identity = await assertReplicaServer(port, mode);
    strictEqual(identity.server, mode);
    strictEqual(identity.digest, digest);
    strictEqual(identity.target, `http://127.0.0.1:${upstream}`);
    await rejects(() => assertReplicaServer(port), /Cannot verify port/);
    await rejects(() => assertReplicaServer(port, backend === 'rust' ? 'hosted-worker' : 'hosted-rust'), /not the current/);
    const before = requests.length;
    const forwarded = await fetch(`http://127.0.0.1:${port}/synthetic`).then(response => response.json());
    strictEqual(forwarded.synthetic, true);
    strictEqual(requests.length, before + 1);
    strictEqual(requests.at(-1)[2], 'mm_session=synthetic-session');
    await stop(proxy);
  }

  await mkdir(path.join(directory, 'capture'));
  await mkdir(path.join(directory, 'replay'));
  for (const [name, content] of [['meta.json', '{}'], ['cards.json', '[]'], ['responses.json', '{}'], ['details.json', '{}'], ['shell.html', '<!doctype html><p>Synthetic fixture</p>']]) {
    await writeFile(path.join(directory, 'capture', name), content);
  }
  await writeFile(path.join(directory, 'replay/index.json'), '{}');
  const port = await unusedPort();
  const backend = await start(command, args, port, { MYMIND_ROOT: directory, PORT: String(port), OFFLINE: '1', STORE_READONLY: '1', STORE_DB: '', AUTH: '' }, true);
  await assertReplicaServer(port);
  await rejects(() => assertReplicaServer(port, 'hosted-rust'), /Cannot verify port/);
  const status = await fetch(`http://127.0.0.1:${port}/__replay/reload`, { method: 'POST' }).then(response => response.json());
  strictEqual(status.offline, true);
  await stop(backend);
  await rejects(() => assertReplicaServer(port), /Cannot verify port/);
  console.log('PASS native listener ownership, proxy identity, explicit Rust/Worker modes, forwarding and backend digests (synthetic loopback only)');
} finally {
  await Promise.all(children.map(stop));
  for (const server of servers) {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
  await rm(directory, { recursive: true, force: true });
}
