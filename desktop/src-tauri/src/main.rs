// Desktop app: a Tauri window around the web UI that server.mjs serves, starting the server when none is running.
// Env: PORT (default 4321), MYMIND_ROOT (repo checkout), MYMIND_NODE (node binary); STORE_* pass through to the server.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod file_drop;

use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::webview::NewWindowResponse;
use tauri::{Manager, RunEvent, Url, WebviewUrl, WebviewWindowBuilder};

struct Server(Mutex<Option<Child>>);

/// True when something accepts connections on the replica's port.
fn listening(port: u16) -> bool {
    TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(200)).is_ok()
}

/// Starts `node server.mjs` in the repo and waits until it listens.
fn start_server(port: u16) -> Result<Child, String> {
    let root = std::env::var_os("MYMIND_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")));
    let node = std::env::var_os("MYMIND_NODE").unwrap_or_else(|| "node".into());
    let mut command = Command::new(&node);
    command.arg("server.mjs").current_dir(&root).env("PORT", port.to_string());
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
            return Err(format!("server.mjs exited ({status}) before listening on port {port}."));
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

fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4321);
    let app = tauri::Builder::default()
        .setup(move |app| {
            let started = if listening(port) { Ok(None) } else { start_server(port).map(Some) };
            let url = match &started {
                Ok(_) => Url::parse(&format!("http://localhost:{port}/everything"))?,
                Err(message) => {
                    eprintln!("{message}");
                    let page = format!("<body style=\"font:15px system-ui;padding:40px\"><h3>mymind could not start</h3><p>{message}</p>");
                    Url::parse(&format!("data:text/html;charset=utf-8,{}", page.replace('%', "%25").replace('#', "%23")))?
                }
            };
            app.manage(Server(Mutex::new(started.unwrap_or(None))));
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
