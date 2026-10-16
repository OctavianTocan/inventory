import { existsSync, readFileSync, rmSync } from 'node:fs';
import { DatabaseSync } from 'node:sqlite';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { nsFromIso, openStore } from '../lib/store.mjs';
import { recordingExtras } from './check-render.mjs';
import { importReadMetadata } from '../lib/read-metadata.mjs';
import { documentText } from '../lib/ai.mjs';
import { embeddingItem } from './embed-library.mjs';

export const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const API = 'https://access.mymind.com';
const NS_PER_SECOND = 1_000_000_000n;
const TIE_STEP_NS = 1_000_000n;

/**
 * Read the raw recordings the seed and its checker are built from.
 * @param {string} [root] Project root containing `capture/`, `replay/` and `mirror/`.
 * @returns {Promise<{capture: {html: string, json: string}[], stream: {html: string, json: string}[],
 *   trash: {html: string, json: string}[], spaces: object[], spaceDetails: Map<string, object>, manual: object[],
 *   recent: {name: string}[], gridHeaders: Record<string, string>, config: object}>} Decoded recordings: the capture's card
 *   list, the newest desktop `GET /cards` and `/cards/trash` streams, the `/spaces` list and per-space bodies, both tag lists,
 *   the headers of the grid response, and the shell's `config` object.
 * @throws {Error} If a recording is missing.
 */
export async function readRecordings(root = ROOT) {
  const msgpack = await import(pathToFileURL(join(root, 'mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js')).href);
  const index = JSON.parse(readFileSync(join(root, 'replay/index.json'), 'utf8'));
  const entry = (key) => {
    if (!index[key]) throw new Error(`Missing recording: ${key}`);
    return index[key];
  };
  const body = (key) => readFileSync(join(root, 'replay/bodies', entry(key).file));
  const stream = (key) => [...msgpack.decodeMulti(body(key))];
  const json = (key) => JSON.parse(body(key).toString('utf8'));

  const spaceDetails = new Map();
  for (const key of Object.keys(index)) {
    const match = key.match(new RegExp(`^GET ${API}/spaces/([^/?]+)$`));
    if (match) spaceDetails.set(match[1], json(key));
  }
  const shell = readFileSync(join(root, 'capture/shell.html'), 'utf8');
  const config = JSON.parse(shell.match(/<script id="config" type="application\/json">([\s\S]*?)<\/script>/)[1]);

  return {
    capture: JSON.parse(readFileSync(join(root, 'capture/cards.json'), 'utf8')),
    stream: stream(`GET ${API}/cards`),
    trash: stream(`GET ${API}/cards/trash`),
    spaces: json(`GET ${API}/spaces`),
    spaceDetails,
    manual: stream(`GET ${API}/tags/manual`)[0],
    recent: json(`GET ${API}/tags/recent?take=10`),
    gridHeaders: entry(`GET ${API}/cards`).headers,
    config,
  };
}

/**
 * Read a recorded card's id from its markup.
 * @param {{html: string}} recording A `{html, json}` card recording.
 * @returns {string} The `data-id` of the card's root element.
 * @throws {Error} If the markup has no card id.
 */
export function recordedCardId(recording) {
  const id = recording.html.match(/^<mymind-card\b[^>]*\sdata-id="([^"]+)"/)?.[1];
  if (!id) throw new Error('Recorded card has no data-id');
  return id;
}

// Overlay the newer replay card bodies on the capture's card list, keyed by card id.
function mergeCards(recordings) {
  const byId = new Map();
  for (const recording of recordings.capture) byId.set(recordedCardId(recording), recording);
  const stats = { capture: byId.size, added: 0, replaced: 0, signedRefresh: 0 };
  for (const recording of recordings.stream) {
    const id = recordedCardId(recording);
    const prior = byId.get(id);
    if (!prior) stats.added++;
    else if (nsFromIso(JSON.parse(recording.json).modified) < nsFromIso(JSON.parse(prior.json).modified)) continue;
    else if (recording.json !== prior.json) stats.replaced++;
    else if (recording.html !== prior.html) stats.signedRefresh++;
    byId.set(id, recording);
  }
  return { byId, stats };
}

