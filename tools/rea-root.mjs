import { existsSync, readdirSync, readFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const VERSION = '6.4.0';

/**
 * Finds the rea-agents package that `npx -y rea-agents@6.4.0` installed, whether or not npm launched this process.
 * @returns {string} The package directory.
 * @throws {Error} When no npx cache holds that version.
 */
export function reaRoot() {
  const caches = [process.env.npm_config_cache, path.join(os.homedir(), '.npm'), path.join(os.tmpdir(), 'npm-cache')];
  for (const cache of caches.filter(Boolean)) {
    const npx = path.join(cache, '_npx');
    for (const dir of existsSync(npx) ? readdirSync(npx) : []) {
      const root = path.join(npx, dir, 'node_modules', 'rea-agents');
      const manifest = path.join(root, 'package.json');
      if (existsSync(manifest) && JSON.parse(readFileSync(manifest, 'utf8')).version === VERSION) return root;
    }
  }
  throw new Error(`rea-agents ${VERSION} not found; run: npx -y rea-agents@${VERSION} --version`);
}
