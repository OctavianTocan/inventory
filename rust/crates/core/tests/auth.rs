//! `createAuth` (`lib/auth.mjs`) against the Rust port: the same request sequences on a fresh auth database each,
//! comparing every response, the fetches made and the database rows after each step.

mod common;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use mm_core::auth::{Auth, AuthOptions, SigninPages};
use mm_core::auth_store::{hash_password, install_auth_schema};
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::sql::Sql;
use mm_core::{Error, Result, Value, obj};
use mm_js::{base64, date, json};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};
use rsa::{RsaPrivateKey, Pkcs1v15Sign, pkcs8::DecodePrivateKey, traits::PublicKeyParts};

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/auth.mjs");
// The fixed synthetic test key already used by the pre-switch oracle, never an account credential.
const SYNTHETIC_KEY_PKCS8: &str = "MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCM5G0cfsoMd1b8Fz+T4nzsg/B+Xk1yMv050rleRsVujX9TJc03CIUASWE69X2XVGan9JwZCfqdNggVhiASTTDQ9VPzJpm5Re+jecWaoBg4fD70EvOVFXvcOBs/VmUd2qV7iDWdx+C3FlIAYiXXPK+NVzfGGQtvd2QwW3ZY1yvT/F98O5yu/NPVnWU3BwefSkk4SQOxgwgv6BED6zqCCcg+ngU3D0+DjWyMc4k1A+Jy7qrtbuplJyG+Z/JXTiG0Ukk1ZgVSkDE7R7t/hN1riqeFSWI2IisvUldyWAPJaXflftYr0bxPohXaFHhzw09QYeJfKzY/YMb+0c9mSC5BoTetAgMBAAECggEAHhOzgUZFzsR5MsapwWGzxm/iGq/5+3uMGFh2TvxvsEqPmEn1FZ8BeY6wrXvzrCcN5SPwYN4C/KWosyOXXCDTX4BHxBwE7ArUfgMEXp53nTNU4xjanARilR/7HRnN60IeWpE41O5y99LRfZh2OI0RcuxgES+Og+SqZDJaQNvqVM310eTC5eUFBjKg0gGvp4zcjAYvlPhmjFqw78qGjt+TZ95DLbWMYBaunTwHf890G5VZYL4BLfFYZFj5lr0Y67fimonFwBfxP7duwUo8pZYmPHkgoD2WdxqFx+Oz1Pp21WDqihqqOeoIieiEnp9bFNxbvW3DkaC2oS8tT5/DFldygQKBgQDGFL2/CnX83Xk6drWx2bn4dCmySxRdhsuS5llptBOTIUB40xcp8ytX0eQUGhFuvdVlQ5woEd5lQBOvIEmo8ivLMK6J29ECFMAxpz1veE3N+itHpGXz5wWW8+1KR3zHQq6Lpi6+79LIZT518JBlOy8ocGO5bnzWoKzuGp2OME1mwQKBgQC2FtmuM7FbXt+TfL4a92Ol4ceDL71PyZOzG3KcZonLi29L9ps8hpCsZ0rHwG+tPsZPIYgkB3ciXWLD1nRvX/T9a6oOKZR/Xdm780AoOnUvDrhh9jvZzYwJwtoqVJqUn9qahWUi0ecsX9srXEnR5dpTgPHUWNl3uuRE+nCUaO3X7QKBgGczbTy4lYp+VWf5a2bTGkpxMzw6U8z6qtvJefJRqRUx8OZCjMIFeKKTr3IttsFs1F6jIBvcW3ODM/49I0OLkPeXUIhw529fIld8W8zU8CyN17jG1XuC9O2V/x6BKmiivE/z8Ez4pGhWH3nRMVocwNAf5kU5ZH9qTU7svQPU9K7BAoGAYZj3zgZI/rdoUI4g+yU0VPoiTO9uofBiaTLCCJBxNRckbWvdouBhdPbcPxqnAQ3PjyRi6yTRNyskv+EXgYbGMnrdqD+jJ21vQe1MqwNCQhm6vE8Vn1UrxSOaeevk+tp/eikbn0EwjjlGqQQLUSTJE87eT7nbve//X/V4GCPxr7UCgYBf0sB2wg3QP91gSy1s3J7DXMy1a/pAsFv/zBCt6oOiRbvsof2QLyUV0dnodO6FlfBH/1tuLkyWa/UYUW2dBldOX+kzeoHcj+I5MYML5ndLHuU8d2do061w1H6GZGtyOjsjrUnbkt9idUxJuYSYn3Bwqlkk2RQrXLCaGHIlRv5iug==";
const NOW: f64 = 1_760_000_000_000.0;
const HOUR: f64 = 3_600_000.0;
const DAY: f64 = 24.0 * HOUR;
const BASE: &str = "http://localhost:3000";
const CERTS: &str = "http://127.0.0.1:9/certs";
const TOKEN: &str = "http://127.0.0.1:9/token";

static COUNTERS: std::sync::Mutex<Vec<(usize, u64)>> = std::sync::Mutex::new(Vec::new());

fn deterministic(size: usize) -> Vec<u8> {
    let mut counters = COUNTERS.lock().unwrap();
    let index = counters.iter().position(|entry| entry.0 == size).unwrap_or_else(|| {
        counters.push((size, 0));
        counters.len() - 1
    });
    counters[index].1 += 1;
    Sha256::digest(format!("r{size}:{}", counters[index].1).as_bytes())[..size].to_vec()
}

fn draw(counter: u64, size: usize) -> String {
    base64::encode_url(&Sha256::digest(format!("r{size}:{counter}").as_bytes())[..size])
}

struct Pages;

#[async_trait(?Send)]
impl SigninPages for Pages {
    async fn page(&self, name: &str) -> Result<String> {
        let path = common::root().join("capture/signin").join(format!("{name}.html"));
        let text = std::fs::read_to_string(path).map_err(|error| Error::internal(error.to_string()))?;
        Ok(text.replace("https://static.accelerator.net", "/_static"))
    }
}

#[derive(Default)]
struct Mocks {
    certs: Option<Value>,
    token: Option<Value>,
    log: Vec<Value>,
}

struct MockFetch(Rc<RefCell<Mocks>>);

#[async_trait(?Send)]
impl Fetch for MockFetch {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = request.take_body().bytes().await?;
        let mut headers = mm_core::Object::new();
        for (name, value) in request.headers.iter() {
            headers.insert(name, value);
        }
        let mut mocks = self.0.borrow_mut();
        mocks.log.push(
            obj! {
                "url" => request.url.as_str(),
                "method" => request.method.as_str(),
                "headers" => headers,
                "body" => if body.is_empty() { Value::Null } else { Value::String(String::from_utf8(body).unwrap()) },
            }
            .into(),
        );
        let mock = if request.method == "POST" { mocks.token.clone() } else { mocks.certs.clone() };
        let Some(mock) = mock else { return Err(Error::invalid("fetch failed")) };
        let mut headers = Headers::new();
        if let Value::Object(pairs) = mock.get("headers") {
            for (name, value) in pairs.iter() {
                headers.append(name.into_owned(), value.to_js_string());
            }
        }
        Ok(Response::new(mock.get("status").to_number() as u16, headers, mock.get("body").to_js_string()))
    }
}

