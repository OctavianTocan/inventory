//! Usage analytics in the Worker (`docs/analytics.md`): events written to Workers Analytics Engine (`EVENTS`), the
//! daily totals the cron trigger keeps in the `Auth` object, and the owner's two routes under `/app/v1/analytics/`.

use mm_core::analytics::{self, Event, Origin};
use mm_core::http::{Headers, Response};
use mm_core::{Error, Result};
use mm_js::{Value, json, obj};
use wasm_bindgen::JsValue;
use worker::{AnalyticsEngineDataPointBuilder, Env};

use crate::util::worker_error;
use crate::{auth_call, forward, json_response, send};

const SQL_API: &str = "https://api.cloudflare.com/client/v4/accounts";

/// Whether this Worker records events: the `EVENTS` binding exists and `ANALYTICS` is not `off`.
fn on(env: &Env) -> bool {
    analytics::enabled(
        crate::runtime::bindings(env)
            .get("ANALYTICS")
            .map(String::as_str),
    )
}

/// Write one event to Analytics Engine. A missing binding or a failed write is logged and never fails the request.
///
/// # Arguments
/// * `env` - The Worker's or object's bindings.
/// * `event` - What happened.
/// * `origin` - Whose it is and where it came from.
pub fn write(env: &Env, event: &Event, origin: &Origin) {
    if !on(env) {
        return;
    }
    let Ok(dataset) = env.analytics_engine("EVENTS") else {
        return;
    };
    let point = analytics::point(event, origin);
    let data = AnalyticsEngineDataPointBuilder::new()
        .indexes([point.index.as_str()])
        .blobs(point.blobs)
        .doubles(point.doubles)
        .build();
    if let Err(error) = dataset.write_data_point(&data) {
        web_sys::console::error_2(&"[analytics]".into(), &error.to_string().into());
    }
}

/// Whether `path` is one of the owner's analytics routes.
pub fn serves(path: &str) -> bool {
    path.starts_with("/app/v1/analytics/")
}

/// The owner's analytics routes. Every other account gets the same 404 as an unknown path, so the routes stay hidden.
///
/// `POST /app/v1/analytics/query` takes `{sql}`, one bounded `SELECT` on the `thymind_events` dataset, and answers
/// [`analytics::shorten`] of Cloudflare's SQL API answer plus the column layout. `GET /app/v1/analytics/daily?since=
/// YYYY-MM-DD&library=` answers the kept daily totals.
///
/// # Arguments
/// * `request` - The incoming request.
/// * `env` - The Worker's bindings.
/// * `url` - The request URL.
/// * `email` - The signed-in account's verified email.
///
/// # Errors
/// A failure reaching the `Auth` object.
pub async fn answer(
    request: &web_sys::Request,
    env: &Env,
    url: &url::Url,
    email: &str,
) -> Result<Response> {
    let vars = crate::runtime::bindings(env);
    let owner = vars
        .get("OWNER_EMAIL")
        .map(|owner| owner.trim().to_lowercase())
        .filter(|owner| !owner.is_empty());
    if owner.is_none_or(|owner| owner != email.trim().to_lowercase()) {
        return json_response(404, &obj! { "error" => "not-found" }.into());
    }
    let method = request.method();
    match url.path() {
        "/app/v1/analytics/query" if method == "POST" => {
            let body = crate::body_text(request).await?;
            let sql = json::parse(&body)
                .ok()
                .and_then(|body| body.get("sql").as_str().map(str::to_owned))
                .unwrap_or_default();
            let sql = match analytics::bounded_select(&sql) {
                Ok(sql) => sql,
                Err(error) => {
                    return json_response(400, &obj! { "error" => error.to_string() }.into());
                }
            };
            let (Some(account), Some(token)) = (
                vars.get("ANALYTICS_ACCOUNT_ID").filter(|v| !v.is_empty()),
                vars.get("ANALYTICS_TOKEN").filter(|v| !v.is_empty()),
            ) else {
                return json_response(503, &obj! { "error" => "Analytics is not set up: ANALYTICS_ACCOUNT_ID and ANALYTICS_TOKEN are missing." }.into());
            };
            let (status, text) = sql_api(account, token, &sql).await?;
            if status != 200 {
                let message = text.chars().take(500).collect::<String>();
                return json_response(
                    502,
                    &obj! { "error" => format!("Analytics Engine answered {status}: {message}") }
                        .into(),
                );
            }
            let mut answer = analytics::shorten(&json::parse(&text)?);
            if let Value::Object(fields) = &mut answer {
                fields.insert("layout".to_owned(), analytics::LAYOUT);
            }
            json_response(200, &answer)
        }
        "/app/v1/analytics/daily" if method == "GET" => {
            let param = |name: &str| {
                url.query_pairs()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.into_owned())
                    .unwrap_or_default()
            };
            let body = json::stringify(
                &obj! { "since" => param("since"), "library" => param("library") }.into(),
            )?
            .unwrap_or_default();
            auth_call(env, "daily", Some(body)).await
        }
        "/app/v1/analytics/query" | "/app/v1/analytics/daily" => {
            json_response(405, &obj! { "error" => "method-not-allowed" }.into())
        }
        _ => json_response(404, &obj! { "error" => "not-found" }.into()),
    }
}

