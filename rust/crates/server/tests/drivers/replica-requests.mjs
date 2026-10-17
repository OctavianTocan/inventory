// The requests the replica differential test sends to `node server.mjs` and to the Rust server: every replayed
// response, every recorded capture response, and the edge cases of each route. Prints them as JSON.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.env.MYMIND_ROOT;
const { APP_ORIGIN, LOCAL_PREFIXES } = await import(join(root, 'lib/site.mjs'));
const read = (file) => JSON.parse(readFileSync(join(root, file), 'utf8'));
const AGENTS = {
  iphone: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148',
  ipad: 'Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148',
  'android-phone': 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/126.0 Mobile Safari/537.36',
  'android-tablet': 'Mozilla/5.0 (Linux; Android 14; Pixel Tablet) AppleWebKit/537.36 Chrome/126.0 Safari/537.36',
  desktop: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36',
};
const requests = [];
const add = (method, target, headers = {}, body = '') => requests.push({ method, target, headers: Object.entries(headers), body });

function localTarget(live) {
  const prefix = LOCAL_PREFIXES.find(([, origin]) => live.startsWith(origin));
  if (prefix) return prefix[0] + live.slice(prefix[1].length);
  return live.startsWith(APP_ORIGIN) ? `/${live.slice(APP_ORIGIN.length)}` : null;
}

for (const key of Object.keys(read('replay/index.json'))) {
  const parts = key.split('|');
  const [kind, live] = parts.pop().split(' ');
  const run = parts.find((part) => part.startsWith('run:'))?.slice('run:'.length);
  const variant = parts.find((part) => !part.startsWith('run:'));
  const target = localTarget(live);
  if (!target) continue;
  const headers = { 'user-agent': AGENTS[variant ?? 'desktop'] };
  if (kind === 'DOC') headers['sec-fetch-dest'] = 'document';
  if (run) headers['x-replay-run'] = run;
  add(kind === 'DOC' ? 'GET' : kind, target, headers, kind === 'GET' || kind === 'DOC' ? '' : '{}');
}
add('GET', '/ui', { 'x-replay-run': 'no-such-run' });
add('GET', '/everything', { 'sec-fetch-dest': 'document', 'user-agent': AGENTS.ipad });
add('GET', '/everything', { 'sec-fetch-dest': 'document', 'user-agent': AGENTS['android-tablet'] });

for (const key of Object.keys(read('capture/responses.json'))) {
  const [method, target] = key.split(' ');
  add(method, target, { accept: 'application/json' }, method === 'GET' ? '' : '{}');
}

const ids = read('capture/cards.json').slice(0, 3).map((card) => /data-id="([^"]+)"/.exec(card.html)[1]);
const detailIds = Object.keys(read('capture/details.json')).slice(0, 5);
add('GET', '/cards');
add('HEAD', '/cards');
for (const body of [JSON.stringify([...ids, 'nope']), '[]', '', 'null', '{"a":1}', '"text"', '[1,null,{}]', 'not json', '[', 'é'])
  add('POST', '/cards/grid', { 'content-type': 'application/json' }, body);
for (const id of [...ids, 'nope', encodeURIComponent(ids[0])]) {
  add('GET', `/cards/${id}`);
  add('GET', `/cards/${id}?template=1`);
}
for (const id of [...detailIds, ids[0], 'nope', 'a%20b', 'x..y', '%ZZ', '%E0%A4%A', '..%2F..%2Fserver', '%00', 'caf%C3%A9', '%3Cb%3E'])
  add('GET', `/cards/${id}/detail`);
add('POST', '/cards/x/detail', {}, '{}');
add('GET', '/changes');
add('POST', '/changes', {}, '{}');
add('GET', '/changes?since=4');

for (const [method, target] of [['POST', '/nothing'], ['PATCH', '/cards/x'], ['DELETE', '/spaces/1'], ['PUT', '/x?y=1'], ['POST', '/jserrors'], ['OPTIONS', '/cards']])
  add(method, target, { 'content-type': 'application/json' }, '{"a":1}');
for (const accept of [undefined, 'application/json', 'text/html', 'application/msgpack, */*', 'APPLICATION/JSON'])
  add('GET', '/no/such/route?x=1', accept ? { accept } : {});
