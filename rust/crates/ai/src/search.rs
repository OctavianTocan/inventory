//! Search your mind with Claude Code or Codex: the engine reads what a search means, the library runs the searches it
//! wrote, and the engine ranks what they found and, for a question, answers it from those cards.

use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;

use ai_kit::login::AgentLogin;
use futures_util::future::join_all;
use serde_json::{Value, json};

use crate::Reply;

/// Most cards one AI search ranks.
const CANDIDATES: usize = 50;
/// Most cards each search of an understanding brings.
const PER_SEARCH: usize = 30;
/// Most cards a matching space brings.
const PER_SPACE: usize = 12;
/// Most searches an understanding keeps.
const SEARCHES: usize = 3;
/// Most tags and spaces an understanding names.
const MATCHES: usize = 5;
/// Most parts the likeliest reading of a search is split into.
const PARTS: usize = 6;
/// The kinds of part a reading is split into.
const PART_KINDS: [&str; 6] = ["type", "about", "time", "site", "tag", "other"];
/// Most of the person's tags, and of their spaces, the engine reads to understand a search.
const KNOWN: usize = 200;
/// Most cards an answer names as its sources.
const SOURCES: usize = 6;
/// Characters of a candidate's description and note the model reads.
const SNIPPET_CHARS: usize = 280;
/// Characters of a search query.
const QUERY_CHARS: usize = 500;
/// Characters of the plain-words reading of a search.
const LABEL_CHARS: usize = 120;
/// Characters of a part's label.
const PART_CHARS: usize = 60;
/// Characters of an answer.
const ANSWER_CHARS: usize = 1200;

/// The engine, model and effort a request asks for.
pub(crate) struct Engine<'a> {
    name: &'a str,
    model: Option<&'a str>,
    effort: Option<&'a str>,
}

fn setting<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty())
}

/// The query and engine of an AI search request, or the 400 that refuses it.
fn request(args: &Value) -> Result<(String, Engine<'_>), Reply> {
    let Some(query) = setting(args, "q") else { return Err(Reply::problem(400, "missing-query")) };
    let name = setting(args, "engine").unwrap_or("claude-code");
    if crate::engine_binary(name).is_none() {
        return Err(Reply::problem(400, "Unknown AI engine."));
    }
    Ok((clip(query, QUERY_CHARS), Engine { name, model: setting(args, "model"), effort: setting(args, "effort") }))
}

/// Runs the engine once and reads its answer, which the kit has checked against `schema`.
async fn ask(login: &AgentLogin, engine: &Engine<'_>, system: &str, prompt: String, schema: Value) -> anyhow::Result<Value> {
    let mut request = json!({ "engine": engine.name, "system": system, "prompt": prompt, "schema": schema });
    for (key, value) in [("model", engine.model), ("effort", engine.effort)] {
        if let Some(value) = value {
            request[key] = json!(value);
        }
    }
    // An AI search ends with its request, so nothing cancels it.
    let (_cancel, cancelled) = tokio::sync::watch::channel(false);
    let (text, _) = ai_kit::process::run_agent(request, login, cancelled, Arc::new(|_| {})).await?;
    Ok(serde_json::from_str(&text)?)
}

fn clip(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match collapsed.char_indices().nth(max) {
        Some((at, _)) => format!("{}…", &collapsed[..at]),
        None => collapsed,
    }
}

