import { escapeHtml as e, escapeProse } from './render/encoding.mjs';
import { imageCover, renderMedia, renderVideo } from './render/media.mjs';
import { renderProse } from './render/prose.mjs';

const classes = {
  XPost: 'x-post post', ThreadsPost: 'threads-post post', SubstackNote: 'substack-note post',
  WebPage: 'web-page', Article: 'article', Repository: 'repository', Image: 'image',
  Screenshot: 'screenshot', Video: 'video', YouTubeVideo: 'youtube-video video',
  InstagramReel: 'instagram-reel', Note: 'note content', Content: 'content',
  SoftwareApplication: 'software-application', Product: 'product', Document: 'document',
  RedditPost: 'reddit-post', Placeholder: 'placeholder unavailable',
};

// Choose the type-specific label shown in the external-link bubble.
function linkLabel(data) {
  return { XPost: 'X Post', ThreadsPost: 'Threads Post', SubstackNote: 'Substack Note',
    RedditPost: 'Reddit Post', InstagramReel: 'Reel', SoftwareApplication: 'App',
    YouTubeVideo: 'YouTube Video', Article: 'Article', Repository: 'Repository',
    Placeholder: 'Placeholder' }[data.type] ?? data.siteName ?? data.domain ?? '';
}

// Build the common inaccessible-site illustration and message.
function inaccessible(data, placeholder) {
  const message = `It looks like ${e(data.siteName ?? data.domain)} is blocking us from displaying information.`;
  const guts = `<div class="inaccessible-guts">\n${placeholder ? '  ' : '    '}<img src="https://static.accelerator.net/134/0.109.0/icons/sadness.svg" width="30" height="30"/>\n${placeholder ? '  ' : '    '}<span class="big">Oh my mind</span>\n`;
  return placeholder
    ? `\n\n\n${guts}  \n  \n\n  \n  <span class="tiny">${message}</span>\n  \n</div>\n\n`
    : `\n\n\n\n\n  \n\n  \n  ${guts}    <span class="tiny">${message}</span>\n  </div>\n  \n\n\n\n\n\n`;
}

// Render the social platform's original rich text and media inside its card frame.
function renderPost(data, extras) {
  const post = extras.post ?? {};
  const platform = { XPost: 'x', ThreadsPost: 'threads', SubstackNote: 'substack' }[data.type];
  const body = post.bodyHtml ?? `<p>${(data.description ?? '').split(/\r\n|\r|\n/).map(escapeProse).join('<br/>')}</p>`;
  const gallery = post.gallery ?? [];
  const cover = extras.cover;
  const media = cover ? `\n    <p>${renderMedia(cover)}</p>\n    ` : '';
  const galleryHtml = gallery.length ? `\n    <div class="gallery">\n      <div class="gallery-wrap gallery-grid ${gallery.length === 3 ? 'gallery-grid--three' : ''}">\n        ${gallery.map(image => `\n        <img class="lazy" ${image.srcset ? `data-srcset="${e(image.srcset)}"` : `data-src="${e(image.src)}"`} width="${image.width}" height="${image.height}"/>\n        `).join('')}\n      </div>\n    </div>\n    ` : '';
  const player = post.hasVideo && cover?.type !== 'video' ? `      \n    <div class="fake-player">\n      <div class="fp-button">\n        <span class="fp-play"></span>\n      </div>\n    </div>    ` : '';
  const author = post.author ?? data.author;
  return `<div class="inner">\n  \n\n  \n  <div class="body ${platform}">\n    <i class="platform"></i>\n\n    ${body}\n\n    ${media}\n\n    ${galleryHtml}\n\n    ${player}${player ? '\n    ' : ''}\n    \n    \n${author ? `    <p class="author">${e(author)}</p>\n    \n` : ''}  </div>\n  \n</div>\n`;
}

