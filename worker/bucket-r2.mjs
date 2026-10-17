// The response headers stored with an object, kept in its custom metadata.
const storedHeaders = (object) => (object.customMetadata?.headers ? JSON.parse(object.customMetadata.headers) : undefined);

// A web stream of the chunks of an async iterable, remembering the iterable's own error so the caller can rethrow it.
function streamOf(iterable, failure) {
  const iterator = iterable[Symbol.asyncIterator]();
  return new ReadableStream({
    async pull(controller) {
      try {
        const { done, value } = await iterator.next();
        if (done) controller.close();
        else controller.enqueue(typeof value === 'string' ? new TextEncoder().encode(value) : value);
      } catch (error) {
        failure.error = error;
        controller.error(error);
      }
    },
    cancel: (reason) => iterator.return?.(reason),
  });
}

/**
 * Open an R2 bucket as the object store the hosted server uses, with the operations of `directoryBucket`
 * (`lib/bucket-fs.mjs`).
 * @param {R2Bucket} bucket The R2 binding.
 * @returns {{head: Function, read: Function, bytes: Function, write: Function, remove: Function, list: Function}}
 *   `head(key)` resolves `{size, headers}` or null, where `headers` are the response headers stored with the object
 *   (or undefined); `read(key, range?)` resolves `{size, headers, body}` with a web stream of the whole object or of
 *   `range` (`{offset, length}`), or null; `bytes(key)` resolves a Buffer or null; `write(key, body, length?,
 *   headers?)` stores a Buffer, string, web stream or async iterable, with its response headers; `remove(key)`
 *   deletes one object; `list(prefix)` resolves every `{key, size}` whose key starts with `prefix`, in key order.
 */
export function r2Bucket(bucket) {
  return {
    async head(key) {
      const object = await bucket.head(key);
      return object ? { size: object.size, headers: storedHeaders(object) } : null;
    },
    async read(key, range) {
      if (range && range.length <= 0) {
        const object = await bucket.head(key);
        return object ? { size: object.size, headers: storedHeaders(object), body: new Blob([]).stream() } : null;
      }
      const object = await bucket.get(key, range ? { range: { offset: range.offset, length: range.length } } : undefined);
      return object ? { size: object.size, headers: storedHeaders(object), body: object.body } : null;
    },
    async bytes(key) {
      const object = await bucket.get(key);
      return object ? Buffer.from(await object.arrayBuffer()) : null;
    },
    async write(key, body, length, headers) {
      const options = headers ? { customMetadata: { headers: JSON.stringify(headers) } } : {};
      if (typeof body === 'string' || body instanceof Uint8Array) return void await bucket.put(key, body, options);
      const failure = {};
      const stream = body instanceof ReadableStream ? body : streamOf(body, failure);
      // R2 needs the length of a streamed body up front.
      const sized = length === undefined ? stream : stream.pipeThrough(new FixedLengthStream(length));
      try { await bucket.put(key, sized, options); }
      catch (error) { throw failure.error ?? error; }
      if (failure.error) throw failure.error;
    },
    async remove(key) {
      await bucket.delete(key);
    },
    async list(prefix) {
      const objects = [];
      let cursor;
      do {
        const page = await bucket.list({ prefix, cursor });
        for (const object of page.objects) objects.push({ key: object.key, size: object.size });
        cursor = page.truncated ? page.cursor : undefined;
      } while (cursor);
      return objects.sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    },
  };
}
