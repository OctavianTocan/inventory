// A port of ai-kit's `redact` (vendor/pulls.review/ai-kit/src/redact.rs) for text that leaves a feedback sandbox: the
// agent's summary and description, CI log tails and errors. ai-kit's contract `kit-secrets-never-leave` names this port.

/** Names whose values are redacted whatever their length (ai-kit `KNOWN_SECRETS`). */
const KNOWN_SECRETS = ["ANTHROPIC_API_KEY", "CLAUDE_CODE_OAUTH_TOKEN", "OPENAI_API_KEY", "GITHUB_TOKEN", "GH_TOKEN", "TYPESAFE_API_KEY"];

/** A name that says it holds a secret (ai-kit `secret_name`). */
function secretName(name: string): boolean {
  const upper = name.toUpperCase();
  return ["TOKEN", "KEY", "SECRET", "PASSWORD"].some((part) => upper.includes(part));
}

/** Every string of 16 or more characters inside a sign-in file, the values a token can hide in. */
export function signInStrings(files: Record<string, unknown>): string[] {
  const found = new Set<string>();
  const walk = (value: unknown) => {
    if (typeof value === "string") {
      if (value.length >= 16) found.add(value);
    } else if (Array.isArray(value)) {
      value.forEach(walk);
    } else if (value && typeof value === "object") {
      Object.values(value).forEach(walk);
    }
  };
  walk(files);
  return [...found];
}

/**
 * Removes secrets from text before it reaches a pull request, a notification or a log.
 *
 * @param text the text to clean
 * @param env the Worker's environment: known secrets and any value of a name with TOKEN, KEY, SECRET or PASSWORD
 * @param extra further exact values to remove, such as the strings of the round's sign-in files
 * @returns the text with each secret replaced by `[redacted]`, and anything shaped like a token removed too
 */
export function redact(text: string, env: object = {}, extra: string[] = []): string {
  let out = text;
  const values: string[] = [...extra];
  for (const [name, value] of Object.entries(env)) {
    if (typeof value !== "string" || !value) continue;
    if (KNOWN_SECRETS.includes(name) || (value.length >= 6 && secretName(name))) values.push(value);
  }
  for (const value of values.sort((a, b) => b.length - a.length)) out = out.split(value).join("[redacted]");
  return out
    .replace(/(sk-ant-[\w-]+|sk-[A-Za-z0-9_-]{20,}|gh[opsur]_\w+|github_pat_\w+|eyJ[\w-]+\.[\w-]+\.[\w-]+)/g, "[redacted]")
    .replace(/"(access_token|refresh_token|id_token|accessToken|refreshToken)"\s*:\s*"[^"]*"/g, '"$1":"[redacted]"');
}
