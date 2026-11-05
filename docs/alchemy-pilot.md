# Alchemy staging pilot

This is an isolated infrastructure pilot around the existing Rust/Wasm Worker. It does not replace the production Wrangler deployment. Offline verification does not establish a successful deployment or remote smoke; cloud operations use the serialized CI workflow and its dedicated staging credentials.

The package in `infra/alchemy` pins **Alchemy Effect v2 `2.0.0-beta.83`**, with matching Effect and Node platform peers at `4.0.5`, not Alchemy v1 or the newer main-branch `@alchemy.run/cloudflare` imports. Its fixed stage is `staging`; resource names are `mymind-alchemy-staging-worker`, `mymind-alchemy-staging-probe`, `mymind-alchemy-staging-media`, and R2 bucket `mymind-alchemy-staging-fixtures`. The stack is `mymind-alchemy-pilot`. Do not rename resources to recover missing state.

`Auth`, `Library`, and `Media` remain Rust exports. Bindings remain `AUTH_OBJECT`, `LIBRARY`, `MEDIA`, and `BUCKET`. Production's `AgentHome` (binding `AGENT_HOME` and its container, where each account's Claude Code and Codex run) is production-only: the pilot neither deploys nor probes it, so its AI routes answer an error there (`PRODUCTION_ONLY` in `scripts/contract.mjs`). The Worker build copies the AI kit from the `vendor/ai-kit` submodule beside the Rust sources. Alchemy uploads the prebuilt ESM and Wasm with `bundle: false`; its fleet container binding declares the `Media` namespace and attaches the unchanged `rust/container/Dockerfile`, repository-root build context, `standard-1` instance, and maximum of one container. This matches Rust's existing `container.start()` contract. A TypeScript pass checks the infrastructure program, not Rust runtime bindings; runtime tests are still necessary.

Both Workers have `workersDev: false`, no preview URLs, no routes, no custom domain, and no Access configuration. R2 public access is disabled. `PUBLIC_URL` uses the reserved `.invalid` suffix only as the private request origin, never as a published hostname. `AUTH=google` remains enabled with no allowed Google domain; testing uses a synthetic password account, never the owner's Google login. The app requires Google client configuration even for password sign-in, so deployment requires real dedicated staging client secrets. Offline-only Wrangler fixtures use inert values and are never accepted as deployment credential evidence.

## Local verification

Requires Node **24.11.1+** (CI uses 24.19.0), Python 3 and `unzip` for the state fixtures, Rust 1.99.0, `wasm32-unknown-unknown`, and `worker-build` **0.8.7**. Run from `infra/alchemy`:

```sh
npm ci --ignore-scripts
npm test
npm run typecheck
node scripts/check.mjs --contract-only
rustup toolchain install 1.99.0 --profile minimal --target wasm32-unknown-unknown
cargo +1.99.0 install worker-build --version 0.8.7 --locked --root "$PWD/.artifacts/toolchain"
export PATH="$PWD/.artifacts/toolchain/bin:$PATH"
export RUSTUP_TOOLCHAIN=1.99.0
npm run build:worker
npm run check
```

The build copies only Rust source/workspace inputs into `.artifacts/rust` and builds there. Cargo output stays in `.artifacts/cargo-target`; it does not edit production build output. The container build still uses the original repository source. Docker, sufficient disk/memory, base-image pulls, Cargo downloads, and a working daemon are prerequisites. In a managed environment use the documented session proxy/CA support for Docker; do not disable certificate verification. The current Dockerfile is unchanged, so a runner whose HTTPS proxy requires injected CA trust may need a supported build setup before this container can build.

Existing runtime evidence lives in `tools/check-worker.mjs`, `rust/crates/server/tests/worker.rs`, `worker_credentials.rs`, and `rust/crates/worker/tests/golden/worker.json`. The golden check exercises the real Rust auth/Library/R2 chain with external networking blocked. From the repository root, after preparing its pinned Wrangler, cached Rust dependencies, and synthetic `rust/test-fixtures/hosted-test.db`, run `node tools/check-worker.mjs`. Set `MYMIND_WRANGLER` to the absolute pilot Wrangler executable if reusing this package. This existing harness uses its own build/config and `rust/target/worker-check`; it does not verify Alchemy's deployment.

For the pilot's actual artifact with the real local media container:

```sh
npm run local:config
CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV=false CLOUDFLARE_INCLUDE_PROCESS_ENV=false \
  ./node_modules/.bin/wrangler dev --local --config .artifacts/wrangler.local.json \
  --env-file .artifacts/empty-vars --ip 127.0.0.1 --port 8798 \
  --persist-to .artifacts/local-state
```

