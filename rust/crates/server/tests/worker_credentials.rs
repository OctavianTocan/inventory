//! Unmasked credential contracts against a prestarted synthetic local Rust Worker.
//! This independent target never reads or updates the frozen Worker oracle.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mm_js::{Value, base64, date, json, obj};
use sha2::{Digest, Sha256};

const USER: &str = "credential-contract-user";
const PASSWORD: &str = "credential-contract-password-only";
const LIBRARY: &str = "credential-contracts";
const DAY_MS: i64 = 86_400_000;

struct Client {
    base: String,
    admin: String,
    http: reqwest::Client,
    requests: Cell<usize>,
}

struct Answer {
    status: u16,
    headers: reqwest::header::HeaderMap,
    body: Vec<u8>,
}

struct Upload {
    path: String,
    token: String,
    expiry: i64,
}

impl Client {
    // Require an explicit local endpoint and synthetic administrative capability.
    fn new() -> Self {
        let base = std::env::var("MYMIND_WORKER_URL")
            .expect("MYMIND_WORKER_URL is required; use the isolated Worker launcher");
        let url = url::Url::parse(&base).expect("valid local Worker URL");
        assert!(
            url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.port().is_some()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none(),
            "Worker endpoint must be plain http://127.0.0.1:<port>"
        );
        let admin = std::env::var("MYMIND_WORKER_ADMIN_TOKEN")
            .expect("MYMIND_WORKER_ADMIN_TOKEN is required");
        assert!(
            !admin.is_empty(),
            "local administrative capability is required"
        );
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(90))
            .build()
            .unwrap();
        Self {
            base: base.trim_end_matches('/').to_owned(),
            admin,
            http,
            requests: Cell::new(0),
        }
    }

    // Send only local requests, without printing credential values on failure.
    async fn request(
        &self,
        label: &str,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Answer {
        assert!(
            path.starts_with('/') && !path.starts_with("//"),
            "request must remain on the configured loopback origin"
        );
        self.requests.set(self.requests.get() + 1);
        let mut request = self
            .http
            .request(method.parse().unwrap(), format!("{}{path}", self.base));
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request
            .body(body)
            .send()
            .await
            .unwrap_or_else(|_| panic!("loopback request failed: {label}"));
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .unwrap_or_else(|_| panic!("loopback body failed: {label}"))
            .to_vec();
        Answer {
            status,
            headers,
            body,
        }
    }

    // Call the local import API with only synthetic inputs.
    async fn admin(&self, path: &str, body: Value) -> Value {
        let authorization = format!("Bearer {}", self.admin);
        let answer = self
            .request(
                "synthetic admin setup",
                "POST",
                &format!("/_admin/{path}"),
                &[
                    ("authorization", &authorization),
                    ("content-type", "application/json"),
                ],
                json::to_string(&body).into_bytes(),
            )
            .await;
        assert!(
            answer.status < 300,
            "synthetic admin setup returned {}",
            answer.status
        );
        if answer.body.is_empty() {
            Value::Null
        } else {
            parsed(&answer)
        }
    }

    // Seed a dedicated synthetic password account and an independent minimal library.
    async fn seed(&self) {
        let salt = b"credential-contract-salt";
        let hash = mm_core::auth_store::hash_password(PASSWORD, salt);
        self.admin("auth/user", obj! { "username" => USER, "salt" => base64::encode(salt), "hash" => base64::encode(&hash), "created_ms" => 1787000000000_f64 }.into()).await;
        self.admin(
            "auth/library",
            obj! { "account" => format!("password:{USER}"), "library" => LIBRARY }.into(),
        )
        .await;
        self.admin(&format!("library/{LIBRARY}/reset"), obj! {}.into())
            .await;
        self.admin(&format!("library/{LIBRARY}/rows"), obj! {
            "table" => "users",
            "columns" => Value::Array(["id", "account_uid", "id_prefix", "settings"].into_iter().map(Value::from).collect()),
            "rows" => vec![Value::Array(["credential-user", "synthetic-credentials", "00000000000", "{}"].into_iter().map(Value::from).collect())],
        }.into()).await;
        self.admin(
            &format!("library/{LIBRARY}/finish"),
            obj! { "dirty" => Vec::<Value>::new(), "sequences" => Vec::<Value>::new() }.into(),
        )
        .await;
    }

    // Mint and validate a session, preserving only its value in test memory.
    async fn session(&self) -> String {
        let form = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("username", USER)
            .append_pair("password", PASSWORD)
            .finish();
        let answer = self
            .request(
                "password session",
                "POST",
                "/signin/password",
                &[("content-type", "application/x-www-form-urlencoded")],
                form.into_bytes(),
            )
            .await;
        assert_eq!(answer.status, 303, "password session status");
        assert!(
            answer
                .headers
                .get("location")
                .is_some_and(|header| header == "/everything"),
            "password session redirect"
        );
        let session = cookie(&answer, "mm_session", "/", 30 * 86400);
        canonical_base64url(&session, 32, "session credential");
        session
    }

    // Mint an app token and verify canonical format and fresh 90-day expiry before reuse.
    async fn app_token(&self) -> String {
        let before = now_ms();
        let body =
            obj! { "username" => USER, "password" => PASSWORD, "device" => "credential-contract" }
                .into();
        let answer = self
            .request(
                "app token",
                "POST",
                "/app/v1/auth/password",
                &[("content-type", "application/json")],
                json::to_string(&body).into_bytes(),
            )
            .await;
        let after = now_ms();
        assert_eq!(answer.status, 200, "app token status");
        let value = parsed(&answer);
        let token = value
            .get("token")
            .as_str()
            .expect("app token must be a string");
        canonical_base64url(token, 32, "app token");
        fresh_expiry(
            value.get("expiresAt"),
            before,
            after,
            90 * DAY_MS,
            "app token",
        );
        assert!(
            value.get("user").get("email").as_str() == Some(USER),
            "app identity must remain synthetic"
        );
        assert!(
            value.get("user").get("kind").as_str() == Some("password"),
            "app identity kind"
        );
        token.to_owned()
    }

    // Require the same synthetic identity when a cookie or bearer credential is reused.
    async fn me(&self, name: &str, credential: &str, status: u16) {
        let answer = self
            .request(
                "credential reuse",
                "GET",
                "/app/v1/me",
                &[(name, credential)],
                Vec::new(),
            )
            .await;
        assert_eq!(answer.status, status, "credential reuse status");
        if status == 200 {
            let value = parsed(&answer);
            assert!(
                value.get("email").as_str() == Some(USER),
                "reused credential must identify only the synthetic user"
            );
            assert!(
                value.get("kind").as_str() == Some("password"),
                "reused credential kind"
            );
        }
    }

    // Mint one upload capability and check its stored 24-hour expiry without exposing it.
    async fn upload(&self, session: &str, older_expiries: &[i64]) -> Upload {
        let cookie = format!("mm_session={session}");
        let before = now_ms();
        let answer = self
            .request(
                "upload authorization",
                "POST",
                "/uploads/authorize",
                &[("cookie", &cookie)],
                Vec::new(),
            )
            .await;
        let after = now_ms();
        assert_eq!(answer.status, 200, "upload authorization status");
        let value = parsed(&answer);
        let path = value
            .get("url")
            .as_str()
            .expect("upload path must be a string");
        let id = path
            .strip_prefix("/uploads/")
            .expect("upload path must use the upload route");
        canonical_hex(id, 11, "upload id");
        let token = value
            .get("token")
            .as_str()
            .expect("upload token must be a string");
        canonical_hex(token, 32, "upload token");
        let remote = self.admin(&format!("library/{LIBRARY}/digest"), obj! { "tables" => vec![Value::from(obj! { "table" => "uploads", "columns" => vec![Value::from("expires_ms")] })] }.into()).await;
        let remote = remote.get("uploads");
        assert_eq!(
            remote.get("rows").as_f64(),
            Some((older_expiries.len() + 1) as f64),
            "each minted capability must create one fresh row"
        );
        let expected = remote.get("sha256").as_str().expect("upload expiry digest");
        assert!(
            after >= before && after - before <= 90_000,
            "bounded upload issuance window"
        );
        let expiry = (before + DAY_MS..=after + DAY_MS)
            .find(|candidate| {
                let mut expiries = older_expiries.to_vec();
                expiries.push(*candidate);
                integer_column_digest(&expiries) == expected
            })
            .expect(
                "stored upload expiry must be within the issuance window plus exactly 24 hours",
            );
        assert!(
            expiry > after,
            "new upload capability must expire in the future"
        );
        Upload {
            path: path.to_owned(),
            token: token.to_owned(),
            expiry,
        }
    }

    // Exercise a capability with the separate session and upload credentials.
    async fn chunk(&self, session: &str, upload: &Upload, token: &str, offset: usize, status: u16) {
        let authorization = format!("Bearer {session}");
        let range = format!("bytes {offset}-{offset}/2");
        let answer = self
            .request(
                "upload capability reuse",
                "PUT",
                &upload.path,
                &[
                    ("authorization", &authorization),
                    ("x-upload-token", token),
                    ("content-range", &range),
                    ("content-type", "image/png"),
                    ("x-file-name", "credential-contract.png"),
                ],
                b"x".to_vec(),
            )
            .await;
        assert_eq!(answer.status, status, "upload capability reuse status");
        if status == 200 {
            let value = parsed(&answer);
            assert_eq!(
                value.get("received").as_f64(),
                Some((offset + 1) as f64),
                "only the expected upload advances"
            );
            assert_eq!(
                value.get("done"),
                &Value::Bool(offset == 1),
                "upload completion boundary"
            );
            assert!(
                value.get("id").as_str() == upload.path.strip_prefix("/uploads/"),
                "upload response must keep its capability identity"
            );
        }
    }
}

