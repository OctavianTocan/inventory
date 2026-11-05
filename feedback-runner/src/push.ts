/** One ref update from a `git push` request: old and new object ids and the ref name. */
export interface RefUpdate {
  old: string;
  new: string;
  ref: string;
}

/** What a push request asks for, or why it cannot be read. */
export type PushRequest =
  | { ok: true; updates: RefUpdate[] }
  | { ok: false; reason: string };

const ZERO = /^0+$/;
const OID = /^[0-9a-f]{40}([0-9a-f]{24})?$/;

/**
 * Reads the ref updates at the start of a `git-receive-pack` request body (smart HTTP, protocol v0/v1).
 *
 * The body is pkt-lines up to the first flush packet, then push options or the pack, which this ignores.
 * Leading `shallow` lines are skipped. A signed push (`push-cert`) or anything else unexpected is refused.
 *
 * @param body the request body, already decompressed if it was sent gzipped
 * @returns the ref updates, or the reason the body could not be read
 */
export function readPush(body: Uint8Array): PushRequest {
  const text = new TextDecoder("latin1");
  const updates: RefUpdate[] = [];
  let at = 0;
  for (;;) {
    if (at + 4 > body.length) return { ok: false, reason: "body ended before the ref list did" };
    const size = Number.parseInt(text.decode(body.subarray(at, at + 4)), 16);
    if (!Number.isInteger(size)) return { ok: false, reason: "bad pkt-line length" };
    if (size === 0) break;
    if (size < 5 || at + size > body.length) return { ok: false, reason: "bad pkt-line length" };
    let line = text.decode(body.subarray(at + 4, at + size));
    at += size;
    const nul = line.indexOf("\0");
    if (nul >= 0) line = line.slice(0, nul);
    line = line.replace(/\n$/, "");
    if (line.startsWith("shallow ")) continue;
    if (line.startsWith("push-cert")) return { ok: false, reason: "signed pushes are not accepted" };
    const [oldId, newId, ref, ...rest] = line.split(" ");
    if (rest.length > 0 || !OID.test(oldId ?? "") || !OID.test(newId ?? "") || !ref) {
      return { ok: false, reason: "unreadable ref line" };
    }
    updates.push({ old: oldId, new: newId, ref });
  }
  if (updates.length === 0) return { ok: false, reason: "no ref updates" };
  return { ok: true, updates };
}

/**
 * Decides whether a push may go through: every update must target `allowedRef` and none may delete it.
 *
 * @param push the parsed request from {@link readPush}
 * @param allowedRef the one ref this sandbox may write, such as `refs/heads/feedback/abc123def456`
 * @returns `null` when allowed, otherwise the sentence git shows the agent
 */
export function refusePush(push: PushRequest, allowedRef: string): string | null {
  if (!push.ok) return push.reason;
  for (const update of push.updates) {
    if (update.ref !== allowedRef) return `pushes may only update ${allowedRef}, not ${update.ref}`;
    if (ZERO.test(update.new)) return `${update.ref} may not be deleted`;
  }
  return null;
}