fn s(text: &str) -> Value {
    Value::String(text.to_owned())
}

fn n(number: f64) -> Value {
    Value::Number(number)
}

fn step(op: &str, fields: Vec<(&str, Value)>) -> Value {
    let mut object = mm_core::Object::new();
    object.insert("op", op);
    for (name, value) in fields {
        object.insert(name, value);
    }
    Value::Object(object)
}

fn request(op: &str, method: &str, path: &str, headers: &[(&str, &str)], body: Option<&str>, remote: &str) -> Value {
    let mut map = mm_core::Object::new();
    for (name, value) in headers {
        map.insert(*name, *value);
    }
    step(
        op,
        vec![
            ("method", s(method)),
            ("url", s(&format!("{BASE}{path}"))),
            ("headers", Value::Object(map)),
            ("body", body.map_or(Value::Null, s)),
            ("remote", s(remote)),
        ],
    )
}

fn req(method: &str, path: &str, headers: &[(&str, &str)]) -> Value {
    request("req", method, path, headers, None, "127.0.0.1")
}

fn get(path: &str) -> Value {
    req("GET", path, &[])
}

fn post(path: &str, headers: &[(&str, &str)], body: &str) -> Value {
    request("req", "POST", path, headers, Some(body), "127.0.0.1")
}

fn clock(ms: f64) -> Value {
    step("clock", vec![("ms", n(ms))])
}

fn mock(certs: Option<Value>, token: Option<Value>) -> Value {
    step("mock", vec![("certs", certs.unwrap_or(Value::Null)), ("token", token.unwrap_or(Value::Null))])
}

fn keys_reply(kids: &[&str], max_age: Option<u32>) -> Value {
    let mut headers = mm_core::Object::new();
    if let Some(age) = max_age {
        headers.insert("cache-control", format!("public, max-age={age}, must-revalidate"));
    }
    let key = synthetic_key();
    let keys: Vec<Value> = kids.iter().map(|kid| {
        let actual: Value = obj! { "kty" => "RSA", "n" => base64::encode_url(&key.n().to_bytes_be()), "e" => base64::encode_url(&key.e().to_bytes_be()), "kid" => *kid, "use" => "sig", "alg" => "RS256" }.into();
        let expected = common::call(DRIVER, "jwk", vec![s(kid)]);
        common::assert_value(&actual, &expected, "synthetic public JWK");
        actual
    }).collect();
    obj! { "status" => 200, "headers" => headers, "body" => json::to_string(&obj! { "keys" => keys }.into()) }.into()
}

fn token_reply(status: u16, id_token: Option<&str>) -> Value {
    let body = id_token.map_or_else(|| "{\"error\":\"nope\"}".to_owned(), |token| json::to_string(&obj! { "id_token" => token }.into()));
    obj! { "status" => status as f64, "headers" => obj! { "content-type" => "application/json" }, "body" => body }.into()
}

// Decode the fixed signing fixture without reading any environment or credential files.
fn synthetic_key() -> RsaPrivateKey {
    RsaPrivateKey::from_pkcs8_der(&base64::decode(SYNTHETIC_KEY_PKCS8)).expect("synthetic RSA fixture")
}

// Sign a synthetic identity token and freeze its exact bytes without storing the token.
fn sign(claims: Value, header: Option<Value>) -> String {
    let mut args = vec![claims.clone()];
    let mut head = obj! { "alg" => "RS256", "kid" => "test-key-1", "typ" => "JWT" };
    if let Some(header) = header {
        args.push(header.clone());
        if let Value::Object(header) = header {
            for (key, value) in header.iter() { head.insert(key.into_owned(), value.clone()); }
        }
    }
    let message = format!("{}.{}", base64::encode_url(json::to_string(&head.into()).as_bytes()), base64::encode_url(json::to_string(&claims).as_bytes()));
    let signature = synthetic_key().sign(Pkcs1v15Sign::new::<Sha256>(), &Sha256::digest(message.as_bytes())).expect("synthetic signature");
    let token = format!("{message}.{}", base64::encode_url(&signature));
    let expected = common::call(DRIVER, "sign", args);
    common::assert_value(&s(&token), &expected, "synthetic RS256 factory");
    token
}

fn claims(nonce: &str, edits: &[(&str, Value)]) -> Value {
    let mut object = mm_core::Object::new();
    object.insert("iss", "https://accounts.google.com");
    object.insert("aud", "client-1");
    object.insert("sub", "google-sub-1");
    object.insert("email", "owner@example.com");
    object.insert("email_verified", true);
    object.insert("nonce", nonce);
    object.insert("iat", (NOW / 1000.0).floor());
    object.insert("exp", (NOW / 1000.0).floor() + 3600.0);
    for (name, value) in edits {
        if matches!(value, Value::Undefined) {
            object.remove(name);
        } else {
            object.insert(*name, value.clone());
        }
    }
    Value::Object(object)
}

fn env(extra: &[(&str, &str)]) -> Value {
    let mut object = mm_core::Object::new();
    for (name, value) in [
        ("AUTH", "google"),
        ("GOOGLE_CLIENT_ID", "client-1"),
        ("GOOGLE_CLIENT_SECRET", "secret-1"),
        ("OWNER_EMAIL", " Owner@Example.com "),
        ("GOOGLE_CERTS_URL", CERTS),
        ("GOOGLE_TOKEN_URL", TOKEN),
    ] {
        object.insert(name, value);
    }
    for (name, value) in extra {
        if value.is_empty() && *name != "APP_NONCE_MS" {
            object.remove(name);
        } else {
            object.insert(*name, *value);
        }
    }
    Value::Object(object)
}

fn users() -> Value {
    let mut list = Vec::new();
    for (username, password, salt) in [("alice", "correct horse battery", "00112233445566778899aabbccddeeff"), ("bob.smith@x-y_z", "another long password!", "ffeeddccbbaa99887766554433221100")] {
        let salt_bytes: Vec<u8> = (0..salt.len() / 2).map(|i| u8::from_str_radix(&salt[2 * i..2 * i + 2], 16).unwrap()).collect();
        list.push(Value::from(obj! {
            "username" => username,
            "salt" => salt,
            "hash" => base64::hex(&hash_password(password, &salt_bytes)),
            "created_ms" => NOW - DAY,
        }));
    }
    Value::Array(list)
}

fn scenario(env: Value, users: Value, seed: Vec<(&str, Vec<Value>)>, steps: Vec<Value>) -> Value {
    let seed: Vec<Value> = seed.into_iter().map(|(sql, params)| obj! { "sql" => sql, "params" => params }.into()).collect();
    obj! { "env" => env, "port" => 3000, "users" => users, "seed" => seed, "clock" => NOW, "steps" => steps }.into()
}

