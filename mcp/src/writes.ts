/**
 * The tools that change a person's library: save, note, tag, sort, pin, trash and restore. Tavi chose (2026-11-05)
 * that agents read, save and edit; each tool builds its push changes from typed arguments, and only a grant with
 * `thymind.write` may call them.
 */
import type { McpServer, ScopeChallengeHandler } from "@modelcontextprotocol/server";
import { z } from "zod";

import { type Card, fenced } from "./cards";
import { type Change, Library, LibraryError, newId } from "./library";
import { READ_SCOPE, WRITE_SCOPE } from "./pages";
import { CARD_REF, cardIdOf } from "./refs";

/** Most cards one tool call may change. */
export const CARD_LIMIT = 25;

const ID = z.string().regex(/^[A-Za-z0-9]{1,64}$/, "a card or space id: letters and digits");
const IDS = z.array(CARD_REF).min(1).max(CARD_LIMIT).describe(`Card ids from thymind_search, or card links the person gave, 1-${CARD_LIMIT}`);

/** The card ids behind ids or card links, already checked by `CARD_REF`. */
const idsOf = (refs: string[]) => refs.map((ref) => cardIdOf(ref) ?? ref);
const TAG = z.string().trim().min(1).max(100);

// The colours the app offers for a new space (`NewSpaceDialog.kt`, SPACE_COLORS).
const SPACE_COLORS = ["#75ffc0", "#c0c2ce", "#ff8fa4", "#19aad1", "#17c37b", "#ffdcd0", "#166ff4"];

const WRITE = { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: false } as const;

const CHANGED = z.object({ changed: z.array(z.string()), not_changed: z.array(z.string()) });

/** Asks the client to connect again with `thymind.write` when the grant only reads. */
const needsWrite: ScopeChallengeHandler = ({ authInfo }) =>
  authInfo?.scopes.includes(WRITE_SCOPE)
    ? undefined
    : { scopes: [READ_SCOPE, WRITE_SCOPE], errorDescription: "Changing Inventory needs the person to allow changes when connecting" };

/** Lists which cards changed and which the library turned down. */
function report(verb: string, ids: string[], refused: Set<string>) {
  const changed = ids.filter((id) => !refused.has(id));
  const notChanged = ids.filter((id) => refused.has(id));
  const lines = [changed.length ? `${verb}: ${changed.join(", ")}.` : `Nothing was ${verb.toLowerCase()}.`];
  if (notChanged.length) lines.push(`Not changed (the card is gone or the change did not apply): ${notChanged.join(", ")}.`);
  return { structuredContent: { changed, not_changed: notChanged }, content: [{ type: "text" as const, text: lines.join("\n") }] };
}

/** The card ids behind refused changes, given which card each change was for. */
function refusedCards(refused: number[], cardOf: string[]): Set<string> {
  return new Set(refused.map((place) => cardOf[place]).filter((id): id is string => id !== undefined));
}

/**
 * Register the write tools.
 *
 * @param server The request's MCP server.
 * @param library The person's library.
 * @param spaceNames Space names by id, read once per request.
 * @param failure Turns a library failure into a tool error (and signs out a dead connection).
 */
