// Desktop app: a Tauri window around the web UI that the Rust server (`mymind serve`) serves, starting the server when
// none is running, or around a hosted library when MYMIND_URL is set (at run time, or at build time for the
// downloadable app). Env: PORT (default 4331), MYMIND_URL, MYMIND_ROOT (repo checkout), MYMIND_SERVER (the server
// binary); STORE_* pass through to the server.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod file_drop;

use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, RunEvent, Url, WebviewUrl, WebviewWindowBuilder};

struct Server(Mutex<Option<Child>>);

/// True when something accepts connections on the replica's port.
fn listening(port: u16) -> bool {
    TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(200)).is_ok()
}

/// The checkout whose library the app serves.
fn checkout() -> PathBuf {
    let root = std::env::var_os("MYMIND_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")));
    root.canonicalize().unwrap_or(root)
}

/// Names the library files the server cannot start without that are missing from `root`.
fn missing_library(root: &Path) -> Vec<&'static str> {
    const REQUIRED: [&str; 1] = ["mirror/static"];
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

/// The server binary: `MYMIND_SERVER`, else the checkout's release build.
fn server_binary(root: &Path) -> PathBuf {
    std::env::var_os("MYMIND_SERVER").map(PathBuf::from).unwrap_or_else(|| root.join("rust/target/release/mymind"))
}

/// Restores `data/` from the local `library/` snapshot (written by `npm run bundle`) when it has no database.
fn prepare(root: &Path) -> Result<(), String> {
    let (bundle, data) = (root.join("library"), root.join("data"));
    if data.join("mymind.db").exists() || !bundle.join("mymind.db").exists() {
        return Ok(());
    }
    let failed = |err: std::io::Error| format!("Could not install the bundled library into {}: {err}", data.display());
    std::fs::create_dir_all(&data).map_err(failed)?;
    std::fs::copy(bundle.join("mymind.db"), data.join("mymind.db")).map_err(failed)?;
    if bundle.join("media").is_dir() {
        copy_dir(&bundle.join("media"), &data.join("media")).map_err(failed)?;
    }
    Ok(())
}

/// Copies the directory `from` into `to`, recursively.
fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Starts `mymind serve` in `root` and waits until it listens.
fn start_server(root: &Path, server: &Path, port: u16) -> Result<Child, String> {
    let mut command = Command::new(server);
    command.arg("serve").current_dir(root).env("PORT", port.to_string());
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
        format!(
            "Could not run {} in {}: {err}. Build it with rust/fetch-libvips.sh and `cargo build --release` in rust/, or set \
             MYMIND_SERVER or MYMIND_ROOT.",
            server.display(),
            root.display()
        )
    })?;
    let started = Instant::now();
    while !listening(port) {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "The server exited ({status}) before listening on port {port}. Run `{} serve` in {} to see why.",
                server.display(),
                root.display()
            ));
        }
        if started.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            return Err(format!("The server did not listen on port {port} within 60 s."));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(child)
}

/// Stops the server with SIGTERM, then waits for it.
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

/// The hosted library to open when no local server runs: `MYMIND_URL` at run time, else at build time.
fn hosted_library() -> Option<Url> {
    let url = std::env::var("MYMIND_URL").ok().or_else(|| option_env!("MYMIND_URL").map(String::from))?;
    Url::parse(&url).ok().filter(|url| matches!(url.scheme(), "http" | "https"))
}

/// True for pages of the library (local replica or `hosted`) and for the about:, blob: and data: URLs the UI creates.
fn is_library(url: &Url, port: u16, hosted: Option<&Url>) -> bool {
    match url.scheme() {
        "about" | "blob" | "data" => true,
        "http" | "https" => {
            hosted.is_some_and(|hosted| hosted.origin() == url.origin())
                || (matches!(url.host_str(), Some("localhost" | "127.0.0.1")) && url.port_or_known_default() == Some(port))
        }
        _ => false,
    }
}

/// True for the Google pages a sign-in passes through (google.com, google.ro, google.co.uk and their subdomains).
fn is_google_sign_in(url: &Url) -> bool {
    let Some(host) = url.host_str().filter(|_| url.scheme() == "https") else {
        return false;
    };
    let labels: Vec<&str> = host.split('.').collect();
    host == "accounts.youtube.com"
        || labels.iter().rposition(|label| *label == "google").is_some_and(|at| match &labels[at + 1..] {
            ["com"] => true,
            [country] | ["co" | "com", country] => country.len() == 2,
            _ => false,
        })
}

