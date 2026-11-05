//! Sign-in for the replica (`lib/auth.mjs`): Google and password sign-in, sessions, app tokens, sign-out and the gate
//! in front of every other request.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use futures_util::future::{FutureExt, LocalBoxFuture, Shared};
use mm_js::{Object, Value, base64, date, json, number, obj, text, uri};
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::{BigUint, RsaPublicKey};
use sha2::{Digest, Sha256};
use url::Url;

use crate::auth_store::{Identity, check_password, install_auth_schema, password_subject};
use crate::host::Fetch;
use crate::http::{Body, Headers, Request, Response, search_param};
use crate::sql::{Row, Sql, SqlExt, SqlValue};
use crate::{Error, Result, params, runtime};

const SESSION_COOKIE: &str = "mm_session";
const STATE_COOKIE: &str = "mm_oauth";
const SESSION_MS: f64 = 30.0 * 24.0 * 3600.0 * 1000.0;
const LOGIN_MS: f64 = 10.0 * 60.0 * 1000.0;
const APP_TOKEN_MS: f64 = 90.0 * 24.0 * 3600.0 * 1000.0;
/// How the MCP server names the app tokens it holds (`mcp/src/signin.ts`): `MCP: <client name>`.
const AGENT_DEVICE: &str = "MCP: ";
const SKEW_S: f64 = 60.0;
const FAILURE_WINDOW_MS: f64 = 15.0 * 60.0 * 1000.0;
const FAILURES_PER_USER: f64 = 10.0;
const FAILURES_PER_ADDRESS: f64 = 30.0;
const KEY_REFRESH_MS: f64 = 60_000.0;
const CERTS_TIMEOUT_MS: u64 = 10_000;
const HOUR_MS: f64 = 3_600_000.0;
const NOT_FOUND: &str = "{\"type\":\"about:blank\",\"status\":401,\"title\":\"Unauthorized\"}";
const SIGNIN_SCRIPT: &str = "new BroadcastChannel('replica-auth').onmessage = (event) => location.assign(event.data);\ndocument.addEventListener('click', (event) => { if (event.target.closest?.('#appleid-signin')) { event.preventDefault(); event.stopImmediatePropagation(); } }, true);\n";

fn password_error(key: &str) -> Option<&'static str> {
    match key {
        "invalid" => Some("That username and password do not match."),
        "locked" => Some("Too many attempts. Try again in 15 minutes."),
        _ => None,
    }
}

fn signin_error(key: &str) -> Option<&'static str> {
    match key {
        "AccountNotFound" => Some("That Google account does not have access."),
        "Cancelled" => Some("Sign-in was cancelled."),
        "SignInFailed" => Some("Sign-in did not work. Try again."),
        _ => None,
    }
}

/// What [`Auth::new`] needs from its host.
pub struct AuthOptions {
    /// Configuration source (`process.env` or the Worker's variables and secrets).
    pub env: HashMap<String, String>,
    /// Port the server listens on, for the default public URL.
    pub port: u16,
    /// Open sign-in database; the tables are created when missing.
    pub db: Rc<dyn Sql>,
    /// Outbound HTTP for Google's token and key endpoints.
    pub fetch: Rc<dyn Fetch>,
    /// Source of random bytes for session ids and tokens.
    pub random: fn(usize) -> Vec<u8>,
}

impl AuthOptions {
    /// Options with the platform's random source.
    ///
    /// # Arguments
    /// * `env` - Configuration source.
    /// * `port` - Port the server listens on.
    /// * `db` - Sign-in database.
    /// * `fetch` - Outbound HTTP.
    pub fn new(env: HashMap<String, String>, port: u16, db: Rc<dyn Sql>, fetch: Rc<dyn Fetch>) -> Self {
        Self { env, port, db, fetch, random: runtime::random_bytes }
    }
}

/// Domain admitted beside the owner when `ALLOWED_GOOGLE_DOMAIN` is unset.
const DEFAULT_GOOGLE_DOMAIN: &str = "octaviantocan.com";

// The Google client and the owner `AUTH=google` needs.
fn credentials(env: &HashMap<String, String>) -> Result<(String, String, String)> {
    let client_id = env.get("GOOGLE_CLIENT_ID").cloned().unwrap_or_default();
    let client_secret = env.get("GOOGLE_CLIENT_SECRET").cloned().unwrap_or_default();
    let owner = env.get("OWNER_EMAIL").map(|owner| text::to_lower(text::trim(owner))).unwrap_or_default();
    if client_id.is_empty() || client_secret.is_empty() || owner.is_empty() {
        return Err(Error::internal("AUTH=google needs GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET and OWNER_EMAIL"));
    }
    Ok((client_id, client_secret, owner))
}

// `ALLOWED_GOOGLE_DOMAIN` when it is a hostname. Unset uses [`DEFAULT_GOOGLE_DOMAIN`]. Empty or not a hostname means
// the owner is the only Google account.
fn allowed_domain(env: &HashMap<String, String>) -> Option<String> {
    match env.get("ALLOWED_GOOGLE_DOMAIN") {
        None => normalize_domain(DEFAULT_GOOGLE_DOMAIN),
        Some(value) => normalize_domain(value),
    }
}

fn normalize_domain(value: &str) -> Option<String> {
    let trimmed = text::to_lower(text::trim(value));
    let domain = trimmed.strip_prefix('@').unwrap_or(&trimmed);
    let ok = !domain.is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.starts_with('-')
        && !domain.ends_with('-')
        && !domain.contains("..")
        && domain.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-');
    ok.then(|| domain.to_owned())
}

/// Whether `env` turns sign-in on, checked before the host opens the sign-in database.
///
/// # Arguments
/// * `env` - Configuration source.
///
/// # Returns
/// `true` when `AUTH` is `google`.
///
/// # Errors
/// The error [`Auth::new`] gives when the Google client or the owner is missing.
pub fn enabled(env: &HashMap<String, String>) -> Result<bool> {
    if env.get("AUTH").map(String::as_str) != Some("google") {
        return Ok(false);
    }
    credentials(env).map(|_| true)
}

/// Who a request is signed in as, with the credential to check later with [`Auth::active`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionIdentity {
    /// Session subject.
    pub sub: String,
    /// Signed-in email.
    pub email: String,
    /// Hash naming the session or app token.
    pub credential: String,
}

