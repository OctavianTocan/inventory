//! Tag picking asks OpenAI's Decisions API one predicate per candidate tag, keeps tags at 0.6 or above, and shows the
//! API a card's picture at any size when the card is understood from its picture.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, Once};

use async_trait::async_trait;
use futures_executor::block_on;
use mm_core::ai::{Ai, AiImage, AiItem, SpaceOption, SuggestedSpace};
use mm_core::host::Fetch;
use mm_core::http::{Body, Headers, Request, Response};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::{Result, Value};
use mm_js::json;

static LOCK: Mutex<()> = Mutex::new(());

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

fn setup(vars: &[(&str, &str)]) -> std::sync::MutexGuard<'static, ()> {
    static ONCE: Once = Once::new();
    let guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    ONCE.call_once(|| runtime::install(&TestRuntime));
    runtime::set_env(vars.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect::<HashMap<_, _>>());
    guard
}

// Answers each predicate with the next scripted probability (or a refusal for `None`), a choice question with
// `choice`, and keeps every request.
struct Decisions {
    probabilities: Vec<Option<f64>>,
    choice: Option<(String, f64)>,
    requests: RefCell<Vec<(String, String, Value)>>,
}

impl Decisions {
    fn new(probabilities: &[Option<f64>]) -> Self {
        Self { probabilities: probabilities.to_vec(), choice: None, requests: RefCell::default() }
    }

    fn choosing(value: &str, p: f64) -> Self {
        Self { choice: Some((value.to_owned(), p)), ..Self::new(&[]) }
    }
}

#[async_trait(?Send)]
impl Fetch for Decisions {
    async fn fetch(&self, mut request: Request) -> Result<Response> {
        let body = json::parse(&String::from_utf8(request.take_body().bytes().await?).unwrap()).unwrap();
        let auth = request.headers.get("authorization").unwrap_or_default().to_owned();
        let answers: Vec<String> = body
            .get("questions")
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, question)| {
                let name = question.get("name").to_js_string();
                if question.get("type").as_str() == Some("choice") {
                    let (value, p) = self.choice.clone().unwrap();
                    return format!(r#"{{"type":"choice","name":"{name}","choice":"{value}","probabilities":[{{"value":"{value}","probability":{p}}}],"confidence":{p}}}"#);
                }
                match self.probabilities.get(index).copied().flatten() {
                    Some(p) => format!(r#"{{"type":"predicate","name":"{name}","probability":{p}}}"#),
                    None => format!(r#"{{"type":"refusal","name":"{name}"}}"#),
                }
            })
            .collect();
        self.requests.borrow_mut().push((request.url.as_str().to_owned(), auth, body));
        let text = format!(r#"{{"model":"gpt-6-luna","answers":[{}],"usage":{{"input_tokens":10}}}}"#, answers.join(","));
        Ok(Response::new(200, Headers::from([("content-type", "application/json".to_owned())]), Body::Bytes(text.into_bytes())))
    }
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn article() -> AiItem {
    AiItem {
        title: Some("Weeknight lemon spaghetti".into()),
        kind: Some("Article".into()),
        text: Some("A twenty minute pasta with lemon, garlic and parmesan. ".repeat(5)),
        url: Some("https://food.example/lemon-spaghetti".into()),
        domain: Some("food.example".into()),
        image: None,
    }
}

// A valid PNG header of the given size; the picture's pixels are never decoded.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());
    data.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'I', b'D', b'A', b'T']);
    data
}

fn content_types(body: &Value) -> Vec<String> {
    body.get("input").get("0").get("content").as_array().unwrap().iter().map(|part| part.get("type").to_js_string()).collect()
}

