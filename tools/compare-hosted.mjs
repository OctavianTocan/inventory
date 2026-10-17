// Ask the recording replica and a hosted server (HOSTED=1, or the Worker) the same requests and report every
// difference in status, headers or body: each recorded key with its device and run, every captured detail view,
// mirrored file and media file, ranged video reads, and the fallbacks.
//   node tools/compare-hosted.mjs <replica url> <hosted url> [--out report.json] [--only <substring>]
// Run both servers on the same library, both with STORE_READONLY=1 and OFFLINE=1.
import { createHash } from 'node:crypto';
import { readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { localize } from '../lib/site.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { values, positionals: [replica, hosted] } = parseArgs({ allowPositionals: true, options: {
  out: { type: 'string' }, only: { type: 'string' }, concurrency: { type: 'string', default: '8' },
} });
if (!replica || !hosted) {
  console.error('Usage: node tools/compare-hosted.mjs <replica url> <hosted url> [--out report.json] [--only <substring>]');
  process.exit(2);
}
const AGENTS = {
  iphone: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
  ipad: 'Mozilla/5.0 (iPad; CPU OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
  'android-phone': 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36',
  'android-tablet': 'Mozilla/5.0 (Linux; Android 14; Pixel Tablet) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
};
const LOCAL_ORIGINS = ['https://access.mymind.com/', 'https://static.accelerator.net/', 'https://media.mymind.host/', 'https://carbon-media.accelerator.net/'];
// Headers that differ between any two responses, plus content-length and content-encoding (Cloudflare compresses text
// on the way out), which the hash of the decoded body covers.
const IGNORED = new Set(['date', 'connection', 'keep-alive', 'transfer-encoding', 'content-length', 'content-encoding']);
const encode = (relative) => relative.split(path.sep).map(encodeURIComponent).join('/');

async function files(folder) {
  const entries = await readdir(folder, { recursive: true, withFileTypes: true }).catch(() => []);
  return entries.filter((entry) => entry.isFile()).map((entry) => path.relative(folder, path.join(entry.parentPath, entry.name)));
}

// Every request to compare, named by what it covers.
async function requests() {
  const list = [];
  const index = JSON.parse(await readFile(path.join(ROOT, 'replay/index.json'), 'utf8'));
  for (const key of Object.keys(index)) {
    const match = /^(?:run:([^|]+)\|)?(?:(iphone|ipad|android-phone|android-tablet)\|)?(\w+) (https:\/\/[^/]+\/)(.*)$/.exec(key);
    if (!match || !LOCAL_ORIGINS.includes(match[4])) continue;
    const [, run, variant, kind] = match;
    const headers = {};
    if (run) headers['x-replay-run'] = run;
    if (variant) headers['user-agent'] = AGENTS[variant];
    if (kind === 'DOC') headers['sec-fetch-dest'] = 'document';
    list.push({ name: `replay ${key}`, method: kind === 'DOC' ? 'GET' : kind, path: localize(match[4] + match[5]), headers });
  }
  const responses = JSON.parse(await readFile(path.join(ROOT, 'capture/responses.json'), 'utf8'));
  for (const key of Object.keys(responses)) list.push({ name: `response ${key}`, method: key.split(' ')[0], path: key.slice(key.indexOf(' ') + 1), headers: {} });
  for (const id of Object.keys(JSON.parse(await readFile(path.join(ROOT, 'capture/details.json'), 'utf8')))) {
    list.push({ name: `detail ${id}`, method: 'GET', path: `/cards/${encodeURIComponent(id)}/detail`, headers: {} });
  }
  for (const file of await files(path.join(ROOT, 'mirror/static'))) list.push({ name: `static ${file}`, method: 'GET', path: `/_static/${encode(file)}`, headers: {} });
  const videos = [];
  for (const [folder, prefix] of [['media', '/media/'], ['mediahost', '/_mediahost/'], ['carbon', '/_carbon/']]) {
    for (const file of await files(path.join(ROOT, 'capture', folder))) {
      list.push({ name: `${folder} ${file}`, method: 'GET', path: prefix + encode(file), headers: {} });
      if (file.endsWith('.mp4') && videos.length < 6) videos.push(prefix + encode(file));
    }
  }
  for (const video of videos) {
    for (const range of ['bytes=0-1023', 'bytes=1000-', 'bytes=-500']) list.push({ name: `range ${range} ${video}`, method: 'GET', path: video, headers: { range } });
  }
  const sized = (await files(path.join(ROOT, 'capture/media'))).find((file) => /;\d+x\d+/.test(file));
  if (sized) list.push({ name: 'sibling rendition', method: 'GET', path: `/media/${encode(sized.replace(/;\d+x\d+/, ';7x7'))}`, headers: {} });
  list.push(
    { name: 'missing image', method: 'GET', path: '/media/nothing/here.webp', headers: {} },
    { name: 'missing video', method: 'GET', path: '/media/nothing/here.mp4', headers: {} },
    { name: 'missing static', method: 'GET', path: '/_static/nothing/here.js', headers: {} },
    { name: 'unknown write', method: 'POST', path: '/nothing/here', headers: {} },
    { name: 'unknown json', method: 'GET', path: '/nothing/here', headers: { accept: 'application/json' } },
    { name: 'unknown page part', method: 'GET', path: '/nothing/here', headers: {} },
    { name: 'unknown document', method: 'GET', path: '/nothing/here', headers: { 'sec-fetch-dest': 'document' } },
    { name: 'phone document', method: 'GET', path: '/nothing/here', headers: { 'sec-fetch-dest': 'document', 'user-agent': AGENTS.iphone } },
    { name: 'traversal', method: 'GET', path: '/_static/..%2F..%2Fpackage.json', headers: {} },
  );
  return values.only ? list.filter((request) => request.name.includes(values.only)) : list;
}

