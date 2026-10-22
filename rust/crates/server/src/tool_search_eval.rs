//! `mymind search-eval`: how often search puts the card you meant first or in the top 10, by words alone, by meaning
//! alone and by both at several blends.
//!
//! Each line of the queries file is `{"q": "what you would type", "expect": "the card's title, link or id"}`;
//! `expect` may also be a list, and any of its cards counts. Blank lines and lines starting with `#` are skipped. The
//! database is opened read-only, Gemini embeds each query once, and `.env` supplies keys the process does not already
//! have.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use mm_core::host::Fetch;
use mm_core::search::{Hybrid, Search, SearchResult};
use mm_core::search_semantic::embeddings_enabled;
use mm_core::store::{GRID_HIDDEN_FLAGS, Store};
use mm_core::{Error, Result, Value};
use mm_js::{json, number};
use mm_sqlite::Sqlite;

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::node_path::resolve;

const USAGE: &str = "Usage: mymind search-eval <queries.jsonl> [--db path] [--boost 1,2,4] [--floor 0.5,0.6,0.7] [--take N]";

/// The blends `search-eval` compares: every boost with every floor, all with the same cap.
#[derive(Clone, Debug, PartialEq)]
pub struct Sweep {
    /// Weights of a card's similarity added to its word score.
    pub boosts: Vec<f64>,
    /// Least similarity for a card that matches no word of the query.
    pub floors: Vec<f64>,
    /// Most cards that join without a matching word.
    pub take: usize,
}

impl Default for Sweep {
    /// Boosts 1, 2 and 4, floors 0.5 to 0.7 in steps of 0.05, and the app's cap.
    fn default() -> Self {
        Self { boosts: vec![1.0, 2.0, 4.0], floors: vec![0.5, 0.55, 0.6, 0.65, 0.7], take: Hybrid::default().take }
    }
}

struct Config {
    queries: PathBuf,
    db: PathBuf,
    sweep: Sweep,
}

struct Known {
    id: String,
    title: String,
    url: String,
    embedded: bool,
}

struct Case {
    line: usize,
    q: String,
    ids: Vec<String>,
    embedded: bool,
}

#[derive(Clone, Copy)]
enum Mode {
    Words,
    Meaning,
    Both(Hybrid),
}

struct Outcome {
    rank: Option<usize>,
    similarity: Option<f64>,
    results: usize,
    ms: f64,
}

// A comma-separated list of finite numbers within `low..=high`.
fn numbers(raw: &str, flag: &str, low: f64, high: f64) -> Result<Vec<f64>> {
    raw.split(',')
        .map(|part| {
            let value = number::string_to_number(part.trim());
            if part.trim().is_empty() || !value.is_finite() || value < low || value > high {
                return Err(Error::internal(format!("{flag} takes numbers from {low} to {high}, separated by commas")));
            }
            Ok(value)
        })
        .collect()
}

fn options(args: &[String]) -> Result<Config> {
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let (mut queries, mut db, mut sweep) = (None, resolve(&cwd, "data/mymind.db"), Sweep::default());
    let mut at = 0;
    while at < args.len() {
        let next = args.get(at + 1).filter(|value| !value.is_empty());
        match (args[at].as_str(), next) {
            ("--db", Some(path)) => db = resolve(&cwd, path),
            ("--boost", Some(raw)) => sweep.boosts = numbers(raw, "--boost", 0.0, 100.0)?,
            ("--floor", Some(raw)) => sweep.floors = numbers(raw, "--floor", -1.0, 1.0)?,
            ("--take", Some(raw)) => {
                if !raw.bytes().all(|byte| byte.is_ascii_digit()) || !number::is_safe_integer(number::string_to_number(raw)) {
                    return Err(Error::internal("--take must be a nonnegative integer"));
                }
                sweep.take = number::string_to_number(raw) as usize;
            }
            (path, _) if !path.starts_with('-') && queries.is_none() => {
                queries = Some(resolve(&cwd, path));
                at += 1;
                continue;
            }
            _ => return Err(Error::internal(USAGE)),
        }
        at += 2;
    }
    Ok(Config { queries: queries.ok_or_else(|| Error::internal(USAGE))?, db, sweep })
}

