/// <reference types="node" />
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

it("round.py passes its Python tests", () => {
  const result = spawnSync("python3", [fileURLToPath(new URL("./round_test.py", import.meta.url))], { encoding: "utf8" });
  expect(result.stderr).toMatch(/OK\s*$/);
});