struct Config {
    client_id: String,
    client_secret: String,
    owner: String,
    allowed_domain: Option<String>,
    public_url: String,
    secure: bool,
    redirect_uri: String,
    authorize: String,
    token: String,
    certs: String,
    issuers: Vec<String>,
    nonce_ms: f64,
}

enum KeyEntry {
    Rsa(RsaPublicKey),
    Unusable,
}

#[derive(Default)]
struct Keys {
    fetched: f64,
    max_age: f64,
    by_kid: HashMap<String, Rc<KeyEntry>>,
}

struct Inner {
    cfg: Config,
    db: Rc<dyn Sql>,
    fetch: Rc<dyn Fetch>,
    random: fn(usize) -> Vec<u8>,
    keys: RefCell<Keys>,
    keys_attempted: Cell<f64>,
    loading: RefCell<Option<Shared<LocalBoxFuture<'static, Result<()>>>>>,
}

/// Sign-in handlers (`createAuth`'s return value).
pub struct Auth(Rc<Inner>);

struct SessionRow {
    hash: String,
    sub: String,
    email: String,
}

enum PasswordResult {
    Name(String),
    Invalid,
    Locked,
}

enum Callback {
    Problem(&'static str),
    Id(String),
}

fn sha256_hex(text: &str) -> String {
    base64::hex(&Sha256::digest(text.as_bytes()))
}

fn bind(value: &Value) -> Result<SqlValue> {
    match value {
        Value::String(text) => Ok(SqlValue::Text(text.clone())),
        Value::Number(number) => Ok(SqlValue::Real(*number)),
        Value::Null | Value::Object(_) => Ok(SqlValue::Null),
        _ => Err(Error::invalid("The value cannot be bound")),
    }
}

fn iso(ms: f64) -> Result<String> {
    date::iso_from_ms(ms).ok_or_else(|| Error::invalid("Invalid time value"))
}

fn cookie_value(req: &Request, name: &str) -> Option<String> {
    let mut found = None;
    for part in req.header("cookie").unwrap_or("").split(';') {
        let part = text::trim(part);
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        if key.is_empty() || key != name {
            continue;
        }
        found = Some(uri::decode_uri_component(value).unwrap_or_else(|_| value.to_owned()));
    }
    found
}

fn cookie(name: &str, value: &str, max_age: f64, secure: bool, path: &str) -> String {
    format!(
        "{name}={}; Path={path}; Max-Age={}; HttpOnly; SameSite=Lax{}",
        uri::encode_uri_component(value),
        number::number_to_string((max_age / 1000.0).floor()),
        if secure { "; Secure" } else { "" }
    )
}

fn is_websocket(req: &Request) -> bool {
    req.header("upgrade").is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
}

fn bearer(req: &Request) -> Option<String> {
    let header = req.header("authorization").unwrap_or("");
    header.get(..6).filter(|prefix| prefix.eq_ignore_ascii_case("bearer"))?;
    let rest = header[6..].trim_start_matches(' ');
    if rest.len() == header.len() - 6 || rest.is_empty() || rest.chars().any(text::is_whitespace) {
        return None;
    }
    Some(rest.to_owned())
}

fn is_app_signin(path: &str) -> Option<&str> {
    path.strip_prefix("/app/v1/auth/").filter(|step| matches!(*step, "nonce" | "google" | "password"))
}

fn is_public(path: &str, tunnelled: bool) -> bool {
    let signin = is_app_signin(path).is_some() || path == "/signin" || path == "/google/oauth2-redirect" || path == "/google/oauth2-callback";
    signin || (!tunnelled && (path.starts_with("/_static/") || path.starts_with("/_auth/") || path == "/manifest.webmanifest" || path == "/favicon.ico" || path == "/jserrors"))
}

fn headers(pairs: &[(&str, &str)]) -> Headers {
    let mut headers = Headers::new();
    for (name, value) in pairs {
        headers.append(*name, *value);
    }
    headers
}

fn page_response(status: u16, content_type: &str, cookies: &[String], body: String) -> Response {
    let mut out = headers(&[("content-type", content_type), ("cache-control", "no-store")]);
    for value in cookies {
        out.append("set-cookie", value.as_str());
    }
    Response::new(status, out, body)
}

fn html(body: String, cookies: &[String]) -> Response {
    page_response(200, "text/html; charset=utf-8", cookies, body)
}

fn redirect_with(status: u16, location: &str, cookies: &[String]) -> Response {
    let mut out = headers(&[("location", location), ("cache-control", "no-store")]);
    for value in cookies {
        out.append("set-cookie", value.as_str());
    }
    Response::new(status, out, Body::Empty)
}

fn redirect(location: &str) -> Response {
    redirect_with(302, location, &[])
}

fn json_reply(status: u16, body: Object) -> Response {
    Response::new(status, headers(&[("content-type", "application/json; charset=utf-8"), ("cache-control", "no-store")]), json::to_string(&body.into()))
}

fn bare(status: u16) -> Response {
    Response::new(status, Headers::new(), Body::Empty)
}

fn popup_page(target: &str) -> String {
    let to = json::quote(target).replace('<', "\\u003c");
    format!("{POPUP_HEAD}{to}{POPUP_TAIL}")
}

fn plain_page(body: &str) -> String {
    format!("{PLAIN_HEAD}{body}</html>")
}

fn plain_signin_page(error: Option<&str>, passwords: bool) -> String {
    let message = error.and_then(signin_error).map_or_else(String::new, |message| format!("<p class=\"error\" role=\"alert\">{message}</p>"));
    let link = if passwords { "<a href=\"/signin/password\">Sign in with a password</a>\n" } else { "" };
    plain_page(&format!(
        "<main class=\"card\">\n<h1>Sign in</h1>\n<p>This library is private. Sign in with an account that has access.</p>{message}\n<a class=\"button\" id=\"google\" href=\"/google/oauth2-redirect\">{GOOGLE_LOGO}Continue with Google</a>\n{link}</main>\n<script>\nnew BroadcastChannel('replica-auth').onmessage = (event) => location.assign(event.data);\ndocument.getElementById('google').addEventListener('click', (event) => {{\n  if (window.open('/google/oauth2-redirect?popup=true', 'signin', 'popup,width=500,height=640')) event.preventDefault();\n}});\n</script>"
    ))
}

fn password_page(error: Option<&str>) -> String {
    let message = error.and_then(password_error).map_or_else(String::new, |message| format!("<p class=\"error\" role=\"alert\">{message}</p>"));
    plain_page(&format!(
        "<form class=\"card\" method=\"post\" action=\"/signin/password\">\n<h1>Sign in</h1>{message}\n<label>Username <input name=\"username\" autocomplete=\"username\" autocapitalize=\"none\" spellcheck=\"false\" required autofocus></label>\n<label>Password <input name=\"password\" type=\"password\" autocomplete=\"current-password\" required></label>\n<button type=\"submit\">Sign in</button>\n<a href=\"/signin\">Sign in with Google instead</a>\n</form>"
    ))
}

async fn read_text(req: &mut Request, limit: usize) -> Result<Option<String>> {
    match req.take_body().bytes_within(limit).await {
        Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
        Err(Error::Status { status: 413, .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

async fn read_json(req: &mut Request) -> Result<Option<Object>> {
    let Some(text) = read_text(req, 16384).await? else { return Ok(None) };
    Ok(match json::parse(if text.is_empty() { "{}" } else { &text }) {
        Ok(Value::Object(object)) => Some(object),
        _ => None,
    })
}

async fn read_form(req: &mut Request) -> Result<Option<Vec<(String, String)>>> {
    let Some(text) = read_text(req, 4096).await? else { return Ok(None) };
    let query = text.strip_prefix('?').unwrap_or(&text);
    Ok(Some(url::form_urlencoded::parse(query.as_bytes()).map(|(key, value)| (key.into_owned(), value.into_owned())).collect()))
}

fn form_get<'a>(form: &'a [(String, String)], name: &str) -> Option<&'a str> {
    form.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

fn kid_key(kid: &Value) -> Option<String> {
    match kid {
        Value::String(text) => Some(format!("s{text}")),
        Value::Number(number) => Some(format!("n{}", number::number_to_string(*number))),
        Value::Bool(value) => Some(format!("b{value}")),
        Value::Null => Some("null".into()),
        Value::Undefined => Some("undefined".into()),
        _ => None,
    }
}

fn read_jwk(jwk: &Value) -> Result<(Option<String>, KeyEntry)> {
    let Value::Object(object) = jwk else { return Err(Error::invalid("The JWK is not an object")) };
    let string = |name: &str| object.get(name).and_then(Value::as_str);
    let entry = match string("kty") {
        Some("RSA") => {
            let (Some(n), Some(e)) = (string("n"), string("e")) else { return Err(Error::invalid("Invalid JWK RSA key")) };
            match RsaPublicKey::new(BigUint::from_bytes_be(&base64::decode(n)), BigUint::from_bytes_be(&base64::decode(e))) {
                Ok(key) => KeyEntry::Rsa(key),
                Err(_) => KeyEntry::Unusable,
            }
        }
        Some("EC") if string("crv").is_some() && string("x").is_some() && string("y").is_some() => KeyEntry::Unusable,
        Some("OKP") if string("crv").is_some() && string("x").is_some() => KeyEntry::Unusable,
        _ => return Err(Error::invalid("Invalid JWK key")),
    };
    Ok((kid_key(object.get("kid").unwrap_or(&Value::Undefined)), entry))
}

fn max_age_ms(header: &str) -> f64 {
    let mut rest = header;
    while let Some(at) = rest.find("max-age=") {
        let tail = &rest[at + 8..];
        let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            return number::string_to_number(&digits) * 1000.0;
        }
        rest = tail;
    }
    3600.0 * 1000.0
}

fn body_text(bytes: &[u8]) -> String {
    text::decode_utf8(bytes)
}

fn header_ci<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value)
}

fn verify_signature(key: &RsaPublicKey, message: &[u8], signature: &[u8]) -> bool {
    let size = rsa::traits::PublicKeyParts::size(key);
    if signature.len() > size {
        return false;
    }
    let mut padded = vec![0; size - signature.len()];
    padded.extend_from_slice(signature);
    let Ok(signature) = Signature::try_from(padded.as_slice()) else { return false };
    VerifyingKey::<Sha256>::new(key.clone()).verify(message, &signature).is_ok()
}

impl Inner {
    fn now(&self) -> f64 {
        runtime::now_ms()
    }

