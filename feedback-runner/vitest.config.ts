/// <reference types="node" />
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// The tests run in Node: Workers-only modules are replaced by small stand-ins, and the desk's SQLite by node:sqlite.
export default defineConfig({
  resolve: {
    alias: {
      "cloudflare:workers": fileURLToPath(new URL("./test/fixtures/workers.ts", import.meta.url)),
      "@cloudflare/sandbox": fileURLToPath(new URL("./test/fixtures/sandbox.ts", import.meta.url)),
    },
  },
});
