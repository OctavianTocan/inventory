// Chromium cannot run in a Worker; on Cloudflare, page screenshots are taken in the media container (worker/container/).
export const chromium = {
  launch() {
    throw new Error('Chromium runs in the media container');
  },
};
