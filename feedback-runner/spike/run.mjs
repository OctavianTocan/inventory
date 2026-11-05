#!/usr/bin/env node
// Runs the feedback spike once on your own Cloudflare account with one reusable GitHub CLI sign-in, then deletes the Worker.
//
//   cd feedback-runner/spike && npm ci && npm run spike
//
// Needs Docker running and `npx wrangler login` done once. The first run signs in to GitHub the way `gh auth login`
// does (device flow, GitHub CLI's public client, scope `repo` without `workflow`) and keeps that sign-in in
// ~/.config/thymind/feedback-github.json for later runs. It reads your Claude Code and Codex sign-ins from this
// computer, sends them only to the sandbox it just deployed, and writes spike-report.json (no secrets in it).
// Options: --engines claude,codex  --claude-model sonnet  --codex-model <name>  --effort low  --no-q4  --keep
//          --down (only delete the Worker)
import { execFileSync, spawn } from "node:child_process";
import { randomBytes } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";

// The GitHub CLI's public OAuth client, as Pully uses it (backend/src/personal_auth/provider.rs).
const GH_CLIENT_ID = "178c6fc778ccc68e1d6a";
const GITHUB_LOGIN = join(homedir(), ".config/thymind/feedback-github.json");

const args = parseArgs(process.argv.slice(2));
const here = new URL(".", import.meta.url).pathname;

if (args.down) {
  wrangler(["delete", "--force"]);
  process.exit(0);
}

const gitHubToken = await gitHubSignIn();
const engines = (args.engines ?? "claude,codex").split(",").filter(Boolean);

const files = {};
if (engines.includes("claude")) Object.assign(files, claudeLogin());
if (engines.includes("codex")) Object.assign(files, codexLogin());

const token = randomBytes(24).toString("hex");

console.log("\n1/4 Deploying the spike Worker and building its image (a few minutes the first time)...");
const deployOutput = await wrangler(["deploy"], true);
const url = deployOutput.match(/https:\/\/feedback-spike\.[\w.-]+\.workers\.dev/)?.[0];
if (!url) fail("Could not find the Worker's workers.dev address in wrangler's output.");

console.log("\n2/4 Setting the spike's secrets...");
const secrets = join(tmpdir(), `feedback-spike-${process.pid}.json`);
writeFileSync(secrets, JSON.stringify({ SPIKE_TOKEN: token, GITHUB_TOKEN: gitHubToken }), { mode: 0o600 });
try {
  wrangler(["secret", "bulk", secrets]);
} finally {
  rmSync(secrets, { force: true });
}

console.log(`\n3/4 Running the spike at ${url} (5-15 minutes)...`);
const lines = await run(url, token, {
  files,
  engines,
  models: { claude: args["claude-model"], codex: args["codex-model"] },
  effort: args.effort ?? "low",
  q4: !args["no-q4"],
});
writeFileSync(join(here, "spike-report.json"), JSON.stringify(lines.filter((l) => !l.heartbeat).map(({ refreshed, ...rest }) => rest), null, 2));
const refreshed = lines.find((l) => l.done)?.refreshed ?? {};
if (Object.keys(refreshed).length > 0) {
  writeFileSync(join(here, "spike-refreshed-logins.json"), JSON.stringify(refreshed, null, 2), { mode: 0o600 });
  console.log(
    "\nThe CLIs refreshed these sign-in files inside the sandbox: " +
      Object.keys(refreshed).join(", ") +
      ".\nIf Claude Code or Codex on this computer now says you are signed out, sign in again (the sandbox's copies are in spike-refreshed-logins.json; delete that file afterwards).",
  );
}

if (args.keep) {
  console.log("\n4/4 Keeping the Worker (--keep). Delete it later with: npm run spike:down");
} else {
  console.log("\n4/4 Deleting the spike Worker...");
  wrangler(["delete", "--force"]);
}
console.log("\nDone. Send spike-report.json to the thread; it holds host names, timings and outcomes, never sign-ins or keys.");

/**
 * The stored GitHub sign-in, or a new one through GitHub's device flow with the GitHub CLI's client (what
 * `gh auth login` does). Scope `repo` only: without `workflow`, GitHub itself refuses pushes that change workflows.
 */
async function gitHubSignIn() {
  if (existsSync(GITHUB_LOGIN)) {
    const saved = JSON.parse(readFileSync(GITHUB_LOGIN, "utf8"));
    const check = await fetch("https://api.github.com/user", { headers: { authorization: `Bearer ${saved.token}`, "user-agent": "thymind-feedback-spike" } });
    if (check.ok) {
      console.log(`Using the saved GitHub sign-in (${(await check.json()).login}, scopes: ${check.headers.get("x-oauth-scopes")}).`);
      return saved.token;
    }
    console.log(`The saved GitHub sign-in no longer works (${check.status}); signing in again.`);
  }
  const form = (body) => ({ method: "POST", headers: { accept: "application/json", "content-type": "application/x-www-form-urlencoded" }, body: new URLSearchParams(body) });
  const code = await (await fetch("https://github.com/login/device/code", form({ client_id: GH_CLIENT_ID, scope: "repo" }))).json();
  if (!code.device_code) fail(`GitHub did not start a sign-in: ${JSON.stringify(code)}`);
  console.log(`\nSign in to GitHub once: open ${code.verification_uri} and enter ${code.user_code} (it says "GitHub CLI").`);
  let wait = (code.interval ?? 5) * 1000;
  for (;;) {
    await new Promise((resolve) => setTimeout(resolve, wait));
    const answer = await (await fetch("https://github.com/login/oauth/access_token", form({ client_id: GH_CLIENT_ID, device_code: code.device_code, grant_type: "urn:ietf:params:oauth:grant-type:device_code" }))).json();
    if (answer.access_token) {
      mkdirSync(dirname(GITHUB_LOGIN), { recursive: true });
      writeFileSync(GITHUB_LOGIN, JSON.stringify({ token: answer.access_token, scope: answer.scope }), { mode: 0o600 });
      console.log(`Signed in (scopes: ${answer.scope}). Saved for later runs in ${GITHUB_LOGIN}.`);
      return answer.access_token;
    }
    if (answer.error === "slow_down") wait += 5000;
    else if (answer.error !== "authorization_pending") fail(`GitHub sign-in stopped: ${answer.error}`);
  }
}

