import { strictEqual, deepStrictEqual, ok, rejects } from 'node:assert';
import { EventEmitter } from 'node:events';
import { Readable } from 'node:stream';
import { gzipSync, brotliCompressSync, deflateSync } from 'node:zlib';
import { createCaptureTransport, publicAddress, captureDestination } from '../lib/capture-http.mjs';
import { captureLink, screenshotResource } from '../lib/capture-link.mjs';

// Simulate socket connection lookup and streaming HTTP responses without any real network access.
function transport(responses, addresses = [{ address: '8.8.8.8', family: 4 }]) {
  const calls = [], connections = [];
  let lookups = 0, chunks = 0;
  const fetch = createCaptureTransport({
    lookup(_hostname, options, callback) {
      strictEqual(options.all, true);
      const answers = typeof addresses === 'function' ? addresses(lookups++) : addresses;
      callback(null, answers);
    },
    request(url, options, receive) {
      calls.push(url.href);
      const req = new EventEmitter();
      req.end = () => options.lookup(url.hostname, { all: true }, (error, answers) => {
        if (error) return queueMicrotask(() => req.emit('error', error));
        connections.push(answers);
        const response = responses[calls.length - 1] ?? responses.at(-1);
        const stream = Readable.from((async function* () {
          for (const chunk of response.chunks ?? [Buffer.from(response.body ?? 'fixture')]) { chunks++; yield chunk; }
        })());
        Object.assign(stream, { statusCode: response.status ?? 200, headers: response.headers ?? { 'content-type': 'text/html' } });
        queueMicrotask(() => receive(stream));
      });
      return req;
    },
  });
  return { fetch, calls, connections, chunks: () => chunks };
}

/**
 * Run offline SSRF, connection-time DNS, decompression and screenshot-budget regressions.
 * @param {Function} check Parent assertion runner accepting a name and async callback.
 * @returns {Promise<void>} All transport and capture-failure fixture checks have completed.
 */
