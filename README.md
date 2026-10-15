# mymind reconstruction

A localhost replica of `https://access.mymind.com/everything`. It runs mymind's real front-end against a local
SQLite library seeded from your recordings. Notes, pasted text, URL placeholders, edits, tags, spaces and settings
persist locally; unsupported reads still use the recordings. No write is forwarded to mymind.

## Run it

```bash
npm run serve                 # http://localhost:4321/everything
OFFLINE=1 npm run serve       # no network at all; static assets come from mirror/
```

When `data/mymind.db` exists, desktop card grids, individual cards, trash, spaces, tags, changes and object quotas
come from SQLite. `POST /cards/grid` also hydrates current SQLite cards. Phone requests keep their device-specific
recordings because the renderer currently generates desktop markup. Unsupported reads retain the existing
`replay/` then `capture/` fallback; without a database, the entire server retains that fallback and ignores writes.

Set `STORE_DB=/path/to/library.db` to select another database. `STORE_READONLY=1` enables store reads while
acknowledging writes without persisting them or broadcasting write events. Grid hydration still works in that mode.
Unrecorded API calls are logged as `[miss]`, media without a recording as `[placeholder]`.

To build the initial database, run `node tools/seed-db.mjs`. Rebuilding replaces an existing database, including its
local edits. The seed also imports response metadata and the recorded space-detail query-check times; card and
space bodies are generated from the stored rows. Older phase 1 databases import that metadata on first startup.

Card and space writes use one transaction, regenerate affected card markup, append change-log entries and broadcast SignalR events
to connected `/hub` clients after commit. Polling returns the latest change per object so a delete cannot be undone
by an older queued update in the client. Drafts stay outside the main grid until committed. New or edited desktop
cards have generated editable details and pinned previews; unchanged detail/template recordings retain their
existing replay path. Shared cards have local URLs with a 24-hour expiry and revocation.

URL capture stops at storing the URL, host title and analyzing flag. `captureLink` in `lib/store-write.mjs` is the
phase 4 enrichment entry point and currently performs no fetch. Uploads and attachments need metadata extraction
and media renditions, so remain in phase 4. AI analysis and thumbnail generation remain in phase 5. Permanent
delete and empty trash have no client calls in the captured client, so only trash and restore are implemented.

Run the local data and HTTP checks without opening a browser:

```bash
node tools/check-reads.mjs
node tools/check-writes.mjs
node tools/check-store.mjs
node tools/check-render.mjs
```

The HTTP checks back up the seeded database to temporary files, start isolated offline servers on OS-assigned
loopback ports, and clean them up. They never use port 4321. If a sandbox prohibits socket listeners, pass
`--in-process` to either HTTP check to exercise the real server request callbacks and hub framing in memory;
that mode does not establish socket or browser parity.

## Re-capture

Needs Chrome with remote debugging on port 9222 and a logged-in mymind tab open.

```bash
npm run capture -- --media-cards 300 --detail-cards 30   # add --videos to include mp4s
npm run mirror                                             # static JS/CSS/fonts for OFFLINE=1
```

## Verify against live, 1:1

```bash
STORE_READONLY=1 npm run serve             # replica on :4321; required for verification
npm run drive -- --profile desktop-light --suite ui     # one profile x suite: live, then replica, then REA diffs
npm run verify                             # every profile (desktop light/dark, iPhone, Android) x suite (ui, cards)
npm run coverage                           # REA's fail-closed verdict over all runs: ready / partial / failed
```

Needs the agent Chromium on CDP `:9222` signed in to mymind; only its two mymind session cookies and the page's
localStorage are read, in memory, to sign in the verifier.

Every verification server must use `STORE_READONLY=1`, because live writes are intercepted and sent to the replica
as well. `tools/verify-all.mjs` sets it when starting a server. If a server is already running, restart it yourself in
that mode before verifying; the verifier does not change the environment of an existing process.

How a run makes live and replica comparable pixel for pixel (`tools/drive.mjs`, `tools/side.mjs`,
`tools/page-determinism.js`, `tools/verifier-browser.mjs`):

- **A deterministic browser.** A separate Chromium rendering offscreen (no windows, same user agent as the agent
  browser) with software raster, no partial re-raster, no deferred image decode, main-thread animations, CPU WebGL
  and no smooth input scrolling. The same page renders the same pixels on every run.
