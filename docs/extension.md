# Browser extension

`extension/` is our own Manifest V3 rebuild of mymind's Chrome extension 3.3.0. Plain JavaScript ES modules, no build
step. It talks to the replica backend, never to `access.mymind.com`.

## Setup

1. `node extension/tools/copy-mymind-assets.mjs [path/to/mymind-extension-3.3.0]` copies mymind's images, icons, gifs,
   the Nunito font and `vendor/tiptap.min.js` into `extension/mymind/`, which is git-ignored. The default source is
   the unpacked extension in the Chromium profile that has it installed.
2. Open `chrome://extensions`, turn on developer mode, "Load unpacked", choose `extension/`.
3. Open the extension's options and set the host. The default is `https://mymind.octaviantocan.com`; for development
   use `http://localhost:4331` or another local port. Only the default host is granted at install. `localhost`,
   `127.0.0.1` and any `https://` host are optional host permissions that the options page requests when you save.
4. Sign in to that host in the same browser profile. The extension reads the `mm_session` cookie of the host with
   `chrome.cookies` and sends it as `Authorization: Bearer`.

## Features and parity

Parity was measured against a patched copy of the real 3.3.0 (host, cookie name and origin handling changed only),
driven through the same flows in a scratch Chromium, 1280x800 at device scale 1. Screenshots: `evidence/extension/shots/`
(`ours-N-*`, `ref-N-*`, `diff-N-*`); `extension/tests/diff.mjs` rebuilds the diffs.

