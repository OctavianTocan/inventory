//! `mymind migrate-media`: migrates captured card media and renditions into `data/media` (and `.renditions`) or the
//! hosted bucket, rewrites mymind-host links to `/media/`, and writes a completion report.

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use futures_util::future::try_join_all;
use mm_core::host::{Fetch, Media};
use mm_core::http::Request;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::{Error, Result, Value, obj};
use mm_js::{json, number, uri, url};
use mm_sqlite::Sqlite;
use regex::Regex;

use crate::env_file::{self, Loaded};
use crate::fetch::NativeFetch;
use crate::local_media::LocalMedia;
use crate::node_path::{extname, join, resolve};
use crate::parse_args::{Kind, Opt, parse};

const USAGE: &str = "Usage: mymind migrate-media [local | bucket <worker-url>] [--library <name>] [--db <file>] [--capture <dir>] [--replay <dir>] [--media <dir>] [--report <file>] [--concurrency <n>]";

static AREA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)x(\d+)").expect("pattern"));
static VIDEO_EXT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\.(mp4|webm|mov)$").expect("pattern"));
static PAGE_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.page-[0-9]+\.png$").expect("pattern"));
static ORIGINAL_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^original\.([a-z0-9]+)$").expect("pattern"));
static REPLAY_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:run:[^|]+\|)?(?:(?:iphone|ipad|android-phone|android-tablet)\|)?(?:GET|DOC) (https://(?:access\.mymind\.com/media/|media\.mymind\.host/|carbon-media\.accelerator\.net/)[^\n\r\x{2028}\x{2029}]*)$").expect("pattern")
});
static MYMIND_MEDIA_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://(?:media\.mymind\.host|carbon-media\.accelerator\.net|access\.mymind\.com/media)/([^\s"',)>?\\]+)(?:\?[^\s"',)>\\]*)?"#).expect("pattern")
});
static LOCAL_MEDIAHOST_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"/(?:_mediahost|_carbon)/([^\s"',)>?\\]+)(?:\?[^\s"',)>\\]*)?"#).expect("pattern")
});
static HTML_MEDIA_ATTR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:data-src|data-srcset|src|srcset|poster)="([^"]+)""#).expect("pattern")
});
static MEDIA_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"/media/[^\s"'\\<>),]+"#).expect("pattern")
});

#[derive(Clone, Debug)]
struct Candidate {
    relative: String,
    source_file: PathBuf,
    size: u64,
}

fn failure(error: std::io::Error, path: &Path) -> Error {
    Error::internal(format!("{error}: {}", path.display()))
}

fn area(name: &str) -> f64 {
    AREA.captures(name).map_or(0.0, |found| {
        found[1].parse::<f64>().unwrap_or(0.0) * found[2].parse::<f64>().unwrap_or(0.0)
    })
}

fn candidate_extension(relative: &str) -> String {
    let mut parts = relative.split(';');
    let source = parts.next().unwrap_or("");
    let suffix = parts.collect::<Vec<_>>().join(";");
    if let Some(found) = ORIGINAL_SUFFIX.captures(&suffix) {
        return format!(".{}", &found[1]);
    }
    let ext = extname(relative).to_ascii_lowercase();
    if !ext.is_empty() {
        return ext;
    }
    extname(source).to_ascii_lowercase()
}

fn is_explicit_original(relative: &str) -> bool {
    let mut parts = relative.split(';');
    let _ = parts.next();
    let suffix = parts.collect::<Vec<_>>().join(";");
    suffix.is_empty() || ORIGINAL_SUFFIX.is_match(&suffix)
}

fn is_video(relative: &str) -> bool {
    VIDEO_EXT.is_match(&candidate_extension(relative))
}

fn compare_candidates(a: &Candidate, b: &Candidate) -> Ordering {
    is_explicit_original(&a.relative)
        .cmp(&is_explicit_original(&b.relative))
        .then_with(|| is_video(&a.relative).cmp(&is_video(&b.relative)))
        .then_with(|| area(&a.relative).partial_cmp(&area(&b.relative)).unwrap_or(Ordering::Equal))
        .then_with(|| a.size.cmp(&b.size))
}

