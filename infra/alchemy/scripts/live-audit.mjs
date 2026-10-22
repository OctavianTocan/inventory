import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { PILOT_ROOT, readContract } from './contract.mjs';

const { pilot } = readContract();
const REPOSITORY = 'mymind-alchemy-staging-media';
const ID = /^(?:[a-f0-9]{32}|[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12})$/;
const TAG = /^[a-zA-Z0-9_][a-zA-Z0-9_.-]{0,127}$/;
const BINDINGS = { AUTH_OBJECT: 'Auth', LIBRARY: 'Library', MEDIA: 'Media' };
const fail = (message) => { throw new Error(message); };

// Reject extra fields and resource names before using any operator-supplied identity.
function expectedIdentities(text) {
  let value;
  try {
    if (typeof text !== 'string' || text.length > 4096) fail('Invalid identity input');
    value = JSON.parse(text);
  } catch { fail('Expected a safe deployment identity JSON object'); }
  const keys = ['workerName', 'workerId', 'probeName', 'probeId', 'bucketName', 'durableObjectNamespaces', 'containerApplicationId'];
  if (!value || Array.isArray(value) || Object.keys(value).sort().join() !== keys.sort().join()
      || value.workerName !== pilot.name || value.probeName !== pilot.probe_name
      || value.bucketName !== pilot.r2_buckets[0].bucket_name) fail('Identity input must name only the fixed pilot resources');
  const namespaces = value.durableObjectNamespaces;
  if (!namespaces || Array.isArray(namespaces) || Object.keys(namespaces).sort().join() !== 'Auth,Library,Media') fail('Expected exactly the three pilot namespace identities');
  const ids = [value.workerId, value.probeId, value.containerApplicationId, ...Object.values(namespaces)];
  if (ids.some((id) => typeof id !== 'string' || !ID.test(id)) || new Set(ids).size !== ids.length) fail('Expected distinct live resource IDs');
  return value;
}

// Parse bounded API bodies without ever copying remote error text into diagnostics.
async function responseJson(response) {
  const reader = response.body?.getReader();
  if (!reader) fail('Pilot audit received an empty response');
  const chunks = [];
  let length = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      length += value.length;
      if (length > 2 * 1024 * 1024) fail('Pilot audit response exceeded its size limit');
      chunks.push(value);
    }
    return JSON.parse(Buffer.concat(chunks).toString('utf8'));
  } finally { await reader.cancel().catch(() => {}); }
}

// Keep the pinned Wrangler route fixed, including its environment and output capture.
function wrangler(args, { timeout, account, token }) {
  const pkg = JSON.parse(readFileSync(new URL('node_modules/wrangler/package.json', PILOT_ROOT), 'utf8'));
  if (pkg.version !== '4.151.1') fail('Image cleanup requires pinned Wrangler 4.151.1');
  // Run the CLI directly: its bin shim forks a child that could outlive a timeout.
  return spawnSync(process.execPath, [fileURLToPath(new URL('node_modules/wrangler/wrangler-dist/cli.js', PILOT_ROOT)),
    ...args, '--env-file', '/dev/null'], {
    cwd: fileURLToPath(PILOT_ROOT), encoding: 'utf8', timeout, maxBuffer: 2 * 1024 * 1024,
    env: { PATH: process.env.PATH, CI: 'true', WRANGLER_SEND_METRICS: 'false', WRANGLER_WRITE_LOGS: 'false',
      CLOUDFLARE_ACCOUNT_ID: account, CLOUDFLARE_API_TOKEN: token },
  });
}

/** Read fixed pilot resources and optionally remove only their retained registry tags after verified teardown.
 * @param {object} options Audit inputs; injected transports are used only by offline tests.
 * @param {'audit'|'cleanup-images'} options.operation Fixed operation.
 * @param {'present'|'absent'} options.expectedState Required cloud state; cleanup requires absent.
 * @param {string} options.expectedJson Safe identity object copied from deployment output.
 * @param {string} options.account Existing CI Cloudflare account ID.
 * @param {string} options.token Existing CI API token; never returned or logged.
 * @param {Function} [options.fetcher] HTTP transport, defaulting to fetch.
 * @param {Function} [options.runWrangler] Pinned registry CLI transport.
 * @returns {Promise<object>} Allowlisted identity/schedule evidence and registry cleanup status.
 */
