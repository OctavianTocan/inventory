import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

/**
 * Import transport metadata and recorded reader documents once for a seed or legacy database.
 * Space detail recordings have an older query-check time than the list recording;
 * only that presentation field is retained, never a recorded card or space body.
 * @param {object} store Open SQLite store; existing metadata is left intact.
 * @param {string} root Project root containing replay recordings.
 * @returns {Promise<void>} Resolves after the metadata transaction commits.
 */
export async function importReadMetadata(store, root) {
  if (store.db.prepare('SELECT COUNT(*) AS n FROM api_metadata').get().n) return;
  const index = JSON.parse(await readFile(join(root, 'replay/index.json'), 'utf8').catch(() => '{}'));
  const rows = [];
  const contents = new Map();
  let account;
  // Retain recorded reader documents so their first local edit starts with the captured prose.
  function rememberContent(path, bytes) {
    const match = /^\/(?:objects\/([^/]+)\/content|cards\/([^/]+)\/reader)$/.exec(path);
    if (!match || !store.card(match[1] ?? match[2])) return;
    let text = bytes.toString('utf8');
    if (match[2]) text = text.match(/<mymind-editor\b[^>]*>[\s\S]*?<script\b[^>]*>([\s\S]*?)<\/script>/)?.[1] ?? '';
    if (!text.trim()) return;
    try {
      const doc = JSON.parse(text);
      if (doc.type === 'doc') contents.set(match[1] ?? match[2], JSON.stringify(doc));
    } catch {}
  }
  for (const [key, entry] of Object.entries(index)) {
    if (!key.startsWith('GET https://access.mymind.com/')) continue;
    const url = new URL(key.slice(4));
    if (/^\/(objects\/[^/]+\/content|cards\/[^/]+\/reader)$/.test(url.pathname)) {
      rememberContent(url.pathname, await readFile(join(root, 'replay/bodies', entry.file)));
    }
    if (url.pathname === '/account.json') {
      account = JSON.parse(await readFile(join(root, 'replay/bodies', entry.file), 'utf8'));
    }
    if (!/^\/(cards(?:\/[^/]+)?|spaces(?:\/[^/]+)?|tags\/(?:manual|recent)|changes)$/.test(url.pathname)) continue;
    const headers = Object.fromEntries(Object.entries(entry.headers).filter(([name]) =>
      ['content-type', 'x-partial', 'access-control-expose-headers'].includes(name)));
    const attributes = {};
    if (/^\/spaces\/[^/]+$/.test(url.pathname)) {
      const space = JSON.parse(await readFile(join(root, 'replay/bodies', entry.file), 'utf8'));
      if (space.query?.checked) attributes.checked = space.query.checked;
    }
    rows.push([url.pathname + url.search, entry.status, JSON.stringify(headers), JSON.stringify(attributes)]);
  }
  const responses = JSON.parse(await readFile(join(root, 'capture/responses.json'), 'utf8').catch(() => '{}'));
  for (const [key, entry] of Object.entries(responses)) {
    if (/^GET \/(objects\/[^/]+\/content|cards\/[^/]+\/reader)$/.test(key) && !contents.has(key.split('/')[2])) {
      rememberContent(key.slice(4), await readFile(join(root, 'capture', entry.file)));
    }
  }
  if (!account) {
    const entry = responses['GET /account.json'];
    if (entry) account = JSON.parse(await readFile(join(root, 'capture', entry.file), 'utf8'));
  }
  if (account) rows.push(['/account.json', 200, JSON.stringify({ 'content-type': 'application/json; charset=utf-8' }), JSON.stringify({ account })]);
  store.transaction(() => {
    const insert = store.db.prepare('INSERT OR IGNORE INTO api_metadata VALUES (?, ?, ?, ?)');
    for (const row of rows) insert.run(...row);
    const content = store.db.prepare('INSERT OR IGNORE INTO card_content VALUES (?, ?)');
    for (const [id, body] of contents) content.run(id, body);
  });
}
