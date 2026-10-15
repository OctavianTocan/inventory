import http from 'node:http';
import { once } from 'node:events';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import sharp from 'sharp';
import { fixtureHttp } from './capture-fixture-http.mjs';
import { tagKey } from '../lib/ai-tags.mjs';

export const cases = [
  ['XPost', 'https://x.com/fixture/status/123', 'x-post post', 'class="body x"'],
  ['ThreadsPost', 'https://www.threads.net/@fixture/post/123', 'threads-post post', 'class="body threads"'],
  ['SubstackNote', 'https://substack.com/@fixture/note/c-123', 'substack-note post', 'class="body substack"'],
  ['YouTubeVideo', 'https://www.youtube.com/watch?v=fixture', 'youtube-video video', '<mymind-image'],
  ['InstagramReel', 'https://www.instagram.com/reel/fixture/', 'instagram-reel', 'class="lazy main-img"'],
  ['Repository', 'https://github.com/fixture/repo', 'repository', 'class="inner github"'],
  ['RedditPost', 'https://www.reddit.com/r/fixtures/comments/123/post/', 'reddit-post', 'class="reddit inner"'],
  ['SoftwareApplication', 'https://apps.apple.com/us/app/fixture/id123', 'software-application', 'distributor_app-store'],
  ['SoftwareApplicationPlay', 'https://play.google.com/store/apps/details?id=fixture', 'software-application', 'distributor_play-store'],
  ['Product', 'https://shop.invalid/fixture', 'product', 'class="price"'],
  ['Article', 'https://journal.invalid/fixture', 'article', 'class="article-card-top"'],
  ['WebPage', 'https://site.invalid/fixture', 'web-page', '<mymind-image'],
];

/**
 * Create synthetic media without copying any of the user's captured library.
 * @param {string} directory Temporary test workspace.
 * @returns {Promise<object>} PNG, WebP, PDF and MP4 fixture bytes.
 */
