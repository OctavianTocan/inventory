//! `mymind`: the replica server of `server.mjs`, configured by the same environment variables and `.env`;
//! `mymind download-server`, the app downloads of `deploy/download-server.mjs`; `mymind container`, the media
//! container of `worker/container/server.mjs`; and the operational tools of `tools/` (`seed-db`, `auth-user`,
//! `import-hosted`, `upload-hosted`, `embed-library`, `search-recordings`, `try-ai`, `capture-fixtures`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use mm_core::api::{Api, ApiOptions, Push};
use mm_core::auth::{Auth, AuthOptions};
use mm_core::host::{Fetch, Screenshot};
use mm_core::hosted::{Hosted, HostedOptions};
use mm_core::hub::Hub;
use mm_core::site::{LibraryApi, Site};
use mm_core::store::Store;
use mm_sqlite::Sqlite;
use mymind::bucket_fs::DirectoryBucket;
use mymind::capture_http::CaptureTransport;
use mymind::container::Container;
use mymind::download::Downloads;
use mymind::env_file::{self, Loaded};
use mymind::fetch::NativeFetch;
use mymind::http::serve;
use mymind::local_media::LocalMedia;
use mymind::media_processor::LocalProcessor;
use mymind::node_path::{listen_error, resolve};
use mymind::read_metadata::import_read_metadata;
use mymind::replica::{Options, Replica};
use mymind::screenshot::ChromiumScreenshot;
use mymind::serve::{FilePages, Listener, Server};
use mymind::uploads_fs::DirectoryUploads;
use tokio::net::TcpListener;

// `Number(text)` as a port for `server.listen`, which throws for anything but an integer below 65536.
fn port(text: &str) -> Result<u16, String> {
    let number = mm_js::number::string_to_number(text);
    if number.fract() == 0.0 && (0.0..65536.0).contains(&number) {
        Ok(number as u16)
    } else {
        Err(format!("RangeError [ERR_SOCKET_BAD_PORT]: options.port should be >= 0 and < 65536. Received type number ({}).", mm_js::number::number_to_string(number)))
    }
}

fn environment(root: &Path) -> Result<HashMap<String, String>, String> {
    let process: Vec<(String, String)> = std::env::vars_os().map(|(name, value)| (name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned())).collect();
    let skip = process.iter().any(|(name, value)| name == "STORE_LOAD_ENV" && value == "0");
    let file = if skip {
        HashMap::new()
    } else {
        match env_file::load(&env_file::path(root)) {
            Ok(Loaded::Vars(vars)) => vars,
            Ok(Loaded::Missing) => HashMap::new(),
            Err(_) => return Err("Error: Could not load server environment".to_owned()),
        }
    };
    Ok(env_file::merged(process, file))
}

async fn listen(port: u16) -> Result<TcpListener, String> {
    TcpListener::bind(("127.0.0.1", port)).await.map_err(|error| listen_error(&error, "127.0.0.1", port))
}

