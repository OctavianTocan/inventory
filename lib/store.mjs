import { DatabaseSync } from 'node:sqlite';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

/** Number of grid cards the client draws before it paints the rest of the stream. */
export const INITIAL_DRAW_COUNT = 250;

/** Card flag bits that hide a card from the grid: private, draft, stub. */
export const GRID_HIDDEN_FLAGS = 8 | 131072 | 262144;

const MANUAL_TAG_FLAG = 8;
const NS_PER_SECOND = 1_000_000_000n;
const BASE62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
const COUNTER_WIDTH = 9;
const COUNTER_LIMIT = 62n ** BigInt(COUNTER_WIDTH);
const KIND_CODES = { card: '0O', tag: '0y', space: 'gl', query: 'iV', note: '9Z' };
const ID_SOURCES = {
  card: ['cards', 'id'],
  tag: ['annotations', 'id'],
  space: ['spaces', 'id'],
  query: ['spaces', "json_extract(query, '$.id')"],
  note: ['cards', "json_extract(data, '$.note.id')"],
};

const SCHEMA = `
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  account_uid TEXT NOT NULL,
  id_prefix TEXT NOT NULL,
  flags INTEGER NOT NULL DEFAULT 0,
  settings TEXT NOT NULL CHECK (json_valid(settings))
) STRICT;

CREATE TABLE IF NOT EXISTS cards (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users (id),
  type TEXT NOT NULL,
  flags INTEGER NOT NULL DEFAULT 0,
  data TEXT NOT NULL CHECK (json_valid(data)),
  html TEXT NOT NULL,
  extras TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(extras)),
  created_ns INTEGER NOT NULL,
  bumped_ns INTEGER NOT NULL,
  modified_ns INTEGER NOT NULL,
  deleted_ns INTEGER
) STRICT;
CREATE INDEX IF NOT EXISTS cards_grid ON cards (user_id, bumped_ns DESC, id DESC);
CREATE INDEX IF NOT EXISTS cards_trash ON cards (user_id, deleted_ns DESC, id DESC) WHERE deleted_ns IS NOT NULL;

CREATE TABLE IF NOT EXISTS annotations (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users (id),
  card_id TEXT NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  flags INTEGER NOT NULL DEFAULT 0,
  position INTEGER NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS annotations_card ON annotations (card_id, position);
CREATE INDEX IF NOT EXISTS annotations_name ON annotations (user_id, name);

CREATE TABLE IF NOT EXISTS tag_names (
  user_id TEXT NOT NULL REFERENCES users (id),
  name TEXT NOT NULL,
  modified_ms INTEGER NOT NULL,
  manual_rank INTEGER,
  PRIMARY KEY (user_id, name)
) STRICT;

CREATE TABLE IF NOT EXISTS spaces (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users (id),
  name TEXT NOT NULL,
  color TEXT NOT NULL,
  position INTEGER NOT NULL,
  created TEXT NOT NULL,
  query TEXT CHECK (query IS NULL OR json_valid(query))
) STRICT;

CREATE TABLE IF NOT EXISTS space_cards (
  space_id TEXT NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
  card_id TEXT NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users (id),
  rank INTEGER NOT NULL,
  score REAL NOT NULL,
  query_id TEXT,
  excluded INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (space_id, card_id)
) STRICT;
CREATE INDEX IF NOT EXISTS space_cards_rank ON space_cards (space_id, rank);
CREATE INDEX IF NOT EXISTS space_cards_card ON space_cards (card_id);

CREATE TABLE IF NOT EXISTS changes (
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL REFERENCES users (id),
  ts_ns INTEGER NOT NULL,
  object_type INTEGER NOT NULL,
  object_id TEXT NOT NULL,
  type INTEGER NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS changes_ts ON changes (user_id, ts_ns);

CREATE TABLE IF NOT EXISTS api_metadata (
  path TEXT PRIMARY KEY,
  status INTEGER NOT NULL,
  headers TEXT NOT NULL CHECK (json_valid(headers)),
  attributes TEXT NOT NULL CHECK (json_valid(attributes))
) STRICT;

CREATE TABLE IF NOT EXISTS card_content (
  card_id TEXT PRIMARY KEY REFERENCES cards (id) ON DELETE CASCADE,
  body TEXT NOT NULL CHECK (json_valid(body))
) STRICT;

CREATE TABLE IF NOT EXISTS revisions (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
  created TEXT NOT NULL,
  body TEXT NOT NULL CHECK (json_valid(body))
) STRICT;

CREATE TABLE IF NOT EXISTS pins (
  card_id TEXT PRIMARY KEY REFERENCES cards (id) ON DELETE CASCADE,
  position INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS shares (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
  created_ns INTEGER NOT NULL,
  expires_ns INTEGER NOT NULL
) STRICT;
`;

