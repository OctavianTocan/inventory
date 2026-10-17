import { fetchResource } from './capture-link.mjs';

// Read one bounded pasted file or attachment without passing its bytes through JSON.
async function binary(req, limit = 64 * 1024 * 1024) {
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > limit) throw Object.assign(new Error('Pasted file is too large; use chunked uploads'), { status: 413 });
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

/**
 * Handle clipboard multipart files and inline image attachments before JSON mutation routing.
 * @param {object} options Store, writer, media, uploads, jobs and response dependencies.
 * @param {object} options.store Open SQLite store.
 * @param {object} options.writer Transactional writer.
 * @param {object} options.media Local media storage.
 * @param {object} options.uploads Completed upload access.
 * @param {object} options.jobs Post-commit enrichment queue.
 * @param {Function} options.reply Send JSON with quota headers.
 * @param {boolean} options.offline Permit only loopback remote attachments.
 * @param {Function} [options.resource=fetchResource] Validated HTTP transport for remote attachments.
 * @returns {(req:object,res:object,url:URL) => Promise<boolean>} True only for a handled media write.
 */
export function createMediaWrites({ store, writer, media, uploads, jobs, reply, offline, resource = fetchResource }) {
  return async (req, res, url) => {
    if (req.method !== 'POST') return false;
    const mime = req.headers['content-type'] ?? '';
    if (url.pathname === '/objects' && mime.startsWith('multipart/form-data')) {
      let form, metadata;
      try {
        form = await new Response(await binary(req), { headers: { 'content-type': mime } }).formData();
        const part = form.get('metadata');
        metadata = part ? JSON.parse(typeof part === 'string' ? part : await part.text()) : {};
      } catch (error) {
        if (error.status) throw error;
        throw Object.assign(new Error('Invalid pasted file form'), { status: 400 });
      }
      const file = form.get('blob');
      if (!file || typeof file.arrayBuffer !== 'function') throw Object.assign(new Error('Pasted file is required'), { status: 400 });
      const id = store.newId('card');
      const prepared = await media.prepare(Buffer.from(await file.arrayBuffer()), file.type, file.name, store.user(), id);
      const result = writer.run(() => writer.create(metadata, { ...prepared, id }));
      reply(res, result.status, result.body);
      jobs.schedule(id);
      return true;
    }
    const match = /^\/objects\/([^/]+)\/attachments$/.exec(url.pathname);
    if (!match) return false;
    const id = match[1];
    if (!store.card(id)) throw Object.assign(new Error('Card not found'), { status: 404 });
    let bytes = await binary(req), type = mime, name = 'attachment', uploadId;
    if (mime.startsWith('application/json')) {
      let body;
      try { body = JSON.parse(bytes); } catch { throw Object.assign(new Error('Invalid attachment request'), { status: 400 }); }
      if (body.uploadId) {
        const source = await uploads.source(body.uploadId);
        bytes = source.input; type = source.type; name = source.name; uploadId = body.uploadId;
      }
      else if (body.remoteUrl) {
        const remote = await resource(body.remoteUrl, { offline, limit: 25 * 1024 * 1024 });
        bytes = remote.bytes; type = remote.mime;
      } else throw Object.assign(new Error('An uploadId or remoteUrl is required'), { status: 400 });
    }
    const prepared = await media.prepare(bytes, type, name, store.user(), id);
    if (prepared.data.type !== 'Image') throw Object.assign(new Error('Attachments must be images'), { status: 415 });
    const blob = prepared.data.blob;
    const attachment = { path: `/media/${blob.path}`, width: blob.width, height: blob.height };
    const current = store.card(id);
    const baseline = { data: JSON.parse(current.data), extras: JSON.parse(current.extras) };
    writer.run(() => {
      writer.enrich(id, { extras: { attachments: [...(baseline.extras.attachments ?? []),
        { ...attachment, ...(uploadId ? { uploadId } : {}) }] } }, baseline);
      if (uploadId) uploads.claim(uploadId, id, true);
    });
    if (uploadId) await uploads.cleanupClaimed(uploadId);
    reply(res, 200, attachment);
    return true;
  };
}
