import { randomBytes } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, writeFile, readdir, rename, stat, copyFile, open, rm } from 'node:fs/promises';
import { createReadStream } from 'node:fs';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { resolve, dirname, join, extname } from 'node:path';
import sharp from 'sharp';
import { MIME, byteRange, cover, parseRendition } from './media-files.mjs';

const exec = promisify(execFile);
const MAX_PIXELS = 80_000_000;

// Resolve only descendants of the local media directory, including decoded paths.
function inside(root, relative) {
  const file = resolve(root, relative);
  return file.startsWith(resolve(root) + '/') ? file : null;
}

/**
 * Downscale an image to the PNG the AI analysis reads.
 * @param {string|Buffer} input Image file path or bytes.
 * @returns {Promise<{mime:string, data:Buffer}>} PNG at most 1400 pixels on each side.
 */
export async function analysisImage(input) {
  return { mime: 'image/png', data: await sharp(input, { limitInputPixels: MAX_PIXELS }).rotate().resize(1400, 1400, { fit: 'inside', withoutEnlargement: true }).png().toBuffer() };
}

/**
 * Create local original/preview storage and on-demand rendition serving.
 * Image decoding uses sharp; PDF pages require pdftoppm, video metadata/posters require ffprobe/ffmpeg.
 * @param {string} root Isolated data/media root, outside capture/media.
 * @param {object} [options] Local processor overrides.
 * @param {Function} [options.run] Async execFile-compatible processor runner; defaults to host executables.
 * @returns {object} prepare, response, send, image and cover operations; no network access.
 */
