import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { runAudit } from '../scripts/live-audit.mjs';

const id = (n) => n.toString(16).padStart(32, '0');
const expected = {
  workerName: 'mymind-alchemy-staging-worker', workerId: id(1),
  probeName: 'mymind-alchemy-staging-probe', probeId: id(2),
  bucketName: 'mymind-alchemy-staging-fixtures',
  durableObjectNamespaces: { Auth: id(3), Library: id(4), Media: id(5) },
  containerApplicationId: '11111111-2222-3333-4444-555555555555',
};
const account = id(99);
const secret = 'synthetic-secret-that-must-never-be-output';
const absent = (code, message = 'not found') => new Response(JSON.stringify({ success: false, errors: [{ code, message }] }), { status: 404 });
const ok = (result, result_info) => Response.json({ success: true, errors: [], result, ...(result_info ? { result_info } : {}) });

// Canonical namespace wire data contains only the fixture's synthetic identities.
const namespaceBody = (state, errors = []) => ({
  success: true, errors,
  result: state === 'present' ? Object.entries(expected.durableObjectNamespaces)
    .map(([name, value]) => ({ id: value, class: name, script: expected.workerName })) : [],
  result_info: { page: 1, per_page: 100, count: state === 'present' ? 3 : 0, total_count: state === 'present' ? 3 : 0, total_pages: 1 },
});

// Exercise the real audit client with Cloudflare-shaped wire responses, never the network.
function fixture({ state = 'present', override = () => undefined, inventory = [], transformBindings = (bindings) => bindings } = {}) {
  const calls = [], commands = [];
  let inventories = 0;
  const fetcher = async (input, options) => {
    const url = new URL(input);
    calls.push(url);
    assert.equal(url.origin, 'https://api.cloudflare.com');
    assert.equal(options.method, 'GET');
    assert.equal(options.redirect, 'error');
    assert.ok(options.signal instanceof AbortSignal);
    assert.equal(options.headers.authorization, `Bearer ${secret}`);
    const custom = override(url, calls);
    if (custom !== undefined) return custom;
    const tail = url.pathname.split(`/accounts/${account}/`)[1];
    const live = state === 'present';
    if (tail === 'workers/scripts') return ok(live ? [
      { id: expected.workerName, tag: expected.workerId }, { id: expected.probeName, tag: expected.probeId },
    ] : []);
    if (tail === 'workers/durable_objects/namespaces') return Response.json(namespaceBody(state));
    if (tail === 'containers/applications') return ok(live ? [{ id: expected.containerApplicationId, name: 'mymind-alchemy-staging-media', durable_objects: { namespace_id: id(5) } }] : []);
    if (tail.startsWith('containers/applications/')) return live ? ok({ id: expected.containerApplicationId, name: 'mymind-alchemy-staging-media', durable_objects: { namespace_id: id(5) }, configuration: { secrets: [secret] } }) : absent(1609, 'APPLICATION_NOT_FOUND');
    if (tail.startsWith('r2/buckets/')) return live ? ok({ name: expected.bucketName }) : absent(10006);
    if (tail.endsWith('/schedules')) return live ? ok({ schedules: [] }) : absent(10007);
    if (tail.endsWith('/settings')) {
      if (!live) return absent(10007);
      const bindings = tail.includes(expected.probeName) ? [
        { name: 'TARGET', type: 'service', service: expected.workerName },
        { name: 'RESULTS', type: 'r2_bucket', bucket_name: expected.bucketName },
      ] : [
        ...Object.entries({ AUTH_OBJECT: 'Auth', LIBRARY: 'Library', MEDIA: 'Media' }).map(([name, class_name]) => ({ name, type: 'durable_object_namespace', class_name, namespace_id: expected.durableObjectNamespaces[class_name] })),
        { name: 'BUCKET', type: 'r2_bucket', bucket_name: expected.bucketName },
      ];
      return ok({ bindings: transformBindings([...bindings, { name: 'ADMIN_TOKEN', type: 'secret_text', text: secret }]), arbitrary: secret });
    }
    throw new Error('Unexpected synthetic request');
  };
  const runWrangler = (args, options) => {
    commands.push(args);
    assert.ok(options.timeout > 0 && options.timeout <= 60_000);
    if (args.includes('list')) return { status: 0, stdout: JSON.stringify(inventories++ === 0 ? inventory : []), stderr: '' };
    return { status: 0, stdout: '', stderr: 'Image deleted, but the garbage-collection request failed: synthetic' };
  };
  return { calls, commands, run: (changes = {}) => runAudit({ operation: 'audit', expectedState: state, expectedJson: JSON.stringify(expected), account, token: secret, fetcher, runWrangler, ...changes }) };
}

