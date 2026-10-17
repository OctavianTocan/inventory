import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathId } from '../modules/ids.js';

const manifest = JSON.parse(readFileSync(new URL('../manifest.json', import.meta.url), 'utf8'));
assert.ok(!manifest.permissions.includes('declarativeNetRequest'), 'no request-rewriting permission');
assert.deepEqual(manifest.host_permissions, ['https://mymind.octaviantocan.com/*'], 'only the default host is granted up front');
assert.ok(manifest.optional_host_permissions.includes('http://localhost/*'), 'loopback is optional');
assert.ok(!manifest.host_permissions.some((pattern) => /localhost|127\.0\.0\.1/.test(pattern)));
const exposed = manifest.web_accessible_resources.flatMap((entry) => entry.resources);
assert.ok(!exposed.some((path) => path.includes('*')), 'no wildcard in the web-accessible resources');
assert.ok(!exposed.some((path) => /\.(js|mjs)$/.test(path)), 'no script is web-accessible');

const ID = '00000002UnE0O4zunrVB9T';
assert.equal(pathId(ID), ID);
for (const bad of [`../objects/${ID}`, `${ID}/../x`, `${ID}?x=1`, `${ID}#x`, `${ID}%2e%2e`, 'a/b', '', 'short', 42, null, undefined, {}, ['a'.repeat(22)], 'x'.repeat(65)]) {
  assert.throws(() => pathId(bad), /invalid id/, `refuses ${JSON.stringify(bad)}`);
}

const calls = { fetch: [], cookies: [] };
let hosts = ['http://localhost:4331'];
globalThis.chrome = {
  storage: { local: { get: async () => ({ host: hosts.length > 1 ? hosts.shift() : hosts[0] }) } },
  cookies: { get: async ({ url }) => (calls.cookies.push(url), { value: `token-of-${url}` }) },
};
globalThis.fetch = async (url, init) => (calls.fetch.push({ url: String(url), init }), new Response('{}', { status: 200 }));
const { API } = await import('../modules/api.js');
const { TagController } = await import('../modules/tags.js');
const api = new API('1.0.0');

await api.getJSON('/objects/x');
assert.equal(calls.fetch[0].url, 'http://localhost:4331/objects/x');
assert.equal(calls.fetch[0].init.credentials, 'omit', 'no ambient cookies');
assert.equal(calls.fetch[0].init.headers.Authorization, 'Bearer token-of-http://localhost:4331');

calls.fetch.length = 0;
for (const path of ['//evil.example/x', 'http://evil.example/x', '\\\\evil.example\\x']) {
  await assert.rejects(api.getJSON(path), /another origin/, `refuses ${path}`);
}
assert.equal(calls.fetch.length, 0, 'the session never leaves the configured origin');

calls.cookies.length = 0;
calls.fetch.length = 0;
hosts = ['http://localhost:4331', 'http://localhost:4332'];
await api.getJSON('/a');
assert.equal(calls.cookies[0], calls.fetch[0].url.replace('/a', ''), 'the cookie lookup and the request name the same host');
assert.equal(calls.fetch[0].init.headers.Authorization, `Bearer token-of-${calls.cookies[0]}`);

const scopeA = await api.scope();
assert.match(scopeA, /^http:\/\/localhost:\d+\|[0-9a-f]{64}$/);
assert.ok(!scopeA.includes('token-of'), 'the scope never contains the raw session');

const stored = {};
chrome.storage.local.get = async (key) => ({ [key]: stored[key] });
chrome.storage.local.set = async (value) => Object.assign(stored, value);
chrome.storage.local.remove = async (key) => delete stored[key];
let scope = 'http://a|1';
let tagsLoaded = 0;
const fakeApi = {
  scope: async () => scope,
  getJSON: async (path) => (path.startsWith('/tags/recent') ? [] : (tagsLoaded++, [{ name: `tag-${scope}`, count: 1 }])),
};
const controller = new TagController(fakeApi);
await controller.ensureInitialized();
assert.deepEqual(controller.query('tag').map((tag) => tag.name), ['tag-http://a|1']);
scope = 'http://a|2';
await controller.ensureInitialized();
assert.deepEqual(controller.query('tag').map((tag) => tag.name), ['tag-http://a|2'], 'another account never sees the first account\'s tags');
assert.equal(stored.tagCache.scope, 'http://a|2');
await controller.clear();
assert.equal(stored.tagCache, undefined);
console.log('unit tests passed');