export async function fixtureMedia(directory) {
  const png = await sharp({ create: { width: 640, height: 480, channels: 3, background: '#cc4422' } }).png().toBuffer();
  const webp = await sharp(png).webp().toBuffer();
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>', '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> /Contents 4 0 R >>',
    '<< /Length 27 >>\nstream\n1 0 0 rg 10 10 100 100 re f\nendstream'];
  let pdf = '%PDF-1.4\n';
  const offsets = [0];
  objects.forEach((body, i) => { offsets.push(Buffer.byteLength(pdf)); pdf += `${i + 1} 0 obj\n${body}\nendobj\n`; });
  const xref = Buffer.byteLength(pdf);
  pdf += `xref\n0 5\n0000000000 65535 f \n${offsets.slice(1).map(n => `${String(n).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  const videoPath = join(directory, 'fixture.mp4');
  await promisify(execFile)('ffmpeg', ['-v', 'error', '-f', 'lavfi', '-i', 'color=c=red:s=320x240:d=0.3', '-an', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-y', videoPath], { timeout: 10000 });
  return { png, webp, pdf: Buffer.from(pdf), mp4: await readFile(videoPath) };
}

/**
 * Start an offline fixture HTTP server, or invoke the same handler in memory when requested.
 * Installs a fail-closed fetch guard so no test can reach a hosted API or live account.
 * @param {object} media Synthetic fixture bytes.
 * @param {boolean} inProcess True when the sandbox disallows socket listeners.
 * @returns {Promise<object>} Base URL, recorded synthetic requests, controls and cleanup.
 */
export async function startFixtures(media, inProcess) {
  const requests = [];
  const control = { aiFailure: false, aiRedirect: false, imageDelay: 0, chatDelay: 0, tags: null };
  let base = 'http://127.0.0.1:45459';
  // Answer only fixture resources and local AI stubs, using one handler for both transports.
  async function handler(req, res) {
    const url = new URL(req.url, base);
    const chunks = [];
    for await (const chunk of req) chunks.push(chunk);
    const text = Buffer.concat(chunks).toString();
    const body = text ? JSON.parse(text) : undefined;
    requests.push({ path: url.pathname, body, method: req.method });
    const send = (bytes, type = 'text/html', status = 200) => { res.writeHead(status, { 'content-type': type }); res.end(bytes); };
    const json = value => send(JSON.stringify(value), 'application/json');
    if (control.aiRedirect && req.method === 'POST') {
      res.writeHead(307, { location: 'https://redirect.invalid/should-never-be-requested' });
      return res.end();
    }
    if (url.pathname === '/cover.webp') {
      if (control.imageDelay) await new Promise(resolve => setTimeout(resolve, control.imageDelay));
      return send(media.webp, 'image/webp');
    }
    if (url.pathname === '/cover.png') return send(media.png, 'image/png');
    if (url.pathname === '/doc.pdf') return send(media.pdf, 'application/pdf');
    if (url.pathname === '/blocked') return send('blocked', 'text/plain', 403);
    if (url.pathname === '/redirect-account') { res.writeHead(302, { location: 'https://access.mymind.com/everything' }); return res.end(); }
    if (url.pathname === '/redirect-external') { res.writeHead(302, { location: 'https://external.invalid/image.png' }); return res.end(); }
    if (url.pathname === '/avatar.png') return send(media.png, 'image/png');
    if (url.pathname === '/x-embed') return json({ author_name: 'Jack Example', thumbnail_url: `${base}/profile_images/avatar.png`,
      html: '<blockquote class="twitter-tweet"><p>just setting up my twttr &mdash; hello &amp; welcome</p>&mdash; jack (@jack) <a href="https://x.com/jack/status/20">March 21, 2006</a></blockquote>' });
    if (url.pathname === '/embed') return json({ title: 'Fixture title', author_name: 'Fixture author', type: 'rich', thumbnail_url: `${base}/cover.png`,
      html: '<blockquote><p>Fixture <strong>post</strong><br>body <a href="javascript:alert(1)" onclick="bad()">bad link</a><a href="https://example.invalid/safe">safe</a></p></blockquote><script>alert(1)</script><img src=x onerror=bad()>' });
    if (url.pathname.includes('batchEmbedContents')) return json({ embeddings: body.requests.map(() => ({ values: Array.from({ length: 768 }, (_, i) => i === 0 ? 1 : 0) })) });
    if (url.pathname.includes('embedContent')) return json({ embedding: { values: Array.from({ length: 768 }, (_, i) => i === 0 ? 1 : 0) } });
    if (url.pathname === '/chat/completions') {
      if (control.chatDelay) await new Promise(resolve => setTimeout(resolve, control.chatDelay));
      if (control.aiFailure) return send('{}', 'application/json', 400);
      return json({ choices: [{ message: { content: JSON.stringify({ title: 'Upgrade Codex Memory Models for Free Now Today Extra', summary: 'A fixture summary <safe>.', tags: control.tags ?? ['design', 'website', 'link', 'page', 'image', 'screenshot'] }) } }] });
    }
    if (url.pathname === '/v1/systemone') return json({ answers: Object.fromEntries(Object.entries(body.questions).map(([key, question]) => [key, { type: 'noul', noul: ['design', 'image'].includes(tagKey(/tag "([^"]+)"/.exec(question.instructions)?.[1])) ? 0.95 : 0.1 }])) });
    const name = url.pathname.slice(1);
    if (name === 'XQuality') return send(`<title>Jack on X</title><link rel="canonical" href="https://x.com/jack/status/20"><meta property="og:image" content="/profile_images/avatar.png"><link rel="alternate" type="application/json+oembed" href="/x-embed">`);
    if (name === 'ArticleQuality') return send(`<title>Mind map - Wikipedia</title><meta property="og:site_name" content="Wikipedia"><meta name="description" content="A mind map organizes ideas visually."><meta property="og:type" content="article"><meta property="og:image" content="/cover.png"><main><nav>Unrelated navigation</nav><p>${'Mind maps connect concepts using branches and help organize knowledge. '.repeat(8)}</p><p>Use central topics and connections to plan a project.</p></main>`);
    const item = cases.find(item => item[0] === name) ?? (name === 'Gallery' ? cases[0] : null);
    const canonical = item?.[1] ?? `${base}/${name}`;
    const structured = name === 'Product' ? { '@type': 'Product', name: 'Fixture title', description: 'Fixture description', brand: { name: 'Fixture brand' }, offers: { priceCurrency: 'USD', price: '12.50' } }
      : name === 'JsonLd' ? { '@type': 'Article', headline: 'JSON-LD title', description: 'JSON-LD description', image: `${base}/cover.png`, author: { name: 'Schema author' } } : null;
    return send(`<!doctype html><title>Fallback title</title><link rel="canonical" href="${canonical}">
      ${item || name === 'Twitter' ? `<meta property="${name === 'Twitter' ? 'twitter' : 'og'}:title" content="${name === 'Repository' ? 'GitHub - fixture/repo: Fixture description' : name === 'SoftwareApplication' ? 'Fixture title on the App Store' : name === 'SoftwareApplicationPlay' ? 'Fixture title - Apps on Google Play' : 'Fixture title'}"><meta property="og:description" content="Fixture description"><meta property="og:site_name" content="Fixture site">` : ''}
      ${name === 'Article' ? '<meta property="og:type" content="article">' : ''}
      ${!['NoImage', 'BrokenImage', 'JsonLd'].includes(name) ? '<meta property="og:image" content="/cover.webp">' : ''}
      ${name === 'Gallery' ? '<meta property="og:image" content="/cover.png">' : ''}
      ${name === 'BrokenImage' ? '<meta property="og:image" content="/blocked">' : ''}
      ${structured ? `<script type="application/ld+json">${JSON.stringify({ '@context': 'https://schema.org', '@graph': [structured] })}</script>` : ''}
      ${item ? '<link rel="alternate" type="application/json+oembed" href="/embed">' : ''}<main>Fixture body text</main>`);
  }
  let server;
  if (!inProcess) {
    server = http.createServer((req, res) => handler(req, res).catch(() => { res.writeHead(500); res.end(); }));
    server.listen(0, '127.0.0.1');
    await once(server, 'listening');
    base = `http://127.0.0.1:${server.address().port}`;
  }
  const originalFetch = globalThis.fetch;
  const origins = new Set([base]);
  const restoreHttp = fixtureHttp(handler, origins, inProcess);
  globalThis.fetch = async (value, options = {}) => {
    const url = new URL(value);
    if (!origins.has(url.origin)) throw new Error('Test tried to leave fixture server');
    if (url.origin !== base) return originalFetch(value, options);
    if (!inProcess) return originalFetch(value, options);
    const response = await new Promise((resolve, reject) => {
      const req = { url: url.pathname + url.search, method: options.method ?? 'GET', async *[Symbol.asyncIterator]() { if (options.body) yield Buffer.from(options.body); } };
      const res = { writeHead(status, headers) { this.status = status; this.headers = headers; }, end(bytes) { resolve(new Response(bytes, { status: this.status, headers: this.headers })); } };
      handler(req, res).catch(reject);
    });
    if ([301, 302, 303, 307, 308].includes(response.status) && options.redirect !== 'manual') {
      return globalThis.fetch(new URL(response.headers.get('location'), value).href, options);
    }
    return response;
  };
  return { base, requests, control, allowOrigin: origin => origins.add(origin), close: async () => { restoreHttp(); globalThis.fetch = originalFetch; if (server) { server.close(); await once(server, 'close'); } } };
}