test('live audit verifies identities and bindings without emitting settings or secrets', async () => {
  const f = fixture();
  const result = await f.run();
  assert.equal(result.state, 'present');
  assert.deepEqual(result.identities, expected);
  assert.deepEqual(result.schedules, { worker: [], probe: [] });
  assert.ok(!JSON.stringify(result).includes(secret));
  assert.ok(!JSON.stringify(result).includes('ADMIN_TOKEN'));
  assert.equal(f.commands.length, 0);
});

test('successful namespace reads accept the observed null errors field for present and absent audits', async () => {
  for (const state of ['present', 'absent']) {
    const f = fixture({ state, override: (url) => url.pathname.endsWith('/namespaces')
      ? Response.json(namespaceBody(state, null)) : undefined });
    const result = await f.run();
    assert.equal(result.state, state);
    assert.deepEqual(result.identities, expected);
    assert.equal(f.commands.length, 0);
  }
});

test('namespace null normalization leaves malformed envelopes and pagination inconclusive', async () => {
  for (const changes of [
    { success: false }, { result: null }, { result: 'not-a-list' },
    { result_info: null }, { result_info: { page: 1, per_page: 100, count: 0 } },
    ...['not-empty', {}, false, 971, [{ code: 971, message: secret }]].map((errors) => ({ errors })),
  ]) {
    const f = fixture({ state: 'absent', override: (url) => url.pathname.endsWith('/namespaces')
      ? Response.json({ ...namespaceBody('absent', null), ...changes }) : undefined });
    await assert.rejects(f.run({ operation: 'cleanup-images' }), (error) => !error.message.includes(secret));
    assert.equal(f.commands.length, 0);
  }
});

test('namespace null normalization does not accept failed HTTP reads', async () => {
  for (const status of [403, 404, 500]) {
    const f = fixture({ state: 'absent', override: (url) => url.pathname.endsWith('/namespaces')
      ? Response.json(namespaceBody('absent', null), { status }) : undefined });
    await assert.rejects(f.run({ operation: 'cleanup-images' }));
    assert.equal(f.commands.length, 0);
  }
});

test('other API endpoints still reject null errors', async () => {
  for (const resource of ['workers/scripts', 'containers/applications',
    `workers/scripts/${expected.workerName}/settings`, `r2/buckets/${expected.bucketName}`]) {
    const f = fixture({ override: (url) => url.pathname.endsWith(`/${resource}`)
      ? Response.json({ success: true, result: [], errors: null }) : undefined });
    await assert.rejects(f.run(), /Pilot audit API read failed/);
    assert.equal(f.commands.length, 0);
  }
});

test('namespace null normalization retains immutable ownership checks', async () => {
  const body = namespaceBody('present', null);
  body.result[0].script = 'foreign-worker';
  const f = fixture({ override: (url) => url.pathname.endsWith('/namespaces') ? Response.json(body) : undefined });
  await assert.rejects(f.run(), /Live namespace ownership does not match/);
  assert.equal(f.commands.length, 0);
});