fn class(error: &Error) -> String {
    match error {
        Error::Invalid(_) => "invalid".into(),
        Error::Internal(_) => "internal".into(),
        Error::Status { status, .. } => format!("status {status}"),
    }
}

fn snapshot(db: &dyn Sql) -> Value {
    let mut out = mm_core::Object::new();
    for table in ["sessions", "app_tokens", "users", "logins", "app_nonces", "failures", "libraries"] {
        let rows = db.query(&format!("SELECT * FROM {table} ORDER BY rowid"), &[]).unwrap();
        let rows: Vec<Value> = rows
            .iter()
            .map(|row| {
                let mut object = mm_core::Object::new();
                for (name, value) in row.to_object().iter() {
                    object.insert(name.into_owned(), if let Value::Bytes(bytes) = value { Value::String(base64::hex(bytes)) } else { value.clone() });
                }
                Value::Object(object)
            })
            .collect();
        out.insert(table, Value::Array(rows));
    }
    Value::Object(out)
}

fn rust_request(spec: &Value) -> Request {
    let mut request = Request::new(spec.get("method").as_str().unwrap(), spec.get("url").as_str().unwrap()).unwrap();
    let host = match request.url.port() {
        Some(port) => format!("{}:{port}", request.url.host_str().unwrap()),
        None => request.url.host_str().unwrap().to_owned(),
    };
    request.headers.set("host", host);
    if let Value::Object(headers) = spec.get("headers") {
        for (name, value) in headers.iter() {
            request.headers.set(name.into_owned(), value.to_js_string());
        }
    }
    request.remote_address = spec.get("remote").as_str().unwrap_or("127.0.0.1").to_owned();
    if let Some(body) = spec.get("body").as_str() {
        request.body = Body::Bytes(body.as_bytes().to_vec());
    }
    request
}

async fn run_rust(scenario: &Value) -> Vec<Value> {
    COUNTERS.lock().unwrap().clear();
    date::set_fixed_now(Some(scenario.get("clock").to_number() as i64));
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("auth-rust-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&directory).unwrap();
    let db: Rc<dyn Sql> = Rc::new(Sqlite::open(directory.join("auth.db"), false).unwrap());
    install_auth_schema(&*db).unwrap();
    let hex = |text: &str| -> Vec<u8> { (0..text.len() / 2).map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap()).collect() };
    for user in scenario.get("users").as_array().into_iter().flatten() {
        db.execute(
            "INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?)",
            &[
                mm_core::sql::SqlValue::Text(user.get("username").to_js_string()),
                mm_core::sql::SqlValue::Blob(hex(&user.get("salt").to_js_string())),
                mm_core::sql::SqlValue::Blob(hex(&user.get("hash").to_js_string())),
                mm_core::sql::SqlValue::Real(user.get("created_ms").to_number()),
            ],
        )
        .unwrap();
    }
    for seed in scenario.get("seed").as_array().into_iter().flatten() {
        db.execute(&seed.get("sql").to_js_string(), &sql_params(seed.get("params"))).unwrap();
    }
    let mocks = Rc::new(RefCell::new(Mocks::default()));
    let env: HashMap<String, String> = match scenario.get("env") {
        Value::Object(object) => object.iter().map(|(name, value)| (name.into_owned(), value.to_js_string())).collect(),
        _ => HashMap::new(),
    };
    let mut options = AuthOptions::new(env, scenario.get("port").to_number() as u16, db.clone(), Rc::new(MockFetch(mocks.clone())), Rc::new(Pages));
    options.random = deterministic;
    let mut answers = Vec::new();
    let auth = match Auth::new(options) {
        Ok(auth) => {
            answers.push(obj! { "op" => "create", "created" => auth.is_some(), "db" => snapshot(&*db) }.into());
            auth
        }
        Err(error) => {
            answers.push(obj! { "op" => "create", "error" => class(&error), "db" => snapshot(&*db) }.into());
            None
        }
    };
    for spec in scenario.get("steps").as_array().unwrap() {
        let op = spec.get("op").to_js_string();
        let mut answer = mm_core::Object::new();
        answer.insert("op", op.as_str());
        match op.as_str() {
            "clock" => date::set_fixed_now(Some(spec.get("ms").to_number() as i64)),
            "mock" => {
                let mut mocks = mocks.borrow_mut();
                mocks.certs = Some(spec.get("certs").clone()).filter(|value| !matches!(value, Value::Null));
                mocks.token = Some(spec.get("token").clone()).filter(|value| !matches!(value, Value::Null));
            }
            "sql" => {
                db.execute(&spec.get("sql").to_js_string(), &sql_params(spec.get("params"))).unwrap();
            }
            _ => {
                mocks.borrow_mut().log.clear();
                let auth = auth.as_ref().unwrap();
                let outcome: Result<()> = async {
                    match op.as_str() {
                        "req" => {
                            let mut request = rust_request(spec);
                            match auth.handle(&mut request).await? {
                                Some(response) => {
                                    answer.insert("handled", true);
                                    answer.insert("status", response.status as f64);
                                    answer.insert("headers", Value::Array(response.headers.iter().map(|(name, value)| Value::Array(vec![s(name), s(value)])).collect()));
                                    answer.insert("body", String::from_utf8(response.body.bytes().await?).unwrap());
                                }
                                None => {
                                    answer.insert("handled", false);
                                    answer.insert("status", Value::Null);
                                    answer.insert("headers", Value::Array(Vec::new()));
                                    answer.insert("body", "");
                                }
                            }
                        }
                        "allows" => answer.insert("value", auth.allows(&rust_request(spec))?),
                        "identity" => {
                            let identity = auth.identity(&rust_request(spec))?;
                            answer.insert(
                                "value",
                                identity.map_or(Value::Null, |who| obj! { "sub" => who.sub, "email" => who.email, "credential" => who.credential }.into()),
                            );
                        }
                        "active" => {
                            let identity = auth.active(&spec.get("credential").to_js_string())?;
                            answer.insert("value", identity.map_or(Value::Null, |who| obj! { "sub" => who.sub, "email" => who.email }.into()));
                        }
                        other => panic!("unknown step {other}"),
                    }
                    Ok(())
                }
                .await;
                if let Err(error) = outcome {
                    answer.insert("error", class(&error));
                }
                answer.insert("fetched", Value::Array(mocks.borrow().log.clone()));
            }
        }
        answer.insert("db", snapshot(&*db));
        answers.push(Value::Object(answer));
    }
    date::set_fixed_now(None);
    answers
}

static NEXT: AtomicU64 = AtomicU64::new(0);

