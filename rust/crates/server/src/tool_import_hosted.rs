//! `mymind import-hosted`: port of `tools/import-hosted.mjs`, which builds what the hosted server reads.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;

use mm_core::host::Bucket;
use mm_core::site::{APP_ORIGIN, LOCAL_PREFIXES, is_text_extension, localize, localize_value};
use mm_core::sql::{Sql, SqlExt};
use mm_core::store::Store;
use mm_core::recorded::Recorded;
use mm_core::{Error, Result};
use mm_js::{Object, Value, json, obj, uri, url};
use mm_msgpack::{DecodeOptions, EncodeOptions, decode_multi, encode_into};
use mm_sqlite::Sqlite;
use regex::Regex;

use crate::bucket_fs::DirectoryBucket;
use crate::node_path::{extname, join, resolve};
use crate::read_metadata::import_read_metadata;

const USAGE: &str = "Usage: mymind import-hosted bucket | library <name> --db <file> [--recordings [--runs]] [--media <dir>] [--out <dir>]";
const MAX_BODY: usize = 1_900_000;

static KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(run:[^|]+\|)?((?:iphone|ipad|android-phone|android-tablet)\|)?([A-Za-z0-9_]+) (https://[^/]+/[^\n\r\x{2028}\x{2029}]*)$").unwrap()
});
static PAGE_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.page-[0-9]+\.png$").unwrap());
static LIBRARY_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9-]+$").unwrap());

#[derive(Default)]
struct Options {
    db: Option<String>,
    media: Option<String>,
    out: Option<String>,
    recordings: bool,
    runs: bool,
    positionals: Vec<String>,
}

struct ParsedKey {
    run: bool,
    variant: bool,
    kind: String,
    url: url::Url,
}

type RecordedRow = (String, i64, Value, Vec<u8>);

struct Importer {
    root: PathBuf,
    bucket_dir: PathBuf,
    bucket: DirectoryBucket,
    index: Value,
    runs: bool,
}

fn failure(error: std::io::Error, path: &Path) -> Error {
    Error::internal(format!("{error}: {}", path.display()))
}

// `util.parseArgs` with this tool's options, `strict` and `allowPositionals`.
fn parse_args(args: &[String]) -> Result<Options> {
    let unknown = |name: &str, original: &str| {
        Error::internal(format!(
            "Unknown option '{name}'. To specify a positional argument starting with a '-', place it at the end of the command after '--', as in '-- \"{original}\""
        ))
    };
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        index += 1;
        if arg == "--" {
            options.positionals.extend(args[index..].iter().cloned());
            break;
        }
        let Some(long) = arg.strip_prefix("--") else {
            if arg.len() > 1 && arg.starts_with('-') {
                let short: String = arg.chars().take(2).collect();
                return Err(unknown(&short, arg));
            }
            options.positionals.push(arg.clone());
            continue;
        };
        let (name, inline) = match long.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (long, None),
        };
        match name {
            "recordings" | "runs" => {
                if inline.is_some() {
                    return Err(Error::internal(format!("Option '--{name}' does not take an argument")));
                }
                if name == "runs" { options.runs = true } else { options.recordings = true }
            }
            "db" | "media" | "out" => {
                let value = match inline {
                    Some(value) => value,
                    None => {
                        let Some(next) = args.get(index) else {
                            return Err(Error::internal(format!("Option '--{name} <value>' argument missing")));
                        };
                        if next.len() > 1 && next.starts_with('-') {
                            return Err(Error::internal(format!(
                                "Option '--{name}' argument is ambiguous.\nDid you forget to specify the option argument for '--{name}'?\nTo specify an option argument starting with a dash use '--{name}=-XYZ'."
                            )));
                        }
                        index += 1;
                        next.clone()
                    }
                };
                match name {
                    "db" => options.db = Some(value),
                    "media" => options.media = Some(value),
                    _ => options.out = Some(value),
                }
            }
            _ => return Err(unknown(&format!("--{name}"), arg)),
        }
    }
    Ok(options)
}

fn parse_key(key: &str) -> Result<Option<ParsedKey>> {
    let Some(found) = KEY.captures(key) else { return Ok(None) };
    let url = url::parse(&found[4], None).ok_or_else(|| Error::internal("Invalid URL"))?;
    Ok(Some(ParsedKey { run: found.get(1).is_some(), variant: found.get(2).is_some(), kind: found[3].to_owned(), url }))
}

