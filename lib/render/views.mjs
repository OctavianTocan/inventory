import { escapeHtml as e } from './encoding.mjs';
import { isoFromNs } from '../store.mjs';

// Escape embedded JSON independently of the surrounding HTML element.
function json(value) {
  return JSON.stringify(value).replaceAll('<', '\\u003c');
}

// Build the editor custom element with the action and document the shipped client reads.
function editor(id, kind, prose) {
  const type = kind === 'note' ? 'note' : 'card';
  return `<mymind-editor class="editor" data-type="${type}" data-card-id="${e(id)}" data-prose-class="card-${type === 'note' ? 'note' : 'detail'}-prose" data-action="/objects/${e(id)}/${kind}"><script type="application/json">${json(prose ?? { type: 'doc', content: [] })}</script></mymind-editor>`;
}

/**
 * Render a draggable/pinned preview from current grid markup, without a replay lookup.
 * @param {object} row Stored CardRow containing independently generated html and id.
 * @returns {string} Top of Mind item with the classes and data-id used by the client.
 */
export function renderPinned(row) {
  const cover = row.html.replace(/^<mymind-card class="([^"]*)"[^>]*>/, '<div class="tom__item-cover $1">')
    .replace(/<div class="hover"[^>]*>[\s\S]*?<\/div>/, '').replace(/<\/mymind-card>$/, '</div>');
  return `<div class="tom__item draggable" data-id="${e(row.id)}">${cover}</div>`;
}

/**
 * Render an editable desktop detail for a locally created or modified card.
 * Unchanged recorded details continue through the server's existing replay path.
 * @param {object} row Stored CardRow with JSON data and current grid HTML.
 * @param {object|null} [share=null] Active share model, or null.
 * @param {object|null} [content=null] Stored reader prose for an Article, when available.
 * @returns {string} Detail partial with the shipped title, tags, note and prose editor hooks.
 */
export function renderDetail(row, share = null, content = null) {
  const data = JSON.parse(row.data);
  const id = e(row.id);
  const type = row.html.match(/class="([^"]*)"/)?.[1] ?? '';
  const body = ['Note', 'Content'].includes(data.type) ? editor(row.id, 'prose', data.prose)
    : data.type === 'Article' ? editor(row.id, 'content', content ?? data.prose) : row.html;
  const tags = e(JSON.stringify(data.tags ?? []));
  const highlighted = data.tags?.some(tag => tag.name === 'highlight') ? ' highlighted' : '';
  return `<mymind-card-header><div class="inner"><button on-click="card:close"><span class="icon close"></span></button><span class="title">${e(data.title)}</span><button class="options-button" on-click="dropdown#card-options:toggle"><span class="icon more"></span></button></div><mymind-dropdown data-name="card-options">${data.source?.url ? `<a href="${e(data.source.url)}" target="_blank">Visit original source</a>` : ''}</mymind-dropdown></mymind-card-header>
<mymind-card-detail class="${e(type)} default-surface default-mode${share ? ' shared' : ''}">
<script class="card-data" type="application/json">${json({ id: row.id, type: data.type, share })}</script>
<div class="trc-actions"><button class="toggle-sidebar" on-click="card:expand"></button></div>
<div class="guts"><div class="body inner${highlighted}"><div class="wrapper preview-guts">${body}</div></div>
<mymind-revision-controls style="display:none"><p class="ro-title"></p><p class="ro-text"></p><div class="buttons"><button class="restore">Restore</button><button class="cancel">Cancel</button></div></mymind-revision-controls></div>
<mymind-card-sidebar data-object-id="${id}" data-modified="${isoFromNs(row.modifiedNs, 7)}">
<header><mymind-editable data-action="/objects/${id}"><input class="title" value="${e(data.title)}" placeholder="Title goes here" maxlength="100"/></mymind-editable><div class="line"><time datetime="${isoFromNs(row.createdNs, 7)}">${e(data.created.slice(0, 10))}</time><ul class="object-revision-list"><li class="loading-indicator"></li></ul></div><div class="advanced-options"></div></header>
<div class="scroll"><mymind-tldr${data.flags & 1 ? ' class="processing"' : ''}>${data.flags & 1 ? '<div class="processing-content"><div class="spinner"></div></div>' : data.summary ? `<p>${e(data.summary)}</p>` : ''}</mymind-tldr><mymind-tag-cloud data-object-id="${id}"><div class="header"><label>Tags</label></div><form class="add-tag-form"><div class="field"><input class="input" autocomplete="off" name="content" maxlength="100" required/><input class="ghost" disabled/><ul class="suggestions"></ul><button class="submit" type="submit">Add</button></div><mymind-recent-tag-block></mymind-recent-tag-block></form><div class="guts"><mymind-tag-list object-type="${e(data.type)}" object-type-name="${e(data.type)}" object-type-slug="${e(data.type.toLowerCase())}" tags="${tags}"></mymind-tag-list><div class="cut-off"></div></div></mymind-tag-cloud>
<mymind-card-note class="card-note"><div class="header"><label>Notes</label></div><div class="guts">${editor(row.id, 'note', data.note?.prose)}</div></mymind-card-note></div>
<div class="bottom"><mymind-card-actions class="actions"><a class="card-action add-to-space-action" on-click="card#${id}:addToSpace"><i class="icon"></i><span class="tooltip">Add to space</span><span class="notice"></span><mymind-space-chooser backdrop="transparent"></mymind-space-chooser></a><a class="card-action accord-action" on-click="card#${id}:share"><i class="icon"></i><i class="countdown" expires="${e(share?.expires ?? '')}"></i><i class="progress"></i><span class="tooltip"></span><div class="mobile-progress"><div class="mobile-disk"></div></div></a><a class="card-action delete-action" on-click="card#${id}:delete"><i class="icon"></i><span class="tooltip">Delete</span></a></mymind-card-actions><mymind-share-link on-click="card#${id}:copyShareUrl"><p><label>Share link</label><a href="${e(share?.url ?? '')}" target="_blank">${e(share?.url ?? '')}</a></p><div class="copy"><i class="icon"></i><div class="tooltip">Copy link</div></div></mymind-share-link></div>
</mymind-card-sidebar></mymind-card-detail>`;
}

/**
 * Render a local share page without editing controls or account configuration.
 * @param {object} row CardRow whose active share was already checked by the caller.
 * @returns {string} Standalone HTML containing the current rendered card.
 */
export function renderShare(row) {
  const data = JSON.parse(row.data);
  const html = row.html.replace(/<(?:img|video)\b[^>]*>/g, tag => {
    for (const attribute of ['src', 'srcset']) {
      if (tag.includes(` data-${attribute}=`)) tag = tag.replace(new RegExp(` ${attribute}="[^"]*"`, 'g'), '').replace(` data-${attribute}=`, ` ${attribute}=`);
    }
    return tag;
  });
  return `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>${e(data.title ?? 'Shared card')}</title></head><body>${html}</body></html>`;
}
