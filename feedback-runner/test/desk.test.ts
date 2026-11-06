import { describe, expect, it } from "vitest";

import { FeedbackDesk, type Send } from "../src/desk";
import { memorySql } from "./fixtures/sql";

/** A desk over an in-memory database, with job stubs that record what the desk asked of them. */
function desk() {
  const created: string[] = [];
  const woken: string[] = [];
  const env = {
    FEEDBACK_JOB: {
      getByName: (id: string) => ({
        create: async () => void created.push(id),
        wake: async () => void woken.push(id),
      }),
    },
  };
  const object = new FeedbackDesk({ storage: { sql: memorySql() } } as any, env as any);
  return { object, created, woken };
}

let next = 0;
function send(email = "ana@example.com", client_id = `client-${next++}-id`): Send {
  return { email, login: "vol-ana", name: "Ana", text: "Keep the share sheet open longer", engine: "claude", model: "opus", effort: "medium", client_id };
}

const finish = (object: FeedbackDesk, id: string, state = "shipped") =>
  object.update({ id, email: "", state, engine: "claude", model: "opus", effort: "medium", pr: 1, rounds: 1, reason: null, created_at: 0, finished_at: 1, ci_log: null });

describe("FeedbackDesk", () => {
  it("returns the same job for a repeated client_id before refusing a second active job", async () => {
    const { object, created } = desk();
    const first = await object.submit(send("ana@example.com", "same-client-id"));
    const again = await object.submit(send("ana@example.com", "same-client-id"));
    expect(first.ok && again.ok && again.id === first.id).toBe(true);
    expect(created).toHaveLength(1);
    expect(await object.submit(send())).toEqual({ ok: false, error: "You already have feedback in progress." });
  });

  it("lets a sender whose last job waits for Tavi's review send another", async () => {
    const { object } = desk();
    const first = await object.submit(send());
    if (!first.ok) throw new Error(first.error);
    object.update({ id: first.id, email: "", state: "review", engine: "claude", model: "opus", effort: "medium", pr: 1, rounds: 1, reason: null, created_at: 0, finished_at: null, ci_log: null });
    expect((await object.submit(send())).ok).toBe(true);
  });

  it("refuses a client_id someone else used", async () => {
    const { object } = desk();
    await object.submit(send("ana@example.com", "shared-client-id"));
    expect((await object.submit(send("bo@example.com", "shared-client-id"))).ok).toBe(false);
  });

  it("allows 5 sends per UTC day and a 6th the next day", async () => {
    const { object } = desk();
    const evening = Date.UTC(2026, 10, 5, 23, 0);
    for (let i = 0; i < 5; i++) {
      const answer = await object.submit(send(), evening);
      if (!answer.ok) throw new Error(answer.error);
      finish(object, answer.id);
    }
    expect(await object.submit(send(), evening + 30 * 60_000)).toEqual({ ok: false, error: "You've sent 5 today. Try again tomorrow." });
    expect((await object.submit(send(), Date.UTC(2026, 10, 6, 0, 1))).ok).toBe(true);
  });

  it("refuses sends while paused", async () => {
    const { object } = desk();
    await object.setPaused(true);
    expect(await object.submit(send())).toEqual({ ok: false, error: "Feedback is paused for now." });
  });

  it("hands out 3 round slots and wakes the longest-waiting job when one frees", async () => {
    const { object, woken } = desk();
    expect([await object.claimSlot("a"), await object.claimSlot("b"), await object.claimSlot("c")]).toEqual([true, true, true]);
    expect(await object.claimSlot("d")).toBe(false);
    expect(await object.claimSlot("e")).toBe(false);
    await object.releaseSlot("a");
    expect(woken).toEqual(["d"]);
    // e is not first in line, so it waits even though a slot is free.
    expect(await object.claimSlot("e")).toBe(false);
    expect(await object.claimSlot("d")).toBe(true);
  });

  it("gives no slot and no merge turn while paused, and wakes both when unpaused", async () => {
    const { object, woken } = desk();
    await object.enqueueMerge("m");
    await object.setPaused(true);
    expect(await object.claimSlot("a")).toBe(false);
    expect(object.mergeTurn("m")).toBe(false);
    await object.setPaused(false);
    expect(woken).toEqual(["a", "m"]);
    expect(object.mergeTurn("m")).toBe(true);
  });

  it("merges first in, first out", async () => {
    const { object, woken } = desk();
    await object.enqueueMerge("first");
    await object.enqueueMerge("second");
    expect([object.mergeTurn("first"), object.mergeTurn("second")]).toEqual([true, false]);
    await object.finishMerge("first");
    expect(woken).toEqual(["second"]);
    expect(object.mergeTurn("second")).toBe(true);
  });

  it("keeps Tavi's record of jobs, newest first", async () => {
    const { object } = desk();
    const answer = await object.submit(send("cy@example.com"), 1000);
    if (!answer.ok) throw new Error(answer.error);
    finish(object, answer.id, "failed");
    expect(object.list()[0]).toMatchObject({ id: answer.id, email: "cy@example.com", state: "failed", pr: 1 });
  });
});