test('HTTP-200 logical API failures identify a fixed endpoint without echoing response data', async () => {
  for (const [resource, label] of [
    ['workers/scripts', 'worker-list'],
    ['workers/durable_objects/namespaces', 'namespace-list'],
    ['containers/applications', 'application-list'],
    [`workers/scripts/${expected.workerName}/settings`, 'worker-settings'],
    [`workers/scripts/${expected.workerName}/schedules`, 'worker-schedules'],
    [`workers/scripts/${expected.probeName}/settings`, 'probe-settings'],
    [`workers/scripts/${expected.probeName}/schedules`, 'probe-schedules'],
    [`containers/applications/${expected.containerApplicationId}`, 'application-detail'],
    [`r2/buckets/${expected.bucketName}`, 'bucket-detail'],
  ]) {
    const f = fixture({ override: (url) => url.pathname.endsWith(`/${resource}`)
      ? Response.json({ success: false, result: null, errors: [{ code: 971, message: secret }], unexpected: secret }) : undefined });
    await assert.rejects(f.run(), (error) => {
      assert.ok(error.message.includes(`endpoint=${label}; success=false; result=present; errors=nonempty-array; errors-kind=array; codes=971;`));
      for (const value of [secret, account, expected.workerName, expected.containerApplicationId]) assert.ok(!error.message.includes(value));
      return true;
    });
    assert.equal(f.commands.length, 0);
  }
});

test('rejected audit envelopes report finite predicate states and bounded numeric codes', async () => {
  for (const [body, metadata] of [
    [{ result: [], errors: [] }, 'success=invalid; result=present; errors=empty-array; errors-kind=array; codes=none;'],
    [{ success: true, errors: [] }, 'success=true; result=missing; errors=empty-array; errors-kind=array; codes=none;'],
    [{ success: true, result: [], errors: null }, 'errors=invalid; errors-kind=null; codes=none;'],
    [{ success: true, result: [], errors: { message: secret } }, 'errors=invalid; errors-kind=object; codes=none;'],
    [{ success: true, result: [], errors: secret }, 'errors=invalid; errors-kind=string; codes=none;'],
    [{ success: true, result: [], errors: 971 }, 'errors=invalid; errors-kind=number; codes=none;'],
    [{ success: true, result: [], errors: false }, 'errors=invalid; errors-kind=boolean; codes=none;'],
    [{ success: false, result: null, errors: [{ code: secret, message: secret }] }, 'success=false; result=present; errors=nonempty-array; errors-kind=array; codes=invalid;'],
    [{ success: false, result: null, errors: Array.from({ length: 20 }, () => ({ code: 971, message: secret })) }, 'codes=971,971,971,971,971;'],
  ]) {
    const f = fixture({ state: 'absent', override: () => Response.json(body) });
    await assert.rejects(f.run({ operation: 'cleanup-images' }), (error) => {
      assert.ok(error.message.includes(metadata));
      assert.ok(!error.message.includes(secret));
      assert.ok(error.message.length < 250);
      return true;
    });
    assert.equal(f.commands.length, 0);
  }
});

test('absence requires definitive reads and complete namespace pagination', async () => {
  const f = fixture({ state: 'absent', override: (url) => {
    if (!url.pathname.endsWith('/namespaces')) return;
    const page = Number(url.searchParams.get('page'));
    return ok(page === 1 ? [{ id: id(80), script: 'unrelated', class: 'Other' }] : [],
      { page, per_page: 100, count: page === 1 ? 1 : 0, total_count: 1, total_pages: 2 });
  } });
  assert.equal((await f.run()).state, 'absent');
  assert.equal(f.calls.filter((url) => url.pathname.endsWith('/namespaces')).length, 2);
});

for (const [label, override] of [
  ['a saved namespace still exists under another Worker', (u) => u.pathname.endsWith('/namespaces') ? ok([{ id: id(3), script: 'production', class: 'Auth' }], { page: 1, per_page: 100, count: 1, total_count: 1, total_pages: 1 }) : undefined],
  ['a replacement pilot application exists', (u) => u.pathname.endsWith('/containers/applications') ? ok([{ id: id(70), name: 'mymind-alchemy-staging-media' }]) : undefined],
  ['permissions are denied', () => new Response(JSON.stringify({ success: false, errors: [{ code: 10000, message: secret }] }), { status: 403 })],
  ['an unknown 404 is returned', () => absent(7003)],
  ['pagination metadata is missing', (u) => u.pathname.endsWith('/namespaces') ? ok([]) : undefined],
  ['pagination repeats a page', (u) => u.pathname.endsWith('/namespaces') ? ok([], { page: 1, per_page: 100, count: 0, total_count: 0, total_pages: 2 }) : undefined],
  ['cursor pagination is unsupported', (u) => u.pathname.endsWith('/namespaces') ? ok([], { page: 1, per_page: 100, count: 0, total_count: 0, total_pages: 1, cursors: { after: 'next' } }) : undefined],
]) {
  test(`cleanup refuses ${label} before invoking registry tooling`, async () => {
    const f = fixture({ state: 'absent', override });
    await assert.rejects(f.run({ operation: 'cleanup-images' }));
    assert.equal(f.commands.length, 0);
  });
}