/// True for Cloudflare Access login pages (`<team>.cloudflareaccess.com`), which a hosted library behind Access
/// redirects to before it loads.
fn is_access_login(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str().is_some_and(|host| host.ends_with(".cloudflareaccess.com"))
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
        let server = server_binary(&root);
        let prepared = check_library(&root).and_then(|()| prepare(&root));
        let main = handle.clone();
        // The server is spawned on the main thread: PR_SET_PDEATHSIG fires when the spawning thread exits.
        let _ = handle.run_on_main_thread(move || {
            let url = match prepared.and_then(|()| start_server(&root, &server, port)) {
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

/// Whether a sign-in started from the library or its Cloudflare Access login is under way, so its Google pages stay
/// in the window.
struct SigningIn(AtomicBool);

fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4331);
    let hosted = hosted_library();
    let app = tauri::Builder::default()
        .setup(move |app| {
            app.manage(Server(Mutex::new(None)));
            let signing_in = Arc::new(SigningIn(AtomicBool::new(false)));
            let running = listening(port);
            let url = match (&hosted, running) {
                (_, true) => replica_url(port),
                (Some(hosted), false) => hosted.join("/everything")?,
                (None, false) => {
                    page("Starting mymind", "Starting the library on this computer.")
                }
            };
            let (navigating, opening) = (hosted.clone(), hosted.clone());
            let (leaving, entering) = (signing_in.clone(), signing_in);
            let handle = app.handle().clone();
            #[cfg_attr(not(target_os = "linux"), allow(unused_variables))]
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("mymind")
                .inner_size(1440.0, 900.0)
                .min_inner_size(400.0, 500.0)
                // Let the page's own drop handler receive dragged files, as in a browser.
                .disable_drag_drop_handler()
                .on_navigation(move |url| {
                    if is_library(url, port, navigating.as_ref()) {
                        if !url.path().starts_with("/google/") {
                            leaving.0.store(false, Ordering::SeqCst);
                        }
                        return true;
                    }
                    if is_access_login(url) {
                        leaving.0.store(true, Ordering::SeqCst);
                        return true;
                    }
                    if leaving.0.load(Ordering::SeqCst) && is_google_sign_in(url) {
                        return true;
                    }
                    open_in_browser(url);
                    false
                })
                .on_new_window(move |url, _| {
                    if is_library(&url, port, opening.as_ref()) && url.path() == "/google/oauth2-redirect" {
                        // mymind signs in through a popup; running it in the window keeps the session cookie here.
                        entering.0.store(true, Ordering::SeqCst);
                        let window = handle.clone();
                        let _ = handle.run_on_main_thread(move || {
                            if let Some(main) = window.get_webview_window("main") {
                                let _ = main.navigate(url);
                            }
                        });
                    } else {
                        open_in_browser(&url);
                    }
                    NewWindowResponse::Deny
                })
                .build()?;
            #[cfg(target_os = "linux")]
            file_drop::attach(&window)?;
            if !running && hosted.is_none() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn library_pages_are_the_local_replica_or_the_hosted_origin() {
        let hosted = url("https://mymind.example.com");
        assert!(is_library(&url("http://localhost:4331/everything"), 4331, None));
        assert!(is_library(&url("http://127.0.0.1:4331/cards/x"), 4331, Some(&hosted)));
        assert!(!is_library(&url("http://localhost:4330/everything"), 4331, None));
        assert!(is_library(&url("https://mymind.example.com/google/oauth2-callback?code=x"), 4331, Some(&hosted)));
        assert!(!is_library(&url("http://mymind.example.com/everything"), 4331, Some(&hosted)));
        assert!(!is_library(&url("https://mymind.example.com.evil.test/"), 4331, Some(&hosted)));
        assert!(!is_library(&url("https://mymind.example.com/everything"), 4331, None));
    }

    #[test]
    fn prepare_installs_the_bundled_library_once() {
        let root = std::env::temp_dir().join(format!("mymind-desktop-prepare-{}", std::process::id()));
        std::fs::create_dir_all(root.join("library/media/ab")).unwrap();
        std::fs::write(root.join("library/mymind.db"), "bundled").unwrap();
        std::fs::write(root.join("library/media/ab/cover.webp"), "image").unwrap();
        prepare(&root).unwrap();
        assert_eq!(std::fs::read_to_string(root.join("data/mymind.db")).unwrap(), "bundled");
        assert_eq!(std::fs::read_to_string(root.join("data/media/ab/cover.webp")).unwrap(), "image");
        std::fs::write(root.join("data/mymind.db"), "edited").unwrap();
        prepare(&root).unwrap();
        assert_eq!(std::fs::read_to_string(root.join("data/mymind.db")).unwrap(), "edited");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn access_login_hosts() {
        assert!(is_access_login(&url("https://team.cloudflareaccess.com/cdn-cgi/access/login/mymind.example.com")));
        for bad in ["http://team.cloudflareaccess.com/", "https://cloudflareaccess.com.evil.test/", "https://evilcloudflareaccess.com/"] {
            assert!(!is_access_login(&url(bad)), "{bad}");
        }
    }

    #[test]
    fn google_sign_in_hosts() {
        for ok in ["https://accounts.google.com/o/oauth2/v2/auth", "https://www.google.ro/", "https://accounts.google.co.uk/", "https://accounts.youtube.com/accounts/SetSID"] {
            assert!(is_google_sign_in(&url(ok)), "{ok}");
        }
        for bad in ["http://accounts.google.com/", "https://google.evil.test/", "https://accounts.google.com.evil.test/", "https://evilgoogle.com/"] {
            assert!(!is_google_sign_in(&url(bad)), "{bad}");
        }
    }
}
