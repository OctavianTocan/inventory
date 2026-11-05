# AGENTS.md

Instructions for any coding agent working in this repo (Claude Code reads it through `CLAUDE.md`; Codex reads it
directly). `README.md` explains how to run and verify the replica; `docs/` holds specs and plans.

## What this is

A localhost replica of mymind (`access.mymind.com`). The Rust binary (`mymind serve`, `rust/`) serves mymind's own
front-end (`mirror/`) and answers its API from the local database and storage.
It is its own product: nothing checks it against mymind any more.

## Hard rules

1. **Never change the user's mymind account.** Nothing in this repo writes to `access.mymind.com`.
2. **The repository is private data.** `mirror/` holds mymind's front-end and is committed so a clone can serve the
   UI; the GitHub repository must stay private. The database stays off Git: `.env`, `data/`, `library/`,
   `downloads/` and `evidence/` stay local. Run `npm run bundle` before committing library changes, and never put
   cookies, JWTs or API keys into commits, reports, logs or prompts (Codex prompts leave the machine too).
3. **Secrets live in `.env` only** (`OPENAI_API_KEY`, LLM keys, OAuth client secrets), read through
   the server process environment, never shipped to the browser.
4. **No bot-detection workarounds.** No user-agent spoofing or stealth tricks against mymind.
5. **Git:** work on a branch, never force-push `main`. Commit each finished, verified step and push the branch to
   `origin` (the user, 2026-10-17: commit and push more often). Subagents and Codex runs leave their work
   uncommitted; the main session reviews it, then commits.
6. **No local models.** Never run or download models locally (no Ollama, llama.cpp, transformers.js). Every AI
   feature calls a hosted API (see External services).
7. **Nothing public without sign-in.** A server reachable from outside the machine (tunnel, port forward) runs
   with `AUTH=google` and `OWNER_EMAIL` (README, Sign-in); a server without sign-in refuses tunnel requests
   (`rust/crates/server/src/serve.rs`). The server's own sign-in is the only gate the user wants: Google for
   `OWNER_EMAIL` and verified `@octaviantocan.com` addresses, passwords only for users made with `mymind auth-user`
   (generated passwords in `.env` via `--save-env`), and no sign-up. Use the `TEST_USER_*` account to test a
   signed-in site, never the owner's Google login. Never weaken `rust/crates/core/src/auth.rs`, the tunnel refusal,
   or the session check in `rust/crates/server/src/download.rs`, and ask the user before publishing any hostname.

   Cloudflare (the user, 2026-10-16): no Cloudflare Access, only the tunnel; Access was removed the same day. The
   hostname is public, so signed-out tunnel visitors get the replica's own plain sign-in page (no mymind name, logo
   or assets, `noindex`), never mymind's recorded one; keep it that way. `node tools/check-hosted.mjs` tests the
   hosted site as the test user. Google sign-in admits `OWNER_EMAIL` and a verified address at
   `ALLOWED_GOOGLE_DOMAIN` (default `octaviantocan.com`; set it empty for owner-only). Do not widen that gate
   further (`docs/accounts-plan.md`).

## Long-running processes

Before starting anything over 5 minutes, say how long it will take, run it in the background and wait for the
completion notification instead of polling. Start runs over 30 minutes with `systemd-run --user --unit <name>` so they
outlive the agent session (a session restart killed one), and wait on the unit.

Claude Code sandbox notes: `git` cannot read `~/.gitconfig` inside the sandbox; run it unsandboxed. Never `pkill -f` a pattern that also
matches your own shell command.

## Delegation

The main session plans, reviews and owns every verdict ("is it 1:1", "is it done"). It reads each subagent result,
spot-checks claims against files or command output, and never passes an unverified claim to the user. Subagents
share the user's usage limits, so route each task to the cheapest model that does it well.

| Task | Model | Invocation |
|---|---|---|
| File search, grep-and-summarize, log triage, running tests or a verify and summarizing the result | Haiku 5.5 | `Agent(subagent_type: "Explore", model: "haiku")` (Explore otherwise inherits the main model) |
| Mechanical edits with a precise spec (rename, boilerplate, fixture data), then verify with a test | Haiku 5.5 | `Agent(model: "haiku")` |
| Web research and synthesis, implementing a feature from a written spec, writing tests, first-pass review | Sonnet 5.5 | `Agent(model: "sonnet")` |
| Hard debugging, architecture, tricky algorithms (e.g. reverse engineering server templates) | Codex `gpt-6.1-sol` at `xhigh`, or the main session | see below |
| Independent second opinion or adversarial review of a risky change | Codex `xhigh`, read-only | see below |
| Long-horizon work where the main model at high effort has already failed | Fable 5.1 | `Agent(model: "fable")`, 2.5x Opus price; ask first |