// Every file below `folder`, as `/`-separated paths relative to it.
fn files(folder: &Path) -> Vec<String> {
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

fn read_text(file: &Path) -> Result<String> {
    Ok(String::from_utf8_lossy(&std::fs::read(file).map_err(|error| failure(error, file))?).into_owned())
}

fn remove_tree(path: &Path) -> Result<()> {
    match std::fs::remove_dir_all(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(failure(error, path)),
        _ => Ok(()),
    }
}

// The database error as the JS driver words it, without the path the Rust driver appends.
fn without_path(error: Error, path: &Path) -> Error {
    let text = error.to_string();
    Error::internal(text.strip_suffix(&format!(": {}", path.display())).unwrap_or(&text).to_owned())
}

fn number(value: usize) -> Value {
    Value::Number(value as f64)
}

impl Importer {
    // Localize recorded MessagePack items and text to the replica origin.
    fn replay_body(&self, entry: &Value) -> Result<Vec<u8>> {
        let Some(file) = entry.get("file").as_str() else { return Err(Error::invalid("The \"path\" argument must be of type string")) };
        let path = join(&self.root, &format!("replay/bodies/{file}"));
        let body = std::fs::read(&path).map_err(|error| failure(error, &path))?;
        let headers = entry.get("headers");
        if headers.is_nullish() {
            return Err(Error::internal("Cannot read properties of undefined (reading 'content-type')"));
        }
        let kind = headers.get("content-type").as_str().unwrap_or("");
        if kind.contains("msgpack") {
            let mut out = Vec::new();
            for item in decode_multi(&body, DecodeOptions::default())? {
                encode_into(&mut out, &localize_value(&item), EncodeOptions::default())?;
            }
            return Ok(out);
        }
        if ["text", "javascript", "json", "svg", "xml"].iter().any(|word| kind.contains(word)) {
            return Ok(localize(&String::from_utf8_lossy(&body)).into_bytes());
        }
        Ok(body)
    }

    // Place `source` at `key` in the store, as a hard link when nothing needs rewriting.
    fn place(&self, key: &str, source: &Path) -> Result<()> {
        let target = join(&self.bucket_dir, key);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| failure(error, parent))?;
        }
        match std::fs::remove_file(&target) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(failure(error, &target)),
            _ => {}
        }
        if std::fs::hard_link(source, &target).is_err() {
            std::fs::copy(source, &target).map_err(|error| failure(error, source))?;
        }
        Ok(())
    }

    // Write `body` at `key` in the store with the response headers mymind sent for it; false when `replace` is off and the key is taken.
    async fn put(&self, key: &str, body: Vec<u8>, replace: bool, headers: Option<&Value>) -> Result<bool> {
        if !replace && self.bucket.head(key).await?.is_some() {
            return Ok(false);
        }
        let length = body.len() as u64;
        self.bucket.write(key, body.into(), Some(length), None).await?;
        if let Some(headers) = headers.filter(|headers| !headers.is_undefined()) {
            let file = join(&self.bucket_dir, &format!(".headers/{key}.json"));
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).map_err(|error| failure(error, parent))?;
            }
            std::fs::write(&file, json::to_string(headers)).map_err(|error| failure(error, &file))?;
        }
        Ok(true)
    }

    // Store the recorded bodies of the URLs below each prefix of `folders` (URL prefix to key folder).
    async fn import_replayed(&self, folders: &[(String, String)]) -> Result<usize> {
        let mut count = 0;
        let Value::Object(index) = &self.index else { return Err(Error::internal("replay/index.json is not an object")) };
        let mut keys = Vec::new();
        for (key, _) in index.iter() {
            let parsed = parse_key(&key)?;
            keys.push((parsed.as_ref().is_some_and(|parsed| parsed.run || parsed.variant), key.into_owned(), parsed));
        }
        keys.sort_by_key(|(plain, ..)| *plain);
        for (_, key, parsed) in keys {
            let Some(parsed) = parsed else { continue };
            let entry = self.index.get(&key);
            if !["GET", "DOC"].contains(&parsed.kind.as_str()) || entry.get("status").as_f64() != Some(200.0) {
                continue;
            }
            let Some((from, folder)) = folders.iter().find(|(prefix, _)| parsed.url.as_str().starts_with(prefix.as_str())) else { continue };
            let base = url::parse(from, None).ok_or_else(|| Error::internal("Invalid URL"))?;
            let path = parsed.url.path();
            let relative = uri::decode_uri_component(path.get(base.path().len()..).unwrap_or(""))?;
            if relative.split('/').any(|part| part == ".." || part == "." || part.is_empty()) {
                continue;
            }
            let body = self.replay_body(entry)?;
            if self.put(&format!("{folder}{relative}"), body, !(parsed.run || parsed.variant), Some(entry.get("headers"))).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn import_bucket(&self) -> Result<()> {
        let mut statics = 0;
        for folder in ["static", ".headers/static"] {
            remove_tree(&self.bucket_dir.join(folder))?;
        }
        let mirror = self.root.join("mirror/static");
        for file in files(&mirror) {
            let source = mirror.join(&file);
            if is_text_extension(&extname(&file).to_lowercase()) {
                self.put(&format!("static/{file}"), localize(&read_text(&source)?).into_bytes(), true, None).await?;
            } else {
                self.place(&format!("static/{file}"), &source)?;
            }
            statics += 1;
        }
        let replayed = self.import_replayed(&[(LOCAL_PREFIXES[0].1.to_owned(), "static/".to_owned())]).await?;
        println!("{}", mm_js::inspect::console_line(&[obj! { "static" => number(statics), "replayed" => number(replayed) }.into()]));
        Ok(())
    }

    // The owner's captured card media, below `capture` in the store.
    async fn import_captured(&self, capture: &str) -> Result<Value> {
        let mut counts = [("media", 0), ("mediahost", 0), ("carbon", 0)];
        for folder in [capture.to_owned(), format!(".headers/{capture}")] {
            remove_tree(&self.bucket_dir.join(folder))?;
        }
        for (folder, count) in &mut counts {
            let directory = self.root.join("capture").join(*folder);
            for file in files(&directory) {
                self.place(&format!("{capture}{folder}/{file}"), &directory.join(&file))?;
                *count += 1;
            }
        }
        let replayed = self
            .import_replayed(&[
                (format!("{APP_ORIGIN}media/"), format!("{capture}media/")),
                (LOCAL_PREFIXES[1].1.to_owned(), format!("{capture}mediahost/")),
                (LOCAL_PREFIXES[2].1.to_owned(), format!("{capture}carbon/")),
            ])
            .await?;
        let mut object = Object::default();
        for (folder, count) in counts {
            object.insert(folder, number(count));
        }
        object.insert("replayed", number(replayed));
        Ok(Value::Object(object))
    }

    // The owner's recorded answers, detail views and page shell, as rows of the `recorded` table.
    fn recorded_rows(&self) -> Result<(Vec<RecordedRow>, Object)> {
        let (mut replay, mut skipped, mut details_count, mut responses_count) = (0, 0, 0, 0);
        let mut rows = Vec::new();
        let Value::Object(index) = &self.index else { return Err(Error::internal("replay/index.json is not an object")) };
        for (key, entry) in index.iter() {
            let Some(parsed) = parse_key(&key)? else { continue };
            if format!("{}/", url::origin(&parsed.url)) != APP_ORIGIN || parsed.url.path().starts_with("/media/") {
                continue;
            }
            if parsed.run && !self.runs {
                continue;
            }
            let body = self.replay_body(entry)?;
            if body.len() > MAX_BODY {
                skipped += 1;
                continue;
            }
            rows.push((key.into_owned(), entry.get("status").as_f64().unwrap_or(0.0) as i64, entry.get("headers").clone(), body));
            replay += 1;
        }
        let details = json::parse(&read_text(&self.root.join("capture/details.json"))?)?;
        if let Value::Object(details) = &details {
            for (id, entry) in details.iter() {
                let Ok(html) = std::fs::read(self.root.join("capture/details").join(format!("{}.html", collapse(&id)))) else { continue };
                let html = String::from_utf8_lossy(&html);
                let mut headers = obj! { "content-type" => "text/html; charset=utf-8" };
                let magic = entry.get("headers").get("x-magic");
                if magic.truthy() {
                    headers.insert("x-magic", magic.clone());
                }
                rows.push((format!("GET {APP_ORIGIN}cards/{id}/detail"), 200, headers.into(), localize(&html).into_bytes()));
                details_count += 1;
            }
        }
        let responses = json::parse(&read_text(&self.root.join("capture/responses.json"))?)?;
        if let Value::Object(responses) = &responses {
            for (key, entry) in responses.iter() {
                let (method, local) = key.split_once(' ').unwrap_or((&key, &key));
                let file = entry.get("file").as_str().unwrap_or_default();
                let body = localize(&read_text(&self.root.join("capture").join(file))?).into_bytes();
                let local = mm_js::text::slice(local, 1, None);
                rows.push((format!("{method} {APP_ORIGIN}{local}"), entry.get("status").as_f64().unwrap_or(0.0) as i64, entry.get("headers").clone(), body));
                responses_count += 1;
            }
        }
        let shell = localize(&read_text(&self.root.join("capture/shell.html"))?).into_bytes();
        rows.push(("shell".to_owned(), 200, obj! { "content-type" => "text/html; charset=utf-8" }.into(), shell));
        let counts = obj! { "replay" => number(replay), "skipped" => number(skipped), "details" => number(details_count), "responses" => number(responses_count) };
        Ok((rows, counts))
    }

    async fn import_library(&self, library: Option<&String>, options: &Options, out: &Path) -> Result<()> {
        let library = library.map_or("", String::as_str);
        if !LIBRARY_NAME.is_match(library) {
            return Err(Error::internal("Library names are lower-case letters, digits and dashes"));
        }
        let Some(source_path) = options.db.as_deref().filter(|db| !db.is_empty()) else { return Err(Error::internal("--db is required")) };
        if options.runs && !options.recordings {
            return Err(Error::internal("--runs needs --recordings"));
        }
        std::fs::create_dir_all(out).map_err(|error| failure(error, out))?;
        let target = out.join(format!("{library}.db"));
        for suffix in ["", "-wal", "-shm"] {
            let file = PathBuf::from(format!("{}{suffix}", target.display()));
            match std::fs::remove_file(&file) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(failure(error, &file)),
                _ => {}
            }
        }
        let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
        let source_file = resolve(&cwd, source_path);
        let source = Sqlite::open(&source_file, true).map_err(|error| without_path(error, &source_file))?;
        source.exec_batch(&format!("VACUUM INTO '{}'", target.to_string_lossy().replace('\'', "''")))?;
        drop(source);
        let db: Rc<dyn Sql> = Rc::new(Sqlite::open(&target, false).map_err(|error| without_path(error, &target))?);
        let store = Store::new(db.clone())?;
        let recorded = Recorded::new(db.clone())?;
        db.transaction_dyn(&mut || db.exec_batch("DELETE FROM recorded"))?;
        let prefix = format!("libraries/{library}/");
        let mut counts = Object::default();
        if options.recordings {
            import_read_metadata(&store, &self.root).await?;
            let (rows, recorded_counts) = self.recorded_rows()?;
            db.transaction_dyn(&mut || {
                for (key, status, headers, body) in &rows {
                    recorded.put(key, *status, headers, body)?;
                }
                Ok(())
            })?;
            counts = recorded_counts;
            counts.insert("captured", self.import_captured(&format!("{prefix}capture/")).await?);
        }
        let mut media = 0;
        if let Some(folder) = options.media.as_deref() {
            let folder = resolve(&cwd, folder);
            for file in files(&folder) {
                if PAGE_IMAGE.is_match(&file) || file.ends_with(".tmp") {
                    continue;
                }
                let key = match file.strip_prefix(".renditions/") {
                    Some(rest) => format!("{prefix}renditions/{rest}"),
                    None => format!("{prefix}media/{file}"),
                };
                self.place(&key, &folder.join(&file))?;
                media += 1;
            }
        }
        let cards = db.one("SELECT COUNT(*) AS n FROM cards", &[])?.and_then(|row| row.number("n")).unwrap_or(0.0);
        let mut summary = obj! { "library" => library, "database" => target.to_string_lossy().into_owned(), "cards" => cards, "recorded" => recorded.count()? as f64 };
        summary.extend(&counts);
        summary.insert("media", number(media));
        println!("{}", mm_js::inspect::console_line(&[summary.into()]));
        Ok(())
    }
}

