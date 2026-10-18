# Rust backend maintenance

Rust is the backend. The pre-switch implementation and its executable oracle adapters were removed after
Stages A and B froze expectations and ported the JS checks. The historical plan is
`docs/plans/2026-10-17-0110-refactor-rust-backend-plan.md`.

## Where things go

Shared behavior belongs in `crates/core/src/`. File-system, process, socket and Chromium work belongs in the native
host `crates/server/src/`, behind core host traits also implemented by the Worker. `mm-core` must build for
`wasm32-unknown-unknown`: no `std::fs`, `std::process`, `std::net` or threads in it.

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
- `mm_core::http`: `Request` (lower-case `Headers`, parsed `url`, `Body`), `Response`, `Response::send`, `search`, `search_param`. A JS handler that returned `false` for "not mine" returns
  `Ok(None)`.
- `mm_core::host`: the async host traits (`#[async_trait(?Send)]`): `Bucket`, `Fetch` (`fetch` follows redirects
  as Node's `fetch` does; `fetch_unfollowed` is `redirect: 'manual'`), `Media`, `UploadStorage`, `Resource` (the
  capture transport) and `Screenshot`.
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

Tests compare Rust with frozen pre-switch golden expectations, without a Node oracle. They rebuild immutable
temporary fixtures from versioned manifests and logical rows; never replace them from mutable `data/`.
`MYMIND_RECORD_GOLDEN=1` fails explicitly: legacy recording requires the recorded pre-switch revision.
The test-support guard verifies source fingerprints, fixture hashes, ordered requests, case IDs, complete batch
inventories and exact response length/SHA-256. Small synthetic scrypt expectations also contain inline values.
No golden is re-recorded for help-prefix normalization or backend removal. An exact reviewed source transition
maps only the post-removal source fingerprints to the frozen identities; later edits still fail closed.

Core tests use `tests/common/mod.rs`; logical identifiers translate to historical golden lookup keys without
filesystem operations or executable driver paths. Native HTTP tests use immutable request inventories, typed state and normalized HTTP,
hub and application log expectations. Operational tests exercise `mymind` subcommands with scratch roots and
loopback fake providers. CLI comparisons normalize only the legacy usage prefix to preserve existing goldens.
Media tests compare frozen byte expectations and deterministic metadata, with ffprobe checks for generated video.

All scratch belongs under `env!("CARGO_TARGET_TMPDIR")`; screenshot uses the short `TMPDIR=/tmp/mmt` when needed.
Tests never write into the library. `runtime::set_fixed_now` is global, so fixed-clock tests run serialized;
hub tests need an idle runtime. Run Cargo with `nice -n 19`, `--offline`, `-j 2` and at most `--test-threads=2`.
`cargo clippy --all-targets -- -D warnings` stays clean.

The native `mymind serve`, `download-server` and `container` subcommands share the same backend.
Operational commands include `seed-db`, `import-hosted`, `upload-hosted`, `auth-user`, `embed-library`, `try-ai`,
`capture-fixtures` and `search-recordings`. Developer tools resolve only a Rust binary through
`tools/server-command.mjs`; `MYMIND_SERVER` overrides its default `rust/target/release/mymind` path.
`tools/digests.mjs` hashes request-serving Rust sources, manifests, build/config inputs and replay helpers for
coverage. Backend or normalization edits make earlier drive evidence `verifier-result-incompatible`.

`rust/container/Dockerfile` builds the media container from `mymind container`, pinned libvips, ffmpeg, Poppler and
Chromium. It needs no Node runtime; the Worker's `wrangler.jsonc` selects this image and its repository build context.

## Images

`vips.rs` calls libvips through its C API. The crate links the build sharp 0.34.5 ships (sharp-libvips 1.2.4,
libvips 8.17.3 with mozjpeg, libspng, libwebp, libheif and cgif), which `rust/fetch-libvips.sh` downloads into
`rust/libvips/<platform>/` and checks against the pinned sha512 values in that script; run it once before the first
`cargo build`. `MYMIND_VIPS_DIR` names another directory with that library, or `system` for the system libvips. A
system libvips (another version, libjpeg-turbo instead of mozjpeg) encodes other bytes, so only sharp's build gives
the JS renditions byte for byte. Each sharp call is ported with every option sharp passes, not libvips' defaults:
`fail_on` at the warning level and sequential access when loading, the full option list of each encoder (for JPEG,
quality 80, optimised Huffman coding, trellis quantisation off; for WebP, quality 80 and effort 4), one libvips
thread on glibc Linux as sharp sets it, and libvips warnings kept out of the log (`g_log_set_handler`). Unlike
sharp, the libvips operation cache is off; that changes memory use, not bytes. `tests/media.rs` compares every
rendition with frozen pre-switch sharp byte expectations.

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

The pinned Wrangler package and sole production configuration live beside the Rust Worker. From the repository
root run `npm --prefix rust/crates/worker run dev` or `npm --prefix rust/crates/worker run deploy`. Generated
workers-rs glue in `crates/worker/build/index.js` is an ignored build artifact required by Wrangler.


`crates/worker` uses workers-rs, built to WebAssembly with `worker-build --release`. It keeps the class
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
