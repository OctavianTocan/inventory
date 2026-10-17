import { createHash } from 'node:crypto';

const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*$/;
const quote = (name) => {
  if (!IDENTIFIER.test(name)) throw Object.assign(new Error(`Invalid SQL name ${JSON.stringify(name)}`), { status: 400 });
  return `"${name}"`;
};

/**
 * Encode one value read from SQLite (read with BigInt integers) for JSON transport, keeping its storage class:
 * integers become `{i: "<digits>"}`, blobs `{b: "<base64>"}`, and text, reals and null stay as they are.
 * @param {bigint|number|string|Uint8Array|null} value Column value.
 * @returns {object|number|string|null} The JSON-safe value.
 * @throws {Error} For a real that JSON cannot carry (infinite).
 */
export function encodeValue(value) {
  if (typeof value === 'bigint') return { i: String(value) };
  if (value instanceof Uint8Array) return { b: Buffer.from(value.buffer, value.byteOffset, value.byteLength).toString('base64') };
  if (typeof value === 'number' && !Number.isFinite(value)) throw new Error('Cannot transport an infinite real');
  return value;
}

/**
 * Insert rows sent by {@link encodeValue} into a table, each integer cast back to an integer and each blob to bytes.
 * @param {object} db SQL handle (`openSql` or the Durable Object adapter).
 * @param {string} table Table name.
 * @param {string[]} columns Column names, including `rowid` when the row ids must be kept.
 * @param {Array<Array<object|number|string|null>>} rows Encoded rows, one value per column.
 * @returns {number} The number of rows inserted.
 * @throws {Error} Status 400 for a name that is not a plain SQL identifier.
 */
export function insertRows(db, table, columns, rows) {
  const into = `INSERT INTO ${quote(table)} (${columns.map(quote).join(', ')}) VALUES `;
  db.transaction(() => {
    for (const row of rows) {
      const params = [];
      const slots = row.map((value) => {
        if (value === null) return 'NULL';
        if (typeof value === 'object' && 'i' in value) { params.push(value.i); return 'CAST(? AS INTEGER)'; }
        if (typeof value === 'object' && 'b' in value) { params.push(Buffer.from(value.b, 'base64')); return '?'; }
        params.push(value);
        return '?';
      });
      db.prepare(`${into}(${slots.join(', ')})`).run(...params);
    }
  });
  return rows.length;
}

/**
 * A digest of every row of a table that is the same on any two databases holding the same rows with the same storage
 * classes, in any order.
 * @param {object} db SQL handle (`openSql` or the Durable Object adapter).
 * @param {string} table Table name.
 * @param {string[]} columns Column names to cover, including `rowid` to cover the row ids.
 * @returns {{rows: number, sha256: string}} Row count and digest.
 * @throws {Error} Status 400 for a name that is not a plain SQL identifier.
 */
export function tableDigest(db, table, columns) {
  const select = columns.map((name, i) => `typeof(${quote(name)}) AS t${i}, CASE WHEN typeof(${quote(name)}) = 'integer' THEN CAST(${quote(name)} AS TEXT) ELSE ${quote(name)} END AS v${i}`);
  const hashes = [];
  for (const row of db.prepare(`SELECT ${select.join(', ')} FROM ${quote(table)}`).iterate()) {
    const hash = createHash('sha256');
    columns.forEach((_, i) => {
      const value = row[`v${i}`];
      const bytes = value instanceof Uint8Array ? value : Buffer.from(value === null ? '' : String(value));
      hash.update(`${row[`t${i}`]}\0${bytes.length}\0`).update(bytes);
    });
    hashes.push(hash.digest('hex'));
  }
  return { rows: hashes.length, sha256: createHash('sha256').update(hashes.sort().join('\n')).digest('hex') };
}
