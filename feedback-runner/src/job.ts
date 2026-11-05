// FeedbackJob: one Durable Object per feedback id (docs/feedback.md). It owns the job's record and state machine, runs
// its rounds (sign-in checkout, sandbox, checkin), watches CI, and merges when the desk gives it the merge turn. An
// alarm drives every step, so nothing waits inside a request.
import { DurableObject } from "cloudflare:workers";

import { ownEntry, withEntry } from "./changelog";
import type { Env } from "./env";
import { GitHub } from "./github";
import { buildPrompt, type RoundKind } from "./prompt";
import { redact, signInStrings } from "./redact";
import { type RoundResult, sandboxName } from "./sandbox";
import { describe, type Engine, FINAL, type Progress, type Reason, step, type Event } from "./states";

const CHANGELOG = "android/CHANGELOG.md";
/** A round is stopped a little after `round.py`'s own 45-minute limit. */
const ROUND_LIMIT_MS = 48 * 60_000;
/** A job that has not ended after this long fails as "took too long". */
const JOB_LIMIT_MS = 10 * 60 * 60_000;
/** How long to wait for the `Android release` run after a merge. */
const RELEASE_WAIT_MS = 60 * 60_000;
/** How long GitHub may take to finish an update-branch. */
const UPDATE_WAIT_MS = 10 * 60_000;

/** Where a job in `merging` is. */
type MergeStage = "waiting" | "update" | "updating" | "ci" | "release";

/** The job's whole record, one per object. */
export interface JobRecord {
  id: string;
  email: string;
  /** The AI login (AgentHome name) whose sign-in the rounds borrow. */
  login: string;
  /** The sender's display name for the pull request. */
  name: string;
  text: string;
  engine: Engine;
  model: string;
  effort: string;
  createdAt: number;
  updatedAt: number;
  progress: Progress;
  pr: number | null;
  head: string | null;
  /** The `main` commit the branch started from (or was last merged from). */
  base: string | null;
  /** The version of the CHANGELOG entry the round script added, while the branch carries one. */
  bumped: string | null;
  /** That entry's lines while the merge queue has taken it out of the branch to merge main in. */
  entry: string[] | null;
  version: string | null;
  summary: string | null;
  description: string | null;
  next: RoundKind;
  roundStartedAt: number | null;
  stage: MergeStage;
  stageAt: number | null;
  mergeSha: string | null;
  /** Links Tavi's record keeps: the last failed check and the release run. */
  ciLog: string | null;
  checkedOut: boolean;
}

/** The row the desk keeps for each job (Tavi's record). */
export interface JobRow {
  id: string;
  email: string;
  state: string;
  engine: string;
  model: string;
  effort: string;
  pr: number | null;
  rounds: number;
  reason: string | null;
  created_at: number;
  finished_at: number | null;
  ci_log: string | null;
}

export class FeedbackJob extends DurableObject<Env> {
  #github = new GitHub(this.env);

  /** Stores a new job and starts it on the next alarm. */
  async create(record: Omit<JobRecord, "updatedAt" | "progress" | "pr" | "head" | "base" | "bumped" | "entry" | "version" | "summary" | "description" | "next" | "roundStartedAt" | "stage" | "stageAt" | "mergeSha" | "ciLog" | "checkedOut">): Promise<void> {
    if (await this.ctx.storage.get("job")) return;
    const job: JobRecord = {
      ...record,
      updatedAt: Date.now(),
      progress: { state: "queued", round: 0, mergeTries: 0, reason: null },
      pr: null,
      head: null,
      base: null,
      bumped: null,
      entry: null,
      version: null,
      summary: null,
      description: null,
      next: { kind: "first" },
      roundStartedAt: null,
      stage: "waiting",
      stageAt: null,
      mergeSha: null,
      ciLog: null,
      checkedOut: false,
    };
    await this.ctx.storage.put("job", job);
    log("submitted", job, { engine: job.engine, model: job.model, effort: job.effort });
    await this.ctx.storage.setAlarm(Date.now());
  }

  /** The phone's view of the job, or null when `email` is not its sender. */
  async status(email: string): Promise<{ state: string; title: string; detail: string | null; version: string | null } | null> {
    const job = await this.#job();
    if (!job || job.email !== email) return null;
    const { title, detail } = describe(job.progress, job.engine, job.summary, job.version);
    return { state: job.progress.state, title, detail, version: job.version };
  }

  /** Runs the next step soon; the desk calls it when a round slot or the merge turn frees. */
  async wake(): Promise<void> {
    await this.ctx.storage.setAlarm(Date.now());
  }

