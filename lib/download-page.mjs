const escape = (text) => String(text).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const size = (bytes) => `${(bytes / 1_048_576).toFixed(1)} MB`;

/**
 * The download page for the packaged desktop app: every packaged file with its size and checksum, and how to
 * install it.
 * @param {{library?: string, files: {name: string, label: string, bytes: number, built: string, sha256: string}[]}}
 *   manifest The `manifest.json` that `deploy/package-app.sh` writes beside the packages.
 * @param {string} user Email or username of the signed-in visitor.
 * @returns {string} The HTML page.
 */
export function downloadPage(manifest, user) {
  const rows = manifest.files.map((file) => `<li><a href="/download/${encodeURIComponent(file.name)}">${escape(file.label)}</a>
    <span>${size(file.bytes)} · built ${escape(file.built.slice(0, 10))}</span><code>sha256 ${escape(file.sha256)}</code></li>`).join('');
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>mymind app</title><style>
:root{color-scheme:light dark;--bg:#f7f8fa;--fg:#1d2433;--muted:#6b7280;--accent:#ff5924;--card:#fff}
@media (prefers-color-scheme:dark){:root{--bg:#14161b;--fg:#e8eaf0;--muted:#9aa1ad;--card:#1d2027}}
body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}
main{max-width:640px;margin:0 auto;padding:48px 16px}h1{font:600 2rem Georgia,serif;margin:0 0 8px}
p,span{color:var(--muted)}ul{list-style:none;padding:0}li{background:var(--card);border-radius:14px;padding:16px;margin:12px 0}
a{color:var(--accent);font-weight:600}li span,li code{display:block;font-size:.85rem;overflow-wrap:anywhere}
pre{background:var(--card);border-radius:14px;padding:16px;overflow-x:auto;font-size:.85rem}
</style></head><body><main><h1>mymind app</h1>
<p>Opens your library at ${escape(manifest.library ?? 'the hosted address')}; sign in with Google the first time.</p>
<ul>${rows || '<li>No app has been packaged yet. Run <code>deploy/package-app.sh</code> on the host.</li>'}</ul>
<pre>tar -xzf mymind-linux-x86_64.tar.gz
mymind/install.sh        # ~/.local/bin/mymind and a launcher entry</pre>
<p>Needs WebKitGTK 4.1 and GStreamer's good plugins (Arch: <code>webkit2gtk-4.1 gst-plugins-good</code>).</p>
<p>Signed in as ${escape(user)} · <a href="/signout">Sign out</a></p>
</main></body></html>`;
}
