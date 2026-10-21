# Alchemy staging pilot

This is an isolated infrastructure pilot around the existing Rust/Wasm Worker. It does not replace the production Wrangler deployment. No successful cloud deployment or remote smoke is implied by these files: this environment has neither Cloudflare credentials nor Cloudflare API network access.

The package in `infra/alchemy` pins **Alchemy Effect v2 `2.0.0-beta.83`**, with matching Effect and Node platform peers at `4.0.5`, not Alchemy v1 or the newer main-branch `@alchemy.run/cloudflare` imports. Its fixed stage is `staging`; resource names are `mymind-alchemy-staging-worker`, `mymind-alchemy-staging-probe`, `mymind-alchemy-staging-media`, and R2 bucket `mymind-alchemy-staging-fixtures`. The stack is `mymind-alchemy-pilot`. Do not rename resources to recover missing state.

`Auth`, `Library`, and `Media` remain Rust exports. Bindings remain `AUTH_OBJECT`, `LIBRARY`, `MEDIA`, and `BUCKET`. Alchemy uploads the prebuilt ESM and Wasm with `bundle: false`; its fleet container binding declares the `Media` namespace and attaches the unchanged `rust/container/Dockerfile`, repository-root build context, `standard-1` instance, and maximum of one container. This matches Rust's existing `container.start()` contract. A TypeScript pass checks the infrastructure program, not Rust runtime bindings; runtime tests are still necessary.

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

The workflow token needs `contents: read`, `actions: read`, and `deployments: write`. Cloud operations run only from `main`; restrict the `alchemy-staging` environment to that branch too. A durable GitHub deployment record marks every operation before cloud changes. Recovery requires a checkpoint from that same workflow run, created after its marker, so cancellation or failed artifact upload blocks reuse of older state. Deleting all artifacts cannot silently permit a fresh bootstrap. Initial deployment requires operation `deploy` with `initialize: true`; every subsequent operation uses `initialize: false`. Missing/expired/unreadable state, a different workflow's artifact, or a wrong encryption key fails closed. The newest failed run's state is authoritative too. Do not substitute an older successful artifact. Recover uncheckpointed changes from the original runner state before another operation. The artifact expires after 90 days; export a private encrypted backup and refresh the checkpoint before expiry while the pilot exists. Do not delete deployment markers to bypass recovery.

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

Repeat `deploy` and `probe`. Require both remote results to pass, `completedRuns` to increase, and Cloudflare Worker settings to retain the same `Auth`, `Library`, and `Media` namespace IDs. Deployment output includes the Worker, namespace, and container application IDs; retain these identities from each run for comparison. Record the deployed commit, real resource/namespace/application IDs, both result timestamps, and container readiness evidence in the private run report. An offline test, plan, artifact build, HTTP 401 alone, or unexecuted procedure is not a successful cloud pilot.

R2 contains synthetic fixtures only; all objects expire after one day (lifecycle deletion is asynchronous). The synthetic account and SQLite rows remain in the pilot namespaces until teardown. The media container keeps its existing three-minute inactivity timeout. `destroy` deletes the pilot Workers, namespaces, container application, and staging R2; `forceDestroy: true` intentionally permits deleting all synthetic bucket contents. Alchemy may retain container registry content/cache; inspect the dedicated `mymind-alchemy-staging-media` repository after teardown and remove its pilot-only images through the registry's normal authenticated tooling if retained. Preserve the encrypted infrastructure state until cloud deletion is verified; retain status evidence separately if needed.

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
