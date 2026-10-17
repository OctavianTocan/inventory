import { createApi } from './api.mjs';
import { bucketUploads, createHostedMedia } from './hosted-media.mjs';
import { createRecorded } from './recorded.mjs';
import { createSite, localize, replayKeyFor, send } from './site.mjs';
import { createStore } from './store.mjs';

/**
 * Serve one person's library from its own database and its own part of the object store, as the Cloudflare Worker
 * does and `server.mjs` does with `HOSTED=1`. The database holds the library and any recordings it was given
 * (`tools/import-hosted.mjs`).
 * @param {object} options
 * @param {object} options.db SQL handle of the library database (`openSql`, or the Durable Object adapter); the
 *   caller closes it.
 * @param {object} options.bucket Object store holding mymind's website code (`static/`, shared) and every library's
 *   own files (`directoryBucket`, or the R2 adapter).
 * @param {string} options.library Library name; its media, uploads and captured card media live below
 *   `libraries/<library>/`, and nothing outside that prefix and `static/` is read.
 * @param {object} options.processor Media processor (`createProcessor`, or the media container's client).
 * @param {object} options.msgpack Mirrored MessagePack codec, with BigInt and timestamp extensions.
 * @param {(target:string, payload:object) => void} options.push Broadcaster for committed changes.
 * @param {boolean} [options.readonly=false] Acknowledge writes without persisting them.
 * @param {boolean} [options.offline=false] Never fetch website code that is missing from the store.
 * @param {Function} [options.screenshot] Page screenshot implementation for link capture.
 * @param {Function} [options.resource] Validated HTTP transport for link capture and remote attachments.
 * @returns {Promise<{api: object, handle: (req:object, res:object, url:URL) => Promise<void>, close: () => void}>}
 *   `handle` answers every request it is given; `api` is the library API, whose `shell` builds page shells.
 */
export async function createHosted({ db, bucket, library, processor, msgpack, push, readonly = false, offline = false, screenshot, resource }) {
  if (!/^[a-z0-9-]+$/.test(library)) throw new Error('Library names are lower-case letters, digits and dashes');
  const prefix = `libraries/${library}/`;
  const recorded = createRecorded(db);
  const api = await createApi({ root: '', store: createStore(db), media: createHostedMedia({ bucket, prefix, processor }),
    uploadStorage: bucketUploads(bucket, prefix), screenshot, resource, readonly, msgpack, localize, send, push,
    hasReplay: (req, url) => recorded.has(req, replayKeyFor(req, url)) });
  const site = createSite({ api, recorded, bucket, capture: `${prefix}capture/`, offline });
  return { api, handle: site.handle, close: () => api.close() };
}
