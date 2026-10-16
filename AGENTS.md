# AGENTS.md

Instructions for any coding agent working in this repo (Claude Code reads it through `CLAUDE.md`; Codex reads it
directly). `README.md` explains how to run and verify the replica; `docs/` holds specs and plans.

## What this is

A localhost replica of mymind (`access.mymind.com`). `server.mjs` serves mymind's own front-end (`mirror/`) and
answers its API from recordings of the user's library (`replay/`, `capture/`). Parity with live mymind is proven
screenshot by screenshot with REA (`npm run verify`, `npm run coverage`).

## Hard rules

1. **Never change the user's mymind account.** Only the live pass of `tools/drive.mjs` and `tools/capture.mjs` talk
   to live mymind, and every non-GET they would send is answered by the replica instead. Keep it that way; never
   add code that writes to `access.mymind.com`.
2. **The repository is private data.** The user chose to commit the recordings of their library (`capture/`,
   `mirror/`, `replay/`) so a clone runs by itself; the GitHub repository must stay private. The database stays
   off Git: `.env`, `data/`, `library/`, `downloads/`, recorded videos and `evidence/` stay local. Run
   `npm run bundle` before committing library changes, and never put cookies, JWTs or API keys into commits,
   reports, logs or prompts (Codex prompts leave the machine too).
3. **Secrets live in `.env` only** (`TYPESAFE_API_KEY`, LLM keys, OAuth client secrets), read through
   `process.env` on the server, never shipped to the browser.
4. **No bot-detection workarounds.** No user-agent spoofing or stealth tricks against mymind.
5. **Git:** work on a branch, never force-push `main`. Commit only when the user asks.
6. **No local models.** Never run or download models locally (no Ollama, llama.cpp, transformers.js). Every AI
   feature calls a hosted API (see External services).
7. **Nothing public without sign-in.** A server reachable from outside the machine (tunnel, port forward) runs
   with `AUTH=google` and `OWNER_EMAIL` (README, Sign-in). Password users exist only through
   `tools/auth-user.mjs`, with generated passwords kept in `.env` (`--save-env`); use the `TEST_USER_*` account to
   test a signed-in site, never the owner's Google login. Reach the hosted site through Access with the service
   token in `.env` (`CF_ACCESS_CLIENT_*`, `node tools/check-hosted.mjs`, README "Testing the hosted library").
   The whole hosted hostname stays behind the Cloudflare Access application "mymind" (owner plus
   `@octaviantocan.com`, through Google or a one-time PIN), and the tunnel checks its token (`deploy/install.sh`).
   Never weaken that, the gate in `lib/auth.mjs`, the tunnel refusal in `server.mjs` or the Access token and mymind
   session checks in `deploy/download-server.mjs`, and ask the user before publishing any hostname. One exception,
   decided by the user on 2026-10-16 and not built yet: the native Android app's API path may leave Access, guarded
   by the app's own sign-in (Google for `OWNER_EMAIL` and verified `@octaviantocan.com` addresses, passwords for
   `tools/auth-user.mjs` users, bearer tokens on every other route). The user also wants the website's Google
   sign-in to admit `@octaviantocan.com`. Only that path; see `docs/android-plan.md`, "Sign-in", and confirm the path with the user
   before changing Access or the tunnel.

## Long-running processes

Verification is slow: a desktop `ui` suite takes 40–60 min, a phone `ui` suite about 25 min, a `cards` suite
15–20 min, the full matrix 1.5–2.5 h. Spend that time only where it proves something.

1. Run the narrowest check that proves the change: a patch suite `--suite <suite>@<part>` or a debug suite
   (`smoke`, `core`) for the one profile you are fixing. Add a patch suite in `tools/suites.mjs` if none fits.
2. Re-run a whole suite only when the change can affect all of it (`server.mjs`, `tools/side.mjs`,
   `tools/page-determinism.js`, verifier flags). Run the full matrix once, at the end, before claiming parity.
3. `npm run coverage` lists exactly which runs are stale (`verifier-result-incompatible`); re-run only those.
4. Before starting anything over 5 minutes, say how long it will take, run it in the background and wait for the
   completion notification instead of polling. Start runs over 30 minutes with `systemd-run --user --unit <name>` so
   they outlive the agent session (a session restart killed one), and wait on the unit. At most 2 drive runs in
   parallel (3 got the agent Chromium OOM-killed); check `free -g` first.
5. Delete `evidence/drive/claims/` before a fresh matrix, or every run is skipped as already claimed.
6. Never edit replica code (`server.mjs`, `tools/replay-store.mjs`, `lib/**/*.mjs`) while drive runs are going.
   Coverage hashes those files when it reports, so every run that finished before the edit becomes
   `verifier-result-incompatible` and has to be re-run.

Claude Code sandbox notes: `drive`/`verify` need the agent Chromium on CDP `:9222`, which the Bash sandbox cannot
reach, and `git` cannot read `~/.gitconfig` inside it; run both unsandboxed. Never `pkill -f` a pattern that also
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

- Plain Node ES modules, no framework or build step; `node:sqlite` for storage. Match the surrounding code.
- JSDoc on exported functions states the caller contract (summary, every `@param`, `@returns`); private helpers get
  a one-line summary; inline comments only for a non-obvious why. No section-divider comments.

## External services

- **Google Gemini API** (`GEMINI_API_KEY`): embeddings with `gemini-embedding-2` (text, PNG/JPEG images, PDF pages,
  audio, video in one space; `POST https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-2:embedContent`,
  task given as a text prefix such as `task: search result | query: …`), and summaries/tags with a cheap chat model
  through the OpenAI-compatible endpoint `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions`.
  EmbeddingGemma 2 itself is open weights only; `gemini-embedding-2` is Google's hosted equivalent.
- **TypeSafe (Jev)**: a classification model (typed yes/no, choice and score answers), not a text generator. Before
  using it, read the skill at https://github.com/typesafe-ai/skills/blob/main/skills/typesafe-ai/SKILL.md and
  the live docs at https://docs.typesafe.ai (start from `llms.txt`). API: `POST https://api.typesafe.ai/v1/systemone`
  with `Authorization: Bearer $TYPESAFE_API_KEY`.
