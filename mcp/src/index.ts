/**
 * ThyMind's remote MCP server (`docs/mcp.md`). Cloudflare's OAuth library makes this Worker the authorization server
 * MCP clients sign in to (metadata, CIMD and registration, PKCE, tokens in KV); the MCP SDK answers `/mcp`
 * statelessly, one fresh server per request, bound to the signed-in person's library.
 */
import { OAuthProvider, type OAuthResourceAuth } from "@cloudflare/workers-oauth-provider";
import { createMcpHandler } from "@modelcontextprotocol/server";

import { Library } from "./library";
import { READ_SCOPE, WRITE_SCOPE } from "./pages";
import { type Env, PATHS, type Props, signIn } from "./signin";
import { createServer } from "./tools";

type ApiContext = ExecutionContext<Props> & { readonly auth: OAuthResourceAuth };

const DAY = 24 * 60 * 60;

async function revoke(env: Env, auth: OAuthResourceAuth): Promise<void> {
  if (!auth.userId || !auth.clientId) return;
  const grants = await env.OAUTH_PROVIDER.listUserGrants(auth.userId);
  for (const grant of grants.items) {
    if (grant.clientId === auth.clientId) await env.OAUTH_PROVIDER.revokeGrant(grant.id, auth.userId);
  }
}

/** `/mcp`, reached only with a valid token for this resource. */
const api = {
  async fetch(request: Request, env: Env, ctx: ApiContext): Promise<Response> {
    if (env.LIMITER) {
      const { success } = await env.LIMITER.limit({ key: ctx.auth.userId ?? "anonymous" });
      if (!success) {
        return Response.json({ error: "Too many requests. Wait a minute and try again." }, { status: 429, headers: { "retry-after": "60" } });
      }
    }
    const library = new Library(env.LIBRARY_API, env.PUBLIC_URL, ctx.props.appToken);
    const onSignedOut = () => ctx.waitUntil(revoke(env, ctx.auth));
    return createMcpHandler(() => createServer(library, onSignedOut)).fetch(request);
  },
};

let provider: { origin: string; value: OAuthProvider<Env> } | undefined;

function oauth(env: Env): OAuthProvider<Env> {
  if (provider?.origin !== env.PUBLIC_URL) {
    const origin = env.PUBLIC_URL;
    provider = {
      origin,
      value: new OAuthProvider<Env>({
        apiRoute: "/mcp",
        apiHandler: api as unknown as ExportedHandler<Env> & { fetch: NonNullable<ExportedHandler<Env>["fetch"]> },
        defaultHandler: signIn,
        authorizeEndpoint: PATHS.authorize,
        tokenEndpoint: PATHS.token,
        // Deprecated in MCP 2026-07-28, kept for clients without Client ID Metadata Documents.
        clientRegistrationEndpoint: PATHS.register,
        clientIdMetadataDocumentEnabled: true,
        scopesSupported: [READ_SCOPE, WRITE_SCOPE],
        requiredScopes: [READ_SCOPE],
        resourceMetadata: { resource: `${origin}/mcp`, authorization_servers: [origin], resource_name: "ThyMind" },
        // A connection lives while it is used, like the app's own sign-in (90 days, renewed on use).
        refreshTokenTTL: 90 * DAY,
        refreshTokenIdleTTL: 90 * DAY,
        clientRegistrationTTL: 90 * DAY,
      }),
    };
  }
  return provider.value;
}

export default {
  fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    return oauth(env).fetch(request, env, ctx);
  },
} satisfies ExportedHandler<Env>;