fn known_cards(store: &Store) -> Result<Vec<Known>> {
    let rows = store.db().query(
        &format!(
            "SELECT c.id, json_extract(c.extras, '$.title') AS extra_title, json_extract(c.data, '$.title') AS title,
      json_extract(c.extras, '$.linkUrl') AS link, json_extract(c.data, '$.source.url') AS url,
      EXISTS (SELECT 1 FROM embeddings e WHERE e.card_id = c.id AND e.user_id = c.user_id) AS embedded
      FROM cards c WHERE c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0"
        ),
        &[store.default_user_id()?.into()],
    )?;
    Ok(rows
        .iter()
        .map(|row| Known {
            id: row.text("id").unwrap_or("").to_owned(),
            title: row.text("extra_title").or_else(|| row.text("title")).unwrap_or("").to_owned(),
            url: row.text("link").or_else(|| row.text("url")).unwrap_or("").to_owned(),
            embedded: row.number("embedded").unwrap_or(0.0) != 0.0,
        })
        .collect())
}

// A link without its scheme, `www.` and trailing slash, lower-cased.
fn plain_link(url: &str) -> String {
    let lower = url.trim().to_lowercase();
    let rest = lower.split_once("://").map_or(lower.as_str(), |(_, rest)| rest);
    rest.strip_prefix("www.").unwrap_or(rest).trim_end_matches('/').to_owned()
}

// The cards an `expect` entry names: an id, a link, an exact title, or the one title that contains it.
fn named<'a>(known: &'a [Known], wanted: &str) -> std::result::Result<Vec<&'a Known>, String> {
    let wanted = wanted.trim();
    if wanted.is_empty() {
        return Err("an empty expect".to_owned());
    }
    if let Some(card) = known.iter().find(|card| card.id == wanted) {
        return Ok(vec![card]);
    }
    if wanted.contains("://") || wanted.starts_with("www.") {
        let link = plain_link(wanted);
        let found: Vec<&Known> = known.iter().filter(|card| !card.url.is_empty() && plain_link(&card.url) == link).collect();
        return if found.is_empty() { Err(format!("no card links to {wanted}")) } else { Ok(found) };
    }
    let lower = wanted.to_lowercase();
    let exact: Vec<&Known> = known.iter().filter(|card| card.title.trim().to_lowercase() == lower).collect();
    if !exact.is_empty() {
        return Ok(exact);
    }
    let partial: Vec<&Known> = known.iter().filter(|card| card.title.to_lowercase().contains(&lower)).collect();
    match partial.as_slice() {
        [] => Err(format!("no card has the title, link or id \"{wanted}\"")),
        [card] => Ok(vec![card]),
        [first, second, ..] => Err(format!(
            "\"{wanted}\" is in {} titles, such as \"{}\" and \"{}\"; use the card's link or id",
            partial.len(),
            first.title,
            second.title
        )),
    }
}

// The queries file's cases, and a note for each line that names no card.
fn cases(text: &str, known: &[Known]) -> Result<(Vec<Case>, Vec<String>)> {
    let (mut out, mut skipped) = (Vec::new(), Vec::new());
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let invalid = || Error::internal(format!("line {line}: expected {{\"q\": \"...\", \"expect\": \"title, link or id\"}}"));
        let value = json::parse(trimmed).map_err(|_| invalid())?;
        let q = value.get("q").as_str().filter(|q| !q.trim().is_empty()).ok_or_else(invalid)?.to_owned();
        let wanted: Vec<String> = match value.get("expect") {
            Value::String(one) => vec![one.clone()],
            Value::Array(many) if !many.is_empty() && many.iter().all(Value::is_string) => many.iter().map(Value::to_js_string).collect(),
            _ => return Err(invalid()),
        };
        let mut cards: Vec<&Known> = Vec::new();
        for entry in &wanted {
            match named(known, entry) {
                Ok(found) => cards.extend(found),
                Err(problem) => skipped.push(format!("line {line}: {problem}")),
            }
        }
        if cards.is_empty() {
            continue;
        }
        out.push(Case { line, q, embedded: cards.iter().any(|card| card.embedded), ids: cards.into_iter().map(|card| card.id.clone()).collect() });
    }
    Ok((out, skipped))
}

