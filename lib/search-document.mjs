import { plainText } from './capture-html.mjs';

/**
 * Extract prose text from editor nodes without indexing metadata or node attributes.
 * @param {object|null} node Prose document or child node.
 * @returns {string} Space-separated visible text.
 */
export function proseText(node) {
  return [node?.text ?? '', ...(node?.content ?? []).map(proseText)].filter(Boolean).join(' ');
}

/**
 * Build the analysis item shared by newly saved cards and the text embedding backfill.
 * @param {object} row Store card row with serialized data and extras.
 * @param {object|null} [content=null] Current Article reader document, when available.
 * @returns {object} AI item; callers may attach an image for new-card analysis.
 */
export function analysisItem(row, content = null) {
  const data = JSON.parse(row.data), extras = JSON.parse(row.extras);
  const text = data.type === 'Article' ? proseText(content ?? data.prose)
    : [...new Set([data.description, proseText(data.prose), extras.captureText, plainText(extras.post?.bodyHtml)].filter(Boolean))].join('\n');
  return { type: data.type, title: data.title, text: text.slice(0, 30000), url: data.source?.url, domain: data.domain };
}

/**
 * Extract searchable fields from a card, its current reader document and authoritative annotations.
 * @param {string} dataJson Serialized card data.
 * @param {string} extrasJson Serialized capture/render metadata.
 * @param {string|null} contentJson Serialized current reader body.
 * @param {string|null} tags Annotation names joined with newlines.
 * @returns {string} JSON fields for the persistent external-content FTS table.
 */
export function searchDocument(dataJson, extrasJson, contentJson, tags) {
  const data = JSON.parse(dataJson), extras = JSON.parse(extrasJson);
  const prose = contentJson ? JSON.parse(contentJson) : data.prose;
  const body = data.type === 'Article' && prose ? proseText(prose)
    : [proseText(prose), extras.captureText, plainText(extras.post?.bodyHtml)].filter(Boolean).join('\n');
  return JSON.stringify({
    title: data.title ?? extras.title ?? '', description: plainText(data.description), summary: data.summary ?? '',
    tags: tags ?? '', url: [data.source?.url, extras.linkUrl, data.domain, data.siteName].filter(Boolean).join(' '),
    body,
    note: proseText(data.note?.prose), ocr: data.ocr ?? '',
  });
}
