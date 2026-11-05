/**
 * Runs Inventory's MCP tools against a live library, signed in as the `TEST_USER_*` password account, through the real
 * MCP client and server (no OAuth: the tools get the app token directly). Read tools only unless `--write` is given.
 *
 *   TEST_USER_NAME=… TEST_USER_PASSWORD=… npx tsx eval/live.ts <tool> '<json args>' [<tool> '<json args>' …]
 *   TEST_USER_NAME=… TEST_USER_PASSWORD=… npx tsx eval/live.ts --eval eval/questions.xml
 *
 * `--eval` serves the tools on 127.0.0.1 for the run and asks Claude Code (`claude -p`, signed in on the machine)
 * each question with only those tools, then compares its answer with the expected one. The token it makes is signed
 * out again at the end.
 */
import { execFile } from "node:child_process";
import { readFileSync } from "node:fs";
import { createServer as createHttpServer } from "node:http";
import { promisify } from "node:util";

import { Client, StreamableHTTPClientTransport } from "@modelcontextprotocol/client";
import { createMcpHandler } from "@modelcontextprotocol/server";

import { Library } from "../src/library";
import { createServer } from "../src/tools";

const ORIGIN = process.env.THYMIND_ORIGIN ?? "https://mymind.octaviantocan.com";

async function signIn(): Promise<string> {
  const response = await fetch(`${ORIGIN}/app/v1/auth/password`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ username: process.env.TEST_USER_NAME, password: process.env.TEST_USER_PASSWORD, device: "MCP eval" }),
  });
  if (!response.ok) throw new Error(`sign-in failed: ${response.status}`);
  return ((await response.json()) as { token: string }).token;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const write = args.includes("--write");
  const token = await signIn();
  try {
    const library = new Library({ fetch: (url: string, init?: RequestInit) => fetch(url, init) } as unknown as Fetcher, ORIGIN, token);
    const handler = createMcpHandler(() => createServer(library, () => {}));
    const scopes = write ? ["thymind.read", "thymind.write"] : ["thymind.read"];
    const client = new Client({ name: "thymind-eval", version: "1.0.0" });
    await client.connect(new StreamableHTTPClientTransport(new URL("http://eval.local/mcp"), {
      fetch: (input, init) => handler.fetch(new Request(String(input instanceof Request ? input.url : input), init), { authInfo: { token: "eval", clientId: "eval", scopes } }),
    }));
    const evalAt = args.indexOf("--eval");
    if (evalAt >= 0) {
      await runEval(readFileSync(args[evalAt + 1]!, "utf8"), (request) => handler.fetch(request, { authInfo: { token: "eval", clientId: "eval", scopes } }));
      return;
    }
    const calls = args.filter((arg) => arg !== "--write");
    for (let i = 0; i < calls.length; i += 2) {
      const result = await client.callTool({ name: calls[i]!, arguments: JSON.parse(calls[i + 1] ?? "{}") });
      for (const part of result.content as Array<{ type: string; text?: string; mimeType?: string; data?: string }>) {
        console.log(part.type === "text" ? part.text : `[${part.type} ${part.mimeType} ${part.data?.length ?? 0} base64 chars]`);
      }
      if (result.isError) console.log("(error)");
      console.log("----");
    }
  } finally {
    await fetch(`${ORIGIN}/app/v1/auth/signout`, { method: "POST", headers: { authorization: `Bearer ${token}` } });
  }
}

const run = promisify(execFile);

/** Lowercased words only, so "Zig." and "zig" compare equal. */
function plain(text: string): string {
  return text.toLowerCase().replace(/[^\p{L}\p{N}.]+/gu, " ").replace(/\.(\s|$)/g, "$1").trim();
}

/** Serves the tools on a local port and asks Claude Code each question in `xml`, then prints the score. */
async function runEval(xml: string, answer: (request: Request) => Promise<Response>): Promise<void> {
  const pairs = [...xml.matchAll(/<question>([\s\S]*?)<\/question>\s*<answer>([\s\S]*?)<\/answer>/g)].map((match) => ({ question: match[1]!.trim(), expected: match[2]!.trim() }));
  const http = createHttpServer(async (incoming, outgoing) => {
    const chunks: Buffer[] = [];
    for await (const chunk of incoming) chunks.push(chunk as Buffer);
    const headers = new Headers();
    for (const [name, value] of Object.entries(incoming.headers)) if (typeof value === "string") headers.set(name, value);
    const body = ["GET", "HEAD"].includes(incoming.method ?? "GET") ? undefined : Buffer.concat(chunks);
    const response = await answer(new Request(`http://127.0.0.1${incoming.url}`, { method: incoming.method, headers, body }));
    outgoing.writeHead(response.status, Object.fromEntries(response.headers));
    outgoing.end(Buffer.from(await response.arrayBuffer()));
  });
  await new Promise<void>((resolve) => http.listen(0, "127.0.0.1", resolve));
  const port = (http.address() as { port: number }).port;
  const config = JSON.stringify({ mcpServers: { thymind: { type: "http", url: `http://127.0.0.1:${port}/mcp` } } });
  let right = 0;
  try {
    for (const [index, { question, expected }] of pairs.entries()) {
      const prompt = `${question}\n\nUse the thymind tools. Reply with only the answer, no other words.`;
      const { stdout } = await run("claude", ["-p", prompt, "--mcp-config", config, "--strict-mcp-config", "--allowedTools", "mcp__thymind__*"], { maxBuffer: 1 << 20, timeout: 300_000 });
      const got = stdout.trim();
      const ok = plain(got) === plain(expected);
      if (ok) right += 1;
      console.log(`${ok ? "PASS" : "FAIL"} ${index + 1}. expected ${JSON.stringify(expected)}, got ${JSON.stringify(got)}`);
    }
  } finally {
    http.close();
  }
  console.log(`${right}/${pairs.length} right`);
}

await main();
