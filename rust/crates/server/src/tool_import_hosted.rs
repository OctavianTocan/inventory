//! `mymind import-hosted`: builds the static bucket and library databases that the hosted server reads.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;

use mm_core::host::Bucket;
use mm_core::site::{is_text_extension, localize};
use mm_core::sql::{Sql, SqlExt};
use mm_core::store::Store;
use mm_core::{Error, Result};
use mm_js::{Value, obj};
use mm_sqlite::Sqlite;
use regex::Regex;

use crate::bucket_fs::DirectoryBucket;
use crate::node_path::{extname, join, resolve};

const USAGE: &str = "Usage: mymind import-hosted bucket | library <name> --db <file> [--media <dir>] [--out <dir>]";

static PAGE_IMAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.page-[0-9]+\.png$").unwrap());
static LIBRARY_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9-]+$").unwrap());

#[derive(Default)]
struct Options {
    db: Option<String>,
    media: Option<String>,
    out: Option<String>,
    positionals: Vec<String>,
}

struct Importer {
    root: PathBuf,
    bucket_dir: PathBuf,
    bucket: DirectoryBucket,
}

fn failure(error: std::io::Error, path: &Path) -> Error {
    Error::internal(format!("{error}: {}", path.display()))
}

fn parse_args(args: &[String]) -> Result<Options> {
    let unknown = |name: &str, original: &str| {
        Error::internal(format!(
            "Unknown option '{name}'. To specify a positional argument starting with a '-', place it at the end of the command after '--', as in '-- \"{original}\"'"
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

fn without_path(error: Error, path: &Path) -> Error {
    let text = error.to_string();
    Error::internal(text.strip_suffix(&format!(": {}", path.display())).unwrap_or(&text).to_owned())
}

fn number(value: usize) -> Value {
    Value::Number(value as f64)
}

impl Importer {
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

    async fn put(&self, key: &str, body: Vec<u8>) -> Result<()> {
        let length = body.len() as u64;
        self.bucket.write(key, body.into(), Some(length), None).await
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
                self.put(&format!("static/{file}"), localize(&read_text(&source)?).into_bytes()).await?;
            } else {
                self.place(&format!("static/{file}"), &source)?;
            }
            statics += 1;
        }
        println!("{}", mm_js::inspect::console_line(&[obj! { "static" => number(statics) }.into()]));
        Ok(())
    }

    async fn import_library(&self, library: Option<&String>, options: &Options, out: &Path) -> Result<()> {
        let library = library.map_or("", String::as_str);
        if !LIBRARY_NAME.is_match(library) {
            return Err(Error::internal("Library names are lower-case letters, digits and dashes"));
        }
        let Some(source_path) = options.db.as_deref().filter(|db| !db.is_empty()) else { return Err(Error::internal("--db is required")) };
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
        let _store = Store::new(db.clone())?;
        let prefix = format!("libraries/{library}/");
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
        let summary = obj! { "library" => library, "database" => target.to_string_lossy().into_owned(), "cards" => cards, "media" => number(media) };
        println!("{}", mm_js::inspect::console_line(&[summary.into()]));
        Ok(())
    }
}

async fn start(root: &Path, args: &[String]) -> Result<u8> {
    let options = parse_args(args)?;
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let out = match options.out.as_deref() {
        Some(out) => resolve(&cwd, out),
        None => join(root, "data/hosted"),
    };
    let bucket_dir = out.join("bucket");
    let importer = Importer { root: root.to_path_buf(), bucket: DirectoryBucket::new(&bucket_dir), bucket_dir };
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

/// Run `mymind import-hosted bucket | library <name> --db <file> [--media <dir>] [--out <dir>]`.
///
/// # Arguments
/// * `root` - Project root; the default output is `data/hosted` below it.
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
