//! The `Auth` Durable Object: sign-in for every library (`worker/auth-object.mjs`).

use std::rc::Rc;

use mm_core::analytics;
use mm_core::auth::{Auth as Gate, AuthOptions};
use mm_core::auth_store::{Identity, library_for_session, library_of, list_users, set_library};
use mm_core::http::{Body, Headers, Response};
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::{Error, Result, params};
use mm_js::{Value, json, obj};
use worker::{State, durable_object};

use crate::fetch::WorkerFetch;
use crate::http::{request_from, response_from};
use crate::sql_do::DoSql;

/// Header naming an internal call from the Worker or a Library object. Only this Worker sets it: `fetch` removes every
/// `x-mymind-*` header from client requests before anything reaches a Durable Object.
pub const CALL: &str = "x-mymind-call";

const UNAUTHORIZED: &str = r#"{"type":"about:blank","status":401,"title":"Unauthorized"}"#;

/// The one Durable Object that holds sign-in for every library: sessions, app tokens, password users and which library
/// each account reaches.
#[durable_object]
pub struct Auth {
    db: Option<Rc<DoSql>>,
    gate: Result<Option<Gate>>,
}

impl Auth {
    fn parts(&self) -> Result<(&Rc<DoSql>, &Gate)> {
        let gate = self.gate.as_ref().map_err(Clone::clone)?.as_ref().ok_or_else(|| Error::internal("This server has no sign-in configured."))?;
        let db = self.db.as_ref().ok_or_else(|| Error::internal("No storage"))?;
        Ok((db, gate))
    }

    async fn answer(&self, request: &web_sys::Request) -> Result<Response> {
        let (db, gate) = self.parts()?;
        let headers = crate::http::headers_of(&request.headers())?;
        if let Some(call) = headers.get(CALL) {
            return internal(db, gate, call, request).await;
        }
        let mut req = request_from(request)?;
        if let Some(response) = gate.handle(&mut req).await? {
            return Ok(response);
        }
        let Some(identity) = gate.identity(&req)? else {
            return Ok(Response::new(401, Headers::from([("content-type", "application/json"), ("cache-control", "no-store")]), UNAUTHORIZED));
        };
        let signed_in = Identity { sub: identity.sub.clone(), email: identity.email.clone() };
        let library = library_for_session(&**db, &signed_in)?;
        let headers = Headers::from([
            ("x-mymind-library", library),
            ("x-mymind-credential", identity.credential),
            ("x-mymind-email", identity.email),
            ("x-mymind-sub", identity.sub),
        ]);
        Ok(Response::new(204, headers, Body::Empty))
    }
}

fn json_response(value: &Value) -> Result<Response> {
    Ok(Response::new(200, Headers::from([("content-type", "application/json")]), json::stringify(value)?.unwrap_or_default()))
}

async fn internal(db: &Rc<DoSql>, gate: &Gate, call: &str, request: &web_sys::Request) -> Result<Response> {
    let body = |request: &web_sys::Request| -> Result<Body> { Ok(request_from(request)?.body) };
    match call {
        "summary" => {
            let count = |table: &str| -> Result<Value> {
                let row = db.one(&format!("SELECT COUNT(*) AS n FROM {table} WHERE expires_ms >= ?"), params![mm_core::runtime::now_ms()])?;
                Ok(Value::Number(row.and_then(|row| row.number("n")).unwrap_or(0.0)))
            };
            let users = list_users(&**db)?.into_iter().map(|user| Value::from(user.username)).collect::<Vec<_>>();
            let libraries = db.query("SELECT account, library FROM libraries ORDER BY account", &[])?.iter().map(|row| {
                obj! { "account" => row.text("account").unwrap_or_default(), "library" => row.text("library").unwrap_or_default() }.into()
            }).collect::<Vec<Value>>();
            json_response(&obj! { "users" => users, "libraries" => libraries, "sessions" => count("sessions")?, "appTokens" => count("app_tokens")? }.into())
        }
        "putUser" => {
            let user = json::parse(&String::from_utf8_lossy(&body(request)?.bytes().await?))?;
            let bytes = |key: &str| mm_js::base64::decode(user.get(key).as_str().unwrap_or_default());
            db.execute(
                "INSERT INTO users (username, salt, hash, created_ms) VALUES (?, ?, ?, ?) ON CONFLICT (username) DO UPDATE SET salt = excluded.salt, hash = excluded.hash",
                &[user.get("username").as_str().unwrap_or_default().into(), bytes("salt").into(), bytes("hash").into(), SqlValue::Real(user.get("created_ms").to_number())],
            )?;
            Ok(Response::new(204, Headers::new(), Body::Empty))
        }
        "setLibrary" => {
            let given = json::parse(&String::from_utf8_lossy(&body(request)?.bytes().await?))?;
            set_library(&**db, given.get("account").as_str().unwrap_or_default(), given.get("library").as_str().unwrap_or_default())?;
            Ok(Response::new(204, Headers::new(), Body::Empty))
        }
        "active" => {
            let given = json::parse(&String::from_utf8_lossy(&body(request)?.bytes().await?))?;
            let library = given.get("library").as_str().unwrap_or_default().to_owned();
            let mut live = Vec::new();
            for credential in given.get("credentials").as_array().into_iter().flatten().filter_map(Value::as_str) {
                if let Some(identity) = gate.active(credential)?
                    && library_of(&**db, &identity)?.as_deref() == Some(library.as_str())
                {
                    live.push(Value::from(credential));
                }
            }
            json_response(&Value::Array(live))
        }
        "putDaily" => {
            let given = json::parse(&String::from_utf8_lossy(&body(request)?.bytes().await?))?;
            let day = given.get("day").as_str().unwrap_or_default().to_owned();
            for row in given.get("rows").as_array().into_iter().flatten() {
                let (library, account) = (row.get("library").as_str().unwrap_or_default(), row.get("account").as_str().unwrap_or_default());
                analytics::put_daily(&**db, &day, library, account, row.get("totals"))?;
            }
            Ok(Response::new(204, Headers::new(), Body::Empty))
        }
        "daily" => {
            let given = json::parse(&String::from_utf8_lossy(&body(request)?.bytes().await?))?;
            let (since, library) = (given.get("since").as_str().unwrap_or_default(), given.get("library").as_str().unwrap_or_default());
            json_response(&analytics::daily(&**db, since, library)?)
        }
        other => Err(Error::status(404, format!("Unknown call {other}"))),
    }
}

impl worker::DurableObject for Auth {
    fn new(state: State, env: worker::Env) -> Self {
        crate::runtime::configure(&env);
        let db = DoSql::open(&state);
        let gate = db.clone().and_then(|db| {
            let sql: Rc<dyn Sql> = db;
            Gate::new(AuthOptions::new(crate::runtime::bindings(&env), 443, sql, Rc::new(WorkerFetch)))
        });
        Self { db: db.ok(), gate }
    }

    async fn fetch(&self, req: worker::Request) -> worker::Result<worker::Response> {
        match self.answer(req.inner()).await {
            Ok(response) => response_from(response).map_err(to_worker),
            Err(error) => Err(to_worker(error)),
        }
    }
}

fn to_worker(error: Error) -> worker::Error {
    worker::Error::RustError(error.to_string())
}
