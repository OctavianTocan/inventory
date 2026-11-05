// Stand-in for `@cloudflare/sandbox` in Node tests; no test reaches a container.
export class Files {
  constructor(_container: unknown) {}
}
