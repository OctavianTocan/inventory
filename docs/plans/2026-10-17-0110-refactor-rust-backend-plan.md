---
title: Rust Backend - Plan
type: refactor
date: 2026-10-17
topic: rust-backend
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-brainstorm
execution: code
---

# Rust Backend - Plan

## Goal Capsule

- **Objective:** every place the library runs (this computer, mini-002, the desktop app, the desktop download server
  and Cloudflare) answers every request exactly as it does today, from typed code that needs no Node, so the Android
  parity work that follows builds on the backend that stays.
- **Means:** rewrite the backend in Rust: one core compiled natively and to WebAssembly for the Worker.
- **Product authority:** the user, 2026-10-17: "fully rewrite all the stuff to Rust for the backend now, and then we
  can move on to parity for Android". The goals are speed, cost, types, and learning what Rust buys.
- **Open blockers:** none.

## Product Contract

### Summary

The backend (`server.mjs`, `lib/`, `worker/`, `deploy/download-server.mjs`) is rewritten in Rust. One core crate runs
on two hosts: a native binary, and a WebAssembly Worker with the same three Durable Objects. The media container runs
the native binary. The Rust backend is built beside the JS one, proven with the parity checks the repository already
has, benchmarked, and then replaces the JS backend in one switch.

### Problem Frame

The backend is about 7,100 lines of untyped JS. It needs Node and npm wherever it runs: the desktop app runs
`npm ci` on its first start, and the media container image is 1.94 GB, built on Node. Nobody has measured where a
request spends its time. Parity with live mymind rests on byte-exact answers, so a rewrite is only worth doing if it
keeps every byte.

### Key Decisions

- **Build beside, switch once** (session-settled: user-approved — chosen over a route-by-route strangler and over Rust
  only where it pays: a library's SQLite database lives in one Durable Object, so the Library object can't be half
  ported, and a native-only port would keep the library logic in two languages). Governs R12, R13.
- **The Worker runs Rust too** (session-settled: user-directed — chosen over a benchmark gate that could have kept
  the Worker in JS). Governs R1, R11.
- **Same storage layout:** SQLite schema, R2 keys and Durable Object class names stay, so no data moves. Governs R6.
- **Media keeps its external programs:** ffmpeg, ffprobe, pdftoppm and Chromium stay; images go through libvips, the
  library sharp wraps. Governs R5.
- **JS is frozen as the reference** during the port: bug fixes only, no features. Governs R13.

### Requirements

**Coverage**

- R1. Rust replaces `server.mjs`, every module in `lib/`, `worker/` (the Worker, the Auth, Library and Media Durable
  Objects, and the container server) and `deploy/download-server.mjs`.
- R2. Every mode keeps working: the replay replica (`OFFLINE`, `STORE_DB`, `STORE_READONLY`, `PORT`), `HOSTED=1`,
  `AUTH=google` with `LOCAL_PORT`, the `/hub` WebSocket, the framed-media redirect, the app API under `/app/v1/`, and
  the download server.
- R3. The same environment variables, files and command-line arguments configure it.

**Behavior parity**

- R4. Answers are byte-identical: `tools/compare-hosted.mjs` reports no difference between the Rust and the JS replay
  replica, and between the replica and each Rust hosted server (native, and the Worker under `wrangler dev`) exactly
  the accepted differences the JS hosted server shows on the same data.
- R5. Writes and media behave the same: every behavior the `tools/check-*.mjs` suites assert holds for the Rust
  backend. Generated image renditions may differ in encoder bytes, never in size, format or what they show.
- R6. Existing data opens unchanged: `data/*.db`, mini-002's database, `data/hosted/`, the R2 bucket and the Durable
  Objects of the deployed JS Worker.
- R7. The 8-run REA matrix passes against the Rust replica, and against the Rust Worker once it is deployed.

**Tooling**

- R8. The operational tools that import `lib/` (`seed-db`, `auth-user`, `import-hosted`, `upload-hosted`,
  `embed-library`, `search-recordings`, `try-ai`, `capture-fixtures`) become subcommands of the Rust binary.
- R9. The browser and HTTP tooling (`drive`, `verify-all`, `coverage`, `compare-hosted`, `capture`, `http-check`)
  stays in Node and starts the Rust server.
