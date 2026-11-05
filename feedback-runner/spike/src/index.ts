import { Files } from "@cloudflare/sandbox";
import { DurableObject, WorkerEntrypoint } from "cloudflare:workers";

import { type GitHubEnv, github, whoAmI } from "./github";
import { readPush, refusePush } from "./push";

// Throwaway spike for the feedback runner (docs/feedback.md). It answers the design's open questions on a real
// Cloudflare sandbox: Q1 (does a Claude Code or Codex subscription sign-in work through the outbound handler, and can
// the handler read a push's refs), Q3 (time per round) and Q4 (update-branch without the workflows permission).

interface Env extends GitHubEnv {
  SANDBOX: DurableObjectNamespace<SpikeSandbox>;
  SPIKE_TOKEN: string;
}

type Engine = "claude" | "codex";
type Phase = "setup" | "git" | "discover" | "strict";

/** What the outbound handler needs to decide a request; kept by the sandbox's Durable Object. */
interface Policy {
  phase: Phase;
  engine: Engine | null;
  ownRef: string;
}

interface HostRow {
  phase: Phase;
  engine: Engine | null;
  host: string;
  count: number;
  methods: string[];
  statuses: number[];
  decision: string;
}

/** The body `run.mjs` posts to `/run`. Login files go into the sandbox and are never stored. */
interface RunInput {
  files: Record<string, unknown>;
  engines: Engine[];
  models: Partial<Record<Engine, string>>;
  effort?: string;
  q4: boolean;
}

const CA = "/etc/cloudflare/certs/cloudflare-containers-ca.crt";
const BUNDLE = "/etc/ssl/certs/ca-certificates.crt";
const HOME = "/home/agent";
const REPO_DIR = "/workspace/repo";
const AGENT_USER = "agent";
const AGENT_TIMEOUT_MS = 10 * 60_000;

// Provisional host lists from the design (Components, Outbound handler); the strict phase checks they are enough.
const ENGINE_HOSTS: Record<Engine, string[]> = {
  claude: ["api.anthropic.com", "claude.ai", "console.anthropic.com", "platform.claude.com"],
  codex: ["chatgpt.com", "auth.openai.com", "api.openai.com"],
};
const PACKAGE_HOSTS = ["index.crates.io", "static.crates.io", "registry.npmjs.org"];

const CLI_ENV = {
  HOME,
  SSL_CERT_FILE: BUNDLE,
  NODE_EXTRA_CA_CERTS: CA,
  GIT_SSL_CAINFO: BUNDLE,
  DISABLE_AUTOUPDATER: "1",
  CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: "1",
  IS_SANDBOX: "1",
};

/** Every HTTP and HTTPS request leaving the sandbox comes here. It logs host names only, never paths or bodies. */
export class Outbound extends WorkerEntrypoint<Env, { name: string }> {
  override async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const sandbox = this.env.SANDBOX.getByName(this.ctx.props.name);
    const policy = await sandbox.policy();
    const { response, decision } = await this.#decide(request, url, policy);
    await sandbox.record(url.hostname, request.method, response.status, decision);
    return response;
  }

  async #decide(request: Request, url: URL, policy: Policy): Promise<{ response: Response; decision: string }> {
    if (url.protocol !== "https:") return refuse("plain HTTP is not allowed");
    if (url.hostname === "github.com") return this.#git(request, url, policy);
    if (policy.phase === "discover") return { response: await fetch(request), decision: "passed (discover)" };
    if (policy.engine && ENGINE_HOSTS[policy.engine].includes(url.hostname)) {
      return { response: await fetch(request), decision: "passed (engine host)" };
    }
    if (PACKAGE_HOSTS.includes(url.hostname) && request.method === "GET") {
      return { response: await fetch(request), decision: "passed (package host)" };
    }
    return refuse(`${url.hostname} is not reachable from this sandbox`);
  }

  /** Git smart HTTP for this one repository: adds the stored GitHub sign-in, and lets pushes through to the own branch only. */
  async #git(request: Request, url: URL, policy: Policy): Promise<{ response: Response; decision: string }> {
    const prefix = `/${this.env.REPO}`;
    const rest = url.pathname.startsWith(`${prefix}.git/`)
      ? url.pathname.slice(prefix.length + 4)
      : url.pathname.startsWith(`${prefix}/`)
        ? url.pathname.slice(prefix.length)
        : null;
    const service = url.searchParams.get("service");
    const known =
      (rest === "/info/refs" && request.method === "GET" && (service === "git-upload-pack" || service === "git-receive-pack")) ||
      (rest === "/git-upload-pack" && request.method === "POST") ||
      (rest === "/git-receive-pack" && request.method === "POST");
    if (!known) return refuse("only git fetch and push to this repository are allowed");

    const headers = new Headers(request.headers);
    // GITHUB_TOKEN "local" is for `wrangler dev` against a public repository, without a sign-in.
    if (this.env.GITHUB_TOKEN !== "local") {
      headers.set("authorization", `Basic ${btoa(`x-access-token:${this.env.GITHUB_TOKEN}`)}`);
    }
    if (rest !== "/git-receive-pack") {
      return { response: await fetch(new Request(request, { headers, redirect: "manual" })), decision: "git (token added)" };
    }
    const body = new Uint8Array(await request.arrayBuffer());
    const plain = request.headers.get("content-encoding") === "gzip" ? await gunzip(body) : body;
    const reason = refusePush(readPush(plain), policy.ownRef);
    if (reason) return refuse(`push refused: ${reason}`);
    const forwarded = new Request(request.url, { method: "POST", headers, body, redirect: "manual" });
    return { response: await fetch(forwarded), decision: "push allowed (own branch)" };
  }
}

