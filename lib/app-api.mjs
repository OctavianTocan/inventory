import { GRID_HIDDEN_FLAGS, isoFromNs } from './store.mjs';
import { imageCover } from './render/media.mjs';

const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 200;

// Plain text of a prose document, one line per block.
function proseText(node) {
  if (node.type === 'hardBreak') return '\n';
  if (typeof node.text === 'string') return node.text;
  const children = node.content ?? [];
  const inline = children.every((child) => child.type === 'text' || child.type === 'hardBreak');
  return children.map(proseText).join(inline ? '' : '\n');
}

// Cursor for the position after `ns`/`id`, opaque to the client.
const keysetCursor = (ns, id) => Buffer.from(JSON.stringify([String(ns), id])).toString('base64url');
const offsetCursor = (offset) => Buffer.from(JSON.stringify([offset])).toString('base64url');

// Parse a cursor made by this module; throws a 400 for anything else.
function readCursor(text, keyset) {
  try {
    const value = JSON.parse(Buffer.from(text, 'base64url').toString('utf8'));
    if (keyset && value.length === 2 && /^\d+$/.test(value[0]) && typeof value[1] === 'string') return { ns: BigInt(value[0]), id: value[1] };
    if (!keyset && value.length === 1 && Number.isSafeInteger(value[0]) && value[0] >= 0) return value[0];
  } catch {}
  throw Object.assign(new Error('bad-cursor'), { status: 400 });
}

/**
 * Create the native app's read endpoints under `/app/v1`: cards, trash, spaces, tags and search as typed JSON.
 * @param {object} options
 * @param {object} options.store Open store (`lib/store.mjs`).
 * @param {object} options.search Search service (`lib/search.mjs`) whose `search(q)` resolves to ordered matches.
 * @param {(text:string) => string} options.localize Rewrite recorded origins to local paths.
 * @param {(res:object,status:number,body:Buffer|string,headers?:object) => void} options.send HTTP sender.
 * @returns {{handle: (req:import('node:http').IncomingMessage, res:import('node:http').ServerResponse, url:URL) => Promise<boolean>}}
 *   `handle` answers `GET /app/v1/cards`, `/cards/<id>`, `/trash`, `/spaces`, `/spaces/<id>/cards`, `/tags` and `/search`
 *   and resolves true; any other request resolves false. Sign-in is the caller's gate, not checked here.
 */
