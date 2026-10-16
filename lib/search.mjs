import { performance } from 'node:perf_hooks';
import { GRID_HIDDEN_FLAGS } from './store.mjs';
import { parseQuery, evaluate, typeSlugs } from './search-query.mjs';
import { createSemantic } from './search-semantic.mjs';

const LABELS = { XPost: 'X Posts', WebPage: 'Web Pages', Repository: 'Repositories', Screenshot: 'Screenshots',
  Article: 'Articles', Image: 'Images', Video: 'Videos', InstagramReel: 'Instagram Reels', Note: 'Notes',
  YouTubeVideo: 'YouTube Videos', Product: 'Products', ThreadsPost: 'Threads Posts', SoftwareApplication: 'Apps',
  Document: 'Documents', Content: 'Snippets', RedditPost: 'Reddit Posts', SubstackNote: 'Substack Notes' };
const COLUMNS = { title: 'title', description: 'description', summary: 'summary', tag: 'tags', label: 'tags',
  url: 'url', text: 'body', body: 'body', prose: 'body', note: 'note', ocr: 'ocr' };
const TEXT_FIELDS = new Set(['title', 'description', 'summary', 'text', 'body', 'prose', 'note', 'ocr', 'url']);

// Render only literal token strings into FTS5, never caller-supplied operators or column names.
function ftsTerm(term) {
  if (!/[\p{L}\p{N}]/u.test(term.value)) return null;
  const value = '"' + term.value.replaceAll('"', '""') + '"' + (term.prefix ? '*' : '');
  return (term.field ? `${COLUMNS[term.field]} : ` : '') + value;
}

// Compare date operands as exact nanoseconds, using UTC for date-only values.
function dateValue(value) {
  const ms = Date.parse(/^\d{4}-\d\d-\d\d$/.test(value) ? value + 'T00:00:00Z' : value);
  return Number.isFinite(ms) ? BigInt(ms) * 1_000_000n : null;
}

/**
 * Create SQLite text, suggestion and hosted semantic search for the default user's visible library.
 * @param {object} store Open store with an installed search index.
 * @param {{offline?:boolean}} [options] Offline mode allows explicitly configured loopback embedding stubs only.
 * @returns {object} text, search, filters, suggestions, embeddings and related search methods.
 */
