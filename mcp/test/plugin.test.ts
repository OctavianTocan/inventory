import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import { PATHS } from "../src/signin";
import { READ_SCOPE, WRITE_SCOPE } from "../src/pages";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const ENDPOINT = "https://mymind.octaviantocan.com/mcp";

describe("the plugin packages and the skill", () => {
  const registered = new Set([...`${read("../src/tools.ts")}${read("../src/writes.ts")}`.matchAll(/registerTool\(\s*"(thymind_\w+)"/g)].map((match) => match[1]));

  it("names only tools the server has, and every one of them", () => {
    const skill = read("../plugin/skills/thymind/SKILL.md");
    const named = new Set([...skill.matchAll(/`(thymind_\w+)`/g)].map((match) => match[1]));
    expect(registered.size).toBe(14);
    expect([...named].sort()).toEqual([...registered].sort());
    expect(skill).toMatch(/^---\nname: thymind\ndescription: .+\n---\n/);
  });

  it("points ChatGPT and Claude Code at the server's own endpoints", () => {
    const chatgpt = JSON.parse(read("../plugin/mcp.json")).mcpServers.thymind;
    expect(chatgpt.url).toBe(ENDPOINT);
    const auth = chatgpt.extensions["com.openai"].auth;
    expect(auth.authorizationUrl).toBe(`https://mymind.octaviantocan.com${PATHS.authorize}`);
    expect(auth.tokenUrl).toBe(`https://mymind.octaviantocan.com${PATHS.token}`);
    expect(auth.defaultScopes).toEqual([READ_SCOPE, WRITE_SCOPE]);
    expect(JSON.parse(read("../plugin/.mcp.json")).mcpServers.thymind).toEqual({ type: "http", url: ENDPOINT });
    const versions = [JSON.parse(read("../plugin/plugin.json")).version, JSON.parse(read("../plugin/.claude-plugin/plugin.json")).version];
    expect(versions[0]).toBe(versions[1]);
  });
});
