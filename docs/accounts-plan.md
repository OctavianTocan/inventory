# Plan: accounts (roadmap phase 8)

Status: plan only; nothing below is built yet.

## Where things stand

The replica serves one person. `lib/store.mjs` already takes a `userId` on every read and write (it defaults to the
first user), but everything around it assumes that one user:

- `server.mjs` falls back to `replay/` and `capture/` for anything the store does not answer. Those recordings are
  the owner's library, so for any other account each fallback would leak the owner's data.
- The page shell is the owner's recorded `/everything` HTML; `api.shell()` only patches settings, pins and quotas.
- `/hub` pushes every write to every connected client.
- Uploaded media lives under `data/media/<account>/`, already per account.

## Design

1. **Off unless configured.** Without `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` the server behaves exactly as
   today: one owner, no sign-in. The desktop app and every verification run use this mode, so pixel parity is
   unaffected.
2. **Google sign-in, hand-rolled OIDC.** `GET /auth/google` redirects to Google with PKCE (S256), `state` and
   `nonce`. `GET /auth/google/callback` exchanges the code, verifies the ID token (RS256 against Google's cached
   JWKS; `iss`, `aud`, `exp`, `nonce`, `email_verified`) and finds or creates the user by Google `sub`.
3. **Sessions in SQLite.** A random 256-bit session id goes into an HttpOnly, SameSite=Lax cookie (`Secure` behind
   HTTPS); only its SHA-256 is stored, with user, expiry (30 days) and last use. `GET /signout` (the link on
   mymind's account page) deletes the session and returns to the sign-in page.
4. **The owner keeps the recorded library.** `OWNER_EMAIL` maps that Google account to the existing user, who keeps
   every replay/capture fallback. Other accounts get only what the store generates, plus account-independent
   recordings (static assets, components, templates); a new account starts with an empty library.
5. **Per-account everything else.** Each request resolves its user once; the API passes it to the store, writer,
   search, uploads and media; hub connections are tagged with their user and only receive that user's events;
   the shell is generated from a template with the account's own config, pins and spaces.
6. **Write protection.** Writes require the per-session authenticity token that the shell embeds, which mymind's
   client already sends as `x-authenticity-token`.
7. **Who may sign in.** `ALLOWED_EMAILS` (comma-separated) limits sign-in; empty means anyone with a Google account.

## Proof

- `tools/check-auth.mjs` against a local stub identity provider (its own JWKS and token endpoint): sign in, sign
  out, expired and tampered tokens, wrong `nonce`/`state`, a second account sees an empty library and cannot read
  or write the owner's cards, hub isolation.
- With accounts off: `npm run verify -- --replica-only` and `npm run coverage` still report every screen identical.

## Needed from you

1. A Google OAuth client (Google Cloud Console → APIs & Services → Credentials → OAuth client ID → Web application)
   with redirect URI `http://localhost:4321/auth/google/callback` (plus the cloud URL later); put
   `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` in `.env`.
2. Who may sign in: only you, a list of people, or anyone.
3. The sign-in page: mymind's own logged-out page (recorded with one cookie-less GET, showing only the Google
   button), or a minimal page of our own.
4. Where the cloud version will run, which decides the public URL, HTTPS and cookie settings.
