//! Launch-policy inspection and real loopback TCP/UDP controls from `check-capture-browser.mjs`.

mod capture_support;

use std::cell::Cell;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::rc::Rc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use mm_core::host::Screenshot;
use mm_core::http::{Headers, Request, Response};
use mm_core::{Value, obj};
use mm_js::{base64, json};
use mymind::capture_http::CaptureTransport;
use mymind::http::{Handler, serve};
use mymind::screenshot::ChromiumScreenshot;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
const DOUBLE: &str = r#"#!/usr/bin/python3
import base64,json,os,sys
config=CONFIG
log=open(config['log'],'w',buffering=1)
log.write(json.dumps({'args':sys.argv[1:]})+'\n')
reader=os.fdopen(3,'rb',buffering=0)
writer=os.fdopen(4,'wb',buffering=0)
pending=b''
index=0
def emit(message):
    writer.write(json.dumps(message,separators=(',',':')).encode()+b'\0')
def event():
    global index
    if index<len(config['urls']):
        emit({'method':'Fetch.requestPaused','sessionId':'session','params':{'requestId':str(index),'request':{'method':'GET','url':config['urls'][index],'headers':{}}}})
        index+=1
    else:
        emit({'method':'Page.lifecycleEvent','sessionId':'session','params':{'name':'DOMContentLoaded','frameId':'frame','loaderId':'loader'}})
while True:
    chunk=reader.read(65536)
    if not chunk: break
    pending+=chunk
    while b'\0' in pending:
        raw,pending=pending.split(b'\0',1)
        command=json.loads(raw)
        method=command['method']; params=command.get('params',{})
        observed={'method':method,'params':dict(params)}
        if 'body' in observed['params']:
            body=base64.b64decode(observed['params'].pop('body'))
            observed['bytes']=len(body)
            if len(body)<1024: observed['body']=base64.b64encode(body).decode()
        log.write(json.dumps(observed)+'\n')
        result={}
        if method=='Target.createBrowserContext': result={'browserContextId':'context'}
        elif method=='Target.createTarget': result={'targetId':'target'}
        elif method=='Target.attachToTarget': result={'sessionId':'session'}
        elif method=='Browser.getWindowForTarget': result={'windowId':1}
        elif method=='Page.navigate': result={'frameId':'frame','loaderId':'loader'}
        elif method=='Page.getFrameTree': result={'frameTree':{'frame':{'id':'frame'}}}
        elif method=='Page.createIsolatedWorld': result={'executionContextId':1}
        elif method=='Page.getLayoutMetrics': result={'visualViewport':{'pageX':0,'pageY':0,'scale':1}}
        elif method=='Page.captureScreenshot': result={'data':config['png']}
        emit({'id':command['id'],'result':result})
        if method=='Page.navigate' or method in ['Fetch.fulfillRequest','Fetch.failRequest']: event()
        if method=='Browser.close': log.close(); sys.exit(0)
"#;

// Keep browser user-data paths short while resolving them only inside the Cargo scratch directory.
fn relative(path: &Path) -> PathBuf {
    let current = std::env::current_dir().unwrap();
    let target: Vec<_> = path.components().collect();
    let from: Vec<_> = current.components().collect();
    let common = target
        .iter()
        .zip(&from)
        .take_while(|(left, right)| left == right)
        .count();
    let mut result = PathBuf::new();
    for _ in common..from.len() {
        result.push("..");
    }
    for component in &target[common..] {
        result.push(component.as_os_str());
    }
    result
}

// Create one protocol-double executable; it records launch arguments and only answers CDP pipe messages.
fn browser_double(directory: &Path, urls: &[String]) -> (PathBuf, PathBuf) {
    let executable = directory.join("chromium-double");
    let log = directory.join("protocol.jsonl");
    let config: Value = obj! { "log" => log.to_string_lossy().into_owned(), "urls" => Value::Array(urls.iter().map(|url| Value::from(url.as_str())).collect()), "png" => PNG }.into();
    std::fs::write(
        &executable,
        DOUBLE.replace("CONFIG", &json::to_string(&config)),
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    (executable, log)
}

// Read synthetic launch options and CDP commands after the browser has closed.
fn transcript(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| json::parse(line).unwrap())
        .collect()
}

