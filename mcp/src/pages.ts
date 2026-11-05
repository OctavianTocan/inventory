/**
 * The few pages a person sees while connecting an AI app: consent, refusal and errors. Same plain look as the
 * server's own signed-out page (`rust/crates/core/src/auth.rs`, PLAIN_HEAD): no mymind name or assets, `noindex`.
 */
import type { ConsentDescription } from "@cloudflare/workers-oauth-provider";

/** The scope every connection gets. */
export const READ_SCOPE = "thymind.read";
/** The scope that lets an agent change the library; ticked by default (Tavi, 2026-11-05: "Read, save and edit"). */
export const WRITE_SCOPE = "thymind.write";

/**
 * Escape text for HTML: client names, domains and scopes come from whoever registered the client.
 *
 * @param value Untrusted text.
 * @returns The text with HTML's special characters as entities.
 */
export function escape(value: string): string {
  return value.replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);
}

const HEAD = `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex, nofollow"><link rel="icon" href="data:,"><title>Connect an app</title><style>
:root { color-scheme: light dark; --bg: #f5f5f3; --card: #fff; --text: #1d1d1f; --muted: #6e6e73; --line: #d8d8dc; --button: #1d1d1f; --button-text: #fff; }
@media (prefers-color-scheme: dark) { :root { --bg: #111113; --card: #1c1c1f; --text: #f2f2f4; --muted: #a1a1a6; --line: #333338; --button: #f2f2f4; --button-text: #111113; } }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 16px; background: var(--bg); color: var(--text); font: 16px/1.4 system-ui, sans-serif; }
.card { width: 100%; max-width: 400px; display: grid; gap: 14px; padding: 32px 28px; background: var(--card); border: 1px solid var(--line); border-radius: 16px; }
h1 { margin: 0 0 6px; font-size: 22px; font-weight: 600; overflow-wrap: anywhere; }
p, li { margin: 0; color: var(--muted); font-size: 15px; overflow-wrap: anywhere; }
ul { margin: 0; padding-left: 20px; display: grid; gap: 4px; }
strong { color: var(--text); font-weight: 600; }
label { display: flex; gap: 10px; align-items: flex-start; font-size: 15px; color: var(--text); }
input[type=checkbox] { margin-top: 3px; }
a { color: var(--muted); font-size: 14px; text-align: center; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
button, .button { display: flex; align-items: center; justify-content: center; gap: 10px; font: inherit; font-weight: 600; padding: 12px; border: 0; border-radius: 10px; background: var(--button); color: var(--button-text); cursor: pointer; text-decoration: none; }
button.quiet { background: transparent; color: var(--text); border: 1px solid var(--line); }
.warn { color: #d93a1a; font-size: 14px; }
</style>
`;

function page(body: string): string {
  return `${HEAD}<main class="card">\n${body}\n</main></html>`;
}

/**
 * The consent page shown before any Google sign-in, every time a client asks (MCP's per-client consent rule).
 *
 * @param details What the OAuth library verified about the client and its redirect.
 * @param handle The single-use consent handle.
 * @returns The page's HTML.
 */
export function consentPage(details: ConsentDescription, handle: string): string {
  const name = escape(details.clientName);
  const origin = details.clientDomain
    ? `<p>Published by <strong>${escape(details.clientDomain)}</strong>. Access goes to <strong>${escape(details.redirectHost)}</strong>.</p>`
    : `<p>This app named itself; its name is not checked. Access goes to <strong>${escape(details.redirectHost)}</strong>.</p>`;
  const local = details.redirectIsLoopback
    ? `<p class="warn">This sends access to an app on your computer. Continue only if you just started connecting from it.</p>`
    : "";
  return page(`<h1>Connect ${name} to Inventory?</h1>
${origin}
${local}
<p>It will be able to search your cards and read their links, notes, text and pictures.</p>
<form method="post" class="card" style="padding:0;border:0;gap:14px">
<input type="hidden" name="handle" value="${escape(handle)}">
<label><input type="checkbox" name="write" value="1" checked> Also let it save, tag, sort into spaces, pin, write notes and move cards to the trash. Nothing is deleted for good.</label>
<p>You will sign in with Google next.</p>
<div class="row"><button type="submit" name="decision" value="deny" class="quiet">Cancel</button><button type="submit" name="decision" value="approve">Continue</button></div>
</form>`);
}

/**
 * The page for a Google account Inventory does not admit. It never says which accounts are allowed.
 *
 * @param back Where to send the person back to the app, with `access_denied`.
 * @returns The page's HTML.
 */
export function refusedPage(back: string): string {
  return page(`<h1>This Google account can't use Inventory</h1>
<p>Inventory is private. Try again with the Google account you use for Inventory.</p>
<a class="button" href="${escape(back)}">Back to the app</a>`);
}

/**
 * A plain error page.
 *
 * @param message What went wrong and what to do, in plain words (escaped here).
 * @returns The page's HTML.
 */
export function errorPage(message: string): string {
  return page(`<h1>Connecting didn't work</h1>\n<p>${escape(message)}</p>`);
}

/**
 * An HTML response for one of the pages above.
 *
 * @param html The page.
 * @param status HTTP status.
 * @param headers Headers to carry (for example the consent page's binding cookie and no-framing headers).
 * @returns The response.
 */
export function html(html: string, status = 200, headers = new Headers()): Response {
  headers.set("content-type", "text/html; charset=utf-8");
  headers.set("cache-control", "no-store");
  if (!headers.has("x-frame-options")) headers.set("x-frame-options", "DENY");
  return new Response(html, { status, headers });
}
