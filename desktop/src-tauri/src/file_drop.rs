// WebKitGTK hands pages an empty `dataTransfer.files` for files dropped from other apps
// (https://bugs.webkit.org/show_bug.cgi?id=323277), so those drops are taken here and replayed to the page.
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gtk::gio;
use gtk::gio::prelude::FileExt;
use gtk::prelude::*;
use tauri::WebviewWindow;

/// Delivers files dropped on `window` from other apps to its page as a DOM `drop` event, as a browser would.
/// Drags that start inside the page are left to WebKit. Fails when the window has no webview.
pub fn attach(window: &WebviewWindow) -> tauri::Result<()> {
    let handle = window.clone();
    window.with_webview(move |webview| connect(&webview.inner(), handle))
}

/// Takes over drops of local files from other apps on `webview`.
fn connect(webview: &webkit2gtk::WebView, window: WebviewWindow) {
    let pending: Rc<RefCell<Option<(gtk::gdk::DragContext, Vec<PathBuf>)>>> = Rc::default();
    let received = pending.clone();
    webview.connect_drag_data_received(move |_, context, _, _, data, _, _| {
        if context.drag_get_source_widget().is_some() || data.target().name() != "text/uri-list" {
            return;
        }
        let paths: Option<Vec<PathBuf>> = data.uris().iter().map(|uri| gio::File::for_uri(uri).path()).collect();
        *received.borrow_mut() = paths.filter(|paths| !paths.is_empty()).map(|paths| (context.clone(), paths));
    });
    webview.connect_drag_drop(move |_, context, x, y, time| {
        let Some((source, paths)) = pending.borrow_mut().take() else { return false };
        if &source != context {
            return false;
        }
        context.drag_finish(true, false, time);
        let window = window.clone();
        std::thread::spawn(move || deliver(&window, &paths, x, y));
        true
    });
}

/// Streams the files into the page, then dispatches `drop` on the element at (x, y) as a browser would.
fn deliver(window: &WebviewWindow, paths: &[PathBuf], x: i32, y: i32) {
    static DROPS: AtomicU64 = AtomicU64::new(0);
    let drop_id = DROPS.fetch_add(1, Ordering::Relaxed);
    let mut files = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        let key = format!("{drop_id}-{index}");
        match stream_file(window, path, &key) {
            Ok(modified) => files.push(serde_json::json!({
                "key": key,
                "name": path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default(),
                "type": mime_type(path),
                "modified": modified,
            })),
            Err(err) => eprintln!("could not read dropped file {}: {err}", path.display()),
        }
    }
    if files.is_empty() {
        return;
    }
    let script = format!(
        r#"(() => {{
  const parts = window.__mymindDrop ?? {{}};
  const transfer = new DataTransfer();
  for (const file of {files}) {{
    transfer.items.add(new File(parts[file.key] ?? [], file.name, {{ type: file.type, lastModified: file.modified }}));
    delete parts[file.key];
  }}
  const target = document.elementFromPoint({x}, {y}) ?? document.body;
  target.dispatchEvent(new DragEvent('drop', {{ bubbles: true, cancelable: true, composed: true, clientX: {x}, clientY: {y}, dataTransfer: transfer }}));
}})()"#,
        files = serde_json::Value::Array(files)
    );
    let _ = window.eval(script);
}

/// Sends one file to the page in base64 chunks under `key`; returns its modification time in ms.
fn stream_file(window: &WebviewWindow, path: &Path, key: &str) -> std::io::Result<u64> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(std::io::Error::other("not a regular file"));
    }
    let mut chunk = vec![0; 3 << 20];
    loop {
        let read = file.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        let encoded = STANDARD.encode(&chunk[..read]);
        window
            .eval(format!(r#"((window.__mymindDrop ??= {{}})["{key}"] ??= []).push(Uint8Array.fromBase64("{encoded}"))"#))
            .map_err(std::io::Error::other)?;
    }
    let modified = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok());
    Ok(modified.map_or(0, |since| since.as_millis() as u64))
}

/// The MIME type a browser would give the file, from its name; empty when unknown.
fn mime_type(path: &Path) -> String {
    let (content_type, _) = gio::content_type_guess(Some(path), &[]);
    gio::content_type_get_mime_type(&content_type)
        .filter(|mime| mime != "application/octet-stream")
        .map(String::from)
        .unwrap_or_default()
}