    fn token_id(&self, bytes: usize) -> String {
        base64::encode_url(&(self.random)(bytes))
    }

    fn cookie_session(&self, req: &Request) -> Result<Option<SessionRow>> {
        let Some(id) = cookie_value(req, SESSION_COOKIE).filter(|id| !id.is_empty()) else { return Ok(None) };
        self.stored_session(&id)
    }

    /// A browser extension cannot send the `HttpOnly` cookie on cross-origin requests, so it sends the cookie's value as a bearer token.
    fn bearer_session(&self, req: &Request) -> Result<Option<SessionRow>> {
        let Some(id) = bearer(req) else { return Ok(None) };
        self.stored_session(&id)
    }

    fn stored_session(&self, id: &str) -> Result<Option<SessionRow>> {
        let Some(row) = self.db.one("SELECT hash, sub, email, expires_ms, used_ms FROM sessions WHERE hash = ?", params![sha256_hex(id)])? else { return Ok(None) };
        if row.number("expires_ms").unwrap_or(f64::NAN) < self.now() {
            return Ok(None);
        }
        let hash = row.text("hash").unwrap_or_default().to_owned();
        if self.now() - row.number("used_ms").unwrap_or(f64::NAN) > HOUR_MS {
            self.db.execute("UPDATE sessions SET used_ms = ? WHERE hash = ?", params![self.now(), &hash])?;
        }
        Ok(Some(session_row(&row, hash)))
    }

    fn app_session(&self, req: &Request) -> Result<Option<SessionRow>> {
        let Some(token) = bearer(req) else { return Ok(None) };
        let Some(row) = self.db.one("SELECT hash, sub, email, expires_ms, used_ms FROM app_tokens WHERE hash = ?", params![sha256_hex(&token)])? else { return Ok(None) };
        if row.number("expires_ms").unwrap_or(f64::NAN) < self.now() {
            return Ok(None);
        }
        let hash = row.text("hash").unwrap_or_default().to_owned();
        if self.now() - row.number("used_ms").unwrap_or(f64::NAN) > HOUR_MS {
            self.db.execute("UPDATE app_tokens SET used_ms = ?, expires_ms = ? WHERE hash = ?", params![self.now(), self.now() + APP_TOKEN_MS, &hash])?;
        }
        Ok(Some(session_row(&row, hash)))
    }