// Start a loopback resource server that counts only requests the native capture transport actually sends.
async fn resources(listener: TcpListener, count: Rc<Cell<usize>>, big: bool) {
    loop {
        let Ok((mut socket, _)) = listener.accept().await else {
            return;
        };
        let count = count.clone();
        tokio::task::spawn_local(async move {
            let mut request = vec![0u8; 8192];
            if socket.read(&mut request).await.is_err() {
                return;
            }
            count.set(count.get() + 1);
            let bytes = if big {
                vec![b'x'; 20 * 1024 * 1024]
            } else {
                base64::decode(PNG)
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: image/png\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                bytes.len()
            );
            if socket.write_all(head.as_bytes()).await.is_ok() {
                let _ = socket.write_all(&bytes).await;
            }
        });
    }
}

struct ProbePage {
    tcp: u16,
    udp: u16,
    seen: Rc<Cell<usize>>,
}

#[async_trait(?Send)]
impl Handler for ProbePage {
    async fn handle(&self, request: Request) -> Response {
        if request.pathname() == "/font" {
            tokio::time::sleep(Duration::from_millis(1200)).await;
            return Response::send(404, "", &Headers::new());
        }
        self.seen.set(self.seen.get() + 1);
        Response::send(
            200,
            format!(
                r#"<!doctype html><link rel="preconnect" crossorigin href="http://127.0.0.1:{}">
<style>@font-face{{font-family:probe;src:url(/font)}}body{{font-family:probe}}</style><p>Offline network probe</p>
<script>window.peer = new RTCPeerConnection({{iceServers:[{{urls:'stun:127.0.0.1:{}'}}]}});
peer.createDataChannel('probe'); peer.createOffer().then(offer=>peer.setLocalDescription(offer));
if(window.WebTransport) new WebTransport('https://127.0.0.1:{}/').closed.catch(()=>{{}});</script>"#,
                self.tcp, self.udp, self.udp
            ),
            &Headers::from([("content-type", "text/html")]),
        )
    }
}

