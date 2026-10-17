//! Port of `deploy/download-server.mjs`: the packaged apps at `/download` for anyone signed in to the hosted library,
//! checked against the sessions and app tokens the library's sign-in stores.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mm_core::auth_store::install_auth_schema;
use mm_core::download_page::{ANDROID_RELEASE, android_release, content_type, download_page};
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::sql::{Sql, SqlExt};
use mm_core::{Error, Result, params};
use mm_js::{Value, base64, json, text, text::decode_utf8, uri::decode_uri_component};
use mm_sqlite::Sqlite;
use sha2::{Digest, Sha256};

use crate::bucket_fs::file_stream;
use crate::http::Handler;
use crate::node_path::join;

/// The download server: the listing and the packaged files of one directory, for requests with a session or an app
/// token.
pub struct Downloads {
    dir: PathBuf,
    auth: Sqlite,
}

// `\s` of a JS regular expression, over the Latin-1 text Node makes of header values.
fn js_space(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}')
}

// The id after `mm_session=` at the start of `rest`, up to the next `;`.
fn id_at(rest: &str) -> Option<&str> {
    rest.strip_prefix("mm_session=").and_then(|rest| rest.split(';').next()).filter(|id| !id.is_empty())
}

// The first match of `/(?:^|;\s*)mm_session=([^;]+)/` in a cookie header.
fn session_id(cookie: &str) -> Option<&str> {
    id_at(cookie).or_else(|| cookie.match_indices(';').find_map(|(index, _)| id_at(cookie[index + 1..].trim_start_matches(js_space))))
}

// The token of an `Authorization: Bearer` header, read as the library's sign-in reads it.
fn bearer(header: &str) -> Option<&str> {
    header.get(..6).filter(|prefix| prefix.eq_ignore_ascii_case("bearer"))?;
    let rest = header[6..].trim_start_matches(' ');
    (rest.len() < header.len() - 6 && !rest.is_empty() && !rest.chars().any(text::is_whitespace)).then_some(rest)
}

fn plain(status: u16, body: &str, extra: &[(&str, &str)]) -> Response {
    let mut headers = Headers::from_pairs([("content-type", "text/plain; charset=utf-8")]);
    for (name, value) in extra {
        headers.set(*name, *value);
    }
    Response::new(status, headers, body)
}

impl Downloads {
    /// Serve `dir` to the sessions in the sign-in database at `auth_db`, creating its tables when missing.
    ///
    /// # Arguments
    /// * `dir` - Directory with `manifest.json` and the packages it lists.
    /// * `auth_db` - The library server's sign-in database.
    ///
    /// # Errors
    /// The database cannot be opened or set up.
    pub fn open(dir: PathBuf, auth_db: &Path) -> Result<Self> {
        let auth = Sqlite::open(auth_db, false)?;
        install_auth_schema(&auth)?;
        auth.exec_batch("PRAGMA busy_timeout = 2000")?;
        Ok(Self { dir, auth })
    }

    // The email of the unexpired session in the request's cookie, else of its unexpired app token, or `None`.
    fn user(&self, request: &Request) -> Result<Option<String>> {
        if let Some(id) = request.header("cookie").and_then(session_id)
            && let Some(email) = self.email("SELECT email, expires_ms FROM sessions WHERE hash = ?", id)?
        {
            return Ok(Some(email));
        }
        match request.header("authorization").and_then(bearer) {
            Some(token) => self.email("SELECT email, expires_ms FROM app_tokens WHERE hash = ?", token),
            None => Ok(None),
        }
    }

    // The email of the unexpired row that `query` finds by the hash of `secret`.
    fn email(&self, query: &str, secret: &str) -> Result<Option<String>> {
        let hash = base64::hex(&Sha256::digest(secret.as_bytes()));
        let Some(row) = self.auth.one(query, params![hash])? else { return Ok(None) };
        let live = row.number("expires_ms").is_some_and(|expires| expires > mm_core::runtime::now_ms());
        Ok(row.text("email").filter(|email| live && !email.is_empty()).map(str::to_owned))
    }

