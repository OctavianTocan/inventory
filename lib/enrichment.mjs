import { captureLink } from './capture-link.mjs';
import { analyze } from './ai.mjs';
import { relevantTag } from './ai-tags.mjs';
import { analysisItem } from './search-document.mjs';

// Detect intervening edits before applying an AI result generated from older content.
function signature(row, store) {
  const data = JSON.parse(row.data), extras = JSON.parse(row.extras);
  return JSON.stringify([data.title, data.description, data.prose, data.source, extras.captureText, extras.cover, articleContent(row, store)]);
}

// Use the reader editor's current document instead of stale scraped Article prose.
function articleContent(row, store) {
  if (row.type !== 'Article') return null;
  const content = store.db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
  return content ? JSON.parse(content.body) : JSON.parse(row.data).prose;
}

/**
 * Queue capture and hosted AI after a successful save, serializing and coalescing jobs per card.
 * @param {object} options Service dependencies and immutable server policy.
 * @param {object} options.store Open store; never used after close().
 * @param {object} options.writer Synchronous writer with enrichment mutations.
 * @param {object} options.media Local media handle.
 * @param {boolean} [options.offline=false] Capture may access loopback fixtures only.
 * @param {boolean} [options.readonly=false] Disable all job creation, API calls and persistence.
 * @param {Function} [options.screenshot] Optional screenshot implementation for deterministic unit checks.
 * @returns {object} schedule(id, capture), drain() for tests/shutdown, and close() to stop new writes.
 */
export function createEnrichment({ store, writer, media, offline = false, readonly = false, screenshot }) {
  const pending = new Map();
  let closed = false;
  let running;

  // Keep OFFLINE tests and verification from calling hosted APIs even when the shell has keys.
  function enabled() {
    if (!process.env.GEMINI_API_KEY || readonly) return false;
    return !offline || ['GEMINI_BASE_URL', 'AI_CHAT_BASE_URL', ...(process.env.TYPESAFE_API_KEY ? ['TYPESAFE_BASE_URL'] : [])]
      .every(name => { try { return ['127.0.0.1', 'localhost', '[::1]'].includes(new URL(process.env[name]).hostname); } catch { return false; } });
  }

  // Process a saved card without holding a transaction across any I/O or hosted API.
  async function processCard(id, capture) {
    let row = store.card(id);
    if (!row || row.deletedNs || (row.flags & (131072 | 262144))) return;
    if (JSON.parse(row.extras).screenshotFailed) return;
    if (capture) {
      const baseline = { data: JSON.parse(row.data), extras: JSON.parse(row.extras) };
      await captureLink(baseline.data, { id, user: store.user(), media, offline, screenshot,
        update: patch => { if (!closed) writer.run(() => writer.enrich(id, patch, baseline)); } });
      if (closed) return;
      if (JSON.parse(store.card(id).extras).screenshotFailed) return;
      writer.run(() => writer.processing(id, false, true));
    }
    if (closed) return;
    row = store.card(id);
    if (!row || row.deletedNs) return;
    const saved = JSON.parse(row.data);
    // A blocked or failed capture leaves only the URL, from which the model invents a summary.
    if (JSON.parse(row.extras).captureFailed && !saved.description && !saved.prose) {
      if (row.flags & 1) writer.run(() => writer.processing(id, false));
      return;
    }
    if (!enabled()) {
      console.log(process.env.GEMINI_API_KEY ? '[ai] skipped: offline hosted services disabled' : '[ai] skipped: GEMINI_API_KEY is not set');
      if (row.flags & 1) writer.run(() => writer.processing(id, false));
      return;
    }
    writer.run(() => writer.processing(id, true));
    row = store.card(id);
    const before = signature(row, store);
    const data = JSON.parse(row.data), extras = JSON.parse(row.extras);
    try {
      const vocabulary = store.db.prepare(`SELECT name, COUNT(*) AS n FROM annotations WHERE user_id = ? GROUP BY name ORDER BY n DESC, name`)
        .all(store.defaultUserId()).map(tag => tag.name).filter(name => relevantTag(name, data.type));
      let image;
      try { image = await media.image(extras.cover ?? extras.post?.gallery?.[0] ?? data.blob); } catch {}
      const result = await analyze({ ...analysisItem(row, articleContent(row, store)), ...(image ? { image } : {}) }, { vocabulary });
      if (closed) return;
      const current = store.card(id);
      if (!current || current.deletedNs) return;
      if (signature(current, store) !== before) { schedule(id); return; }
      writer.run(() => writer.analysis(id, result));
    } catch {
      if (!closed) {
        console.log('[ai] analysis failed; saved card retained');
        writer.run(() => writer.processing(id, false));
      }
    }
  }

  // Start a bounded queue on the next event-loop turn, after the HTTP save has completed.
  function schedule(id, capture = false) {
    if (closed || readonly) return;
    pending.set(id, { capture: capture || pending.get(id)?.capture || false });
    if (running) return;
    running = new Promise(resolve => setImmediate(resolve)).then(async () => {
      while (!closed && pending.size) {
        const [id, task] = pending.entries().next().value;
        pending.delete(id);
        try { await processCard(id, task.capture); }
        catch { if (!closed) console.log('[capture] enrichment failed; saved card retained'); }
      }
    }).finally(() => { running = undefined; });
  }

  return {
    schedule, enabled,
    async drain() { while (running) await running; },
    close() { closed = true; pending.clear(); },
  };
}
