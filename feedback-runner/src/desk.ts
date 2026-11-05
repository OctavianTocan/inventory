// FeedbackDesk: the one index and queue (docs/feedback.md). It answers sends (dedup by client_id, the pause switch,
// one active job per person, 5 a day), hands out the 3 round slots, gives the merge turn first in first out, and keeps
// Tavi's record of the last 100 jobs.
import { DurableObject } from "cloudflare:workers";

import type { Env } from "./env";
import type { JobRow } from "./job";
import { FINAL, type Engine, type State } from "./states";

/** At most this many rounds run at once, over everyone. */
export const ROUND_SLOTS = 3;
/** At most this many sends per person per UTC day. */
export const DAILY_LIMIT = 5;

/** A send as the phone-facing route forwards it. */
export interface Send {
  email: string;
  login: string;
  name: string;
  text: string;
  engine: Engine;
  model: string;
  effort: string;
  client_id: string;
}

/** The desk's answer to a send: the job, or a refusal sentence. */
export type SendAnswer = { ok: true; id: string; state: State } | { ok: false; error: string };

export class FeedbackDesk extends DurableObject<Env> {
  #sql = this.ctx.storage.sql;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    this.#sql.exec(`CREATE TABLE IF NOT EXISTS jobs (
      id TEXT PRIMARY KEY, client_id TEXT UNIQUE, email TEXT, state TEXT, created_at INTEGER, finished_at INTEGER,
      engine TEXT, model TEXT, effort TEXT, pr INTEGER, rounds INTEGER, reason TEXT, ci_log TEXT)`);
    this.#sql.exec("CREATE TABLE IF NOT EXISTS running (id TEXT PRIMARY KEY)");
    this.#sql.exec("CREATE TABLE IF NOT EXISTS waiting (position INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE)");
    this.#sql.exec("CREATE TABLE IF NOT EXISTS queue (position INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE)");
    this.#sql.exec("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT)");
  }

  /**
   * Takes a send. A repeated `client_id` returns its job; then the pause switch, an active job and the daily limit refuse.
   *
   * @param send the checked send from the phone-facing route
   * @param now the current time, for the UTC day
   * @returns the job's id and state, or the refusal sentence
   */
  async submit(send: Send, now = Date.now()): Promise<SendAnswer> {
    const same = this.#sql.exec<{ id: string; state: State; email: string }>("SELECT id, state, email FROM jobs WHERE client_id = ?", send.client_id).toArray()[0];
    if (same) return same.email === send.email ? { ok: true, id: same.id, state: same.state } : { ok: false, error: "That send was already used." };
    if (this.paused()) return { ok: false, error: "Feedback is paused for now." };
    const finals = [...FINAL].map((state) => `'${state}'`).join(",");
    const active = this.#sql.exec(`SELECT COUNT(*) AS n FROM jobs WHERE email = ? AND state NOT IN (${finals})`, send.email).one().n as number;
    if (active > 0) return { ok: false, error: "You already have feedback in progress." };
    const day = new Date(now);
    const midnight = Date.UTC(day.getUTCFullYear(), day.getUTCMonth(), day.getUTCDate());
    const today = this.#sql.exec("SELECT COUNT(*) AS n FROM jobs WHERE email = ? AND created_at >= ?", send.email, midnight).one().n as number;
    if (today >= DAILY_LIMIT) return { ok: false, error: `You've sent ${DAILY_LIMIT} today. Try again tomorrow.` };

    const id = newId();
    this.#sql.exec(
      "INSERT INTO jobs (id, client_id, email, state, created_at, engine, model, effort, rounds) VALUES (?, ?, ?, 'queued', ?, ?, ?, ?, 0)",
      id, send.client_id, send.email, now, send.engine, send.model, send.effort,
    );
    await this.env.FEEDBACK_JOB.getByName(id).create({
      id,
      email: send.email,
      login: send.login,
      name: send.name,
      text: send.text,
      engine: send.engine,
      model: send.model,
      effort: send.effort,
      createdAt: now,
    });
    return { ok: true, id, state: "queued" };
  }

  /** Records a job's latest row. */
  update(row: JobRow): void {
    this.#sql.exec(
      "UPDATE jobs SET state = ?, finished_at = ?, pr = ?, rounds = ?, reason = ?, ci_log = ? WHERE id = ?",
      row.state, row.finished_at, row.pr, row.rounds, row.reason, row.ci_log, row.id,
    );
    if (FINAL.has(row.state as State)) {
      this.#sql.exec("DELETE FROM waiting WHERE id = ?", row.id);
      this.#sql.exec("DELETE FROM queue WHERE id = ?", row.id);
    }
  }

  /** Gives `id` a round slot unless paused or all are taken; a refused job waits and is woken when one frees. */
  async claimSlot(id: string): Promise<boolean> {
    if (this.#sql.exec("SELECT 1 FROM running WHERE id = ?", id).toArray().length) return true;
    const running = this.#sql.exec("SELECT COUNT(*) AS n FROM running").one().n as number;
    const first = this.#sql.exec<{ id: string }>("SELECT id FROM waiting ORDER BY position LIMIT 1").toArray()[0];
    if (this.paused() || running >= ROUND_SLOTS || (first && first.id !== id)) {
      this.#sql.exec("INSERT OR IGNORE INTO waiting (id) VALUES (?)", id);
      return false;
    }
    this.#sql.exec("DELETE FROM waiting WHERE id = ?", id);
    this.#sql.exec("INSERT INTO running (id) VALUES (?)", id);
    return true;
  }

  /** Frees `id`'s round slot and wakes the job that has waited longest. */
  async releaseSlot(id: string): Promise<void> {
    this.#sql.exec("DELETE FROM running WHERE id = ?", id);
    await this.#wakeWaiting();
  }

  /** Puts `id` at the back of the merge queue. */
  async enqueueMerge(id: string): Promise<void> {
    this.#sql.exec("INSERT OR IGNORE INTO queue (id) VALUES (?)", id);
  }

  /** Whether `id` may run its merge steps now: it is first in the queue and feedback is not paused. */
  mergeTurn(id: string): boolean {
    if (this.paused()) return false;
    const first = this.#sql.exec<{ id: string }>("SELECT id FROM queue ORDER BY position LIMIT 1").toArray()[0];
    return first?.id === id;
  }

  /** Takes `id` out of the merge queue and wakes the next job in it. */
  async finishMerge(id: string): Promise<void> {
    this.#sql.exec("DELETE FROM queue WHERE id = ?", id);
    const next = this.#sql.exec<{ id: string }>("SELECT id FROM queue ORDER BY position LIMIT 1").toArray()[0];
    if (next) await this.env.FEEDBACK_JOB.getByName(next.id).wake();
  }

  /** Whether the kill switch is on. */
  paused(): boolean {
    return this.#sql.exec<{ value: string }>("SELECT value FROM settings WHERE key = 'paused'").toArray()[0]?.value === "1";
  }

  /** Turns the kill switch on or off; turning it off wakes waiting jobs and the merge queue. */
  async setPaused(paused: boolean): Promise<boolean> {
    this.#sql.exec("INSERT OR REPLACE INTO settings (key, value) VALUES ('paused', ?)", paused ? "1" : "0");
    if (!paused) {
      await this.#wakeWaiting();
      const next = this.#sql.exec<{ id: string }>("SELECT id FROM queue ORDER BY position LIMIT 1").toArray()[0];
      if (next) await this.env.FEEDBACK_JOB.getByName(next.id).wake();
    }
    return paused;
  }

  /** Tavi's record: the last 100 jobs, newest first. */
  list(): JobRow[] {
    return this.#sql.exec<JobRow & Record<string, SqlStorageValue>>(
      "SELECT id, email, state, engine, model, effort, pr, rounds, reason, created_at, finished_at, ci_log FROM jobs ORDER BY created_at DESC LIMIT 100",
    ).toArray();
  }

  async #wakeWaiting(): Promise<void> {
    if (this.paused()) return;
    const running = this.#sql.exec("SELECT COUNT(*) AS n FROM running").one().n as number;
    if (running >= ROUND_SLOTS) return;
    const first = this.#sql.exec<{ id: string }>("SELECT id FROM waiting ORDER BY position LIMIT 1").toArray()[0];
    if (first) await this.env.FEEDBACK_JOB.getByName(first.id).wake();
  }
}

/** A new feedback id: 12 lowercase letters and digits (it names the branch `feedback/<id>`). */
export function newId(): string {
  const alphabet = "abcdefghijklmnopqrstuvwxyz0123456789";
  const bytes = crypto.getRandomValues(new Uint8Array(12));
  return [...bytes].map((byte) => alphabet[byte % alphabet.length]).join("");
}
