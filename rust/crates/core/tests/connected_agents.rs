//! Connected agents: an account lists and signs out the app tokens the MCP server holds for it, and nothing else.

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
use mm_core::auth_store;
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::{Error, Result, Value, obj};

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
    Gate { auth, db }
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

fn send(auth: &Auth, method: &str, path: &str, bearer: Option<&str>, body: Option<&str>) -> Response {
    let mut request = Request::new(method, &format!("http://localhost:3000{path}")).unwrap();
    if let Some(token) = bearer {
        request.headers.set("authorization", &format!("Bearer {token}"));
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

impl Gate {
    /// Signs `email` in on the app as `device` and returns the app token.
    fn token(&self, email: &str, device: &str) -> String {
        let nonce = json::parse(&text(send(&self.auth, "POST", "/app/v1/auth/nonce", None, Some(""))))
            .unwrap()
            .get("nonce")
            .as_str()
            .unwrap()
            .to_owned();
        let body = json::to_string(&obj! { "nonce" => nonce.as_str(), "idToken" => sign(&claims(&nonce, email, true)), "device" => device }.into());
        let answer = json::parse(&text(send(&self.auth, "POST", "/app/v1/auth/google", None, Some(&body)))).unwrap();
        answer.get("token").as_str().expect("signed in").to_owned()
    }

    fn agents(&self, token: &str) -> Value {
        let response = send(&self.auth, "GET", "/app/v1/agents", Some(token), None);
        assert_eq!(response.status, 200);
        json::parse(&text(response)).unwrap().get("agents").clone()
    }
}

fn hash(token: &str) -> String {
    base64::hex(&Sha256::digest(token.as_bytes()))
}

#[test]
fn an_account_lists_and_removes_only_its_own_agents() {
    let gate = open(&[]);
    auth_store::set_library(&*gate.db, "google:owner@example.com", "owner").unwrap();
    let phone = gate.token("owner@example.com", "Pixel 9");
    let claude = gate.token("owner@example.com", "MCP: Claude");
    let chatgpt = gate.token("owner@example.com", "MCP: ChatGPT");
    let ada_phone = gate.token("ada@octaviantocan.com", "Pixel 8");
    let ada_agent = gate.token("ada@octaviantocan.com", "MCP: Cursor");

    let Value::Array(listed) = gate.agents(&phone) else { panic!("a list") };
    let mut names: Vec<String> = listed.iter().map(|agent| agent.get("name").to_js_string()).collect();
    names.sort();
    assert_eq!(names, ["ChatGPT", "Claude"]);
    let claude_row = listed.iter().find(|agent| agent.get("name").as_str() == Some("Claude")).unwrap();
    assert_eq!(claude_row.get("id").as_str(), Some(hash(&claude).as_str()));
    assert!(claude_row.get("connected").as_str().unwrap().ends_with('Z'));
    let Value::Array(theirs) = gate.agents(&ada_phone) else { panic!("a list") };
    assert_eq!(theirs.len(), 1);
    assert_eq!(theirs[0].get("name").as_str(), Some("Cursor"));

    // Another account's agent, a phone and an unknown id are not this account's agents to remove.
    for id in [hash(&ada_agent), hash(&phone), "nothing".to_owned()] {
        assert_eq!(send(&gate.auth, "DELETE", &format!("/app/v1/agents/{id}"), Some(&phone), None).status, 404, "{id}");
    }
    assert_eq!(send(&gate.auth, "GET", "/app/v1/me", Some(&ada_agent), None).status, 200);

    assert_eq!(send(&gate.auth, "DELETE", &format!("/app/v1/agents/{}", hash(&claude)), Some(&phone), None).status, 204);
    assert_eq!(send(&gate.auth, "GET", "/app/v1/me", Some(&claude), None).status, 401);
    assert_eq!(send(&gate.auth, "GET", "/app/v1/me", Some(&chatgpt), None).status, 200);
    let Value::Array(left) = gate.agents(&phone) else { panic!("a list") };
    assert_eq!(left.len(), 1);

    // Signed out, the list is not there.
    assert_eq!(send(&gate.auth, "GET", "/app/v1/agents", None, None).status, 401);
    assert_eq!(send(&gate.auth, "DELETE", &format!("/app/v1/agents/{}", hash(&chatgpt)), None, None).status, 401);
    assert_eq!(send(&gate.auth, "GET", "/app/v1/me", Some(&chatgpt), None).status, 200);
}
