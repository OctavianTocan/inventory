// Every HTTP and HTTPS request leaving a feedback sandbox goes through `decide` (docs/feedback.md, Outbound handler).
// It adds the GitHub sign-in to git traffic for the one repository, lets pushes reach only the round's own branch, lets
// the read-only repositories (`READ_REPOS`: Pully and ai-kit) be fetched but never pushed, and passes the chosen engine's
// hosts and package downloads. Everything else is refused. Host names are logged; paths,
// headers and bodies never are.
import type { Fetcher, GitHubEnv } from "./github";
import { readPush, refusePush } from "./push";
import type { Engine } from "./states";

/** What the handler needs to decide a request, kept by the round's sandbox. */
export interface Policy {
  engine: Engine;
  /** `refs/heads/feedback/<id>`: the only ref a push may update. */
  ownRef: string;
}

/** The engine hosts the spike run showed each CLI needs (docs/feedback.md, Q1a), plus their sign-in refresh hosts. */
export const ENGINE_HOSTS: Record<Engine, string[]> = {
  claude: ["api.anthropic.com", "claude.ai", "console.anthropic.com", "platform.claude.com"],
  codex: ["chatgpt.com", "auth.openai.com", "api.openai.com"],
};

/** Package downloads the agent may make while it builds, GET only. */
export const PACKAGE_HOSTS = ["index.crates.io", "static.crates.io", "registry.npmjs.org"];

/** A decided request: the answer for the sandbox and a short decision for the log. */
export interface Decision {
  response: Response;
  decision: string;
}

/**
 * Decides one request from the sandbox.
 *
 * @param request the request as it left the sandbox
 * @param policy the round's engine and own branch
 * @param env the stored GitHub sign-in and repository (`GITHUB_TOKEN` "local" adds none, for `wrangler dev`)
 * @param send how allowed requests are sent on
 * @returns the answer to give the sandbox, and the decision
 */
export async function decide(request: Request, policy: Policy, env: GitHubEnv & { READ_REPOS?: string }, send: Fetcher = (r) => fetch(r)): Promise<Decision> {
  const url = new URL(request.url);
  if (url.protocol !== "https:") return refuse("plain HTTP is not allowed");
  if (url.hostname === "github.com") return git(request, url, policy, env, send);
  if (ENGINE_HOSTS[policy.engine].includes(url.hostname)) return { response: await send(request), decision: "engine host" };
  if (PACKAGE_HOSTS.includes(url.hostname) && (request.method === "GET" || request.method === "HEAD")) {
    return { response: await send(request), decision: "package host" };
  }
  return refuse(`${url.hostname} is not reachable from this sandbox`);
}

/** `owner/name` of each repository rounds may fetch but never push (`READ_REPOS`, comma-separated). */
export function readRepos(value: string | undefined): string[] {
  return (value ?? "").split(",").map((name) => name.trim()).filter(Boolean);
}

/** The git path after `/<repo>` or `/<repo>.git`, or null when the path is not under that repository. */
function under(pathname: string, repo: string): string | null {
  const prefix = `/${repo}`.toLowerCase();
  const path = pathname.toLowerCase();
  if (path.startsWith(`${prefix}.git/`)) return pathname.slice(prefix.length + 4);
  if (path.startsWith(`${prefix}/`)) return pathname.slice(prefix.length);
  return null;
}

/** Git smart HTTP: the one repository may be fetched and pushed to the own branch; read-only ones only fetched. */
async function git(request: Request, url: URL, policy: Policy, env: GitHubEnv & { READ_REPOS?: string }, send: Fetcher): Promise<Decision> {
  const own = under(url.pathname, env.REPO);
  const readOnly = own === null ? readRepos(env.READ_REPOS).map((repo) => under(url.pathname, repo)).find((rest) => rest !== null) ?? null : null;
  const rest = own ?? readOnly;
  const service = url.searchParams.get("service");
  const fetches =
    (rest === "/info/refs" && request.method === "GET" && service === "git-upload-pack") ||
    (rest === "/git-upload-pack" && request.method === "POST");
  const pushes =
    own !== null &&
    ((rest === "/info/refs" && request.method === "GET" && service === "git-receive-pack") ||
      (rest === "/git-receive-pack" && request.method === "POST"));
  if (!fetches && !pushes) return refuse("only git fetch of the allowed repositories and push to this one are allowed");

  const headers = new Headers(request.headers);
  headers.delete("authorization");
  if (env.GITHUB_TOKEN !== "local") headers.set("authorization", `Basic ${btoa(`x-access-token:${env.GITHUB_TOKEN}`)}`);
  if (rest !== "/git-receive-pack") {
    const response = await send(new Request(request, { headers, redirect: "manual" }));
    return { response: scrub(response), decision: "git" };
  }
  const body = new Uint8Array(await request.arrayBuffer());
  const plain = request.headers.get("content-encoding") === "gzip" ? await gunzip(body) : body;
  const reason = refusePush(readPush(plain), policy.ownRef);
  if (reason) return refuse(`push refused: ${reason}`);
  const response = await send(new Request(request.url, { method: "POST", headers, body, redirect: "manual" }));
  return { response: scrub(response), decision: "push to own branch" };
}

/** A GitHub answer without headers that could carry the sign-in back into the sandbox. */
function scrub(response: Response): Response {
  const headers = new Headers(response.headers);
  for (const name of ["authorization", "set-cookie", "www-authenticate", "location"]) headers.delete(name);
  return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
}

function refuse(reason: string): Decision {
  return { response: new Response(`${reason}\n`, { status: 403 }), decision: `refused: ${reason}` };
}

async function gunzip(bytes: Uint8Array): Promise<Uint8Array> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}
