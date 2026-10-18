// Scratch auth databases are initialized and password users are managed by the Rust command.
import { spawn } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { fileURLToPath } from 'node:url';
import { toolCommand } from '../server-command.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');

// Run auth-user without loading the checkout's environment file or logging stdin.
async function command(databasePath, args, input = '') {
  const selected = toolCommand('auth-user', [...args, '--db', databasePath]);
  const child = spawn(selected.command, selected.args, {
    cwd: ROOT,
    env: { PATH: process.env.PATH, MYMIND_ENV_FILE: join(dirname(databasePath), 'absent.env'), MYMIND_ROOT: ROOT },
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  let output = '';
  let errors = '';
  child.stdout.setEncoding('utf8').on('data', chunk => { output += chunk; });
  child.stderr.setEncoding('utf8').on('data', chunk => { errors += chunk; });
  child.stdin.on('error', () => {});
  const exited = new Promise((done, fail) => {
    child.once('error', fail);
    child.once('close', code => {
      if (code === 0) done(output);
      else fail(new Error(`Rust auth-user ${args[0]} failed (${code}): ${errors.trim()}`));
    });
  });
  child.stdin.end(input);
  return exited;
}

/**
 * Create the Rust auth schema and open a scratch database for synthetic row assertions.
 * @param {string} databasePath Absolute path inside the caller's temporary directory.
 * @returns {Promise<{db: DatabaseSync, setPassword: (username: string, password: string) => Promise<void>, removeUser: (username: string) => Promise<boolean>, close: () => void}>} Scratch SQLite handle and Rust-backed user operations.
 */
export async function authFixture(databasePath) {
  await command(databasePath, ['list']);
  const db = new DatabaseSync(databasePath);
  db.exec('PRAGMA busy_timeout = 2000');
  return {
    db,
    async setPassword(username, password) {
      await command(databasePath, ['add', username], `${password}\n`);
    },
    async removeUser(username) {
      const output = await command(databasePath, ['remove', username]);
      return output.includes(' removed, and signed out everywhere.');
    },
    close() { db.close(); },
  };
}
