/**
 * The whole path an AI app takes, run against the Worker with fake bindings and a fake Google: discovery,
 * registration, consent, Google, the library's own sign-in, tokens, then MCP tool calls over the real MCP client.
 */
import { Client, StreamableHTTPClientTransport } from "@modelcontextprotocol/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import worker from "../src/index";
import type { Env } from "../src/signin";
import { CARDS, MemoryKV, fakeLibrary } from "./fakes";

const ORIGIN = "https://mymind.example";
const REDIRECT = "https://client.example/callback";
const VERIFIER = "v".repeat(64);
const CIMD = "https://client.example/oauth/client.json";

async function s256(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return Buffer.from(digest).toString("base64url");
}

function setup(options: { refuse?: boolean; owner?: string } = {}) {
  const library = fakeLibrary("app-token-1", options);
  const waits: Promise<unknown>[] = [];
  const env = {
    OAUTH_KV: new MemoryKV(),
    LIBRARY_API: library.binding,
    PUBLIC_URL: ORIGIN,
    GOOGLE_CLIENT_ID: "google-client",
    GOOGLE_CLIENT_SECRET: "google-secret",
    GOOGLE_AUTH_URL: "https://google.example/auth",
    GOOGLE_TOKEN_URL: "https://google.example/token",
    ...(options.owner === undefined ? {} : { OWNER_EMAIL: options.owner }),
  } as unknown as Env;
  const ctx = { waitUntil: (promise: Promise<unknown>) => waits.push(promise), passThroughOnException() {}, props: {} } as unknown as ExecutionContext;
  const call = (input: string | Request, init?: RequestInit) => worker.fetch(new Request(typeof input === "string" ? new URL(input, ORIGIN) : input, init), env, ctx);
  return { env, library, call, waits };
}

function cookies(response: Response): string {
  return response.headers.getSetCookie().map((cookie) => cookie.split(";")[0]).join("; ");
}

/** Runs the browser part of connecting and returns the client's tokens. */
async function connect(call: ReturnType<typeof setup>["call"], write = true, cimd = false) {
  let client_id = CIMD;
  if (!cimd) {
    const registered = await call("/mcp-auth/register", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ redirect_uris: [REDIRECT], client_name: "Test <Agent>", token_endpoint_auth_method: "none" }),
    });
    expect(registered.status).toBe(201);
    ({ client_id } = (await registered.json()) as { client_id: string });
  }

  const authorize = new URL("/mcp-auth/authorize", ORIGIN);
  authorize.search = new URLSearchParams({
    response_type: "code",
    client_id,
    redirect_uri: REDIRECT,
    scope: "thymind.read",
    state: "client-state",
    code_challenge: await s256(VERIFIER),
    code_challenge_method: "S256",
    resource: `${ORIGIN}/mcp`,
  }).toString();
  const consent = await call(authorize.toString());
  expect(consent.status).toBe(200);
  expect(consent.headers.get("x-frame-options")).toBe("DENY");
  const page = await consent.text();
  expect(page).toContain(cimd ? "Connect ChatGPT to Inventory?" : "Connect Test &#60;Agent&#62; to Inventory?");
  expect(page).toContain(cimd ? "Published by <strong>client.example</strong>" : "This app named itself");
  const handle = /name="handle" value="([^"]+)"/.exec(page)![1]!;

  const form = new URLSearchParams({ handle, decision: "approve", ...(write ? { write: "1" } : {}) });
  const approved = await call(authorize.toString(), {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded", cookie: cookies(consent) },
    body: form,
  });
  expect(approved.status).toBe(302);
  const google = new URL(approved.headers.get("location")!);
  expect(google.origin + google.pathname).toBe("https://google.example/auth");
  expect(google.searchParams.get("redirect_uri")).toBe(`${ORIGIN}/mcp-auth/callback`);
  expect(google.searchParams.get("nonce")).toMatch(/^nonce-/);
  expect(google.searchParams.get("code_challenge_method")).toBe("S256");

  const callback = new URL("/mcp-auth/callback", ORIGIN);
  callback.search = new URLSearchParams({ code: "google-code", state: google.searchParams.get("state")! }).toString();
  const back = await call(callback.toString(), { headers: { cookie: [cookies(consent), cookies(approved)].filter(Boolean).join("; ") } });
  return { client_id, back };
}

