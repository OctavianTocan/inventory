import { createCipheriv, createDecipheriv, randomBytes, scryptSync } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { PILOT_ROOT } from './contract.mjs';

export const ARTIFACT_NAME = 'mymind-alchemy-staging-state-v1';

/** Encrypt local Alchemy state, which can contain raw secret values.
 * @param {object} state JSON-compatible state payload.
 * @param {string} password Dedicated state encryption key.
 * @returns {Buffer} Authenticated encrypted payload.
 */
export function seal(state, password) {
  const salt = randomBytes(16), iv = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', scryptSync(password, salt, 32), iv);
  const encrypted = Buffer.concat([cipher.update(JSON.stringify(state)), cipher.final()]);
  return Buffer.concat([Buffer.from('MMP1'), salt, iv, cipher.getAuthTag(), encrypted]);
}

/** Decrypt a state archive, rejecting wrong keys and modified ciphertext.
 * @param {Buffer} payload Previously sealed state.
 * @param {string} password Dedicated state encryption key.
 * @returns {object} Original state payload.
 */
export function unseal(payload, password) {
  if (payload.subarray(0, 4).toString() !== 'MMP1' || payload.length < 48) throw new Error('Invalid encrypted pilot state');
  const decipher = createDecipheriv('aes-256-gcm', scryptSync(password, payload.subarray(4, 20), 32), payload.subarray(20, 32));
  decipher.setAuthTag(payload.subarray(32, 48));
  return JSON.parse(Buffer.concat([decipher.update(payload.subarray(48)), decipher.final()]).toString());
}

/** Choose the newest artifact by creation time and refuse ambiguous or expired recovery points.
 * @param {Array<object>} artifacts GitHub artifact records.
 * @param {boolean} initialize Explicit first deployment flag.
 * @returns {object | undefined} The exact newest state artifact, or no first-run state.
 */
export function selectArtifact(artifacts, initialize) {
  const checkpoints = artifacts.filter((item) => item.name === ARTIFACT_NAME).map((item) => {
    const stamp = typeof item.created_at === 'string' && /^(\d{4}-\d{2}-\d{2})T(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:\.\d{1,3})?(?:Z|[+-](?:[01]\d|2[0-3]):[0-5]\d)$/.exec(item.created_at);
    const time = stamp ? Date.parse(item.created_at) : NaN;
    const day = stamp ? Date.parse(`${stamp[1]}T00:00:00Z`) : NaN;
    if (!Number.isFinite(time) || !Number.isFinite(day) || new Date(day).toISOString().slice(0, 10) !== stamp[1]) {
      throw new Error('Pilot state artifact timestamp is invalid; recover the authoritative checkpoint');
    }
    if (!Number.isSafeInteger(item.id) || item.id <= 0 || typeof item.expired !== 'boolean') throw new Error('Pilot state artifact metadata is invalid');
    return { item, time };
  }).sort((a, b) => b.time - a.time);
  const newest = checkpoints[0];
  if (newest && checkpoints.some(({ item, time }) => time === newest.time
      && (item.id !== newest.item.id || item.workflow_run?.id !== newest.item.workflow_run?.id))) {
    throw new Error('Newest pilot state artifacts have ambiguous timestamps; recover the authoritative checkpoint');
  }
  const latest = newest?.item;
  if (!latest && !initialize) throw new Error('State artifact is missing; recover original state or explicitly initialize a never-deployed pilot');
  if (latest?.expired) throw new Error('Newest pilot state artifact expired; recover it instead of using older state');
  if (latest && initialize) throw new Error('Initialize is forbidden after a state artifact exists');
  return latest;
}

// Verify the complete, stable per_page=100 listing before choosing any checkpoint.
function artifactPages(pages) {
  const fail = () => { throw new Error('Pilot state artifact pagination is incomplete or invalid'); };
  if (!Array.isArray(pages) || pages.length === 0) fail();
  const total = pages[0]?.total_count;
  if (!Number.isSafeInteger(total) || total < 0 || pages.length !== Math.max(1, Math.ceil(total / 100))) fail();
  const seen = new Set();
  return pages.flatMap((page, index) => {
    if (!page || typeof page !== 'object' || Array.isArray(page) || page.total_count !== total || !Array.isArray(page.artifacts)
        || page.artifacts.length !== Math.min(100, Math.max(0, total - index * 100))) fail();
    for (const item of page.artifacts) {
      if (!item || typeof item !== 'object' || Array.isArray(item) || typeof item.name !== 'string'
          || !Number.isSafeInteger(item.id) || item.id <= 0 || seen.has(item.id)) fail();
      seen.add(item.id);
    }
    return page.artifacts;
  });
}

/** Refuse checkpoints predating the latest operation, including canceled runs.
 * @param {object | undefined} artifact Latest encrypted checkpoint.
 * @param {object | undefined} deployment Latest operation marker.
 * @returns {void}
 */
