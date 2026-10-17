// The backend the Node tools start: `node server.mjs`, or the Rust `mymind` binary that `MYMIND_SERVER` names.
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SCRIPTS = { serve: 'server.mjs', 'download-server': 'deploy/download-server.mjs' };

/**
 * Which backend the tools start and verify.
 * @returns {'node' | 'rust'} `rust` when `MYMIND_SERVER` is set.
 */
export function serverKind() {
  return process.env.MYMIND_SERVER ? 'rust' : 'node';
}

/**
 * The command that starts the replica server, run from the repository root.
 * @returns {{ command: string, args: string[] }} The Rust binary (`MYMIND_SERVER`, a relative path read from the
 *   repository root) with `serve`, or Node with `server.mjs`.
 */
export function serverCommand() {
  return toolCommand('serve', []);
}

/**
 * The command that runs one of the programs the Rust binary took over (`download-server`, `seed-db`, …).
 * @param {string} name - The subcommand in Rust; in Node `tools/<name>.mjs`, or `server.mjs` for `serve` and
 *   `deploy/download-server.mjs` for `download-server`.
 * @param {string[]} args - The tool's arguments.
 * @returns {{ command: string, args: string[] }} The command to spawn from the repository root.
 */
export function toolCommand(name, args) {
  if (serverKind() === 'rust') return { command: path.resolve(ROOT, process.env.MYMIND_SERVER), args: [name, ...args] };
  return { command: process.execPath, args: [SCRIPTS[name] ?? `tools/${name}.mjs`, ...args] };
}