add('HEAD', '/no/such/route');

const staticFiles = readdirSync(join(root, 'mirror/static'), { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile()).map((entry) => join(entry.parentPath, entry.name).slice(join(root, 'mirror/static').length + 1)).sort();
const byExtension = new Map();
for (const file of staticFiles) {
  const extension = /\.[^./]*$/.exec(file)?.[0] ?? '';
  if ((byExtension.get(extension)?.length ?? 0) < 2) byExtension.set(extension, [...(byExtension.get(extension) ?? []), file]);
}
for (const file of [...byExtension.values()].flat()) {
  add('GET', `/_static/${file}`);
  add('GET', `/_static/${file}?v=2`);
}
for (const target of ['/_static/', '/_static/missing.js', '/_static/%2e%2e/server.mjs', '/_static/..%2F..%2Fserver.mjs', '/_static/%E0%A4%A', '/_static/a%00b', '/_static/%2Fetc%2Fpasswd'])
  add('GET', target);

const mediaFiles = (folder) => readdirSync(join(root, 'capture', folder), { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile()).map((entry) => join(entry.parentPath, entry.name).slice(join(root, 'capture', folder).length + 1)).sort();
const media = mediaFiles('media');
const mediahost = mediaFiles('mediahost');
const image = media.find((file) => /\.(jpe?g|webp|png)$/.test(file));
const video = media.find((file) => file.endsWith('.mp4'));
const pdf = media.find((file) => file.endsWith('.pdf'));
for (const file of [image, video, pdf, ...media.slice(0, 3)].filter(Boolean)) {
  add('GET', `/media/${file}`);
  add('GET', `/media/${encodeURI(file)}`);
}
for (const range of ['bytes=0-99', 'bytes=100-', 'bytes=-50', 'bytes=999999999-', 'bytes=abc', 'bytes=0-1,5-9', 'bytes=50-10', 'items=0-1'])
  add('GET', `/media/${video}`, { range });
add('HEAD', `/media/${video}`, { range: 'bytes=0-9' });
const stem = image.replace(/;[^/]*$/, '');
for (const target of [`${stem};1x1.jpeg`, `${stem};`, `${stem.replace(/\/[^/]*$/, '/')}zz;10x10.png`, 'missing.png', 'missing.mp4', 'missing.MOV', '%2e%2e/meta.json', '..%2Fmeta.json', '%E0%A4%A', 'a/b/'])
  add('GET', `/media/${target}`);
for (const file of [...mediahost.slice(0, 3), media[0], 'missing.webp', `${mediahost[0].replace(/;[^/]*$/, '')};5x5.webp`, '%2e%2e/meta.json'])
  add('GET', `/_mediahost/${file}`);
for (const file of ['x/y.png', 'missing.mp4', '%2e%2e/meta.json']) add('GET', `/_carbon/${file}`);

for (const dest of ['iframe', 'frame', 'image', 'document']) add('GET', `/media/${pdf ?? image}?page=1`, { 'sec-fetch-dest': dest });
add('GET', `/media/${image}`, { 'sec-fetch-dest': 'iframe', host: '127.0.0.1' });
add('POST', `/media/${image}`, { 'sec-fetch-dest': 'iframe' }, '{}');

for (const [target, agent] of [['/everything', 'desktop'], ['/everything', 'iphone'], ['/everything', 'android-phone'], ['/spaces/nope', 'desktop'], ['/no/such/page?q=1', 'iphone']])
  add('GET', target, { 'sec-fetch-dest': 'document', 'user-agent': AGENTS[agent] });
add('POST', '/everything', { 'sec-fetch-dest': 'document' }, '{}');

add('GET', '/everything', { 'cf-connecting-ip': '203.0.113.9' });
add('GET', '/hub', { 'cf-connecting-ip': '203.0.113.9' });
add('POST', '/__replay/reload');
add('GET', '/__replay/reload');
add('GET', '/x', { host: '[::1' });
add('GET', '//double//slash');
add('GET', '/%');
add('GET', '/cards?x=%ZZ');

console.log(JSON.stringify(requests));
