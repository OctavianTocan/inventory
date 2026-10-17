import { deviceVariant } from './device.mjs';

const SCHEMA = `CREATE TABLE IF NOT EXISTS recorded (
  key TEXT PRIMARY KEY,
  status INTEGER NOT NULL,
  headers TEXT NOT NULL CHECK(json_valid(headers)),
  body BLOB NOT NULL
) STRICT`;

/**
 * Keep a library's recorded mymind responses in its own database, so a hosted server answers what the replica cannot
 * build yet (page shells, detail views of recorded cards, small replies) without reading `replay/` or `capture/`.
 * Keys have the replay format, `[run:<run>|][<device>|]<KIND> https://access.mymind.com<path>[?search]`; `shell`
 * holds the fallback page shell. Bodies are stored already localized.
 * @param {object} db SQL handle (`openSql` or the Durable Object adapter).
 * @returns {{entry: Function, has: Function, put: Function, count: Function}} `entry(req, key)` returns the
 *   `{status, headers, body}` recorded for a request's key (a phone's own recording first, then a verification run's
 *   with `x-replay-run`), or null; `has(req, key)` says whether one exists; `put(key, status, headers, body)` stores
 *   one unless the key is taken; `count()` is the number stored.
 */
export function createRecorded(db) {
  db.exec(SCHEMA);
  const find = (key) => db.prepare('SELECT status, headers, body FROM recorded WHERE key = ?').get(key);
  const exists = (key) => Boolean(db.prepare('SELECT 1 FROM recorded WHERE key = ?').get(key));
  // The stored key a request reads, following the replay server's device and run precedence.
  function resolve(req, key) {
    const variant = deviceVariant(req.headers['user-agent']);
    if (variant && exists(`${variant}|${key}`)) key = `${variant}|${key}`;
    const run = req.headers['x-replay-run'];
    return run && exists(`run:${run}|${key}`) ? `run:${run}|${key}` : key;
  }
  return {
    entry(req, key) {
      const row = find(resolve(req, key));
      return row ? { status: row.status, headers: JSON.parse(row.headers), body: new Uint8Array(row.body) } : null;
    },
    has: (req, key) => exists(resolve(req, key)),
    put(key, status, headers, body) {
      db.prepare('INSERT OR IGNORE INTO recorded (key, status, headers, body) VALUES (?, ?, ?, ?)').run(key, status, JSON.stringify(headers), body);
    },
    count: () => db.prepare('SELECT COUNT(*) AS n FROM recorded').get().n,
  };
}
