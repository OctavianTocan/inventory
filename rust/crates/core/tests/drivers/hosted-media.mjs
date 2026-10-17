import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const ROOT = process.env.MYMIND_ROOT;
const load = (file) => import(pathToFileURL(path.join(ROOT, file)).href);
const { createHostedMedia, bucketUploads } = await load('lib/hosted-media.mjs');
const { directoryBucket } = await load('lib/bucket-fs.mjs');

const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');

async function collect(body) {
  if (body instanceof Uint8Array) return Buffer.from(body);
  const chunks = [];
  for await (const chunk of body) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks);
}

const extension = (type) => ({ 'image/png': 'png', 'application/pdf': 'pdf', 'video/mp4': 'mp4' })[type] ?? 'bin';

function failure(error) {
  return { name: error?.constructor?.name ?? 'Error', message: String(error?.message ?? error), status: error?.status ?? null };
}

/** The scripted processor of tests/hosted_media.rs, logging every call. */
function fakeProcessor(log) {
  return {
    async prepare(body, type, name, user, id) {
      const bytes = await collect(body);
      log.push(['prepare', { length: bytes.length, sha: sha(bytes), type, name, user, id }]);
      if (name === 'fail-plain') throw new Error('boom');
      if (name === 'fail-415') throw Object.assign(new Error('Unsupported media'), { status: 415 });
      if (name === 'noblob') return { data: { type: 'Image' }, extras: {} };
      if (name === 'nopath') return { data: { type: 'Image', blob: {} }, extras: {} };
      const file = name === 'travpath' ? '../escape.bin' : name === 'escape' ? '../../../../escape.bin' : `${user.accountUid}/${id.slice(user.idPrefix.length)}/${sha(bytes).slice(0, 8)}.${extension(type)}`;
      return { data: { type: 'Image', title: name, blob: { id, path: file, type, size: bytes.length, name } }, extras: { cover: { src: `/media/${file};800x450-t.webp`, width: 800, height: 450 } } };
    },
    async rendition(original, relative) {
      const bytes = await collect(original.body);
      log.push(['rendition', { name: original.name, size: original.size, sha: sha(bytes), relative }]);
      if (relative.includes('null')) return null;
      if (relative.includes('boom')) throw new Error('rendition boom');
      if (relative.includes('teapot')) throw Object.assign(new Error('teapot'), { status: 418 });
      const body = Buffer.from(`R:${relative}:${sha(bytes)}`);
      if (relative.includes('nosize')) return { body };
      if (relative.includes('streamy')) return { body: new Blob([body]).stream() };
      if (relative.includes('liar')) return { body, size: 3 };
      return { body, size: body.length };
    },
    async image(body) {
      const bytes = await collect(body);
      log.push(['image', { length: bytes.length, sha: sha(bytes) }]);
      return { mime: 'image/png', data: Buffer.from(`I:${sha(bytes)}`) };
    },
  };
}

function recording(bucket, log) {
  return {
    head: (key) => { log.push(['head', key]); return bucket.head(key); },
    read: (key, range) => { log.push(['read', key, range ? [range.offset, range.length] : null]); return bucket.read(key, range); },
    bytes: (key) => { log.push(['bytes', key]); return bucket.bytes(key); },
    write: (key, body, length, headers) => { log.push(['write', key, length ?? null, Boolean(headers)]); return bucket.write(key, body, length, headers); },
    remove: (key) => { log.push(['remove', key]); return bucket.remove(key); },
    list: (prefix) => { log.push(['list', prefix]); return bucket.list(prefix); },
  };
}

async function body(input) {
  if (input.bytesFile) return readFile(path.join(ROOT, input.bytesFile));
  if (input.bytesText !== undefined) return Buffer.from(input.bytesText);
  if (input.key !== undefined) return { key: input.key };
  if (input.upload !== undefined) return { upload: input.upload };
  return { file: '/nowhere' };
}

async function send(hosted, res, result, step) {
  await hosted.send(res, result, { method: step.method, range: step.range ?? undefined });
  const { status, headers } = res.out;
  const bytes = res.out.done ? await res.out.done : Buffer.alloc(0);
  return { status, headers, length: bytes.length, sha: sha(bytes) };
}

/** Run a scenario: seed a directory store, then run each step, recording the processor and store calls it caused. */
export async function scenario(spec) {
  const root = await mkdtemp(path.join(tmpdir(), 'hosted-media-'));
  try {
    const store = directoryBucket(root);
    for (const object of spec.objects) {
      const data = object.file ? await readFile(path.join(ROOT, object.file)) : Buffer.from(object.text ?? '');
      await store.write(object.key, data);
    }
    const storeLog = [];
    const processorLog = [];
    const bucket = recording(store, storeLog);
    const hosted = createHostedMedia({ bucket, prefix: spec.prefix, processor: fakeProcessor(processorLog) });
    const uploads = bucketUploads(bucket, spec.prefix);
    const outputs = [];
    for (const step of spec.steps) {
      storeLog.length = 0;
      processorLog.length = 0;
      const res = { out: {}, writeHead(status, headers) { this.out.status = status; this.out.headers = Object.entries(headers); }, end(data) { if (data) this.out.done = collect(data); } };
      let output;
      try {
        let ok;
        if (step.op === 'response') ok = (await hosted.response(step.relative)) ?? null;
        else if (step.op === 'send') {
          const result = await hosted.response(step.relative);
          ok = result ? await send(hosted, res, result, step) : null;
        } else if (step.op === 'sendResult') ok = await send(hosted, res, step.result, step);
        else if (step.op === 'prepare') ok = await hosted.prepare(await body(step.input), step.type, step.name, step.user, step.id);
        else if (step.op === 'image') {
          const image = await hosted.image(step.arg);
          ok = image ? { mime: image.mime, data: Buffer.from(image.data).toString() } : null;
        } else if (step.op === 'cover') ok = hosted.cover(step.blob);
        else if (step.op === 'concurrent') ok = await Promise.all(step.relatives.map((relative) => hosted.response(relative).then((result) => result ?? null)));
        else if (step.op === 'upload') {
          if (step.fn === 'write') ok = (await uploads.write(step.id, step.start, Buffer.from(step.text), step.text.length)) ?? null;
          else if (step.fn === 'key') ok = uploads.key;
          else if (step.fn === 'input') ok = uploads.input(step.id);
          else ok = (await uploads[step.fn](...step.args)) ?? null;
        } else throw new Error(`unknown step ${step.op}`);
        output = { ok };
      } catch (error) {
        output = { error: failure(error) };
      }
      output.processor = processorLog.slice();
      output.bucket = step.unordered ? storeLog.map((entry) => JSON.stringify(entry)).sort() : storeLog.slice();
      outputs.push(output);
    }
    return outputs;
  } finally { await rm(root, { recursive: true, force: true }); }
}