// Parse an expected JSON answer without displaying its body on failure.
fn parsed(answer: &Answer) -> Value {
    json::parse(
        &String::from_utf8(answer.body.clone())
            .unwrap_or_else(|_| panic!("expected UTF-8 JSON body")),
    )
    .unwrap_or_else(|_| panic!("expected JSON response"))
}

// Read milliseconds from the same system clock used by the local runtime.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// Check exact unpadded base64url encoding and decoded random-byte length without logging values.
fn canonical_base64url(value: &str, bytes: usize, label: &str) {
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
        "{label} alphabet must be base64url"
    );
    let decoded = base64::decode(value);
    assert!(
        decoded.len() == bytes && base64::encode_url(&decoded) == value,
        "{label} must encode exactly {bytes} random bytes canonically"
    );
}

// Check exact lowercase hexadecimal encoding without logging capability values.
fn canonical_hex(value: &str, bytes: usize, label: &str) {
    assert!(
        value.len() == bytes * 2
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{label} must encode exactly {bytes} bytes as lowercase hexadecimal"
    );
}

// Check canonical UTC millisecond timestamps against the complete local issuance window.
fn fresh_expiry(value: &Value, before: i64, after: i64, lifetime: i64, label: &str) {
    let text = value.as_str().expect("credential expiry must be a string");
    let expiry = date::parse(text);
    assert!(
        expiry.is_finite() && date::iso_from_ms(expiry).as_deref() == Some(text),
        "{label} expiry must be canonical UTC ISO text"
    );
    assert!(
        after >= before
            && expiry >= (before + lifetime) as f64
            && expiry <= (after + lifetime) as f64
            && expiry > after as f64,
        "{label} must have a fresh exact lifetime"
    );
}

