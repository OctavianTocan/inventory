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
