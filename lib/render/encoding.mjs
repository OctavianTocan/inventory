/**
 * Encode a value using the server's default Razor HTML encoder.
 * @param {unknown} value Text or an attribute value; nullish values become empty.
 * @returns {string} HTML with non-ASCII code points encoded as hexadecimal entities.
 */
export function escapeHtml(value) {
  return String(value ?? '').replace(/[&<>"'+]|[^\x20-\x7e]/gu, char => {
    return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#x27;' }[char]
      ?? `&#x${char.codePointAt(0).toString(16).toUpperCase()};`;
  });
}

/**
 * Encode text inside the server's ProseMirror serializer.
 * @param {unknown} value Plain text, with nullish values treated as empty.
 * @returns {string} Text safe to insert in an HTML text node or quoted attribute.
 */
export function escapeProse(value) {
  return escapeHtml(value);
}

/**
 * Decode the named and numeric entities emitted by the recorded templates.
 * @param {string} value Entity-encoded text.
 * @returns {string} Decoded Unicode text; unknown named entities are retained.
 */
export function unescapeHtml(value) {
  return value.replace(/&(#x[\da-f]+|#\d+|amp|lt|gt|quot|apos|nbsp);/gi, (entity, name) => {
    if (name[0] === '#') return String.fromCodePoint(Number.parseInt(name.slice(name[1].toLowerCase() === 'x' ? 2 : 1), name[1].toLowerCase() === 'x' ? 16 : 10));
    return { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: '\u00a0' }[name.toLowerCase()] ?? entity;
  });
}
