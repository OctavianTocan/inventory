import path from 'node:path';
import { pathToFileURL } from 'node:url';

const { evaluate, parseQuery } = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib/search-query.mjs')).href);

const hash = (text) => [...text].reduce((n, c) => (n * 31 + c.codePointAt(0)) % 1000003, 7);

/** Run `evaluate` over a fixed universe with a resolver derived from each term's field, value and prefix flag. */
export function run(query, wildcard, ids, negativeOnly) {
  const { tree } = parseQuery(query, wildcard);
  const universe = new Map(ids.map((id) => [id, 1]));
  const trace = [];
  const resolve = (term, negative) => {
    trace.push([term.field, term.value, term.quoted, term.prefix, term.boost, negative]);
    const seed = hash(`${term.field}:${term.value}:${term.prefix}`);
    const out = new Map();
    ids.forEach((id, index) => {
      if ((seed + hash(id) + index) % 3 === 0) out.set(id, term.boost + (seed % 5) / 4);
    });
    if (negativeOnly && negative) out.set('extra', 0.5);
    return out;
  };
  return { result: [...evaluate(tree, universe, resolve)], trace };
}