test('a foreign expected resource is rejected before any HTTP request', async () => {
  const f = fixture();
  await assert.rejects(f.run({ expectedJson: JSON.stringify({ ...expected, workerName: 'production' }) }));
  assert.equal(f.calls.length, 0);
  await assert.rejects(f.run({ expectedJson: JSON.stringify({ ...expected, extra: secret }) }));
  assert.equal(f.calls.length, 0);
});

test('a wrong live binding or immutable Worker ID fails the present audit', async () => {
  const f = fixture({ override: (u) => u.pathname.endsWith('/workers/scripts') ? ok([{ id: expected.workerName, tag: id(77) }, { id: expected.probeName, tag: id(2) }]) : undefined });
  await assert.rejects(f.run());
  const wrong = fixture({ transformBindings: (bindings) => bindings.map((binding) => binding.name === 'AUTH_OBJECT'
    ? { ...binding, namespace_id: id(88) } : binding) });
  await assert.rejects(wrong.run());
});

test('present audit rejects foreign service environments and entrypoints', async () => {
  for (const selector of [{ environment: 'foreign' }, { entrypoint: 'ForeignEntrypoint' }]) {
    const f = fixture({ transformBindings: (bindings) => bindings.map((binding) => binding.name === 'TARGET'
      ? { ...binding, ...selector } : binding) });
    await assert.rejects(f.run());
  }
});

test('present audit accepts absent or default entrypoints without an environment selector', async () => {
  for (const selector of [{ environment: null, entrypoint: null }, { entrypoint: 'default' }]) {
    const f = fixture({ transformBindings: (bindings) => bindings.map((binding) => binding.name === 'TARGET'
      ? { ...binding, ...selector } : binding) });
    assert.equal((await f.run()).state, 'present');
  }
});

test('cleanup rejects application rows with an inconclusive name before registry access', async () => {
  for (const name of [undefined, null, 0, '', ' ']) {
    const f = fixture({ state: 'absent', override: (url) => url.pathname.endsWith('/containers/applications')
      ? ok([{ id: id(70), name, configuration: { image: 'docker.io/library/alpine:latest' } }]) : undefined });
    await assert.rejects(f.run({ operation: 'cleanup-images' }));
    assert.equal(f.commands.length, 0);
  }
});

test('cleanup blocks tagged and digest image consumers under unrelated application names', async () => {
  for (const reference of [':abc123', `@sha256:${'a'.repeat(64)}`]) {
    const f = fixture({ state: 'absent', inventory: [{ name: 'mymind-alchemy-staging-media', tags: ['abc123'] }],
      override: (url) => url.pathname.endsWith('/containers/applications') ? ok([{
        id: id(70), name: 'unrelated-live-app',
        configuration: { image: `registry.cloudflare.com/${account}/mymind-alchemy-staging-media${reference}` },
      }]) : undefined });
    await assert.rejects(f.run({ operation: 'cleanup-images' }));
    assert.equal(f.commands.length, 0);
  }
});

test('cleanup rejects unknown live application image usage', async () => {
  const f = fixture({ state: 'absent', override: (url) => url.pathname.endsWith('/containers/applications')
    ? ok([{ id: id(70), name: 'unrelated-live-app', configuration: {} }]) : undefined });
  await assert.rejects(f.run({ operation: 'cleanup-images' }));
  assert.equal(f.commands.length, 0);
});

test('other repositories and accounts do not prevent fixed pilot image cleanup', async () => {
  for (const image of [`registry.cloudflare.com/${account}/other-repository:abc123`,
    `registry.cloudflare.com/${id(98)}/mymind-alchemy-staging-media:abc123`]) {
    const f = fixture({ state: 'absent', inventory: [{ name: 'mymind-alchemy-staging-media', tags: ['buildcache'] }],
      override: (url) => url.pathname.endsWith('/containers/applications')
        ? ok([{ id: id(70), name: 'unrelated-live-app', configuration: { image } }]) : undefined });
    assert.deepEqual((await f.run({ operation: 'cleanup-images' })).deletedTags, ['buildcache']);
  }
});

