// One side of a live-vs-replica comparison: an isolated browser context inside the user's logged-in
// Chromium whose time, randomness, animations and network all move only when the driver says so.
import { createHash } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadResource, loadThroughDevTools } from './cdp.mjs';
import { deviceVariant, keepHeaders, replayKey } from './replay-store.mjs';

const PAGE_SCRIPT = path.join(path.dirname(fileURLToPath(import.meta.url)), 'page-determinism.js');
const CHUNK_MS = 50;
const QUIET_MS = 30;
const SLOW_REQUEST_MS = 20000;
const DRAIN_LIMIT_MS = 45000;
// Real time a page gets after a response arrives before we look again (module evaluation, header delivery).
const RELEASE_GRACE_MS = 50;
const SETTLE_RASTER_MS = 400;
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const STRIP_ON_FULFILL = /^(content-encoding|content-length|transfer-encoding)$/i;
// Players that decide what to show from the embedding site (a video can refuse to embed on one origin and
// not another), so their content is not the replica's to match. They are hidden for the capture; the space
// they take still has to match.
const THIRD_PARTY_EMBEDS = ['youtube.com', 'youtube-nocookie.com', 'vimeo.com', 'spotify.com', 'twitter.com', 'instagram.com', 'tiktok.com', 'loom.com']
  .map((host) => `iframe[src*="${host}/"]`)
  .join(', ');

export class Side {
  pending = new Map();
  held = [];
  seq = 0;
  lastActivity = Date.now();
  redirects = new Map();
  mediaUrls = new Set();
  writes = [];
  log = [];

