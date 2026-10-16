import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { openStore, nsFromIso } from './store.mjs';
import { createWriter } from './store-write.mjs';
import { importReadMetadata } from './read-metadata.mjs';
import { deviceVariant } from '../tools/replay-store.mjs';
import { renderDetail, renderPinned, renderShare } from './render/views.mjs';
import { createMedia } from './local-media.mjs';
import { createUploads } from './uploads.mjs';
import { createEnrichment } from './enrichment.mjs';
import { createMediaWrites } from './media-writes.mjs';
import { createSearch } from './search.mjs';
import { createAppApi } from './app-api.mjs';

// Parse a bounded JSON request without including private request data in errors.
async function requestBody(req) {
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > 8 * 1024 * 1024) throw Object.assign(new Error('Request body too large'), { status: 413 });
    chunks.push(chunk);
  }
  const text = Buffer.concat(chunks).toString('utf8');
  if (req.headers['content-type']?.startsWith('text/markdown')) return text;
  if (!text) return {};
  try { return JSON.parse(text); }
  catch { throw Object.assign(new Error('Invalid JSON body'), { status: 400 }); }
}

// Extract the current card model used by both object reads and change messages.
function objectModel(store, row) {
  const data = JSON.parse(row.data);
  const extras = JSON.parse(row.extras);
  const spaces = store.db.prepare('SELECT space_id AS id FROM space_cards WHERE card_id = ? AND excluded = 0 ORDER BY rank').all(row.id);
  return {
    ...data, id: row.id, entityType: data.type, spaces,
    mainEntity: { '@type': data.type, '@id': extras.linkUrl ?? data.source?.url ?? row.id,
      title: data.title ?? '', ...(data.author ? { author: data.author } : {}),
      ...(extras.cover ? { cover: extras.cover } : {}) },
    ...(extras.pinPosition !== undefined ? { position: extras.pinPosition } : {}),
  };
}

// Convert markdown blocks the local editor supports into prose without external services.
function convertMarkdown(text) {
  const inline = value => {
    const parts = value.split(/(\*\*[^*]+\*\*|`[^`]+`|\*[^*]+\*|\[[^\]]+\]\([^)]+\))/g).filter(Boolean);
    return parts.map(part => {
      const link = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(part);
      if (link) return { type: 'text', text: link[1], marks: [{ type: 'link', attrs: { href: link[2] } }] };
      const mark = part.startsWith('**') ? 'bold' : part.startsWith('`') ? 'code' : part.startsWith('*') ? 'italic' : null;
      const trim = mark === 'bold' ? 2 : 1;
      return { type: 'text', text: mark ? part.slice(trim, -trim) : part, ...(mark ? { marks: [{ type: mark }] } : {}) };
    });
  };
  const content = [];
  let code;
  for (const line of text.split(/\r?\n/)) {
    if (line.startsWith('```')) {
      if (code) { content.push({ type: 'codeBlock', content: [{ type: 'text', text: code.join('\n') }] }); code = undefined; }
      else code = [];
    } else if (code) code.push(line);
    else if (/^#{1,6} /.test(line)) content.push({ type: 'heading', attrs: { level: line.indexOf(' ') }, content: inline(line.slice(line.indexOf(' ') + 1)) });
    else if (/^([-*] |\d+\. )/.test(line)) {
      const type = /^\d/.test(line) ? 'orderedList' : 'bulletList';
      if (content.at(-1)?.type !== type) content.push({ type, content: [] });
      content.at(-1).content.push({ type: 'listItem', content: [{ type: 'paragraph', content: inline(line.replace(/^([-*] |\d+\. )/, '')) }] });
    } else if (line.startsWith('> ')) content.push({ type: 'blockquote', content: [{ type: 'paragraph', content: inline(line.slice(2)) }] });
    else if (/^---+$/.test(line)) content.push({ type: 'horizontalRule' });
    else content.push({ type: 'paragraph', ...(line ? { content: inline(line) } : {}) });
  }
  if (code) content.push({ type: 'codeBlock', content: code.length ? [{ type: 'text', text: code.join('\n') }] : [] });
  return { type: 'doc', content };
}

