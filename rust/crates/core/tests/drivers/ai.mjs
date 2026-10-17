// Runs lib/ai.mjs against a scripted fake of the hosted APIs. Reads {cases: [{fn, args, script, env}]} on stdin and
// prints one {result | error, log} per case. A script maps a route (embedContent, batchEmbedContents, chat, systemone)
// to steps {status, headers, body, statusText, fail, hang, hangBody}; the last step repeats once the list runs out.
import { readFileSync } from 'node:fs';
import { STATUS_CODES } from 'node:http';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = process.env.MYMIND_ROOT ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../..');
const ENV_NAMES = ['GEMINI_API_KEY', 'AI_CHAT_API_KEY', 'TYPESAFE_API_KEY', 'AI_CHAT_MODEL', 'GEMINI_BASE_URL', 'AI_CHAT_BASE_URL', 'TYPESAFE_BASE_URL'];
const RANDOM = [0.1, 0.35, 0.6, 0.85];
const ROUTES = [[':embedContent', 'embedContent'], [':batchEmbedContents', 'batchEmbedContents'], ['/chat/completions', 'chat'], ['/v1/systemone', 'systemone']];

const request = JSON.parse(readFileSync(0, 'utf8'));
const out = [];
let session = 0;

const toItem = (item) => item && {
  title: item.title, type: item.kind, text: item.text, url: item.url, domain: item.domain,
  ...(item.image ? { image: { mime: item.image.mime, data: Buffer.from(item.image.b64, 'base64') } } : {}),
};

for (const one of request.cases) {
  const log = [];
  const counters = {};
  let draws = 0;
  for (const name of ENV_NAMES) delete process.env[name];
  Object.assign(process.env, one.env);
  Math.random = () => RANDOM[draws++ % RANDOM.length];
  globalThis.setTimeout = (fn, ms) => { log.push({ sleep: ms }); queueMicrotask(fn); return 0; };
  let current;
  AbortSignal.timeout = (ms) => { current = new AbortController(); current.ms = ms; return current.signal; };
  globalThis.fetch = async (url, options) => {
    const route = ROUTES.find(([suffix]) => url.endsWith(suffix))?.[1] ?? 'other';
    const steps = (one.script ?? {})[route] ?? [];
    const step = steps[Math.min(counters[route] = (counters[route] ?? -1) + 1, steps.length - 1)] ?? { fail: 'ECONNREFUSED' };
    log.push({ route, url, method: options.method, headers: Object.entries(options.headers), body: String(options.body).replace(/mymind-image-[0-9a-f-]{36}/g, 'mymind-image-UUID'), timeoutMs: current.ms });
    const timeout = () => new DOMException('timed out', 'TimeoutError');
    if (step.hang) throw timeout();
    if (step.fail) throw Object.assign(new Error(step.fail), { name: step.fail });
    const body = step.hangBody ? null : (step.body ?? '');
    const read = async () => { if (body === null) throw timeout(); return body; };
    return {
      ok: step.status >= 200 && step.status < 300, status: step.status, statusText: STATUS_CODES[step.status] ?? '',
      headers: new Headers(step.headers ?? {}), text: read, json: async () => JSON.parse(await read()),
    };
  };
  const ai = await import(`${pathToFileURL(path.resolve(ROOT, 'lib/ai.mjs')).href}?s=${session++}`);
  const args = one.args;
  try {
    let result;
    switch (one.fn) {
      case 'documentText': result = ai.documentText(toItem(args.item)); break;
      case 'cosine': result = ai.cosine(args.a, args.b); break;
      case 'embed': result = Array.from(await ai.embed(toItem(args.item), { kind: args.kind })); break;
      case 'embedTexts': {
        const usage = [];
        const texts = args.texts.map((text) => (typeof text === 'string' ? text : toItem(text)));
        const vectors = await ai.embedTexts(texts, { kind: args.kind, onUsage: (value) => usage.push(value) });
        result = { vectors: vectors.map((vector) => Array.from(vector)), usage };
        break;
      }
      case 'describe': result = await ai.describe(toItem(args.item)); break;
      case 'pickTags': result = await ai.pickTags(toItem(args.item), args.candidates); break;
      case 'embedVocabulary': result = (await ai.embedVocabulary(args.vocabulary)).map((tag) => ({ name: tag.name, embedding: Array.from(tag.embedding) })); break;
      case 'embedVocabularyTwice': {
        const rows = async (list) => (await ai.embedVocabulary(list)).map((tag) => ({ name: tag.name, embedding: Array.from(tag.embedding) }));
        result = [await rows(args.first), await rows(args.second)];
        break;
      }
      case 'analyze': {
        const analysis = await ai.analyze(toItem(args.item), { vocabulary: args.vocabulary });
        result = { ...analysis, embedding: Array.from(analysis.embedding) };
        break;
      }
      default: throw new Error(`unknown fn ${one.fn}`);
    }
    out.push({ result, log });
  } catch (error) {
    out.push({ error: { name: error?.constructor?.name ?? 'Error', message: String(error?.message ?? error), service: error?.service ?? null, status: error?.status ?? null }, log });
  }
}
process.stdout.write(JSON.stringify(out));