async fn start(root: PathBuf) -> Result<(), String> {
    let env = environment(&root)?;
    mm_core::runtime::set_env(env.clone());
    let var = |name: &str| env.get(name).map(String::as_str);
    let main_port = port(var("PORT").unwrap_or("4321"))?;
    let offline = var("OFFLINE") == Some("1");
    let hosted = var("HOSTED") == Some("1");
    let fetch: Rc<dyn Fetch> = Rc::new(NativeFetch::new());
    let replica = Rc::new(Replica::load(&Options { root: root.clone(), offline, hosted }, Rc::clone(&fetch)).await.map_err(|error| error.to_string())?);
    if !hosted {
        mm_core::runtime::spawn(Rc::clone(&replica).watch());
    }
    let auth = if mm_core::auth::enabled(&env).map_err(|error| format!("Error: {error}"))? {
        let file = var("AUTH_DB").map_or_else(|| root.join("data").join("auth.db"), PathBuf::from);
        let db = Rc::new(mm_sqlite::Sqlite::open(&file, false).map_err(|error| error.to_string())?);
        let options = AuthOptions::new(env.clone(), main_port, db, Rc::clone(&fetch), Rc::new(FilePages::new(&root)));
        Auth::new(options).map_err(|error| format!("Error: {error}"))?.map(Rc::new)
    } else {
        None
    };
    let hubs: Rc<RefCell<Vec<Rc<Hub>>>> = Rc::default();
    let push = || -> Push {
        let hubs = Rc::clone(&hubs);
        Box::new(move |target, payload| {
            for hub in hubs.borrow().iter() {
                let _ = hub.broadcast(target, payload);
            }
        })
    };
    let transport = Rc::new(CaptureTransport::new());
    let chromium = var("VERIFIER_CHROMIUM").unwrap_or("/usr/lib/chromium/chromium");
    let screenshot: Rc<dyn Screenshot> = Rc::new(ChromiumScreenshot::new(PathBuf::from(chromium), Rc::clone(&transport)));
    let readonly = var("STORE_READONLY") == Some("1");
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let mut api: Option<Rc<dyn LibraryApi>> = None;
    let mut site: Option<Rc<dyn Site>> = None;
    let mut library = String::new();
    if hosted {
        let directory = resolve(&root, var("HOSTED_DIR").unwrap_or("data/hosted"));
        library = var("HOSTED_LIBRARY").unwrap_or("owner").to_owned();
        let file = directory.join(format!("{library}.db"));
        if !file.exists() {
            return Err(format!("Error: No hosted library at {}; run tools/import-hosted.mjs first", file.display()));
        }
        let db = Rc::new(Sqlite::open(&file, false).map_err(|error| error.to_string())?);
        let options = HostedOptions {
            db,
            bucket: Rc::new(DirectoryBucket::new(directory.join("bucket"))),
            library: library.clone(),
            processor: Rc::new(LocalProcessor::new(None)),
            push: push(),
            readonly,
            offline,
            screenshot,
            resource: transport,
            fetch: Rc::clone(&fetch),
            random: mm_core::runtime::random_bytes,
        };
        site = Some(Rc::new(Hosted::new(options).await.map_err(|error| error.to_js_string())?));
    } else if let Some(database) = var("STORE_DB").map_or_else(|| Some(root.join("data/mymind.db")), |path| (!path.is_empty()).then(|| resolve(&cwd, path)))
        && database.exists()
    {
        let store = Rc::new(Store::new(Rc::new(Sqlite::open(&database, false).map_err(|error| error.to_string())?)).map_err(|error| error.to_js_string())?);
        import_read_metadata(&store, &root).await.map_err(|error| error.to_js_string())?;
        let folder = database.parent().map(Path::to_path_buf).unwrap_or_default();
        let media = var("STORE_MEDIA").map_or_else(|| folder.join("media"), |path| resolve(&cwd, path));
        let has_replay = {
            let replica = Rc::clone(&replica);
            Box::new(move |request: &mm_core::http::Request| {
                let detail = request.pathname().strip_prefix("/cards/").and_then(|rest| rest.strip_suffix("/detail")).filter(|id| !id.is_empty() && !id.contains('/'));
                Ok(replica.replayed_for(request, &request.url).is_some() || replica.has_detail(detail.unwrap_or("")))
            })
        };
        let options = ApiOptions {
            store,
            media: Rc::new(LocalMedia::new(media)),
            upload_storage: Rc::new(DirectoryUploads::new(folder.join("uploads"))),
            screenshot,
            resource: transport,
            fetch: Rc::clone(&fetch),
            readonly,
            localize: mm_core::site::localize,
            push: push(),
            has_replay,
            random: mm_core::runtime::random_bytes,
        };
        api = Some(Rc::new(Api::new(options).await.map_err(|error| error.to_js_string())?));
    }
    let store_db = var("STORE_DB").filter(|path| !path.is_empty());
    let server = Rc::new(Server {
        replica: Rc::clone(&replica),
        auth: auth.clone(),
        api,
        hosted: site.clone(),
        offline,
        store: store_db.map(|path| resolve(&cwd, path)),
    });
    let local_port = match (&auth, var("LOCAL_PORT").filter(|text| !text.is_empty())) {
        (Some(_), Some(text)) => Some(port(text)?),
        _ => None,
    };
    let main = listen(main_port).await?;
    let local = match local_port {
        Some(local_port) => Some(listen(local_port).await?),
        None => None,
    };
    let bound = main.local_addr().map_err(|error| error.to_string())?.port();
    println!("mymind replica on http://localhost:{bound}/everything");
    let flag = if offline { ", offline" } else { "" };
    if site.is_some() {
        println!("hosted library {library}{flag}");
    } else {
        let (cards, details, replayed) = replica.counts();
        println!("{cards} cards, {details} detail views, captured {}{flag}", replica.captured_at());
        println!("{replayed} replayed responses from REA scenarios");
    }
    let listener = Rc::new(Listener::new(Rc::clone(&server), true));
    hubs.borrow_mut().push(listener.hub());
    if let Some(local) = local {
        let bound = local.local_addr().map_err(|error| error.to_string())?.port();
        println!("without sign-in for this computer on http://127.0.0.1:{bound}");
        let unsigned = Rc::new(Listener::new(Rc::clone(&server), false));
        hubs.borrow_mut().push(unsigned.hub());
        tokio::task::spawn_local(serve(local, unsigned));
    }
    serve(main, listener).await;
    Ok(())
}

