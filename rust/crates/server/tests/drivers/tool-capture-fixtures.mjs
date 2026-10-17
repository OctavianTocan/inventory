// Runs tools/capture-fixtures.mjs `startFixtures` on a real listener and takes JSON control commands on stdin.
// Usage: node tool-capture-fixtures.mjs <project root> <media directory>
import { createInterface } from 'node:readline';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const [root, directory] = process.argv.slice(2);
const { fixtureMedia, startFixtures } = await import(pathToFileURL(join(root, 'tools/capture-fixtures.mjs')).href);
const media = await fixtureMedia(directory);
const fixtures = await startFixtures(media, false);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
console.log(JSON.stringify({ base: fixtures.base, pdf: media.pdf.toString('hex'), mp4: sha(media.mp4), png: media.png.toString('hex'), webp: media.webp.toString('hex') }));

for await (const line of createInterface({ input: process.stdin })) {
  const command = JSON.parse(line);
  if (command.control) Object.assign(fixtures.control, command.control);
  if (command.embedding === true) fixtures.control.embedding = async body => ({ body: { echo: body }, status: 202 });
  if (command.embedding === false) delete fixtures.control.embedding;
  if (command.requests) console.log(JSON.stringify({ requests: fixtures.requests }));
  if (command.exit) break;
}
await fixtures.close();
process.exit(0);
