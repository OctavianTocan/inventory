import { randomBytes, scrypt, timingSafeEqual } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { promisify } from 'node:util';

const derive = promisify(scrypt);
const KEY_BYTES = 64;
const USERNAME = /^[a-z0-9._@-]{1,64}$/;
// Checked when the username is unknown, so a miss costs as much time as a wrong password.
const DUMMY = { salt: randomBytes(16), hash: randomBytes(KEY_BYTES) };

const normalize = (username) => String(username ?? '').trim().toLowerCase();

/**
 * Open the sign-in database, creating its sessions and password users tables when missing.
 * @param {string} path Database file, usually `data/auth.db`.
 * @returns {DatabaseSync} The open database; the caller closes it.
 */
export function openAuthStore(path) {
  mkdirSync(dirname(path), { recursive: true });
  const db = new DatabaseSync(path);
  db.exec(`CREATE TABLE IF NOT EXISTS sessions (hash TEXT PRIMARY KEY, sub TEXT NOT NULL, email TEXT NOT NULL,
    created_ms INTEGER NOT NULL, expires_ms INTEGER NOT NULL, used_ms INTEGER NOT NULL) STRICT`);
  db.exec(`CREATE TABLE IF NOT EXISTS users (username TEXT PRIMARY KEY, salt BLOB NOT NULL, hash BLOB NOT NULL,
    created_ms INTEGER NOT NULL) STRICT`);
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
 * @param {DatabaseSync} db Database from {@link openAuthStore}.
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
 * @param {DatabaseSync} db Database from {@link openAuthStore}.
 * @param {string} username Username as typed.
 * @param {string} password Password as typed.
 * @returns {Promise<string|null>} The stored username when the password is right, otherwise null.
 */
export async function checkPassword(db, username, password) {
  const name = normalize(username);
  const row = USERNAME.test(name) ? db.prepare('SELECT salt, hash FROM users WHERE username = ?').get(name) : undefined;
  const { salt, hash } = row ?? DUMMY;
  const attempt = await derive(String(password ?? ''), salt, KEY_BYTES);
  return timingSafeEqual(attempt, hash) && row ? name : null;
}

/**
 * Remove a password user and end their sessions.
 * @param {DatabaseSync} db Database from {@link openAuthStore}.
 * @param {string} username Password user, in any case.
 * @returns {boolean} True when the user existed.
 */
export function removeUser(db, username) {
  db.prepare('DELETE FROM sessions WHERE sub = ?').run(passwordSubject(username));
  return db.prepare('DELETE FROM users WHERE username = ?').run(normalize(username)).changes > 0;
}

/**
 * List the password users.
 * @param {DatabaseSync} db Database from {@link openAuthStore}.
 * @returns {{username: string, created_ms: number}[]} Users by name.
 */
export function listUsers(db) {
  return db.prepare('SELECT username, created_ms FROM users ORDER BY username').all();
}
