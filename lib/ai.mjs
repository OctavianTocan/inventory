// Hosted-API AI features for saved cards: embeddings, TL;DR summaries, tag suggestions, tag picking.
// Keys come from process.env only: GEMINI_API_KEY (embeddings, chat), TYPESAFE_API_KEY (Jev).
import { Buffer } from 'node:buffer';

const EMBED_MODEL = 'gemini-embedding-2';
const EMBED_DIMENSIONS = 768;
const EMBED_BATCH_SIZE = 100;
const EMBED_TEXT_CHARS = 6000;
const CHAT_TEXT_CHARS = 8000;
const JEV_TEXT_CHARS = 6000;
const JEV_MODEL = 'jev-latest';
const JEV_MIN_P = 0.6;
const JEV_QUESTIONS_PER_REQUEST = 100;
const MAX_TAGS = 8;
const NEAREST_TAGS = 20;
const CHAT_IMAGE_MAX_BYTES = 10 * 1024 * 1024;
const JEV_IMAGE = { minSide: 256, maxSide: 1568, minPixels: 78_400, maxPixels: 2_458_624, maxBytes: 16 * 1024 * 1024 };
const MAX_RETRIES = 3;
const TIMEOUT_MS = 60_000;

const SUMMARY_PROMPT = [
  'You file items saved to a personal visual notebook, like mymind. Reply with one JSON object: {"summary": string, "tags": string[]}.',
  'summary: the TL;DR. One or two plain sentences (at most 40 words) saying what the item is and what it is about.',
  'No markdown, no quotes, no filler such as "This article" or "The image shows". Use the language of the item.',
  'tags: 3 to 8 short lowercase labels (one or two words each) for the subject, the field and the kind of thing it is,',
  'such as "typography", "recipe" or "tutorial". No "#", no site names, no generic words like "website", "link", "page" or "image".',
].join('\n');
const SUMMARY_SCHEMA = {
  type: 'object',
  properties: { summary: { type: 'string' }, tags: { type: 'array', items: { type: 'string' } } },
  required: ['summary', 'tags'],
};

/**
 * A card to analyze or embed.
 * @typedef {object} AiItem
 * @property {string} [title] Card title; may be empty.
 * @property {string} [text] Description or body text; for a query embedding, the query string.
 * @property {string} [url] Source URL.
 * @property {string} [domain] Source domain.
 * @property {{ mime: string, data: Buffer }} [image] Cover image; used only when the target API accepts its format and size.
 */

/** Failure of a hosted AI call, after retries; never carries credentials. */
export class AiError extends Error {
  /**
   * @param {string} message Human-readable cause.
   * @param {{ service?: 'gemini' | 'chat' | 'typesafe', status?: number, cause?: unknown }} [details] `service` names the API, `status` is the final HTTP status when there was one.
   */
  constructor(message, { service, status, cause } = {}) {
    super(message, { cause });
    this.name = 'AiError';
    this.service = service;
    this.status = status;
  }
}

const trimBase = (url) => url.replace(/\/+$/, '');
const geminiBase = () => trimBase(process.env.GEMINI_BASE_URL || 'https://generativelanguage.googleapis.com/v1beta');
const chatBase = () => trimBase(process.env.AI_CHAT_BASE_URL || 'https://generativelanguage.googleapis.com/v1beta/openai');
const typesafeBase = () => trimBase(process.env.TYPESAFE_BASE_URL || 'https://api.typesafe.ai');
const chatModel = () => process.env.AI_CHAT_MODEL || 'gemini-2.5-flash-lite';

/** Reads a required key from the environment or fails with a message naming it. */
function requireKey(name, purpose) {
  const value = process.env[name];
  if (!value) throw new AiError(`${name} is not set (needed for ${purpose}); add it to .env`);
  return value;
}

/** Removes any configured key value from text that may reach logs or error messages. */
function redact(text) {
  let out = String(text);
  for (const name of ['GEMINI_API_KEY', 'AI_CHAT_API_KEY', 'TYPESAFE_API_KEY']) {
    const value = process.env[name];
    if (value) out = out.split(value).join('[redacted]');
  }
  return out;
}

