// GitHub calls the runner makes with the stored GitHub CLI sign-in (docs/feedback.md, GitHub sign-in). The token stays in
// this Worker; sandboxes never see it.

/** The GitHub settings from the Worker's environment. */
export interface GitHubEnv {
  /** One GitHub CLI sign-in (device flow, scope `repo`, no `workflow`), made once and reused for every job. */
  GITHUB_TOKEN: string;
  /** `owner/name` of the one repository feedback changes. */
  REPO: string;
}

/** One GitHub API answer: status and parsed JSON (or text when it is not JSON). */
export interface GitHubAnswer {
  status: number;
  body: any;
}

/** Fetches through which every call goes; tests replace it with recorded answers. */
export type Fetcher = (request: Request) => Promise<Response>;

/** The GitHub REST API for one repository. Paths starting with `/repo` are relative to that repository. */
export class GitHub {
  constructor(
    private readonly env: GitHubEnv,
    private readonly send: Fetcher = (request) => fetch(request),
  ) {}

  /**
   * One API call.
   *
   * @param method the HTTP method
   * @param path the API path; `/repo/...` means `/repos/<REPO>/...`
   * @param body optional JSON body
   * @returns status and body
   */
  async call(method: string, path: string, body?: unknown): Promise<GitHubAnswer> {
    const full = path.startsWith("/repo/") ? `/repos/${this.env.REPO}${path.slice(5)}` : path;
    const response = await this.send(
      new Request(`https://api.github.com${full}`, {
        method,
        headers: {
          accept: "application/vnd.github+json",
          authorization: `Bearer ${this.env.GITHUB_TOKEN}`,
          "user-agent": "thymind-feedback-runner",
          "x-github-api-version": "2022-11-28",
          ...(body === undefined ? {} : { "content-type": "application/json" }),
        },
        body: body === undefined ? undefined : JSON.stringify(body),
      }),
    );
    const text = await response.text();
    let parsed: unknown = text;
    try {
      parsed = JSON.parse(text);
    } catch {}
    return { status: response.status, body: parsed };
  }

  /** The commit a branch points at, or null when it does not exist. */
  async branchHead(branch: string): Promise<string | null> {
    const answer = await this.call("GET", `/repo/git/ref/heads/${branch}`);
    return answer.status === 200 ? answer.body.object.sha : null;
  }

  /** Opens a pull request from `branch` into main and returns its number. */
  async openPull(branch: string, title: string, body: string): Promise<number> {
    const answer = await this.call("POST", "/repo/pulls", { title, head: branch, base: "main", body });
    if (answer.status === 201) return answer.body.number;
    // A retried round can find its pull request already open.
    const open = await this.call("GET", `/repo/pulls?state=open&head=${this.env.REPO.split("/")[0]}:${branch}`);
    if (open.status === 200 && open.body[0]?.number) return open.body[0].number;
    throw new Error(`opening the pull request answered ${answer.status}: ${answer.body?.message ?? ""}`);
  }

  /** Closes a pull request and deletes its branch; both are allowed to fail. */
  async closePull(pr: number | null, branch: string): Promise<void> {
    if (pr) await this.call("PATCH", `/repo/pulls/${pr}`, { state: "closed" });
    await this.call("DELETE", `/repo/git/refs/heads/${branch}`);
  }

  /**
   * The result of the required checks on a commit.
   *
   * @param sha the commit
   * @param required the check names that must pass
   * @returns `green` when all passed, `red` with the first failed check when one failed, otherwise `pending`
   */
  async checks(sha: string, required: string[]): Promise<{ result: "green" } | { result: "pending" } | { result: "red"; name: string; jobUrl: string; jobId: number | null }> {
    const answer = await this.call("GET", `/repo/commits/${sha}/check-runs?per_page=100`);
    if (answer.status !== 200) return { result: "pending" };
    const runs: any[] = answer.body.check_runs ?? [];
    for (const name of required) {
      const run = runs.filter((r) => r.name === name).sort((a, b) => b.id - a.id)[0];
      if (run?.status === "completed" && !["success", "neutral", "skipped"].includes(run.conclusion)) {
        return { result: "red", name, jobUrl: run.html_url, jobId: run.id ?? null };
      }
    }
    const done = required.every((name) => runs.some((r) => r.name === name && r.status === "completed"));
    return done ? { result: "green" } : { result: "pending" };
  }

  /** The last lines of a failed job's log, or a note when it cannot be read. */
  async jobLog(jobId: number | null, lines = 400): Promise<string> {
    if (!jobId) return "(no log)";
    const answer = await this.call("GET", `/repo/actions/jobs/${jobId}/logs`);
    if (answer.status !== 200 || typeof answer.body !== "string") return `(the log could not be read: ${answer.status})`;
    return answer.body.split("\n").slice(-lines).join("\n");
  }

