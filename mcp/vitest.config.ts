import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// The OAuth library imports `cloudflare:workers` only to recognise WorkerEntrypoint handlers; the tests run in Node.
export default defineConfig({
  resolve: { alias: { "cloudflare:workers": fileURLToPath(new URL("./test/workers-stub.ts", import.meta.url)) } },
  test: { include: ["test/**/*.test.ts"], server: { deps: { inline: ["@cloudflare/workers-oauth-provider"] } } },
});
