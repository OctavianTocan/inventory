import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { MIME, byteRange, cover, parseRendition } from './media-files.mjs';

// Reject keys that are absolute or climb out of their prefix.
function safe(relative) {
  return !relative.startsWith('/') && !relative.split('/').some((part) => part === '..' || part === '.');
}

/**
 * Send a web stream as the rest of a response whose head is written.
 * @param {object} res Node response, or an adapter without `write` whose `end` takes a web stream.
 * @param {ReadableStream} body Response body.
 * @returns {Promise<void>} Resolves when the body has been handed over.
 */
export async function sendStream(res, body) {
  if (typeof res.write !== 'function') return res.end(body);
  try { await pipeline(Readable.fromWeb(body), res); }
  catch (error) { if (!res.destroyed) res.destroy(error); }
}

// One stream of several stored objects, read in order.
function concat(bucket, keys) {
  return new ReadableStream({
    async start(controller) {
      try {
        for (const key of keys) {
          const object = await bucket.read(key);
          if (!object) throw new Error('Upload part is missing');
          for await (const chunk of object.body) controller.enqueue(chunk);
        }
        controller.close();
      } catch (error) { controller.error(error); }
    },
  });
}

/**
 * Store a library's media in an object store and serve it with the operations of `createMedia`
 * (`lib/local-media.mjs`), leaving the image and video work to a processor.
 * @param {object} options
 * @param {object} options.bucket Object store (`directoryBucket` or `worker/bucket-r2.mjs`).
 * @param {string} [options.prefix=''] Key prefix of this library, such as `libraries/owner/`; originals live below
 *   `<prefix>media/` and cached renditions below `<prefix>renditions/`.
 * @param {object} options.processor Media processor with `prepare`, `rendition` and `image` (`createProcessor`, or
 *   the media container's client).
 * @returns {object} `prepare`, `response`, `send`, `image` and `cover`, as `createMedia` returns; `response` results
 *   name an object `key` instead of a file.
 */
