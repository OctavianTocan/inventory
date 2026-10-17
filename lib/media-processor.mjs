import { createWriteStream } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve, sep } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { analysisImage, createMedia } from './local-media.mjs';

// Write bytes, a web stream or an async iterable of chunks to a new file.
async function save(file, body) {
  await mkdir(dirname(file), { recursive: true });
  if (body instanceof Uint8Array) return writeFile(file, body);
  await pipeline(body instanceof ReadableStream ? Readable.fromWeb(body) : Readable.from(body), createWriteStream(file));
}

/**
 * Process media whose bytes are stored elsewhere (R2 or a bucket directory) with the local processors of
 * `lib/local-media.mjs` (sharp, ffmpeg, pdftoppm), each call in its own temporary directory. The hosted media
 * (`lib/hosted-media.mjs`) uses it in-process on Node and through the media container on Cloudflare.
 * @param {object} [options]
 * @param {string} [options.directory=os.tmpdir()] Parent of the per-call temporary directories.
 * @param {Function} [options.run] Processor runner, as for `createMedia`.
 * @returns {{prepare: Function, rendition: Function, image: Function}} `prepare(body, type, name, user, id)` resolves
 *   what `createMedia().prepare` does, whose `data.blob.path` is where the caller stores the original;
 *   `rendition({body, name}, relative)` resolves `{body}`, the rendition `relative` names of the original stored at
 *   `name`, or null when `relative` is not a rendition of it; `image(body)` resolves the PNG the AI analysis reads.
 *   Bodies are bytes, web streams or async iterables.
 */
export function createProcessor({ directory = tmpdir(), run } = {}) {
  const local = (root) => createMedia(root, run ? { run } : undefined);
  // Run `task` in a fresh directory that is removed afterwards.
  async function within(task) {
    const folder = await mkdtemp(join(directory, 'mymind-media-'));
    try { return await task(folder); } finally { await rm(folder, { recursive: true, force: true }); }
  }
  return {
    prepare: (body, type, name, user, id) => within(async (folder) => {
      await save(join(folder, 'input'), body);
      return local(join(folder, 'media')).prepare({ file: join(folder, 'input') }, type, name, user, id);
    }),
    rendition: (original, relative) => within(async (folder) => {
      const root = join(folder, 'media');
      const target = resolve(root, original.name);
      if (!target.startsWith(root + sep)) throw Object.assign(new Error('Invalid media path'), { status: 400 });
      await save(target, original.body);
      const result = await local(root).response(relative);
      return result ? { body: await readFile(result.file) } : null;
    }),
    image: (body) => within(async (folder) => {
      await save(join(folder, 'input'), body);
      return analysisImage(join(folder, 'input'));
    }),
  };
}
