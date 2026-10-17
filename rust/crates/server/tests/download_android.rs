//! `mymind download-server` with an Android package: the app's token reaches `/download/android.json` and the
//! package, which goes out as an Android package, and no credential, an expired or revoked token or an unknown
//! session is refused.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{Answer, Client, Sent, Server, repo};
use mm_core::auth_store::install_auth_schema;
use mm_core::sql::{Sql, SqlValue};
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

const READY: &str = "downloads for signed-in users on http://127.0.0.1:";
const APK: &str = "mymind 0.2.0.apk";
const HOUR: f64 = 3_600_000.0;

fn digest(text: &str) -> String {
    base64::hex(&Sha256::digest(text.as_bytes()))
}

fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64
}

fn package() -> Vec<u8> {
    (0..70_000u32).map(|index| (index * 31 % 251) as u8).collect()
}

fn fixture(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("download-android-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("downloads")).unwrap();
    let auth = Sqlite::open(dir.join("auth.db"), false).unwrap();
    install_auth_schema(&auth).unwrap();
    let now = now();
    for (token, expires) in [("live-token", now + HOUR), ("old-token", now - HOUR), ("revoked-token", now + HOUR)] {
        auth.execute(
            "INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlValue::Text(digest(token)),
                SqlValue::Text("password:phone".into()),
                SqlValue::Text("phone@example.com".into()),
                SqlValue::Text("password".into()),
                SqlValue::Text("{}".into()),
                SqlValue::Real(now - HOUR),
                SqlValue::Real(expires),
                SqlValue::Real(now),
            ],
        )
        .unwrap();
    }
    auth.execute("DELETE FROM app_tokens WHERE hash = ?", &[SqlValue::Text(digest("revoked-token"))]).unwrap();
    auth.execute(
        "INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)",
        &[SqlValue::Text(digest("browser-session")), SqlValue::Text("sub".into()), SqlValue::Text("owner@example.com".into()), SqlValue::Real(now), SqlValue::Real(now + HOUR), SqlValue::Real(now)],
    )
    .unwrap();
    std::fs::write(dir.join("downloads").join(APK), package()).unwrap();
    std::fs::write(dir.join("downloads/mymind-linux-x86_64.tar.gz"), b"tarball").unwrap();
    let manifest = format!(
        r#"{{"library":"https://library.example","files":[
          {{"name":"mymind-linux-x86_64.tar.gz","label":"Linux (x86_64)","bytes":7,"built":"2026-10-17T01:02:03Z","sha256":"ab"}},
          {{"name":"{APK}","label":"Android","platform":"android","bytes":70000,"built":"2026-10-18T09:00:00Z","sha256":"{}",
            "versionName":"0.2.0","versionCode":2,"notes":["Updates itself."],
            "history":[{{"versionName":"0.2.0","versionCode":2,"date":"2026-10-18","notes":["Updates itself."]}}]}}]}}"#,
        base64::hex(&Sha256::digest(package()))
    );
    std::fs::write(dir.join("downloads/manifest.json"), manifest).unwrap();
    dir
}

fn start(dir: &Path) -> Server {
    let root = repo();
    let port = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mymind"));
    command
        .arg("download-server")
        .current_dir(&root)
        .env_clear()
        .env("MYMIND_ROOT", &root)
        .env("DOWNLOAD_DIR", dir.join("downloads"))
        .env("AUTH_DB", dir.join("auth.db"))
        .env("DOWNLOAD_PORT", port.to_string());
    Server::start(command, READY)
}

fn header<'a>(answer: &'a Answer, name: &str) -> Option<&'a str> {
    answer.headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

#[test]
fn the_app_token_downloads_the_android_package() {
    let dir = fixture("token");
    let server = start(&dir);
    let mut client = Client::new(server.port);
    let get = |client: &mut Client, method: &str, target: &str, headers: &[(&str, &str)]| client.send(&Sent::new(method, target, headers));
    let bearer = [("authorization", "Bearer live-token")];

    let release = get(&mut client, "GET", "/download/android.json", &bearer);
    assert!(release.status.ends_with("200 OK"), "{release:?}");
    assert_eq!(header(&release, "content-type"), Some("application/json; charset=utf-8"));
    assert_eq!(header(&release, "cache-control"), Some("no-store"));
    let body = json::parse(std::str::from_utf8(&release.body).unwrap()).unwrap();
    assert_eq!(body.get("versionCode").to_number(), 2.0);
    assert_eq!(body.get("url").as_str(), Some("/download/mymind%200.2.0.apk"));
    assert_eq!(body.get("sha256").as_str(), Some(base64::hex(&Sha256::digest(package())).as_str()));
    assert_eq!(json::stringify(body.get("notes")).unwrap().unwrap(), r#"["Updates itself."]"#);

    let apk = get(&mut client, "GET", "/download/mymind%200.2.0.apk", &bearer);
    assert!(apk.status.ends_with("200 OK"), "{apk:?}");
    assert_eq!(header(&apk, "content-type"), Some("application/vnd.android.package-archive"));
    assert_eq!(header(&apk, "content-disposition"), Some("attachment; filename=\"mymind 0.2.0.apk\""));
    assert!(apk.body == package(), "the package arrives unchanged");
    let head = get(&mut client, "HEAD", "/download/mymind%200.2.0.apk", &bearer);
    assert_eq!(header(&head, "content-length"), Some("70000"));

    let tarball = get(&mut client, "GET", "/download/mymind-linux-x86_64.tar.gz", &bearer);
    assert_eq!(header(&tarball, "content-type"), Some("application/gzip"));
    let page = get(&mut client, "GET", "/download", &bearer);
    let page = String::from_utf8_lossy(&page.body);
    assert!(page.contains("<h2>Android</h2>") && page.contains("Version 0.2.0 (2)") && page.contains("Signed in as phone@example.com"), "{page}");

    let cookie = get(&mut client, "GET", "/download/android.json", &[("cookie", "mm_session=browser-session")]);
    assert!(cookie.status.ends_with("200 OK"), "a browser session still works: {cookie:?}");
}

#[test]
fn downloads_refuse_without_a_live_credential() {
    let dir = fixture("refusals");
    let server = start(&dir);
    let mut client = Client::new(server.port);
    let refusals: [&[(&str, &str)]; 7] = [
        &[],
        &[("authorization", "Bearer old-token")],
        &[("authorization", "Bearer revoked-token")],
        &[("authorization", "Bearer unknown")],
        &[("authorization", "Bearer  ")],
        &[("authorization", "Basic live-token")],
        &[("cookie", "mm_session=elsewhere-session")],
    ];
    for headers in refusals {
        for target in ["/download/android.json", "/download/mymind%200.2.0.apk"] {
            let answer = client.send(&Sent::new("GET", target, headers));
            assert!(answer.status.ends_with("401 Unauthorized"), "{target} {headers:?}: {answer:?}");
            assert!(!answer.body.windows(4).any(|window| window == b"0.2."), "{target} {headers:?} leaked the release");
        }
        let listing = client.send(&Sent::new("GET", "/download", headers));
        assert_eq!(header(&listing, "location"), Some("/signin"), "{headers:?}");
    }
}
