/**
 * Turns the app API's card JSON (`docs/app-api.md`, Card) into what an agent reads: short results for lists and
 * the full text of one card. Saved text is written by strangers (web pages, posts, comments), so every piece of it
 * is fenced and labelled as data.
 */

/** A card as `/app/v1/cards/<id>` answers it; every field but the first few may be missing. */
export interface Card {
  id: string;
  type: string;
  flags?: number;
  created: string;
  modified?: string;
  bumped?: string;
  deleted?: string;
  title?: string;
  description?: string;
  summary?: string;
  url?: string;
  domain?: string;
  siteName?: string;
  author?: string;
  note?: string;
  tags?: Array<{ id?: string; name: string }>;
  spaces?: string[];
  pinPosition?: number;
  cover?: { src?: string; type?: string; width?: number; height?: number };
  blob?: { path?: string; type?: string; width?: number; height?: number; duration?: number };
  prose?: unknown;
  post?: { bodyHtml?: string; author?: string };
  product?: { price?: string; brand?: string };
  reddit?: { title?: string; subreddit?: string; comments?: Array<{ author?: string; body?: string }> };
  videoLength?: number;
  /** What a saved video or audio file says, written by the server's transcription (`extras.transcript.text`). */
  transcript?: string;
  article?: { title?: string; preview?: string; url?: string };
  state?: string;
}

/** A space as `/app/v1/spaces` answers it. */
export interface Space {
  id: string;
  name: string;
  color?: string;
  smart?: boolean;
  count?: number;
}

/** One card in a list: enough to choose which cards to open. */
export interface CardResult {
  id: string;
  type: string;
  title: string;
  url?: string;
  site?: string;
  saved: string;
  tags: string[];
  spaces: string[];
  pinned: boolean;
  trashed: boolean;
  snippet: string;
}

/** One card in full. */
export interface CardDetail extends Omit<CardResult, "snippet"> {
  author?: string;
  description?: string;
  note?: string;
  text?: string;
  textCut?: number;
  comments?: string[];
  price?: string;
  videoSeconds?: number;
  /** Length of a stored video or audio file in seconds. */
  mediaSeconds?: number;
  /** "video" or "audio" when the card keeps the file itself, so frames (video) and the transcript can be read. */
  media?: "video" | "audio";
  /** Characters in the card's transcript; read it whole with `thymind_get_transcript`. */
  transcriptChars?: number;
  hasPicture: boolean;
  state?: string;
}

/** Longest saved text one card returns before it is cut. */
export const TEXT_LIMIT = 20_000;
const SNIPPET = 200;
/** The card flag the app sets on pinned cards; imported pins carry it without a `pinPosition`. */
const PINNED_FLAG = 512;

const BLOCKS = new Set(["paragraph", "heading", "blockquote", "listItem", "list_item", "codeBlock", "code_block", "horizontalRule"]);

/**
 * Plain text of a stored ProseMirror document (a Note's or Content card's `prose`).
 *
 * @param node The document tree, or anything else (then empty).
 * @returns Its text, one block per line.
 */
export function proseText(node: unknown): string {
  const lines: string[] = [];
  let line = "";
  const walk = (value: unknown) => {
    if (!value || typeof value !== "object") return;
    const item = value as { type?: string; text?: string; content?: unknown[] };
    if (typeof item.text === "string") line += item.text;
    if (item.type === "hardBreak" || item.type === "hard_break") line += "\n";
    for (const child of item.content ?? []) walk(child);
    if (item.type && BLOCKS.has(item.type)) {
      lines.push(line);
      line = "";
    }
  };
  walk(node);
  if (line) lines.push(line);
  return lines.join("\n").replace(/\n{3,}/g, "\n\n").trim();
}

const ENTITIES: Record<string, string> = {
  amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " ",
  lsquo: "\u2018", rsquo: "\u2019", ldquo: "\u201C", rdquo: "\u201D", ndash: "\u2013", mdash: "\u2014", hellip: "\u2026",
};