fn sql_params(params: &Value) -> Vec<mm_core::sql::SqlValue> {
    params
        .as_array()
        .into_iter()
        .flatten()
        .map(|value| match value {
            Value::String(text) => mm_core::sql::SqlValue::Text(text.clone()),
            Value::Number(number) => mm_core::sql::SqlValue::Real(*number),
            _ => mm_core::sql::SqlValue::Null,
        })
        .collect()
}

fn compare(name: &str, scenario: &Value) -> Vec<Value> {
    let rust = futures_executor::block_on(run_rust(scenario));
    let js = common::call(DRIVER, "run", vec![scenario.clone()]);
    assert_eq!(rust.len(), scenario.get("steps").as_array().unwrap().len() + 1, "{name}: step count");
    common::assert_value(&Value::Array(rust.clone()), &js, name);
    rust
}

fn field<'a>(answers: &'a [Value], step: usize, name: &str) -> &'a Value {
    answers[step].get(name)
}

fn expect_status(answers: &[Value], step: usize, status: u32) {
    assert_eq!(field(answers, step, "status").to_number(), f64::from(status), "step {step}: {}", common::text(&answers[step]));
}

fn header(answers: &[Value], step: usize, name: &str) -> String {
    let Value::Array(headers) = field(answers, step, "headers") else { panic!("no headers at step {step}") };
    headers.iter().find(|pair| pair.at(0).as_str() == Some(name)).map(|pair| pair.at(1).to_js_string()).unwrap_or_default()
}

fn header_of(answer: &Value, name: &str) -> String {
    let Value::Array(headers) = answer.get("headers") else { return String::new() };
    headers.iter().find(|pair| pair.at(0).as_str() == Some(name)).map(|pair| pair.at(1).to_js_string()).unwrap_or_default()
}

fn form(username: &str, password: &str) -> String {
    format!("username={}&password={}", mm_js::uri::encode_uri_component(username), mm_js::uri::encode_uri_component(password))
}

const FORM: (&str, &str) = ("content-type", "application/x-www-form-urlencoded");

fn pages_scenario() -> Value {
    let tunnel = [("cf-connecting-ip", "203.0.113.9")];
    let html = [("accept", "text/html,application/xhtml+xml")];
    scenario(
        env(&[]),
        users(),
        vec![],
        vec![
            get("/robots.txt"),
            req("POST", "/robots.txt", &[]),
            get("/signin"),
            get("/signin?error=AccountNotFound"),
            get("/signin?error=Cancelled"),
            req("GET", "/signin", &tunnel),
            req("GET", "/signin?error=AccountNotFound", &tunnel),
            req("GET", "/signin?error=Cancelled", &tunnel),
            req("GET", "/signin?error=SignInFailed", &tunnel),
            req("GET", "/signin?error=constructor", &tunnel),
            req("GET", "/signin?error=%3Cb%3E", &tunnel),
            get("/_auth/signin.js"),
            get("/_auth/appleid.js"),
            req("POST", "/_auth/signin.js", &[]),
            get("/signin/password"),
            get("/signin/password?error=invalid"),
            get("/signin/password?error=locked"),
            get("/signin/password?error=other"),
            req("PUT", "/signin/password", &[]),
            req("DELETE", "/signin/password", &[]),
            req("HEAD", "/signin", &[]),
            req("POST", "/signin", &[]),
            req("POST", "/signin", &[("origin", "http://evil.example")]),
            req("POST", "/signin", &[("origin", "http://localhost:3000")]),
            req("POST", "/signin", &[("origin", "http://localhost:3000/")]),
            req("PUT", "/anything", &[("origin", "null")]),
            req("POST", "/anything", &[("origin", "http://other"), ("host", "other")]),
            req("POST", "/anything", &[("origin", "http://other"), ("host", "other")]),
            req("POST", "/app/v1/auth/nonce", &[("origin", "http://evil.example")]),
            req("GET", "/anything", &[("origin", "http://evil.example")]),
            req("GET", "/everything", &html),
            req("GET", "/everything", &[("sec-fetch-dest", "document")]),
            req("GET", "/everything", &[("sec-fetch-dest", "document"), ("authorization", "Bearer abc")]),
            get("/everything"),
            req("GET", "/everything", &[("accept", "application/json")]),
            req("POST", "/everything", &html),
            req("HEAD", "/everything", &html),
            get("/_static/app.css"),
            get("/_static/"),
            get("/_staticx/app.css"),
            req("GET", "/_static/app.css", &tunnel),
            get("/_auth/other"),
            req("GET", "/_auth/other", &tunnel),
            get("/manifest.webmanifest"),
            get("/favicon.ico"),
            get("/jserrors"),
            req("POST", "/jserrors", &[]),
            req("GET", "/jserrors", &tunnel),
            req("GET", "/signin", &[("cf-connecting-ip", "")]),
            req("GET", "/google/oauth2-callback?state=x", &tunnel),
            req("POST", "/google/oauth2-callback", &[]),
            req("POST", "/google/oauth2-redirect", &[]),
            req("POST", "/signout", &[]),
            req("POST", "/app/v1/auth/signout", &[]),
            req("POST", "/app/v1/auth/signout", &[("authorization", "Bearer nothing")]),
            req("POST", "/app/v1/auth/signout", &[("authorization", "Bearer nothing"), ("accept", "text/html")]),
            get("/app/v1/me"),
            get("/signout"),
            get("/signout/"),
            get("/signin/"),
            get("//signin"),
            get("/%73ignin"),
            get("/app/v1/auth/nonce"),
            get("/app/v1/auth/google"),
            req("PUT", "/app/v1/auth/password", &[]),
            req("GET", "/app/v1/auth/other", &[]),
            req("GET", "/google/oauth2-redirect/", &[]),
            req("GET", "/google/oauth2-callback", &[]),
            req("GET", "/google/oauth2-callback?state=zzz", &[("cookie", "mm_oauth=zzz")]),
        ],
    )
}

#[test]
fn auth_matches_js() {
    let _goldens = common::test_guard();
    let mut cases = 0;

    let pages = compare("pages", &pages_scenario());
    expect_status(&pages, 1, 200);
    expect_status(&pages, 2, 401);
    expect_status(&pages, 19, 405);
    expect_status(&pages, 23, 403);
    expect_status(&pages, 31, 302);
    cases += pages.len();

    let no_users = compare(
        "no users",
        &scenario(
            env(&[("PUBLIC_URL", "https://replica.example.com/")]),
            Value::Array(vec![]),
            vec![],
            vec![
                req("GET", "/signin", &[("cf-connecting-ip", "198.51.100.1")]),
                get("/signin/password"),
                req("POST", "/signin/password", &[]),
                get("/signout"),
                post("/signin", &[("origin", "https://replica.example.com")], ""),
                post("/signin", &[("origin", "http://localhost:3000")], ""),
                post("/signin", &[("origin", "https://replica.example.com/x")], ""),
            ],
        ),
    );
    expect_status(&no_users, 2, 404);
    assert!(header(&no_users, 4, "set-cookie").ends_with("; Secure"));
    cases += no_users.len();

    cases += password_scenarios();
    cases += session_scenarios();
    cases += app_scenarios();
    cases += web_google_scenarios();
    cases += config_scenarios();
    println!("auth: {cases} steps compared");
}