export function createHostedMedia({ bucket, prefix = '', processor }) {
  const pending = new Map();
  const mediaKey = (relative) => `${prefix}media/${relative}`;

  // The stored original for a media path, including extensionless recorded blob URLs.
  async function original(source) {
    if (!safe(source)) return null;
    const type = (key) => MIME[key.split('/').at(-1).split('.').at(-1)] ?? 'application/octet-stream';
    const exact = await bucket.head(mediaKey(source));
    if (exact) return { key: mediaKey(source), size: exact.size, type: type(source) };
    const folder = mediaKey(source.slice(0, source.lastIndexOf('/') + 1));
    const found = (await bucket.list(`${mediaKey(source)}.`))
      .find((object) => !object.key.slice(folder.length).includes('/') && !object.key.includes('.page-'));
    return found ? { key: found.key, size: found.size, type: type(found.key) } : null;
  }

  // The original, or a rendition made once and then kept beside the library's media.
  async function rendition(relative) {
    // A path with no original in this library belongs to a recorded card, whatever its suffix; the site answers it.
    const found = await original(relative.split(';')[0]);
    if (!found) return null;
    const wanted = parseRendition(relative);
    if (!wanted.suffix) return found;
    const key = `${prefix}renditions/${relative.replaceAll('/', '_')}`;
    const cached = await bucket.head(key);
    if (cached) return { key, size: cached.size, type: wanted.type };
    const object = await bucket.read(found.key);
    const made = await processor.rendition({ body: object.body, size: object.size, name: found.key.slice(mediaKey('').length) }, relative);
    if (!made) return null;
    await bucket.write(key, made.body, made.size ?? made.body.length);
    return { key, size: (await bucket.head(key)).size, type: wanted.type };
  }

  // A body and its size for each kind of input `prepare` accepts.
  async function source(input) {
    if (input instanceof Uint8Array) return { size: input.length, open: () => input };
    if (input.key) {
      const info = await bucket.head(input.key);
      if (!info) throw Object.assign(new Error('Upload media is unavailable'), { status: 404 });
      return { size: info.size, open: async () => (await bucket.read(input.key)).body };
    }
    if (input.upload) {
      const parts = (await bucket.list(input.upload)).sort((a, b) => Number(a.key.slice(input.upload.length)) - Number(b.key.slice(input.upload.length)));
      return { size: parts.reduce((sum, part) => sum + part.size, 0), open: () => concat(bucket, parts.map((part) => part.key)) };
    }
    throw Object.assign(new Error('Unsupported media input'), { status: 415 });
  }

  // Identify the saved bytes, keep the original and return the card media fields.
  async function prepare(input, type, name, user, id) {
    const bytes = await source(input);
    const prepared = await processor.prepare(await bytes.open(), type, name, user, id);
    await bucket.write(mediaKey(prepared.data.blob.path), await bytes.open(), bytes.size);
    return prepared;
  }

  return {
    async prepare(...args) {
      try { return await prepare(...args); }
      catch (error) {
        if (error.status) throw error;
        throw Object.assign(new Error('Media preview could not be generated'), { status: 422 });
      }
    },
    cover,
    async send(res, result, { method = 'GET', range } = {}) {
      const { key, size, type } = result;
      const headers = { 'cache-control': 'no-store', 'content-type': type, 'accept-ranges': 'bytes' };
      const { status, start, end } = byteRange(range, size);
      if (status === 416) {
        res.writeHead(416, { ...headers, 'content-range': `bytes */${size}`, 'content-length': '0' });
        res.end();
        return;
      }
      if (status === 206) headers['content-range'] = `bytes ${start}-${end}/${size}`;
      headers['content-length'] = String(Math.max(0, end - start + 1));
      const object = method === 'HEAD' || size === 0 ? null : await bucket.read(key, { offset: start, length: end - start + 1 });
      res.writeHead(status, headers);
      if (!object) { res.end(); return; }
      await sendStream(res, object.body);
    },
    async response(relative) {
      if (!pending.has(relative)) pending.set(relative, rendition(relative).catch((error) => {
        if (error.status) throw error;
        throw Object.assign(new Error('Media rendition unavailable'), { status: 422 });
      }).finally(() => pending.delete(relative)));
      return pending.get(relative);
    },
    async image(blobOrCover) {
      if (!blobOrCover) return undefined;
      const relative = blobOrCover.src?.replace(/^\/media\//, '') ?? `${blobOrCover.path};800x1000-t.webp`;
      const result = await this.response(relative);
      if (!result) return undefined;
      return processor.image((await bucket.read(result.key)).body);
    },
  };
}

/**
 * Keep chunked uploads in an object store, one object per chunk, with the chunk storage operations
 * `createUploads` (`lib/uploads.mjs`) expects.
 * @param {object} bucket Object store (`directoryBucket` or `worker/bucket-r2.mjs`).
 * @param {string} [prefix=''] Key prefix of this library; chunks live below `<prefix>uploads/<id>/<offset>`.
 * @returns {object} Chunk storage for `createUploads`; its `input(id)` is what `createHostedMedia().prepare` reads.
 */
export function bucketUploads(bucket, prefix = '') {
  const folder = `${prefix}uploads/`;
  const parts = (id) => bucket.list(`${folder}${id}/`);
  return {
    key: `bucket:${prefix}`,
    async names() { return [...new Set((await bucket.list(folder)).map((object) => object.key.slice(folder.length).split('/')[0]))]; },
    async size(id) {
      const found = await parts(id);
      return found.length ? found.reduce((sum, part) => sum + part.size, 0) : undefined;
    },
    async truncate(id, size) {
      for (const part of await parts(id)) if (Number(part.key.split('/').at(-1)) >= size) await bucket.remove(part.key);
    },
    async remove(id) {
      for (const part of await parts(id)) await bucket.remove(part.key);
    },
    input: (id) => ({ upload: `${folder}${id}/` }),
    write: (id, start, chunks, length) => bucket.write(`${folder}${id}/${String(start).padStart(12, '0')}`, chunks, length),
  };
}