// The download server, configured by the process environment alone as `download-server.mjs` was.
async fn download_server(root: PathBuf) -> Result<(), String> {
    let var = |name: &str| std::env::var_os(name).map(|value| value.to_string_lossy().into_owned());
    let port = port(&var("DOWNLOAD_PORT").unwrap_or_else(|| "4332".to_owned()))?;
    let dir = resolve(&root, &var("DOWNLOAD_DIR").unwrap_or_else(|| "downloads".to_owned()));
    let auth_db = var("AUTH_DB").map_or_else(|| root.join("data").join("auth.db"), PathBuf::from);
    let downloads = Downloads::open(dir, &auth_db).map_err(|error| error.to_string())?;
    let listener = listen(port).await?;
    println!("downloads for signed-in users on http://127.0.0.1:{port}/download");
    serve(listener, Rc::new(downloads)).await;
    Ok(())
}

async fn container() -> Result<(), String> {
    let port = port(&std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned()))?;
    let listener = TcpListener::bind(("0.0.0.0", port)).await.map_err(|error| listen_error(&error, "0.0.0.0", port))?;
    println!("media container on :{port}");
    serve(listener, Rc::new(Container::new())).await;
    Ok(())
}

fn main() -> ExitCode {
    let root = std::env::var_os("MYMIND_ROOT").map(PathBuf::from).or_else(|| std::env::current_dir().ok()).unwrap_or_else(|| PathBuf::from("."));
    let arguments: Vec<String> = std::env::args().skip(2).collect();
    let program = match std::env::args().nth(1).as_deref() {
        Some("capture-fixtures") => return ExitCode::from(mymind::runtime::run(mymind::tool_capture_fixtures::run(&root, &arguments))),
        Some("seed-db") => return ExitCode::from(mymind::runtime::run(mymind::tool_seed_db::run(&root, &arguments))),
        Some("import-hosted") => return ExitCode::from(mymind::runtime::run(mymind::tool_import_hosted::run(&root, &arguments))),
        Some("auth-user") => return ExitCode::from(mymind::runtime::run(mymind::tool_auth_user::run(&root, &arguments))),
        Some("upload-hosted") => return ExitCode::from(mymind::runtime::run(mymind::tool_upload_hosted::run(&root, &arguments))),
        Some("embed-library") => return ExitCode::from(mymind::runtime::run(mymind::tool_embed_library::run(&root, &arguments))),
        Some("try-ai") => return ExitCode::from(mymind::runtime::run(mymind::tool_try_ai::run(&root, &arguments))),
        Some("search-recordings") => return ExitCode::from(mymind::runtime::run(mymind::tool_search_recordings::run(&root, &arguments))),
        None | Some("serve") => mymind::runtime::run(start(root)),
        Some("download-server") => mymind::runtime::run(download_server(root)),
        Some("container") => mymind::runtime::run(container()),
        Some(other) => {
            eprintln!("mymind: unknown command {other}");
            return ExitCode::from(2);
        }
    };
    match program {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
