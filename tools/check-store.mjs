import { existsSync } from 'node:fs';
import { deepStrictEqual } from 'node:assert/strict';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { INITIAL_DRAW_COUNT, openStore } from '../lib/store.mjs';
import { renderCard } from '../lib/render-card.mjs';
import { ROOT, readRecordings, recordedCardId } from './seed-db.mjs';

const plain = (value) => JSON.parse(JSON.stringify(value));
const ids = (rows) => rows.map((row) => row.id);

// Blank CDN signatures, which live mymind re-issues on every recording and the store cannot mint.
const unsign = (html) => html?.replace(/Expires=\d+(&(?:amp;)?)Signature=[^&"\s]+/g, 'Expires=$1Signature=');

// Name the first position where two id lists differ.
function firstOrderDifference(actual, expected) {
  const i = Array.from({ length: Math.max(actual.length, expected.length) }, (_, n) => n).find((n) => actual[n] !== expected[n]);
  return i === undefined ? null : `position ${i}: store ${actual[i]}, recording ${expected[i]}`;
}

// Report the first unequal line of two markup strings.
function firstLineDifference(expected, actual) {
  const a = expected.split('\n');
  const b = actual.split('\n');
  const i = Array.from({ length: Math.max(a.length, b.length) }, (_, n) => n).find((n) => a[n] !== b[n]);
  return `line ${i + 1}: expected ${JSON.stringify(a[i]).slice(0, 120)}, got ${JSON.stringify(b[i]).slice(0, 120)}`;
}

function assertEqual(actual, expected, label) {
  try {
    deepStrictEqual(actual, expected);
  } catch {
    throw new Error(`${label} differs`);
  }
}

// Define every assertion against the seeded store, each returning a short detail string or throwing.
function buildChecks(store, rec) {
  const stream = rec.stream;
  const streamIds = stream.map(recordedCardId);
  const trashIds = rec.trash.map(recordedCardId);
  const grid = store.gridCards();
  const trash = store.trashCards();
  const all = [...grid, ...trash];
  const renderEquality = (rows) => {
    const failures = [];
    for (const row of rows) {
      try {
        const html = renderCard(JSON.parse(row.data), JSON.parse(row.extras));
        if (html !== row.html) failures.push({ row, detail: firstLineDifference(row.html, html) });
      } catch (error) {
        failures.push({ row, detail: `${error.name}: ${error.message}` });
      }
    }
    if (failures.length) {
      const sample = failures.slice(0, 3).map(({ row, detail }) => `${row.type} ${row.id.slice(-8)} ${detail}`);
      throw new Error(`${failures.length}/${rows.length} differ; ${sample.join(' | ')}`);
    }
    return `${rows.length} of ${rows.length} cards equal`;
  };

  return [
    ['grid card count', () => {
      assertEqual(store.gridCount(), stream.length, 'grid count');
      return `${grid.length} cards`;
    }],
    ['grid order (full)', () => {
      const diff = firstOrderDifference(ids(grid), streamIds);
      if (diff) throw new Error(diff);
      return `${grid.length} ids in recorded order`;
    }],
    ['first page and paging', () => {
      const first = store.gridCards({ limit: INITIAL_DRAW_COUNT });
      const diff = firstOrderDifference(ids(first), streamIds.slice(0, INITIAL_DRAW_COUNT));
      if (diff) throw new Error(diff);
      const next = store.gridCards({ offset: INITIAL_DRAW_COUNT, limit: 50 });
      assertEqual(ids(next), streamIds.slice(INITIAL_DRAW_COUNT, INITIAL_DRAW_COUNT + 50), 'second page');
      return `first ${first.length}, next ${next.length}`;
    }],
    ['grid html/json bytes', () => {
      const byId = new Map(grid.map((row) => [row.id, row]));
      const bad = stream.filter((r) => unsign(byId.get(recordedCardId(r))?.html) !== unsign(r.html) || byId.get(recordedCardId(r))?.data !== r.json);
      if (bad.length) throw new Error(`${bad.length} cards differ, first ${recordedCardId(bad[0])}`);
      return `${stream.length} cards byte-equal`;
    }],
    ['capture cards present', () => {
      const missing = rec.capture.map(recordedCardId).filter((id) => !store.card(id));
      if (missing.length) throw new Error(`${missing.length} missing, first ${missing[0]}`);
      return `${rec.capture.length} of ${rec.capture.length}`;
    }],
    ['trash contents and order', () => {
      const diff = firstOrderDifference(ids(trash), trashIds);
      if (diff) throw new Error(diff);
      const byId = new Map(trash.map((row) => [row.id, row]));
      for (const r of rec.trash) {
        const row = byId.get(recordedCardId(r));
        if (row.html !== r.html || row.data !== r.json) throw new Error(`${row.id} body differs`);
        if (row.deletedNs !== BigInt(Date.parse(JSON.parse(r.json).deleted)) * 1_000_000n) throw new Error(`${row.id} deleted time differs`);
      }
      if (trashIds.some((id) => streamIds.includes(id))) throw new Error('trash card also in grid');
      return `${trash.length} cards: ${trashIds.map((id) => id.slice(-8)).join(', ')}`;
    }],
    ['spaces list', () => {
      assertEqual(plain(store.spaces()), plain(rec.spaces), 'spaces');
      return `${rec.spaces.length} spaces, ${rec.spaces.reduce((n, s) => n + s.objects.length, 0)} members`;
    }],
    ['space detail bodies', () => {
      for (const [id, recorded] of rec.spaceDetails) {
        const actual = plain(store.space(id));
        if (recorded.query) delete recorded.query.checked;
        if (actual.query) delete actual.query.checked;
        assertEqual(actual, recorded, `space ${id}`);
      }
      return `${rec.spaceDetails.size} recorded (ignoring older checked)`;
    }],
    ['space card counts', () => {
      for (const space of rec.spaces) {
        assertEqual(ids(store.spaceCards(space.id)), space.objects.map((o) => o.id), space.name);
      }
      return rec.spaces.map((s) => `${s.name.slice(0, 12)}=${s.objects.length}`).join(' ').slice(0, 90);
    }],
    ['card tags (all cards)', () => {
      for (const row of all) assertEqual(store.cardTags(row.id), JSON.parse(row.data).tags ?? [], `tags of ${row.id}`);
      for (const r of [...stream, ...rec.trash]) assertEqual(store.cardTags(recordedCardId(r)), JSON.parse(r.json).tags ?? [], 'recorded tags');
      return `${all.length} cards, ${all.reduce((n, row) => n + store.cardTags(row.id).length, 0)} tags`;
    }],
    ['tags/manual', () => {
      assertEqual(store.tagsManual(), rec.manual, 'manual tags');
      return `${rec.manual.length} names, counts and modified equal`;
    }],
    ['tags/recent', () => {
      assertEqual(store.tagsRecent(10), rec.recent, 'recent tags');
      return rec.recent.map((r) => r.name).join(',').slice(0, 70);
    }],
    ['grid checkpoint and newer changes', () => {
      const expected = rec.gridHeaders['x-checkpoint'];
      if (store.gridCheckpoint() !== expected) throw new Error(`checkpoint ${store.gridCheckpoint()} != ${expected}`);
      const newer = store.changesSince(expected);
      if (newer.some((change) => change.type !== 2)) throw new Error('non-deletion changes newer than the grid checkpoint');
      return `${expected}, ${newer.length} newer deletions`;
    }],
    ['changes log', () => {
      const changes = store.changesSince(0, { limit: all.length + 1 });
      if (changes.length !== all.length) throw new Error(`${changes.length} changes for ${all.length} cards`);
      for (let i = 1; i < changes.length; i++) if (changes[i].ts <= changes[i - 1].ts) throw new Error('timestamps not strictly increasing');
      for (const change of changes) {
        if (change.ts !== change.card.modifiedNs) throw new Error(`${change.objectId} ts differs from modified`);
        if (change.type !== (change.card.deletedNs === null ? 1 : 2)) throw new Error(`${change.objectId} wrong type`);
      }
      return `${changes.length} changes, newest ${changes.at(-1).objectId.slice(-8)}`;
    }],
    ['object quota', () => {
      assertEqual(store.quota(), JSON.parse(rec.gridHeaders['x-object-quota']), 'quota');
      return rec.gridHeaders['x-object-quota'];
    }],
    ['user settings', () => {
      const user = store.user();
      const c = rec.config;
      assertEqual([user.id, user.accountUid, user.flags], [c.user.uid, c.account.uid, c.user.flags], 'user');
      assertEqual(user.settings, {
        colorScheme: c.colorScheme, timeZoneId: c.timeZoneId, locale: c.locale, currency: c.currency,
        aiLevel: c.aiLevel, onboardFlags: c.onboardFlags, objectLimit: c.quotas.objects.limit,
      }, 'settings');
      return `${user.settings.locale}, ${user.settings.timeZoneId}, ${user.settings.colorScheme}`;
    }],
    ['newId format and ordering', () => {
      const user = store.user();
      const codes = { card: '0O', tag: '0y', space: 'gl', query: 'iV', note: '9Z' };
      for (const [kind, code] of Object.entries(codes)) {
        const first = store.newId(kind);
        const second = store.newId(kind);
        if (!new RegExp(`^${user.idPrefix}${code}[0-9A-Za-z]{9}$`).test(first)) throw new Error(`${kind} id malformed: ${first}`);
        if (!(second > first)) throw new Error(`${kind} ids not increasing`);
      }
      const existing = store.db.prepare('SELECT MAX(id) AS id FROM cards WHERE substr(id, 12, 2) = ?').get('0O').id;
      if (!(store.newId('card') > existing)) throw new Error('card id does not sort after existing ids');
      return `${user.idPrefix} + kind + 9 chars`;
    }],
    ['renderCard reproduces grid html', () => renderEquality(grid)],
    ['renderCard reproduces trash html', () => renderEquality(trash)],
  ];
}

// Resolve the database path from the command-line flags.
function parseArgs(args) {
  let db = join(ROOT, 'data/verify.db');
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--db' && args[i + 1]) db = resolve(args[++i]);
    else if (args[i].startsWith('--db=')) db = resolve(args[i].slice(5));
    else throw new Error('Usage: node tools/check-store.mjs [--db data/verify.db]');
  }
  return db;
}

async function main() {
  const dbPath = parseArgs(process.argv.slice(2));
  if (!existsSync(dbPath)) throw new Error(`No database at ${dbPath}; run node tools/seed-db.mjs first`);
  const store = openStore(dbPath);
  const recordings = await readRecordings();
  const results = buildChecks(store, recordings).map(([name, run]) => {
    try {
      return { name, ok: true, detail: run() };
    } catch (error) {
      return { name, ok: false, detail: error.message };
    }
  });
  store.close();
  for (const { name, ok, detail } of results) console.log(`${ok ? 'PASS' : 'FAIL'}  ${name.padEnd(36)} ${detail}`);
  const failed = results.filter((r) => !r.ok).length;
  console.log(`\n${results.length - failed}/${results.length} checks passed`);
  process.exitCode = failed ? 1 : 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try {
    await main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