// Recorded `bumped` has second precision but a fixed order, so encode that order as sub-second offsets within each second.
function bumpedNsByOrder(orderedIds, byId) {
  const seconds = orderedIds.map((id) => nsFromIso(JSON.parse(byId.get(id).json).bumped));
  const sizes = new Map();
  for (const s of seconds) sizes.set(s, (sizes.get(s) ?? 0) + 1);
  const seen = new Map();
  const result = new Map();
  let inversions = 0;
  orderedIds.forEach((id, i) => {
    const k = seen.get(seconds[i]) ?? 0;
    seen.set(seconds[i], k + 1);
    result.set(id, seconds[i] + BigInt(sizes.get(seconds[i]) - 1 - k) * TIE_STEP_NS);
    if (i > 0 && seconds[i] > seconds[i - 1]) inversions++;
  });
  return { result, inversions };
}

// Build the single user row from the recorded shell config.
function userRecord(config, idPrefix) {
  const settings = {
    colorScheme: config.colorScheme,
    timeZoneId: config.timeZoneId,
    locale: config.locale,
    currency: config.currency,
    aiLevel: config.aiLevel,
    onboardFlags: config.onboardFlags,
    objectLimit: config.quotas.objects.limit,
  };
  return { id: config.user.uid, accountUid: config.account.uid, idPrefix, flags: config.user.flags, settings: JSON.stringify(settings) };
}

