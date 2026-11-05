//! `Ai::transcribe`: the multipart request OpenAI's speech-to-text reads, and what its answers become.

use std::cell::RefCell;
use std::collections::HashMap;

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::ai::{Ai, Service};
use mm_core::host::Fetch;
use mm_core::http::{Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};

struct TestRuntime;

impl Runtime for TestRuntime {
    fn spawn(&self, task: Task) {
        block_on(task);
    }

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::ready(()))
    }

    fn log(&self, _line: &str) {}
}

struct Fake {
    answers: RefCell<Vec<(u16, &'static str)>>,
    seen: RefCell<Vec<(String, Headers, Vec<u8>)>>,
}

impl Fake {
    fn new(answers: Vec<(u16, &'static str)>) -> Self {
        Self { answers: RefCell::new(answers), seen: RefCell::default() }
    }
}

#[async_trait(?Send)]
impl Fetch for Fake {
    async fn fetch(&self, mut request: Request) -> mm_core::Result<Response> {
        let body = request.take_body().bytes().await?;
        self.seen.borrow_mut().push((request.url.as_str().to_owned(), request.headers.clone(), body));
        let (status, text) = self.answers.borrow_mut().remove(0);
        Ok(Response::new(status, Headers::from([("content-type", "application/json")]), text))
    }
}

fn setup(key: Option<&str>) {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(|| runtime::install(&TestRuntime));
    let mut vars = HashMap::new();
    if let Some(key) = key {
        vars.insert("OPENAI_API_KEY".to_owned(), key.to_owned());
    }
    runtime::set_env(vars);
}

#[test]
fn transcription_requests_and_answers() {
    // One test, because the environment is shared by the whole process.
    setup(Some("openai-secret"));
    let fake = Fake::new(vec![(200, r#"{"text":"  Hello there. ","languages":[{"code":"en"}],"usage":{"type":"duration","seconds":4}}"#)]);
    let heard = block_on(Ai::new(&fake).transcribe(b"OggS-audio", "audio.ogg")).expect("transcript");
    assert_eq!(heard.text, "Hello there.");
    assert_eq!(heard.language.as_deref(), Some("en"));

    let seen = fake.seen.borrow();
    let (url, headers, body) = &seen[0];
    assert_eq!(url, "https://api.openai.com/v1/audio/transcriptions");
    assert_eq!(headers.get("authorization"), Some("Bearer openai-secret"));
    let boundary = headers.get("content-type").and_then(|value| value.strip_prefix("multipart/form-data; boundary=")).expect("boundary").to_owned();
    let body = String::from_utf8_lossy(body);
    let model = body.find("name=\"model\"\r\n\r\ngpt-transcribe\r\n").expect("model field");
    let file = body.find("name=\"file\"; filename=\"audio.ogg\"").expect("file field");
    assert!(model < file, "the file goes last");
    assert!(body.contains("\r\n\r\nOggS-audio\r\n"));
    assert!(body.ends_with(&format!("\r\n--{boundary}--\r\n")));
    drop(seen);

    let retried = Fake::new(vec![(503, "{}"), (200, r#"{"text":"","languages":[]}"#)]);
    let silent = block_on(Ai::new(&retried).transcribe(b"x", "audio.ogg")).expect("retried transcript");
    assert_eq!((silent.text.as_str(), silent.language), ("", None));
    assert_eq!(retried.seen.borrow().len(), 2);

    let refused = Fake::new(vec![(401, r#"{"error":"bad key openai-secret"}"#)]);
    let error = block_on(Ai::new(&refused).transcribe(b"x", "audio.ogg")).expect_err("refused");
    assert_eq!((error.service, error.status), (Some(Service::OpenAi), Some(401)));
    assert!(!error.message.contains("openai-secret"), "the key never reaches an error: {}", error.message);

    setup(None);
    let unset = Fake::new(Vec::new());
    let error = block_on(Ai::new(&unset).transcribe(b"x", "audio.ogg")).expect_err("no key");
    assert!(error.message.starts_with("OPENAI_API_KEY is not set"));
    assert!(unset.seen.borrow().is_empty());
}
