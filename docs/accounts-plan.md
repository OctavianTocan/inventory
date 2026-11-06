# Accounts (roadmap phase 8)

Current invitation setup (2026-11-06): [Inviting another person](second-user-setup.md). The Worker creates a private
empty library for a newly admitted account; `ALLOWED_GOOGLE_EMAILS` adds exact verified Google invitations beside
the owner and the allowed domain. The historical local-server plan below is still not built.

Status: owner-only Google sign-in is built (`rust/crates/core/src/auth.rs`, proven by `tools/check-auth.mjs`), and Cloudflare Access
is gone (2026-10-16). On the Worker, an account the gate already admits gets an empty library at its first library
request when it has none (`library_for_session` in `rust/crates/core/src/auth_store.rs`). Google admits `OWNER_EMAIL`
and, unless `ALLOWED_GOOGLE_DOMAIN` is set empty, a verified address whose host is exactly that domain (default
`octaviantocan.com`). The local-server plan below is not built.

## What is built

1. **Off unless configured.** Without `AUTH=google` the server behaves as before: one owner, no sign-in. The
   desktop app and every verification run use this mode, so pixel parity is unaffected. In this mode requests that
   arrive through a Cloudflare tunnel are refused.
2. **mymind's own sign-in page.** `tools/capture-signin.mjs` records `/signin` and its "account not found" variant
   from a cookie-less context, and mirrors the assets they load. The server serves them with its own Google flow
   wired in; the Apple button does nothing.
3. **Google sign-in, hand-rolled OIDC.** mymind's button opens `/google/oauth2-redirect` in a popup; the server
   redirects to Google with PKCE (S256), `state` (bound to a cookie) and `nonce`. `/google/oauth2-callback`
   exchanges the code, verifies the ID token (RS256 against Google's cached JWKS; `iss`, `aud`, `exp`/`iat`,
   `nonce`, `email_verified`) and lets in `OWNER_EMAIL` plus a verified address at `ALLOWED_GOOGLE_DOMAIN`. Unset, that
   domain is `octaviantocan.com`. An empty value, or one that is not a hostname, admits only the owner. The host
   must match exactly: no subdomain, and no longer or lookalike suffix.
4. **Sessions in SQLite** (`AUTH_DB`, default `data/auth.db`). A random 256-bit id in an HttpOnly, SameSite=Lax
   cookie (`Secure` when `PUBLIC_URL` is https); only its SHA-256 is stored, for 30 days. `/signout` deletes it.
5. **The gate.** Signed-out page loads redirect to `/signin`; API calls and `/hub` upgrades get 401; cross-origin
   writes get 403.
6. **An empty library at first sign-in, on the Worker.** A signed-in account with no `libraries` row gets one on the
   first request that needs a library (`library_for_session` in `rust/crates/core/src/auth_store.rs`). The Auth
   object routes that request to a new Durable Object, and that object inserts one user row so the grid and a card
   save work (`rust/crates/core/src/store_member.rs`). An account that already has a library keeps it. The name is
   `u-<slug>-<sha256 of the account>`, lower-case letters, digits and dashes. If that name is already another
   account's library, the suffix is `-2`, `-3`, and so on. Password accounts and Google accounts
   both get one. A domain account gets its own library; the owner's existing mapping stays.

## Hosting

Installed on mini-002 (README, Hosting): `deploy/install.sh` runs the server as a systemd user service with
`AUTH=google` on `127.0.0.1:4330`, behind the Cloudflare tunnel `mymind` at `mymind.octaviantocan.com`, and serves
the packaged desktop app at `/download` to anyone with a session. Nothing sits in front of the server's sign-in (see
"Done: removing Cloudflare Access" below). The Google
client (project `mymind-replica`, consent screen in Testing with the owner as the only test user) lists that host's
callback and the two localhost ones. The same process serves the host without sign-in on `LOCAL_PORT` (4331), so
one process owns `data/mymind.db`; verification reads its own `data/verify.db`.

The desktop app opens the hosted library when built with `MYMIND_URL` and signs in with Google inside its window.
Password users (`mymind auth-user`, `/signin/password`) exist for testing the signed-in site without a Google
login.

## Decisions (the user, 2026-10-16)

1. **Who is welcome:** the owner's Google account, any verified `@octaviantocan.com` Google account, and the password
   test user. No sign-up.
2. **A library per person**, in a proper multi-user database: not every account reaching the owner's library, as
   today. Plan it before the Kotlin app or in parallel; both are fine.
3. **The test user gets the current library, one to one**, and verification runs signed in as the test user, so
   every parity check keeps passing while the owner's own library changes with use.
4. **No Cloudflare Access.** The server's sign-in is the only gate; the Cloudflare tunnel stays.
5. **Google Cloud setup by the agent.** Consent screen, test users or publishing, and the Android OAuth client: the
   agent does them itself in the agent browser, as it did for the Web client.

## Plan: a library per person (not built)

The user (2026-10-16): one database per person is fine if it runs on Cloudflare, with D1 and a Worker in place of the
tunnel; how to set that up is being discussed in `docs/cloudflare-plan.md`. The local plan below is the starting point.

### Where each library lives

**Recommended: one SQLite file per person**, all with today's schema, plus a list of who owns which file.

