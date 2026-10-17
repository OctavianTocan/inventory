import { DatabaseSync } from 'node:sqlite';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

// Durable Objects cannot bind BigInt, so both platforms bind nanoseconds as decimal text.
const bind = params => params.map(value => (typeof value === 'bigint' ? String(value) : value));

/**
 * Open a SQLite file behind the synchronous interface the store shares with a Durable Object
 * (`worker/sql-do.mjs`). Integers beyond 2^53 throw when read, so select them as text.
 * @param {string} path Database file, or `:memory:`. Parent directories are created unless read-only.
 * @param {{readOnly?: boolean}} [options] Open without write access.
 * @returns {{prepare: (sql: string) => {get: Function, all: Function, iterate: Function, run: Function},
 *   exec: (sql: string) => void, transaction: <T>(fn: () => T) => T, close: () => void}} SQL handle; a
 *   statement's `run` returns `{changes}`, and `transaction` nests, rolling back only the inner work when it throws.
 */
export function openSql(path, { readOnly = false } = {}) {
  if (path !== ':memory:' && !readOnly) mkdirSync(dirname(path), { recursive: true });
  const db = new DatabaseSync(path, { readOnly });
  if (!readOnly) db.exec('PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;');
  let depth = 0;
  return {
    prepare(sql) {
      const statement = db.prepare(sql);
      return {
        get: (...params) => statement.get(...bind(params)),
        all: (...params) => statement.all(...bind(params)),
        iterate: (...params) => statement.iterate(...bind(params)),
        run: (...params) => ({ changes: Number(statement.run(...bind(params)).changes) }),
      };
    },
    exec(sql) {
      db.exec(sql);
    },
    transaction(fn) {
      const savepoint = depth ? `nested_${depth}` : null;
      db.exec(savepoint ? `SAVEPOINT ${savepoint}` : 'BEGIN IMMEDIATE');
      depth++;
      try {
        const result = fn();
        db.exec(savepoint ? `RELEASE ${savepoint}` : 'COMMIT');
        return result;
      } catch (error) {
        db.exec(savepoint ? `ROLLBACK TO ${savepoint}; RELEASE ${savepoint}` : 'ROLLBACK');
        throw error;
      } finally {
        depth--;
      }
    },
    close() {
      db.close();
    },
  };
}
