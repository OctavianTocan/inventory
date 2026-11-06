/**
 * How a tool names one card: its id, or the card's agent link that the app's Share sheet copies
 * (`<server>/c/<id>`, Tavi 2026-11-06: "I give that link to one of the clients that has the MCP connected and then it
 * can immediately look through the video"). A link names a card in the signed-in person's own library only; it
 * reaches nothing for anyone else.
 */
import { z } from "zod";

/** The path a card's agent link starts with, before the card's id. */
export const CARD_LINK_PATH = "/c/";

const BARE_ID = /^[A-Za-z0-9]{1,64}$/;
const LINK = /^https?:\/\/[^\s/?#]+\/c\/([A-Za-z0-9]{1,64})\/?(?:[?#]\S*)?$/;

/**
 * The card id an id or agent link names.
 *
 * @param ref A card id, or a link like `https://mymind.octaviantocan.com/c/<id>`.
 * @returns The id, or undefined when `ref` is neither.
 */
export function cardIdOf(ref: string): string | undefined {
  const text = ref.trim();
  if (BARE_ID.test(text)) return text;
  return LINK.exec(text)?.[1];
}

/**
 * A card's agent link.
 *
 * @param origin The library's public origin, such as `https://mymind.octaviantocan.com`.
 * @param id The card's id.
 * @returns The link the app's Share sheet copies for the same card.
 */
export function cardLink(origin: string, id: string): string {
  return `${origin.replace(/\/+$/, "")}${CARD_LINK_PATH}${id}`;
}

/** A card id or agent link in a tool's input; the tool reads the id with `cardIdOf`. */
export const CARD_REF = z
  .string()
  .max(300)
  .refine((ref) => cardIdOf(ref) !== undefined, "a card id (letters and digits) or a card link like https://mymind.octaviantocan.com/c/<id>");