export function registerWrites(
  server: McpServer,
  library: Library,
  spaceNames: () => Promise<Map<string, string>>,
  failure: (error: unknown) => { isError: true; content: Array<{ type: "text"; text: string }> },
): void {
  const run = async <T>(work: () => Promise<T>) => {
    try {
      return await work();
    } catch (error) {
      return failure(error);
    }
  };

  server.registerTool(
    "thymind_save",
    {
      title: "Save to Inventory",
      description:
        "Save a web link or a piece of text as a new card, with an optional note, tags and spaces. Inventory reads a link's page itself. A link already saved is brought to the top and keeps its own note.",
      inputSchema: z.object({
        content: z.string().trim().min(1).max(100_000).describe("A web address (https://...) to save as a link, or text to save as a note"),
        note: z.string().max(10_000).optional().describe("The person's note on the card"),
        tags: z.array(TAG).max(10).default([]).describe("Tags to add"),
        space_ids: z.array(ID).max(10).default([]).describe("Spaces to put it in (ids from thymind_list_spaces)"),
      }),
      outputSchema: z.object({ id: z.string(), already_saved: z.boolean(), not_applied: z.array(z.string()) }),
      annotations: { ...WRITE, idempotentHint: false },
      scopeChallenge: needsWrite,
    },
    ({ content, note, tags, space_ids }) =>
      run(async () => {
        const made = newId();
        const changes: Change[] = [{ name: "create-card", args: { cardId: made, content, source: "mcp", ...(note?.trim() ? { note } : {}) } }];
        for (const name of tags) changes.push({ name: "add-tag", args: { cardId: made, name } });
        for (const spaceId of space_ids) changes.push({ name: "add-to-space", args: { cardId: made, spaceId } });
        const pushed = await library.push(changes);
        if (pushed.refused.includes(0)) return { isError: true, content: [{ type: "text" as const, text: "Inventory could not save that." }] };
        const id = pushed.cardIds.get(0) ?? made;
        const alreadySaved = id !== made;
        const notApplied = pushed.refused.map((place) => (place <= tags.length ? `tag ${tags[place - 1]}` : `space ${space_ids[place - 1 - tags.length]}`));
        const lines = [alreadySaved ? `Already saved; brought it to the top. Card id: ${id}.` : `Saved. Card id: ${id}.`];
        if (notApplied.length) lines.push(`Not applied: ${notApplied.join(", ")}.`);
        return { structuredContent: { id, already_saved: alreadySaved, not_applied: notApplied }, content: [{ type: "text" as const, text: lines.join("\n") }] };
      }),
  );

  server.registerTool(
    "thymind_set_note",
    {
      title: "Write an Inventory card's note",
      description: "Replace the person's note on one card (empty text clears it). Returns the note it replaced, so it can be put back.",
      inputSchema: z.object({ id: CARD_REF.describe("The card's id, or its card link"), note: z.string().max(10_000).describe("The new note; one paragraph per line") }),
      outputSchema: z.object({ id: z.string(), previous_note: z.string() }),
      annotations: { ...WRITE, destructiveHint: true },
      scopeChallenge: needsWrite,
    },
    ({ id: ref, note }) =>
      run(async () => {
        const id = cardIdOf(ref) ?? ref;
        const card = await library.json<Card>(`/app/v1/cards/${id}`);
        const previous = card.note ?? "";
        const pushed = await library.push([{ name: "note", args: { cardId: id, text: note } }]);
        if (pushed.refused.length) return { isError: true, content: [{ type: "text" as const, text: "Inventory did not change that note." }] };
        const said = previous ? `The note it replaced:\n${fenced(previous)}` : "The card had no note before.";
        return { structuredContent: { id, previous_note: previous }, content: [{ type: "text" as const, text: `Note saved on ${id}. ${said}` }] };
      }),
  );

  server.registerTool(
    "thymind_tag",
    {
      title: "Tag Inventory cards",
      description: "Add and remove tags on up to 25 cards. Tag names are matched without regard to case when removing.",
      inputSchema: z.object({
        ids: IDS,
        add: z.array(TAG).max(10).default([]).describe("Tags to add"),
        remove: z.array(TAG).max(10).default([]).describe("Tags to remove"),
      }),
      outputSchema: CHANGED,
      annotations: WRITE,
      scopeChallenge: needsWrite,
    },
    ({ ids: refs, add, remove }) =>
      run(async () => {
        const ids = idsOf(refs);
        if (!add.length && !remove.length) return { isError: true, content: [{ type: "text" as const, text: "Name tags to add or remove." }] };
        const changes: Change[] = [];
        const cardOf: string[] = [];
        const missing = new Set<string>();
        const unwanted = new Set(remove.map((name) => name.toLowerCase()));
        for (const id of ids) {
          for (const name of add) {
            changes.push({ name: "add-tag", args: { cardId: id, name } });
            cardOf.push(id);
          }
          if (unwanted.size) {
            const card = await library.json<Card>(`/app/v1/cards/${id}`).catch((error: unknown) => {
              if (error instanceof LibraryError && error.status === 404) return undefined;
              throw error;
            });
            if (!card) missing.add(id);
            for (const tag of card?.tags ?? []) {
              if (tag.id && unwanted.has(tag.name.toLowerCase())) {
                changes.push({ name: "remove-tag", args: { tagId: tag.id } });
                cardOf.push(id);
              }
            }
          }
        }
        if (!changes.length) return report("Tagged", ids, missing);
        const pushed = await library.push(changes);
        const refused = refusedCards(pushed.refused, cardOf);
        for (const id of missing) refused.add(id);
        return report("Tagged", ids, refused);
      }),
  );

  server.registerTool(
    "thymind_sort",
    {
      title: "Sort Inventory cards into a space",
      description:
        "Put up to 25 cards into a space, or take them out (remove: true). Name a new space with new_space_name to make it first. Cards stay in their other spaces.",
      inputSchema: z.object({
        ids: IDS,
        space_id: ID.optional().describe("An existing space (from thymind_list_spaces)"),
        new_space_name: z.string().trim().min(1).max(100).optional().describe("Make a new space with this name and use it"),
        remove: z.boolean().default(false).describe("Take the cards out of the space instead"),
      }),
      outputSchema: CHANGED.extend({ space_id: z.string() }),
      annotations: WRITE,
      scopeChallenge: needsWrite,
    },
    ({ ids: refs, space_id, new_space_name, remove }) =>
      run(async () => {
        const ids = idsOf(refs);
        if (Boolean(space_id) === Boolean(new_space_name)) {
          return { isError: true, content: [{ type: "text" as const, text: "Give either space_id or new_space_name." }] };
        }
        if (remove && new_space_name) return { isError: true, content: [{ type: "text" as const, text: "A new space has no cards to take out." }] };
        const changes: Change[] = [];
        let spaceId = space_id ?? "";
        if (new_space_name) {
          const existing = [...(await spaceNames())].find(([, name]) => name.toLowerCase() === new_space_name.toLowerCase());
          if (existing) {
            spaceId = existing[0];
          } else {
            spaceId = newId();
            const color = SPACE_COLORS[Math.floor(Math.random() * SPACE_COLORS.length)] ?? SPACE_COLORS[0]!;
            changes.push({ name: "create-space", args: { spaceId, name: new_space_name, color } });
          }
        }
        const first = changes.length;
        for (const cardId of ids) changes.push({ name: remove ? "remove-from-space" : "add-to-space", args: { cardId, spaceId } });
        const pushed = await library.push(changes);
        if (first && pushed.refused.includes(0)) return { isError: true, content: [{ type: "text" as const, text: "Inventory could not make that space." }] };
        const refused = refusedCards(pushed.refused, [...Array<string>(first).fill(""), ...ids]);
        refused.delete("");
        const answer = report(remove ? `Taken out of the space` : `Put in the space`, ids, refused);
        return { ...answer, structuredContent: { ...answer.structuredContent, space_id: spaceId } };
      }),
  );

  server.registerTool(
    "thymind_pin",
    {
      title: "Pin Inventory cards",
      description: "Pin up to 25 cards to Top of Mind, or unpin them (pinned: false).",
      inputSchema: z.object({ ids: IDS, pinned: z.boolean().default(true).describe("true to pin, false to unpin") }),
      outputSchema: CHANGED,
      annotations: WRITE,
      scopeChallenge: needsWrite,
    },
    ({ ids: refs, pinned }) =>
      run(async () => {
        const ids = idsOf(refs);
        const pushed = await library.push(ids.map((cardId) => ({ name: "pin" as const, args: { cardId, pinned } })));
        return report(pinned ? "Pinned" : "Unpinned", ids, refusedCards(pushed.refused, ids));
      }),
  );

  server.registerTool(
    "thymind_trash",
    {
      title: "Move Inventory cards to the trash",
      description:
        "Move up to 25 cards to the trash. The person can still see them in the trash, and thymind_restore takes them out again. Confirm with the person before trashing cards they did not name.",
      inputSchema: z.object({ ids: IDS }),
      outputSchema: CHANGED,
      annotations: { ...WRITE, destructiveHint: true },
      scopeChallenge: needsWrite,
    },
    ({ ids: refs }) =>
      run(async () => {
        const ids = idsOf(refs);
        const pushed = await library.push(ids.map((cardId) => ({ name: "trash" as const, args: { cardId } })));
        return report("Moved to the trash", ids, refusedCards(pushed.refused, ids));
      }),
  );

  server.registerTool(
    "thymind_restore",
    {
      title: "Restore Inventory cards from the trash",
      description: "Take up to 25 cards out of the trash.",
      inputSchema: z.object({ ids: IDS }),
      outputSchema: CHANGED,
      annotations: WRITE,
      scopeChallenge: needsWrite,
    },
    ({ ids: refs }) =>
      run(async () => {
        const ids = idsOf(refs);
        const refused = new Set<string>();
        for (const id of ids) {
          await library.restore(id).catch((error: unknown) => {
            if (error instanceof LibraryError && error.status === 404) return refused.add(id);
            throw error;
          });
        }
        return report("Restored", ids, refused);
      }),
  );
}
