/**
 * The card grid that MCP Apps hosts (Claude, ChatGPT) show for `thymind_search` results instead of a text list. It is
 * one self-contained page: the results come from the tool result the host hands it, and each picture from a
 * `thymind_get_picture` call the page makes through the host, so the page itself never reaches the library or holds a
 * token, and its frame needs no network at all.
 */

/** The view's resource URI, named in the search tool's `_meta.ui.resourceUri`. */
export const VIEW_URI = "ui://thymind/cards.html";

/** MCP Apps' HTML resource type. */
export const VIEW_MIME = "text/html;profile=mcp-app";

/** The page. Card text is set with `textContent` only, never parsed as HTML. */
export const VIEW_HTML = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Inventory cards</title>
<style>
:root {
  --bg: #F8F7F3; --raised: #FFFFFF; --text: #262520; --muted: #716E64; --accent: #745B28; --shadow: rgba(38, 37, 32, .16);
  color-scheme: light;
}
:root[data-theme="dark"] {
  --bg: #10100F; --raised: #1F1F1D; --text: #F4F2EC; --muted: #B5B2A8; --accent: #E2BC78; --shadow: rgba(0, 0, 0, .5);
  color-scheme: dark;
}
* { box-sizing: border-box; }
html, body { margin: 0; background: var(--bg); color: var(--text); }
body { font: 15px/21px var(--font-sans, "Google Sans Flex", "Google Sans", system-ui, -apple-system, "Segoe UI", sans-serif); padding: 16px; }
#grid { columns: 3 168px; column-gap: 12px; }
.tile { break-inside: avoid; margin: 0 0 12px; display: block; width: 100%; padding: 0; border: 0; text-align: left; font: inherit; color: inherit;
  background: var(--raised); border-radius: 12px; box-shadow: 0 1px 2px var(--shadow), 0 2px 6px var(--shadow); overflow: hidden; }
button.tile { cursor: pointer; }
button.tile:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.print { padding: 4px 4px 0; }
.print img { display: block; width: 100%; height: auto; max-height: 340px; object-fit: cover; object-position: top; border-radius: 8px; background: var(--bg); }
.words { padding: 10px 12px 12px; }
.title { font-weight: 600; font-size: 13px; line-height: 18px; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }
.snippet { margin-top: 4px; font-size: 13px; line-height: 18px; color: var(--muted); display: -webkit-box; -webkit-line-clamp: 4; -webkit-box-orient: vertical; overflow: hidden; }
.source { margin-top: 6px; font-size: 11px; line-height: 14px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tile.has-picture .snippet { display: none; }
#foot { color: var(--muted); font-size: 13px; line-height: 18px; padding: 4px 2px 0; }
#foot:empty { display: none; }
</style>
</head>
<body>
<div id="grid" role="list"></div>
<div id="foot"></div>
<script>
(() => {
  const grid = document.getElementById("grid");
  const foot = document.getElementById("foot");
  const pending = new Map();
  let nextId = 1;
  let shown = "";
  const openai = window.openai;

  function send(message) { window.parent.postMessage(Object.assign({ jsonrpc: "2.0" }, message), "*"); }
  function request(method, params) {
    const id = nextId++;
    send({ id, method, params });
    return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
  }

  function theme(context) {
    const dark = context && context.theme ? context.theme === "dark" : window.matchMedia("(prefers-color-scheme: dark)").matches;
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    const font = context && context.styles && context.styles.variables && context.styles.variables["--font-sans"];
    if (font) document.documentElement.style.setProperty("--font-sans", font);
  }

  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text) node.textContent = text;
    return node;
  }

  function openLink(url) {
    if (openai && openai.openExternal) openai.openExternal({ href: url });
    else request("ui/open-link", { url }).catch(() => {});
  }

  async function picture(card, tile) {
    try {
      const args = { id: card.id, size: 384 };
      const result = openai && openai.callTool ? await openai.callTool("thymind_get_picture", args) : await request("tools/call", { name: "thymind_get_picture", arguments: args });
      const image = result && !result.isError && (result.content || []).find((part) => part.type === "image");
      if (!image || !/^image\\/(png|jpeg|webp|gif|avif)$/.test(image.mimeType)) return;
      const img = element("img");
      img.alt = "";
      img.src = "data:" + image.mimeType + ";base64," + image.data;
      const print = element("div", "print");
      print.append(img);
      tile.prepend(print);
      tile.classList.add("has-picture");
    } catch (error) {}
  }

  async function pictures(jobs) {
    const workers = [0, 1, 2].map(async () => {
      while (jobs.length) {
        const [card, tile] = jobs.shift();
        await picture(card, tile);
      }
    });
    await Promise.all(workers);
  }

  function show(data) {
    if (!data || !Array.isArray(data.cards)) return;
    const key = JSON.stringify(data.cards.map((card) => card.id));
    if (key === shown) return;
    shown = key;
    grid.replaceChildren();
    const jobs = [];
    for (const card of data.cards) {
      const link = typeof card.url === "string" && /^https?:\\/\\//.test(card.url) ? card.url : "";
      const tile = element(link ? "button" : "div", "tile");
      tile.setAttribute("role", "listitem");
      if (link) {
        tile.type = "button";
        tile.title = link;
        tile.addEventListener("click", () => openLink(link));
      }
      const words = element("div", "words");
      words.append(element("div", "title", card.title || "Untitled"));
      if (card.snippet && card.snippet !== card.title) words.append(element("div", "snippet", card.snippet));
      const source = [card.site, (card.spaces || []).join(", ")].filter(Boolean).join(" · ");
      if (source) words.append(element("div", "source", source));
      tile.append(words);
      grid.append(tile);
      if (card.type !== "Note") jobs.push([card, tile]);
    }
    foot.textContent = data.cards.length === 0 ? "No cards found." : data.has_more ? "More cards match; ask for the next page." : "";
    pictures(jobs);
  }

  window.addEventListener("message", (event) => {
    if (event.source !== window.parent) return;
    const message = event.data;
    if (!message || message.jsonrpc !== "2.0") return;
    if (message.id !== undefined && pending.has(message.id) && !message.method) {
      const waiting = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) waiting.reject(message.error); else waiting.resolve(message.result);
      return;
    }
    if (message.method === "ui/notifications/tool-result") show(message.params && message.params.structuredContent);
    if (message.method === "ui/notifications/host-context-changed") theme(message.params);
  });

  new ResizeObserver(() => {
    send({ method: "ui/notifications/size-changed", params: { width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight } });
  }).observe(document.body);

  theme(null);
  if (openai && openai.toolOutput) show(openai.toolOutput);
  window.addEventListener("openai:set_globals", () => { if (window.openai) show(window.openai.toolOutput); });
  request("ui/initialize", { appCapabilities: {}, appInfo: { name: "Inventory cards", version: "1.0.0" }, protocolVersion: "2026-01-26" })
    .then((result) => {
      theme(result && result.hostContext);
      send({ method: "ui/notifications/initialized" });
    })
    .catch(() => {});
})();
</script>
</body>
</html>
`;