/** One sandbox per spike run. `fetch` streams one JSON line per step so the caller sees progress. */
export class SpikeSandbox extends DurableObject<Env> {
  #policy: Policy = { phase: "setup", engine: null, ownRef: "" };
  #hosts = new Map<string, HostRow>();

  policy(): Policy {
    return this.#policy;
  }

  record(host: string, method: string, status: number, decision: string): void {
    const { phase, engine } = this.#policy;
    const key = `${phase}|${engine}|${host}|${decision}`;
    const row = this.#hosts.get(key) ?? { phase, engine, host, count: 0, methods: [], statuses: [], decision };
    row.count += 1;
    if (!row.methods.includes(method)) row.methods.push(method);
    if (!row.statuses.includes(status)) row.statuses.push(status);
    this.#hosts.set(key, row);
  }

  override async fetch(request: Request): Promise<Response> {
    const input = (await request.json()) as RunInput & { name: string };
    const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();
    const writer = writable.getWriter();
    const encoder = new TextEncoder();
    const emit = (line: unknown) => writer.write(encoder.encode(`${JSON.stringify(line)}\n`));
    const beat = setInterval(() => void emit({ heartbeat: new Date().toISOString() }), 20_000);
    this.ctx.waitUntil(
      this.#run(input, emit)
        .catch((cause) => emit({ step: "spike", ok: false, error: redact(String(cause?.stack ?? cause)) }))
        .finally(async () => {
          clearInterval(beat);
          await writer.close();
        }),
    );
    return new Response(readable, { headers: { "content-type": "application/x-ndjson" } });
  }

  async #run(input: RunInput & { name: string }, emit: (line: unknown) => Promise<void>): Promise<void> {
    const container = this.ctx.container;
    if (!container) throw new Error("no container binding");
    const files = new Files(container);
    const id = input.name;
    const ownBranch = `feedback/${id}`;
    this.#policy = { phase: "setup", engine: null, ownRef: `refs/heads/${ownBranch}` };
    const started = Date.now();
    const step = async (name: string, work: () => Promise<Record<string, unknown>>) => {
      const at = Date.now();
      try {
        const detail = await work();
        await emit({ step: name, ms: Date.now() - at, ...detail });
        return detail;
      } catch (cause) {
        await emit({ step: name, ms: Date.now() - at, ok: false, error: redact(String((cause as Error)?.message ?? cause)) });
        return { ok: false };
      }
    };
    const exec = async (argv: string[], options: { user?: string; cwd?: string; timeoutMs?: number; env?: Record<string, string> } = {}) => {
      const abort = new AbortController();
      const timer = setTimeout(() => abort.abort(), options.timeoutMs ?? 120_000);
      const at = Date.now();
      try {
        const process = await container.exec(argv, {
          cwd: options.cwd ?? "/workspace",
          user: options.user,
          env: { ...CLI_ENV, ...options.env },
          signal: abort.signal,
        });
        const out = await process.output();
        const decode = (buffer: ArrayBuffer) => new TextDecoder().decode(buffer);
        return { exitCode: out.exitCode, stdout: decode(out.stdout), stderr: decode(out.stderr), ms: Date.now() - at };
      } finally {
        clearTimeout(timer);
      }
    };
    const asAgent = (script: string, timeoutMs?: number) =>
      exec(["/bin/sh", "-c", script], { user: AGENT_USER, cwd: REPO_DIR, timeoutMs });

