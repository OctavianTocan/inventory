import { createHash, randomBytes, timingSafeEqual } from 'node:crypto';
import { mkdir, open, readdir, rm, stat, truncate } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const coordinators = new Map();
const DEFAULT_MAX_BYTES = 1024 * 1024 * 1024;

// Reject malformed upload requests without echoing filenames, tokens or body bytes.
function fail(status, message) { throw Object.assign(new Error(message), { status }); }

// Hash an ephemeral upload capability before persisting or comparing it.
function hash(value) { return createHash('sha256').update(value).digest(); }

/**
 * Build resumable local chunk uploads with bounded temporary storage.
 * @param {object} store Current user's open SQLite store.
 * @param {string} directory Private temporary upload directory alongside the database.
 * @param {object} media createMedia handle for validating completed files and resolving claimed originals.
 * @param {object} [options] Temporary storage limits.
 * @param {number} [options.maxBytes=1073741824] Maximum persisted and reserved temporary bytes across uploads.
 * @returns {object} authorize, put, prepare, claim, cleanupClaimed, source and sweep operations; never forwards writes.
 */
export function createUploads(store, directory, media, { maxBytes = DEFAULT_MAX_BYTES } = {}) {
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 1) throw new RangeError('Invalid temporary upload limit');
  const { db } = store;
  const user = store.user();
  const key = resolve(directory);
  if (!coordinators.has(key)) coordinators.set(key, { busy: new Set(), reservations: new Map(), sweep: null });
  const state = coordinators.get(key);
  const { busy, reservations } = state;

  // Require an unexpired upload owned by this library, including consumed idempotency records.
  function find(id) {
    const row = db.prepare('SELECT * FROM uploads WHERE id = ? AND user_id = ?').get(id, user.id);
    if (!row || row.expires_ms <= Date.now()) fail(404, 'Upload not found or expired');
    return row;
  }

  // Remove expired rows, orphaned files and consumed originals without disturbing active uploads.
  async function sweep() {
    if (state.sweep) return state.sweep;
    state.sweep = (async () => {
      const rows = db.prepare('SELECT id, card_id, received, expires_ms FROM uploads').all();
      const known = new Set(rows.map(row => row.id));
      for (const row of rows) {
        if (busy.has(row.id)) continue;
        if (row.card_id || row.expires_ms <= Date.now()) await rm(join(directory, row.id), { force: true });
        else {
          const file = join(directory, row.id);
          const info = await stat(file).catch(error => { if (error.code !== 'ENOENT') throw error; });
          if (info?.size > row.received) await truncate(file, row.received);
        }
        if (row.expires_ms <= Date.now()) db.prepare('DELETE FROM uploads WHERE id = ?').run(row.id);
      }
      const names = await readdir(directory).catch(error => { if (error.code === 'ENOENT') return []; throw error; });
      for (const name of names) {
        if (/^[a-f0-9]{22}$/.test(name) && !known.has(name) && !busy.has(name)) await rm(join(directory, name), { force: true });
      }
    })().finally(() => { state.sweep = null; });
    return state.sweep;
  }

  // Reserve a chunk before consuming its request so concurrent requests share one aggregate bound.
  function reserve(id, length) {
    const used = db.prepare('SELECT COALESCE(SUM(received), 0) AS bytes FROM uploads WHERE card_id IS NULL').get().bytes;
    const held = [...reservations.values()].reduce((sum, value) => sum + value, 0);
    if (used + held + length > maxBytes) fail(413, 'Temporary upload storage limit exceeded');
    reservations.set(id, length);
  }

  // Resolve consumed uploads through permanent card media after deleting the temporary copy.
  async function source(id) {
    await sweep();
    const row = find(id);
    if (!row.done || row.received !== row.size) fail(409, 'Upload is incomplete');
    if (!row.card_id) return { file: join(directory, id), type: row.mime, name: row.name };
    const card = store.card(row.card_id);
    const data = card ? JSON.parse(card.data) : {};
    const extras = card ? JSON.parse(card.extras) : {};
    const attachment = extras.attachments?.find(item => item.uploadId === id);
    const path = attachment?.path?.replace(/^\/media\//, '') ?? data.blob?.path;
    const found = path ? await media.response(path) : null;
    if (!found) fail(404, 'Upload media is unavailable');
    return { file: found.file, type: found.type, name: row.name };
  }

  return {
    sweep,

    async authorize(count) {
      await sweep();
      const n = count === null ? 1 : Number(count);
      if (!Number.isInteger(n) || n < 1 || n > 100) fail(400, 'Upload count must be 1 to 100');
      const results = store.transaction(() => Array.from({ length: n }, () => {
        const id = randomBytes(11).toString('hex');
        const token = randomBytes(32).toString('hex');
        db.prepare('INSERT INTO uploads (id, user_id, token_hash, expires_ms) VALUES (?, ?, ?, ?)')
          .run(id, user.id, hash(token).toString('hex'), Date.now() + 24 * 60 * 60 * 1000);
        return { url: `/uploads/${id}`, token };
      }));
      return count === null ? results[0] : results;
    },

    async put(id, req) {
      await sweep();
      const row = find(id);
      const token = /^Bearer (.+)$/.exec(req.headers.authorization ?? '')?.[1];
      if (!token || !timingSafeEqual(hash(token), Buffer.from(row.token_hash, 'hex'))) fail(401, 'Invalid upload authorization');
      if (busy.has(id)) fail(409, 'Upload chunk already in progress');
      if (row.done) fail(409, 'Upload already complete');
      if (req.headers['x-upload-id'] && req.headers['x-upload-id'] !== id) fail(409, 'Upload ID mismatch');
      const range = /^bytes (\d+)-(\d+)\/(\d+)$/.exec(req.headers['content-range'] ?? '');
      if (!range) fail(400, 'Content-Range is required');
      const [start, end, total] = range.slice(1).map(Number);
      if (![start, end, total].every(Number.isSafeInteger) || start !== row.received || end < start || end >= total || (row.size !== null && total !== row.size)) fail(409, 'Upload range mismatch');
      if (total > 500000000 || end - start + 1 > 32 * 1024 * 1024) fail(413, 'Upload exceeds size limit');
      const mime = (req.headers['content-type'] ?? '').split(';')[0];
      let name;
      try { name = decodeURI(req.headers['x-file-name'] ?? 'upload').slice(0, 255); } catch { fail(400, 'Invalid upload filename'); }
      if (start && (row.mime !== mime || row.name !== name)) fail(409, 'Upload metadata mismatch');
      reserve(id, end - start + 1);
      busy.add(id);
      let file;
      try {
        await mkdir(directory, { recursive: true });
        file = await open(join(directory, id), start ? 'r+' : 'w');
        let length = 0;
        for await (const chunk of req) {
          if (length + chunk.length > end - start + 1) fail(400, 'Chunk length mismatch');
          for (let written = 0; written < chunk.length;) {
            const result = await file.write(chunk, written, chunk.length - written, start + length + written);
            if (!result.bytesWritten) throw new Error('Upload write made no progress');
            written += result.bytesWritten;
          }
          length += chunk.length;
        }
        if (length !== end - start + 1) fail(400, 'Chunk length mismatch');
        const done = end + 1 === total;
        db.prepare('UPDATE uploads SET name = ?, mime = ?, size = ?, received = ?, done = ? WHERE id = ?')
          .run(name, mime, total, end + 1, done ? 1 : 0, id);
        return { id, blob: { id }, received: end + 1, done };
      } catch (error) {
        await file?.truncate(row.received);
        throw error;
      } finally {
        try { await file?.close(); }
        finally { reservations.delete(id); busy.delete(id); }
      }
    },

    async prepare(id) {
      await sweep();
      const row = find(id);
      if (!row.done || row.received !== row.size) fail(409, 'Upload is incomplete');
      if (row.card_id) return { existingId: row.card_id };
      if (busy.has(id)) fail(409, 'Upload is being processed');
      busy.add(id);
      try {
        const cardId = store.newId('card');
        const prepared = await media.prepare({ file: join(directory, id) }, row.mime, row.name, user, cardId);
        return { ...prepared, id: cardId };
      } catch (error) {
        if (error.status) throw error;
        fail(422, 'Media preview could not be generated');
      } finally { busy.delete(id); }
    },

    claim(id, cardId, reuse = false) {
      const row = find(id);
      if (row.card_id && reuse) return;
      if (row.card_id && row.card_id !== cardId) fail(409, 'Upload already consumed');
      db.prepare('UPDATE uploads SET card_id = ? WHERE id = ?').run(cardId, id);
    },

    async cleanupClaimed(id) {
      const row = db.prepare('SELECT card_id FROM uploads WHERE id = ? AND user_id = ?').get(id, user.id);
      if (row?.card_id) await rm(join(directory, id), { force: true });
    },

    source,
  };
}
