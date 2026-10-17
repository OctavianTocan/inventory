//! `mymind try-ai`: the port of `tools/try-ai.mjs`, which runs the AI features on a few recorded cards and prints
//! what they return.

use std::collections::HashMap;
use std::path::Path;

use mm_core::Value;
use mm_core::ai::{Ai, AiImage, AiItem, EmbedKind, TagSource, VocabularyEntry, cosine};
use mm_js::{json, number, text};

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::parse_args::{Kind, Opt, parse};

const USAGE: &str = "Usage: mymind try-ai [--cards N] [--vocab N] [--query \"text\"]...";

struct Cover {
    mime: &'static str,
    data: Vec<u8>,
    file: String,
}

struct Card {
    data: Value,
    image: Option<Cover>,
}

// `String(value ?? '')`.
fn text_of(value: &Value) -> String {
    match value {
        Value::Undefined | Value::Null => String::new(),
        other => other.to_js_string(),
    }
}

// `decodeURIComponent`.
fn decode_component(input: &str) -> Result<String, String> {
    let malformed = || "URI malformed".to_owned();
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            let hex = bytes.get(at + 1..at + 3).and_then(|pair| std::str::from_utf8(pair).ok()).filter(|pair| pair.bytes().all(|byte| byte.is_ascii_hexdigit()));
            out.push(u8::from_str_radix(hex.ok_or_else(malformed)?, 16).map_err(|_| malformed())?);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(out).map_err(|_| malformed())
}

// The cover image on disk for a recorded card, preferring formats Jev and Gemini embeddings accept.
fn find_image(root: &Path, html: &str) -> Result<Option<Cover>, String> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = html[at..].find("/media/") {
        let start = at + offset;
        let body = start + "/media/".len();
        let run = html[body..].find(|c: char| c == '"' || text::is_whitespace(c)).unwrap_or(html.len() - body);
        if run == 0 {
            at = start + 1;
            continue;
        }
        at = body + run;
        let url = decode_component(html[start..at].replace("&amp;", "&").split('?').next().unwrap_or(""))?;
        if root.join("capture").join(url.trim_start_matches('/')).exists() {
            found.push(url);
        }
    }
    let is_photo = |url: &&String| {
        let lower = url.to_lowercase();
        lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png")
    };
    let Some(pick) = found.iter().find(is_photo).or_else(|| found.first()) else {
        return Ok(None);
    };
    let extension = pick.rsplit('.').next().unwrap_or("").to_lowercase();
    let mime = match extension.as_str() {
        "jpeg" | "jpg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => return Ok(None),
    };
    let path = root.join("capture").join(pick.trim_start_matches('/'));
    let data = std::fs::read(&path).map_err(|error| error.to_string())?;
    let file = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(Some(Cover { mime, data, file }))
}

// Tag names by usage, merging spellings that differ only in case.
fn library_vocabulary(cards: &[Card], limit: f64) -> Vec<String> {
    struct Entry {
        total: usize,
        spellings: Vec<(String, usize)>,
    }
    let mut order: Vec<String> = Vec::new();
    let mut counts: HashMap<String, Entry> = HashMap::new();
    for card in cards {
        let Value::Array(tags) = card.data.get("tags") else { continue };
        for tag in tags {
            let name = tag.get("name");
            if !name.truthy() || name.as_str() == Some("highlight") {
                continue;
            }
            let name = name.to_js_string();
            let key = name.to_lowercase();
            if !counts.contains_key(&key) {
                order.push(key.clone());
                counts.insert(key.clone(), Entry { total: 0, spellings: Vec::new() });
            }
            let entry = counts.get_mut(&key).expect("inserted above");
            entry.total += 1;
            match entry.spellings.iter_mut().find(|(spelling, _)| *spelling == name) {
                Some((_, count)) => *count += 1,
                None => entry.spellings.push((name, 1)),
            }
        }
    }
    let mut entries: Vec<&Entry> = order.iter().map(|key| &counts[key]).collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.total));
    entries
        .into_iter()
        .take(limit as usize)
        .map(|entry| {
            let mut spellings: Vec<&(String, usize)> = entry.spellings.iter().collect();
            spellings.sort_by_key(|spelling| std::cmp::Reverse(spelling.1));
            spellings[0].0.clone()
        })
        .collect()
}