    fn foreign_origin(&self, req: &Request) -> Result<bool> {
        let Some(origin) = req.header("origin").filter(|origin| !origin.is_empty()) else { return Ok(false) };
        let own = mm_js::url::parse(&self.cfg.public_url, None).map(|url| mm_js::url::origin(&url)).ok_or_else(|| Error::invalid("Invalid URL"))?;
        let local = format!("http://{}", req.header("host").unwrap_or("undefined"));
        Ok(origin != own && origin != local)
    }

    fn session(&self, req: &Request) -> Result<Option<SessionRow>> {
        if let Some(row) = self.cookie_session(req)? {
            return Ok(Some(row));
        }
        match self.app_session(req)? {
            Some(row) => Ok(Some(row)),
            None => self.bearer_session(req),
        }
    }

    // The owner, or a verified address whose host is exactly `allowed_domain`. `email_verified` must be boolean true.
    // A subdomain, a longer suffix, or a different character that only looks like the domain does not match.
    fn google_access(&self, claims: &Value) -> Option<String> {
        if !matches!(claims.get("email_verified"), Value::Bool(true)) {
            return None;
        }
        let email = text::to_lower(text::trim(&claims.get("email").to_js_string()));
        if email == self.cfg.owner {
            return Some(self.cfg.owner.clone());
        }
        let domain = self.cfg.allowed_domain.as_deref()?;
        let (local, host) = email.split_once('@')?;
        (host == domain && !local.is_empty()).then_some(email)
    }

    // Connected agents (`docs/mcp.md`): the app tokens the MCP server holds for this account, which it names
    // `MCP: <client>`. `GET /app/v1/agents` lists them and `DELETE /app/v1/agents/<id>` signs one out; the MCP server
    // then drops its grant on that agent's next call.
    fn agents(&self, req: &Request, pathname: &str, row: &SessionRow) -> Result<Option<Response>> {
        let agent_like = format!("{AGENT_DEVICE}%");
        if pathname == "/app/v1/agents" && req.method == "GET" {
            let rows = self.db.query(
                "SELECT hash, device, created_ms, used_ms FROM app_tokens WHERE sub = ? AND device LIKE ? AND expires_ms > ? ORDER BY used_ms DESC",
                params![row.sub.as_str(), agent_like.as_str(), self.now()],
            )?;
            let mut agents = Vec::with_capacity(rows.len());
            for agent in rows {
                let name = agent.text("device").unwrap_or("").strip_prefix(AGENT_DEVICE).unwrap_or("").trim().to_owned();
                agents.push(Value::from(obj! {
                    "id" => agent.text("hash").unwrap_or(""),
                    "name" => if name.is_empty() { "An AI app".to_owned() } else { name },
                    "connected" => iso(agent.number("created_ms").unwrap_or(0.0))?,
                    "used" => iso(agent.number("used_ms").unwrap_or(0.0))?,
                }));
            }
            return Ok(Some(json_reply(200, obj! { "agents" => agents })));
        }
        if req.method == "DELETE"
            && let Some(id) = pathname.strip_prefix("/app/v1/agents/")
        {
            let removed = self.db.execute(
                "DELETE FROM app_tokens WHERE hash = ? AND sub = ? AND device LIKE ?",
                params![id, row.sub.as_str(), agent_like.as_str()],
            )?;
            return Ok(Some(if removed == 0 { json_reply(404, obj! { "error" => "not-found" }) } else { bare(204) }));
        }
        Ok(None)
    }

    fn start_app_token(&self, sub: &Value, email: &str, kind: &str, device: &Value) -> Result<Object> {
        let token = self.token_id(32);
        let now = self.now();
        let device = if device.is_nullish() { String::new() } else { device.to_js_string() };
        let sub = bind(sub);
        self.db.execute(
            "INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlValue::Text(sha256_hex(&token)),
                sub?,
                SqlValue::Text(email.to_owned()),
                SqlValue::Text(kind.to_owned()),
                SqlValue::Text(text::slice(&device, 0, Some(100))),
                SqlValue::Real(now),
                SqlValue::Real(now + APP_TOKEN_MS),
                SqlValue::Real(now),
            ],
        )?;
        Ok(obj! { "token" => token, "expiresAt" => iso(now + APP_TOKEN_MS)?, "user" => obj! { "email" => email, "kind" => kind } })
    }

    fn start_session(&self, sub: &Value, email: &str) -> Result<String> {
        let id = self.token_id(32);
        let now = self.now();
        let sub = bind(sub);
        self.db.execute(
            "INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES (?, ?, ?, ?, ?, ?)",
            &[SqlValue::Text(sha256_hex(&id)), sub?, SqlValue::Text(email.to_owned()), SqlValue::Real(now), SqlValue::Real(now + SESSION_MS), SqlValue::Real(now)],
        )?;
        Ok(id)
    }

    fn recent_failures(&self, key: &str) -> Result<f64> {
        let row = self.db.one("SELECT COUNT(*) AS n FROM failures WHERE key = ? AND at_ms > ?", params![key, self.now() - FAILURE_WINDOW_MS])?;
        Ok(row.and_then(|row| row.number("n")).unwrap_or(f64::NAN))
    }

    async fn fetch_keys(self: Rc<Self>) -> Result<()> {
        self.keys_attempted.set(self.now());
        let request = Request::new("GET", &self.cfg.certs)?;
        let fetched = runtime::timeout(CERTS_TIMEOUT_MS, async {
            let response = self.fetch.fetch(request).await?;
            if !(200..300).contains(&response.status) {
                return Err(Error::internal(format!("certs {}", response.status)));
            }
            let max_age = max_age_ms(header_ci(&response, "cache-control").unwrap_or(""));
            Ok((max_age, response.body.bytes().await?))
        });
        let (max_age, bytes) = fetched.await.ok_or_else(|| Error::internal("TimeoutError: The operation was aborted due to timeout"))??;
        let body = json::parse(&body_text(&bytes))?;
        let Value::Array(list) = (match &body {
            Value::Object(object) => object.get("keys").cloned().unwrap_or_default(),
            _ => return Err(Error::invalid("The key set is not an object")),
        }) else {
            return Err(Error::invalid("The key set has no keys"));
        };
        let mut by_kid = HashMap::new();
        for jwk in &list {
            let (kid, entry) = read_jwk(jwk)?;
            if let Some(kid) = kid {
                by_kid.insert(kid, Rc::new(entry));
            }
        }
        *self.keys.borrow_mut() = Keys { fetched: self.now(), max_age, by_kid };
        Ok(())
    }

