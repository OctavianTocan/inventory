/// <reference types="node" />
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { ownEntry, topVersion, withEntry } from "../src/changelog";

const SCRIPT = fileURLToPath(new URL("../../deploy/android-release.py", import.meta.url));

/** Parses a CHANGELOG with `history()` from deploy/android-release.py, the release gate's own reader. */
function history(text: string): { versionName: string; versionCode: number }[] {
  const code = `import importlib.util, json, sys
spec = importlib.util.spec_from_file_location("release", sys.argv[1])
release = importlib.util.module_from_spec(spec); spec.loader.exec_module(release)
print(json.dumps(release.history(sys.stdin.read())))`;
  const result = spawnSync("python3", ["-c", code, SCRIPT], { input: text, encoding: "utf8" });
  if (result.status !== 0) throw new Error(result.stderr);
  return JSON.parse(result.stdout);
}

const STARTED = `# Changelog

## 0.6.53 (64), 2026-11-05
Search sits near your thumb.

### Added
- Opening search shows your last searches.

## 0.6.52 (63), 2026-11-05
- Swipe between pages.
`;

const BRANCH = STARTED.replace("## 0.6.53", "## 0.6.54 (65), 2026-11-06\n- The share sheet stays open a second longer.\n\n## 0.6.53");
const MAIN = STARTED.replace("## 0.6.53", "## 0.6.55 (66), 2026-11-06\n- Someone else's change.\n\n## 0.6.54 (65), 2026-11-06\n- Another change.\n\n## 0.6.53");

describe("CHANGELOG entries", () => {
  it("reads the top version as the release gate does", () => {
    expect(topVersion(BRANCH)).toEqual({ name: "0.6.54", code: 65 });
    expect(history(BRANCH)[0]).toMatchObject({ versionName: "0.6.54", versionCode: 65 });
  });

  it("finds the branch's own entry and nothing when it has none", () => {
    expect(ownEntry(BRANCH, STARTED)).toEqual(["## 0.6.54 (65), 2026-11-06", "- The share sheet stays open a second longer.", ""]);
    expect(ownEntry(STARTED, STARTED)).toBeNull();
  });

  it("puts the entry back above main's newest, renumbered, and the gate still reads it", () => {
    const next = withEntry(MAIN, ownEntry(BRANCH, STARTED)!);
    expect(next.version).toEqual({ name: "0.6.56", code: 67 });
    const parsed = history(next.text);
    expect(parsed.map((entry) => entry.versionCode)).toEqual([67, 66, 65, 64, 63]);
    expect(next.text).toContain("## 0.6.56 (67), 2026-11-06\n- The share sheet stays open a second longer.\n\n## 0.6.55 (66)");
  });

  it("renumbers against the real CHANGELOG", async () => {
    const { readFileSync } = await import("node:fs");
    const real = readFileSync(fileURLToPath(new URL("../../android/CHANGELOG.md", import.meta.url)), "utf8");
    const top = topVersion(real)!;
    const next = withEntry(real, ["## 9.9.9 (9999), 2026-11-06", "- A change from feedback."]);
    expect(next.version.code).toBe(top.code + 1);
    expect(history(next.text)[0]).toMatchObject({ versionCode: top.code + 1 });
  });
});
