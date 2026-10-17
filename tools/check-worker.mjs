// Run the Rust Worker corpus in a fresh local runtime with external networking disabled.
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CRATE = path.join(REPO, 'rust/crates/worker');
const args = process.argv.slice(2);
if (args.includes('--help')) {
  console.log('node tools/check-worker.mjs [--container-smoke]\nRequires an existing Rust Worker build and pinned Wrangler. Runs only on isolated loopback.\n--container-smoke requires a prestarted local Worker with real containers at MYMIND_WORKER_CONTAINER_URL.');
  process.exit(0);
}
if (args.some((arg) => !['--container-smoke', '--isolated', '--record-js'].includes(arg))) throw new Error('Unknown Worker check argument');
if (args.includes('--record-js') && process.env.MYMIND_RECORD_GOLDEN !== '1') throw new Error('--record-js requires MYMIND_RECORD_GOLDEN=1');
const scratch = path.join(REPO, 'rust/target/worker-check');
mkdirSync(scratch, { recursive: true });
// A user/network namespace has no external interfaces; ioctl enables only its private loopback.
if (!args.includes('--isolated') && !args.includes('--container-smoke')) {
  const init = 'import os,socket,fcntl,struct,sys; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); fcntl.ioctl(s,0x8914,struct.pack("16sH14x",b"lo",0x49)); s.close(); os.environ["MYMIND_WORKER_NETWORK_ISOLATED"]="1"; os.execv(sys.argv[1],sys.argv[1:])';
  const child = spawn('unshare', ['-Urn', 'python3', '-c', init, process.execPath, fileURLToPath(import.meta.url), ...args, '--isolated'], { stdio: 'inherit', env: Object.fromEntries(Object.entries(process.env).filter(([name]) => !/proxy/i.test(name)).concat([['NODE_USE_ENV_PROXY', '0']])) });
  child.on('error', (error) => { console.error(`Cannot create offline Worker environment: ${error.message}`); process.exitCode = 1; });
  child.on('exit', (code) => { process.exitCode = code ?? 1; });
} else {
  await check();
}

// Allocate a private loopback port before starting a local process.
async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => server.once('error', reject).listen(0, '127.0.0.1', resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

// Forward exit status while retaining only bounded runtime diagnostics.
async function run(command, argv, options) {
  const child = spawn(command, argv, options);
  return new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code) => resolve(code ?? 1));
  });
}

