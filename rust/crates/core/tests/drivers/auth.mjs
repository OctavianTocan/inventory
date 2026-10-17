// Drives lib/auth.mjs's createAuth for mm-core's differential tests: a fake clock, a deterministic random source, a
// mocked Google endpoint and a fake request/response pair, answering one snapshot per step.
import { createHash, createPrivateKey, createPublicKey, createSign } from 'node:crypto';
import { createRequire, syncBuiltinESMExports } from 'node:module';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const crypto = require('node:crypto');
const ROOT = process.env.MYMIND_ROOT;
const lib = (name) => import(pathToFileURL(join(ROOT, 'lib', name)).href);

const PEM = `-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCM5G0cfsoMd1b8
Fz+T4nzsg/B+Xk1yMv050rleRsVujX9TJc03CIUASWE69X2XVGan9JwZCfqdNggV
hiASTTDQ9VPzJpm5Re+jecWaoBg4fD70EvOVFXvcOBs/VmUd2qV7iDWdx+C3FlIA
YiXXPK+NVzfGGQtvd2QwW3ZY1yvT/F98O5yu/NPVnWU3BwefSkk4SQOxgwgv6BED
6zqCCcg+ngU3D0+DjWyMc4k1A+Jy7qrtbuplJyG+Z/JXTiG0Ukk1ZgVSkDE7R7t/
hN1riqeFSWI2IisvUldyWAPJaXflftYr0bxPohXaFHhzw09QYeJfKzY/YMb+0c9m
SC5BoTetAgMBAAECggEAHhOzgUZFzsR5MsapwWGzxm/iGq/5+3uMGFh2TvxvsEqP
mEn1FZ8BeY6wrXvzrCcN5SPwYN4C/KWosyOXXCDTX4BHxBwE7ArUfgMEXp53nTNU
4xjanARilR/7HRnN60IeWpE41O5y99LRfZh2OI0RcuxgES+Og+SqZDJaQNvqVM31
0eTC5eUFBjKg0gGvp4zcjAYvlPhmjFqw78qGjt+TZ95DLbWMYBaunTwHf890G5VZ
YL4BLfFYZFj5lr0Y67fimonFwBfxP7duwUo8pZYmPHkgoD2WdxqFx+Oz1Pp21WDq
ihqqOeoIieiEnp9bFNxbvW3DkaC2oS8tT5/DFldygQKBgQDGFL2/CnX83Xk6drWx
2bn4dCmySxRdhsuS5llptBOTIUB40xcp8ytX0eQUGhFuvdVlQ5woEd5lQBOvIEmo
8ivLMK6J29ECFMAxpz1veE3N+itHpGXz5wWW8+1KR3zHQq6Lpi6+79LIZT518JBl
Oy8ocGO5bnzWoKzuGp2OME1mwQKBgQC2FtmuM7FbXt+TfL4a92Ol4ceDL71PyZOz
G3KcZonLi29L9ps8hpCsZ0rHwG+tPsZPIYgkB3ciXWLD1nRvX/T9a6oOKZR/Xdm7
80AoOnUvDrhh9jvZzYwJwtoqVJqUn9qahWUi0ecsX9srXEnR5dpTgPHUWNl3uuRE
+nCUaO3X7QKBgGczbTy4lYp+VWf5a2bTGkpxMzw6U8z6qtvJefJRqRUx8OZCjMIF
eKKTr3IttsFs1F6jIBvcW3ODM/49I0OLkPeXUIhw529fIld8W8zU8CyN17jG1XuC
9O2V/x6BKmiivE/z8Ez4pGhWH3nRMVocwNAf5kU5ZH9qTU7svQPU9K7BAoGAYZj3
zgZI/rdoUI4g+yU0VPoiTO9uofBiaTLCCJBxNRckbWvdouBhdPbcPxqnAQ3PjyRi
6yTRNyskv+EXgYbGMnrdqD+jJ21vQe1MqwNCQhm6vE8Vn1UrxSOaeevk+tp/eikb
n0EwjjlGqQQLUSTJE87eT7nbve//X/V4GCPxr7UCgYBf0sB2wg3QP91gSy1s3J7D
XMy1a/pAsFv/zBCt6oOiRbvsof2QLyUV0dnodO6FlfBH/1tuLkyWa/UYUW2dBldO
X+kzeoHcj+I5MYML5ndLHuU8d2do061w1H6GZGtyOjsjrUnbkt9idUxJuYSYn3Bw
qlkk2RQrXLCaGHIlRv5iug==
-----END PRIVATE KEY-----
`;
const KID = 'test-key-1';

const base64url = (value) => Buffer.from(value).toString('base64url');

/** The test key's public JWK (with `kid`). */
export function jwk(kid = KID) {
  return { ...createPublicKey(createPrivateKey(PEM)).export({ format: 'jwk' }), kid, use: 'sig', alg: 'RS256' };
}

/** An RS256 token for `claims` signed with the test key; `header` overrides header fields. */
export function sign(claims, header = {}) {
  const head = base64url(JSON.stringify({ alg: 'RS256', kid: KID, typ: 'JWT', ...header }));
  const body = base64url(JSON.stringify(claims));
  const signature = createSign('RSA-SHA256').update(`${head}.${body}`).sign(PEM);
  return `${head}.${body}.${base64url(signature)}`;
}

