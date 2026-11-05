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

function setup(options: { refuse?: boolean } = {}) {
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
  expect(page).toContain(cimd ? "Connect ChatGPT to ThyMind?" : "Connect Test &#60;Agent&#62; to ThyMind?");
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
    expect(tools.map((tool) => tool.name)).toEqual(["thymind_search", "thymind_get_card", "thymind_get_picture", "thymind_list_spaces", "thymind_list_tags"]);
    expect(tools.every((tool) => tool.annotations?.readOnlyHint)).toBe(true);

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

    const missing = await client.callTool({ name: "thymind_get_card", arguments: { id: "00000002GoneAAAAAAAAAA" } });
    expect(missing.isError).toBe(true);

    const pinned = await client.callTool({ name: "thymind_search", arguments: { pinned: true } });
    expect(pinned.structuredContent).toMatchObject({ count: 1, has_more: false });

    // Every library request was a read on the allowed list, carrying the app token.
    const reads = library.calls.filter((found) => !found.path.startsWith("/app/v1/auth/"));
    expect(reads.every((found) => found.method === "GET" && found.authorization === "Bearer app-token-1")).toBe(true);
  });

  it("connects a client that names itself with a metadata document (CIMD), as ChatGPT does, and speaks MCP 2026-07-28", async () => {
    const { call, library } = setup();
    const { client_id, back } = await connect(call, true, true);
    const { access_token } = await tokens(call, client_id, back);
    expect(library.calls.find((found) => found.path === "/app/v1/auth/google")!.body).toMatchObject({ device: "MCP: ChatGPT" });
    const client = await mcp(call, access_token, true);
    const spaces = await client.callTool({ name: "thymind_list_spaces", arguments: {} });
    expect(spaces.structuredContent).toEqual({ spaces: [{ id: "00000002SpaceAAAAAAAAA", name: "Kitchen", cards: 1, smart: false }] });
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
    expect(page).toContain("This Google account can't use ThyMind");
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
    expect((result.content as Array<{ text: string }>)[0]!.text).toContain("connect ThyMind again");
    await Promise.all(waits);
    const after = await call("/mcp", { method: "POST", headers: { authorization: `Bearer ${access_token}`, "content-type": "application/json" }, body: "{}" });
    expect(after.status).toBe(401);
  });
});
