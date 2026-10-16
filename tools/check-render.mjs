import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { renderCard } from '../lib/render-card.mjs';
import { unescapeHtml } from '../lib/render/encoding.mjs';
import { linkRenderCases } from './check-link-lifecycle.mjs';

// Read a quoted attribute without interpreting or executing the recorded markup.
function attr(tag, name) {
  const value = tag?.match(new RegExp(`(?:^|\\s)${name}="([^"]*)"`))?.[1];
  return value === undefined ? undefined : unescapeHtml(value);
}

// Recover media metadata that the card JSON omits, without passing layout markup through.
function coverFromHtml(html) {
  const match = html.match(/<mymind-image\b([^>]*)>([\s\S]*?)<\/mymind-image>/);
  if (!match) return undefined;
  const ratio = attr(match[1], 'style')?.match(/aspect-ratio:(\d+)\/(\d+)/);
  const tag = match[2].match(/<(?:img|video)\b[^>]*>/)?.[0];
  if (!ratio || !tag) return undefined;
  const src = attr(tag, 'data-src');
  const rendition = src?.match(/;(\d+)x(\d+)/);
  return {
    src, width: Number(rendition?.[1] ?? ratio[1]), height: Number(rendition?.[2] ?? ratio[2]),
    dominantColor: attr(match[1], 'style')?.match(/background-color:([^;]+)/)?.[1],
    type: tag.startsWith('<video') ? 'video' : 'image',
    svg: attr(match[1], 'class') === 'svg',
    imageWidth: attr(tag, 'width') === undefined ? undefined : Number(attr(tag, 'width')),
    imageHeight: attr(tag, 'height') === undefined ? undefined : Number(attr(tag, 'height')),
  };
}

/**
 * Recover only source/entity fields missing from a recorded card's JSON.
 * Layout, flags and prose are rendered independently. Blob renditions use data.blob
 * unless the entity selected a different cover; full titles are recovered only when
 * the recorded title proves that JSON clipped a matching prefix at 200 characters.
 * Social bodyHtml is the original rich source content, not a card-template fragment;
 * matching it does not prove that the lossy JSON can recover the original social text.
 * @param {{html:string,json:string}} recording One locally captured card.
 * @returns {object} Metadata accepted by renderCard's documented extras contract.
 */
export function recordingExtras(recording) {
  const data = JSON.parse(recording.json);
  const html = recording.html;
  const extras = { id: attr(html.match(/<mymind-card\b[^>]*>/)?.[0], 'data-id') };
  const linkUrl = attr(html.match(/<a class="link-bubble"[^>]*>/)?.[0], 'href');
  if (linkUrl !== undefined && linkUrl !== data.source?.url) extras.linkUrl = linkUrl;
  if (data.type === 'Article' && data.title?.length === 200) {
    const title = unescapeHtml(html.match(/<h3>([\s\S]*?)<\/h3>/)?.[1] ?? '');
    if (title.startsWith(data.title) && title !== data.title) extras.title = title;
  }
  if (!['Image', 'Video', 'Document', 'Note', 'Content', 'Repository'].includes(data.type)) {
    extras.cover = coverFromHtml(html);
  }
  if (data.type === 'Image') {
    const cover = coverFromHtml(html);
    if (cover.dominantColor !== data.blob.dominantColor || cover.src?.endsWith('-t.webp')) extras.cover = cover;
  }
  if (['XPost', 'ThreadsPost', 'SubstackNote'].includes(data.type)) {
    const body = html.match(/<i class="platform"><\/i>\n\n    ([\s\S]*?)\n\n    /)?.[1];
    extras.post = {
      bodyHtml: body,
      author: unescapeHtml(html.match(/<p class="author">([\s\S]*?)<\/p>/)?.[1] ?? ''),
      hasVideo: html.includes('class="fake-player"') || extras.cover?.type === 'video',
      gallery: [...(html.match(/<div class="gallery">([\s\S]*?)<\/div>\s*<\/div>/)?.[1] ?? '').matchAll(/<img class="lazy" data-(srcset|src)="([^"]*)" width="(\d+)" height="(\d+)"\/>/g)]
        .map(match => ({ [match[1]]: unescapeHtml(match[2]), width: Number(match[3]), height: Number(match[4]) })),
    };
  }
  if (data.type === 'InstagramReel') {
    const image = html.match(/<img class="lazy main-img"[^>]*>/)?.[0];
    extras.cover = { src: attr(image, 'data-src'), width: Number(attr(image, 'width')), height: Number(attr(image, 'height')),
      isThumbnail: !html.includes('instagram-reel not-thumbnail') };
  }
  if (data.type === 'SoftwareApplication') {
    const image = html.match(/<img srcset="[^"]*"[^>]*>/)?.[0];
    extras.app = {
      distributor: html.match(/class="inner distributor_([^"]*)"/)?.[1],
      srcset: attr(image, 'srcset'), width: Number(attr(image, 'width')), height: Number(attr(image, 'height')),
    };
  }
  if (data.type === 'Product') extras.product = { price: unescapeHtml(html.match(/<span class="price">([^<]*)<\/span>/)?.[1] ?? '') };
  if (data.type === 'RedditPost') extras.reddit = {
    title: unescapeHtml(html.match(/<h3>([\s\S]*?)<\/h3>/)?.[1] ?? ''),
    subreddit: unescapeHtml(html.match(/<div class="label">([\s\S]*?)<\/div>/)?.[1] ?? ''),
  };
  return extras;
}

