// Runs lib/ai.mjs on a few recorded cards and prints what the AI features return.
// Usage: node --env-file-if-exists=.env tools/try-ai.mjs [--cards 5] [--vocab 400] [--query "text"]...
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { analyze, cosine, embed } from '../lib/ai.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MIME = { jpeg: 'image/jpeg', jpg: 'image/jpeg', png: 'image/png', webp: 'image/webp' };

// Cover image on disk for a recorded card, preferring formats Jev and Gemini embeddings accept.
function findImage(html) {
  const found = [...html.matchAll(/\/media\/[^"\s]+/g)]
    .map((match) => decodeURIComponent(match[0].replace(/&amp;/g, '&').split('?')[0]))
    .filter((url) => existsSync(path.join(ROOT, 'capture', url)));
  const pick = found.find((url) => /\.(jpe?g|png)$/i.test(url)) ?? found[0];
  if (!pick) return undefined;
  const mime = MIME[pick.split('.').pop().toLowerCase()];
  return mime && { mime, data: readFileSync(path.join(ROOT, 'capture', pick)), file: path.basename(pick) };
}

// Tag names by usage, merging spellings that differ only in case.
function libraryVocabulary(cards, limit) {
  const counts = new Map();
  for (const card of cards) {
    for (const { name } of card.data.tags ?? []) {
      if (!name || name === 'highlight') continue;
      const entry = counts.get(name.toLowerCase()) ?? { total: 0, spellings: new Map() };
      entry.total++;
      entry.spellings.set(name, (entry.spellings.get(name) ?? 0) + 1);
      counts.set(name.toLowerCase(), entry);
    }
  }
  return [...counts.values()]
    .sort((a, b) => b.total - a.total)
    .slice(0, limit)
    .map((entry) => [...entry.spellings].sort((a, b) => b[1] - a[1])[0][0]);
}

// Evenly spaced picks; about 60% image cards, the first a JPEG/PNG one so Jev's image path runs too.
function pickCards(cards, count) {
  const spread = (list, n) => list.filter((_, i) => i % Math.max(1, Math.floor(list.length / n)) === 0).slice(0, n);
  const withImage = cards.filter((card) => card.image);
  // Many recorded video covers are black frames; the largest file is the likeliest real picture.
  const jpeg = withImage.filter((card) => card.image.mime !== 'image/webp').sort((a, b) => b.image.data.length - a.image.data.length);
  const webp = withImage.filter((card) => card.image.mime === 'image/webp' && card.data.tags?.length);
  const textOnly = cards.filter((card) => !card.image && (card.data.description ?? '').length > 40);
  const imageCount = Math.min(withImage.length, Math.ceil(count * 0.6));
  const images = [...spread(jpeg, Math.min(1, imageCount)), ...spread(webp, Math.max(0, imageCount - 1))];
  return [...images, ...spread(textOnly, count - images.length)];
}

function loadCards() {
  const recorded = JSON.parse(readFileSync(path.join(ROOT, 'capture', 'cards.json'), 'utf8'));
  return recorded.map(({ html, json }) => ({ html, data: JSON.parse(json), image: findImage(html) }));
}

function toItem(card) {
  const { data, image } = card;
  return {
    title: data.title ?? '',
    text: data.description ?? '',
    url: data.source?.url ?? '',
    domain: data.domain ?? '',
    image: image && { mime: image.mime, data: image.data },
  };
}

const norm = (vector) => Math.sqrt(vector.reduce((sum, value) => sum + value * value, 0));
const tagLine = (tag) => `${tag.name} p=${tag.p === null ? 'n/a' : tag.p.toFixed(2)} [${tag.source}]`;

async function main() {
  const { values } = parseArgs({
    options: { cards: { type: 'string', default: '5' }, vocab: { type: 'string', default: '400' }, query: { type: 'string', multiple: true } },
    strict: true,
  });
  const count = Number(values.cards);
  const vocabSize = Number(values.vocab);
  if (!Number.isInteger(count) || count < 1 || !Number.isInteger(vocabSize) || vocabSize < 0) {
    throw new Error('Usage: node tools/try-ai.mjs [--cards N] [--vocab N] [--query "text"]...');
  }

  const cards = loadCards();
  const vocabulary = libraryVocabulary(cards, vocabSize);
  const sample = pickCards(cards, count);
  console.log(`library: ${cards.length} cards, vocabulary of ${vocabulary.length} tags, analyzing ${sample.length}`);

  const analyzed = [];
  for (const card of sample) {
    const item = toItem(card);
    console.log(`\n${item.title || '(untitled)'}`);
    console.log(`  type: ${card.data.type}  domain: ${item.domain || '-'}  image: ${card.image ? `${card.image.file} (${card.image.mime})` : 'none'}`);
    console.log(`  recorded tags: ${(card.data.tags ?? []).map((tag) => tag.name).join(', ') || '-'}`);
    const result = await analyze(item, { vocabulary });
    console.log(`  summary: ${result.summary}`);
    console.log(`  tags: ${result.tags.map(tagLine).join('; ') || '(none)'}`);
    console.log(`  embedding: ${result.embedding.length} dims, norm ${norm(result.embedding).toFixed(4)}`);
    analyzed.push({ card, item, embedding: result.embedding });
  }

  const tagged = analyzed.filter(({ card }) => card.data.tags?.length);
  const queries = values.query?.length ? values.query : tagged.slice(0, 2).map(({ card }) => card.data.tags[0].name);
  for (const query of queries) {
    const vector = await embed({ text: query }, { kind: 'query' });
    console.log(`\nquery "${query}"`);
    analyzed
      .map(({ item, embedding }) => ({ title: item.title, score: cosine(vector, embedding) }))
      .sort((a, b) => b.score - a.score)
      .forEach(({ title, score }) => console.log(`  ${score.toFixed(3)}  ${(title || '(untitled)').slice(0, 70)}`));
  }
}

try {
  await main();
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exitCode = 1;
}