    await step("start", async () => {
      container.start({ image: container.images.sandbox, instance: "standard-1", enableInternet: false });
      const outbound = (this.ctx as any).exports.Outbound({ props: { name: id } });
      await container.interceptAllOutboundHttp(outbound);
      await container.interceptOutboundHttps("*", outbound);
      await container.setInactivityTimeout(30 * 60_000);
      return { ok: true };
    });

    await step("trust", async () => {
      // The handler re-signs HTTPS with a per-start CA; add it to the system bundle that git and Codex read.
      const script = `for i in $(seq 1 30); do [ -s ${CA} ] && break; sleep 1; done
cp ${CA} /usr/local/share/ca-certificates/cloudflare-containers.crt && update-ca-certificates >/dev/null
install -d -o ${AGENT_USER} -g ${AGENT_USER} /workspace ${HOME}`;
      const result = await exec(["/bin/sh", "-c", script], { user: "root" });
      return { ok: result.exitCode === 0, stderr: tail(result.stderr) };
    });

    const loginHashes = async () => {
      const hashes: Record<string, string> = {};
      for (const path of Object.keys(input.files)) {
        const text = await files.readFile(`${HOME}/${path}`).then((file) => file.text()).catch(() => "");
        hashes[path] = await sha256(text);
      }
      return hashes;
    };

    await step("login-files", async () => {
      for (const [path, content] of Object.entries(input.files)) {
        await files.mkdir(`${HOME}/${path}`.replace(/\/[^/]+$/, ""), { recursive: true });
        await files.writeFile(`${HOME}/${path}`, JSON.stringify(content, null, 2));
      }
      const result = await exec(["/bin/sh", "-c", `chown -R ${AGENT_USER}:${AGENT_USER} ${HOME} && chmod -R go-rwx ${HOME}`], { user: "root" });
      return { ok: result.exitCode === 0, written: Object.keys(input.files) };
    });
    const before = await loginHashes();

    this.#policy.phase = "git";
    await step("clone", async () => {
      const result = await exec(
        ["/bin/sh", "-c", `git clone --depth 1 https://github.com/${this.env.REPO} ${REPO_DIR} && cd ${REPO_DIR} && git switch -c ${ownBranch} && git config user.name 'Feedback spike' && git config user.email spike@thymind.invalid && du -sh .git . | tr '\\n' ' '`],
        { user: AGENT_USER, timeoutMs: 5 * 60_000 },
      );
      return { ok: result.exitCode === 0, size: result.stdout.trim(), stderr: tail(result.stderr) };
    });

    for (const engine of input.engines) {
      if (!input.files[engine === "claude" ? ".claude/.credentials.json" : ".codex/auth.json"]) {
        await emit({ step: `${engine}`, ok: false, error: "no sign-in file sent for this engine" });
        continue;
      }
      this.#policy = { ...this.#policy, phase: "discover", engine };
      const task = `Create the file spike/${engine}.txt containing exactly one line: written by ${engine} in the feedback spike. Then commit it with git using the message "spike: ${engine}". Do nothing else.`;
      await step(`${engine}-task (all hosts allowed and logged)`, async () => {
        const result = await exec(agentArgv(engine, task, input.models[engine], input.effort), { user: AGENT_USER, cwd: REPO_DIR, timeoutMs: AGENT_TIMEOUT_MS });
        const committed = await asAgent(`git log -1 --format=%s`);
        return {
          ok: result.exitCode === 0 && committed.stdout.trim() === `spike: ${engine}`,
          exitCode: result.exitCode,
          lastCommit: committed.stdout.trim(),
          outcome: outcome(engine, result.stdout),
          stderr: tail(result.stderr),
        };
      });
      this.#policy = { ...this.#policy, phase: "strict", engine };
      await step(`${engine}-reply (only the design's host list)`, async () => {
        const result = await exec(agentArgv(engine, "Reply with the single word OK and nothing else.", input.models[engine], input.effort), { user: AGENT_USER, cwd: REPO_DIR, timeoutMs: AGENT_TIMEOUT_MS });
        const said = outcome(engine, result.stdout);
        return { ok: result.exitCode === 0 && /\bOK\b/.test(said), exitCode: result.exitCode, outcome: said, stderr: tail(result.stderr) };
      });
    }

    const after = await loginHashes();
    const changed = Object.keys(before).filter((path) => before[path] !== after[path]);
    const refreshed: Record<string, unknown> = {};
    for (const path of changed) {
      refreshed[path] = JSON.parse(await (await files.readFile(`${HOME}/${path}`)).text());
    }
    await emit({ step: "login-files-after", changed, ok: true });

