// Answers mm-js's differential tests with what Node itself does. Reads one JSON request on stdin, writes the answers.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { inspect } from 'node:util';

const fromBits = (hex) => {
  const view = new DataView(new ArrayBuffer(8));
  view.setBigUint64(0, BigInt(`0x${hex}`));
  return view.getFloat64(0);
};
const show = (number) => (Object.is(number, -0) ? '-0' : String(number));
const attempt = (run) => {
  try {
    return run();
  } catch {
    return null;
  }
};
const sha = (text) => createHash('sha256').update(text).digest('hex');

const ops = {
  num: (bits) => String(fromBits(bits)),
  fixed: (bits, digits) => fromBits(bits).toFixed(digits),
  round: (bits) => show(Math.round(fromBits(bits))),
  Number: (text) => show(Number(text)),
  parseFloat: (text) => show(parseFloat(text)),
  parseInt: (text, radix) => show(parseInt(text, radix)),
  trim: (text) => text.trim(),
  len: (text) => text.length,
  slice: (text, start, end) => text.slice(start, end ?? undefined),
  substring: (text, start, end) => text.substring(start, end ?? undefined),
  padStart: (text, target, fill) => text.padStart(target, fill),
  indexOf: (text, search, from) => text.indexOf(search, from),
  lower: (text) => text.toLowerCase(),
  upper: (text) => text.toUpperCase(),
  encodeURIComponent: (text) => encodeURIComponent(text),
  encodeURI: (text) => encodeURI(text),
  decodeURIComponent: (text) => attempt(() => decodeURIComponent(text)),
  decodeURI: (text) => attempt(() => decodeURI(text)),
  dateParse: (text) => show(Date.parse(text)),
  iso: (bits) => attempt(() => new Date(fromBits(bits)).toISOString()),
  b64: (text) => Buffer.from(text, 'base64').toString('hex'),
  b64url: (text) => Buffer.from(text, 'base64url').toString('hex'),
  b64encode: (hex) => Buffer.from(hex, 'hex').toString('base64'),
  b64urlencode: (hex) => Buffer.from(hex, 'hex').toString('base64url'),
  sort: (items) => [...items].sort((a, b) => a.localeCompare(b)),
  stringify: (text) => attempt(() => JSON.stringify(JSON.parse(text))),
  inspect: (text) => inspect(JSON.parse(text)),
  parseError: (text) => { try { JSON.parse(text); return null; } catch (error) { return `${error.name}: ${error.message}`; } },
};

const request = JSON.parse(readFileSync(0, 'utf8'));
let answer;
if (request.files) {
  answer = request.files.map((file) => {
    const parsed = attempt(() => JSON.parse(readFileSync(file, 'utf8')));
    return parsed === null ? null : [sha(JSON.stringify(parsed)), sha(JSON.stringify(parsed, null, 2))];
  });
} else if (request.db) {
  const { DatabaseSync } = await import('node:sqlite');
  const db = new DatabaseSync(request.db, { readOnly: true });
  answer = [];
  const tables = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'search_fts%'").all();
  for (const { name } of tables) {
    const select = db.prepare(`SELECT * FROM "${name}"`);
    select.setReadBigInts(true);
    for (const row of select.all()) {
      for (const value of Object.values(row)) {
        if (typeof value !== 'string' || !/^[[{]/.test(value)) continue;
        const parsed = attempt(() => JSON.parse(value));
        if (parsed !== null) answer.push([value, sha(JSON.stringify(parsed)), sha(JSON.stringify(parsed, null, 2))]);
      }
    }
  }
} else {
  answer = request.cases.map(([op, ...args]) => ops[op](...args));
}
process.stdout.write(JSON.stringify(answer));