export function createSearch(store, { offline = false } = {}) {
  const { db } = store;
  const userId = store.defaultUserId();
  const visible = `c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & ${GRID_HIDDEN_FLAGS}) = 0`;
  const semantic = createSemantic(store, { offline });

  // Load a small universe that also supplies deterministic timestamp tie-breaks without rounding ns integers.
  function universe() {
    const stmt = db.prepare(`SELECT c.id, c.bumped_ns, c.created_ns, c.modified_ns FROM cards c WHERE ${visible}`);
    stmt.setReadBigInts(true);
    return stmt.all(userId);
  }

  // Resolve one exact structured predicate with bound values and a fixed SQL vocabulary.
  function predicate(term) {
    const { field, value, prefix } = term;
    let sql = '0', args = [];
    if (['tag', 'label'].includes(field)) {
      sql = `EXISTS (SELECT 1 FROM annotations a WHERE a.card_id = c.id AND ${prefix ? 'substr(lower(a.name), 1, length(?)) = lower(?)' : 'lower(a.name) = lower(?)'})`;
      args = prefix ? [value, value] : [value];
    } else if (field === 'type') {
      const types = db.prepare('SELECT DISTINCT type FROM cards WHERE user_id = ?').all(userId)
        .filter(row => typeSlugs(row.type).some(slug => prefix ? slug.startsWith(value.toLowerCase()) : slug === value.toLowerCase())).map(row => row.type);
      sql = `c.type IN (${types.map(() => '?').join(',') || 'NULL'})`; args = types;
    } else if (['site', 'domain'].includes(field)) {
      let host = value.toLowerCase().replace(/^www\./, '');
      try { host = new URL(value.includes('://') ? value : `https://${value}`).hostname.replace(/^www\./, ''); } catch {}
      const domain = "lower(COALESCE(json_extract(c.data, '$.domain'), ''))";
      sql = `${domain} = ? OR ${domain} LIKE ? ESCAPE '\\'`;
      const escaped = host.replace(/[\\%_]/g, '\\$&');
      args = [host, '%.' + escaped];
      if (!host.includes('.')) {
        sql += ` OR instr('.' || ${domain} || '.', ?) > 0`; args.push('.' + host + '.');
      }
    } else if (field === 'format') {
      sql = "lower(COALESCE(json_extract(c.data, '$.format'), '')) = lower(?)"; args = [value];
    } else if (field === 'completed') {
      sql = '(c.flags & 65536) ' + (value === 'true' ? '!=' : '=') + ' 0';
    } else if (field === 'action') {
      const groups = { read: ['Article', 'Book', 'Document', 'WebPage', 'Content'],
        watch: ['Video', 'YouTubeVideo', 'VimeoVideo', 'InstagramReel', 'Movie', 'TVEpisode', 'TVSeries', 'TedTalk'],
        listen: ['Audio', 'Podcast', 'PodcastEpisode', 'MusicRecording', 'MusicAlbum', 'MusicPlaylist'], play: ['Game', 'VideoGame'] };
      args = groups[value.toLowerCase()] ?? [];
      sql = `c.type IN (${args.map(() => '?').join(',') || 'NULL'})`;
    } else if (['before', 'after', 'date', 'created', 'bumped', 'modified'].includes(field)) {
      const column = ['bumped', 'modified'].includes(field) ? `c.${field}_ns` : 'c.created_ns';
      const range = /^\[(.+?)\s+TO\s+(.+?)\]$/i.exec(value);
      if (range) {
        const start = range[1] === '*' ? 0n : dateValue(range[1]);
        const end = range[2] === '*' ? 9_223_372_036_854_775_807n : dateValue(range[2]);
        if (start !== null && end !== null) { sql = `${column} >= ? AND ${column} <= ?`; args = [start, end]; }
      } else {
        const op = /^(>=|<=|>|<)/.exec(value)?.[0];
        const date = dateValue(op ? value.slice(op.length) : value);
        if (date !== null) {
          sql = `${column} ${op ?? (field === 'before' ? '<' : field === 'after' ? '>' : '>=')} ?`; args = [date];
          if (!op && !['before', 'after'].includes(field)) { sql += ` AND ${column} < ?`; args.push(date + 86_400_000_000_000n); }
        }
      }
    }
    return new Map(db.prepare(`SELECT c.id FROM cards c WHERE ${visible} AND (${sql})`).all(userId, ...args).map(row => [row.id, term.boost]));
  }

  // Resolve FTS atoms once per expression, optionally broadening positive text for semantic candidates.
  function matches(parsed, rows, candidates = false) {
    const all = new Map(rows.map(row => [row.id, 1]));
    const cache = new Map();
    return evaluate(parsed.tree, all, (term, negative) => {
      if (candidates && !negative && !term.field) return all;
      if (cache.has(term)) return cache.get(term);
      let result;
      if (term.field && !TEXT_FIELDS.has(term.field)) result = predicate(term);
      else {
        const expression = ftsTerm(term);
        result = expression ? new Map(db.prepare(`SELECT d.id, bm25(search_fts, 4, 2, 1, 4, 1, 2, 2, 2) AS rank
          FROM search_fts JOIN search_documents d ON d.rowid = search_fts.rowid
          JOIN cards c ON c.id = d.id WHERE search_fts MATCH ? AND ${visible}`)
          .all(expression, userId).map(row => [row.id, (1 - row.rank) * term.boost])) : new Map();
      }
      cache.set(term, result);
      return result;
    });
  }

  // Sort results by the client's fixed timestamp options or by descending relevance with stable ties.
  function ordered(scores, rows, sortBy = 'score') {
    const metadata = new Map(rows.map(row => [row.id, row]));
    const sort = /^(bumped|created|modified)(?:\s+(asc|desc))?$/i.exec(sortBy);
    return [...scores].filter(([id]) => metadata.has(id)).map(([id, score]) => ({ id, score }))
      .sort((a, b) => {
        const key = sort ? `${sort[1].toLowerCase()}_ns` : 'bumped_ns';
        const x = metadata.get(a.id)[key], y = metadata.get(b.id)[key];
        const dateOrder = (x > y ? -1 : x < y ? 1 : 0) * (sort?.[2]?.toLowerCase() === 'asc' ? -1 : 1);
        return (sort ? dateOrder : b.score - a.score || dateOrder) || b.id.localeCompare(a.id);
      });
  }

  const api = {
    // Synchronous text search is also the smart-space evaluator inside write transactions.
    text(q, options = {}) {
      const started = performance.now(), rows = universe(), parsed = parseQuery(q, options.wildcard === true);
      const result = ordered(matches(parsed, rows), rows, options.sortBy);
      return { elapsed: Math.round(performance.now() - started), count: result.length, query: parsed.query, matches: result };
    },

    // Blend semantic candidates into lexical scores while preserving explicit predicates and exclusions.
    async search(q, options = {}) {
      if (String(q).startsWith('similar:')) return api.related(String(q).slice(8).trim(), options);
      const started = performance.now(), parsed = parseQuery(q, options.wildcard === true);
      let rows = universe(), scores = matches(parsed, rows);
      const text = parsed.terms.filter(t => !t.field).map(t => t.value).join(' ');
      if (options.semantic === true && text) {
        const boost = Number.isFinite(Number(options.semanticBoost)) ? Math.max(0, Math.min(100, Number(options.semanticBoost))) : 2;
        if (boost > 0) {
          const similar = await semantic.search(text, { take: -1 });
          rows = universe(); scores = matches(parsed, rows);
          const eligible = matches(parsed, rows, true);
          for (const match of similar) {
            if (eligible.has(match.id) && match.score > 0) scores.set(match.id, (scores.get(match.id) ?? 0) + boost * match.score);
          }
        }
      }
      const result = ordered(scores, rows, options.sortBy);
      return { elapsed: Math.round(performance.now() - started), count: result.length, query: parsed.query, matches: result };
    },

    // Filters represent intersecting client bubbles, including whole-expression negation.
    filters(filters) {
      if (!Array.isArray(filters) || filters.some(filter => typeof filter !== 'string')) throw Object.assign(new Error('Expected query filters'), { status: 400 });
      if (!filters.length) return api.text('').matches;
      return api.text(filters.map(filter => `(${filter})`).join(' AND ')).matches;
    },

    // Return types without q, or prefix tag completions with manual tags weighted twice as highly.
    suggestions(q = '') {
      if (!q.trim()) return db.prepare(`SELECT c.type, COUNT(*) AS count FROM cards c WHERE ${visible} AND c.type != 'Placeholder' GROUP BY c.type`)
        .all(userId).map(row => ({ type: 'type', text: LABELS[row.type] ?? row.type.replace(/([a-z])([A-Z])/g, '$1 $2') + 's',
          queryText: 'type:' + typeSlugs(row.type)[0], count: row.count, score: row.count * 3, isAI: false }))
        .sort((a, b) => b.score - a.score || a.text.localeCompare(b.text));
      const prefix = q.trim().replace(/^(?:tag:|label:|#)/i, '').toLowerCase();
      return db.prepare(`SELECT a.name, COUNT(DISTINCT a.card_id) AS count, MAX((a.flags & 8) != 0) AS manual
        FROM annotations a JOIN cards c ON c.id = a.card_id WHERE ${visible}
        AND substr(lower(a.name), 1, length(?)) = ? GROUP BY lower(a.name)`)
        .all(userId, prefix, prefix).map(row => ({ type: 'tag', text: row.name, queryText: 'tag:' + row.name,
          count: row.count, score: row.count * (row.manual ? 2 : 1), isAI: !row.manual }))
        .sort((a, b) => b.score - a.score || a.text.localeCompare(b.text));
    },

    // Legacy model ids select the one hosted model used for all stored vectors.
    async embeddings(q, options = {}) {
      const started = performance.now();
      if (options.filter) parseQuery(options.filter);
      const filter = options.filter ? () => new Set(api.text(options.filter).matches.map(m => m.id)) : null;
      const result = await semantic.search(q, { ...options, eligible: filter });
      return { elapsed: Math.round(performance.now() - started), count: result.length, query: String(q ?? ''), matches: result };
    },

    // Related vectors need no network or key; missing vectors yield an empty matches array.
    related(id, options = {}) {
      const started = performance.now(), result = semantic.related(id, options);
      return { elapsed: Math.round(performance.now() - started), count: result.length, query: `similar:${id}`, matches: result };
    },
  };
  return api;
}