const VISIBLE = `c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & ${GRID_HIDDEN_FLAGS}) = 0`;

/**
 * Convert an ISO timestamp with up to nine fractional digits to nanoseconds since the epoch.
 * @param {string} iso UTC timestamp such as `2026-10-15T09:35:08.050534Z`.
 * @returns {bigint} Exact nanoseconds since the epoch.
 */
export function nsFromIso(iso) {
  const match = /^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)(?:\.(\d{1,9}))?Z$/.exec(iso);
  if (!match) throw new RangeError(`Not a UTC ISO timestamp: ${iso}`);
  const seconds = BigInt(Date.parse(`${match[1]}Z`) / 1000);
  return seconds * NS_PER_SECOND + BigInt((match[2] ?? '').padEnd(9, '0'));
}

/**
 * Format nanoseconds since the epoch as a UTC ISO timestamp.
 * @param {bigint|number|string} ns Nanoseconds since the epoch.
 * @param {number} [digits=0] Fractional digits to include, 0 to 9.
 * @returns {string} ISO timestamp ending in `Z`, truncated (not rounded) to `digits`.
 */
export function isoFromNs(ns, digits = 0) {
  const value = BigInt(ns);
  const base = new Date(Number(value / NS_PER_SECOND) * 1000).toISOString().slice(0, 19);
  if (digits === 0) return `${base}Z`;
  return `${base}.${String(value % NS_PER_SECOND).padStart(9, '0').slice(0, digits)}Z`;
}

function decodeCounter(text) {
  return [...text].reduce((n, char) => n * 62n + BigInt(BASE62.indexOf(char)), 0n);
}

function encodeCounter(value) {
  let rest = value;
  let out = '';
  for (let i = 0; i < COUNTER_WIDTH; i++, rest /= 62n) out = BASE62[Number(rest % 62n)] + out;
  return out;
}

function cardRow(row) {
  return {
    id: row.id,
    type: row.type,
    flags: Number(row.flags),
    data: row.data,
    html: row.html,
    extras: row.extras,
    createdNs: row.created_ns,
    bumpedNs: row.bumped_ns,
    modifiedNs: row.modified_ns,
    deletedNs: row.deleted_ns,
  };
}

/**
 * @typedef {object} CardRow
 * @property {string} id Card id.
 * @property {string} type Card type such as `XPost` or `Article`.
 * @property {number} flags Card flag bitmask.
 * @property {string} data Card JSON exactly as mymind sends it in the `json` field.
 * @property {string} html Grid card markup exactly as mymind sends it in the `html` field.
 * @property {string} extras JSON object of source metadata that `renderCard` needs besides `data`.
 * @property {bigint} createdNs Creation time in nanoseconds since the epoch.
 * @property {bigint} bumpedNs Grid sort key in nanoseconds since the epoch.
 * @property {bigint} modifiedNs Last change in nanoseconds since the epoch.
 * @property {bigint|null} deletedNs Time moved to trash in nanoseconds, or null when not deleted.
 */

