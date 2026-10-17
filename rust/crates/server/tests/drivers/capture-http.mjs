// lib/capture-http.mjs driven by a list of fetches against a local fixture server, for the native transport's differential test.
import path from 'node:path';
import { brotliCompressSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

const KNOWN = new Set(['Page unavailable', 'Too many capture redirects', 'Capture resource too large', 'Unsupported capture encoding',
  'Invalid capture redirect', 'Unsupported capture destination']);

export function brotli(base64) {
  return brotliCompressSync(Buffer.from(base64, 'base64')).toString('base64');
}

export async function run(cases, remaining) {
  const { fetchResource } = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib/capture-http.mjs')).href);
  const budget = { remaining };
  const out = [];
  for (const item of cases) {
    const options = { offline: true };
    if (item.limit != null) options.limit = item.limit;
    if (item.budget) options.budget = budget;
    let answer;
    try {
      const result = await fetchResource(item.url, options);
      answer = { ok: { bytes: Buffer.from(result.bytes).toString('base64'), url: result.url, mime: result.mime, status: result.status,
        headers: Object.fromEntries(Object.entries(result.headers).sort(([a], [b]) => (a < b ? -1 : 1))) } };
    } catch (error) {
      answer = { error: { status: error.status ?? null, code: error.code === 'CAPTURE_BLOCKED' ? error.code : null, message: KNOWN.has(error.message) ? error.message : 'other' } };
    }
    out.push({ ...answer, remaining: budget.remaining });
  }
  return out;
}
