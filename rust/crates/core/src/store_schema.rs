//! Port of `lib/store-schema.mjs`: the tables shared by new stores and startup migrations.

/// SQLite tables created with `IF NOT EXISTS` when a store opens (`SCHEMA`).
pub const SCHEMA: &str = r#"
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

CREATE TABLE IF NOT EXISTS uploads (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users (id),
  token_hash TEXT NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  mime TEXT NOT NULL DEFAULT '',
  size INTEGER,
  received INTEGER NOT NULL DEFAULT 0,
  done INTEGER NOT NULL DEFAULT 0,
  card_id TEXT REFERENCES cards (id),
  expires_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS embeddings (
  card_id TEXT PRIMARY KEY REFERENCES cards (id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users (id),
  dimensions INTEGER NOT NULL CHECK (dimensions = 768),
  vector BLOB NOT NULL CHECK (length(vector) = 3072)
) STRICT;
"#;