// Select a card's content template; only absent source metadata comes from extras.
function renderBody(data, extras) {
  if (data.type === 'Placeholder' && extras.capturePending && !extras.captureFailed) return `\n<div class="processing-guts">\n  <img class="icon" src="https://static.accelerator.net/134/0.109.0/icons/head_spin.png" width="30" height="30"/>\n\n  <span class="big">One moment.</span>\n  <span class="big">I'm saving this for you.</span>\n</div>\n\n\n\n`;
  if (data.type === 'WebPage' && extras.screenshotPending) return `\n\n\n\n\n  \n  <div class="processing-guts">\n   <div class="spinner"></div>\n   <span class="text">Taking screenshot…</span>\n  </div>\n  \n\n  \n\n\n\n\n\n`;
  if (extras.capturePending && !extras.captureFailed) return `<div class="processing-guts"><div class="spinner"></div><div class="text">${e(data.domain ?? '')}</div></div>`;
  switch (data.type) {
    case 'Repository':
      return `<div class="inner github">\n  <i class="icon"></i>\n  <div class="wrap">    \n    <p>${e(data.title)}</p>\n    <div class="label">${e(data.siteName)}</div>    \n  </div>\n</div>\n`;
    case 'Image':
      return `${renderMedia(extras.cover ?? imageCover(data.blob))}\n`;
    case 'Screenshot':
    case 'YouTubeVideo':
      return `${renderMedia(extras.cover)}\n`;
    case 'Video':
      return `${renderVideo(data.blob)}\n`;
    case 'WebPage':
      return extras.cover ? `\n\n\n${renderMedia(extras.cover)}\n\n\n\n\n\n\n\n` : inaccessible(data, false);
    case 'Article':
      return `<div class="inner">\n  <div class="article-card-top">\n    <div class="logo">\n      \n\n      <span class="default-logo"></span>\n    </div>\n\n    <h3>${e(extras.title ?? (data.title || 'Untitled'))}</h3>\n  </div>\n\n  ${renderMedia(extras.cover)}\n</div>\n`;
    case 'Note':
      return `\n\n${renderProse(data.prose ?? { type: 'doc' })}`;
    case 'Content':
      return renderProse(data.prose ?? { type: 'doc' });
    case 'Document':
      return `\n\n<div class="document-background">\n  <mymind-image style="aspect-ratio:642/840">\n    <img class="lazy" data-src="${e(`/media/${data.blob.path};background(fff)/page(1)/642x840-t.webp`)}" width="642" height="840"/>\n  </mymind-image>\n</div>\n\n\n\n\n`;
    case 'InstagramReel':
      return `<div class="reel instagram-reel ${(extras.cover?.isThumbnail ?? !(data.flags & 8388608)) ? '' : 'not-thumbnail'}">\n  <i class="icon"></i>\n  <div class="body">\n    \n    <img class="lazy main-img" data-src="${e(extras.cover?.src)}" width="580" height="580"/>\n    <div class="icon-play"></div>\n    \n    \n  </div>\n</div>\n`;
    case 'SoftwareApplication': {
      const app = extras.app ?? {};
      return `<div class="inner distributor_${e(app.distributor)}"> <!-- app-store or play-store classes -->\n  <img srcset="${String(app.srcset ?? '').replace(/"/g, '&quot;')}" width="${app.width ?? 250}" height="${app.height ?? 250}"/>\n</div>\n`;
    }
    case 'Product':
      return `\n${extras.product?.price ? `<span class="price">${e(extras.product.price)}</span>\n` : ''}\n\n${renderMedia(extras.cover)}\n\n\n`;
    case 'Placeholder':
      return inaccessible(data, true);
    case 'RedditPost':
      return `<div class="reddit inner">\n  <i class="icon"></i>\n  <div class="body">\n    <h3>${e(extras.reddit?.title ?? data.title)}</h3>\n    <div class="label">${e(extras.reddit?.subreddit)}</div>\n  </div>\n</div>\n`;
    case 'XPost':
    case 'ThreadsPost':
    case 'SubstackNote':
      return renderPost(data, extras);
    default:
      throw new TypeError(`Unsupported card type: ${data.type}`);
  }
}

