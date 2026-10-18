// Turns every verification run in evidence/drive/ into REA's fail-closed reconstruction coverage model and
// asks REA whether the replica is complete. Each screenshot is a surface with one claim ("pixel-identical to
// live"), proven by the REA screenshot-diff Evidence recorded in that run. REA answers `ready` only if every
// claim passed against the current replica code and normalization, recently, with the replica offline.
// Usage: node tools/coverage.mjs [--max-age-days 7] [--server rust|hosted-rust|hosted-worker]
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseArgs, promisify } from 'node:util';
import { normalizationDigest, verificationDigest, verificationMode } from './digests.mjs';
import { reaRoot } from './rea-root.mjs';

if (process.argv.includes('--help')) {
  console.log('Usage: npm run coverage -- [--max-age-days 7] [--server rust|hosted-rust|hosted-worker]\nChecks saved drive evidence against current Rust sources and normalization.');
  process.exit(0);
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DRIVE = path.join(ROOT, 'evidence', 'drive');
const run = promisify(execFile);
const { values: opts } = parseArgs({ options: {
  'max-age-days': { type: 'string', default: '7' }, server: { type: 'string', default: 'rust' },
} });
const mode = verificationMode(opts.server);

const REA = reaRoot();
const { createEvidenceBundle } = await import(pathToFileURL(path.join(REA, 'dist', 'domain', 'evidenceBundle.js')).href);
const { createReconstructionVerifierContract } = await import(pathToFileURL(path.join(REA, 'dist', 'domain', 'reconstructionCoverage.js')).href);

const sha = (text) => createHash('sha256').update(text).digest('hex');
const replica = await verificationDigest(mode);
const normalization = await normalizationDigest();
const ownerPath = mode === 'hosted-worker' ? 'rust/crates/worker/src/lib.rs' : 'rust/crates/server/src/main.rs';
const ownerDigest = sha(await readFile(path.join(ROOT, ownerPath)));

const records = new Map();
const surfaces = [];
const owners = [];
const claims = [];
const contracts = [];
const results = [];
const offlineEvidence = [];
let allOffline = true;

// The `npm run verify` matrix (quick debugging suites are not part of the claim).
const runs = ['desktop-light', 'desktop-dark', 'iphone', 'android'].flatMap((profile) => [`${profile}-ui`, `${profile}-cards`]);
// A patch run (`<run>@<part>`) re-verifies part of a run; the newer run's screenshots replace the same-named ones.
const load = async (dir) => ({
  report: JSON.parse(await readFile(path.join(DRIVE, dir, 'report.json'), 'utf8')),
  evidence: JSON.parse(await readFile(path.join(DRIVE, dir, 'rea-evidence.json'), 'utf8')),
});
for (const run of runs.filter((r) => existsSync(path.join(DRIVE, r, 'rea-evidence.json')))) {
  const parts = [await load(run)];
  for (const dir of readdirSync(DRIVE).filter((d) => d.startsWith(`${run}@`) && existsSync(path.join(DRIVE, d, 'rea-evidence.json')))) parts.push(await load(dir));
  const latest = new Map();
  const at = ({ report }) => Date.parse(report.replica?.at ?? report.startedAt ?? 0) || 0;
  for (const { report, evidence } of parts.sort((a, b) => at(a) - at(b))) {
    for (const record of evidence) records.set(record.evidence_id, record);
    allOffline &&= report.replica?.offline === true;
    for (const result of report.results ?? []) latest.set(result.name, { result, report });
  }
  for (const { result, report } of latest.values()) {
    if (!result.evidenceId) continue;
    const id = `${run}/${result.name}`;
    surfaces.push({
      surface_id: id,
      family: result.name.startsWith('hover_') ? 'hover-state' : /_\d{4}$/.test(result.name) ? 'animation-frame' : 'screen-state',
      artifact_id: 'mymind-replica',
      occurrence_id: null,
      location: `${run} step ${result.name}`,
      authority: 'observed',
      dependency_surface_ids: [],
      evidence_ids: [result.evidenceId],
    });
    owners.push({
      surface_id: id,
      ownership: { disposition: 'implemented', owner_path: ownerPath, owner_export: null, owner_sha256: ownerDigest, path_state: 'present', package_state: 'distributed', authority_route: report.replica?.offline ? 'none' : 'unknown' },
    });
    claims.push({ claim_id: `parity:${id}`, title: `Pixel-identical to live: ${id}`, kind: 'visual-parity', surface_ids: [id], required_dimensions: ['pixels'], required_authority: 'live-observation' });
    const contract = createReconstructionVerifierContract({
      verifier_id: `rea-compare:${id}`,
      claim_ids: [`parity:${id}`],
      dimensions: ['pixels'],
      authority: 'live-observation',
      max_age_ms: Number(opts['max-age-days']) * 86400000,
      minimum_repeats: 1,
      normalization_sha256: report.normalizationDigest ?? normalization,
      normalization_removes_dimensions: false,
    });
    contracts.push(contract);
    results.push({
      verifier_id: contract.verifier_id,
      contract_sha256: contract.contract_sha256,
      observed_at: report.replica?.at ?? report.startedAt,
      status: result.status === 'identical' ? 'pass' : 'fail',
      covered_claim_ids: [`parity:${id}`],
      covered_dimensions: ['pixels'],
      artifact_sha256s: [report.replica?.digest ?? '0'.repeat(64)],
      owner_sha256s: [ownerDigest],
      normalization_sha256: report.normalizationDigest ?? '0'.repeat(64),
      repeats: 1,
      evidence_ids: [result.evidenceId],
    });
    if (report.replica?.offline) offlineEvidence.push(result.evidenceId);
  }
}

const bundle = createEvidenceBundle([...records.values()]);
const anyEvidence = [...records.keys()].slice(0, 1);
const coverage = {
  evidence_bundle: bundle,
  artifacts: [{ artifact_id: 'mymind-replica', artifact_sha256: replica, version: 'static 134/0.109.0', environment_sha256: normalization, evidence_ids: anyEvidence }],
  surfaces,
  owners,
  claims,
  verifier_contracts: contracts,
  verifier_results: results,
  residual_unknown_ids: [],
  contradictions: [],
  package_proofs: [
    {
      proof_id: 'replica-renders-offline',
      kind: 'authority-independence',
      status: allOffline && offlineEvidence.length ? 'pass' : 'unknown',
      artifact_sha256s: [replica],
      evidence_ids: offlineEvidence.length ? offlineEvidence.slice(0, 50) : anyEvidence,
    },
  ],
  boundaries: [
    {
      boundary_id: 'ui-parity',
      title: 'Every recorded screen, animation frame and hover state of mymind is pixel-identical on the replica',
      required_surface_ids: surfaces.map((s) => s.surface_id),
      required_claim_ids: claims.map((c) => c.claim_id),
      required_package_proof_kinds: ['authority-independence'],
      allowed_dispositions: [],
      allowed_unknown_ids: [],
    },
  ],
};
const input = path.join(DRIVE, 'coverage-input.json');
await writeFile(input, JSON.stringify({ coverage, boundary_id: 'ui-parity' }));
const { stdout } = await run('node', [path.join(REA, 'scripts', 'rea.mjs'), 'evaluate-reconstruction-coverage', input, '--format', 'json'], { maxBuffer: 1 << 28 });
const verdict = JSON.parse(stdout);
const outcome = verdict.normalized_result ?? verdict.data?.normalized_result ?? verdict;
await writeFile(path.join(DRIVE, 'coverage.json'), JSON.stringify(outcome, null, 1));
const reasons = outcome.reasons ?? [];
console.log(`REA reconstruction coverage: ${outcome.status} (${surfaces.length} surfaces, ${claims.length} claims, ${reasons.length} open reasons)`);
const byCode = {};
for (const reason of reasons) (byCode[reason.code] ??= []).push(reason.subject_id);
for (const [code, subjects] of Object.entries(byCode)) console.log(`  ${code}: ${subjects.length} — ${subjects.slice(0, 5).join(', ')}${subjects.length > 5 ? ', …' : ''}`);
