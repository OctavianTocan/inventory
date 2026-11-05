// Stand-ins for `cloudflare:workers` in Node tests.
export class DurableObject<Env = unknown> {
  constructor(
    public ctx: any,
    public env: Env,
  ) {}
}

export class WorkerEntrypoint<Env = unknown, Props = unknown> {
  constructor(
    public ctx: { props: Props },
    public env: Env,
  ) {}
}
