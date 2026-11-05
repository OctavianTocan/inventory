//! Port of `lib/ai-tags.mjs`: AI tag normalization, relevance and curation.

use std::collections::{HashMap, HashSet};

use mm_js::{Value, text::{is_whitespace, len, to_lower, trim}};

use crate::render::access::{member, prototype_member, text};
use crate::{Error, Result};

fn redundant(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "YouTubeVideo" | "Video" => &["video"],
        "Audio" => &["audio", "audio file", "recording"],
        "Document" => &["pdf", "document", "file"],
        "WebPage" => &["web page", "webpage"],
        "SoftwareApplication" => &["app", "application", "mobile app", "mobile application"],
        "XPost" | "ThreadsPost" | "SubstackNote" | "RedditPost" | "BlueskyPost" | "MastodonPost" => &["tweet", "post"],
        _ => return None,
    })
}

/// Normalize tag identity across case, separators, whitespace and simple English plurals.
///
/// # Arguments
/// * `value` - Tag name; its stored spelling is not modified.
///
/// # Returns
/// Comparison key for card deduplication and library vocabulary matching.
pub fn tag_key(value: &str) -> String {
    let lowered = to_lower(trim(value));
    let mut spaced = String::with_capacity(lowered.len());
    let mut separator_run = false;
    for character in lowered.chars() {
        if character == '-' || character == '_' || is_whitespace(character) {
            separator_run = true;
            continue;
        }
        if separator_run {
            spaced.push(' ');
            separator_run = false;
        }
        spaced.push(character);
    }
    if separator_run {
        spaced.push(' ');
    }
    spaced
        .split(' ')
        .map(|word| {
            let units = len(word);
            if units > 4 && word.ends_with("ies") {
                format!("{}y", &word[..word.len() - 3])
            } else if ["ches", "shes", "xes", "zes", "sses"].iter().any(|suffix| word.ends_with(suffix)) {
                word[..word.len() - 2].to_string()
            } else if units > 3 && word.ends_with('s') && !["ss", "us", "is"].iter().any(|suffix| word.ends_with(suffix)) {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Keep subject labels while excluding control annotations and labels that repeat the card type.
///
/// # Arguments
/// * `name` - Candidate tag spelling.
/// * `kind` - Renderer card type.
///
/// # Returns
/// True when an AI tag adds useful subject information.
///
/// # Errors
/// `kind` names an `Object.prototype` member, which the JS lookup turns into a TypeError.
pub fn relevant_tag(name: &str, kind: &str) -> Result<bool> {
    let key = tag_key(name);
    if key.is_empty() || ["highlight", "private", "hidden", "nsfw", "website", "link", "page"].contains(&key.as_str()) {
        return Ok(false);
    }
    if key == "image" {
        return Ok(kind == "Image");
    }
    if key == "screenshot" {
        return Ok(kind == "Screenshot");
    }
    if prototype_member(kind).is_some() {
        return Err(Error::invalid("(redundant[type] ?? []).includes is not a function"));
    }
    Ok(!redundant(kind).unwrap_or(&[]).contains(&key.as_str()))
}

/// Deduplicate AI tags and reuse the first matching library spelling, preserving existing manual tags.
///
/// # Arguments
/// * `tags` - Candidate records with `name` and optional provider fields.
/// * `kind` - Card type used to remove redundant labels.
/// * `vocabulary` - Existing library names, most-used spelling first.
/// * `existing` - Names already on this card, never replaced by AI.
///
/// # Returns
/// At most ten distinct, relevant tag records with canonical spelling.
///
/// # Errors
/// A candidate is null or undefined, or `kind` names an `Object.prototype` member.
pub fn curate_tags(tags: &[Value], kind: &str, vocabulary: &[String], existing: &[String]) -> Result<Vec<Value>> {
    let mut spelling: HashMap<String, &str> = HashMap::new();
    for name in vocabulary {
        spelling.entry(tag_key(name)).or_insert(name);
    }
    let mut seen: HashSet<String> = existing.iter().map(|name| tag_key(name)).collect();
    let mut result = Vec::new();
    for tag in tags {
        let name = text(member(tag, "name")?);
        let key = tag_key(&name);
        if seen.contains(&key) || !relevant_tag(&name, kind)? {
            continue;
        }
        let name = match spelling.get(&key) {
            Some(known) => known.to_string(),
            None => trim(&tag.get("name").to_js_string()).to_string(),
        };
        let words = name.chars().fold((0usize, false), |(runs, inside), character| {
            let space = is_whitespace(character);
            (runs + usize::from(space && !inside), space)
        });
        if len(&name) > 40 || words.0 + 1 > 4 {
            continue;
        }
        seen.insert(key);
        let mut record = tag.clone();
        record.set("name", name);
        result.push(record);
        if result.len() == 10 {
            break;
        }
    }
    Ok(result)
}
