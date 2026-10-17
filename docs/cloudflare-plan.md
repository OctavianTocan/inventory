# Hosting the library on Cloudflare, one database per person

Status (2026-10-17): deployed. The Worker `mymind` serves `https://mymind.octaviantocan.com` (version
`7a727490-5a16-494b-a99d-5407cb527868`). A Worker route, `mymind.octaviantocan.com/*` on the `octaviantocan.com`
zone, takes every request before the tunnel's DNS record is reached, so the tunnel's record stays in place as the
fallback: deleting the route sends traffic back to mini-002. The user approved Workers Paid, R2 for the media and
website code, the database per person on their Cloudflare account, and the hostname. Cloudflare facts were checked
in its docs on 2026-10-16.

What was uploaded, from `evidence/upload.log`:

| What | Contents | Check |
|---|---|---|
| `static/` | 368 objects, 17.5 MB | object count and bytes match |
| `downloads/` | 2 objects, 2.9 MB | object count and bytes match |
| owner library | 2,278 cards, backed up from mini-002 at 00:58 | every table's digest matches the Worker's |
| test library | 2,280 cards | every table's digest matches the Worker's |
| each library's media | 6,575 objects, 2.84 GB | object count and bytes match |

R2 holds 13,520 objects, 5.70 GB in all. The import left one password user (`tester`) and no sessions, so the owner
signs in with Google again on the first visit; that sign-in hasn't been tried on the Worker yet.
`node tools/check-hosted.mjs --browser` passes every check as the test user. It first failed on `/download`, which
answered a signed-out visitor 401 instead of sending them to sign in as the tunnel's download server did; the Worker
now redirects to `/signin` there. The drive matrix against the Worker is running.

## How it works

```
browser / app ──> Worker (worker/index.mjs)
                    ├─ Auth Durable Object "auth"          sign-in: lib/auth.mjs on its own SQLite
                    ├─ Library Durable Object per library  the library: lib/hosted.mjs on its own SQLite, /hub sockets
                    ├─ R2 bucket "mymind"                  website code, card media, each library's own files
                    └─ Media Durable Object + container    sharp, ffmpeg, poppler, Chromium
```

1. The Worker removes any `x-mymind-*` header a client sent, then asks the Auth object about the request.
   `lib/auth.mjs` answers exactly as `server.mjs` does with `AUTH=google`: a signed-out visitor gets the plain
   sign-in page (no mymind name or assets, `noindex`), an API call gets 401, and Google sign-in admits only
   `OWNER_EMAIL`. For a signed-in request it names the account's library.
2. The Worker sends the request to that library's Durable Object, which serves it with the same code as
   `HOSTED=1 node server.mjs`. An account without a library gets 403. A library reads only its own database, its own
   R2 folder `libraries/<name>/`, and `static/` (mymind's website code, the one shared folder).
3. `/download` (the desktop app) is served from R2 to any signed-in visitor.
4. `/_admin/` exists only while the `ADMIN_TOKEN` secret is set. It is how the import tool writes R2 objects,
   library rows and sign-in rows. Without the token those paths go through sign-in like any other.

| Piece | Where | Code |
|---|---|---|
| Sessions, app tokens, password users, which library each account reaches | Auth object's SQLite | `worker/auth-object.mjs`, `lib/auth.mjs`, `lib/auth-store.mjs` |
| A library: cards, tags, spaces, search index, embeddings | That library's object | `worker/library-object.mjs`, `lib/hosted.mjs`, `lib/store.mjs` |
| Recorded mymind answers the replica still serves (`recorded` table, 2,942 rows in the test library) | That library's object | `lib/recorded.mjs` |
| Live updates (`/hub`) | WebSockets held by that library's object, so events reach only that library's sessions; a socket is closed within 60 s of its session ending or its account moving to another library | `worker/library-object.mjs` |
| mymind's website code | R2 `static/`, shared | `lib/site.mjs` |
| Captured card media from the recordings of the owner's library | R2 `libraries/<name>/capture/`, only in the libraries imported with `--recordings` (the owner's and the test library) | `lib/site.mjs` |
| A library's own media and upload chunks | R2 `libraries/<name>/media/`, `renditions/`, `uploads/` | `lib/hosted-media.mjs` |
| The desktop app | R2 `downloads/` | `lib/download-page.mjs` |
| Image and video renditions, PDF previews, link screenshots, link fetches | One container, started on demand | `worker/media-object.mjs`, `worker/media-client.mjs`, `worker/container/` |

