import { DurableObject } from 'cloudflare:workers';
import * as msgpack from '../mirror/static/134/0.109.0/scripts/vendor/msgpack.min.js';
import { createHosted } from '../lib/hosted.mjs';
import { insertRows, tableDigest } from '../lib/library-rows.mjs';
import { createRecorded } from '../lib/recorded.mjs';
import { send } from '../lib/site.mjs';
import { createStore } from '../lib/store.mjs';
import { r2Bucket } from './bucket-r2.mjs';
import { nodePair } from './http.mjs';
import { mediaClient } from './media-client.mjs';
import { openDurableSql } from './sql-do.mjs';

const PING = '{"type":6}\x1e';
const RECORD = '\x1e';
const MAX_MESSAGE = 1_048_576;
// How often connected hubs are checked for sessions that ended.
const SESSION_CHECK_MS = 60_000;

/**
 * One person's library: its database lives in this object's SQLite storage, its media in R2, and its live updates
 * (`/hub`) on WebSockets that only this library's events reach. The Worker sends only signed-in requests here, with
 * the library's name in `x-mymind-library` and the session in `x-mymind-credential`.
 */
export class Library extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.db = openDurableSql(ctx.storage);
    this.sending = Promise.resolve();
    ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair(PING, PING));
  }

  // The library's server, opened on its first request.
  site(library) {
    this.opening ??= createHosted({ db: this.db, bucket: r2Bucket(this.env.BUCKET), library, msgpack,
      ...mediaClient(this.env), push: (target, payload) => this.broadcast(target, payload),
      readonly: this.env.STORE_READONLY === '1', offline: this.env.OFFLINE === '1' })
      .catch((error) => { this.opening = null; throw error; });
    return this.opening;
  }

  /**
   * Answer one signed-in request to this library, a `/hub` WebSocket upgrade, or an import step from the Worker's
   * admin routes (`x-mymind-admin`).
   * @param {Request} request Request from the Worker.
   * @returns {Promise<Response>} The library's answer.
   */
  async fetch(request) {
    const library = request.headers.get('x-mymind-library');
    if (!/^[a-z0-9-]+$/.test(library ?? '')) return new Response('Unknown library', { status: 400 });
    const step = request.headers.get('x-mymind-admin');
    if (step) {
      try { return Response.json(await this.admin(step, library, request)); }
      catch (error) { return Response.json({ error: error.message }, { status: error.status ?? 500 }); }
    }
    const url = new URL(request.url);
    if (url.pathname === '/hub' && request.headers.get('upgrade')?.toLowerCase() === 'websocket') return this.connect(request, library);
    const site = await this.site(library);
    const { req, res, response } = nodePair(request);
    site.handle(req, res, url).then(() => {
      if (!res.writableEnded) res.destroy(new Error('No response'));
    }, (error) => {
      console.error(`[error] ${request.method} ${url.pathname}${url.search}`, error);
      if (!res.headersSent) send(res, 500, String(error));
      else res.destroy(error);
    });
    return response();
  }

  // Accept a SignalR connection; it receives events once its handshake is done.
  connect(request, library) {
    if (!request.headers.get('sec-websocket-key')) return new Response(null, { status: 400 });
    const [client, server] = Object.values(new WebSocketPair());
    this.ctx.acceptWebSocket(server);
    server.serializeAttachment({ credential: request.headers.get('x-mymind-credential'), library, ready: false, text: '' });
    this.ctx.storage.getAlarm().then((alarm) => alarm ?? this.ctx.storage.setAlarm(Date.now() + SESSION_CHECK_MS));
    return new Response(null, { status: 101, webSocket: client });
  }

  /**
   * Read SignalR records from a connection: the JSON handshake is answered, everything else is ignored.
   * @param {WebSocket} socket The connection.
   * @param {string|ArrayBuffer} message One WebSocket message.
   * @returns {void}
   */
  webSocketMessage(socket, message) {
    const state = socket.deserializeAttachment();
    state.text += typeof message === 'string' ? message : new TextDecoder().decode(message);
    if (state.text.length > MAX_MESSAGE) return socket.close(1009, 'Message too large');
    let end;
    while ((end = state.text.indexOf(RECORD)) >= 0) {
      const record = state.text.slice(0, end);
      state.text = state.text.slice(end + 1);
      let parsed;
      try { parsed = JSON.parse(record); } catch { return socket.close(1002, 'Invalid record'); }
      if (!state.ready && parsed.protocol === 'json' && parsed.version === 1) {
        socket.send(`{}${RECORD}`);
        state.ready = true;
      }
    }
    socket.serializeAttachment(state);
  }

  /**
   * Close the connections whose sessions ended or whose accounts left this library, and check again later while any
   * remain.
   * @returns {Promise<void>}
   */
  async alarm() {
    const sockets = this.ctx.getWebSockets();
    if (!sockets.length) return;
    await this.signedIn(sockets);
    if (this.ctx.getWebSockets().length) await this.ctx.storage.setAlarm(Date.now() + SESSION_CHECK_MS);
  }

  // The given connections still signed in to an account that reaches this library; the others are closed.
  async signedIn(sockets) {
    const states = sockets.map((socket) => socket.deserializeAttachment() ?? {});
    const library = states[0]?.library;
    const credentials = [...new Set(states.map((state) => state.credential).filter(Boolean))];
    const auth = this.env.AUTH_OBJECT.get(this.env.AUTH_OBJECT.idFromName('auth'));
    const live = new Set(library && credentials.length ? await auth.active(credentials, library) : []);
    return sockets.filter((socket, index) => {
      if (states[index].library === library && live.has(states[index].credential)) return true;
      socket.close(1008, 'Signed out');
      return false;
    });
  }

  // Send one committed change to every connection of this library that is still signed in, in commit order.
  broadcast(target, payload) {
    const message = JSON.stringify({ type: 1, target, arguments: [payload] }) + RECORD;
    this.sending = this.sending.then(async () => {
      const ready = this.ctx.getWebSockets().filter((socket) => socket.deserializeAttachment()?.ready);
      if (!ready.length) return;
      for (const socket of await this.signedIn(ready)) {
        try { socket.send(message); } catch {}
      }
    }).catch((error) => console.error('[hub]', error));
  }

  // One import step (`tools/upload-hosted.mjs`): reset, rows, finish, digest or counts.
  async admin(step, library, request) {
    const body = request.method === 'POST' ? await request.json() : {};
    if (step === 'reset') {
      const open = await this.opening?.catch(() => null);
      this.opening = null;
      open?.close();
      await this.ctx.storage.deleteAll();
      createStore(this.db);
      createRecorded(this.db);
      return { library, tables: this.tables() };
    }
    if (step === 'rows') return { inserted: insertRows(this.db, body.table, body.columns, body.rows) };
    if (step === 'finish') {
      this.db.transaction(() => {
        this.db.exec('DELETE FROM search_dirty');
        insertRows(this.db, 'search_dirty', ['id'], body.dirty ?? []);
        for (const { name, seq } of body.sequences ?? []) {
          if (!this.db.prepare('UPDATE sqlite_sequence SET seq = CAST(? AS INTEGER) WHERE name = ?').run(seq, name).changes) {
            this.db.prepare('INSERT INTO sqlite_sequence (name, seq) VALUES (?, CAST(? AS INTEGER))').run(name, seq);
          }
        }
      });
      this.db.exec("INSERT INTO search_fts(search_fts) VALUES ('integrity-check')");
      return { library, tables: this.tables() };
    }
    if (step === 'digest') return Object.fromEntries(body.tables.map(({ table, columns }) => [table, tableDigest(this.db, table, columns)]));
    if (step === 'counts') return { library, tables: this.tables() };
    throw Object.assign(new Error('Unknown import step'), { status: 404 });
  }

  // Row counts of the library's own tables.
  tables() {
    const names = this.db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_%' ESCAPE '\\' AND name NOT LIKE 'search_fts%' ORDER BY name").all();
    return Object.fromEntries(names.map(({ name }) => [name, this.db.prepare(`SELECT COUNT(*) AS n FROM "${name}"`).get().n]));
  }
}
