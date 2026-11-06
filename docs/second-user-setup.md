# Inviting another person

Assessed and configured on 2026-11-06. The user authorized an invitation for Esther's Google account and required
her own private library. Use the existing hosted Inventory service at `https://mymind.octaviantocan.com`.

## What an invitation needs

The Worker already creates a separate empty library on an admitted account's first library request. Its Auth
object keeps the identity-to-library mapping; each Library Durable Object has its own SQLite database, R2 prefix
and live-update connections. Existing mappings, including the owner's and the test user's, are preserved.
Do not copy the owner's database or recordings, import a library, or map another person to `owner` or the test
library. No new hostname, Worker, R2 bucket or Google OAuth client is needed for another person.

The Google gate admits the owner, verified addresses at `ALLOWED_GOOGLE_DOMAIN`, and exact verified addresses in
`ALLOWED_GOOGLE_EMAILS`. The latter is a comma-separated invitation list, trimmed and compared in lower case.
An unset or empty list adds nobody. It does not expand wildcards, Gmail dots or plus aliases. Keep the domain at
`octaviantocan.com`; setting it to `gmail.com` would admit everyone at Gmail.

Set `ALLOWED_GOOGLE_EMAILS` as a secret binding on the existing `mymind` Worker. Secret bindings survive the
normal Wrangler deployment; the Worker's runtime explicitly forwards this binding to the shared auth code.
The local server can read the same setting from `.env`, but it still serves one shared local database, so use
the Cloudflare Worker for separate libraries. `mymind auth-user` changes the local auth database; it does not
create an account on the hosted Worker. The hosted password uploader only supports `TEST_USER_*`.

## Google Cloud

Use the existing agent Chromium, signed in as `tocanoctavian@gmail.com`, and verify that account in the page
before editing project `mymind-replica` (display name `mymind replica`). Under Google Auth Platform > Audience,
keep the existing audience and add the invited account as a test user if the project is in Testing.
Save, reopen the page and confirm the address remains in the table. Preserve the owner and every callback URL.

Inventory's web flow requests only `openid email`; Android uses Sign in with Google. Google's
[audience documentation](https://support.google.com/cloud/answer/15549945?hl=en) exempts basic identity scopes
from the usual Testing allowlist and seven-day authorization expiry. Adding the invited test user explicitly
still records the intended access. Do not publish the OAuth app merely to invite one person. Workspace
administrator restrictions can independently block Google sign-in.

## What the person does

1. Open the existing hosted site and choose Continue with Google using the invited account.
2. The first library request creates an empty library and its initial user row. Saving a note, link or upload
   works without a manual library import.
3. For Android, sign in to the hosted website, open `/download` and install the existing signed Inventory APK.
   Then sign in on the phone with that same Google account. An invitation needs no new APK or app login.
4. Claude Code or Codex sign-in is optional and belongs to that person. It enables the subscription-backed AI
   search and feedback features. Ordinary saving does not require either. Hosted capture, summaries, tags,
   embeddings and transcription use the service's existing Gemini/OpenAI configuration.

The current service setting is `FEEDBACK_OPEN=everyone`: a signed-in person's feedback, using their own AI
subscription, can become an automatically merged change released to all users. See [feedback operations](feedback.md).
Libraries are isolated between ordinary accounts, but this is an operator-hosted service: usage analytics include
emails and search words, and the owner can query those analytics. See [analytics](analytics.md).

## Verification

The invitation is checked only after Google's token signature, audience, issuer, expiry, nonce and verified-email
checks. Web, Android and the MCP sign-in use the shared gate. The invitation does not grant owner-only analytics
or administration routes.

Focused checks, with synthetic Google tokens and in-memory databases:

```bash
cd rust
cargo test --offline --locked -j 2 -p mm-core \
  --test google_domain --test library_claim --test hosted_media --test auth --test connected_agents
cargo check --offline --locked -j 2 -p mm-worker --target wasm32-unknown-unknown
```

On 2026-11-06 the invitation acceptance tests failed before the implementation, then all 13 tests across these
five targets passed after it. They cover both sign-in paths, exact matching, unverified and uninvited emails,
case normalization, empty lists, disabled domain access, existing owner mappings, first-use library creation,
cross-library card reads, media handling and account-scoped connected agents. The Worker wasm check passed.

`node tools/check-hosted.mjs` checks the live signed-out surface, existing test-user sign-in, library reads and
protected downloads. It passed before the change; rerun after deployment. This uses `TEST_USER_*` from `.env`,
never the owner's Google login. It does not prove the invited person's first Google sign-in or phone behavior;
that final confirmation requires them to sign in with their own account.

Removing an address from the invitation list prevents new sign-ins, but existing sessions and app tokens remain
valid until revoked or expired; do not describe an invitation-list edit as immediate session revocation.