fn password_scenarios() -> usize {
    let alice = "correct horse battery";
    let mut steps = vec![
        post("/signin/password", &[FORM], &form("alice", alice)),
        get("/everything"),
        post("/signin/password", &[FORM], &form("alice", "wrong password")),
        post("/signin/password", &[FORM], &form("  ALICE ", alice)),
        post("/signin/password", &[FORM], &form("nobody", alice)),
        post("/signin/password", &[FORM], &form("bob.smith@x-y_z", "another long password!")),
        post("/signin/password", &[FORM], "?username=alice&password=correct+horse+battery"),
        post("/signin/password", &[FORM], "username=alice"),
        post("/signin/password", &[FORM], "password=correct+horse+battery"),
        post("/signin/password", &[FORM], ""),
        post("/signin/password", &[FORM], &format!("username=alice&password={}", "x".repeat(4100))),
        post("/signin/password", &[FORM], &format!("username=alice&password={}", "x".repeat(4070))),
        post("/signin/password", &[], "{\"username\":\"alice\"}"),
        post("/signin/password", &[FORM], "username=alice&password=correct%20horse%20battery&username=bob"),
        post("/signin/password", &[FORM], "username=%E0%A4%A&password=%zz"),
        post("/signin/password", &[("origin", "http://localhost:3000"), FORM], &form("alice", alice)),
        post("/signin/password", &[("origin", "http://evil.example"), FORM], &form("alice", alice)),
    ];
    for _ in 0..11 {
        steps.push(post("/signin/password", &[FORM], &form("alice", "guess guess guess")));
    }
    steps.push(post("/signin/password", &[FORM], &form("alice", alice)));
    steps.push(post("/signin/password", &[FORM], &form("bob.smith@x-y_z", "another long password!")));
    steps.push(clock(NOW + 15.0 * 60_000.0 + 1.0));
    steps.push(post("/signin/password", &[FORM], &form("alice", alice)));
    steps.push(clock(NOW + 16.0 * 60_000.0));
    for index in 0..31 {
        steps.push(request("req", "POST", "/signin/password", &[FORM], Some(&form(&format!("user{index}"), "guess guess guess")), "192.0.2.77"));
    }
    steps.push(request("req", "POST", "/signin/password", &[FORM], Some(&form("alice", alice)), "192.0.2.77"));
    steps.push(request("req", "POST", "/signin/password", &[FORM, ("cf-connecting-ip", "192.0.2.78")], Some(&form("alice", alice)), "192.0.2.77"));
    steps.push(request("req", "POST", "/signin/password", &[FORM, ("cf-connecting-ip", "192.0.2.78")], Some(&form("alice", alice)), "192.0.2.99"));
    let answers = compare("password", &scenario(env(&[]), users(), vec![], steps));
    expect_status(&answers, 1, 303);
    assert_eq!(header(&answers, 1, "location"), "/everything");
    assert!(header(&answers, 1, "set-cookie").contains("Max-Age=2592000"));
    assert_eq!(header(&answers, 3, "location"), "/signin/password?error=invalid");
    assert_eq!(header(&answers, 4, "location"), "/everything");
    assert!(answers.iter().any(|answer| header_of(answer, "location") == "/signin/password?error=locked"));
    answers.len()
}

fn session_scenarios() -> usize {
    let alice = "correct horse battery";
    let sign_in = post("/signin/password", &[FORM], &form("alice", alice));
    // The cookie value is predicted: the first draw (32 bytes) is the session id.
    let id = draw(1, 32);
    let cookie = format!("mm_session={id}");
    let with = |extra: &[(&str, &str)]| -> Vec<(String, String)> {
        let mut headers = vec![("cookie".to_owned(), cookie.clone())];
        headers.extend(extra.iter().map(|(a, b)| ((*a).to_owned(), (*b).to_owned())));
        headers
    };
    let get_with = |path: &str, extra: &[(&str, &str)]| {
        let headers = with(extra);
        let pairs: Vec<(&str, &str)> = headers.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        req("GET", path, &pairs)
    };
    let steps = vec![
        sign_in,
        get_with("/everything", &[]),
        get_with("/signin", &[]),
        get_with("/signin/password", &[]),
        get_with("/app/v1/me", &[]),
        step("identity", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "cookie" => cookie.as_str() }.into())]),
        step("allows", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "cookie" => cookie.as_str() }.into())]),
        step("allows", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "cookie" => "mm_session=nope" }.into())]),
        step("allows", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "cookie" => "mm_session=" }.into())]),
        clock(NOW + HOUR),
        get_with("/everything", &[]),
        clock(NOW + HOUR + 1.0),
        get_with("/everything", &[]),
        clock(NOW + 2.0 * HOUR + 5.0),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(id.as_bytes()))))]),
        step("active", vec![("credential", s("nothing"))]),
        step("active", vec![("credential", s(""))]),
        get_with("/signout", &[]),
        get_with("/everything", &[]),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(id.as_bytes()))))]),
        get("/signout"),
        post("/signin/password", &[FORM], &form("alice", alice)),
        clock(NOW + 2.0 * HOUR + 5.0 + 30.0 * DAY),
        get_with("/everything", &[]),
        req("GET", "/everything", &[("cookie", "a=b; mm_session=%E0%A4%A; x"), ("accept", "text/html")]),
        req("GET", "/everything", &[("cookie", &format!("mm_session=bad; mm_session={id}"))]),
        req("GET", "/everything", &[("cookie", &format!("  mm_session = {id}"))]),
        req("GET", "/everything", &[("cookie", &format!("=x; mm_session={id}=="))]),
        req("GET", "/everything", &[("cookie", &format!("mm_session=%{}", &id[..2]))]),
        req("GET", "/everything", &[("cookie", "mm_session=" )]),
        post("/signin/password", &[FORM], &form("alice", alice)),
        clock(NOW + 31.0 * DAY + 3.0 * HOUR),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(draw(5, 32).as_bytes()))))]),
        get("/signin"),
    ];
    let answers = compare("sessions", &scenario(env(&[]), users(), vec![], steps));
    expect_status(&answers, 1, 303);
    assert_eq!(field(&answers, 2, "handled").as_bool(), Some(false));
    assert_eq!(header(&answers, 3, "location"), "/everything");
    expect_status(&answers, 5, 200);
    answers.len()
}

