// Forward operational tooling to the Rust binary with inherited streams and exit status.
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { toolCommand } from './server-command.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

/**
 * Run one Rust subcommand with the caller's stdin, stdout, stderr and environment.
 * @param {string} name Rust subcommand.
 * @param {string[]} [arguments_] Arguments to forward, defaulting to the current command line.
 * @returns {Promise<void>} Resolves after exit and sets this process's exit status or signal.
 */
export async function runTool(name, arguments_ = process.argv.slice(2)) {
  const { command, args } = toolCommand(name, arguments_);
  const child = spawn(command, args, { stdio: 'inherit', env: { ...process.env, MYMIND_ROOT: process.env.MYMIND_ROOT ?? ROOT } });
  // Forward a termination signal only while the child is running.
  const forward = (signal) => { if (child.exitCode === null && child.signalCode === null) child.kill(signal); };
  const signals = ['SIGINT', 'SIGTERM', 'SIGHUP'];
  const handlers = signals.map(signal => [signal, () => forward(signal)]);
  for (const [signal, handler] of handlers) process.on(signal, handler);
  let result;
  try {
    result = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('exit', (code, signal) => resolve({ code, signal }));
    });
  } finally {
    for (const [signal, handler] of handlers) process.off(signal, handler);
  }
  if (result.signal) process.kill(process.pid, result.signal);
  else process.exitCode = result.code ?? 1;
}
