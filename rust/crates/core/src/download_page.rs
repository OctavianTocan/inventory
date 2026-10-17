//! Port of `lib/download-page.mjs`: the download page for the packaged apps and the Chrome extension, and what
//! `/download` serves for the Android app's updater.

use mm_js::{Value, number::to_fixed, obj, text::slice, uri::encode_uri_component};

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

/// Media type of the Android package (`.apk`).
pub const APK_TYPE: &str = "application/vnd.android.package-archive";

/// Name under `/download/` of the newest Android release's description, [`android_release`].
pub const ANDROID_RELEASE: &str = "android.json";

/// Media type of the zipped Chrome extension.
pub const EXTENSION_TYPE: &str = "application/zip";

fn is_android(file: &Value) -> bool {
    file.get("platform").as_str() == Some("android")
}

fn is_extension(file: &Value) -> bool {
    file.get("platform").as_str() == Some("chrome-extension")
}

/// The media type `/download/<name>` sends a packaged file with.
///
/// # Arguments
/// * `file` - The file's manifest entry.
///
/// # Returns
/// [`APK_TYPE`] for an entry with `platform: "android"`, [`EXTENSION_TYPE`] for `platform: "chrome-extension"`,
/// `application/gzip` for any other.
pub fn content_type(file: &Value) -> &'static str {
    if is_android(file) {
        APK_TYPE
    } else if is_extension(file) {
        EXTENSION_TYPE
    } else {
        "application/gzip"
    }
}

// The Android entry with the highest `versionCode`; the publish script keeps only one. A package named like the
// release description could never be downloaded, so it does not count.
fn newest_android(manifest: &Value) -> Option<&Value> {
    let files = manifest.get("files").as_array()?;
    files.iter().filter(|file| is_android(file) && file.get("name").as_str() != Some(ANDROID_RELEASE)).max_by(|a, b| a.get("versionCode").to_number().total_cmp(&b.get("versionCode").to_number()))
}

/// The newest Android package in a manifest, as the app's updater reads it at `/download/android.json`.
///
/// # Arguments
/// * `manifest` - The downloads `manifest.json`. An Android entry has `platform: "android"`, `name`, `bytes`,
///   `sha256`, `built`, `versionName`, `versionCode`, `notes` (that version's changelog lines) and `history`
///   (`versionName`, `versionCode`, `date` and `notes` of every version, newest first).
///
/// # Returns
/// `{versionName, versionCode, url, bytes, sha256, built, notes, history}`, with `url` the package's path on this
/// server; `None` when the manifest lists no Android package.
pub fn android_release(manifest: &Value) -> Option<Value> {
    let file = newest_android(manifest)?;
    let list = |key: &str| if file.get(key).as_array().is_some() { file.get(key).clone() } else { Value::Array(Vec::new()) };
    Some(
        obj! {
            "versionName" => file.get("versionName").clone(),
            "versionCode" => file.get("versionCode").clone(),
            "url" => format!("/download/{}", encode_uri_component(&file.get("name").to_js_string())),
            "bytes" => file.get("bytes").clone(),
            "sha256" => file.get("sha256").clone(),
            "built" => file.get("built").clone(),
            "notes" => list("notes"),
            "history" => list("history"),
        }
        .into(),
    )
}

// The newest extension entry by `built`; the publish script keeps only one.
fn newest_extension(manifest: &Value) -> Option<&Value> {
    let files = manifest.get("files").as_array()?;
    files.iter().filter(|file| is_extension(file)).max_by(|a, b| a.get("built").to_js_string().cmp(&b.get("built").to_js_string()))
}

// The Android and Chrome extension sections and the Linux heading after them; empty when the manifest lists neither,
// so the page stays as it was.
fn app_sections(manifest: &Value) -> String {
    let (android, extension) = (android_section(manifest), extension_section(manifest));
    if android.is_empty() && extension.is_empty() { String::new() } else { format!("{android}{extension}<h2>Linux</h2>\n") }
}

// The page's Android section; empty without an Android package.
fn android_section(manifest: &Value) -> String {
    let Some(file) = newest_android(manifest) else { return String::new() };
    let notes: String = file.get("notes").as_array().into_iter().flatten().map(|note| format!("<span>- {}</span>", escape(&note.to_js_string()))).collect();
    format!(
        "<h2>Android</h2>
<ul><li><a href=\"/download/{}\">{}</a>
    <span>Version {} ({}) · {} · built {}</span><code>sha256 {}</code>{notes}</li></ul>
<p>The first time, Android asks you to allow installs from this browser. After that, the app updates itself.</p>
",
        encode_uri_component(&file.get("name").to_js_string()),
        escape(&file.get("label").to_js_string()),
        escape(&file.get("versionName").to_js_string()),
        escape(&file.get("versionCode").to_js_string()),
        size(file.get("bytes")),
        escape(&file.get("built").as_str().map(|text| slice(text, 0, Some(10))).unwrap_or_default()),
        escape(&file.get("sha256").to_js_string()),
    )
}

// The page's Chrome extension section; empty without an extension.
fn extension_section(manifest: &Value) -> String {
    let Some(file) = newest_extension(manifest) else { return String::new() };
    format!(
        "<h2>Chrome extension</h2>
<ul><li><a href=\"/download/{}\">{}</a>
    <span>Version {} · {} · built {}</span><code>sha256 {}</code></li></ul>
<p>Unzip it, open <code>chrome://extensions</code>, turn on Developer mode, choose Load unpacked and pick the \
<code>mymind-extension</code> folder. To update, unzip the new version over that folder and press Reload on the \
extension. Sign in to this site in the same browser; the extension saves to that library.</p>
",
        encode_uri_component(&file.get("name").to_js_string()),
        escape(&file.get("label").to_js_string()),
        escape(&file.get("version").to_js_string()),
        size(file.get("bytes")),
        escape(&file.get("built").as_str().map(|text| slice(text, 0, Some(10))).unwrap_or_default()),
        escape(&file.get("sha256").to_js_string()),
    )
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

/// The download page for the packaged apps: the newest Android package with its version and changelog and the newest
/// Chrome extension when there are ones, then every desktop package with its size and checksum, and how to install
/// each.
///
/// # Arguments
/// * `manifest` - The `manifest.json` that `deploy/package-app.sh` writes: optional `library` and a `files` array of
///   `name`, `label`, `bytes`, `built` and `sha256`, plus the Android entries [`android_release`] describes and
///   Chrome extension entries (`platform: "chrome-extension"` with a `version`).
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
        if is_android(file) || is_extension(file) {
            continue;
        }
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
    let rows = match (rows.is_empty(), newest_android(manifest).is_some() || newest_extension(manifest).is_some()) {
        (false, _) => rows,
        (true, false) => "<li>No app has been packaged yet. Run <code>deploy/package-app.sh</code> on the host.</li>".into(),
        (true, true) => "<li>No Linux app has been packaged yet. Run <code>deploy/package-app.sh</code> on the host.</li>".into(),
    };
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
{}<ul>{rows}</ul>
<pre>tar -xzf mymind-linux-x86_64.tar.gz
mymind/install.sh        # ~/.local/bin/mymind and a launcher entry</pre>
<p>Needs WebKitGTK 4.1 and GStreamer's good plugins (Arch: <code>webkit2gtk-4.1 gst-plugins-good</code>).</p>
<p>Signed in as {} · <a href=\"/signout\">Sign out</a></p>
</main></body></html>",
        escape(&library),
        app_sections(manifest),
        escape(user)
    ))
}