fn percent(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

/// Whether `id` can go into a library path as it is.
fn plain_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// The phone's date, `YYYY-MM-DD`, when the request carries one.
fn today(args: &Value) -> Option<&str> {
    setting(args, "today").filter(|day| {
        day.len() == 10 && day.bytes().enumerate().all(|(at, byte)| if at == 4 || at == 7 { byte == b'-' } else { byte.is_ascii_digit() })
    })
}

/// One part of the likeliest reading of a search, such as its card type or its time range, with the query text that
/// applies it, so the app can drop a part and search again without the engine.
#[derive(Clone, Debug, PartialEq)]
struct Part {
    /// One of [`PART_KINDS`].
    kind: String,
    label: String,
    query: String,
}

impl Part {
    fn to_json(&self) -> Value {
        json!({ "kind": self.kind, "label": self.label, "query": self.query })
    }

    /// A part within bounds, of a known kind, or `None` without a label or query.
    fn read(value: &Value) -> Option<Self> {
        let text = |key: &str, max: usize| clip(value.get(key).and_then(Value::as_str).unwrap_or(""), max);
        let kind = value.get("kind").and_then(Value::as_str).filter(|kind| PART_KINDS.contains(kind)).unwrap_or("other");
        let (label, query) = (text("label", PART_CHARS), text("query", QUERY_CHARS));
        (!label.is_empty() && !query.is_empty()).then(|| Self { kind: kind.to_owned(), label, query })
    }
}

/// The parts of a reading, read with [`Part::read`], at most [`PARTS`].
fn parts(value: Option<&Value>) -> Vec<Part> {
    value.and_then(Value::as_array).into_iter().flatten().filter_map(Part::read).take(PARTS).collect()
}

/// What an engine made of a search: a plain-words reading and its parts, the searches the library runs for it, the
/// person's own tags and spaces that match it, and whether it asks a question.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Understood {
    label: String,
    parts: Vec<Part>,
    /// The first is the parts joined, when there are parts.
    searches: Vec<String>,
    tags: Vec<String>,
    /// Ids and names.
    spaces: Vec<(String, String)>,
    question: bool,
}

impl Understood {
    fn to_json(&self) -> Value {
        json!({
            "label": self.label,
            "parts": self.parts.iter().map(Part::to_json).collect::<Vec<_>>(),
            "searches": self.searches,
            "tags": self.tags,
            "spaces": self.spaces.iter().map(|(id, name)| json!({ "id": id, "name": name })).collect::<Vec<_>>(),
            "question": self.question,
        })
    }

    /// An understanding the app sends back with its search, kept within the bounds the engine's own has.
    fn from_json(value: &Value) -> Option<Self> {
        let value = value.as_object()?;
        let texts = |key: &str, max: usize| -> Vec<String> {
            value.get(key).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
                .map(|text| clip(text, max)).filter(|text| !text.is_empty()).take(if key == "searches" { SEARCHES } else { MATCHES }).collect()
        };
        let spaces = value.get("spaces").and_then(Value::as_array).into_iter().flatten()
            .filter_map(|space| Some((space.get("id")?.as_str()?.to_owned(), clip(space.get("name")?.as_str()?, LABEL_CHARS))))
            .filter(|(id, _)| plain_id(id))
            .take(MATCHES)
            .collect();
        Some(Self {
            label: clip(value.get("label").and_then(Value::as_str).unwrap_or(""), LABEL_CHARS),
            parts: parts(value.get("parts")),
            searches: texts("searches", QUERY_CHARS),
            tags: texts("tags", LABEL_CHARS),
            spaces,
            question: value.get("question").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    /// The engine's answer, keeping only the person's own tags and spaces and searches within bounds, with the parts
    /// joined as the first search.
    fn checked(answer: &Value, known: &Known) -> Self {
        let texts = |key: &str| answer.get(key).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str);
        let parts = parts(answer.get("parts"));
        let joined = clip(&parts.iter().map(|part| part.query.as_str()).collect::<Vec<_>>().join(" "), QUERY_CHARS);
        let mut searches: Vec<String> = Vec::new();
        let written = texts("searches").map(|search| clip(search, QUERY_CHARS));
        for search in std::iter::once(joined).chain(written).filter(|search| !search.is_empty()) {
            if !searches.contains(&search) && searches.len() < SEARCHES {
                searches.push(search);
            }
        }
        let mut tags: Vec<String> = Vec::new();
        for name in texts("tags") {
            if let Some(tag) = known.tags.iter().find(|tag| tag.eq_ignore_ascii_case(name.trim().trim_start_matches('#')))
                && !tags.contains(tag)
                && tags.len() < MATCHES
            {
                tags.push(tag.clone());
            }
        }
        let mut spaces: Vec<(String, String)> = Vec::new();
        for id in texts("spaces") {
            if let Some(space) = known.spaces.iter().find(|(known, _)| known == id.trim())
                && !spaces.contains(space)
                && spaces.len() < MATCHES
            {
                spaces.push(space.clone());
            }
        }
        Self {
            label: clip(answer.get("label").and_then(Value::as_str).unwrap_or(""), LABEL_CHARS),
            parts,
            searches,
            tags,
            spaces,
            question: answer.get("question").and_then(Value::as_bool).unwrap_or(false),
        }
    }
}

/// The person's tags, most used first, and spaces, as ids and names.
#[derive(Debug, Default)]
struct Known {
    tags: Vec<String>,
    spaces: Vec<(String, String)>,
}

/// Reads the person's tags and spaces; either is empty when the library does not answer for it.
async fn known<F, Fut>(library: &F) -> Known
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let (tags, spaces) = futures_util::join!(library("/app/v1/tags".into()), library("/app/v1/spaces".into()));
    let list = |found: anyhow::Result<Value>, key: &str| found.ok().and_then(|found| found.get(key).and_then(Value::as_array).cloned()).unwrap_or_default();
    let mut tags = list(tags, "tags");
    tags.sort_by(|a, b| b.get("count").and_then(Value::as_f64).unwrap_or(0.0).total_cmp(&a.get("count").and_then(Value::as_f64).unwrap_or(0.0)));
    Known {
        tags: tags.iter().filter_map(|tag| tag.get("name").and_then(Value::as_str)).map(str::to_owned).take(KNOWN).collect(),
        spaces: list(spaces, "spaces")
            .iter()
            .filter_map(|space| Some((space.get("id")?.as_str()?.to_owned(), space.get("name")?.as_str()?.to_owned())))
            .filter(|(id, _)| plain_id(id))
            .take(KNOWN)
            .collect(),
    }
}

fn understanding_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "label": { "type": "string" },
            "parts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "enum": PART_KINDS },
                        "label": { "type": "string" },
                        "query": { "type": "string" }
                    },
                    "required": ["kind", "label", "query"],
                    "additionalProperties": false
                }
            },
            "searches": { "type": "array", "items": { "type": "string" } },
            "tags": { "type": "array", "items": { "type": "string" } },
            "spaces": { "type": "array", "items": { "type": "string" } },
            "question": { "type": "boolean" }
        },
        "required": ["label", "parts", "searches", "tags", "spaces", "question"],
        "additionalProperties": false
    })
}