test('network and response-body failures remain inconclusive, without leaking error text', async () => {
  const f = fixture({ state: 'absent', override: () => { throw new Error(secret); } });
  await assert.rejects(f.run({ operation: 'cleanup-images' }), (error) => !error.message.includes(secret));
  assert.equal(f.commands.length, 0);
  const body = fixture({ state: 'absent', override: () => new Response(new ReadableStream({ start(controller) { controller.error(new Error(secret)); } })) });
  await assert.rejects(body.run({ operation: 'cleanup-images' }), (error) => !error.message.includes(secret));
  assert.equal(body.commands.length, 0);
});

test('a late namespace page containing a pilot namespace blocks deletion', async () => {
  const f = fixture({ state: 'absent', override: (url) => {
    if (!url.pathname.endsWith('/namespaces')) return;
    const page = Number(url.searchParams.get('page'));
    return Response.json({ success: true, errors: null,
      result: [{ id: page === 1 ? id(80) : id(5), script: page === 1 ? 'unrelated' : expected.workerName, class: 'Media' }],
      result_info: { page, per_page: 1, count: 1, total_count: 2, total_pages: 2 } });
  } });
  await assert.rejects(f.run({ operation: 'cleanup-images' }), /Pilot resources or saved identities still exist/);
  assert.equal(f.calls.filter((url) => url.pathname.endsWith('/namespaces')).length, 2);
  assert.equal(f.commands.length, 0);
});

test('the audit budget stops further reads before registry tooling can run', async (t) => {
  t.mock.timers.enable({ apis: ['Date'], now: 1000 });
  const f = fixture({ state: 'absent', override: () => {
    t.mock.timers.tick(5 * 60_000);
    return ok([]);
  } });
  await assert.rejects(f.run({ operation: 'cleanup-images' }));
  assert.equal(f.calls.length, 1);
  assert.equal(f.commands.length, 0);
});

test('a live resource appearing during registry inventory blocks every deletion', async () => {
  let secondAudit = false;
  const f = fixture({ state: 'absent', override: (url) => secondAudit && url.pathname.endsWith('/workers/scripts')
    ? ok([{ id: expected.workerName, tag: id(1) }]) : undefined });
  const runWrangler = () => {
    secondAudit = true;
    return { status: 0, stdout: JSON.stringify([{ name: 'mymind-alchemy-staging-media', tags: ['buildcache'] }]), stderr: '' };
  };
  await assert.rejects(f.run({ operation: 'cleanup-images', runWrangler }));
  assert.equal(f.calls.filter((url) => url.pathname.endsWith('/workers/scripts')).length, 2);
});

test('cleanup deletes only inventoried pilot tags including buildcache and inventories again', async () => {
  const f = fixture({ state: 'absent', inventory: [{ name: 'mymind-alchemy-staging-media', tags: ['buildcache', 'abc123'] }] });
  const result = await f.run({ operation: 'cleanup-images' });
  assert.deepEqual(result.deletedTags, ['abc123', 'buildcache']);
  assert.equal(result.layerReclamationVerified, false);
  assert.equal(result.garbageCollectionWarnings, 2);
  assert.equal(f.commands.filter((a) => a.includes('list')).length, 2);
  for (const args of f.commands.filter((a) => a.includes('delete'))) {
    assert.match(args[3], /^mymind-alchemy-staging-media:(abc123|buildcache)$/);
    assert.ok(args.includes('--skip-confirmation'));
  }
});

test('registry output cannot widen cleanup to a foreign repository or invalid tag', async () => {
  for (const inventory of [[{ name: 'production', tags: ['latest'] }], [{ name: 'mymind-alchemy-staging-media', tags: ['../../foreign'] }]]) {
    const f = fixture({ state: 'absent', inventory });
    await assert.rejects(f.run({ operation: 'cleanup-images' }));
    assert.equal(f.commands.filter((a) => a.includes('delete')).length, 0);
  }
});