fn has_tags(card: &Card) -> bool {
    match card.data.get("tags") {
        Value::Array(tags) => !tags.is_empty(),
        Value::String(text) => !text.is_empty(),
        _ => false,
    }
}

// Evenly spaced picks (`spread`).
fn spread<'a>(list: &[&'a Card], n: f64) -> Vec<&'a Card> {
    if n < 1.0 {
        return Vec::new();
    }
    let step = ((list.len() as f64 / n).floor()).max(1.0);
    list.iter().enumerate().filter(|(index, _)| (*index as f64) % step == 0.0).map(|(_, card)| *card).take(n as usize).collect()
}

// About 60% image cards, the first a JPEG or PNG one so Jev's image path runs too.
fn pick_cards(cards: &[Card], count: f64) -> Vec<&Card> {
    let with_image: Vec<&Card> = cards.iter().filter(|card| card.image.is_some()).collect();
    let mut jpeg: Vec<&Card> = with_image.iter().copied().filter(|card| card.image.as_ref().is_some_and(|image| image.mime != "image/webp")).collect();
    jpeg.sort_by(|a, b| b.image.as_ref().map_or(0, |image| image.data.len()).cmp(&a.image.as_ref().map_or(0, |image| image.data.len())));
    let webp: Vec<&Card> = with_image.iter().copied().filter(|card| card.image.as_ref().is_some_and(|image| image.mime == "image/webp") && has_tags(card)).collect();
    let text_only: Vec<&Card> = cards.iter().filter(|card| card.image.is_none() && text::len(&text_of(card.data.get("description"))) > 40).collect();
    let image_count = (with_image.len() as f64).min((count * 0.6).ceil());
    let mut images = spread(&jpeg, image_count.min(1.0));
    images.extend(spread(&webp, (image_count - 1.0).max(0.0)));
    let rest = spread(&text_only, count - images.len() as f64);
    images.extend(rest);
    images
}

fn load_cards(root: &Path) -> Result<Vec<Card>, String> {
    let bytes = std::fs::read(root.join("capture").join("cards.json")).map_err(|error| error.to_string())?;
    let recorded = json::parse(&String::from_utf8_lossy(&bytes)).map_err(|error| error.message().to_owned())?;
    let Value::Array(entries) = recorded else {
        return Err("recorded.map is not a function".to_owned());
    };
    entries
        .iter()
        .map(|entry| {
            let html = text_of(entry.get("html"));
            let data = json::parse(&text_of(entry.get("json"))).map_err(|error| error.message().to_owned())?;
            let image = find_image(root, &html)?;
            Ok(Card { data, image })
        })
        .collect()
}

fn to_item(card: &Card) -> AiItem {
    AiItem {
        title: Some(text_of(card.data.get("title"))),
        kind: None,
        text: Some(text_of(card.data.get("description"))),
        url: Some(text_of(card.data.get("source").get("url"))),
        domain: Some(text_of(card.data.get("domain"))),
        image: card.image.as_ref().map(|image| AiImage { mime: image.mime.to_owned(), data: image.data.clone() }),
    }
}

fn norm(vector: &[f32]) -> f64 {
    vector.iter().map(|&value| f64::from(value) * f64::from(value)).sum::<f64>().sqrt()
}

fn integer(raw: &str) -> Option<f64> {
    let value = number::string_to_number(raw);
    (value.is_finite() && value.fract() == 0.0).then_some(value)
}

fn clip_units(title: &str, units: usize) -> String {
    String::from_utf16_lossy(&title.encode_utf16().take(units).collect::<Vec<_>>())
}