// Validate a single cookie's name, scope and lifetime before retaining its value.
fn cookie(answer: &Answer, name: &str, path: &str, seconds: usize) -> String {
    let cookies: Vec<_> = answer.headers.get_all("set-cookie").iter().collect();
    assert_eq!(cookies.len(), 1, "one generated cookie is required");
    let text = cookies[0].to_str().expect("cookie must be ASCII");
    let mut pieces = text.split("; ");
    let (found, value) = pieces
        .next()
        .and_then(|piece| piece.split_once('='))
        .expect("cookie name and value");
    assert!(found == name, "expected generated cookie name");
    let attributes: Vec<_> = pieces.collect();
    assert!(
        attributes
            == [
                format!("Path={path}"),
                format!("Max-Age={seconds}"),
                "HttpOnly".into(),
                "SameSite=Lax".into()
            ],
        "generated cookie scope, exact lifetime and protection attributes"
    );
    value.to_owned()
}

// Independently reproduce the documented storage-class-sensitive digest for integer-only columns.
fn integer_column_digest(values: &[i64]) -> String {
    let mut rows: Vec<String> = values
        .iter()
        .map(|value| {
            let text = value.to_string();
            let bytes = format!("integer\0{}\0{text}", text.len());
            base64::hex(&Sha256::digest(bytes.as_bytes()))
        })
        .collect();
    rows.sort();
    base64::hex(&Sha256::digest(rows.join("\n").as_bytes()))
}