async fn sql_api(account: &str, token: &str, sql: &str) -> Result<(u16, String)> {
    let headers = worker::Headers::new();
    headers
        .set("authorization", &format!("Bearer {token}"))
        .map_err(worker_error)?;
    let mut init = worker::RequestInit::new();
    init.with_method(worker::Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(sql)));
    let request =
        worker::Request::new_with_init(&format!("{SQL_API}/{account}/analytics_engine/sql"), &init)
            .map_err(worker_error)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(worker_error)?;
    Ok((
        response.status_code(),
        response.text().await.map_err(worker_error)?,
    ))
}

/// Keep today's totals for every library: list them from the `Auth` object, ask each `Library` object for its counts
/// and store one row per library for the UTC day. A library that fails gets an `{error}` row; the others still run.
///
/// # Arguments
/// * `env` - The Worker's bindings.
/// * `at_ms` - When the cron trigger fired, which names the day.
///
/// # Errors
/// A failure reading the list of libraries or storing the rows.
pub async fn daily_totals(env: &Env, at_ms: f64) -> Result<()> {
    let day = js_sys::Date::new(&JsValue::from_f64(at_ms))
        .to_iso_string()
        .as_string()
        .unwrap_or_default()
        .chars()
        .take(10)
        .collect::<String>();
    let summary = auth_call(env, "summary", None).await?;
    let summary = json::parse(&String::from_utf8_lossy(&summary.body.bytes().await?))?;
    let namespace = env.durable_object("LIBRARY").map_err(worker_error)?;
    let mut rows = Vec::new();
    for entry in summary.get("libraries").as_array().into_iter().flatten() {
        let (account, library) = (
            entry.get("account").to_js_string(),
            entry.get("library").to_js_string(),
        );
        let totals = async {
            let stub = namespace.get_by_name(&library).map_err(worker_error)?;
            let headers = Headers::from([
                ("x-mymind-library", library.as_str()),
                ("x-mymind-admin", "totals"),
            ]);
            let response = send(
                &stub,
                forward("http://library/_admin/totals", "GET", &headers, None)?,
            )
            .await?;
            if response.status != 200 {
                return Err(Error::internal(format!(
                    "library answered {}",
                    response.status
                )));
            }
            Ok(json::parse(&String::from_utf8_lossy(
                &response.body.bytes().await?,
            ))?)
        }
        .await
        .unwrap_or_else(|error| obj! { "error" => error.to_string() }.into());
        rows.push(Value::from(
            obj! { "account" => account, "library" => library, "totals" => totals },
        ));
    }
    let body = json::stringify(&obj! { "day" => day, "rows" => rows }.into())?.unwrap_or_default();
    auth_call(env, "putDaily", Some(body)).await?;
    Ok(())
}

/// Log a failed daily run where Workers Logs keeps it.
pub fn log_failure(error: &Error) {
    web_sys::console::error_2(
        &"[analytics] daily totals".into(),
        &error.to_string().into(),
    );
}