export function assertCheckpoint(artifact, deployment) {
  if (!artifact && !deployment) return;
  const checkpointTime = Date.parse(artifact?.created_at);
  const operationTime = Date.parse(deployment?.created_at);
  if (!artifact || !deployment || !Number.isFinite(checkpointTime) || !Number.isFinite(operationTime)
      || String(artifact.workflow_run?.id) !== String(deployment.payload?.run_id)
      || checkpointTime < operationTime) {
    throw new Error('Latest pilot operation has no authoritative checkpoint; recover its original runner state');
  }
}

// Invoke GitHub without emitting its response bodies or credentials.
function gh(args, binary = false, input) {
  const result = spawnSync('gh', args, { input, encoding: binary ? undefined : 'utf8', maxBuffer: 32 * 1024 * 1024 });
  if (result.status !== 0) throw new Error('Private GitHub state API operation failed');
  return result.stdout;
}

// Snapshot only state files, excluding the deployment cache and local credential stores.
function snapshot(directory, prefix = '') {
  if (!existsSync(directory)) return {};
  return Object.fromEntries(readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    const file = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) throw new Error('Pilot state cannot contain symbolic links');
    return entry.isDirectory() ? Object.entries(snapshot(file, relative)) : [[relative, readFileSync(file).toString('base64')]];
  }));
}

async function main() {
  const command = process.argv[2];
  const password = process.env.MYMIND_ALCHEMY_STATE_KEY;
  if (!password || password.length < 32) throw new Error('A dedicated MYMIND_ALCHEMY_STATE_KEY of at least 32 characters is required');
  const root = fileURLToPath(PILOT_ROOT);
  const artifactDir = path.join(root, '.artifacts/state');
  const stateDir = path.join(root, '.alchemy/state');
  mkdirSync(artifactDir, { recursive: true });
  if (command === 'seal') {
    writeFileSync(path.join(artifactDir, 'state.enc'), seal({ version: 1, files: snapshot(stateDir) }, password), { mode: 0o600 });
  } else if (command === 'restore') {
    const repository = process.env.GITHUB_REPOSITORY;
    if (!/^[\w.-]+\/[\w.-]+$/.test(repository ?? '')) throw new Error('A private GITHUB_REPOSITORY is required');
    if (existsSync(stateDir)) throw new Error('Restore requires a clean runner state directory');
    const meta = JSON.parse(gh(['api', `repos/${repository}`]));
    if (!meta.private) throw new Error('Pilot infrastructure state requires a private repository');
    const listing = JSON.parse(gh(['api', `repos/${repository}/actions/artifacts?name=${ARTIFACT_NAME}&per_page=100`, '--paginate', '--slurp']));
    const artifact = selectArtifact(artifactPages(listing), process.env.MYMIND_ALCHEMY_INITIALIZE === 'true');
    const history = JSON.parse(gh(['api', `repos/${repository}/deployments?environment=mymind-alchemy-staging&per_page=1`]));
    assertCheckpoint(artifact, history[0]);
    if (!process.env.GITHUB_SHA || !/^\d+$/.test(process.env.GITHUB_RUN_ID ?? '')) throw new Error('State restore requires an explicit GitHub commit and workflow run ID');
    const markOperation = () => gh(['api', '--method', 'POST', `repos/${repository}/deployments`, '--input', '-'], false, JSON.stringify({
      ref: process.env.GITHUB_SHA, environment: 'mymind-alchemy-staging', auto_merge: false,
      required_contexts: [], transient_environment: true,
      payload: { run_id: process.env.GITHUB_RUN_ID },
      description: 'Alchemy staging operation started; recover its checkpoint before another operation',
    }));
    mkdirSync(stateDir, { recursive: true });
    if (!artifact) {
      markOperation();
      return;
    }
    const run = JSON.parse(gh(['api', `repos/${repository}/actions/runs/${artifact.workflow_run.id}`]));
    if (run.path !== '.github/workflows/alchemy-pilot.yml') throw new Error('State artifact belongs to a different workflow');
    const archive = path.join(artifactDir, 'state.zip');
    writeFileSync(archive, gh(['api', `repos/${repository}/actions/artifacts/${artifact.id}/zip`], true), { mode: 0o600 });
    const unzip = spawnSync('unzip', ['-p', archive, 'state.enc'], { maxBuffer: 32 * 1024 * 1024 });
    if (unzip.status !== 0) throw new Error('Encrypted state archive is unreadable');
    const decoded = unseal(unzip.stdout, password);
    if (decoded.version !== 1 || !decoded.files || typeof decoded.files !== 'object') throw new Error('Unsupported pilot state archive');
    for (const [relative, contents] of Object.entries(decoded.files)) {
      if (!relative || relative.split('/').some((part) => !part || part === '..' || part === '.') || path.isAbsolute(relative) || relative.includes('\\')) throw new Error('Unsafe path in pilot state archive');
      const file = path.join(stateDir, relative);
      mkdirSync(path.dirname(file), { recursive: true });
      writeFileSync(file, Buffer.from(contents, 'base64'), { mode: 0o600 });
    }
    markOperation();
  } else throw new Error('Use state.mjs restore or seal');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
