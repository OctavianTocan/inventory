// replay/: a content-addressed store of live responses that server.mjs answers from before falling
// back to capture/. Filled by tools/drive.mjs during live runs, or from a `rea capture-browser-scenario`
// result recorded with response bodies.
// Usage: node tools/replay-store.mjs <scenario.out.json> [...more] (later files win)
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const STORE = path.join(ROOT, 'replay');
const PATH_ONLY_ORIGINS = new Set(['https://media.mymind.host', 'https://carbon-media.accelerator.net']);
const DROP_HEADERS = /^(x-amz-|x-cache$|x-authenticity-token$)/;

/**
 * Builds the lookup key the replay server uses for a live request.
 * @param {string} method HTTP method.
 * @param {string} url Absolute live URL.
 * @param {boolean} isDocument Whether the browser requested it as a top-level document.
 * @param {string} [variant] Device variant from `deviceVariant`; empty for desktop.
 * @returns {string} Key; signed media URLs collapse to their path so expiring signatures still match.
 */
export function replayKey(method, url, isDocument, variant = '') {
  const u = new URL(url);
  const pathOnly = PATH_ONLY_ORIGINS.has(u.origin) || u.pathname.startsWith('/media/');
  const key = `${isDocument ? 'DOC' : method} ${u.origin}${u.pathname}${pathOnly ? '' : u.search}`;
  return variant ? `${variant}|${key}` : key;
}

export { deviceVariant } from '../lib/device.mjs';

/**
 * Picks the response headers worth replaying.
 * @param {Iterable<{ name: string, value: string | null }>} headers Live response headers.
 * @returns {Record<string, string>} Lower-cased content-type, CORS exposure and app `x-` headers.
 */
export function keepHeaders(headers) {
  const kept = {};
  for (const { name, value } of headers) {
    const lower = name.toLowerCase();
    if (value !== null && (lower === 'content-type' || lower === 'access-control-expose-headers' || (lower.startsWith('x-') && !DROP_HEADERS.test(lower)))) {
      kept[lower] = value;
    }
  }
  return kept;
}

/**
 * Opens replay/ for writing.
 * @returns {Promise<{ index: Record<string, object>, put: (keys: string[], status: number, headers: Record<string, string>, body: Buffer) => Promise<void>, save: () => Promise<void> }>}
 *   `put` stores one body under every key given; `save` writes the index.
 */
export async function openStore() {
  await mkdir(path.join(STORE, 'bodies'), { recursive: true });
  const indexPath = path.join(STORE, 'index.json');
  const index = JSON.parse(await readFile(indexPath, 'utf8').catch(() => '{}'));
  const written = {};
  return {
    index,
    async put(keys, status, headers, body) {
      const file = createHash('sha256').update(body).digest('hex');
      await writeFile(path.join(STORE, 'bodies', file), body);
      for (const key of keys) index[key] = written[key] = { status, headers, file };
    },
    // Merges into whatever is on disk now, so concurrent runs don't drop each other's entries.
    async save() {
      const current = JSON.parse(await readFile(indexPath, 'utf8').catch(() => '{}'));
      await writeFile(indexPath, JSON.stringify(Object.assign(current, written), null, 1));
    },
  };
}

async function main(inputs) {
  const store = await openStore();
  let added = 0;
  for (const input of inputs) {
    const { items } = JSON.parse(await readFile(input, 'utf8')).normalized_result.events;
    const requests = new Map(items.filter((e) => e.kind === 'request').map((e) => [e.transaction_id, e]));
    const responses = new Map(items.filter((e) => e.kind === 'response').map((e) => [e.transaction_id, e]));
    const originalOf = (tx) => {
      let request = requests.get(tx);
      while (request?.redirected_from_transaction_id) request = requests.get(request.redirected_from_transaction_id);
      return request;
    };
    for (const content of items) {
      if (content.kind !== 'network-content' || content.phase !== 'response' || content.body?.state !== 'captured') continue;
      const final = requests.get(content.transaction_id);
      const response = responses.get(content.transaction_id);
      if (!final || !response || response.status !== 200) continue;
      const keys = [...new Set([final, originalOf(content.transaction_id)])].map((r) => replayKey(r.method, r.url.url, r.resource_type === 'document'));
      await store.put(keys, 200, keepHeaders(content.headers?.items ?? []), Buffer.from(content.body.content, 'base64'));
      added += keys.length;
    }
  }
  await store.save();
  console.log(`replay/: ${Object.keys(store.index).length} keys (${added} written this run)`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main(process.argv.slice(2));