  /**
   * Opens a fresh, deterministic browser context and page.
   * @param {import('playwright-core').Browser} browser Browser from `chromium.connectOverCDP`.
   * @param {object} options
   * @param {string} options.name Label used in logs (`live` or `replica`).
   * @param {string} options.origin Origin every route is resolved against.
   * @param {object} options.contextOptions Playwright context options (viewport, device, storage state…).
   * @param {number} options.startTime Virtual epoch milliseconds both sides start at.
   * @param {(request: { url: string, method: string }) => boolean} [options.firewall] Requests to answer from `options.mirror` instead of sending.
   * @param {string} [options.mirror] Origin whose answers stand in for firewalled requests.
   * @param {{ put: Function }} [options.store] Replay store that receives every 200 response.
   * @param {string} [options.runId] Names this run's recordings, and asks the replica for them.
   * @returns {Promise<Side>}
   */
  static async open(browser, options) {
    const side = new Side();
    Object.assign(side, options);
    side.variant = deviceVariant(options.contextOptions.userAgent ?? '');
    side.restPoint = [(options.contextOptions.viewport?.width ?? 1280) - 1, 0];
    side.context = await browser.newContext({ serviceWorkers: 'block', ...options.contextOptions });
    await side.context.addInitScript({ path: PAGE_SCRIPT });
    await side.context.clock.install({ time: options.startTime });
    await side.context.clock.pauseAt(options.startTime + 1000);
    // Playwright's fake performance.now() (and the rAF timestamps derived from it) keeps counting real time, so
    // rebase both on the fake Date, which only moves when the driver moves it. GSAP and OGL time everything
    // with these. Nothing here may read the clock at injection time: that would make Playwright replay its
    // pause log before the rest of it is injected, and the clock would then run on real time.
    // Frames are also re-timed off the fake Date: Playwright aligns them to an internal tick counter whose phase
    // depends on how long the page took to load.
    await side.context.addInitScript((origin) => {
      const now = () => Date.now() - origin;
      performance.now = now;
      const { setTimeout: later, clearTimeout: cancel } = window;
      window.requestAnimationFrame = (callback) => later(() => callback(now()), 16 - (now() % 16));
      window.cancelAnimationFrame = (id) => cancel(id);
    }, options.startTime);
    side.page = await side.context.newPage();
    side.page.on('pageerror', (error) => side.note(`page error: ${error.message.split('\n')[0]}`));
    side.page.on('console', (message) => message.type() === 'error' && side.note(`console error: ${message.text().slice(0, 200)}`));
    // Hover follows the last mouse event Chrome saw; without this it is wherever the real pointer sits.
    if (!options.contextOptions.isMobile) await side.page.mouse.move(side.restPoint[0], side.restPoint[1]);
    side.cdp = await side.context.newCDPSession(side.page);
    side.cdp.on('Fetch.requestPaused', (event) => side.#onPaused(event).catch((error) => side.note(`fetch ${event.request.url}: ${error.message}`)));
    // Overlay scrollbars fade out on a real-time timer that nothing on the page controls.
    await side.cdp.send('Emulation.setScrollbarsHidden', { hidden: true });
    await side.cdp.send('Network.enable');
    await side.cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
    side.mainFrameId = (await side.cdp.send('Page.getFrameTree')).frameTree.frame.id;
    await side.cdp.send('Fetch.enable', { patterns: [{ urlPattern: '*', requestStage: 'Request' }] });
    return side;
  }

  note(message) {
    this.log.push(message);
  }

  async #onPaused(event) {
    const { requestId, request, resourceType } = event;
    const atResponse = event.responseStatusCode !== undefined || event.responseErrorReason !== undefined;
    this.lastActivity = Date.now();
    if (!atResponse) {
      if (event.redirectedRequestId) this.redirects.set(requestId, this.redirects.get(event.redirectedRequestId) ?? event.redirectedRequestId);
      const seq = ++this.seq;
      if (this.firewall?.(request)) return this.#answerFromMirror(event, seq);
      // Embedded players can send the whole tab to their own site (a mobile YouTube embed jumps to
      // m.youtube.com). A 204 cancels such a navigation and keeps the app on screen, on both sides alike.
      if (resourceType === 'Document' && event.frameId === this.mainFrameId && !request.url.startsWith(this.origin)) {
        this.note(`kept the app on screen; blocked navigation to ${request.url.slice(0, 100)}`);
        return this.cdp.send('Fetch.fulfillRequest', { requestId, responseCode: 204, body: '' });
      }
      if (resourceType === 'Media') {
        this.mediaUrls.add(request.url);
        return this.cdp.send('Fetch.continueRequest', { requestId });
      }
      this.pending.set(requestId, { seq, url: request.url, since: Date.now() });
      return this.cdp.send('Fetch.continueRequest', { requestId, interceptResponse: true, ...this.#tagged(request) });
    }

    const { seq = ++this.seq } = this.pending.get(requestId) ?? {};
    this.pending.delete(requestId);
    const status = event.responseStatusCode;
    if (event.responseErrorReason && event.responseErrorReason !== 'Aborted' && request.method === 'GET') {
      if (await this.#retry(event, seq)) return;
    }
    if (event.responseErrorReason || (status >= 300 && status < 400)) {
      if (status >= 300 && status < 400) this.redirects.set(`url:${requestId}`, request.url);
      return this.cdp.send('Fetch.continueRequest', { requestId });
    }
    let body;
    try {
      const response = await this.cdp.send('Fetch.getResponseBody', { requestId });
      body = Buffer.from(response.body, response.base64Encoded ? 'base64' : 'utf8');
    } catch {
      return this.cdp.send('Fetch.continueRequest', { requestId });
    }
    this.held.push({ seq, requestId, status, headers: event.responseHeaders ?? [], body });
    if (this.store && status === 200) await this.#record(event, body);
  }

  // Transient network failures (Chrome aborts in-flight loads with ERR_NETWORK_CHANGED whenever an interface
  // changes) would otherwise break one side at random. Load it again through the browser and hand that over.
  async #retry(event, seq) {
    // Only for the two servers under comparison; third parties fail for their own reasons.
    if (!event.request.url.startsWith(this.origin) && !event.request.url.startsWith(this.mirror ?? this.origin)) return false;
    const { frameTree } = await this.cdp.send('Page.getFrameTree');
    const response = await Promise.race([loadResource(this.cdp, frameTree.frame.id, event.request.url).catch(() => null), sleep(15000).then(() => null)]);
    if (!response) return false;
    this.note(`retried after ${event.responseErrorReason}: ${event.request.url.slice(0, 100)}`);
    const headers = Object.entries(response.headers).map(([name, value]) => ({ name, value }));
    this.held.push({ seq, requestId: event.requestId, status: response.status, headers, body: response.body });
    if (this.store && response.status === 200) await this.#record({ ...event, responseHeaders: headers }, response.body);
    return true;
  }

  // Replica requests name the run, so the server answers with exactly what live sent in this run even when
  // other runs are recording at the same time.
  #tagged(request) {
    if (!this.runId || this.store || !request.url.startsWith(this.origin)) return {};
    const headers = Object.entries(request.headers).map(([name, value]) => ({ name, value }));
    return { headers: [...headers, { name: 'x-replay-run', value: this.runId }] };
  }

  #keysFor(method, url, isDocument) {
    const key = replayKey(method, url, isDocument, this.variant);
    return this.runId ? [key, `run:${this.runId}|${key}`] : [key];
  }