async function tokens(call: ReturnType<typeof setup>["call"], client_id: string, back: Response) {
  expect(back.status).toBe(302);
  const to = new URL(back.headers.get("location")!);
  expect(to.origin + to.pathname).toBe(REDIRECT);
  expect(to.searchParams.get("state")).toBe("client-state");
  expect(to.searchParams.get("iss")).toBe(ORIGIN);
  const exchanged = await call("/mcp-auth/token", {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      grant_type: "authorization_code",
      code: to.searchParams.get("code")!,
      redirect_uri: REDIRECT,
      client_id,
      code_verifier: VERIFIER,
      resource: `${ORIGIN}/mcp`,
    }),
  });
  expect(exchanged.status).toBe(200);
  return (await exchanged.json()) as { access_token: string; scope: string };
}

async function mcp(call: ReturnType<typeof setup>["call"], accessToken: string, modern = false) {
  const client = new Client({ name: "test", version: "1.0.0" }, modern ? { versionNegotiation: { mode: { pin: "2026-07-28" } } } : {});
  const transport = new StreamableHTTPClientTransport(new URL("/mcp", ORIGIN), {
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      headers.set("authorization", `Bearer ${accessToken}`);
      return call(String(input instanceof Request ? input.url : input), { ...init, headers });
    },
  });
  await client.connect(transport);
  return client;
}

