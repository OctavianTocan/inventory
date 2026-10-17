//! Local Worker integration against frozen JS expectations. No Node process is launched by this test.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mm_core::library_rows::{encode_value, table_digest};
use mm_core::sql::Sql;
use mm_js::{Value, base64, json, obj};
use mm_sqlite::Sqlite;
use regex::Regex;
use sha2::{Digest, Sha256};

const PASSWORD: &str = "offline-worker-password-123";
const LIBRARY: &str = "testlib";
const HS: &str = "{\"protocol\":\"json\",\"version\":1}\u{1e}";
const DROP: [&str; 9] = [
    "date",
    "content-length",
    "transfer-encoding",
    "connection",
    "keep-alive",
    "content-encoding",
    "vary",
    "cf-ray",
    "server",
];

#[derive(Clone)]
struct Case {
    name: String,
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
    session: &'static str,
    masks: Vec<&'static str>,
}

impl Case {
    // Describe one request with credential placeholders retained in its identity.
    fn new(name: &str, method: &str, path: &str) -> Self {
        Self {
            name: name.into(),
            method: method.into(),
            path: path.into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            session: "none",
            masks: Vec::new(),
        }
    }

    // Select a reusable synthetic session for this request.
    fn session(mut self, session: &'static str) -> Self {
        self.session = session;
        self
    }

    // Add one exact request header.
    fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    // Set an encoded request body and its content type.
    fn body(mut self, kind: &str, bytes: impl Into<Vec<u8>>) -> Self {
        self.headers.insert("content-type".into(), kind.into());
        self.body = bytes.into();
        self
    }

    // Mask only explicitly nondeterministic JSON properties in this answer.
    fn masks(mut self, masks: &[&'static str]) -> Self {
        self.masks = masks.to_vec();
        self
    }

    // Fingerprint the exact ordered request without storing private paths or bodies.
    fn identity(&self) -> String {
        let headers: Vec<Value> = self
            .headers
            .iter()
            .map(|(name, value)| Value::Array(vec![name.clone().into(), value.clone().into()]))
            .collect();
        digest(json::to_string(&obj! { "name" => self.name.as_str(), "method" => self.method.as_str(), "path" => self.path.as_str(), "headers" => headers, "body" => base64::encode(&self.body), "session" => self.session, "masks" => strings(&self.masks) }.into()).as_bytes())
    }
}

struct Client {
    base: String,
    admin: String,
    http: reqwest::Client,
    cookie: String,
    nolib: String,
    token: String,
}

struct Answer {
    status: u16,
    headers: BTreeMap<String, Vec<String>>,
    body: Vec<u8>,
}

impl Client {
    // Refuse every destination other than an explicitly configured local Worker.
    fn new(variable: &str) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let base = std::env::var(variable).unwrap_or_else(|_| {
            panic!("{variable} is required; start the isolated local Worker first")
        });
        let url = url::Url::parse(&base).expect("valid Worker URL");
        assert!(
            url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.port().is_some()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none(),
            "Worker URL must be plain http://127.0.0.1:<port>"
        );
        Self {
            base: base.trim_end_matches('/').into(),
            admin: std::env::var("MYMIND_WORKER_ADMIN_TOKEN").unwrap_or_default(),
            http: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(90))
                .build()
                .unwrap(),
            cookie: String::new(),
            nolib: String::new(),
            token: String::new(),
        }
    }

    // Send one request using real reusable credentials without logging them.
    async fn raw(&self, case: &Case) -> Answer {
        let mut headers = case.headers.clone();
        match case.session {
            "user" => {
                headers.insert("cookie".into(), self.cookie.clone());
            }
            "nolib" => {
                headers.insert("cookie".into(), self.nolib.clone());
            }
            "bearer" => {
                headers.insert("authorization".into(), format!("Bearer {}", self.token));
            }
            "session-bearer" => {
                headers.insert(
                    "authorization".into(),
                    format!("Bearer {}", self.cookie.split_once('=').unwrap().1),
                );
            }
            "badbearer" => {
                headers.insert("authorization".into(), "Bearer nope".into());
            }
            "admin" => {
                headers.insert("authorization".into(), format!("Bearer {}", self.admin));
            }
            "wrong-admin" => {
                headers.insert("authorization".into(), "Bearer wrong-token".into());
            }
            _ => {}
        }
        let mut request = self.http.request(
            case.method.parse().unwrap(),
            format!("{}{}", self.base, case.path),
        );
        for (name, value) in headers {
            request = request.header(name, value);
        }
        if !case.body.is_empty() {
            request = request.body(case.body.clone());
        }
        let response = request
            .send()
            .await
            .unwrap_or_else(|_| panic!("local request failed: {}", case.name));
        let status = response.status().as_u16();
        let mut headers = BTreeMap::<String, Vec<String>>::new();
        for (name, value) in response.headers() {
            headers
                .entry(name.to_string())
                .or_default()
                .push(value.to_str().unwrap().to_owned());
        }
        Answer {
            status,
            headers,
            body: response.bytes().await.unwrap().to_vec(),
        }
    }

    // Invoke the local administrative import API and require success.
    async fn admin(&self, path: &str, value: Value) -> Value {
        let answer = self
            .raw(
                &Case::new("seed", "POST", &format!("/_admin/{path}"))
                    .session("admin")
                    .body("application/json", json::to_string(&value)),
            )
            .await;
        assert!(
            answer.status < 300,
            "seed {path} returned {}",
            answer.status
        );
        if answer.body.is_empty() {
            Value::Null
        } else {
            json::parse(&String::from_utf8(answer.body).unwrap()).unwrap()
        }
    }

    // Sign in a synthetic user and retain its real session cookie.
    async fn signin(&self, username: &str) -> String {
        let answer = self
            .raw(
                &Case::new("session setup", "POST", "/signin/password").body(
                    "application/x-www-form-urlencoded",
                    form(username, PASSWORD),
                ),
            )
            .await;
        assert_eq!(answer.status, 303);
        answer
            .headers
            .get("set-cookie")
            .and_then(|values| values.first())
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }
}