    async fn signing_key(self: &Rc<Self>, kid: &Value) -> Result<Rc<KeyEntry>> {
        let stale = self.now() - self.keys.borrow().fetched > self.keys.borrow().max_age;
        let known = kid_key(kid).is_some_and(|key| self.keys.borrow().by_kid.contains_key(&key));
        if stale || (!known && self.now() - self.keys_attempted.get() > KEY_REFRESH_MS) {
            let shared = self
                .loading
                .borrow_mut()
                .get_or_insert_with(|| {
                    let me = self.clone();
                    async move {
                        let result = me.clone().fetch_keys().await;
                        *me.loading.borrow_mut() = None;
                        result
                    }
                    .boxed_local()
                    .shared()
                })
                .clone();
            shared.await?;
        }
        kid_key(kid).and_then(|key| self.keys.borrow().by_kid.get(&key).cloned()).ok_or_else(|| Error::internal("unknown signing key"))
    }

    async fn verify_id_token(self: &Rc<Self>, token: &str, nonce: &str) -> Result<Value> {
        let mut parts = token.split('.');
        let header = parts.next().unwrap_or_default();
        let payload = parts.next();
        let signature = parts.next();
        let head = json::parse(&String::from_utf8_lossy(&base64::decode(header)))?;
        let Value::Object(head) = head else { return Err(Error::internal("unexpected token algorithm")) };
        if !matches!(head.get("alg"), Some(Value::String(alg)) if alg == "RS256") {
            return Err(Error::internal("unexpected token algorithm"));
        }
        let message = format!("{header}.{}", payload.unwrap_or("undefined"));
        let key = self.signing_key(head.get("kid").unwrap_or(&Value::Undefined)).await?;
        let signature = signature.ok_or_else(|| Error::invalid("The signature is missing"))?;
        let verified = match &*key {
            KeyEntry::Rsa(key) => verify_signature(key, message.as_bytes(), &base64::decode(signature)),
            KeyEntry::Unusable => false,
        };
        if !verified {
            return Err(Error::internal("bad token signature"));
        }
        let claims = json::parse(&String::from_utf8_lossy(&base64::decode(payload.unwrap_or_default())))?;
        if !matches!(claims, Value::Object(_)) {
            return Err(Error::internal("unexpected issuer"));
        }
        let now = self.now() / 1000.0;
        if !matches!(claims.get("iss"), Value::String(iss) if self.cfg.issuers.contains(iss)) {
            return Err(Error::internal("unexpected issuer"));
        }
        if !matches!(claims.get("aud"), Value::String(aud) if *aud == self.cfg.client_id) {
            return Err(Error::internal("unexpected audience"));
        }
        // Negated on purpose: a NaN claim (missing or non-numeric) must fail, as the JS `!(a > b)` does.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let expired = !(claims.get("exp").to_number() > now - SKEW_S) || !(claims.get("iat").to_number() < now + SKEW_S);
        if expired {
            return Err(Error::internal("token expired"));
        }
        if !matches!(claims.get("nonce"), Value::String(found) if found == nonce) {
            return Err(Error::internal("nonce mismatch"));
        }
        Ok(claims)
    }

    async fn exchange(&self, code: &str, verifier: &str) -> Result<String> {
        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("code", code)
            .append_pair("client_id", &self.cfg.client_id)
            .append_pair("client_secret", &self.cfg.client_secret)
            .append_pair("redirect_uri", &self.cfg.redirect_uri)
            .append_pair("grant_type", "authorization_code")
            .append_pair("code_verifier", verifier)
            .finish();
        let mut request = Request::new("POST", &self.cfg.token)?;
        request.headers = headers(&[("content-type", "application/x-www-form-urlencoded"), ("accept", "application/json")]);
        request.body = Body::Bytes(body.into_bytes());
        let response = self.fetch.fetch(request).await?;
        let status = response.status;
        let bytes = response.body.bytes().await.unwrap_or_default();
        let parsed = json::parse(&body_text(&bytes)).unwrap_or_else(|_| Value::Object(Object::new()));
        match parsed.get("id_token") {
            Value::String(token) if (200..300).contains(&status) => Ok(token.clone()),
            _ => Err(Error::internal(format!("token exchange {status}"))),
        }
    }

    async fn callback(self: &Rc<Self>, url: &Url, req: &Request) -> Result<Callback> {
        let state = search_param(url, "state").unwrap_or_default();
        let login = self.db.one("SELECT verifier, nonce, created_ms FROM logins WHERE state = ?", params![&state])?;
        self.db.execute("DELETE FROM logins WHERE state = ?", params![&state])?;
        let Some(login) = login else { return Err(Error::internal("unknown sign-in attempt")) };
        if cookie_value(req, STATE_COOKIE).as_deref() != Some(state.as_str()) || self.now() - login.number("created_ms").unwrap_or(f64::NAN) > LOGIN_MS {
            return Err(Error::internal("unknown sign-in attempt"));
        }
        if search_param(url, "error").is_some_and(|error| !error.is_empty()) {
            return Ok(Callback::Problem("Cancelled"));
        }
        let code = search_param(url, "code").unwrap_or_default();
        let id_token = self.exchange(&code, login.text("verifier").unwrap_or_default()).await?;
        let claims = self.verify_id_token(&id_token, login.text("nonce").unwrap_or_default()).await?;
        let Some(email) = self.google_access(&claims) else { return Ok(Callback::Problem("AccountNotFound")) };
        Ok(Callback::Id(self.start_session(claims.get("sub"), &email)?))
    }

