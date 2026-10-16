import { searchDocument } from './search-document.mjs';

const FIELDS = ['title', 'description', 'summary', 'tags', 'url', 'body', 'note', 'ocr'];
const names = FIELDS.join(', ');
const values = prefix => FIELDS.map(name => `${prefix}.${name}`).join(', ');

// Rebuild just the changed document; generated columns feed the external-content FTS triggers.
const refresh = id => `INSERT INTO search_documents (id, fields)
  SELECT c.id, search_document(c.data, c.extras,
    (SELECT body FROM card_content WHERE card_id = c.id),
    (SELECT group_concat(name, char(10)) FROM annotations WHERE card_id = c.id))
  FROM cards c WHERE c.id = ${id}
  ON CONFLICT(id) DO UPDATE SET fields = excluded.fields;`;

/**
 * Install FTS5 and transactional synchronization, indexing existing rows on the first open.
 * @param {object} db Writable node:sqlite DatabaseSync; call after the base schema is installed.
 * @returns {void}
 */
export function installSearch(db) {
  db.function('search_document', { deterministic: true }, searchDocument);
  if (db.prepare("SELECT 1 FROM sqlite_master WHERE name = 'search_fts'").get()) return;
  db.exec('BEGIN IMMEDIATE');
  try {
    if (db.prepare("SELECT 1 FROM sqlite_master WHERE name = 'search_fts'").get()) { db.exec('COMMIT'); return; }
    db.exec(`CREATE TABLE search_documents (
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
      INSERT INTO search_fts(rowid, ${names}) VALUES (new.rowid, ${values('new')}); END;
    CREATE TRIGGER search_cards_insert AFTER INSERT ON cards BEGIN ${refresh('new.id')} END;
    CREATE TRIGGER search_cards_update AFTER UPDATE OF data, extras ON cards BEGIN ${refresh('new.id')} END;
    CREATE TRIGGER search_cards_delete AFTER DELETE ON cards BEGIN DELETE FROM search_documents WHERE id = old.id; END;`);
    for (const table of ['annotations', 'card_content']) {
      for (const action of ['INSERT', 'UPDATE', 'DELETE']) {
        const ids = action === 'UPDATE' ? ['old.card_id', 'new.card_id'] : [action === 'DELETE' ? 'old.card_id' : 'new.card_id'];
        db.exec(`CREATE TRIGGER search_${table}_${action.toLowerCase()} AFTER ${action} ON ${table}
          BEGIN ${ids.map(refresh).join('\n')} END;`);
      }
    }
    db.exec(refresh('c.id'));
    db.exec('COMMIT');
  } catch (error) { db.exec('ROLLBACK'); throw error; }
}
