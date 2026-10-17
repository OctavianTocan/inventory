// Statuses whose responses carry no body.
const EMPTY = new Set([101, 204, 205, 304]);

/**
 * Present a Workers request and a response under construction as the Node `req` and `res` the library code
 * handles: `req` has `method`, `url`, lower-case `headers`, `socket.remoteAddress` and yields its body as Buffers;
 * `res` has `writeHead`, `setHeader`, `end` (with a string, bytes or a web stream), `headersSent` and `destroy`.
 * @param {Request} request Incoming request.
 * @returns {{req: object, res: object, response: () => Promise<Response>}} `response()` resolves the Response once
 *   `res.end` or `res.destroy` has been called.
 */
export function nodePair(request) {
  const url = new URL(request.url);
  const headers = Object.fromEntries(request.headers);
  const body = request.body;
  const req = {
    method: request.method,
    url: url.pathname + url.search,
    headers,
    socket: { remoteAddress: headers['cf-connecting-ip'] ?? '127.0.0.1', localPort: Number(url.port) || (url.protocol === 'https:' ? 443 : 80) },
    async *[Symbol.asyncIterator]() {
      if (!body) return;
      for await (const chunk of body) yield Buffer.from(chunk.buffer, chunk.byteOffset, chunk.byteLength);
    },
  };
  let settle;
  const finished = new Promise((resolve) => { settle = resolve; });
  const outgoing = new Headers();
  const res = {
    statusCode: 200,
    headersSent: false,
    destroyed: false,
    writableEnded: false,
    setHeader(name, value) {
      outgoing.delete(name);
      for (const item of [value].flat()) outgoing.append(name, String(item));
      return this;
    },
    getHeader: (name) => outgoing.get(name) ?? undefined,
    writeHead(status, values = {}) {
      this.statusCode = status;
      for (const [name, value] of Object.entries(values)) if (value !== undefined) this.setHeader(name, value);
      this.headersSent = true;
      return this;
    },
    end(content) {
      if (this.writableEnded) return this;
      this.writableEnded = this.headersSent = true;
      const empty = EMPTY.has(this.statusCode) || content === undefined || content === null;
      // As bytes, a string body gets no content type the code did not set, as in Node.
      const body = typeof content === 'string' ? new TextEncoder().encode(content) : content;
      settle(new Response(empty ? null : body, { status: this.statusCode, headers: outgoing }));
      return this;
    },
    destroy(error) {
      if (this.destroyed) return;
      this.destroyed = true;
      if (!this.writableEnded) {
        this.writableEnded = true;
        settle(new Response(error ? String(error.message ?? error) : null, { status: 500 }));
      }
    },
  };
  return { req, res, response: () => finished };
}
