import { escapeProse as e } from './encoding.mjs';

// Count visible text and lines before deciding where a card preview ends.
function measure(node) {
  const child = (node.content ?? []).map(measure);
  return {
    chars: (node.text ?? '').length + child.reduce((n, x) => n + x.chars, 0),
    lines: (node.type === 'hardBreak' || ['paragraph', 'heading', 'image'].includes(node.type) ? 1 : 0)
      + child.reduce((n, x) => n + x.lines, 0),
  };
}

// Serialize the document nodes supported by the recorded ProseMirror documents.
function renderNode(node) {
  const children = (node.content ?? []).map(renderNode).join('');
  if (node.type === 'text') {
    let text = e(node.text);
    for (const mark of [...(node.marks ?? [])].reverse()) {
      const tag = { bold: 'strong', italic: 'em', underline: 'u', strike: 's', code: 'code',
        highlight: 'mark', subscript: 'sub', superscript: 'sup' }[mark.type];
      if (tag) text = `<${tag}>${text}</${tag}>`;
      else if (mark.type === 'link') text = `<a href="${e(mark.attrs?.href)}">${text}</a>`;
    }
    return text;
  }
  if (node.type === 'hardBreak') return '<br />';
  if (node.type === 'doc') return children;
  if (node.type === 'wikiLink') {
    const attributes = Object.entries({ id: 'id', target: 'target', targetId: 'target-id', targetType: 'target-type', text: 'text' })
      .filter(([key]) => node.attrs?.[key]).map(([key, name]) => ` data-${name}="${e(node.attrs[key])}"`).join('');
    return `<wiki-link${attributes}><span>${e(node.attrs?.text ?? node.attrs?.target ?? '')}</span></wiki-link>`;
  }
  if (node.type === 'taskList') return `<ul data-type="taskList">${children}</ul>`;
  if (node.type === 'taskItem') return `<li data-type="taskItem" data-checked="${Boolean(node.attrs?.checked)}"><label><input type="checkbox"${node.attrs?.checked ? ' checked' : ''} disabled /><span></span></label><div>${children}</div></li>`;
  if (node.type === 'table') return `<table><tbody>${children}</tbody></table>`;
  if (node.type === 'tableRow') return `<tr>${children}</tr>`;
  if (node.type === 'tableCell' || node.type === 'tableHeader') {
    const tag = node.type === 'tableCell' ? 'td' : 'th';
    return `<${tag} colspan="${e(node.attrs?.colspan ?? 1)}" rowspan="${e(node.attrs?.rowspan ?? 1)}">${children}</${tag}>`;
  }
  const tag = { paragraph: 'p', heading: `h${node.attrs?.level ?? 1}`, bulletList: 'ul', orderedList: 'ol', listItem: 'li', blockquote: 'blockquote', codeBlock: 'pre' }[node.type];
  if (tag) return `<${tag}>${children}</${tag}>`;
  if (node.type === 'horizontalRule') return '<hr />';
  if (node.type === 'image') return `<img src="${e(node.attrs?.src)}" />`;
  throw new TypeError(`Unsupported prose node: ${node.type}`);
}

/**
 * Render a ProseMirror document's card preview, ending at a block boundary.
 * @param {{type:string,content?:object[]}} doc Card's prose document.
 * @returns {string} The inner prose div, including its trailing newline.
 */
export function renderProse(doc) {
  const total = measure(doc);
  let chars = 0, lines = 0;
  const blocks = [];
  for (const node of doc.content ?? []) {
    blocks.push(renderNode(node));
    const size = measure(node);
    chars += size.chars;
    lines += size.lines;
    if (chars > 1000 || lines > 20) break;
  }
  return `<div class="inner body prose ${total.chars > 500 || total.lines > 10 ? 'long' : ''}">\n  ${blocks.join('')}\n</div>\n`;
}
