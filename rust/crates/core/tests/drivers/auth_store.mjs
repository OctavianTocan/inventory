// Runs lib/auth-store.mjs operations on a database file for mm-core's differential tests, with a fixed clock.
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const crypto = require('node:crypto');
const store = await import(pathToFileURL(join(process.env.MYMIND_ROOT, 'lib', 'auth-store.mjs')).href);

/** scrypt(password, salt, 64) as hex for each (password, salt hex) pair. */
export function hashes(pairs) {
  return pairs.map(([password, salt]) => crypto.scryptSync(password, Buffer.from(salt, 'hex'), 64).toString('hex'));
}

const rows = (db, sql) => db.prepare(sql).all().map((row) => Object.fromEntries(Object.entries(row).map(([key, value]) => [key, value instanceof Uint8Array ? Buffer.from(value).toString('hex') : value])));

/** Run `ops` on the database at `path`; answers one `{ok}` or `{error: {name, message}}` per op plus the final rows. */
export async function run(path, ops, now) {
  const realNow = Date.now;
  Date.now = () => now;
  const db = store.openAuthStore(path);
  const answers = [];
  try {
    for (const [name, ...args] of ops) {
      try {
        let value;
        if (name === 'set') value = await store.setPassword(db, ...args);
        else if (name === 'check') value = await store.checkPassword(db, ...args);
        else if (name === 'remove') value = store.removeUser(db, ...args);
        else if (name === 'list') value = store.listUsers(db);
        else if (name === 'subject') value = store.passwordSubject(...args);
        else if (name === 'account') value = store.accountOf(...args);
        else if (name === 'setLibrary') value = store.setLibrary(db, ...args) ?? null;
        else if (name === 'library') value = store.libraryOf(db, ...args);
        else if (name === 'sql') value = db.prepare(args[0]).run(...args.slice(1)) && null;
        answers.push({ ok: value === undefined ? null : value });
      } catch (error) {
        answers.push({ error: { name: error?.constructor?.name, message: String(error?.message ?? error) } });
      }
    }
    return { answers, users: rows(db, 'SELECT username, created_ms FROM users ORDER BY username'), libraries: rows(db, 'SELECT * FROM libraries ORDER BY account'), sessions: rows(db, 'SELECT hash, sub FROM sessions ORDER BY hash'), tokens: rows(db, 'SELECT hash, sub FROM app_tokens ORDER BY hash') };
  } finally {
    Date.now = realNow;
  }
}
