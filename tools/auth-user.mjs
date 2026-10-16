// Manages the password users who can sign in at /signin/password on a server with sign-in (data/auth.db, or AUTH_DB).
// Usage:
//   node tools/auth-user.mjs add <username>                     password from stdin (e.g. `read -s pw; echo "$pw" | …`)
//   node tools/auth-user.mjs add <username> --save-env TEST_USER  generates one and writes TEST_USER_NAME/_PASSWORD to .env
//   node tools/auth-user.mjs remove <username>                  also ends that user's sessions
//   node tools/auth-user.mjs list
// Passwords are never printed.
import { randomBytes } from 'node:crypto';
import { chmodSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { listUsers, openAuthStore, removeUser, setPassword } from '../lib/auth-store.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

const { positionals: [command, username], values } = parseArgs({ allowPositionals: true, options: {
  'save-env': { type: 'string' },
  db: { type: 'string', default: process.env.AUTH_DB ?? join(ROOT, 'data', 'auth.db') },
} });
const usage = 'usage: node tools/auth-user.mjs add <username> [--save-env PREFIX] | remove <username> | list';

// Set `entries` in the repository's .env, replacing earlier values and keeping the file private.
function saveEnv(entries) {
  const file = join(ROOT, '.env');
  let lines = [];
  try { lines = readFileSync(file, 'utf8').split('\n').filter((line, i, all) => line || i < all.length - 1); } catch {}
  lines = lines.filter((line) => !Object.keys(entries).some((key) => line.startsWith(`${key}=`)));
  writeFileSync(file, `${[...lines, ...Object.entries(entries).map(([key, value]) => `${key}=${value}`)].join('\n')}\n`, { mode: 0o600 });
  chmodSync(file, 0o600);
}

// The password piped to stdin, without its trailing newline.
async function stdinPassword() {
  if (process.stdin.isTTY) throw new Error('pipe the password to stdin, or use --save-env to generate one');
  let text = '';
  for await (const chunk of process.stdin) text += chunk;
  return text.replace(/\r?\n$/, '');
}

const db = openAuthStore(values.db);
try {
  if (command === 'add' && username) {
    const prefix = values['save-env'];
    if (prefix && !/^[A-Z][A-Z0-9_]*$/.test(prefix)) throw new Error('--save-env takes an uppercase prefix such as TEST_USER');
    const password = prefix ? randomBytes(24).toString('base64url') : await stdinPassword();
    const name = await setPassword(db, username, password);
    if (prefix) saveEnv({ [`${prefix}_NAME`]: name, [`${prefix}_PASSWORD`]: password });
    console.log(`${name} can sign in at /signin/password${prefix ? `; ${prefix}_NAME and ${prefix}_PASSWORD are in .env` : ''}.`);
  } else if (command === 'remove' && username) {
    console.log(removeUser(db, username) ? `${username} removed, and signed out everywhere.` : `No password user ${username}.`);
  } else if (command === 'list') {
    for (const user of listUsers(db)) console.log(`${user.username}\tadded ${new Date(user.created_ms).toISOString()}`);
  } else {
    console.error(usage);
    process.exitCode = 2;
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
} finally {
  db.close();
}