fn app_scenarios() -> usize {
    let mut total = 0;
    let json_header = ("content-type", "application/json");
    let alice = "correct horse battery";
    let nonce_a = draw(1, 24);
    let sub = |nonce: &str, edits: &[(&str, Value)]| claims(nonce, edits);

    // Draw counter: nonce step 1 -> r1; each google sign-in nonce is the draw of its nonce step.
    let steps = vec![
        mock(Some(keys_reply(&["test-key-1"], Some(300))), None),
        post("/app/v1/auth/nonce", &[], ""),
        post("/app/v1/auth/google", &[json_header], &json::to_string(&obj! { "nonce" => nonce_a.as_str(), "idToken" => sign(sub(&nonce_a, &[]), None), "device" => "Pixel 9" }.into())),
        post("/app/v1/auth/google", &[json_header], &json::to_string(&obj! { "nonce" => nonce_a.as_str(), "idToken" => sign(sub(&nonce_a, &[]), None) }.into())),
        get("/app/v1/auth/nonce"),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => 12345 }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => " Alice ", "password" => "wrong" }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice" }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => 5 }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "password" => alice }.into())),
        post("/app/v1/auth/password", &[json_header], "[]"),
        post("/app/v1/auth/password", &[json_header], "null"),
        post("/app/v1/auth/password", &[json_header], "not json"),
        post("/app/v1/auth/password", &[json_header], ""),
        post("/app/v1/auth/password", &[json_header], &format!("{{\"username\":\"alice\",\"password\":\"{}\"}}", "y".repeat(16400))),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => "d".repeat(150) }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => Value::Null }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => obj! { "a" => 1 } }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => vec![Value::from(1.0), Value::from(2.0)] }.into())),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => true }.into())),
        req("PUT", "/app/v1/auth/password", &[]),
        req("GET", "/app/v1/auth/google", &[]),
    ];
    let answers = compare("app basics", &scenario(env(&[]), users(), vec![], steps));
    expect_status(&answers, 2, 200);
    expect_status(&answers, 3, 200);
    expect_status(&answers, 4, 400);
    expect_status(&answers, 6, 200);
    expect_status(&answers, 7, 401);
    expect_status(&answers, 16, 200);
    total += answers.len();

    // Bearer tokens: the password sign-in draws attempt (1) then the token (2).
    let token = draw(1, 32);
    let bearer = format!("Bearer {token}");
    let auth_header = [("authorization", bearer.as_str())];
    let steps = vec![
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice, "device" => "phone" }.into())),
        req("GET", "/app/v1/me", &auth_header),
        req("GET", "/everything", &auth_header),
        req("GET", "/everything", &[("authorization", &format!("bearer   {token}"))]),
        req("GET", "/everything", &[("authorization", &format!("BEARER {token}"))]),
        req("GET", "/everything", &[("authorization", &format!("Bearer {token} x"))]),
        req("GET", "/everything", &[("authorization", &format!("Bearer\t{token}"))]),
        req("GET", "/everything", &[("authorization", &format!("Bearer{token}"))]),
        req("GET", "/everything", &[("authorization", &format!(" Bearer {token}"))]),
        req("GET", "/everything", &[("authorization", &format!("Basic {token}"))]),
        req("GET", "/everything", &[("authorization", "Bearer ")]),
        req("GET", "/everything", &[("authorization", "Bearer  \u{a0}x")]),
        step("identity", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "authorization" => bearer.as_str() }.into())]),
        step("allows", vec![("method", s("GET")), ("url", s(&format!("{BASE}/x"))), ("headers", obj! { "authorization" => bearer.as_str() }.into())]),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(token.as_bytes()))))]),
        clock(NOW + HOUR + 1.0),
        req("GET", "/app/v1/me", &auth_header),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(token.as_bytes()))))]),
        clock(NOW + 91.0 * DAY),
        req("GET", "/app/v1/me", &auth_header),
        clock(NOW + 100.0 * DAY),
        req("GET", "/app/v1/me", &auth_header),
        step("active", vec![("credential", s(&base64::hex(&Sha256::digest(token.as_bytes()))))]),
        req("POST", "/app/v1/auth/signout", &auth_header),
        clock(NOW + 100.0 * DAY),
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice }.into())),
    ];
    let answers = compare("app bearer", &scenario(env(&[]), users(), vec![], steps));
    expect_status(&answers, 1, 200);
    expect_status(&answers, 2, 200);
    assert_eq!(field(&answers, 3, "handled").as_bool(), Some(false));
    total += answers.len();

    // Sign-out with a live token.
    let token = draw(1, 32);
    let bearer = format!("Bearer {token}");
    let steps = vec![
        post("/app/v1/auth/password", &[json_header], &json::to_string(&obj! { "username" => "alice", "password" => alice }.into())),
        req("POST", "/app/v1/auth/signout", &[("authorization", bearer.as_str())]),
        req("POST", "/app/v1/auth/signout", &[("authorization", bearer.as_str())]),
        req("GET", "/app/v1/me", &[("authorization", bearer.as_str())]),
        req("GET", "/app/v1/auth/signout", &[("authorization", bearer.as_str())]),
    ];
    let answers = compare("app signout", &scenario(env(&[]), users(), vec![], steps));
    expect_status(&answers, 2, 204);
    expect_status(&answers, 3, 401);
    total += answers.len();

    // Nonce lifetimes and the cap.
    let nonce_one = draw(1, 24);
    let google = |nonce: &str, token: String| post("/app/v1/auth/google", &[json_header], &json::to_string(&obj! { "nonce" => nonce, "idToken" => token }.into()));
    let steps = vec![
        mock(Some(keys_reply(&["test-key-1"], None)), None),
        post("/app/v1/auth/nonce", &[], ""),
        clock(NOW + 4001.0),
        google(&nonce_one, sign(sub(&nonce_one, &[]), None)),
        post("/app/v1/auth/nonce", &[], ""),
        clock(NOW + 4001.0 + 3999.0),
        google(&draw(2, 24), sign(sub(&draw(2, 24), &[]), None)),
        post("/app/v1/auth/nonce", &[], ""),
        google("", sign(sub("", &[]), None)),
        post("/app/v1/auth/google", &[json_header], "{}"),
        post("/app/v1/auth/google", &[json_header], "[]"),
        post("/app/v1/auth/google", &[json_header], "oops"),
        post("/app/v1/auth/google", &[json_header], &json::to_string(&obj! { "nonce" => draw(4, 24), "idToken" => 5 }.into())),
        step("sql", vec![("sql", s("WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 10005) INSERT INTO app_nonces SELECT 'seed' || i, ? + i FROM n")), ("params", Value::Array(vec![n(NOW + 4001.0 + 3999.0)]))]),
        post("/app/v1/auth/nonce", &[], ""),
    ];
    let answers = compare("app nonces", &scenario(env(&[("APP_NONCE_MS", "4000")]), users(), vec![], steps));
    expect_status(&answers, 4, 400);
    expect_status(&answers, 7, 200);
    total += answers.len();

    // Google sign-in variants; every case draws one nonce, so nonce k is draw k.
    type Case = (&'static str, Vec<(&'static str, Value)>, Option<Value>);
    let cases: Vec<Case> = vec![
        ("ok", vec![], None),
        ("other email", vec![("email", s("someone@example.com"))], None),
        ("email case", vec![("email", s("OWNER@example.com"))], None),
        ("unverified", vec![("email_verified", Value::Bool(false))], None),
        ("verified as string", vec![("email_verified", s("true"))], None),
        ("no verified", vec![("email_verified", Value::Undefined)], None),
        ("wrong aud", vec![("aud", s("other"))], None),
        ("wrong iss", vec![("iss", s("https://evil.example"))], None),
        ("bare iss", vec![("iss", s("accounts.google.com"))], None),
        ("expired", vec![("exp", n(NOW / 1000.0 - 61.0))], None),
        ("skew ok", vec![("exp", n(NOW / 1000.0 - 59.0))], None),
        ("no exp", vec![("exp", Value::Undefined)], None),
        ("exp string", vec![("exp", s("9999999999"))], None),
        ("future iat", vec![("iat", n(NOW / 1000.0 + 120.0))], None),
        ("iat skew ok", vec![("iat", n(NOW / 1000.0 + 59.0))], None),
        ("wrong nonce", vec![("nonce", s("other"))], None),
        ("nonce number", vec![("nonce", n(1.0))], None),
        ("sub number", vec![("sub", n(7.0))], None),
        ("sub missing", vec![("sub", Value::Undefined)], None),
        ("sub null", vec![("sub", Value::Null)], None),
        ("sub bool", vec![("sub", Value::Bool(true))], None),
        ("hs256", vec![], Some(obj! { "alg" => "HS256" }.into())),
        ("none", vec![], Some(obj! { "alg" => "none" }.into())),
        ("other kid", vec![], Some(obj! { "kid" => "unknown" }.into())),
        ("kid number", vec![], Some(obj! { "kid" => 1 }.into())),
        ("kid missing", vec![], Some(obj! { "kid" => Value::Undefined }.into())),
    ];
    let mut steps = vec![mock(Some(keys_reply(&["test-key-1"], Some(3600))), None)];
    for (index, (_, edits, header)) in cases.iter().enumerate() {
        let counter = index as u64 + 1;
        steps.push(post("/app/v1/auth/nonce", &[], ""));
        let nonce = draw(counter, 24);
        let token = sign(claims(&nonce, edits), header.clone());
        steps.push(google(&nonce, token));
    }
    let answers = compare("app google variants", &scenario(env(&[]), users(), vec![], steps));
    assert!(answers.iter().filter(|answer| answer.get("status").to_number() == 200.0).count() >= 6, "google sign-ins that must work");
    total += answers.len();

    // Bad tokens: signature, shape.
    let nonce = draw(1, 24);
    let good = sign(claims(&nonce, &[]), None);
    let parts: Vec<&str> = good.split('.').collect();
    let flip = |signature: &str| {
        let mut bytes = base64::decode(signature);
        bytes[0] ^= 1;
        base64::encode_url(&bytes)
    };
    let short = base64::encode_url(&base64::decode(parts[2])[1..]);
    let long = base64::encode_url(&[vec![0u8; 3], base64::decode(parts[2])].concat());
    let zero_led = parts[2].to_owned();
    let variants = vec![
        format!("{}.{}.{}", parts[0], parts[1], flip(parts[2])),
        format!("{}.{}", parts[0], parts[1]),
        format!("{}.{}.", parts[0], parts[1]),
        format!("{}.{}.{}", parts[0], base64::encode_url(b"{\"iss\":\"x\"}"), parts[2]),
        format!("{}.{}.{}.extra", parts[0], parts[1], parts[2]),
        format!("{}.{}.{}", parts[0], parts[1], short),
        format!("{}.{}.{}", parts[0], parts[1], long),
        parts[0].to_owned(),
        String::new(),
        "a.b.c".to_owned(),
        format!("!!.{}.{}", parts[1], parts[2]),
        format!("{}.{}.{}", base64::encode_url(b"[1]"), parts[1], parts[2]),
        format!("{}.{}.{}", base64::encode_url(b"null"), parts[1], parts[2]),
        format!("{}.{}.{}", parts[0], parts[1].replace(parts[1], &base64::encode_url(b"null")), zero_led),
        format!("{}.{}.{}", parts[0], base64::encode_url(b"[]"), parts[2]),
    ];
    let mut steps = vec![mock(Some(keys_reply(&["test-key-1"], Some(3600))), None)];
    for (index, token) in variants.iter().enumerate() {
        steps.push(post("/app/v1/auth/nonce", &[], ""));
        let nonce = draw(index as u64 + 1, 24);
        steps.push(google(&nonce, token.clone()));
    }
    let answers = compare("app bad tokens", &scenario(env(&[]), users(), vec![], steps));
    total += answers.len();
    total
}

