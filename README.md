# Inventory

Inventory (called ThyMind until 5 November 2026) is a private library of things you save: an Android app, a web
library and an MCP server for AI agents. The app's name is Inventory everywhere a person reads it; internal names
(`mymind` binary, `com.octaviantocan.mymind` package, `thymind_*` MCP tools, Cloudflare Worker and Docker names)
stay as they are so installed phones, sign-ins and connected agents keep working.

It began as a localhost replica of `https://access.mymind.com/everything`. It runs mymind's real front-end against a local
SQLite library. Notes, pasted text, URL placeholders, edits, tags, spaces and settings persist locally. No write
is forwarded to mymind.

## Run it

```bash
git submodule update --init             # once per clone: the AI kit shared with Pully, vendor/ai-kit
SHARP_IGNORE_GLOBAL_LIBVIPS=1 npm ci   # Node developer tooling
rust/fetch-libvips.sh                   # once per clone, platform libvips
(cd rust && nice -n 19 cargo build --offline --release -j 2 -p mymind)
rust/target/release/mymind serve        # http://localhost:4321/everything
OFFLINE=1 rust/target/release/mymind serve # no network at all; static assets come from mirror/
```

mymind's front-end is committed (see [Private data](#private-data)). The working database `data/mymind.db` is not: it stays on the computer that hosts the library,
which other machines reach over HTTPS (see [Hosting](#hosting)).
Node dependencies support developer tooling; runtime media uses the pinned libvips build, ffmpeg, ffprobe, Poppler and Chromium.
`SHARP_IGNORE_GLOBAL_LIBVIPS=1` keeps the extension screenshot-diff dependency on prebuilt binaries.

Pages, cards, trash, spaces, tags, changes, search and object quotas come from SQLite.

Set `STORE_DB=/path/to/library.db` to select another database. `STORE_READONLY=1` enables store reads while
acknowledging writes without persisting them or broadcasting write events.

To build the initial database, run `rust/target/release/mymind seed-db`. Rebuilding replaces an existing database, including its
local edits. Card and space bodies are generated from the stored rows.

Card and space writes use one transaction, regenerate affected card markup, append change-log entries and broadcast SignalR events
to connected `/hub` clients after commit. Polling returns the latest change per object so a delete cannot be undone
by an older queued update in the client. Drafts stay outside the main grid until committed. New or edited desktop
cards have generated editable details and pinned previews. Shared cards have local URLs with a 24-hour expiry and revocation.

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
early-failure and text-bearing screenshot-failure paths retain their previous behavior.

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
`OPENAI_API_KEY` optionally selects final tags with OpenAI's Decisions API (`gpt-6-luna`), and transcribes saved video
and audio files (Audio cards: MP3, M4A, AAC, Ogg/Opus, WAV, FLAC, AMR) once with OpenAI's `gpt-transcribe` ($0.0045 a minute): ffmpeg (the media container on Cloudflare) pulls the
sound out as mono Opus (up to OpenAI's 25 MB, about 2 hours), and the words land in `extras.transcript`, the analysis,
search and the app's reader. Cover images are converted to PNG for analysis. The
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
semantic matches. The Android app's search adds cards close in meaning (`/app/v1/search?semantic=1`), and
`mymind search-eval queries.jsonl` measures how well each mode finds the cards you meant. See
[search contracts and measured recording differences](docs/search.md).

Backfill historical text embeddings with `rust/target/release/mymind embed-library` (estimate only), then explicitly add
`--run`; `--db /path/to/library.db` chooses a database and `--limit 10` selects a trial. `.env` supplies API keys
the process environment does not already set, as in
`rust/target/release/mymind embed-library --db /path/to/library.db --limit 10 --run`. It retries 429/5xx, commits each batch, skips existing vectors on
resume, and reports provider token usage when available. Empty text-only documents are reported and skipped.
In a worktree with shared read-only `data/`, always select a temporary database; never backfill the symlink target.

Install dependencies with `npm ci` (see [Run it](#run-it)). PDF/video previews require host `pdftoppm` (Poppler), `ffmpeg` and `ffprobe`.
Page screenshots use `VERIFIER_CHROMIUM`, default `/usr/lib/chromium/chromium`, without touching the verifier's
running browser.
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

The HTTP checks copy the test database (`data/verify.db`, seeded from the recordings with
`mymind seed-db --db data/verify.db`) into temporary files and start isolated Rust servers on OS-assigned loopback
ports. They never use port 4321. The HTTP harness requires loopback sockets; `--in-process` now reports the
corresponding Rust test command. `check-auth` runs a stand-in Google with synthetic keys, token endpoint and
JWKS; `check-download` uses scratch sign-in data. Neither contacts Google or loads the user's `.env`.

Native capture assertions run with `cargo test -p mymind --test check_capture --test check_capture_http
--test capture_browser --test check_hosted_media` from `rust/`, with the same offline/job/thread limits above. They use synthetic local
HTTP and hosted-provider stubs, generated media and isolated databases. The browser target launches its own
Chromium and tests local TCP/UDP sinks with an unguarded positive control. It needs Unix socket IPC as well as
loopback TCP/UDP; an IPC-restricted sandbox fails the control rather than skipping it. Native media checks use
real filesystem/media processing with stubbed media executables. Capture and browser assertions run entirely in Rust after the native Chromium gates passed. The capture fixture CLI is
`mymind capture-fixtures`.

## Sign-in (remote access)

Sign-in is off by default: the server trusts everyone who can reach it, which is fine on localhost and is how the
desktop app and every verification run work. With sign-in off the server refuses any request that arrives through
a Cloudflare tunnel (`cf-connecting-ip`), so publishing it by mistake exposes nothing.

To reach the library from other computers, run the server with Google sign-in and put it behind HTTPS:

| Variable | Effect |
|---|---|
| `AUTH=google` | Turns sign-in on. Requires the three below. |
| `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` | A Google OAuth "Web application" client whose redirect URI is `{PUBLIC_URL}/google/oauth2-callback`. |
| `OWNER_EMAIL` | Google account that is always let in. Its library mapping is left as it is. |
| `ALLOWED_GOOGLE_DOMAIN` | Also admit a verified address whose host is exactly this domain. Unset means `octaviantocan.com`. Empty means only `OWNER_EMAIL`. A subdomain or a longer suffix is not a match. |
| `PUBLIC_URL` | The address browsers use, default `http://localhost:PORT`. An `https://` URL makes cookies `Secure`. |
| `AUTH_DB` | Sessions and password users, default `data/auth.db`. |
| `LOCAL_PORT` | Also serves this computer without sign-in on `127.0.0.1:LOCAL_PORT`; tunnelled requests there get 403. |

Signed-out visitors get the replica's own plain sign-in page, with no mymind name, logo, favicon or assets,
`noindex`, and a `robots.txt` that disallows everything. After sign-in everything is mymind's front-end. The Google
button opens a popup; the server runs the OAuth flow (PKCE, `state`, `nonce`), verifies Google's ID token and sets a
30-day HttpOnly session cookie. API requests and the `/hub` socket without a session get 401, cross-origin writes 403.
`/signout` ends the session.

Password users (test accounts, or a browser without Google) sign in at `/signin/password`, which exists only once a
user has been added; the plain sign-in page links to it. A password session reaches the same library as the owner.
Ten wrong passwords for a username, or 30 from one address, lock it for 15 minutes.

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

The deploy also builds the AI agents' container (`rust/agent/Dockerfile`), where each account's Claude Code and
Codex run, from the AI kit in the `vendor/ai-kit` submodule (private OctavianTocan/ai-kit); run `git submodule update --init` first.

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

```

To move the host to another machine, stop and disable the services on the old one first (two connectors would split
the tunnel's traffic, and the database must have one writer), copy `data/`, `.env`, `downloads/` and
`~/.cloudflared/<tunnel-id>.json`, enable lingering (`sudo loginctl enable-linger $USER`), then run
`deploy/install.sh` there.

#### Setting up a host

`deploy/install.sh` runs the replica as systemd user services that start at boot: the server with sign-in on
`127.0.0.1:4330`, the desktop app downloads on `127.0.0.1:4332`, and a Cloudflare tunnel that publishes both at your
hostname. There is no Cloudflare Access in front (the user's decision, 2026-10-16): every request meets the library's
own sign-in, which admits `OWNER_EMAIL` and a verified address at `ALLOWED_GOOGLE_DOMAIN` through Google, and the
password users. It needs `cloudflared` logged in
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
  and menus to native code; without it the page runs as the browser UI.

### Troubleshooting

- **Grid stays blank, log says `GStreamer element autoaudiosink not found`:** install `gst-plugins-good` (step 2).
- **Window says the checkout "has no copy of your library":** run `git pull` in it; clones made before the library
  was committed lack it. Or point `MYMIND_ROOT` at a complete checkout.
- **Window says it "could not run" the server, or that the server exited:** build it (Install, step 2) or set
  `MYMIND_SERVER` to its path, and run `rust/target/release/mymind serve` in the checkout to see the server's error.
- **Testing drops without a file manager** (e.g. on Xvfb): `cargo run --release --example drag_source -- <file>`
  opens a small window whose button drags that file.

## How it was reverse engineered

1. Find the page: `rea list-browser-targets http://127.0.0.1:9222`.
2. Capture it passively, with the CDN origin allowed (otherwise every script is filtered out):
   `rea inspect-web-page <cdp> <target> --allowed-origins https://access.mymind.com --allowed-origins https://static.accelerator.net --include-script-sources --include-json-body-shapes --include-accessibility-text`
3. Export and statically analyse the scripts:
   `rea export-web-scripts evidence/page-capture.json evidence/scripts` then
   `rea analyze-javascript-application evidence/scripts`.
4. The data the UI needed was recorded, then replaced by the local database (`mymind serve`).

Hopper and Ghidra are not involved. They read native machine code; this app ships readable JavaScript.

## What the app is

- Server-rendered HTML plus ~80 unbundled, unminified ES modules from `static.accelerator.net/134/0.109.0/`.
  No framework and no build step. Vendor code: Lit, TipTap 3, GSAP, OGL (WebGL), Sortable, Lottie, SignalR, msgpack.
- `GET /cards` streams msgpack values shaped `{ html, json }`: the server renders every card, the client only
  lays them out (`grid.js`, `modules/layouts.js`).
- Opening a card fetches server-rendered HTML from `/cards/:id/detail`.
- Live updates come from `/changes?since=` (msgpack) and a SignalR hub at `/hub`.

## Known gaps

- Local search approximates the live ranking engine; historical cards without vectors need an explicit hosted
  embedding backfill. Text-only backfill skips historical cards with no usable text; new visual analysis can
  embed their image.

## Private data

`mirror/` holds mymind's front-end and is committed, which makes this repository private data: keep it private.
`npm run bundle` snapshots `data/` into `library/` as a local backup.

Never committed: `.env` (API keys), `data/` (the working database), `library/` (its snapshot) and `evidence/`
(verification screenshots). Commits made before the database left Git still contain `library/mymind.db`; removing
it from them means rewriting history. Git history still contains the old `capture/` and `replay/` recordings.
