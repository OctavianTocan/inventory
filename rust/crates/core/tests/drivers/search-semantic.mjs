import path from 'node:path';
import { pathToFileURL } from 'node:url';

const { embeddingsEnabled, decodeVector } = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', 'search-semantic.mjs')).href);

export function enabled(key, base, offline) {
  for (const [name, value] of [['GEMINI_API_KEY', key], ['GEMINI_BASE_URL', base]]) {
    if (value === null) delete process.env[name]; else process.env[name] = value;
  }
  return embeddingsEnabled(offline);
}

export function decode(bytes) {
  const vector = decodeVector(bytes);
  return vector === null ? null : Array.from(vector).map((value) => (Number.isNaN(value) ? 'NaN' : value));
}