- R10. Coverage freshness (`tools/digests.mjs`) hashes the Rust sources that answer requests.

**Speed and cost**

- R11. A benchmark report compares JS and Rust per heavy route on both hosts: latency, CPU time, memory, cold start,
  container image size and Worker CPU milliseconds, including where Rust is slower.

**Switch**

- R12. One switch moves mini-002, the desktop app, the container and the Worker to Rust, after R4, R5 and R7 hold.
- R13. The JS backend is deleted at the switch; nothing is added to it before then.

### Success Criteria

- R4, R5 and R7 green, and `docs/rust-benchmarks.md` written from measured numbers.
- The desktop app and the media container start with no Node installed.

### Scope Boundaries

- The mirrored front-end (`mirror/`) and the recordings are untouched.
- Per-person libraries (`docs/accounts-plan.md`, step 8) are built once, in Rust, after the switch.
- Android parity is the next phase, after the switch.
- The Kotlin app and the Tauri shell change only where they start the server.

### Dependencies / Assumptions

- workers-rs (`worker` 0.8.7) covers Durable Object SQL, alarms, hibernating WebSockets, R2 and containers; it has no
  `transactionSync` wrapper, so the Rust Worker calls it through JS interop.
- Durable Objects called each other with JS RPC; workers-rs objects answer only `fetch`, so those calls become internal
  requests between objects of the same Worker.
- No Rust SignalR server exists; the hub speaks only the JSON protocol, which is small enough to write by hand.
- Workers limits: 128 MB of memory, a 64 MiB script, 1 second of startup.

### Outstanding Questions

- Answered on 2026-10-17: link screenshots over the raw Chrome DevTools Protocol match Playwright's pixels once the
  Rust host sends what Playwright sends (its launch switches, its page setup, its `route.fulfill` headers). On ten
  real sites the Rust and the JS screenshots differ no more than two JS screenshots of the same page do, so the
  container needs no Node.

### Sources

- `docs/cloudflare-plan.md`, `docs/roadmap.md`, `docs/accounts-plan.md`, `docs/app-api.md`, `docs/write-api.md`
- `tools/compare-hosted.mjs`, `tools/digests.mjs`, `tools/verify-all.mjs`, `tools/http-check.mjs`
- workers-rs: https://github.com/cloudflare/workers-rs

## Implementation Units

Execution started with the plan, so the units are recorded here.

- U0. `rust/` workspace and `mm-js`: the JS value model (objects in JS key order, numbers as JS prints them,
  `undefined`), `JSON.parse`/`JSON.stringify`, UTF-16 string helpers, `decodeURIComponent`, base64, ISO dates.
- U1. `mm-msgpack`: the codec mymind's front-end ships (`mirror/.../vendor/msgpack.min.js`), byte-tested against it on
  every recorded MessagePack body.
- U2. Host interfaces in `mm-core`: synchronous SQL, an async object store, outbound fetch, the media processor,
  screenshots, hub pushes, clock and randomness; `Request`/`Response` in place of Node's `req`/`res`.
- U3. Store: `store-schema`, `store`, `store-query`, `store-enrich`, `search-index`, `search-document`, `recorded`,
  `library-rows`.
- U4. Render: `render-card`, `render/*`.
- U5. Writes and media: `store-write`, `read-metadata`, `uploads`, `media-writes`, `media-files`, `local-media`,
  `hosted-media`, `media-processor`, `bucket-fs`.
- U6. Search: `search`, `search-query`, `search-semantic`.
- U7. AI and capture: `ai`, `ai-tags`, `enrichment`, `capture-html`, `capture-entities`, `capture-http`,
  `capture-link`.
- U8. Serving: `api`, `app-api`, `site`, `hosted`, `device`, `download-page`.
- U9. Sign-in and hub: `auth`, `auth-store`, `hub`.
- U10. Native host `mymind`: `serve` (all of `server.mjs`), `download-server`, `container`, and the R8 subcommands.
- U11. Cloudflare host `mm-worker`: the Worker and the three Durable Objects, R2 and Durable Object SQL adapters, built
  with `worker-build`; the container image builds the native binary without Node.
