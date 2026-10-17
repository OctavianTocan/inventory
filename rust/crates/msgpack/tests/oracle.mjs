// Answers mm-msgpack's differential tests with the mirrored JS codec. Reads one JSON request on stdin.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const msgpack = await import(pathToFileURL(path.join(ROOT, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);

// A copy of encodeMsgpack in server.mjs, which is not importable without starting the server.
function encodeMsgpack(value, parts = []) {
  const header = (fix, b16, b32, n, fixMax) => {
    if (n <= fixMax) parts.push(Buffer.from([fix | n]));
    else if (n < 0x10000) parts.push(Buffer.from([b16, n >> 8, n & 0xff]));
    else {
      const h = Buffer.alloc(5);
      h[0] = b32;
      h.writeUInt32BE(n, 1);
      parts.push(h);
    }
  };
  if (value === null || value === undefined) parts.push(Buffer.from([0xc0]));
  else if (typeof value === 'boolean') parts.push(Buffer.from([value ? 0xc3 : 0xc2]));
  else if (typeof value === 'number') {
    if (Number.isInteger(value) && value >= 0 && value < 0x80) parts.push(Buffer.from([value]));
    else {
      const b = Buffer.alloc(9);
      b[0] = 0xcb;
      b.writeDoubleBE(value, 1);
      parts.push(b);
    }
  } else if (typeof value === 'string') {
    const bytes = Buffer.from(value, 'utf8');
    if (bytes.length >= 32 && bytes.length < 0x100) parts.push(Buffer.from([0xd9, bytes.length]));
    else header(0xa0, 0xda, 0xdb, bytes.length, 31);
    parts.push(bytes);
  } else if (Array.isArray(value)) {
    header(0x90, 0xdc, 0xdd, value.length, 15);
    for (const item of value) encodeMsgpack(item, parts);
  } else {
    const entries = Object.entries(value);
    header(0x80, 0xde, 0xdf, entries.length, 15);
    for (const [k, v] of entries) {
      encodeMsgpack(k, parts);
      encodeMsgpack(v, parts);
    }
  }
  return parts;
}

const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const attempt = (run) => {
  try {
    return run();
  } catch {
    return null;
  }
};
const roundTrip = (bytes, useBigInt64) =>
  attempt(() => Buffer.concat([...msgpack.decodeMulti(bytes, { useBigInt64 })].map((item) => msgpack.encode(item, { useBigInt64 }))));

const request = JSON.parse(readFileSync(0, 'utf8'));
let answer;
if (request.files) {
  answer = request.files.map((file) => {
    const bytes = readFileSync(file);
    return [false, true].map((big) => {
      const out = roundTrip(bytes, big);
      return out && sha(out);
    });
  });
} else if (request.hex) {
  answer = request.hex.map((hex) => [false, true].map((big) => attempt(() => sha(roundTrip(Buffer.from(hex, 'hex'), big))) ?? null));
} else if (request.json) {
  answer = request.json.map((text) => {
    const value = JSON.parse(text);
    return [sha(msgpack.encode(value, { useBigInt64: true })), sha(msgpack.encode(value)), sha(Buffer.concat(encodeMsgpack(value)))];
  });
} else if (request.db) {
  const { DatabaseSync } = await import('node:sqlite');
  const db = new DatabaseSync(request.db, { readOnly: true });
  answer = [];
  for (const { name } of db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%'").all()) {
    const select = db.prepare(`SELECT * FROM "${name}"`);
    select.setReadBigInts(true);
    for (const row of select.all()) {
      for (const value of Object.values(row)) {
        if (typeof value !== 'string' || !/^[[{]/.test(value)) continue;
        const parsed = attempt(() => JSON.parse(value));
        if (parsed !== null) answer.push([value, sha(msgpack.encode(parsed, { useBigInt64: true })), sha(msgpack.encode(parsed)), sha(Buffer.concat(encodeMsgpack(parsed)))]);
      }
    }
  }
} else if (request.timestamps) {
  answer = request.timestamps.map(([sec, nsec]) => Buffer.from(msgpack.encodeTimeSpecToTimestamp({ sec, nsec })).toString('hex'));
} else if (request.dates) {
  answer = request.dates.map((ms) => Buffer.from(msgpack.encode(new Date(ms))).toString('hex'));
}
process.stdout.write(JSON.stringify(answer));