const UNDERSTAND_SYSTEM: &str = "You read one person's search of their own saved library of links, articles, notes, \
images, videos, products and posts, and write the searches that find what they mean.

The library's search syntax: free words match by words and by meaning. Fields narrow the search: \
type:image, type:video, type:article, type:note, type:web-page, type:product, type:x-post, type:youtube-video, \
type:instagram-reel, type:screenshot, type:document, type:app, type:repository, type:snippet; \
tag:\"name\" for one of their tags; site:example.com; after:YYYY-MM-DD, before:YYYY-MM-DD and \
created:[YYYY-MM-DD TO YYYY-MM-DD] for when it was saved; action:read, action:watch, action:listen, action:play. \
A leading - excludes a term, OR joins alternatives and parentheses group them.

Split the likeliest reading into parts, each one thing a chip could show: its kind (type, about, time, site, tag or \
other), a short label (Articles, walking in the woods, March to May) and the query text that applies it (type:article, \
walking woods, created:[2026-03-01 TO 2026-05-31]). Keep the free words to what the thing is about and drop words \
such as find, show, that, the one, I saved. Use a field only when the search asks for it; use tag: only with tags \
from their list. The parts joined make the first search; in searches add up to two other readings, if the search \
can mean something else. In label say in a few plain words what you are looking for, as a person would. In tags and \
spaces name theirs that match what they are after: tag names from their list and space ids from their list, or none. \
Set question when the search asks something to be answered (who, what, when, which, how many, why), not only things \
to be shown.";

/// Has the engine read a search, with the person's tags and spaces at hand.
async fn understand_query<F, Fut>(login: &AgentLogin, engine: &Engine<'_>, query: &str, today: Option<&str>, library: &F) -> anyhow::Result<Understood>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let known = known(library).await;
    let mut prompt = String::new();
    if let Some(today) = today {
        prompt.push_str(&format!("Today is {today}.\n\n"));
    }
    prompt.push_str(&format!("Search: {query}\n\n"));
    prompt.push_str(&format!("Their tags: {}\n\n", if known.tags.is_empty() { "none".to_owned() } else { known.tags.join(", ") }));
    prompt.push_str("Their spaces, id then name:\n");
    if known.spaces.is_empty() {
        prompt.push_str("none\n");
    }
    for (id, name) in &known.spaces {
        prompt.push_str(&format!("{id} {}\n", clip(name, LABEL_CHARS)));
    }
    let answer = ask(login, engine, UNDERSTAND_SYSTEM, prompt, understanding_schema()).await?;
    Ok(Understood::checked(&answer, &known))
}

