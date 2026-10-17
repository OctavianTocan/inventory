// The fixture the download-server differential test serves: a sign-in database with sessions in every state, a
// downloads directory with its manifest, and one without. Usage: node download-fixture.mjs <empty directory>.
// Package names stay ASCII: for a name JSON.parse read from UTF-8, Node's writeHead refuses the content-disposition
// and the JS server exits.
import { createHash } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [dir] = process.argv.slice(2);
const { openAuthStore } = await import(join(process.env.MYMIND_ROOT, 'lib/auth-store.mjs'));
const auth = openAuthStore(join(dir, 'auth.db'));
const hour = 3_600_000;
const sessions = {
  owner: ['owner-session-id', 'owner@example.com', hour],
  escaped: ['escaped-session-id', '<b>"x"&@example.com', hour],
  expired: ['expired-session-id', 'owner@example.com', -hour],
  blank: ['blank-session-id', '', hour],
  // The id as Node reads the UTF-8 bytes of `café-id` in a header: Latin-1.
  latin: [Buffer.from('café-id', 'utf8').toString('latin1'), 'latin@example.com', hour],
};
const now = Date.now();
for (const [id, email, ttl] of Object.values(sessions)) {
  auth.prepare('INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)')
    .run(createHash('sha256').update(id).digest('hex'), `sub-${email}`, email, now, now + ttl, now);
}
auth.close();

const downloads = join(dir, 'downloads');
mkdirSync(join(downloads, 'sub'), { recursive: true });
mkdirSync(join(dir, 'empty'));
let seed = 7;
const bytes = (count) => Buffer.from(Array.from({ length: count }, () => (seed = (seed * 1103515245 + 12345) % 2 ** 31) & 0xff));
const files = { 'mymind-linux-x86_64.tar.gz': 300_000, 'my app.tar.gz': 1_000, 'a+b&c=d.tar.gz': 10, 'sub/inner.tar.gz': 20, 'notlisted.tar.gz': 5 };
for (const [name, size] of Object.entries(files)) writeFileSync(join(downloads, name), bytes(size));
const entry = (name, extra = {}) => ({ name, label: `Label <${name}>`, bytes: 52_428_800, built: '2026-10-17T01:02:03Z', sha256: 'ab&cd', ...extra });
writeFileSync(join(downloads, 'manifest.json'), JSON.stringify({
  library: 'https://library.example/<x>',
  files: [entry('mymind-linux-x86_64.tar.gz'), entry('my app.tar.gz'), entry('a+b&c=d.tar.gz'), entry('sub/inner.tar.gz'),
    entry('sub/../my app.tar.gz'), entry('missing.tar.gz'), entry(5), { label: 'no name', bytes: 1, built: '', sha256: '' }],
}));
console.log(JSON.stringify(Object.fromEntries(Object.entries(sessions).map(([key, [id]]) => [key, id]))));