// Validate nonce syntax, independent issuance and bounded ten-minute expiry before masking could hide them.
async fn nonce_contracts(client: &Client) {
    let mut nonces = Vec::new();
    for _ in 0..2 {
        let before = now_ms();
        let answer = client
            .request(
                "app nonce",
                "POST",
                "/app/v1/auth/nonce",
                &[("content-type", "application/json")],
                b"{}".to_vec(),
            )
            .await;
        let after = now_ms();
        assert_eq!(answer.status, 200, "nonce issuance status");
        let value = parsed(&answer);
        let nonce = value.get("nonce").as_str().expect("nonce must be a string");
        canonical_base64url(nonce, 24, "app nonce");
        fresh_expiry(value.get("expiresAt"), before, after, 600_000, "app nonce");
        nonces.push(nonce.to_owned());
    }
    assert!(
        nonces[0] != nonces[1],
        "independent app nonces must be unique"
    );
}

// Check independent OAuth attempts and consume them through local cancellation without an external exchange.
async fn oauth_contracts(client: &Client) {
    let mut attempts = Vec::new();
    for _ in 0..2 {
        let answer = client
            .request(
                "OAuth redirect",
                "GET",
                "/google/oauth2-redirect",
                &[],
                Vec::new(),
            )
            .await;
        assert_eq!(answer.status, 302, "OAuth redirect status");
        let location = answer
            .headers
            .get("location")
            .and_then(|value| value.to_str().ok())
            .expect("OAuth location header");
        let url = url::Url::parse(location).expect("OAuth location URL");
        assert!(
            url.scheme() == "https"
                && url.host_str() == Some("accounts.google.com")
                && url.path() == "/o/oauth2/v2/auth",
            "configured authorization destination is preserved without following it"
        );
        let query: BTreeMap<_, _> = url
            .query_pairs()
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect();
        let state = query.get("state").expect("OAuth state");
        let nonce = query.get("nonce").expect("OAuth nonce");
        let challenge = query.get("code_challenge").expect("PKCE challenge");
        canonical_base64url(state, 24, "OAuth state");
        canonical_base64url(nonce, 24, "OAuth nonce");
        canonical_base64url(challenge, 32, "PKCE challenge");
        assert!(
            state != nonce,
            "OAuth state and nonce must be independently generated"
        );
        assert!(
            query
                .get("code_challenge_method")
                .is_some_and(|value| value == "S256"),
            "PKCE must use S256"
        );
        assert!(
            query
                .get("redirect_uri")
                .is_some_and(|value| value == "http://127.0.0.1/google/oauth2-callback"),
            "callback remains on the synthetic public origin"
        );
        assert!(
            query
                .get("client_id")
                .is_some_and(|value| value == "offline-client"),
            "only the synthetic OAuth client may be used"
        );
        let value = cookie(&answer, "mm_oauth", "/google", 600);
        assert!(
            value == *state,
            "OAuth state cookie must match the freshly issued attempt"
        );
        attempts.push((state.to_owned(), nonce.to_owned(), challenge.to_owned()));
    }
    assert!(
        attempts[0].0 != attempts[1].0
            && attempts[0].1 != attempts[1].1
            && attempts[0].2 != attempts[1].2,
        "independent OAuth state, nonce and PKCE challenge must be unique"
    );
    for (state, _, _) in &attempts {
        let cookie = format!("mm_oauth={state}");
        let path = format!("/google/oauth2-callback?state={state}&error=access_denied");
        let answer = client
            .request(
                "local OAuth cancellation",
                "GET",
                &path,
                &[("cookie", &cookie)],
                Vec::new(),
            )
            .await;
        assert_eq!(answer.status, 200, "OAuth cancellation status");
        assert!(
            String::from_utf8_lossy(&answer.body).contains("error=Cancelled"),
            "fresh OAuth attempt must be recognized and canceled locally"
        );
        assert!(
            self::cookie(&answer, "mm_oauth", "/google", 0).is_empty(),
            "OAuth state cookie must be cleared"
        );
        let replay = client
            .request(
                "consumed OAuth cancellation",
                "GET",
                &path,
                &[("cookie", &cookie)],
                Vec::new(),
            )
            .await;
        assert_eq!(replay.status, 200, "consumed OAuth attempt status");
        assert!(
            String::from_utf8_lossy(&replay.body).contains("error=SignInFailed"),
            "consumed OAuth state must not be reusable"
        );
    }
}

