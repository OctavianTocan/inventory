import { randomBytes } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { mkdir, readdir, readFile, rename, rm, stat, writeFile } from 'node:fs/promises';
import { dirname, join, relative as relativePath, resolve, sep } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';

const HEADERS = '.headers/';

/**
 * Open a directory as an object store with the operations the hosted server uses on Cloudflare R2
 * (`worker/bucket-r2.mjs`): keys are `/`-separated paths below `root`.
 * @param {string} root Directory holding the objects; created on the first write.
 * @returns {{head: Function, read: Function, bytes: Function, write: Function, remove: Function, list: Function}}
 *   `head(key)` resolves `{size, headers}` or null, where `headers` are the response headers stored with the object
 *   (or undefined); `read(key, range?)` resolves `{size, headers, body}` with a web stream of the whole object or of
 *   `range` (`{offset, length}`), or null; `bytes(key)` resolves a Buffer or null; `write(key, body, length?,
 *   headers?)` stores a Buffer, string, web stream or async iterable atomically, with its response headers;
 *   `remove(key)` deletes one object; `list(prefix)` resolves every `{key, size}` whose key starts with `prefix`.
 * @throws {Error} Status 400 from any operation given a key that leaves `root`.
 */
export function directoryBucket(root) {
  const base = resolve(root);
  const file = (key) => {
    const target = resolve(base, key);
    if (!target.startsWith(base + sep)) throw Object.assign(new Error('Invalid object key'), { status: 400 });
    return target;
  };
  const missing = (error) => { if (error.code !== 'ENOENT' && error.code !== 'ENOTDIR' && error.code !== 'EISDIR') throw error; return null; };
  // Stored headers sit in a parallel tree, as R2 keeps them in object metadata, so listings never see them.
  const headersFile = (key) => { file(key); return file(`${HEADERS}${key}.json`); };
  const head = async (key) => {
    const info = await stat(file(key)).catch(missing);
    if (!info?.isFile()) return null;
    const headers = await readFile(headersFile(key), 'utf8').catch(missing);
    return { size: info.size, headers: headers ? JSON.parse(headers) : undefined };
  };
  return {
    head,
    async read(key, range) {
      const info = await head(key);
      if (!info) return null;
      const options = range ? { start: range.offset, end: range.offset + range.length - 1 } : {};
      if (range && range.length <= 0) return { ...info, body: new Blob([]).stream() };
      return { ...info, body: Readable.toWeb(createReadStream(file(key), options)) };
    },
    async bytes(key) {
      return readFile(file(key)).catch(missing);
    },
    async write(key, body, length, headers) {
      const target = file(key);
      await mkdir(dirname(target), { recursive: true });
      const temp = `${target}.${randomBytes(6).toString('hex')}.tmp`;
      try {
        if (typeof body === 'string' || body instanceof Uint8Array) await writeFile(temp, body);
        else await pipeline(body instanceof ReadableStream ? Readable.fromWeb(body) : Readable.from(body), createWriteStream(temp));
        await rename(temp, target);
      } finally { await rm(temp, { force: true }); }
      if (!headers) return rm(headersFile(key), { force: true });
      await mkdir(dirname(headersFile(key)), { recursive: true });
      await writeFile(headersFile(key), JSON.stringify(headers));
    },
    async remove(key) {
      await rm(file(key), { force: true });
      await rm(headersFile(key), { force: true });
    },
    async list(prefix) {
      const folder = prefix.includes('/') ? prefix.slice(0, prefix.lastIndexOf('/')) : '';
      const directory = folder ? file(folder) : base;
      const names = await readdir(directory, { recursive: true, withFileTypes: true }).catch(missing) ?? [];
      const objects = [];
      for (const entry of names) {
        if (!entry.isFile() || entry.name.endsWith('.tmp')) continue;
        const key = relativePath(base, join(entry.parentPath, entry.name)).split(sep).join('/');
        if (key.startsWith(prefix) && !key.startsWith(HEADERS)) objects.push({ key, size: (await stat(join(entry.parentPath, entry.name))).size });
      }
      return objects.sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    },
  };
}