export async function runAudit({ operation, expectedState, expectedJson, account, token, fetcher = fetch, runWrangler = wrangler }) {
  if (!['audit', 'cleanup-images'].includes(operation) || !['present', 'absent'].includes(expectedState)
      || (operation === 'cleanup-images' && expectedState !== 'absent')) fail('Cleanup requires an explicit absent expectation');
  const expected = expectedIdentities(expectedJson);
  if (!/^[a-f0-9]{32}$/.test(account ?? '') || !token) fail('Audit requires the existing CI account and API token');
  const imageRepository = new RegExp(`^(?:https?://)?registry\\.cloudflare\\.com(?::443)?/${account}/${REPOSITORY}(?:[:@]|$)`, 'i');
  const deadline = Date.now() + 5 * 60_000;
  const remaining = () => {
    const ms = deadline - Date.now();
    if (ms <= 0) fail('Pilot audit operation deadline exceeded');
    return ms;
  };
  const request = async (resource, notFound) => {
    let response, body;
    try {
      response = await fetcher(`https://api.cloudflare.com/client/v4/accounts/${account}/${resource}`, {
        method: 'GET', redirect: 'error', headers: { authorization: `Bearer ${token}` },
        signal: AbortSignal.timeout(Math.min(15_000, remaining())),
      });
      body = await responseJson(response);
    } catch { fail('Pilot audit read failed or timed out; absence is unverified'); }
    if (response.status === 404 && body?.success === false && Array.isArray(body.errors)
        && body.errors.length > 0 && notFound && body.errors.every(notFound)) return undefined;
    // Live namespace lists report errors:null on successful reads; other endpoints retain their existing envelope.
    const emptyNamespaceErrors = body?.errors === null && resource.split('?')[0] === 'workers/durable_objects/namespaces';
    if (!response.ok || body?.success !== true || !Object.hasOwn(body, 'result')
        || (body.errors !== undefined && !emptyNamespaceErrors && (!Array.isArray(body.errors) || body.errors.length))) {
      // Only fixed endpoint labels and predicate states may leave the response boundary.
      const endpoint = new Map([
        ['workers/scripts', 'worker-list'],
        ['workers/durable_objects/namespaces', 'namespace-list'],
        ['containers/applications', 'application-list'],
        ...[['worker', pilot.name], ['probe', pilot.probe_name]].flatMap(([role, name]) => [
          [`workers/scripts/${name}/settings`, `${role}-settings`],
          [`workers/scripts/${name}/schedules`, `${role}-schedules`],
        ]),
        [`containers/applications/${expected.containerApplicationId}`, 'application-detail'],
        [`r2/buckets/${expected.bucketName}`, 'bucket-detail'],
      ]).get(resource.split('?')[0]) ?? 'unknown';
      const success = body?.success === true ? 'true' : body?.success === false ? 'false' : 'invalid';
      const result = body != null && Object.hasOwn(body, 'result') ? 'present' : 'missing';
      const errors = body?.errors === undefined ? 'absent' : Array.isArray(body.errors)
        ? body.errors.length === 0 ? 'empty-array' : 'nonempty-array' : 'invalid';
      const errorsKind = body?.errors === null ? 'null' : Array.isArray(body?.errors) ? 'array' : typeof body?.errors;
      const codes = Array.isArray(body?.errors) && body.errors.length ? body.errors.slice(0, 5)
        .map((error) => Number.isSafeInteger(error?.code) && error.code >= 0 && error.code <= 1_000_000_000 ? error.code : 'invalid').join(',') : 'none';
      fail(`Pilot audit API read failed with HTTP ${response.status}; endpoint=${endpoint}; success=${success}; result=${result}; errors=${errors}; errors-kind=${errorsKind}; codes=${codes}; absence is unverified`);
    }
    const nextLink = [...(response.headers.get('link') ?? '').matchAll(/(?:^|[;,])\s*rel\s*=\s*(?:"([^"]*)"|([^;,\s]+))/gi)]
      .some(([, quoted, bare]) => (quoted ?? bare).toLowerCase().split(/\s+/).includes('next'));
    if (nextLink
        && (!body.result_info || (resource.split('?')[0] === 'containers/applications'
          && !Object.hasOwn(body.result_info, 'next_page_token')))) fail('Pilot audit cannot verify cursor pagination');
    return body;
  };
  const list = async (resource, paginated = false) => {
    const items = [];
    let pages = 1, total;
    for (let page = 1; page <= pages; page++) {
      if (page > 100) fail('Pilot audit pagination limit reached');
      const body = await request(`${resource}${paginated || page > 1 ? `?page=${page}&per_page=100` : ''}`);
      if (!Array.isArray(body.result)) fail('Pilot audit expected a resource list');
      const info = body.result_info;
      if (paginated || info != null) {
        if (!info || info.page !== page || !Number.isInteger(info.per_page) || info.per_page < 1
            || !Number.isInteger(info.total_count) || info.total_count < 0
            || info.count !== body.result.length || info.cursors?.after
            || (info.total_pages !== undefined && (!Number.isInteger(info.total_pages) || info.total_pages < 0))) fail('Pilot audit pagination is inconclusive');
        const advertised = Math.max(1, info.total_pages ?? Math.ceil(info.total_count / info.per_page));
        if (!Number.isInteger(advertised) || advertised > 100 || advertised < page
            || (total !== undefined && (total !== info.total_count || pages !== advertised))) fail('Pilot audit pagination changed or exceeded its limit');
        pages = advertised;
        total = info.total_count;
      }
      items.push(...body.result);
    }
    if (total !== undefined && items.length !== total) fail('Pilot audit resource listing is incomplete');
    if (items.some((item) => !item || typeof item.id !== 'string')
        || new Set(items.map((item) => item.id)).size !== items.length) fail('Pilot audit resource listing contains invalid or repeated identities');
    return items;
  };
  // Containers use opaque cursors; only an absent next-page token proves traversal is complete.
  const listApplications = async () => {
    const items = [], tokens = new Set(), ids = new Set();
    let token;
    for (let page = 1; page <= 100; page++) {
      const query = new URLSearchParams({ per_page: '100' });
      if (token !== undefined) query.set('page_token', token);
      const body = await request(`containers/applications?${query}`);
      if (!Array.isArray(body.result)) fail('Pilot audit expected an application list');
      const info = body.result_info;
      if (!info || typeof info !== 'object' || Array.isArray(info)
          || Object.keys(info).some((key) => !['next_page_token', 'page_token', 'per_page'].includes(key))
          || (Object.hasOwn(info, 'per_page') && info.per_page !== 100)
          || (Object.hasOwn(info, 'page_token') && info.page_token !== token)) fail('Pilot audit application pagination is inconclusive');
      if (body.result.length > 100 || items.length + body.result.length > 10_000) fail('Pilot audit application row limit reached');
      for (const item of body.result) {
        if (!item || typeof item.id !== 'string' || !ID.test(item.id) || ids.has(item.id)) fail('Pilot audit application listing contains invalid or repeated identities');
        ids.add(item.id);
        items.push(item);
      }
      if (!Object.hasOwn(info, 'next_page_token')) return items;
      const next = info.next_page_token;
      if (typeof next !== 'string' || next.length === 0 || next.length > 4096 || tokens.has(next)) fail('Pilot audit application cursor is invalid or repeated');
      if (page === 100) fail('Pilot audit application pagination limit reached');
      tokens.add(next);
      token = next;
    }
  };
  const audit = async () => {
    const scripts = await list('workers/scripts');
    const namespaces = await list('workers/durable_objects/namespaces', true);
    const applications = await listApplications();
    if (applications.some((app) => typeof app.name !== 'string' || !app.name.trim())) fail('Pilot audit application names are inconclusive');
    const workers = [];
    const schedules = {};
    for (const [role, name, id] of [['worker', pilot.name, expected.workerId], ['probe', pilot.probe_name, expected.probeId]]) {
      const settings = (await request(`workers/scripts/${name}/settings`, (error) => error.code === 10007))?.result;
      const schedule = (await request(`workers/scripts/${name}/schedules`, (error) => error.code === 10007))?.result;
      const matches = scripts.filter((script) => script.id === name || script.tag === id);
      if (expectedState === 'absent') {
        if (settings !== undefined || schedule !== undefined || matches.length) fail('A pilot Worker or saved Worker identity still exists');
        schedules[role] = 'worker-not-found';
        continue;
      }
      if (matches.length !== 1 || matches[0].id !== name || matches[0].tag !== id || !Array.isArray(settings?.bindings)
          || !Array.isArray(schedule?.schedules)) fail('Live Worker identity or settings do not match the pilot');
      const crons = schedule.schedules.map((item) => item.cron);
      if (crons.some((cron) => role !== 'probe' || cron !== '*/5 * * * *') || crons.length > 1) fail('Unexpected pilot schedule');
      schedules[role] = crons;
      const safeBindings = [];
      if (role === 'probe' && settings.bindings.some((binding) => binding.type === 'durable_object_namespace')) fail('Unexpected probe namespace binding');
      if (role === 'worker') {
        const bindings = settings.bindings.filter((binding) => binding.type === 'durable_object_namespace');
        if (bindings.length !== 3) fail('Unexpected pilot namespace bindings');
        for (const [bindingName, className] of Object.entries(BINDINGS)) {
          const matches = bindings.filter((binding) => binding.name === bindingName && binding.class_name === className
            && binding.namespace_id === expected.durableObjectNamespaces[className]
            && (binding.script_name == null || binding.script_name === pilot.name));
          if (matches.length !== 1) fail('A pilot namespace binding points elsewhere');
          safeBindings.push({ bindingName, className, namespaceId: expected.durableObjectNamespaces[className] });
        }
      }
      for (const [bindingName, type, field, value] of role === 'worker'
        ? [['BUCKET', 'r2_bucket', 'bucket_name', expected.bucketName]]
        : [['TARGET', 'service', 'service', pilot.name], ['RESULTS', 'r2_bucket', 'bucket_name', expected.bucketName]]) {
        const matches = settings.bindings.filter((binding) => binding.name === bindingName && binding.type === type && binding[field] === value);
        if (matches.length !== 1) fail('A pilot service or bucket binding points elsewhere');
        // Plain Alchemy Worker bindings select no environment and use the default export.
        if (bindingName === 'TARGET' && (matches[0].environment != null
            || (matches[0].entrypoint != null && matches[0].entrypoint !== 'default'))) fail('Pilot target uses an unexpected environment or entrypoint');
        safeBindings.push({ bindingName });
      }
      workers.push({ workerName: name, workerId: id, bindings: safeBindings });
    }
    const relevantNamespaces = namespaces.filter((ns) => Object.values(expected.durableObjectNamespaces).includes(ns.id)
      || ns.script === pilot.name || ns.script === pilot.probe_name);
    const relevantApps = applications.filter((app) => app.id === expected.containerApplicationId || app.name === pilot.container_name
      || app.durable_objects?.namespace_id === expected.durableObjectNamespaces.Media);
    const application = (await request(`containers/applications/${expected.containerApplicationId}`,
      (error) => error.code === 1609 && /Container application not found|APPLICATION_NOT_FOUND/.test(error.message)))?.result;
    const bucket = (await request(`r2/buckets/${expected.bucketName}`, (error) => error.code === 10006))?.result;
    if (expectedState === 'absent') {
      if (relevantNamespaces.length || relevantApps.length || application !== undefined || bucket !== undefined) fail('Pilot resources or saved identities still exist; image cleanup is blocked');
      if (operation === 'cleanup-images') {
        for (const app of applications) {
          const image = app.configuration?.image;
          if (typeof image !== 'string' || !image.trim()) fail('Live application image usage is inconclusive; cleanup is blocked');
          if (imageRepository.test(image.trim())) fail('A live application still uses pilot images; cleanup is blocked');
        }
      }
    } else {
      if (relevantNamespaces.length !== 3 || Object.entries(expected.durableObjectNamespaces).some(([className, id]) =>
        relevantNamespaces.filter((ns) => ns.id === id && ns.class === className && ns.script === pilot.name).length !== 1)) fail('Live namespace ownership does not match the pilot');
      if (relevantApps.length !== 1 || relevantApps[0].id !== expected.containerApplicationId || relevantApps[0].name !== pilot.container_name
          || application?.id !== expected.containerApplicationId || application?.name !== pilot.container_name
          || application?.durable_objects?.namespace_id !== expected.durableObjectNamespaces.Media
          || bucket?.name !== expected.bucketName) fail('Live application or bucket does not match the pilot');
    }
    return { state: expectedState, identities: expected, schedules, workers };
  };
  const evidence = await audit();
  if (operation === 'audit') return evidence;
  const registryCommand = (args) => {
    const result = runWrangler(args, { timeout: Math.min(60_000, remaining()), account, token });
    if (result.error || result.status !== 0) fail('Pinned pilot registry command failed; cleanup is incomplete');
    return result;
  };
  const inventory = () => {
    const result = registryCommand(['containers', 'images', 'list', '--filter', `(^|/)${REPOSITORY}$`, '--json']);
    let entries;
    try { entries = JSON.parse(result.stdout); } catch { fail('Pilot registry inventory is unreadable'); }
    if (!Array.isArray(entries) || entries.length > 1 || entries.some((entry) => entry.name !== REPOSITORY || !Array.isArray(entry.tags)
      || entry.tags.some((tag) => typeof tag !== 'string' || !TAG.test(tag)))) fail('Registry inventory includes a foreign repository or invalid tags');
    const tags = entries.flatMap((entry) => entry.tags);
    if (new Set(tags).size !== tags.length) fail('Registry inventory contains repeated tags');
    return tags.sort();
  };
  const tags = inventory();
  // Inventory may take time: repeat the complete absence proof immediately before deleting images.
  await audit();
  let garbageCollectionWarnings = 0;
  for (const tag of tags) {
    const result = registryCommand(['containers', 'images', 'delete', `${REPOSITORY}:${tag}`, '--skip-confirmation']);
    if (/garbage-collection request failed/.test(`${result.stdout}\n${result.stderr}`)) garbageCollectionWarnings++;
  }
  if (inventory().length) fail('Pilot registry still contains tags; cleanup is incomplete');
  return { ...evidence, deletedTags: tags, garbageCollectionWarnings, layerReclamationVerified: false };
}

async function main() {
  if (process.argv.length !== 3 || process.env.GITHUB_ACTIONS !== 'true' || process.env.GITHUB_REF !== 'refs/heads/main'
      || process.env.MYMIND_ALCHEMY_INITIALIZE !== 'false' || !existsSync(new URL('.alchemy/state/', PILOT_ROOT))) fail('Run audit/cleanup only through the serialized main workflow with restored state and initialize=false');
  const evidence = await runAudit({ operation: process.argv[2], expectedState: process.env.MYMIND_ALCHEMY_EXPECTED_STATE,
    expectedJson: process.env.MYMIND_ALCHEMY_EXPECTED_IDENTITIES, account: process.env.CLOUDFLARE_ACCOUNT_ID,
    token: process.env.CLOUDFLARE_API_TOKEN });
  console.log(JSON.stringify(evidence));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