Response headers recorded with a file (content type, `x-raw`) travel with it: `.headers/<key>.json` beside the file
on disk, custom metadata on the R2 object.

## Code shared with the Node server

The Node server and the Worker run the same library code, so parity proven on one holds for the other:

- `lib/sql-node.mjs` (`node:sqlite`) and `worker/sql-do.mjs` (Durable Object SQLite) give the store one synchronous
  SQL interface. On a Durable Object, large integers are read back exactly with `CAST(... AS TEXT)`, and
  transactions use `transactionSync`.
- `lib/bucket-fs.mjs` (a folder) and `worker/bucket-r2.mjs` (R2) give one object store interface.
- `worker/http.mjs` presents a Workers request as the Node `req` and `res` the library code handles.
- `lib/media-processor.mjs` and `lib/capture-link.mjs` run in Node locally and in the container on Cloudflare. The
  Worker stubs `sharp` and `playwright-core` (`worker/stubs/`), so nothing in the Worker loads them.

## Decisions that changed from the first proposal

| Proposal | Built | Why |
|---|---|---|
| D1 for sign-in | One Auth Durable Object | `lib/auth.mjs` uses synchronous SQL; a Durable Object runs it unchanged with the same adapter as the libraries. D1's API is async |
| Cloudflare Images for renditions, Browser Rendering for screenshots | One container with sharp, ffmpeg, poppler and Chromium | The same code as the Node server makes the same bytes. Browser Rendering always identifies itself as a bot. Images can't cut video frames or PDF pages |
| Recordings removed from the hosted server | Kept in each library's `recorded` table | The recorded answers the replica still needs are rows of that library, not files in R2 |

## Import

1. `node tools/import-hosted.mjs bucket` copies mymind's website code into `data/hosted/bucket/static/` (368
   objects, 18 MB).
2. `node tools/import-hosted.mjs library <name> --db <file> [--recordings [--runs]] [--media <dir>]` builds
   `data/hosted/<name>.db` and puts the library's own media under `libraries/<name>/`. `--recordings` adds this
   repository's recordings of the owner's library: the recorded answers, the page shell, and the captured card media
   under `libraries/<name>/capture/` (6,522 objects, 2.7 GB). Only the owner's and the test library take them; a
   library for anyone else is imported without them. The test library is built from `data/verify.db` with
   `--recordings --runs`.
3. `node tools/upload-hosted.mjs <worker url> bucket|library <name>|user|account <account> <library>|summary`
   sends them through `/_admin/`. A library upload ends with a digest of every table on both sides (row count and a
   SHA-256 over every value and its storage class) and fails if any table differs. The test user's password is
   hashed on this computer; only the salt and hash are sent. Bucket objects already there with the same size are
   skipped, so an interrupted upload resumes.

## Tested under wrangler dev (wrangler 4.128.0 with workerd 2026-08-31, then 4.151.1 with workerd 2026-10-12)

1. The test library uploaded with every table matching (2,279 cards, 17,680 annotations, 2,165 embeddings, 2,942
   recorded answers), and all 6,890 bucket objects in 3 min 21 s.
2. `tools/compare-hosted.mjs` sent 17,616 requests (every recorded key on every device and run, every detail
   view, mirrored file and media file, ranged video reads, the fallbacks) to the replica and, through
   `tools/hosted-proxy.mjs`, to the Worker signed in as the test user. 44 differ: the 40 `x-raw` headers already
   accepted for `HOSTED=1`, and 4 recorded answers for `/signin`, which sends a signed-in visitor to `/everything`
   as `AUTH=google` does on mini-002. Cloudflare compresses text on the way out, so the tool now compares decoded
   bodies and ignores `content-encoding`. After the review fixes below, with each library's captured media in its
   own folder, two more runs under wrangler 4.151.1 gave the same 44.