// Read an existing database's embeddings with the text each was computed from, so a reseed need not pay for them again.
function previousEmbeddings(dbPath) {
  if (!existsSync(dbPath)) return new Map();
  const db = new DatabaseSync(dbPath, { readOnly: true });
  try {
    if (!db.prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'embeddings'").get()) return new Map();
    return new Map(db.prepare(`SELECT e.card_id, e.vector, c.type, c.data, c.extras, b.body FROM embeddings e
      JOIN cards c ON c.id = e.card_id LEFT JOIN card_content b ON b.card_id = e.card_id`).all()
      .map((row) => [row.card_id, { vector: row.vector, text: documentText(embeddingItem(row, row.body)) }]));
  } finally { db.close(); }
}

// Keep each previous vector whose card still has exactly the text it was computed from.
function restoreEmbeddings(store, previous) {
  const { db } = store;
  const insert = db.prepare('INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, ?, 768, ?)');
  const content = db.prepare('SELECT body FROM card_content WHERE card_id = ?');
  let kept = 0;
  store.transaction(() => {
    for (const card of db.prepare('SELECT id, user_id, type, data, extras FROM cards').all()) {
      const old = previous.get(card.id);
      if (!old || documentText(embeddingItem(card, content.get(card.id)?.body)) !== old.text) continue;
      insert.run(card.id, card.user_id, old.vector);
      kept++;
    }
  });
  return kept;
}

/**
 * Rebuild the SQLite database from the recordings, replacing any existing file.
 * @param {string} dbPath Database file path; the file and its WAL sidecars are deleted first, keeping only the
 *   embeddings of cards whose text is unchanged.
 * @param {string} [root] Project root containing the recordings.
 * @returns {Promise<Record<string, number>>} Counts of what was written, plus `tieInversions`, `extrasFailures`
 *   and `leftoverCaptureCards`, which are 0 for a clean seed.
 */
export async function seedDatabase(dbPath, root = ROOT) {
  const embeddings = previousEmbeddings(dbPath);
  for (const suffix of ['', '-wal', '-shm']) rmSync(`${dbPath}${suffix}`, { force: true });
  const recordings = await readRecordings(root);
  const { byId, stats } = mergeCards(recordings);

  const trashIds = new Set(recordings.trash.map(recordedCardId));
  const trashById = new Map(recordings.trash.map((recording) => [recordedCardId(recording), recording]));
  const gridIds = recordings.stream.map(recordedCardId).filter((id) => !trashIds.has(id));
  const inGrid = new Set(gridIds);
  const leftover = [...byId.keys()].filter((id) => !inGrid.has(id) && !trashIds.has(id));
  const { result: bumpedNs, inversions } = bumpedNsByOrder([...gridIds, ...leftover], byId);

  const allCards = [
    ...[...gridIds, ...leftover].map((id) => ({ id, recording: byId.get(id), deleted: null })),
    ...[...trashIds].map((id) => {
      const recording = trashById.get(id);
      return { id, recording, deleted: nsFromIso(JSON.parse(recording.json).deleted) };
    }),
  ];
  const idPrefix = allCards[0].id.slice(0, 11);
  const stray = allCards.filter(({ id }) => !id.startsWith(idPrefix));
  if (stray.length) throw new Error(`${stray.length} card ids do not share the prefix ${idPrefix}`);

  const store = openStore(dbPath);
  const counts = { cards: 0, trash: 0, annotations: 0, spaces: 0, spaceCards: 0, tagNames: 0, changes: 0 };
  let extrasFailures = 0;
  store.transaction(() => {
    const { db } = store;
    const user = userRecord(recordings.config, idPrefix);
    db.prepare('INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES (?, ?, ?, ?, ?)')
      .run(user.id, user.accountUid, user.idPrefix, user.flags, user.settings);

    const insertCard = db.prepare(`INSERT INTO cards
      (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns, deleted_ns)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`);
    const insertTag = db.prepare('INSERT INTO annotations (id, user_id, card_id, name, flags, position) VALUES (?, ?, ?, ?, ?, ?)');
    const insertChange = db.prepare('INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, 9, ?, ?)');
    const changes = [];
    for (const { id, recording, deleted } of allCards) {
      const data = JSON.parse(recording.json);
      let extras;
      try {
        extras = recordingExtras(recording);
      } catch {
        extras = { id };
        extrasFailures++;
      }
      const modified = nsFromIso(data.modified);
      insertCard.run(id, user.id, data.type, data.flags ?? 0, recording.json, recording.html, JSON.stringify(extras),
        nsFromIso(data.created), deleted === null ? bumpedNs.get(id) : nsFromIso(data.bumped), modified, deleted);
      (data.tags ?? []).forEach((tag, position) => {
        insertTag.run(tag.id, user.id, id, tag.name, tag.flags ?? 0, position);
        counts.annotations++;
      });
      changes.push({ id, ts: modified, type: deleted === null ? 1 : 2 });
      counts[deleted === null ? 'cards' : 'trash']++;
    }
    for (const change of changes.sort((a, b) => (a.ts < b.ts ? -1 : a.ts > b.ts ? 1 : 0))) {
      insertChange.run(user.id, change.ts, change.id, change.type);
      counts.changes++;
    }

    const insertTagName = db.prepare('INSERT INTO tag_names (user_id, name, modified_ms, manual_rank) VALUES (?, ?, ?, ?)');
    recordings.manual.forEach((tag, rank) => {
      insertTagName.run(user.id, tag.name, tag.modified.getTime(), rank);
      counts.tagNames++;
    });

    const insertSpace = db.prepare('INSERT INTO spaces (id, user_id, name, color, position, created, query) VALUES (?, ?, ?, ?, ?, ?, ?)');
    const insertMember = db.prepare(`INSERT INTO space_cards (space_id, card_id, user_id, rank, score, query_id, excluded)
      VALUES (?, ?, ?, ?, ?, ?, 0)`);
    for (const space of recordings.spaces) {
      insertSpace.run(space.id, user.id, space.name, space.color, space.position, space.created,
        space.query ? JSON.stringify(space.query) : null);
      space.objects.forEach((member, rank) => {
        insertMember.run(space.id, member.id, user.id, rank, member.score, member.queryId ?? null);
        counts.spaceCards++;
      });
      counts.spaces++;
    }
  });
  await importReadMetadata(store, root);
  counts.embeddingsKept = restoreEmbeddings(store, embeddings);
  store.close();
  return { ...counts, ...Object.fromEntries(Object.entries(stats).map(([k, v]) => [`merge_${k}`, v])),
    tieInversions: inversions, extrasFailures, leftoverCaptureCards: leftover.length };
}

// Resolve the database path from the command-line flags.
function parseArgs(args) {
  let db = join(ROOT, 'data/mymind.db');
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--db' && args[i + 1]) db = resolve(args[++i]);
    else if (args[i].startsWith('--db=')) db = resolve(args[i].slice(5));
    else throw new Error('Usage: node tools/seed-db.mjs [--db data/mymind.db]');
  }
  return db;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try {
    const dbPath = parseArgs(process.argv.slice(2));
    const summary = await seedDatabase(dbPath);
    console.log(`Seeded ${dbPath}`);
    for (const [key, value] of Object.entries(summary)) console.log(`  ${key.padEnd(22)} ${value}`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