/** `crypto.scrypt(password, salt, 64)` as hex for each (password, salt hex) pair. */
export function hashes(pairs) {
  return pairs.map(([password, salt]) => crypto.scryptSync(password, Buffer.from(salt, 'hex'), 64).toString('hex'));
}

function deterministic(real) {
  const counters = new Map();
  const draw = (size) => {
    if (size === 16 || size === 64) return real(size);
    const counter = (counters.get(size) ?? 0) + 1;
    counters.set(size, counter);
    return createHash('sha256').update(`r${size}:${counter}`).digest().subarray(0, size);
  };
  return draw;
}

function snapshot(db) {
  const out = {};
  for (const table of ['sessions', 'app_tokens', 'users', 'logins', 'app_nonces', 'failures', 'libraries']) {
    out[table] = db.prepare(`SELECT * FROM ${table} ORDER BY rowid`).all().map((row) => Object.fromEntries(Object.entries(row).map(([key, value]) => [key, value instanceof Uint8Array ? Buffer.from(value).toString('hex') : value])));
  }
  return out;
}

function fakeRequest(spec) {
  const url = new URL(spec.url);
  const headers = { host: url.host, ...(spec.headers ?? {}) };
  const body = spec.body === null || spec.body === undefined ? [] : [Buffer.from(spec.body, 'utf8')];
  return {
    method: spec.method,
    url: `${url.pathname}${url.search}`,
    headers,
    socket: { remoteAddress: spec.remote ?? '127.0.0.1' },
    async *[Symbol.asyncIterator]() { yield* body; },
  };
}

function fakeResponse() {
  const res = { status: null, headers: [], body: '', ended: false };
  res.writeHead = (status, headers = {}) => {
    res.status = status;
    res.headers = Object.entries(headers).flatMap(([name, value]) => (Array.isArray(value) ? value.map((item) => [name, item]) : [[name, String(value)]]));
  };
  res.end = (body = '') => { res.body = String(body); res.ended = true; };
  return res;
}

const classify = (error) => (error instanceof TypeError || error instanceof RangeError ? 'invalid' : 'internal');

/** Run one scenario: `{env, port, users, clock, steps}`; answers one record per step. */
export async function run(scenario) {
  const realRandom = crypto.randomBytes;
  crypto.randomBytes = deterministic(realRandom);
  syncBuiltinESMExports();
  const realFetch = globalThis.fetch;
  const realNow = Date.now;
  let clock = scenario.clock;
  Date.now = () => clock;
  const fetched = [];
  let mocks = { certs: null, token: null };
  globalThis.fetch = async (url, options = {}) => {
    const logged = { url: String(url), method: options.method ?? 'GET', headers: options.headers ?? {}, body: options.body === undefined ? null : String(options.body) };
    fetched.push(logged);
    const mock = logged.method === 'POST' ? mocks.token : mocks.certs;
    if (!mock) throw new TypeError('fetch failed');
    return new Response(mock.body, { status: mock.status, headers: mock.headers ?? {} });
  };
  const directory = mkdtempSync(join(tmpdir(), 'mm-auth-'));
  const { openSql } = await lib('sql-node.mjs');
  const { installAuthSchema } = await lib('auth-store.mjs');
  const { createAuth } = await lib('auth.mjs');
  const db = openSql(join(directory, 'auth.db'));
  installAuthSchema(db);
  for (const user of scenario.users ?? []) {
    db.prepare('INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)').run(user.username, Buffer.from(user.salt, 'hex'), Buffer.from(user.hash, 'hex'), user.created_ms);
  }
  for (const statement of scenario.seed ?? []) db.prepare(statement.sql).run(...statement.params);
  const answers = [];
  let auth = null;
  try {
    try {
      auth = createAuth({ root: ROOT, localize: (text) => text.replaceAll('https://static.accelerator.net', '/_static'), env: scenario.env, port: scenario.port, db });
      answers.push({ op: 'create', created: auth !== null, db: snapshot(db) });
    } catch (error) {
      answers.push({ op: 'create', error: classify(error), db: snapshot(db) });
    }
    for (const step of scenario.steps) {
      const answer = { op: step.op };
      try {
        if (step.op === 'clock') clock = step.ms;
        else if (step.op === 'mock') mocks = { certs: step.certs ?? null, token: step.token ?? null };
        else if (step.op === 'sql') db.prepare(step.sql).run(...step.params);
        else {
          fetched.length = 0;
          if (step.op === 'req') {
            const req = fakeRequest(step);
            const res = fakeResponse();
            const handled = await auth.handle(req, res, new URL(step.url));
            Object.assign(answer, { handled, status: res.status, headers: res.headers, body: res.body });
          } else if (step.op === 'allows') answer.value = auth.allows(fakeRequest(step));
          else if (step.op === 'identity') answer.value = auth.identity(fakeRequest(step));
          else if (step.op === 'active') answer.value = auth.active(step.credential);
          answer.fetched = [...fetched];
        }
      } catch (error) {
        answer.error = classify(error);
        answer.fetched = [...fetched];
      }
      answer.db = snapshot(db);
      answers.push(answer);
    }
  } finally {
    Date.now = realNow;
    globalThis.fetch = realFetch;
    crypto.randomBytes = realRandom;
    syncBuiltinESMExports();
    db.close();
  }
  return answers;
}