fn walk_files(folder: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![(folder.to_path_buf(), String::new())];
    while let Some((directory, prefix)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else { continue };
            let relative = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if kind.is_dir() {
                pending.push((entry.path(), format!("{relative}/")));
            } else if kind.is_file() {
                found.push(relative);
            }
        }
    }
    found.sort();
    found
}

fn collect_sources(capture_dir: &Path, replay_dir: &Path) -> Result<BTreeMap<String, Vec<Candidate>>> {
    let mut map: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    let mut seen_rel: BTreeSet<String> = BTreeSet::new();

    for subfolder in ["media", "mediahost", "carbon"] {
        let base = capture_dir.join(subfolder);
        for rel in walk_files(&base) {
            if rel.split('/').any(|part| part == ".." || part == "." || part.is_empty()) {
                continue;
            }
            let file = base.join(&rel);
            let Ok(meta) = std::fs::metadata(&file) else { continue };
            let stem = rel.split(';').next().unwrap_or(&rel).to_owned();
            if seen_rel.insert(rel.clone()) {
                map.entry(stem).or_default().push(Candidate { relative: rel, source_file: file, size: meta.len() });
            }
        }
    }

    let index_file = replay_dir.join("index.json");
    if index_file.is_file() {
        let raw = std::fs::read(&index_file).map_err(|e| failure(e, &index_file))?;
        let index = json::parse(&String::from_utf8_lossy(&raw))?;
        if let Value::Object(entries) = &index {
            let bodies_dir = replay_dir.join("bodies");
            for (key, entry) in entries.iter() {
                if entry.get("status").as_f64() != Some(200.0) {
                    continue;
                }
                let Some(found) = REPLAY_KEY.captures(&key) else { continue };
                let Some(parsed_url) = url::parse(&found[1], None) else { continue };
                let path = parsed_url.path();
                let raw_rel = path.strip_prefix("/media/").unwrap_or_else(|| path.trim_start_matches('/'));
                let Ok(rel) = uri::decode_uri_component(raw_rel) else { continue };
                if rel.is_empty() || rel.split('/').any(|part| part == ".." || part == "." || part.is_empty()) {
                    continue;
                }
                let Some(body_name) = entry.get("file").as_str() else { continue };
                let file = bodies_dir.join(body_name);
                let Ok(meta) = std::fs::metadata(&file) else { continue };
                let stem = rel.split(';').next().unwrap_or(&rel).to_owned();
                if seen_rel.insert(rel.clone()) {
                    map.entry(stem).or_default().push(Candidate { relative: rel, source_file: file, size: meta.len() });
                }
            }
        }
    }

    Ok(map)
}

fn rewrite_text(input: &str) -> String {
    let out = MYMIND_MEDIA_URL.replace_all(input, "/media/$1");
    LOCAL_MEDIAHOST_URL.replace_all(&out, "/media/$1").into_owned()
}