In another terminal from the repository root, run `MYMIND_WORKER_CONTAINER_URL=http://127.0.0.1:8798 node tools/check-worker.mjs --container-smoke`. This uses synthetic fixtures and the real Rust password/session/upload/media path; local Docker networking must work. On hosts needing the existing sidecar workaround, inspect `deploy/worker-dev-sidecars.sh` before using it; it requires privileged namespace access. Never expose the local listener through a tunnel.

## State and manual CI

`.github/workflows/alchemy-pilot.yml` verifies relevant PRs without cloud credentials. Cloud operations require `workflow_dispatch` on main, with default operation `verify`; no push or PR deploys infrastructure. The `cloud` job uses GitHub environment `alchemy-staging`, one fixed concurrency group, and the latest **encrypted** private Actions artifact `mymind-alchemy-staging-state-v1` as the shared CI state. Its decrypted contents are Alchemy's `.alchemy/state` directory. Alchemy local state can contain raw secrets, so the artifact uses AES-256-GCM and a dedicated state key; only `state.enc` is uploaded, including after failed operations. No Cloudflare state backend is bootstrapped: beta.83's `Cloudflare.state()` can publish its own workers.dev hostname.

Configure the private environment with `ALCHEMY_CLOUDFLARE_ACCOUNT_ID`, `ALCHEMY_CLOUDFLARE_API_TOKEN`, `MYMIND_ALCHEMY_STATE_KEY` (at least 32 characters), `MYMIND_ALCHEMY_GOOGLE_CLIENT_ID`, `MYMIND_ALCHEMY_GOOGLE_CLIENT_SECRET`, `MYMIND_ALCHEMY_ADMIN_TOKEN`, and `MYMIND_ALCHEMY_PASSWORD` (at least 16 characters). Provision these through the normal secret-management process; no secrets are generated by this pilot. Retain the state key as long as the infrastructure exists.

The workflow token needs `contents: read`, `actions: read`, and `deployments: write`. Cloud operations run only from `main`; restrict the `alchemy-staging` environment to that branch too. A durable GitHub deployment record marks every operation before cloud changes. Recovery requires a checkpoint from that same workflow run, created after its marker, so cancellation or failed artifact upload blocks reuse of older state. Recovery fetches every artifact page and validates stable totals, complete pages and unique IDs before choosing the newest matching `created_at`; artifact IDs do not establish chronology. Invalid matching timestamps or distinct newest checkpoints with the same timestamp block restoration. Deleting all artifacts cannot silently permit a fresh bootstrap. Initial deployment requires operation `deploy` with `initialize: true`; every subsequent operation uses `initialize: false`. Missing/expired/unreadable state, a different workflow's artifact, or a wrong encryption key fails closed. The newest failed run's state is authoritative too. Do not substitute an older successful artifact. Recover uncheckpointed changes from the original runner state before another operation. The artifact expires after 90 days; export a private encrypted backup and refresh the checkpoint before expiry while the pilot exists. Do not delete deployment markers to bypass recovery.

Cloudflare needs an account-scoped API token with Workers Scripts edit, Workers R2 Storage edit, Containers edit, and Workers Containers Images write (plus the corresponding read permissions/account membership needed by the pinned provider). R2 must be enabled and Containers available on the account/plan. No DNS, zone route, Access, or production-resource permissions are needed. Cloudflare's token labels/availability can vary; a denied API must be reported, never treated as success. Network prerequisites include `api.cloudflare.com`, `registry.cloudflare.com`, the package/Cargo registries, Docker Hub base-image hosts, and Debian package mirrors used by the unchanged image. A saved network-policy draft does not enable these hosts.

Local cloud commands use the same program and explicit state restore. State restoration and mutation belong in the serialized workflow, which supplies `GH_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_SHA`, `GITHUB_RUN_ID`, and the state key. An operator recovering a failed run must preserve that run's identity and checkpoint; do not invent a new run ID. Credentials are process-environment inputs; the CLI is passed `/dev/null` as its env file. The supported commands reject arbitrary Alchemy flags/stages:

```sh
npm run plan        # real authenticated Cloudflare read/plan, not an offline dry run
npm run deploy      # deploy with no probe schedule
npm run probe       # arm a private service-bound scheduled smoke
npm run probe:result
npm run probe:stop  # remove the schedule, including after a failed probe
npm run destroy    # teardown this pilot using its original shared state
```

Local mutation must not race CI. After any local cloud operation, seal/checkpoint the resulting state back through the canonical private CI recovery procedure before allowing the next run. Prefer the serialized workflow for mutation so it captures the final state automatically. The fixed resource names and original state permit repeat deployment without creating a second set of namespaces. Never pass `--adopt`, `--force`, transfer declarations, or production names.

## Remote smoke and teardown

