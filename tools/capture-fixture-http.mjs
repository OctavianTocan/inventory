import http from 'node:http';
import https from 'node:https';
import { EventEmitter } from 'node:events';
import { PassThrough, Readable } from 'node:stream';
import { isIP } from 'node:net';

/**
 * Guard native capture HTTP requests and optionally dispatch fixture handlers without sockets.
 * @param {Function} handler Fixture's Node-compatible request callback.
 * @param {Set<string>} origins Explicitly permitted fixture origins.
 * @param {boolean} inProcess Use memory streams rather than host sockets.
 * @returns {Function} Restore the original HTTP and HTTPS request functions.
 */
export function fixtureHttp(handler, origins, inProcess) {
  const originals = [http.request, https.request];
  for (const [index, protocol] of [http, https].entries()) protocol.request = (value, options, receive) => {
    const url = new URL(value);
    if (!origins.has(url.origin)) throw new Error('Test native request tried to leave fixture server');
    if (!inProcess) return originals[index](value, options, receive);
    const request = new EventEmitter();
    let response;
    request.destroy = error => { if (error) request.emit('error', error); response?.destroy(error); };
    request.end = () => queueMicrotask(async () => {
      try {
        if (!isIP(url.hostname.replace(/^\[|\]$/g, ''))) await new Promise((resolve, reject) => options.lookup(url.hostname, {}, error => error ? reject(error) : resolve()));
        const req = Readable.from([]);
        Object.assign(req, { url: url.pathname + url.search, method: options.method, headers: options.headers });
        response = new PassThrough();
        const res = {
          writeHead(status, headers) { Object.assign(response, { statusCode: status, headers }); receive(response); },
          end(bytes) { response.end(bytes); },
        };
        await handler(req, res);
      } catch (error) { request.destroy(error); }
    });
    return request;
  };
  return () => { [http.request, https.request] = originals; };
}
