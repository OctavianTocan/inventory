// Durable Objects bind no BigInt, and take bytes as plain typed arrays.
const bind = (params) => params.map((value) =>
  typeof value === 'bigint' ? String(value) : ArrayBuffer.isView(value) ? new Uint8Array(value.buffer, value.byteOffset, value.byteLength) : value);

// Blobs come back as ArrayBuffer; node:sqlite returns Uint8Array, which the store code expects.
function row(value) {
  for (const key in value) if (value[key] instanceof ArrayBuffer) value[key] = new Uint8Array(value[key]);
  return value;
}

/**
 * Open a Durable Object's SQLite storage behind the synchronous interface of `openSql` (`lib/sql-node.mjs`).
 * Integers beyond 2^53 lose precision when read, so select them as text.
 * @param {DurableObjectStorage} storage The object's `ctx.storage`.
 * @returns {{prepare: (sql: string) => {get: Function, all: Function, iterate: Function, run: Function},
 *   exec: (sql: string) => void, transaction: <T>(fn: () => T) => T, close: () => void}} SQL handle; a
 *   statement's `run` returns `{changes}`, and `transaction` nests, rolling back only the inner work when it throws.
 */
export function openDurableSql(storage) {
  const { sql } = storage;
  return {
    prepare(text) {
      return {
        get: (...params) => {
          const next = sql.exec(text, ...bind(params)).next();
          return next.done ? undefined : row(next.value);
        },
        all: (...params) => sql.exec(text, ...bind(params)).toArray().map(row),
        *iterate(...params) {
          for (const value of sql.exec(text, ...bind(params))) yield row(value);
        },
        run: (...params) => {
          for (const _ of sql.exec(text, ...bind(params)));
          return { changes: sql.exec('SELECT changes() AS n').one().n };
        },
      };
    },
    exec(text) {
      for (const _ of sql.exec(text));
    },
    transaction: (fn) => storage.transactionSync(fn),
    close() {},
  };
}