    this.#policy = { ...this.#policy, phase: "git", engine: null };
    const token = this.env.GITHUB_TOKEN === "local" ? "no-token-in-local-runs" : this.env.GITHUB_TOKEN;
    await step("push-own-branch (expect allowed)", async () => {
      const result = await asAgent(`git push origin HEAD:refs/heads/${ownBranch} 2>&1`);
      return { ok: result.exitCode === 0, output: tail(result.stdout) };
    });
    await step("push-other-branch (expect refused by the handler)", async () => {
      const result = await asAgent(`git push origin HEAD:refs/heads/${ownBranch}-other 2>&1`);
      return { ok: result.exitCode !== 0, output: tail(result.stdout) };
    });
    await step("push-delete-own (expect refused by the handler)", async () => {
      const result = await asAgent(`git push origin :refs/heads/${ownBranch} 2>&1`);
      return { ok: result.exitCode !== 0, output: tail(result.stdout) };
    });
    await step("push-workflow-change (expect refused by GitHub: the sign-in has no workflow scope)", async () => {
      const result = await asAgent(
        `mkdir -p .github/workflows && printf 'name: spike\\non: workflow_dispatch\\njobs: {}\\n' > .github/workflows/spike-${id}.yml && git add .github/workflows && git commit -qm 'spike: workflow' && git push origin HEAD:refs/heads/${ownBranch} 2>&1; code=$?; git reset -q --hard HEAD~1; exit $code`,
      );
      return { ok: result.exitCode !== 0, output: tail(result.stdout) };
    });
    await step("token-not-in-sandbox", async () => {
      const result = await exec(["/bin/sh", "-c", `env; cat ${REPO_DIR}/.git/config; grep -rl "${token}" ${HOME} ${REPO_DIR}/.git 2>/dev/null; true`], { user: "root" });
      return { ok: !result.stdout.includes(token) };
    });

    if (input.q4) await this.#q4(id, step);

    await step("cleanup", async () => {
      if (this.env.GITHUB_TOKEN === "local") {
        await container.destroy();
        return { ok: true };
      }
      const deleted = await github(this.env, "DELETE", `/repos/${this.env.REPO}/git/refs/heads/${ownBranch}`);
      await container.destroy();
      return { ok: true, ownBranchDeleted: deleted.status };
    });

    const signIn = this.env.GITHUB_TOKEN === "local" ? {} : await whoAmI(this.env).catch(() => ({}));
    await emit({
      done: true,
      seconds: Math.round((Date.now() - started) / 1000),
      gitHubSignIn: signIn,
      hosts: [...this.#hosts.values()],
      refreshed,
    });
  }

  /** Q4: does update-branch (and the merges API) work for a sign-in without the workflow scope when main has a workflow change the branch lacks? */
  async #q4(id: string, step: (name: string, work: () => Promise<Record<string, unknown>>) => Promise<Record<string, unknown>>) {
    const repo = this.env.REPO;
    const branch = `feedback/${id}-q4`;
    let base = "";
    let pr = 0;
    await step("q4-setup (branch from before main's last workflow change)", async () => {
      const commits = await github(this.env, "GET", `/repos/${repo}/commits?sha=main&path=.github/workflows&per_page=1`);
      const workflowCommit = commits.body[0]?.sha;
      const detail = await github(this.env, "GET", `/repos/${repo}/commits/${workflowCommit}`);
      base = detail.body.parents[0].sha;
      const ref = await github(this.env, "POST", `/repos/${repo}/git/refs`, { ref: `refs/heads/${branch}`, sha: base });
      const file = await github(this.env, "PUT", `/repos/${repo}/contents/spike/q4-${id}.txt`, {
        message: "spike: q4",
        content: btoa("update-branch test\n"),
        branch,
      });
      return { ok: ref.status === 201 && file.status === 201, workflowCommit, base, createRef: ref.status, commit: file.status };
    });
    await step("q4-update-branch", async () => {
      const opened = await github(this.env, "POST", `/repos/${repo}/pulls`, {
        title: `Feedback spike ${id}: update-branch test (closes itself)`,
        head: branch,
        base: "main",
        body: "Throwaway pull request from the feedback spike (docs/feedback.md, Q4). It is closed and its branch deleted within a minute.",
        draft: true,
      });
      pr = opened.body?.number ?? 0;
      if (opened.status !== 201) return { ok: false, openPr: opened.status, message: opened.body?.message };
      const head = (await github(this.env, "GET", `/repos/${repo}/git/ref/heads/${branch}`)).body.object.sha;
      const update = await github(this.env, "PUT", `/repos/${repo}/pulls/${pr}/update-branch`, { expected_head_sha: head });
      let moved = false;
      for (let i = 0; i < 12 && update.status === 202 && !moved; i++) {
        await new Promise((resolve) => setTimeout(resolve, 5_000));
        moved = (await github(this.env, "GET", `/repos/${repo}/git/ref/heads/${branch}`)).body.object.sha !== head;
      }
      return { ok: moved, pr, status: update.status, message: update.body?.message, branchMoved: moved };
    });
    await step("q4-merges-api (same merge without a pull request)", async () => {
      const merged = await github(this.env, "POST", `/repos/${repo}/merges`, { base: branch, head: "main", commit_message: "spike: merge main" });
      return { ok: merged.status === 201 || merged.status === 204, status: merged.status, message: merged.body?.message };
    });
    await step("q4-cleanup", async () => {
      const closed = pr ? await github(this.env, "PATCH", `/repos/${repo}/pulls/${pr}`, { state: "closed" }) : { status: 0 };
      const deleted = await github(this.env, "DELETE", `/repos/${repo}/git/refs/heads/${branch}`);
      return { ok: deleted.status === 204, prClosed: closed.status, branchDeleted: deleted.status };
    });
  }
}

