//! Verified addresses at the allowed domain sign in on the web and the app, and get their own library.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_js::{base64, json};
use mm_sqlite::Sqlite;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use sha2::{Digest, Sha256};

use mm_core::auth::{Auth, AuthOptions};
use mm_core::auth_store::{self, Identity};
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::sql::{Sql, SqlExt};
use mm_core::{Error, Result, Value, obj, params};

const KEY: &str = "MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCM5G0cfsoMd1b8Fz+T4nzsg/B+Xk1yMv050rleRsVujX9TJc03CIUASWE69X2XVGan9JwZCfqdNggVhiASTTDQ9VPzJpm5Re+jecWaoBg4fD70EvOVFXvcOBs/VmUd2qV7iDWdx+C3FlIAYiXXPK+NVzfGGQtvd2QwW3ZY1yvT/F98O5yu/NPVnWU3BwefSkk4SQOxgwgv6BED6zqCCcg+ngU3D0+DjWyMc4k1A+Jy7qrtbuplJyG+Z/JXTiG0Ukk1ZgVSkDE7R7t/hN1riqeFSWI2IisvUldyWAPJaXflftYr0bxPohXaFHhzw09QYeJfKzY/YMb+0c9mSC5BoTetAgMBAAECggEAHhOzgUZFzsR5MsapwWGzxm/iGq/5+3uMGFh2TvxvsEqPmEn1FZ8BeY6wrXvzrCcN5SPwYN4C/KWosyOXXCDTX4BHxBwE7ArUfgMEXp53nTNU4xjanARilR/7HRnN60IeWpE41O5y99LRfZh2OI0RcuxgES+Og+SqZDJaQNvqVM310eTC5eUFBjKg0gGvp4zcjAYvlPhmjFqw78qGjt+TZ95DLbWMYBaunTwHf890G5VZYL4BLfFYZFj5lr0Y67fimonFwBfxP7duwUo8pZYmPHkgoD2WdxqFx+Oz1Pp21WDqihqqOeoIieiEnp9bFNxbvW3DkaC2oS8tT5/DFldygQKBgQDGFL2/CnX83Xk6drWx2bn4dCmySxRdhsuS5llptBOTIUB40xcp8ytX0eQUGhFuvdVlQ5woEd5lQBOvIEmo8ivLMK6J29ECFMAxpz1veE3N+itHpGXz5wWW8+1KR3zHQq6Lpi6+79LIZT518JBlOy8ocGO5bnzWoKzuGp2OME1mwQKBgQC2FtmuM7FbXt+TfL4a92Ol4ceDL71PyZOzG3KcZonLi29L9ps8hpCsZ0rHwG+tPsZPIYgkB3ciXWLD1nRvX/T9a6oOKZR/Xdm780AoOnUvDrhh9jvZzYwJwtoqVJqUn9qahWUi0ecsX9srXEnR5dpTgPHUWNl3uuRE+nCUaO3X7QKBgGczbTy4lYp+VWf5a2bTGkpxMzw6U8z6qtvJefJRqRUx8OZCjMIFeKKTr3IttsFs1F6jIBvcW3ODM/49I0OLkPeXUIhw529fIld8W8zU8CyN17jG1XuC9O2V/x6BKmiivE/z8Ez4pGhWH3nRMVocwNAf5kU5ZH9qTU7svQPU9K7BAoGAYZj3zgZI/rdoUI4g+yU0VPoiTO9uofBiaTLCCJBxNRckbWvdouBhdPbcPxqnAQ3PjyRi6yTRNyskv+EXgYbGMnrdqD+jJ21vQe1MqwNCQhm6vE8Vn1UrxSOaeevk+tp/eikbn0EwjjlGqQQLUSTJE87eT7nbve//X/V4GCPxr7UCgYBf0sB2wg3QP91gSy1s3J7DXMy1a/pAsFv/zBCt6oOiRbvsof2QLyUV0dnodO6FlfBH/1tuLkyWa/UYUW2dBldOX+kzeoHcj+I5MYML5ndLHuU8d2do061w1H6GZGtyOjsjrUnbkt9idUxJuYSYn3Bwqlkk2RQrXLCaGHIlRv5iug==";

#[derive(Default)]
struct Mocks {
    certs: Option<Value>,
    token: Option<Value>,
}

struct MockFetch(Rc<RefCell<Mocks>>);

