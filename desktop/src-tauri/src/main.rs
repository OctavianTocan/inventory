// Desktop app: a Tauri window around the web UI that server.mjs serves, starting the server when none is running.
// Env: PORT (default 4321), MYMIND_ROOT (repo checkout), MYMIND_NODE (node binary); STORE_* pass through to the server.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod file_drop;

use std::ffi::{OsStr, OsString};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, RunEvent, Url, WebviewUrl, WebviewWindowBuilder};

struct Server(Mutex<Option<Child>>);

/// True when something accepts connections on the replica's port.
fn listening(port: u16) -> bool {
    TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(200)).is_ok()
}

/// The checkout whose `server.mjs` the app runs.
fn checkout() -> PathBuf {
    let root = std::env::var_os("MYMIND_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")));
    root.canonicalize().unwrap_or(root)
}

/// Names the library files `server.mjs` cannot start without that are missing from `root`.
fn missing_library(root: &Path) -> Vec<&'static str> {
    const REQUIRED: [&str; 6] = [
        "capture/meta.json",
        "capture/shell.html",
        "capture/cards.json",
        "capture/responses.json",
        "capture/details.json",
        "mirror/static",
    ];
    REQUIRED.into_iter().filter(|file| !root.join(file).exists()).collect()
}

/// Fails with a readable message when `root` lacks part of the library.
fn check_library(root: &Path) -> Result<(), String> {
    let missing = missing_library(root);
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} has no copy of your library: {} {} missing. The library is part of the Git repository; run \
         `git pull` in that checkout, or point MYMIND_ROOT at a complete one.",
        root.display(),
        missing.join(", "),
        if missing.len() == 1 { "is" } else { "are" }
    ))
}

/// PATH with the directory of `node` first, so npm's `#!/usr/bin/env node` runs the same node.
fn path_with(node: &OsStr) -> OsString {
    let mut dirs: Vec<PathBuf> = Path::new(node).parent().filter(|dir| !dir.as_os_str().is_empty()).map(Path::to_path_buf).into_iter().collect();
    dirs.extend(std::env::var_os("PATH").map(|path| std::env::split_paths(&path).collect::<Vec<_>>()).unwrap_or_default());
    std::env::join_paths(dirs).unwrap_or_default()
}

/// Installs npm dependencies on a fresh checkout and restores `data/` from a local `library/` snapshot when it has none.
fn prepare(root: &Path, node: &OsStr) -> Result<(), String> {
    let (program, args): (&OsStr, &[&str]) = if root.join("node_modules").exists() {
        (node, &["tools/install-library.mjs"])
    } else {
        // postinstall runs tools/install-library.mjs.
        (OsStr::new("npm"), &["ci", "--no-audit", "--no-fund"])
    };
    let shown = format!("{} {}", program.to_string_lossy(), args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .env("PATH", path_with(node))
        // With a system libvips installed, sharp would otherwise try to compile itself against it and fail.
        .env("SHARP_IGNORE_GLOBAL_LIBVIPS", "1")
        .status()
        .map_err(|err| format!("Could not run {shown} in {}: {err}. Set MYMIND_NODE to the full path of node.", root.display()))?;
    if !status.success() {
        return Err(format!("{shown} failed ({status}) in {}. Run it there to see why.", root.display()));
    }
    Ok(())
}

/// Starts `node server.mjs` in `root` and waits until it listens.
fn start_server(root: &Path, node: &OsStr, port: u16) -> Result<Child, String> {
    let mut command = Command::new(node);
    command.arg("server.mjs").current_dir(root).env("PORT", port.to_string());
    // The server must not outlive the app, however the app ends.
    #[cfg(target_os = "linux")]
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            Ok(())
        });
    }
    let mut child = command.spawn().map_err(|err| {
        format!("Could not run {} server.mjs in {}: {err}. Set MYMIND_NODE or MYMIND_ROOT.", node.to_string_lossy(), root.display())
    })?;
    let started = Instant::now();
    while !listening(port) {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "server.mjs exited ({status}) before listening on port {port}. Run `node server.mjs` in {} to see why.",
                root.display()
            ));
        }
        if started.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            return Err(format!("server.mjs did not listen on port {port} within 60 s."));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(child)
}