export default {
  async fetch(request, env): Promise<Response> {
    const url = new URL(request.url);
    const auth = request.headers.get("authorization") ?? "";
    if (!env.SPIKE_TOKEN || !(await sameText(auth, `Bearer ${env.SPIKE_TOKEN}`))) {
      return new Response("Not found", { status: 404 });
    }
    if (url.pathname === "/run" && request.method === "POST") {
      const input = (await request.json()) as RunInput;
      const name = `spike-${crypto.randomUUID().replace(/-/g, "").slice(0, 12)}`;
      return env.SANDBOX.getByName(name).fetch("https://spike/run", {
        method: "POST",
        body: JSON.stringify({ ...input, name }),
      });
    }
    return new Response("Not found", { status: 404 });
  },
} satisfies ExportedHandler<Env>;

/** The CLI command line, built like ai-kit's `process.rs` builds it for the chosen model and effort. */
function agentArgv(engine: Engine, prompt: string, model: string | undefined, effort: string | undefined): string[] {
  if (engine === "claude") {
    return [
      "claude", "-p", "--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions",
      "--no-session-persistence", "--max-turns", "20",
      ...(model ? ["--model", model] : []),
      ...(effort && effort !== "none" ? ["--effort", effort] : []),
      "--", prompt,
    ];
  }
  return [
    "codex", "exec", "--json", "--sandbox", "danger-full-access", "--skip-git-repo-check",
    ...(model ? ["-m", model] : []),
    ...(effort && effort !== "none" ? ["-c", `model_reasoning_effort=${JSON.stringify(effort)}`] : []),
    prompt,
  ];
}

/** The agent's final answer or error from its JSON event stream, shortened and redacted. */
function outcome(engine: Engine, stdout: string): string {
  const events = stdout.split("\n").flatMap((line) => {
    try {
      return [JSON.parse(line)];
    } catch {
      return [];
    }
  });
  if (engine === "claude") {
    const result = events.filter((event) => event.type === "result").pop();
    return redact(result ? `${result.is_error ? "error: " : ""}${result.result ?? result.subtype}` : "no result event").slice(0, 400);
  }
  const last = events.filter((event) => event.type === "item.completed" || event.type === "error" || event.type === "turn.failed").pop();
  return redact(JSON.stringify(last?.item?.text ?? last?.message ?? last?.error ?? last ?? "no events")).slice(0, 400);
}

function refuse(reason: string): { response: Response; decision: string } {
  return { response: new Response(`${reason}\n`, { status: 403 }), decision: `refused: ${reason}` };
}

function tail(text: string): string {
  return redact(text.slice(-1500));
}

/** Drops anything shaped like a credential before it leaves the spike. */
function redact(text: string): string {
  return text
    .replace(/(sk-ant-[\w-]+|gh[opsur]_\w+|eyJ[\w-]+\.[\w-]+\.[\w-]+)/g, "[redacted]")
    .replace(/"(access_token|refresh_token|id_token|accessToken|refreshToken)"\s*:\s*"[^"]*"/g, '"$1":"[redacted]"');
}

async function sha256(text: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function sameText(a: string, b: string): Promise<boolean> {
  const [x, y] = await Promise.all([sha256(a), sha256(b)]);
  return x === y;
}

async function gunzip(bytes: Uint8Array): Promise<Uint8Array> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}
