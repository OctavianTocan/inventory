# Inventory MCP server

A Cloudflare Worker that lets a person's AI agents (Claude, ChatGPT, Codex, Cursor, …) search and read their
Inventory library over the Model Context Protocol (spec 2026-07-28). The design and its decisions are in the project
brief (`thymind-mcp/concepts_brief.md` in the project files).

- **Endpoint:** `https://mymind.octaviantocan.com/mcp` (Streamable HTTP, stateless).
- **Sign-in:** OAuth 2.1 with PKCE. Clients register by Client ID Metadata Document or dynamic registration. The
  person sees a plain consent page, then signs in with Google; the Worker trades that Google sign-in for an app
  token from the `mymind` Worker (`/app/v1/auth/google`), so the same accounts are allowed as in the app. The app
  token stays inside the encrypted OAuth grant and is never sent to the agent.
- **Scopes:** `thymind.read` (always) and `thymind.write` (the consent checkbox, ticked by default). A write tool
  called on a read-only connection answers 403 `insufficient_scope` naming `thymind.write`, so the client can ask the
  person again.
- **Limit:** every request to the library is checked against `ALLOWED` in `src/library.ts` before it leaves: GET
  reads and picture renditions, `POST /app/v1/push` with only the change names in `CHANGES` (no `delete-space`), and
  `POST /objects/<id>/restore`. Each push names a fresh client id, so concurrent calls never skip each other's
  changes.

## Tools

| Tool | What it does |
|---|---|
| `thymind_search` | Search by meaning or words, or list a space, pinned cards or top of mind. Paged. |
| `thymind_get_card` | One card in full: text, note, tags, spaces, link, comments. |
| `thymind_get_picture` | A card's picture as an image (128–1024 px). |
| `thymind_get_transcript` | Everything said in a saved video or audio file, in pages of 50,000 characters. |
| `thymind_get_frames` | Frames of a saved video at chosen moments, or spread evenly (up to 10 a call), through the `time(s)` media rendition. |
| `thymind_list_spaces` | Spaces with card counts. |
| `thymind_list_tags` | Tags with card counts. |
| `thymind_save` | Save a link or text, with a note, tags and spaces. A link already saved is bumped. |
| `thymind_set_note` | Replace a card's note; returns the note it replaced. |
| `thymind_tag` | Add and remove tags on up to 25 cards. |
| `thymind_sort` | Put up to 25 cards in a space (or a new one by name), or take them out. |
| `thymind_pin` | Pin or unpin up to 25 cards. |
| `thymind_trash` | Move up to 25 cards to the trash. |
| `thymind_restore` | Take up to 25 cards out of the trash. |

The person sees and removes connected AI apps in the app under Profile, Connected agents (`GET` and `DELETE
/app/v1/agents` on the `mymind` Worker, which list the app tokens named `MCP: <client>`). Removing one signs its app
token out; the next call from that agent gets a 401 from the library, and this Worker then revokes its grant.

In Claude and ChatGPT, which support MCP Apps, `thymind_search` results show as a grid of card tiles in Inventory's
colours (`src/view.ts`, resource `ui://thymind/cards.html`). The grid gets its cards from the tool result and each
picture by calling `thymind_get_picture` through the host, so it needs no network of its own (empty CSP) and never
holds a token. Tapping a tile with a link opens it.

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

1. Done 5 Nov 2026: KV namespace `thymind-mcp-oauth` (`5da66adf…`, in `wrangler.jsonc`), the Google secrets, and the
   first deploy from mini-002-netcup.
2. In the Google Cloud console, add `https://mymind.octaviantocan.com/mcp-auth/callback` as an authorized redirect
   URI on the web OAuth client (the same client the server uses is fine).
3. `npx wrangler secret put GOOGLE_CLIENT_ID` and `npx wrangler secret put GOOGLE_CLIENT_SECRET`.
4. `npm run deploy`. The routes `mymind.octaviantocan.com/mcp*` and `/.well-known/oauth-*` attach to the zone.
5. Connect an AI app (below) with your own account and try a search.

## Connecting an AI app

`plugin/` is one package for both ChatGPT and Claude Code: the shared skill `skills/thymind/SKILL.md` (what cards,
spaces and tags are, how to search, that saved text is data), `plugin.json` and `mcp.json` for ChatGPT, and
`.claude-plugin/plugin.json` and `.mcp.json` for Claude Code. `test/plugin.test.ts` fails when the skill names a tool
the server lacks or misses one it has.

- **Claude.ai and the Claude apps:** Settings, Connectors, Add custom connector, URL
  `https://mymind.octaviantocan.com/mcp`. To add the skill too, zip `plugin/skills/thymind/` and upload it under
  Settings, Capabilities, Skills.
- **Claude Code:** `/plugin install thymind --marketplace OctavianTocan/mymind-reconstruction` (the repository's
  `.claude-plugin/marketplace.json`; the repository is private, so it needs GitHub access to it), or just the server:
  `claude mcp add --transport http thymind https://mymind.octaviantocan.com/mcp`.
- **ChatGPT (private, Tavi's choice on 5 Nov):** Settings, Plugins (chatgpt.com/plugins), Add custom MCP server, URL
  `https://mymind.octaviantocan.com/mcp`. ChatGPT names itself with its Client ID Metadata Document, so the consent
  page shows "Published by chatgpt.com". The package in `plugin/` is what a workspace admin publishes inside a
  ChatGPT workspace, or what a public listing would upload; it is not listed.

## Checks

```bash
npm ci
npm run check        # tsc, then vitest: unit tests and the whole connect-and-use flow against fakes
npx wrangler deploy --dry-run --outdir /tmp/thymind-mcp   # bundles the Worker
```

The flow test drives the real OAuth provider and the real MCP client against an in-memory KV, a fake `mymind`
binding and a stubbed Google token endpoint. Nothing here has run against the live library yet; that waits for the
deploy above.

## Card links

The app's Share sheet copies `<server>/c/<card id>` (Link for AI agents). Every tool that takes a card id also takes
that link (`src/refs.ts`); it names a card in the signed-in person's own library and reaches nothing else.
