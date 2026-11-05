# ThyMind MCP server

A Cloudflare Worker that lets a person's AI agents (Claude, ChatGPT, Codex, Cursor, …) search and read their
ThyMind library over the Model Context Protocol (spec 2026-07-28). The design and its decisions are in the project
brief (`thymind-mcp/concepts_brief.md` in the project files).

- **Endpoint:** `https://mymind.octaviantocan.com/mcp` (Streamable HTTP, stateless).
- **Sign-in:** OAuth 2.1 with PKCE. Clients register by Client ID Metadata Document or dynamic registration. The
  person sees a plain consent page, then signs in with Google; the Worker trades that Google sign-in for an app
  token from the `mymind` Worker (`/app/v1/auth/google`), so the same accounts are allowed as in the app. The app
  token stays inside the encrypted OAuth grant and is never sent to the agent.
- **Scopes:** `thymind.read` (always) and `thymind.write` (the consent checkbox; no write tools yet).
- **Limit:** every request to the library is checked against `ALLOWED` in `src/library.ts` before it leaves.
  Only GET reads and picture renditions are on it today.

## Tools

| Tool | What it does |
|---|---|
| `thymind_search` | Search by meaning or words, or list a space, pinned cards or top of mind. Paged. |
| `thymind_get_card` | One card in full: text, note, tags, spaces, link, comments. |
| `thymind_get_picture` | A card's picture as an image (128–1024 px). |
| `thymind_list_spaces` | Spaces with card counts. |
| `thymind_list_tags` | Tags with card counts. |

Saved text comes back inside `<saved-content>` fences, and the server instructions tell the agent to treat it as
data, never as instructions.

## Paths on the host

| Path | Owner |
|---|---|
| `/mcp` | MCP endpoint (bearer token required) |
| `/mcp-auth/authorize`, `/mcp-auth/callback`, `/mcp-auth/token`, `/mcp-auth/register` | OAuth pages and endpoints |
| `/.well-known/oauth-authorization-server`, `/.well-known/oauth-protected-resource/mcp` | OAuth metadata |

The OAuth pages sit under `/mcp-auth/` because the OAuth library guards `/mcp` and every path below it.

## Going live (Tavi)

Deploy the `mymind` Worker first; this Worker reaches it through the `LIBRARY_API` service binding.

1. `npx wrangler kv namespace create thymind-mcp-oauth` and paste the id into `wrangler.jsonc` (`OAUTH_KV`).
2. In the Google Cloud console, add `https://mymind.octaviantocan.com/mcp-auth/callback` as an authorized redirect
   URI on the web OAuth client (the same client the server uses is fine).
3. `npx wrangler secret put GOOGLE_CLIENT_ID` and `npx wrangler secret put GOOGLE_CLIENT_SECRET`.
4. `npm run deploy`. The routes `mymind.octaviantocan.com/mcp*` and `/.well-known/oauth-*` attach to the zone.
5. In Claude: Settings, Connectors, Add custom connector, URL `https://mymind.octaviantocan.com/mcp`.

## Checks

```bash
npm ci
npm run check        # tsc, then vitest: unit tests and the whole connect-and-use flow against fakes
npx wrangler deploy --dry-run --outdir /tmp/thymind-mcp   # bundles the Worker
```

The flow test drives the real OAuth provider and the real MCP client against an in-memory KV, a fake `mymind`
binding and a stubbed Google token endpoint. Nothing here has run against the live library yet; that waits for the
deploy above.
