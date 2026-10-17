# Porting the backend to Rust

How a JS module in `lib/` becomes a Rust module in `mm-core`. The plan is
`docs/plans/2026-10-17-0110-refactor-rust-backend-plan.md`; the JS stays frozen as the reference until the switch.

## Where things go

- `lib/<name>.mjs` becomes `crates/core/src/<name>.rs` with `-` replaced by `_` (`store-write` is
  `store_write.rs`); `lib/render/<name>.mjs` becomes `crates/core/src/render/<name>.rs`.
- Code that needs the file system, child processes, sockets or Chromium (`bucket-fs`, `local-media`,
  `media-processor`, `capture-http`, `capture-link`, `read-metadata`, the directory half of `uploads`) goes in the
  native host crate, behind a trait in `mm-core` that the Worker implements too.
- `mm-core` must build for `wasm32-unknown-unknown`: no `std::fs`, `std::process`, `std::net` or threads in it.

## What `mm-core` gives every module

- `mm_js`: `Value` and `Object` (JS key order), `json::parse`/`stringify`, `number_to_string`, `text::*` (UTF-16
  lengths, `slice`, `padStart`), `uri::*`, `base64::*`, `date::*` (`Date.parse`, ISO text), `locale_compare`, and
  `inspect::{inspect, console_line}` for what `console.log(object)` prints (`util.inspect` with Node's defaults).
- `mm_msgpack`: the front-end's MessagePack codec, byte for byte.
- `mm_core::sql`: the `Sql` trait (`query`, `query_each`, `execute`, `exec_batch`, nested `transaction` through
  `SqlExt`), `SqlValue`, `Row`, and `params![]`. Bind exactly what the JS bound: a JS number binds REAL (so pass
  `f64` or an integer type, both bind REAL), a `BigInt` binds as decimal text (`SqlValue::big`), a string as TEXT,
  a `Uint8Array` as BLOB, `null` as NULL. Read rows through `row.text()`, `row.number()`, `row.int()`,
  `row.blob()`, or `row.get(...).to_value()` for the value JS saw. Never rely on INTEGER versus REAL when reading:
  Durable Objects read both as numbers.
- `mm_core::http`: `Request` (lower-case `Headers`, parsed `url`, `Body`), `Response`, `Response::send` (the
  `send()` of `lib/site.mjs`), `search`, `search_param`. A JS handler that returned `false` for "not mine" returns
  `Ok(None)`.
- `mm_core::host`: the async host traits (`#[async_trait(?Send)]`): `Bucket`, `Fetch` (`fetch` follows redirects
  as Node's `fetch` does; `fetch_unfollowed` is `redirect: 'manual'`), `Media`, `UploadStorage`, `Resource` (the
  capture transport) and `Screenshot`. `mm_core::auth::SigninPages` reads the recorded sign-in pages.
- `mm_core::runtime`: `spawn`, `sleep`, `timeout` (`AbortSignal.timeout`), `log` (`console.log`), `env`
  (`process.env`), `random_bytes`, `random_uuid`, `now_ms` (`Date.now()`).
- `mm_core::{Error, Result}`: `Error::status(400, "message")` where JS threw an error with a `status`;
  `Error::invalid(..)` where JS threw (or would throw at run time) a `TypeError` or `RangeError` on bad input,
  which the API answers as 400 `Invalid request data`; `Error::internal(..)` for everything else (a 500).
  A failed `json::parse` converts with `Error::from` (or `?`) into `SyntaxError: <message>`, V8's message for the
  same input, which is what `String(error)` shows; `ParseError::message()` is `error.message`.

Add a trait to `mm-core` only for something a host provides (the media processor, screenshots, the capture
transport, hub pushes). Shapes the JS passes around as plain objects become structs and enums where the shape is
fixed; JSON that is stored or sent as it is (card `data`, `extras`, recorded bodies) stays `Value`.

## Parity

Answers must be byte-identical to the JS backend. What that takes:

- Build JSON with `Value` and `obj!` in the JS property order, and serialize with `mm_js::json`; never
  `serde_json`, whose key order and number formatting differ.
- Numbers print as JS prints them (`number_to_string`, `number::to_fixed`); integer arithmetic that JS did in
  doubles stays in `f64` unless the values are known to be small integers.
- String lengths, slices and indexes that JS measured in UTF-16 units use `mm_js::text`.
- `new URL(input, base)` is `mm_js::url::parse(input, base)`, never `Url::parse` or `Url::join`: it fixes the places
  where the `url` crate lags the URL Standard Node follows (`^` in paths, extra slashes before a host). Read
  `url.search`, `url.hash`, `url.host` and `url.origin` through `mm_js::url` too.
- Sorting with `localeCompare` uses `mm_js::locale_compare`; JS `Array.prototype.sort` is stable, so use
  `sort_by` (stable), not `sort_unstable_by`.
- Regexes: the `regex` crate where the pattern has no lookaround or backreference, else hand-written matching
  (or `fancy-regex`). JS `.` does not match `\n`, `\r`, U+2028 or U+2029; `\s` and `\d`, `\w` are ASCII-only in
  `\d`/`\w` and Unicode in `\s`; `i` without `u` folds ASCII only. Check each pattern you translate.
- Where JS would throw at run time on malformed input (reading a property of `undefined`, `String.fromCodePoint`
  out of range, `BigInt("x")`), the Rust port returns the matching `Error`, so the API answers the same status.
- A 500 body is `String(error)`: `Error::to_js_string` gives `Error: …`, `TypeError: …` or `URIError: …` as the
  JS error would print.
- `node:sqlite` binding decides several answers. A BigInt binds as decimal text, so a nanosecond checkpoint compared
  with a stored integer never matches a huge `since` (it does not fail). An object or array in the first parameter
  position is read as named parameters: a non-empty one throws `Unknown named parameter` (a 500), an empty one
  leaves the parameter unbound. Booleans and `undefined` throw a TypeError (a 400). `media_writes::bind_id` holds
  these rules.
- Every media store keeps `String(name ?? '')` as the blob name, so a missing title is `""`, never `"undefined"`.

## Testing

Every ported module gets differential tests in `crates/core/tests/<area>.rs` that run the same inputs through Rust
and through the JS module and compare the answers as JSON text (`tests/common/mod.rs`):

- `common::calls(module, &[(export, args)])` calls exported functions of any JS file below the repository root in
  one Node process and returns `Ok(value)` or the error it threw. `common::assert_same(rust_result, js_answer,
  context)` compares.
- For stateful JS (a store over a database, a handler), write a small driver module in
  `crates/core/tests/drivers/<name>.mjs` that imports `lib/` and exports plain functions taking JSON arguments,
  and call it with `common::calls`.
- Test on the real library: the fixtures are copies, never the files in `data/`:
  `MYMIND_TEST_DB` (the owner library, about 1,400 cards) and `MYMIND_TEST_HOSTED_DB` (the hosted test library
  with its recordings). Copy a fixture to a temporary directory before any test that writes.
- Cover every exported function on recorded data (every card, every recorded body) plus generated edge cases.
- Temporary files go under `env!("CARGO_TARGET_TMPDIR")`, never `/tmp` (a RAM-backed tmpfs here), and a test
  never writes into the repository.
- `runtime::set_fixed_now` is global to the process, so tests that fix the time run serialized; hub tests need an
  idle runtime (no other task holding the `LocalSet`).
- The workspace lists its members in `Cargo.toml`; a new crate is added there.
- Agents porting in parallel work in copies under `rust/target/work/<name>/rust` (on disk, not tmpfs), with their
  own target directories, and the main session merges them by hand.
- `cargo clippy --all-targets` stays clean; run cargo with `nice -n 19` and `-j 3` (a long verification runs on
  this machine).

## The native server

`crates/server` is the host for `server.mjs`: `http.rs` puts Node's `http` semantics on hyper (header merging,
`req.headers`, `half_close` as Node's `allowHalfOpen`, no `content-length` on HEAD answers), `serve.rs` is
`handle()`, `socket.rs` carries `/hub` upgrades, `fetch.rs` is Node's `fetch`. `tests/replica.rs` sends every
replayed and recorded request plus the edge cases of each route to `node server.mjs` and to `mymind`, and compares
status, headers (case-insensitive, without `date` and framing headers), bodies and request logs.

`tests/library.rs` runs both servers on fixture copies with a library: the API over `STORE_DB` (writable, with
reads only, and read-only with every request) and `HOSTED=1` over the hosted test library and a copy of its
objects (`MYMIND_TEST_HOSTED_BUCKET`, which `fixtures.py` fills), and checks that a write reaches a `/hub` client
alike. The two servers' clocks write into the answers whenever a request writes (a card's `modified`, a smart
space's `checked`, the `x-checkpoint` header), so the comparison zeroes the digits of ISO times and of 13-digit or
longer epoch numbers dated after the test started, keeping lengths and MessagePack prefixes, and compares every
older time exactly. `tests/read_metadata.rs` covers the seed import, `tests/screenshot.rs` the Chromium screenshots
(CDP over `--remote-debugging-pipe`, every request answered by the capture transport), `tests/media.rs` the
renditions and media preparation against sharp's and the JS processor's bytes.

The operational tools of `tools/` that imported `lib/` are subcommands of `mymind` (`crates/server/src/tool_*.rs`,
`mymind seed-db`, `mymind import-hosted`, …), with the JS tool's arguments, output and exit statuses. Their tests run
the JS tool from a temporary root (a copy of `tools/` with `lib/`, `mirror/`, `capture/`, `replay/` linked in and no
`.env`), because the tools find the repository from their own path and some read or write its `.env`. Where a JS
tool dies on an uncaught error, Node prints the source line and stack before `Error: …`; the subcommand prints only
the `Error: …` line, with the same exit status 1.

The Node tooling that stays (`drive`, `verify-all`, `coverage`, the HTTP checks, `check-download`) starts the backend
through `tools/server-command.mjs`: `MYMIND_SERVER=rust/target/release/mymind` swaps `node server.mjs` for the
binary, and `tools/digests.mjs` then hashes the Rust crates instead of `server.mjs` and `lib/`, so coverage judges
each backend by its own runs.

`rust/container/Dockerfile` builds the media container from the Rust binary (`mymind container`), with sharp's
libvips, ffmpeg, poppler and Chromium and no Node; `crates/worker/wrangler.jsonc` builds its image from it.

## Images

`vips.rs` calls libvips through its C API. The crate links the build sharp 0.34.5 ships (sharp-libvips 1.2.4,
libvips 8.17.3 with mozjpeg, libspng, libwebp, libheif and cgif), which `rust/fetch-libvips.sh` downloads into
`rust/libvips/<platform>/` and checks against the sha512 in `package-lock.json`; run it once before the first
`cargo build`. `MYMIND_VIPS_DIR` names another directory with that library, or `system` for the system libvips. A
system libvips (another version, libjpeg-turbo instead of mozjpeg) encodes other bytes, so only sharp's build gives
the JS renditions byte for byte. Each sharp call is ported with every option sharp passes, not libvips' defaults:
`fail_on` at the warning level and sequential access when loading, the full option list of each encoder (for JPEG,
quality 80, optimised Huffman coding, trellis quantisation off; for WebP, quality 80 and effort 4), one libvips
thread on glibc Linux as sharp sets it, and libvips warnings kept out of the log (`g_log_set_handler`). Unlike
sharp, the libvips operation cache is off; that changes memory use, not bytes. `tests/media.rs` compares every
rendition with sharp's bytes.

## Link screenshots

`screenshot.rs` drives Chromium over the DevTools protocol on a pipe, with no Playwright. It sends what Playwright
sends, because each step changes pixels on real pages:

- Playwright's launch switches in Playwright's order, plus `--disable-features=IsolateOrigins,site-per-process` so
  one renderer serves every frame and the page's `Fetch` interception sees cross-origin iframe requests (Playwright
  attaches to each iframe target instead).
