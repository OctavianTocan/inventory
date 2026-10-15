# mymind reconstruction

A localhost replica of `https://access.mymind.com/everything`, UI only. It runs mymind's real
front-end against a recording of your library, so nothing talks to mymind's servers.

## Run it

```bash
npm run serve                 # http://localhost:4321/everything
OFFLINE=1 npm run serve       # no network at all; static assets come from mirror/
```

Writes (save, tag, delete) are accepted and ignored. Unrecorded API calls are logged as `[miss]`.

## Re-capture

Needs Chrome with remote debugging on port 9222 and a logged-in mymind tab open.

```bash
npm run capture -- --media-cards 300 --detail-cards 30   # add --videos to include mp4s
npm run mirror                                             # static JS/CSS/fonts for OFFLINE=1
```

## How it was reverse engineered

1. Find the page: `rea list-browser-targets http://127.0.0.1:9222`.
2. Capture it passively, with the CDN origin allowed (otherwise every script is filtered out):
   `rea inspect-web-page <cdp> <target> --allowed-origins https://access.mymind.com --allowed-origins https://static.accelerator.net --include-script-sources --include-json-body-shapes --include-accessibility-text`
3. Export and statically analyse the scripts:
   `rea export-web-scripts evidence/page-capture.json evidence/scripts` then
   `rea analyze-javascript-application evidence/scripts`.
4. Record the data the UI needs (`tools/capture.mjs`) and replay it (`server.mjs`).
5. Prove the result: `rea capture-web-screenshot` on both tabs, then `rea compare-web-screenshots`.
   The live and offline replica viewports compared identical: 380,568 pixels, 0 changed.

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

`capture/` holds your own library. `mirror/` and `evidence/` hold mymind's code. All three are gitignored;
keep them on this machine.
