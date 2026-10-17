// Run isolated Rust regression assertions from retained developer-tool entry points.
import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const RUST = fileURLToPath(new URL('../../rust/', import.meta.url));

/**
 * Execute one exact native regression test through offline Cargo with bounded build/test parallelism.
 * @param {string} target Cargo integration target in the mymind package.
 * @param {string} test Exact Rust test function name in that target.
 * @returns {Promise<void>} Resolves only when every assertion in the selected test passes.
 * @throws {Error} If the target/name is invalid, Cargo cannot start, or the regression test fails.
 */
export async function runRustCheck(target, test) {
  if (!/^[a-z_]+$/.test(target) || !/^[a-z_]+$/.test(test)) throw new Error('Rust checks require an integration target and exact test name');
  const temporary = join(RUST, 'target/stage-b/tmp');
  await mkdir(temporary, { recursive: true });
  const env = { PATH: process.env.PATH, HOME: process.env.HOME, TMPDIR: temporary, STORE_LOAD_ENV: '0', MYMIND_ENV_FILE: join(temporary, 'absent-env') };
  for (const name of ['CARGO_HOME', 'RUSTUP_HOME', 'RUSTUP_TOOLCHAIN', 'MYMIND_NODE_SHIM_LOG']) {
    if (process.env[name] !== undefined) env[name] = process.env[name];
  }
  const child = spawn('nice', ['-n', '19', 'cargo', 'test', '--offline', '-j', '2', '-p', 'mymind', '--test', target, test, '--', '--exact', '--test-threads=2'], {
    cwd: RUST, stdio: 'inherit', env,
  });
  await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`Rust ${target}/${test} failed (${signal ?? code})`));
    });
  });
}