test('cleanup requires an absent expectation and rejects nonempty final inventory', async () => {
  const live = fixture();
  await assert.rejects(live.run({ operation: 'cleanup-images' }));
  assert.equal(live.calls.length, 0);
  const f = fixture({ state: 'absent' });
  await assert.rejects(f.run({ operation: 'cleanup-images', runWrangler: () => ({ status: 0, stdout: JSON.stringify([{ name: 'mymind-alchemy-staging-media', tags: ['remaining'] }]), stderr: '' }) }));
});

test('the actual CI entrypoint uses only the fixed pinned CLI and emits safe absence evidence', (t) => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'alchemy-audit-cli-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const project = path.join(root, 'infra/alchemy');
  mkdirSync(path.join(project, 'scripts'), { recursive: true });
  mkdirSync(path.join(project, '.alchemy/state'), { recursive: true });
  mkdirSync(path.join(root, 'rust/crates/worker'), { recursive: true });
  for (const file of ['live-audit.mjs', 'contract.mjs']) copyFileSync(new URL(`../scripts/${file}`, import.meta.url), path.join(project, 'scripts', file));
  copyFileSync(new URL('../pilot.json', import.meta.url), path.join(project, 'pilot.json'));
  copyFileSync(new URL('../../../rust/crates/worker/wrangler.jsonc', import.meta.url), path.join(root, 'rust/crates/worker/wrangler.jsonc'));
  const wrangler = path.join(project, 'node_modules/wrangler');
  mkdirSync(path.join(wrangler, 'wrangler-dist'), { recursive: true });
  writeFileSync(path.join(wrangler, 'package.json'), JSON.stringify({ version: '4.151.1' }));
  writeFileSync(path.join(wrangler, 'wrangler-dist/cli.js'), `
    const fs = require('node:fs');
    if (process.env.WRANGLER_WRITE_LOGS !== 'false') process.exit(2);
    fs.appendFileSync(${JSON.stringify(path.join(root, 'commands'))}, JSON.stringify(process.argv.slice(2))+'\\n');
    console.log('[]');
  `);
  const loader = path.join(root, 'fetch.mjs');
  writeFileSync(loader, `globalThis.fetch = async (input) => {
    const p = new URL(input).pathname;
    if (p.endsWith('/namespaces')) return Response.json({success:true, result:[], result_info:{page:1,per_page:100,count:0,total_count:0,total_pages:1}});
    if (p.endsWith('/workers/scripts') || p.endsWith('/containers/applications')) return Response.json({success:true,result:[]});
    const code = p.includes('/r2/') ? 10006 : p.includes('/containers/') ? 1609 : 10007;
    return new Response(JSON.stringify({success:false,errors:[{code,message:'APPLICATION_NOT_FOUND'}]}),{status:404});
  };`);
  const env = { PATH: process.env.PATH, GITHUB_ACTIONS: 'true', GITHUB_REF: 'refs/heads/main',
    MYMIND_ALCHEMY_INITIALIZE: 'false', MYMIND_ALCHEMY_EXPECTED_STATE: 'absent',
    MYMIND_ALCHEMY_EXPECTED_IDENTITIES: JSON.stringify(expected), CLOUDFLARE_ACCOUNT_ID: account, CLOUDFLARE_API_TOKEN: secret };
  const args = ['--import', pathToFileURL(loader).href, path.join(project, 'scripts/live-audit.mjs'), 'cleanup-images'];
  const result = spawnSync(process.execPath, args, { env, encoding: 'utf8', timeout: 5000 });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).state, 'absent');
  assert.ok(!result.stdout.includes(secret));
  const commands = readFileSync(path.join(root, 'commands'), 'utf8').trim().split('\n').map(JSON.parse);
  assert.equal(commands.length, 2);
  for (const command of commands) assert.deepEqual(command, ['containers', 'images', 'list', '--filter', '(^|/)mymind-alchemy-staging-media$', '--json', '--env-file', '/dev/null']);
  const blocked = spawnSync(process.execPath, args, { env: { ...env, GITHUB_REF: 'refs/heads/foreign' }, encoding: 'utf8', timeout: 5000 });
  assert.notEqual(blocked.status, 0);
  assert.equal(readFileSync(path.join(root, 'commands'), 'utf8').trim().split('\n').length, 2);
});