// Status, kept headers and a body hash of one response.
async function ask(base, request) {
  const response = await fetch(base + request.path, { method: request.method, headers: request.headers, redirect: 'manual',
    body: ['GET', 'HEAD'].includes(request.method) ? undefined : '' });
  const hash = createHash('sha256');
  let size = 0;
  for await (const chunk of response.body ?? []) { hash.update(chunk); size += chunk.length; }
  const headers = Object.fromEntries([...response.headers].filter(([name]) => !IGNORED.has(name)));
  return { status: response.status, headers, size, sha256: hash.digest('hex') };
}

function differences(a, b) {
  const found = [];
  if (a.status !== b.status) found.push(`status ${a.status} != ${b.status}`);
  for (const name of new Set([...Object.keys(a.headers), ...Object.keys(b.headers)])) {
    if (a.headers[name] !== b.headers[name]) found.push(`${name}: ${a.headers[name] ?? '-'} != ${b.headers[name] ?? '-'}`);
  }
  if (a.sha256 !== b.sha256) found.push(`body ${a.size} != ${b.size} bytes`);
  return found;
}

const list = await requests();
const report = [];
let next = 0;
await Promise.all(Array.from({ length: Number(values.concurrency) }, async () => {
  while (next < list.length) {
    const request = list[next++];
    try {
      const [a, b] = [await ask(replica, request), await ask(hosted, request)];
      const found = differences(a, b);
      if (found.length) report.push({ ...request, differences: found, replica: a, hosted: b });
    } catch (error) {
      report.push({ ...request, differences: [`error ${error.message}`] });
    }
  }
}));
const kinds = {};
for (const entry of report) for (const difference of entry.differences) {
  const kind = `${entry.name.split(' ')[0]} | ${difference.replace(/\d+/g, 'N').slice(0, 120)}`;
  kinds[kind] = (kinds[kind] ?? 0) + 1;
}
console.log(`${list.length} requests, ${report.length} differ`);
for (const [kind, count] of Object.entries(kinds).sort((a, b) => b[1] - a[1])) console.log(`${String(count).padStart(6)}  ${kind}`);
if (values.out) await writeFile(values.out, JSON.stringify(report, null, 2));
process.exitCode = report.length ? 1 : 0;
