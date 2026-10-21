import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { PILOT_ROOT, readContract } from './contract.mjs';

const command = process.argv[2];
if (!['plan', 'deploy', 'probe', 'stop', 'destroy'].includes(command) || process.argv.length !== 3) throw new Error('Use a fixed pilot npm command without extra arguments');
const { pilot } = readContract();
const required = ['CLOUDFLARE_ACCOUNT_ID', 'CLOUDFLARE_API_TOKEN', 'MYMIND_ALCHEMY_GOOGLE_CLIENT_ID', 'MYMIND_ALCHEMY_GOOGLE_CLIENT_SECRET', 'MYMIND_ALCHEMY_ADMIN_TOKEN', 'MYMIND_ALCHEMY_PASSWORD'];
const missing = required.filter((name) => !process.env[name]);
if (missing.length) throw new Error(`Cloud operation blocked; missing environment names: ${missing.join(', ')}. Offline checks do not deploy.`);
if (command !== 'destroy' && !existsSync(new URL(pilot.main, PILOT_ROOT))) throw new Error('Run npm run build:worker first');
if (!existsSync(new URL('.alchemy/state/', PILOT_ROOT))) throw new Error('Cloud operations require shared pilot state; restore it from the private CI artifact first');
const run = process.env.MYMIND_ALCHEMY_RUN ?? randomUUID();
if (!/^[A-Za-z0-9_-]{1,80}$/.test(run)) throw new Error('Invalid synthetic probe run identifier');
if (command === 'probe') {
  mkdirSync(new URL('.artifacts/', PILOT_ROOT), { recursive: true });
  writeFileSync(new URL('.artifacts/probe-run', PILOT_ROOT), run, { mode: 0o600 });
}
const result = spawnSync(process.execPath, [
  './node_modules/alchemy/bin/cli.js', command === 'destroy' ? 'destroy' : command === 'plan' ? 'plan' : 'deploy',
  '--stage', 'staging', '--config', 'alchemy.run.ts', '--env-file', '/dev/null',
  ...(command === 'plan' ? [] : ['--yes']),
], { cwd: fileURLToPath(PILOT_ROOT), stdio: 'inherit', env: {
  ...process.env, ALCHEMY_STAGE: 'staging', MYMIND_ALCHEMY_RUN: run,
  ALCHEMY_HOME: fileURLToPath(new URL('.alchemy/auth/', PILOT_ROOT)),
  MYMIND_ALCHEMY_PROBE: command === 'probe' ? '1' : '0', NO_TRACK: '1',
} });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
