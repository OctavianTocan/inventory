// The media container's server: the work of lib/media-processor.mjs and lib/capture-link.mjs that needs sharp,
// ffmpeg, poppler and Chromium, for the Worker's Media Durable Object. Each operation is a POST whose arguments are
// base64url JSON in x-media-args and whose body is the input bytes; errors answer JSON {error, status, code}.
import http from 'node:http';
import { createProcessor } from '../../lib/media-processor.mjs';
import { fetchResource, screenshotPage } from '../../lib/capture-link.mjs';

const PORT = Number(process.env.PORT ?? 8080);
const processor = createProcessor();

const operations = {
  prepare: async (req, { type, name, user, id }) => ({ json: await processor.prepare(req, type, name, user, id) }),
  async rendition(req, { name, relative }) {
    const made = await processor.rendition({ body: req, name }, relative);
    return made ? { bytes: made.body } : { status: 204 };
  },
  image: async (req) => ({ bytes: (await processor.image(req)).data }),
  async resource(req, { url, offline, limit }) {
    const { bytes, ...meta } = await fetchResource(url, { offline, limit });
    return { bytes, headers: { 'x-resource': Buffer.from(JSON.stringify(meta)).toString('base64url') } };
  },
  screenshot: async (req, { url, offline }) => ({ bytes: await screenshotPage(url, { offline }) }),
};

http.createServer(async (req, res) => {
  const { pathname } = new URL(req.url, 'http://container');
  if (pathname === '/health') return res.writeHead(200).end('ok');
  const operation = req.method === 'POST' && Object.hasOwn(operations, pathname.slice(1)) && operations[pathname.slice(1)];
  if (!operation) return res.writeHead(404).end();
  try {
    const args = JSON.parse(Buffer.from(req.headers['x-media-args'] ?? 'e30', 'base64url').toString());
    const result = await operation(req, args);
    if (result.json) return res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(result.json));
    res.writeHead(result.status ?? 200, { 'content-type': 'application/octet-stream', ...result.headers }).end(result.bytes);
  } catch (error) {
    const status = Number.isInteger(error.status) && error.status >= 400 && error.status < 600 ? error.status : 500;
    // Unexpected errors (Chromium's above all) can quote the page URL, and a saved URL can carry its owner's tokens.
    console.error(`[media] ${pathname} failed: ${status} ${error.code ?? error.name ?? 'Error'}`);
    if (!res.headersSent) res.writeHead(status, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ error: error.status ? error.message : 'Media operation failed', status: error.status, code: error.code }));
  }
}).listen(PORT, '0.0.0.0', () => console.log(`media container on :${PORT}`));