/** Posts the run and prints each step as it arrives. Retries while a fresh deploy is still starting. */
async function run(url, token, body) {
  for (let attempt = 1; ; attempt++) {
    const response = await fetch(`${url}/run`, {
      method: "POST",
      headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (response.ok) return printStream(response);
    if (attempt >= 6) fail(`The spike answered ${response.status}: ${(await response.text()).slice(0, 300)}`);
    console.log(`  not ready yet (${response.status}), trying again in 20 s`);
    await new Promise((resolve) => setTimeout(resolve, 20_000));
  }
}

async function printStream(response) {
  const lines = [];
  let buffered = "";
  const decoder = new TextDecoder();
  for await (const chunk of response.body) {
    buffered += decoder.decode(chunk, { stream: true });
    const parts = buffered.split("\n");
    buffered = parts.pop();
    for (const part of parts) {
      if (!part.trim()) continue;
      const line = JSON.parse(part);
      lines.push(line);
      if (line.heartbeat) continue;
      if (line.done) {
        console.log(`  finished in ${line.seconds} s`);
        for (const host of line.hosts) console.log(`  ${host.phase}/${host.engine ?? "-"} ${host.host} x${host.count} ${host.decision}`);
        continue;
      }
      const mark = line.ok === false ? "FAIL" : " ok ";
      console.log(`  ${mark} ${line.step} (${Math.round((line.ms ?? 0) / 1000)} s)${line.ok === false ? `: ${line.error ?? line.output ?? line.outcome ?? line.stderr ?? line.message ?? ""}` : ""}`);
    }
  }
  return lines;
}

/** Claude Code's sign-in: the file on Linux, the login keychain on macOS. Keeps only what ai-kit keeps. */
function claudeLogin() {
  const path = join(homedir(), ".claude/.credentials.json");
  let raw = existsSync(path) ? readFileSync(path, "utf8") : undefined;
  if (!raw && process.platform === "darwin") {
    try {
      raw = execFileSync("security", ["find-generic-password", "-s", "Claude Code-credentials", "-w"], { encoding: "utf8" });
    } catch {}
  }
  const oauth = raw ? JSON.parse(raw).claudeAiOauth : undefined;
  if (!oauth) fail("No Claude Code subscription sign-in found on this computer. Run `claude` and sign in with your Claude account, or pass --engines codex.");
  if (oauth.expiresAt && oauth.expiresAt - Date.now() < 60 * 60_000) {
    fail("Your Claude Code sign-in refreshes within the hour. Run `claude -p hi` once so it refreshes here first, then run the spike again.");
  }
  const result = { ".claude/.credentials.json": { claudeAiOauth: oauth } };
  try {
    const account = JSON.parse(readFileSync(join(homedir(), ".claude.json"), "utf8")).oauthAccount;
    if (account) result[".claude.json"] = { oauthAccount: account };
  } catch {}
  return result;
}

/** Codex's ChatGPT sign-in from ~/.codex/auth.json (or $CODEX_HOME). Keeps only what ai-kit keeps. */
function codexLogin() {
  const path = join(process.env.CODEX_HOME ?? join(homedir(), ".codex"), "auth.json");
  const auth = existsSync(path) ? JSON.parse(readFileSync(path, "utf8")) : undefined;
  if (!auth?.tokens) fail("No Codex ChatGPT sign-in found in ~/.codex/auth.json. Run `codex login`, or pass --engines claude.");
  const kept = { tokens: auth.tokens };
  if (auth.auth_mode) kept.auth_mode = auth.auth_mode;
  if (auth.last_refresh) kept.last_refresh = auth.last_refresh;
  return { ".codex/auth.json": kept };
}

/** Runs wrangler from this folder; with `capture`, also returns its output. */
function wrangler(argv, capture = false) {
  if (!capture) {
    execFileSync("npx", ["wrangler", ...argv], { cwd: here, stdio: "inherit" });
    return "";
  }
  const child = spawn("npx", ["wrangler", ...argv], { cwd: here, stdio: ["inherit", "pipe", "inherit"] });
  let output = "";
  child.stdout.on("data", (chunk) => {
    output += chunk;
    process.stdout.write(chunk);
  });
  return new Promise((resolve, reject) =>
    child.on("close", (code) => (code === 0 ? resolve(output) : reject(new Error(`wrangler ${argv[0]} exited ${code}`)))),
  );
}

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    const name = argv[i].replace(/^--/, "");
    if (argv[i + 1] && !argv[i + 1].startsWith("--")) out[name] = argv[++i];
    else out[name] = true;
  }
  return out;
}

function fail(message) {
  console.error(`\n${message}`);
  process.exit(1);
}