/** Collapses whitespace and cuts text to a character budget. */
const clip = (value, max) => String(value ?? '').replace(/\s+/g, ' ').trim().slice(0, max);

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Delay before retry number `attempt` (0-based): honours Retry-After, else 1 s, 2 s, 4 s with jitter. */
function retryDelay(res, attempt) {
  const header = Number(res?.headers.get('retry-after'));
  if (Number.isFinite(header) && header >= 0) return Math.min(header, 30) * 1000;
  return 1000 * 2 ** attempt * (0.75 + Math.random() * 0.5);
}

/** Extracts a short provider error message from a failed response. */
async function errorDetail(res) {
  const raw = await res.text().catch(() => '');
  let detail = raw;
  try {
    const parsed = JSON.parse(raw);
    const body = Array.isArray(parsed) ? parsed[0] : parsed;
    const found = body?.error?.message ?? body?.message ?? body?.detail ?? body?.error ?? body;
    detail = typeof found === 'string' ? found : JSON.stringify(found);
  } catch {
    // body was not JSON; keep the raw text
  }
  return redact(detail).replace(/\s+/g, ' ').trim().slice(0, 300) || res.statusText;
}

/** POSTs JSON and returns the parsed JSON response, retrying 429, 5xx and network failures with backoff. */
async function postJson(service, url, { headers, body }) {
  for (let attempt = 0; ; attempt++) {
    let res;
    try {
      res = await fetch(url, {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...headers },
        body: JSON.stringify(body),
        signal: AbortSignal.timeout(TIMEOUT_MS),
      });
    } catch (cause) {
      if (attempt < MAX_RETRIES) {
        await sleep(retryDelay(null, attempt));
        continue;
      }
      const reason = cause?.cause?.code ?? cause?.name ?? 'network error';
      throw new AiError(`${service} request failed (${reason}); check the connection`, { service, cause });
    }
    if (res.ok) {
      try {
        return await res.json();
      } catch (cause) {
        throw new AiError(`${service} returned a non-JSON body`, { service, status: res.status, cause });
      }
    }
    if ((res.status === 429 || res.status >= 500) && attempt < MAX_RETRIES) {
      await res.text().catch(() => '');
      await sleep(retryDelay(res, attempt));
      continue;
    }
    throw new AiError(`${service} ${res.status}: ${await errorDetail(res)}`, { service, status: res.status });
  }
}

/** Identifies a PNG, JPEG or WebP from its bytes and reads its pixel size; null for anything else. */
function sniffImage(data) {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data ?? []);
  if (buf.length >= 24 && buf.readUInt32BE(0) === 0x89504e47 && buf.readUInt32BE(4) === 0x0d0a1a0a) {
    let animated = false;
    for (let at = 8; at + 8 <= buf.length; at += 12 + buf.readUInt32BE(at)) {
      const type = buf.toString('latin1', at + 4, at + 8);
      if (type === 'acTL') animated = true;
      if (type === 'IDAT') break;
    }
    return { mime: 'image/png', width: buf.readUInt32BE(16), height: buf.readUInt32BE(20), animated, bytes: buf.length };
  }
  if (buf.length >= 4 && buf[0] === 0xff && buf[1] === 0xd8) {
    let at = 2;
    while (at + 4 <= buf.length) {
      if (buf[at] !== 0xff) { at++; continue; }
      const marker = buf[at + 1];
      if (marker === 0xff) { at++; continue; }
      if (marker === 0x01 || (marker >= 0xd0 && marker <= 0xd9)) { at += 2; continue; }
      const length = buf.readUInt16BE(at + 2);
      const isFrame = marker >= 0xc0 && marker <= 0xcf && ![0xc4, 0xc8, 0xcc].includes(marker);
      if (isFrame && at + 9 <= buf.length) {
        return { mime: 'image/jpeg', width: buf.readUInt16BE(at + 7), height: buf.readUInt16BE(at + 5), animated: false, bytes: buf.length };
      }
      if (marker === 0xda) break;
      at += 2 + length;
    }
    return null;
  }
  if (buf.length >= 12 && buf.toString('latin1', 0, 4) === 'RIFF' && buf.toString('latin1', 8, 12) === 'WEBP') {
    return { mime: 'image/webp', width: 0, height: 0, animated: false, bytes: buf.length };
  }
  return null;
}