3. The `smoke` drive suite (desktop, light) against live mymind and the Worker: 6 of 6 screenshots pixel-identical
   (REA, channel threshold 0), with the page's `/hub` socket connected through the Worker. Run again after the
   review fixes, the Worker's 6 screenshots are identical in every pixel to the replica's. Both now differ from live
   mymind by 23.5%, because the live library has a card saved after the recordings were made.
4. Sign-in: signed out, a page is sent to the plain sign-in page and the API gets 401; password sign-in,
   sign-out and `/app/v1/me` work.
5. `/hub`: the SignalR handshake, pings, an `object:created` event after a note was written, and close code 1008
   for a socket whose session signed out.
6. The media container image, run with Docker and called through the Worker's media client: PNG, MP4 and PDF
   uploads prepared with their covers, a WebP rendition, a page fetch and a 1400x940 screenshot of example.com.
   Invalid media gets 415 and a private address gets 400 `CAPTURE_BLOCKED`, as in Node. A failing screenshot or
   fetch of a URL with a token in it answers "Media operation failed", and the container log holds only the
   operation, status and error code.
7. On the wire: HEAD answers carry no body bytes, captured media keeps its `content-length` on HEAD, and a range
   past the end gets 416.
8. The Worker starting the container itself under `wrangler dev`, with the two fixes in
   [Running it locally](#running-it-locally): an image upload got its 640x400 WebP rendition in 1.3 s, and link
   saves of example.edu and iana.org got their page text and an 800x537 screenshot. With the firewall rule removed,
   the upload still worked and the link screenshot failed. With the local state restored after these writes, a
   comparison run gave the same 44 differences.

Found and fixed during these tests: `lib/auth-store.mjs` made random bytes at module load, which Workers refuse;
string bodies got a `text/plain` content type Node never sent; and the Worker followed sign-in redirects instead of
passing them to the browser.

Not tested yet:

- The drive matrix against the Worker (1.5 to 2.5 hours), planned before switching the hostname. The recordings
  were refreshed on 2026-10-16 (6 cards saved on live mymind since the last capture), and the matrix against live
  is being re-recorded from them; the Worker's run comes after it.
- Owner sign-in with Google, which uses the owner's account and stays with the owner.

Under wrangler 4.128.0, `wrangler dev` exited with an empty error in 4 of 6 full comparison runs, each time while
serving media from its local R2, and workerd left no core dump. Through those runs the Worker's own JavaScript heap
stayed between 20 and 43 MB in use (89 MB allocated; a Durable Object may use 128 MB), while the two workerd
processes grew to 500 to 900 MB. Under wrangler 4.151.1 all three runs finished, in about 2 minutes instead of 7 to
9, so `worker/package.json` pins 4.151.1. The same 17,616 requests against `HOSTED=1` in Node left the heap at
5.3 MB, the size it started at.

## Running it locally

```bash
npm ci --prefix worker               # wrangler 4.151.1, pinned; kept out of the root package and the container image
npm --prefix worker run dev          # http://127.0.0.1:8787, local state in data/hosted/wrangler-state
npm --prefix worker run sidecars     # beside dev, only when the media container has to start (below)
```

`worker/.dev.vars` sets `STORE_READONLY=1` and `OFFLINE=1`; `--var STORE_READONLY:0 --var OFFLINE:0` on the dev
command turns writes and online capture on for one run.

The container needs two things on this computer, both found while testing it:

1. A firewall rule. The container's outbound requests reach workerd on the Docker bridge address, and ufw denies
   incoming traffic, so this rule was added on 2026-10-16:
   `sudo ufw allow in on docker0 from 172.17.0.0/16 to 172.17.0.1 proto tcp comment wrangler-dev-containers`
   (`sudo ufw delete allow in on docker0 from 172.17.0.0/16 to 172.17.0.1 proto tcp` removes it).
2. `npm --prefix worker run sidecars` (`worker/dev-sidecars.sh`). wrangler's networking sidecar inherits the host's
   `net.ipv4.conf.all.src_valid_mark=1`, which makes the kernel drop the host's packets to the sidecar, so workerd
   never reaches it and the container never starts. The script turns that setting off inside each sidecar as it
   starts and leaves the host's own value alone, since something on the host set it at runtime and may rely on it.

## Security review (Codex `gpt-6.1-sol`, xhigh, read-only, 2026-10-16)

| Finding | What was done |
|---|---|
| Captured card media sat in a shared R2 folder that every library served | Moved to `libraries/<name>/capture/`; `createSite` refuses to start without that prefix. Tested: another library's file and a file in the old shared folder both get the placeholder |
| Importing any library copied the owner's recordings into it | Recordings and captured media are added only with `--recordings` |
| A `/hub` socket kept receiving a library's events after its account was moved to another library | The session check also asks whether the account still reaches the library. Tested: a socket stayed open through one check, then closed with 1008 at the first check after the move |
| The media container logged raw error messages, and Chromium's can quote the page URL | It logs the operation, status and error code; unexpected errors answer "Media operation failed" |
| An unsatisfiable `Range` got a 206 with an inverted span | 416 with `bytes */<size>`, the answer the library's own media already gave |
| HEAD responses were built with a body | Not a problem on the wire: workerd sent 0 body bytes for every HEAD tried. Captured media no longer reads R2 for a HEAD |
| The container can fetch other Workers' public URLs | Left as is: it fetches them as any browser would, with no cookie, token or binding, and private addresses stay blocked |

## Cost

Workers Paid, $5 a month. At today's size the rest fits in what Paid includes:

- Durable Object storage (5 GB included) and R2 (10 GB free). The owner's and the test library each hold their own
  2.7 GB copy of the captured media, so about 5.4 GB plus the owner's own uploads.
- Containers: Paid includes 25 GiB-hours of memory, 375 vCPU-minutes and 200 GB-hours of disk a month. The
  container is `standard-1` (1/2 vCPU, 4 GiB, 8 GB disk; the JS image is 1.94 GB, the Rust one 1.70 GB), so the
  included memory covers about 6 hours of running time a month. Beyond that it costs about 4 to 7 cents an hour. It
  stops 3 minutes after its last request.

## Steps left

Done on 2026-10-17: the R2 bucket, the secrets, the deploy, both uploads, the route, and removing the `ADMIN_TOKEN`
secret once the imports were done (`.env` still holds the old value, which nothing accepts now).

1. The drive matrix against the Worker finished on 2026-10-17 and is not yet at parity: iphone-ui 119 of 121,
   android-ui 119 of 121, desktop-dark-ui 258 of 284, desktop-light-ui 129 of 284, and the cards suites 0 of 18
   (desktop light), 16 of 18 (desktop dark), 18 of 36 (Android) and 25 of 36 (iPhone). The causes, most screenshots
   first: colour-scheme state kept only in the light runs, script errors on the hosted pages, media that loads
   slowly or not at all in the cards runs, spaces whose data drifted from the recordings, and `WebAssembly is not
   defined` with detail screenshots 64 to 70% different. The Rust Worker answers every request as the JS Worker
   does (the same 56 differences from the replica), so these causes stay until each is fixed; the matrix then runs
   again against the deployed Rust Worker.
2. The owner signs in with Google on the Worker once.
3. Keep mini-002 and the tunnel until 2026-10-24, then remove them.
4. Domain accounts: an empty library at the first sign-in of a verified `@octaviantocan.com` address, a page shell
   built from that person's own data, and the Google consent screen opened to them. Approved, not built; it will be
   built in Rust (`docs/plans/2026-10-17-0110-refactor-rust-backend-plan.md`).

The Android app and the desktop app keep the same URL and API, so neither changes.
