/** The GitHub settings the spike reads from the Worker's environment. */
export interface GitHubEnv {
  /**
   * One GitHub CLI sign-in (device flow with gh's public client, scope `repo`), made once and reused for every job,
   * as Pully does. `local` means `wrangler dev` without GitHub.
   */
  GITHUB_TOKEN: string;
  REPO: string;
}

/** One GitHub API answer: status, parsed JSON (or text when it is not JSON) and the token's scopes. */
export interface GitHubAnswer {
  status: number;
  body: any;
  scopes: string | null;
}

/**
 * Calls the GitHub REST API with the stored sign-in.
 *
 * @param env the token and repository
 * @param method the HTTP method
 * @param path the API path, starting with `/`
 * @param body optional JSON body
 * @returns status, body and the `x-oauth-scopes` header
 */
export async function github(env: GitHubEnv, method: string, path: string, body?: unknown): Promise<GitHubAnswer> {
  const response = await fetch(`https://api.github.com${path}`, {
    method,
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${env.GITHUB_TOKEN}`,
      "user-agent": "thymind-feedback-spike",
      "x-github-api-version": "2022-11-28",
      ...(body === undefined ? {} : { "content-type": "application/json" }),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  let parsed: unknown = text;
  try {
    parsed = JSON.parse(text);
  } catch {}
  return { status: response.status, body: parsed, scopes: response.headers.get("x-oauth-scopes") };
}

/** Who the sign-in is and what it may do, so the report shows whether it lacks the `workflow` scope. */
export async function whoAmI(env: GitHubEnv): Promise<{ login: string; scopes: string | null }> {
  const user = await github(env, "GET", "/user");
  return { login: user.body?.login ?? `unknown (${user.status})`, scopes: user.scopes };
}
