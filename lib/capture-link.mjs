import { chromium } from 'playwright-core';
import { pageMetadata, sourceUrl, detectType, sanitizePost, plainText, captureTitle, profileImage } from './capture-html.mjs';
import { captureDestination, fetchResource } from './capture-http.mjs';
export { fetchResource } from './capture-http.mjs';

/**
 * Fulfill a screenshot resource only after checking every redirect destination.
 * Playwright's ordinary route.continue() does not re-intercept redirected requests.
 * @param {object} route Playwright Route, or a compatible offline fixture double.
 * @param {object} [options] Screenshot network policy.
 * @param {boolean} [options.offline=false] Permit loopback resources only.
 * @param {object} [options.budget] Shared per-screenshot remaining byte budget.
 * @param {Function} [options.onBlocked] Notify the screenshot owner of a rejected destination.
 * @param {Function} [options.resource=fetchResource] Validated transport; offline tests may inject a stub.
 * @returns {Promise<void>} Fulfilled bounded response or an aborted forbidden/failed resource.
 */
export async function screenshotResource(route, { offline = false, budget = { remaining: 100 * 1024 * 1024 }, onBlocked, resource = fetchResource } = {}) {
  try {
    if (route.request().method() !== 'GET') return await route.abort();
    const response = await resource(route.request().url(), { offline, limit: 25 * 1024 * 1024, budget });
    await route.fulfill({ status: response.status, headers: response.headers, body: response.bytes });
  } catch (error) {
    if (error.code === 'CAPTURE_BLOCKED') onBlocked?.(error);
    await route.abort().catch(() => {});
  }
}

/**
 * Capture a page in a fresh headless context using the verifier's Chromium executable.
 * No authentication state, user-agent override, stealth options or non-GET requests are used.
 * @param {string} url Safe public page URL.
 * @param {object} [options] Network policy.
 * @param {boolean} [options.offline=false] Restrict all browser resources to loopback.
 * @param {Function} [options.resource=fetchResource] Validated HTTP transport, injectable for offline fixtures.
 * @returns {Promise<Buffer>} PNG screenshot of the 1400x940 viewport.
 */
export async function screenshotPage(url, { offline = false, resource = fetchResource } = {}) {
  captureDestination(url, offline);
  // Native preconnect, WebRTC and QUIC bypass Playwright routing, so only the Node transport may reach sources.
  const browser = await chromium.launch({ executablePath: process.env.VERIFIER_CHROMIUM ?? '/usr/lib/chromium/chromium', headless: true,
    proxy: { server: 'http://capture-denied.invalid:9' }, args: ['--proxy-bypass-list=<-loopback>', '--disable-quic',
      '--force-webrtc-ip-handling-policy=disable_non_proxied_udp', '--host-resolver-rules=MAP * ~NOTFOUND'] });
  try {
    const context = await browser.newContext({ viewport: { width: 1400, height: 940 }, serviceWorkers: 'block' });
    // The WebRTC policy switch still lets STUN reach loopback in this Chromium, so remove the APIs in every frame.
    await context.addInitScript(() => {
      for (const name of ['RTCPeerConnection', 'webkitRTCPeerConnection', 'RTCDataChannel', 'WebTransport']) {
        try { Object.defineProperty(globalThis, name, { value: undefined, configurable: false, writable: false }); } catch {}
      }
    });
    await context.routeWebSocket('**/*', socket => socket.close());
    const budget = { remaining: 100 * 1024 * 1024 };
    let blocked;
    await context.route('**/*', route => screenshotResource(route, { offline, resource, budget, onBlocked: error => { blocked = error; } }));
    const page = await context.newPage();
    try { await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 20000 }); }
    catch (error) { throw blocked ?? error; }
    await Promise.race([page.evaluate(() => document.fonts.ready), new Promise(resolve => setTimeout(resolve, 2000))]).catch(() => {});
    if (blocked) throw blocked;
    const bytes = await page.screenshot({ type: 'png', timeout: 15000 });
    if (blocked) throw blocked;
    return bytes;
  } finally { await browser.close(); }
}

