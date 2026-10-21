import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import * as Effect from 'effect/Effect';

const fixture = new URL('./record-stack.mjs', import.meta.url).href;
const stubs = new Set(['alchemy', 'alchemy/Cloudflare', 'effect/Config']);
registerHooks({
  resolve(specifier, context, nextResolve) {
    return stubs.has(specifier) ? { url: fixture, shortCircuit: true } : nextResolve(specifier, context);
  },
});
globalThis.fetch = () => { throw new Error('Cloud calls are forbidden by the declaration fixture'); };
const { resources } = await import(fixture);
const { default: stack } = await import('../../alchemy.run.ts');
if (process.env.STACK_FIXTURE_STAGE === 'production') {
  await assert.rejects(Effect.runPromise(stack.program), /Only the staging pilot stage is allowed/);
  assert.equal(resources.length, 0);
} else {
  await Effect.runPromise(stack.program);
  assert.equal(stack.name, 'mymind-alchemy-pilot');
  assert.equal(stack.options.state.kind, 'local-state');
  const bucket = resources.find((resource) => resource.kind === 'bucket');
  assert.equal(bucket.props.name, 'mymind-alchemy-staging-fixtures');
  assert.equal(bucket.props.publicAccess, false);
  assert.deepEqual(bucket.props.domains, []);
  const container = resources.find((resource) => resource.kind === 'container');
  assert.equal(container.props.name, 'mymind-alchemy-staging-media');
  assert.equal(container.props.className, 'Media');
  assert.equal(container.props.dockerfile, 'rust/container/Dockerfile');
  assert.equal(container.props.maxInstances, 1);
  const workers = resources.filter((resource) => resource.kind === 'worker');
  assert.deepEqual(workers.map((resource) => resource.props.name), ['mymind-alchemy-staging-worker', 'mymind-alchemy-staging-probe']);
  for (const { props } of workers) {
    assert.equal(props.workersDev, false);
    assert.deepEqual(props.routes, []);
    assert.equal(props.domain, undefined);
    assert.equal(props.preview, undefined);
    assert.equal(props.observability.enabled, false);
  }
  const [worker, probe] = workers;
  assert.equal(worker.props.bundle, false);
  assert.equal(worker.props.main, '.artifacts/rust/crates/worker/build/index.js');
  assert.ok(worker.props.rules.flatMap((rule) => rule.globs).includes('**/*.wasm'));
  assert.equal(worker.props.env.AUTH, 'google');
  assert.equal(worker.props.env.ALLOWED_GOOGLE_DOMAIN, '');
  assert.equal(worker.props.env.PUBLIC_URL, 'https://mymind-alchemy-staging.invalid');
  assert.equal(worker.props.env.AUTH_OBJECT.props.className, 'Auth');
  assert.equal(worker.props.env.LIBRARY.props.className, 'Library');
  assert.equal(worker.props.env.MEDIA, container);
  assert.equal(worker.props.env.BUCKET, bucket.output);
  for (const name of ['ADMIN_TOKEN', 'GOOGLE_CLIENT_ID', 'GOOGLE_CLIENT_SECRET']) assert.ok(Effect.isEffect(worker.props.env[name]));
  assert.equal(probe.props.env.TARGET, worker.output);
  assert.equal(probe.props.env.RESULTS, bucket.output);
  assert.deepEqual(probe.props.crons, process.env.MYMIND_ALCHEMY_PROBE === '1' ? ['*/5 * * * *'] : []);
}
