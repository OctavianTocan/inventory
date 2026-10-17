import { DurableObject } from 'cloudflare:workers';

const PORT = 8080;
const START_TIMEOUT_MS = 60_000;
// The container keeps running this long after its last request, so a burst of work starts it once.
const IDLE_MS = 3 * 60_000;

/**
 * The media container (`worker/container/`): image and video renditions, PDF previews, page screenshots and
 * validated fetches, the work that needs sharp, ffmpeg, poppler and Chromium. One instance serves every library.
 */
export class Media extends DurableObject {
  // Start the container when it is not running, and resolve once its server answers.
  ready() {
    const { container } = this.ctx;
    if (!container.running) {
      this.started = null;
      container.start({ enableInternet: true });
      container.monitor().catch((error) => console.error('[media] container stopped', error)).finally(() => { this.started = null; });
    }
    this.started ??= this.answering().catch((error) => { this.started = null; throw error; });
    return this.started;
  }

  // Poll the container's health endpoint until it answers.
  async answering() {
    const deadline = Date.now() + START_TIMEOUT_MS;
    for (;;) {
      try {
        const response = await this.ctx.container.getTcpPort(PORT).fetch('http://container/health');
        if (response.ok) break;
      } catch (error) {
        if (Date.now() > deadline) throw error;
      }
      if (Date.now() > deadline) throw new Error('The media container did not start');
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
    try { await this.ctx.container.setInactivityTimeout(IDLE_MS); }
    catch (error) { console.warn('[media] inactivity timeout not set', error.message); }
  }

  /**
   * Pass one operation to the container's server.
   * @param {Request} request Operation from `worker/media-client.mjs`.
   * @returns {Promise<Response>} The container's answer.
   */
  async fetch(request) {
    await this.ready();
    return this.ctx.container.getTcpPort(PORT).fetch(request);
  }
}