// The detail file name `capture/details/<id>.html` for an id: runs of unsafe characters become one `_`.
fn collapse(id: &str) -> String {
    let mut out = String::new();
    let mut unsafe_run = false;
    for c in id.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
            out.push(c);
            unsafe_run = false;
        } else if !unsafe_run {
            out.push('_');
            unsafe_run = true;
        }
    }
    out
}

async fn start(root: &Path, args: &[String]) -> Result<u8> {
    let options = parse_args(args)?;
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let out = match options.out.as_deref() {
        Some(out) => resolve(&cwd, out),
        None => join(root, "data/hosted"),
    };
    let bucket_dir = out.join("bucket");
    let index_file = root.join("replay/index.json");
    let index = json::parse(&read_text(&index_file)?)?;
    let importer = Importer { root: root.to_path_buf(), bucket: DirectoryBucket::new(&bucket_dir), bucket_dir, index, runs: options.runs };
    match options.positionals.first().map(String::as_str) {
        Some("bucket") => importer.import_bucket().await?,
        Some("library") => importer.import_library(options.positionals.get(1), &options, &out).await?,
        _ => {
            eprintln!("{USAGE}");
            return Ok(2);
        }
    }
    Ok(0)
}

/// Run `mymind import-hosted bucket | library <name> --db <file> [--recordings [--runs]] [--media <dir>] [--out <dir>]` (`tools/import-hosted.mjs`).
///
/// # Arguments
/// * `root` - Project root, the JS `ROOT`; the default output is `data/hosted` below it.
/// * `args` - The arguments after the subcommand name.
///
/// # Returns
/// The exit status: 0, 2 after the usage line, or 1 after `Error: <message>` was printed to standard error.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    match start(root, args).await {
        Ok(status) => status,
        Err(error) => {
            let text = error.to_string();
            let message = ["SyntaxError: ", "TypeError: ", "RangeError: ", "Error: "].iter().find_map(|prefix| text.strip_prefix(prefix)).unwrap_or(&text);
            eprintln!("Error: {message}");
            1
        }
    }
}