#[test]
fn picks_tags_with_one_predicate_each() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai"), ("OPENAI_BASE_URL", "https://openai.example/v1/")]);
    let fake = Decisions::new(&[Some(0.94), Some(0.59), Some(0.6), None, Some(0.83)]);
    let picked = block_on(Ai::new(&fake).pick_tags(&article(), &names(&["recipes", "typography", "Pasta", "dinner", "cooking", " recipes "]))).unwrap();
    let got: Vec<(&str, f64)> = picked.iter().map(|tag| (tag.name.as_str(), tag.p)).collect();
    assert_eq!(got, vec![("recipes", 0.94), ("cooking", 0.83), ("Pasta", 0.6)]);

    let requests = fake.requests.borrow();
    assert_eq!(requests.len(), 1);
    let (url, auth, body) = &requests[0];
    assert_eq!(url, "https://openai.example/v1/decisions");
    assert_eq!(auth, "Bearer test-openai");
    assert_eq!(body.get("model").as_str(), Some("gpt-6-luna"));
    let questions = body.get("questions").as_array().unwrap();
    assert_eq!(questions.len(), 5, "the duplicate \" recipes \" is asked once");
    assert_eq!(questions[2].get("type").as_str(), Some("predicate"));
    assert_eq!(questions[2].get("name").as_str(), Some("t2"));
    assert!(questions[2].get("instructions").to_js_string().ends_with("under the tag \"Pasta\"?"));
    let item = body.get("input").get("0").get("content").get("1").get("data").get("item");
    assert_eq!(item.get("title").as_str(), Some("Weeknight lemon spaghetti"));
    assert_eq!(item.get("source").as_str(), Some("food.example"));
    assert_eq!(content_types(body), ["input_text", "input_json"], "a card read from its words is sent without its picture");
}

#[test]
fn shows_pictures_of_any_size() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    for (width, height) in [(40, 30), (4000, 3000), (12_000, 200)] {
        let fake = Decisions::new(&[Some(0.9)]);
        let item = AiItem { kind: Some("Image".into()), image: Some(AiImage { mime: "image/png".into(), data: png(width, height) }), ..AiItem::default() };
        let picked = block_on(Ai::new(&fake).pick_tags(&item, &names(&["sunset"]))).unwrap();
        assert_eq!(picked.len(), 1, "{width}x{height}");
        let requests = fake.requests.borrow();
        let (url, _, body) = &requests[0];
        assert_eq!(url, "https://api.openai.com/v1/decisions");
        assert_eq!(content_types(body), ["input_text", "input_json", "input_text", "input_image"], "{width}x{height}");
        let image = body.get("input").get("0").get("content").get("3").get("image_url").to_js_string();
        assert!(image.starts_with("data:image/png;base64,"), "{width}x{height}");
    }
}

#[test]
fn asks_nothing_without_candidates_or_content() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    let fake = Decisions::new(&[Some(0.9)]);
    assert!(block_on(Ai::new(&fake).pick_tags(&article(), &names(&["", "  "]))).unwrap().is_empty());
    assert!(block_on(Ai::new(&fake).pick_tags(&AiItem::default(), &names(&["cats"]))).unwrap().is_empty());
    assert!(fake.requests.borrow().is_empty());
}

#[test]
fn needs_the_openai_key() {
    let _guard = setup(&[]);
    let fake = Decisions::new(&[Some(0.9)]);
    let error = block_on(Ai::new(&fake).pick_tags(&article(), &names(&["cats"]))).unwrap_err();
    assert!(error.message.contains("OPENAI_API_KEY"), "{}", error.message);
    assert!(fake.requests.borrow().is_empty());
}

#[test]
fn splits_more_than_400_candidates() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    let fake = Decisions::new(&[Some(0.7); 400]);
    let candidates: Vec<String> = (0..401).map(|index| format!("tag{index}")).collect();
    let picked = block_on(Ai::new(&fake).pick_tags(&article(), &candidates)).unwrap();
    assert_eq!(picked.len(), 10);
    let sizes: Vec<usize> = fake.requests.borrow().iter().map(|(_, _, body)| body.get("questions").as_array().unwrap().len()).collect();
    assert_eq!(sizes, [400, 1]);
}

