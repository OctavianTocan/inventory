// tools/search-recordings.mjs's compareRecordings over a copy of a database, printed the way the native subcommand does.
// Usage: node tool-search-recordings.mjs <database copy>   (MYMIND_ROOT names the project root)
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const root = process.env.MYMIND_ROOT;
const { compareRecordings } = await import(pathToFileURL(path.join(root, 'tools/search-recordings.mjs')).href);
const { openStore } = await import(pathToFileURL(path.join(root, 'lib/store.mjs')).href);
const store = openStore(process.argv[2]);
try {
  console.log(JSON.stringify(await compareRecordings(store, root), null, 2));
} finally {
  store.close();
}
