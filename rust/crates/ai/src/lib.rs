//! Claude Code and Codex for ThyMind, through the AI kit it shares with Pully (`vendor/ai-kit`): each
//! person's own subscription sign-in, the CLIs' model lists, and search that Claude or Codex reads, ranks and answers
//! ([`search`]).
//!
//! The native server runs the CLIs in Docker (`rust/agent/setup.sh build`); the Worker runs them in each login's own
//! `AgentHome` container. Both answer the same `/app/v1/ai/` routes with [`answer`].

use std::future::Future;
use std::sync::LazyLock;

use ai_kit::login::{AgentLogin, Logins};
use ai_kit::{Config, Preparation};
use serde_json::{Value, json};

/// ThyMind's names for the kit. `volume` and `lock_file` are where sign-ins live; they MUST stay as they are.
static THYMIND: Config = Config {
    volume: "thymind-agents-home",
    container_prefix: "thymind",
    lock_file: ".thymind-auth.lock",
    env: "MYMIND_AGENT",
    image: "thymind-agents",
    image_setup: "rust/agent/setup.sh build",
    container_marker: "/.thymind-agent-container",
    sign_in_program: None,
    tools: None,
    missing,
    secrets: &["GEMINI_API_KEY", "AI_CHAT_API_KEY", "ADMIN_TOKEN", "GOOGLE_CLIENT_SECRET"],
    preparations: &[],
    prepare,
};

mod search;

/// Sign-ins in progress on the native server; in the Worker each login's container keeps its own.
static LOGINS: LazyLock<Logins> = LazyLock::new(Logins::default);

/// Configures the kit with ThyMind's names. Safe to call more than once.
pub fn configure() {
    ai_kit::configure(&THYMIND).expect("ThyMind is the only host of its AI kit");
}

fn missing(login: &AgentLogin, name: &str, binary: &str) -> String {
    match login {
        AgentLogin::Server => format!("{name} is not signed in on the server. Run rust/agent/setup.sh login-{binary} on the server."),
        AgentLogin::Account(_) => format!("Sign in to {name} first: Profile, AI, {name}."),
    }
}

fn prepare(_: Preparation, schema: &Value, raw: &Value) -> anyhow::Result<Value> {
    ai_kit::config::strict(schema, raw)
}

/// The login whose sign-ins a request uses: the signed-in account's own, or the server's without one.
///
/// # Arguments
/// * `email` - The session's email (`x-mymind-email`), or `None` without sign-in.
pub fn login_for(email: Option<&str>) -> AgentLogin {
    match email.map(str::trim).filter(|email| !email.is_empty()) {
        Some(email) => AgentLogin::Account(email.to_lowercase()),
        None => AgentLogin::Server,
    }
}

/// An answer to an `/app/v1/ai/` request.
#[derive(Debug, PartialEq)]
pub struct Reply {
    /// HTTP status.
    pub status: u16,
    /// JSON body.
    pub body: Value,
}

impl Reply {
    fn ok(body: Value) -> Self {
        Self { status: 200, body }
    }

    fn problem(status: u16, message: impl Into<String>) -> Self {
        Self { status, body: json!({ "error": message.into() }) }
    }

    fn from(result: anyhow::Result<Value>) -> Self {
        match result {
            Ok(body) => Self::ok(body),
            Err(error) => Self::failed(&error),
        }
    }

    /// A failed AI call: 403 busy while a feedback round has the sign-in, otherwise 502 with the redacted error.
    pub(crate) fn failed(error: &anyhow::Error) -> Self {
        if busy(error) { Self::problem(403, BUSY) } else { Self::problem(502, ai_kit::redact::safe_error(error)) }
    }
}

/// What AI search and status answer while a feedback round has the login's sign-in (docs/feedback.md).
pub const BUSY: &str = "Your AI is busy with your feedback.";

/// Whether an error is the login's `AgentHome` answering 403 busy: its sign-in is checked out to a feedback round.
fn busy(error: &anyhow::Error) -> bool {
    error.to_string().contains(r#"answered 403: {"error":"busy"}"#)
}

/// Whether `path` is one of the routes [`answer`] serves.
pub fn serves(path: &str) -> bool {
    path.starts_with("/app/v1/ai/")
}

fn engine_binary(engine: &str) -> Option<&'static str> {
    match engine {
        "claude" | "claude-code" => Some("claude"),
        "codex" => Some("codex"),
        _ => None,
    }
}