fn label(mode: Mode) -> String {
    match mode {
        Mode::Words => "words (the app today)".to_owned(),
        Mode::Meaning => "meaning only".to_owned(),
        Mode::Both(tuning) => format!(
            "both, boost {}, floor {}{}",
            number::number_to_string(tuning.boost),
            number::number_to_string(tuning.floor),
            if tuning == Hybrid::default() { " *" } else { "" }
        ),
    }
}

async fn run_mode(search: &Search, mode: Mode, case: &Case) -> Result<Outcome> {
    let started = Instant::now();
    let found: SearchResult = match mode {
        Mode::Words => search.search(&case.q, &Value::Object(mm_js::Object::new())).await?,
        Mode::Meaning => search.meaning(&case.q).await?,
        Mode::Both(tuning) => search.hybrid(&case.q, &tuning).await?,
    };
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    let rank = found.matches.iter().position(|item| case.ids.contains(&item.id)).map(|at| at + 1);
    let similarity = matches!(mode, Mode::Meaning).then(|| found.matches.iter().filter(|item| case.ids.contains(&item.id)).map(|item| item.score).fold(f64::NAN, f64::max));
    Ok(Outcome { rank, similarity: similarity.filter(|score| !score.is_nan()), results: found.matches.len(), ms })
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 1 { values[middle] } else { (values[middle - 1] + values[middle]) / 2.0 }
}

fn count(outcomes: &[Outcome], within: usize) -> usize {
    outcomes.iter().filter(|outcome| outcome.rank.is_some_and(|rank| rank <= within)).count()
}