export async function checkCaptureHttp(check) {
  await check('A1: reject non-public IPv4/IPv6 and alternate numeric host forms before connecting', async () => {
    const denied = ['0.0.0.0', '10.1.2.3', '100.64.0.1', '100.127.255.254', '127.0.0.1', '169.254.169.254', '172.16.0.1',
      '192.168.1.1', '192.0.2.1', '198.18.0.1', '224.0.0.1', '239.1.1.1', '240.0.0.1', '255.255.255.255',
      '[::]', '[::1]', '[fc00::1]', '[fe80::1]', '[ff02::1]', '[::ffff:127.0.0.1]', '[::127.0.0.1]', '[::ffff:8.8.8.8]',
      '[2001:db8::1]', '2130706433', '0x7f000001', '127.1', '134744072', '0x08080808', '010.010.010.010', 'localhost'];
    const stub = transport([{}]);
    for (const host of denied) await rejects(() => stub.fetch(`http://${host}/`), error => error.code === 'CAPTURE_BLOCKED', host);
    strictEqual(stub.calls.length, 0);
    for (const ip of ['8.8.8.8', '1.1.1.1', '2606:4700:4700::1111']) ok(publicAddress(ip), ip);
    for (const host of ['127.0.0.1', 'localhost', '[::1]']) ok(captureDestination(`http://${host}/`, true));
    await rejects(() => stub.fetch('http://8.8.8.8/', { offline: true }));
  });

  await check('A1: validate the connecting DNS answer, mixed DNS results and redirect rebinding', async () => {
    const unsafe = transport([{}], [{ address: '169.254.169.254', family: 4 }]);
    await rejects(() => unsafe.fetch('https://rebind.invalid/'), error => error.code === 'CAPTURE_BLOCKED');
    strictEqual(unsafe.connections.length, 0);
    const mixed = transport([{}], [{ address: '8.8.8.8', family: 4 }, { address: '::ffff:127.0.0.1', family: 6 }]);
    await rejects(() => mixed.fetch('https://mixed.invalid/'));
    strictEqual(mixed.connections.length, 0);
    const rebind = transport([{ status: 302, headers: { location: '/again' } }, {}], n => [{ address: n ? '127.0.0.1' : '8.8.8.8', family: 4 }]);
    await rejects(() => rebind.fetch('http://rebind.invalid/'), error => error.code === 'CAPTURE_BLOCKED');
    strictEqual(rebind.connections.length, 1);
    const good = transport([{}]);
    strictEqual((await good.fetch('https://public.invalid/')).bytes.toString(), 'fixture');
    deepStrictEqual(good.connections[0], [{ address: '8.8.8.8', family: 4 }]);
    for (const target of ['http://10.0.0.1/', 'http://134744072/', 'http://[fd00::1]/', 'https://access.mymind.com/']) {
      const redirect = transport([{ status: 302, headers: { location: target } }]);
      await rejects(() => redirect.fetch('https://public.invalid/'), error => error.code === 'CAPTURE_BLOCKED');
      strictEqual(redirect.calls.length, 1);
    }
  });

  await check('A1: saved URL, redirects, OG/JSON-LD images, oEmbed and thumbnails fail as readable cards', async () => {
    const cases = [
      ['http://127.0.0.1/', [{}]],
      ['http://public.invalid/', [{ status: 302, headers: { location: 'http://169.254.169.254/' } }]],
      ['http://public.invalid/', [{ body: '<meta property="og:image" content="http://10.0.0.1/image.png">' }]],
      ['http://public.invalid/', [{ body: '<meta property="og:image" content="http://134744072/image.png">' }]],
      ['http://public.invalid/', [{ body: '<meta property="og:image" content="//134744072/image.png">' }]],
      ['http://public.invalid/', [{ body: '<script type="application/ld+json">{"@type":"Article","image":"http://[::ffff:127.0.0.1]/image.png"}</script>' }]],
      ['http://public.invalid/', [{ body: '<link type="application/json+oembed" href="http://100.64.0.1/oembed">' }]],
      ['http://public.invalid/', [{ body: '<link type="application/json+oembed" href="/embed">' }, { body: '{"thumbnail_url":"http://[fe80::1]/image.png"}' }]],
    ];
    for (const [url, responses] of cases) {
      const stub = transport(responses);
      let patch;
      await captureLink({ source: { url }, type: 'WebPage', title: 'Fixture' }, { id: 'fixture', user: {}, media: {}, resource: stub.fetch,
        update: value => { patch = value; }, screenshot: async () => { throw new Error('Unexpected screenshot'); } });
      strictEqual(patch.data.type, 'Placeholder');
      strictEqual(patch.extras.captureFailed, true);
    }
  });

  await check('A2: enforce streaming wire/decoded limits for chunked and compressed resources', async () => {
    const chunked = transport([{ chunks: Array.from({ length: 100 }, () => Buffer.alloc(8192)) }]);
    await rejects(() => chunked.fetch('http://public.invalid/', { limit: 16000 }), /too large/);
    ok(chunked.chunks() < 100, 'stops reading the source before buffering the whole response');
    await rejects(() => transport([{ headers: { 'content-encoding': 'constructor' } }]).fetch('http://public.invalid/'), /Unsupported capture encoding/);
    for (const [encoding, compress] of [['gzip', gzipSync], ['deflate', deflateSync], ['br', brotliCompressSync]]) {
      const bytes = compress(Buffer.alloc(1024 * 1024));
      const compressed = transport([{ chunks: [bytes], headers: { 'content-encoding': encoding } }]);
      await rejects(() => compressed.fetch('http://public.invalid/', { limit: 32000 }), /too large/);
      const bounded = transport([{ chunks: [compress(Buffer.from('hello'))], headers: { 'content-encoding': encoding, 'content-length': '25' } }]);
      const result = await bounded.fetch('http://public.invalid/');
      strictEqual(result.bytes.toString(), 'hello');
      ok(!result.headers['content-encoding'] && !result.headers['content-length']);
    }
  });

  await check('A2: share screenshot budget, abort forbidden subresources and never use route.fetch', async () => {
    const stub = transport([{ body: 'first resource' }]);
    const budget = { remaining: 40 };
    let fulfilled = 0, aborted = 0, blocked = 0;
    const route = url => ({ request: () => ({ method: () => 'GET', url: () => url }),
      fetch: () => { throw new Error('route.fetch is forbidden'); },
      fulfill: async result => { strictEqual(result.body.toString(), 'first resource'); fulfilled++; }, abort: async () => { aborted++; } });
    await screenshotResource(route('http://public.invalid/1'), { resource: stub.fetch, budget });
    await screenshotResource(route('http://public.invalid/2'), { resource: stub.fetch, budget });
    strictEqual(fulfilled, 1); strictEqual(aborted, 1); ok(budget.remaining < 0);
    const before = stub.calls.length;
    await screenshotResource(route('http://public.invalid/3'), { resource: stub.fetch, budget });
    strictEqual(stub.calls.length, before, 'no connection after exhaustion');
    await screenshotResource(route('http://127.0.0.1/private'), { resource: stub.fetch, onBlocked: () => { blocked++; } });
    strictEqual(blocked, 1); strictEqual(aborted, 3);
    for (const remaining of [0, -100]) await rejects(() => stub.fetch('http://public.invalid/', { budget: { remaining } }), /too large/);
    strictEqual(stub.calls.length, before);
  });
}
