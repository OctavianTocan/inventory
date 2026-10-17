/** Content types of the media formats the replica stores and serves. */
export const MIME = { png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', webp: 'image/webp',
  gif: 'image/gif', avif: 'image/avif', svg: 'image/svg+xml', mp4: 'video/mp4', webm: 'video/webm',
  mov: 'video/quicktime', pdf: 'application/pdf' };

/**
 * Fit intrinsic dimensions to the cover bounding box.
 * @param {{path:string,width:number,height:number,dominantColor?:string}} blob Stored original's metadata.
 * @param {number} [maxWidth=800] Box width.
 * @param {number} [maxHeight=1000] Box height.
 * @returns {{src:string,width:number,height:number,dominantColor?:string}} Cover pointing at a WebP rendition.
 */
export function cover(blob, maxWidth = 800, maxHeight = 1000) {
  const scale = Math.min(1, maxWidth / blob.width, maxHeight / blob.height);
  const width = Math.max(1, Math.floor(blob.width * scale));
  const height = Math.max(1, Math.floor(blob.height * scale));
  return { src: `/media/${blob.path};${width}x${height}-t.webp`, width, height, dominantColor: blob.dominantColor };
}

/**
 * Split a media path into its original and the rendition it asks for, using mymind's rendition grammar
 * (`path;WxH[-t|-c].ext`, `path;WxH/hevc.mp4`, optional `background(fff)` and `page(n)`; `path;original.ext` is the
 * original itself).
 * @param {string} relative Path below `/media/`, already decoded.
 * @returns {{source:string, suffix:string|null, width?:number, height?:number, mode?:string, format?:string,
 *   page?:number, flatten?:boolean, type?:string}} The original's path, with a null suffix when the original is
 *   wanted; for a rendition also its size, fit mode (`t`, `c` or undefined), output format, PDF page, white
 *   background and output content type.
 * @throws {Error} Status 400 when the rendition suffix is malformed or out of bounds.
 */
export function parseRendition(relative) {
  const [source, ...suffixes] = relative.split(';');
  if (!suffixes.length) return { source, suffix: null };
  const suffix = suffixes.join(';');
  if (/^original\.[a-z0-9]+$/.test(suffix)) return { source, suffix: null };
  const size = /(?:^|\/)(\d+)x(\d+)(?:-([tc]))?\.(webp|jpe?g|png|mp4)(?:\/hevc\.mp4)?$/.exec(suffix);
  const video = /(?:^|\/)(\d+)x(\d+)\/hevc\.mp4$/.exec(suffix);
  const width = Number((size ?? video)?.[1]), height = Number((size ?? video)?.[2]);
  const page = Number(/page\((\d+)\)/.exec(suffix)?.[1] ?? 1);
  if ((!size && !video) || width < 1 || height < 1 || width > 4096 || height > 4096 || page < 1 || page > 1000) {
    throw Object.assign(new Error('Invalid media rendition'), { status: 400 });
  }
  const format = video ? 'mp4' : size[4];
  return { source, suffix, width, height, mode: size?.[3], format, page, flatten: suffix.includes('background(fff)'),
    type: format === 'mp4' ? 'video/mp4' : MIME[format] };
}

/**
 * Resolve a `Range` header against a body size, as the media routes answer it.
 * @param {string|undefined} range The request's Range header, or undefined for the whole body.
 * @param {number} size Body size in bytes.
 * @returns {{status:200|206|416, start:number, end:number}} The status to send and the inclusive byte span; a 416
 *   carries no meaningful span.
 */
export function byteRange(range, size) {
  if (range === undefined) return { status: 200, start: 0, end: size - 1 };
  const match = /^bytes=(\d*)-(\d*)$/.exec(range);
  const first = match?.[1] ? Number(match[1]) : null;
  const last = match?.[2] ? Number(match[2]) : null;
  const invalid = !match || (first === null && last === null) ||
    [first, last].some(value => value !== null && !Number.isSafeInteger(value));
  const start = first ?? Math.max(0, size - (last ?? 0));
  const end = first !== null && last !== null ? Math.min(last, size - 1) : size - 1;
  if (invalid || start > end || start >= size || (first === null && last === 0)) return { status: 416, start: 0, end: -1 };
  return { status: 206, start, end };
}
