# Feedback agents

A signed-in ThyMind user writes feedback in the Android app, picks Claude Code or Codex with a model and an effort,
and an agent on their own subscription turns it into a merged pull request inside a Cloudflare sandbox. The design
(Tavi's decisions of 5 November 2026, the parts, job states and build order) is `feedback-agent/design_draft.md` in
the ThyMind project files. This page holds what the repository needs to know: the spike's answers so far, and later
the routes, the GitHub sign-in, the kill switch and the runbook for a bad release.

## The spike (PR 1)

`feedback-runner/spike/` is a throwaway Worker that answers the design's open questions on a real sandbox. PR 3
deletes it. It deploys one sandbox (`standard-1`, no direct internet), sends every HTTP and HTTPS request through an
outbound handler that logs host names only, and:

1. writes the sign-in files the way ai-kit keeps them (`.claude/.credentials.json`, `.claude.json`,
   `.codex/auth.json`, only the fields ai-kit keeps) into the agent user's home;
2. clones the repository shallowly (`--depth 1`) with no credentials inside the sandbox; the handler adds the stored
   GitHub sign-in to git requests for this one repository and refuses every other GitHub path;
3. asks each CLI to write and commit one file with every host allowed and logged, then asks it for a one-word reply
   with only the design's host list allowed;
4. pushes to its own branch `feedback/<id>` (allowed), to another branch and as a delete (both refused by the
   handler, which reads the ref lines of the push), and a commit that changes `.github/workflows` (refused by GitHub,
   because the sign-in has no `workflow` scope), and checks the token appears nowhere in the sandbox;
5. for Q4, branches from just before `main`'s latest workflow change, opens a draft pull request, calls update-branch
   and the merges API with the same sign-in, then closes the pull request and deletes the branches.

### GitHub sign-in

There is no GitHub App (Tavi, 5 November 2026). Like Pully (`backend/src/personal_auth/`), the runner uses one normal
GitHub CLI sign-in: GitHub's device flow with the GitHub CLI's public client, which GitHub lists as a privileged app,
so the consent page says "GitHub CLI". It is made once and reused for every job. It asks for `repo` only; without
`workflow`, GitHub itself refuses any push that changes `.github/workflows`. The token stays in the Worker (a secret)
and never enters a sandbox. Pushes and pull requests appear as the account that signed in.

### Running it

It needs Docker running, `npx wrangler login` once, and Claude Code and Codex signed in on the computer that runs it.

```sh
cd feedback-runner/spike && npm ci
npm run spike
```

The first run asks you to approve a GitHub CLI sign-in at github.com/login/device and keeps it in
`~/.config/thymind/feedback-github.json` (mode 600) for later runs. It then deploys `feedback-spike` to your account, sets its secrets, runs once (5-15 minutes), writes
`spike-report.json` and deletes the Worker. The report holds host names, timings and outcomes, never sign-ins or
keys. Options: `--engines claude` or `codex`, `--claude-model`, `--codex-model`, `--effort`, `--no-q4`, `--keep`.

Sign-ins leave the computer only for the sandbox the run just deployed and are never stored. The run refuses a Claude
sign-in that would refresh within the hour (run `claude -p hi` first), so the sandbox does not rotate the token your
computer holds; if a CLI refreshes anyway, the report says so and the sandbox's copies are written to
`spike-refreshed-logins.json` (git-ignored; delete it after).

`npm run dev` runs it under `wrangler dev` with a `.dev.vars` of `GITHUB_TOKEN=local` and a public `REPO`: the git
steps then run without a token, which checks the sandbox, certificate and handler plumbing but not GitHub.

## Open questions and answers

| Question | Answer | Source |
|---|---|---|
| Q1a: do Claude Code and Codex subscription sign-ins work through the outbound handler? | Yes. Both wrote and committed a file (Claude 9 s, Codex 18 s) and both answered with only the strict host list allowed. Claude needs `api.anthropic.com`; it also calls `mcp-proxy.anthropic.com` (the account's claude.ai connectors), which the strict list refused 18 times without harm. Codex needs `chatgpt.com` (HTTPS and its WebSocket, status 101); `ab.chatgpt.com` and `sdmntpr*.oaiusercontent.com` were refused without harm. Neither CLI refreshed its sign-in; only `.claude.json` (settings, not the token) changed. | Spike run, 5 Nov 2026, `standard-1`, Claude Code 2.1.318, Codex 0.166.0 |
| Q1b: can the handler read a push's refs and allow only the job's branch? | Yes. A push to `feedback/<id>` went through; a push to another branch and a delete of the job's branch were refused with 403 by the handler; a push changing `.github/workflows` was refused by GitHub ("without `workflow` scope"). The token never appeared inside the sandbox. | Spike run, 5 Nov 2026 |
| Q2: can the agent run the Android unit tests inside the sandbox? | Yes on `standard-3` (2 vCPU, 8 GiB, 16 GB) or larger, not on `standard-1`. With Gradle's dependency caches already in the image (no network, no build cache), a fresh clone builds and runs all 649 unit tests in 232 s on `standard-3` and 183 s on `standard-4`. On `standard-1` (4 GiB) the test JVM was killed for memory after 11 minutes and 160 tests, because Gradle (`-Xmx3g`) and the test JVM (2 GB) don't fit together. The image would grow by about 2.8 GB (Gradle 1.7 GB, Maven 0.4 GB, Android SDK 0.6 GB) plus a JDK. | Docker on a 4-core cloud machine with `--cpus`/`--memory` set to each instance type, 5 Nov 2026 |
| Q3: container cost per job | At most $0.0037 per minute on `standard-3` and $0.0012 on `standard-1`: memory and disk are billed as provisioned, CPU only when busy, so these are ceilings. A round capped at 45 minutes costs at most $0.17 on `standard-3`, and a job of 3 rounds at most $0.50. The whole spike run (start, clone, two short agent tasks, pushes, Q4) took 75 s; the sandbox started in 13 s and the shallow clone took 4 s (52 MB). | Cloudflare Containers pricing, 5 Nov 2026: $0.000020 per vCPU-second, $0.0000025 per GiB-second, $0.00000007 per GB-second |
| Q4: does update-branch work for a sign-in without the `workflow` scope when `main` has workflow changes the branch lacks? | Yes. From a branch made just before `main`'s last workflow commit, update-branch answered 202 and moved the branch, and the merges API answered 204, both with the `repo`-only sign-in. | Spike run, 5 Nov 2026 (draft PR #76, closed) |
