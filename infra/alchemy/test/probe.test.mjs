import assert from 'node:assert/strict';
import test from 'node:test';
import probe from '../probe.mjs';

// Model the existing private app API and R2 while exercising the real probe lifecycle.
function fixture({ mediaStatus = 201, readStatus = 200 } = {}) {
  const objects = new Map();
  const requests = [];
  let revoked = true;
  const env = {
    ADMIN_TOKEN: 'synthetic-admin', PILOT_PASSWORD: 'synthetic-password', PILOT_HASH: 'synthetic-hash', PILOT_RUN: 'synthetic-first',
    RESULTS: {
      async get(key) { return objects.has(key) ? JSON.parse(objects.get(key)) : null; },
      async put(key, value) { objects.set(key, value); },
    },
    TARGET: {
      async fetch(request) {
        const url = new URL(request.url);
        assert.equal(url.origin, 'https://mymind-alchemy-staging.invalid');
        requests.push(`${request.method} ${url.pathname}`);
        if (url.pathname.startsWith('/_admin/')) {
          if (request.headers.get('authorization') !== 'Bearer synthetic-admin') return new Response(null, { status: 401 });
          const body = await request.json();
          if (url.pathname === '/_admin/auth/user') assert.equal(body.username, 'alchemy-staging-synthetic');
          else assert.equal(body.library, 'alchemy-staging-synthetic');
          return new Response(null, { status: 204 });
        }
        if (url.pathname === '/signin/password') {
          revoked = false;
          return new Response(null, { status: 303, headers: { location: '/everything', 'set-cookie': 'session=synthetic; Secure; HttpOnly' } });
        }
        if (url.pathname === '/objects' && request.method === 'GET') return new Response(null, { status: 401 });
        assert.equal(request.headers.get('cookie'), 'session=synthetic');
        if (url.pathname === '/signout') { revoked = true; return new Response(null, { status: 302 }); }
        if (url.pathname === '/app/v1/me') return Response.json({}, { status: revoked ? 401 : 200 });
        if (url.pathname === '/uploads/authorize') return Response.json({ url: '/uploads/synthetic', token: 'synthetic-upload-token' });
        if (url.pathname === '/uploads/synthetic') {
          assert.equal(request.headers.get('x-upload-token'), 'synthetic-upload-token');
          return Response.json({ done: true, id: 'synthetic-upload' });
        }
        if (url.pathname === '/objects') return Response.json({ id: 'synthetic-card' }, { status: mediaStatus });
        if (url.pathname === '/objects/synthetic-card' && request.method === 'GET') return Response.json({}, { status: readStatus });
        if (url.pathname === '/objects/synthetic-card' && request.method === 'DELETE') return Response.json({});
        if (url.pathname === '/objects/synthetic-card/restore') return Response.json({});
        throw new Error('Unexpected private probe request');
      },
    },
  };
  return { env, requests, objects, result: () => JSON.parse(objects.get(`pilot/results/${env.PILOT_RUN}.json`)) };
}

test('a scheduled success records status-only evidence and stops repeating work', async () => {
  const f = fixture();
  await probe.scheduled({}, f.env);
  const result = f.result();
  assert.equal(result.passed, true);
  assert.equal(result.completedRuns, 1);
  assert.ok(f.requests.includes('DELETE /objects/synthetic-card'));
  assert.equal(f.requests.at(-2), 'GET /signout');
  assert.equal(f.requests.at(-1), 'GET /app/v1/me');
  const count = f.requests.length;
  await probe.scheduled({}, f.env);
  assert.equal(f.requests.length, count);
  assert.ok(!JSON.stringify(result).includes('synthetic-password'));
  assert.ok(!JSON.stringify(result).includes('synthetic-admin'));
});

test('media failures revoke sessions, retain failure evidence and stop after two attempts', async () => {
  const f = fixture({ mediaStatus: 500 });
  await probe.scheduled({}, f.env);
  assert.equal(f.result().passed, false);
  assert.equal(f.result().attempts, 1);
  await probe.scheduled({}, f.env);
  assert.equal(f.result().attempts, 2);
  assert.match(f.result().failure, /real media container: received 500/);
  assert.equal(f.requests.filter((entry) => entry === 'GET /signout').length, 2);
  const count = f.requests.length;
  await probe.scheduled({}, f.env);
  assert.equal(f.requests.length, count);
});

test('a repeated synthetic object is restored, read and cleaned up', async () => {
  const f = fixture({ mediaStatus: 200 });
  f.objects.set('pilot/repeatability.json', JSON.stringify({ runs: 1, lastRun: 'synthetic-before' }));
  await probe.scheduled({}, f.env);
  assert.equal(f.result().passed, true);
  assert.equal(f.result().completedRuns, 2);
  assert.ok(f.requests.includes('POST /objects/synthetic-card/restore'));
  assert.ok(f.requests.includes('DELETE /objects/synthetic-card'));
});

test('a failure after media creation still deletes the synthetic object and revokes its session', async () => {
  const f = fixture({ readStatus: 500 });
  await probe.scheduled({}, f.env);
  assert.equal(f.result().passed, false);
  assert.match(f.result().failure, /persisted Library read: expected 200, received 500/);
  assert.ok(f.requests.includes('DELETE /objects/synthetic-card'));
  assert.equal(f.requests.at(-2), 'GET /signout');
  assert.equal(f.requests.at(-1), 'GET /app/v1/me');
  assert.equal(f.objects.has('pilot/repeatability.json'), false);
});