/**
 * Render one grid card without reading recordings, accessing the network, or mutating inputs.
 * Values are plain, unescaped strings unless explicitly described as trusted HTML.
 * The card JSON omits its ID and most scraped entity/media fields, so persist these
 * extras alongside new cards when collecting or importing them:
 * - id: object ID (or provide data.id), required because captured JSON has no ID.
 * - linkUrl: canonical entity URL, which can differ from the saved source URL through
 *   tracking removal, normalization or redirects. Defaults to data.source.url.
 * - title: full entity title when JSON clipped data.title to 200 characters.
 * - cover: {src,width,height,dominantColor?,type?:'image'|'video',svg?:boolean}.
 *   src is the prepared media rendition URL; dimensions describe that rendition.
 *   Required for scraped covers and screenshots; uploaded Image/Video use data.blob.
 *   imageWidth/imageHeight supply SVG intrinsic size; isThumbnail identifies an
 *   Instagram thumbnail versus a reel poster. An explicit cover overrides blob media.
 * - post: {bodyHtml?,author?,gallery?:[{srcset?|src?,width,height}],hasVideo?:boolean}.
 *   bodyHtml is trusted, sanitized source-post HTML, absent from the lossy description;
 *   no card layout HTML belongs here. Without it, description becomes an escaped paragraph.
 *   hasVideo records a source video whose playable rendition may be unavailable.
 * - app: {distributor?,srcset,width,height}, missing store ID and prepared icon renditions.
 *   Signed CDN URLs must be supplied by the caller; this renderer cannot sign them.
 * - product: {price}, the formatted offer price absent from the captured product JSON.
 * - reddit: {title,subreddit}, the original headline and community absent from JSON.
 * - capturePending/captureFailed: local capture status; a pending Placeholder uses the saving template.
 * - screenshotPending: fetched WebPage screenshot template, retained after a textless screenshot failure.
 * @param {object} data Parsed card JSON, with a supported type and that type's content fields.
 * @param {object} [extras={}] Missing entity metadata described above; never a recorded card template.
 * @returns {string} Server-style grid HTML, with exact template whitespace and no trailing newline.
 * @throws {TypeError} If the card type is unsupported or the object ID is missing.
 */
export function renderCard(data, extras = {}) {
  const id = extras.id ?? data.id;
  if (!id) throw new TypeError('renderCard requires extras.id or data.id');
  if (!Object.hasOwn(classes, data.type)) throw new TypeError(`Unsupported card type: ${data.type}`);
  const deleted = Boolean(data.deleted);
  const highlighted = data.tags?.some(tag => tag.name === 'highlight');
  const processing = extras.screenshotPending || (extras.capturePending && !extras.captureFailed);
  const baseClass = data.type === 'Placeholder' && processing ? 'placeholder' : classes[data.type];
  const className = baseClass + (deleted ? ' deleted' : '') + (highlighted ? ' highlighted' : '') + (processing ? ' processing' : '');
  const url = extras.linkUrl ?? data.source?.url ?? '';
  const linked = url && !['Image', 'Screenshot', 'Video', 'Note', 'Content', 'Document'].includes(data.type);
  const bubble = linked && !deleted ? `<a class="link-bubble" href="${e(url)}" on-click="card:link" target="_blank"><span>${e(linkLabel(data))}</span></a>` : '';
  const title = extras.title ?? (data.title || (data.type === 'WebPage' ? url : ''));
  const showTitle = Boolean(title) && (deleted || !['Article', 'RedditPost', 'Placeholder'].includes(data.type));
  const titleHtml = showTitle ? `  <p class="title">${linked ? `<a href="${e(url)}" target="_blank">${e(title)}</a>` : e(title)}</p>\n  \n` : '';
  return `<mymind-card class="${className}" data-id="${e(id)}">\n  <div class="hover" on-click="card:${deleted ? 'restore' : 'open'}">${bubble}</div>\n  ${renderBody(data, extras)}\n  \n${titleHtml}</mymind-card>`;
}