#[test]
fn launch_policy_and_shared_budget_assertions() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _scratch = capture_support::ScratchTemp::new();
    mymind::runtime::run(async {
        let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stage-b-browser-{}-{}",
            std::process::id(),
            base64::hex(&mm_core::runtime::random_bytes(4))
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let resource_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = resource_listener.local_addr().unwrap().port();
        let count = Rc::new(Cell::new(0));
        let task = tokio::task::spawn_local(resources(resource_listener, count.clone(), false));
        let urls: Vec<String> = (0..2)
            .map(|i| format!("http://127.0.0.1:{port}/{i}.png"))
            .collect();
        let (executable, log) = browser_double(&directory, &urls);
        let screenshot = tokio::time::timeout(
            Duration::from_secs(30),
            ChromiumScreenshot::new(executable, Rc::new(CaptureTransport::new()))
                .capture(&format!("http://127.0.0.1:{port}/"), true),
        )
        .await
        .expect("launch double timed out")
        .unwrap();
        assert_eq!(
            screenshot,
            base64::decode(PNG),
            "check-capture-browser.mjs:40-43"
        );
        assert_eq!(
            count.get(),
            2,
            "check-capture-browser.mjs:44 resource count"
        );
        let commands = transcript(&log);
        let args = commands[0].get("args").as_array().unwrap();
        assert!(
            args.contains(&"--proxy-server=http://capture-denied.invalid:9".into()),
            "check-capture-browser.mjs:45"
        );
        for flag in [
            "--proxy-bypass-list=<-loopback>",
            "--disable-quic",
            "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
            "--host-resolver-rules=MAP * ~NOTFOUND",
        ] {
            assert!(
                args.contains(&flag.into()),
                "check-capture-browser.mjs:46 {flag}"
            );
        }
        assert!(
            commands
                .iter()
                .any(|command| command.get("method").as_str() == Some("Browser.close")),
            "check-capture-browser.mjs:47"
        );
        let fulfilled: Vec<_> = commands
            .iter()
            .filter(|command| command.get("method").as_str() == Some("Fetch.fulfillRequest"))
            .collect();
        assert_eq!(
            fulfilled.len(),
            2,
            "check-capture-browser.mjs:44 both intercepted resources fulfilled"
        );
        for command in fulfilled {
            assert_eq!(command.get("body").as_str(), Some(PNG));
        }
        assert!(
            !commands
                .iter()
                .any(|command| command.get("method").as_str() == Some("Fetch.continueRequest")),
            "check-capture-http.mjs:118 HTTP resources are fulfilled through the transport and never passed to native browser fetching"
        );
        task.abort();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Rc::new(Cell::new(0));
        let task = tokio::task::spawn_local(resources(listener, count.clone(), true));
        let urls: Vec<String> = (0..4)
            .map(|i| format!("http://127.0.0.1:{port}/{i}.png"))
            .collect();
        let (executable, log) = browser_double(&directory, &urls);
        tokio::time::timeout(
            Duration::from_secs(30),
            ChromiumScreenshot::new(executable, Rc::new(CaptureTransport::new()))
                .capture(&format!("http://127.0.0.1:{port}/"), true),
        )
        .await
        .expect("shared-budget launch double timed out")
        .unwrap();
        let commands = transcript(&log);
        assert_eq!(
            commands
                .iter()
                .filter(|command| command.get("method").as_str() == Some("Fetch.fulfillRequest"))
                .count(),
            2,
            "check-capture-browser.mjs:44 one budget permits only the first two resources"
        );
        assert_eq!(
            commands
                .iter()
                .filter(|command| command.get("method").as_str() == Some("Fetch.failRequest"))
                .count(),
            2,
            "check-capture-browser.mjs:44 later resources exhaust that same budget"
        );
        assert_eq!(
            count.get(),
            3,
            "check-capture-browser.mjs:44 exhausted budget prevents fourth connection"
        );
        task.abort();
        let denied = ["http://127.0.0.2/private".to_owned()];
        let (executable, log) = browser_double(&directory, &denied);
        let error = tokio::time::timeout(
            Duration::from_secs(30),
            ChromiumScreenshot::new(executable, Rc::new(CaptureTransport::new()))
                .capture(&format!("http://127.0.0.1:{port}/"), true),
        )
        .await
        .expect("blocked-resource launch double timed out")
        .unwrap_err();
        assert!(
            mm_core::capture_http::is_blocked(&error),
            "check-capture-http.mjs:127 one forbidden subresource reports a blocked error"
        );
        let blocked = transcript(&log);
        let rejected = blocked
            .iter()
            .filter(|command| command.get("method").as_str() == Some("Fetch.failRequest"))
            .count();
        assert_eq!(
            rejected, 1,
            "check-capture-http.mjs:127 exactly one forbidden subresource"
        );
        assert_eq!(
            commands
                .iter()
                .filter(|command| command.get("method").as_str() == Some("Fetch.failRequest"))
                .count()
                + rejected,
            3,
            "check-capture-http.mjs:127 two budget aborts plus the forbidden-resource abort"
        );
        assert_eq!(
            count.get(),
            3,
            "forbidden browser subresource cannot open a connection"
        );
        std::fs::remove_dir_all(directory).unwrap();
    });
}

