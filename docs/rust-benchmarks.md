# Rust backend benchmarks

The JS backend and its Rust rewrite (`rust/`), measured on the same data on 2026-10-17, for requirement R11 of
`docs/plans/2026-10-17-0110-refactor-rust-backend-plan.md`. Every Rust answer here is the JS answer byte for byte
(R4), so the numbers compare the same work.

Machine: AMD Ryzen 7 7735HS (8 cores, 16 threads), 30 GB, Linux 7.2. Node 24.12.0, rustc 1.99.0, Docker 29.9.1,
wrangler 4.151.1. Latencies are medians over loopback with the p95 where it matters; CPU time is the server
process's user plus system time per request.

## Summary

- **Native server:** starts 6 to 16 times faster, idles in a quarter to an eighth of the memory, peaks at a seventh
  to a tenth, and answers the heavy routes 1.3 to 6 times faster. A 61 MB media file costs a sixth of the CPU.
- **Media container:** the image is 238 MB smaller, it starts in about half the time, idles in 1.8 MiB instead of
  66 MiB, and preparing a 12-megapixel JPEG takes a third of the time. `image()` costs the same; every other
  operation is 3 to 20% faster.
- **Worker:** under `wrangler dev` the Rust Worker answers as fast as the JS one, within run-to-run noise, with
  `/changes` about 8% slower. In production the client waits 1 to 14 ms longer per route than with JS, but
  `/changes`, rewritten to batch its queries, now costs a fifth of the JS CPU and answers in a third of the time.
- **Where Rust is slower or bigger:** the Worker bundle is 16 times larger (1.43 MB against 90 KB compressed),
  and in production the Rust Worker answers the small routes 1 to 14 ms later than the JS one did. In the container, `image()` costs the same.

These are historical pre-switch comparisons. Future native measurements use `node tools/bench-backend.mjs`
with the Rust binary; container comparisons require explicitly supplied historical and Rust images.

## Native server

Historical pre-switch command: `node tools/bench-backend.mjs` (report in `evidence/bench-backend.json`): `node server.mjs` and `mymind serve` on
fresh copies of the same fixtures, 30 requests per route after a warm-up, in two modes: the replica's library mode
(`STORE_DB` on the owner library, read-only) and hosted mode (`HOSTED=1` on the test library and a copy of its
objects).

| Measure | Node | Rust |
|---|---|---|
| Library mode: start to listening | 393 ms | 60 ms |
| Library mode: memory idle / peak (RSS) | 164 / 496 MB | 45 / 71 MB |
| Hosted mode: start to listening | 347 ms | 22 ms |
| Hosted mode: memory idle / peak (RSS) | 128 / 470 MB | 17 / 45 MB |

Hosted mode, per route (library mode is within a few percent of these):

| Route | Size | Node median (CPU) | Rust median (CPU) |
|---|---|---|---|
| `GET /cards` (JSON) | 5.2 MB | 45.2 ms (58.1) | 23.2 ms (19.5) |
| `GET /cards` (MessagePack) | 5.2 MB | 45.8 ms (57.2) | 23.1 ms (19.9) |
| `GET /changes?since=0` | 3.6 MB | 323.0 ms (337.0) | 248.1 ms (246.7) |
| `GET /embeddings/related/:id` | 6 KB | 119.3 ms (154.3) | 19.1 ms (18.8) |
| `GET /search?q=design` | 43 KB | 9.9 ms (10.3) | 7.5 ms (7.2) |
| `GET /tags/manual` | 2 KB | 5.1 ms (5.1) | 4.8 ms (4.6) |
| `GET /spaces` | 57 KB | 1.2 ms (1.1) | 1.0 ms (0.9) |
| `GET /cards/:id/detail` | 7 KB | 0.2 ms (0.2) | 0.1 ms (0.1) |
| `POST /cards/grid` | 7 KB | 0.3 ms (0.4) | 0.2 ms (0.1) |
| `GET /everything` (page) | 57 KB | 0.4 ms (0.5) | 0.2 ms (0.1) |
| `GET /_static/…` (script) | 589 KB | 1.0 ms (2.7) | 0.6 ms (0.3) |
| `GET /media/…` (largest file) | 61 MB | 122.9 ms (165.3) | 95.6 ms (26.3) |

- `/embeddings/related` gains most. Both decode and compare all 2,165 stored embeddings of 768 floats per request;
  the JS reads each float with a `buffer.readFloatLE` call, Rust converts the bytes in one loop. The answer carries
  its own `elapsed` milliseconds, so it is the one body whose bytes differ between the two (121 against 24, say);
  with `elapsed` zeroed, the first 40 cards answer alike.
- `/changes` gains least (23%); it wasn't profiled.
- Both stream the 61 MB file from disk, so the wall time is mostly the loopback transfer; Rust's CPU time is a sixth
  of Node's.
- The release binary is 13.7 MB, plus the 16.6 MB libvips library next to it. The Node backend needs Node (about
  120 MB) and `node_modules`.

## Media container