fn rewrite_database(db: &Sqlite) -> Result<usize> {
    let mut rewritten = 0usize;
    let rows = db.query("SELECT id, data, html, extras FROM cards", &[])?;
    let mut card_updates = Vec::new();
    for row in &rows {
        let id = row.text("id").unwrap_or_default().to_owned();
        let data = row.text("data").unwrap_or_default();
        let html = row.text("html").unwrap_or_default();
        let extras = row.text("extras").unwrap_or_default();
        let new_data = rewrite_text(data);
        let new_html = rewrite_text(html);
        let new_extras = rewrite_text(extras);
        if new_data != data || new_html != html || new_extras != extras {
            card_updates.push((id, new_data, new_html, new_extras));
        }
    }

    let content_rows = if db.one("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'card_content'", &[])?.is_some() {
        db.query("SELECT card_id, body FROM card_content", &[])?
    } else {
        Vec::new()
    };
    let mut content_updates = Vec::new();
    for row in &content_rows {
        let card_id = row.text("card_id").unwrap_or_default().to_owned();
        let body = row.text("body").unwrap_or_default();
        let new_body = rewrite_text(body);
        if new_body != body {
            content_updates.push((card_id, new_body));
        }
    }

    if !card_updates.is_empty() || !content_updates.is_empty() {
        let mut touched_ids: BTreeSet<String> = BTreeSet::new();
        db.transaction(|| {
            for (id, data, html, extras) in &card_updates {
                db.execute(
                    "UPDATE cards SET data = ?, html = ?, extras = ? WHERE id = ?",
                    &[
                        SqlValue::Text(data.clone()),
                        SqlValue::Text(html.clone()),
                        SqlValue::Text(extras.clone()),
                        SqlValue::Text(id.clone()),
                    ],
                )?;
                touched_ids.insert(id.clone());
            }
            for (card_id, body) in &content_updates {
                db.execute(
                    "UPDATE card_content SET body = ? WHERE card_id = ?",
                    &[SqlValue::Text(body.clone()), SqlValue::Text(card_id.clone())],
                )?;
                touched_ids.insert(card_id.clone());
            }
            Ok(())
        })?;
        rewritten = touched_ids.len();
    }
    Ok(rewritten)
}

fn add_media_path(stems: &mut BTreeSet<String>, raw: &str) {
    for part in raw.split(',') {
        let token = part.split_whitespace().next().unwrap_or("");
        let clean = token.split('?').next().unwrap_or("");
        if let Some(rel) = clean.strip_prefix("/media/") {
            let stem = rel.split(';').next().unwrap_or("");
            if !stem.is_empty() && !stem.split('/').any(|p| p == ".." || p == "." || p.is_empty()) {
                stems.insert(stem.to_owned());
            }
        }
    }
}
fn stems_in_text(text: &str, stems: &mut BTreeSet<String>) {
    for found in HTML_MEDIA_ATTR.captures_iter(text) {
        add_media_path(stems, &found[1]);
    }
    for found in MEDIA_PATH.captures_iter(text) {
        add_media_path(stems, &found[0]);
    }
}

fn extract_card_stems(data_str: &str, html_str: &str, extras_str: &str) -> BTreeSet<String> {
    let mut stems = BTreeSet::new();
    if let Ok(data) = json::parse(data_str)
        && let Some(path) = data.get("blob").get("path").as_str()
    {
        let stem = path.split(';').next().unwrap_or("");
        if !stem.is_empty() {
            stems.insert(stem.to_owned());
        }
    }
    if let Ok(extras) = json::parse(extras_str) {
        if let Some(src) = extras.get("cover").get("src").as_str() {
            add_media_path(&mut stems, src);
        }
        if let Some(srcset) = extras.get("app").get("srcset").as_str() {
            add_media_path(&mut stems, srcset);
        }
        if let Some(gallery) = extras.get("post").get("gallery").as_array() {
            for item in gallery {
                if let Some(src) = item.get("src").as_str() {
                    add_media_path(&mut stems, src);
                }
                if let Some(srcset) = item.get("srcset").as_str() {
                    add_media_path(&mut stems, srcset);
                }
            }
        }
    }
    for found in HTML_MEDIA_ATTR.captures_iter(html_str) {
        add_media_path(&mut stems, &found[1]);
    }
    stems
}

fn has_existing_original(media_dir: &Path, stem: &str) -> bool {
    let exact = media_dir.join(stem);
    if exact.is_file() {
        return true;
    }
    let Some(parent) = exact.parent() else { return false };
    let Some(name) = exact.file_name().map(|n| n.to_string_lossy().into_owned()) else { return false };
    let prefix = format!("{name}.");
    let Ok(entries) = std::fs::read_dir(parent) else { return false };
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with(&prefix) && !file_name.contains(".page-") && entry.path().is_file() {
            return true;
        }
    }
    false
}

