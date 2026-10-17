import { searchDocument } from './search-document.mjs';

const FIELDS = ['title', 'description', 'summary', 'tags', 'url', 'body', 'note', 'ocr'];
const names = FIELDS.join(', ');
const values = prefix => FIELDS.map(name => `${prefix}.${name}`).join(', ');
// Triggers only queue cards: a Durable Object cannot call JavaScript from SQL.
const dirty = id => `INSERT OR IGNORE INTO search_dirty (id) VALUES (${id});`;

/**
 * Install FTS5 and change tracking, indexing existing rows on the first open. Changed cards are reindexed when
 * {@link refreshSearch} runs.
 * @param {object} db Writable SQL handle from `openSql` or the Durable Object adapter; call after the base schema.
 * @returns {void}
 */
export function installSearch(db) {
  const current = () => db.prepare("SELECT 1 FROM sqlite_master WHERE name = 'search_dirty'").get()
    && !db.prepare("SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND sql LIKE '%search_document(%'").get();
  if (current()) return refreshSearch(db);
  db.transaction(() => {
    if (current()) return;
    const fresh = !db.prepare("SELECT 1 FROM sqlite_master WHERE name = 'search_fts'").get();
    if (fresh) db.exec(`CREATE TABLE search_documents (
      id TEXT NOT NULL UNIQUE REFERENCES cards(id) ON DELETE CASCADE,
      fields TEXT NOT NULL CHECK(json_valid(fields)),
      ${FIELDS.map(name => `${name} TEXT GENERATED ALWAYS AS (json_extract(fields, '$.${name}')) STORED`).join(',\n')}
    );
    CREATE VIRTUAL TABLE search_fts USING fts5(${names}, content='search_documents', content_rowid='rowid',
      tokenize='unicode61 remove_diacritics 2', prefix='2 3 4 5 6');
    CREATE TRIGGER search_documents_insert AFTER INSERT ON search_documents BEGIN
      INSERT INTO search_fts(rowid, ${names}) VALUES (new.rowid, ${values('new')}); END;
    CREATE TRIGGER search_documents_delete AFTER DELETE ON search_documents BEGIN
      INSERT INTO search_fts(search_fts, rowid, ${names}) VALUES ('delete', old.rowid, ${values('old')}); END;
    CREATE TRIGGER search_documents_update AFTER UPDATE ON search_documents BEGIN
      INSERT INTO search_fts(search_fts, rowid, ${names}) VALUES ('delete', old.rowid, ${values('old')});
      INSERT INTO search_fts(rowid, ${names}) VALUES (new.rowid, ${values('new')}); END;`);
    for (const { name } of db.prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND sql LIKE '%search_document(%'").all()) {
      db.exec(`DROP TRIGGER ${name}`);
    }
    db.exec(`CREATE TABLE IF NOT EXISTS search_dirty (id TEXT PRIMARY KEY) STRICT;
    CREATE TRIGGER IF NOT EXISTS search_cards_insert AFTER INSERT ON cards BEGIN ${dirty('new.id')} END;
    CREATE TRIGGER IF NOT EXISTS search_cards_update AFTER UPDATE OF data, extras ON cards BEGIN ${dirty('new.id')} END;
    CREATE TRIGGER IF NOT EXISTS search_cards_delete AFTER DELETE ON cards BEGIN DELETE FROM search_documents WHERE id = old.id; END;`);
    for (const table of ['annotations', 'card_content']) {
      for (const action of ['INSERT', 'UPDATE', 'DELETE']) {
        const ids = action === 'UPDATE' ? ['old.card_id', 'new.card_id'] : [action === 'DELETE' ? 'old.card_id' : 'new.card_id'];
        db.exec(`CREATE TRIGGER IF NOT EXISTS search_${table}_${action.toLowerCase()} AFTER ${action} ON ${table}
          BEGIN ${ids.map(dirty).join('\n')} END;`);
      }
    }
    if (fresh) db.exec('INSERT OR IGNORE INTO search_dirty (id) SELECT id FROM cards');
  });
  refreshSearch(db);
}

/**
 * Reindex the cards changed since the last refresh.
 * @param {object} db Writable SQL handle with the search index installed.
 * @returns {void}
 */
export function refreshSearch(db) {
  if (!db.prepare('SELECT 1 FROM search_dirty LIMIT 1').get()) return;
  db.transaction(() => {
    for (const { id } of db.prepare('SELECT id FROM search_dirty').all()) {
      const row = db.prepare(`SELECT c.data, c.extras, (SELECT body FROM card_content WHERE card_id = c.id) AS content,
        (SELECT group_concat(name, char(10)) FROM annotations WHERE card_id = c.id) AS tags FROM cards c WHERE c.id = ?`).get(id);
      if (row) {
        db.prepare(`INSERT INTO search_documents (id, fields) VALUES (?, ?)
          ON CONFLICT(id) DO UPDATE SET fields = excluded.fields WHERE fields IS NOT excluded.fields`)
          .run(id, searchDocument(row.data, row.extras, row.content, row.tags));
      } else db.prepare('DELETE FROM search_documents WHERE id = ?').run(id);
      db.prepare('DELETE FROM search_dirty WHERE id = ?').run(id);
    }
  });
}
