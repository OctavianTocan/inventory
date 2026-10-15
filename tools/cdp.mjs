/**
 * Attach to the first open page whose URL starts with `urlPrefix` in a Chrome exposing CDP.
 * @param {string} endpoint CDP HTTP endpoint, e.g. `http://127.0.0.1:9222`.
 * @param {string} urlPrefix Page URL prefix that selects the tab.
 * @returns {Promise<{ url: string, send: (method: string, params?: object) => Promise<any>, evaluate: (fn: Function, arg?: unknown) => Promise<any>, close: () => void }>}
 *   `send` issues a raw CDP command against the page; `evaluate` runs a self-contained function in the page
 *   with one JSON argument and resolves its awaited return value.
 * @throws {Error} When no matching page is open or the page throws.
 */
export async function connectToPage(endpoint, urlPrefix) {
  const targets = await (await fetch(`${endpoint}/json/list`)).json();
  const target = targets.find((t) => t.type === 'page' && t.url.startsWith(urlPrefix));
  if (!target) throw new Error(`No open tab at ${urlPrefix} on ${endpoint}. Open it in Chrome first.`);

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    ws.onopen = resolve;
    ws.onerror = () => reject(new Error(`Could not attach to ${target.url}`));
  });

  let nextId = 0;
  const pending = new Map();
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    const call = pending.get(msg.id);
    if (!call) return;
    pending.delete(msg.id);
    if (msg.error) call.reject(new Error(msg.error.message));
    else call.resolve(msg.result);
  };
  const send = (method, params = {}) =>
    new Promise((resolve, reject) => {
      const id = ++nextId;
      pending.set(id, { resolve, reject });
      ws.send(JSON.stringify({ id, method, params }));
    });

  const evaluate = async (fn, arg = null) => {
    const result = await send('Runtime.evaluate', {
      expression: `(${fn})(${JSON.stringify(arg)})`,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text);
    }
    return result.result.value;
  };

  return { url: target.url, send, evaluate, close: () => ws.close() };
}

/**
 * Downloads a URL through the browser's own network stack, with the page's cookies and no CORS checks.
 * @param {{ send: (method: string, params?: object) => Promise<any> }} page Page from `connectToPage`, or a CDP session.
 * @param {string} frameId Main frame ID of that page.
 * @param {string} url Absolute URL to load.
 * @returns {Promise<{ status: number, headers: Record<string, string>, body: Buffer } | null>} The response, or null if it failed.
 */
export async function loadResource(page, frameId, url) {
  const { resource } = await page.send('Network.loadNetworkResource', {
    frameId,
    url,
    options: { disableCache: false, includeCredentials: true },
  });
  if (!resource.success || !resource.stream) return null;
  const chunks = [];
  for (;;) {
    const read = await page.send('IO.read', { handle: resource.stream, size: 1 << 20 });
    chunks.push(Buffer.from(read.data, read.base64Encoded ? 'base64' : 'utf8'));
    if (read.eof) break;
  }
  await page.send('IO.close', { handle: resource.stream });
  return { status: resource.httpStatusCode, headers: resource.headers ?? {}, body: Buffer.concat(chunks) };
}

/**
 * Downloads a URL with the attached tab's cookies, bypassing CORS (media redirects cross-origin without it).
 * @param {{ send: (method: string, params?: object) => Promise<any> }} page Page from `connectToPage`.
 * @param {string} frameId Main frame ID of that page.
 * @param {string} url Absolute URL to load.
 * @returns {Promise<Buffer | null>} The body, or null unless the final response was 200.
 */
export async function loadThroughDevTools(page, frameId, url) {
  const response = await loadResource(page, frameId, url);
  return response?.status === 200 ? response.body : null;
}