/** Returns the item's image when Gemini embeddings accept it (PNG or JPEG), else null. */
function embedImage(item) {
  const info = item.image?.data ? sniffImage(item.image.data) : null;
  if (!info || info.animated || (info.mime !== 'image/png' && info.mime !== 'image/jpeg')) return null;
  return { mime: info.mime, base64: Buffer.from(item.image.data).toString('base64') };
}

/** Returns the item's image when the chat model accepts it (PNG, JPEG or WebP, bounded size), else null. */
function chatImage(item) {
  const info = item.image?.data ? sniffImage(item.image.data) : null;
  if (!info || info.bytes > CHAT_IMAGE_MAX_BYTES) return null;
  return { mime: info.mime, base64: Buffer.from(item.image.data).toString('base64') };
}

/** Returns the item's image when Jev accepts it (PNG or JPEG, 256-1568 px per side, pixel and size bounds), else null. */
function jevImage(item) {
  const info = item.image?.data ? sniffImage(item.image.data) : null;
  if (!info || info.animated || (info.mime !== 'image/png' && info.mime !== 'image/jpeg')) return null;
  const { width, height, bytes } = info;
  const { minSide, maxSide, minPixels, maxPixels, maxBytes } = JEV_IMAGE;
  const inRange = width >= minSide && height >= minSide && width <= maxSide && height <= maxSide
    && width * height >= minPixels && width * height <= maxPixels && bytes <= maxBytes;
  if (!inRange) return null;
  return { format: info.mime === 'image/png' ? 'png' : 'jpg', base64: Buffer.from(item.image.data).toString('base64') };
}