  override async alarm(): Promise<void> {
    const job = await this.#job();
    if (!job || FINAL.has(job.progress.state)) return;
    let next = 60_000;
    try {
      next = await this.#tick(job);
    } catch (error) {
      log("error", job, { error: redact(String((error as Error)?.message ?? error), this.env) });
      await this.#move(job, { type: "stop", reason: "error" });
    }
    const after = await this.#job();
    if (after && !FINAL.has(after.progress.state)) await this.ctx.storage.setAlarm(Date.now() + next);
  }

  /** One step of the job; returns how long until the next. */
  async #tick(job: JobRecord): Promise<number> {
    if (Date.now() - job.createdAt > JOB_LIMIT_MS && job.progress.state !== "working") {
      await this.#move(job, { type: "stop", reason: "too_long" });
      return 0;
    }
    switch (job.progress.state) {
      case "queued":
      case "fixing":
        return (await this.#desk().claimSlot(job.id)) ? this.#startRound(job) : 5 * 60_000;
      case "working":
        return this.#watchRound(job);
      case "testing":
        return this.#watchChecks(job);
      case "merging":
        return this.#merge(job);
      default:
        return 60_000;
    }
  }

  async #startRound(job: JobRecord): Promise<number> {
    const checkout = await this.#home(job, "/feedback/checkout");
    if (checkout.status === 403) {
      // The login is still out from a round that died; its lease ends within the hour.
      await this.#desk().releaseSlot(job.id);
      return 5 * 60_000;
    }
    if (checkout.status === 409) {
      await this.#desk().releaseSlot(job.id);
      await this.#move(job, { type: "stop", reason: "signin" });
      return 0;
    }
    if (checkout.status !== 200) throw new Error(`the sign-in checkout answered ${checkout.status}`);
    const files = ((await checkout.json()) as { files: Record<string, unknown> }).files;
    job.checkedOut = true;
    job.roundStartedAt = Date.now();
    await this.#move(job, { type: "round_start" });
    const round = job.progress.round;
    log("round_started", job, { round, kind: job.next.kind });
    try {
      await this.#sandbox(job).begin({
        id: job.id,
        round,
        engine: job.engine,
        model: job.model,
        effort: job.effort,
        prompt: buildPrompt(job.text, job.next),
        base: job.base,
        conflict: job.next.kind === "conflict",
        files,
      });
    } catch (error) {
      await this.#endRound(job, false);
      throw error;
    }
    return 60_000;
  }

  async #watchRound(job: JobRecord): Promise<number> {
    const polled = await this.#sandbox(job).poll();
    if (polled.state === "running" && Date.now() - (job.roundStartedAt ?? 0) < ROUND_LIMIT_MS) return 60_000;
    const result: RoundResult | null = polled.state === "done" ? polled.result : null;
    const files = await this.#endRound(job, true);
    const secrets = signInStrings(files);
    const outcome = result?.outcome ?? (polled.state === "running" ? "timeout" : "error");
    log("round_finished", job, { round: job.progress.round, outcome, seconds: result?.seconds ?? null, error: result?.error ? redact(result.error, this.env, secrets) : null });
    if (result) {
      job.summary = redact(result.summary, this.env, secrets).slice(0, 200) || job.summary;
      job.description = redact(result.description, this.env, secrets).slice(0, 4000) || job.description;
      job.base = result.base ?? job.base;
      job.bumped = result.bumped ?? job.bumped;
      if (result.head) job.head = result.head;
    }
    await this.#move(job, { type: "round_done", outcome });
    if (job.progress.state === "testing" && !job.pr) {
      job.pr = await this.#github.openPull(`feedback/${job.id}`, `Feedback: ${job.summary || "a change from feedback"}`, this.#pullBody(job));
      await this.#save(job);
      log("pr_opened", job, { pr: job.pr });
    }
    if (job.progress.state === "testing" && job.next.kind === "conflict") {
      // A merge round brought main in; go straight back to the queue.
      await this.#move(job, { type: "ci", result: "green" });
      job.stage = "waiting";
      await this.#save(job);
      await this.#desk().enqueueMerge(job.id);
    }
    return 2 * 60_000;
  }

  /** Checks the sign-in back in (with the sandbox's refreshed files when asked), destroys the sandbox, frees the slot. */
  async #endRound(job: JobRecord, readFiles: boolean): Promise<Record<string, unknown>> {
    const sandbox = this.#sandbox(job);
    let files: Record<string, unknown> = {};
    if (readFiles) {
      try {
        files = await sandbox.loginFiles();
      } catch {}
    }
    if (job.checkedOut) {
      const checkin = await this.#home(job, "/feedback/checkin", Object.keys(files).length ? { files } : {});
      if (checkin.status !== 200 && checkin.status !== 409) log("checkin_failed", job, { status: checkin.status });
      job.checkedOut = false;
    }
    await sandbox.end().catch(() => {});
    await this.#desk().releaseSlot(job.id);
    job.roundStartedAt = null;
    await this.#save(job);
    return files;
  }

  async #watchChecks(job: JobRecord): Promise<number> {
    const checks = await this.#github.checks(job.head!, this.#required());
    if (checks.result === "pending") return 2 * 60_000;
    log("ci_result", job, { result: checks.result, head: job.head });
    if (checks.result === "green") {
      await this.#move(job, { type: "ci", result: "green" });
      job.stage = "waiting";
      await this.#save(job);
      await this.#desk().enqueueMerge(job.id);
      return 60_000;
    }
    await this.#red(job, checks.name, checks.jobUrl, checks.jobId);
    return 0;
  }

  /** A failed check: the next round fixes it, or the job ends when the rounds are spent. */
  async #red(job: JobRecord, name: string, url: string, jobId: number | null): Promise<void> {
    job.ciLog = url;
    job.next = { kind: "fix", check: name, log: redact(await this.#github.jobLog(jobId), this.env) };
    await this.#move(job, { type: "ci", result: "red" });
  }

  /** The merge steps, run while the desk gives this job the merge turn (docs/feedback.md, Merging). */
  async #merge(job: JobRecord): Promise<number> {
    const branch = `feedback/${job.id}`;
    if (job.stage === "release") {
      const result = await this.#github.release(job.mergeSha!);
      if (!result && Date.now() - (job.stageAt ?? 0) < RELEASE_WAIT_MS) return 2 * 60_000;
      job.version = result === "success" ? job.bumped : null;
      log("released", job, { result: result ?? "none", version: job.version });
      await this.#move(job, { type: "release", result: result ?? "none" });
      return 0;
    }
    if (!(await this.#desk().mergeTurn(job.id))) return 2 * 60_000;
    const head = await this.#github.branchHead(branch);
    if (!head) throw new Error("the feedback branch is gone");
    job.head = head;

    if (job.stage === "waiting" || job.stage === "update") {
      if (await this.#github.containsMain(branch)) return this.#stage(job, "ci", 0);
      let current = head;
      if (job.bumped && job.base) {
        // Two branches that each add a top CHANGELOG entry always conflict, so take ours out before merging main.
        const started = await this.#github.file(CHANGELOG, job.base);
        const ours = await this.#github.file(CHANGELOG, head);
        const entry = started && ours ? ownEntry(ours.text, started.text) : null;
        if (started && ours && entry) {
          job.entry = entry;
          await this.#save(job);
          current = await this.#github.writeFile(CHANGELOG, branch, started.text, ours.sha, `Take out the CHANGELOG entry before merging main (Feedback-Id: ${job.id})`);
        }
      }
      const update = await this.#github.updateBranch(job.pr!, current);
      if (update === "stale") return 60_000;
      if (update === "conflict") {
        await this.#desk().finishMerge(job.id);
        job.next = { kind: "conflict" };
        job.entry = null;
        await this.#move(job, { type: "conflict" });
        return 0;
      }
      job.head = current;
      return this.#stage(job, "updating", 30_000);
    }

    if (job.stage === "updating") {
      if (!(await this.#github.containsMain(branch))) {
        if (Date.now() - (job.stageAt ?? 0) > UPDATE_WAIT_MS) throw new Error("update-branch did not finish");
        return 30_000;
      }
      const main = await this.#github.branchHead("main");
      if (job.entry && main) {
        // Put the entry back on top of main's, numbered one above main's newest.
        const ours = await this.#github.file(CHANGELOG, head);
        if (!ours) throw new Error("the branch has no CHANGELOG");
        const next = withEntry(ours.text, job.entry);
        job.head = await this.#github.writeFile(CHANGELOG, branch, next.text, ours.sha, `Android ${next.version.name} (${next.version.code}) (Feedback-Id: ${job.id})`);
        job.bumped = next.version.name;
        job.entry = null;
      }
      job.base = main ?? job.base;
      return this.#stage(job, "ci", 60_000);
    }

    // stage "ci": the updated branch's checks, then the squash merge.
    const checks = await this.#github.checks(head, this.#required());
    if (checks.result === "pending") return 2 * 60_000;
    if (checks.result === "red") {
      await this.#desk().finishMerge(job.id);
      await this.#red(job, checks.name, checks.jobUrl, checks.jobId);
      return 0;
    }
    if (!(await this.#github.containsMain(branch))) return this.#mainMoved(job);
    const merged = await this.#github.squash(job.pr!, head, `Feedback: ${job.summary || "a change from feedback"} (#${job.pr})`, `${job.description ?? ""}\n\nFeedback-Id: ${job.id}`.trim());
    if (merged === "moved" || merged === "refused") return this.#mainMoved(job);
    job.mergeSha = merged.sha;
    await this.#desk().finishMerge(job.id);
    await this.#github.call("DELETE", `/repo/git/refs/heads/${branch}`);
    log("merged", job, { sha: merged.sha, android: Boolean(job.bumped) });
    await this.#move(job, { type: "merged", android: Boolean(job.bumped) });
    if (job.progress.state === "merging") return this.#stage(job, "release", 2 * 60_000);
    return 0;
  }

  async #mainMoved(job: JobRecord): Promise<number> {
    await this.#move(job, { type: "main_moved" });
    if (job.progress.state !== "merging") {
      await this.#desk().finishMerge(job.id);
      return 0;
    }
    return this.#stage(job, "update", 0);
  }

  async #stage(job: JobRecord, stage: MergeStage, next: number): Promise<number> {
    job.stage = stage;
    job.stageAt = Date.now();
    await this.#save(job);
    return next;
  }

  /** Applies an event, saves, tells the desk, and cleans up when the job ends. */
  async #move(job: JobRecord, event: Event): Promise<void> {
    const before = job.progress.state;
    job.progress = step(job.progress, event);
    await this.#save(job);
    if (FINAL.has(job.progress.state) && !FINAL.has(before)) {
      if (job.checkedOut || job.roundStartedAt) await this.#endRound(job, false).catch(() => {});
      await this.#desk().finishMerge(job.id).catch(() => {});
      if (job.progress.state === "failed" || job.progress.state === "no_change") {
        await this.#github.closePull(job.pr, `feedback/${job.id}`).catch(() => {});
      }
      if (job.progress.state === "failed") log("failed", job, { reason: job.progress.reason as Reason });
    }
  }

  async #save(job: JobRecord): Promise<void> {
    job.updatedAt = Date.now();
    await this.ctx.storage.put("job", job);
    const row: JobRow = {
      id: job.id,
      email: job.email,
      state: job.progress.state,
      engine: job.engine,
      model: job.model,
      effort: job.effort,
      pr: job.pr,
      rounds: job.progress.round,
      reason: job.progress.reason,
      created_at: job.createdAt,
      finished_at: FINAL.has(job.progress.state) ? job.updatedAt : null,
      ci_log: job.ciLog,
    };
    await this.#desk().update(row);
  }

  #pullBody(job: JobRecord): string {
    const quoted = job.text.split("\n").map((line) => `> ${line}`).join("\n");
    return `Feedback from ${job.name}, sent from ThyMind:\n\n${quoted}\n\n${job.description ?? ""}\n\nMade by ${job.engine === "claude" ? "Claude Code" : "Codex"} (${job.model}, ${job.effort}) on the sender's own plan. The runner merges it when CI is green (docs/feedback.md).\n\nFeedback-Id: ${job.id}`;
  }

  #required(): string[] {
    return (this.env.REQUIRED_CHECKS ?? "android,rust").split(",").map((name) => name.trim()).filter(Boolean);
  }

  #desk() {
    return this.env.FEEDBACK_DESK.getByName("main");
  }

  #sandbox(job: JobRecord) {
    return this.env.FEEDBACK_SANDBOX.getByName(sandboxName(job.id, job.progress.round));
  }

  /** Calls the job's AgentHome in the `mymind` Worker. */
  #home(job: JobRecord, path: string, body?: unknown): Promise<Response> {
    return this.env.AGENT_HOME.getByName(job.login).fetch(`https://agent${path}`, {
      method: "POST",
      headers: { "content-type": "application/json", "x-agent-login": job.login },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  async #job(): Promise<JobRecord | undefined> {
    return this.ctx.storage.get<JobRecord>("job");
  }
}

/** One log line per job event; never the feedback text, tokens or sign-in files. */
function log(event: string, job: JobRecord, detail: Record<string, unknown> = {}): void {
  console.log(JSON.stringify({ event, id: job.id, state: job.progress.state, ...detail }));
}