export function createMedia(root, { run = exec } = {}) {
  const pending = new Map();

  // Read metadata from validated image bytes and derive the dominant sRGB color.
  async function imageInfo(input) {
    const image = sharp(input, { limitInputPixels: MAX_PIXELS }).rotate();
    const info = await image.metadata();
    const swapped = info.orientation >= 5 && info.orientation <= 8;
    const { dominant } = await image.clone().stats();
    return { width: swapped ? info.height : info.width, height: swapped ? info.width : info.height,
      dominantColor: '#' + [dominant.r, dominant.g, dominant.b].map(n => n.toString(16).padStart(2, '0')).join('') };
  }

  // Publish previews atomically so parallel renditions never see an unfinished image.
  async function preview(file, type, page = 1) {
    const target = `${file}.page-${page}.png`;
    try { if ((await stat(target)).isFile()) return target; } catch {}
    const temp = `${target}.${randomBytes(6).toString('hex')}.png`;
    try {
      if (type === 'application/pdf') {
        await run('pdftoppm', ['-f', String(page), '-l', String(page), '-singlefile', '-scale-to', '1600', '-png', file, temp.slice(0, -4)], { timeout: 30000, maxBuffer: 1024 * 1024 });
      } else {
        await run('ffmpeg', ['-v', 'error', '-y', '-i', file, '-frames:v', '1', '-vf', 'scale=1600:1600:force_original_aspect_ratio=decrease', temp], { timeout: 30000, maxBuffer: 1024 * 1024 });
      }
      await rename(temp, target);
      return target;
    } finally { await rm(temp, { force: true }); }
  }

  // Identify the saved bytes and prepare dimensions and a usable image cover.
  async function prepare(input, type, name, user, id) {
    let header, length;
    const source = Buffer.isBuffer(input) ? input : input.file;
    if (Buffer.isBuffer(source)) { header = source.subarray(0, 16); length = source.length; }
    else {
      const handle = await open(source, 'r');
      try {
        length = (await handle.stat()).size;
        header = Buffer.alloc(16);
        const result = await handle.read(header, 0, header.length, 0);
        header = header.subarray(0, result.bytesRead);
      } finally { await handle.close(); }
    }
    let kind, extension;
    if (header.subarray(0, 5).toString() === '%PDF-') { kind = 'Document'; type = 'application/pdf'; extension = 'pdf'; }
    else if (header.toString('latin1', 4, 8) === 'ftyp' || header.subarray(0, 4).equals(Buffer.from([0x1a, 0x45, 0xdf, 0xa3]))) {
      kind = 'Video'; extension = header.toString('latin1', 4, 8) === 'ftyp' ? 'mp4' : 'webm'; type = MIME[extension];
    } else {
      let metadata;
      try { metadata = await sharp(source, { limitInputPixels: MAX_PIXELS }).metadata(); }
      catch { throw Object.assign(new Error('Unsupported or invalid media'), { status: 415 }); }
      kind = 'Image'; extension = metadata.format === 'jpeg' ? 'jpg' : metadata.format;
      if (!MIME[extension] || extension === 'svg') throw Object.assign(new Error('Unsupported image format'), { status: 415 });
      type = MIME[extension];
    }
    const blobId = randomBytes(11).toString('hex');
    const relative = `${user.accountUid}/${id.slice(user.idPrefix.length)}/${blobId}.${extension}`;
    const file = inside(root, relative);
    await mkdir(dirname(file), { recursive: true });
    if (Buffer.isBuffer(source)) await writeFile(file, source);
    else await copyFile(source, file);
    const blob = { id: blobId, path: relative, type, size: length, name: String(name ?? '').slice(0, 255) };
    if (kind === 'Image') Object.assign(blob, await imageInfo(file));
    else {
      if (kind === 'Video') {
        const { stdout } = await run('ffprobe', ['-v', 'error', '-select_streams', 'v:0', '-show_entries', 'stream=width,height:format=duration', '-of', 'json', file], { timeout: 15000, maxBuffer: 1024 * 1024 });
        const metadata = JSON.parse(stdout);
        Object.assign(blob, metadata.streams?.[0], { duration: Number(metadata.format?.duration) || 0 });
      }
      const image = await imageInfo(await preview(file, type));
      if (kind === 'Document') Object.assign(blob, image);
      else blob.dominantColor = image.dominantColor;
    }
    if (!(blob.width > 0 && blob.height > 0)) throw Object.assign(new Error('Media has no usable dimensions'), { status: 415 });
    return { data: { type: kind, title: blob.name, blob, ...(kind === 'Document' ? { format: 'PDF' } : {}) }, extras: { cover: cover(blob) } };
  }

  // Describe originals without loading their bytes, including extensionless recorded blob URLs.
  async function original(relative) {
    const exact = inside(root, relative);
    if (!exact) return null;
    try {
      const info = await stat(exact);
      if (info.isFile()) return { file: exact, size: info.size };
    } catch {}
    const names = await readdir(dirname(exact)).catch(() => []);
    const name = names.find(name => name.startsWith(relative.split('/').at(-1) + '.') && !name.includes('.page-'));
    if (!name) return null;
    const file = join(dirname(exact), name);
    const info = await stat(file);
    return info.isFile() ? { file, size: info.size } : null;
  }

  // Decode a bounded rendition grammar shared by recorded cards and the local renderer.
  async function rendition(relative) {
    // A path with no local original belongs to a recorded card, whatever its suffix; the replay answers it.
    const found = await original(relative.split(';')[0]);
    if (!found) return null;
    const wanted = parseRendition(relative);
    const { width, height, page } = wanted;
    const type = MIME[extname(found.file).slice(1)] ?? 'application/octet-stream';
    if (!wanted.suffix) return { ...found, type };
    const cache = inside(root, `.renditions/${relative.replaceAll('/', '_')}`);
    const outputType = wanted.type;
    try {
      const info = await stat(cache);
      if (info.isFile()) return { file: cache, size: info.size, type: outputType };
    } catch {}
    await mkdir(dirname(cache), { recursive: true });
    const temp = `${cache}.${randomBytes(6).toString('hex')}.tmp`;
    try {
      if (outputType === 'video/mp4') {
        await run('ffmpeg', ['-v', 'error', '-y', '-i', found.file, '-vf', `scale=${width}:${height}:force_original_aspect_ratio=decrease`, '-an', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', '-f', 'mp4', temp], { timeout: 60000, maxBuffer: 1024 * 1024 });
      } else {
        const input = type === 'application/pdf' || type.startsWith('video/') ? await preview(found.file, type, page) : found.file;
        let image = sharp(input, { limitInputPixels: MAX_PIXELS }).rotate();
        if (wanted.flatten) image = image.flatten({ background: '#fff' });
        await image.resize(width, height, { fit: wanted.mode === 'c' ? 'cover' : 'inside' })
          .toFormat(wanted.format === 'jpg' ? 'jpeg' : wanted.format).toFile(temp);
      }
      await rename(temp, cache);
      return { file: cache, size: (await stat(cache)).size, type: outputType };
    } finally { await rm(temp, { force: true }); }
  }

  // Stream only the requested bytes; HEAD never opens a file stream.
  async function send(res, result, { method = 'GET', range } = {}) {
    const { file, size, type } = result;
    const headers = { 'cache-control': 'no-store', 'content-type': type, 'accept-ranges': 'bytes' };
    const { status, start, end } = byteRange(range, size);
    if (status === 416) {
      res.writeHead(416, { ...headers, 'content-range': `bytes */${size}`, 'content-length': '0' });
      res.end();
      return;
    }
    if (status === 206) headers['content-range'] = `bytes ${start}-${end}/${size}`;
    headers['content-length'] = String(Math.max(0, end - start + 1));
    res.writeHead(status, headers);
    if (method === 'HEAD' || size === 0) { res.end(); return; }
    const stream = createReadStream(file, { start, end });
    if (typeof res.write !== 'function') res.end(Readable.toWeb(stream));
    else {
      try { await pipeline(stream, res); }
      catch (error) { if (!res.destroyed) res.destroy(error); }
    }
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
    send,
    async response(relative) {
      if (!pending.has(relative)) pending.set(relative, rendition(relative).catch(error => {
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
      return analysisImage(result.file);
    },
  };
}