/// Answers an `/app/v1/ai/` request.
///
/// Routes: `GET sign-ins`; `POST sign-ins/{claude|codex}/start`, `/finish` (`{code}`), `/cancel` and `/sign-out`;
/// `GET models?engine=claude-code|codex&refresh=1`; `POST search/understand` (`{q, engine, model, effort, today}`),
/// which answers `{understood}`; and `POST search` (`{q, engine, model, effort, understood}`), which answers
/// `{cards, next, ranked, understood, answer}`.
///
/// # Arguments
/// * `login` - From [`login_for`]; only an account may change a sign-in.
/// * `method` - HTTP method.
/// * `url` - The request URL.
/// * `body` - The request body.
/// * `library` - Answers a GET of the library's own `/app/v1/` routes (`search?q=…&semantic=1&limit=…`, `tags`,
///   `spaces` and `spaces/{id}/cards?limit=…`) with that route's JSON. A search calls it several times at once.
///
/// # Returns
/// The answer, or `None` when the path is not one of these routes.
pub async fn answer<F, Fut>(login: &AgentLogin, method: &str, url: &url::Url, body: &[u8], library: F) -> Option<Reply>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    configure();
    let rest = url.path().strip_prefix("/app/v1/ai/")?;
    let parts: Vec<&str> = rest.split('/').collect();
    let args: Value = if body.is_empty() { json!({}) } else { serde_json::from_slice(body).unwrap_or(Value::Null) };
    Some(match (method, parts.as_slice()) {
        ("GET", ["sign-ins"]) => Reply::from(ai_kit::login::status(login).await.map(|mut value| {
            value["account"] = json!(matches!(login, AgentLogin::Account(_)));
            value
        })),
        ("POST", ["sign-ins", cli, action]) => sign_in(login, cli, action, &args).await,
        ("GET", ["models"]) => {
            let query = |name: &str| url.query_pairs().find(|(key, _)| key == name).map(|(_, value)| value.into_owned());
            let engine = query("engine").unwrap_or_default();
            if engine_binary(&engine).is_none() {
                return Some(Reply::problem(400, "Unknown AI engine."));
            }
            Reply::from(ai_kit::process::catalog(&engine, login).await)
        }
        ("POST", ["search"]) => search::search(login, &args, &library).await,
        ("POST", ["search", "understand"]) => search::understand(login, &args, &library).await,
        _ => Reply::problem(404, "not-found"),
    })
}

async fn sign_in(login: &AgentLogin, cli: &str, action: &str, args: &Value) -> Reply {
    // 403, not 401: the app signs its session out on a 401.
    if login == &AgentLogin::Server {
        return Reply::problem(403, "Sign in to ThyMind first, so the AI sign-in is yours.");
    }
    let Some(binary) = engine_binary(cli) else { return Reply::problem(404, "not-found") };
    Reply::from(match action {
        "start" => LOGINS.start(login, binary).await,
        "finish" => LOGINS.finish(login, binary, args.get("code").and_then(Value::as_str)).await,
        "cancel" => {
            LOGINS.cancel(login, binary).await;
            Ok(json!({ "ok": true }))
        }
        "sign-out" => {
            LOGINS.cancel(login, binary).await;
            ai_kit::login::sign_out(login, binary).await.map(|_| json!({ "ok": true }))
        }
        _ => return Reply::problem(404, "not-found"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logins_are_per_account_and_case_blind() {
        configure();
        assert_eq!(login_for(None), AgentLogin::Server);
        assert_eq!(login_for(Some("  ")), AgentLogin::Server);
        assert_eq!(login_for(Some("Tavi@Example.com")), login_for(Some("tavi@example.com")));
        assert!(login_for(Some("a@example.com")).volume().starts_with("thymind-agents-home-"));
        assert_eq!(AgentLogin::Server.volume(), "thymind-agents-home");
        assert_eq!(login_for(Some("a@example.com")).missing("codex"), "Sign in to Codex first: Profile, AI, Codex.");
    }

    #[test]
    fn routes_refuse_what_they_do_not_serve() {
        let url = |path: &str| url::Url::parse(&format!("https://thymind.test{path}")).unwrap();
        let run = |login: AgentLogin, method: &'static str, path: &str, body: &'static [u8]| {
            let url = url(path);
            futures_lite_block(async move { answer(&login, method, &url, body, |_| async { Ok(json!({ "cards": [] })) }).await })
        };
        assert_eq!(run(AgentLogin::Server, "GET", "/app/v1/search", b""), None);
        assert_eq!(run(AgentLogin::Server, "POST", "/app/v1/ai/sign-ins/claude/start", b"").unwrap().status, 403);
        let account = login_for(Some("a@example.com"));
        assert_eq!(run(account.clone(), "POST", "/app/v1/ai/sign-ins/gemini/start", b"").unwrap().status, 404);
        assert_eq!(run(account.clone(), "GET", "/app/v1/ai/models?engine=pi", b"").unwrap().status, 400);
        assert_eq!(run(account.clone(), "POST", "/app/v1/ai/search", br#"{"q":" "}"#).unwrap().status, 400);
        assert_eq!(run(account.clone(), "POST", "/app/v1/ai/search/understand", br#"{"q":""}"#).unwrap().status, 400);
        assert_eq!(run(account.clone(), "POST", "/app/v1/ai/search/understand", br#"{"q":"calm","engine":"pi"}"#).unwrap().status, 400);
        assert_eq!(run(account.clone(), "GET", "/app/v1/ai/search/understand", b"").unwrap().status, 404);
        assert_eq!(
            run(account, "POST", "/app/v1/ai/search", br#"{"q":"calm","engine":"codex"}"#).unwrap(),
            Reply::ok(json!({
                "cards": [], "next": null, "ranked": 0, "answer": null,
                "understood": { "label": "", "parts": [], "searches": [], "tags": [], "spaces": [], "question": false }
            }))
        );
    }

    #[test]
    fn a_checked_out_login_answers_busy() {
        let error = anyhow::anyhow!(r#"The AI container answered 403: {{"error":"busy"}}"#);
        assert_eq!(Reply::from(Err(error)), Reply::problem(403, BUSY));
        assert_eq!(Reply::from(Err(anyhow::anyhow!("The AI container answered 500: boom"))).status, 502);
    }

    fn futures_lite_block<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }
}
