# Accounts (roadmap phase 8)

Status: owner-only Google sign-in is built (`lib/auth.mjs`, proven by `tools/check-auth.mjs`). A library per
person and dropping Cloudflare Access are decided, not built.

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
the packaged desktop app at `/download` to anyone with a session. The whole host sits behind the Cloudflare Access
application "mymind" (owner plus `@octaviantocan.com`) until Access is removed (below). The Google
client (project `mymind-replica`, consent screen in Testing with the owner as the only test user) lists that host's
callback and the two localhost ones. The same process serves the host without sign-in on `LOCAL_PORT` (4331), so
one process owns `data/mymind.db`; verification reads its own `data/verify.db`.

The desktop app opens the hosted library when built with `MYMIND_URL` and signs in with Google inside its window.
Password users (`tools/auth-user.mjs`, `/signin/password`) exist for testing the signed-in site without a Google
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

## Not built: a library per person

- `server.mjs` falls back to `replay/` and `capture/`, the recorded library. That becomes the test user's library;
  the owner's library is the one in `data/mymind.db` today; every other account starts empty.
- Each request resolves its user and passes it to the store, writer, search, uploads and media
  (`lib/store.mjs` already takes a `userId`, every table has `user_id`); hub connections only receive their own
  user's events.
- The page shell is the recorded `/everything`; other accounts need one generated from their own config, pins and
  spaces.
- A Google sign-in from an allowed domain address with no account creates one with an empty library; Google admits
  only `OWNER_EMAIL` until this exists. The consent screen is in Testing with only the owner listed, so domain
  accounts also need adding as test users or the consent screen published.
- Verification (`tools/drive.mjs`) signs in as the test user instead of running with sign-in off.
- Proof: the test user's screenshots stay pixel-identical; a second account sees an empty library and cannot read or
  write anyone else's cards.

## Not done: removing Cloudflare Access

Remove the Access application "mymind" and its policies (including the "mymind-bypass" service token), drop
`access.required` from the tunnel config (`deploy/install.sh`), drop the Access token check from
`deploy/download-server.mjs` (keeping the session check), and switch `tools/check-hosted.mjs`, README and AGENTS.md
rule 7 to the sign-in gate alone.

Open, to settle with the user first: what a signed-out visitor sees. Today `/signin` is mymind's own recorded sign-in
page and `/signin/password` a "Sign in to mymind" form. Behind Access only the owner's domain can load them; without
it anyone can, and an exact copy of a real company's sign-in page with a password form on another domain is what
phishing filters (Google Safe Browsing, Cloudflare) flag, which can put a warning on the whole domain and invites
people to type their mymind password. Proposed: signed-out visitors on the public hostname get a plain sign-in page
of our own (no mymind name or logo) and `noindex`; mymind's recorded page stays for loopback and verification, and
everything after sign-in is unchanged.