// Select public platform oEmbed endpoints when the page has no discovery link.
function platformEmbed(url, type) {
  const endpoint = { XPost: 'https://publish.twitter.com/oembed', YouTubeVideo: 'https://www.youtube.com/oembed',
    RedditPost: 'https://www.reddit.com/oembed', InstagramReel: 'https://www.instagram.com/api/v1/oembed/' }[type];
  if (!endpoint) return undefined;
  const result = new URL(endpoint);
  result.searchParams.set('url', url);
  result.searchParams.set('format', 'json');
  result.searchParams.set('omit_script', 'true');
  return result.href;
}

/**
 * Enrich a committed URL placeholder in independently committable metadata and media stages.
 * Textless WebPage screenshot failures retain the recorded processing card; earlier failures use the readable fallback.
 * @param {object} card Saved card JSON with source.url and optional requested Screenshot type.
 * @param {object} options Capture dependencies, separate from the synchronous writer.
 * @param {string} options.id Card ID used for local media layout.
 * @param {object} options.user Store user with accountUid and idPrefix.
 * @param {object} options.media Local createMedia handle.
 * @param {(patch:object) => void} options.update Commit a data/extras patch in a fresh transaction.
 * @param {boolean} [options.offline=false] Allow loopback fixture reads only.
 * @param {Function} [options.screenshot=screenshotPage] Page screenshot implementation.
 * @param {Function} [options.resource=fetchResource] Validated transport, injectable for offline fixtures.
 * @returns {Promise<void>} Resolves after capture stages, including persisted failure state.
 */