// Convert textual arrays to the JS value representation.
fn strings(items: &[impl AsRef<str>]) -> Value {
    Value::Array(items.iter().map(|s| s.as_ref().into()).collect())
}

// Hash opaque bytes without persisting private response contents.
fn digest(bytes: &[u8]) -> String {
    base64::hex(&Sha256::digest(bytes))
}

// Encode a password form exactly like URLSearchParams.
fn form(username: &str, password: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_pair("username", username)
        .append_pair("password", password)
        .finish()
}

// Locate only the immutable Stage A hosted fixture.
fn fixture() -> PathBuf {
    PathBuf::from(
        std::env::var_os("MYMIND_TEST_HOSTED_DB")
            .expect("MYMIND_TEST_HOSTED_DB frozen fixture is required"),
    )
}

// Locate Worker expectations beside the deployment crate.
fn golden_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../worker/tests/golden/worker.json")
}

// Canonicalize response headers, bounded runtime errors, credentials and explicit time properties.
fn normalized(answer: &Answer, masks: &[&str]) -> Vec<u8> {
    let cookie = Regex::new(r"^([^=]+)=[^;]*").unwrap();
    let expiry = Regex::new(r"(?i)Expires=[^;]*").unwrap();
    let location = Regex::new(r"(state|nonce|code_challenge)=[^&]*").unwrap();
    let mut headers = mm_js::Object::new();
    for (name, values) in &answer.headers {
        if DROP.contains(&name.as_str()) || (answer.status == 500 && name == "content-type") {
            continue;
        }
        let values: Vec<Value> = values
            .iter()
            .map(|value| {
                if name == "set-cookie" {
                    expiry
                        .replace_all(&cookie.replace(value, "$1=<v>"), "Expires=<d>")
                        .into_owned()
                        .into()
                } else if name == "location" {
                    location.replace_all(value, "$1=<r>").into_owned().into()
                } else {
                    value.clone().into()
                }
            })
            .collect();
        headers.insert(name, Value::Array(values));
    }
    let content_type = answer
        .headers
        .get("content-type")
        .and_then(|v| v.first())
        .map(String::as_str)
        .unwrap_or("");
    let mut body = if answer.status == 500 {
        "<runtime error page>".into()
    } else if !answer.body.is_empty()
        && !content_type.starts_with("text/")
        && !content_type.starts_with("application/json")
    {
        format!(
            "sha256:{} ({} bytes)",
            digest(&answer.body),
            answer.body.len()
        )
    } else {
        String::from_utf8_lossy(&answer.body).into_owned()
    };
    for key in masks {
        body = Regex::new(&format!(r#"("{key}":)("[^"]*"|[0-9.]+)"#))
            .unwrap()
            .replace_all(&body, "$1\"<masked>\"")
            .into_owned();
    }
    json::to_string(&obj! { "status" => answer.status as usize, "headers" => Value::Object(headers), "body" => body }.into()).into_bytes()
}

#[derive(Clone)]
struct Table {
    name: String,
    columns: Vec<String>,
    rowid: bool,
    parents: Vec<String>,
}

// Reconstruct import order and storage-class-preserving columns from the frozen schema.
fn tables(db: &dyn Sql) -> Vec<Table> {
    let rows = db.query("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'search_fts%' AND name != 'search_dirty'", &[]).unwrap();
    let all: Vec<Table> = rows
        .iter()
        .map(|row| {
            let name = row.text("name").unwrap().to_owned();
            let cols = db
                .query(
                    "SELECT name, type, pk, hidden FROM pragma_table_xinfo(?)",
                    &[name.as_str().into()],
                )
                .unwrap();
            let keys: Vec<_> = cols
                .iter()
                .filter(|col| col.int("pk").unwrap_or(0) != 0)
                .collect();
            let alias = keys.len() == 1
                && keys[0]
                    .text("type")
                    .unwrap()
                    .eq_ignore_ascii_case("INTEGER");
            let rowid = !row
                .text("sql")
                .unwrap()
                .to_uppercase()
                .contains("WITHOUT ROWID")
                && !alias;
            let columns = rowid
                .then(|| "rowid".to_owned())
                .into_iter()
                .chain(
                    cols.iter()
                        .filter(|col| col.int("hidden") == Some(0))
                        .map(|col| col.text("name").unwrap().to_owned()),
                )
                .collect();
            let parents = db
                .query(
                    "SELECT \"table\" AS parent FROM pragma_foreign_key_list(?)",
                    &[name.as_str().into()],
                )
                .unwrap()
                .iter()
                .map(|row| row.text("parent").unwrap().to_owned())
                .filter(|parent| parent != &name)
                .collect();
            Table {
                name,
                columns,
                rowid,
                parents,
            }
        })
        .collect();
    let mut out = Vec::new();
    while out.len() < all.len() {
        let before = out.len();
        for table in &all {
            if !out.iter().any(|done: &Table| done.name == table.name)
                && table
                    .parents
                    .iter()
                    .all(|parent| out.iter().any(|done| &done.name == parent))
            {
                out.push(table.clone());
            }
        }
        assert!(out.len() > before, "cyclic fixture import schema");
    }
    out
}

// Seed synthetic auth/download objects and every logical hosted row, then check every table digest.
async fn seed(client: &Client, db: &Sqlite) {
    assert!(
        !client.admin.is_empty(),
        "MYMIND_WORKER_ADMIN_TOKEN is required for the isolated local runtime"
    );
    let manifest: Value = obj! { "library" => LIBRARY, "files" => vec![
        Value::from(obj! { "name" => "mymind-linux-x86_64.tar.gz", "label" => "Linux x86_64", "bytes" => 6000, "built" => "2026-10-01T00:00:00Z", "sha256" => "ab".repeat(32) }),
        Value::from(obj! { "name" => "with space & \"q\".tar.gz", "label" => "Odd", "bytes" => 3, "built" => "2026-10-02T00:00:00Z", "sha256" => "cd".repeat(32) })
    ] }.into();
    for (key, body, kind) in [
        (
            "downloads/manifest.json",
            json::to_string(&manifest).into_bytes(),
            "application/json",
        ),
        ("downloads/mymind-linux-x86_64.tar.gz", vec![7; 6000], ""),
        ("downloads/with space & \"q\".tar.gz", b"abc".to_vec(), ""),
        ("static/a.txt", b"hello".to_vec(), "text/plain"),
    ] {
        put(client, key, body, kind).await;
    }
    let salt = b"offline-worker-salt";
    let hash = mm_core::auth_store::hash_password(PASSWORD, salt);
    for username in ["tester", "nolib"] {
        client.admin("auth/user", obj! { "username" => username, "salt" => base64::encode(salt), "hash" => base64::encode(&hash), "created_ms" => 1787000000000_f64 }.into()).await;
    }
    client
        .admin(
            "auth/library",
            obj! { "account" => "password:tester", "library" => LIBRARY }.into(),
        )
        .await;
    let tables = tables(db);
    let reset = client
        .admin(&format!("library/{LIBRARY}/reset"), obj! {}.into())
        .await;
    for table in &tables {
        assert!(
            !reset.get("tables").get(&table.name).is_undefined(),
            "Worker schema lacks {}",
            table.name
        );
        let columns = table
            .columns
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let order = if table.rowid { " ORDER BY rowid" } else { "" };
        let rows = db
            .query(
                &format!("SELECT {columns} FROM \"{}\"{order}", table.name),
                &[],
            )
            .unwrap();
        for batch in rows.chunks(200) {
            let rows: Vec<Value> = batch
                .iter()
                .map(|row| {
                    Value::Array(
                        row.values()
                            .iter()
                            .map(|value| encode_value(value).unwrap())
                            .collect(),
                    )
                })
                .collect();
            client.admin(&format!("library/{LIBRARY}/rows"), obj! { "table" => table.name.as_str(), "columns" => strings(&table.columns), "rows" => rows }.into()).await;
        }
    }
    let dirty: Vec<Value> = db
        .query("SELECT id FROM search_dirty", &[])
        .unwrap()
        .iter()
        .map(|row| Value::Array(vec![row.get("id").to_value()]))
        .collect();
    let sequences: Vec<Value> = db
        .query(
            "SELECT name, CAST(seq AS TEXT) AS seq FROM sqlite_sequence",
            &[],
        )
        .unwrap()
        .iter()
        .map(|row| {
            Value::from(
                obj! { "name" => row.text("name").unwrap(), "seq" => row.text("seq").unwrap() },
            )
        })
        .collect();
    client
        .admin(
            &format!("library/{LIBRARY}/finish"),
            obj! { "dirty" => dirty, "sequences" => sequences }.into(),
        )
        .await;
    let mut checked = tables;
    checked.push(Table {
        name: "search_dirty".into(),
        columns: vec!["id".into()],
        rowid: false,
        parents: Vec::new(),
    });
    let inventory: Vec<Value> = checked
        .iter()
        .map(|table| {
            Value::from(
                obj! { "table" => table.name.as_str(), "columns" => strings(&table.columns) },
            )
        })
        .collect();
    let remote = client
        .admin(
            &format!("library/{LIBRARY}/digest"),
            obj! { "tables" => inventory }.into(),
        )
        .await;
    for table in &checked {
        assert_eq!(
            json::to_string(remote.get(&table.name)),
            json::to_string(
                &table_digest(db, &table.name, &table.columns)
                    .unwrap()
                    .to_value()
            ),
            "seed digest {}",
            table.name
        );
    }
    println!(
        "seeded and verified {} complete table digests",
        checked.len()
    );
}

// Write one synthetic object through the import API.
async fn put(client: &Client, key: &str, body: Vec<u8>, kind: &str) {
    let path = key
        .split('/')
        .map(mm_js::uri::encode_uri_component)
        .collect::<Vec<_>>()
        .join("/");
    let mut case =
        Case::new("object seed", "PUT", &format!("/_admin/bucket/{path}")).session("admin");
    case.body = body;
    if !kind.is_empty() {
        case = case.header(
            "x-object-headers",
            &json::to_string(&obj! { "content-type" => kind }.into()),
        );
    }
    assert_eq!(client.raw(&case).await.status, 204);
}

// Preserve the original differential suite's ordered 160 HTTP requests before sign-out and hub cases.
fn cases(db: &Sqlite) -> Vec<Case> {
    let mut out = Vec::new();
    for (path, method) in [
        ("/signin", "GET"),
        ("/signin?error=SignInFailed", "GET"),
        ("/signin/password", "GET"),
        ("/signin/password?error=invalid", "GET"),
        ("/robots.txt", "GET"),
        ("/_auth/signin.js", "GET"),
        ("/signout", "GET"),
        ("/everything", "GET"),
        ("/api/x", "GET"),
        ("/google/oauth2-redirect", "GET"),
        ("/google/oauth2-callback", "GET"),
        ("/signin/password", "DELETE"),
        ("/app/v1/me", "GET"),
    ] {
        out.push(Case::new(
            &format!("signed-out {method} {path}"),
            method,
            path,
        ));
    }
    for (name, username, password) in [
        ("wrong password", "tester", "wrong-wrong-wrong-1"),
        ("unknown user", "ghost", PASSWORD),
    ] {
        out.push(Case::new(name, "POST", "/signin/password").body(
            "application/x-www-form-urlencoded",
            form(username, password),
        ));
    }
    out.push(Case::new("password no body", "POST", "/signin/password"));
    for user in ["nolib", "tester"] {
        out.push(
            Case::new(&format!("password ok {user}"), "POST", "/signin/password")
                .body("application/x-www-form-urlencoded", form(user, PASSWORD)),
        );
    }
    for path in ["/signin", "/signin/password"] {
        out.push(Case::new("signed-in signin redirect", "GET", path).session("user"));
    }
    let json_case = |name: &str, path: &str, body: Value| {
        Case::new(name, "POST", path).body("application/json", json::to_string(&body))
    };
    out.push(
        json_case("app nonce", "/app/v1/auth/nonce", obj! {}.into()).masks(&["nonce", "expiresAt"]),
    );
    out.push(json_case(
        "app google bad token",
        "/app/v1/auth/google",
        obj! { "nonce" => "x", "idToken" => "y" }.into(),
    ));
    out.push(json_case(
        "app password wrong",
        "/app/v1/auth/password",
        obj! { "username" => "tester", "password" => "wrong-wrong-wrong-1" }.into(),
    ));
    out.push(
        Case::new("app password invalid json", "POST", "/app/v1/auth/password")
            .body("application/json", "{nope"),
    );
    out.push(
        json_case(
            "app password ok",
            "/app/v1/auth/password",
            obj! { "username" => "tester", "password" => PASSWORD, "device" => "diff2" }.into(),
        )
        .masks(&[
            "token",
            "accessToken",
            "access_token",
            "expiresAt",
            "expires_at",
            "id",
        ]),
    );
    out.push(Case::new("app GET nonce", "GET", "/app/v1/auth/nonce"));
    for session in ["bearer", "user", "badbearer"] {
        out.push(Case::new("app me credential", "GET", "/app/v1/me").session(session));
    }
    out.push(
        Case::new("app signout bad token", "POST", "/app/v1/auth/signout").session("badbearer"),
    );
    for path in ["/everything", "/app/v1/me"] {
        out.push(Case::new("no-library account", "GET", path).session("nolib"));
    }
    for session in ["admin", "wrong-admin", "none"] {
        out.push(
            Case::new("admin summary", "GET", "/_admin/auth")
                .session(session)
                .masks(&["created_ms"]),
        );
    }
    out.push(
        Case::new("admin basic", "GET", "/_admin/auth")
            .header("authorization", "Basic worker-oracle-local-admin-only"),
    );
    for path in [
        "nothing",
        "",
        "bucket",
        "bucket?prefix=downloads%2F",
        "bucket?prefix=zzz",
    ] {
        out.push(Case::new("admin route", "GET", &format!("/_admin/{path}")).session("admin"));
    }
    for path in [
        "static/a.txt",
        "static/missing.txt",
        "a/../b",
        "a//b",
        "%E0%A4%A",
    ] {
        out.push(
            Case::new(
                "admin bucket head",
                "HEAD",
                &format!("/_admin/bucket/{path}"),
            )
            .session("admin"),
        );
    }
    for method in ["GET", "DELETE"] {
        out.push(
            Case::new(
                "admin bucket unsupported",
                method,
                "/_admin/bucket/static/a.txt",
            )
            .session("admin"),
        );
    }
    let mut put = Case::new("admin bucket put", "PUT", "/_admin/bucket/tmp/put.txt")
        .session("admin")
        .header("x-object-headers", "{\"content-type\":\"text/x-test\"}");
    put.body = b"payload".to_vec();
    out.push(put);
    out.push(
        Case::new(
            "admin bucket head after put",
            "HEAD",
            "/_admin/bucket/tmp/put.txt",
        )
        .session("admin"),
    );
    let mut bad = Case::new(
        "admin bucket bad headers",
        "PUT",
        "/_admin/bucket/tmp/bad.txt",
    )
    .session("admin")
    .header("x-object-headers", "{nope");
    bad.body = b"x".to_vec();
    out.push(bad);
    for step in ["counts", "nope"] {
        out.push(
            json_case(
                "admin library step",
                &format!("/_admin/library/{LIBRARY}/{step}"),
                obj! {}.into(),
            )
            .session("admin"),
        );
    }
    out.push(
        json_case(
            "admin bad library name",
            "/_admin/library/Bad_Name/counts",
            obj! {}.into(),
        )
        .session("admin"),
    );
    for table in ["cards", "nope"] {
        out.push(json_case("admin digest", &format!("/_admin/library/{LIBRARY}/digest"), obj! { "tables" => vec![Value::from(obj! { "table" => table, "columns" => strings(&["id"]) })] }.into()).session("admin"));
    }
    out.push(json_case("admin bad rows", &format!("/_admin/library/{LIBRARY}/rows"), obj! { "table" => "nope", "columns" => strings(&["id"]), "rows" => vec![Value::Array(vec![1.into()])] }.into()).session("admin"));
    out.push(
        json_case(
            "admin wrong counts",
            &format!("/_admin/library/{LIBRARY}/counts"),
            obj! {}.into(),
        )
        .session("wrong-admin"),
    );
    out.push(
        Case::new("admin auth bad json", "POST", "/_admin/auth/user")
            .session("admin")
            .body("application/json", "{nope"),
    );
    out.push(
        json_case(
            "admin auth missing fields",
            "/_admin/auth/library",
            obj! {}.into(),
        )
        .session("admin"),
    );
    out.push(
        Case::new("admin summary after", "GET", "/_admin/auth")
            .session("admin")
            .masks(&["created_ms"]),
    );
    for path in [
        "/download",
        "/download/",
        "/download/mymind-linux-x86_64.tar.gz",
    ] {
        out.push(Case::new("download signed out", "GET", path));
    }
    for (name, method, path, session) in [
        ("listing", "GET", "/download", "user"),
        ("listing slash", "GET", "/download/", "user"),
        ("listing bearer", "GET", "/download", "bearer"),
        (
            "file",
            "GET",
            "/download/mymind-linux-x86_64.tar.gz",
            "user",
        ),
        (
            "file HEAD",
            "HEAD",
            "/download/mymind-linux-x86_64.tar.gz",
            "user",
        ),
        (
            "odd name",
            "GET",
            "/download/with%20space%20%26%20%22q%22.tar.gz",
            "user",
        ),
        ("missing", "GET", "/download/nope.tar.gz", "user"),
        ("not in manifest", "GET", "/download/manifest.json", "user"),
        ("bad encoding", "GET", "/download/%E0%A4%A", "user"),
        ("POST", "POST", "/download", "user"),
        ("POST file", "POST", "/download/x", "user"),
        ("lookalike", "GET", "/downloads", "user"),
    ] {
        out.push(Case::new(&format!("download {name}"), method, path).session(session));
    }
    for (path, method) in [
        ("/everything", "GET"),
        ("/api/cards", "GET"),
        ("/app/v1/cards", "GET"),
        ("/static/x.css", "GET"),
        ("/", "GET"),
        ("/everything", "POST"),
    ] {
        out.push(Case::new("library site", method, path).session("user"));
    }
    let cards: Vec<_> = db
        .query(
            "SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 3",
            &[],
        )
        .unwrap()
        .iter()
        .map(|row| row.text("id").unwrap().to_owned())
        .collect();
    let spaces: Vec<_> = db
        .query("SELECT id FROM spaces ORDER BY id LIMIT 2", &[])
        .unwrap()
        .iter()
        .map(|row| row.text("id").unwrap().to_owned())
        .collect();
    let mut reads: Vec<String> = [
        "/cards",
        "/cards/trash",
        "/spaces",
        "/tags/manual",
        "/tags/recent",
        "/changes?since=0",
        "/account.json",
        "/quota",
        "/search?q=design",
        "/search/suggestions?q=d",
        "/tags/suggest?q=a",
        "/app/v1/me",
        "/app/v1/cards?limit=5",
        "/app/v1/spaces",
        "/objects/nope",
    ]
    .iter()
    .map(|s| (*s).into())
    .collect();
    for id in &cards {
        for path in [
            format!("/cards/{id}"),
            format!("/cards/{id}/detail"),
            format!("/objects/{id}"),
            format!("/objects/{id}/content"),
            format!("/objects/{id}/tags"),
            format!("/embeddings/related/{id}"),
            format!("/app/v1/cards/{id}"),
        ] {
            reads.push(path);
        }
    }
    reads.extend(spaces.iter().map(|id| format!("/spaces/{id}")));
    for path in reads {
        out.push(
            Case::new("library read JSON", "GET", &path)
                .session("user")
                .header("accept", "application/json")
                .masks(&["elapsed"]),
        );
        out.push(
            Case::new("library read MessagePack", "GET", &path)
                .session("user")
                .masks(&["elapsed"]),
        );
    }
    out.push(
        Case::new("library cards bearer", "GET", "/cards")
            .session("bearer")
            .header("accept", "application/json"),
    );
    out.push(json_case("library grid", "/cards/grid", strings(&cards)).session("user"));
    out
}

// Exchange masked client frames and retain the exact six legacy hub observations.
fn websocket(
    client: &Client,
    cookie: Option<&str>,
    key: bool,
    messages: &[String],
    origin: Option<&str>,
) -> Vec<u8> {
    let url = url::Url::parse(&client.base).unwrap();
    let mut socket = TcpStream::connect(("127.0.0.1", url.port().unwrap())).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut request = format!(
        "GET /hub HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\n",
        url.port().unwrap()
    );
    if key {
        request.push_str("Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n");
    }
    if let Some(cookie) = cookie {
        request.push_str(&format!("Cookie: {cookie}\r\n"));
    }
    if let Some(origin) = origin {
        request.push_str(&format!("Origin: {origin}\r\n"));
    }
    request.push_str("\r\n");
    socket.write_all(request.as_bytes()).unwrap();
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let status: usize = line.split_whitespace().nth(1).unwrap().parse().unwrap();
    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
    }
    let mut frames = Vec::<String>::new();
    if status == 101 {
        for message in messages {
            let length = message.len();
            let mut frame = vec![0x81];
            if length < 126 {
                frame.push(0x80 | length as u8);
            } else if length < 65536 {
                frame.push(0xfe);
                frame.extend_from_slice(&(length as u16).to_be_bytes());
            } else {
                frame.push(0xff);
                frame.extend_from_slice(&(length as u64).to_be_bytes());
            }
            frame.extend_from_slice(&[0; 4]);
            frame.extend_from_slice(message.as_bytes());
            if reader.get_mut().write_all(&frame).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(150));
        }
        reader
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(600)))
            .unwrap();
        loop {
            let mut head = [0; 2];
            if reader.read_exact(&mut head).is_err() {
                break;
            }
            let mut length = (head[1] & 127) as usize;
            if length == 126 {
                let mut b = [0; 2];
                reader.read_exact(&mut b).unwrap();
                length = u16::from_be_bytes(b) as usize;
            }
            if length == 127 {
                let mut b = [0; 8];
                reader.read_exact(&mut b).unwrap();
                length = u64::from_be_bytes(b) as usize;
            }
            assert!(length < 2_000_000, "oversized server frame");
            let mut payload = vec![0; length];
            reader.read_exact(&mut payload).unwrap();
            let opcode = head[0] & 15;
            let frame = if opcode == 8 {
                format!(
                    "close {} {}",
                    u16::from_be_bytes(payload[..2].try_into().unwrap()),
                    String::from_utf8_lossy(&payload[2..])
                )
            } else {
                format!(
                    "{} {}",
                    if opcode == 1 {
                        "text".to_owned()
                    } else {
                        format!("op{opcode}")
                    },
                    json::to_string(&String::from_utf8_lossy(&payload).to_string().into())
                )
            };
            frames.push(frame);
            if opcode == 8 {
                break;
            }
        }
    }
    json::to_string(&obj! { "status" => status, "frames" => strings(&frames) }.into()).into_bytes()
}