/// Answers `POST /app/v1/ai/search/understand` (`{q, engine, model, effort, today}`) with `{understood}`.
pub(crate) async fn understand<F, Fut>(login: &AgentLogin, args: &Value, library: &F) -> Reply
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let (query, engine) = match request(args) {
        Ok(found) => found,
        Err(refused) => return refused,
    };
    match understand_query(login, &engine, &query, today(args), library).await {
        Ok(understood) => Reply::ok(json!({ "understood": understood.to_json() })),
        Err(error) => Reply::failed(&error),
    }
}

/// The cards the searches of `understood`, the query itself and the matching spaces find, taken in turn from each so
/// every one is heard, without repeats and at most [`CANDIDATES`].
///
/// # Errors
/// The last failure when the library answered none of them.
async fn gather<F, Fut>(library: &F, query: &str, understood: &Understood) -> anyhow::Result<Vec<Value>>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let mut searches: Vec<&str> = vec![query];
    for search in &understood.searches {
        if !searches.contains(&search.as_str()) {
            searches.push(search);
        }
    }
    let mut targets: Vec<String> = searches.iter().map(|search| format!("/app/v1/search?q={}&semantic=1&limit={PER_SEARCH}", percent(search))).collect();
    targets.extend(understood.spaces.iter().map(|(id, _)| format!("/app/v1/spaces/{id}/cards?limit={PER_SPACE}")));
    let mut lists = Vec::new();
    let mut failure = None;
    for found in join_all(targets.into_iter().map(library)).await {
        match found {
            Ok(found) => lists.push(found.get("cards").and_then(Value::as_array).cloned().unwrap_or_default()),
            Err(error) => failure = Some(error),
        }
    }
    if lists.is_empty() {
        return Err(failure.unwrap_or_else(|| anyhow::anyhow!("The library search did not answer.")));
    }
    let mut seen = HashSet::new();
    let mut cards = Vec::new();
    for at in 0..lists.iter().map(Vec::len).max().unwrap_or(0) {
        for card in lists.iter().filter_map(|list| list.get(at)) {
            let Some(id) = card.get("id").and_then(Value::as_str) else { continue };
            if cards.len() < CANDIDATES && seen.insert(id.to_owned()) {
                cards.push(card.clone());
            }
        }
    }
    Ok(cards)
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

/// What the model reads of one card: its id, the day it was saved and the words that say what it is.
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
    let saved = card.get("created").and_then(Value::as_str).and_then(|created| created.get(..10)).map(str::to_owned);
    let mut item = json!({ "id": id });
    for (key, value) in [
        ("type", text("type", 40)),
        ("saved", saved),
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

/// The answer schema of a ranking: the matching ids, best first, and the answer to a question with its sources.
fn ranking_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ids": { "type": "array", "items": { "type": "string" } },
            "answer": { "type": "string" },
            "sources": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["ids", "answer", "sources"],
        "additionalProperties": false
    })
}

const RANK_SYSTEM: &str = "You search one person's saved library of links, notes, images and videos. Rank the \
items by how well each one matches what the search means, not by shared words: a search for \"calm places\" matches \
a lake photo with no words in common. Answer with the ids of the items that match, best match first, and leave out \
the ones that do not match at all. Use only ids from the list.

When the search is a question, also answer it in answer: two to four plain sentences drawn from these items alone, \
speaking to the person about their own things, and list in sources the ids of the items the answer draws from. When \
the items do not answer it, say so in one sentence. When the search is not a question, leave answer empty and \
sources empty.";