    fn verify_password(&self, req: &Request, raw_username: &Value, password: Option<&str>) -> Result<PasswordResult> {
        let username = text::to_lower(text::trim(&if raw_username.is_nullish() { String::new() } else { raw_username.to_js_string() }));
        let address = req.header("cf-connecting-ip").unwrap_or(&req.remote_address).to_owned();
        let keys = [format!("user:{username}"), format!("address:{address}")];
        self.db.execute("DELETE FROM failures WHERE at_ms <= ?", params![self.now() - FAILURE_WINDOW_MS])?;
        if self.recent_failures(&keys[0])? >= FAILURES_PER_USER || self.recent_failures(&keys[1])? >= FAILURES_PER_ADDRESS {
            return Ok(PasswordResult::Locked);
        }
        let attempt = self.token_id(12);
        for key in &keys {
            self.db.execute("INSERT INTO failures (key, attempt, at_ms) VALUES (?, ?, ?)", params![key, &attempt, self.now()])?;
        }
        let name = match password {
            Some(password) => check_password(&*self.db, &username, password)?,
            None => None,
        };
        let Some(name) = name else { return Ok(PasswordResult::Invalid) };
        self.db.execute("DELETE FROM failures WHERE key = ?", params![&keys[0]])?;
        self.db.execute("DELETE FROM failures WHERE key = ? AND attempt = ?", params![&keys[1], &attempt])?;
        Ok(PasswordResult::Name(name))
    }

    async fn app_sign_in(self: &Rc<Self>, req: &mut Request, step: &str) -> Result<Response> {
        if req.method != "POST" {
            return Ok(Response::new(405, headers(&[("allow", "POST")]), Body::Empty));
        }
        if step == "nonce" {
            self.db.execute("DELETE FROM app_nonces WHERE created_ms < ?", params![self.now() - self.cfg.nonce_ms])?;
            self.db.execute("DELETE FROM app_nonces WHERE nonce IN (SELECT nonce FROM app_nonces ORDER BY created_ms DESC LIMIT -1 OFFSET 9999)", &[])?;
            let nonce = self.token_id(24);
            self.db.execute("INSERT INTO app_nonces (nonce, created_ms) VALUES (?, ?)", params![&nonce, self.now()])?;
            return Ok(json_reply(200, obj! { "nonce" => nonce, "expiresAt" => iso(self.now() + self.cfg.nonce_ms)? }));
        }
        let body = read_json(req).await?;
        let invalid = || json_reply(400, obj! { "error" => "invalid-token" });
        if step == "google" {
            let nonce = match &body {
                Some(body) => body.get("nonce").filter(|value| !value.is_nullish()).map(Value::to_js_string).unwrap_or_default(),
                None => String::new(),
            };
            let created = self.db.one("SELECT created_ms FROM app_nonces WHERE nonce = ?", params![&nonce])?.and_then(|row| row.number("created_ms"));
            self.db.execute("DELETE FROM app_nonces WHERE nonce = ?", params![&nonce])?;
            let Some(created) = created else { return Ok(invalid()) };
            let Some(body) = body else { return Err(Error::invalid("Cannot read properties of null")) };
            let id_token = match body.get("idToken") {
                Some(Value::String(token)) if self.now() - created <= self.cfg.nonce_ms || self.cfg.nonce_ms.is_nan() => token.clone(),
                _ => return Ok(invalid()),
            };
            let Ok(claims) = self.verify_id_token(&id_token, &nonce).await else { return Ok(invalid()) };
            if !matches!(claims.get("email_verified"), Value::Bool(true)) {
                return Ok(invalid());
            }
            return match self.google_access(&claims) {
                Some(email) => Ok(json_reply(200, self.start_app_token(claims.get("sub"), &email, "google", body.get("device").unwrap_or(&Value::Undefined))?)),
                None => Ok(json_reply(401, obj! { "error" => "not-allowed" })),
            };
        }
        let result = match &body {
            Some(body) => {
                let password = match body.get("password") {
                    Some(Value::String(password)) => Some(password.as_str()),
                    _ => None,
                };
                self.verify_password(req, body.get("username").unwrap_or(&Value::Undefined), password)?
            }
            None => PasswordResult::Invalid,
        };
        match result {
            PasswordResult::Locked => Ok(json_reply(429, obj! { "error" => "locked" })),
            PasswordResult::Invalid => Ok(json_reply(401, obj! { "error" => "invalid" })),
            PasswordResult::Name(name) => {
                let body = body.unwrap_or_default();
                Ok(json_reply(200, self.start_app_token(&Value::String(password_subject(&name)), &name, "password", body.get("device").unwrap_or(&Value::Undefined))?))
            }
        }
    }

    fn passwords(&self) -> Result<bool> {
        Ok(self.db.one("SELECT 1 FROM users LIMIT 1", &[])?.is_some())
    }
}

