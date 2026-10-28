//! Claude Code and Codex for ThyMind, through the AI kit it shares with Pully (`vendor/pulls.review/ai-kit`): each
//! person's own subscription sign-in, the CLIs' model lists, and search that Claude or Codex re-ranks by meaning.
//!
//! The native server runs the CLIs in Docker (`rust/agent/setup.sh build`); the Worker runs them in each login's own
//! `AgentHome` container. Both answer the same `/app/v1/ai/` routes with [`answer`].

use std::future::Future;
use std::sync::{Arc, LazyLock};

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

/// Most candidates one AI search ranks.
const CANDIDATES: usize = 40;
/// Characters of a candidate's description and note the model reads.
const SNIPPET_CHARS: usize = 280;
/// Characters of a search query.
const QUERY_CHARS: usize = 500;

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
            Err(error) => Self::problem(502, ai_kit::redact::safe_error(&error)),
        }
    }
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
/// `GET models?engine=claude-code|codex&refresh=1`; `POST search` (`{q, engine, model, effort}`).
///
/// # Arguments
/// * `login` - From [`login_for`]; only an account may change a sign-in.
/// * `method` - HTTP method.
/// * `url` - The request URL.
/// * `body` - The request body.
/// * `candidates` - Answers `/app/v1/search` for the query it is given (`?q=…&semantic=1&limit=…`) as that route's JSON.
///
/// # Returns
/// The answer, or `None` when the path is not one of these routes.
pub async fn answer<F, Fut>(login: &AgentLogin, method: &str, url: &url::Url, body: &[u8], candidates: F) -> Option<Reply>
where
    F: FnOnce(String) -> Fut,
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
        ("POST", ["search"]) => search(login, &args, candidates).await,
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

/// The answer schema of a search: the matching ids, best first.
fn ranking_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "ids": { "type": "array", "items": { "type": "string" } } },
        "required": ["ids"],
        "additionalProperties": false
    })
}

const SEARCH_SYSTEM: &str = "You search one person's saved library of links, notes, images and videos. Rank the \
items by how well each one matches what the search means, not by shared words: a search for \"calm places\" matches \
a lake photo with no words in common. Answer with the ids of the items that match, best match first, and leave out \
the ones that do not match at all. Use only ids from the list.";

fn clip(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match collapsed.char_indices().nth(max) {
        Some((at, _)) => format!("{}…", &collapsed[..at]),
        None => collapsed,
    }
}

/// The words of a ProseMirror document (a note's or an article's `prose`), in order.
fn prose_text(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        out.push_str(text);
        out.push(' ');
    }
    for child in node.get("content").and_then(Value::as_array).into_iter().flatten() {
        prose_text(child, out);
    }
}

/// What the model reads of one card: its id and the words that say what it is.
fn candidate(card: &Value) -> Option<Value> {
    let id = card.get("id")?.as_str()?;
    let text = |key: &str, max: usize| card.get(key).and_then(Value::as_str).map(|value| clip(value, max)).filter(|value| !value.is_empty());
    let tags: Vec<&str> = card
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tag| tag.as_str().or_else(|| tag.get("name").and_then(Value::as_str)))
        .collect();
    let mut body = String::new();
    prose_text(card.get("prose").unwrap_or(&Value::Null), &mut body);
    let body = Some(clip(&body, SNIPPET_CHARS)).filter(|value| !value.is_empty());
    let mut item = json!({ "id": id });
    for (key, value) in [
        ("type", text("type", 40)),
        ("title", text("title", SNIPPET_CHARS)),
        ("description", text("description", SNIPPET_CHARS)),
        ("text", body),
        ("note", text("note", SNIPPET_CHARS)),
        ("domain", text("domain", 80)),
        ("site", text("siteName", 80)),
        ("author", text("author", 80)),
    ] {
        if let Some(value) = value {
            item[key] = json!(value);
        }
    }
    if !tags.is_empty() {
        item["tags"] = json!(tags);
    }
    Some(item)
}