`node tools/bench-container.mjs` (report in `evidence/bench-container.json`): the deployed JS image
(`cloudflare-dev/media:3ee08ddf`) and the Rust image (`rust/container/Dockerfile`), each started 5 times for the
cold start, then twice for the operations, 10 runs of each operation per round through the Worker's media client.
The medians below are the two rounds.

| Measure | JS | Rust |
|---|---|---|
| Image size | 1,940.6 MB | 1,702.3 MB |
| Cold start (`docker run` to `/health` answering) | 631 ms | 337 ms |
| Memory 2 s after start | 65.0 to 67.5 MiB | 1.8 MiB |

| Operation | JS | Rust |
|---|---|---|
| Prepare a 4000x3000 JPEG (3.3 MB): metadata, cover, colours | 360.5 / 368.1 ms | 111.5 / 113.0 ms |
| Its cover rendition | 95.4 / 94.6 ms | 88.8 / 89.5 ms |
| Its 400x400 WebP rendition | 57.2 / 57.8 ms | 53.8 / 54.5 ms |
| `image()` of it (decode and measure) | 134.1 / 138.0 ms | 134.6 / 129.8 ms |
| Prepare a 4.9 MB MP4 (ffprobe, poster frame) | 325.0 / 327.3 ms | 254.2 / 266.1 ms |
| Its cover rendition | 223.9 / 238.7 ms | 207.2 / 214.7 ms |
| Prepare a 406 KB PDF (pdftoppm, first page) | 371.8 / 370.4 ms | 318.2 / 320.5 ms |
| Its cover rendition | 375.6 / 372.2 ms | 360.9 / 362.1 ms |
| Screenshot of a page on the container itself (offline) | 348.2 / 348.1 ms | 321.9 / 328.2 ms |
| Screenshot of `https://example.com/` | 465.6 / 454.5 ms | 420.4 / 432.4 ms |
| Fetch of `https://example.com/` (capture transport) | 41.1 / 46.9 ms | 35.7 / 40.6 ms |

