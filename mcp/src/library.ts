/**
 * The only way this Worker reaches a person's library: the `mymind` Worker's own app API over the service binding,
 * with the app token kept in the OAuth grant. Tavi chose (2026-11-05) to keep the limit on what an agent can reach
 * here rather than in the Rust sign-in code, so every request is checked against `ALLOWED` before it leaves.
 */

/** A request the library refused or could not answer, with a message an agent can act on. */
export class LibraryError extends Error {
  /**
   * @param status HTTP status from the library, or 0 when the request never left this Worker.
   * @param message Plain words for the agent.
   * @param signedOut True when the app token no longer works and the person must connect again.
   */
  constructor(
    readonly status: number,
    message: string,
    readonly signedOut = false,
  ) {
    super(message);
  }
}

const ID = "[A-Za-z0-9]{1,64}";

/** Method and exact path shape of every library request this Worker may make. Nothing else is sent. */
export const ALLOWED: ReadonlyArray<readonly [string, RegExp]> = [
  ["GET", /^\/app\/v1\/cards$/],
  ["GET", new RegExp(`^/app/v1/cards/${ID}$`)],
  ["GET", /^\/app\/v1\/spaces$/],
  ["GET", new RegExp(`^/app/v1/spaces/${ID}/cards$`)],
  ["GET", /^\/app\/v1\/tags$/],
  ["GET", /^\/app\/v1\/search$/],
  ["GET", /^\/app\/v1\/top-of-mind$/],
  ["GET", /^\/media\/[^?#]+$/],
  ["POST", /^\/app\/v1\/push$/],
  ["POST", new RegExp(`^/objects/${ID}/restore$`)],
];

/**
 * Named changes this Worker may push (`POST /app/v1/push`, `docs/app-api.md`). Tavi chose (2026-11-05) that agents
 * read, save and edit; deleting a space, uploads, sharing and account settings stay out.
 */
export const CHANGES = ["create-card", "note", "add-tag", "remove-tag", "add-to-space", "remove-from-space", "create-space", "pin", "trash"] as const;

/** One change for the library's push. */
export interface Change {
  name: (typeof CHANGES)[number];
  args: Record<string, string | boolean>;
}

/** What the library answered for a push, by the change's place in the list (0-based). */
export interface Pushed {
  /** Places of the changes the library turned down (a card or tag that is gone, a blank name). */
  refused: number[];
  /** Tag id each `add-tag` change made, by place. */
  tagIds: Map<number, string>;
  /** Card id the library keeps for each `create-card` change, by place. */
  cardIds: Map<number, string>;
}

/** The push takes at most this many changes at once. */
export const PUSH_LIMIT = 100;

/** @returns 32 random lowercase hex digits: the id shape the push accepts for a new card or space. */
export function newId(): string {
  return [...crypto.getRandomValues(new Uint8Array(16))].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/**
 * Whether a library request is on the allowed list.
 *
 * @param method HTTP method.
 * @param path Path without query, as it will be sent.
 * @returns True only for an exact match of method and path shape.
 */
export function allowed(method: string, path: string): boolean {
  return !path.includes("..") && ALLOWED.some(([m, pattern]) => m === method && pattern.test(path));
}

/** A small client for one person's library. */
export class Library {
  /**
   * @param binding Service binding to the `mymind` Worker.
   * @param origin The `mymind` Worker's public origin, so its own routing sees the usual host.
   * @param appToken The person's app token from the OAuth grant; never logged or returned.
   */
  constructor(
    private readonly binding: Fetcher,
    private readonly origin: string,
    private readonly appToken: string,
  ) {}

  /**
   * GET a JSON answer from the library.
   *
   * @param path Allowed path (see `ALLOWED`).
   * @param query Query parameters; empty values are left out.
   * @returns The parsed JSON body.
   * @throws LibraryError for a refused path, a missing card, a dead token or any other failure.
   */
  async json<T>(path: string, query: Record<string, string | number | undefined> = {}): Promise<T> {
    const response = await this.send("GET", path, query);
    return (await response.json()) as T;
  }

  /**
   * GET bytes (a picture rendition) from the library.
   *
   * @param path An allowed `/media/` path.
   * @returns The body and its content type.
   * @throws LibraryError as for `json`.
   */
  async bytes(path: string): Promise<{ data: ArrayBuffer; type: string }> {
    const response = await this.send("GET", path, {});
    return { data: await response.arrayBuffer(), type: response.headers.get("content-type") ?? "application/octet-stream" };
  }

  /**
   * Apply changes through the library's push, in order.
   *
   * Every push names a new client, so its change ids start at 1 and no earlier push can make the library skip them;
   * concurrent tool calls never share a counter.
   *
   * @param changes 1 to `PUSH_LIMIT` changes, each named in `CHANGES`.
   * @returns Which changes the library turned down and the ids it made.
   * @throws LibraryError for an unknown change name, too many changes, or as for `json`.
   */
  async push(changes: readonly Change[]): Promise<Pushed> {
    if (changes.length === 0 || changes.length > PUSH_LIMIT || changes.some((change) => !CHANGES.includes(change.name))) {
      throw new LibraryError(0, "This request is not one Inventory's MCP server makes.");
    }
    const body = { clientId: `mcp-${newId()}`, changes: changes.map((change, place) => ({ id: place + 1, name: change.name, args: change.args })) };
    const response = await this.send("POST", "/app/v1/push", {}, body);
    const answer = (await response.json()) as { lastChangeId?: number; refused?: number[]; tagIds?: Record<string, string>; cardIds?: Record<string, string> };
    if (answer.lastChangeId === undefined) throw new LibraryError(501, "This Inventory server is too old to take changes from agents.");
    const byPlace = (ids: Record<string, string> = {}) => new Map(Object.entries(ids).map(([id, value]) => [Number(id) - 1, value]));
    return { refused: (answer.refused ?? []).map((id) => id - 1), tagIds: byPlace(answer.tagIds), cardIds: byPlace(answer.cardIds) };
  }

  /**
   * Take a card out of the trash.
   *
   * @param id The card's id.
   * @throws LibraryError as for `json` (404 for a card that does not exist).
   */
  async restore(id: string): Promise<void> {
    await this.send("POST", `/objects/${id}/restore`, {});
  }

  private async send(method: string, path: string, query: Record<string, string | number | undefined>, body?: unknown): Promise<Response> {
    if (!allowed(method, path)) {
      throw new LibraryError(0, "This request is not one Inventory's MCP server makes.");
    }
    const url = new URL(path, this.origin);
    // URL parsing resolves `%2e%2e` segments, so the path is checked again as it will actually be sent.
    if (url.origin !== new URL(this.origin).origin || !allowed(method, url.pathname)) {
      throw new LibraryError(0, "This request is not one Inventory's MCP server makes.");
    }
    for (const [key, value] of Object.entries(query)) {
      if (value !== undefined && value !== "") url.searchParams.set(key, String(value));
    }
    const response = await this.binding.fetch(url.toString(), {
      method,
      headers: { authorization: `Bearer ${this.appToken}`, accept: "application/json", ...(body === undefined ? {} : { "content-type": "application/json" }) },
      body: body === undefined ? undefined : JSON.stringify(body),
      redirect: "manual",
    });
    if (response.ok) return response;
    if (response.status === 401) {
      throw new LibraryError(401, "Inventory signed this connection out. Ask the person to connect Inventory again.", true);
    }
    if (response.status === 404) throw new LibraryError(404, "Not found: the card or space does not exist.");
    if (response.status === 400) {
      const problem = (await response.json().catch(() => ({}))) as { error?: string };
      throw new LibraryError(400, `Inventory could not read that request (${problem.error ?? "bad-request"}).`);
    }
    throw new LibraryError(response.status, `Inventory did not answer (${response.status}). Try again in a moment.`);
  }
}
