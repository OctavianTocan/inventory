//! A valid cookie session also authenticates as a bearer token (the browser extension's way in), a bearer alone is
//! exempt from the cross-origin checks, a cookie session's `/hub` upgrade needs the server's own origin, and every
//! other credential path stays as it was.

use std::collections::HashMap;
use std::rc::Rc;

use async_trait::async_trait;
use mm_core::auth::{Auth, AuthOptions, SigninPages};
use mm_core::auth_store::install_auth_schema;
use mm_core::host::Fetch;
use mm_core::http::{Request, Response};
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::{Error, Result};
use mm_js::{base64, date};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

const NOW: f64 = 1_760_000_000_000.0;
const HOUR: f64 = 3_600_000.0;
const DAY: f64 = 24.0 * HOUR;

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

fn setup() -> (Auth, Rc<dyn Sql>) {
    date::set_fixed_now(Some(NOW as i64));
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("extension-auth-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("auth-{}.db", NOW as i64 + rand_suffix()));
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(path, false).unwrap());
    install_auth_schema(&*db).unwrap();
    let env: HashMap<String, String> = [("AUTH", "google"), ("GOOGLE_CLIENT_ID", "c"), ("GOOGLE_CLIENT_SECRET", "s"), ("OWNER_EMAIL", "owner@example.com")]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    let auth = Auth::new(AuthOptions::new(env, 3000, db.clone(), Rc::new(NoFetch), Rc::new(Pages))).unwrap().unwrap();
    (auth, db)
}

