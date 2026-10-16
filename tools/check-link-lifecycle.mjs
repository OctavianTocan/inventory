import { readFileSync } from 'node:fs';
import { ok, strictEqual, deepStrictEqual } from 'node:assert/strict';
import { join, dirname } from 'node:path';
import { EventEmitter } from 'node:events';
import { renderCard } from '../lib/render-card.mjs';
import { escapeHtml } from '../lib/render/encoding.mjs';
import { openStore } from '../lib/store.mjs';
import { createWriter } from '../lib/store-write.mjs';
import { createMedia } from '../lib/local-media.mjs';
import { createEnrichment } from '../lib/enrichment.mjs';
import { createHub } from '../lib/hub.mjs';

/**
 * Read the private save recording locally without logging or copying its card data.
 * @returns {{changes:object[],events:object[]}} Recorded cards and SignalR invocations in order.
 */
export function linkSaveRecording() {
  const lines = readFileSync(new URL('../evidence/live-saves/2026-10-15-apk-link.jsonl', import.meta.url), 'utf8')
    .trim().split('\n').map(line => JSON.parse(line));
  const recording = {
    changes: lines.flatMap(line => line.changes ?? []),
    events: lines.flatMap(line => (line.ws ?? '').split('\x1e').filter(Boolean).map(frame => JSON.parse(frame))).filter(frame => frame.type === 1),
  };
  if (recording.changes.length !== 3 || recording.events.length !== 5) throw new Error('Incomplete link save recording');
  return recording;
}

/**
 * Compare each save stage byte for byte with cardHtml, preserving every recorded whitespace byte.
 * @returns {Array<[string,Function]>} Named assertions; failures never print private recorded values.
 */
export function linkRenderCases() {
  return linkSaveRecording().changes.map((change, index) => [`link save stage ${[1, 2, 4][index]}`, () => {
    const data = JSON.parse(change.cardData);
    const extras = { id: change.object.id, capturePending: true, screenshotPending: data.type === 'WebPage' };
    const actual = renderCard(data, extras);
    const bytes = Buffer.from(actual), expected = Buffer.from(change.cardHtml);
    const offset = expected.findIndex((byte, i) => byte !== bytes[i]);
    ok(actual === change.cardHtml, `recorded cardHtml differs at byte ${offset}`);
  }]);
}

// Decode the real hub's WebSocket output without opening a listener or contacting any account.
function recordingHub() {
  const server = new EventEmitter(), socket = new EventEmitter(), messages = [];
  const push = createHub(server);
  socket.destroyed = false;
  socket.write = bytes => {
    if (bytes.toString().startsWith('HTTP/1.1')) return;
    const size = bytes[1] & 127;
    const offset = size === 126 ? 4 : size === 127 ? 10 : 2;
    for (const frame of bytes.subarray(offset).toString().split('\x1e').filter(Boolean)) messages.push(JSON.parse(frame));
  };
  socket.destroy = socket.end = () => { socket.destroyed = true; socket.emit('close'); };
  server.emit('upgrade', { url: '/hub', headers: { 'sec-websocket-key': 'bG9jYWwtZml4dHVyZQ==' } }, socket, Buffer.alloc(0));
  const handshake = Buffer.from('{"protocol":"json","version":1}\x1e');
  socket.emit('data', Buffer.concat([Buffer.from([0x81, 0x80 | handshake.length, 0, 0, 0, 0]), handshake]));
  return { push, messages, close: socket.end };
}

/**
 * Exercise the recorded failure lifecycle through the writer, capture queue and real SignalR encoder.
 * @param {object} context Isolated capture-check dependencies.
 * @param {Function} context.check Named assertion runner that accumulates failures.
 * @param {object} context.server Temporary server with its copied database path.
 * @param {object} context.fixtures Offline HTTP fixtures and request log.
 * @param {Function} context.row Decode one persisted test row.
 * @returns {Promise<void>} Stage and event comparisons have completed; no recorded data is written.
 */