    async fn manifest(&self) -> Result<Value> {
        let text = match tokio::fs::read(self.dir.join("manifest.json")).await {
            Ok(bytes) => decode_utf8(&bytes),
            Err(_) => "{\"files\":[]}".to_owned(),
        };
        Ok(json::parse(&text)?)
    }

    async fn answer(&self, request: &Request) -> Result<Response> {
        let pathname = request.pathname();
        if request.method != "GET" && request.method != "HEAD" {
            return Ok(Response::new(405, Headers::from_pairs([("allow", "GET, HEAD")]), Body::Empty));
        }
        let listing = pathname == "/download" || pathname == "/download/";
        let Some(user) = self.user(request)? else {
            eprintln!("refused {} {}: no session", request.method, request.target);
            return Ok(if listing {
                Response::new(302, Headers::from_pairs([("location", "/signin"), ("cache-control", "no-store")]), Body::Empty)
            } else {
                plain(401, "Sign in to download the app.\n", &[("cache-control", "no-store")])
            });
        };
        if listing {
            let page = download_page(&self.manifest().await?, &user)?;
            return Ok(Response::new(200, Headers::from_pairs([("content-type", "text/html; charset=utf-8"), ("cache-control", "no-store")]), page));
        }
        let name = decode_uri_component(pathname.get("/download/".len()..).unwrap_or_default()).ok();
        let manifest = self.manifest().await?;
        if name.as_deref() == Some(ANDROID_RELEASE)
            && let Some(release) = android_release(&manifest)
        {
            let headers = Headers::from_pairs([("content-type", "application/json; charset=utf-8"), ("cache-control", "no-store")]);
            let body = json::stringify(&release)?.unwrap_or_default();
            return Ok(Response::new(200, headers, if request.method == "HEAD" { Body::Empty } else { body.into() }));
        }
        let Value::Array(files) = (match manifest {
            Value::Null | Value::Undefined => return Err(Error::invalid("Cannot read properties of null (reading 'files')")),
            other => other.get("files").clone(),
        }) else {
            return Err(Error::invalid("manifest.files.find is not a function"));
        };
        let mut found = None;
        for entry in &files {
            if entry.is_nullish() {
                return Err(Error::invalid(format!("Cannot read properties of {} (reading 'name')", entry.to_js_string())));
            }
            if let (Value::String(listed), Some(name)) = (entry.get("name"), &name)
                && listed == name
            {
                found = Some((listed.clone(), content_type(entry)));
                break;
            }
        }
        let file = found.map(|(name, kind)| (join(&self.dir, &name), name, kind));
        let info = match &file {
            Some((path, _, _)) => tokio::fs::metadata(path).await.ok(),
            None => None,
        };
        let (Some((path, name, kind)), Some(info)) = (file, info) else { return Ok(plain(404, "Not found.\n", &[])) };
        let headers = Headers::from_pairs([
            ("content-type", kind.to_owned()),
            ("content-length", info.len().to_string()),
            ("content-disposition", format!("attachment; filename=\"{name}\"")),
            ("cache-control", "no-store".to_owned()),
        ]);
        if request.method == "HEAD" {
            return Ok(Response::new(200, headers, Body::Empty));
        }
        let opened = tokio::fs::File::open(&path).await.map_err(|error| Error::internal(error.to_string()))?;
        Ok(Response::new(200, headers, Body::Stream(file_stream(opened, info.len()))))
    }
}

#[async_trait(?Send)]
impl Handler for Downloads {
    async fn handle(&self, request: Request) -> Response {
        match self.answer(&request).await {
            Ok(response) => response,
            // download-server.mjs left these rejections unhandled and Node exited; this answers 500 and keeps serving.
            Err(error) => {
                eprintln!("[error] {} {}: {error}", request.method, request.target);
                plain(500, "Internal error.\n", &[("cache-control", "no-store")])
            }
        }
    }

    fn base(&self) -> Option<&str> {
        Some("http://download")
    }
}