fn spaces() -> Vec<SpaceOption> {
    vec![
        SpaceOption { id: "sp-food".into(), name: "Food".into(), examples: vec!["Miso soup".into(), "Bread".into(), "Tacos".into(), "Extra".into()] },
        SpaceOption { id: "sp-work".into(), name: "Work".into(), examples: Vec::new() },
    ]
}

#[test]
fn suggests_the_space_chosen_from_every_space_and_none() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    let fake = Decisions::choosing("s0", 0.82);
    let suggested = block_on(Ai::new(&fake).suggest_space(&article(), &spaces())).unwrap();
    assert_eq!(suggested, Some(SuggestedSpace { id: "sp-food".into(), p: 0.82 }));
    let requests = fake.requests.borrow();
    let question = requests[0].2.get("questions").get("0").clone();
    assert_eq!(question.get("type").as_str(), Some("choice"));
    let choices: Vec<(String, String)> = question
        .get("choices")
        .as_array()
        .unwrap()
        .iter()
        .map(|choice| (choice.get("value").to_js_string(), choice.get("description").to_js_string()))
        .collect();
    assert_eq!(
        choices,
        [
            ("s0".to_owned(), "The space \"Food\", which holds cards like \"Miso soup\", \"Bread\", \"Tacos\"".to_owned()),
            ("s1".to_owned(), "The space \"Work\"".to_owned()),
            ("none".to_owned(), "None of these spaces clearly fits the item".to_owned()),
        ]
    );
}

#[test]
fn suggests_nothing_when_unsure_or_none_fits() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    for (value, p) in [("none", 0.9), ("s1", 0.49), ("s9", 0.9)] {
        let fake = Decisions::choosing(value, p);
        assert_eq!(block_on(Ai::new(&fake).suggest_space(&article(), &spaces())).unwrap(), None, "{value} {p}");
    }
    let fake = Decisions::choosing("s0", 0.9);
    assert_eq!(block_on(Ai::new(&fake).suggest_space(&article(), &[])).unwrap(), None);
    assert!(fake.requests.borrow().is_empty(), "no spaces asks nothing");
}

#[test]
fn offers_at_most_254_spaces() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    let many: Vec<SpaceOption> = (0..300).map(|index| SpaceOption { id: format!("sp{index}"), name: format!("Space {index}"), examples: Vec::new() }).collect();
    let fake = Decisions::choosing("s253", 0.7);
    let suggested = block_on(Ai::new(&fake).suggest_space(&article(), &many)).unwrap();
    assert_eq!(suggested.map(|space| space.id), Some("sp253".to_owned()));
    assert_eq!(fake.requests.borrow()[0].2.get("questions").get("0").get("choices").as_array().unwrap().len(), 255);
}

#[test]
fn asks_whether_a_new_card_repeats_each_earlier_one() {
    let _guard = setup(&[("OPENAI_API_KEY", "test-openai")]);
    let fake = Decisions::new(&[Some(0.97), None, Some(0.2)]);
    let earlier = vec![article(), AiItem { title: Some("Other".into()), ..AiItem::default() }, AiItem { text: Some("Third".into()), ..AiItem::default() }];
    let same = block_on(Ai::new(&fake).same_as(&article(), &earlier)).unwrap();
    assert_eq!(same, [0.97, 0.0, 0.2], "a refusal counts as not the same");
    let requests = fake.requests.borrow();
    let body = &requests[0].2;
    let names: Vec<String> = body.get("questions").as_array().unwrap().iter().map(|question| question.get("name").to_js_string()).collect();
    assert_eq!(names, ["e0", "e1", "e2"]);
    let data = body.get("input").get("0").get("content").get("1").get("data");
    assert_eq!(data.get("new").get("title").as_str(), Some("Weeknight lemon spaghetti"));
    assert_eq!(data.get("earlier").get("e1").get("title").as_str(), Some("Other"));
    assert!(block_on(Ai::new(&fake).same_as(&article(), &[])).unwrap().is_empty());
}