// Check genuine independent sessions and tokens, then revoke each without invalidating its peer.
async fn session_contracts(client: &Client) -> String {
    let sessions = [client.session().await, client.session().await];
    let tokens = [client.app_token().await, client.app_token().await];
    let mut all = sessions.to_vec();
    all.extend(tokens.iter().cloned());
    all.sort();
    all.dedup();
    assert_eq!(
        all.len(),
        4,
        "session and app credentials must be independently unique"
    );
    for _ in 0..2 {
        for session in &sessions {
            client
                .me("cookie", &format!("mm_session={session}"), 200)
                .await;
            client
                .me("authorization", &format!("Bearer {session}"), 200)
                .await;
        }
        for token in &tokens {
            client
                .me("authorization", &format!("Bearer {token}"), 200)
                .await;
        }
    }
    let answer = client
        .request(
            "revoke first session",
            "GET",
            "/signout",
            &[("cookie", &format!("mm_session={}", sessions[0]))],
            Vec::new(),
        )
        .await;
    assert_eq!(answer.status, 302, "session sign-out status");
    assert!(
        cookie(&answer, "mm_session", "/", 0).is_empty(),
        "session cookie must be cleared"
    );
    client
        .me("cookie", &format!("mm_session={}", sessions[0]), 401)
        .await;
    client
        .me("authorization", &format!("Bearer {}", sessions[0]), 401)
        .await;
    client
        .me("cookie", &format!("mm_session={}", sessions[1]), 200)
        .await;
    for token in &tokens {
        client
            .me("authorization", &format!("Bearer {token}"), 200)
            .await;
    }
    let answer = client
        .request(
            "revoke first app token",
            "POST",
            "/app/v1/auth/signout",
            &[("authorization", &format!("Bearer {}", tokens[0]))],
            Vec::new(),
        )
        .await;
    assert_eq!(answer.status, 204, "app sign-out status");
    client
        .me("authorization", &format!("Bearer {}", tokens[0]), 401)
        .await;
    client
        .me("authorization", &format!("Bearer {}", tokens[1]), 200)
        .await;
    client
        .me("cookie", &format!("mm_session={}", sessions[1]), 200)
        .await;
    client
        .me("authorization", &format!("Bearer {}", sessions[1]), 200)
        .await;
    sessions[1].clone()
}

// Check capability uniqueness, exact lifetime, two-chunk reuse, token isolation and independent completion.
async fn upload_contracts(client: &Client, session: &str) {
    let first = client.upload(session, &[]).await;
    let second = client.upload(session, &[first.expiry]).await;
    assert!(
        first.path != second.path && first.token != second.token,
        "independent upload identifiers and capabilities must be unique"
    );
    client.chunk(session, &first, &second.token, 0, 401).await;
    client.chunk(session, &first, &first.token, 0, 200).await;
    client.chunk(session, &second, &second.token, 0, 200).await;
    client.chunk(session, &first, &first.token, 1, 200).await;
    client.chunk(session, &first, &first.token, 1, 409).await;
    client.chunk(session, &second, &second.token, 1, 200).await;
    client.chunk(session, &second, &second.token, 1, 409).await;
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires isolated local Rust Wrangler; run node tools/check-worker.mjs"]
async fn worker_credential_contracts() {
    let client = Client::new();
    client.seed().await;
    nonce_contracts(&client).await;
    oauth_contracts(&client).await;
    let session = session_contracts(&client).await;
    upload_contracts(&client, &session).await;
    println!(
        "unmasked Worker credential contracts passed: 2 nonces, 2 sessions, 2 app tokens, 2 OAuth attempts, 2 uploads; {} loopback requests",
        client.requests.get()
    );
}
