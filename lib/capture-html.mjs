import { escapeHtml } from './render/encoding.mjs';
import { decodeEntities as decoded } from './capture-entities.mjs';

// Parse attributes without executing third-party markup or accepting duplicate overrides.
function attributes(tag) {
  const result = Object.create(null);
  const source = tag.replace(/^<\/?[\w:-]+/, '').replace(/\/?\s*>$/, '');
  for (const match of source.matchAll(/([^\s=<>"'`]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s<>`]+)))?/g)) {
    const name = match[1].toLowerCase();
    if (!Object.hasOwn(result, name)) result[name] = decoded(match[2] ?? match[3] ?? match[4] ?? '');
  }
  return result;
}

/**
 * Resolve a scraped HTTP(S) URL, excluding credentials and the live account origin.
 * @param {string|object} value Relative/absolute URL or schema.org image object.
 * @param {string} base Page URL used for relative resolution.
 * @returns {string|undefined} Absolute URL, preserving numeric host syntax for transport policy validation.
 */
export function sourceUrl(value, base) {
  try {
    const raw = typeof value === 'object' ? value?.url ?? value?.contentUrl : value;
    const url = new URL(raw, base);
    if (!value || !['http:', 'https:'].includes(url.protocol) || url.username || url.password || url.hostname.replace(/\.$/, '') === 'access.mymind.com') return;
    const authority = /^(?:https?:)?\/\/([^/?#]+)/i.exec(String(raw))?.[1]?.replace(/:\d+$/, '').toLowerCase();
    if (/^\d+\.\d+\.\d+\.\d+$/.test(url.hostname) && /^(?:https?:|[\\/]{2})/i.test(String(raw)) && authority !== url.hostname) return String(raw).startsWith('//') ? url.protocol + raw : String(raw);
    return url.href;
  } catch { return undefined; }
}

/**
 * Extract readable text from untrusted HTML without preserving executable markup.
 * @param {string} html Third-party HTML fragment.
 * @returns {string} Whitespace-normalized decoded text.
 */
export function plainText(html) {
  return decoded(String(html ?? '').replace(/<(script|style)\b[^>]*>[\s\S]*?<\/\1\s*>/gi, '')
    .replace(/<[^>]*>/g, ' ')).replace(/\s+/g, ' ').trim();
}

/**
 * Rebuild source post HTML using recorded paragraph, break and text-formatting elements only.
 * Scripts, frames, images, styles and all source attributes are discarded; safe links retain href.
 * @param {string} html Untrusted oEmbed/source body, never a card layout template.
 * @param {string} base Page URL for relative links.
 * @returns {string} Sanitized HTML safe for extras.post.bodyHtml.
 */
export function sanitizePost(html, base) {
  const allowed = new Set(['p', 'br', 'span', 'strong', 'a']);
  const stack = [];
  let result = '';
  let clean = String(html ?? '').replace(/<!--[\s\S]*?-->/g, '').replace(/<(script|style|iframe|object|svg|math|cite|footer)\b[^>]*>[\s\S]*?<\/\1\s*>/gi, '');
  const quote = /<blockquote\b[^>]*>([\s\S]*?)<\/blockquote\s*>/i.exec(clean)?.[1];
  if (quote) clean = quote.match(/<p\b[^>]*>[\s\S]*?<\/p\s*>/gi)?.join('') ?? quote;
  for (const token of clean.match(/<[^>]*>|[^<]+|</g) ?? []) {
    if (!token.startsWith('<') || token === '<') { result += escapeHtml(decoded(token)); continue; }
    const match = /^<(\/?)\s*([\w:-]+)/.exec(token);
    const name = match?.[2].toLowerCase();
    if (!allowed.has(name)) continue;
    if (match[1]) {
      const index = stack.lastIndexOf(name);
      if (index >= 0) while (stack.length > index) result += `</${stack.pop()}>`;
    } else if (name === 'br') result += '<br/>';
    else {
      const href = name === 'a' ? sourceUrl(attributes(token).href, base) : undefined;
      result += `<${name}${href ? ` href="${escapeHtml(href)}"` : ''}>`;
      stack.push(name);
    }
  }
  while (stack.length) result += `</${stack.pop()}>`;
  return result;
}

/**
 * Read OpenGraph, Twitter, JSON-LD and oEmbed discovery metadata from fetched HTML.
 * @param {string} html Bounded page response, never executed.
 * @param {string} url Final fetched page URL.
 * @returns {object} Normalized metadata and typed schema.org entities.
 */
export function pageMetadata(html, url) {
  const meta = Object.create(null);
  for (const match of html.matchAll(/<meta\b(?:[^>"']|"[^"]*"|'[^']*')*>/gi)) {
    const attrs = attributes(match[0]);
    const key = (attrs.property ?? attrs.name ?? '').toLowerCase();
    if (attrs.content) (meta[key] ??= []).push(attrs.content);
  }
  const entities = [];
  // JSON-LD graphs and arrays are flattened because publishers use both interchangeably.
  function collect(value, depth = 0) {
    if (!value || typeof value !== 'object' || depth > 12) return;
    if (value['@type']) entities.push(value);
    for (const child of Object.values(value)) if (typeof child === 'object') {
      if (Array.isArray(child)) child.forEach(item => collect(item, depth + 1));
      else collect(child, depth + 1);
    }
  }
  for (const match of html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script\s*>/gi)) {
    if (attributes(`<script ${match[1]}>`).type?.toLowerCase() !== 'application/ld+json') continue;
    try { collect(JSON.parse(match[2])); } catch {}
  }
  let canonical, oembed;
  for (const match of html.matchAll(/<link\b(?:[^>"']|"[^"]*"|'[^']*')*>/gi)) {
    const attrs = attributes(match[0]);
    if (attrs.rel?.split(/\s+/).includes('canonical')) canonical = sourceUrl(attrs.href, url);
    if (attrs.type?.toLowerCase() === 'application/json+oembed') oembed = sourceUrl(attrs.href, url);
  }
  const first = key => meta[key]?.[0];
  const entity = entities.find(item => [item['@type']].flat().some(type => /(?:Article|BlogPosting|Product|SoftwareApplication)$/.test(type))) ?? entities.find(item => item.headline || item.name) ?? {};
  const images = [...(meta['og:image:secure_url'] ?? []), ...(meta['og:image'] ?? []), ...(meta['twitter:image'] ?? meta['twitter:image:src'] ?? []), ...[entity.image ?? entity.thumbnailUrl].flat().filter(Boolean)]
    .map(image => sourceUrl(image, url)).filter(Boolean);
  const main = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/i)?.[1] ?? html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i)?.[1] ?? html;
  const readable = main.replace(/<(nav|header|footer|aside|script|style)\b[^>]*>[\s\S]*?<\/\1\s*>/gi, '');
  const paragraphs = readable.match(/<p\b[^>]*>[\s\S]*?<\/p\s*>/gi)?.map(plainText).filter(Boolean).join('\n') ?? plainText(readable);
  const description = first('og:description') ?? first('twitter:description') ?? first('description') ?? entity.description ?? '';
  const authors = [...entities.filter(item => [item['@type']].flat().includes('Person')), ...[entity.author].flat().filter(item => typeof item === 'object' && item)];
  return { meta, entities, canonical: canonical ?? sourceUrl(first('og:url'), url) ?? url, oembed,
    title: first('og:title') ?? first('twitter:title') ?? entity.headline ?? entity.name ?? plainText(html.match(/<title\b[^>]*>([\s\S]*?)<\/title>/i)?.[1]),
    description,
    siteName: first('og:site_name') ?? new URL(url).hostname,
    author: first('author') ?? first('article:author') ?? entity.author?.name ?? entity.author,
    text: [...new Set([description, plainText(entity.articleBody ?? entity.text ?? ''), paragraphs].filter(Boolean))].join('\n').slice(0, 30000),
    authorImages: authors.flatMap(author => [author.image].flat()).map(image => sourceUrl(image, url)).filter(Boolean),
    images: [...new Set(images)] };
}

/**
 * Match recorded entity titles by removing publisher/store boilerplate for supported types.
 * @param {string} title Scraped title, already entity-decoded.
 * @param {string} type Detected card type.
 * @param {object} metadata Page metadata with canonical URL, siteName and optional entities.
 * @returns {string} Repository/app name or a title without a matching trailing site name.
 */
export function captureTitle(title, type, metadata) {
  const url = new URL(metadata.canonical);
  if (type === 'Repository') return decodeURIComponent(url.pathname.split('/').filter(Boolean).at(-1)).replace(/\.git$/, '');
  if (type === 'SoftwareApplication') return String(metadata.entities.find(item => [item['@type']].flat().includes('SoftwareApplication'))?.name ?? title)
    .replace(/\s*(?:-\s*Apps on Google Play|on the App Store)\s*$/i, '').trim();
  if (!['Article', 'WebPage'].includes(type)) return title;
  const parts = url.hostname.replace(/^www\./, '').split('.');
  const brand = ['co', 'com', 'org', 'net'].includes(parts.at(-2)) && parts.at(-1).length === 2 ? parts.at(-3) : parts.at(-2);
  const key = value => String(value ?? '').toLowerCase().replace(/[^\p{L}\p{N}]/gu, '');
  const suffix = /\s+(?:-|\||—)\s+([^|]+)$/.exec(title);
  if (suffix && [metadata.siteName, url.hostname, brand].some(site => key(site) && key(site) === key(suffix[1]))) return title.slice(0, suffix.index).trim();
  return title;
}

/**
 * Exclude known profile-image endpoints from post cover and gallery candidates.
 * @param {string} value Absolute candidate image URL.
 * @returns {boolean} True for avatar/profile-image URLs rather than post media.
 */
export function profileImage(value) {
  const url = new URL(value);
  let path = url.pathname;
  try { path = decodeURIComponent(path); } catch {}
  return /(?:^|[\/_.-])(?:profile(?:[_-]?(?:images?|pictures?|photos?|pics?|banners?))?|avatars?)(?:[\/_.-]|$)/i.test(path)
    || /^(?:avatars?\.|(?:www\.)?unavatar\.)/i.test(url.hostname)
    || (/googleusercontent\.com$/.test(url.hostname) && /^\/a(?:\/|-)/.test(path))
    || /\/t51\.\d+-19\//.test(path);
}

/**
 * Detect the renderer type from a source URL, response MIME and publisher metadata.
 * @param {string} value Saved or canonical entity URL.
 * @param {object} metadata Parsed pageMetadata result.
 * @param {string} [mime=''] HTTP Content-Type.
 * @returns {string} Supported renderCard type.
 */
export function detectType(value, metadata = {}, mime = '') {
  const url = new URL(value);
  const host = url.hostname.toLowerCase().replace(/^www\./, '');
  const path = url.pathname;
  if (mime.includes('application/pdf') || /\.pdf$/i.test(path)) return 'Document';
  if (['x.com', 'twitter.com', 'mobile.twitter.com'].includes(host) && /\/status\/\d+/.test(path)) return 'XPost';
  if (['threads.net', 'threads.com'].includes(host) && /\/post\//.test(path)) return 'ThreadsPost';
  if ((host === 'substack.com' || host.endsWith('.substack.com')) && /\/(?:note|notes)\//.test(path)) return 'SubstackNote';
  if (['youtube.com', 'm.youtube.com', 'youtu.be'].includes(host)) return 'YouTubeVideo';
  if (host === 'instagram.com' && /\/reels?\//.test(path)) return 'InstagramReel';
  if (host === 'github.com' && /^\/[^/]+\/[^/]+\/?$/.test(path)) return 'Repository';
  if ((host === 'reddit.com' || host.endsWith('.reddit.com')) && /\/comments\//.test(path)) return 'RedditPost';
  if (host === 'apps.apple.com' || host === 'play.google.com') return 'SoftwareApplication';
  if (metadata.entities?.some(item => [item['@type']].flat().some(type => /(?:^|\/)Product$/.test(type)))) return 'Product';
  if (metadata.meta?.['og:type']?.includes('article')) return 'Article';
  if (metadata.entities?.some(item => [item['@type']].flat().some(type => /(?:Article|BlogPosting)$/.test(type)))) return 'Article';
  return 'WebPage';
}
