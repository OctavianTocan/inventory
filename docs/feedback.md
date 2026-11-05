# Feedback agents

A signed-in ThyMind user writes feedback in the Android app, picks Claude Code or Codex with a model and an effort,
and an agent on their own subscription turns it into a merged pull request inside a Cloudflare sandbox. The design
(Tavi's decisions of 5 November 2026, the parts, job states and build order) is `feedback-agent/design_draft.md` in
the ThyMind project files. This page holds what the repository needs to know: the runner, the GitHub sign-in and
the spike's answers; the phone-facing routes, the kill switch and the runbook for a bad release come with the next
step.

## The runner

`feedback-runner/` is a TypeScript Worker (the Sandbox SDK is TypeScript) with no public route: ThyMind's Rust Worker
reaches it through the `FEEDBACK_RUNNER` service binding after its sign-in gate, with the sender's verified email in
`x-feedback-email` and their AI login in `x-feedback-login`.

- `FeedbackDesk` (`src/desk.ts`, one object): the `client_id` map, one active job per person, 5 sends per person per
  UTC day, 3 round slots for everyone, the merge queue (first in, first out), the pause switch and Tavi's record of
  the last 100 jobs.
- `FeedbackJob` (`src/job.ts`, one per feedback id): the job's record and state machine (`src/states.ts`, a pure
  function tested as a table per journey), driven by an alarm: every minute while a round runs, every 2 minutes while
  it waits on GitHub.
- `FeedbackSandbox` (`src/sandbox.ts`, one per round, `standard-1`): starts the container, trusts the outbound
  handler's certificate, writes the sign-in files into the agent's home, and runs `src/round.py` in the background.
  The round clones shallowly, runs the CLI for at most 45 minutes, adds the CHANGELOG entry the release gate asks for,
  refuses to push a commit holding a sign-in file's text, and pushes `feedback/<id>`.
- The outbound handler (`src/outbound.ts`) adds the GitHub sign-in to git traffic for this repository only, lets a push
  update only the round's own branch and never delete it, passes the engine's hosts (Claude Code: `api.anthropic.com`
  plus its sign-in hosts; Codex: `chatgpt.com`, `auth.openai.com`, `api.openai.com`) and GET downloads from
  `index.crates.io`, `static.crates.io` and `registry.npmjs.org`, and refuses everything else.
- Each round borrows the sender's sign-in from their `AgentHome` (`POST /feedback/checkout`, ai-kit) and gives it back
  with any refreshed tokens (`POST /feedback/checkin`). While it is out, the sender's AI search answers busy; a lost
  checkin ends after 60 minutes.
- Merging: the job at the head of the queue takes its CHANGELOG entry out of the branch (two branches that each add a
  top entry always conflict), merges `main` in with update-branch, puts the entry back numbered one above `main`'s,
  waits for the `android` and `rust` checks, and squash-merges with a `Feedback-Id:` trailer. A conflict sends the job
  back for a round that resolves it; `main` moving during CI retries at most 3 times.

`npm run check` in `feedback-runner/` type-checks and runs the tests (state tables, desk limits and queue, outbound
decisions, CHANGELOG renumbering checked with the release script's own reader, recorded GitHub answers); the `rust`
check runs it on every PR.

### GitHub sign-in

There is no GitHub App (Tavi, 5 November 2026). Like Pully (`backend/src/personal_auth/`), the runner uses one normal
GitHub CLI sign-in: GitHub's device flow with the GitHub CLI's public client, which GitHub lists as a privileged app,
so the consent page says "GitHub CLI". It is made once and reused for every job. It asks for `repo` only; without
`workflow`, GitHub itself refuses any push that changes `.github/workflows`. The token is the runner's `GITHUB_TOKEN`
secret and never enters a sandbox. Pushes, pull requests and merges appear as the account that signed in.

The spike (PR 1, removed in PR 3) answered the questions below on a real sandbox on 5 November 2026; its code is in
the history of `feedback-runner/spike/`.

## Open questions and answers

| Question | Answer | Source |
|---|---|---|
| Q1a: do Claude Code and Codex subscription sign-ins work through the outbound handler? | Yes. Both wrote and committed a file (Claude 9 s, Codex 18 s) and both answered with only the strict host list allowed. Claude needs `api.anthropic.com`; it also calls `mcp-proxy.anthropic.com` (the account's claude.ai connectors), which the strict list refused 18 times without harm. Codex needs `chatgpt.com` (HTTPS and its WebSocket, status 101); `ab.chatgpt.com` and `sdmntpr*.oaiusercontent.com` were refused without harm. Neither CLI refreshed its sign-in; only `.claude.json` (settings, not the token) changed. | Spike run, 5 Nov 2026, `standard-1`, Claude Code 2.1.318, Codex 0.166.0 |
| Q1b: can the handler read a push's refs and allow only the job's branch? | Yes. A push to `feedback/<id>` went through; a push to another branch and a delete of the job's branch were refused with 403 by the handler; a push changing `.github/workflows` was refused by GitHub ("without `workflow` scope"). The token never appeared inside the sandbox. | Spike run, 5 Nov 2026 |
| Q2: can the agent run the Android unit tests inside the sandbox? | Yes on `standard-3` (2 vCPU, 8 GiB, 16 GB) or larger, not on `standard-1`. With Gradle's dependency caches already in the image (no network, no build cache), a fresh clone builds and runs all 649 unit tests in 232 s on `standard-3` and 183 s on `standard-4`. On `standard-1` (4 GiB) the test JVM was killed for memory after 11 minutes and 160 tests, because Gradle (`-Xmx3g`) and the test JVM (2 GB) don't fit together. The image would grow by about 2.8 GB (Gradle 1.7 GB, Maven 0.4 GB, Android SDK 0.6 GB) plus a JDK. | Docker on a 4-core cloud machine with `--cpus`/`--memory` set to each instance type, 5 Nov 2026 |
| Q3: container cost per job | At most $0.0037 per minute on `standard-3` and $0.0012 on `standard-1`: memory and disk are billed as provisioned, CPU only when busy, so these are ceilings. A round capped at 45 minutes costs at most $0.17 on `standard-3`, and a job of 3 rounds at most $0.50. The whole spike run (start, clone, two short agent tasks, pushes, Q4) took 75 s; the sandbox started in 13 s and the shallow clone took 4 s (52 MB). | Cloudflare Containers pricing, 5 Nov 2026: $0.000020 per vCPU-second, $0.0000025 per GiB-second, $0.00000007 per GB-second |
| Q4: does update-branch work for a sign-in without the `workflow` scope when `main` has workflow changes the branch lacks? | Yes. From a branch made just before `main`'s last workflow commit, update-branch answered 202 and moved the branch, and the merges API answered 204, both with the `repo`-only sign-in. | Spike run, 5 Nov 2026 (draft PR #76, closed) |
