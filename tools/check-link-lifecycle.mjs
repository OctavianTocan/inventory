// Run the retired recorded link lifecycle suite through its native Rust assertions.
import { pathToFileURL } from 'node:url';
import { runRustCheck } from './lib/rust-check.mjs';

/**
 * Run every original recorded link lifecycle assertion against isolated native fixtures.
 * @param {object} context Parent checker context.
 * @param {Function} context.check Assertion runner accepting a label and an asynchronous operation.
 * @returns {Promise<void>} Resolves after the parent runner observes the native regression result.
 */
export async function checkLinkCapture({ check }) {
  await check('Rust recorded link lifecycle assertions', () => runRustCheck('check_capture', 'original_recorded_link_lifecycle_assertions'));
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  await runRustCheck('check_capture', 'original_recorded_link_lifecycle_assertions');
}
