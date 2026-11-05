import type { FeedbackDesk } from "./desk";
import type { FeedbackJob } from "./job";
import type { FeedbackSandbox } from "./sandbox";

/** The runner's bindings, variables and secrets (wrangler.jsonc). */
export interface Env {
  FEEDBACK_DESK: DurableObjectNamespace<FeedbackDesk>;
  FEEDBACK_JOB: DurableObjectNamespace<FeedbackJob>;
  FEEDBACK_SANDBOX: DurableObjectNamespace<FeedbackSandbox>;
  /** ai-kit's AgentHome in the `mymind` Worker, one per AI login, for the sign-in checkout. */
  AGENT_HOME: DurableObjectNamespace;
  /** `owner/name` of the repository feedback changes. */
  REPO: string;
  /** Comma-separated `owner/name` repositories rounds may fetch (with history) but never push: Pully and ai-kit. */
  READ_REPOS?: string;
  /** The stored GitHub CLI sign-in (secret). */
  GITHUB_TOKEN: string;
  /** Comma-separated check names a change must pass; default `android,rust`. */
  REQUIRED_CHECKS?: string;
  /** The container instance type for rounds; default `standard-1`. */
  SANDBOX_INSTANCE?: string;
}
