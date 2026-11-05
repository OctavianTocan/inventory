/// <reference types="node" />
import { DatabaseSync } from "node:sqlite";

/** A Durable Object `storage.sql` over node:sqlite, enough for the desk's queries. */
export function memorySql() {
  const db = new DatabaseSync(":memory:");
  return {
    exec(query: string, ...args: unknown[]) {
      const statement = db.prepare(query);
      const rows = /^\s*(SELECT|WITH)/i.test(query) ? (statement.all(...(args as any[])) as any[]) : (statement.run(...(args as any[])), []);
      return {
        toArray: () => rows.map((row) => ({ ...row })),
        one: () => {
          if (rows.length !== 1) throw new Error(`expected one row, got ${rows.length}`);
          return { ...rows[0] };
        },
      };
    },
  };
}
