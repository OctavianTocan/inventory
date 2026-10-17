import { DurableObject } from 'cloudflare:workers';
import { createAuth } from '../lib/auth.mjs';
import { libraryOf, listUsers, setLibrary } from '../lib/auth-store.mjs';
import { localize } from '../lib/site.mjs';
import { nodePair } from './http.mjs';
import { openDurableSql } from './sql-do.mjs';

const UNAUTHORIZED = '{"type":"about:blank","status":401,"title":"Unauthorized"}';

/**
 * The one Durable Object that holds sign-in for every library: sessions, app tokens, password users and which library
 * each account reaches, answered by `lib/auth.mjs` exactly as `server.mjs` does with `AUTH=google`.
 */
export class Auth extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.db = openDurableSql(ctx.storage);
    this.auth = createAuth({ root: '', localize, env, port: 443, db: this.db });
  }

  /**
   * Answer sign-in, sign-out and every signed-out request; for a signed-in request, answer 204 naming its library in
   * `x-mymind-library`, its session in `x-mymind-credential` and its email in `x-mymind-email`.
   * @param {Request} request Incoming request; its body is read only by the sign-in endpoints.
   * @returns {Promise<Response>} The answer, or the 204 that lets the request through.
   */
  async fetch(request) {
    const { req, res, response } = nodePair(request);
    if (await this.auth.handle(req, res, new URL(request.url))) return response();
    const identity = this.auth.identity(req);
    if (!identity) return new Response(UNAUTHORIZED, { status: 401, headers: { 'content-type': 'application/json', 'cache-control': 'no-store' } });
    const library = libraryOf(this.db, identity);
    if (!library) return new Response('This account has no library yet.', { status: 403, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' } });
    return new Response(null, { status: 204, headers: { 'x-mymind-library': library, 'x-mymind-credential': identity.credential, 'x-mymind-email': identity.email } });
  }

  /**
   * The credentials, among `credentials`, that are still signed in to an account that reaches `library`.
   * @param {string[]} credentials Values of `x-mymind-credential` from earlier requests.
   * @param {string} library The library those requests were sent to.
   * @returns {string[]} Those whose session or app token is still valid and whose account still has that library.
   */
  active(credentials, library) {
    return credentials.filter((credential) => {
      const identity = this.auth.active(credential);
      return Boolean(identity) && libraryOf(this.db, identity) === library;
    });
  }

  /**
   * Store a password user exactly as another auth database holds it (`tools/upload-hosted.mjs`).
   * @param {{username: string, salt: Uint8Array, hash: Uint8Array, created_ms: number}} user Row of the `users` table.
   * @returns {void}
   */
  putUser({ username, salt, hash, created_ms }) {
    this.db.prepare(`INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)
      ON CONFLICT (username) DO UPDATE SET salt = excluded.salt, hash = excluded.hash`).run(username, salt, hash, created_ms);
  }

  /**
   * Give an account a library.
   * @param {string} account Account name: `google:<email>` or `password:<username>`.
   * @param {string} library Library name.
   * @returns {void}
   */
  setLibrary(account, library) {
    setLibrary(this.db, account, library);
  }

  /**
   * What sign-in holds, without secrets.
   * @returns {{users: string[], libraries: {account: string, library: string}[], sessions: number, appTokens: number}}
   *   Password users, library assignments and the number of live sessions and app tokens.
   */
  summary() {
    const count = (table) => this.db.prepare(`SELECT COUNT(*) AS n FROM ${table} WHERE expires_ms >= ?`).get(Date.now()).n;
    return { users: listUsers(this.db).map((user) => user.username), libraries: this.db.prepare('SELECT account, library FROM libraries ORDER BY account').all(),
      sessions: count('sessions'), appTokens: count('app_tokens') };
  }
}