Dispatch `deploy`, then `probe`. The probe Worker has a service binding to the Rust Worker and no reachable hostname. An opt-in five-minute cron calls the real authenticated app; no unauthenticated ingress or auth bypass is introduced. The probe stops after one success or two failures per run. It verifies signed-out/admin denial, seeds only `alchemy-staging-synthetic`, creates a genuine password session, uploads a one-pixel PNG through the real upload capability, creates a Media object through the real container, reads the Library object, trashes that synthetic object, signs out, and checks session revocation. Status-only results go to `pilot/results/<run-id>.json` in staging R2 and are read with the authenticated Cloudflare API. The workflow always attempts to remove the cron afterward. If disarming fails, dispatch `stop` and confirm schedule removal before leaving the pilot running.

The result reader allows 26 minutes: up to 15 minutes for [Cloudflare cron propagation](https://developers.cloudflare.com/workers/configuration/cron-triggers/), two five-minute attempts, and one minute to finish and read the result. If the deadline expires, it reports counts of missing objects and timed-out R2 reads.

Repeat `deploy` and `probe`. Require both remote results to pass, `completedRuns` to increase, and Cloudflare Worker settings to retain the same `Auth`, `Library`, and `Media` namespace IDs. Deployment output includes the Worker, namespace, and container application IDs; retain these identities from each run for comparison. Record the deployed commit, real resource/namespace/application IDs, both result timestamps, and container readiness evidence in the private run report. An offline test, plan, artifact build, HTTP 401 alone, or unexecuted procedure is not a successful cloud pilot.

R2 reads return an object body, not parsed JSON: the probe calls `get(key)` and then the returned body's `json()` method. The strict synthetic fixture checks this contract; a pinned local workerd check rejected the old KV-style `get(key, 'json')` call. That local proof does not establish a successful remote probe.

After successful disarm, allow at least 15 minutes for cron removal to propagate before a new operation changes `PILOT_RUN`. This addresses the documented propagation overlap, not cancellation of an already-running event. The existing three-minute media inactivity timeout is separate; waiting alone proves neither drained cron work nor a stopped container.

## Live identity audit and image cleanup

Use the same `Alchemy staging pilot` workflow on `main`, with `initialize: false`. Its existing concurrency group, credential route, state restore, operation marker, encryption and final checkpoint apply to both new operations. They do not deploy a Worker or change schedules. Local audit/cleanup CLI execution is rejected outside the main CI workflow; no local Cloudflare token or account connector is needed.

Save the safe deployment output as JSON **before destroying** the pilot. The `expected_identities` input requires exactly `workerName`, `workerId`, `probeName`, `probeId`, `bucketName`, `durableObjectNamespaces` (exactly `Auth`, `Library`, `Media`), and `containerApplicationId`. Use the actual distinct live IDs and fixed pilot names; no environment values, credentials or additional fields are accepted.

1. Dispatch `audit` with `expected_state: present` and that JSON after each deployment. It compares live immutable Worker IDs, settings bindings, schedules, namespace ownership, the container application and the bucket against the saved identities. The `TARGET` service binding must select the fixed Rust Worker's default environment and default entrypoint. Compare two passed smoke results and two matching live audits before calling repeat deployment successful.
2. Preserve smoke/status evidence separately. Dispatch `destroy` after disarm and the propagation interval. R2 contains synthetic fixtures only; `forceDestroy: true` permits deletion of all contents. Synthetic accounts and SQLite rows remain in the pilot namespaces until teardown.
3. Dispatch `audit` with `expected_state: absent` and the same saved JSON. It requires definitive not-found reads plus complete account listings. A namespace or application still associated with a fixed pilot name blocks absence even if an incorrect supplied ID returns not-found. A permission error, timeout, malformed response, partial deletion or incomplete pagination fails closed. A post-destroy `plan` proposing creates does not substitute for this readback.
4. Dispatch `cleanup-images` with `expected_state: absent` and that same JSON. It proves resource absence, inventories only `mymind-alchemy-staging-media`, repeats the absence proof immediately before deletion, deletes only the exact inventoried tags (including `buildcache`), and inventories again. A live application using that repository blocks cleanup even under another application name; inconclusive application names or image usage also fail closed. A foreign repository, invalid tag or nonempty final inventory fails the operation. Partial cleanup can be retried through the same workflow with its newest checkpoint.

Audit output contains only fixed names, IDs, binding/class names and schedule evidence; complete settings, application configuration and remote error bodies are never printed. Reads have 15-second request timeouts, bounded response bodies and pagination, and a five-minute operation deadline. Pinned Wrangler calls have at most 60 seconds within that deadline.

Container application listings follow the [Cloudflare cursor contract](https://github.com/cloudflare/api-schemas/blob/d3e86eec17211653ca9bc18c0d4bfcace2904148/openapi.yaml#L119961): request `per_page=100`, then send each opaque `result_info.next_page_token` as `page_token` using URL encoding. Every page requires a result array and a metadata object containing only the documented `next_page_token`, `page_token`, and `per_page` fields. An optional page size must equal 100 and an optional echoed token must match the request. Only an absent next token marks completion; `{}` is valid terminal metadata, including after a full page. Empty intermediate pages do not end traversal. Null, empty or malformed tokens, repeated tokens or application IDs, conflicting next-page links, unknown metadata, and missing metadata fail closed. Traversal is bounded to 100 pages, 10,000 rows and 4,096 characters per token; tokens and raw links are never printed. Both absence proofs finish all application pages before permitting image deletion, so a late-page pilot application or image consumer blocks cleanup. Namespace listings retain their numeric pagination contract. Synthetic tests verify these parser rules; they do not establish a passed live audit.

Cloudflare's [service-binding schema](https://github.com/cloudflare/api-schemas/blob/d3e86eec17211653ca9bc18c0d4bfcace2904148/openapi.yaml#L513627) declares `production` as the default environment label, including for the fixed staging Worker. The audit accepts that explicit label only after reading `workers/services/{fixedWorkerName}` and matching the service name, default-environment name, script name and saved immutable Worker tag. [Wrangler uses that metadata tag as the deployed Worker identity](https://github.com/cloudflare/workers-sdk/blob/b49580ebf4cc79d7c7e08ea6e2fb487f39f6e344/packages/deploy-helpers/src/deploy/helpers/match-tag.ts#L52). An omitted/null selector uses the implicit default. Empty or named entrypoints remain rejected; omitted/null and `default` entrypoints are accepted. Rejection diagnostics contain only finite selector classes and the default-target proof state, never arbitrary selector values. Successful evidence records the target Worker ID and whether its default environment was implicit or verified through metadata. These rules support a documented representation; the selectors responsible for audit 39756757478's rejection were not observed.

An API response rejected by the audit reports a fixed endpoint label, finite envelope-predicate states, the JSON kind of its errors field, and at most five numeric error codes. The kind distinguishes null from malformed scalar or object values without including their contents. HTTP 200 can contain a logical API failure; the diagnostic never substitutes for successful identity or absence checks and never includes remote messages, headers, or response bodies.

The live namespace-list endpoint was observed returning `errors: null` with HTTP 200 and `success: true`. Only that endpoint treats explicit null as no errors. Successful HTTP status, `success: true`, a valid resource array, complete pagination, saved identities and namespace ownership remain mandatory; other endpoints retain their existing error-envelope checks. This is compatibility with an observed response, rather than a nullable-error guarantee from Cloudflare's schema.

Alchemy beta.83's container delete hook deletes the application, not the registry manifests. Builds publish content-hash tags and `buildcache` in the explicitly named repository. Cleanup uses installed Wrangler **4.151.1** `containers images list --filter '(^|/)mymind-alchemy-staging-media$' --json` and `containers images delete mymind-alchemy-staging-media:TAG --skip-confirmation`; it never deletes a registry or accepts arbitrary commands/repositories. Wrangler requests layer garbage collection after manifest deletion, but GC failure can be only a warning. The result therefore reports `layerReclamationVerified: false`; an empty tag inventory does not prove physical storage reclamation. See [Cloudflare image management](https://developers.cloudflare.com/containers/guides/image-management/).

Preserve the latest encrypted infrastructure checkpoint and its key until live absence is verified. R2's one-day object lifecycle is asynchronous and does not replace teardown. The current workflow exposes no public hostname.

Any future public hostname requires a separate, concrete hostname approval under `AGENTS.md` rule 7. This pilot needs none.

## Pinned sources

All API decisions were checked against release commit `db20b54e8c55e8cede9ef66ad2dd25b7e74d35da` (`v2.0.0-beta.83`):

- [Worker props: prebuilt ESM/Wasm, workersDev, and compatibility](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/Cloudflare/Workers/Worker.ts)
- [Async Worker native namespace and container binding behavior](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/Cloudflare/Workers/WorkerAsyncBindings.ts)
- [Container properties, build context, fleet limits, and registry](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/Cloudflare/Containers/ContainerApplication.ts)
- [R2 lifecycle, public access, and forceDestroy](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/Cloudflare/R2/Bucket.ts)
- [Local state persistence](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/State/LocalState.ts) and [secret encoding](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/State/StateEncoding.ts)
- [Cloudflare state bootstrap behavior](https://github.com/alchemy-run/alchemy/blob/db20b54e8c55e8cede9ef66ad2dd25b7e74d35da/packages/alchemy/src/Cloudflare/StateStore/State.ts)

Generated private/local paths to ignore: `infra/alchemy/node_modules/`, `.alchemy/`, `.artifacts/`, `.wrangler/`, and any local environment files. Commit the actual npm-generated `infra/alchemy/package-lock.json`; do not invent a lockfile or commit state/build output.
