import { randomBytes, scrypt, timingSafeEqual } from 'node:crypto';
import { promisify } from 'node:util';
import { openSql } from './sql-node.mjs';

const derive = promisify(scrypt);
const KEY_BYTES = 64;
const USERNAME = /^[a-z0-9._@-]{1,64}$/;
// Checked when the username is unknown, so a miss costs as much time as a wrong password. Made on first use because
// Workers allow no random values at module load.
let dummy;

const normalize = (username) => String(username ?? '').trim().toLowerCase();

/**
 * Open the sign-in database file, creating its tables when missing.
 * @param {string} path Database file, usually `data/auth.db`.
 * @returns {object} The open SQL handle (`openSql`); the caller closes it.
 */
export function openAuthStore(path) {
  return installAuthSchema(openSql(path));
}

/**
 * Create the sign-in tables on an open SQL handle when missing: sessions, app tokens, password users, sign-ins in
 * progress, app nonces, recent password failures and which library each account reaches.
 * @param {object} db Writable SQL handle from `openSql` or the Durable Object adapter.
 * @returns {object} The same handle.
 */
export function installAuthSchema(db) {
  db.exec(`CREATE TABLE IF NOT EXISTS sessions (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL,
    created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT`);
  db.exec(`CREATE TABLE IF NOT EXISTS app_tokens (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL, kind TEXT NOT NULL,
    device TEXT NOT NULL, created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT`);
  db.exec(`CREATE TABLE IF NOT EXISTS users (username TEXT PRIMARY KEY, salt BLOB NOT NULL, hash BLOB NOT NULL,
    created_ms INTEGER NOT NULL) STRICT`);
  db.exec(`CREATE TABLE IF NOT EXISTS logins (state TEXT PRIMARY KEY, verifier TEXT NOT NULL, nonce TEXT NOT NULL,
    created_ms INTEGER NOT NULL) STRICT`);
  db.exec('CREATE TABLE IF NOT EXISTS app_nonces (nonce TEXT PRIMARY KEY, created_ms INTEGER NOT NULL) STRICT');
  db.exec('CREATE TABLE IF NOT EXISTS failures (key TEXT NOT NULL, attempt TEXT NOT NULL, at_ms INTEGER NOT NULL) STRICT');
  db.exec('CREATE INDEX IF NOT EXISTS failures_by_key ON failures (key, at_ms)');
  db.exec('CREATE TABLE IF NOT EXISTS libraries (account TEXT PRIMARY KEY, library TEXT NOT NULL) STRICT');
  return db;
}

/**
 * Session subject that marks a session as signed in with this user's password.
 * @param {string} username Password user, in any case.
 * @returns {string} The `sub` stored with the user's sessions.
 */
export const passwordSubject = (username) => `password:${normalize(username)}`;

/**
 * Add a password user, or give an existing one a new password.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @param {string} username 1–64 characters of letters, digits and `._@-`; stored lowercase.
 * @param {string} password At least 16 characters.
 * @returns {Promise<string>} The stored username.
 * @throws {Error} If the username or password is not acceptable.
 */
export async function setPassword(db, username, password) {
  const name = normalize(username);
  if (!USERNAME.test(name)) throw new Error('usernames are 1-64 characters of letters, digits and ._@-');
  if (typeof password !== 'string' || password.length < 16) throw new Error('passwords need at least 16 characters');
  const salt = randomBytes(16);
  const hash = await derive(password, salt, KEY_BYTES);
  db.prepare(`INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)
    ON CONFLICT (username) DO UPDATE SET salt = excluded.salt, hash = excluded.hash`).run(name, salt, hash, Date.now());
  return name;
}

/**
 * Check a username and password.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @param {string} username Username as typed.
 * @param {string} password Password as typed.
 * @returns {Promise<string|null>} The stored username when the password is right, otherwise null.
 */
export async function checkPassword(db, username, password) {
  const name = normalize(username);
  const row = USERNAME.test(name) ? db.prepare('SELECT salt, hash FROM users WHERE username = ?').get(name) : undefined;
  const { salt, hash } = row ?? (dummy ??= { salt: randomBytes(16), hash: randomBytes(KEY_BYTES) });
  const attempt = await derive(String(password ?? ''), salt, KEY_BYTES);
  return timingSafeEqual(attempt, hash) && row ? name : null;
}

/**
 * Remove a password user and end their sessions and app tokens.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @param {string} username Password user, in any case.
 * @returns {boolean} True when the user existed.
 */
export function removeUser(db, username) {
  db.prepare('DELETE FROM sessions WHERE sub = ?').run(passwordSubject(username));
  db.prepare('DELETE FROM app_tokens WHERE sub = ?').run(passwordSubject(username));
  return db.prepare('DELETE FROM users WHERE username = ?').run(normalize(username)).changes > 0;
}

/**
 * List the password users.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @returns {{username: string, created_ms: number}[]} Users by name.
 */
export function listUsers(db) {
  return db.prepare('SELECT username, created_ms FROM users ORDER BY username').all();
}

/**
 * The account name a signed-in identity has in the libraries table: `google:<email>` for Google, the session subject
 * (`password:<username>`) for password users, so a password user can never take a Google account's library.
 * @param {{sub: string, email: string}} identity Signed-in session or app token.
 * @returns {string} The account name.
 */
export const accountOf = (identity) => (identity.sub.startsWith('password:') ? identity.sub : `google:${identity.email.toLowerCase()}`);

/**
 * Give an account a library, or move it to another one.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @param {string} account Account name from {@link accountOf}.
 * @param {string} library Library name: lower-case letters, digits and dashes.
 * @returns {void}
 * @throws {Error} If the library name is not acceptable.
 */
export function setLibrary(db, account, library) {
  if (!/^[a-z0-9-]+$/.test(library)) throw new Error('library names are lower-case letters, digits and dashes');
  db.prepare('INSERT INTO libraries (account, library) VALUES (?, ?) ON CONFLICT (account) DO UPDATE SET library = excluded.library')
    .run(account, library);
}

/**
 * The library a signed-in identity reaches.
 * @param {object} db SQL handle from {@link openAuthStore}.
 * @param {{sub: string, email: string}} identity Signed-in session or app token.
 * @returns {string|null} The library name, or null when the account has none.
 */
export function libraryOf(db, identity) {
  return db.prepare('SELECT library FROM libraries WHERE account = ?').get(accountOf(identity))?.library ?? null;
}
