import { describe, expect, it } from "vitest";

import { describe as describeJob, type Event, type Progress, step } from "../src/states";

const start: Progress = { state: "queued", round: 0, mergeTries: 0, reason: null };

/** Runs events from a fresh job and returns each state on the way. */
function run(...events: Event[]): string[] {
  let job = start;
  return events.map((event) => {
    job = step(job, event);
    return job.reason ? `${job.state}:${job.reason}` : job.state;
  });
}

const roundStart: Event = { type: "round_start" };
const pushed: Event = { type: "round_done", outcome: "pushed" };
const green: Event = { type: "ci", result: "green" };
const red: Event = { type: "ci", result: "red" };
const approved: Event = { type: "approved" };

describe("job states, one table per journey (docs/feedback.md, CUJs)", () => {
  it("CUJ 1: feedback becomes an app release", () => {
    expect(run(roundStart, pushed, green, approved, green, { type: "merged", android: true }, { type: "release", result: "success" })).toEqual([
      "working", "testing", "review", "merging", "merging", "merging", "shipped",
    ]);
  });

  it("CUJ 2: a server-only change ends merged", () => {
    expect(run(roundStart, pushed, green, approved, { type: "merged", android: false })).toEqual(["working", "testing", "review", "merging", "merged_server"]);
  });

  it("CUJ 3: three red results end the job and no fourth round starts", () => {
    expect(run(roundStart, pushed, red, roundStart, pushed, red, roundStart, pushed, red, roundStart)).toEqual([
      "working", "testing", "fixing", "working", "testing", "fixing", "working", "testing", "failed:tests", "failed:tests",
    ]);
  });

  it("CUJ 4: a plan limit ends the job without retrying", () => {
    expect(run(roundStart, { type: "round_done", outcome: "plan_limit" })).toEqual(["working", "failed:plan_limit"]);
  });

  it("an expired sign-in and a timed-out round end the job", () => {
    expect(run(roundStart, { type: "round_done", outcome: "signin" })).toEqual(["working", "failed:signin"]);
    expect(run(roundStart, { type: "round_done", outcome: "timeout" })).toEqual(["working", "failed:too_long"]);
  });

  it("CUJ 5: main moving re-queues at most 3 times", () => {
    const moved: Event = { type: "main_moved" };
    expect(run(roundStart, pushed, green, approved, moved, moved, moved, moved)).toEqual([
      "working", "testing", "review", "merging", "merging", "merging", "merging", "failed:main_moved",
    ]);
  });

  it("CUJ 6: a first round without commits is no change; a fix round without them fails", () => {
    const none: Event = { type: "round_done", outcome: "no_commits" };
    expect(run(roundStart, none)).toEqual(["working", "no_change"]);
    expect(run(roundStart, pushed, red, roundStart, none)).toEqual(["working", "testing", "fixing", "working", "failed:tests"]);
  });

  it("CUJ 7: a failed or missing release ends merged_not_released", () => {
    expect(run(roundStart, pushed, green, approved, { type: "merged", android: true }, { type: "release", result: "failure" })).toEqual([
      "working", "testing", "review", "merging", "merging", "merged_not_released",
    ]);
    expect(run(roundStart, pushed, green, approved, { type: "merged", android: true }, { type: "release", result: "none" }).pop()).toBe("merged_not_released");
  });

  it("a merge conflict sends the job back for a round, counted toward the 3", () => {
    expect(run(roundStart, pushed, green, approved, { type: "conflict" }, roundStart)).toEqual(["working", "testing", "review", "merging", "fixing", "working"]);
  });

  it("red CI after merging main in goes back for a fix round", () => {
    expect(run(roundStart, pushed, green, approved, red)).toEqual(["working", "testing", "review", "merging", "fixing"]);
  });

  it("green CI waits for Tavi's review: nothing but approved moves it on, and a declined job fails", () => {
    expect(run(roundStart, pushed, green, green, { type: "merged", android: true }, red, approved)).toEqual([
      "working", "testing", "review", "review", "review", "review", "merging",
    ]);
    expect(run(roundStart, pushed, green, { type: "declined" })).toEqual(["working", "testing", "review", "failed:declined"]);
  });

  it("a fix round after an approval goes back to review", () => {
    expect(run(roundStart, pushed, green, approved, red, roundStart, pushed, green)).toEqual([
      "working", "testing", "review", "merging", "fixing", "working", "testing", "review",
    ]);
  });

  it("stop ends any running job, and a final state never changes", () => {
    expect(run(roundStart, { type: "stop", reason: "stopped" }, roundStart, green)).toEqual(["working", "failed:stopped", "failed:stopped", "failed:stopped"]);
  });

  it("events that do not apply leave the job as it is", () => {
    expect(run(green, pushed, { type: "merged", android: true })).toEqual(["queued", "queued", "queued"]);
  });

  it("describes each state with the notification's words", () => {
    expect(describeJob({ ...start, state: "shipped" }, "claude", "Share sheet stays open longer", "0.6.54").title).toBe("Shipped in 0.6.54. Tap to update");
    expect(describeJob({ ...start, state: "failed", reason: "plan_limit" }, "claude", null, null)).toEqual({
      title: "Couldn't ship this",
      detail: "your Claude plan's limit was reached. Try again later.",
    });
    expect(describeJob({ ...start, state: "review" }, "claude", "Bigger tiles", null)).toEqual({ title: "Waiting for Tavi's review", detail: "Bigger tiles" });
    expect(describeJob({ ...start, state: "failed", reason: "declined" }, "codex", null, null).detail).toBe("Tavi decided not to ship it.");
    expect(describeJob({ ...start, state: "no_change" }, "codex", "It already does that", null)).toEqual({ title: "No change needed", detail: "It already does that" });
  });
});
