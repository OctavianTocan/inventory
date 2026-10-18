// Verify the executable that owns a local listener before assigning it a Rust source digest.
import { readFile, readdir, readlink, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { serverCommand } from './server-command.mjs';
import { verificationDigest, verificationMode } from './digests.mjs';

const PROXY = fileURLToPath(new URL('./hosted-proxy.mjs', import.meta.url));

// Collect every listening socket inode at the requested TCP port.
async function listeners(port) {
  const inodes = new Set();
  for (const file of ['/proc/net/tcp', '/proc/net/tcp6']) {
    const text = await readFile(file, 'utf8').catch(error => {
      if (file.endsWith('tcp6') && error.code === 'ENOENT') return '';
      throw error;
    });
    for (const line of text.trim().split('\n').slice(1)) {
      const fields = line.trim().split(/\s+/);
      if (fields[3] === '0A' && Number.parseInt(fields[1].split(':')[1], 16) === port) inodes.add(fields[9]);
    }
  }
  return inodes;
}

/**
 * Require every listener at a local port to belong to the selected, current Rust executable.
 * @param {number} port TCP port of the already responding local replica.
 * @returns {Promise<void>} Resolves only after socket ownership and executable device/inode match.
 * @throws {Error} If ownership is inaccessible, no listener exists, or a different executable owns the port.
 */
export async function assertRustServer(port) {
  const { command } = serverCommand();
  await assertListener(port, command);
}

// Require socket ownership by one executable, optionally checking each owning process's command line.
async function assertListener(port, command, checkOwner = async () => true) {
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Replica identity requires a valid local TCP port');
  const expected = await stat(command, { bigint: true });
  const failure = `Cannot verify port ${port} as the selected executable ${command}. Stop the existing listener and start the selected verification server.`;
  try {
    const pending = await listeners(port);
    if (!pending.size) throw new Error(failure);
    for (const pid of (await readdir('/proc')).filter(name => /^\d+$/.test(name))) {
      const files = await readdir(`/proc/${pid}/fd`).catch(() => []);
      for (const fd of files) {
        const target = await readlink(`/proc/${pid}/fd/${fd}`).catch(() => '');
        const inode = /^socket:\[(\d+)\]$/.exec(target)?.[1];
        if (!pending.has(inode)) continue;
        const owner = await stat(`/proc/${pid}/exe`, { bigint: true }).catch(() => null);
        if (!owner || owner.dev !== expected.dev || owner.ino !== expected.ino || !await checkOwner(pid)) throw new Error(failure);
        pending.delete(inode);
      }
      if (!pending.size) return;
    }
  } catch {
    throw new Error(failure);
  }
  throw new Error(failure);
}

// Check the Node entry point rather than trusting a lookalike proxy identity response from another listener.
async function isHostedProxy(pid) {
  const [command, cwd] = await Promise.all([readFile(`/proc/${pid}/cmdline`, 'utf8'), readlink(`/proc/${pid}/cwd`)]);
  const script = command.split('\0')[1];
  return Boolean(script) && path.resolve(cwd, script) === PROXY;
}

/**
 * Identify the selected native listener or this checkout's explicitly declared signed-in proxy.
 * @param {number} port Local TCP listener port.
 * @param {string} mode Verification server mode; hosted modes never fall back to a native listener.
 * @returns {Promise<object|null>} Proxy startup identity, or null for the native Rust listener.
 * @throws {Error} If listener ownership, declared backend or its startup source digest does not match.
 */
export async function assertReplicaServer(port, mode = 'rust') {
  verificationMode(mode);
  if (mode === 'rust') { await assertRustServer(port); return null; }
  await assertListener(port, process.execPath, isHostedProxy);
  const response = await fetch(`http://127.0.0.1:${port}/__mymind/verification`, { signal: AbortSignal.timeout(5000), redirect: 'error' });
  const identity = response.ok ? await response.json() : null;
  if (identity?.proxy !== 'mymind-hosted-proxy' || identity.server !== mode || identity.digest !== await verificationDigest(mode)) {
    throw new Error(`Port ${port} is not the current ${mode} proxy. Start tools/hosted-proxy.mjs with --backend ${mode.slice('hosted-'.length)} and the current sources.`);
  }
  return identity;
}
