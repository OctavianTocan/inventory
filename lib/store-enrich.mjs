import { curateTags } from './ai-tags.mjs';

// Compare JSON metadata values without relying on object identity after SQLite decoding.
function same(a, b) { return JSON.stringify(a) === JSON.stringify(b); }

/**
 * Build synchronous enrichment mutations for use exclusively inside writer.run().
 * @param {object} store Open SQLite store.
 * @param {object} hooks Writer's card, save, now and emit transaction helpers.
 * @returns {object} enrich, analysis and processing mutations, all merging into current rows.
 */
export function createEnricher(store, { card, save, now, emit }) {
  const { db } = store;
  const user = store.user();
  return {
    // Merge only scraper-owned fields that have not changed since the preceding stage.
    enrich(id, patch, baseline) {
      const row = card(id);
      if (row.data.deleted || !same(row.data.source, baseline.data.source)) return false;
      const screenshotStarted = patch.extras?.screenshotPending && !row.extras.screenshotPending;
      const screenshotFailed = patch.extras?.screenshotFailed && !row.extras.screenshotFailed;
      const titleOwned = !row.extras.titleEdited && same(row.data.title, baseline.data.title);
      for (const [key, value] of Object.entries(patch.data ?? {})) {
        if (key === 'title' && !titleOwned) continue;
        if (same(row.data[key], baseline.data[key])) {
          row.data[key] = value;
          baseline.data[key] = structuredClone(value);
        }
      }
      for (const [key, value] of Object.entries(patch.extras ?? {})) {
        if (key === 'title' && !titleOwned) continue;
        if (same(row.extras[key], baseline.extras[key])) {
          row.extras[key] = value;
          baseline.extras[key] = structuredClone(value);
        }
      }
      if (row.data.type === 'Article') row.data.flags |= 2;
      if (screenshotStarted) row.data.flags |= 2097152 | 1;
      if (screenshotFailed) {
        row.data.flags |= 4194304;
        delete row.extras.capturePending;
      }
      save(row, screenshotStarted || screenshotFailed ? null : 'object:updated');
      if (screenshotStarted) emit('object:updated', { id, type: 'WebPage', flags: 2097152, bumped: '0001-01-01T00:00:00' });
      if (screenshotFailed) {
        if (!row.extras.captureText && !row.data.description && !row.data.prose) {
          emit('ai:failed', { objectId: id });
          emit('exception', { id, type: 'InvalidObjectException', message: 'no resource text' });
        }
        emit('screenshot:failed', { objectId: id });
      }
      return true;
    },

    // Clear processing state after capture, a missing key or a recoverable provider error.
    processing(id, enabled, captureDone = false) {
      const row = card(id);
      if (row.extras.screenshotFailed) return;
      row.data.flags = enabled ? row.data.flags | 1 : row.data.flags & ~1;
      if (captureDone) {
        delete row.extras.capturePending;
        if (row.extras.screenshotPending) {
          delete row.extras.screenshotPending;
          row.data.flags &= ~2097152;
        }
      }
      save(row);
    },

    // Store one complete AI result without changing manual annotations or resurrecting trash.
    analysis(id, result) {
      const row = card(id);
      if (row.data.deleted) return;
      const vector = Float32Array.from(result.embedding);
      if (vector.length !== 768 || !vector.every(Number.isFinite)) throw new TypeError('Invalid embedding');
      row.data.summary = String(result.summary).slice(0, 600);
      if (['XPost', 'ThreadsPost', 'SubstackNote'].includes(row.data.type) && !row.extras.titleEdited && row.data.title === row.extras.capturedTitle && result.title) {
        row.data.title = String(result.title).trim().split(/\s+/).slice(0, 8).join(' ');
        row.extras.title = row.data.title;
        row.extras.capturedTitle = row.data.title;
      }
      row.data.flags &= ~1;
      const vocabulary = db.prepare('SELECT name FROM annotations WHERE user_id = ? GROUP BY name ORDER BY COUNT(*) DESC, name').all(user.id).map(tag => tag.name);
      db.prepare('DELETE FROM annotations WHERE card_id = ? AND (flags & 2) != 0 AND (flags & 8) = 0').run(id);
      const existing = store.cardTags(id).map(tag => tag.name);
      let position = db.prepare('SELECT COALESCE(MAX(position), -1) + 1 AS n FROM annotations WHERE card_id = ?').get(id).n;
      for (const tag of curateTags(result.tags, row.data.type, vocabulary, existing)) {
        const name = String(tag.name).trim();
        const tagId = store.newId('tag');
        db.prepare('INSERT INTO annotations VALUES (?, ?, ?, ?, 2, ?)').run(tagId, user.id, id, name, position++);
        db.prepare('INSERT INTO tag_names (user_id, name, modified_ms) VALUES (?, ?, ?) ON CONFLICT DO NOTHING')
          .run(user.id, name, Number(now() / 1_000_000n));
        emit('tag:created', { id: tagId, name, objectId: id, flags: 2 });
      }
      const bytes = Buffer.alloc(768 * 4);
      vector.forEach((value, index) => bytes.writeFloatLE(value, index * 4));
      db.prepare('INSERT OR REPLACE INTO embeddings VALUES (?, ?, 768, ?)').run(id, user.id, bytes);
      save(row, 'object:analyzed');
    },
  };
}

