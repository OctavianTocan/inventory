const ORIGIN = 'https://mymind-alchemy-staging.invalid';
const USER = 'alchemy-staging-synthetic';
// A generated name lets Library seed this new account; the explicit assignment also repairs older pilot mappings.
const LIBRARY = 'u-alchemy-staging-synthetic-0ed301e039c627bdc50e031aad057f82426c417622d5df870882d1098755f9a0';
const PNG = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a9S8AAAAASUVORK5CYII=';

// Compare status only: never put response bodies, credentials or cookies in diagnostics.
function expect(response, status, step) {
  if (response.status !== status) throw new Error(`${step}: expected ${status}, received ${response.status}`);
  return response;
}

/** Exercise the real Rust auth, Library, R2 and container chain using synthetic data.
 * @param {object} env Private Worker bindings.
 * @returns {Promise<object>} Status-only evidence, suitable for the private result bucket.
 */
export async function smoke(env) {
  const call = (path, init = {}) => env.TARGET.fetch(new Request(`${ORIGIN}${path}`, { ...init, redirect: 'manual' }));
  const admin = async (path, value) => {
    const response = await call(`/_admin/${path}`, {
      method: 'POST', headers: { authorization: `Bearer ${env.ADMIN_TOKEN}`, 'content-type': 'application/json' },
      body: JSON.stringify(value),
    });
    if (!response.ok) throw new Error(`synthetic setup: received ${response.status}`);
  };
  expect(await call('/objects'), 401, 'signed-out denial');
  expect(await call('/_admin/auth', { headers: { authorization: 'Bearer invalid-pilot-token' } }), 401, 'admin denial');
  await admin('auth/user', {
    username: USER, salt: btoa('alchemy-staging-synthetic-salt'), hash: env.PILOT_HASH, created_ms: 1787000000000,
  });
  await admin('auth/library', { account: `password:${USER}`, library: LIBRARY });
  const login = expect(await call('/signin/password', {
    method: 'POST', headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ username: USER, password: env.PILOT_PASSWORD }).toString(),
  }), 303, 'password login');
  if (login.headers.get('location') !== '/everything') throw new Error('password login did not create a session');
  const cookie = login.headers.get('set-cookie')?.split(';')[0];
  if (!cookie) throw new Error('password login omitted its session cookie');
  const headers = { cookie };
  let card;
  try {
    expect(await call('/app/v1/me', { headers }), 200, 'authenticated request');
    const capability = await expect(await call('/uploads/authorize', { method: 'POST', headers }), 200, 'upload authorization').json();
    if (!/^\/uploads\/[A-Za-z0-9_-]+$/.test(capability.url) || typeof capability.token !== 'string') throw new Error('invalid upload capability shape');
    const png = Uint8Array.from(atob(PNG), (byte) => byte.charCodeAt(0));
    const upload = await expect(await call(capability.url, {
      method: 'PUT', headers: {
        ...headers, 'x-upload-token': capability.token, 'content-range': `bytes 0-${png.length - 1}/${png.length}`,
        'x-file-name': 'alchemy-synthetic.png', 'content-type': 'image/png',
      }, body: png,
    }), 200, 'R2 upload').json();
    if (upload.done !== true || typeof upload.id !== 'string') throw new Error('upload did not finish');
    const prepared = await call('/objects', {
      method: 'POST', headers: { ...headers, 'content-type': 'application/json' },
      body: JSON.stringify({ type: 'Media', uploadId: upload.id }),
    });
    // Repeated one-pixel fixtures can resolve to an existing, trashed card.
    if (![200, 201].includes(prepared.status)) throw new Error(`real media container: received ${prepared.status}`);
    const created = await prepared.json();
    card = created.id;
    if (typeof card !== 'string') throw new Error('container preparation omitted object identity');
    if (prepared.status === 200) expect(await call(`/objects/${encodeURIComponent(card)}/restore`, { method: 'POST', headers }), 200, 'repeat synthetic object restore');
    expect(await call(`/objects/${encodeURIComponent(card)}`, { headers }), 200, 'persisted Library read');
    const markerKey = 'pilot/repeatability.json';
    const previousObject = await env.RESULTS.get(markerKey);
    const previous = previousObject ? await previousObject.json() : null;
    await env.RESULTS.put(markerKey, JSON.stringify({ runs: (previous?.runs ?? 0) + 1, lastRun: env.PILOT_RUN }));
    return { passed: true, checks: ['signed-out-denied', 'admin-denied', 'password-session', 'authenticated-read', 'r2-upload', 'media-container', 'library-read'], completedRuns: (previous?.runs ?? 0) + 1 };
  } finally {
    try {
      if (card) expect(await call(`/objects/${encodeURIComponent(card)}`, { method: 'DELETE', headers }), 200, 'synthetic object cleanup');
    } finally {
      expect(await call('/signout', { headers }), 302, 'session cleanup');
      expect(await call('/app/v1/me', { headers }), 401, 'revoked session');
    }
  }
}

export default {
  fetch() { return new Response('Private scheduled probe', { status: 404 }); },
  async scheduled(_event, env) {
    const key = `pilot/results/${env.PILOT_RUN}.json`;
    const previousObject = await env.RESULTS.get(key);
    const previous = previousObject ? await previousObject.json() : null;
    if (previous?.passed || previous?.attempts >= 2) return;
    const result = { run: env.PILOT_RUN, attempts: (previous?.attempts ?? 0) + 1, timestamp: new Date().toISOString() };
    try {
      Object.assign(result, await smoke(env));
    } catch (error) {
      // All thrown messages above contain only fixed labels and status codes.
      Object.assign(result, { passed: false, failure: error instanceof Error && /^[a-zA-Z0-9 :,-]+$/.test(error.message) ? error.message.slice(0, 160) : 'probe failed; inspect private runtime diagnostics' });
    }
    await env.RESULTS.put(key, JSON.stringify(result), { httpMetadata: { contentType: 'application/json' } });
  },
};
