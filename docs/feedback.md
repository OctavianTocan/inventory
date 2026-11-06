# Feedback agents

A signed-in Inventory user writes feedback in the Android app, picks Claude Code or Codex with a model and an effort,
and an agent on their own subscription turns it into a pull request inside a Cloudflare sandbox, which the runner
merges once CI is green and Tavi has approved it. The design
(Tavi's decisions of 5 November 2026, the parts, job states and build order) is `feedback-agent/design_draft.md` in
the Inventory project files. This page holds what the repository needs to know: the routes, the runner, the GitHub
sign-in, review, deploying, the kill switch, the runbook for a bad release and the spike's answers.

## Routes

The app's routes are in `docs/app-api.md` (Feedback). Inventory's Rust Worker (`rust/crates/feedback`, `mm-feedback`)
checks each send: the phase (`FEEDBACK_OPEN`: `owner` lets only `OWNER_EMAIL` send, `everyone` lets everyone signed
in), the text, that the sender is signed in to the engine and that the model is on its current list. It then passes
the send to the runner with the verified email and the sender's AI login. The runner answers for the job limits
(one active job per person, 5 a day, the pause switch).

## The runner

`feedback-runner/` is a TypeScript Worker (the Sandbox SDK is TypeScript) with no public route: Inventory's Rust Worker
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
  The round clones shallowly with the `vendor/ai-kit` submodule, clones each `READ_REPOS` repository (Pully and
  ai-kit) with its whole history but without file contents until read (`--filter=blob:none`) into
  `/workspace/context/<name>`, runs the CLI for at most 45 minutes, adds the CHANGELOG entry the release gate asks for,
  refuses to push a commit holding a sign-in file's text, and pushes `feedback/<id>`.
- The outbound handler (`src/outbound.ts`) adds the GitHub sign-in to git traffic for this repository and the
  `READ_REPOS` ones (fetch only, never push; Tavi, 5 Nov 2026), lets a push update only the round's own branch and
  never delete it, passes the engine's hosts (Claude Code: `api.anthropic.com`
  plus its sign-in hosts; Codex: `chatgpt.com`, `auth.openai.com`, `api.openai.com`) and GET downloads from
  `index.crates.io`, `static.crates.io` and `registry.npmjs.org`, and refuses everything else.
- Each round borrows the sender's sign-in from their `AgentHome` (`POST /feedback/checkout`, ai-kit) and gives it back
  with any refreshed tokens (`POST /feedback/checkin`). While it is out, the sender's AI search answers busy; a lost
  checkin ends after 60 minutes.
- Review: a pull request with green checks is not merged yet (see Review below); the job waits in `review` until
  Tavi approves it.
- Merging: once approved, the job at the head of the queue takes its CHANGELOG entry out of the branch (two branches that each add a
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

## Review

Tavi, 6 November 2026: "When someone sends feedback through the app, we need to mark that PR for me to review,
instead of letting it through directly." This replaces the 5 November decision to merge on green CI with no human
check.

When the `android` and `rust` checks pass, the runner adds the `needs-review` label (`REVIEW_LABEL`) to the pull
request and the job moves to `review`; the sender's notification reads "Waiting for Tavi's review". The runner then
asks GitHub every 5 minutes and nothing else moves the job:

- **Approve:** add the `approved` label (`APPROVE_LABEL`; the runner creates it so it is in the label menu). The job
  joins the merge queue, which brings `main` in, renumbers the CHANGELOG entry above `main`'s, waits for the checks
  again and squash-merges. This is the safe way, since a hand merge can collide with another release's version.
- **Merge by hand:** the runner notices the merge and goes on to the release check as if it had merged it.
- **Turn it down:** close the pull request. The job fails with "Tavi decided not to ship it."

Any new agent round after an approval (a red check after `main` came in, or a conflict) takes the `approved` label off
and ends in `review` again, because the code changed after Tavi read it. Review has no time limit, and a job in review
does not count as the sender's active job, so they can send more; the job's 10-hour limit counts again from the
approval. The pull requests appear as the account of the GitHub sign-in (Tavi's), so GitHub cannot request his review
or take his approval; find them with the `needs-review` label.

## Deploying

Before the first deploy, once, make the GitHub sign-in: GitHub's device flow with the GitHub CLI's client and scope
`repo`, as Pully's `backend/src/personal_auth/` and the spike's `run.mjs` did (the spike kept it in
`~/.config/thymind/feedback-github.json` on the machine that ran it). Set it with `cd feedback-runner && npx wrangler
secret put GITHUB_TOKEN`. `main` needs no branch protection for feedback: the runner merges only when every check in
its `REQUIRED_CHECKS` var (`android,rust`) passed and the pull request carries the approve label.

Then `deploy/deploy-workers.sh` (or `npm run deploy:all` in `rust/crates/worker`) deploys `feedback-runner` first and
`mymind` second, since each binds to the other. Before deploying it lists every commit with a `Feedback-Id:` trailer
since the last deploy and flags changes to the sign-in files (`rust/crates/core/src/auth.rs`,
`rust/crates/server/src/serve.rs`, `rust/crates/server/src/download.rs`), any `wrangler.jsonc` (which carries Durable
Object migrations), `feedback-runner/` and the ai-kit pin, so they are read before saying yes. `npm run deploy` stays
the plain `mymind` deploy, because the VPS timer `mymind-worker-update.timer` runs it every two minutes when `main`
moves and cannot answer a prompt; `feedback-runner` is only deployed by hand.

## Kill switch

`POST /app/v1/feedback/pause` with `{"paused": true}`, as the owner, stops new rounds and merges at once without a
deploy; rounds already running finish and their jobs wait. `{"paused": false}` starts them again. To stop sends from
everyone but the owner, set `FEEDBACK_OPEN` back to `owner` and deploy `mymind`. `GET /app/v1/feedback` shows the
last 100 jobs.

## A bad release

A merged feedback change publishes an Android release to every phone once Tavi approves it on green CI.

- If the app still starts and updates: pause feedback, then revert the feedback's commit (its `Feedback-Id:` trailer
  names the job) in a pull request that also adds a newer `android/CHANGELOG.md` entry, and merge it. Phones update
  to the revert as usual.
- If the change broke the updater or start-up, phones cannot fetch a fix: publish the revert as above, then install
  it by hand on each phone from `/download` (signed in) or with `adb install`.
- A bad Worker change is undone with Cloudflare's version rollback (`npx wrangler rollback` in `rust/crates/worker`
  or `feedback-runner`), then a revert on `main`. Durable Object storage changes do not roll back.

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
