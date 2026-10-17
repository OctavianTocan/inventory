//! `.env` files as Node reads them (`process.loadEnvFile`, `util.parseEnv`).

use std::collections::HashMap;
use std::path::Path;

fn trim(text: &str) -> &str {
    text.trim_matches([' ', '\t', '\n'])
}

// The text after the next newline, or nothing when there is none.
fn after_line(text: &str) -> Option<&str> {
    text.find('\n').map(|newline| &text[newline + 1..])
}

/// The variables an env file defines (`util.parseEnv`); a later definition of a name replaces an earlier one.
///
/// # Arguments
/// * `input` - The file's text.
pub fn parse(input: &str) -> HashMap<String, String> {
    let lines = input.replace('\r', "");
    let mut store = HashMap::new();
    let mut content = trim(&lines);
    while !content.is_empty() {
        if content.starts_with(['\n', '#']) {
            content = after_line(content).unwrap_or("");
            continue;
        }
        let Some(split) = content.find(['=', '\n']).filter(|&index| content.as_bytes()[index] == b'=') else {
            match after_line(content) {
                Some(rest) => {
                    content = trim(rest);
                    continue;
                }
                None => break,
            }
        };
        let mut key = trim(&content[..split]);
        content = &content[split + 1..];
        if content.is_empty() || content.starts_with('\n') {
            store.insert(key.to_owned(), String::new());
            continue;
        }
        content = trim(content);
        if key.is_empty() {
            continue;
        }
        if let Some(rest) = key.strip_prefix("export ") {
            key = trim(rest);
        }
        let (line, rest) = content.find('\n').map_or((content, ""), |newline| (&content[..newline], &content[newline + 1..]));
        if let Some(quote) = content.chars().next().filter(|first| ['"', '\'', '`'].contains(first)) {
            match content[1..].find(quote).map(|index| index + 1) {
                Some(close) => {
                    let value = &content[1..close];
                    store.insert(key.to_owned(), if quote == '"' { value.replace("\\n", "\n") } else { value.to_owned() });
                    content = content[close + 1..].find('\n').map_or("", |newline| &content[close + 1 + newline + 1..]);
                }
                // An unclosed quote keeps the whole line, quote and comment included.
                None => {
                    store.insert(key.to_owned(), line.to_owned());
                    content = trim(rest);
                }
            }
            continue;
        }
        let value = line.find('#').map_or(line, |hash| &line[..hash]);
        store.insert(key.to_owned(), trim(value).to_owned());
        content = trim(rest);
    }
    store
}

/// What reading an env file found.
#[derive(Debug)]
pub enum Loaded {
    /// The file's variables.
    Vars(HashMap<String, String>),
    /// There is no such file.
    Missing,
}

/// Read and parse an env file, as `process.loadEnvFile(path)` reads it.
///
/// # Arguments
/// * `path` - The file.
///
/// # Errors
/// Any read failure other than a missing file.
pub fn load(path: &Path) -> std::io::Result<Loaded> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Loaded::Vars(parse(&String::from_utf8_lossy(&bytes)))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Loaded::Missing),
        Err(error) => Err(error),
    }
}

/// The process environment with an env file's variables added; variables already set keep their value, as
/// `process.loadEnvFile` leaves them.
///
/// # Arguments
/// * `environment` - The process environment.
/// * `file` - The env file's variables.
pub fn merged(environment: impl IntoIterator<Item = (String, String)>, file: HashMap<String, String>) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = environment.into_iter().collect();
    for (name, value) in file {
        vars.entry(name).or_insert(value);
    }
    vars
}