| Feature | Status |
|---|---|
| Toolbar action and `Ctrl+Shift+S` save the page, then upload a screenshot | Matches (toast and card). Screenshot upload untested: headless runs have no user gesture, so `captureVisibleTab` is refused in both extensions |
| Context menus: add page, link, image, video, selection; "Open my mind" | Matches for page, image, selection. Menus themselves are not driven (no native menu in headless) |
| Omnibox keyword `mm` | Untested (no omnibox in headless); code mirrors the reference |
| Toast: saving, saved, error, sign-in notice, upgrade (402) | Matches, pixel diff 0 except an animated gif frame (62 px in step 2) |
| Tag drawer: add, remove, suggestions, recent tags (now in an extension-origin frame) | Matches, pixel diff 0 except 13 px of the gif in step 4 |
| Note editor (tiptap), saved with `PUT /objects/:id/note` (same frame) | Matches, pixel diff 0; note text verified in the library |
| Blob video error toast | Matches |
| Activation ping on install | Implemented; endpoint acknowledges |
| Instagram integration | Not implemented (needs optional host permission and mymind's overlay; omitted on purpose) |

## Backend endpoints used

`POST /objects` (types `WebPage`, `Media`, `Content`), `POST /objects/:id/attachments?type=screenshot`,
`POST /objects/:id/tags`, `DELETE /annotations/:id`, `PUT /objects/:id/note`, `GET /tags/recent?take=5`,
`GET /tags/manual`, `POST /browser-extension/activate`.

Backend changes, all in `rust/crates/core/src/`:

- `auth.rs`: a bearer value that is a valid, unexpired `mm_session` cookie session authenticates, with the cookie's
  expiry and sliding rules. Cookie and app-token paths are unchanged; tests in `rust/crates/core/tests/extension_auth.rs`.
- `api.rs`: `POST /browser-extension/activate` is acknowledged; `WebPage` and URL-only `Media` creates become link
  cards; `GET /tags/manual` answers JSON when asked for `application/json`.
- Known gaps: the `?type=screenshot` cover is not applied to the card, `Content` creates become notes without
  `source`, and `Media` by URL is left to the capture jobs.

## Security model

The saved page is hostile. What it can and cannot reach:

- **Cannot read account data.** Tag suggestions, recent tags, the card's tags and the note live in `drawer.html`, an
  extension-origin iframe the toast code only places and sizes. The page cannot read its DOM (cross-origin), see
  the tags in the toast's shadow root, or send runtime messages; its `postMessage` calls are ignored because the
  toast checks `event.source` against the frame, and the frame only accepts a `hover` boolean. The frame sends layout
  and timing signals to the page, never account data.
- **Cannot name a card or annotation.** The worker remembers, per tab, the card it created and the annotation ids the
  server returned for it. `addTag`, `deleteTag` and `saveObjectNote` take no card id; a `deleteTag` id must be one
  of the card's annotations. Every id that goes into a URL path must match the replica id format and is passed
  through `encodeURIComponent` (`modules/ids.js`), so `../objects/<id>` never reaches the server.
- **Cannot call privileged actions.** Messages are sorted by sender. Drawer actions need a frame of a tab
  (`frameId != 0`, `drawer.html`) with a bound card. `openNewTab` ignores the message and opens the redirect the worker
  last set for that tab, only if it is `http:` or `https:`. The test seams (`savePageOfTab`, `saveSelectionOfTab`,
  `saveObject` by tab id) are accepted only from the extension's own top-level `manifest.json` page, which is not
  web-accessible. `drawer.html` is web-accessible, so a page can embed it or navigate to it; neither gets a card or
  a seam.
- **Host snapshot.** Each request reads the host once, uses it for the cookie lookup and the URL, and refuses a URL
  whose origin differs. The token is sent with `credentials: 'omit'` and `redirect: 'error'`.
- **Per-library caches.** The tag cache and the card map are keyed by `host|sha256(session)` (the raw value is
  never stored) and cleared when the host or the `mm_session` cookie changes. The card map lives in
  `chrome.storage.session` (one `card:<tabId>` entry, readable only by the extension's own contexts), so a save
  still accepts tags and a note after Chrome stops the idle service worker; `tests/security.mjs` stops it on purpose.
- **`drawer.html` keeps a static URL.** `use_dynamic_url` would stop other pages from embedding it by its static
  address, but `chrome.runtime.getURL` does not return the dynamic address, so the content script could not place the
  frame. An embedded copy gets nothing: it has no bound card and the sender check refuses it.
- **Narrow surface.** `web_accessible_resources` lists `drawer.html`, `components/notification.css` and the icons,
  images and font the toast shows; no script. Only the default host is granted at install.

Server side, since Worker version `e83ecfcf` (2026-10-17): a `/hub` WebSocket upgrade signed in by the cookie needs
the server's own `Origin` (a sibling `*.octaviantocan.com` page used to get in), in the Worker through `Auth::handle`
and in the standalone server through `Auth::allows_upgrade`. A session used as a bearer is revoked only by the site's
normal cookie sign-out, not by a bearer-only sign-out; signing out on the site is how to cut the extension off.

## Origin header

The server refuses cross-origin writes, and extension requests carry `Origin: chrome-extension://...`. There is no
request-rewriting rule and no `declarativeNetRequest` permission. Instead `handle()` in `auth.rs` skips the
cross-origin write check when the request is signed in by a bearer and carries no `mm_session` cookie at all (an empty
one counts as present). The extension sends `credentials: 'omit'`, so it never has one. Cookie-authenticated writes
from other origins are still refused. A request with an invalid bearer and an extension `Origin` gets `403 cross-origin
write refused` instead of 401; the worker treats that answer as "session not valid" so the toast stays the same.
Tests: `rust/crates/core/tests/extension_auth.rs`.

## Tests

All tests create what they need under `evidence/extension/scratch/` (the seeded library with `mymind seed-db`, the
password user with `mymind auth-user`) and need only the built replica (`cd rust && cargo build --release -p mymind`)
and the copied mymind assets (setup step 1). Pages and their image are in `extension/tests/pages/`. Browser tests run
a copy of the extension in `scratch/ext-under-test/` whose manifest also grants `localhost` and `127.0.0.1`, because
optional host permissions cannot be granted headless. Chromium needs to run outside the Claude Code sandbox.

- `node extension/tests/units.mjs`: id validation, manifest permissions, host snapshot, token never sent to another
  origin, cache keyed by account. No browser.
- `node extension/tests/security.mjs`: a page forging tag and removal elements, forged and foreign ids, drawer
  frames in tabs without a card, `drawer.html` opened top-level, test seams from the drawer, oversize input, cache
  clearing on a session change; a recording stub shows no traversal reaches the server.
- `node extension/tests/e2e.mjs`: save page, tag add and remove, and the note in the library, with real writes from
  the extension to the replica (no Origin rule).
- `node extension/tests/flows.mjs <label> <extensionDir> [host]` takes the comparison screenshots; `node
  extension/tests/diff.mjs` diffs `ours-*` against `ref-*`. Only `flows.mjs ref` depends on `evidence/`: it loads the
  patched reference 3.3.0 from `evidence/extension/reference-local/`.
- `cd rust && cargo test -p mm-core -j 2 -- --test-threads=2` and `cargo clippy -p mm-core --all-targets -j 2`.