Prices per million tokens (input/output): Haiku 5.5 $0.10/$0.50, Sonnet 5.5 $2/$10, Opus 5.5 $4/$20, Fable 5.1
$10/$50. Codex bills the user's Codex plan, not Claude usage.

Codex runs non-interactively; write the prompt to a file and run it in the background (`xhigh` can take 10–30 min):

```bash
# Review / second opinion (cannot edit files)
codex exec -m gpt-6.1-sol -c model_reasoning_effort=xhigh -s read-only -C "$PWD" --ephemeral \
  -o "$TMPDIR/codex-review.md" - < "$TMPDIR/codex-prompt.md"
# Implementation, on its own branch or worktree; review the diff before keeping any of it
codex exec -m gpt-6.1-sol -c model_reasoning_effort=xhigh -c approval_policy=never -s workspace-write \
  -C "$PWD" --ephemeral -o "$TMPDIR/codex-impl.md" - < "$TMPDIR/codex-prompt.md"
```

Do not delegate:

- dependent chains where each step needs the previous result plus judgment;
- edits under about 20 lines;
- anything that touches the live account.

Every subagent prompt is self-contained: paths, goal, constraints (read-only or not, never contact live mymind), the
output format and a length cap.

## Code conventions

- Before changing Android UI, read applicable ancestor and area `CONTRACTS` files. Card-sheet navigation follows
  `android/app/src/main/kotlin/com/octaviantocan/mymind/ui/menu/CONTRACTS`; its pinned Pully source is documented in
  `docs/pully-ui-reference.md`. Keep the implementation, tests and contracts consistent in the same change.
- The backend is Rust; public rustdoc states the caller contract. Retained Node tooling uses plain ES modules,
  no framework or build step. Match the surrounding code.
- `feedback-runner/` is the one TypeScript Worker, because the Cloudflare Sandbox SDK is TypeScript
  (`docs/feedback.md`). Its `npm run check` runs in the `rust` check.
- JSDoc on exported functions states the caller contract (summary, every `@param`, `@returns`); private helpers get
  a one-line summary; inline comments only for a non-obvious why. No section-divider comments.

## Feedback agents

Feedback sent from the app is implemented by an agent in a sandbox and merged on green CI (`docs/feedback.md`). In
that work, do not change version numbers or `android/CHANGELOG.md`; the feedback runner does that.

## External services

- **Google Gemini API** (`GEMINI_API_KEY`): embeddings with `gemini-embedding-2` (text, PNG/JPEG images, PDF pages,
  audio, video in one space; `POST https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-2:embedContent`,
  task given as a text prefix such as `task: search result | query: …`), and summaries/tags with a cheap chat model
  through the OpenAI-compatible endpoint `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions`.
  EmbeddingGemma 2 itself is open weights only; `gemini-embedding-2` is Google's hosted equivalent.
- **OpenAI Decisions API** (`OPENAI_API_KEY`): typed answers, not text: `predicate` (a probability), `choice` (one
  of up to 255 options, with probabilities) and `score`, up to 400 questions per request about one input of text,
  JSON and images of any size. Model `gpt-6-luna`; $0.10 per 1M input tokens, output free. API:
  `POST https://api.openai.com/v1/decisions` with `Authorization: Bearer $OPENAI_API_KEY`; read the live guide at
  https://developers.openai.com/api/docs/guides/decisions before changing a call. It picks final tags (`Ai::pick_tags`,
  replacing TypeSafe Jev on 5 Nov 2026). The same key runs video transcription.
- **OpenAI speech to text** (`OPENAI_API_KEY`, the user's choice on 2026-11-05 over xAI): transcribes saved videos and audio files,
  `POST https://api.openai.com/v1/audio/transcriptions` (multipart `model`, `file`), model `gpt-transcribe`
  (OpenAI's recommended one; `gpt-4o-*-transcribe` and `whisper-1` shut down 2027-02-26), 25 MB per file,
  $0.0045 a minute; docs at https://developers.openai.com/api/docs/guides/speech-to-text.
