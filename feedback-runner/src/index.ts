// The feedback runner (docs/feedback.md). It has no public route: only Inventory's Rust Worker reaches it, through the
// FEEDBACK_RUNNER service binding, after its sign-in gate and checks, with the sender's verified email in
// `x-feedback-email` and their AI login in `x-feedback-login`. The owner check for the record and the pause switch is the
// Rust Worker's too.
import type { Send } from "./desk";
import type { Env } from "./env";

export { FeedbackDesk } from "./desk";
export { FeedbackJob } from "./job";
export { FeedbackSandbox, Outbound } from "./sandbox";

const ID = /^[a-z0-9]{12}$/;
const EFFORTS = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

export default {
  async fetch(request, env): Promise<Response> {
    const url = new URL(request.url);
    const email = request.headers.get("x-feedback-email") ?? "";
    if (!email) return json({ error: "Not found" }, 404);
    const desk = env.FEEDBACK_DESK.getByName("main");

    if (url.pathname === "/feedback" && request.method === "POST") {
      const send = parseSend(await request.json().catch(() => null), request.headers);
      if (typeof send === "string") return json({ error: send }, 400);
      const answer = await desk.submit(send);
      return answer.ok ? json({ id: answer.id, state: answer.state }, 202) : json({ error: answer.error }, 403);
    }
    if (url.pathname === "/feedback" && request.method === "GET") {
      return json({ jobs: await desk.list() }, 200);
    }
    if (url.pathname === "/feedback/pause" && request.method === "POST") {
      const body = (await request.json().catch(() => null)) as { paused?: unknown } | null;
      if (typeof body?.paused !== "boolean") return json({ error: "Send {\"paused\": true} or false." }, 400);
      return json({ paused: await desk.setPaused(body.paused) }, 200);
    }
    const match = /^\/feedback\/([a-z0-9]+)$/.exec(url.pathname);
    if (match && request.method === "GET" && ID.test(match[1])) {
      const status = await env.FEEDBACK_JOB.getByName(match[1]).status(email);
      return status ? json(status, 200) : json({ error: "Not found" }, 404);
    }
    return json({ error: "Not found" }, 404);
  },
} satisfies ExportedHandler<Env>;

/**
 * Reads and checks a send's body and headers.
 *
 * @returns the send, or the sentence for a 400
 */
export function parseSend(body: unknown, headers: Headers): Send | string {
  const b = body as Record<string, unknown> | null;
  if (!b || typeof b !== "object") return "Send a JSON body.";
  const text = typeof b.text === "string" ? b.text.trim() : "";
  if (text.length < 1 || text.length > 4000) return "Write between 1 and 4000 characters.";
  if (b.engine !== "claude" && b.engine !== "codex") return "Pick Claude Code or Codex.";
  if (typeof b.model !== "string" || !b.model || b.model.length > 100) return "Pick a model.";
  if (typeof b.effort !== "string" || !EFFORTS.includes(b.effort)) return "Pick an effort.";
  if (typeof b.client_id !== "string" || !/^[\w-]{8,64}$/.test(b.client_id)) return "Missing client_id.";
  const login = headers.get("x-feedback-login") ?? "";
  if (!login) return "Missing the AI login.";
  return {
    email: headers.get("x-feedback-email") ?? "",
    login,
    name: (headers.get("x-feedback-name") ?? "").slice(0, 100) || "an Inventory user",
    text,
    engine: b.engine,
    model: b.model,
    effort: b.effort,
    client_id: b.client_id,
  };
}

function json(body: unknown, status: number): Response {
  return new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });
}