/**
 * Open (creating if needed) the SQLite store and return its read API.
 * @param {string} path Database file path, or `:memory:`. Parent directories are created.
 * @returns {object} Store handle: `db` and `transaction` for writers, plus the read methods below.
 */
export function openStore(path) {
  if (path !== ':memory:') mkdirSync(dirname(path), { recursive: true });
  const db = new DatabaseSync(path);
  db.exec('PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;');
  db.exec(SCHEMA);

  const statements = new Map();
  const stmt = (sql) => {
    let prepared = statements.get(sql);
    if (!prepared) {
      prepared = db.prepare(sql);
      prepared.setReadBigInts(true);
      statements.set(sql, prepared);
    }
    return prepared;
  };
  let cachedUserId = null;
  const resolveUser = (userId) => {
    if (userId) return userId;
    cachedUserId ??= stmt('SELECT id FROM users ORDER BY rowid LIMIT 1').get()?.id;
    if (!cachedUserId) throw new Error('The store has no user; seed it first');
    return cachedUserId;
  };
  const counters = new Map();

  const store = {
    db,

    /**
     * Run `fn` inside one write transaction, rolling back if it throws.
     * @template T
     * @param {() => T} fn Synchronous work that mutates `db`.
     * @returns {T} Whatever `fn` returns.
     * @throws {Error} Whatever `fn` throws, after the rollback.
     */
    transaction(fn) {
      db.exec('BEGIN IMMEDIATE');
      try {
        const result = fn();
        db.exec('COMMIT');
        return result;
      } catch (error) {
        db.exec('ROLLBACK');
        throw error;
      }
    },

    /**
     * Close the database.
     * @returns {void}
     */
    close() {
      db.close();
    },

    /**
     * Id of the store's first (default) user.
     * @returns {string} User id.
     * @throws {Error} If the store has no user.
     */
    defaultUserId() {
      return resolveUser();
    },

    /**
     * Account record and settings for a user.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {{id: string, accountUid: string, idPrefix: string, flags: number, settings: object}|null} User, or null if unknown.
     */
    user(userId) {
      const row = stmt('SELECT * FROM users WHERE id = ?').get(resolveUser(userId));
      if (!row) return null;
      return {
        id: row.id,
        accountUid: row.account_uid,
        idPrefix: row.id_prefix,
        flags: Number(row.flags),
        settings: JSON.parse(row.settings),
      };
    },

    /**
     * Visible grid cards, newest `bumped` first: the order `GET /cards` streams them.
     * @param {object} [options] Paging window; the client draws the first {@link INITIAL_DRAW_COUNT} cards before the rest.
     * @param {string} [options.userId] User id; defaults to the first user.
     * @param {number} [options.offset=0] Cards to skip.
     * @param {number} [options.limit] Maximum cards to return; omit for all.
     * @returns {CardRow[]} Cards that are not deleted, private, draft or stub.
     */
    gridCards({ userId, offset = 0, limit = -1 } = {}) {
      return stmt(`SELECT * FROM cards c WHERE ${VISIBLE} ORDER BY c.bumped_ns DESC, c.id DESC LIMIT ? OFFSET ?`)
        .all(resolveUser(userId), limit, offset).map(cardRow);
    },

    /**
     * Number of visible grid cards, the `count` of `x-object-quota`.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {number} Visible card count.
     */
    gridCount(userId) {
      return Number(stmt(`SELECT COUNT(*) AS n FROM cards c WHERE ${VISIBLE}`).get(resolveUser(userId)).n);
    },

    /**
     * Object quota for the `x-object-quota` header.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {{count: number, limit: number}} Visible card count and the user's object limit.
     */
    quota(userId) {
      const user = store.user(userId);
      return { count: store.gridCount(user.id), limit: user.settings.objectLimit };
    },

    /**
     * One card by id, including deleted ones.
     * @param {string} id Card id.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {CardRow|null} The card, or null if it does not exist.
     */
    card(id, userId) {
      const row = stmt('SELECT * FROM cards WHERE id = ? AND user_id = ?').get(id, resolveUser(userId));
      return row ? cardRow(row) : null;
    },

    /**
     * Cards for `POST /cards/grid`, in the order requested.
     * @param {string[]} ids Card ids.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {CardRow[]} The cards that exist (deleted ones included); unknown ids are skipped.
     */
    cardsByIds(ids, userId) {
      const user = resolveUser(userId);
      return ids.flatMap((id) => {
        const row = stmt('SELECT * FROM cards WHERE id = ? AND user_id = ?').get(id, user);
        return row ? [cardRow(row)] : [];
      });
    },

    /**
     * Trashed cards, most recently deleted first: the order `GET /cards/trash` streams them.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {CardRow[]} Cards with a deletion time.
     */
    trashCards(userId) {
      return stmt('SELECT * FROM cards WHERE user_id = ? AND deleted_ns IS NOT NULL ORDER BY deleted_ns DESC, id DESC')
        .all(resolveUser(userId)).map(cardRow);
    },

    /**
     * A card's tags in the order mymind shows them.
     * @param {string} cardId Card id.
     * @returns {{id: string, name: string, flags?: number}[]} Tags as they appear in card JSON; `flags` is omitted when 0.
     */
    cardTags(cardId) {
      return stmt('SELECT id, name, flags FROM annotations WHERE card_id = ? ORDER BY position')
        .all(cardId).map((row) => (row.flags ? { id: row.id, name: row.name, flags: Number(row.flags) } : { id: row.id, name: row.name }));
    },

    /**
     * Spaces as `GET /spaces` returns them.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {object[]} Spaces sorted by `position` ascending, each `{id, name, color, position, objects, created, query?}`.
     */
    spaces(userId) {
      return stmt('SELECT id FROM spaces WHERE user_id = ? ORDER BY position, id').all(resolveUser(userId))
        .map((row) => store.space(row.id));
    },

    /**
     * One space as `GET /spaces/<id>` returns it.
     * @param {string} id Space id.
     * @returns {object|null} `{id, name, color, position, objects, created, query?}`, or null if unknown.
     */
    space(id) {
      const row = stmt('SELECT * FROM spaces WHERE id = ?').get(id);
      if (!row) return null;
      const objects = stmt('SELECT card_id, score, query_id FROM space_cards WHERE space_id = ? AND excluded = 0 ORDER BY rank')
        .all(id).map((member) => (member.query_id
          ? { id: member.card_id, score: member.score, queryId: member.query_id }
          : { id: member.card_id, score: member.score }));
      const space = { id: row.id, name: row.name, color: row.color, position: Number(row.position), objects, created: row.created };
      if (row.query) space.query = JSON.parse(row.query);
      return space;
    },

    /**
     * Cards in a space, for hydrating its grid.
     * @param {string} spaceId Space id.
     * @returns {CardRow[]} Member cards in space order, trashed ones included.
     */
    spaceCards(spaceId) {
      return stmt('SELECT c.* FROM space_cards s JOIN cards c ON c.id = s.card_id WHERE s.space_id = ? AND s.excluded = 0 ORDER BY s.rank')
        .all(spaceId).map(cardRow);
    },

    /**
     * Manual tags as `GET /tags/manual` returns them.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {{name: string, count: number, flags: number, modified: Date}[]} One entry per manual tag name, most-used first.
     */
    tagsManual(userId) {
      return stmt(`
        SELECT a.name AS name, COUNT(*) AS count,
               COALESCE(MAX(t.modified_ms), MAX(c.modified_ns) / 1000000) AS modified
        FROM annotations a
        JOIN cards c ON c.id = a.card_id AND c.deleted_ns IS NULL
        LEFT JOIN tag_names t ON t.user_id = a.user_id AND t.name = a.name
        WHERE a.user_id = ? AND (a.flags & ${MANUAL_TAG_FLAG}) != 0
        GROUP BY a.name
        ORDER BY count DESC, COALESCE(MIN(t.manual_rank), 2147483647), a.name`)
        .all(resolveUser(userId))
        .map((row) => ({ name: row.name, count: Number(row.count), flags: 0, modified: new Date(Number(row.modified)) }));
    },

    /**
     * Recently used manual tags as `GET /tags/recent` returns them.
     * @param {number} [take=10] Maximum names to return.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {{name: string}[]} Tag names, most recently modified first.
     */
    tagsRecent(take = 10, userId) {
      return store.tagsManual(userId)
        .sort((a, b) => b.modified - a.modified)
        .slice(0, take)
        .map(({ name }) => ({ name }));
    },

    /**
     * Newest change timestamp, the `x-checkpoint` header value.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {string} Nanoseconds since the epoch as a decimal string, `"0"` when there are no changes.
     */
    checkpoint(userId) {
      const row = stmt('SELECT MAX(ts_ns) AS ts FROM changes WHERE user_id = ?').get(resolveUser(userId));
      return String(row.ts ?? 0n);
    },

    /**
     * Changes newer than a checkpoint, for `GET /changes?since=`.
     * @param {bigint|number|string} since Checkpoint in nanoseconds; only strictly newer changes are returned.
     * @param {object} [options] Query options.
     * @param {string} [options.userId] User id; defaults to the first user.
     * @param {number} [options.limit=1000] Maximum changes to return.
     * @param {boolean} [options.coalesce=false] Return only the newest change for each object, for the UI synchronizer.
     * @returns {{seq: number, ts: bigint, type: number, objectType: number, objectId: string, card: CardRow|null}[]} Oldest first;
     *   `type` is 1 created or updated, 2 deleted; `objectType` 9 is a card; `card` is the card's current row when it exists.
     */
    changesSince(since, { userId, limit = 1000, coalesce = false } = {}) {
      const user = resolveUser(userId);
      const latest = coalesce ? ` AND seq = (SELECT MAX(newer.seq) FROM changes newer WHERE newer.user_id = changes.user_id
        AND newer.object_type = changes.object_type AND newer.object_id = changes.object_id)` : '';
      return stmt(`SELECT * FROM changes WHERE user_id = ? AND ts_ns > ?${latest} ORDER BY ts_ns, seq LIMIT ?`)
        .all(user, BigInt(since), limit)
        .map((row) => ({
          seq: Number(row.seq),
          ts: row.ts_ns,
          type: Number(row.type),
          objectType: Number(row.object_type),
          objectId: row.object_id,
          card: Number(row.object_type) === 9 ? store.card(row.object_id, user) : null,
        }));
    },

    /**
     * Generate the next unused id of a kind, in mymind's format: account prefix, 2-char kind code, 9 base62 chars.
     * @param {'card'|'tag'|'space'|'query'|'note'} kind What the id is for.
     * @param {string} [userId] User id; defaults to the first user.
     * @returns {string} A 22-character id that sorts after every existing id of that kind.
     * @throws {RangeError} If `kind` is unknown or the counter is exhausted.
     */
    newId(kind, userId) {
      const code = KIND_CODES[kind];
      if (!code) throw new RangeError(`Unknown id kind: ${kind}`);
      const user = store.user(userId);
      const key = `${user.id}:${kind}`;
      if (!counters.has(key)) {
        const [table, column] = ID_SOURCES[kind];
        const row = stmt(`SELECT MAX(${column}) AS id FROM ${table} WHERE user_id = ? AND substr(${column}, ?, 2) = ?`)
          .get(user.id, user.idPrefix.length + 1, code);
        counters.set(key, row.id ? decodeCounter(row.id.slice(user.idPrefix.length + 2)) : 0n);
      }
      const next = counters.get(key) + 1n;
      if (next >= COUNTER_LIMIT) throw new RangeError(`Id counter exhausted for ${kind}`);
      counters.set(key, next);
      return `${user.idPrefix}${code}${encodeCounter(next)}`;
    },
  };
  return store;
}
