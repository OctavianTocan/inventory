// Resolve the Rust backend used by developer tools.
import { accessSync, constants } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/**
 * Name the backend selected by developer tools.
 * @returns {'rust'} The Rust backend.
 */
export function serverKind() {
  return 'rust';
}

/**
 * Resolve the executable that serves the replica.
 * @returns {{ command: string, args: string[] }} Absolute executable and its serve subcommand.
 * @throws {Error} If the configured executable is missing or cannot be executed.
 */
export function serverCommand() {
  return toolCommand('serve', []);
}

/**
 * Resolve a Rust operational command, preserving its arguments.
 * @param {string} name Rust subcommand.
 * @param {string[]} args Arguments passed unchanged to the subcommand.
 * @returns {{ command: string, args: string[] }} Absolute executable and subcommand arguments.
 * @throws {Error} If the configured executable is missing or cannot be executed.
 */
export function toolCommand(name, args) {
  const command = path.resolve(ROOT, process.env.MYMIND_SERVER ?? 'rust/target/release/mymind');
  try { accessSync(command, constants.X_OK); }
  catch { throw new Error(`Rust executable is missing or not executable: ${command}. Build it with: cd rust && cargo build --offline --release -p mymind; or set MYMIND_SERVER to a built binary.`); }
  return { command, args: [name, ...args] };
}
