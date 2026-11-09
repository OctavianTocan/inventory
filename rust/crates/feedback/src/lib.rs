//! The phone-facing feedback routes (docs/feedback.md, Wire contract). They check a send and pass it to the feedback
//! runner (`feedback-runner/`, a TypeScript Worker reached through a service binding), which owns the jobs. Every
//! refusal answers 403 with a sentence for the person, never 401, because the app signs out on a 401.
//!
//! The caller supplies the I/O as closures, so the checks run natively in tests: whether the person is signed in to
//! an engine, that engine's current models, and the forward to the runner.

use std::future::Future;

use serde_json::{Value, json};

/// The efforts ai-kit's pickers offer, `none` to `max`.
const EFFORTS: &[&str] = &["none", "minimal", "low", "medium", "high", "xhigh", "max"];
/// The longest feedback a person may send, in characters.
const MAX_TEXT: usize = 4000;

/// Who may send feedback (`FEEDBACK_OPEN`): only the owner in phase 1, everyone signed in in phase 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Open {
    Owner,
    Everyone,
}

impl Open {
    /// Reads `FEEDBACK_OPEN`: `everyone` opens it; anything else, or nothing, keeps it to the owner.
    pub fn from_var(value: Option<&str>) -> Self {
        if value.map(str::trim) == Some("everyone") { Self::Everyone } else { Self::Owner }
    }
}

/// The signed-in caller.
#[derive(Debug, Clone)]
pub struct Caller<'a> {
    /// The verified email from the sign-in gate.
    pub email: &'a str,
    /// `OWNER_EMAIL`, when set.
    pub owner: Option<&'a str>,
    pub open: Open,
}

impl Caller<'_> {
    fn is_owner(&self) -> bool {
        self.owner.is_some_and(|owner| !owner.trim().is_empty() && owner.trim().eq_ignore_ascii_case(self.email.trim()))
    }
}

/// An answer: HTTP status and JSON body.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub status: u16,
    pub body: Value,
}

impl Reply {
    /// A refusal with a sentence for the person.
    pub fn problem(status: u16, message: &str) -> Self {
        Self { status, body: json!({ "error": message }) }
    }
}

/// A request for the feedback runner: method, path (`/feedback...`) and JSON body.
#[derive(Debug, Clone, PartialEq)]
pub struct Forward {
    pub method: &'static str,
    pub path: String,
    pub body: Option<Value>,
}

/// Whether `path` is one of the feedback routes.
pub fn serves(path: &str) -> bool {
    path == "/app/v1/feedback" || path.starts_with("/app/v1/feedback/")
}

/// The engine's name in the runner (`claude` or `codex`) and in sentences.
fn engine(value: &str) -> Option<(&'static str, &'static str)> {
    match value {
        "claude" | "claude-code" => Some(("claude", "Claude Code")),
        "codex" => Some(("codex", "Codex")),
        _ => None,
    }
}