  async #record(event, body) {
    const { request, resourceType, requestId } = event;
    if (!/^https?:/.test(request.url)) return;
    const urls = [request.url];
    const original = this.redirects.get(requestId);
    if (original) urls.push(this.redirects.get(`url:${original}`));
    const keys = urls.filter(Boolean).flatMap((url) => this.#keysFor(request.method, url, resourceType === 'Document'));
    await this.store.put(keys, 200, keepHeaders(event.responseHeaders ?? []), body);
  }

  async #answerFromMirror(event, seq) {
    const { request, requestId } = event;
    const url = new URL(request.url);
    this.pending.set(requestId, { seq, url: request.url, since: Date.now() });
    const response = await fetch(this.mirror + url.pathname + url.search, {
      method: request.method,
      headers: { 'content-type': request.headers['Content-Type'] ?? request.headers['content-type'] ?? 'text/plain', 'user-agent': request.headers['User-Agent'] ?? '' },
      body: request.postData,
    });
    const body = Buffer.from(await response.arrayBuffer());
    this.writes.push(`${request.method} ${url.pathname} -> ${response.status} ${body.toString('utf8').slice(0, 80)}`);
    this.held.push({ seq, requestId, status: response.status, headers: [...response.headers].map(([name, value]) => ({ name, value })), body });
    this.pending.delete(requestId);
  }

  async #release() {
    const batch = this.held.splice(0).sort((a, b) => a.seq - b.seq);
    if (batch.length) this.lastRelease = Date.now();
    for (const item of batch) {
      await this.cdp
        .send('Fetch.fulfillRequest', {
          requestId: item.requestId,
          responseCode: item.status,
          responseHeaders: item.headers.filter(({ name }) => !STRIP_ON_FULFILL.test(name)),
          body: item.body.toString('base64'),
        })
        .catch((error) => this.note(`fulfill ${item.requestId}: ${error.message}`));
    }
    return batch.length;
  }

  async #quiet() {
    for (;;) {
      const now = Date.now();
      for (const [id, request] of this.pending) {
        if (now - request.since > SLOW_REQUEST_MS) {
          this.pending.delete(id);
          this.note(`gave up waiting for ${request.url}`);
        }
      }
      if (!this.pending.size && now - this.lastActivity >= QUIET_MS) return;
      await sleep(5);
    }
  }

  async #pageBusy() {
    try {
      return (await this.page.evaluate(() => window.__vt?.pending() ?? ['no __vt'])) ?? [];
    } catch {
      return ['navigating'];
    }
  }

  async #frames() {
    await this.page.evaluate(() => window.__vt?.frames()).catch(() => {});
  }

  /** Releases held responses until the network is idle and the page has nothing left loading. */
  async drain() {
    const start = Date.now();
    for (;;) {
      await this.#quiet();
      const released = await this.#release();
      const busy = await this.#pageBusy();
      const graceOver = Date.now() - (this.lastRelease ?? 0) >= RELEASE_GRACE_MS;
      if (!released && graceOver && !this.pending.size && !this.held.length && !busy.length) return;
      if (Date.now() - start > DRAIN_LIMIT_MS) {
        this.note(`drain gave up: ${busy.join(', ')}`);
        return;
      }
      await sleep(released ? 25 : 15);
    }
  }

  /**
   * Advances virtual time, letting the network settle between every chunk.
   * @param {number} ms Virtual milliseconds to advance.
   * @param {number} [chunk] Chunk size; smaller is closer to real frame-by-frame time.
   */
  async settle(ms, chunk = CHUNK_MS) {
    await this.#rendered();
    for (let t = 0; t < ms; t += chunk) {
      const dt = Math.min(chunk, ms - t);
      await this.context.clock.runFor(dt);
      // A frame that never got a document (a PDF turned into a download on phones) would wait forever.
      await Promise.all(this.page.frames().map((frame) => Promise.race([frame.evaluate((d) => window.__vt?.advance(d), dt).catch(() => {}), sleep(2000)])));
      await this.#rendered();
    }
  }

  // Network idle, then two real frames (which may start more loads), then idle again.
  async #rendered() {
    await this.drain();
    await this.#frames();
    await this.drain();
  }

  /**
   * Navigates to a route on this side's origin and settles.
   * @param {string} route Path plus query, e.g. `/everything`.
   * @param {number} ms Virtual milliseconds to settle afterwards.
   */
  async goto(route, ms) {
    let committed = false;
    const go = () => this.page.goto(this.origin + route, { waitUntil: 'commit', timeout: 90000 });
    // A freshly started browser can report a network change and abort its first navigation.
    const navigation = go()
      .catch((error) => (/ERR_NETWORK_CHANGED/.test(error.message) ? go() : Promise.reject(error)))
      .finally(() => (committed = true));
    while (!committed) {
      await this.#quiet();
      await this.#release();
      await sleep(15);
    }
    await navigation;
    await this.settle(ms);
  }

  /**
   * Finds the centre of the first element matching a selector.
   * @param {string} selector CSS selector, `x,y` viewport coordinates, or `center` / `top` of the viewport.
   * @param {{ reveal?: boolean }} [options] `reveal` scrolls the target into view first when its centre is off screen.
   * @returns {Promise<[number, number]>}
   * @throws {Error} When nothing visible matches.
   */
  async point(selector, { reveal = false } = {}) {
    const coords = /^(\d+),(\d+)$/.exec(selector);
    if (coords) return [Number(coords[1]), Number(coords[2])];
    const { width, height } = this.page.viewportSize();
    if (selector === 'center') return [Math.round(width / 2), Math.round(height / 2)];
    if (selector === 'top') return [Math.round(width / 2), Math.round(height * 0.2)];
    const target = this.page.locator(selector).filter({ visible: true }).first();
    let box = await target.boundingBox({ timeout: 8000 });
    const [cx, cy] = box ? [box.x + box.width / 2, box.y + box.height / 2] : [0, 0];
    if (reveal && box && (cy < 0 || cy >= height || cx < 0 || cx >= width)) {
      await target.scrollIntoViewIfNeeded({ timeout: 8000 });
      await this.settle(0);
      box = await target.boundingBox({ timeout: 8000 });
    }
    if (!box) throw new Error(`${this.name}: ${selector} is not visible`);
    return [box.x + box.width / 2, box.y + box.height / 2];
  }

  /**
   * Captures the viewport as PNG plus the REA screenshot artifact JSON.
   * @param {string} file Output path without extension.
   * @returns {Promise<string>} SHA-256 of the PNG.
   */
  async shot(file) {
    await this.#rendered();
    await this.page.evaluate(() => window.__vt?.decodeImages()).catch(() => {});
    // Chrome re-rasterizes transformed layers at their final scale only after they sit still for a while.
    await sleep(SETTLE_RASTER_MS);
    // Virtual time is frozen here, so anything still changing is real-time rendering in another process (PDF
    // viewer, cross-origin frames): wait until two captures in a row agree.
    // Playwright's own `mask` waits for an animation frame, which never comes on a paused clock.
    const hide = (hidden) =>
      this.page.evaluate(
        ({ selector, hidden }) =>
          document.querySelectorAll(selector).forEach((el) => (hidden ? el.style.setProperty('visibility', 'hidden', 'important') : el.style.removeProperty('visibility'))),
        { selector: THIRD_PARTY_EMBEDS, hidden },
      );
    const masked = await hide(true).then(() => true, () => false);
    // Software WebGL (serendipity) can take many seconds per frame on a busy machine.
    const capture = () => this.page.screenshot({ caret: 'hide', animations: 'allow', timeout: 180000 });
    let png = await capture();
    for (let attempt = 0; attempt < 12; attempt++) {
      await sleep(250);
      const next = await capture();
      if (next.equals(png)) break;
      png = next;
      if (attempt === 11) this.note(`still changing after 3s: ${path.basename(file)}`);
    }
    if (masked) await hide(false).catch(() => {});
    const sha256 = createHash('sha256').update(png).digest('hex');
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(`${file}.png`, png);
    await writeFile(`${file}.artifact.json`, JSON.stringify({ sha256, bytes: png.length, media_type: 'image/png', data_base64: png.toString('base64') }));
    return sha256;
  }

  /**
   * Downloads media the page streamed (videos, audio) into the replay store through this context.
   * @returns {Promise<number>} How many were added.
   */
  async recordMedia() {
    if (!this.store) return 0;
    const { frameTree } = await this.cdp.send('Page.getFrameTree');
    let added = 0;
    for (const url of this.mediaUrls) {
      const keys = this.#keysFor('GET', url, false);
      if (keys.every((key) => this.store.index[key])) continue;
      const body = await loadThroughDevTools(this.cdp, frameTree.frame.id, url).catch(() => null);
      if (!body) {
        this.note(`media not downloaded: ${url.split('?')[0]}`);
        continue;
      }
      const ext = new URL(url).pathname.split('.').pop().toLowerCase();
      const type = { mp4: 'video/mp4', webm: 'video/webm', mov: 'video/quicktime', mp3: 'audio/mpeg', m4a: 'audio/mp4' }[ext] ?? 'application/octet-stream';
      await this.store.put(keys, 200, { 'content-type': type }, body);
      added++;
    }
    return added;
  }

  async close() {
    await this.context.close().catch(() => {});
  }
}
