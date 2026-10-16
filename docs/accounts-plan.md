# Accounts (roadmap phase 8)

Status: owner-only Google sign-in is built (`lib/auth.mjs`, proven by `tools/check-auth.mjs`). Multiple accounts
are not.

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
   `nonce`, `email_verified`) and lets in only `OWNER_EMAIL`.
4. **Sessions in SQLite** (`AUTH_DB`, default `data/auth.db`). A random 256-bit id in an HttpOnly, SameSite=Lax
   cookie (`Secure` when `PUBLIC_URL` is https); only its SHA-256 is stored, for 30 days. `/signout` deletes it.
5. **The gate.** Signed-out page loads redirect to `/signin`; API calls and `/hub` upgrades get 401; cross-origin
   writes get 403.

## Hosting

Installed on mini-002 (README, Hosting): `deploy/install.sh` runs the server as a systemd user service with
`AUTH=google` on `127.0.0.1:4330`, behind the Cloudflare tunnel `mymind` at `mymind.octaviantocan.com`, and serves
the packaged desktop app at `/download` behind a Cloudflare Access application (owner's email only). The Google
client (project `mymind-replica`, consent screen in Testing with the owner as the only test user) lists that host's
callback and the two localhost ones. The same process serves the host without sign-in on `LOCAL_PORT` (4331), so
one process owns `data/mymind.db`; verification reads its own `data/verify.db`.

The desktop app opens the hosted library when built with `MYMIND_URL` and signs in with Google inside its window.
Password users (`tools/auth-user.mjs`, `/signin/password`) exist for testing the signed-in site without a Google
login.

## Not built: more than one account

Needed only if anyone besides the owner should get a library:

- `server.mjs` falls back to `replay/` and `capture/`, which are the owner's library; every other account must
  skip those fallbacks and start empty.
- Each request resolves its user and passes it to the store, writer, search, uploads and media
  (`lib/store.mjs` already takes a `userId`); hub connections only receive their own user's events.
- The page shell is the owner's recorded `/everything`; other accounts need one generated from their own config,
  pins and spaces.
- Proof: a second account sees an empty library and cannot read or write the owner's cards.