/// `cards` with the ones in `ids` first, in that order, then the rest as they were. Unknown and repeated ids are
/// skipped. Returns the cards and how many the ranking placed.
fn reorder(cards: Vec<Value>, ids: &[String]) -> (Vec<Value>, usize) {
    let mut rest: Vec<Option<Value>> = cards.into_iter().map(Some).collect();
    let mut ranked = Vec::new();
    for id in ids {
        if let Some(slot) = rest.iter_mut().find(|slot| slot.as_ref().is_some_and(|card| card.get("id").and_then(Value::as_str) == Some(id))) {
            ranked.push(slot.take().expect("found above"));
        }
    }
    let count = ranked.len();
    ranked.extend(rest.into_iter().flatten());
    (ranked, count)
}

fn percent(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

async fn search<F, Fut>(login: &AgentLogin, args: &Value, candidates: F) -> Reply
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let setting = |key: &str| args.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty());
    let Some(query) = setting("q") else { return Reply::problem(400, "missing-query") };
    let query = clip(query, QUERY_CHARS);
    let engine = setting("engine").unwrap_or("claude-code");
    if engine_binary(engine).is_none() {
        return Reply::problem(400, "Unknown AI engine.");
    }
    let found = match candidates(format!("/app/v1/search?q={}&semantic=1&limit={CANDIDATES}", percent(&query))).await {
        Ok(found) => found,
        Err(error) => return Reply::problem(502, ai_kit::redact::safe_error(&error)),
    };
    let cards: Vec<Value> = found.get("cards").and_then(Value::as_array).cloned().unwrap_or_default();
    if cards.is_empty() {
        return Reply::ok(json!({ "cards": [], "next": null, "ranked": 0 }));
    }
    let items: Vec<String> = cards.iter().filter_map(candidate).map(|item| item.to_string()).collect();
    let mut request = json!({
        "engine": engine,
        "system": SEARCH_SYSTEM,
        "prompt": format!("Search: {query}\n\nItems, one JSON object a line:\n{}", items.join("\n")),
        "schema": ranking_schema(),
    });
    for key in ["model", "effort"] {
        if let Some(value) = setting(key) {
            request[key] = json!(value);
        }
    }
    // An AI search ends with its request, so nothing cancels it.
    let (_cancel, cancelled) = tokio::sync::watch::channel(false);
    let answer = ai_kit::process::run_agent(request, login, cancelled, Arc::new(|_| {})).await;
    let ids: Vec<String> = match answer.and_then(|(text, _)| Ok(serde_json::from_str::<Value>(&text)?)) {
        Ok(answer) => answer
            .get("ids")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|id| id.as_str().map(str::to_owned))
            .collect(),
        Err(error) => return Reply::problem(502, ai_kit::redact::safe_error(&error)),
    };
    let (cards, ranked) = reorder(cards, &ids);
    Reply::ok(json!({ "cards": cards, "next": null, "ranked": ranked }))
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
    fn candidates_carry_only_what_says_what_a_card_is() {
        let card = json!({
            "id": "c1", "type": "WebPage", "title": "  A\n lake ", "description": "x".repeat(400),
            "domain": "example.com", "tags": [{ "name": "calm" }, "water"], "url": "https://example.com/?token=1",
            "blob": { "path": "a.png" },
            "prose": { "type": "doc", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Still" }, { "type": "text", "text": "water." }] }] }
        });
        let item = candidate(&card).unwrap();
        assert_eq!(item["title"], "A lake");
        assert_eq!(item["text"], "Still water.");
        assert_eq!(item["description"].as_str().unwrap().chars().count(), SNIPPET_CHARS + 1);
        assert_eq!(item["tags"], json!(["calm", "water"]));
        assert!(item.get("url").is_none() && item.get("blob").is_none());
        assert_eq!(candidate(&json!({ "title": "no id" })), None);
    }

    #[test]
    fn the_ranking_comes_first_and_nothing_is_lost() {
        let cards: Vec<Value> = ["a", "b", "c", "d"].iter().map(|id| json!({ "id": id })).collect();
        let (ordered, ranked) = reorder(cards, &["c".into(), "zzz".into(), "a".into(), "c".into()]);
        let ids: Vec<&str> = ordered.iter().map(|card| card["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["c", "a", "b", "d"]);
        assert_eq!(ranked, 2);
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
        assert_eq!(
            run(account, "POST", "/app/v1/ai/search", br#"{"q":"calm","engine":"codex"}"#).unwrap(),
            Reply::ok(json!({ "cards": [], "next": null, "ranked": 0 }))
        );
    }

    fn futures_lite_block<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }
}
