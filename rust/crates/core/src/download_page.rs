//! Port of `lib/download-page.mjs`: the packaged desktop app's download page.

use mm_js::{Value, number::to_fixed, text::slice, uri::encode_uri_component};

use crate::render::access::member;
use crate::{Error, Result};

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

fn size(bytes: &Value) -> String {
    format!("{} MB", to_fixed(bytes.to_number() / 1_048_576.0, 1))
}

fn built(file: &Value) -> Result<String> {
    match member(file, "built")? {
        Value::String(text) => Ok(slice(text, 0, Some(10))),
        Value::Array(items) => Ok(Value::Array(items.iter().take(10).cloned().collect()).to_js_string()),
        other => Err(Error::invalid(format!("file.built.slice is not a function ({})", other.type_of()))),
    }
}

/// The download page for the packaged desktop app: every packaged file with its size and checksum, and how to
/// install it.
///
/// # Arguments
/// * `manifest` - The `manifest.json` that `deploy/package-app.sh` writes: optional `library` and a `files` array of
///   `name`, `label`, `bytes`, `built` and `sha256`.
/// * `user` - Email or username of the signed-in visitor.
///
/// # Returns
/// The HTML page.
///
/// # Errors
/// `manifest` or one of its files is null, or `files` is not an array.
pub fn download_page(manifest: &Value, user: &str) -> Result<String> {
    let Value::Array(files) = member(manifest, "files")? else {
        return Err(Error::invalid("manifest.files.map is not a function"));
    };
    let mut rows = String::new();
    for file in files {
        let name = member(file, "name")?.to_js_string();
        rows.push_str(&format!(
            "<li><a href=\"/download/{}\">{}</a>\n    <span>{} · built {}</span><code>sha256 {}</code></li>",
            encode_uri_component(&name),
            escape(&file.get("label").to_js_string()),
            size(file.get("bytes")),
            escape(&built(file)?),
            escape(&file.get("sha256").to_js_string()),
        ));
    }
    let library = manifest.get("library");
    let library = if library.is_nullish() { "the hosted address".to_string() } else { library.to_js_string() };
    let rows = if rows.is_empty() { "<li>No app has been packaged yet. Run <code>deploy/package-app.sh</code> on the host.</li>".into() } else { rows };
    Ok(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">
<title>mymind app</title><style>
:root{{color-scheme:light dark;--bg:#f7f8fa;--fg:#1d2433;--muted:#6b7280;--accent:#ff5924;--card:#fff}}
@media (prefers-color-scheme:dark){{:root{{--bg:#14161b;--fg:#e8eaf0;--muted:#9aa1ad;--card:#1d2027}}}}
body{{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}}
main{{max-width:640px;margin:0 auto;padding:48px 16px}}h1{{font:600 2rem Georgia,serif;margin:0 0 8px}}
p,span{{color:var(--muted)}}ul{{list-style:none;padding:0}}li{{background:var(--card);border-radius:14px;padding:16px;margin:12px 0}}
a{{color:var(--accent);font-weight:600}}li span,li code{{display:block;font-size:.85rem;overflow-wrap:anywhere}}
pre{{background:var(--card);border-radius:14px;padding:16px;overflow-x:auto;font-size:.85rem}}
</style></head><body><main><h1>mymind app</h1>
<p>Opens your library at {}; sign in with Google the first time.</p>
<ul>{rows}</ul>
<pre>tar -xzf mymind-linux-x86_64.tar.gz
mymind/install.sh        # ~/.local/bin/mymind and a launcher entry</pre>
<p>Needs WebKitGTK 4.1 and GStreamer's good plugins (Arch: <code>webkit2gtk-4.1 gst-plugins-good</code>).</p>
<p>Signed in as {} · <a href=\"/signout\">Sign out</a></p>
</main></body></html>",
        escape(&library),
        escape(user)
    ))
}
