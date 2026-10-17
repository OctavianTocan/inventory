import path from 'node:path';
import { pathToFileURL } from 'node:url';

process.env.GEMINI_API_KEY = 'test-key';
process.env.GEMINI_BASE_URL = 'http://127.0.0.1:1/v1beta';
delete process.env.AI_CHAT_API_KEY;

const sum = (text) => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)).reduce((n, code) => n + code, 0);
globalThis.fetch = async (url, options) => {
  const body = JSON.parse(options.body);
  const seed = sum(body.content.parts[0].text);
  const values = Array.from({ length: 768 }, (_, i) => (((seed * 31 + i * 17) % 101) - 50) / 50);
  const text = JSON.stringify({ embedding: { values } });
  return { ok: true, status: 200, statusText: 'OK', headers: new Headers(), text: async () => text, json: async () => JSON.parse(text) };
};

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');
const { createSearch } = await lib('search.mjs');

function plain(value) {
  if (value && typeof value === 'object' && 'elapsed' in value) {
    if (!Number.isInteger(value.elapsed) || value.elapsed < 0) throw new Error('elapsed');
    return { ...value, elapsed: 0 };
  }
  return value;
}

export async function run(file, cases) {
  const store = createStore(openSql(file));
  const searches = { false: createSearch(store, { offline: false }), true: createSearch(store, { offline: true }) };
  const out = [];
  for (const [offline, name, ...args] of cases) {
    try {
      out.push({ ok: plain(await searches[offline][name](...args)) });
    } catch (error) {
      out.push(error.status ? { status: error.status, message: error.message } : { error: error.constructor.name });
    }
  }
  return out;
}
