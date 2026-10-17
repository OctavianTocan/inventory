// lib/capture-link.mjs driven by generated scenarios with table-driven fakes, for the native port's differential test.
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const OEMBED = [
  '{"html":"<blockquote class=\\"twitter-tweet\\"><p>hi &amp; bye <a href=\\"https://t.co/x\\">l</a></p></blockquote>","author_name":"Ann","title":"Embed T","thumbnail_url":"https://img.example/t.png","type":"video","author_avatar_url":"https://img.example/a.png"}',
  '{"title":"Only title","author_name":"By Bob","thumbnail_url":"/rel-thumb.png","author_image_url":"https://img.example/1.png"}',
  'not json at all',
  '{"html":"<p>plain</p>","type":"rich","avatar_url":"https://pbs.twimg.com/profile_images/1/a.jpg"}',
  'null',
  '{"title":7,"author_name":null,"html":"","thumbnail_url":"https://img.example/2.png"}',
];

const hash = (text, seed) => {
  let h = (2166136261 ^ seed) >>> 0;
  for (const byte of Buffer.from(text)) h = Math.imul(h ^ byte, 16777619) >>> 0;
  return h;
};

const blocked = () => Object.assign(new Error('Unsupported capture destination'), { code: 'CAPTURE_BLOCKED', status: 400 });

function answer(url, scenario) {
  if (url === scenario.card.source.url && scenario.page) {
    const page = scenario.page;
    return { bytes: Buffer.from(page.text), url: page.final ?? url, mime: page.mime, status: 200, headers: {} };
  }
  const h = hash(url, scenario.seed);
  if (url.includes('oembed')) {
    const kind = h % 8;
    if (kind === 6) throw blocked();
    if (kind === 7) throw new Error('Page unavailable');
    return { bytes: Buffer.from(OEMBED[kind % OEMBED.length]), url, mime: 'application/json', status: 200, headers: {} };
  }
  switch (h % 8) {
    case 0: case 1: return { bytes: Buffer.from(`IMG${h % 97}`), url, mime: 'image/png', status: 200, headers: {} };
    case 2: return { bytes: Buffer.from('JPG'), url, mime: 'image/jpeg', status: 200, headers: {} };
    case 3: return { bytes: Buffer.from('XBAD'), url, mime: 'image/png', status: 200, headers: {} };
    case 4: return { bytes: Buffer.from('<html></html>'), url, mime: 'text/html', status: 200, headers: {} };
    case 5: throw new Error('Page unavailable');
    case 6: throw blocked();
    default: return { bytes: Buffer.from('WEBP'), url, mime: 'image/webp', status: 200, headers: {} };
  }
}

export async function run(scenarios) {
  const { captureLink } = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib/capture-link.mjs')).href);
  const out = [];
  for (const scenario of scenarios) {
    const events = [];
    const user = { accountUid: 'acc', idPrefix: 'p' };
    const resource = async (url, options) => {
      events.push({ op: 'resource', url, offline: options.offline, limit: options.limit });
      return answer(url, scenario);
    };
    const screenshot = async (url, options) => {
      events.push({ op: 'screenshot', url, offline: options.offline });
      if (scenario.screenshot === 'blocked') throw blocked();
      if (scenario.screenshot === 'fail') throw new Error('browser failed');
      return Buffer.from(scenario.screenshot === 'bad' ? 'XSHOT' : 'SHOT');
    };
    const media = {
      async prepare(bytes, mime, rawName, who, id) {
        const name = String(rawName ?? '');
        events.push({ op: 'prepare', length: bytes.length, mime, name, user: who, id });
        if (bytes.length > 0 && bytes[0] === 0x58) throw Object.assign(new Error('Media preview could not be generated'), { status: 422 });
        const key = `${who.accountUid}/${id}/${bytes.length}`;
        if (mime === 'application/pdf') return { data: { type: 'Document', title: name, blob: { path: `${key}.pdf` }, format: 'pdf' }, extras: {} };
        if (mime.startsWith('image/')) return { data: { type: 'Image', title: name, blob: { path: key } }, extras: { cover: { src: `/media/${key};800x1000-t.webp`, width: 800, height: 1000 } } };
        return { data: { type: 'File', title: name }, extras: {} };
      },
    };
    const update = patch => { events.push({ op: 'update', patch: structuredClone(patch) }); };
    let result = 'ok';
    try { await captureLink(structuredClone(scenario.card), { id: 'card1', user, media, update, offline: scenario.offline, screenshot, resource }); }
    catch (error) { result = error instanceof TypeError || error instanceof RangeError ? 'invalid' : 'other'; }
    out.push({ result, events });
  }
  return out;
}
