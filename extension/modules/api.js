import { SESSION_COOKIE, getHost } from './config.js';

/** A failed API call; `data` is the problem+json body or a generic stand-in. */
export class Problem extends Error {
  constructor(data) {
    super(data.detail ?? 'Something went wrong');
    this.name = 'Problem';
    this.data = data;
  }

  get status() {
    return this.data.status;
  }
}

/** Raised when no session cookie exists for the configured host. */
export class SignedOut extends Error {
  constructor() {
    super('signed out');
    this.name = 'SignedOut';
  }
}

/**
 * Reads the replica's session cookie value for one host.
 * @param {string} host - Origin to read the cookie of.
 * @returns {Promise<string | null>} The cookie value, or null when signed out.
 */
async function readSession(host) {
  const cookie = await chrome.cookies.get({ name: SESSION_COOKIE, url: host });
  return cookie?.value ?? null;
}

/**
 * Hashes a session value so caches can be keyed by account without keeping the credential.
 * @param {string} value - The session cookie value.
 * @returns {Promise<string>} Lowercase hex SHA-256.
 */
async function sha256(value) {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(value));
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

/** Client for the replica's JSON API; the session cookie is sent as a bearer token. */
export class API {
  /**
   * @param {string} version - Sent as `x-extension-version`.
   */
  constructor(version) {
    this.version = version;
  }

  /**
   * Names the library and account the next request would reach.
   * @returns {Promise<string | null>} `host|sha256(session)`, or null when signed out.
   */
  async scope() {
    const host = await getHost();
    const token = await readSession(host);
    return token ? `${host}|${await sha256(token)}` : null;
  }

  async send(method, path, headers = {}, body) {
    const host = await getHost();
    const token = await readSession(host);
    if (!token) throw new SignedOut();
    const url = new URL(path, host);
    if (url.origin !== host) throw new Error('refusing to send the session to another origin');
    return fetch(url, {
      method,
      credentials: 'omit',
      redirect: 'error',
      headers: { Accept: 'application/json', Authorization: `Bearer ${token}`, 'x-extension-version': this.version, ...headers },
      body,
    });
  }

  /**
   * GETs JSON.
   * @param {string} path - Path and query.
   * @returns {Promise<any>} The parsed body.
   * @throws {Problem | SignedOut} On a non-2xx answer or a missing session.
   */
  async getJSON(path) {
    const response = await this.send('GET', path);
    if (!response.ok) throw new Problem(await this.getProblem(response));
    return response.json();
  }

  /**
   * @param {string} path - Path and query.
   * @returns {Promise<Response>} The raw response.
   */
  delete(path) {
    return this.send('DELETE', path);
  }

  /**
   * @param {string} path - Path and query.
   * @param {unknown} value - JSON body.
   * @returns {Promise<Response>} The raw response.
   */
  putJSON(path, value) {
    return this.send('PUT', path, { 'Content-Type': 'application/json' }, JSON.stringify(value));
  }

  /**
   * @param {string} path - Path and query.
   * @param {unknown} value - JSON body.
   * @returns {Promise<Response>} The raw response.
   */
  postJSON(path, value) {
    return this.send('POST', path, { 'Content-Type': 'application/json' }, JSON.stringify(value));
  }

  /**
   * @param {string} path - Path and query.
   * @param {Blob} blob - Body, sent with its own content type.
   * @returns {Promise<Response>} The raw response.
   */
  postBlob(path, blob) {
    return this.send('POST', path, { 'Content-Type': blob.type }, blob);
  }

  /**
   * @param {Response} response - A response.
   * @returns {boolean} Whether the body is problem+json.
   */
  hasProblem(response) {
    return response.headers.get('Content-Type')?.startsWith('application/problem+json') ?? false;
  }

  /**
   * @param {Response} response - A failed response.
   * @returns {Promise<{detail: string, status: number}>} The problem body, or a generic one.
   */
  async getProblem(response) {
    if (this.hasProblem(response)) return response.json();
    return { detail: 'Something went wrong', status: response.status };
  }
}
