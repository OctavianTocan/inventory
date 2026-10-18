// Start the Rust replica.
import { runTool } from './run-tool.mjs';

if (process.argv.includes('--help')) {
  console.log('Usage: npm run serve\nStarts mymind serve. MYMIND_SERVER overrides rust/target/release/mymind; OFFLINE=1 disables external fetches.');
  process.exit(0);
}

await runTool('serve');