- **Virtual time.** Both sides run on a paused fake clock (`Date`, timers, `requestAnimationFrame`, `performance`)
  that only the driver advances. CSS animations, transitions and `element.animate()` are frozen the moment they
  exist and stepped by the same amount; media holds its first frame. `performance.now()` and animation-frame
  timestamps are rebased on the fake `Date` (Playwright's own versions keep counting real time). The browser's
  smooth scrolling is made instant on both sides, since its scroll events land on whatever real frame comes next.
- **Network in lockstep.** Every response is held and released at the same virtual instant on both sides, and time
  does not move while a fetch, stream read, IndexedDB transaction, worker reply or history navigation is pending.
  A slow live server and a fast localhost therefore reach each moment in the same state.
- **Randomness.** `Math.random` and `crypto.randomUUID` get one seeded stream per calling line, so call order
  across async work cannot shift values.
- **Same data.** The live pass records every response into `replay/` (per device class, since mymind renders
  different HTML for phones, and per run, since some responses are random, e.g. the serendipity deck); the replica
  pass asks for exactly those.
- **Media and other processes.** Videos never play (poster or first frame on both sides), and each screenshot is
  retaken until two in a row agree, which lets out-of-process renderers (the PDF viewer, embeds) finish. Failed
  loads (Chrome aborts in-flight requests whenever a network interface changes) are retried through the browser.
- **Third-party players are masked.** YouTube, Vimeo, Spotify and similar iframes are painted grey on both sides:
  their owners decide per embedding site what to show (a video can refuse to embed on `access.mymind.com` and show
  "unavailable" on `localhost`), so their content is not the replica's to match. Their size and position still are.
- **No writes reach your account.** On live, every non-GET request is answered by the replica server instead of
  mymind (the same answer the replica gives itself). Report: `writesAnsweredLocally` in each `report.json`.
- **Same input.** The pointer rests on a neutral spot, hover tests move it to identical coordinates, and virtual time
  only moves after the real frame that delivers each input, scroll event and observer callback.

Each run writes `evidence/drive/<profile>-<suite>/{live,replica}/*.png`, `diffs/`, `report.json` (REA verdict per
screenshot) and `rea-evidence.json` (the REA diff Evidence records `tools/coverage.mjs` builds its verdict from).

To re-verify only part of a run, use a patch suite named `<suite>@<part>` (e.g. `--suite ui@serendipity`, or
`--suite cards@media` on phones); its screenshots replace the same-named ones of `<profile>-<suite>` in the coverage
verdict. If `evidence/drive/claims/` exists, each run is done once across parallel `npm run verify` processes, and a
claim file written by hand skips a run that is already verified; delete the folder afterwards.
Suites (`tools/suites.mjs`) cover every route, card type, hover state of every element mymind styles on hover,
frame-by-frame films of each transition, drag, color scheme, density, focus mode, news, shortcuts, search and
serendipity, plus touch flows (swipe, pull to refresh) on phones.

## How it was reverse engineered

1. Find the page: `rea list-browser-targets http://127.0.0.1:9222`.
2. Capture it passively, with the CDN origin allowed (otherwise every script is filtered out):
   `rea inspect-web-page <cdp> <target> --allowed-origins https://access.mymind.com --allowed-origins https://static.accelerator.net --include-script-sources --include-json-body-shapes --include-accessibility-text`
3. Export and statically analyse the scripts:
   `rea export-web-scripts evidence/page-capture.json evidence/scripts` then
   `rea analyze-javascript-application evidence/scripts`.
4. Record the data the UI needs (`tools/capture.mjs`) and replay it (`server.mjs`).
5. Prove the result: `npm run verify` (REA `compare-web-screenshots` on every state), then
   `npm run coverage` (REA `evaluate-reconstruction-coverage`).

Hopper and Ghidra are not involved. They read native machine code; this app ships readable JavaScript.

## What the app is

- Server-rendered HTML plus ~80 unbundled, unminified ES modules from `static.accelerator.net/134/0.109.0/`.
  No framework and no build step. Vendor code: Lit, TipTap 3, GSAP, OGL (WebGL), Sortable, Lottie, SignalR, msgpack.
- `GET /cards` streams msgpack values shaped `{ html, json }`: the server renders every card, the client only
  lays them out (`grid.js`, `modules/layouts.js`).
- Opening a card fetches server-rendered HTML from `/cards/:id/detail`.
- Live updates come from `/changes?since=` (msgpack) and a SignalR hub at `/hub`.

## Known gaps

- Recorded detail views cover the first 30 cards; images cover the first 300. Unchanged uncaptured cards use placeholders.
- Generated details and rich prose nodes have route/data coverage, but still need browser and REA parity checks.
- Media without recordings, link enrichment, uploads, AI and full search remain later phases.

## Private data

`capture/` and `replay/` hold your own library. `mirror/` and `evidence/` hold mymind's code. All four are
gitignored; keep them on this machine.
