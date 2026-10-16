import { embed, cosine } from './ai.mjs';
import { GRID_HIDDEN_FLAGS } from './store.mjs';

/**
 * Decide whether hosted embeddings are allowed, permitting only an explicit loopback stub offline.
 * @param {boolean} [offline=false] Disable hosted API traffic.
 * @returns {boolean} Key and transport policy permit an embedding call.
 */
export function embeddingsEnabled(offline = false) {
  if (!process.env.GEMINI_API_KEY) return false;
  if (!offline) return true;
  try { return ['127.0.0.1', 'localhost', '[::1]'].includes(new URL(process.env.GEMINI_BASE_URL).hostname); }
  catch { return false; }
}

/**
 * Decode a stored little-endian 768-float embedding, ignoring corrupt/nonfinite vectors.
 * @param {Uint8Array} bytes SQLite vector BLOB.
 * @returns {Float32Array|null} Vector, or null if invalid.
 */
export function decodeVector(bytes) {
  if (bytes?.byteLength !== 3072) return null;
  const buffer = Buffer.from(bytes);
  const vector = Float32Array.from({ length: 768 }, (_, i) => buffer.readFloatLE(i * 4));
  return vector.every(Number.isFinite) ? vector : null;
}

/**
 * Build cosine search over this user's active vectors; never loads a local model.
 * @param {object} store Open SQLite store.
 * @param {{offline?:boolean}} [options] Hosted API policy.
 * @returns {object} Async search and synchronous related methods returning ordered id/score matches.
 */
export function createSemantic(store, { offline = false } = {}) {
  // Select eligible vectors after I/O so trash/edits during a query cannot resurrect hidden cards.
  function vectors() {
    return store.db.prepare(`SELECT e.card_id AS id, e.vector FROM embeddings e JOIN cards c ON c.id = e.card_id
      WHERE e.user_id = ? AND c.user_id = e.user_id AND c.deleted_ns IS NULL AND (c.flags & ${GRID_HIDDEN_FLAGS}) = 0`)
      .all(store.defaultUserId());
  }

  // Bound the client's requested result count, then order nearest vectors with deterministic ties.
  function nearest(vector, { take = 100, eligible = null, exclude = null } = {}) {
    const count = take === -1 ? Infinity : Math.max(0, Math.min(1000, Number.isFinite(Number(take)) ? Math.floor(Number(take)) : 100));
    const allowed = typeof eligible === 'function' ? eligible() : eligible;
    return vectors().filter(row => row.id !== exclude && (!allowed || allowed.has(row.id)))
      .flatMap(row => { const other = decodeVector(row.vector); return other ? [{ id: row.id, score: cosine(vector, other) }] : []; })
      .sort((a, b) => b.score - a.score || a.id.localeCompare(b.id)).slice(0, count);
  }

  return {
    async search(q, options) {
      if (!String(q ?? '').trim() || !embeddingsEnabled(offline) || !vectors().length) return [];
      try { return nearest(await embed({ text: q }, { kind: 'query' }), options); }
      catch { return []; }
    },
    related(id, options) {
      const row = vectors().find(row => row.id === id), vector = row && decodeVector(row.vector);
      return vector ? nearest(vector, { ...options, exclude: id }) : [];
    },
  };
}
