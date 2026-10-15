const redundant = {
  YouTubeVideo: ['video'], Video: ['video'], Document: ['pdf', 'document', 'file'],
  WebPage: ['web page', 'webpage'], SoftwareApplication: ['app', 'application', 'mobile app', 'mobile application'],
  XPost: ['tweet', 'post'], ThreadsPost: ['tweet', 'post'], SubstackNote: ['tweet', 'post'], RedditPost: ['tweet', 'post'],
};

/**
 * Normalize tag identity across case, separators, whitespace and simple English plurals.
 * @param {string} value Tag name; its stored spelling is not modified.
 * @returns {string} Comparison key for card deduplication and library vocabulary matching.
 */
export function tagKey(value) {
  return String(value ?? '').trim().toLowerCase().replace(/[-_]+/g, ' ').replace(/\s+/g, ' ').split(' ').map(word => {
    if (word.length > 4 && word.endsWith('ies')) return word.slice(0, -3) + 'y';
    if (/(?:ches|shes|xes|zes|sses)$/.test(word)) return word.slice(0, -2);
    if (word.length > 3 && /s$/.test(word) && !/(?:ss|us|is)$/.test(word)) return word.slice(0, -1);
    return word;
  }).join(' ');
}

/**
 * Keep subject labels while excluding control annotations and labels that repeat the card type.
 * @param {string} name Candidate tag spelling.
 * @param {string} type Renderer card type.
 * @returns {boolean} True when an AI tag adds useful subject information.
 */
export function relevantTag(name, type) {
  const key = tagKey(name);
  if (!key || ['highlight', 'private', 'hidden', 'nsfw', 'website', 'link', 'page'].includes(key)) return false;
  if (key === 'image') return type === 'Image';
  if (key === 'screenshot') return type === 'Screenshot';
  return !(redundant[type] ?? []).includes(key);
}

/**
 * Deduplicate AI tags and reuse the first matching library spelling, preserving existing manual tags.
 * @param {object[]} tags Candidate records with name and optional provider fields.
 * @param {string} type Card type used to remove redundant labels.
 * @param {string[]} [vocabulary=[]] Existing library names, most-used spelling first.
 * @param {string[]} [existing=[]] Names already on this card, never replaced by AI.
 * @returns {object[]} At most ten distinct, relevant AI tag records with canonical spelling.
 */
export function curateTags(tags, type, vocabulary = [], existing = []) {
  const spelling = new Map();
  for (const name of vocabulary) if (!spelling.has(tagKey(name))) spelling.set(tagKey(name), name);
  const seen = new Set(existing.map(tagKey));
  const result = [];
  for (const tag of tags) {
    const key = tagKey(tag.name);
    if (seen.has(key) || !relevantTag(tag.name, type)) continue;
    const name = spelling.get(key) ?? String(tag.name).trim();
    if (name.length > 40 || name.split(/\s+/).length > 4) continue;
    seen.add(key);
    result.push({ ...tag, name });
    if (result.length === 10) break;
  }
  return result;
}
