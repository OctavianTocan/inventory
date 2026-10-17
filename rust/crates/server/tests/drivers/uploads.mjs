// lib/uploads.mjs driven by generated operation programs over a directory, for the native uploads' differential test.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);

const SYMBOL = /@([utc])([0-9a-z]+)/g;

export async function run(dbFile, programs) {
  const { openSql } = await lib('sql-node.mjs');
  const { createStore } = await lib('store.mjs');
  const { createUploads } = await lib('uploads.mjs');
  const store = createStore(openSql(dbFile));
  const out = [];
  for (const program of programs) {
    store.db.prepare('DELETE FROM uploads').run();
    let now = program.now;
    Date.now = () => now;
    const ids = [], tokens = [];
    let fresh = null;
    const cards = program.cards;
    const resolve = (text) => text.replace(SYMBOL, (whole, kind, rest) => {
      if (kind === 'u') return ids[Number(rest)] ?? whole;
      if (kind === 't') return tokens[Number(rest)] ?? whole;
      return cards[rest] ?? whole;
    });
    const deep = (value) => (typeof value === 'string' ? resolve(value) : Array.isArray(value) ? value.map(deep)
      : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([key, item]) => [key, deep(item)])) : value);
    const normalize = (value) => {
      let text = JSON.stringify(value ?? null);
      const swaps = [[program.dir, '@dir'], ...tokens.map((token, i) => [token, `@t${i}`]), ...ids.map((id, i) => [id, `@u${i}`]),
        ...Object.entries(cards).map(([key, id]) => [id, `@c${key}`])];
      if (fresh) swaps.push([fresh, '@new']);
      for (const [from, to] of swaps) text = text.split(from).join(to);
      return JSON.parse(text);
    };
    const media = {
      async prepare(input, type, name, who, cardId) {
        fresh = cardId;
        if (name === 'bad.bin') throw new Error('cannot preview');
        if (name === 'status.bin') throw Object.assign(new Error('quota'), { status: 507 });
        const size = input.file ? fs.statSync(input.file).size : null;
        return { data: { type: 'Image', title: name, blob: { path: `p/${cardId}` }, size, mime: type, input: input.file ?? input.key ?? null, user: who.accountUid }, extras: {} };
      },
      async response(file) {
        return file.includes('gone') ? null : { file, type: 'image/jpeg', size: 0 };
      },
    };
    const uploads = createUploads(store, program.dir, media, program.maxBytes == null ? {} : { maxBytes: program.maxBytes });
    const request = (op) => {
      const headers = deep(op.headers);
      const body = Buffer.alloc(op.body.length);
      for (let i = 0; i < body.length; i++) body[i] = (i * 7 + op.body.byte) % 251;
      const pieces = [];
      for (let at = 0, k = 0; at < body.length; k++) {
        const size = Math.max(1, op.body.chunks[k % op.body.chunks.length]);
        pieces.push(body.subarray(at, at + size));
        at += size;
      }
      return { headers, async *[Symbol.asyncIterator]() {
        for (let i = 0; i < pieces.length; i++) {
          if (op.body.failAfter === i) throw new Error('boom');
          yield pieces[i];
        }
      } };
    };
    const execute = async (raw) => {
      const op = deep(raw);
      switch (raw.op) {
        case 'authorize': {
          const result = await uploads.authorize(raw.count === undefined ? undefined : raw.count);
          for (const item of [result].flat()) { ids.push(item.url.split('/').pop()); tokens.push(item.token); }
          return result;
        }
        case 'put': return uploads.put(op.upload, request(raw));
        case 'prepare': { const result = await uploads.prepare(op.upload); return result; }
        case 'claim': return uploads.claim(op.upload, op.card, op.reuse) ?? null;
        case 'cleanup': return (await uploads.cleanupClaimed(op.upload)) ?? null;
        case 'source': return uploads.source(op.upload);
        case 'sweep': return (await uploads.sweep()) ?? null;
        case 'time': now += raw.ms; return null;
        case 'sql': store.db.prepare(op.sql).run(...op.params); return null;
        case 'plant': fs.mkdirSync(program.dir, { recursive: true }); fs.writeFileSync(path.join(program.dir, op.name), op.text); return null;
        case 'tail': fs.appendFileSync(path.join(program.dir, op.upload), op.text); return null;
        case 'race': return Promise.all(raw.ops.map((item) => attempt(item)));
        default: throw new Error(`unknown op ${raw.op}`);
      }
    };
    const attempt = async (op) => {
      try {
        const value = await execute(op);
        if (op.op === 'prepare' && value?.id) fresh = value.id;
        return { ok: normalize(value) };
      } catch (error) {
        return { error: { status: error.status ?? null, message: error.status ? error.message : 'other' } };
      }
    };
    const results = [];
    for (const op of program.ops) results.push(await attempt(op));
    const rows = store.db.prepare('SELECT * FROM uploads ORDER BY rowid').all().map(({ token_hash, ...row }) => normalize(row));
    const files = fs.existsSync(program.dir) ? fs.readdirSync(program.dir).map((name) => [normalize(name), fs.readFileSync(path.join(program.dir, name)).toString('base64')]).sort((a, b) => (a[0] < b[0] ? -1 : 1)) : [];
    out.push({ results, rows, files });
  }
  return out;
}