fn place_if_needed(target: &Path, source: &Path, size: u64) -> Result<bool> {
    if let Ok(meta) = std::fs::metadata(target)
        && meta.is_file()
        && meta.len() == size
    {
        return Ok(false);
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| failure(e, parent))?;
    }
    match std::fs::remove_file(target) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(failure(error, target)),
        _ => {}
    }
    if std::fs::hard_link(source, target).is_err() {
        std::fs::copy(source, target).map_err(|e| failure(e, source))?;
    }
    Ok(true)
}

fn migrate_stem(media_dir: &Path, stem: &str, candidates: &[Candidate]) -> Result<(usize, usize)> {
    let mut originals = 0usize;
    let mut renditions = 0usize;

    for candidate in candidates {
        if !is_explicit_original(&candidate.relative) {
            let cache_name = candidate.relative.replace('/', "_");
            let cache_target = media_dir.join(".renditions").join(cache_name);
            if place_if_needed(&cache_target, &candidate.source_file, candidate.size)? {
                renditions += 1;
            }
        }
    }

    if !has_existing_original(media_dir, stem)
        && let Some(best) = candidates.iter().max_by(|a, b| compare_candidates(a, b))
    {
        let ext = candidate_extension(&best.relative);
        let stem_ext = extname(stem);
        let rel_target = if !stem_ext.is_empty() || ext.is_empty() {
            stem.to_owned()
        } else {
            format!("{stem}{ext}")
        };
        let target = media_dir.join(rel_target);
        if place_if_needed(&target, &best.source_file, best.size)? {
            originals += 1;
        }
    }

    Ok((originals, renditions))
}