- Playwright's setup of a new page, sent together as Playwright sends it: lifecycle events, the network cache off,
  service workers bypassed and their registration blocked, focus emulation, the window bounds and device metrics,
  the default font families, the emulated media features, then the init script.
- `route.fulfill`: header names in lower case, `content-length` only for a non-empty body, the status phrase
  Playwright sends, and the CORS headers Playwright adds when the page asked with an `Origin` for another origin
  (without them a page's cross-origin web fonts fail). `route.abort()` is `errorReason: "Failed"`.
- `page.screenshot`: the caret hidden in every frame from an isolated world, fonts awaited, then the visual
  viewport captured.

`capture_http.rs` connects as Node does: Nagle's algorithm off (`TCP_NODELAY`; with it on, the request after the TLS
handshake waited about 40 ms for an ACK), and `autoSelectFamily`'s happy eyeballs (families interleaved, 250 ms per
attempt but the last), so an unreachable address costs a quarter second, not the 15 s connect timeout. The native
server sets `TCP_NODELAY` on every accepted connection, as Node's `http` server does.

On ten real sites the Rust screenshots differ from the JS ones no more than two JS screenshots of the same page
differ from each other (`docs/rust-benchmarks.md`).

## The Worker

`crates/worker` is `worker/` on workers-rs, built to WebAssembly with `worker-build --release`. It keeps the class
names, bindings, Durable Object SQL schema and R2 keys of the JS Worker, so it opens the deployed data unchanged.