// Replace the Top of Mind contents while respecting nested divs in card previews.
function pinnedList(html, contents) {
  const start = /<div\b[^>]*class="[^"]*\btom__list\b(?!_)[^"]*"[^>]*>/.exec(html);
  if (!start) return html;
  const offset = start.index + start[0].length;
  const tags = /<\/?div\b[^>]*>/g;
  tags.lastIndex = offset;
  let depth = 1;
  for (let match; (match = tags.exec(html));) {
    depth += match[0].startsWith('</') ? -1 : 1;
    if (depth === 0) return html.slice(0, offset) + contents + html.slice(match.index);
  }
  return html;
}

/**
 * Open store-backed API routing when a seeded database exists; otherwise return null.
 * Unsupported reads return false so the server retains its existing replay fallback.
 * @param {object} options Server integration callbacks and paths.
 * @param {string} options.root Project directory.
 * @param {string} [options.dbPath] Database file; defaults to data/mymind.db.
 * @param {boolean} [options.readonly=false] Acknowledge writes without parsing or persisting them.
 * @param {object} options.msgpack Mirrored MessagePack codec, with BigInt and timestamp extensions.
 * @param {(text:string) => string} options.localize Rewrite recorded origins to local paths.
 * @param {(res:object,status:number,body:Buffer|string,headers?:object) => void} options.send HTTP sender.
 * @param {(target:string,payload:object) => void} options.push Committed SignalR broadcaster.
 * @param {(req:object,url:URL)=>boolean} [options.hasReplay] Whether the server can replay this exact request.
 * @returns {Promise<object|null>} Handle with `handle`, `shell` and `close`, or null without a database.
 */