/// The answer to a question and the cards it draws from, or null.
fn answer_json(answer: &Value, question: bool, cards: &[Value]) -> Value {
    let text = clip(answer.get("answer").and_then(Value::as_str).unwrap_or(""), ANSWER_CHARS);
    if !question || text.is_empty() {
        return Value::Null;
    }
    let known: HashSet<&str> = cards.iter().filter_map(|card| card.get("id").and_then(Value::as_str)).collect();
    let mut sources: Vec<&str> = Vec::new();
    for id in answer.get("sources").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
        if known.contains(id) && !sources.contains(&id) && sources.len() < SOURCES {
            sources.push(id);
        }
    }
    json!({ "text": text, "ids": sources })
}

/// Answers `POST /app/v1/ai/search` (`{q, engine, model, effort, understood}`): the cards the understood searches find,
/// the ones the engine ranks first, and the answer when the understanding says the search is a question. Without
/// `understood` it ranks the query's own matches, as before understanding existed.
pub(crate) async fn search<F, Fut>(login: &AgentLogin, args: &Value, library: &F) -> Reply
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let (query, engine) = match request(args) {
        Ok(found) => found,
        Err(refused) => return refused,
    };
    let understood = args.get("understood").and_then(Understood::from_json).unwrap_or_default();
    let cards = match gather(library, &query, &understood).await {
        Ok(cards) => cards,
        Err(error) => return Reply::failed(&error),
    };
    if cards.is_empty() {
        return Reply::ok(json!({ "cards": [], "next": null, "ranked": 0, "understood": understood.to_json(), "answer": null }));
    }
    let items: Vec<String> = cards.iter().filter_map(candidate).map(|item| item.to_string()).collect();
    let mut prompt = format!("Search: {query}\n");
    if !understood.label.is_empty() {
        prompt.push_str(&format!("What they are looking for: {}\n", understood.label));
    }
    prompt.push_str(if understood.question { "The search is a question.\n" } else { "The search is not a question.\n" });
    prompt.push_str(&format!("\nItems, one JSON object a line:\n{}", items.join("\n")));
    let answer = match ask(login, &engine, RANK_SYSTEM, prompt, ranking_schema()).await {
        Ok(answer) => answer,
        Err(error) => return Reply::failed(&error),
    };
    let ids: Vec<String> = answer.get("ids").and_then(Value::as_array).into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect();
    let said = answer_json(&answer, understood.question, &cards);
    let (cards, ranked) = reorder(cards, &ids);
    Reply::ok(json!({ "cards": cards, "next": null, "ranked": ranked, "understood": understood.to_json(), "answer": said }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }

    fn cards(ids: &[&str]) -> Value {
        json!({ "cards": ids.iter().map(|id| json!({ "id": id })).collect::<Vec<_>>(), "next": null })
    }

    #[test]
    fn candidates_carry_only_what_says_what_a_card_is() {
        let card = json!({
            "id": "c1", "type": "WebPage", "title": "  A\n lake ", "description": "x".repeat(400),
            "domain": "example.com", "tags": [{ "name": "calm" }, "water"], "url": "https://example.com/?token=1",
            "blob": { "path": "a.png" }, "created": "2026-10-01T08:00:00Z",
            "prose": { "type": "doc", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Still" }, { "type": "text", "text": "water." }] }] }
        });
        let item = candidate(&card).unwrap();
        assert_eq!(item["title"], "A lake");
        assert_eq!(item["text"], "Still water.");
        assert_eq!(item["saved"], "2026-10-01");
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
    fn an_understanding_names_only_the_persons_own_tags_and_spaces() {
        let known = Known { tags: vec!["Recipes".into(), "travel".into()], spaces: vec![("s1".into(), "Kitchen".into())] };
        let answer = json!({
            "label": "Cooking   videos", "question": true,
            "searches": ["type:video cooking", "type:video cooking", " ", "a", "b", "c"],
            "tags": ["#recipes", "made-up", "recipes"], "spaces": ["s1", "s9"]
        });
        let understood = Understood::checked(&answer, &known);
        assert_eq!(understood.label, "Cooking videos");
        assert_eq!(understood.searches, ["type:video cooking", "a", "b"]);
        assert_eq!(understood.tags, ["Recipes"]);
        assert_eq!(understood.spaces, [("s1".to_owned(), "Kitchen".to_owned())]);
        assert!(understood.question);
    }

    #[test]
    fn the_parts_of_a_reading_make_its_first_search() {
        let answer = json!({
            "label": "The walking essay", "question": false, "tags": [], "spaces": [],
            "parts": [
                { "kind": "type", "label": "Articles", "query": "type:article" },
                { "kind": "mood", "label": "walking in the woods", "query": "walking woods" },
                { "kind": "time", "label": "", "query": "after:2026-03-01" },
                { "kind": "time", "label": "March to May", "query": "created:[2026-03-01 TO 2026-05-31]" }
            ],
            "searches": ["walking", "type:article walking woods created:[2026-03-01 TO 2026-05-31]"]
        });
        let understood = Understood::checked(&answer, &Known::default());
        let kinds: Vec<&str> = understood.parts.iter().map(|part| part.kind.as_str()).collect();
        assert_eq!(kinds, ["type", "other", "time"]);
        assert_eq!(understood.searches, ["type:article walking woods created:[2026-03-01 TO 2026-05-31]", "walking"]);
    }

    #[test]
    fn an_understanding_comes_back_from_the_app_as_it_left() {
        let understood = Understood {
            label: "Calm places".into(),
            parts: vec![Part { kind: "about".into(), label: "calm lakes".into(), query: "calm lake".into() }],
            searches: vec!["calm lake".into()],
            tags: vec!["travel".into()],
            spaces: vec![("s1".into(), "Trips".into())],
            question: false,
        };
        assert_eq!(Understood::from_json(&understood.to_json()), Some(understood));
        let sent = json!({ "searches": ["a", "b", "c", "d"], "spaces": [{ "id": "../tags", "name": "x" }, { "id": "s2", "name": "Ok" }] });
        let back = Understood::from_json(&sent).unwrap();
        assert_eq!(back.searches.len(), SEARCHES);
        assert_eq!(back.spaces, [("s2".to_owned(), "Ok".to_owned())]);
        assert_eq!(Understood::from_json(&json!("nope")), None);
    }

    #[test]
    fn gathering_takes_from_every_search_in_turn_without_repeats() {
        let understood = Understood { searches: vec!["first".into(), "calm".into()], spaces: vec![("s1".into(), "Trips".into())], ..Default::default() };
        let library = |target: String| async move {
            if target.contains("q=calm&") {
                Ok(cards(&["a", "b", "c"]))
            } else if target.contains("q=first&") {
                Ok(cards(&["x", "a"]))
            } else if target == format!("/app/v1/spaces/s1/cards?limit={PER_SPACE}") {
                Ok(cards(&["s"]))
            } else {
                anyhow::bail!("unexpected {target}")
            }
        };
        let found = block(gather(&library, "calm", &understood)).unwrap();
        let ids: Vec<&str> = found.iter().map(|card| card["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["a", "x", "s", "b", "c"]);
    }

    #[test]
    fn gathering_fails_only_when_no_search_answers() {
        let understood = Understood { searches: vec!["broken (".into()], ..Default::default() };
        let some = |target: String| async move { if target.contains("broken") { anyhow::bail!("bad query") } else { Ok(cards(&["a"])) } };
        assert_eq!(block(gather(&some, "calm", &understood)).unwrap().len(), 1);
        let none = |_: String| async move { anyhow::bail!("down") };
        assert_eq!(block(gather(&none, "calm", &understood)).unwrap_err().to_string(), "down");
    }

    #[test]
    fn an_answer_is_kept_only_for_a_question_and_cites_only_found_cards() {
        let found: Vec<Value> = ["a", "b"].iter().map(|id| json!({ "id": id })).collect();
        let answer = json!({ "ids": ["a"], "answer": "  You saved two. ", "sources": ["b", "zzz", "b"] });
        assert_eq!(answer_json(&answer, true, &found), json!({ "text": "You saved two.", "ids": ["b"] }));
        assert_eq!(answer_json(&answer, false, &found), Value::Null);
        assert_eq!(answer_json(&json!({ "answer": "" }), true, &found), Value::Null);
    }

    #[test]
    fn the_phone_date_is_read_only_in_its_one_shape() {
        assert_eq!(today(&json!({ "today": "2026-10-28" })), Some("2026-10-28"));
        assert_eq!(today(&json!({ "today": "28/10/2026" })), None);
        assert_eq!(today(&json!({ "today": "2026-10-28T10:00" })), None);
        assert_eq!(today(&json!({})), None);
    }
}