- U12. Tooling: `MYMIND_SERVER` selects the server the Node tools start; `tools/digests.mjs` hashes `rust/`.
- U13. Parity: R4 (`compare-hosted` three ways), R5 (each `check-*` suite against the Rust server, or ported to Rust
  tests), R7 (the matrix).
- U14. Benchmarks and `docs/rust-benchmarks.md` (R11).
- U15. The switch (R12, R13).

Order: U0 to U2 first, since every module depends on them; then U3, U4, U6, U7 and U9 in parallel; then U5 and U8;
then U10 and U11; then U12 to U15.

## Progress

- U0 to U12 are built. The Rust workspace tests pass, and `cargo clippy --all-targets` is clean for the workspace
  and for the Worker on `wasm32-unknown-unknown`.
- R4 holds. Native: 17,869 replayed, recorded and edge-case requests, no difference between `node server.mjs` and
  `mymind`; the replica against each hosted server shows the same 52 differences (the accepted `x-raw` headers, six
  of them also on a PDF the bucket holds in another version) for JS and Rust. Worker under `wrangler dev` on fresh
  state filled by `upload-hosted`: JS and Rust each differ from the replica in the same 56 requests (those 52 and the
  4 accepted `/signin` redirects), request by request, and a direct comparison of the two Workers on 168 requests
  found no difference.
- R5 holds for what the Rust tests cover: writes, uploads and media preparation through `tests/library.rs` and the
  core tests, every image rendition byte for byte with sharp's, and link screenshots and resources through the
  capture transport.
- R6 holds. Durable Objects, under `wrangler dev` on one state directory: the JS Worker filled it with
  `upload-hosted`; the Rust Worker opened it read-only, accepted the JS session cookie and app token, and differed from
  the replica in the same 56 requests as JS; it then ran writable and signed in, and the JS Worker, started again on
  that state, accepted the Rust cookie and token and still showed the same 56 differences. mini-002: copies of its
  `data/mymind.db` and `data/auth.db` opened by `mymind serve` and by the JS server answered 17,872 requests with no
  difference, and the Rust server signed the test user in from the `auth.db` the JS server wrote.
- R7 holds against the native server: the 8-run matrix against `mymind serve` (read-only, offline, `data/verify.db`)
  matched 918 of 918 screenshots on 2026-10-17, and `npm run coverage` with `MYMIND_SERVER` answers `ready`
  (`evidence/drive-rust-native-20261017/`). Coverage now lets the newer of a run and its patch runs win, since a
  patch run made against the JS server was replacing screenshots of the newer Rust run.
