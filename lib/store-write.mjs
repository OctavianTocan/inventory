import { randomBytes } from 'node:crypto';
import { isoFromNs, nsFromIso } from './store.mjs';
import { renderCard } from './render-card.mjs';
import { createEnricher } from './store-enrich.mjs';
import { sourceUrl } from './capture-html.mjs';
import { createSearch } from './search.mjs';
import { createQueryChecker } from './store-query.mjs';
export { captureLink } from './capture-link.mjs';

const PRIVACY_TAGS = new Set(['private', 'nsfw', 'hidden']);

// Reject a request with the problem status understood by the API router.
function fail(status, detail) {
  throw Object.assign(new Error(detail), { status });
}

// Turn pasted plain text into one paragraph per line, preserving empty lines.
function textDocument(text) {
  return { type: 'doc', content: text.split(/\r\n|\r|\n/).map(line =>
    ({ type: 'paragraph', ...(line ? { content: [{ type: 'text', text: line }] } : {}) })) };
}

// Validate the document envelope before rendering or storing it.
function documentBody(body) {
  if (!body || body.type !== 'doc' || (body.content !== undefined && !Array.isArray(body.content))) {
    fail(400, 'Expected a prose document');
  }
  return body;
}

// Recognize nonempty document content before publishing a wiki stub.
function hasContent(node) {
  return Boolean(node.text?.trim() || ['image', 'wikiLink'].includes(node.type) || node.content?.some(hasContent));
}

/**
 * Build synchronous mutation operations on an open store. Call mutations only inside
 * `run`; it commits all rows together and emits queued hub events after commit.
 * @param {object} store Open store returned by openStore.
 * @param {(target:string,payload:object) => void} push Broadcast a committed hub event.
 * @returns {object} Transaction runner and card, annotation, space, settings and share mutations.
 */