- Card, space and tag ids are the tables' primary keys, and the test user's copy must keep the recorded ids, or the
  recorded detail views and replays (keyed by those ids) stop lining up with it. In one shared file the owner's
  2,278 cards would collide with the test user's copy: either the owner's real library gets new ids (rewriting its
  rows and the media paths that embed them) or every key in every table becomes (user, id) and every query in
  `rust/crates/core/src/` changes. Separate files keep every id as it is and need no SQL changes.
- Isolation comes from the file, not from remembering a `WHERE user_id = ?`. Today `store.space(id)`,
  `store.cardTags`, `spaceCards`, shares, pins and `api_metadata` read by id alone, and the search index has no
  user column; in a shared file each of those would leak another person's cards.
- The other choice, one shared file with per-user rows, is possible only with the key change above.

Layout:

| Who | Library | Recordings (`capture/`, `replay/`) |
|---|---|---|
| Owner (`OWNER_EMAIL`, Google) | `data/mymind.db`, `data/media`, `data/uploads`: unchanged, nothing moves | yes |
| Test user (`TEST_USER_NAME`, password) | `data/libraries/<name>/library.db` (+ `media/`, `uploads/`), a one-to-one copy of `data/verify.db`, the recorded library | yes |
| A verified `@octaviantocan.com` account (stage 2) | `data/libraries/<id>/library.db`, created empty at first sign-in | no |

`data/auth.db` gets a `libraries` table: sign-in identity (the Google email, or `password:<name>`), folder, whether
the recordings apply, created time. Without `AUTH=google` nothing changes: one library from `STORE_DB`, as the
desktop app and verification use today.

### Stage 1: owner and test user

1. `rust/crates/core/src/auth.rs` exports who a request is (`user(req)`: identity, email, kind) next to `allows(req)`.
2. A library registry opens a library's store, writer, search, uploads, media, enrichment and app API on its first
   request and keeps them open; `rust/crates/server/src/serve.rs` routes each signed-in request to its library. `LOCAL_PORT` (loopback,
   no sign-in) keeps serving the owner's library.
3. The hub ties each connection to its library and sends a library's events only to its own connections.
4. The recordings are served only to libraries marked for them, so a library without them can never be answered
   from the owner's recorded cards, details or shell.
5. `tools/library.mjs` creates the test user's library from `data/verify.db` and lists libraries.
6. Verification: `tools/drive.mjs` signs the replica side in as the test user through `/signin/password`; the
   server it checks keeps reading a pristine copy (`data/verify.db`), so writes made while testing as the test user
   never reach a parity run.
7. Proof: the owner's library is byte-for-byte unchanged (row counts and a checksum before and after); the test
   user sees the recorded library; a second password user sees nothing of either and gets 404 for their card ids,
   media and spaces, on the web routes, `/app/v1/*` and the hub; every check suite passes; a drive sample signed in
   as the test user stays pixel-identical (the full re-proof can wait, as the user agreed).

### Stage 2: domain accounts

The access rule and the empty library are built. A verified Google address at `ALLOWED_GOOGLE_DOMAIN` (default
`octaviantocan.com`) signs in on the web and at `/app/v1/auth/google`, then gets its own library on the first
request that needs one. `OWNER_EMAIL` keeps the library it already has.

Still open: the Google Cloud OAuth consent screen has to allow those accounts. That is a manual step for the owner.
For a Workspace domain, set the screen to Internal. Otherwise add each address as a test user, or publish the app.
The code cannot do that.

- A library without recordings uses the shared shell from its own settings, pins and spaces. The owner's recorded
  page is not copied.
- Proof in `google_domain`: a verified domain account is admitted and gets a new library; an unverified domain
  email, another domain, and lookalike suffixes are rejected; the owner's library name stays the same. A card save
  on a new library is covered by `library_claim`.

## Done: removing Cloudflare Access (2026-10-16)

- **Cloudflare:** the Access application "mymind", its two policies ("mymind-bypass", "owner and
  @octaviantocan.com") and the "mymind-bypass" service token are deleted; the tunnel stays. No other Access
  application, policy or token was touched.
- **Code:** the tunnel config no longer requires an Access token (`deploy/install.sh`), and `/download` checks only
  the library session (`rust/crates/server/src/download.rs`). `tools/check-hosted.mjs` needs no service token, and the
  `CF_ACCESS_*` variables are gone from both `.env` files.
- **What a signed-out visitor sees (the user agreed):** an exact copy of a real company's sign-in page on another
  domain is what phishing filters flag, so requests arriving through the tunnel (`cf-connecting-ip`) get a plain
  sign-in page of our own:
  - no mymind name, logo, favicon or assets, and `noindex`, with `robots.txt` disallowing everything;
  - only `/signin` and the Google OAuth paths are open; mymind's assets and the API answer 401.
  Loopback visitors and verification still get mymind's recorded page, and everything after sign-in is unchanged
  (`rust/crates/core/src/auth.rs`, `tools/check-auth.mjs`).
- **Proof:** `tools/check-auth.mjs` 40/40, `tools/check-download.mjs` 7/7, `tools/check-hosted.mjs --browser` 11/11
  against `https://mymind.octaviantocan.com`.
