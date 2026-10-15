import http from 'node:http';
import https from 'node:https';
import { lookup as resolveAddress } from 'node:dns';
import { isIP, BlockList } from 'node:net';
import { createGunzip, createInflate, createBrotliDecompress } from 'node:zlib';
import { Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { sourceUrl } from './capture-html.mjs';

const blocked = new BlockList();
for (const [address, prefix] of [['0.0.0.0', 8], ['10.0.0.0', 8], ['100.64.0.0', 10], ['127.0.0.0', 8],
  ['169.254.0.0', 16], ['172.16.0.0', 12], ['192.0.0.0', 24], ['192.0.2.0', 24], ['192.88.99.0', 24],
  ['192.168.0.0', 16], ['198.18.0.0', 15], ['198.51.100.0', 24], ['203.0.113.0', 24], ['224.0.0.0', 3]]) blocked.addSubnet(address, prefix, 'ipv4');
for (const [address, prefix] of [['2001::', 23], ['2001:db8::', 32], ['2002::', 16], ['3fff::', 20]]) blocked.addSubnet(address, prefix, 'ipv6');
// Honest self-identification; sites such as Wikimedia answer 403 to requests without a user agent.
const USER_AGENT = 'mymind-replica/1.0 (personal link capture)';
const globalV6 = new BlockList();
globalV6.addSubnet('2000::', 3, 'ipv6');

// Identify a policy rejection without exposing destinations in errors or logs.
function forbidden() { return Object.assign(new Error('Unsupported capture destination'), { code: 'CAPTURE_BLOCKED', status: 400 }); }

/**
 * Test a resolved IP against public unicast ranges, rejecting mapped/compatible and special IPv6.
 * @param {string} address DNS result or literal IP, without URL brackets.
 * @returns {boolean} True only for a public unicast address.
 */
export function publicAddress(address) {
  const family = isIP(address);
  if (family === 4) return !blocked.check(address, 'ipv4');
  return family === 6 && globalV6.check(address, 'ipv6') && !blocked.check(address, 'ipv6');
}

/**
 * Validate URL syntax and literal destinations before any request or redirect.
 * @param {string} value Absolute HTTP(S) URL without credentials or live account origin.
 * @param {boolean} [offline=false] Allow only loopback fixture hosts and addresses.
 * @returns {string} Normalized URL; throws a CAPTURE_BLOCKED error for rejected targets.
 */
export function captureDestination(value, offline = false) {
  const safe = sourceUrl(value, value);
  if (!safe) throw forbidden();
  const url = new URL(safe);
  const host = url.hostname.replace(/^\[|\]$/g, '');
  if (offline) {
    if (!['127.0.0.1', 'localhost', '::1'].includes(host)) throw forbidden();
  } else {
    if (host.replace(/\.$/, '').toLowerCase() === 'localhost' || host.endsWith('.localhost')) throw forbidden();
    const rawHost = /^https?:\/\/([^/?#]+)/i.exec(String(value))?.[1]?.replace(/:\d+$/, '').toLowerCase();
    if (isIP(host) === 4 && rawHost !== host) throw forbidden();
    if (isIP(host) && !publicAddress(host)) throw forbidden();
  }
  return url.href;
}

/**
 * Create a GET-only transport whose actual socket lookup pins a validated address.
 * Dependency seams exist for offline streaming/DNS fixtures; production uses Node's native transport.
 * @param {object} [dependencies] Offline fixture seams.
 * @param {Function} [dependencies.lookup] Node-compatible DNS lookup; defaults to node:dns.lookup.
 * @param {Function} [dependencies.request] Node-compatible HTTP request; defaults to protocol-specific request.
 * @returns {Function} fetchResource(value, options), yielding bounded decoded bytes, URL, MIME, status and headers.
 */
export function createCaptureTransport({ lookup = resolveAddress, request } = {}) {
  return async function fetchResource(value, { offline = false, limit = 8 * 1024 * 1024, budget } = {}) {
    let url = captureDestination(value, offline);
    for (let redirects = 0; redirects <= 5; redirects++) {
      if (budget && budget.remaining <= 0) throw new Error('Capture resource too large');
      const target = new URL(url);
      // Validate all answers and return those exact addresses to the connecting socket, never a second lookup.
      const validatingLookup = (hostname, options, callback) => {
        lookup(hostname, { all: true, verbatim: true }, (error, addresses) => {
          if (error) return callback(error);
          if (!addresses?.length || addresses.some(({ address }) => offline ? !['127.0.0.1', '::1'].includes(address) : !publicAddress(address))) return callback(forbidden());
          const matches = options.family ? addresses.filter(item => item.family === options.family) : addresses;
          if (!matches.length) return callback(forbidden());
          if (options.all) callback(null, matches);
          else callback(null, matches[0].address, matches[0].family);
        });
      };
      const response = await new Promise((resolve, reject) => {
        const send = request ?? (target.protocol === 'https:' ? https.request : http.request);
        const req = send(target, { method: 'GET', agent: false, lookup: validatingLookup,
          headers: { 'accept-encoding': 'gzip, deflate, br', 'user-agent': USER_AGENT }, signal: AbortSignal.timeout(15000) }, resolve);
        req.once('error', reject);
        req.end();
      });
      const status = response.statusCode;
      const headers = { ...response.headers };
      if ([301, 302, 303, 307, 308].includes(status)) {
        response.destroy();
        if (!headers.location) throw new Error('Invalid capture redirect');
        const next = sourceUrl(headers.location, url);
        if (!next) throw forbidden();
        url = captureDestination(next, offline);
        continue;
      }
      if (status < 200 || status >= 300) { response.destroy(); throw new Error('Page unavailable'); }
      const encoding = String(headers['content-encoding'] ?? 'identity').toLowerCase();
      const decoder = new Map([['gzip', createGunzip], ['deflate', createInflate], ['br', createBrotliDecompress]]).get(encoding);
      if (!decoder && encoding !== 'identity') { response.destroy(); throw new Error('Unsupported capture encoding'); }
      const chunks = [];
      let encoded = 0, decoded = 0;
      // Charge both wire and decoded bytes, including failed resources, to bound concurrent screenshot work.
      const count = stage => new Transform({ transform(chunk, _encoding, callback) {
        if (stage === 'wire') encoded += chunk.length;
        else decoded += chunk.length;
        if (budget) budget.remaining -= chunk.length;
        if (encoded > limit || decoded > limit || (budget && budget.remaining < 0)) return callback(new Error('Capture resource too large'));
        if (stage === 'body') chunks.push(chunk);
        callback(null, stage === 'wire' ? chunk : undefined);
      } });
      const timer = setTimeout(() => response.destroy(new Error('Capture request timed out')), 15000);
      try { await pipeline(response, count('wire'), ...(decoder ? [decoder()] : []), count('body')); }
      finally { clearTimeout(timer); }
      for (const name of ['content-length', 'content-encoding', 'transfer-encoding', 'set-cookie']) delete headers[name];
      return { bytes: Buffer.concat(chunks), url, mime: headers['content-type'] ?? '', status, headers };
    }
    throw new Error('Too many capture redirects');
  };
}

/**
 * Fetch a public resource through validated socket DNS and bounded streaming decompression.
 * @param {string} value Public URL; every redirect uses the same destination policy.
 * @param {object} [options] offline, per-resource limit and optional shared {remaining} byte budget.
 * @returns {Promise<object>} Decoded bytes, final URL, MIME, status and sanitized response headers.
 */
export const fetchResource = createCaptureTransport();