// Add one complete ordered observation to a digest-only golden manifest.
fn observation(entries: &mut Vec<Value>, identity: String, answer: &[u8]) {
    entries.push(
        obj! { "id" => identity, "length" => answer.len(), "sha256" => digest(answer) }.into(),
    );
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires isolated local Wrangler; run node tools/check-worker.mjs"]
async fn worker_matches_frozen_js() {
    let mut client = Client::new("MYMIND_WORKER_URL");
    mm_test_support::golden::recording();
    let file = golden_file();
    let expected = Some(json::parse(&std::fs::read_to_string(&file).expect("Worker golden missing; restore the frozen pre-switch expectations")).unwrap());
    let fixture_bytes = std::fs::read(fixture()).expect("frozen hosted fixture missing");
    let fixture_hash = digest(&fixture_bytes);
    let source_hash = mm_test_support::golden::source_identity(env!("CARGO_MANIFEST_DIR"), "worker", "source", &digest(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/worker.rs")).unwrap()));
    if let Some(expected) = &expected {
        assert_eq!(expected.get("version").as_f64(), Some(1.0));
        assert_eq!(
            expected.get("fixture_sha256").as_str(),
            Some(fixture_hash.as_str()),
            "Worker fixture changed"
        );
        assert_eq!(
            expected.get("source_sha256").as_str(),
            Some(source_hash.as_str()),
            "Worker corpus implementation changed"
        );
    }
    let db = Sqlite::open(fixture(), true).unwrap();
    seed(&client, &db).await;
    client.cookie = client.signin("tester").await;
    client.nolib = client.signin("nolib").await;
    let token = client
        .raw(
            &Case::new("app token setup", "POST", "/app/v1/auth/password").body(
                "application/json",
                json::to_string(
                    &obj! { "username" => "tester", "password" => PASSWORD, "device" => "diff" }
                        .into(),
                ),
            ),
        )
        .await;
    assert_eq!(token.status, 200);
    let token = json::parse(&String::from_utf8(token.body).unwrap()).unwrap();
    let expires = mm_js::date::parse(token.get("expiresAt").as_str().expect("app expiry present"));
    assert!(
        expires > mm_core::runtime::now_ms() + 89.0 * 86400000.0,
        "app token expiry must retain the 90 day lifetime"
    );
    client.token = token
        .get("token")
        .as_str()
        .expect("app token present")
        .to_owned();
    assert!(
        !client.cookie.ends_with('=') && !client.nolib.ends_with('=') && !client.token.is_empty()
    );
    let cases = cases(&db);
    assert_eq!(
        cases.len(),
        160,
        "preserve the original HTTP case inventory"
    );
    let mut observations = Vec::new();
    for case in cases {
        let answer = client.raw(&case).await;
        observation(
            &mut observations,
            case.identity(),
            &normalized(&answer, &case.masks),
        );
    }
    for case in [
        Case::new("sign-out", "GET", "/signout").session("user"),
        Case::new("after sign-out", "GET", "/app/v1/me").session("user"),
    ] {
        let answer = client.raw(&case).await;
        if case.name == "after sign-out" {
            assert_eq!(answer.status, 401, "revoked cookie must fail");
        }
        observation(
            &mut observations,
            case.identity(),
            &normalized(&answer, &[]),
        );
    }
    let fresh = client.signin("tester").await;
    let hubs = [
        ("hub signed out", false, true, Vec::new()),
        ("hub no key", true, false, Vec::new()),
        (
            "hub handshake + ping",
            true,
            true,
            vec![
                HS.into(),
                "{\"type\":6}\u{1e}".into(),
                "{\"type\":6}\u{1e}".into(),
            ],
        ),
        (
            "hub handshake split records",
            true,
            true,
            vec![HS[..10].into(), HS[10..].into()],
        ),
        (
            "hub invalid record",
            true,
            true,
            vec![HS.into(), "not json\u{1e}".into()],
        ),
        (
            "hub too large",
            true,
            true,
            vec![HS.into(), format!("{}\u{1e}", "x".repeat(1_048_577))],
        ),
    ];
    for (name, signed_in, key, messages) in hubs {
        let answer = websocket(
            &client,
            signed_in.then_some(fresh.as_str()),
            key,
            &messages,
            None,
        );
        observation(&mut observations, digest(json::to_string(&obj! { "name" => name, "signed_in" => signed_in, "key" => key, "messages" => strings(&messages) }.into()).as_bytes()), &answer);
    }
    assert_eq!(
        observations.len(),
        168,
        "full HTTP, revocation and six-hub inventory"
    );
    if let Some(expected) = expected {
        assert_eq!(
            expected.get("count").as_f64(),
            Some(observations.len() as f64),
            "Worker golden case count changed"
        );
        let wanted = expected
            .get("cases")
            .as_array()
            .expect("Worker case manifest");
        assert_eq!(wanted.len(), observations.len(), "Worker golden truncated");
        for (index, (actual, expected)) in observations.iter().zip(wanted).enumerate() {
            assert_eq!(
                json::to_string(actual),
                json::to_string(expected),
                "Worker case {index}: normalized response length/hash or ordered request identity differs"
            );
        }
        merged_behavior(&mut client, &db).await;
    } else {
        panic!("Legacy recording requires the recorded pre-switch revision");
    }
    println!("168 ordered Worker golden cases passed");
}

// Assert intentional merged Rust behavior independently of the frozen legacy Worker answers.
async fn merged_behavior(client: &mut Client, db: &Sqlite) {
    client.cookie = client.signin("tester").await;
    let session = client
        .raw(&Case::new("extension session bearer", "GET", "/app/v1/me").session("session-bearer"))
        .await;
    assert_eq!(
        session.status, 200,
        "extension must reuse a valid cookie value as bearer"
    );
    let origin = client
        .raw(
            &Case::new(
                "foreign cookie write",
                "POST",
                "/browser-extension/activate",
            )
            .session("user")
            .header("origin", "https://foreign.invalid"),
        )
        .await;
    assert_eq!(origin.status, 403);
    let no_cookie = client
        .raw(
            &Case::new(
                "foreign bearer write",
                "POST",
                "/browser-extension/activate",
            )
            .session("session-bearer")
            .header("origin", "https://foreign.invalid"),
        )
        .await;
    assert_ne!(
        no_cookie.status, 403,
        "explicit bearer avoids cookie forgery restriction"
    );
    let foreign_hub = websocket(
        client,
        Some(&client.cookie),
        true,
        &[],
        Some("https://foreign.invalid"),
    );
    assert_eq!(
        json::parse(&String::from_utf8(foreign_hub).unwrap())
            .unwrap()
            .get("status")
            .as_f64(),
        Some(403.0)
    );
    let own_hub = websocket(
        client,
        Some(&client.cookie),
        true,
        &[HS.into()],
        Some("http://127.0.0.1"),
    );
    assert_eq!(
        json::parse(&String::from_utf8(own_hub).unwrap())
            .unwrap()
            .get("status")
            .as_f64(),
        Some(101.0)
    );
    let tags = client
        .raw(
            &Case::new("extension tags JSON", "GET", "/tags/manual")
                .session("session-bearer")
                .header("accept", "application/json"),
        )
        .await;
    assert_eq!(tags.status, 200);
    assert!(
        json::parse(&String::from_utf8(tags.body).unwrap())
            .unwrap()
            .is_array()
    );
    let tom = client
        .raw(&Case::new("top of mind", "GET", "/app/v1/top-of-mind").session("bearer"))
        .await;
    assert_eq!(tom.status, 200);
    assert!(
        json::parse(&String::from_utf8(tom.body).unwrap())
            .unwrap()
            .get("cards")
            .is_array()
    );
    let created = client.raw(&Case::new("extension Content create", "POST", "/objects").session("session-bearer").body("application/json", json::to_string(&obj! { "type" => "Content", "content" => "Selected words", "source" => "https://example.test/page?q=a#section" }.into()))).await;
    assert_eq!(created.status, 201);
    let created = json::parse(&String::from_utf8(created.body).unwrap()).unwrap();
    let created_id = created.get("id").as_str().expect("synthetic Content id");
    let patched = client
        .raw(
            &Case::new(
                "app author field",
                "PATCH",
                &format!("/objects/{created_id}"),
            )
            .session("session-bearer")
            .body(
                "application/json",
                json::to_string(&obj! { "author" => "Synthetic Author" }.into()),
            ),
        )
        .await;
    assert_eq!(patched.status, 200);
    let metadata = client
        .raw(
            &Case::new(
                "app merged fields",
                "GET",
                &format!("/app/v1/cards/{created_id}"),
            )
            .session("bearer"),
        )
        .await;
    assert_eq!(metadata.status, 200);
    let metadata = json::parse(&String::from_utf8(metadata.body).unwrap()).unwrap();
    assert_eq!(metadata.get("author").as_str(), Some("Synthetic Author"));
    assert_eq!(
        metadata.get("url").as_str(),
        Some("https://example.test/page?q=a#section")
    );
    let markdown = client
        .raw(
            &Case::new(
                "extension Markdown",
                "GET",
                &format!("/cards/{created_id}/markdown"),
            )
            .session("session-bearer"),
        )
        .await;
    assert_eq!(markdown.status, 200);
    assert_eq!(
        String::from_utf8(markdown.body).unwrap(),
        "Selected words\n"
    );
    upload(client, b"png".to_vec()).await;
    let id = db
        .query(
            "SELECT id FROM cards WHERE deleted_ns IS NULL ORDER BY id LIMIT 1",
            &[],
        )
        .unwrap()[0]
        .text("id")
        .unwrap()
        .to_owned();
    for field in ["detail", "markdown"] {
        let answer = client
            .raw(
                &Case::new(
                    "extension field endpoint",
                    "GET",
                    &format!("/cards/{id}/{field}"),
                )
                .session("session-bearer"),
            )
            .await;
        assert_eq!(answer.status, 200, "extension field {field}");
    }
    let manifest: Value = obj! { "files" => vec![Value::from(obj! { "name" => "mymind.apk", "platform" => "android", "label" => "Android", "bytes" => 3, "built" => "2026-10-17T00:00:00Z", "sha256" => "ab".repeat(32), "versionName" => "1.0", "versionCode" => 7, "notes" => strings(&["offline update"]), "history" => Vec::<Value>::new() })] }.into();
    put(
        client,
        "downloads/manifest.json",
        json::to_string(&manifest).into_bytes(),
        "application/json",
    )
    .await;
    put(
        client,
        "downloads/mymind.apk",
        b"apk".to_vec(),
        "application/vnd.android.package-archive",
    )
    .await;
    let release = client
        .raw(&Case::new("Android release", "GET", "/download/android.json").session("user"))
        .await;
    assert_eq!(release.status, 200);
    let release = json::parse(&String::from_utf8(release.body).unwrap()).unwrap();
    assert_eq!(release.get("versionCode").as_f64(), Some(7.0));
    assert_eq!(release.get("url").as_str(), Some("/download/mymind.apk"));
    for (method, session) in [("GET", "user"), ("HEAD", "user"), ("GET", "bearer")] {
        let answer = client
            .raw(&Case::new("Android package", method, "/download/mymind.apk").session(session))
            .await;
        assert_eq!(answer.status, 200);
        assert_eq!(
            answer.headers["content-type"],
            ["application/vnd.android.package-archive"]
        );
        assert_eq!(
            answer.body,
            if method == "HEAD" {
                Vec::new()
            } else {
                b"apk".to_vec()
            }
        );
    }
    let signout = client
        .raw(&Case::new("app token sign-out", "POST", "/app/v1/auth/signout").session("bearer"))
        .await;
    assert_eq!(signout.status, 204);
    assert_eq!(
        client
            .raw(&Case::new("revoked app token", "GET", "/app/v1/me").session("bearer"))
            .await
            .status,
        401
    );
    assert_eq!(
        client
            .raw(&Case::new("cookie sign-out", "GET", "/signout").session("user"))
            .await
            .status,
        302
    );
    assert_eq!(
        client
            .raw(
                &Case::new("revoked session bearer", "GET", "/app/v1/me").session("session-bearer")
            )
            .await
            .status,
        401
    );
    println!(
        "merged extension bearer, origin, fields, Android and credential-revocation assertions passed"
    );
}

// Authorize and complete an upload using separate session and upload capabilities.
async fn upload(client: &Client, body: Vec<u8>) -> String {
    let answer = client
        .raw(&Case::new("upload capability", "POST", "/uploads/authorize").session("user"))
        .await;
    assert_eq!(answer.status, 200);
    let value = json::parse(&String::from_utf8(answer.body).unwrap()).unwrap();
    let path = value.get("url").as_str().unwrap();
    let token = value.get("token").as_str().unwrap();
    let answer = client
        .raw(
            &Case::new("separate upload token", "PUT", path)
                .session("user")
                .header("x-upload-token", token)
                .header(
                    "content-range",
                    &format!("bytes 0-{}/{}", body.len() - 1, body.len()),
                )
                .header("x-file-name", "worker-smoke.png")
                .body("image/png", body),
        )
        .await;
    assert_eq!(answer.status, 200);
    let value = json::parse(&String::from_utf8(answer.body).unwrap()).unwrap();
    assert_eq!(value.get("done"), &Value::Bool(true));
    value.get("id").as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires a separately started local Rust Worker with its real media container"]
async fn worker_real_container_smoke() {
    let mut client = Client::new("MYMIND_WORKER_CONTAINER_URL");
    let db = Sqlite::open(fixture(), true).unwrap();
    seed(&client, &db).await;
    client.cookie = client.signin("tester").await;
    let png = base64::decode(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a9S8AAAAASUVORK5CYII=",
    );
    let id = upload(&client, png).await;
    let answer = client
        .raw(
            &Case::new("real media container", "POST", "/objects")
                .session("user")
                .body(
                    "application/json",
                    json::to_string(&obj! { "type" => "Media", "uploadId" => id }.into()),
                ),
        )
        .await;
    assert_eq!(
        answer.status, 201,
        "real media container preparation failed"
    );
}
