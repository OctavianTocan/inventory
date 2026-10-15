# mymind reconstruction

A localhost replica of `https://access.mymind.com/everything`, UI only. It runs mymind's real
front-end against a recording of your library, so nothing talks to mymind's servers.

## Run it

```bash
npm run serve                 # http://localhost:4321/everything
OFFLINE=1 npm run serve       # no network at all; static assets come from mirror/
```

Writes (save, tag, delete) are accepted and ignored. Unrecorded API calls are logged as `[miss]`, media without a
recording as `[placeholder]`. Responses in `replay/` (see below) win over `capture/`.

## Re-capture

Needs Chrome with remote debugging on port 9222 and a logged-in mymind tab open.

```bash
npm run capture -- --media-cards 300 --detail-cards 30   # add --videos to include mp4s
npm run mirror                                             # static JS/CSS/fonts for OFFLINE=1
```

## Verify against live, 1:1

```bash
npm run serve                              # replica on :4321 (OFFLINE=1 to prove it needs nothing from mymind)
npm run drive -- --profile desktop-light --suite ui     # one profile x suite: live, then replica, then REA diffs
npm run verify                             # every profile (desktop light/dark, iPhone, Android) x suite (ui, cards)
npm run coverage                           # REA's fail-closed verdict over all runs: ready / partial / failed
```

Needs the agent Chromium on CDP `:9222` signed in to mymind; only its two mymind session cookies and the page's
localStorage are read, in memory, to sign in the verifier.

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

- Detail views exist only for the first 30 cards; images only for the first 300. Others show a placeholder.
- Videos, the realtime hub, `/tags/manual`, `/cards/:id?template=PinnedCard` and the web manifest are not replayed.

## Private data

`capture/` and `replay/` hold your own library. `mirror/` and `evidence/` hold mymind's code. All four are
gitignored; keep them on this machine.