- Both images run the same ffmpeg, poppler and Chromium from Debian bookworm and the same libvips build (sharp's),
  so the operations that spend their time in those programs (the PDF cover, the screenshots) gain least. Why the
  JPEG prepare gains most wasn't profiled.
- The memory after the operations swings between rounds (JS 191 and 198 MiB, Rust 200 and 57 MiB), so only the
  idle figure is steady enough to compare.
- Containers are billed by running time and memory; both images run as `standard-1` and stop 3 minutes after the
  last request, so the faster operations and the smaller image change the bill little.

## Worker

Both Workers under `wrangler dev` (workerd on this machine), each on its own fresh local state filled by
`upload-hosted`, restarted read-only and reached through `tools/hosted-proxy.mjs` signed in as the test user, so
every request also passes the `Auth` object. 20 requests per route after 3 warm-ups, both Workers back to back on
the same state:

| Route | Size | JS | Rust |
|---|---|---|---|
| `GET /cards` (JSON) | 5.2 MB | 59.8 ms | 62.9 ms |
| `GET /changes?since=0` | 3.6 MB | 410.4 ms | 443.8 ms |
| `GET /search?q=design` | 43 KB | 24.0 ms | 24.3 ms |
| `GET /spaces` | 57 KB | 14.1 ms | 14.0 ms |
| `GET /everything` (page) | 57 KB | 11.9 ms | 11.0 ms |
| `GET /_static/…` (script from R2) | 589 KB | 22.0 ms | 21.9 ms |
| `GET /robots.txt` | | 6.0 ms | 5.8 ms |
| `compare-hosted`: 17,869 requests | | 138 s | 139 s |

- Under `wrangler dev` the numbers move between runs: two full runs (fill, compare, time) had the Rust Worker 4 to
  18 ms slower on the small routes, an earlier one had it 1 to 18 ms faster on all but `/search`. The table is the run that measured
  both on the same state minutes apart, so the honest result is no measurable difference on the Worker.
- The Worker's time goes to the runtime and the Durable Object's SQLite, which both Workers share; a profile of
  `/search` showed the remaining time stepping SQLite cursors in both. The native server, which runs SQLite in the
  same process, is where Rust's speed shows.
- From start: the read-only Worker answered after 1,557 ms (JS) and 1,636 ms (Rust); the first `/cards` took 114
  to 175 ms (JS) and 125 to 127 ms (Rust) across runs.
- Bundle, from `wrangler deploy --dry-run`: JS 322.77 KiB (89.68 KiB compressed), Rust 3,880.57 KiB (1,433.37 KiB
  compressed), of which the WebAssembly is 3,914,269 bytes.
- workerd's memory after the comparison run was 1,569 MB (JS) and 1,610 MB (Rust), for every object of the test
  library in one dev process; it isn't the 128 MB isolate limit production applies, which wasn't measured.

### Production

The deployed Worker on `mymind.octaviantocan.com`, tailed with `wrangler tail` while 20 requests per route went
through `tools/hosted-proxy.mjs` as the test user. CPU per request is the Worker invocation's CPU plus the CPU of
the Durable Object invocations it makes (`Auth` and `Library`), as Cloudflare reports them. JS was measured on
2026-10-17 before the switch, Rust twice the same day after it; client time is from this machine.

| Route | CPU, JS | CPU, Rust (two runs) | Client median, JS | Client median, Rust |
|---|---|---|---|---|
| `GET /cards` (JSON) | 281.8 ms | 234.8 / 170.4 ms | 280 ms | 291 / 286 ms |
| `GET /changes?since=0` | 505.6 ms | 565.9 / 580.0 ms | 690 ms | 782 / 825 ms |
| `GET /search?q=design` | 15.5 ms | 19.9 / 18.4 ms | 73 ms | 86 / 87 ms |
| `GET /spaces` | 7.7 ms | 9.3 / 7.5 ms | 52 ms | 59 / 60 ms |
| `GET /everything` (page) | 11.4 ms | 11.8 / 13.3 ms | 67 ms | 75 / 77 ms |
| `GET /_static/…` (script from R2) | 0.5 ms | 1.5 / 2.4 ms | 190 ms | 191 / 198 ms |
| `GET /robots.txt` | 0.0 ms | 0.2 / 0.9 ms | 22 ms | 31 / 32 ms |

- In production the Rust Worker isn't faster. `/cards` takes 17 to 40% less CPU, `/changes` and `/search` take 12 to
  28% more. The client waits 1 to 14 ms longer on every route but `/changes`, which takes 92 to 135 ms longer.
- Each Rust Worker invocation costs about 1 ms of CPU where the JS one costs 0.1 to 0.5 ms. Starting the WebAssembly
  and copying requests and answers between JavaScript and Rust would explain it; it wasn't profiled.
- At this traffic the cost difference is nothing: Workers Paid includes 30 million CPU milliseconds a month.

### `/changes` after batching

`/changes` asked SQLite for the newest change of each object with a correlated subquery that rescanned the user's
whole change log for every row, then read each card and its spaces with two queries per card (2,004 queries for
`since=0`). Since 2026-10-17 the Rust store finds the newest changes with one `GROUP BY` and reads cards and spaces 50
ids at a time (44 queries); the JS backend keeps the old queries. The answers are unchanged: JS and Rust matched on 824
requests per database (every 25th checkpoint on desktop and iPhone, and every card in a space) on `data/verify.db` and
`data/mymind.db`, and production's answers kept the same SHA-256 across the deploy.

| | Before | After |
|---|---|---|
| Native `mymind`, median of 20 | 344.4 ms | 59.3 ms |
| Production CPU per request | 566 to 580 ms (JS: 506) | 104.5 ms |
| Production, client median | 782 to 825 ms (JS: 690) | 247 ms |

## What the port had to match

Byte-identical answers and identical pixels needed these, each found by measuring:

- **Image bytes.** A system libvips (another version, libjpeg-turbo instead of mozjpeg) encodes other bytes, so the
  Rust server links the libvips build sharp ships, with every option sharp passes, and keeps libvips warnings out
  of the log. It runs libvips on one thread on glibc, as sharp does; with more threads the JPEG prepare took 80 ms
  but `image()` 160 ms.
- **Sorting.** A hand-written insertion sort in the search port made `/search` slower than JS; a stable `sort_by`
  (what `Array.prototype.sort` is) fixed it.
- **Worker bodies.** Copying R2 objects and container answers through wasm memory made the compare run 415 s
  against 154 s; bodies now stay JavaScript streams until something reads them.
- **Nagle's algorithm.** The capture transport's requests were 40 to 50 ms slower until `TCP_NODELAY` was set as
  Node sets it (the request after the TLS handshake waited for a delayed ACK). The native server sets it on every
  connection as Node's `http` server does.
- **Happy eyeballs.** An unreachable address took 15 s (the connect timeout) where Node gives up after 250 ms per
  address and tries the next; the Rust connect now does the same.
- **Link screenshots.** Chromium driven over the DevTools protocol drew pages differently until it got Playwright's
  launch switches, page setup (fonts, emulated media, window bounds) and `route.fulfill` headers (without the CORS
  headers Playwright adds, a page's cross-origin web fonts failed). On ten real sites the Rust and the JS
  screenshots now differ no more than two JS screenshots of the same page do:

| Page | JS against JS | Rust against Rust | JS against Rust |
|---|---|---|---|
| example.com | 0.00% | 0.00% | 0.00% |
| iana.org/help/example-domains | 0.00% | 0.00% | 0.00% |
| news.ycombinator.com | 0.00% | 0.00% | 0.00% |
| developer.mozilla.org (HTML docs) | 0.00% | 0.00% | 0.00% |
| rust-lang.org | 0.00% | 0.00% | 0.00% |
| docs.rs (tokio) | 0.00% | 0.00% | 0.00% |
| gov.uk | 0.00% | 0.00% | 0.00% |
| stripe.com | 0.28% | 0.35% | 0.30%, 0.36% |
| en.wikipedia.org (Rust) | 0.57% | 0.01% | 0.57%, 0.01% |
| github.com/rust-lang/rust | 0.91% | 0.82% | 1.07%, 0.85% |

Share of pixels that differ, two screenshots per backend taken alternately; the pages that change by themselves
(animations, rotating content) differ by the same amount whichever backend took them.
