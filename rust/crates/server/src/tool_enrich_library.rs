//! `mymind enrich-library`: estimate or run a resumable summary, tag and embedding backfill.
//!
//! The local form writes the database in `--db`. The hosted form calls the owner-only `/ai/backfill` route and does
//! not replace rows. A summary or tag a person wrote is left as it is. `.env` supplies keys the process does not
//! already have.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use mm_core::backfill::{self, Estimate, MediaImages, Report, HOSTED_BATCH};
use mm_core::enrichment::ai_configured;
use mm_core::host::Fetch;
use mm_core::http::Request;
use mm_core::store::Store;
use mm_core::store_write::Writer;
use mm_core::{Error, Result, Value};
use mm_js::{json, number};
use mm_sqlite::Sqlite;

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::local_media::LocalMedia;
use crate::node_path::resolve;

const USAGE: &str = "\
Usage: mymind enrich-library [--db path] [--limit N] [--run]
       mymind enrich-library --hosted URL [--limit N] [--run] [--cookie VALUE]";
const FAILED: &str = "Enrichment backfill failed; completed cards are retained. Re-run the same command to continue.";
const HOST: &str = "mymind.octaviantocan.com";
const PAUSE_LIMIT: u32 = 5;

struct Config {
    db: PathBuf,
    hosted: Option<String>,
    cookie: String,
    limit: usize,
    run: bool,
}

fn options(args: &[String]) -> Result<Config> {
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let mut config = Config {
        db: resolve(&cwd, "data/mymind.db"),
        hosted: None,
        cookie: mm_core::runtime::env("MYMIND_SESSION").unwrap_or_default(),
        limit: usize::MAX,
        run: false,
    };
    let mut at = 0;
    while at < args.len() {
        let next = args.get(at + 1).filter(|value| !value.is_empty());
        match (args[at].as_str(), next) {
            ("--run", _) => config.run = true,
            ("--db", Some(path)) => {
                config.db = resolve(&cwd, path);
                at += 1;
            }
            ("--hosted", Some(url)) => {
                config.hosted = Some(url.to_owned());
                at += 1;
            }
            ("--cookie", Some(value)) => {
                config.cookie = value.to_owned();
                at += 1;
            }
            ("--limit", Some(raw)) => {
                at += 1;
                let number = number::string_to_number(raw);
                if !raw.bytes().all(|byte| byte.is_ascii_digit()) || !number::is_safe_integer(number) {
                    return Err(Error::internal("--limit must be a nonnegative integer"));
                }
                config.limit = number as usize;
            }
            _ => return Err(Error::internal(USAGE)),
        }
        at += 1;
    }
    if config.hosted.is_some() && args.iter().any(|arg| arg == "--db") {
        return Err(Error::internal("--db and --hosted cannot be used together"));
    }
    Ok(config)
}

fn print_estimate(estimate: &Estimate) {
    println!(
        "Cards to enrich: {}; missing summary {}; missing tags {}; missing embedding {}",
        estimate.cards, estimate.missing_summary, estimate.missing_tags, estimate.missing_embedding
    );
    println!("Skipped complete: {}; skipped with nothing to read: {}", estimate.skipped_complete, estimate.skipped_empty);
    println!(
        "Estimated chat input: {} tokens; chat output: {} tokens; embeddings: {} tokens",
        estimate.chat_input_tokens, estimate.chat_output_tokens, estimate.embed_tokens
    );
    println!(
        "Estimated cost: ${} USD (chat input ${}/1M, output ${}/1M, embeddings ${}/1M; OpenAI tag picking not included)",
        number::to_fixed(estimate.estimated_usd, 6),
        number::number_to_string(0.10),
        number::number_to_string(0.40),
        number::number_to_string(0.20)
    );
    if estimate.decisions {
        println!("OpenAI tag picking is configured and is not included in the cost.");
    }
}