struct Plan {
    n24: u64,
    n32: u64,
}

impl Plan {
    /// The (state, nonce) a redirect draws next.
    fn redirect(&mut self) -> (String, String) {
        let state = draw(self.n24 + 1, 24);
        let nonce = draw(self.n24 + 2, 24);
        self.n24 += 2;
        self.n32 += 1;
        (state, nonce)
    }

    /// The session id a successful sign-in draws next.
    fn session(&mut self) -> String {
        self.n32 += 1;
        draw(self.n32, 32)
    }
}

fn callback(state: &str, cookie_state: &str, query: &str) -> Value {
    req("GET", &format!("/google/oauth2-callback?state={state}{query}"), &[("cookie", &format!("mm_oauth={cookie_state}"))])
}

fn web_google_scenarios() -> usize {
    let mut total = 0;
    let mut plan = Plan { n24: 0, n32: 0 };
    let keys = keys_reply(&["test-key-1"], Some(3600));
    let ok_token = |nonce: &str, edits: &[(&str, Value)]| token_reply(200, Some(&sign(claims(nonce, edits), None)));
    let mut steps = vec![mock(Some(keys.clone()), None)];

    let (state, nonce) = plan.redirect();
    steps.push(get("/google/oauth2-redirect"));
    steps.push(mock(Some(keys.clone()), Some(ok_token(&nonce, &[]))));
    steps.push(callback(&state, &state, "&code=abc"));
    plan.session();
    steps.push(callback(&state, &state, "&code=abc"));

    let (state, nonce) = plan.redirect();
    steps.push(req("GET", "/google/oauth2-redirect?popup=true", &[("cf-connecting-ip", "203.0.113.5")]));
    steps.push(callback(&state, "other", "&code=abc"));
    steps.push(callback(&state, &state, "&code=abc"));
    let _ = nonce;

    let (state, nonce) = plan.redirect();
    steps.push(get("/google/oauth2-redirect"));
    steps.push(clock(NOW + 600_001.0));
    steps.push(mock(Some(keys.clone()), Some(ok_token(&nonce, &[]))));
    steps.push(callback(&state, &state, "&code=abc"));
    steps.push(clock(NOW));

    let (state, _) = plan.redirect();
    steps.push(get("/google/oauth2-redirect"));
    steps.push(callback(&state, &state, "&error=access_denied"));

    for (label, edits) in [
        ("unverified", vec![("email_verified", Value::Bool(false))]),
        ("stranger", vec![("email", s("stranger@example.com"))]),
        ("bad nonce", vec![("nonce", s("zzz"))]),
        ("bad aud", vec![("aud", s("other"))]),
        ("expired", vec![("exp", n(NOW / 1000.0 - 100.0))]),
    ] {
        let _ = label;
        let (state, nonce) = plan.redirect();
        steps.push(get("/google/oauth2-redirect"));
        steps.push(mock(Some(keys.clone()), Some(ok_token(&nonce, &edits))));
        steps.push(callback(&state, &state, "&code=abc"));
    }

    let (state, _) = plan.redirect();
    steps.push(get("/google/oauth2-redirect"));
    steps.push(mock(Some(keys.clone()), Some(token_reply(400, None))));
    steps.push(callback(&state, &state, "&code=abc"));
    let (state, _) = plan.redirect();
    steps.push(get("/google/oauth2-redirect"));
    steps.push(mock(Some(keys.clone()), Some(token_reply(200, None))));
    steps.push(callback(&state, &state, ""));
    steps.push(callback("", "", ""));
    steps.push(req("GET", "/google/oauth2-callback", &[]));

    let answers = compare("web google", &scenario(env(&[]), users(), vec![], steps));
    let signed_in = answers.iter().filter(|answer| !header_of(answer, "set-cookie").is_empty() && matches!(answer.get("body"), Value::String(body) if body.contains("/everything"))).count();
    assert!(signed_in >= 1, "web google sign-in must succeed once");
    total += answers.len();

    let mut plan = Plan { n24: 0, n32: 0 };
    let (state, nonce) = plan.redirect();
    let ok = ok_token(&nonce, &[]);
    let (state2, nonce2) = plan.redirect();
    let ok2 = ok_token(&nonce2, &[]);
    let steps = vec![
        mock(None, Some(ok.clone())),
        get("/google/oauth2-redirect"),
        callback(&state, &state, "&code=abc"),
        mock(Some(obj! { "status" => 500, "headers" => obj! {}, "body" => "x" }.into()), Some(ok.clone())),
        get("/google/oauth2-redirect"),
        mock(Some(keys_reply(&["test-key-1"], Some(1))), Some(ok2.clone())),
        callback(&state2, &state2, "&code=abc"),
        callback(&state2, &state2, "&code=abc"),
    ];
    let answers = compare("web google certs failing", &scenario(env(&[]), users(), vec![], steps));
    total += answers.len();
    let _ = (state2, nonce2);

    let mut plan = Plan { n24: 0, n32: 0 };
    let mut steps = vec![mock(Some(keys_reply(&["test-key-1"], Some(1))), None)];
    for index in 0..4 {
        let (state, nonce) = plan.redirect();
        let header = if index == 2 { Some(obj! { "kid" => "other-kid" }.into()) } else { None };
        let token = sign(claims(&nonce, &[]), header);
        steps.push(get("/google/oauth2-redirect"));
        steps.push(mock(Some(keys_reply(&["test-key-1"], Some(1))), Some(token_reply(200, Some(&token)))));
        steps.push(callback(&state, &state, "&code=abc"));
        if index < 2 {
            plan.session();
        }
        steps.push(clock(NOW + (index as f64 + 1.0) * 30_000.0));
    }
    let answers = compare("web google key cache", &scenario(env(&[]), users(), vec![], steps));
    total += answers.len();

    let mut plan = Plan { n24: 0, n32: 0 };
    let (state, nonce) = plan.redirect();
    let steps = vec![
        mock(Some(keys.clone()), Some(ok_token(&nonce, &[]))),
        req("GET", "/google/oauth2-redirect", &[("host", "replica.example.com")]),
        callback(&state, &state, "&code=abc"),
    ];
    let answers = compare(
        "web google https",
        &scenario(env(&[("PUBLIC_URL", "https://replica.example.com"), ("GOOGLE_AUTH_URL", "https://auth.example.com/a?x=1"), ("GOOGLE_ISSUER", "https://issuer.example")]), users(), vec![], steps),
    );
    assert!(header_of(&answers[2], "location").starts_with("https://auth.example.com/a?x=1?client_id=") || header_of(&answers[2], "location").contains("client_id="));
    total += answers.len();
    total
}

