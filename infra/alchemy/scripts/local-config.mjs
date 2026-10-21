import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { PILOT_ROOT, ROOT, readContract } from './contract.mjs';

const { pilot, production } = readContract();
mkdirSync(new URL('.artifacts/', PILOT_ROOT), { recursive: true });
writeFileSync(new URL('.artifacts/wrangler.local.json', PILOT_ROOT), JSON.stringify({
  name: pilot.name, main: fileURLToPath(new URL(pilot.main, PILOT_ROOT)),
  compatibility_date: production.compatibility_date, compatibility_flags: production.compatibility_flags,
  workers_dev: false, preview_urls: false, routes: [],
  vars: { ...pilot.vars, PUBLIC_URL: 'http://127.0.0.1:8798', GOOGLE_CLIENT_ID: 'offline-client', GOOGLE_CLIENT_SECRET: 'offline-secret', ADMIN_TOKEN: 'worker-oracle-local-admin-only' },
  durable_objects: pilot.durable_objects, migrations: production.migrations,
  containers: pilot.containers.map((container) => ({ ...container, image: fileURLToPath(new URL(container.image, ROOT)), image_build_context: fileURLToPath(ROOT) })),
  r2_buckets: pilot.r2_buckets.map((bucket) => ({ ...bucket, remote: false })),
}, null, 2));
writeFileSync(new URL('.artifacts/empty-vars', PILOT_ROOT), '');
console.log('Local-only synthetic configuration: .artifacts/wrangler.local.json');
