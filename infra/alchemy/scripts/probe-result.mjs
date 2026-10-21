import { readFileSync } from 'node:fs';
import { setTimeout } from 'node:timers/promises';
import { PILOT_ROOT, readContract } from './contract.mjs';

const { pilot } = readContract();
const run = process.env.MYMIND_ALCHEMY_RUN ?? readFileSync(new URL('.artifacts/probe-run', PILOT_ROOT), 'utf8').trim();
if (!/^[A-Za-z0-9_-]{1,80}$/.test(run)) throw new Error('Invalid probe run identifier');
const account = process.env.CLOUDFLARE_ACCOUNT_ID;
const token = process.env.CLOUDFLARE_API_TOKEN;
if (!account || !token) throw new Error('Cloudflare account and API token are required to read private smoke evidence');
const endpoint = `https://api.cloudflare.com/client/v4/accounts/${encodeURIComponent(account)}/r2/buckets/${pilot.r2_buckets[0].bucket_name}/objects/pilot/results/${encodeURIComponent(run)}.json`;
// Allow 15 minutes for cron propagation before the two five-minute attempts.
const deadline = Date.now() + 26 * 60_000;
let missingReads = 0;
let timedOutReads = 0;
for (;;) {
  try {
    const response = await fetch(endpoint, { headers: { authorization: `Bearer ${token}` }, signal: AbortSignal.timeout(15_000) });
    if (response.ok) {
      const result = await response.json();
      if (result.run !== run || typeof result.passed !== 'boolean') throw new Error('Unexpected private probe result');
      console.log(JSON.stringify(result));
      if (result.passed) break;
      if (result.attempts >= 2) throw new Error('Private smoke failed; stop the probe and inspect its status-only result');
    } else if (response.status === 404) {
      missingReads += 1;
    } else throw new Error(`Private R2 result read failed with HTTP ${response.status}`);
  } catch (error) {
    // One request timeout must not end the bounded polling window.
    if (error?.name !== 'TimeoutError') throw error;
    timedOutReads += 1;
  }
  if (Date.now() >= deadline) throw new Error(`No successful private smoke result within 26 minutes; R2 reads: ${missingReads} not found, ${timedOutReads} timed out; stop the probe`);
  await setTimeout(15_000);
}