fn config_scenarios() -> usize {
    let mut total = 0;
    let old = NOW - 40.0 * DAY;
    let seed = vec![
        ("INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES ('old', 'a', 'b', ?, ?, ?)", vec![n(old), n(NOW - 1.0), n(old)]),
        ("INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES ('live', 'a', 'b', ?, ?, ?)", vec![n(old), n(NOW + 1000.0), n(old)]),
        ("INSERT INTO sessions (hash, sub, email, created_ms, expires_ms, used_ms) VALUES ('edge', 'a', 'b', ?, ?, ?)", vec![n(old), n(NOW), n(old)]),
        ("INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES ('t1', 'a', 'b', 'google', '', ?, ?, ?)", vec![n(old), n(NOW - 1.0), n(old)]),
        ("INSERT INTO app_tokens (hash, sub, email, kind, device, created_ms, expires_ms, used_ms) VALUES ('t2', 'a', 'b', 'google', '', ?, ?, ?)", vec![n(old), n(NOW + 5.0), n(old)]),
    ];
    for (name, config, expect) in [
        ("off", env(&[("AUTH", "")]), "none"),
        ("off value", env(&[("AUTH", "password")]), "none"),
        ("no client", env(&[("GOOGLE_CLIENT_ID", "")]), "error"),
        ("no secret", env(&[("GOOGLE_CLIENT_SECRET", "")]), "error"),
        ("no owner", env(&[("OWNER_EMAIL", "")]), "error"),
        ("blank owner", env(&[("OWNER_EMAIL", "   ")]), "error"),
        ("invalid public url", env(&[("PUBLIC_URL", "not a url")]), "ok"),
        ("trailing slash", env(&[("PUBLIC_URL", "https://x.example/")]), "ok"),
        ("startup purge", env(&[]), "ok"),
    ] {
        let steps = if expect == "ok" { vec![get("/signin"), req("GET", "/everything", &[("cookie", "mm_session=x")])] } else { vec![] };
        let answers = compare(&format!("config {name}"), &scenario(config, users(), seed.clone(), steps));
        match expect {
            "none" => assert_eq!(answers[0].get("created").as_bool(), Some(false)),
            "error" => assert!(!matches!(answers[0].get("error"), Value::Undefined)),
            _ => assert_eq!(answers[0].get("created").as_bool(), Some(true)),
        }
        total += answers.len();
    }
    total
}
