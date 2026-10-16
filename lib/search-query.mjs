const TYPE_ALIASES = {
  XPost: 'x-post post tweet', WebPage: 'web-page webpage website bookmark',
  SoftwareApplication: 'software-application app', Content: 'content snippet text',
  Quotation: 'quotation quote text', Quote: 'quotation quote text', Note: 'note text',
  YouTubeVideo: 'youtube-video video', VimeoVideo: 'vimeo-video video',
  InstagramReel: 'instagram-reel instagramreel video reel post', InstagramPost: 'instagram-post post',
  ThreadsPost: 'threads-post post', BlueskyPost: 'bluesky-post post', SubstackNote: 'substack-note post',
  TikTokPost: 'tiktok-post tiktok-video post video', Embed: 'embed video', TedTalk: 'ted-talk video',
  RealProperty: 'real-property real-estate house', Palette: 'palette color colour',
  MusicRecording: 'music-recording track song', MusicAlbum: 'music-album album',
  MusicPlaylist: 'music-playlist playlist', VideoGame: 'video-game game',
};

/**
 * Return the type slugs understood by the shipped client's canonicalizer and named queries.
 * @param {string} type Stored PascalCase entity type.
 * @returns {string[]} Exact type and client aliases, including parent categories.
 */
export function typeSlugs(type) {
  const slug = type.replace(/([a-z0-9])([A-Z])/g, '$1-$2').replace(/([A-Z])([A-Z][a-z])/g, '$1-$2').toLowerCase();
  return [...new Set([...(TYPE_ALIASES[type] ?? '').split(' ').filter(Boolean), slug])];
}

// Read quoted values and date ranges as single tokens, preserving literal punctuation inside them.
function tokens(text) {
  const out = [];
  for (let at = 0; at < text.length;) {
    if (/\s/.test(text[at])) { at++; continue; }
    const pair = text.slice(at, at + 2);
    if (['&&', '||'].includes(pair)) { out.push(pair); at += 2; continue; }
    if ('()-!+'.includes(text[at])) { out.push(text[at++]); continue; }
    let word = '', quote = false, range = false;
    while (at < text.length) {
      const char = text[at];
      if (char === '\\' && at + 1 < text.length) { word += char + text[++at]; at++; continue; }
      if (char === '"') quote = !quote;
      if (!quote && char === '[') range = true;
      if (!quote && char === ']') range = false;
      if (!quote && !range && (/\s/.test(char) || '()'.includes(char) || ['&&', '||'].includes(text.slice(at, at + 2)))) break;
      word += char; at++;
    }
    if (word) out.push(word);
  }
  return out;
}

// Decode one field/value term without ever passing caller syntax directly to SQLite MATCH.
function atom(raw) {
  const boost = /\^(\d+(?:\.\d+)?)$/.exec(raw);
  if (boost) raw = raw.slice(0, boost.index);
  const fieldMatch = /^([a-z]+):/i.exec(raw);
  const field = fieldMatch ? fieldMatch[1].toLowerCase() : raw.startsWith('#') ? 'tag' : '';
  let value = raw.slice(fieldMatch?.[0].length ?? (field === 'tag' ? 1 : 0));
  const quoted = value.startsWith('"');
  let prefix = !quoted && value.endsWith('*');
  if (prefix) value = value.slice(0, -1);
  if (quoted) value = value.replace(/^"|"(?:~\d+)?$/g, '').replace(/\\(.)/g, '$1');
  return { kind: 'term', field, value, quoted, prefix, boost: Math.min(100, Number(boost?.[1] ?? 1)) };
}

/**
 * Parse client query syntax into an expression tree with AND precedence over OR.
 * @param {string} query User text; limited to 8,192 characters and 256 terms.
 * @param {boolean} [wildcard=false] Prefix-match the final unquoted free-text term.
 * @returns {{tree:object, terms:object[], query:string}} Safe terms and normalized display query.
 */
export function parseQuery(query, wildcard = false) {
  const text = String(query ?? '').trim().replace(/[“”«»]/g, '"');
  if (text.length > 8192) throw Object.assign(new Error('Search query is too long'), { status: 400 });
  const parts = tokens(text);
  if (parts.length > 256) throw Object.assign(new Error('Search query has too many terms'), { status: 400 });
  const terms = [];
  let at = 0;
  const isOr = part => part === 'OR' || part === '||';
  const isAnd = part => part === 'AND' || part === '&&';
  // Parse negation and groups before combining adjacent terms as implicit AND.
  function unary() {
    const part = parts[at++];
    if (['-', '!', 'NOT'].includes(part)) return { kind: 'not', child: unary() };
    if (part === '+') return unary();
    if (part === '(') {
      const node = disjunction();
      if (parts[at] === ')') at++;
      return node;
    }
    if (!part || part === ')') return { kind: 'empty' };
    let raw = part;
    if (/^(tag|label):[^"[]/i.test(raw) || raw.startsWith('#')) {
      while (at < parts.length && !isOr(parts[at]) && !isAnd(parts[at]) && !['(', ')', '-', '!', '+', 'NOT'].includes(parts[at])
        && !/^[a-z]+:|^#|^"/i.test(parts[at])) raw += ' ' + parts[at++];
    }
    if (/^[a-z]+:$/i.test(raw) && parts[at] && !['(', ')', '-'].includes(parts[at])) raw += parts[at++];
    const node = atom(raw);
    terms.push(node);
    return node;
  }
  // Intersect terms until a group boundary or lower-precedence OR.
  function conjunction() {
    let node = unary();
    while (at < parts.length && parts[at] !== ')' && !isOr(parts[at])) {
      if (isAnd(parts[at])) at++;
      node = { kind: 'and', left: node, right: unary() };
    }
    return node;
  }
  // Union groups without allowing an OR to consume a closing parenthesis.
  function disjunction() {
    let node = conjunction();
    while (isOr(parts[at])) { at++; node = { kind: 'or', left: node, right: conjunction() }; }
    return node;
  }
  const tree = parts.length ? disjunction() : { kind: 'all' };
  const last = terms.at(-1);
  const append = wildcard && last && !last.field && !last.quoted && !last.prefix && !text.endsWith(')');
  if (append) last.prefix = true;
  return { tree, terms, query: text + (append ? '*' : '') };
}

/**
 * Evaluate a parsed expression using score maps supplied by the search engine.
 * @param {object} node Parsed expression node.
 * @param {Map<string,number>} universe Visible cards with a neutral score.
 * @param {(term:object,negative:boolean)=>Map<string,number>} resolve Resolve one term.
 * @param {boolean} [negative=false] Whether positive text is under an odd number of negations.
 * @returns {Map<string,number>} Matching ids with combined scores.
 */
export function evaluate(node, universe, resolve, negative = false) {
  if (node.kind === 'all') return new Map(universe);
  if (node.kind === 'empty') return new Map();
  if (node.kind === 'term') return resolve(node, negative);
  if (node.kind === 'not') {
    const excluded = evaluate(node.child, universe, resolve, !negative);
    return new Map([...universe].filter(([id]) => !excluded.has(id)));
  }
  const left = evaluate(node.left, universe, resolve, negative), right = evaluate(node.right, universe, resolve, negative);
  if (node.kind === 'and') return new Map([...left].filter(([id]) => right.has(id)).map(([id, score]) => [id, score + right.get(id)]));
  for (const [id, score] of right) left.set(id, (left.get(id) ?? 0) + score);
  return left;
}
