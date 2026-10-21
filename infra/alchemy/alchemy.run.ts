import { scryptSync } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import * as Alchemy from 'alchemy';
import * as Cloudflare from 'alchemy/Cloudflare';
import * as Config from 'effect/Config';
import * as Effect from 'effect/Effect';
import * as Redacted from 'effect/Redacted';
import { readContract, ROOT } from './scripts/contract.mjs';

const { pilot, production } = readContract();

export default Alchemy.Stack(pilot.stack, {
  providers: Cloudflare.providers(),
  // Cloudflare.state() bootstraps a public hostname; this pilot must not do that.
  state: Alchemy.localState(),
}, Effect.gen(function* () {
  const stage = yield* Alchemy.Stage;
  if (stage !== pilot.stage) throw new Error('Only the staging pilot stage is allowed');
  const bucket = yield* Cloudflare.R2.Bucket('Fixtures', {
    name: pilot.r2_buckets[0].bucket_name,
    publicAccess: false,
    domains: [],
    forceDestroy: true,
    lifecycleRules: [{
      id: 'synthetic-fixtures-expire', enabled: true, prefix: '',
      deleteObjectsTransition: { condition: { type: 'Age', maxAge: 86400 } },
      abortMultipartUploadsTransition: { condition: { type: 'Age', maxAge: 86400 } },
    }],
  });
  // Fleet binding preserves the Rust Media class's existing start() API.
  const media = Cloudflare.Container('MediaImage', {
    name: pilot.container_name,
    className: 'Media',
    context: fileURLToPath(ROOT),
    dockerfile: fileURLToPath(new URL(pilot.containers[0].image, ROOT)),
    instanceType: pilot.containers[0].instance_type,
    maxInstances: pilot.containers[0].max_instances,
    publish: { repository: pilot.container_name },
  });
  const worker = yield* Cloudflare.Worker('RustWorker', {
    name: pilot.name,
    main: pilot.main,
    bundle: false,
    rules: [{ globs: ['**/*.js', '**/*.mjs', '**/*.wasm'] }],
    workersDev: false,
    routes: [],
    compatibility: { date: production.compatibility_date, flags: production.compatibility_flags },
    limits: { cpuMs: production.limits.cpu_ms },
    observability: { enabled: false },
    env: {
      ...pilot.vars,
      GOOGLE_CLIENT_ID: Config.Redacted('MYMIND_ALCHEMY_GOOGLE_CLIENT_ID'),
      GOOGLE_CLIENT_SECRET: Config.Redacted('MYMIND_ALCHEMY_GOOGLE_CLIENT_SECRET'),
      ADMIN_TOKEN: Config.Redacted('MYMIND_ALCHEMY_ADMIN_TOKEN'),
      AUTH_OBJECT: Cloudflare.DurableObject('Auth', { className: 'Auth' }),
      LIBRARY: Cloudflare.DurableObject('Library', { className: 'Library' }),
      MEDIA: media,
      BUCKET: bucket,
    },
  });
  const password = yield* Config.Redacted('MYMIND_ALCHEMY_PASSWORD');
  if (Redacted.value(password).length < 16) throw new Error('Synthetic pilot password must have at least 16 characters');
  const hash = scryptSync(Redacted.value(password), 'alchemy-staging-synthetic-salt', 64, { N: 16384, r: 8, p: 1 });
  const probe = yield* Cloudflare.Worker('PrivateProbe', {
    name: pilot.probe_name,
    main: './probe.mjs',
    workersDev: false,
    routes: [],
    compatibility: { date: production.compatibility_date, flags: production.compatibility_flags },
    observability: { enabled: false },
    crons: process.env.MYMIND_ALCHEMY_PROBE === '1' ? ['*/5 * * * *'] : [],
    env: {
      TARGET: worker,
      RESULTS: bucket,
      ADMIN_TOKEN: Config.Redacted('MYMIND_ALCHEMY_ADMIN_TOKEN'),
      PILOT_PASSWORD: password,
      PILOT_HASH: Redacted.make(hash.toString('base64')),
      PILOT_RUN: process.env.MYMIND_ALCHEMY_RUN ?? 'manual',
    },
  });
  const container = yield* media.Application;
  return {
    workerName: worker.workerName,
    workerId: worker.workerId,
    probeName: probe.workerName,
    probeId: probe.workerId,
    bucketName: bucket.bucketName,
    durableObjectNamespaces: worker.durableObjectNamespaces,
    containerApplicationId: container.applicationId,
  };
}));