export function createAppApi({ store, search, localize, send }) {
  const json = (res, status, body) => send(res, status, localize(JSON.stringify(body)), { 'content-type': 'application/json; charset=utf-8' });
  const problem = (res, status, error) => json(res, status, { error });

  // The card in the app's typed JSON shape (docs/app-api.md); absent values are omitted.
  function cardJson(row) {
    const data = JSON.parse(row.data);
    const extras = JSON.parse(row.extras);
    const cover = extras.cover ?? (data.type === 'Image' && data.blob ? imageCover(data.blob) : undefined);
    const content = store.db.prepare('SELECT body FROM card_content WHERE card_id = ?').get(row.id);
    const prose = content ? JSON.parse(content.body) : data.prose;
    const spaces = store.db.prepare('SELECT space_id FROM space_cards WHERE card_id = ? AND excluded = 0 ORDER BY rank').all(row.id).map((r) => r.space_id);
    const tags = store.cardTags(row.id);
    const note = data.note?.prose ? proseText(data.note.prose).trim() : undefined;
    const blob = data.blob && { path: data.blob.path, type: data.blob.type, width: data.blob.width, height: data.blob.height, dominantColor: data.blob.dominantColor };
    const state = extras.captureFailed ? 'failed' : extras.screenshotPending ? 'screenshot' : extras.capturePending ? 'saving' : 'ready';
    const card = {
      id: row.id, type: row.type, flags: row.flags,
      created: data.created ?? isoFromNs(row.createdNs), modified: data.modified ?? isoFromNs(row.modifiedNs), bumped: data.bumped ?? isoFromNs(row.bumpedNs),
      deleted: row.deletedNs === null ? undefined : data.deleted ?? isoFromNs(row.deletedNs),
      title: extras.title ?? data.title, description: data.description, url: extras.linkUrl ?? data.source?.url,
      domain: data.domain, siteName: data.siteName, author: typeof data.author === 'string' ? data.author : undefined,
      note: note || undefined, tags: tags.length ? tags : undefined, spaces: spaces.length ? spaces : undefined,
      pinPosition: extras.pinPosition, cover, blob, prose, post: extras.post, app: extras.app, product: extras.product, reddit: extras.reddit, state,
    };
    return JSON.parse(JSON.stringify(card));
  }

  // limit and cursor from the query string; throws a 400 for values outside the contract.
  function paging(url, keyset) {
    const raw = url.searchParams.get('limit');
    const limit = raw === null ? DEFAULT_LIMIT : Number(raw);
    if (!Number.isInteger(limit) || limit < 1 || limit > MAX_LIMIT) throw Object.assign(new Error('bad-limit'), { status: 400 });
    const cursor = url.searchParams.get('cursor');
    return { limit, after: cursor === null ? null : readCursor(cursor, keyset) };
  }

  // Answer a keyset page: fetch one extra row to know whether another page follows.
  function keysetPage(res, url, fetchPage, nsOf) {
    const { limit, after } = paging(url, true);
    const rows = fetchPage({ after, limit: limit + 1 });
    const page = rows.slice(0, limit);
    json(res, 200, { cards: page.map(cardJson), next: rows.length > limit ? keysetCursor(nsOf(page.at(-1)), page.at(-1).id) : null });
  }

  // Answer a page of an already ordered list of card rows.
  function offsetPage(res, url, rows) {
    const { limit, after: offset } = paging(url, false);
    const page = rows.slice(offset ?? 0, (offset ?? 0) + limit);
    json(res, 200, { cards: page.map(cardJson), next: (offset ?? 0) + limit < rows.length ? offsetCursor((offset ?? 0) + limit) : null });
  }

  const listed = (row) => row.deletedNs === null && (row.flags & GRID_HIDDEN_FLAGS) === 0;

  return {
    async handle(req, res, url) {
      if (!url.pathname.startsWith('/app/v1/')) return false;
      const path = url.pathname.slice('/app/v1'.length);
      let match;
      try {
        if (!/^\/(cards|trash|spaces|tags|search)(\/|$)/.test(path)) return false;
        if (req.method !== 'GET') { send(res, 405, '', { allow: 'GET' }); return true; }
        if (path === '/cards') keysetPage(res, url, ({ after, limit }) => store.gridCardsAfter({ after, limit }), (row) => row.bumpedNs);
        else if (path === '/trash') keysetPage(res, url, ({ after, limit }) => store.trashCardsAfter({ after, limit }), (row) => row.deletedNs);
        else if ((match = /^\/cards\/([^/]+)$/.exec(path))) {
          const row = store.card(decodeURIComponent(match[1]));
          if (row) json(res, 200, cardJson(row)); else problem(res, 404, 'not-found');
        } else if (path === '/spaces') {
          json(res, 200, { spaces: store.spaces().map((space) => ({ id: space.id, name: space.name, color: space.color, position: space.position,
            smart: Boolean(space.query), count: space.objects.length })) });
        } else if ((match = /^\/spaces\/([^/]+)\/cards$/.exec(path))) {
          const id = decodeURIComponent(match[1]);
          if (store.space(id)) offsetPage(res, url, store.spaceCards(id).filter(listed)); else problem(res, 404, 'not-found');
        } else if (path === '/tags') {
          json(res, 200, { tags: store.tagsManual().map(({ name, count }) => ({ name, count })) });
        } else if (path === '/search') {
          const q = url.searchParams.get('q');
          if (!q?.trim()) problem(res, 400, 'missing-query');
          else {
            const { matches } = await search.search(q, {});
            offsetPage(res, url, store.cardsByIds(matches.map((m) => m.id)));
          }
        } else return false;
        return true;
      } catch (error) {
        if (error.status !== 400) throw error;
        problem(res, 400, /^[a-z-]+$/.test(error.message) ? error.message : 'bad-request');
        return true;
      }
    },
  };
}
