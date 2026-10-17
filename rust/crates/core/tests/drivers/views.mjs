import path from 'node:path';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createStore } = await lib('store.mjs');
const { renderPinned, renderDetail, renderShare } = await lib('render/views.mjs');

const SHARE = { url: '/shares/abc<d>', expires: '2026-10-18T00:00:00.000Z' };
const CONTENT = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'reader <b>' }] }] };

export function all(file) {
  const store = createStore(openSql(file));
  const ids = store.db.prepare('SELECT id FROM cards ORDER BY id').all().map((row) => row.id);
  return ids.map((id) => {
    const row = store.card(id);
    const attempt = (work) => {
      try {
        return work();
      } catch (error) {
        return { threw: error.constructor.name };
      }
    };
    return {
      id,
      pinned: attempt(() => renderPinned(row)),
      detail: attempt(() => renderDetail(row)),
      shared: attempt(() => renderDetail(row, SHARE, CONTENT)),
      share: attempt(() => renderShare(row)),
    };
  });
}
