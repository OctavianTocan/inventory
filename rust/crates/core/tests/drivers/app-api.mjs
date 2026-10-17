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
const { createAppApi } = await lib('app-api.mjs');

export async function run(file, requests) {
  const store = createStore(openSql(file));
  const search = createSearch(store, { offline: false });
  let sent;
  const send = (res, status, body, headers = {}) => {
    sent = { status, headers: Object.entries({ 'cache-control': 'no-store', ...headers }), body: String(body) };
  };
  const api = createAppApi({ store, search, localize: (text) => text.replaceAll('https://x.com', '/x'), send });
  const answer = async (method, target) => {
    sent = null;
    try {
      const handled = await api.handle({ method }, {}, new URL(target, 'http://localhost'));
      return handled ? sent : null;
    } catch (error) {
      return { threw: error.constructor.name };
    }
  };
  const out = [];
  for (const [method, target, crawl] of requests) {
    let current = target;
    for (let page = 0; page < (crawl ? 60 : 1); page += 1) {
      const reply = await answer(method, current);
      out.push([current, reply]);
      const next = reply?.status === 200 && crawl ? JSON.parse(reply.body).next : null;
      if (!next) break;
      current = `${target}${target.includes('?') ? '&' : '?'}cursor=${next}`;
    }
  }
  return out;
}
