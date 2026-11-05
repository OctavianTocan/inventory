import { describe, expect, it } from "vitest";

import { decide, type Policy } from "../src/outbound";

const ENV = { GITHUB_TOKEN: "gho_secretsecretsecret", REPO: "OctavianTocan/mymind-reconstruction" };
const POLICY: Policy = { engine: "claude", ownRef: "refs/heads/feedback/abc123def456" };
const Z = "0".repeat(40);
const A = "ac6a8bf1e27bc5ebbe1cc1e60b16ebe52002bcc8";

function pushBody(...lines: string[]): Uint8Array {
  const body = lines.map((line) => (line.length + 4).toString(16).padStart(4, "0") + line).join("") + "0000PACK";
  return new TextEncoder().encode(body);
}

/** A sender that records what reached it and answers with a header that must not reach the sandbox. */
function recorder() {
  const sent: Request[] = [];
  const send = async (request: Request) => {
    sent.push(request);
    return new Response("ok", { status: 200, headers: { "www-authenticate": `Basic ${ENV.GITHUB_TOKEN}`, "x-kept": "1" } });
  };
  return { sent, send };
}

const receivePack = (body: Uint8Array) =>
  new Request(`https://github.com/${ENV.REPO}.git/git-receive-pack`, { method: "POST", body });

describe("outbound handler", () => {
  it("adds the sign-in to a push to the round's own branch and keeps it out of the answer", async () => {
    const { sent, send } = recorder();
    const result = await decide(receivePack(pushBody(`${Z} ${A} ${POLICY.ownRef}\0report-status`)), POLICY, ENV, send);
    expect(result.decision).toBe("push to own branch");
    expect(sent[0].headers.get("authorization")).toBe(`Basic ${btoa(`x-access-token:${ENV.GITHUB_TOKEN}`)}`);
    expect(result.response.headers.get("www-authenticate")).toBeNull();
    expect(result.response.headers.get("x-kept")).toBe("1");
  });

  it("refuses pushes to main, to another feedback branch, and deletes", async () => {
    for (const line of [`${Z} ${A} refs/heads/main`, `${Z} ${A} refs/heads/feedback/zzz999zzz999`, `${A} ${Z} ${POLICY.ownRef}`]) {
      const { sent, send } = recorder();
      const result = await decide(receivePack(pushBody(`${line}\0report-status`)), POLICY, ENV, send);
      expect(result.response.status).toBe(403);
      expect(sent).toHaveLength(0);
    }
  });

  it("allows fetches of this repository only, with the sign-in added", async () => {
    const { sent, send } = recorder();
    const ok = await decide(new Request(`https://github.com/${ENV.REPO}/info/refs?service=git-upload-pack`), POLICY, ENV, send);
    expect(ok.response.status).toBe(200);
    expect(sent[0].headers.get("authorization")).toContain("Basic ");
    const other = await decide(new Request("https://github.com/someone/else/info/refs?service=git-upload-pack"), POLICY, ENV, send);
    expect(other.response.status).toBe(403);
    const api = await decide(new Request(`https://github.com/${ENV.REPO}/settings`), POLICY, ENV, send);
    expect(api.response.status).toBe(403);
  });

  it("passes the chosen engine's hosts and package downloads, and refuses the rest", async () => {
    const { send } = recorder();
    const status = async (url: string, method = "GET", policy = POLICY) => (await decide(new Request(url, { method }), policy, ENV, send)).response.status;
    expect(await status("https://api.anthropic.com/v1/messages", "POST")).toBe(200);
    expect(await status("https://chatgpt.com/backend-api/codex", "POST")).toBe(403);
    expect(await status("https://chatgpt.com/backend-api/codex", "POST", { ...POLICY, engine: "codex" })).toBe(200);
    expect(await status("https://registry.npmjs.org/left-pad")).toBe(200);
    expect(await status("https://registry.npmjs.org/-/v1/login", "PUT")).toBe(403);
    expect(await status("https://mcp-proxy.anthropic.com/v1", "POST")).toBe(403);
    expect(await status("https://api.github.com/user")).toBe(403);
    expect(await status("http://api.anthropic.com/")).toBe(403);
  });

  it("never sends the token in a refusal", async () => {
    const { send } = recorder();
    const result = await decide(new Request("https://evil.example/"), POLICY, ENV, send);
    expect(await result.response.text()).not.toContain(ENV.GITHUB_TOKEN);
  });
});
