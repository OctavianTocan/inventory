# Usage analytics

What each account has in Inventory (ThyMind inside the code) and how it is used, read by asking Claude through the MCP server. The brief and
its six reviews are in the project files (`analytics/concepts_brief.md`).

Tavi's decisions (2026-11-05):

- **Everything is recorded**, including card ids and search words.
- **No dashboard.** Tavi asks Claude, through two MCP tools that only `OWNER_EMAIL` gets.
- **Events are kept 3 months**, the Analytics Engine limit. **Daily totals are kept forever.**
- **People are not told.** There is no notice and no opt-out.

## What is recorded

Every event is one Workers Analytics Engine data point in the dataset `thymind_events` (binding `EVENTS` in
`rust/crates/worker/wrangler.jsonc`). The layout is `mm_core::analytics::point`. Columns are positional and phones
keep sending whatever layout they shipped with, so **new columns are only ever added at the end and nothing moves**.
`rust/crates/core/tests/analytics.rs` fails if a column moves.

| Column | Meaning |
|---|---|
| index1 | library key: first 32 hex digits of the SHA-256 of the library name |
| blob1 | event name |
| blob2 | app version |
| blob3 | screen |
| blob4 | card id |
| blob5 | card kind |
| blob6 | search words |
| blob7 | detail (route, change name, feature or error) |
| blob8 | batch id |
| blob9 | client: `android`, `mcp`, `extension` or `web` |
| blob10 | account email |
| double1 | layout version (1) |
| double2 | count |
| double3 | result count |
| double4 | duration in ms |
| double5 | HTTP status |
| double6 | when the phone saw it, ms since 1970 (phone events only; `timestamp` is when it arrived) |

Text is cut to 200 characters.

Events the server records today:

| Event | Where | Detail |
|---|---|---|
| `request` | `rust/crates/worker/src/lib.rs`, every API call (`/app/v1/*`, `/api`, `/objects`, …) | blob7 `GET app/v1/cards`, double4 time, double5 status |
| `signin` | the same, for the sign-in POSTs | blob7 the path, double5 status |
| `change` | `app_push::push`, one per change pushed by the app or an agent | blob7 the change name, blob4 its card, double5 200, or 409 when turned down |
| `search` | `/app/v1/search`, first page only | blob6 the words, blob7 `words` or `meaning`, double3 results |

The phone's own events come through `POST /app/v1/events` (`rust/crates/worker/src/analytics.rs`), in the same
layout with client `android`:

| Event | Where in the app | Fields |
|---|---|---|
| `start` | the app comes to the front (`MymindApp`) | |
| `screen` | All, Spaces, Ask or a space shown (`MainShell`) | blob3 `all`, `spaces`, `ask` or `space` |
| `open` | a card opened (`MainShell`) | blob3, blob4 card, blob5 kind |
| `search` | a search pressed (`SearchModel.submit`) | blob6 the words, blob7 `similar` for Same Vibe |
| `action` | a card sheet action (`LibraryViewModel.perform`) | blob7 the action (`Pin`, `Delete`, …), blob4, blob5 |
| `share` | text or a picture handed to the share sheet | blob7 `text` or `picture` |
| `pinch` | a pinch that changed the columns, counted (`MosaicLayout`) | blob7 the new column count, double2 how many |

The phone keeps them in `events.json` (`data/Analytics.kt`), apart from the outbox: at most 2000, oldest dropped
first, wiped at sign-out. About once a minute and whenever the app leaves the screen, it sends them in batches of
100 (`{batch, version, events: [{name, screen, card, kind, words, detail, count, results, ms, at}]}`). The server
keeps only the names above, drops anything else, and answers `{kept}`, or `{kept: 0, stop: true}` while analytics
is off, which makes the phone drop what it holds until it starts again. A batch writes no `request` event of its
own.

A library's host installs the sink with `Store::set_analytics`. The native server never installs one, so it records
nothing. The client of a `change` or `search` event belongs to the library's latest request. A library answers one
person, so only that field can be wrong when two requests overlap.

`ANALYTICS=off` (a Worker variable) stops every write.

## Daily totals

A cron trigger (`30 23 * * *` UTC) runs `analytics::daily_totals` in the Worker. It lists every library from the
`Auth` object, asks each `Library` object for `mm_core::analytics::library_totals` through the admin path, and
stores one row per library per day in the `Auth` object's `analytics_daily` table. Writing the same day again
replaces the row. A row holds counts only: cards, trashed, notes, spaces, tags, pinned, shared, uploaded bytes, cards
by kind, and the last save time. A library that fails gets an `{error}` row, and the others still run.

## Asking

Two owner-only routes on the `mymind` Worker. Every other account gets a 404, the same as an unknown path:

- `POST /app/v1/analytics/query` with `{"sql": "SELECT …"}`. It takes one `SELECT` (or `WITH … SELECT`). It adds
  `LIMIT 200` when there is none and refuses a larger one. It answers `{columns, rows, total, cut, layout}`, with at
  most 200 rows and text cut to 200 characters. It calls Cloudflare's SQL API with `ANALYTICS_ACCOUNT_ID` and
  `ANALYTICS_TOKEN`, and answers 503 while those are not set.
- `GET /app/v1/analytics/daily?since=YYYY-MM-DD&library=` answers `{days: [{day, library, account, totals}]}`,
  newest first.

The MCP server lists `thymind_analytics_query` and `thymind_analytics_daily` only when the signed-in email is its
`OWNER_EMAIL` secret. Their answers are wrapped in `<saved-content>` because they hold words other people typed.
Rows are sampled under load, so count with `SUM(_sample_interval)`. The query tool's description carries the layout
and that rule.

## Going live

1. In the Cloudflare dashboard, create an API token with **Account › Account Analytics › Read**.
2. In `rust/crates/worker`, run `npx wrangler secret put ANALYTICS_TOKEN` (the token) and
   `npx wrangler secret put ANALYTICS_ACCOUNT_ID` (the account id).
3. In `mcp/`, run `npx wrangler secret put OWNER_EMAIL` (Tavi's Google address).
4. The `mymind` Worker deploys itself when `main` moves (the VPS timer). Deploy the MCP Worker by hand with
   `npm run deploy` in `mcp/`. The dataset appears on its first write, and the first daily row on the next 23:30 UTC.