export async function checkLinkCapture({ check, server, fixtures, row }) {
  const keys = ['GEMINI_API_KEY', 'GEMINI_BASE_URL', 'AI_CHAT_BASE_URL'];
  const previous = Object.fromEntries(keys.map(key => [key, process.env[key]]));
  Object.assign(process.env, { GEMINI_API_KEY: 'fixture-key', GEMINI_BASE_URL: fixtures.base, AI_CHAT_BASE_URL: fixtures.base });
  const recorded = linkSaveRecording(), hub = recordingHub();
  const store = openStore(server.dbPath), media = createMedia(join(dirname(server.dbPath), 'media'));
  const snapshots = [], before = fixtures.requests.length;
  const writer = createWriter(store, (target, payload) => {
    hub.push(target, payload);
    snapshots.push({ target, row: row(payload.id ?? payload.objectId) });
  });
  let shots = 0;
  const jobs = createEnrichment({ store, writer, media, offline: true, screenshot: async () => {
    shots++;
    throw new Error('Fixture download has no screenshot');
  } });
  const { body: { id } } = writer.run(() => writer.create({ content: `${fixtures.base}/failed.apk` }));
  // Substitute only synthetic identity/source fields; the expected layout is entirely recorded cardHtml.
  const compareStage = (snapshot, change) => {
    ok(snapshot, 'Missing lifecycle snapshot');
    const expectedData = JSON.parse(change.cardData), actual = snapshot.data;
    strictEqual(actual.type, expectedData.type);
    strictEqual(actual.flags, expectedData.flags);
    let expected = change.cardHtml;
    for (const [a, b] of [[change.object.id, id], [expectedData.source.url, actual.source.url],
      [expectedData.domain, actual.domain]]) expected = expected.replaceAll(escapeHtml(a), escapeHtml(b));
    ok(snapshot.html === expected, 'Persisted cardHtml differs from recording with synthetic fields');
  };
  try {
    await check('live save stage 1: Placeholder bytes, flags and committed creation event', () => compareStage(snapshots[0]?.row, recorded.changes[0]));
    jobs.schedule(id, true);
    await jobs.drain();
    await check('live save stage 2: fetched WebPage bytes and flags', () => compareStage(snapshots.find(s => s.target === 'object:updated')?.row, recorded.changes[1]));
    await check('live save stage 3: ai:failed and exception match recorded payloads and order', () => {
      const actual = hub.messages.filter(m => ['ai:failed', 'exception'].includes(m.target));
      const expected = recorded.events.filter(m => ['ai:failed', 'exception'].includes(m.target));
      ok(JSON.stringify(actual) === JSON.stringify(expected).replaceAll(recorded.changes[0].object.id, id), 'AI failure frames differ from recording');
    });
    await check('live save stage 4: screenshot failure preserves recorded spinner and flags', () => {
      compareStage(snapshots.find(s => s.target === 'screenshot:failed')?.row, recorded.changes[2]);
      strictEqual(shots, 1);
      strictEqual(fixtures.requests.slice(before).filter(r => r.path.includes('completions') || r.path.includes('embedContent')).length, 0);
    });
    for (const event of recorded.events) await check(`live save hub frame: ${event.target}`, () => {
      const actual = hub.messages.filter(m => m.type === 1)[recorded.events.indexOf(event)];
      ok(JSON.stringify(actual) === JSON.stringify(event).replaceAll(recorded.changes[0].object.id, id), 'SignalR frame differs from recording');
    });
    await check('live save emits exactly the recorded sequence and remains settled on rescheduling', async () => {
      const messages = hub.messages.filter(m => m.type === 1);
      deepStrictEqual(messages.map(m => m.target), recorded.events.map(m => m.target));
      const requests = fixtures.requests.length;
      jobs.schedule(id, true); await jobs.drain();
      strictEqual(fixtures.requests.length, requests);
      strictEqual(hub.messages.filter(m => m.type === 1).length, recorded.events.length);
      compareStage(row(id), recorded.changes[2]);
    });
    await check('unrecorded screenshot failure with usable text retains fallback and successful AI behavior', async () => {
      const other = writer.run(() => writer.create({ content: `${fixtures.base}/NoImage?failed=1` })).body.id;
      jobs.schedule(other, true); await jobs.drain();
      const saved = row(other);
      strictEqual(saved.data.type, 'WebPage'); strictEqual(saved.data.flags, 0);
      ok(saved.html.includes('inaccessible-guts'));
      ok(!saved.extras.screenshotPending && !saved.extras.screenshotFailed);
      strictEqual(saved.data.summary, 'A fixture summary <safe>.');
      ok(!hub.messages.some(m => ['ai:failed', 'exception', 'screenshot:failed'].includes(m.target) && (m.arguments[0].id ?? m.arguments[0].objectId) === other));
    });
  } finally {
    jobs.close(); hub.close(); store.close();
    for (const [key, value] of Object.entries(previous)) { if (value === undefined) delete process.env[key]; else process.env[key] = value; }
  }
}
