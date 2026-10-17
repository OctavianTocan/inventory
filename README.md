# mymind reconstruction

A localhost replica of `https://access.mymind.com/everything`. It runs mymind's real front-end against a local
SQLite library seeded from your recordings. Notes, pasted text, URL placeholders, edits, tags, spaces and settings
persist locally; unsupported reads still use the recordings. No write is forwarded to mymind.

## Run it

```bash
SHARP_IGNORE_GLOBAL_LIBVIPS=1 npm ci   # Node developer tooling
rust/fetch-libvips.sh                   # once per clone, platform libvips
(cd rust && nice -n 19 cargo build --offline --release -j 2 -p mymind)
rust/target/release/mymind serve        # http://localhost:4321/everything
OFFLINE=1 rust/target/release/mymind serve # no network at all; static assets come from mirror/
```

The recordings of your library and mymind's front-end are committed (see [Private data](#private-data)), so a clone can serve them after building the Rust executable. The working database `data/mymind.db` is not: it stays on the computer that hosts the library,
which other machines reach over HTTPS (see [Hosting](#hosting)).
Node dependencies support developer tooling; runtime media uses the pinned libvips build, ffmpeg, ffprobe, Poppler and Chromium.
`SHARP_IGNORE_GLOBAL_LIBVIPS=1` keeps the extension screenshot-diff dependency on prebuilt binaries.

When `data/mymind.db` exists, desktop card grids, individual cards, trash, spaces, tags, changes and object quotas
come from SQLite. `POST /cards/grid` also hydrates current SQLite cards. Phone requests keep their device-specific
recordings because the renderer currently generates desktop markup. Unsupported reads retain the existing
`replay/` then `capture/` fallback; without a database, the entire server retains that fallback and ignores writes.

Set `STORE_DB=/path/to/library.db` to select another database. `STORE_READONLY=1` enables store reads while
acknowledging writes without persisting them or broadcasting write events. Grid hydration still works in that mode.
Search, suggestions and smart-space query checks use SQLite on both devices. In `STORE_READONLY=1`, exact
recorded search/suggestion/query-check requests retain their device/run replay, while unrecorded requests use
SQLite without persisting query-check membership changes. Normal writable mode always searches the store.
Unrecorded API calls are logged as `[miss]`, media without a recording as `[placeholder]`.

To build the initial database, run `rust/target/release/mymind seed-db`. Rebuilding replaces an existing database, including its
local edits. The seed also imports response metadata and the recorded space-detail query-check times; card and
space bodies are generated from the stored rows. Older phase 1 databases import that metadata on first startup.

Card and space writes use one transaction, regenerate affected card markup, append change-log entries and broadcast SignalR events
to connected `/hub` clients after commit. Polling returns the latest change per object so a delete cannot be undone
by an older queued update in the client. Drafts stay outside the main grid until committed. New or edited desktop
cards have generated editable details and pinned previews; unchanged detail/template recordings retain their
existing replay path. Shared cards have local URLs with a 24-hour expiry and revocation.

URL saves return `201` immediately, then `captureLink` enriches the placeholder in the background. Capture reads
OpenGraph, Twitter, JSON-LD and public oEmbed metadata, detects social posts, videos, repositories, apps, products,
articles and PDFs, downloads covers and sanitizes post HTML. Web pages without a usable cover and explicitly saved
Screenshots use a fresh headless Chromium context with the default user agent; page fetches identify themselves as
`mymind-replica/1.0`, since some sites (Wikimedia) refuse requests without a user agent. Failures before page metadata retain a
readable "blocking us" card and skip AI, which would otherwise invent a summary from the bare URL. Original source
URLs and user edits made during capture are preserved.
Online capture permits public destinations only, including every redirect, media/oEmbed request and screenshot
resource. Rust capture validates every DNS answer before its pinned connection and limits streamed wire and decoded bodies. Screenshot
resources share a 100 MiB wire-plus-decoded budget; native browser networking uses an unresolvable proxy with
QUIC disabled, and WebRTC/WebTransport are removed from every frame. New links start as a `Placeholder` (flags 0).
The 2026-10-15 APK save in `evidence/live-saves/2026-10-15-apk-link.jsonl` supplies the exact saving and screenshot
templates: the fetched WebPage has flags 2097153; after AI reports no resource text and the screenshot fails,
flags become 6291457 and the screenshot spinner remains. The hub uses the recorded event shapes and order.
These stages are compared byte for byte by Rust `check_render` and exercised offline by `check_capture`; success,
early-failure and text-bearing screenshot-failure paths retain their previous behavior without claiming live parity.

Uploads use `POST /uploads/authorize`, bearer-authorized chunked `PUT /uploads/:id`, then `POST /objects` with
`{type:"Media",uploadId}`. PNG/JPEG/WebP and other decoded raster images, videos and PDFs become local cards.
Originals live in `data/media/<account>/<card-tail>/<blob>.<ext>`; recorded-style rendition URLs are generated on
demand with libvips, with cached resized/cropped WebP/JPEG/PNG, PDF pages and video posters/previews. Local files
take precedence over captured media; streaming originals and videos support byte ranges. Set `STORE_MEDIA` to
override the media root (the default is beside `STORE_DB`). Clipboard multipart image/PDF paste and inline editor
image attachments (raw bytes, completed upload IDs or public remote URLs) use the same local media storage.
HEAD and byte ranges stat and stream files without loading whole originals. Video renditions and previews publish
by atomic rename after successful processing. Temporary uploads have a shared 1 GiB byte limit, expire after
24 hours, and are swept at writable startup and upload operations. Claiming an upload removes its temporary
original after commit; its row retains idempotency until expiry and attachment reuse resolves permanent media.

AI runs after capture, after saved notes and on `POST /cards/:id/analyze`. The native environment loader reads `.env` when present. Set `GEMINI_API_KEY` for hosted summaries and 768-dimension embeddings;
`TYPESAFE_API_KEY` optionally selects final tags with Jev. Cover images are converted to PNG for analysis. The
library's tag vocabulary is passed to `analyze`; generic labels are filtered, manual tags are preserved, AI tags
use flag `2`, summaries appear in the detail TL;DR and embeddings persist in the SQLite `embeddings` BLOB table.
Missing keys and API failures retain the card and clear processing state. `STORE_READONLY=1` disables all capture
and AI work. `OFFLINE=1` allows only loopback capture fixtures and explicitly configured loopback AI stubs.
Article reader edits schedule analysis and invalidate in-flight results based on the effective reader content.
Summaries and tags use text for text-rich cards; images inform visual cards and items with under 200 text
characters. Post analysis supplies headlines of at most eight words, applied only to titles the user hasn't
edited. AI tags use normalized identity, existing vocabulary spelling, type-label filtering and a ten-tag cap.

Search uses FTS5 over titles, descriptions, summaries, annotations, URLs/domains, prose/current reader content,
card notes, capture text and OCR. It supports `type:`, `tag:`/`label:`/`#`, `site:`/`domain:`, quotes, prefix `*`,
negation, boolean groups, date filters, formats and the client's action/completed filters. Smart spaces evaluate
their filters at creation and refresh memberships on query checks, retaining manual additions and exclusions.
`/embeddings/search` embeds queries with hosted Gemini; `/embeddings/related/:id` compares stored vectors without
an API call. Missing keys, disabled hosted calls offline and provider failures return text matches or empty
semantic matches. See [search contracts and measured recording differences](docs/search.md).

Backfill historical text embeddings with `rust/target/release/mymind embed-library` (estimate only), then explicitly add
`--run`; `--db /path/to/library.db` chooses a database and `--limit 10` selects a trial. The tool does not load
`.env`; supply API keys in the process environment before using
`rust/target/release/mymind embed-library --db /path/to/library.db --limit 10 --run`. It retries 429/5xx, commits each batch, skips existing vectors on
resume, and reports provider token usage when available. Empty text-only documents are reported and skipped.
In a worktree with shared read-only `data/`, always select a temporary database; never backfill the symlink target.

Install dependencies with `npm ci` (see [Run it](#run-it)). PDF/video previews require host `pdftoppm` (Poppler), `ffmpeg` and `ffprobe`.
Page screenshots use `VERIFIER_CHROMIUM`, default `/usr/lib/chromium/chromium`, without touching the verifier's
running browser. New screenshot output still needs host/browser verification before making a parity claim.
Permanent delete and empty trash have no captured client calls, so only trash and restore are implemented.

Run the local data and HTTP checks with the Rust binary built first:

```bash
node tools/check-reads.mjs
node tools/check-auth.mjs
node tools/check-app-api.mjs
node tools/check-download.mjs
(cd rust && nice -n 19 cargo test --offline -j 2 -p mm-core --test check_store --test check_render -- --test-threads=2)
(cd rust && nice -n 19 cargo test --offline -j 2 -p mymind --test check_writes --test check_search --test check_media -- --test-threads=2)
```

The HTTP checks copy the verification database (`data/verify.db`, see [Verify against
live](#verify-against-live-11)) into temporary files and start isolated Rust servers on OS-assigned loopback
ports. They never use port 4321. The HTTP harness requires loopback sockets; `--in-process` now reports the
corresponding Rust test command. `check-auth` runs a stand-in Google with synthetic keys, token endpoint and
JWKS; `check-download` uses scratch sign-in data. Neither contacts Google or loads the user's `.env`.

Native capture assertions run with `cargo test -p mymind --test check_capture --test check_capture_http
--test capture_browser --test check_hosted_media` from `rust/`, with the same offline/job/thread limits above. They use synthetic local
HTTP and hosted-provider stubs, generated media and isolated databases. The browser target launches its own
Chromium and tests local TCP/UDP sinks with an unguarded positive control. It needs Unix socket IPC as well as
loopback TCP/UDP; an IPC-restricted sandbox fails the control rather than skipping it. Native media checks use
real filesystem/media processing with stubbed media executables. Capture and browser assertions run entirely in Rust after the native Chromium gates passed. The capture fixture CLI is
`mymind capture-fixtures`; the JS in-process fixture exports are retired. `sharp` remains a developer dependency
for `extension/tests/diff.mjs`, which compares extension screenshots. Retirement dispositions are
recorded in `evidence/r13/stage-b-report.md`. Browser/REA verification remains a separate host operation.

## Sign-in (remote access)

Sign-in is off by default: the server trusts everyone who can reach it, which is fine on localhost and is how the
desktop app and every verification run work. With sign-in off the server refuses any request that arrives through
a Cloudflare tunnel (`cf-connecting-ip`), so publishing it by mistake exposes nothing.

To reach the library from other computers, run the server with Google sign-in and put it behind HTTPS:

| Variable | Effect |
|---|---|
| `AUTH=google` | Turns sign-in on. Requires the three below. |
| `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` | A Google OAuth "Web application" client whose redirect URI is `{PUBLIC_URL}/google/oauth2-callback`. |
| `OWNER_EMAIL` | The only Google account let in; any other gets an "account not found" message. |
| `PUBLIC_URL` | The address browsers use, default `http://localhost:PORT`. An `https://` URL makes cookies `Secure`. |
| `AUTH_DB` | Sessions and password users, default `data/auth.db`. |
| `LOCAL_PORT` | Also serves this computer without sign-in on `127.0.0.1:LOCAL_PORT`; tunnelled requests there get 403. |

Signed-out visitors on this machine get mymind's own sign-in page (`capture/signin/`, recorded signed out by
`node tools/capture-signin.mjs`); its Apple button does nothing. Signed-out visitors through a Cloudflare tunnel
(`cf-connecting-ip`) get a plain sign-in page of the replica's own instead, with no mymind name, logo, favicon or
assets, `noindex`, and a `robots.txt` that disallows everything: a public copy of a real company's sign-in page is
what phishing filters flag. After sign-in everything is mymind's front-end as usual. Both pages' Google buttons open
a popup; the server runs the OAuth flow (PKCE, `state`, `nonce`), verifies Google's ID token and sets a 30-day
HttpOnly session cookie. API requests and the `/hub` socket without a session get 401, cross-origin writes 403.
`/signout` ends the session.

Password users (test accounts, or a browser without Google) sign in at `/signin/password`, which exists only once a
user has been added; mymind's own sign-in page stays as recorded and does not link to it, the plain one does. A
password session reaches the same library as the owner. Ten wrong passwords for a username, or 30 from one address, lock it for 15
minutes.

```bash
rust/target/release/mymind auth-user add tester --save-env TEST_USER   # generates a password into .env as TEST_USER_NAME/_PASSWORD
read -rs pw && printf '%s\n' "$pw" | rust/target/release/mymind auth-user add alice   # your own password (16+ characters)
rust/target/release/mymind auth-user list
rust/target/release/mymind auth-user remove tester                     # also signs that user out everywhere
```

### Hosting

Since 2026-10-17 `https://mymind.octaviantocan.com` is served by the Cloudflare Worker, written in Rust in
`rust/crates/worker/`: each library lives in its own Durable Object with its media in R2, behind the library's own
sign-in (`docs/cloudflare-plan.md` has the setup, the import and what was checked). Deploy a change, the Worker and
the media container image together, from the repository root.
`npm ci --prefix rust/crates/worker` installs the pinned Wrangler 4.151.1 tooling:

```bash
npm --prefix rust/crates/worker run deploy
```

Until 2026-10-24, **mini-002** (`mini-002-netcup` on Tailscale, a Debian server) keeps running the previous host from
the checkout `~/apps/mymind-reconstruction`, behind the Cloudflare tunnel `mymind`, as the fallback: the Worker's
route on `mymind.octaviantocan.com/*` takes every request first, and deleting the route sends traffic back to the
tunnel. Reach the machine over Tailscale (`ssh mini-002-netcup`). That checkout holds its `data/mymind.db` and
`data/auth.db`, plus `.env`, the capture videos and `downloads/`, none of which are in Git; the owner library on the
Worker was imported from its backup at 00:58 on 2026-10-17, and nothing written on mini-002 since then reaches the
Worker.

Since 2026-10-17 11:02 mini-002 runs the Rust server (`mymind serve` and `mymind download-server`), and its checkout
is on branch `cloudflare`. To deploy a change, push that branch (`receive.denyCurrentBranch=updateInstead` updates
the checked-out files on push), copy the server and the libvips it links (both built here; Git leaves them out), and
restart the services:

```bash
git remote add mini-002 mini-002-netcup:apps/mymind-reconstruction   # once
git push mini-002 cloudflare
scp rust/target/release/mymind rust/libvips/linux-x64/libvips-cpp.so.8.17.3 \
  mini-002-netcup:apps/mymind-reconstruction/rust/target/release/
ssh mini-002-netcup 'systemctl --user restart mymind-replica mymind-download'
# after a re-capture, also copy the videos Git leaves out:
rsync -a -m --include='*/' --include='*.mp4' --include='*.webm' --include='*.mov' --exclude='*' \
  capture/ mini-002-netcup:apps/mymind-reconstruction/capture/
```

To move the host to another machine, stop and disable the services on the old one first (two connectors would split
the tunnel's traffic, and the database must have one writer), copy `data/`, `.env`, `downloads/` and
`~/.cloudflared/<tunnel-id>.json`, enable lingering (`sudo loginctl enable-linger $USER`), then run
`deploy/install.sh` there.

#### Setting up a host

`deploy/install.sh` runs the replica as systemd user services that start at boot: the server with sign-in on
`127.0.0.1:4330`, the desktop app downloads on `127.0.0.1:4332`, and a Cloudflare tunnel that publishes both at your
hostname. There is no Cloudflare Access in front (the user's decision, 2026-10-16): every request meets the library's
own sign-in, which admits only `OWNER_EMAIL` through Google and the password users. It needs `cloudflared` logged in
(`cloudflared tunnel login`), the sign-in variables in `.env`, and a Google client whose redirect URIs include
`https://<hostname>/google/oauth2-callback`.

Create the tunnel, package the app and install:

```bash
cloudflared tunnel create mymind                          # once; prints the tunnel id
cloudflared tunnel route dns mymind mymind.example.com    # once
deploy/package-app.sh https://mymind.example.com          # downloads/mymind-linux-x86_64.tar.gz, about 5 min the first time
deploy/install.sh mymind.example.com <tunnel-id>
journalctl --user -u mymind-replica -u mymind-tunnel -f   # logs
systemctl --user stop mymind-replica                      # stops the tunnel too
```

The service is the one process that writes `data/mymind.db`. `LOCAL_PORT=4331` serves the host from the same
process without sign-in (loopback only; tunnelled requests are refused), and the desktop app opens that port by
default, so an app on the host and the hosted site share one library and its live updates.

#### Downloading the desktop app

`https://<hostname>/download` serves the desktop app, packaged to open the hosted library, to anyone signed in to
the library (Google or password). The tunnel sends `/download` to `mymind download-server`, which itself checks
each request's session (`data/auth.db`), so the files stay private even if the tunnel's routing changes. Visitors
without a session are sent to the sign-in page; open `/download` again after signing in. It logs why it refused a
request (`journalctl --user -u mymind-download`). There is a Linux build only: `deploy/package-app.sh` builds for the
machine it runs on.

Run `deploy/package-app.sh` again after changing `desktop/`.

#### Testing the hosted library

Scripts and agents sign in as the `TEST_USER_*` password user (`TEST_USER_NAME` and `TEST_USER_PASSWORD` in `.env`),
never with the owner's Google login:

```bash
node tools/check-hosted.mjs             # signed-out surface, test-user sign-in, library reads and /download over HTTP
node tools/check-hosted.mjs --browser   # also signs in with headless Chromium; screenshot in evidence/hosted/
```

## Desktop app

`desktop/` wraps the replica in a native window (Tauri 2). The app uses the server already listening on `PORT`
(default 4331, which the hosted service serves on its host), or starts the Rust server (`mymind serve`) from this
checkout and stops it when the window closes; with `MYMIND_URL` it opens that hosted library instead of starting a server.
It stays off `:4321`, which belongs to verification.

### Install

1. Install Rust (`curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`) and the system libraries from
   [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/). On Linux, also install GStreamer's good
   plugins; without them WebKitGTK cannot create an audio sink, leaves the grid blank and aborts within a minute:

   ```bash
   sudo pacman -S --needed webkit2gtk-4.1 base-devel gst-plugins-good                 # Arch
   sudo apt install libwebkit2gtk-4.1-dev build-essential gstreamer1.0-plugins-good   # Debian/Ubuntu
   ```

2. Build the server and the app (about 5 min each the first time, about 1 min after a change):

   ```bash
   rust/fetch-libvips.sh                             # once: the libvips build the server links
   (cd rust && cargo build --release)                # the server, rust/target/release/mymind
   (cd desktop/src-tauri && cargo build --release)   # the app
   ```

3. Run `desktop/src-tauri/target/release/mymind-desktop`. The binary remembers this checkout's path, so it runs
   from any directory as long as the checkout stays where it was built (or `MYMIND_ROOT` points to it).

To start it from your app launcher on Linux, add a desktop entry. Run this from `desktop/src-tauri`:

```bash
cat > ~/.local/share/applications/mymind.desktop <<EOF
[Desktop Entry]
Type=Application
Name=mymind
Exec=$PWD/target/release/mymind-desktop
Icon=$PWD/icons/icon.png
Categories=Utility;
EOF
```

### On another machine

Your library lives in `data/mymind.db` on the computer that hosts it and is not in Git. From another machine, use
the hosted address: in a browser, or in the desktop app from `https://<hostname>/download` (see [Downloading the
desktop app](#downloading-the-desktop-app)), which needs no checkout, Node or Rust:

```bash
tar -xzf mymind-linux-x86_64.tar.gz
mymind/install.sh    # ~/.local/bin/mymind and a launcher entry; needs webkit2gtk-4.1 and gst-plugins-good
```

The app signs in with Google inside its window the first time and keeps the session for 30 days. A build from a
checkout opens the hosted library the same way with `MYMIND_URL=https://<hostname>`.

A clone elsewhere still runs: build the app as above and start it. Without a database it shows the recorded library
and ignores edits; `rust/target/release/mymind seed-db` gives it a writable copy seeded from the recordings, which lacks what
changed on the host since. A clone that kept `data/mymind.db` from when the database was committed keeps that copy,
but its edits never reach the host.

### Configure

| Variable | Default | Effect |
|---|---|---|
| `PORT` | `4331` | Port of the replica the window opens, and of the server the app starts. |
| `MYMIND_URL` | none; the build's value | A hosted library to open when nothing listens on `PORT`, instead of starting a server. Set at build time, it becomes the default (`deploy/package-app.sh`). |
| `MYMIND_ROOT` | the checkout it was built in | The checkout the server runs in (recordings, `data/`, `.env`). |
| `MYMIND_SERVER` | `rust/target/release/mymind` in the checkout | The server binary it starts. |
| `STORE_DB`, `STORE_MEDIA`, … | as for `mymind serve` | Passed through to the server it starts. |

The server reads `.env` in the checkout, so AI features work in the app when they work in `mymind serve`.

### How it behaves

- Links to other sites open in your default browser; the window only shows the replica.
- Files dropped on the window go through mymind's own upload code, as in a browser. WebKitGTK currently gives pages
  an empty `dataTransfer.files` ([WebKit bug 323277](https://bugs.webkit.org/show_bug.cgi?id=323277)), so on Linux
  the app reads dropped files itself and hands them to the page as a `drop` event (`src/file_drop.rs`).
- The app does not define `mymind.native`. mymind's Mac and phone apps use that bridge to hand card views, sheets
  and menus to native code; without it the page runs as the browser UI that the REA runs verify.

### Troubleshooting

- **Grid stays blank, log says `GStreamer element autoaudiosink not found`:** install `gst-plugins-good` (step 2).
- **Window says the checkout "has no copy of your library":** run `git pull` in it; clones made before the library
  was committed lack it. Or point `MYMIND_ROOT` at a complete checkout.
- **Window says it "could not run" the server, or that the server exited:** build it (Install, step 2) or set
  `MYMIND_SERVER` to its path, and run `rust/target/release/mymind serve` in the checkout to see the server's error.
- **Testing drops without a file manager** (e.g. on Xvfb): `cargo run --release --example drag_source -- <file>`
  opens a small window whose button drags that file.

## Re-capture

Needs Chrome with remote debugging on port 9222 and a logged-in mymind tab open.

```bash
npm run capture -- --media-cards 300 --detail-cards 30   # add --videos to include mp4s
npm run capture -- --media-only                            # only the card media still missing (add --videos for mp4s)
npm run capture -- --details-only                          # detail views, article readers and detail media of every card lacking them
npm run mirror                                             # static JS/CSS/fonts for OFFLINE=1
node tools/capture-signin.mjs                              # the signed-out sign-in page, from a cookie-less context
```

Every pass only sends GETs. A full pass replaces the card list and shell, and adds to the recorded detail views,
readers and media instead of replacing them. `--details-only` reads the card list from `data/mymind.db` (`--db`
picks another).

## Verify against live, 1:1

```bash
OFFLINE=1 STORE_READONLY=1 STORE_DB=data/verify.db npm run serve   # replica on :4321, as verification requires
npm run drive -- --profile desktop-light --suite ui     # one profile x suite: live, then replica, then REA diffs
npm run verify                             # every profile (desktop light/dark, iPhone, Android) x suite (ui, cards)
npm run verify -- --reseed                 # after the live library changed: all live passes, reseed, then replica passes
npm run verify -- --replica-only           # after a code change: replica passes against the recorded live passes
npm run coverage                           # REA's fail-closed verdict over all runs: ready / partial / failed
```

Verification never touches your working library: its server reads `data/verify.db`, a database seeded from the
recordings that `tools/verify-all.mjs` creates on first use. `--reseed` needs no server running on `:4321`: it
starts one, reseeds `data/verify.db` from the fresh recordings and restarts the server before the first replica
pass.

Needs the agent Chromium on CDP `:9222` signed in to mymind; only its two mymind session cookies and the page's
localStorage are read, in memory, to sign in the verifier.

Every verification server must use `STORE_DB=data/verify.db`, `STORE_READONLY=1`, because live writes are
intercepted and sent to the replica as well, and `OFFLINE=1`, because `npm run coverage` only accepts runs that
prove the replica renders without mymind's servers. `tools/verify-all.mjs` sets all three when starting a server. If a server is already running, restart it yourself in
that mode before verifying; the verifier does not change the environment of an existing process.

The Rust backend (`rust/`) builds with `rust/fetch-libvips.sh` once (it downloads the libvips build sharp ships), then
`cargo build --release` in `rust/`. It is verified the same way: set `MYMIND_SERVER=rust/target/release/mymind` and
every tool that starts a server (`drive`, `verify`, the HTTP checks, `check-download`) selects that Rust binary.
By default all tools select `rust/target/release/mymind`, and `npm run coverage` judges the runs against the Rust sources. An already running listener must match the selected Rust executable; hosted modes also validate the
proxy and backend digests.

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
4. Record the data the UI needs (`tools/capture.mjs`) and replay it (`mymind serve`).
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

- Recorded detail views cover every card that existed at the last `--details-only` pass. A card without one
  (e.g. saved on live mymind afterwards) gets a generated detail whose body is its grid card.
- Generated details and rich prose nodes have route/data coverage, but still need browser and REA parity checks.
- Uncaptured historical media retains its placeholder fallback: 6 of the 511 detail media failed to download, all
  reader images of one card ("Sub-agents are just wrong"); every grid video is present. Generated AI thumbnails
  remain later work.
- The sign-in page is not part of the REA matrix, whose verifier is signed in. Compared signed out under a frozen
  clock with its animated WebGL background hidden, it matches live at 390x844 (0 pixels differ, also for "account
  not found") and at 1440x900 differs in 1,505 edge pixels by at most 3 levels, the same noise as live against
  itself. The background runs mymind's own mirrored script, but its phase is not frozen.
- New cards saved on live mymind reach the replica only through a re-capture and reseed (`npm run verify --
  --reseed` for verification, `rust/target/release/mymind seed-db` for the working library, which replaces local edits).
- Local search approximates the live ranking engine; historical cards without vectors need an explicit hosted
  embedding backfill. Text-only backfill skips historical cards with no usable text; new visual analysis can
  embed their image. Browser and screenshot parity have not been claimed for generated search results.

## Private data

`capture/` and `replay/` hold your own library (cards and recorded responses) and `mirror/` holds mymind's
front-end. All three are committed so that a clone runs by itself, which makes this repository private data: keep it
private. Before committing them, run `npm run bundle`: it moves replay bodies no recording uses into the gitignored
`replay/unreferenced/` and snapshots `data/` into `library/` as a local backup.

Never committed: `.env` (API keys), `data/` (the working database), `library/` (its snapshot), recorded videos
(`capture/**/*.mp4`, `*.webm`, `*.mov`), `evidence/` (verification screenshots) and `replay/unreferenced/`. Commits
made before the database left Git still contain `library/mymind.db`; removing it from them means rewriting history.

Replay recordings keep only content-type, CORS and app `x-` headers, so no cookies are stored; recorded pages still
carry mymind's per-session authenticity token, which is useless without the session cookie.