/** Lowercases, strips leading '#', collapses spaces; returns '' for names that are not usable tags. */
function normalizeTag(value) {
  const name = String(value ?? '').trim().replace(/^#+/, '').replace(/\s+/g, ' ').toLowerCase();
  return name.length > 0 && name.length <= 40 && name.split(' ').length <= 4 ? name : '';
}

/** Validates an embedding response vector and returns it as a unit-length Float32Array. */
function toVector(values) {
  if (!Array.isArray(values) || values.length !== EMBED_DIMENSIONS) {
    throw new AiError(`gemini returned ${Array.isArray(values) ? values.length : 'no'} embedding dimensions, expected ${EMBED_DIMENSIONS}`, { service: 'gemini' });
  }
  const vector = Float32Array.from(values);
  const length = Math.hypot(...vector);
  return length > 0 ? vector.map((value) => value / length) : vector;
}

/** True when embed() can build a document vector from the item's own title, text or PNG/JPEG image. */
const hasEmbeddableContent = (item) => Boolean(clip(item.title, 1) || clip(item.text, 1) || embedImage(item));

const geminiHeaders = () => ({ 'x-goog-api-key': requireKey('GEMINI_API_KEY', 'Gemini embeddings and chat') });

/**
 * Embeds one item with gemini-embedding-2 into the shared 768-dimension space.
 * @param {AiItem} item For `kind: 'document'` the card; for `kind: 'query'` only `text` (or `title`) is read and holds the query.
 * @param {{ kind: 'document' | 'query' }} options `document` formats the text as `title: … | text: …` and attaches the image when it is PNG or JPEG; `query` formats `task: search result | query: …` and ignores any image.
 * @returns {Promise<Float32Array>} Unit-length vector of 768 floats; compare with {@link cosine}.
 * @throws {AiError} When GEMINI_API_KEY is missing, there is nothing to embed, or the API fails after retries.
 */
export async function embed(item, { kind } = {}) {
  if (kind !== 'document' && kind !== 'query') throw new TypeError(`embed: kind must be 'document' or 'query', got ${kind}`);
  const parts = [];
  if (kind === 'query') {
    const query = clip(item.text || item.title, EMBED_TEXT_CHARS);
    if (!query) throw new AiError('embed: the query is empty');
    parts.push({ text: `task: search result | query: ${query}` });
  } else {
    const title = clip(item.title, 500);
    const text = clip(item.text, EMBED_TEXT_CHARS);
    const image = embedImage(item);
    if (!title && !text && !image) throw new AiError('embed: the item has no title, text or usable image');
    if (title || text) parts.push({ text: `title: ${title || 'none'} | text: ${text}` });
    if (image) parts.push({ inline_data: { mime_type: image.mime, data: image.base64 } });
  }
  const data = await postJson('gemini', `${geminiBase()}/models/${EMBED_MODEL}:embedContent`, {
    headers: geminiHeaders(),
    body: { model: `models/${EMBED_MODEL}`, content: { parts }, output_dimensionality: EMBED_DIMENSIONS },
  });
  return toVector(data?.embedding?.values);
}

/**
 * Embeds many short texts (tag names) with gemini-embedding-2 batch embedding.
 * @param {string[]} texts Non-empty strings; any number (sent in batches of 100).
 * @param {{ kind?: 'document' | 'query' }} [options] `query` (default) formats each as `task: search result | query: …`, the right side for tag names matched against card embeddings; `document` formats `title: none | text: …`.
 * @returns {Promise<Float32Array[]>} One 768-dimension vector per input, same order.
 * @throws {AiError} When GEMINI_API_KEY is missing, a text is empty, or the API fails after retries.
 */
export async function embedTexts(texts, { kind = 'query' } = {}) {
  if (kind !== 'document' && kind !== 'query') throw new TypeError(`embedTexts: kind must be 'document' or 'query', got ${kind}`);
  const prepared = texts.map((text) => {
    const clean = clip(text, EMBED_TEXT_CHARS);
    if (!clean) throw new AiError('embedTexts: empty text');
    return kind === 'query' ? `task: search result | query: ${clean}` : `title: none | text: ${clean}`;
  });
  const vectors = [];
  for (let start = 0; start < prepared.length; start += EMBED_BATCH_SIZE) {
    const batch = prepared.slice(start, start + EMBED_BATCH_SIZE);
    const data = await postJson('gemini', `${geminiBase()}/models/${EMBED_MODEL}:batchEmbedContents`, {
      headers: geminiHeaders(),
      body: {
        requests: batch.map((text) => ({
          model: `models/${EMBED_MODEL}`,
          content: { parts: [{ text }] },
          output_dimensionality: EMBED_DIMENSIONS,
        })),
      },
    });
    if (!Array.isArray(data?.embeddings) || data.embeddings.length !== batch.length) {
      throw new AiError(`gemini batch returned ${data?.embeddings?.length ?? 0} embeddings for ${batch.length} texts`, { service: 'gemini' });
    }
    for (const entry of data.embeddings) vectors.push(toVector(entry?.values));
  }
  return vectors;
}

/**
 * Cosine similarity of two embeddings.
 * @param {ArrayLike<number>} a First vector.
 * @param {ArrayLike<number>} b Second vector, same length as `a`.
 * @returns {number} Similarity in [-1, 1]; 0 when either vector is all zeros.
 */
export function cosine(a, b) {
  let dot = 0;
  let normA = 0;
  let normB = 0;
  for (let i = 0; i < a.length; i++) {
    dot += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }
  return normA && normB ? dot / Math.sqrt(normA * normB) : 0;
}

/**
 * Asks the cheap chat model (env AI_CHAT_MODEL, default gemini-2.5-flash-lite) for a TL;DR and tag suggestions.
 * @param {AiItem} item The card; the image is sent when it is PNG, JPEG or WebP.
 * @returns {Promise<{ summary: string, tags: string[] }>} One or two plain sentences, and 0-8 distinct lowercase labels.
 * @throws {AiError} When the key is missing, the model returns no usable JSON or summary, or the API fails after retries.
 */
export async function describe(item) {
  const model = chatModel();
  const key = process.env.AI_CHAT_API_KEY || requireKey('GEMINI_API_KEY', 'the chat model');
  const lines = [
    `Title: ${clip(item.title, 500) || '(none)'}`,
    `Source: ${[clip(item.domain, 200), clip(item.url, 500)].filter(Boolean).join(' ') || '(unknown)'}`,
    `Text: ${clip(item.text, CHAT_TEXT_CHARS) || '(none)'}`,
  ];
  const content = [{ type: 'text', text: lines.join('\n') }];
  const image = chatImage(item);
  if (image) content.push({ type: 'image_url', image_url: { url: `data:${image.mime};base64,${image.base64}` } });
  let parsed;
  for (let attempt = 1; !parsed; attempt++) {
    let data;
    try {
      data = await postJson('chat', `${chatBase()}/chat/completions`, {
        headers: { authorization: `Bearer ${key}` },
        body: {
          model,
          temperature: 0.2,
          response_format: { type: 'json_schema', json_schema: { name: 'card', schema: SUMMARY_SCHEMA } },
          messages: [{ role: 'system', content: SUMMARY_PROMPT }, { role: 'user', content }],
        },
      });
    } catch (error) {
      if (error instanceof AiError && (error.status === 404 || error.status === 403)) {
        error.message += ` (model ${model}; set AI_CHAT_MODEL to a model this key can use)`;
      }
      throw error;
    }
    try {
      parsed = JSON.parse(String(data?.choices?.[0]?.message?.content).replace(/^\s*```(?:json)?\s*|\s*```\s*$/g, ''));
    } catch (cause) {
      // Rare malformed output (seen with image inputs) is not repeatable, so one more try usually succeeds.
      if (attempt >= 2) throw new AiError(`chat model ${model} did not return valid JSON`, { service: 'chat', cause });
    }
  }
  const summary = clip(parsed?.summary, 600);
  if (!summary) throw new AiError(`chat model ${model} returned no summary`, { service: 'chat' });
  const tags = [...new Set((Array.isArray(parsed.tags) ? parsed.tags : []).map(normalizeTag).filter(Boolean))].slice(0, MAX_TAGS);
  return { summary, tags };
}

/**
 * Judges candidate tags against an item with TypeSafe Jev, one yes/no question per candidate in a single request.
 * @param {AiItem} item The card; the image is sent only when it is PNG or JPEG, 256-1568 px per side and within Jev's pixel and size limits, otherwise it is left out (never resized).
 * @param {string[]} candidates Candidate tag names; duplicates (case-insensitive) and empty names are dropped, and more than 100 are judged in further requests.
 * @returns {Promise<{ name: string, p: number }[]>} Candidates with probability of fit at least 0.6, highest first, at most 8; `name` keeps the caller's spelling. Empty when the item has no text and no usable image.
 * @throws {AiError} When TYPESAFE_API_KEY is missing or the API fails after retries.
 */
export async function pickTags(item, candidates) {
  const key = requireKey('TYPESAFE_API_KEY', 'tag picking with TypeSafe Jev');
  const unique = new Map();
  for (const candidate of candidates) {
    const name = String(candidate ?? '').trim();
    if (name && !unique.has(name.toLowerCase())) unique.set(name.toLowerCase(), name);
  }
  const names = [...unique.values()];
  if (!names.length) return [];

  const image = jevImage(item);
  const itemState = {
    title: clip(item.title, 500) || undefined,
    source: clip(item.domain, 200) || undefined,
    url: clip(item.url, 500) || undefined,
    text: clip(item.text, JEV_TEXT_CHARS) || undefined,
  };
  const imageRef = image ? `mymind-image-${globalThis.crypto.randomUUID()}` : null;
  if (imageRef) itemState.image = imageRef;
  if (!Object.values(itemState).some(Boolean)) return [];
  const headers = { authorization: `Bearer ${key}` };
  if (image) headers['TypeSafe-Danger-Zone-Images'] = 'true';

  const picked = [];
  for (let start = 0; start < names.length; start += JEV_QUESTIONS_PER_REQUEST) {
    const batch = names.slice(start, start + JEV_QUESTIONS_PER_REQUEST);
    const questions = Object.fromEntries(batch.map((name, i) => [
      `t${i}`,
      { type: 'noul', instructions: `Someone saved \`item\` to their personal library and wants to find it later. Would they file it under the tag ${JSON.stringify(name)}?` },
    ]));
    const data = await postJson('typesafe', `${typesafeBase()}/v1/systemone`, {
      headers,
      body: {
        model: JEV_MODEL,
        state: { item: itemState },
        ...(image && { assets: { [imageRef]: { type: 'image', format: image.format, data_base64: image.base64 } } }),
        questions,
      },
    });
    batch.forEach((name, i) => {
      const answer = data?.answers?.[`t${i}`];
      if (typeof answer?.noul === 'number') picked.push({ name, p: answer.noul });
    });
  }
  return picked.filter((tag) => tag.p >= JEV_MIN_P).sort((a, b) => b.p - a.p).slice(0, MAX_TAGS);
}

/** In-process cache of vocabulary tag embeddings, keyed by lowercase name. */
const vocabularyVectors = new Map();

/**
 * Embeds a tag vocabulary as queries, reusing vectors already computed in this process or supplied by the caller.
 * @param {(string | { name: string, embedding?: ArrayLike<number> })[]} vocabulary Existing tag names; an entry may carry a stored 768-dimension embedding to skip the API call. Duplicates (case-insensitive) keep the first spelling.
 * @returns {Promise<{ name: string, embedding: Float32Array }[]>} One entry per distinct tag, in input order.
 * @throws {AiError} When embeddings are needed and GEMINI_API_KEY is missing or the API fails.
 */
export async function embedVocabulary(vocabulary) {
  const entries = new Map();
  for (const entry of vocabulary) {
    const name = String(typeof entry === 'string' ? entry : entry?.name ?? '').trim();
    const key = name.toLowerCase();
    if (!name || entries.has(key)) continue;
    entries.set(key, name);
    if (typeof entry === 'object' && entry.embedding?.length === EMBED_DIMENSIONS) vocabularyVectors.set(key, Float32Array.from(entry.embedding));
  }
  const missing = [...entries].filter(([key]) => !vocabularyVectors.has(key));
  if (missing.length) {
    const vectors = await embedTexts(missing.map(([, name]) => name), { kind: 'query' });
    missing.forEach(([key], i) => vocabularyVectors.set(key, vectors[i]));
  }
  return [...entries].map(([key, name]) => ({ name, embedding: vocabularyVectors.get(key) }));
}

/**
 * Auto-labels a card: summary, final tags and embedding.
 * The chat model proposes tags, the vocabulary's 20 nearest tags by embedding similarity join them as candidates,
 * and Jev picks the final set; without TYPESAFE_API_KEY the chat model's tags are returned unjudged.
 * An item with no title, no text and no PNG/JPEG image (for example a WebP-only upload) is embedded from its generated summary instead.
 * @param {AiItem} item The card to analyze.
 * @param {{ vocabulary?: (string | { name: string, embedding?: ArrayLike<number> })[] }} [options] `vocabulary` lists the library's existing tag names, most used first; entries may carry stored embeddings (see {@link embedVocabulary}).
 * @returns {Promise<{ summary: string, tags: { name: string, p: number | null, source: 'vocabulary' | 'chat' }[], embedding: Float32Array }>} `summary` is the TL;DR; `tags` has at most 8 entries, `source` is `vocabulary` when the name is an existing library tag (spelled as in the vocabulary) and `chat` for a new one, `p` is Jev's probability or null in the fallback; `embedding` is the document vector.
 * @throws {AiError} When a required key is missing or any hosted API fails after retries.
 */
export async function analyze(item, { vocabulary = [] } = {}) {
  const description = describe(item);
  const [embedding, { summary, tags: proposed }, vocab] = await Promise.all([
    hasEmbeddableContent(item)
      ? embed(item, { kind: 'document' })
      : description.then((result) => embed({ text: result.summary }, { kind: 'document' })),
    description,
    embedVocabulary(vocabulary),
  ]);
  const spelling = new Map(vocab.map((tag) => [tag.name.toLowerCase(), tag.name]));
  const sourceOf = (name) => (spelling.has(name.toLowerCase()) ? 'vocabulary' : 'chat');
  const resolve = (name) => spelling.get(name.toLowerCase()) ?? name;

  if (!process.env.TYPESAFE_API_KEY) {
    return { summary, tags: proposed.map(resolve).map((name) => ({ name, p: null, source: sourceOf(name) })), embedding };
  }

  const nearest = vocab
    .map((tag) => ({ name: tag.name, score: cosine(embedding, tag.embedding) }))
    .sort((a, b) => b.score - a.score)
    .slice(0, NEAREST_TAGS)
    .map((tag) => tag.name);
  const picked = await pickTags(item, [...nearest, ...proposed.map(resolve)]);
  return { summary, tags: picked.map(({ name, p }) => ({ name, p, source: sourceOf(name) })), embedding };
}
