import { fit } from './media.mjs';

const VOID = /^(img|br|source|input|meta|link|hr|wbr)$/i;

// Find the common divisor used in the server's CSS aspect ratios.
function gcd(a, b) {
  return b ? gcd(b, a % b) : a;
}

// Open elements enclosing `offset`, outermost first, as tag names and class attributes.
function ancestors(html, offset) {
  const stack = [];
  for (const tag of html.slice(0, offset).matchAll(/<(\/?)([\w-]+)([^>]*)>/g)) {
    if (tag[1]) {
      const open = stack.map((element) => element.name).lastIndexOf(tag[2]);
      if (open >= 0) stack.splice(open);
    } else if (!VOID.test(tag[2]) && !tag[3].endsWith('/')) {
      stack.push({ name: tag[2], className: /class="([^"]*)"/.exec(tag[3])?.[1] ?? '' });
    }
  }
  return stack;
}

/**
 * Convert a card's grid HTML to the version mymind serves an iPhone, whose cover renditions fit a 400x500 box
 * (400x300 for YouTube) instead of the desktop's.
 * @param {string} html Card HTML as rendered for desktop browsers.
 * @param {string} json The card's data JSON; its type and video blob size decide the box and rounding.
 * @returns {string} The iPhone card HTML; post bodies, reels and other images are unchanged.
 */
export function iphoneCard(html, json) {
  const data = JSON.parse(json);
  const box = data.type === 'YouTubeVideo' ? [400, 300] : [400, 500];
  return html.replace(/<mymind-image\b([^>]*)>([\s\S]*?)<\/mymind-image>/g, (whole, attributes, inner, offset) => {
    const [card, ...rest] = ancestors(html, offset);
    if (card?.name !== 'mymind-card' || !rest.every((element) => element.name === 'div' && element.className.trim() === 'inner')) return whole;
    let ratio = null;
    const body = inner.replace(/;(\d+)x(\d+)(-[tc])?\.(webp|jpe?g|png|mp4)/g, (token, width, height, mode, extension) => {
      const video = extension === 'jpeg' || extension === 'mp4';
      const source = video && data.blob?.width ? data.blob : { width: Number(width), height: Number(height) };
      const size = fit(source, ...box, video);
      if (size.width === Number(width) && size.height === Number(height)) return token;
      const divisor = gcd(size.width, size.height);
      ratio = `${size.width / divisor}/${size.height / divisor}`;
      return `;${size.width}x${size.height}${mode ?? ''}.${extension}`;
    });
    const head = ratio ? attributes.replace(/aspect-ratio:\d+\/\d+/, `aspect-ratio:${ratio}`) : attributes;
    return `<mymind-image${head}>${body}</mymind-image>`;
  });
}
