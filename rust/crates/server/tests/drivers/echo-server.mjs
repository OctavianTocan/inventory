// A Node HTTP server that answers each request with what Node made of it, for the native server's HTTP tests.
// Prints its port, then serves until killed.
import http from 'node:http';

const server = http.createServer(async (req, res) => {
  let url;
  try { url = new URL(req.url, `http://${req.headers.host}`); } catch {
    res.writeHead(400, { 'cache-control': 'no-store' });
    return res.end('Bad request');
  }
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  const headers = Object.fromEntries(Object.entries(req.headers).map(([name, value]) => [name, Array.isArray(value) ? value.join(', ') : value]));
  res.writeHead(200, { 'content-type': 'application/json' });
  res.end(JSON.stringify({ method: req.method, url: url.href, pathname: url.pathname, search: url.search, headers,
    remote: req.socket.remoteAddress, body: Buffer.concat(chunks).toString('latin1') }));
});
server.listen(0, '127.0.0.1', () => console.log(server.address().port));
