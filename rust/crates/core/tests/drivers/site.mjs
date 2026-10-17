// lib/site.mjs's helpers with plain arguments, for mm-core's site tests.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const site = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib/site.mjs')).href);

export const { localize, localizeValue } = site;

/** `sendRange` for a request with this Range header (null for none), as status, header entries and body. */
export function sendRange(range, status, body, headers) {
  let out;
  const res = {
    writeHead(code, written) { out = { status: code, headers: Object.entries(written) }; },
    end(sent) { out.body = typeof sent === 'string' ? new TextEncoder().encode(sent) : new Uint8Array(sent); },
  };
  site.sendRange({ headers: range === null ? {} : { range } }, res, status, body, headers);
  return out;
}

/** `replayKeyFor` for a request target on the replica, with this Sec-Fetch-Dest (null for none). */
export function replayKeyFor(method, dest, target) {
  if (!URL.canParse(target, 'http://localhost:4321')) return null;
  return site.replayKeyFor({ method, headers: dest === null ? {} : { 'sec-fetch-dest': dest } }, new URL(target, 'http://localhost:4321'));
}