export async function captureLink(card, { id, user, media, update, offline = false, screenshot = screenshotPage, resource = fetchResource }) {
  const url = card.source.url;
  try {
    const response = await resource(url, { offline, limit: 64 * 1024 * 1024 });
    if (detectType(response.url, {}, response.mime) === 'Document') {
      const prepared = await media.prepare(response.bytes, 'application/pdf', card.title === new URL(url).host ? decodeURIComponent(new URL(url).pathname.split('/').at(-1)) : card.title, user, id);
      update(prepared);
      return;
    }
    const binary = !/^\s*(?:$|text\/|application\/(?:xhtml\+xml|xml|json|ld\+json|pdf))/i.test(response.mime)
      || /\.apk$/i.test(new URL(response.url).pathname);
    const metadata = pageMetadata(binary ? '' : response.bytes.toString('utf8'), response.url);
    const type = card.type === 'Screenshot' ? 'Screenshot' : detectType(metadata.canonical, metadata, response.mime);
    let embed = {};
    const embedUrl = metadata.oembed ?? platformEmbed(metadata.canonical, type);
    if (embedUrl) {
      try { embed = JSON.parse((await resource(embedUrl, { offline, limit: 1024 * 1024 })).bytes); }
      catch (error) { if (error.code === 'CAPTURE_BLOCKED') throw error; }
    }
    const post = ['XPost', 'ThreadsPost', 'SubstackNote'].includes(type);
    const bodyHtml = post ? sanitizePost(embed.html || `<p>${metadata.description.replaceAll('&', '&amp;').replaceAll('<', '&lt;')}</p>`, response.url) : '';
    const title = captureTitle(String(metadata.title || embed.title || card.title || new URL(url).host), type, metadata);
    const description = String(metadata.description || plainText(bodyHtml) || plainText(embed.html) || '');
    const displayName = String(embed.author_name ?? (typeof metadata.author === 'object' ? metadata.author?.name : metadata.author) ?? '');
    const author = post && displayName ? `by ${displayName.replace(/^by\s+/i, '')}` : displayName;
    const data = { type, ...(binary ? {} : { title: title.slice(0, 200) }), description, domain: new URL(metadata.canonical).hostname,
      siteName: binary ? new URL(metadata.canonical).hostname : metadata.siteName, ...(author ? { author } : {}) };
    const extras = { ...(binary ? {} : { title }), linkUrl: metadata.canonical, captureText: post ? plainText(bodyHtml) : metadata.text, ...(post ? { capturedTitle: title } : {}) };
    if (type === 'Article') data.prose = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: metadata.text || description }] }] };
    if (post) {
      extras.post = { bodyHtml, author, hasVideo: embed.type === 'video', gallery: [] };
    }
    if (type === 'RedditPost') extras.reddit = { title: embed.title ?? title, subreddit: /\/r\/([^/]+)/.exec(metadata.canonical)?.[1] ?? '' };
    if (type === 'Product') {
      const product = metadata.entities.find(item => [item['@type']].flat().some(type => /(?:^|\/)Product$/.test(type))) ?? {};
      const offer = [product.offers].flat()[0] ?? {};
      extras.product = { price: offer.price !== undefined ? [offer.priceCurrency, offer.price].filter(value => value !== undefined).join(' ') : '' };
      if (product.brand) data.brand = product.brand.name ?? product.brand;
    }
    if (type === 'SoftwareApplication') extras.app = { distributor: new URL(metadata.canonical).hostname === 'apps.apple.com' ? 'app-store' : 'play-store', srcset: '', width: 250, height: 250 };
    const authorImages = [...metadata.authorImages, ...[embed.author_avatar_url, embed.author_image_url, embed.avatar_url].map(image => sourceUrl(image, response.url)).filter(Boolean)];
    const images = [...new Set([...metadata.images, sourceUrl(embed.thumbnail_url, response.url)].filter(Boolean))].filter(image => !post || (!profileImage(image) && !authorImages.includes(image)));
    if (type === 'WebPage' && !images.length) extras.screenshotPending = true;
    update({ data, extras });
    let prepared;
    for (const image of images.slice(0, 6)) {
      try {
        const imageResource = await resource(image, { offline, limit: 25 * 1024 * 1024 });
        prepared = await media.prepare(imageResource.bytes, imageResource.mime, 'cover', user, id);
        if (prepared.data.type !== 'Image') { prepared = undefined; continue; }
        const cover = prepared.extras.cover;
        if (extras.post && metadata.images.length > 1 && metadata.images.includes(image)) {
          extras.post.gallery.push(cover);
        }
        if (!extras.cover) extras.cover = cover;
        update({ data: {}, extras: { ...extras } });
      } catch (error) { if (error.code === 'CAPTURE_BLOCKED') throw error; }
    }
    if ((type === 'WebPage' && !extras.cover) || type === 'Screenshot') {
      if (type === 'WebPage' && !extras.screenshotPending) {
        extras.screenshotPending = true;
        update({ data: {}, extras: { screenshotPending: true } });
      }
      try {
        prepared = await media.prepare(await screenshot(response.url, { offline }), 'image/png', 'screenshot', user, id);
        extras.cover = prepared.extras.cover;
      } catch (error) {
        if (error.code === 'CAPTURE_BLOCKED') throw error;
        if (type === 'WebPage' && !extras.captureText && !description) {
          update({ data: {}, extras: { screenshotFailed: true } });
          return;
        }
      }
    }
    if (type === 'SoftwareApplication' && extras.cover) {
      extras.app = { ...extras.app, srcset: `${extras.cover.src.replace(/;\d+x\d+-t\.webp$/, ';250x250-c.webp')} 1x`, width: 250, height: 250 };
    }
    if (extras.post?.gallery.length > 1) extras.cover = undefined;
    else if (extras.post) extras.post.gallery = [];
    if (type === 'InstagramReel' && extras.cover) extras.cover.isThumbnail = true;
    if (type === 'Screenshot' && !extras.cover) data.type = 'WebPage';
    update({ data, extras });
  } catch {
    update({ data: { type: 'Placeholder', title: card.title || new URL(url).host, domain: card.domain || new URL(url).hostname, siteName: card.siteName || new URL(url).hostname },
      extras: { linkUrl: url, cover: undefined, post: undefined, captureFailed: true } });
  }
}