  /**
   * Merges main into a pull request's branch (GitHub's update-branch).
   *
   * @returns `started` when GitHub accepted it, `conflict` when the merge conflicts, `stale` when the head moved
   */
  async updateBranch(pr: number, head: string): Promise<"started" | "conflict" | "stale"> {
    const answer = await this.call("PUT", `/repo/pulls/${pr}/update-branch`, { expected_head_sha: head });
    if (answer.status === 202) return "started";
    if (answer.status === 422 && /expected head sha/i.test(answer.body?.message ?? "")) return "stale";
    if (answer.status === 422 || answer.status === 409) return "conflict";
    throw new Error(`update-branch answered ${answer.status}: ${answer.body?.message ?? ""}`);
  }

  /** A file's text and blob sha on a ref, or null when missing. */
  async file(path: string, ref: string): Promise<{ text: string; sha: string } | null> {
    const answer = await this.call("GET", `/repo/contents/${path}?ref=${encodeURIComponent(ref)}`);
    if (answer.status !== 200) return null;
    const bytes = Uint8Array.from(atob(answer.body.content.replace(/\n/g, "")), (c) => c.charCodeAt(0));
    return { text: new TextDecoder().decode(bytes), sha: answer.body.sha };
  }

  /** Commits one file's new text to a branch and returns the new commit. */
  async writeFile(path: string, branch: string, text: string, sha: string, message: string): Promise<string> {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    for (const byte of bytes) binary += String.fromCharCode(byte);
    const answer = await this.call("PUT", `/repo/contents/${path}`, { message, content: btoa(binary), sha, branch });
    if (answer.status !== 200 && answer.status !== 201) throw new Error(`writing ${path} answered ${answer.status}: ${answer.body?.message ?? ""}`);
    return answer.body.commit.sha;
  }

  /**
   * Squash-merges a pull request at an exact head.
   *
   * @returns the merge commit, or `moved` when the head or main changed first
   */
  async squash(pr: number, head: string, title: string, message: string): Promise<{ sha: string } | "moved" | "refused"> {
    const answer = await this.call("PUT", `/repo/pulls/${pr}/merge`, { sha: head, merge_method: "squash", commit_title: title, commit_message: message });
    if (answer.status === 200 && answer.body.merged) return { sha: answer.body.sha };
    if (answer.status === 409) return "moved";
    if (answer.status === 405) return "refused";
    throw new Error(`merging answered ${answer.status}: ${answer.body?.message ?? ""}`);
  }

  /**
   * Marks a pull request for review: adds `label` to it and makes sure `approve` exists, so it can be picked in
   * GitHub's label menu. Both calls are allowed to fail (GitHub creates a missing label when it is added).
   *
   * @param pr the pull request
   * @param label the label that asks for review
   * @param approve the label that approves the merge
   */
  async markForReview(pr: number, label: string, approve: string): Promise<void> {
    await this.call("POST", "/repo/labels", { name: approve, color: "0e8a16", description: "Lets the feedback runner merge this pull request" });
    await this.call("POST", `/repo/issues/${pr}/labels`, { labels: [label] });
  }

  /** Takes a label off a pull request; a label it does not have is fine. */
  async removeLabel(pr: number, label: string): Promise<void> {
    await this.call("DELETE", `/repo/issues/${pr}/labels/${encodeURIComponent(label)}`);
  }

  /**
   * Where Tavi's review of a pull request stands.
   *
   * @param pr the pull request
   * @param approve the label that approves the merge
   * @returns `merged` with the merge commit when it was merged by hand, `closed` when closed without merging,
   *   `approved` when it is open and carries `approve`, otherwise `waiting`
   */
  async review(pr: number, approve: string): Promise<{ result: "merged"; sha: string } | { result: "closed" } | { result: "approved" } | { result: "waiting" }> {
    const answer = await this.call("GET", `/repo/pulls/${pr}`);
    if (answer.status !== 200) return { result: "waiting" };
    if (answer.body.merged) return { result: "merged", sha: answer.body.merge_commit_sha };
    if (answer.body.state === "closed") return { result: "closed" };
    const labels: { name: string }[] = answer.body.labels ?? [];
    return labels.some((l) => l.name === approve) ? { result: "approved" } : { result: "waiting" };
  }

  /** Whether main is behind nothing a branch has: `true` when the branch already contains main's head. */
  async containsMain(branch: string): Promise<boolean> {
    const answer = await this.call("GET", `/repo/compare/${branch}...main`);
    return answer.status === 200 && answer.body.ahead_by === 0;
  }

  /** The conclusion of the `Android release` run for a commit, or null while none has finished. */
  async release(sha: string, workflow = "android-release.yml"): Promise<"success" | "failure" | null> {
    const answer = await this.call("GET", `/repo/actions/workflows/${workflow}/runs?head_sha=${sha}&per_page=10`);
    if (answer.status !== 200) return null;
    const runs: any[] = answer.body.workflow_runs ?? [];
    const done = runs.filter((run) => run.status === "completed");
    if (done.some((run) => run.conclusion === "success")) return "success";
    if (done.length && runs.every((run) => run.status === "completed")) return "failure";
    return null;
  }
}