/// Run the queries through words, meaning and every blend in `sweep`, and describe how each ranked the cards meant.
///
/// # Arguments
/// * `store` - The library; nothing is written to it.
/// * `fetch` - Outbound HTTP for the query embeddings.
/// * `queries` - The queries file's text.
/// * `sweep` - The blends to compare.
///
/// # Returns
/// The report: the library's size, any lines that name no card, a table with one row per mode, the best blend and
/// each query's ranks.
///
/// # Errors
/// A line that is not a query object, a query search rejects, or the database's error.
pub async fn evaluate(store: Rc<Store>, fetch: Rc<dyn Fetch>, queries: &str, sweep: &Sweep) -> Result<String> {
    let known = known_cards(&store)?;
    let (cases, skipped) = cases(queries, &known)?;
    let offline = mm_core::runtime::env("OFFLINE").as_deref() == Some("1");
    let search = Search::new(Rc::clone(&store), Some(fetch), offline)?;
    let mut report = String::new();
    let embedded = known.iter().filter(|card| card.embedded).count();
    let _ = writeln!(report, "Library: {} cards, {embedded} with an embedding.", known.len());
    let _ = writeln!(report, "Queries: {}, {} lines skipped.", cases.len() + skipped.len(), skipped.len());
    for problem in &skipped {
        let _ = writeln!(report, "  {problem}");
    }
    if cases.is_empty() {
        let _ = writeln!(report, "No query names a card in this library.");
        return Ok(report);
    }
    let mut first = Vec::new();
    let mut failed = 0;
    for case in &cases {
        let outcome = run_mode(&search, Mode::Meaning, case).await?;
        failed += usize::from(outcome.results == 0);
        first.push(outcome.ms);
    }
    let _ = writeln!(report, "First search of each query, with its Gemini call: median {:.0} ms.", median(first));
    if failed > 0 {
        let _ = writeln!(report, "Meaning found nothing for {failed} queries: the Gemini call failed or the query has only field terms.");
    }
    let mut modes = vec![Mode::Words, Mode::Meaning];
    for &boost in &sweep.boosts {
        for &floor in &sweep.floors {
            modes.push(Mode::Both(Hybrid { boost, floor, take: sweep.take }));
        }
    }
    let mut results: Vec<(Mode, Vec<Outcome>)> = Vec::new();
    for mode in modes {
        let mut outcomes = Vec::new();
        for case in &cases {
            outcomes.push(run_mode(&search, mode, case).await?);
        }
        results.push((mode, outcomes));
    }
    let total = cases.len();
    let width = results.iter().map(|(mode, _)| label(*mode).len()).max().unwrap_or(0);
    let _ = writeln!(report, "\n{:width$}  {:>7}  {:>7}  {:>7}  {:>7}  {:>5}", "mode", "top 1", "top 10", "found", "results", "ms");
    for (mode, outcomes) in &results {
        let _ = writeln!(
            report,
            "{:width$}  {:>7}  {:>7}  {:>7}  {:>7}  {:>5.0}",
            label(*mode),
            format!("{}/{total}", count(outcomes, 1)),
            format!("{}/{total}", count(outcomes, 10)),
            format!("{}/{total}", count(outcomes, usize::MAX)),
            median(outcomes.iter().map(|outcome| outcome.results as f64).collect()),
            median(outcomes.iter().map(|outcome| outcome.ms).collect()),
        );
    }
    let _ = writeln!(report, "* the blend the app asks for with semantic=1. results and ms are medians over the queries.");
    let mut best: Option<(usize, (usize, usize, usize), f64)> = None;
    for (index, (mode, outcomes)) in results.iter().enumerate() {
        if !matches!(mode, Mode::Both(_)) {
            continue;
        }
        let hits = (count(outcomes, 1), count(outcomes, 10), count(outcomes, usize::MAX));
        let noise = median(outcomes.iter().map(|outcome| outcome.results as f64).collect());
        if best.as_ref().is_none_or(|(_, top, least)| hits > *top || (hits == *top && noise < *least)) {
            best = Some((index, hits, noise));
        }
    }
    let ranks = |mode_index: usize, case_index: usize| -> String {
        let outcome = &results[mode_index].1[case_index];
        match (outcome.rank, outcome.similarity) {
            (Some(rank), Some(similarity)) => format!("{rank} ({similarity:.2})"),
            (Some(rank), None) => rank.to_string(),
            (None, _) => "-".to_owned(),
        }
    };
    let Some((best, _, _)) = best else { return Ok(report) };
    if let Mode::Both(tuning) = results[best].0 {
        let _ = writeln!(
            report,
            "Best blend: boost {}, floor {}, at most {} cards by meaning alone.",
            number::number_to_string(tuning.boost),
            number::number_to_string(tuning.floor),
            tuning.take
        );
    }
    let _ = writeln!(report, "\nRank of the card you meant, - when it is not in the results (meaning's similarity in brackets):");
    let _ = writeln!(report, "{:>5}  {:>6}  {:>12}  {:>6}  query", "line", "words", "meaning", "best");
    for (index, case) in cases.iter().enumerate() {
        let note = if case.embedded { "" } else { "  (no embedding)" };
        let _ = writeln!(report, "{:>5}  {:>6}  {:>12}  {:>6}  {}{note}", case.line, ranks(0, index), ranks(1, index), ranks(best, index), case.q);
    }
    Ok(report)
}

async fn work(args: &[String]) -> Result<String> {
    let config = options(args)?;
    if !embeddings_enabled(mm_core::runtime::env("OFFLINE").as_deref() == Some("1")) {
        return Err(Error::internal("GEMINI_API_KEY and an allowed Gemini endpoint are required"));
    }
    let queries = std::fs::read_to_string(&config.queries).map_err(|error| Error::internal(format!("{}: {error}", config.queries.display())))?;
    let store = Rc::new(Store::new(Rc::new(Sqlite::open(&config.db, true)?))?);
    evaluate(store, Rc::new(NativeFetch::new()), &queries, &config.sweep).await
}

/// Compare search modes on a file of queries (`mymind search-eval`).
///
/// # Arguments
/// * `root` - The repository root. `.env` there fills keys the process has not set. The default database follows the
///   current directory.
/// * `args` - The arguments after the subcommand name: `<queries.jsonl> [--db path] [--boost list] [--floor list]
///   [--take N]`.
///
/// # Returns
/// The exit status: 0 after printing the report, or 1 after printing the error.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        return 0;
    }
    let file = match env_file::load(&env_file::path(root)) {
        Ok(Loaded::Vars(vars)) => vars,
        _ => HashMap::new(),
    };
    mm_core::runtime::set_env(env_file::merged(std::env::vars(), file));
    match work(args).await {
        Ok(report) => {
            print!("{report}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}