- Durable Objects called each other with JS RPC; workers-rs objects answer only `fetch`, so a call is an internal
  request marked with the `x-mymind-call` header (`auth_object::CALL`), which outside requests can't carry.
- `sql_do.rs` runs `transactionSync` through JS interop, since workers-rs has no wrapper for it.
- Request and response bodies stay JavaScript streams until something reads them (`http.rs`), so an R2 object or a
  container answer passes through without being copied into wasm memory.
- `runtime.rs` gives mm-core the isolate's scheduler, `console.log` and the Worker's variables and secrets.

## Known differences

- The hyper front: a raw `<`, `>`, `` ` `` or byte above 0x7F in the path, or `"`, `<`, `>` or a byte above 0x7F in
  the query, is answered 400 by hyper's URI parser where Node answers the request. Browsers and OkHttp
  percent-encode all of them, and Cloudflare normalizes URLs before a Worker sees them.
- Link screenshots: Playwright's `routeWebSocket` mock answers a page's WebSockets in the page and closes them; in
  Rust they fail at the denied proxy. Neither delivers a message, so the page sees a closed socket either way.

## Comments

Rustdoc on public items states the caller contract: a summary, `# Arguments` for every parameter, `# Returns` for
a non-obvious result, `# Errors` for every `Result`. Private helpers get one line. Inline comments only for a
non-obvious why. Name the JS function a port replaces in its summary (`(`escapeHtml`)`). No section dividers.