// Stage synthetic local bindings and invoke the native Rust integration client.
async function check() {
  const container = args.includes('--container-smoke');
  if (!container && process.env.MYMIND_WORKER_NETWORK_ISOLATED !== '1') throw new Error('The local runtime requires the loopback-only network namespace');
  const record = args.includes('--record-js');
  if (container) {
    const url = process.env.MYMIND_WORKER_CONTAINER_URL;
    localUrl(url, 'MYMIND_WORKER_CONTAINER_URL');
    process.exitCode = await cargo('worker_real_container_smoke', { MYMIND_WORKER_CONTAINER_URL: url, MYMIND_WORKER_ADMIN_TOKEN: 'worker-oracle-local-admin-only' });
    return;
  }
  const wrangler = process.env.MYMIND_WRANGLER ?? path.join(CRATE, 'node_modules/wrangler/bin/wrangler.js');
  if (!existsSync(wrangler)) throw new Error('Pinned Wrangler is missing; install the Worker package first or set MYMIND_WRANGLER to its local executable');
  const configRoot = record ? path.join(REPO, 'worker') : CRATE;
  const config = JSON.parse(readFileSync(path.join(configRoot, 'wrangler.jsonc'), 'utf8').replace(/^\s*\/\/.*$/gm, ''));
  if (!existsSync(path.resolve(configRoot, config.main))) throw new Error('Rust Worker build is missing; run worker-build --release offline before this check');
  const work = mkdtempSync(path.join(scratch, record ? 'js-' : 'rust-'));
  const port = await freePort();
  const token = 'worker-oracle-local-admin-only';
  const { containers, routes, build, ...local } = config;
  local.main = path.resolve(configRoot, config.main);
  if (local.alias) local.alias = Object.fromEntries(Object.entries(local.alias).map(([name, file]) => [name, path.resolve(configRoot, file)]));
  local.name = 'mymind-offline-oracle';
  local.workers_dev = false;
  local.preview_urls = false;
  local.vars = { AUTH: 'google', ADMIN_TOKEN: token, GOOGLE_CLIENT_ID: 'offline-client', GOOGLE_CLIENT_SECRET: 'offline-secret', OWNER_EMAIL: 'owner@example.test', PUBLIC_URL: 'http://127.0.0.1' };
  local.r2_buckets = local.r2_buckets?.map(({ binding, bucket_name }) => ({ binding, bucket_name, remote: false }));
  const file = path.join(work, 'wrangler.json');
  writeFileSync(file, `${JSON.stringify(local, null, 2)}\n`);
  const envFile = path.join(work, 'fixture-vars');
  writeFileSync(envFile, '');
  const env = { PATH: process.env.PATH, HOME: work, TMPDIR: work, XDG_CONFIG_HOME: path.join(work, 'xdg'), WRANGLER_LOG_PATH: path.join(work, 'wrangler.log'), WRANGLER_SEND_METRICS: 'false', CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: 'false', CLOUDFLARE_INCLUDE_PROCESS_ENV: 'false', CI: '1', NO_PROXY: '127.0.0.1,localhost', no_proxy: '127.0.0.1,localhost' };
  const child = spawn(process.execPath, [wrangler, 'dev', '--local', '--config', file, '--env-file', envFile, '--port', String(port), '--ip', '127.0.0.1', '--persist-to', path.join(work, 'state'), '--inspector-port', String(await freePort())], { cwd: work, env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  let log = '', errors = '';
  const diagnostic = (line) => /panicked|RuntimeError|unreachable|CPU|memory|ERROR|Error|exception/.test(line) && !/token|secret|cookie|authorization|state=|nonce=|code_challenge=/i.test(line);
  const collect = (chunk) => { log = (log + chunk).slice(-4000); errors = (errors + String(chunk).split('\n').filter(diagnostic).join('\n')).slice(-8000); };
  child.stdout.on('data', collect);
  child.stderr.on('data', collect);
  const url = `http://127.0.0.1:${port}`;
  try {
    const until = Date.now() + 90000;
    for (;;) {
      if (child.exitCode !== null || Date.now() > until) throw new Error(`Local Wrangler could not start in the offline namespace:\n${log}`);
      try { await fetch(`${url}/robots.txt`, { redirect: 'manual', signal: AbortSignal.timeout(1500) }); break; } catch { await new Promise((resolve) => setTimeout(resolve, 200)); }
    }
    process.exitCode = await cargo('worker_matches_frozen_js', { MYMIND_WORKER_URL: url, MYMIND_WORKER_ADMIN_TOKEN: token, ...(record ? { MYMIND_RECORD_GOLDEN: '1', MYMIND_WORKER_ORACLE: 'js' } : {}) });
    if (!process.exitCode && !record) process.exitCode = await cargo('worker_credential_contracts', { MYMIND_WORKER_URL: url, MYMIND_WORKER_ADMIN_TOKEN: token }, 'worker_credentials');
    if (process.exitCode) console.error(errors);
  } finally {
    try { process.kill(-child.pid, 'SIGTERM'); } catch {}
    rmSync(path.join(work, 'wrangler.log'), { force: true });
  }
}

// Require an explicit loopback HTTP endpoint for a prestarted local integration environment.
function localUrl(value, name) {
  if (!value) throw new Error(`${name} is required`);
  const url = new URL(value);
  if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || url.username || url.password || url.pathname !== '/' || url.search || url.hash) throw new Error(`${name} must be a plain http://127.0.0.1:<port> URL`);
}

// Run only the selected ignored native test; Cargo may use cached dependencies only.
async function cargo(test, vars, target = 'worker') {
  const binary = target === 'worker_credentials' ? process.env.MYMIND_WORKER_CREDENTIALS_TEST_BINARY : process.env.MYMIND_WORKER_TEST_BINARY;
  if (binary) {
    return run('nice', ['-n', '19', path.resolve(binary), test, '--ignored', '--exact', '--test-threads=2', '--nocapture'], { cwd: path.join(REPO, 'rust'), stdio: 'inherit', env: { PATH: process.env.PATH, TMPDIR: scratch, MYMIND_TEST_HOSTED_DB: path.join(REPO, 'rust/test-fixtures/hosted-test.db'), ...vars } });
  }
  return run('nice', ['-n', '19', 'cargo', 'test', '--offline', '--locked', '-j', '2', '-p', 'mymind', '--test', target, test, '--', '--ignored', '--exact', '--test-threads=2', '--nocapture'], {
    cwd: path.join(REPO, 'rust'), stdio: 'inherit',
    env: { PATH: process.env.PATH, HOME: process.env.HOME, CARGO_HOME: process.env.CARGO_HOME ?? path.join(process.env.HOME, '.cargo'), RUSTUP_HOME: process.env.RUSTUP_HOME ?? path.join(process.env.HOME, '.rustup'), TMPDIR: scratch, ...vars },
  });
}