#[test]
fn native_network_positive_and_negative_controls() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _scratch = capture_support::ScratchTemp::new();
    mymind::runtime::run(async {
        let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stage-b-browser-native-{}-{}",
            std::process::id(),
            base64::hex(&mm_core::runtime::random_bytes(4))
        ));
        std::fs::create_dir_all(&directory).unwrap();
        native_controls(&directory).await;
        std::fs::remove_dir_all(directory).unwrap();
    });
}

// Require a positive WebRTC control before asserting that the protected native browser sends no TCP/UDP traffic.
async fn native_controls(directory: &Path) {
    let executable = PathBuf::from(
        std::env::var_os("VERIFIER_CHROMIUM")
            .unwrap_or_else(|| "/usr/lib/chromium/chromium".into()),
    );
    assert!(
        executable.exists(),
        "the original browser probes require a Chromium executable"
    );
    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tcp_port = tcp.local_addr().unwrap().port();
    let tcp_connections = Rc::new(Cell::new(0usize));
    let connections = tcp_connections.clone();
    let tcp_task = tokio::task::spawn_local(async move {
        while let Ok((socket, _)) = tcp.accept().await {
            connections.set(connections.get() + 1);
            drop(socket);
        }
    });
    let udp = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let udp_port = udp.local_addr().unwrap().port();
    let udp_packets = Rc::new(Cell::new(0usize));
    let packets = udp_packets.clone();
    let udp_task = tokio::task::spawn_local(async move {
        let mut packet = [0u8; 65536];
        while udp.recv_from(&mut packet).await.is_ok() {
            packets.set(packets.get() + 1);
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://127.0.0.1:{}/",
        listener.local_addr().unwrap().port()
    );
    let seen = Rc::new(Cell::new(0usize));
    let page_task = tokio::task::spawn_local(serve(
        listener,
        Rc::new(ProbePage {
            tcp: tcp_port,
            udp: udp_port,
            seen: seen.clone(),
        }),
    ));
    let profile = relative(&directory.join("control-profile"));
    let mut control = tokio::process::Command::new(&executable)
        .args([
            "--headless",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--disable-breakpad",
            "--disable-background-networking",
            "--disable-component-update",
            "--no-first-run",
            "--disable-default-apps",
            "--disable-sync",
            "--disable-quic",
            "--proxy-server=http://127.0.0.1:1",
            "--proxy-bypass-list=127.0.0.1,localhost",
            "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1",
        ])
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(&url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(
            std::fs::File::create(directory.join("control.stderr")).unwrap(),
        ))
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while seen.get() == 0 {
        if let Some(status) = control.try_wait().unwrap() {
            panic!(
                "unguarded Chromium control exited {status}: {}",
                std::fs::read_to_string(directory.join("control.stderr")).unwrap()
            );
        }
        assert!(
            Instant::now() < deadline,
            "unguarded control page did not load; inspect {}",
            directory.join("control.stderr").display()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(
        udp_packets.get() > 0,
        "check-capture-browser.mjs:83 unguarded WebRTC positive control must reach UDP sink"
    );
    if tcp_connections.get() == 0 {
        eprintln!(
            "this Chromium does not preconnect to loopback; TCP positive probe was not exercised, as permitted by check-capture-browser.mjs:81-82"
        );
    }
    control.kill().await.unwrap();
    control.wait().await.unwrap();
    tcp_connections.set(0);
    udp_packets.set(0);
    let screenshot = ChromiumScreenshot::new(executable, Rc::new(CaptureTransport::new()))
        .capture(&url, true)
        .await
        .unwrap();
    assert_eq!(&screenshot[1..4], b"PNG", "check-capture-browser.mjs:87");
    assert_eq!(
        tcp_connections.get(),
        0,
        "check-capture-browser.mjs:88 TCP probe"
    );
    assert_eq!(
        udp_packets.get(),
        0,
        "check-capture-browser.mjs:88 UDP probe"
    );
    page_task.abort();
    tcp_task.abort();
    udp_task.abort();
}