fn print_report(report: &Report) {
    println!(
        "Enriched {}; summarized {}; tagged {}; embedded {}; skipped {}; remaining {}",
        report.processed, report.summarized, report.tagged, report.embedded, report.skipped, report.remaining
    );
}

fn cookie_header(value: &str) -> String {
    if value.contains('=') { value.to_owned() } else { format!("mm_session={value}") }
}

fn allowed_hosted(raw: &str) -> Result<url::Url> {
    let parsed = url::Url::parse(raw).map_err(|_| Error::internal("hosted URL is not valid"))?;
    let host = parsed.host_str().unwrap_or("");
    let local = parsed.scheme() == "http" && host == "127.0.0.1";
    let live = parsed.scheme() == "https" && host == HOST;
    if !local && !live {
        return Err(Error::internal(format!("hosted backfill only talks to https://{HOST} or http://127.0.0.1")));
    }
    if parsed.path() != "/" && !parsed.path().is_empty() {
        return Err(Error::internal("hosted URL must not include a path"));
    }
    Ok(parsed)
}

fn endpoint(base: &url::Url, query: &str) -> String {
    let mut url = base.clone();
    url.set_path("/ai/backfill");
    url.set_query((!query.is_empty()).then_some(query));
    url.to_string()
}

fn field_number(value: &Value, name: &str) -> usize {
    value.get(name).as_f64().unwrap_or(0.0) as usize
}

fn estimate_from(value: &Value) -> Estimate {
    Estimate {
        cards: field_number(value, "cards"),
        missing_summary: field_number(value, "missingSummary"),
        missing_tags: field_number(value, "missingTags"),
        missing_embedding: field_number(value, "missingEmbedding"),
        skipped_complete: field_number(value, "skippedComplete"),
        skipped_empty: field_number(value, "skippedEmpty"),
        chat_input_tokens: field_number(value, "chatInputTokens"),
        chat_output_tokens: field_number(value, "chatOutputTokens"),
        embed_tokens: field_number(value, "embedTokens"),
        estimated_usd: value.get("estimatedUsd").as_f64().unwrap_or(0.0),
        decisions: value.get("decisions").truthy(),
    }
}

fn report_from(value: &Value) -> Report {
    let retry = value.get("retryAfterMs").as_f64().filter(|ms| ms.is_finite() && *ms >= 0.0).map(|ms| ms as u64);
    let error = value.get("error").as_str().filter(|text| !text.is_empty()).map(str::to_owned);
    Report {
        processed: field_number(value, "processed"),
        summarized: field_number(value, "summarized"),
        tagged: field_number(value, "tagged"),
        embedded: field_number(value, "embedded"),
        skipped: field_number(value, "skipped"),
        failed: field_number(value, "failed"),
        remaining: field_number(value, "remaining"),
        retry_after_ms: retry,
        error,
    }
}

async fn call(fetch: &NativeFetch, method: &str, url: &str, cookie: &str) -> Result<(u16, Value)> {
    let mut request = Request::new(method, url)?;
    if !cookie.is_empty() {
        request.headers.set("cookie", cookie_header(cookie));
    }
    let response = fetch.fetch(request).await.map_err(|error| Error::internal(error.to_string()))?;
    let bytes = response.body.bytes().await.map_err(|error| Error::internal(error.to_string()))?;
    let text = String::from_utf8_lossy(&bytes);
    let value = json::parse(&text).unwrap_or(Value::Null);
    if !(200..300).contains(&response.status) && response.status != 429 && response.status != 502 {
        let detail = value.get("detail").as_str().unwrap_or(text.trim());
        return Err(Error::internal(format!("hosted backfill returned {}: {detail}", response.status)));
    }
    Ok((response.status, value))
}

