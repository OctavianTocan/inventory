import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

const lib = (name) => import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib', name)).href);
const { openSql } = await lib('sql-node.mjs');
const { createRecorded } = await lib('recorded.mjs');

const look = (recorded, [agent, run, key]) => {
  const headers = {};
  if (agent) headers['user-agent'] = agent;
  if (run) headers['x-replay-run'] = run;
  const req = { headers };
  const entry = recorded.entry(req, key);
  const has = recorded.has(req, key);
  if (!entry) return [false, null, null, null, null, has];
  return [true, entry.status, JSON.stringify(entry.headers), entry.body.length, createHash('sha256').update(entry.body).digest('hex'), has];
};

export function session(file, lookups, puts, after) {
  const recorded = createRecorded(openSql(file));
  const first = lookups.map((lookup) => look(recorded, lookup));
  const before = recorded.count();
  for (const [key, status, headers, body] of puts) recorded.put(key, status, headers, Buffer.from(body, 'base64'));
  return { first, before, count: recorded.count(), after: after.map((lookup) => look(recorded, lookup)) };
}