fn session_row(row: &Row, hash: String) -> SessionRow {
    SessionRow { hash, sub: row.text("sub").unwrap_or_default().to_owned(), email: row.text("email").unwrap_or_default().to_owned() }
}
const GOOGLE_LOGO: &str = r####"<svg width="18" height="18" viewBox="0 0 48 48" aria-hidden="true"><path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"/><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"/><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"/><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"/></svg>"####;
const PLAIN_HEAD: &str = r####"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex, nofollow"><link rel="icon" href="data:,"><title>Sign in</title><style>
:root { color-scheme: light dark; --bg: #f5f5f3; --card: #fff; --text: #1d1d1f; --muted: #6e6e73; --line: #d8d8dc; --button: #1d1d1f; --button-text: #fff; }
@media (prefers-color-scheme: dark) { :root { --bg: #111113; --card: #1c1c1f; --text: #f2f2f4; --muted: #a1a1a6; --line: #333338; --button: #f2f2f4; --button-text: #111113; } }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 16px; background: var(--bg); color: var(--text); font: 16px/1.4 system-ui, sans-serif; }
.card { width: 100%; max-width: 360px; display: grid; gap: 14px; padding: 32px 28px; background: var(--card); border: 1px solid var(--line); border-radius: 16px; }
h1 { margin: 0 0 6px; font-size: 22px; font-weight: 600; }
p { margin: 0; color: var(--muted); font-size: 15px; }
label { display: grid; gap: 6px; font-size: 14px; color: var(--muted); }
input { font: inherit; color: var(--text); background: transparent; padding: 11px 12px; border: 1px solid var(--line); border-radius: 10px; }
input:focus { outline: 2px solid var(--text); outline-offset: -1px; border-color: transparent; }
a { color: var(--muted); font-size: 14px; text-align: center; }
button, .button { display: flex; align-items: center; justify-content: center; gap: 10px; font: inherit; font-weight: 600; margin-top: 6px; padding: 12px; border: 0; border-radius: 10px; background: var(--button); color: var(--button-text); cursor: pointer; text-decoration: none; }
.error { color: #d93a1a; font-size: 14px; }
</style>
"####;
const POPUP_HEAD: &str = r####"<!doctype html><meta charset="utf-8"><title>Signing in…</title><script>
const to = "####;
const POPUP_TAIL: &str = r####";
try { new BroadcastChannel('replica-auth').postMessage(to); } catch {}
try { if (opener && opener.location.origin === location.origin) { opener.location.assign(to); close(); } } catch {}
if (opener) close();
setTimeout(() => location.replace(to), 300);
</script>"####;

impl Auth {
    /// Turn sign-in on from configuration (`createAuth`), or report that it is off.
    ///
    /// # Arguments
    /// * `options` - Configuration, database, outbound HTTP and sign-in pages.
    ///
    /// # Returns
    /// The handlers, or `None` unless `AUTH` is `google`.
    ///
    /// # Errors
    /// An internal error when `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` or `OWNER_EMAIL` is missing, or the database's error.
    pub fn new(options: AuthOptions) -> Result<Option<Auth>> {
        let env = &options.env;
        if env.get("AUTH").map(String::as_str) != Some("google") {
            return Ok(None);
        }
        let (client_id, client_secret, owner) = credentials(env)?;
        let allowed_domain = allowed_domain(env);
        let configured = env.get("PUBLIC_URL").cloned().unwrap_or_else(|| format!("http://localhost:{}", options.port));
        let public_url = configured.strip_suffix('/').unwrap_or(&configured).to_owned();
        let var = |name: &str, default: &str| env.get(name).cloned().unwrap_or_else(|| default.to_owned());
        let cfg = Config {
            secure: public_url.starts_with("https://"),
            redirect_uri: format!("{public_url}/google/oauth2-callback"),
            authorize: var("GOOGLE_AUTH_URL", "https://accounts.google.com/o/oauth2/v2/auth"),
            token: var("GOOGLE_TOKEN_URL", "https://oauth2.googleapis.com/token"),
            certs: var("GOOGLE_CERTS_URL", "https://www.googleapis.com/oauth2/v3/certs"),
            issuers: var("GOOGLE_ISSUER", "https://accounts.google.com,accounts.google.com").split(',').map(str::to_owned).collect(),
            nonce_ms: env.get("APP_NONCE_MS").map_or(LOGIN_MS, |value| number::string_to_number(value)),
            public_url,
            client_id,
            client_secret,
            owner,
            allowed_domain,
        };
        install_auth_schema(&*options.db)?;
        let now = runtime::now_ms();
        options.db.execute("DELETE FROM sessions WHERE expires_ms < ?", params![now])?;
        options.db.execute("DELETE FROM app_tokens WHERE expires_ms < ?", params![now])?;
        Ok(Some(Auth(Rc::new(Inner {
            cfg,
            db: options.db,
            fetch: options.fetch,
            random: options.random,
            keys: RefCell::new(Keys::default()),
            keys_attempted: Cell::new(0.0),
            loading: RefCell::new(None),
        }))))
    }

    /// Answer sign-in (Google and password), sign-out and gate every other request without a valid session (`handle`).
    ///
    /// # Arguments
    /// * `req` - The request; its body is consumed by the sign-in endpoints.
    ///
    /// # Returns
    /// The response, or `None` when the request may continue to the rest of the server.
    ///
    /// # Errors
    /// The database's, the recorded pages' or the body's error, or invalid data where the JS threw a `TypeError`.
    pub async fn handle(&self, req: &mut Request) -> Result<Option<Response>> {
        let me = &self.0;
        let url = req.url.clone();
        let pathname = url.path().to_owned();
        let signed_in = me.session(req)?;
        let document = req.method == "GET" && (req.header("sec-fetch-dest") == Some("document") || req.header("accept").is_some_and(|accept| accept.contains("text/html")));
        let tunnelled = req.header("cf-connecting-ip").is_some_and(|ip| !ip.is_empty());
        // Browsers never attach a bearer token on their own, so a write signed in by one alone cannot be a cross-site forgery.
        let bearer_only = signed_in.is_some() && cookie_value(req, SESSION_COOKIE).is_none();
        let write = req.method != "GET";
        if (write || is_websocket(req)) && !bearer_only && me.foreign_origin(req)? {
            let refusal = if write { "cross-origin write refused" } else { "cross-origin connection refused" };
            return Ok(Some(Response::new(403, Headers::new(), refusal)));
        }
        if let Some(step) = is_app_signin(&pathname) {
            return me.app_sign_in(req, step).await.map(Some);
        }
        let get = req.method == "GET";
        let secure = me.cfg.secure;
        if pathname == "/robots.txt" && get {
            return Ok(Some(Response::new(200, headers(&[("content-type", "text/plain; charset=utf-8")]), "User-agent: *\nDisallow: /\n")));
        }
        if pathname == "/signin" && get {
            if signed_in.is_some() {
                return Ok(Some(redirect("/everything")));
            }
            let error = search_param(&url, "error");
            return Ok(Some(html(plain_signin_page(error.as_deref(), me.passwords()?), &[])));
        }
        if pathname == "/signin/password" {
            if !me.passwords()? {
                return Ok(Some(bare(404)));
            }
            if get {
                return Ok(Some(if signed_in.is_some() { redirect("/everything") } else { html(password_page(search_param(&url, "error").as_deref()), &[]) }));
            }
            if req.method != "POST" {
                return Ok(Some(Response::new(405, headers(&[("allow", "GET, POST")]), Body::Empty)));
            }
            let form = read_form(req).await?;
            let password = form.as_ref().map(|form| form_get(form, "password").unwrap_or_default());
            let username = form.as_ref().and_then(|form| form_get(form, "username")).map_or(Value::Null, |name| Value::String(name.to_owned()));
            return Ok(Some(match me.verify_password(req, &username, password)? {
                PasswordResult::Invalid => redirect_with(303, "/signin/password?error=invalid", &[]),
                PasswordResult::Locked => redirect_with(303, "/signin/password?error=locked", &[]),
                PasswordResult::Name(name) => {
                    let id = me.start_session(&Value::String(password_subject(&name)), &name)?;
                    redirect_with(303, "/everything", &[cookie(SESSION_COOKIE, &id, SESSION_MS, secure, "/")])
                }
            }));
        }
        if pathname == "/_auth/signin.js" {
            return Ok(Some(Response::new(200, headers(&[("content-type", "text/javascript; charset=utf-8")]), SIGNIN_SCRIPT)));
        }
        if pathname == "/_auth/appleid.js" {
            return Ok(Some(Response::new(200, headers(&[("content-type", "text/javascript; charset=utf-8")]), "window.AppleID = { auth: { init() {} } };\n")));
        }
        if pathname == "/google/oauth2-redirect" && get {
            let state = me.token_id(24);
            let verifier = me.token_id(32);
            let nonce = me.token_id(24);
            me.db.execute("DELETE FROM logins WHERE created_ms < ?", params![me.now() - LOGIN_MS])?;
            me.db.execute("INSERT INTO logins (state, verifier, nonce, created_ms) VALUES (?, ?, ?, ?)", params![&state, &verifier, &nonce, me.now()])?;
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("client_id", &me.cfg.client_id)
                .append_pair("redirect_uri", &me.cfg.redirect_uri)
                .append_pair("response_type", "code")
                .append_pair("scope", "openid email")
                .append_pair("state", &state)
                .append_pair("nonce", &nonce)
                .append_pair("code_challenge", &base64::encode_url(&Sha256::digest(verifier.as_bytes())))
                .append_pair("code_challenge_method", "S256")
                .append_pair("prompt", "select_account")
                .finish();
            return Ok(Some(redirect_with(302, &format!("{}?{query}", me.cfg.authorize), &[cookie(STATE_COOKIE, &state, LOGIN_MS, secure, "/google")])));
        }
        if pathname == "/google/oauth2-callback" && get {
            let clear = cookie(STATE_COOKIE, "", 0.0, secure, "/google");
            return Ok(Some(match me.callback(&url, req).await {
                Ok(Callback::Problem(problem)) => html(popup_page(&format!("/signin?error={problem}&identityProvider=Google")), &[clear]),
                Ok(Callback::Id(id)) => html(popup_page("/everything"), &[clear, cookie(SESSION_COOKIE, &id, SESSION_MS, secure, "/")]),
                Err(_) => html(popup_page("/signin?error=SignInFailed&identityProvider=Google"), &[clear]),
            }));
        }
        if pathname == "/signout" && get {
            if let Some(id) = cookie_value(req, SESSION_COOKIE).filter(|id| !id.is_empty()) {
                me.db.execute("DELETE FROM sessions WHERE hash = ?", params![sha256_hex(&id)])?;
            }
            return Ok(Some(redirect_with(302, "/signin", &[cookie(SESSION_COOKIE, "", 0.0, secure, "/")])));
        }
        if pathname == "/app/v1/auth/signout"
            && req.method == "POST"
            && let Some(token) = bearer(req)
            && me.app_session(req)?.is_some()
        {
            me.db.execute("DELETE FROM app_tokens WHERE hash = ?", params![sha256_hex(&token)])?;
            return Ok(Some(Response::new(204, headers(&[("cache-control", "no-store")]), Body::Empty)));
        }
        if pathname == "/app/v1/me"
            && get
            && let Some(row) = &signed_in
        {
            let kind = if row.sub.starts_with("password:") { "password" } else { "google" };
            return Ok(Some(json_reply(200, obj! { "email" => row.email.as_str(), "kind" => kind })));
        }
        if let Some(row) = &signed_in
            && let Some(response) = me.agents(req, &pathname, row)?
        {
            return Ok(Some(response));
        }
        if signed_in.is_some() && pathname != "/app/v1/auth/signout" {
            return Ok(None);
        }
        if is_public(&pathname, tunnelled) {
            return Ok(None);
        }
        if document && bearer(req).is_none() {
            return Ok(Some(redirect("/signin")));
        }
        Ok(Some(Response::new(401, headers(&[("content-type", "application/json"), ("cache-control", "no-store")]), NOT_FOUND)))
    }

    /// Whether a request (including a WebSocket upgrade) carries a valid session (`allows`).
    ///
    /// # Arguments
    /// * `req` - The request.
    ///
    /// # Errors
    /// The database's error.
    pub fn allows(&self, req: &Request) -> Result<bool> {
        Ok(self.0.session(req)?.is_some())
    }

    /// Whether a WebSocket upgrade may connect: a valid session, and for a cookie session an `Origin` that is this
    /// server's own or absent.
    ///
    /// # Arguments
    /// * `req` - The upgrade request.
    ///
    /// # Errors
    /// The database's error, or invalid data when the server's public URL does not parse.
    pub fn allows_upgrade(&self, req: &Request) -> Result<bool> {
        if self.0.session(req)?.is_none() {
            return Ok(false);
        }
        Ok(cookie_value(req, SESSION_COOKIE).is_none() || !self.0.foreign_origin(req)?)
    }

    /// Who a request is signed in as (`identity`).
    ///
    /// # Arguments
    /// * `req` - The request.
    ///
    /// # Returns
    /// The session's or app token's identity with a credential for [`Auth::active`]; `None` when signed out.
    ///
    /// # Errors
    /// The database's error.
    pub fn identity(&self, req: &Request) -> Result<Option<SessionIdentity>> {
        Ok(self.0.session(req)?.map(|row| SessionIdentity { sub: row.sub, email: row.email, credential: row.hash }))
    }

    /// Who a session or app token named by an earlier [`Auth::identity`] is still signed in as (`active`).
    ///
    /// # Arguments
    /// * `credential` - The `credential` of an earlier identity.
    ///
    /// # Returns
    /// Its identity, or `None` once it was signed out or has expired.
    ///
    /// # Errors
    /// The database's error.
    pub fn active(&self, credential: &str) -> Result<Option<Identity>> {
        let row = self.0.db.one(
            "SELECT sub, email, expires_ms FROM sessions WHERE hash = ? UNION ALL SELECT sub, email, expires_ms FROM app_tokens WHERE hash = ?",
            params![credential, credential],
        )?;
        Ok(row.filter(|row| row.number("expires_ms").unwrap_or(f64::NAN) >= self.0.now()).map(|row| Identity {
            sub: row.text("sub").unwrap_or_default().to_owned(),
            email: row.text("email").unwrap_or_default().to_owned(),
        }))
    }
}
