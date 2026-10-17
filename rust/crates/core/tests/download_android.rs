//! The Android package on `/download`: its section of the page, the release description the app's updater reads,
//! its media type, and the sign-in gate letting an app token through to them.

use std::collections::HashMap;
use std::rc::Rc;

use async_trait::async_trait;
use mm_core::auth::{Auth, AuthOptions, SigninPages};
use mm_core::auth_store::install_auth_schema;
use mm_core::download_page::{APK_TYPE, android_release, content_type, download_page};
use mm_core::host::Fetch;
use mm_core::http::{Request, Response};
use mm_core::sql::{Sql, SqlValue};
use mm_core::{Error, Result};
use mm_js::{Value, base64, date, json};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

const NOW: f64 = 1_760_000_000_000.0;
const DAY: f64 = 24.0 * 3_600_000.0;

fn linux() -> Value {
    json::parse(r#"{"name":"mymind-linux-x86_64.tar.gz","label":"Linux (x86_64)","bytes":52428800,"built":"2026-10-17T01:02:03Z","sha256":"ab"}"#).unwrap()
}

fn apk(version_name: &str, version_code: u32, notes: &[&str]) -> Value {
    let notes: Vec<Value> = notes.iter().map(|note| Value::String((*note).to_owned())).collect();
    let mut entry = json::parse(&format!(
        r#"{{"name":"mymind {version_name}.apk","label":"Android","platform":"android","bytes":10485760,"built":"2026-10-18T09:00:00Z","sha256":"cd","versionName":"{version_name}","versionCode":{version_code}}}"#
    ))
    .unwrap();
    if let Value::Object(object) = &mut entry {
        object.insert("notes", Value::Array(notes.clone()));
        object.insert("history", Value::Array(vec![mm_js::obj! { "versionName" => version_name, "versionCode" => version_code as f64, "date" => "2026-10-18", "notes" => Value::Array(notes) }.into()]));
    }
    entry
}

fn manifest(files: Vec<Value>) -> Value {
    mm_js::obj! { "library" => "https://library.example", "files" => Value::Array(files) }.into()
}

#[test]
fn a_manifest_without_android_keeps_the_page_as_it_was() {
    let page = download_page(&manifest(vec![linux()]), "owner@example.com").unwrap();
    assert!(!page.contains("Android") && !page.contains("<h2>"), "{page}");
    assert!(page.contains("<li><a href=\"/download/mymind-linux-x86_64.tar.gz\">Linux (x86_64)</a>"));
}

#[test]
fn the_page_lists_the_android_package_with_its_version_and_notes() {
    let page = download_page(&manifest(vec![linux(), apk("0.2.0", 2, &["Updates itself.", "<b>&</b>"])]), "owner@example.com").unwrap();
    let android = page.find("<h2>Android</h2>").expect("an Android section");
    let linux = page.find("<h2>Linux</h2>").expect("a Linux heading");
    assert!(android < linux && linux < page.find("mymind-linux-x86_64.tar.gz").unwrap());
    assert!(page.contains("<a href=\"/download/mymind%200.2.0.apk\">Android</a>"), "{page}");
    assert!(page.contains("<span>Version 0.2.0 (2) · 10.0 MB · built 2026-10-18</span><code>sha256 cd</code>"), "{page}");
    assert!(page.contains("<span>- Updates itself.</span><span>- &lt;b&gt;&amp;&lt;/b&gt;</span>"), "{page}");
    assert_eq!(page.matches("mymind 0.2.0.apk").count() + page.matches("mymind%200.2.0.apk").count(), 1, "the package is not listed twice");
}

#[test]
fn the_release_is_the_highest_version_code() {
    assert!(android_release(&manifest(vec![linux()])).is_none());
    assert!(android_release(&Value::Null).is_none());
    let release = android_release(&manifest(vec![apk("0.3.0", 3, &["Newer."]), linux(), apk("0.2.0", 2, &["Older."])])).unwrap();
    assert_eq!(
        json::stringify(&release).unwrap().unwrap(),
        r#"{"versionName":"0.3.0","versionCode":3,"url":"/download/mymind%200.3.0.apk","bytes":10485760,"sha256":"cd","built":"2026-10-18T09:00:00Z","notes":["Newer."],"history":[{"versionName":"0.3.0","versionCode":3,"date":"2026-10-18","notes":["Newer."]}]}"#
    );
    let bare = json::parse(r#"{"files":[{"name":"a.apk","platform":"android","versionCode":1}]}"#).unwrap();
    let release = android_release(&bare).unwrap();
    assert_eq!(release.get("notes"), &Value::Array(Vec::new()));
    assert_eq!(release.get("history"), &Value::Array(Vec::new()));
}

#[test]
fn a_package_named_like_the_release_description_is_not_a_release() {
    let mut entry = apk("0.2.0", 2, &[]);
    if let Value::Object(object) = &mut entry {
        object.insert("name", "android.json");
    }
    assert!(android_release(&manifest(vec![entry.clone(), linux()])).is_none());
    let page = download_page(&manifest(vec![entry]), "owner@example.com").unwrap();
    assert!(!page.contains("<h2>Android</h2>"), "{page}");
}

#[test]
fn an_android_only_manifest_says_the_linux_app_is_missing() {
    let page = download_page(&manifest(vec![apk("0.2.0", 2, &[])]), "owner@example.com").unwrap();
    assert!(page.contains("<h2>Linux</h2>\n<ul><li>No Linux app has been packaged yet."), "{page}");
}

#[test]
fn only_android_entries_are_sent_as_packages() {
    assert_eq!(content_type(&apk("0.2.0", 2, &[])), APK_TYPE);
    assert_eq!(content_type(&linux()), "application/gzip");
    assert_eq!(content_type(&json::parse(r#"{"name":"x","platform":"ios"}"#).unwrap()), "application/gzip");
}

struct Pages;

#[async_trait(?Send)]
impl SigninPages for Pages {
    async fn page(&self, _name: &str) -> Result<String> {
        Ok("<html></html>".to_owned())
    }
}

struct NoFetch;

#[async_trait(?Send)]
impl Fetch for NoFetch {
    async fn fetch(&self, _request: Request) -> Result<Response> {
        Err(Error::invalid("fetch failed"))
    }
}

fn digest(text: &str) -> String {
    base64::hex(&Sha256::digest(text.as_bytes()))
}

fn gate() -> (Auth, Rc<dyn Sql>) {
    date::set_fixed_now(Some(NOW as i64));
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("download-android-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(directory.join("auth.db"), false).unwrap());
    install_auth_schema(&*db).unwrap();
    db.execute("DELETE FROM app_tokens", &[]).unwrap();
    let env: HashMap<String, String> = [("AUTH", "google"), ("GOOGLE_CLIENT_ID", "c"), ("GOOGLE_CLIENT_SECRET", "s"), ("OWNER_EMAIL", "owner@example.com")]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    let auth = Auth::new(AuthOptions::new(env, 3000, db.clone(), Rc::new(NoFetch), Rc::new(Pages))).unwrap().unwrap();
    (auth, db)
}

fn app_token(db: &dyn Sql, token: &str, expires: f64) {
    db.execute(
        "INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        &[
            SqlValue::Text(digest(token)),
            SqlValue::Text("sub-app".into()),
            SqlValue::Text("app@example.com".into()),
            SqlValue::Text("password".into()),
            SqlValue::Text("{}".into()),
            SqlValue::Real(NOW - DAY),
            SqlValue::Real(expires),
            SqlValue::Real(NOW),
        ],
    )
    .unwrap();
}

// The gate's status for a GET of `path` with `authorization`; `None` lets the request through to `/download`.
fn status(auth: &Auth, path: &str, authorization: Option<&str>) -> Option<u16> {
    let mut request = Request::new("GET", &format!("http://localhost:3000{path}")).unwrap();
    request.headers.set("host", "localhost:3000");
    if let Some(value) = authorization {
        request.headers.set("authorization", value);
    }
    futures_executor::block_on(auth.handle(&mut request)).unwrap().map(|response| response.status)
}

#[test]
fn the_gate_lets_an_app_token_through_to_downloads() {
    let (auth, db) = gate();
    app_token(&*db, "live-token", NOW + DAY);
    app_token(&*db, "old-token", NOW - 1.0);
    app_token(&*db, "revoked-token", NOW + DAY);
    db.execute("DELETE FROM app_tokens WHERE hash = ?", &[SqlValue::Text(digest("revoked-token"))]).unwrap();
    for path in ["/download/android.json", "/download/mymind%200.2.0.apk"] {
        assert_eq!(status(&auth, path, Some("Bearer live-token")), None, "{path}");
        assert_eq!(status(&auth, path, None), Some(401), "{path}");
        assert_eq!(status(&auth, path, Some("Bearer old-token")), Some(401), "{path}");
        assert_eq!(status(&auth, path, Some("Bearer revoked-token")), Some(401), "{path}");
        assert_eq!(status(&auth, path, Some("Bearer unknown")), Some(401), "{path}");
    }
}