/// Answers a feedback route.
///
/// Routes: `POST /app/v1/feedback` (`{text, engine, model, effort, client_id}`), `GET /app/v1/feedback/{id}`, and for
/// the owner `GET /app/v1/feedback` and `POST /app/v1/feedback/pause` (`{paused}`).
///
/// # Arguments
/// * `caller` - The signed-in caller, the owner's email and the phase.
/// * `method` - HTTP method.
/// * `path` - The request path.
/// * `body` - The request body.
/// * `signed_in` - Whether the caller is signed in to an engine (`claude` or `codex`).
/// * `models` - The ids of an engine's current models for the caller.
/// * `forward` - Sends a request to the runner and gives back its answer.
///
/// # Returns
/// The answer, or `None` when the path is not a feedback route.
pub async fn answer<S, SF, M, MF, F, FF>(caller: &Caller<'_>, method: &str, path: &str, body: &[u8], signed_in: S, models: M, forward: F) -> Option<Reply>
where
    S: FnOnce(&'static str) -> SF,
    SF: Future<Output = anyhow::Result<bool>>,
    M: FnOnce(&'static str) -> MF,
    MF: Future<Output = anyhow::Result<Vec<String>>>,
    F: FnOnce(Forward) -> FF,
    FF: Future<Output = anyhow::Result<Reply>>,
{
    if !serves(path) {
        return None;
    }
    let rest = path.trim_start_matches("/app/v1/feedback");
    let plan = match (method, rest) {
        ("POST", "" | "/") => match check_send(caller, body) {
            Ok((send, binary, name)) => {
                match signed_in(binary).await {
                    Ok(true) => {}
                    Ok(false) => return Some(Reply::problem(403, &format!("Sign in to {name} first."))),
                    Err(_) => return Some(Reply::problem(403, &format!("Couldn't check your {name} sign-in. Try again."))),
                }
                let model = send["model"].as_str().unwrap_or_default().to_owned();
                match models(binary).await {
                    Ok(list) if list.iter().any(|id| *id == model) => {}
                    Ok(_) => return Some(Reply::problem(403, "That model isn't available any more.")),
                    Err(_) => return Some(Reply::problem(403, &format!("Couldn't load {name}'s models. Try again."))),
                }
                Forward { method: "POST", path: "/feedback".into(), body: Some(send) }
            }
            Err(reply) => return Some(reply),
        },
        ("GET", "" | "/") if caller.is_owner() => Forward { method: "GET", path: "/feedback".into(), body: None },
        ("POST", "/pause") if caller.is_owner() => {
            let parsed: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
            let Some(paused) = parsed.get("paused").and_then(Value::as_bool) else {
                return Some(Reply::problem(400, "Send {\"paused\": true} or false."));
            };
            Forward { method: "POST", path: "/feedback/pause".into(), body: Some(json!({ "paused": paused })) }
        }
        ("GET", "" | "/") | ("POST", "/pause") => return Some(Reply::problem(403, "Only Octavian can see this.")),
        ("GET", id) if is_id(id.trim_start_matches('/')) => Forward { method: "GET", path: format!("/feedback{id}"), body: None },
        _ => return Some(Reply::problem(404, "Not found.")),
    };
    Some(forward(plan).await.unwrap_or_else(|_| Reply::problem(502, "Feedback isn't reachable right now. Try again later.")))
}

/// A feedback id: 12 lowercase letters and digits.
fn is_id(id: &str) -> bool {
    id.len() == 12 && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// The send's own checks (phase, body), before the engine checks that need I/O. Returns the body to forward with the
/// engine named as the runner names it, the engine, and its name for sentences.
fn check_send(caller: &Caller<'_>, body: &[u8]) -> Result<(Value, &'static str, &'static str), Reply> {
    if caller.open == Open::Owner && !caller.is_owner() {
        return Err(Reply::problem(403, "Only Octavian can send feedback for now."));
    }
    let bad = |message: &str| Reply::problem(400, message);
    let parsed: Value = serde_json::from_slice(body).map_err(|_| bad("Send a JSON body."))?;
    let text = parsed["text"].as_str().map(str::trim).unwrap_or_default();
    let length = text.chars().count();
    if length == 0 || length > MAX_TEXT {
        return Err(bad("Write between 1 and 4000 characters."));
    }
    let (binary, name) = parsed["engine"].as_str().and_then(engine).ok_or_else(|| bad("Pick Claude Code or Codex."))?;
    let model = parsed["model"].as_str().filter(|model| !model.is_empty() && model.len() <= 100).ok_or_else(|| bad("Pick a model."))?;
    let effort = parsed["effort"].as_str().filter(|effort| EFFORTS.contains(effort)).ok_or_else(|| bad("Pick an effort."))?;
    let client_id = parsed["client_id"]
        .as_str()
        .filter(|id| (8..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
        .ok_or_else(|| bad("Missing client_id."))?;
    Ok((json!({ "text": text, "engine": binary, "model": model, "effort": effort, "client_id": client_id }), binary, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const OCTAVIAN: &str = "octavian@example.com";

    fn caller(email: &str, open: Open) -> Caller<'_> {
        Caller { email, owner: Some(OCTAVIAN), open }
    }

    fn send(text: &str) -> Vec<u8> {
        json!({ "text": text, "engine": "claude-code", "model": "opus", "effort": "medium", "client_id": "client-123" }).to_string().into_bytes()
    }

    /// Runs one request with the given sign-in and model answers; returns the reply and what reached the runner.
    fn run(caller: &Caller<'_>, method: &str, path: &str, body: &[u8], signed: bool, models: &[&str]) -> (Option<Reply>, Option<Forward>) {
        let sent = RefCell::new(None);
        let models: Vec<String> = models.iter().map(|m| m.to_string()).collect();
        let reply = tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(answer(
            caller,
            method,
            path,
            body,
            |_| async move { Ok(signed) },
            |_| async move { Ok(models) },
            |forward| {
                *sent.borrow_mut() = Some(forward);
                async { Ok(Reply { status: 202, body: json!({ "id": "abcdef123456", "state": "queued" }) }) }
            },
        ));
        (reply, sent.into_inner())
    }

    fn status(result: (Option<Reply>, Option<Forward>)) -> (u16, String) {
        let reply = result.0.unwrap();
        (reply.status, reply.body["error"].as_str().unwrap_or_default().to_owned())
    }

    #[test]
    fn a_good_send_reaches_the_runner_with_its_client_id_unchanged() {
        let (reply, sent) = run(&caller(OCTAVIAN, Open::Owner), "POST", "/app/v1/feedback", &send(" Keep it open "), true, &["opus"]);
        assert_eq!(reply.unwrap().status, 202);
        let sent = sent.unwrap();
        assert_eq!(sent.path, "/feedback");
        assert_eq!(sent.body.unwrap(), json!({ "text": "Keep it open", "engine": "claude", "model": "opus", "effort": "medium", "client_id": "client-123" }));
    }

    #[test]
    fn every_refusal_has_its_status_and_sentence_and_none_is_401() {
        let octavian = caller(OCTAVIAN, Open::Owner);
        let ana = caller("ana@example.com", Open::Everyone);
        let cases = [
            (run(&caller("ana@example.com", Open::Owner), "POST", "/app/v1/feedback", &send("x"), true, &["opus"]), 403, "Only Octavian can send feedback for now."),
            (run(&octavian, "POST", "/app/v1/feedback", b"nope", true, &["opus"]), 400, "Send a JSON body."),
            (run(&octavian, "POST", "/app/v1/feedback", &send(""), true, &["opus"]), 400, "Write between 1 and 4000 characters."),
            (run(&octavian, "POST", "/app/v1/feedback", &send(&"a".repeat(4001)), true, &["opus"]), 400, "Write between 1 and 4000 characters."),
            (run(&ana, "POST", "/app/v1/feedback", &send("x"), false, &["opus"]), 403, "Sign in to Claude Code first."),
            (run(&ana, "POST", "/app/v1/feedback", &send("x"), true, &["sonnet"]), 403, "That model isn't available any more."),
            (run(&ana, "GET", "/app/v1/feedback", b"", true, &[]), 403, "Only Octavian can see this."),
            (run(&ana, "POST", "/app/v1/feedback/pause", br#"{"paused":true}"#, true, &[]), 403, "Only Octavian can see this."),
            (run(&ana, "GET", "/app/v1/feedback/NOT-AN-ID", b"", true, &[]), 404, "Not found."),
        ];
        for (result, code, sentence) in cases {
            assert_eq!(status(result), (code, sentence.to_owned()));
        }
        let mut bad_effort: Value = serde_json::from_slice(&send("x")).unwrap();
        bad_effort["effort"] = json!("turbo");
        assert_eq!(status(run(&octavian, "POST", "/app/v1/feedback", bad_effort.to_string().as_bytes(), true, &["opus"])).0, 400);
    }

    #[test]
    fn text_length_counts_characters_not_bytes() {
        let (reply, _) = run(&caller(OCTAVIAN, Open::Owner), "POST", "/app/v1/feedback", &send(&"é".repeat(4000)), true, &["opus"]);
        assert_eq!(reply.unwrap().status, 202);
    }

    #[test]
    fn owner_routes_and_status_are_forwarded() {
        let octavian = caller("Octavian@Example.com", Open::Owner);
        assert_eq!(run(&octavian, "GET", "/app/v1/feedback", b"", true, &[]).1.unwrap().path, "/feedback");
        let pause = run(&octavian, "POST", "/app/v1/feedback/pause", br#"{"paused":true}"#, true, &[]).1.unwrap();
        assert_eq!((pause.path.as_str(), pause.body.unwrap()), ("/feedback/pause", json!({ "paused": true })));
        let ana = caller("ana@example.com", Open::Everyone);
        assert_eq!(run(&ana, "GET", "/app/v1/feedback/abcdef123456", b"", true, &[]).1.unwrap().path, "/feedback/abcdef123456");
    }

    #[test]
    fn other_paths_are_not_ours() {
        assert!(!serves("/app/v1/feedbacks"));
        assert!(serves("/app/v1/feedback"));
        assert!(run(&caller(OCTAVIAN, Open::Owner), "GET", "/app/v1/ai/models", b"", true, &[]).0.is_none());
    }

    #[test]
    fn the_phase_comes_from_feedback_open() {
        assert_eq!(Open::from_var(None), Open::Owner);
        assert_eq!(Open::from_var(Some("owner")), Open::Owner);
        assert_eq!(Open::from_var(Some("everyone")), Open::Everyone);
    }
}
