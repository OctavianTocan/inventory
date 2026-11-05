// FeedbackSandbox: one Cloudflare sandbox per round (docs/feedback.md). Its container runs `round.py` in the
// background; the round's FeedbackJob polls it from an alarm and destroys it when the round ends.
import { Files } from "@cloudflare/sandbox";
import { DurableObject, WorkerEntrypoint } from "cloudflare:workers";

import type { Env } from "./env";
import { decide, type Policy } from "./outbound";
import type { Engine } from "./states";

const CA = "/etc/cloudflare/certs/cloudflare-containers-ca.crt";
const HOME = "/home/agent";
const ROUND = "/workspace/round";
const LOGIN_PATHS = [".claude/.credentials.json", ".claude.json", ".codex/auth.json"];

/** What a round needs; the login files go into the sandbox and are never stored. */
export interface RoundInput {
  id: string;
  round: number;
  engine: Engine;
  model: string;
  effort: string;
  prompt: string;
  /** The `main` commit the branch started from, or null on the first round. */
  base: string | null;
  /** True when this round resolves a merge conflict with `main`. */
  conflict: boolean;
  files: Record<string, unknown>;
}

/** What `round.py` writes when it is done. */
export interface RoundResult {
  done: true;
  outcome: "pushed" | "no_commits" | "plan_limit" | "signin" | "timeout" | "error";
  summary: string;
  description: string;
  head: string | null;
  base: string | null;
  bumped: string | null;
  error: string | null;
  seconds: number;
}

/** The sandbox's name for one round of one job; it also names the round in logs. */
export function sandboxName(id: string, round: number): string {
  return `${id}-r${round}`;
}

/** Every request leaving a sandbox comes here; `props.name` names the sandbox. */
export class Outbound extends WorkerEntrypoint<Env, { name: string }> {
  override async fetch(request: Request): Promise<Response> {
    const sandbox = this.env.FEEDBACK_SANDBOX.getByName(this.ctx.props.name);
    const policy = await sandbox.policy();
    const host = new URL(request.url).hostname;
    if (!policy) return new Response("this sandbox has no round\n", { status: 403 });
    const { response, decision } = await decide(request, policy, this.env);
    console.log(JSON.stringify({ event: "outbound", sandbox: this.ctx.props.name, host, method: request.method, status: response.status, decision }));
    return response;
  }
}

export class FeedbackSandbox extends DurableObject<Env> {
  /** The engine and own branch the outbound handler enforces, or null before a round starts. */
  async policy(): Promise<Policy | null> {
    return (await this.ctx.storage.get<Policy>("policy")) ?? null;
  }

  /**
   * Starts the round: the container, its trust of the outbound handler's certificate, the sign-in files in the agent's
   * home (outside the repository folder), and `round.py` in the background.
   *
   * @param input the round's job, prompt and sign-in files
   */
  async begin(input: RoundInput): Promise<void> {
    const container = this.#container();
    await this.ctx.storage.put("policy", { engine: input.engine, ownRef: `refs/heads/feedback/${input.id}` } satisfies Policy);
    container.start({ image: (container as any).images.sandbox, instance: this.env.SANDBOX_INSTANCE ?? "standard-1", enableInternet: false } as any);
    const outbound = (this.ctx as any).exports.Outbound({ props: { name: sandboxName(input.id, input.round) } });
    await (container as any).interceptAllOutboundHttp(outbound);
    await (container as any).interceptOutboundHttps("*", outbound);
    // The round stops itself at 45 minutes; this only reaps a container whose job stopped polling.
    await (container as any).setInactivityTimeout(70 * 60_000);
    await this.#root(`for i in $(seq 1 30); do [ -s ${CA} ] && break; sleep 1; done
cp ${CA} /usr/local/share/ca-certificates/cloudflare-containers.crt && update-ca-certificates >/dev/null
install -d -o agent -g agent /workspace ${HOME} && install -d -m 700 ${ROUND}`);
    const files = new Files(container as any);
    for (const [path, content] of Object.entries(input.files)) {
      if (!LOGIN_PATHS.includes(path)) continue;
      await files.mkdir(`${HOME}/${path}`.replace(/\/[^/]+$/, ""), { recursive: true });
      await files.writeFile(`${HOME}/${path}`, JSON.stringify(content, null, 2));
    }
    const { files: _files, prompt, ...job } = input;
    await files.writeFile(`${ROUND}/input.json`, JSON.stringify({ ...job, repo: this.env.REPO }));
    await files.writeFile(`${ROUND}/prompt.txt`, prompt);
    await this.#root(`chown -R agent:agent ${HOME} && chmod -R go-rwx ${HOME}
setsid nohup python3 /opt/feedback/round.py > ${ROUND}/log 2>&1 < /dev/null &`);
  }

  /**
   * Where the round is.
   *
   * @returns `running` with the round's phase, `done` with its result, or `gone` when the container is not running
   */
  async poll(): Promise<{ state: "running"; phase: string } | { state: "done"; result: RoundResult } | { state: "gone" }> {
    const container = this.#container();
    if (!container.running) return { state: "gone" };
    const files = new Files(container as any);
    const read = (path: string) => files.readFile(path).then((file: any) => file.text() as Promise<string>).catch(() => "");
    const result = await read(`${ROUND}/result.json`);
    if (result) return { state: "done", result: JSON.parse(result) as RoundResult };
    const status = await read(`${ROUND}/status.json`);
    return { state: "running", phase: status ? (JSON.parse(status).phase ?? "starting") : "starting" };
  }

  /** The sign-in files as they are now in the sandbox (the CLI may have refreshed them), by path. */
  async loginFiles(): Promise<Record<string, unknown>> {
    const container = this.#container();
    if (!container.running) return {};
    const files = new Files(container as any);
    const found: Record<string, unknown> = {};
    for (const path of LOGIN_PATHS) {
      const text = await files.readFile(`${HOME}/${path}`).then((file: any) => file.text() as Promise<string>).catch(() => "");
      if (!text) continue;
      try {
        found[path] = JSON.parse(text);
      } catch {}
    }
    return found;
  }

  /** Destroys the container and forgets the round. */
  async end(): Promise<void> {
    const container = this.ctx.container;
    if (container?.running) await container.destroy().catch(() => {});
    await this.ctx.storage.delete("policy");
  }

  #container() {
    const container = this.ctx.container;
    if (!container) throw new Error("no container binding");
    return container;
  }

  /** Runs a shell script as root in the container and throws when it fails. */
  async #root(script: string): Promise<void> {
    const process = await (this.#container() as any).exec(["/bin/sh", "-c", script], { user: "root", cwd: "/" });
    const out = await process.output();
    if (out.exitCode !== 0) throw new Error(`sandbox setup failed: ${new TextDecoder().decode(out.stderr).slice(-400)}`);
  }
}
