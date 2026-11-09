// The job state machine (docs/feedback.md, Job states) as a pure function of a job and an event, so every journey
// can be tested as a table.

export type State =
  | "queued"
  | "working"
  | "testing"
  | "fixing"
  | "review"
  | "merging"
  | "shipped"
  | "merged_server"
  | "merged_not_released"
  | "no_change"
  | "failed";

export type Engine = "claude" | "codex";

/** Why a job failed; each has the sentence the sender sees after "Couldn't ship this: ". */
export type Reason = "tests" | "plan_limit" | "signin" | "too_long" | "main_moved" | "stopped" | "declined" | "error";

export const FINAL: ReadonlySet<State> = new Set(["shipped", "merged_server", "merged_not_released", "no_change", "failed"]);

/** At most 3 agent rounds: the first attempt and 2 fix rounds. */
export const MAX_ROUNDS = 3;
/** At most 3 more tries when `main` moves while a job waits to merge. */
export const MAX_MERGE_TRIES = 3;

/** The part of a job the state machine reads and writes. */
export interface Progress {
  state: State;
  round: number;
  mergeTries: number;
  reason: Reason | null;
}

export type Event =
  | { type: "round_start" }
  | { type: "round_done"; outcome: "pushed" | "no_commits" | "plan_limit" | "signin" | "timeout" | "error" }
  | { type: "ci"; result: "green" | "red" }
  | { type: "approved" }
  | { type: "declined" }
  | { type: "conflict" }
  | { type: "main_moved" }
  | { type: "merged"; android: boolean }
  | { type: "release"; result: "success" | "failure" | "none" }
  | { type: "stop"; reason: Reason };

/**
 * The job after one event. An event that does not apply in the job's state leaves it unchanged, and a final state
 * never changes.
 *
 * @param job the job's state, round count, merge tries and reason
 * @param event what happened
 * @returns the new progress (a new object; `job` is not changed)
 */
export function step(job: Progress, event: Event): Progress {
  if (FINAL.has(job.state)) return job;
  const fail = (reason: Reason): Progress => ({ ...job, state: "failed", reason });
  const another = (reason: Reason): Progress => (job.round < MAX_ROUNDS ? { ...job, state: "fixing" } : fail(reason));
  if (event.type === "stop") return fail(event.reason);
  switch (job.state) {
    case "queued":
    case "fixing":
      return event.type === "round_start" ? { ...job, state: "working", round: job.round + 1 } : job;
    case "working":
      if (event.type !== "round_done") return job;
      switch (event.outcome) {
        case "pushed":
          return { ...job, state: "testing" };
        case "no_commits":
          // A first round without commits means the feedback was not a code change; a fix round without them failed.
          return job.round === 1 ? { ...job, state: "no_change" } : fail("tests");
        case "plan_limit":
          return fail("plan_limit");
        case "signin":
          return fail("signin");
        case "timeout":
          return fail("too_long");
        default:
          return fail("error");
      }
    case "testing":
      if (event.type !== "ci") return job;
      // Green CI waits for Octavian's review instead of merging (Octavian, 6 Nov 2026).
      return event.result === "green" ? { ...job, state: "review" } : another("tests");
    case "review":
      if (event.type === "approved") return { ...job, state: "merging" };
      if (event.type === "declined") return fail("declined");
      return job;
    case "merging":
      switch (event.type) {
        case "ci":
          return event.result === "green" ? job : another("tests");
        case "conflict":
          return another("main_moved");
        case "main_moved":
          return job.mergeTries < MAX_MERGE_TRIES ? { ...job, mergeTries: job.mergeTries + 1 } : fail("main_moved");
        case "merged":
          return event.android ? job : { ...job, state: "merged_server" };
        case "release":
          return { ...job, state: event.result === "success" ? "shipped" : "merged_not_released" };
        default:
          return job;
      }
    default:
      return job;
  }
}

const ENGINE_NAMES: Record<Engine, string> = { claude: "Claude", codex: "ChatGPT" };

/** The sentence after "Couldn't ship this: " for each reason. */
export function reasonText(reason: Reason, engine: Engine): string {
  switch (reason) {
    case "tests":
      return "the tests kept failing.";
    case "plan_limit":
      return `your ${ENGINE_NAMES[engine]} plan's limit was reached. Try again later.`;
    case "signin":
      return `your ${engine === "claude" ? "Claude Code" : "Codex"} sign-in has expired. Sign in again in Providers.`;
    case "too_long":
      return "it took too long.";
    case "main_moved":
      return "the app changed too often to merge.";
    case "stopped":
      return "stopped by Octavian.";
    case "declined":
      return "Octavian decided not to ship it.";
    default:
      return "something went wrong on our side.";
  }
}

/**
 * The notification's title and detail for a job (docs/feedback.md, Job states).
 *
 * @param job the job's progress
 * @param engine the job's engine, named in plan and sign-in reasons
 * @param summary the agent's one-line reason, shown for `no_change`
 * @param version the shipped version, shown for `shipped`
 * @returns the title and an optional detail line
 */
export function describe(job: Progress, engine: Engine, summary: string | null, version: string | null): { title: string; detail: string | null } {
  switch (job.state) {
    case "queued":
      return { title: "Your feedback is waiting its turn", detail: null };
    case "working":
      return { title: "Working on your feedback", detail: null };
    case "testing":
      return { title: "Testing your change", detail: summary };
    case "fixing":
      return { title: "Fixing a failed test", detail: summary };
    case "review":
      return { title: "Waiting for Octavian's review", detail: summary };
    case "merging":
      return { title: "Getting ready to ship", detail: summary };
    case "shipped":
      return { title: `Shipped in ${version ?? "a new version"}. Tap to update`, detail: summary };
    case "merged_server":
      return { title: "Merged. It goes live with the next server update", detail: summary };
    case "merged_not_released":
      return { title: "Merged, but the release didn't go out. Octavian can see why", detail: summary };
    case "no_change":
      return { title: "No change needed", detail: summary };
    case "failed":
      return { title: "Couldn't ship this", detail: reasonText(job.reason ?? "error", engine) };
  }
}