async fn hosted(config: &Config) -> Result<()> {
    let base = allowed_hosted(config.hosted.as_deref().unwrap_or(""))?;
    if config.cookie.is_empty() {
        return Err(Error::internal("a session cookie is required (--cookie or MYMIND_SESSION)"));
    }
    let fetch = NativeFetch::new();
    let (_, estimate) = call(&fetch, "GET", &endpoint(&base, ""), &config.cookie).await?;
    print_estimate(&estimate_from(&estimate));
    if !config.run {
        println!("Estimate only. Add --run to fill gaps; hand-written summaries and tags are kept.");
        return Ok(());
    }
    let mut budget = config.limit;
    let mut pauses = 0;
    while budget > 0 {
        let batch = budget.min(HOSTED_BATCH);
        let (_, body) = call(&fetch, "POST", &endpoint(&base, &format!("limit={batch}")), &config.cookie).await?;
        let report = report_from(&body);
        if let Some(ms) = report.retry_after_ms {
            pauses += 1;
            if pauses > PAUSE_LIMIT {
                return Err(Error::internal("the provider kept rate-limiting; re-run the same command to continue"));
            }
            println!("Rate limited; retrying after {ms} ms");
            mm_core::runtime::sleep(ms).await;
            continue;
        }
        pauses = 0;
        print_report(&report);
        if let Some(error) = &report.error {
            return Err(Error::internal(error.clone()));
        }
        let done = report.processed + report.skipped;
        if report.remaining == 0 || done == 0 {
            break;
        }
        budget = budget.saturating_sub(done);
    }
    Ok(())
}

async fn local(config: &Config) -> Result<()> {
    if config.run && mm_core::runtime::env("STORE_READONLY").as_deref() == Some("1") {
        return Err(Error::internal("Backfill is disabled by STORE_READONLY=1"));
    }
    let store = Rc::new(Store::new(Rc::new(Sqlite::open(&config.db, false)?))?);
    let estimate = backfill::estimate(&store)?;
    print_estimate(&estimate);
    if !config.run {
        println!("Estimate only. Add --run to fill gaps; hand-written summaries and tags are kept.");
        return Ok(());
    }
    if estimate.cards == 0 || config.limit == 0 {
        return Ok(());
    }
    let offline = mm_core::runtime::env("OFFLINE").as_deref() == Some("1");
    if !ai_configured(offline) {
        return Err(Error::internal("GEMINI_API_KEY and an allowed Gemini endpoint are required"));
    }
    let writer = Writer::new(Rc::clone(&store), |_, _| {})?;
    let media = Rc::new(LocalMedia::new(config.db.parent().unwrap_or(Path::new(".")).join("media")));
    let images = MediaImages(media);
    let fetch = NativeFetch::new();
    let mut budget = config.limit;
    let mut pauses = 0;
    while budget > 0 {
        let report = backfill::run(&store, &writer, &fetch, &images, budget).await?;
        print_report(&report);
        if let Some(ms) = report.retry_after_ms {
            pauses += 1;
            if pauses > PAUSE_LIMIT {
                return Err(Error::internal("the provider kept rate-limiting; re-run the same command to continue"));
            }
            println!("Rate limited; retrying after {ms} ms");
            mm_core::runtime::sleep(ms).await;
            budget = budget.saturating_sub(report.processed + report.skipped);
            continue;
        }
        if let Some(error) = &report.error {
            return Err(Error::internal(error.clone()));
        }
        break;
    }
    Ok(())
}

async fn work(args: &[String]) -> Result<()> {
    let config = options(args)?;
    if config.hosted.is_some() { hosted(&config).await } else { local(&config).await }
}

/// Estimate or run the summary, tag and embedding backfill.
///
/// # Arguments
/// * `root` - Repository root. `.env` there fills keys the process has not set.
/// * `args` - Arguments after the subcommand: `[--db path] [--hosted URL] [--cookie VALUE] [--limit N] [--run]`.
///
/// # Returns
/// The exit status: 0, or 1 after the failure line.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    let file = match env_file::load(&env_file::path(root)) {
        Ok(Loaded::Vars(vars)) => vars,
        _ => HashMap::new(),
    };
    mm_core::runtime::set_env(env_file::merged(std::env::vars(), file));
    match work(args).await {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{FAILED}");
            eprintln!("{error}");
            1
        }
    }
}