#[async_trait(?Send)]
impl Fetch for MockFetch {
    async fn fetch(&self, request: Request) -> Result<Response> {
        let mock = if request.method == "POST" { self.0.borrow().token.clone() } else { self.0.borrow().certs.clone() };
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

struct Gate {
    auth: Auth,
    db: Rc<Sqlite>,
    mocks: Rc<RefCell<Mocks>>,
}

fn open(extra: &[(&str, &str)]) -> Gate {
    let db = Rc::new(Sqlite::open(":memory:", false).unwrap());
    let mut env = HashMap::from([
        ("AUTH".into(), "google".into()),
        ("GOOGLE_CLIENT_ID".into(), "client-1".into()),
        ("GOOGLE_CLIENT_SECRET".into(), "secret-1".into()),
        ("OWNER_EMAIL".into(), " Owner@Example.com ".into()),
        ("GOOGLE_CERTS_URL".into(), "http://127.0.0.1:9/certs".into()),
        ("GOOGLE_TOKEN_URL".into(), "http://127.0.0.1:9/token".into()),
    ]);
    for (name, value) in extra {
        env.insert((*name).to_owned(), (*value).to_owned());
    }
    let mocks = Rc::new(RefCell::new(Mocks::default()));
    let auth = Auth::new(AuthOptions::new(env, 3000, db.clone(), Rc::new(MockFetch(mocks.clone())))).unwrap().unwrap();
    let key = RsaPrivateKey::from_pkcs8_der(&base64::decode(KEY)).unwrap();
    let jwk = obj! {
        "kty" => "RSA",
        "n" => base64::encode_url(&key.n().to_bytes_be()),
        "e" => base64::encode_url(&key.e().to_bytes_be()),
        "kid" => "test-key-1",
        "alg" => "RS256",
    };
    mocks.borrow_mut().certs = Some(
        obj! {
            "status" => 200.0,
            "headers" => obj! { "cache-control" => "public, max-age=3600" },
            "body" => json::to_string(&obj! { "keys" => vec![Value::from(jwk)] }.into()),
        }
        .into(),
    );
    Gate { auth, db, mocks }
}

fn sign(claims: &Value) -> String {
    let header = obj! { "alg" => "RS256", "kid" => "test-key-1", "typ" => "JWT" };
    let message = format!(
        "{}.{}",
        base64::encode_url(json::to_string(&header.into()).as_bytes()),
        base64::encode_url(json::to_string(claims).as_bytes())
    );
    let key = RsaPrivateKey::from_pkcs8_der(&base64::decode(KEY)).unwrap();
    let signature = key.sign(Pkcs1v15Sign::new::<Sha256>(), &Sha256::digest(message.as_bytes())).unwrap();
    format!("{message}.{}", base64::encode_url(&signature))
}

fn claims(nonce: &str, email: &str, verified: bool) -> Value {
    let now = mm_core::runtime::now_ms() / 1000.0;
    obj! {
        "iss" => "https://accounts.google.com",
        "aud" => "client-1",
        "sub" => format!("sub-{email}"),
        "email" => email,
        "email_verified" => verified,
        "nonce" => nonce,
        "iat" => now,
        "exp" => now + 3600.0,
    }
    .into()
}

fn send(auth: &Auth, method: &str, path: &str, cookie: Option<&str>, body: Option<&str>) -> Response {
    let mut request = Request::new(method, &format!("http://localhost:3000{path}")).unwrap();
    if let Some(cookie) = cookie {
        request.headers.set("cookie", cookie);
    }
    if let Some(body) = body {
        request.headers.set("content-type", "application/json");
        request.body = Body::Bytes(body.as_bytes().to_vec());
    }
    block_on(auth.handle(&mut request)).unwrap().expect("sign-in handled the request")
}

fn text(response: Response) -> String {
    String::from_utf8(block_on(response.body.bytes()).unwrap()).unwrap()
}

fn libraries(db: &dyn Sql) -> i64 {
    db.one("SELECT COUNT(*) AS n FROM libraries", &[]).unwrap().unwrap().number("n").unwrap() as i64
}

fn library_for(db: &dyn Sql, email: &str) -> String {
    let row = db
        .one("SELECT sub, email FROM sessions WHERE email = ? UNION ALL SELECT sub, email FROM app_tokens WHERE email = ?", params![email, email])
        .unwrap()
        .unwrap();
    auth_store::library_for_session(
        db,
        &Identity { sub: row.text("sub").unwrap().to_owned(), email: row.text("email").unwrap().to_owned() },
    )
    .unwrap()
}

impl Gate {
    fn app(&self, email: &str, verified: bool) -> (u16, String) {
        let nonce = json::parse(&text(send(&self.auth, "POST", "/app/v1/auth/nonce", None, Some(""))))
            .unwrap()
            .get("nonce")
            .as_str()
            .unwrap()
            .to_owned();
        let body = json::to_string(&obj! { "nonce" => nonce.as_str(), "idToken" => sign(&claims(&nonce, email, verified)), "device" => "phone" }.into());
        let response = send(&self.auth, "POST", "/app/v1/auth/google", None, Some(&body));
        let status = response.status;
        (status, text(response))
    }

    fn web(&self, email: &str, verified: bool) -> (u16, String) {
        let started = send(&self.auth, "GET", "/google/oauth2-redirect", None, None);
        let status = started.status;
        assert_eq!(status, 302, "{}", text(started));
        let row = self.db.one("SELECT state, nonce FROM logins ORDER BY rowid DESC LIMIT 1", &[]).unwrap().unwrap();
        let state = row.text("state").unwrap().to_owned();
        let nonce = row.text("nonce").unwrap().to_owned();
        let token = sign(&claims(&nonce, email, verified));
        self.mocks.borrow_mut().token = Some(
            obj! { "status" => 200.0, "headers" => obj! {}, "body" => json::to_string(&obj! { "id_token" => token }.into()) }.into(),
        );
        let response = send(
            &self.auth,
            "GET",
            &format!("/google/oauth2-callback?state={state}&code=abc"),
            Some(&format!("mm_oauth={state}")),
            None,
        );
        let status = response.status;
        (status, text(response))
    }
}

#[test]
fn verified_domain_user_is_admitted_and_gets_a_new_library() {
    let gate = open(&[]);
    auth_store::set_library(&*gate.db, "google:owner@example.com", "owner").unwrap();

    let (status, body) = gate.web("Owner@Example.com", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("/everything"), "{body}");
    assert_eq!(library_for(&*gate.db, "owner@example.com"), "owner");
    assert_eq!(libraries(&*gate.db), 1);

    let (status, body) = gate.app("Owner@Example.com", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("owner@example.com"), "{body}");
    assert_eq!(library_for(&*gate.db, "owner@example.com"), "owner");
    assert_eq!(libraries(&*gate.db), 1);

    let (status, body) = gate.web("Ada@OctavianTocan.com", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("/everything"), "{body}");
    let created = library_for(&*gate.db, "ada@octaviantocan.com");
    assert!(created.starts_with("u-ada-octaviantocan-com-"), "{created}");
    assert!(auth_store::generated_library("google:ada@octaviantocan.com", &created));
    assert_ne!(created, "owner");
    assert_eq!(libraries(&*gate.db), 2);

    let (status, body) = gate.app("Ada@OctavianTocan.com", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("ada@octaviantocan.com"), "{body}");
    assert!(!body.contains("owner@example.com"), "{body}");
    assert_eq!(library_for(&*gate.db, "ada@octaviantocan.com"), created);
    assert_eq!(auth_store::library_of(&*gate.db, &Identity { sub: "sub-owner".into(), email: "owner@example.com".into() }).unwrap().as_deref(), Some("owner"));
    assert_eq!(libraries(&*gate.db), 2);
}

#[test]
fn unverified_other_domain_and_lookalikes_are_rejected() {
    let gate = open(&[]);
    auth_store::set_library(&*gate.db, "google:owner@example.com", "owner").unwrap();
    let rejected = [
        ("ada@octaviantocan.com", false),
        ("ada@example.com", true),
        ("ada@octaviantocan.com.evil.com", true),
        ("ada@sub.octaviantocan.com", true),
        ("ada@eviloctaviantocan.com", true),
        ("ada@evil\u{043e}ctaviantocan.com", true),
        ("ada@octaviantocan.com@evil.com", true),
        ("@octaviantocan.com", true),
        ("octaviantocan.com", true),
        ("ada@octaviantocan.com. ", true),
    ];
    for (email, verified) in rejected {
        let (status, body) = gate.app(email, verified);
        let expected = if verified { 401 } else { 400 };
        assert_eq!(status, expected, "{email} app {body}");
        if verified {
            assert!(body.contains("not-allowed"), "{email} {body}");
        } else {
            assert!(body.contains("invalid-token"), "{email} {body}");
        }
        let (status, body) = gate.web(email, verified);
        assert_eq!(status, 200, "{email} web {body}");
        assert!(body.contains("AccountNotFound"), "{email} web {body}");
        assert!(!body.contains("/everything"), "{email} web {body}");
        assert_eq!(libraries(&*gate.db), 1, "{email}");
    }

    let owner_only = open(&[("ALLOWED_GOOGLE_DOMAIN", "")]);
    auth_store::set_library(&*owner_only.db, "google:owner@example.com", "owner").unwrap();
    let (status, body) = owner_only.app("ada@octaviantocan.com", true);
    assert_eq!(status, 401, "{body}");
    let (status, body) = owner_only.web("Owner@Example.com", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("/everything"), "{body}");
    assert_eq!(library_for(&*owner_only.db, "owner@example.com"), "owner");
    assert_eq!(libraries(&*owner_only.db), 1);

    let custom = open(&[("ALLOWED_GOOGLE_DOMAIN", "Example.TEST")]);
    let (status, body) = custom.web("Ada@Example.TEST", true);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("/everything"), "{body}");
    let name = library_for(&*custom.db, "ada@example.test");
    assert!(auth_store::generated_library("google:ada@example.test", &name), "{name}");
    let (status, body) = custom.app("ada@octaviantocan.com", true);
    assert_eq!(status, 401, "{body}");
    assert_eq!(libraries(&*custom.db), 1);
}