- Switched: mini-002 runs `mymind serve` and `mymind download-server` since 2026-10-17 11:02 (its checkout on branch
  `cloudflare`, the binary and `libvips-cpp.so.8.17.3` in `rust/target/release/`), checked as the test user on the
  real databases: sign-in, the page, cards, search, the app API, sign-out and the download server. The desktop app's
  local mode starts `mymind serve` (`MYMIND_SERVER` or the checkout's release build); the packaged app opens the
  hosted library, so it doesn't change.
- R7 holds for the Worker: the 8-run matrix against the Rust Worker under `wrangler dev`, on state loaded from the
  recordings, matched 916 of 918 screenshots (`evidence/drive-rust-worker-20261017/`); the 2 left are the PDF the
  bucket holds in another version, which the JS Worker shows the same way.
- Deployed: production runs the Rust Worker (version `dd42f3fa`) and the Rust media container image since
  2026-10-17 11:44 CEST. The first deploy, at 11:38, uploaded the Worker but timed out pushing the image, so for six
  minutes the Rust Worker ran with the JS container; the second deploy pushed the image. Checked as the test user:
  `tools/check-hosted.mjs --browser` passes, and a saved link card got its screenshot from the container, served as
  WebP. Rollback while the JS is still in the tree: `npm --prefix worker run deploy` (the JS Worker and its
  container); `wrangler rollback cfede62c-28a5-4198-b40e-1ff4edae2ea7` reverts only the Worker.
- `/changes` batches its queries since 2026-10-17 (Worker version `8126d81e`): 2,004 queries become 44 for the whole
  library, natively 344 ms become 59 ms and in production 566 ms of CPU become 105 ms, with the same answers.
- Browser extension support and a `/hub` origin check since 2026-10-17 (Worker version `e83ecfcf`, media image
  `sha256:c53b41c2`): a valid session works as a bearer, a bearer alone skips the cross-origin checks, and a cookie
  session's `/hub` upgrade needs the server's own origin (`docs/extension.md`). Reviewed twice by Codex (read-only).
  Checked on production as the test user: `tools/check-hosted.mjs --browser` passes; `/hub` with the cookie answers
  101 from the own origin and 403 from a sibling subdomain or another site (101 for all three before); the session as
  a bearer from an extension origin reads (200) and pings (200), the cookie from that origin is refused (403).
  `wrangler rollback 8126d81e-d811-4e40-ab0d-41f3fa1c8e4a` reverts the Worker.
- App API additions since 2026-10-17 (Worker version `f3c367b5`, media image `sha256:b410dd23`): `GET
  /app/v1/top-of-mind` and the `x-upload-token` header for uploads (`docs/app-api.md`). Checked on production as the
  test user: `tools/check-hosted.mjs --browser` passes; `/app/v1/me` answers `{email, kind}`; `/top-of-mind` lists 4
  pinned cards; an authorized upload refuses a chunk with the app token alone and with a wrong `x-upload-token` (401
  both), and nothing was uploaded. `wrangler rollback e83ecfcf-669d-42fd-9099-9289755f6107` reverts the Worker.
- Backend gaps and Android downloads since 2026-10-17 (Worker version `ca97f888`, media image `sha256:736b7065`):
  `GET /cards/<id>/markdown`, screenshot attachments as covers, `Content` cards keeping their source,
  `/download/android.json` and Android packages for app tokens. Checked on production as the test user:
  `tools/check-hosted.mjs --browser` and `evidence/app-api/probe-prod.mjs` pass; a card's Markdown answers 200
  `text/markdown`; `/download/android.json` answers 404 until a package is published.
  `wrangler rollback f3c367b5-4829-4089-b79e-22fea29ae52c` reverts the Worker.
- The Chrome extension on `/download` since 2026-10-17 (Worker version `70a7222d`): its own section and
  `application/zip` for `/download/<name>.zip`. `deploy/publish-extension.sh` published `mymind-extension-1.0.0.zip`;
  `evidence/extension/check-download.mjs` downloaded it as the test user, matched its checksum, and loaded the
  unzipped folder in a headless Chromium. `wrangler rollback 9d442edf-6b22-4bee-95a2-58a625c65cab` reverts the Worker.
- R13, the JS backend deleted, since 2026-10-18 (Worker version `2f9f43c8`, media image `sha256:a0923485`): `server.mjs`,
  `lib/`, `worker/` and the oracle drivers are gone (merge `a850d9c`); the Worker deploys with
  `npm --prefix rust/crates/worker run deploy` after `npm ci --prefix rust/crates/worker`. Only comments and help
  texts of the deployed sources changed. Checked on production as the test user: `tools/check-hosted.mjs --browser`,
  `evidence/app-api/probe-prod.mjs`, `probe-gaps.mjs` and `evidence/extension/check-download.mjs` pass.
  `wrangler rollback 702332fd-e03f-4c12-b0f4-fa89d85c064c` reverts the Worker.
- R7 after R13: the 8-run matrix against `mymind serve` built from `a850d9c` (read-only, offline, `data/verify.db`,
  replica passes against the recorded live passes) matched 918 of 918 screenshots on 2026-10-18 with no step errors,
  and `npm run coverage` with `MYMIND_SERVER` answers `ready` (918 surfaces, 0 open reasons).
- R11: `docs/rust-benchmarks.md`, production numbers included.

Decisions made while building:

- Images go through the libvips build sharp ships, linked from `rust/libvips/` (`rust/fetch-libvips.sh`), with every
  option sharp passes; a system libvips encodes other bytes.
- Link screenshots drive Chromium over the DevTools protocol, mirroring Playwright step by step (Outstanding
  Questions above); the outbound connections mirror Node's (`TCP_NODELAY`, happy eyeballs). `rust/PORTING.md`
  lists each step and the differences that remain.