describe("connecting an AI app", () => {
  beforeEach(() => {
    vi.stubGlobal("Cloudflare", { compatibilityFlags: { global_fetch_strictly_public: true } });
    vi.stubGlobal("fetch", async (input: string | Request, init?: RequestInit) => {
      const request = new Request(input, init);
      if (request.url === CIMD) {
        return Response.json({ client_id: CIMD, client_name: "ChatGPT", redirect_uris: [REDIRECT], token_endpoint_auth_method: "none" });
      }
      if (request.url !== "https://google.example/token") throw new Error(`unexpected fetch ${request.url}`);
      const form = new URLSearchParams(await request.text());
      expect(form.get("client_secret")).toBe("google-secret");
      expect(form.get("code_verifier")).toBeTruthy();
      return Response.json({ id_token: "google-id-token", access_token: "google-access" });
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  it("points a signed-out client at the sign-in metadata", async () => {
    const { call } = setup();
    const response = await call("/mcp", { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
    expect(response.status).toBe(401);
    expect(response.headers.get("www-authenticate")).toContain(`${ORIGIN}/.well-known/oauth-protected-resource/mcp`);
    const resource = (await (await call("/.well-known/oauth-protected-resource/mcp")).json()) as { resource: string; scopes_supported: string[] };
    expect(resource.resource).toBe(`${ORIGIN}/mcp`);
    expect(resource.scopes_supported).toEqual(["thymind.read"]);
    const server = (await (await call("/.well-known/oauth-authorization-server")).json()) as Record<string, unknown>;
    expect(server.authorization_endpoint).toBe(`${ORIGIN}/mcp-auth/authorize`);
    expect(server.client_id_metadata_document_supported).toBe(true);
    expect(server.authorization_response_iss_parameter_supported).toBe(true);
  });

  it("signs in through the library's own Google check and reads cards", async () => {
    const { call, library } = setup();
    const { client_id, back } = await connect(call);
    const { access_token, scope } = await tokens(call, client_id, back);
    expect(scope.split(" ").sort()).toEqual(["thymind.read", "thymind.write"]);

    const signIn = library.calls.find((found) => found.path === "/app/v1/auth/google")!;
    expect(signIn.body).toMatchObject({ idToken: "google-id-token", nonce: "nonce-1", device: "MCP: Test <Agent>" });

    const client = await mcp(call, access_token);
    const { tools } = await client.listTools();
    const reading = ["thymind_search", "thymind_get_card", "thymind_get_picture", "thymind_get_transcript", "thymind_get_frames", "thymind_list_spaces", "thymind_list_tags"];
    const writing = ["thymind_save", "thymind_set_note", "thymind_tag", "thymind_sort", "thymind_pin", "thymind_trash", "thymind_restore"];
    expect(tools.map((tool) => tool.name)).toEqual([...reading, ...writing]);
    expect(tools.filter((tool) => tool.annotations?.readOnlyHint).map((tool) => tool.name)).toEqual(reading);
    expect(tools.filter((tool) => tool.annotations?.destructiveHint).map((tool) => tool.name)).toEqual(["thymind_set_note", "thymind_trash"]);

    // MCP Apps hosts show search results in the card grid, which needs no network of its own.
    const searchTool = tools.find((tool) => tool.name === "thymind_search")!;
    expect(searchTool._meta).toMatchObject({ ui: { resourceUri: "ui://thymind/cards.html" } });
    const view = await client.readResource({ uri: "ui://thymind/cards.html" });
    const page = view.contents[0] as unknown as { mimeType: string; text: string; _meta: { ui: { csp: Record<string, unknown> } } };
    expect(page.mimeType).toBe("text/html;profile=mcp-app");
    expect(page.text).toContain("thymind_get_picture");
    expect(page.text).not.toContain("innerHTML");
    expect(page._meta.ui.csp).toEqual({});

    const search = await client.callTool({ name: "thymind_search", arguments: { query: "pasta" } });
    expect(search.isError).toBeFalsy();
    expect(search.structuredContent).toMatchObject({ count: 2, has_more: true, next_cursor: "cursor-2" });
    const text = (search.content as Array<{ text: string }>)[0]!.text;
    expect(text).toContain("**Pasta with lemon**");
    expect(text).toContain("<saved-content>");
    const searched = library.calls.find((found) => found.path === "/app/v1/search")!;
    expect(new URLSearchParams(searched.search).get("semantic")).toBe("1");
    expect(searched.authorization).toBe("Bearer app-token-1");

    const card = await client.callTool({ name: "thymind_get_card", arguments: { id: CARDS.link.id } });
    expect(card.structuredContent).toMatchObject({ title: "How to make bread", note: "try this weekend", pinned: true, hasPicture: true });

    const picture = await client.callTool({ name: "thymind_get_picture", arguments: { id: CARDS.link.id, size: 256 } });
    expect(picture.content).toEqual([{ type: "image", data: "/9j/2Q==", mimeType: "image/jpeg" }]);
    expect(library.calls.some((found) => found.path === "/media/abc/shot.png;256x256-t.jpg")).toBe(true);

    // A card link from the app's Share sheet opens the same card, and the card says where its link is.
    const byLink = await client.callTool({ name: "thymind_get_card", arguments: { id: `https://mymind.octaviantocan.com/c/${CARDS.video.id}` } });
    expect(byLink.structuredContent).toMatchObject({ id: CARDS.video.id, media: "video", mediaSeconds: 60, transcriptChars: CARDS.video.transcript.length, link: `https://mymind.example/c/${CARDS.video.id}` });
    expect((byLink.content as Array<{ text: string }>)[0]!.text).toContain("thymind_get_frames");
    const badLink = await client.callTool({ name: "thymind_get_card", arguments: { id: "https://example.org/bread" } });
    expect(badLink.isError).toBe(true);

    const transcript = await client.callTool({ name: "thymind_get_transcript", arguments: { id: CARDS.video.id } });
    expect(transcript.structuredContent).toMatchObject({ chars: CARDS.video.transcript.length, offset: 0, next_offset: 50_000 });
    const rest = await client.callTool({ name: "thymind_get_transcript", arguments: { id: CARDS.video.id, offset: 50_000 } });
    const restData = rest.structuredContent as { text: string; next_offset: number | null };
    expect(restData.next_offset).toBeNull();
    expect((transcript.structuredContent as { text: string }).text + restData.text).toBe(CARDS.video.transcript);
    expect((await client.callTool({ name: "thymind_get_transcript", arguments: { id: CARDS.link.id } })).isError).toBe(true);

    const frames = await client.callTool({ name: "thymind_get_frames", arguments: { id: CARDS.video.id, at: [12.5, 90], size: 256 } });
    const parts = frames.content as Array<{ type: string; text?: string }>;
    expect(parts.map((part) => part.type)).toEqual(["text", "text", "image", "text", "image"]);
    expect(parts[1]!.text).toBe("Frame at 0:12.5 (12.5 s)");
    // A moment past the end is the video's last frame.
    expect(parts[3]!.text).toBe("Frame at 0:59.9 (59.9 s)");
    expect(library.calls.some((found) => found.path === "/media/u1/vid/walk.mp4;time(12.5)/256x256-t.jpg")).toBe(true);
    const spread = await client.callTool({ name: "thymind_get_frames", arguments: { id: CARDS.video.id, count: 3 } });
    expect((spread.content as Array<{ type: string }>).filter((part) => part.type === "image")).toHaveLength(3);
    expect(library.calls.some((found) => found.path === "/media/u1/vid/walk.mp4;time(50)/768x768-t.jpg")).toBe(true);
    const unknown = await client.callTool({ name: "thymind_get_frames", arguments: { id: CARDS.oldVideo.id } });
    // An older video's length comes from the file itself, so its frames spread across all of it.
    const unknownParts = unknown.content as Array<{ type: string; text?: string }>;
    expect(unknownParts[0]!.text).toContain("0:30 long");
    expect(unknownParts.filter((part) => part.type === "image")).toHaveLength(6);
    expect(library.calls.some((found) => found.path === "/media/u1/old/clip.mp4;time(27.5)/768x768-t.jpg")).toBe(true);
    const audioFrames = await client.callTool({ name: "thymind_get_frames", arguments: { id: CARDS.audio.id } });
    expect(audioFrames.isError).toBe(true);

    const missing = await client.callTool({ name: "thymind_get_card", arguments: { id: "00000002GoneAAAAAAAAAA" } });
    expect(missing.isError).toBe(true);

    const bySpaceLink = await client.callTool({ name: "thymind_search", arguments: { space_id: "https://mymind.example/s/00000002SpaceAAAAAAAAA" } });
    expect(bySpaceLink.isError).toBeFalsy();
    expect(library.calls.some((found) => found.path === "/app/v1/spaces/00000002SpaceAAAAAAAAA/cards")).toBe(true);
    const spaces = await client.callTool({ name: "thymind_list_spaces", arguments: {} });
    expect((spaces.structuredContent as { spaces: Array<{ link: string }> }).spaces[0]!.link).toBe("https://mymind.example/s/00000002SpaceAAAAAAAAA");

    const pinned = await client.callTool({ name: "thymind_search", arguments: { pinned: true } });
    expect(pinned.structuredContent).toMatchObject({ count: 1, has_more: false });

    // Every library request so far was a read on the allowed list, carrying the app token.
    const reads = library.calls.filter((found) => !found.path.startsWith("/app/v1/auth/"));
    expect(reads.every((found) => found.method === "GET" && found.authorization === "Bearer app-token-1")).toBe(true);
  });

  it("lists the usage analytics tools to the owner only", async () => {
    const owner = setup({ owner: " Octavian@OctavianTocan.com " });
    const signedIn = await connect(owner.call);
    const client = await mcp(owner.call, (await tokens(owner.call, signedIn.client_id, signedIn.back)).access_token);
    const { tools } = await client.listTools();
    expect(tools.map((tool) => tool.name).slice(-2)).toEqual(["thymind_analytics_query", "thymind_analytics_daily"]);
    expect(tools.find((tool) => tool.name === "thymind_analytics_query")!.description).toContain("SUM(_sample_interval)");

    const query = await client.callTool({ name: "thymind_analytics_query", arguments: { sql: "SELECT blob1 AS event, SUM(_sample_interval) AS n FROM thymind_events GROUP BY event" } });
    const text = (query.content as Array<{ text: string }>)[0]!.text;
    expect(text).toContain("<saved-content>");
    expect(text).toContain('"event": "search"');
    const sent = owner.library.calls.find((found) => found.path === "/app/v1/analytics/query")!;
    expect(sent.body).toEqual({ sql: "SELECT blob1 AS event, SUM(_sample_interval) AS n FROM thymind_events GROUP BY event" });
    expect(sent.userAgent).toBe("ThyMind-MCP");

    const refused = await client.callTool({ name: "thymind_analytics_query", arguments: { sql: "DROP TABLE x" } });
    expect(refused.isError).toBe(true);
    expect((refused.content as Array<{ text: string }>)[0]!.text).toBe("Only one SELECT question is allowed.");

    const daily = await client.callTool({ name: "thymind_analytics_daily", arguments: { since: "2026-11-01" } });
    expect((daily.content as Array<{ text: string }>)[0]!.text).toContain("google:octavian@octaviantocan.com");
    expect(new URLSearchParams(owner.library.calls.find((found) => found.path === "/app/v1/analytics/daily")!.search).get("since")).toBe("2026-11-01");

    for (const other of [setup({ owner: "someone@octaviantocan.com" }), setup()]) {
      const connected = await connect(other.call);
      const otherClient = await mcp(other.call, (await tokens(other.call, connected.client_id, connected.back)).access_token);
      const names = (await otherClient.listTools()).tools.map((tool) => tool.name);
      expect(names.some((name) => name.startsWith("thymind_analytics"))).toBe(false);
    }
  });

  it("changes cards when the person allowed changes", async () => {
    const { call, library } = setup();
    const { client_id, back } = await connect(call);
    const { access_token } = await tokens(call, client_id, back);
    const client = await mcp(call, access_token);
    const tool = async (name: string, args: Record<string, unknown>) => {
      const result = await client.callTool({ name, arguments: args });
      expect(result.isError, `${name}: ${JSON.stringify(result.content)}`).toBeFalsy();
      return result.structuredContent as Record<string, unknown>;
    };

    expect(await tool("thymind_save", { content: CARDS.link.url, note: "again" })).toEqual({ id: CARDS.link.id, already_saved: true, not_applied: [] });
    const saved = await tool("thymind_save", { content: "A thought", tags: ["ideas"], space_ids: ["00000002SpaceAAAAAAAAA", "00000002NoSpaceAAAAAAA"] });
    expect(saved).toMatchObject({ already_saved: false, not_applied: ["space 00000002NoSpaceAAAAAAA"] });
    expect(saved.id).toMatch(/^[0-9a-f]{32}$/);

    expect(await tool("thymind_set_note", { id: CARDS.link.id, note: "made it" })).toEqual({ id: CARDS.link.id, previous_note: "try this weekend" });
    expect(await tool("thymind_tag", { ids: [CARDS.note.id], add: ["dinner"], remove: ["Recipes"] })).toEqual({ changed: [CARDS.note.id], not_changed: [] });
    expect(await tool("thymind_tag", { ids: ["00000002GoneAAAAAAAAAA"], remove: ["recipes"] })).toEqual({ changed: [], not_changed: ["00000002GoneAAAAAAAAAA"] });

    const sorted = await tool("thymind_sort", { ids: [CARDS.note.id, CARDS.link.id], new_space_name: "Reading" });
    expect(sorted).toMatchObject({ changed: [CARDS.note.id, CARDS.link.id], not_changed: [] });
    expect(await tool("thymind_sort", { ids: [CARDS.note.id], new_space_name: "kitchen", remove: false })).toMatchObject({ space_id: "00000002SpaceAAAAAAAAA" });
    expect(await tool("thymind_pin", { ids: [CARDS.note.id], pinned: false })).toEqual({ changed: [CARDS.note.id], not_changed: [] });
    expect(await tool("thymind_trash", { ids: [CARDS.note.id, "00000002GoneAAAAAAAAAA"] })).toEqual({ changed: [CARDS.note.id], not_changed: ["00000002GoneAAAAAAAAAA"] });
    expect(await tool("thymind_restore", { ids: [CARDS.note.id, "00000002GoneAAAAAAAAAA"] })).toEqual({ changed: [CARDS.note.id], not_changed: ["00000002GoneAAAAAAAAAA"] });

    const pushes = library.calls.filter((found) => found.path === "/app/v1/push").map((found) => found.body as { clientId: string; changes: Array<{ id: number; name: string; args: Record<string, unknown> }> });
    expect(new Set(pushes.map((push) => push.clientId)).size).toBe(pushes.length);
    const names = pushes.flatMap((push) => push.changes.map((change) => change.name));
    expect(names).toContain("create-space");
    expect(names).not.toContain("delete-space");
    const removed = pushes.flatMap((push) => push.changes).find((change) => change.name === "remove-tag");
    expect(removed?.args).toEqual({ tagId: "t1" });
    const space = pushes.flatMap((push) => push.changes).find((change) => change.name === "create-space");
    expect(space?.args).toMatchObject({ name: "Reading", spaceId: sorted.space_id });
    expect(library.calls.filter((found) => found.method !== "GET" && !found.path.startsWith("/app/v1/auth/")).every((found) => found.path === "/app/v1/push" || found.path.endsWith("/restore"))).toBe(true);
  });

  it("asks a read-only connection to connect again before it changes anything", async () => {
    const { call, library } = setup();
    const { client_id, back } = await connect(call, false);
    const { access_token } = await tokens(call, client_id, back);
    const answers: Response[] = [];
    const recording = async (input: string | Request, init?: RequestInit) => {
      const response = await call(input, init);
      answers.push(response.clone());
      return response;
    };
    const client = await mcp(recording, access_token);
    const search = await client.callTool({ name: "thymind_search", arguments: {} });
    expect(search.isError).toBeFalsy();
    await expect(client.callTool({ name: "thymind_pin", arguments: { ids: [CARDS.note.id] } })).rejects.toThrow();
    const challenge = answers.find((answer) => answer.status === 403)!;
    expect(challenge.headers.get("www-authenticate")).toContain('error="insufficient_scope"');
    expect(challenge.headers.get("www-authenticate")).toContain("thymind.write");
    expect(library.calls.some((found) => found.path === "/app/v1/push")).toBe(false);
  });

  it("connects a client that names itself with a metadata document (CIMD), as ChatGPT does, and speaks MCP 2026-07-28", async () => {
    const { call, library } = setup();
    const { client_id, back } = await connect(call, true, true);
    const { access_token } = await tokens(call, client_id, back);
    expect(library.calls.find((found) => found.path === "/app/v1/auth/google")!.body).toMatchObject({ device: "MCP: ChatGPT" });
    const client = await mcp(call, access_token, true);
    const spaces = await client.callTool({ name: "thymind_list_spaces", arguments: {} });
    expect(spaces.structuredContent).toEqual({ spaces: [{ id: "00000002SpaceAAAAAAAAA", name: "Kitchen", cards: 1, smart: false, link: "https://mymind.example/s/00000002SpaceAAAAAAAAA" }] });
  });

  it("grants read only when the person unticks changes", async () => {
    const { call } = setup();
    const { client_id, back } = await connect(call, false);
    const { scope } = await tokens(call, client_id, back);
    expect(scope).toBe("thymind.read");
  });

  it("shows a refusal page for a Google account the library does not admit", async () => {
    const { call } = setup({ refuse: true });
    const { back } = await connect(call);
    expect(back.status).toBe(403);
    const page = await back.text();
    expect(page).toContain("This Google account can't use Inventory");
    expect(page).toContain("error=access_denied");
    expect(page).not.toContain("octaviantocan");
  });

  it("asks the app to connect again once the library signs the token out", async () => {
    const { call, library, waits } = setup();
    const { client_id, back } = await connect(call);
    const { access_token } = await tokens(call, client_id, back);
    const client = await mcp(call, access_token);
    library.state.dead = true;
    const result = await client.callTool({ name: "thymind_list_tags", arguments: {} });
    expect(result.isError).toBe(true);
    expect((result.content as Array<{ text: string }>)[0]!.text).toContain("connect Inventory again");
    await Promise.all(waits);
    const after = await call("/mcp", { method: "POST", headers: { authorization: `Bearer ${access_token}`, "content-type": "application/json" }, body: "{}" });
    expect(after.status).toBe(401);
  });
});
