import { describe, expect, it } from "vitest";

import { GitHub } from "../src/github";

const ENV = { GITHUB_TOKEN: "gho_test", REPO: "OctavianTocan/mymind-reconstruction" };
const BASE = `https://api.github.com/repos/${ENV.REPO}`;

/** A GitHub that answers each `METHOD url` with a recorded status and body, and remembers what it was sent. */
function recorded(answers: Record<string, [number, unknown]>) {
  const sent: { method: string; url: string; body: unknown }[] = [];
  const github = new GitHub(ENV, async (request) => {
    const key = `${request.method} ${request.url.replace(BASE, "")}`;
    sent.push({ method: request.method, url: request.url, body: request.body ? await request.json() : null });
    const [status, body] = answers[key] ?? [404, { message: `no recording for ${key}` }];
    return new Response(typeof body === "string" ? body : JSON.stringify(body), { status });
  });
  return { github, sent };
}

const run = (name: string, status: string, conclusion: string | null, id = 1) => ({ id, name, status, conclusion, html_url: `https://github.com/x/actions/runs/1/job/${id}` });

describe("GitHub calls", () => {
  it("opens a pull request, or finds the one a retried round already opened", async () => {
    const opened = recorded({ "POST /pulls": [201, { number: 81 }] });
    expect(await opened.github.openPull("feedback/abc", "Feedback: x", "body")).toBe(81);
    expect(opened.sent[0].body).toMatchObject({ head: "feedback/abc", base: "main" });
    const existing = recorded({
      "POST /pulls": [422, { message: "A pull request already exists" }],
      "GET /pulls?state=open&head=OctavianTocan:feedback/abc": [200, [{ number: 82 }]],
    });
    expect(await existing.github.openPull("feedback/abc", "t", "b")).toBe(82);
  });

  it("reads required checks as pending, red with the failed check, or green", async () => {
    const path = "GET /commits/abc/check-runs?per_page=100";
    const pending = recorded({ [path]: [200, { check_runs: [run("android", "completed", "success")] }] });
    expect(await pending.github.checks("abc", ["android", "rust"])).toEqual({ result: "pending" });
    const red = recorded({ [path]: [200, { check_runs: [run("android", "completed", "success"), run("rust", "completed", "failure", 7)] }] });
    expect(await red.github.checks("abc", ["android", "rust"])).toMatchObject({ result: "red", name: "rust", jobId: 7 });
    const green = recorded({ [path]: [200, { check_runs: [run("android", "completed", "success"), run("rust", "completed", "success"), run("other", "completed", "failure")] }] });
    expect(await green.github.checks("abc", ["android", "rust"])).toEqual({ result: "green" });
    // A re-run counts: the newest run of a check decides.
    const rerun = recorded({ [path]: [200, { check_runs: [run("rust", "completed", "failure", 1), run("rust", "completed", "success", 2), run("android", "completed", "success", 3)] }] });
    expect(await rerun.github.checks("abc", ["android", "rust"])).toEqual({ result: "green" });
  });

  it("tells a started update-branch from a conflict and a moved head", async () => {
    const path = "PUT /pulls/5/update-branch";
    expect(await recorded({ [path]: [202, { message: "Updating pull request branch." }] }).github.updateBranch(5, "h")).toBe("started");
    expect(await recorded({ [path]: [422, { message: "merge conflict between base and head" }] }).github.updateBranch(5, "h")).toBe("conflict");
    expect(await recorded({ [path]: [422, { message: "expected head sha didn't match current head ref." }] }).github.updateBranch(5, "h")).toBe("stale");
  });

  it("marks a pull request for review and reads Octavian's answer from its state and labels", async () => {
    const marked = recorded({ "POST /labels": [422, { message: "Validation Failed" }], "POST /issues/5/labels": [200, [{ name: "needs-review" }]] });
    await marked.github.markForReview(5, "needs-review", "approved");
    expect(marked.sent.map((s) => [s.method, s.url.replace(BASE, ""), s.body])).toEqual([
      ["POST", "/labels", { name: "approved", color: "0e8a16", description: "Lets the feedback runner merge this pull request" }],
      ["POST", "/issues/5/labels", { labels: ["needs-review"] }],
    ]);
    const pull = (body: object) => recorded({ "GET /pulls/5": [200, { state: "open", merged: false, labels: [], ...body }] }).github.review(5, "approved");
    expect(await pull({ labels: [{ name: "needs-review" }] })).toEqual({ result: "waiting" });
    expect(await pull({ labels: [{ name: "needs-review" }, { name: "approved" }] })).toEqual({ result: "approved" });
    expect(await pull({ state: "closed" })).toEqual({ result: "closed" });
    expect(await pull({ state: "closed", merged: true, merge_commit_sha: "m9" })).toEqual({ result: "merged", sha: "m9" });
    expect(await recorded({}).github.review(5, "approved")).toEqual({ result: "waiting" });
  });

  it("squash-merges at an exact head and reports main moving", async () => {
    const merged = recorded({ "PUT /pulls/5/merge": [200, { merged: true, sha: "m1" }] });
    expect(await merged.github.squash(5, "h", "Feedback: x (#5)", "Feedback-Id: abc")).toEqual({ sha: "m1" });
    expect(merged.sent[0].body).toMatchObject({ sha: "h", merge_method: "squash" });
    expect(await recorded({ "PUT /pulls/5/merge": [409, { message: "Head branch was modified" }] }).github.squash(5, "h", "t", "m")).toBe("moved");
  });

  it("reads the Android release run for a merge commit", async () => {
    const path = "GET /actions/workflows/android-release.yml/runs?head_sha=m1&per_page=10";
    expect(await recorded({ [path]: [200, { workflow_runs: [{ status: "completed", conclusion: "success" }] }] }).github.release("m1")).toBe("success");
    expect(await recorded({ [path]: [200, { workflow_runs: [{ status: "completed", conclusion: "failure" }] }] }).github.release("m1")).toBe("failure");
    expect(await recorded({ [path]: [200, { workflow_runs: [{ status: "in_progress", conclusion: null }] }] }).github.release("m1")).toBeNull();
    expect(await recorded({ [path]: [200, { workflow_runs: [] }] }).github.release("m1")).toBeNull();
  });

  it("reads and writes a file through the contents API", async () => {
    const content = btoa("## 0.6.54 (65), 2026-11-06\n- é\n".replace("é", "Ã©"));
    const files = recorded({
      "GET /contents/android/CHANGELOG.md?ref=feedback%2Fabc": [200, { content, sha: "blob1" }],
      "PUT /contents/android/CHANGELOG.md": [200, { commit: { sha: "c2" } }],
    });
    expect(await files.github.file("android/CHANGELOG.md", "feedback/abc")).toEqual({ text: "## 0.6.54 (65), 2026-11-06\n- é\n", sha: "blob1" });
    expect(await files.github.writeFile("android/CHANGELOG.md", "feedback/abc", "text é", "blob1", "msg")).toBe("c2");
    expect(files.sent[1].body).toMatchObject({ sha: "blob1", branch: "feedback/abc" });
  });
});
