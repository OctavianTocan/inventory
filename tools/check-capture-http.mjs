// Run the retired capture HTTP suite through its native Rust assertions.
import { pathToFileURL } from 'node:url';
import { runRustCheck } from './lib/rust-check.mjs';

/**
 * Run native destination, DNS, HTTPS, streaming and shared-budget checks for the retained capture parent.
 * @param {Function} check Parent assertion runner accepting a label and an asynchronous operation.
 * @returns {Promise<void>} Resolves after the parent observes both native suite results.
 */
export async function checkCaptureHttp(check) {
  await check('Rust capture HTTP policy and budget assertions', () => runRustCheck('check_capture_http', 'original_ssrf_dns_streaming_and_shared_budget_assertions'));
  await check('Rust public HTTPS and pinned DNS assertions', () => runRustCheck('check_capture_http', 'public_https_pins_checked_dns_answers_without_relookup'));
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  await checkCaptureHttp(async (_label, run) => run());
}
