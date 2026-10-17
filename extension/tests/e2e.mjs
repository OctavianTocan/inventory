import assert from 'node:assert/strict';
import { join } from 'node:path';
import { runFlows } from './flows.mjs';
import { prepareExtension, repo } from './harness.mjs';

const extension = prepareExtension();
const facts = await runFlows({
  label: 'e2e',
  extension,
  outDir: join(repo, 'evidence', 'extension', 'shots'),
  setup: async (worker, origin) => {
    await worker.evaluate((host) => chrome.storage.local.set({ host }), origin);
    await new Promise((resolve) => setTimeout(resolve, 800));
  },
});

assert.ok(facts.page, 'the saved page exists in the replica library');
assert.deepEqual(facts.page.tags.map((tag) => tag.name), ['de'], 'the added tag stays and the removed one is gone');
assert.equal(facts.page.note.prose.content[0].content[0].text, 'A note written in the toast.');
console.log('extension e2e passed');