fn encode_component(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

async fn upload_media_to_bucket(base_url: &str, token: &str, library: &str, media_dir: &Path, concurrency: usize) -> std::result::Result<(u64, u64), String> {
    let fetch = NativeFetch::new();
    let all_files = walk_files(media_dir);
    let mut items: Vec<(String, PathBuf, u64)> = Vec::new();
    for rel in all_files {
        if PAGE_IMAGE.is_match(&rel) || rel.ends_with(".tmp") {
            continue;
        }
        let path = media_dir.join(&rel);
        let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
        let key = match rel.strip_prefix(".renditions/") {
            Some(rest) => format!("libraries/{library}/renditions/{rest}"),
            None => format!("libraries/{library}/media/{rel}"),
        };
        items.push((key, path, size));
    }

    let (uploaded, skipped) = (Cell::new(0u64), Cell::new(0u64));
    let next = Cell::new(0usize);
    let workers = concurrency.max(1).min(items.len().max(1));
    let base = base_url.strip_suffix('/').unwrap_or(base_url);

    let worker_fn = |_| {
        let (items, fetch, uploaded, skipped, next) = (&items, &fetch, &uploaded, &skipped, &next);
        async move {
            while next.get() < items.len() {
                let idx = next.get();
                next.set(idx + 1);
                let (key, file, size) = &items[idx];
                let encoded_key = key.split('/').map(encode_component).collect::<Vec<_>>().join("/");
                let url = format!("{base}/_admin/bucket/{encoded_key}");

                let mut head_req = Request::new("HEAD", &url).map_err(|e| e.to_string())?;
                head_req.headers.set("authorization", format!("Bearer {token}"));
                let head_resp = fetch.fetch(head_req).await.map_err(|e| e.to_string())?;
                if head_resp.status == 200 {
                    let reported = head_resp.headers.get("x-object-size").map_or(0.0, number::string_to_number);
                    if reported == *size as f64 {
                        skipped.set(skipped.get() + 1);
                        continue;
                    }
                } else if head_resp.status != 404 {
                    let body = head_resp.body.bytes().await.unwrap_or_default();
                    return Err(format!("HEAD bucket/{key}: {} {}", head_resp.status, String::from_utf8_lossy(&body)));
                }

                let mut put_req = Request::new("PUT", &url).map_err(|e| e.to_string())?;
                put_req.headers.set("authorization", format!("Bearer {token}"));
                let put_resp = fetch.fetch_file(put_req, file).await.map_err(|e| e.to_string())?;
                if !(200..300).contains(&put_resp.status) {
                    let body = put_resp.body.bytes().await.unwrap_or_default();
                    return Err(format!("PUT bucket/{key}: {} {}", put_resp.status, String::from_utf8_lossy(&body)));
                }
                uploaded.set(uploaded.get() + 1);
            }
            Ok::<(), String>(())
        }
    };

    try_join_all((0..workers).map(worker_fn)).await?;
    Ok((uploaded.get(), skipped.get()))
}

async fn run_inner(root: &Path, args: &[String]) -> std::result::Result<u8, String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(0);
    }
    let declared = [
        Opt { name: "library", kind: Kind::Text },
        Opt { name: "db", kind: Kind::Text },
        Opt { name: "capture", kind: Kind::Text },
        Opt { name: "replay", kind: Kind::Text },
        Opt { name: "media", kind: Kind::Text },
        Opt { name: "report", kind: Kind::Text },
        Opt { name: "concurrency", kind: Kind::Text },
    ];
    let parsed = parse(args, &declared, true)?;
    let mode = parsed.positionals.first().map_or("local", String::as_str);
    if !matches!(mode, "local" | "bucket") {
        eprintln!("{USAGE}");
        return Ok(2);
    }
    let worker_url = if mode == "bucket" {
        let Some(url) = parsed.positionals.get(1) else {
            eprintln!("{USAGE}");
            return Ok(2);
        };
        Some(url.clone())
    } else {
        None
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| root.to_path_buf());
    let resolve_opt = |opt: Option<&str>, default_rel: &str| -> PathBuf {
        match opt {
            Some(given) => resolve(&cwd, given),
            None => join(root, default_rel),
        }
    };

    let db_path = resolve_opt(parsed.text("db"), "data/mymind.db");
    let capture_dir = resolve_opt(parsed.text("capture"), "capture");
    let replay_dir = resolve_opt(parsed.text("replay"), "replay");
    let media_dir = resolve_opt(parsed.text("media"), "data/media");
    let report_path = resolve_opt(parsed.text("report"), "data/migrate-media-report.json");
    let library = parsed.text("library").unwrap_or("owner");
    let concurrency = parsed.text("concurrency").and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);

    std::fs::create_dir_all(&media_dir).map_err(|e| failure(e, &media_dir).to_string())?;
    let db = std::rc::Rc::new(Sqlite::open(&db_path, false).map_err(|e| e.to_string())?);
    let sql: std::rc::Rc<dyn Sql> = db.clone();
    let _store = mm_core::store::Store::new(sql).map_err(|e| e.to_string())?;
    let rewritten_cards = rewrite_database(&db).map_err(|e| e.to_string())?;
    let sources = collect_sources(&capture_dir, &replay_dir).map_err(|e| e.to_string())?;

    let rows = db.query("SELECT id, type, data, html, extras FROM cards", &[]).map_err(|e| e.to_string())?;
    let content_exists = db.one("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'card_content'", &[]).map_err(|e| e.to_string())?;
    let content_rows = if content_exists.is_some() {
        db.query("SELECT card_id, body FROM card_content", &[]).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let mut content_by_card: BTreeMap<String, String> = BTreeMap::new();
    for row in &content_rows {
        content_by_card.insert(row.text("card_id").unwrap_or_default().to_owned(), row.text("body").unwrap_or_default().to_owned());
    }

    let mut migrated_originals = 0usize;
    let mut migrated_renditions = 0usize;
    let mut processed_stems: BTreeSet<String> = BTreeSet::new();
    let mut missing: Vec<Value> = Vec::new();
    let mut document_stems: Vec<String> = Vec::new();
    for row in &rows {
        let card_id = row.text("id").unwrap_or_default();
        let kind = row.text("type").unwrap_or_default();
        let data = row.text("data").unwrap_or_default();
        let html = row.text("html").unwrap_or_default();
        let extras = row.text("extras").unwrap_or_default();
        let mut stems = extract_card_stems(data, html, extras);
        if let Some(body) = content_by_card.get(card_id) {
            stems_in_text(body, &mut stems);
        }
        for stem in stems {
            if kind == "Document" {
                document_stems.push(stem.clone());
            }
            if processed_stems.insert(stem.clone())
                && let Some(candidates) = sources.get(&stem)
            {
                let (orig, rend) = migrate_stem(&media_dir, &stem, candidates).map_err(|e| e.to_string())?;
                migrated_originals += orig;
                migrated_renditions += rend;
            }
            if !has_existing_original(&media_dir, &stem) {
                missing.push(
                    obj! {
                        "cardId" => card_id,
                        "type" => kind,
                        "path" => stem.as_str(),
                    }
                    .into(),
                );
            }
        }
    }

    // Also migrate any remaining non-marketing stems in the source folders (such as reader view images).
    for (stem, candidates) in &sources {
        if stem.starts_with("00000000001/") {
            continue;
        }
        if processed_stems.insert(stem.clone()) {
            let (orig, rend) = migrate_stem(&media_dir, stem, candidates).map_err(|e| e.to_string())?;
            migrated_originals += orig;
            migrated_renditions += rend;
        }
    }

    // Pre-generate Document page-1 cover renditions when missing so hosted bucket has them cached.
    if !document_stems.is_empty() {
        let local_media = LocalMedia::new(&media_dir);
        for stem in document_stems {
            let rel = format!("{stem};background(fff)/page(1)/642x840-t.webp");
            let cache_target = media_dir.join(".renditions").join(rel.replace('/', "_"));
            if !cache_target.is_file() && local_media.response(&rel).await.ok().flatten().is_some() {
                migrated_renditions += 1;
            }
        }
    }

    let (mut uploaded, mut skipped) = (0u64, 0u64);
    if let Some(worker_url) = &worker_url {
        let file = match env_file::load(&env_file::path(root)) {
            Ok(Loaded::Vars(vars)) => vars,
            Ok(Loaded::Missing) => HashMap::new(),
            Err(_) => return Err("Could not load .env".to_owned()),
        };
        let env = env_file::merged(std::env::vars(), file);
        let Some(token) = env.get("ADMIN_TOKEN").filter(|token| !token.is_empty()) else {
            return Err("ADMIN_TOKEN is not set".to_owned());
        };
        let (up, sk) = upload_media_to_bucket(worker_url, token, library, &media_dir, concurrency).await?;
        uploaded = up;
        skipped = sk;
    }

    let mut report = obj! {
        "cards" => rows.len() as f64,
        "rewrittenCards" => rewritten_cards as f64,
        "migratedOriginals" => migrated_originals as f64,
        "migratedRenditions" => migrated_renditions as f64,
        "missing" => Value::Array(missing),
    };
    if worker_url.is_some() {
        report.insert("uploaded", uploaded as f64);
        report.insert("skipped", skipped as f64);
    }
    let report_val: Value = report.into();
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| failure(e, parent).to_string())?;
    }
    let serialized = json::to_string(&report_val);
    std::fs::write(&report_path, format!("{serialized}\n")).map_err(|e| failure(e, &report_path).to_string())?;
    println!("{}", mm_js::inspect::console_line(&[report_val]));
    Ok(0)
}

/// Run `mymind migrate-media`.
///
/// # Arguments
/// * `root` - Project root.
/// * `args` - Arguments after the subcommand name.
///
/// # Returns
/// Exit status (`0` on success, `2` on usage error, `1` on runtime error).
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    match run_inner(root, args).await {
        Ok(status) => status,
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}