/// Stops the server with SIGTERM so it can close its browsers, then waits for it.
fn stop_server(mut child: Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
    }
    #[cfg(not(unix))]
    let _ = child.kill();
    let _ = child.wait();
}

/// A `data:` URL of a page with a heading and a message.
fn page(title: &str, message: &str) -> Url {
    let text = message.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let html = format!("<body style=\"font:15px/1.5 system-ui;padding:40px;max-width:720px\"><h3>{title}</h3><p>{text}</p>");
    let mut url = String::from("data:text/html;charset=utf-8,");
    for byte in html.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    Url::parse(&url).expect("an escaped data: URL parses")
}

/// True for pages of the replica itself and for the about:, blob: and data: URLs the UI creates.
fn is_replica(url: &Url, port: u16) -> bool {
    match url.scheme() {
        "about" | "blob" | "data" => true,
        "http" | "https" => {
            matches!(url.host_str(), Some("localhost" | "127.0.0.1")) && url.port_or_known_default() == Some(port)
        }
        _ => false,
    }
}

/// Hands web and mail links to the system; anything else (a file dropped outside a drop zone) goes nowhere.
fn open_in_browser(url: &Url) {
    if !matches!(url.scheme(), "http" | "https" | "mailto") {
        return;
    }
    if let Err(err) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
        eprintln!("could not open {url}: {err}");
    }
}

/// Prepares the checkout and starts the server off the main thread, then shows the replica or the error.
fn start_in_background(handle: AppHandle, port: u16) {
    std::thread::spawn(move || {
        let root = checkout();
        let node = std::env::var_os("MYMIND_NODE").unwrap_or_else(|| "node".into());
        let prepared = check_library(&root).and_then(|()| prepare(&root, &node));
        let main = handle.clone();
        // The server is spawned on the main thread: PR_SET_PDEATHSIG fires when the spawning thread exits.
        let _ = handle.run_on_main_thread(move || {
            let url = match prepared.and_then(|()| start_server(&root, &node, port)) {
                Ok(child) => {
                    *main.state::<Server>().0.lock().unwrap() = Some(child);
                    replica_url(port)
                }
                Err(message) => {
                    eprintln!("{message}");
                    page("mymind could not start", &message)
                }
            };
            if let Some(window) = main.get_webview_window("main") {
                let _ = window.navigate(url);
            }
        });
    });
}

/// The replica's library page.
fn replica_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{port}/everything")).expect("a localhost URL parses")
}

fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4321);
    let app = tauri::Builder::default()
        .setup(move |app| {
            app.manage(Server(Mutex::new(None)));
            let running = listening(port);
            let url = if running {
                replica_url(port)
            } else {
                page("Starting mymind", "The first start on a new machine installs dependencies, which takes about a minute.")
            };
            #[cfg_attr(not(target_os = "linux"), allow(unused_variables))]
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("mymind")
                .inner_size(1440.0, 900.0)
                .min_inner_size(400.0, 500.0)
                // Let the page's own drop handler receive dragged files, as in a browser.
                .disable_drag_drop_handler()
                .on_navigation(move |url| {
                    let inside = is_replica(url, port);
                    if !inside {
                        open_in_browser(url);
                    }
                    inside
                })
                .on_new_window(move |url, _| {
                    open_in_browser(&url);
                    NewWindowResponse::Deny
                })
                .build()?;
            #[cfg(target_os = "linux")]
            file_drop::attach(&window)?;
            if !running {
                start_in_background(app.handle().clone(), port);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the mymind app");
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(child) = handle.state::<Server>().0.lock().unwrap().take() {
                stop_server(child);
            }
        }
    });
}