export function createWriter(store, push) {
  const { db } = store;
  const user = store.user();
  const search = createSearch(store);
  let clock = BigInt(store.checkpoint());
  let events;

  // Allocate at least a millisecond between mutations because the UI compares Date.getTime().
  function now() {
    const checkpoint = BigInt(store.checkpoint());
    if (checkpoint > clock) clock = checkpoint;
    const wall = BigInt(Date.now()) * 1_000_000n;
    clock = wall > clock ? wall : clock + 1_000_000n;
    return clock;
  }

  // Queue a hub notification for delivery after the enclosing transaction commits.
  function emit(target, payload) {
    events.push([target, payload]);
  }

  // Record a checkpoint for the changed card or space.
  function change(id, objectType, type, ts) {
    db.prepare('INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, ?, ?, ?)')
      .run(user.id, ts, objectType, id, type);
  }

  // Require a card belonging to the current user and decode its editable state.
  function card(id) {
    const row = store.card(id);
    if (!row) fail(404, 'Card not found');
    return { ...row, data: JSON.parse(row.data), extras: JSON.parse(row.extras) };
  }

  // Require a space belonging to the current user.
  function space(id) {
    const result = store.space(id);
    if (!result || !db.prepare('SELECT id FROM spaces WHERE id = ? AND user_id = ?').get(id, user.id)) {
      fail(404, 'Space not found');
    }
    return result;
  }

  // Save an edited card and its independently rendered grid markup.
  function save(row, event = 'object:updated', type = row.data.deleted ? 2 : 1) {
    const ts = now();
    row.data.modified = isoFromNs(ts, 6);
    row.extras.local = true;
    const tags = store.cardTags(row.id);
    if (tags.length) row.data.tags = tags;
    else delete row.data.tags;
    const html = renderCard(row.data, row.extras);
    db.prepare(`UPDATE cards SET type = ?, flags = ?, data = ?, html = ?, extras = ?,
      bumped_ns = ?, modified_ns = ?, deleted_ns = ? WHERE id = ? AND user_id = ?`)
      .run(row.data.type, row.data.flags, JSON.stringify(row.data), html, JSON.stringify(row.extras),
        row.bumpedNs, ts, row.data.deleted ? nsFromIso(row.data.deleted) : null, row.id, user.id);
    change(row.id, 9, type, ts);
    if (event) emit(event, { id: row.id });
    return row;
  }

  // Mark a space stale for both polling and connected tabs.
  function saveSpace(id, event = 'space:updated', type = 1) {
    db.prepare('DELETE FROM api_metadata WHERE path = ?').run(`/spaces/${id}`);
    change(id, 190, type, now());
    emit(event, { id });
  }

  // Persist the current editable body as a revision.
  function revision(id, prose) {
    db.prepare('INSERT INTO revisions VALUES (?, ?, ?, ?)')
      .run(randomBytes(16).toString('hex'), id, isoFromNs(now(), 6), JSON.stringify(prose));
  }

  // Preserve an imported document before its first local edit so it can be restored.
  function preserveBody(row) {
    if (db.prepare('SELECT id FROM revisions WHERE card_id = ? LIMIT 1').get(row.id)) return;
    const content = db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
    const before = content ? JSON.parse(content.body) : row.data.prose;
    if (before) revision(row.id, before);
  }

  // Add a card to a manual space without replacing its existing rank.
  function addToSpace(spaceId, id) {
    space(spaceId);
    card(id);
    const rank = db.prepare('SELECT COALESCE(MAX(rank), -1) + 1 AS n FROM space_cards WHERE space_id = ?').get(spaceId).n;
    db.prepare(`INSERT INTO space_cards (space_id, card_id, user_id, rank, score, excluded)
      VALUES (?, ?, ?, ?, 1, 0) ON CONFLICT(space_id, card_id) DO UPDATE SET excluded = 0`)
      .run(spaceId, id, user.id, rank);
    saveSpace(spaceId);
    save(card(id));
  }

  const writer = {
    ...createEnricher(store, { card, save, now, emit }),
    checkQuery: createQueryChecker(store, search, { now, save, saveSpace, card }),
    // Commit a mutation group before broadcasting its queued events.
    run(work) {
      events = [];
      const result = store.transaction(work);
      const committed = events;
      events = undefined;
      for (const [target, payload] of committed) push(target, payload);
      return result;
    },

    // Save a note, plain text or URL, bumping an existing URL when present.
    create(body, media) {
      if ((body.uploadId || body.type === 'Media') && !media) fail(400, 'A completed upload is required');
      let url;
      if (typeof body.content === 'string' && /^https?:\/\/\S+$/.test(body.content.trim())) {
        try { url = new URL(body.content.trim()); } catch { fail(400, 'Invalid URL'); }
      }
      if (url) {
        const existing = db.prepare(`SELECT id, json_extract(data, '$.source.url') AS url FROM cards
          WHERE user_id = ? AND deleted_ns IS NULL AND json_extract(data, '$.source.url') IS NOT NULL
          ORDER BY bumped_ns DESC`).all(user.id).find(row => {
          try { return new URL(row.url).href === url.href; } catch { return false; }
        });
        if (existing) {
          const row = card(existing.id);
          row.bumpedNs = now();
          row.data.bumped = isoFromNs(row.bumpedNs);
          save(row, 'object:bumped');
          if (body.spaceId) addToSpace(body.spaceId, row.id);
          return { status: 200, body: { id: row.id } };
        }
      }
      const id = media?.id ?? store.newId('card');
      const ts = now();
      let data = {
        type: url ? (body.type === 'Screenshot' ? 'Screenshot' : 'Placeholder') : 'Note', flags: url && body.type === 'Screenshot' ? 1 : 0,
        modified: isoFromNs(ts, 6), bumped: isoFromNs(ts), created: isoFromNs(ts),
      };
      if (url) data = { ...data, ...(body.type === 'Screenshot' ? { title: url.host } : {}), domain: url.hostname, siteName: url.hostname,
        source: { name: body.source ?? 'clipboard', url: sourceUrl(body.content.trim(), body.content.trim()) ?? url.href } };
      else if (media) Object.assign(data, media.data);
      else data.prose = documentBody(body.prose ?? textDocument(body.content ?? ''));
      if (media && body.source) data.source = { name: String(body.source) };
      if (body.title !== undefined) data.title = String(body.title);
      if (body.isDraft) data.flags |= 131072;
      if (body.isStub) data.flags |= 262144;
      const extras = { id, local: true, ...(url ? { capturePending: true } : {}), ...(body.title !== undefined ? { titleEdited: true } : {}), ...media?.extras };
      db.prepare(`INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`)
        .run(id, user.id, data.type, data.flags, JSON.stringify(data), renderCard(data, extras),
          JSON.stringify(extras), ts, ts, ts);
      change(id, 9, 1, ts);
      emit('object:created', { id });
      if (data.prose) revision(id, data.prose);
      if (body.spaceId) addToSpace(body.spaceId, id);
      return { status: 201, body: { id } };
    },

    // Apply editable card fields without changing its grid bump time.
    patch(id, body) {
      const row = card(id);
      if (Object.hasOwn(body, 'flags') && (!Number.isInteger(body.flags) || body.flags < 0)) fail(400, 'Invalid card flags');
      if (Object.hasOwn(body, 'prose')) {
        documentBody(body.prose);
        preserveBody(row);
        revision(id, body.prose);
        if (db.prepare('SELECT card_id FROM card_content WHERE card_id = ?').get(id)) {
          db.prepare('UPDATE card_content SET body = ? WHERE card_id = ?').run(JSON.stringify(body.prose), id);
        }
      }
      for (const key of ['title', 'description', 'flags', 'prose', 'source', 'siteName', 'domain', 'ocr', 'author', 'brand', 'format', 'objectNames']) {
        if (Object.hasOwn(body, key)) row.data[key] = body[key];
      }
      if (Object.hasOwn(body, 'title')) {
        row.extras.titleEdited = true;
        delete row.extras.title;
        if (row.extras.reddit) row.extras.reddit.title = body.title;
      }
      if (Object.hasOwn(body, 'description') && row.extras.post) delete row.extras.post.bodyHtml;
      if (Object.hasOwn(body, 'author') && row.extras.post) row.extras.post.author = body.author?.name ?? body.author;
      if (body.prose && hasContent(body.prose)) row.data.flags &= ~262144;
      save(row);
      return {};
    },

    // Save a document in its editor resource and refresh the card.
    prose(id, body, kind = 'prose') {
      documentBody(body);
      const row = card(id);
      if (kind !== 'note' && hasContent(body)) row.data.flags &= ~262144;
      let result = { id };
      if (kind === 'note') {
        const noteId = row.data.note?.id ?? store.newId('note');
        row.data.note = { id: noteId, prose: body, modified: isoFromNs(now(), 6) };
        result = { id: noteId };
      } else if (kind === 'content') {
        preserveBody(row);
        db.prepare('INSERT OR REPLACE INTO card_content VALUES (?, ?)').run(id, JSON.stringify(body));
        if (['Note', 'Content'].includes(row.data.type)) row.data.prose = body;
        revision(id, body);
      } else {
        preserveBody(row);
        row.data.prose = body;
        if (db.prepare('SELECT card_id FROM card_content WHERE card_id = ?').get(id)) {
          db.prepare('UPDATE card_content SET body = ? WHERE card_id = ?').run(JSON.stringify(body), id);
        }
        revision(id, body);
        if (kind === 'commit') {
          row.data.flags &= ~131072;
          row.bumpedNs = now();
          row.data.bumped = isoFromNs(row.bumpedNs);
        }
      }
      save(row);
      return result;
    },

    // Publish a draft with the creation status used by the composer.
    commit(id, body) {
      const wasDraft = Boolean(card(id).data.flags & 131072);
      return { status: wasDraft ? 201 : 200, body: writer.prose(id, body, 'commit') };
    },

    // Create a per-card annotation unless that name already exists.
    addTag(id, name) {
      const row = card(id);
      if (typeof name !== 'string' || !name.trim()) fail(400, 'Tag name is required');
      name = name.replace(/^#/, '').trim();
      if (!name) fail(400, 'Tag name is required');
      const existing = store.cardTags(id).find(tag => tag.name === name);
      if (existing) return existing;
      const tag = { id: store.newId('tag'), name, ...(name === 'highlight' ? {} : { flags: 8 }) };
      const position = db.prepare('SELECT COALESCE(MAX(position), -1) + 1 AS n FROM annotations WHERE card_id = ?').get(id).n;
      db.prepare('INSERT INTO annotations VALUES (?, ?, ?, ?, ?, ?)').run(tag.id, user.id, id, name, tag.flags ?? 0, position);
      if (PRIVACY_TAGS.has(name)) row.data.flags |= 8;
      db.prepare(`INSERT INTO tag_names (user_id, name, modified_ms) VALUES (?, ?, ?)
        ON CONFLICT(user_id, name) DO UPDATE SET modified_ms = excluded.modified_ms`)
        .run(user.id, name, Number(now() / 1_000_000n));
      save(row);
      emit('tag:created', { ...tag, objectId: id, flags: tag.flags ?? 0 });
      return tag;
    },

    // Remove only the annotation identified by its per-card id.
    removeTag(id) {
      const tag = db.prepare('SELECT card_id, name FROM annotations WHERE id = ? AND user_id = ?').get(id, user.id);
      if (!tag) fail(404, 'Annotation not found');
      db.prepare('DELETE FROM annotations WHERE id = ?').run(id);
      const row = card(tag.card_id);
      if (PRIVACY_TAGS.has(tag.name) && !store.cardTags(row.id).some(item => PRIVACY_TAGS.has(item.name))) row.data.flags &= ~8;
      save(row);
      emit('tag:deleted', { objectId: tag.card_id, name: tag.name });
      return {};
    },

    // Toggle a client-visible card flag and maintain pin membership.
    flag(id, bit, enabled, position = 1) {
      const row = card(id);
      row.data.flags = enabled ? row.data.flags | bit : row.data.flags & ~bit;
      if (bit === 512) {
        const settings = store.user().settings;
        settings.pinsChanged = true;
        db.prepare('UPDATE users SET settings = ? WHERE id = ?').run(JSON.stringify(settings), user.id);
        db.prepare('DELETE FROM pins WHERE card_id = ?').run(id);
        if (enabled) {
          const rank = Math.max(0, Number.isInteger(position) ? position - 1 : 0);
          db.prepare('UPDATE pins SET position = position + 1 WHERE position >= ?').run(rank);
          db.prepare('INSERT INTO pins VALUES (?, ?)').run(id, rank);
          row.extras.pinPosition = rank;
        } else delete row.extras.pinPosition;
      }
      save(row);
      return {};
    },

    // Persist the requested pin order and notify other tabs.
    reorderPins(ids) {
      if (!Array.isArray(ids)) fail(400, 'Expected an array of card ids');
      const settings = store.user().settings;
      settings.pinsChanged = true;
      db.prepare('UPDATE users SET settings = ? WHERE id = ?').run(JSON.stringify(settings), user.id);
      ids.forEach((id, position) => {
        const row = card(id);
        row.data.flags |= 512;
        row.extras.pinPosition = position;
        db.prepare('INSERT OR REPLACE INTO pins VALUES (?, ?)').run(id, position);
        save(row);
      });
      return {};
    },

    // Move a card into trash or restore it to the active library.
    trash(id, restore = false) {
      const row = card(id);
      if (restore) delete row.data.deleted;
      else { row.data.deleted = isoFromNs(now()); row.data.flags &= ~1; }
      save(row, restore ? 'object:updated' : 'object:deleted', restore ? 1 : 2);
      return {};
    },

    // Create a manual space or evaluate smart filters against the current store.
    createSpace(body) {
      if (typeof body.name !== 'string' || !body.name.trim()) fail(400, 'Space name is required');
      const id = store.newId('space');
      const created = isoFromNs(now(), 6);
      const query = body.query ? { id: store.newId('query'), filters: body.query.filters ?? [], checked: isoFromNs(now()) } : null;
      const position = body.position ?? (query ? -Math.floor(Date.parse(created) / 1000)
        : db.prepare('SELECT COALESCE(MAX(position), 0) + 1 AS n FROM spaces WHERE user_id = ?').get(user.id).n);
      db.prepare('INSERT INTO spaces VALUES (?, ?, ?, ?, ?, ?, ?)')
        .run(id, user.id, body.name, body.color ?? '#e8c4a0', position, created, query ? JSON.stringify(query) : null);
      for (const [rank, member] of (query ? search.filters(query.filters) : []).entries()) {
        card(member.id);
        db.prepare('INSERT INTO space_cards VALUES (?, ?, ?, ?, ?, ?, 0)')
          .run(id, member.id, user.id, rank, member.score ?? 1, query.id);
        save(card(member.id));
      }
      saveSpace(id, 'space:created');
      return store.space(id);
    },

    // Rename or recolor an existing space.
    patchSpace(id, body) {
      const row = space(id);
      db.prepare('UPDATE spaces SET name = ?, color = ? WHERE id = ?')
        .run(body.name ?? row.name, body.color ?? row.color, id);
      saveSpace(id);
      return {};
    },

    // Assign ascending positions to the supplied space ids.
    reorderSpaces(ids) {
      if (!Array.isArray(ids)) fail(400, 'Expected an array of space ids');
      ids.forEach((id, position) => {
        space(id);
        db.prepare('UPDATE spaces SET position = ? WHERE id = ?').run(position + 1, id);
        saveSpace(id);
      });
      return {};
    },

    // Delete a space while retaining its member cards.
    deleteSpace(id) {
      const row = space(id);
      db.prepare('DELETE FROM spaces WHERE id = ?').run(id);
      saveSpace(id, 'space:deleted', 2);
      for (const member of row.objects) save(card(member.id));
      return {};
    },

    // Add or remove a manual membership and refresh affected readers.
    membership(spaceId, id, remove = false) {
      if (!remove) addToSpace(spaceId, id);
      else {
        space(spaceId);
        card(id);
        db.prepare('DELETE FROM space_cards WHERE space_id = ? AND card_id = ?').run(spaceId, id);
        saveSpace(spaceId);
        save(card(id));
      }
      return {};
    },

    // Retain a smart-space exclusion without recomputing search matches.
    exclude(queryId, id) {
      const row = db.prepare("SELECT id FROM spaces WHERE user_id = ? AND json_extract(query, '$.id') = ?").get(user.id, queryId);
      if (!row) fail(404, 'Query not found');
      card(id);
      db.prepare('UPDATE space_cards SET excluded = 1 WHERE space_id = ? AND card_id = ?').run(row.id, id);
      saveSpace(row.id);
      save(card(id));
      return {};
    },

    // Persist supported settings and notify the account refresh listener.
    settings(body) {
      const current = store.user();
      for (const key of ['colorScheme', 'locale', 'timeZoneId', 'soundEnabled', 'onboardFlags']) {
        if (Object.hasOwn(body, key)) current.settings[key] = body[key];
      }
      if (Object.hasOwn(body, 'soundEnabled')) current.flags = body.soundEnabled ? current.flags & ~64 : current.flags | 64;
      db.prepare('UPDATE users SET settings = ?, flags = ? WHERE id = ?')
        .run(JSON.stringify(current.settings), current.flags, user.id);
      emit('account:updated', { id: user.accountUid });
      return { status: 200 };
    },

    // Retain local dismissal or completion state in user settings.
    remember(key, id) {
      const current = store.user();
      current.settings[key] = [...new Set([...(current.settings[key] ?? []), id])];
      db.prepare('UPDATE users SET settings = ? WHERE id = ?').run(JSON.stringify(current.settings), user.id);
      return {};
    },

    // Set or clear the requested onboarding bit.
    onboard(value, enabled) {
      const flags = store.user().settings.onboardFlags ?? 0;
      writer.settings({ onboardFlags: enabled ? flags | value : flags & ~value });
      return {};
    },

    // Create a local share with a 24-hour expiry.
    share(id) {
      card(id);
      const shareId = randomBytes(16).toString('hex');
      const ts = now();
      db.prepare('INSERT INTO shares VALUES (?, ?, ?, ?)').run(shareId, id, ts, ts + 86_400_000_000_000n);
      save(card(id));
      return { id: shareId, url: `/shares/${shareId}` };
    },

    // Remove a share belonging to the specified card.
    revokeShare(id, shareId) {
      card(id);
      db.prepare('DELETE FROM shares WHERE id = ? AND card_id = ?').run(shareId, id);
      save(card(id));
      return {};
    },

    // Restore a recorded local document revision to its editor resource.
    restoreRevision(id, revisionId) {
      card(id);
      const row = db.prepare('SELECT body FROM revisions WHERE id = ? AND card_id = ?').get(revisionId, id);
      if (!row) fail(404, 'Revision not found');
      const kind = db.prepare('SELECT card_id FROM card_content WHERE card_id = ?').get(id) ? 'content' : 'prose';
      return writer.prose(id, JSON.parse(row.body), kind);
    },

    // Remove generated cover metadata without running analysis.
    removeThumbnail(id) {
      const row = card(id);
      delete row.extras.cover;
      row.data.flags &= ~8388608;
      save(row);
      return {};
    },
  };
  return writer;
}