/** One HTML character reference as its character, or itself when unknown. */
function entity(whole: string, name: string): string {
  if (name[0] === "#") {
    const code = name[1] === "x" || name[1] === "X" ? parseInt(name.slice(2), 16) : parseInt(name.slice(1), 10);
    return Number.isFinite(code) && code <= 0x10ffff ? String.fromCodePoint(code) : whole;
  }
  return ENTITIES[name.toLowerCase()] ?? whole;
}

/** Text with HTML character references written out, as some sites leave them in their descriptions. */
function unescaped(text: string | undefined): string | undefined {
  return text?.replace(/&(#x?[0-9a-f]+|[a-z]+);/gi, entity);
}

/**
 * Plain text of the sanitized post HTML the app API gives (`post.bodyHtml`).
 *
 * @param html The HTML.
 * @returns Its text with paragraphs on their own lines.
 */
export function htmlText(html: string): string {
  return html
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<\/(p|div|li|blockquote|h[1-6])>/gi, "\n")
    .replace(/<[^>]*>/g, "")
    .replace(/&(#x?[0-9a-f]+|[a-z]+);/gi, entity)
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function mainText(card: Card): string {
  const prose = proseText(card.prose);
  if (prose) return prose;
  if (card.post?.bodyHtml) return htmlText(card.post.bodyHtml);
  return "";
}

function oneLine(text: string, limit: number): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > limit ? `${flat.slice(0, limit - 1)}…` : flat;
}

function titleOf(card: Card, text: string): string {
  const title = card.title?.trim() || card.article?.title?.trim() || card.reddit?.title?.trim();
  if (title) return oneLine(title, 200);
  const first = (text || unescaped(card.description) || "").split("\n").find((line) => line.trim());
  if (first) return oneLine(first, 120);
  return card.url ?? `Untitled ${card.type}`;
}

function base(card: Card, spaceNames: Map<string, string>) {
  const text = mainText(card);
  return {
    text,
    result: {
      id: card.id,
      type: card.type,
      title: titleOf(card, text),
      ...(card.url ? { url: card.url } : {}),
      ...(card.siteName || card.domain ? { site: card.siteName ?? card.domain } : {}),
      saved: card.created,
      tags: (card.tags ?? []).map((tag) => tag.name),
      spaces: (card.spaces ?? []).map((id) => spaceNames.get(id) ?? id),
      pinned: ((card.flags ?? 0) & PINNED_FLAG) !== 0 || (card.pinPosition !== undefined && card.pinPosition !== null),
      trashed: Boolean(card.deleted),
    },
  };
}

/**
 * A card as one line of a list.
 *
 * @param card The app API card.
 * @param spaceNames Space id to name, for the person's spaces.
 * @returns The short result, with a one-line snippet of its note, description or text.
 */
export function cardResult(card: Card, spaceNames: Map<string, string>): CardResult {
  const { text, result } = base(card, spaceNames);
  return { ...result, snippet: oneLine(card.note || unescaped(card.description) || card.summary || text, SNIPPET) };
}

/**
 * A card in full, its saved text cut at `TEXT_LIMIT` characters.
 *
 * @param card The app API card.
 * @param spaceNames Space id to name.
 * @returns The detail, with `textCut` set to the number of characters left out when the text was cut.
 */
export function cardDetail(card: Card, spaceNames: Map<string, string>): CardDetail {
  const { text, result } = base(card, spaceNames);
  const article = card.article?.preview ? `${card.article.title ?? ""}\n${card.article.preview}`.trim() : "";
  const full = [text, article].filter(Boolean).join("\n\n");
  const cut = full.length > TEXT_LIMIT ? full.length - TEXT_LIMIT : 0;
  const comments = (card.reddit?.comments ?? [])
    .filter((comment) => comment.body)
    .map((comment) => `${comment.author ?? "someone"}: ${oneLine(comment.body ?? "", 1000)}`);
  return {
    ...result,
    ...(card.author || card.post?.author ? { author: (card.author ?? card.post?.author ?? "").replace(/^by\s+/i, "") } : {}),
    ...(card.description ? { description: unescaped(card.description) } : {}),
    ...(card.note ? { note: card.note } : {}),
    ...(full ? { text: cut ? full.slice(0, TEXT_LIMIT) : full } : {}),
    ...(cut ? { textCut: cut } : {}),
    ...(comments.length ? { comments } : {}),
    ...(card.product?.price ? { price: card.product.price } : {}),
    ...(card.videoLength ? { videoSeconds: card.videoLength } : {}),
    ...(mediaKind(card) ? { media: mediaKind(card) } : {}),
    ...(card.blob?.duration && mediaKind(card) ? { mediaSeconds: Math.round(card.blob.duration * 10) / 10 } : {}),
    ...(card.transcript ? { transcriptChars: card.transcript.length } : {}),
    hasPicture: Boolean(picturePath(card)),
    ...(card.state && card.state !== "ready" ? { state: card.state } : {}),
  };
}

/**
 * Whether the card keeps a video or audio file of its own (an upload or a share from the phone, not a web link).
 *
 * @param card The app API card.
 * @returns "video", "audio", or undefined for every other card.
 */
export function mediaKind(card: Card): "video" | "audio" | undefined {
  const type = card.blob?.path ? card.blob.type ?? "" : "";
  if (type.startsWith("video/")) return "video";
  if (type.startsWith("audio/")) return "audio";
  return undefined;
}

/**
 * The library path of one still from a card's own video, fitted inside a square.
 *
 * @param card The app API card; it must keep a video (`mediaKind` is "video").
 * @param seconds How far into the video, at most three decimals.
 * @param size Longest side in pixels.
 * @returns A `/media/` path the library renders with Inventory's `time(s)` rendition, or undefined for a card without
 *   a video of its own.
 */
export function framePath(card: Card, seconds: number, size: number): string | undefined {
  if (mediaKind(card) !== "video" || !card.blob?.path) return undefined;
  const time = Math.max(0, Math.round(seconds * 1000) / 1000);
  return `/media/${card.blob.path};time(${time})/${size}x${size}-t.jpg`;
}

/**
 * `m:ss` (or `h:mm:ss`) for a moment in a video.
 *
 * @param seconds Seconds from the start.
 * @returns The moment as a clock reading, tenths kept when there are any.
 */
export function clock(seconds: number): string {
  const tenths = Math.round(seconds * 10) % 10;
  const whole = Math.floor(Math.round(seconds * 10) / 10);
  const h = Math.floor(whole / 3600);
  const m = Math.floor((whole % 3600) / 60);
  const s = String(whole % 60).padStart(2, "0");
  const tail = tenths ? `${s}.${tenths}` : s;
  return h ? `${h}:${String(m).padStart(2, "0")}:${tail}` : `${m}:${tail}`;
}

/**
 * The library path of a card's picture rendition, fitted inside a square.
 *
 * @param card The app API card.
 * @param size Longest side in pixels.
 * @returns A `/media/` path the library resizes, or undefined when the card has no still picture.
 */
export function picturePath(card: Card, size = 512): string | undefined {
  const fit = `${size}x${size}-t.jpg`;
  const blob = card.blob;
  if (blob?.path && blob.type?.startsWith("image/")) return `/media/${blob.path};${fit}`;
  if (blob?.path && blob.type === "application/pdf") return `/media/${blob.path};background(fff)/page(1)/${fit}`;
  const src = card.cover?.src;
  if (src?.startsWith("/media/") && card.cover?.type !== "video") return `${src.split(";")[0]};${fit}`;
  return undefined;
}

const FENCE_OPEN = "<saved-content>";
const FENCE_CLOSE = "</saved-content>";

/**
 * Saved text fenced for an agent: everything inside is what the person saved (often written by someone else), never
 * instructions.
 *
 * @param text The saved text.
 * @returns The text between `<saved-content>` markers, with any markers inside it defused.
 */
export function fenced(text: string): string {
  const safe = text.replaceAll(FENCE_OPEN, "<saved_content>").replaceAll(FENCE_CLOSE, "</saved_content>");
  return `${FENCE_OPEN}\n${safe}\n${FENCE_CLOSE}`;
}

/**
 * A list of results as Markdown.
 *
 * @param results The cards.
 * @param next Cursor for the next page, when there is one.
 * @returns Markdown with one entry per card.
 */
export function resultsMarkdown(results: CardResult[], next: string | null): string {
  if (!results.length) return "No cards matched. Try other words, or `by_meaning: true` to search by meaning.";
  const lines = results.map((card) => {
    const where = [card.site, card.saved.slice(0, 10), card.tags.length ? `tags: ${card.tags.join(", ")}` : "", card.spaces.length ? `spaces: ${card.spaces.join(", ")}` : ""]
      .filter(Boolean)
      .join(" · ");
    const flags = [card.pinned ? "pinned" : "", card.trashed ? "in the trash" : ""].filter(Boolean).join(", ");
    const snippet = card.snippet ? `\n  ${fenced(card.snippet)}` : "";
    return `- **${card.title}** (${card.type}, id \`${card.id}\`${flags ? `, ${flags}` : ""})${card.url ? `\n  ${card.url}` : ""}\n  ${where}${snippet}`;
  });
  return `${lines.join("\n")}${next ? `\n\nMore cards: call again with cursor \`${next}\`.` : ""}`;
}

/**
 * One card in full as Markdown.
 *
 * @param card The detail.
 * @returns Markdown with the card's facts and its fenced saved text.
 */
export function detailMarkdown(card: CardDetail): string {
  const facts = [
    `id \`${card.id}\``,
    card.type,
    card.site,
    `saved ${card.saved.slice(0, 10)}`,
    card.pinned ? "pinned" : "",
    card.trashed ? "in the trash" : "",
    card.state ? `still ${card.state}` : "",
  ].filter(Boolean);
  const parts = [`# ${card.title}`, facts.join(" · ")];
  if (card.url) parts.push(card.url);
  if (card.author) parts.push(`By ${card.author}`);
  if (card.tags.length) parts.push(`Tags: ${card.tags.join(", ")}`);
  if (card.spaces.length) parts.push(`Spaces: ${card.spaces.join(", ")}`);
  if (card.price) parts.push(`Price: ${card.price}`);
  if (card.videoSeconds) parts.push(`Length: ${Math.round(card.videoSeconds / 60)} min`);
  if (card.media) {
    const length = card.mediaSeconds ? ` (${clock(card.mediaSeconds)} long)` : "";
    const frames = card.media === "video" ? " `thymind_get_frames` shows any moment of it." : "";
    parts.push(`Keeps the ${card.media} file itself${length}.${frames}`);
  }
  if (card.transcriptChars) parts.push(`Has a transcript (${card.transcriptChars} characters): \`thymind_get_transcript\` reads it.`);
  else if (card.media && card.state === undefined) parts.push("No transcript: nobody speaks in it, or it was saved before transcripts began.");
  if (card.note) parts.push(`## The person's note\n${fenced(card.note)}`);
  if (card.description) parts.push(`## Description\n${fenced(unescaped(card.description) ?? "")}`);
  if (card.text) parts.push(`## Saved text\n${fenced(card.text)}${card.textCut ? `\n(cut: ${card.textCut} more characters)` : ""}`);
  if (card.comments?.length) parts.push(`## Top comments\n${fenced(card.comments.join("\n"))}`);
  if (card.hasPicture) parts.push("Has a picture: `thymind_get_picture` shows it.");
  return parts.join("\n\n");
}