export async function createApi({ root, dbPath = join(root, 'data/mymind.db'), readonly = false, msgpack, localize, send, push, hasReplay = () => false }) {
  if (!existsSync(dbPath)) return null;
  const store = openStore(dbPath);
  await importReadMetadata(store, root);
  const writer = createWriter(store, push);
  const search = createSearch(store, { offline: process.env.OFFLINE === '1' });
  const appApi = createAppApi({ store, search, localize, send });
  const media = createMedia(process.env.STORE_MEDIA ?? join(dirname(dbPath), 'media'));
  const uploads = createUploads(store, join(dirname(dbPath), 'uploads'), media);
  if (!readonly) await uploads.sweep();
  const jobs = createEnrichment({ store, writer, media, readonly, offline: process.env.OFFLINE === '1' });
  if (!readonly) for (const row of store.db.prepare("SELECT id, extras FROM cards WHERE json_extract(extras, '$.local') = 1 AND ((flags & 1) != 0 OR json_extract(extras, '$.capturePending') = 1)").all()) {
    jobs.schedule(row.id, Boolean(JSON.parse(row.extras).capturePending));
  }
  const pack = items => Buffer.concat(items.map(item => Buffer.from(msgpack.encode(item, { useBigInt64: true }))));
  const cardValue = row => ({ html: localize(row.html), json: localize(row.data) });
  const quota = () => ({ 'x-object-quota': JSON.stringify(store.quota()) });
  const jsonHeaders = { 'content-type': 'application/json; charset=utf-8' };
  const mediaWrites = createMediaWrites({ store, writer, media, uploads, jobs, offline: process.env.OFFLINE === '1',
    reply: (res, status, body) => send(res, status, JSON.stringify(body), { ...jsonHeaders, ...quota() }) });
  const requireCard = id => {
    const row = store.card(id);
    if (!row) throw Object.assign(new Error('Card not found'), { status: 404 });
    return row;
  };
  const timestamp = ns => new msgpack.ExtData(-1, msgpack.encodeTimeSpecToTimestamp({
    sec: Number(ns / 1_000_000_000n), nsec: Number(ns % 1_000_000_000n),
  }));

  // Read share timestamps as BigInts so SQLite never rounds or rejects nanoseconds.
  function activeShare(id, byCard = false) {
    const statement = store.db.prepare(`SELECT * FROM shares WHERE ${byCard ? 'card_id' : 'id'} = ?
      AND expires_ns > ? ORDER BY created_ns DESC LIMIT 1`);
    statement.setReadBigInts(true);
    return statement.get(id, BigInt(Date.now()) * 1_000_000n);
  }

  // Encode a poll item with exact uint64 checkpoints and MessagePack timestamp dates.
  function changeValue(change) {
    let object = change.objectType === 190 ? store.space(change.objectId) ?? { id: change.objectId }
      : change.card ? objectModel(store, change.card) : { id: change.objectId };
    object = { ...object };
    for (const key of ['created', 'bumped', 'modified', 'deleted']) {
      if (object[key]) object[key] = timestamp(nsFromIso(object[key]));
    }
    const hiddenDraft = change.card && (change.card.flags & 131072) !== 0;
    return { type: hiddenDraft ? 2 : change.type, timestamp: change.ts, objectType: change.objectType, object,
      cardHtml: change.card ? localize(change.card.html) : '', cardData: change.card ? localize(change.card.data) : '' };
  }

  // Restore recorded transport headers while generating current bodies and quota headers.
  function reply(res, url, body, { status = 200, headers = jsonHeaders } = {}) {
    const saved = store.db.prepare('SELECT headers FROM api_metadata WHERE path = ?').get(url.pathname + url.search);
    send(res, status, body, { ...(saved ? JSON.parse(saved.headers) : {}), ...headers });
  }

  // Return a route result without reading a request body until a write matches.
  function mutation(method, path) {
    let match;
    if (method === 'POST' && path === '/objects') return body => writer.create(body);
    if (method === 'POST' && path === '/spaces') return body => ({ body: writer.createSpace(body) });
    if (method === 'POST' && path === '/spaces/reorder') return body => ({ body: writer.reorderSpaces(body) });
    if (method === 'POST' && path === '/objects/reorder-pins') return body => ({ body: writer.reorderPins(body) });
    if (method === 'POST' && path === '/convert') return body => ({ body: convertMarkdown(body), headers: { 'content-type': 'application/prose+json' } });
    if (method === 'POST' && path === '/jserrors') return () => ({ body: {} });
    if (method === 'POST' && path === '/serendipity/complete') return () => ({ body: writer.remember('completedFlows', 'serendipity') });
    if (method === 'POST' && (match = /^\/promotions\/([^/]+)\/dismiss$/.exec(path))) {
      return () => ({ body: writer.remember('dismissedPromotions', match[1]) });
    }
    if (method === 'POST' && (match = /^\/batch\/(tag|delete|add-to-space)$/.exec(path))) return body => {
      if (!Array.isArray(body.objectIds)) throw Object.assign(new Error('Expected objectIds array'), { status: 400 });
      for (const id of body.objectIds) {
        if (match[1] === 'tag') writer.addTag(id, body.tag);
        else if (match[1] === 'delete') writer.trash(id);
        else writer.membership(body.spaceId, id);
      }
      return { body: {} };
    };
    if ((match = /^\/objects\/([^/]+)$/.exec(path))) {
      if (method === 'PATCH') return body => ({ body: writer.patch(match[1], body) });
      if (method === 'DELETE') return () => ({ body: writer.trash(match[1]) });
    }
    if ((match = /^\/objects\/([^/]+)\/(prose|note|content|entity|commit|tags|restore|pin|unpin|complete|uncomplete)$/.exec(path))) {
      const [, id, action] = match;
      if ((method === 'PUT' && ['prose', 'note', 'content'].includes(action)) || (method === 'PATCH' && action === 'prose')) {
        return body => ({ body: writer.prose(id, body, action) });
      }
      if (method === 'PATCH' && action === 'entity') return body => ({ body: writer.patch(id, body) });
      if (method === 'PATCH' && action === 'commit') return body => writer.commit(id, body.prose);
      if (method === 'POST') {
        if (action === 'tags') return body => ({ body: writer.addTag(id, body.name) });
        if (action === 'commit') return body => writer.commit(id, body.prose);
        if (action === 'restore') return () => ({ body: writer.trash(id, true) });
        if (['pin', 'unpin', 'complete', 'uncomplete'].includes(action)) return body =>
          ({ body: writer.flag(id, action.includes('pin') ? 512 : 65536, action === 'pin' || action === 'complete', body.position) });
      }
    }
    if (method === 'DELETE' && (match = /^\/annotations\/([^/]+)$/.exec(path))) return () => ({ body: writer.removeTag(match[1]) });
    if ((match = /^\/spaces\/([^/]+)$/.exec(path))) {
      if (method === 'PATCH') return body => ({ body: writer.patchSpace(match[1], body) });
      if (method === 'DELETE') return () => ({ body: writer.deleteSpace(match[1]) });
    }
    if (['PUT', 'DELETE'].includes(method) && (match = /^\/spaces\/([^/]+)\/objects\/([^/]+)$/.exec(path))) {
      return () => ({ body: writer.membership(match[1], match[2], method === 'DELETE') });
    }
    if (method === 'POST' && (match = /^\/queries\/([^/]+)\/objects\/([^/]+)\/exclude$/.exec(path))) {
      return () => ({ body: writer.exclude(match[1], match[2]) });
    }
    if (method === 'PATCH' && (match = /^\/users\/([^/]+)$/.exec(path))) return body => {
      if (![store.user().id, store.user().accountUid].includes(match[1])) throw Object.assign(new Error('User not found'), { status: 404 });
      return { body: writer.settings(body) };
    };
    if (method === 'POST' && (match = /^\/onboard\/flags\/(\d+)\/(set|unset)$/.exec(path))) {
      return () => ({ body: writer.onboard(Number(match[1]), match[2] === 'set') });
    }
    if (method === 'POST' && (match = /^\/objects\/([^/]+)\/revisions\/([^/]+)\/restore$/.exec(path))) {
      return () => ({ body: writer.restoreRevision(match[1], match[2]) });
    }
    if (method === 'POST' && (match = /^\/cards\/([^/]+)\/shares$/.exec(path))) return () => ({ body: writer.share(match[1]) });
    if (method === 'DELETE' && (match = /^\/cards\/([^/]+)\/shares\/([^/]+)$/.exec(path))) {
      return () => ({ body: writer.revokeShare(match[1], match[2]) });
    }
    if (method === 'POST' && (match = /^\/cards\/([^/]+)\/remove-ai-thumbnail$/.exec(path))) {
      return () => ({ body: writer.removeThumbnail(match[1]) });
    }
    return null;
  }

  return {
    close: () => { jobs.close(); store.close(); },

    shell(html) {
      if (readonly) return html;
      const localPin = store.user().settings.pinsChanged || store.db.prepare("SELECT id FROM cards WHERE json_extract(extras, '$.local') = 1 AND (flags & 512) != 0 LIMIT 1").get();
      if (localPin) {
        const pinned = store.db.prepare(`SELECT c.id FROM cards c LEFT JOIN pins p ON p.card_id = c.id
          WHERE c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & 512) != 0 AND (c.flags & 393224) = 0
          ORDER BY COALESCE(p.position, 2147483647), c.bumped_ns DESC, c.id DESC`).all(store.defaultUserId());
        html = pinnedList(html, pinned.map(row => localize(renderPinned(store.card(row.id)))).join(''));
      }
      const user = store.user();
      // Onboarding flag 128 is the one the client sets after showing the Top of Mind introduction.
      if ((user.settings.onboardFlags ?? 0) & 128) html = html.replace('<mymind-top-of-mind class="tom ">', '<mymind-top-of-mind class="tom introduced">');
      return html.replace(/(<script id="config" type="application\/json">)([\s\S]*?)(<\/script>)/, (_, start, text, end) => {
        const config = JSON.parse(text);
        for (const key of ['colorScheme', 'locale', 'timeZoneId', 'soundEnabled', 'onboardFlags']) {
          if (Object.hasOwn(user.settings, key)) config[key] = user.settings[key];
        }
        config.user.flags = user.flags;
        config.quotas.objects = store.quota();
        return start + JSON.stringify(config).replaceAll('<', '\\u003c') + end;
      });
    },

    async handle(req, res, url) {
      const path = decodeURI(url.pathname);
      const method = req.method;
      const phone = Boolean(deviceVariant(req.headers['user-agent']));
      let match;
      try {
        if (await appApi.handle(req, res, url)) return true;
        if (['GET', 'HEAD'].includes(method) && (path.startsWith('/media/') || path.startsWith('/_mediahost/'))) {
          const result = await media.response(path.replace(/^\/(?:media|_mediahost)\//, ''));
          if (!result) return false;
          await media.send(res, result, { method, range: req.headers.range });
          return true;
        }
        if (!readonly && method === 'POST' && path === '/uploads/authorize') {
          send(res, 200, JSON.stringify(await uploads.authorize(url.searchParams.get('count'))), { ...jsonHeaders, ...quota() });
          return true;
        }
        if (!readonly && await mediaWrites(req, res, url)) return true;
        if (!readonly && method === 'PUT' && (match = /^\/uploads\/([a-z0-9]+)$/.exec(path))) {
          send(res, 200, JSON.stringify(await uploads.put(match[1], req)), { ...jsonHeaders, ...quota() });
          return true;
        }
        if (!readonly && method === 'POST' && (match = /^\/cards\/([^/]+)\/analyze$/.exec(path))) {
          const row = requireCard(match[1]);
          const data = JSON.parse(row.data);
          jobs.schedule(row.id);
          send(res, 200, JSON.stringify(jobs.enabled() ? { status: 'Processing' } : { status: 'Skipped', summary: data.summary ?? '', tags: store.cardTags(row.id) }), { ...jsonHeaders, ...quota() });
          return true;
        }
        if (method === 'GET' && url.searchParams.get('dismiss') === 'true' && (match = /^\/achievements\/([^/]+)$/.exec(path))) {
          const body = readonly ? {} : writer.run(() => writer.remember('dismissedAchievements', match[1]));
          reply(res, url, JSON.stringify(body));
          return true;
        }
        if (!phone && method === 'GET' && (match = /^\/shares\/([^/]+)$/.exec(path))) {
          const share = activeShare(match[1]);
          if (!share || !store.card(share.card_id) || store.card(share.card_id).deletedNs !== null) {
            send(res, 410, 'Share expired or revoked', { 'content-type': 'text/plain; charset=utf-8' });
          } else send(res, 200, localize(renderShare(store.card(share.card_id))), { 'content-type': 'text/html; charset=utf-8' });
          return true;
        }
        if (method === 'GET' && req.headers['sec-fetch-dest'] === 'document') return false;
        if (method === 'GET' && (['/search', '/search/suggestions'].includes(path) || /^\/queries\/[^/]+\/check$/.test(path)) && readonly && hasReplay(req, url)) return false;
        if (method === 'GET' && path === '/search') {
          const options = Object.fromEntries(url.searchParams);
          options.wildcard = options.wildcard === 'true'; options.semantic = options.semantic === 'true';
          reply(res, url, JSON.stringify(await search.search(options.q ?? '', options)));
        } else if (method === 'GET' && path === '/search/suggestions') {
          reply(res, url, JSON.stringify(search.suggestions(url.searchParams.get('q') ?? '')));
        } else if (method === 'GET' && path === '/embeddings/search') {
          reply(res, url, JSON.stringify(await search.embeddings(url.searchParams.get('q') ?? '', Object.fromEntries(url.searchParams))));
        } else if (method === 'GET' && (match = /^\/embeddings\/related\/([^/]+)$/.exec(path))) {
          reply(res, url, JSON.stringify(search.related(match[1], Object.fromEntries(url.searchParams))));
        } else if (method === 'GET' && (match = /^\/queries\/([^/]+)\/check$/.exec(path))) {
          reply(res, url, JSON.stringify(readonly ? writer.checkQuery(match[1], false) : writer.run(() => writer.checkQuery(match[1]))));
        } else if (!phone && method === 'POST' && path === '/cards/grid') {
          const ids = await requestBody(req);
          if (!Array.isArray(ids)) throw Object.assign(new Error('Expected card id array'), { status: 400 });
          reply(res, url, pack(store.cardsByIds(ids).map(cardValue)), { headers: { 'content-type': 'application/msgpack' } });
        } else if (!phone && method === 'GET' && (path === '/cards' || path === '/cards/trash')) {
          const rows = path === '/cards' ? store.gridCards() : store.trashCards();
          reply(res, url, pack(rows.map(cardValue)), { headers: { 'content-type': 'application/msgpack',
            ...(path === '/cards' ? { ...quota(), 'x-checkpoint': store.gridCheckpoint(), 'access-control-expose-headers': 'x-object-quota, x-checkpoint' } : {}) } });
        } else if (!phone && method === 'GET' && (match = /^\/cards\/([^/]+)\/detail$/.exec(path))) {
          const row = store.card(match[1]);
          if (!row || (!JSON.parse(row.extras).local && hasReplay(req, url))) return false;
          const shareRow = activeShare(row.id, true);
          const share = shareRow ? { id: shareRow.id, url: `/shares/${shareRow.id}`, expires: new Date(Number(shareRow.expires_ns) / 1_000_000).toISOString() } : null;
          const content = store.db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
          reply(res, url, localize(renderDetail(row, share, content ? JSON.parse(content.body) : null)), { headers: { 'content-type': 'text/html; charset=utf-8', 'x-partial': 'CardDetail' } });
        } else if (!phone && method === 'GET' && (match = /^\/cards\/([^/]+)$/.exec(path)) && url.searchParams.get('template') === 'PinnedCard') {
          const row = store.card(match[1]);
          if (!row || !JSON.parse(row.extras).local) return false;
          reply(res, url, localize(renderPinned(row)), { headers: { 'content-type': 'text/html; charset=utf-8', 'x-partial': 'PinnedCard' } });
        } else if (!phone && method === 'GET' && (match = /^\/cards\/([^/]+)$/.exec(path)) && !url.searchParams.has('template')) {
          const row = requireCard(match[1]);
          const json = req.headers.accept?.includes('application/json');
          reply(res, url, json ? localize(row.data) : pack([cardValue(row)]), { headers: json ? jsonHeaders : { 'content-type': 'application/msgpack' } });
        } else if (!phone && method === 'GET' && path === '/spaces') {
          reply(res, url, JSON.stringify(store.spaces()));
        } else if (!phone && method === 'GET' && (match = /^\/spaces\/([^/]+)$/.exec(path))) {
          const space = store.space(match[1]);
          if (!space) throw Object.assign(new Error('Space not found'), { status: 404 });
          const metadata = store.db.prepare('SELECT attributes FROM api_metadata WHERE path = ?').get(path);
          if (space.query && metadata) space.query.checked = JSON.parse(metadata.attributes).checked ?? space.query.checked;
          reply(res, url, JSON.stringify(space));
        } else if (!phone && method === 'GET' && path === '/tags/manual') {
          reply(res, url, pack([store.tagsManual()]), { headers: { 'content-type': 'application/msgpack' } });
        } else if (!phone && method === 'GET' && path === '/tags/recent') {
          reply(res, url, JSON.stringify(store.tagsRecent(Number(url.searchParams.get('take') ?? 10))));
        } else if (!phone && method === 'GET' && path === '/changes') {
          const since = url.searchParams.get('since') ?? '0';
          if (!/^\d+$/.test(since)) throw Object.assign(new Error('Invalid checkpoint'), { status: 400 });
          const changes = store.changesSince(since, { coalesce: true });
          reply(res, url, pack(changes.map(changeValue)), { headers: { 'content-type': 'application/msgpack', ...quota(),
            'x-count': String(changes.length), 'access-control-expose-headers': 'x-object-quota, x-count' } });
        } else if (!phone && method === 'GET' && (match = /^\/objects\/([^/]+)(?:\/(tags|content|revisions))?$/.exec(path))) {
          const row = requireCard(match[1]);
          const data = JSON.parse(row.data);
          let body = objectModel(store, row);
          let headers = jsonHeaders;
          if (match[2] === 'tags') body = store.cardTags(row.id);
          if (match[2] === 'content') {
            const content = store.db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
            if (!content && !data.prose) return false;
            body = content ? JSON.parse(content.body) : data.prose;
            headers = { 'content-type': 'application/prose+json' };
          }
          if (match[2] === 'revisions') {
            const revisions = store.db.prepare('SELECT * FROM revisions WHERE card_id = ? ORDER BY created DESC').all(row.id);
            if (!revisions.length) return false;
            body = revisions.map((revision, i) => ({ id: revision.id, created: revision.created, isCurrent: i === 0,
              content: { body: JSON.parse(revision.body) } }));
          }
          reply(res, url, localize(JSON.stringify(body)), { headers });
        } else if (!phone && method === 'GET' && ['/tags/suggest', '/tags/detail'].includes(path)) {
          const name = url.searchParams.get(path === '/tags/detail' ? 'name' : 'q') ?? '';
          const body = path === '/tags/detail'
            ? { objects: store.db.prepare(`SELECT DISTINCT a.card_id AS id, 1 AS score FROM annotations a
                JOIN cards c ON c.id = a.card_id WHERE a.user_id = ? AND a.name = ? AND c.deleted_ns IS NULL`)
              .all(store.defaultUserId(), name) }
            : store.db.prepare(`SELECT name, COUNT(*) AS count FROM annotations WHERE user_id = ? AND instr(lower(name), lower(?)) > 0
                GROUP BY name ORDER BY count DESC, name LIMIT 20`).all(store.defaultUserId(), name);
          reply(res, url, JSON.stringify(body));
        } else if (!phone && method === 'GET' && ['/quota', '/quotas/objects'].includes(path)) {
          reply(res, url, JSON.stringify(store.quota()), { headers: { ...jsonHeaders, ...quota() } });
        } else if (!phone && method === 'GET' && path === '/account.json') {
          const metadata = store.db.prepare('SELECT attributes FROM api_metadata WHERE path = ?').get(path);
          const account = metadata ? JSON.parse(metadata.attributes).account : {};
          account.quotas = { ...account.quotas, objects: store.quota() };
          reply(res, url, JSON.stringify(account), { headers: { ...jsonHeaders, ...quota() } });
        } else {
          if (method === 'GET') return false;
          if (readonly) return false;
          const write = mutation(method, path);
          if (!write) return false;
          const body = await requestBody(req);
          const prepared = method === 'POST' && path === '/objects' && (body.uploadId || body.type === 'Media') ? await uploads.prepare(body.uploadId) : null;
          const result = writer.run(() => {
            if (prepared?.existingId) return { status: 200, body: { id: prepared.existingId } };
            const result = prepared ? writer.create(body, prepared) : write(body);
            if (prepared) uploads.claim(body.uploadId, result.body.id);
            return result;
          });
          if (prepared) await uploads.cleanupClaimed(body.uploadId);
          send(res, result.status ?? 200, JSON.stringify(result.body ?? {}), { ...jsonHeaders, ...quota(), ...result.headers });
          if (method === 'POST' && path === '/objects' && result.status === 201) {
            const row = store.card(result.body.id);
            jobs.schedule(row.id, Boolean(JSON.parse(row.extras).capturePending));
          } else if (method === 'POST' && (match = /^\/objects\/([^/]+)\/restore$/.exec(path))) {
            const row = store.card(match[1]);
            jobs.schedule(row.id, Boolean(JSON.parse(row.extras).capturePending));
          } else if ((match = /^\/objects\/([^/]+)(?:\/(?:prose|commit|content)|\/revisions\/[^/]+\/restore)?$/.exec(path)) && ['PUT', 'PATCH', 'POST'].includes(method)) {
            const row = store.card(match[1]);
            if (row && ['Note', 'Content', 'Article'].includes(row.type)) jobs.schedule(row.id);
          }
        }
        return true;
      } catch (error) {
        if (!error.status && !(error instanceof TypeError || error instanceof RangeError)) throw error;
        const status = error.status ?? 400;
        send(res, status, JSON.stringify({ type: 'about:blank', status, detail: error.status ? error.message : 'Invalid request data' }),
          { 'content-type': 'application/problem+json' });
        return true;
      }
    },
  };
}
