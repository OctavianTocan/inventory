import { escapeHtml as e } from './encoding.mjs';

// Find the common divisor used in the server's CSS aspect ratios.
function gcd(a, b) {
  return b ? gcd(b, a % b) : a;
}

/**
 * Fit intrinsic media dimensions into a bounding box without upscaling.
 * @param {{width:number,height:number}} media Intrinsic pixel dimensions.
 * @param {number} width Maximum output width.
 * @param {number} height Maximum output height.
 * @param {boolean} [even=false] Round video dimensions to the nearest even pixel.
 * @returns {{width:number,height:number}} Output pixel dimensions.
 */
export function fit(media, width, height, even = false) {
  const scale = Math.min(1, width / media.width, height / media.height);
  const round = even ? n => Math.round(n / 2) * 2 : n => Math.floor(n + 1e-9);
  return { width: round(media.width * scale), height: round(media.height * scale) };
}

/**
 * Render a prepared cover, whose URL and display dimensions come from media metadata.
 * @param {{src:string,width:number,height:number,dominantColor?:string,type?:string,svg?:boolean,imageWidth?:number,imageHeight?:number}} media Cover metadata; type is image or video, imageWidth/imageHeight describe intrinsic SVG size.
 * @returns {string} A mymind-image element, without a trailing newline.
 */
export function renderMedia(media) {
  if (!media) return '';
  const divisor = gcd(media.width, media.height);
  const style = `aspect-ratio:${media.width / divisor}/${media.height / divisor}${media.dominantColor ? `;background-color:${media.dominantColor}` : ''}`;
  const image = media.type === 'video'
    ? `<video autoplay muted playsinline loop  class="lazy" data-src="${e(media.src)}"></video>`
    : `<img class="lazy" data-src="${e(media.src)}"${media.svg ? ` width="${media.imageWidth ?? media.width}" height="${media.imageHeight ?? media.height}"` : ''}/>`;
  return `<mymind-image style="${e(style)}"${media.svg ? ' class="svg"' : ''}>${image}</mymind-image>`;
}

/**
 * Derive the image cover from an uploaded blob's intrinsic metadata.
 * @param {{path:string,width:number,height:number,type:string,dominantColor?:string}} blob Uploaded image metadata.
 * @returns {object} Cover accepted by renderMedia.
 */
export function imageCover(blob) {
  const size = fit(blob, 800, 1000);
  return { ...size, dominantColor: blob.dominantColor, type: blob.type === 'image/gif' ? 'video' : 'image',
    src: `/media/${blob.path};${size.width}x${size.height}.${blob.type === 'image/gif' ? 'mp4' : 'webp'}` };
}

/**
 * Render the poster and hover preview for an uploaded video.
 * @param {{path:string,width:number,height:number}} blob Uploaded video's intrinsic metadata.
 * @returns {string} A mymind-image element containing both video renditions.
 */
export function renderVideo(blob) {
  const poster = fit(blob, 800, 1000, true);
  const preview = fit(blob, 1280, 1280, true);
  const divisor = gcd(poster.width, poster.height);
  return `<mymind-image style="aspect-ratio:${poster.width / divisor}/${poster.height / divisor}"><img class="lazy" data-src="${e(`/media/${blob.path};${poster.width}x${poster.height}.jpeg`)}" /><video class="hover-preview" playsinline muted loop src="${e(`/media/${blob.path};${preview.width}x${preview.height}/hevc.mp4`)}"></video></mymind-image>`;
}