async fn work(root: &Path, args: &[String]) -> Result<(), String> {
    let declared = [Opt { name: "cards", kind: Kind::Text }, Opt { name: "vocab", kind: Kind::Text }, Opt { name: "query", kind: Kind::Text }];
    let parsed = parse(args, &declared, false).map_err(|message| message.split_once("]: ").map_or(message.clone(), |(_, rest)| rest.to_owned()))?;
    let count = integer(parsed.text("cards").unwrap_or("5"));
    let vocab_size = integer(parsed.text("vocab").unwrap_or("400"));
    let (Some(count), Some(vocab_size)) = (count.filter(|count| *count >= 1.0), vocab_size.filter(|size| *size >= 0.0)) else {
        return Err(USAGE.to_owned());
    };
    let cards = load_cards(root)?;
    let vocabulary = library_vocabulary(&cards, vocab_size);
    let sample = pick_cards(&cards, count);
    println!("library: {} cards, vocabulary of {} tags, analyzing {}", cards.len(), vocabulary.len(), sample.len());
    let entries: Vec<VocabularyEntry> = vocabulary.iter().map(VocabularyEntry::from).collect();
    let fetch = NativeFetch::new();
    let ai = Ai::new(&fetch);
    let mut analyzed: Vec<(&Card, AiItem, Vec<f32>)> = Vec::new();
    for card in sample {
        let item = to_item(card);
        let title = item.title.clone().unwrap_or_default();
        println!("\n{}", if title.is_empty() { "(untitled)" } else { &title });
        let domain = item.domain.clone().unwrap_or_default();
        let cover = card.image.as_ref().map_or_else(|| "none".to_owned(), |image| format!("{} ({})", image.file, image.mime));
        println!("  type: {}  domain: {}  image: {cover}", card.data.get("type").to_js_string(), if domain.is_empty() { "-" } else { &domain });
        let recorded = match card.data.get("tags") {
            Value::Array(tags) => tags.iter().map(|tag| text_of(tag.get("name"))).collect::<Vec<_>>().join(", "),
            _ => String::new(),
        };
        println!("  recorded tags: {}", if recorded.is_empty() { "-" } else { &recorded });
        let result = ai.analyze(&item, &entries).await.map_err(|error| error.to_string())?;
        println!("  summary: {}", result.summary);
        let tags: Vec<String> = result
            .tags
            .iter()
            .map(|tag| {
                let source = match tag.source {
                    TagSource::Vocabulary => "vocabulary",
                    TagSource::Chat => "chat",
                };
                format!("{} p={} [{source}]", tag.name, tag.p.map_or_else(|| "n/a".to_owned(), |p| number::to_fixed(p, 2)))
            })
            .collect();
        println!("  tags: {}", if tags.is_empty() { "(none)".to_owned() } else { tags.join("; ") });
        println!("  embedding: {} dims, norm {}", result.embedding.len(), number::to_fixed(norm(&result.embedding), 4));
        analyzed.push((card, item, result.embedding));
    }
    let given = parsed.texts("query");
    let queries: Vec<String> = if given.is_empty() {
        analyzed.iter().filter(|(card, ..)| has_tags(card)).take(2).map(|(card, ..)| card.data.get("tags").at(0).get("name").to_js_string()).collect()
    } else {
        given.iter().map(|query| (*query).to_owned()).collect()
    };
    for query in queries {
        let vector = ai.embed(&AiItem { text: Some(query.clone()), ..AiItem::default() }, EmbedKind::Query).await.map_err(|error| error.to_string())?;
        println!("\nquery \"{query}\"");
        let mut scored: Vec<(String, f64)> = analyzed.iter().map(|(_, item, embedding)| (item.title.clone().unwrap_or_default(), cosine(&vector, embedding))).collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        for (title, score) in scored {
            let shown = if title.is_empty() { "(untitled)".to_owned() } else { title };
            println!("  {}  {}", number::to_fixed(score, 3), clip_units(&shown, 70));
        }
    }
    Ok(())
}

/// Run the AI features on a few recorded cards (`main` of `tools/try-ai.mjs`).
///
/// # Arguments
/// * `root` - The repository root: `.env` (without overriding the process environment) and `capture/`.
/// * `args` - The arguments after the subcommand name: `[--cards N] [--vocab N] [--query text]...`.
///
/// # Returns
/// The exit status: 0, or 1 after printing `error: <message>`.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    let file = match env_file::load(&env_file::path(root)) {
        Ok(Loaded::Vars(vars)) => vars,
        _ => HashMap::new(),
    };
    mm_core::runtime::set_env(env_file::merged(std::env::vars(), file));
    match work(root, args).await {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("error: {message}");
            1
        }
    }
}