fn rand_suffix() -> i64 {
    static NEXT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn session(db: &dyn Sql, id: &str, expires: f64, used: f64) {
    db.execute(
        "INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)",
        &[SqlValue::Text(digest(id)), SqlValue::Text("sub-1".into()), SqlValue::Text("owner@example.com".into()), SqlValue::Real(NOW - DAY), SqlValue::Real(expires), SqlValue::Real(used)],
    )
    .unwrap();
}

fn app_token(db: &dyn Sql, token: &str) {
    db.execute(
        "INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        &[SqlValue::Text(digest(token)), SqlValue::Text("sub-2".into()), SqlValue::Text("app@example.com".into()), SqlValue::Text("google".into()), SqlValue::Text("{}".into()), SqlValue::Real(NOW - DAY), SqlValue::Real(NOW + DAY), SqlValue::Real(NOW)],
    )
    .unwrap();
}

fn request(headers: &[(&str, &str)]) -> Request {
    let mut request = Request::new("GET", "http://localhost:3000/tags/recent").unwrap();
    request.headers.set("host", "localhost:3000");
    for (name, value) in headers {
        request.headers.set(*name, *value);
    }
    request
}

fn write_status(auth: &Auth, headers: &[(&str, &str)]) -> Option<u16> {
    let mut request = Request::new("POST", "http://localhost:3000/objects").unwrap();
    request.headers.set("host", "localhost:3000");
    for (name, value) in headers {
        request.headers.set(*name, *value);
    }
    futures_executor::block_on(auth.handle(&mut request)).unwrap().map(|response| response.status)
}

fn expires(db: &dyn Sql, table: &str, id: &str) -> (f64, f64) {
    let row = db.one(&format!("SELECT expires_ms, used_ms FROM {table} WHERE hash = ?"), &[SqlValue::Text(digest(id))]).unwrap().unwrap();
    (row.number("expires_ms").unwrap(), row.number("used_ms").unwrap())
}

#[test]
fn valid_cookie_session_authenticates_as_bearer() {
    let (auth, db) = setup();
    session(&*db, "sid-valid", NOW + DAY, NOW);
    let identity = auth.identity(&request(&[("authorization", "Bearer sid-valid")])).unwrap().expect("signed in");
    assert_eq!(identity.email, "owner@example.com");
    assert!(auth.allows(&request(&[("authorization", "Bearer sid-valid")])).unwrap());
}

#[test]
fn unknown_or_expired_bearer_is_refused() {
    let (auth, db) = setup();
    session(&*db, "sid-old", NOW - 1.0, NOW - DAY);
    assert!(!auth.allows(&request(&[("authorization", "Bearer sid-old")])).unwrap());
    assert!(!auth.allows(&request(&[("authorization", "Bearer nothing-like-it")])).unwrap());
    assert!(!auth.allows(&request(&[("authorization", "Bearer")])).unwrap());
    assert!(!auth.allows(&request(&[])).unwrap());
    let mut handled = request(&[("authorization", "Bearer nothing-like-it")]);
    let response = futures_executor::block_on(auth.handle(&mut handled)).unwrap().expect("refused");
    assert_eq!(response.status, 401);
}

#[test]
fn bearer_session_follows_the_cookie_rules() {
    let (auth, db) = setup();
    session(&*db, "sid-stale", NOW + DAY, NOW - 2.0 * HOUR);
    assert!(auth.allows(&request(&[("authorization", "Bearer sid-stale")])).unwrap());
    let (expiry, used) = expires(&*db, "sessions", "sid-stale");
    assert_eq!(expiry, NOW + DAY, "a session's expiry never slides");
    assert_eq!(used, NOW, "use time is refreshed after an hour, as for the cookie");
}

#[test]
fn cookie_and_app_token_paths_are_unchanged() {
    let (auth, db) = setup();
    session(&*db, "sid-cookie", NOW + DAY, NOW);
    app_token(&*db, "app-token");
    assert!(auth.allows(&request(&[("cookie", "mm_session=sid-cookie")])).unwrap());
    assert_eq!(auth.identity(&request(&[("authorization", "Bearer app-token")])).unwrap().unwrap().email, "app@example.com");
    assert!(!auth.allows(&request(&[("cookie", "mm_session=sid-unknown")])).unwrap());
    let both = request(&[("cookie", "mm_session=sid-cookie"), ("authorization", "Bearer app-token")]);
    assert_eq!(auth.identity(&both).unwrap().unwrap().email, "owner@example.com", "the cookie wins over a bearer token");
    let app_expiry = expires(&*db, "app_tokens", "app-token").0;
    assert_eq!(app_expiry, NOW + DAY);
}

#[test]
fn cross_origin_writes_pass_only_when_a_bearer_alone_signs_in() {
    let (auth, db) = setup();
    session(&*db, "sid-valid", NOW + DAY, NOW);
    app_token(&*db, "app-token");
    let foreign = ("origin", "chrome-extension://abcdefghijklmnop");
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer sid-valid")]), None);
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer app-token")]), None);
    assert_eq!(write_status(&auth, &[foreign, ("cookie", "mm_session=sid-valid")]), Some(403), "a cookie session keeps the origin check");
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer sid-valid"), ("cookie", "mm_session=sid-valid")]), Some(403));
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer sid-valid"), ("cookie", "mm_session=")]), Some(403), "an empty cookie still counts as a cookie");
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer sid-valid"), ("cookie", "mm_session=sid-unknown")]), Some(403));
    assert_eq!(write_status(&auth, &[foreign, ("authorization", "Bearer nothing-like-it")]), Some(403), "an unknown bearer is no exemption");
    assert_eq!(write_status(&auth, &[foreign]), Some(403));
}

fn upgrade(headers: &[(&str, &str)]) -> Request {
    let mut request = Request::new("GET", "http://localhost:3000/hub").unwrap();
    request.headers.set("host", "localhost:3000");
    request.headers.set("upgrade", "websocket");
    request.headers.set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==");
    for (name, value) in headers {
        request.headers.set(*name, *value);
    }
    request
}

#[test]
fn hub_upgrades_with_a_cookie_need_the_servers_own_origin() {
    let (auth, db) = setup();
    session(&*db, "sid-valid", NOW + DAY, NOW);
    app_token(&*db, "app-token");
    let sibling = ("origin", "https://other.octaviantocan.com");
    let cookie = ("cookie", "mm_session=sid-valid");
    let cases: [(&[(&str, &str)], bool); 6] = [
        (&[cookie, ("origin", "http://localhost:3000")], true),
        (&[cookie], true),
        (&[cookie, sibling], false),
        (&[sibling, ("authorization", "Bearer app-token")], true),
        (&[sibling, ("authorization", "Bearer sid-valid")], true),
        (&[sibling, ("authorization", "Bearer sid-valid"), ("cookie", "mm_session=")], false),
    ];
    for (headers, allowed) in cases {
        assert_eq!(auth.allows_upgrade(&upgrade(headers)).unwrap(), allowed, "allows_upgrade {headers:?}");
        let status = futures_executor::block_on(auth.handle(&mut upgrade(headers))).unwrap().map(|response| response.status);
        assert_eq!(status, if allowed { None } else { Some(403) }, "handle {headers:?}");
    }
    assert!(!auth.allows_upgrade(&upgrade(&[("origin", "http://localhost:3000")])).unwrap(), "no session, no hub");
    let mut plain = request(&[cookie, sibling]);
    assert_eq!(futures_executor::block_on(auth.handle(&mut plain)).unwrap().map(|response| response.status), None, "a plain GET keeps passing");
}