// Parse the two supported flags and reject typos instead of silently checking less.
function options(args) {
  const result = { type: undefined, show: 3 };
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--type' && args[i + 1] && !args[i + 1].startsWith('--')) result.type = args[++i];
    else if (args[i] === '--show' && /^\d+$/.test(args[i + 1] ?? '')) result.show = Number(args[++i]);
    else throw new Error('Usage: node tools/check-render.mjs [--type X] [--show N]');
  }
  return result;
}

// Report the first unequal line, including whitespace, without dumping entire cards.
function difference(expected, actual) {
  const a = expected.split('\n'), b = actual.split('\n');
  const index = Array.from({ length: Math.max(a.length, b.length) }, (_, i) => i).find(i => a[i] !== b[i]);
  return `  line ${index + 1}\n  expected: ${JSON.stringify(a[index])}\n  actual:   ${JSON.stringify(b[index])}`;
}

// Check every selected recording against independently generated template HTML.
function main() {
  const config = options(process.argv.slice(2));
  const cards = JSON.parse(readFileSync(new URL('../capture/cards.json', import.meta.url), 'utf8'));
  const counts = new Map();
  const failures = [];
  for (const [index, recording] of cards.entries()) {
    const data = JSON.parse(recording.json);
    if (config.type && data.type !== config.type) continue;
    if (!counts.has(data.type)) counts.set(data.type, { passed: 0, total: 0 });
    const count = counts.get(data.type);
    count.total++;
    try {
      const html = renderCard(data, recordingExtras(recording));
      if (html === recording.html) count.passed++;
      else failures.push({ index, type: data.type, detail: difference(recording.html, html) });
    } catch (error) {
      failures.push({ index, type: data.type, detail: `  ${error.name}: ${error.message}` });
    }
  }
  if (!config.type) {
    const count = { passed: 0, total: 0 };
    counts.set('Link save lifecycle', count);
    for (const [index, [name, check]] of linkRenderCases().entries()) {
      count.total++;
      try { check(); count.passed++; }
      catch (error) { failures.push({ index, type: name, detail: error.message }); }
    }
  }
  if (!counts.size) throw new Error(`No recorded cards have type ${JSON.stringify(config.type)}`);
  for (const [type, count] of counts) console.log(`${type.padEnd(20)} ${count.passed}/${count.total}`);
  const total = [...counts.values()].reduce((sum, n) => sum + n.total, 0);
  console.log(`Total                ${total - failures.length}/${total}`);
  for (const failure of failures.slice(0, config.show)) console.log(`\n${failure.type} recording #${failure.index + 1}:\n${failure.detail}`);
  process.exitCode = failures.length ? 1 : 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try { main(); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
