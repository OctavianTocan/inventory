//! `mymind search-recordings`: the port of `compareRecordings` in `tools/search-recordings.mjs`, which compares the
//! store's prefix searches with the five recorded ones.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use mm_core::search::Search;
use mm_core::sql::SqlValue;
use mm_core::store::Store;
use mm_core::{Error, Result, Value, obj};
use mm_js::json;
use mm_sqlite::Sqlite;

use crate::node_path::resolve;
use crate::parse_args::{Kind, Opt, parse};

const FIELDS: [&str; 8] = ["title", "description", "summary", "tags", "url", "body", "note", "ocr"];

fn read_json(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path).map_err(|error| Error::internal(error.to_string()))?;
    json::parse(&String::from_utf8_lossy(&bytes)).map_err(|error| Error::internal(error.message().to_owned()))
}

/// Compare store results with the five recorded prefix searches (`compareRecordings`).
///
/// # Arguments
/// * `store` - An open temporary store copy, never the shared seeded database.
/// * `root` - The project directory holding the read-only replay recordings.
///
/// # Returns
/// One object per query with counts, overlap and field evidence.
///
/// # Errors
/// A missing recording, unreadable or malformed recording files, or the database's error.
pub fn compare_recordings(store: &Rc<Store>, root: &Path) -> Result<Vec<Value>> {
    let index = read_json(&root.join("replay/index.json"))?;
    let search = Search::new(Rc::clone(store), None, false)?;
    let user = store.default_user_id()?;
    let mut rows = Vec::new();
    for q in ["de", "des", "desi", "desig", "design"] {
        let key = format!("GET https://access.mymind.com/search?q={q}&wildcard=true");
        let entry = index.get(&key);
        if !entry.truthy() {
            return Err(Error::internal(format!("Missing recorded {q} prefix search")));
        }
        let file = entry.get("file").to_js_string();
        let live = read_json(&root.join("replay/bodies").join(file))?;
        let live_ids: HashSet<String> = match live.get("matches") {
            Value::Array(matches) => matches.iter().map(|m| m.get("id").to_js_string()).collect(),
            _ => return Err(Error::internal("Cannot read properties of undefined (reading 'map')")),
        };
        let ours = search.text(q, &obj! { "wildcard" => true }.into())?;
        let overlap = ours.matches.iter().filter(|m| live_ids.contains(&m.id)).count();
        let mut fields = mm_js::Object::new();
        for field in FIELDS {
            let found = store.db().query(
                "SELECT d.id FROM search_fts JOIN search_documents d ON d.rowid = search_fts.rowid
        JOIN cards c ON c.id = d.id WHERE search_fts MATCH ? AND c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & 393224) = 0",
                &[SqlValue::from(format!("{field}: \"{q}\"*")), SqlValue::from(user.as_str())],
            )?;
            fields.insert(field, found.iter().filter(|row| row.text("id").is_some_and(|id| live_ids.contains(id))).count() as f64);
        }
        let live_count = live.get("count").as_f64().unwrap_or(f64::NAN);
        rows.push(
            obj! {
                "q" => q,
                "live" => live.get("count").clone(),
                "ours" => ours.count as f64,
                "overlap" => overlap as f64,
                "liveOnly" => live_count - overlap as f64,
                "storeOnly" => (ours.count - overlap) as f64,
                "fields" => Value::Object(fields),
                "reason" => "Live-only text is absent from indexed fields; URL/body prefixes add store-only matches.",
            }
            .into(),
        );
    }
    Ok(rows)
}

fn work(root: &Path, db: &Path) -> Result<String> {
    let copy: PathBuf = std::env::temp_dir().join(format!("search-recordings-{}-{}.db", std::process::id(), runtime_stamp()));
    std::fs::copy(db, &copy).map_err(|error| Error::internal(error.to_string()))?;
    let outcome = (|| {
        let store = Rc::new(Store::new(Rc::new(Sqlite::open(&copy, false)?))?);
        let rows = compare_recordings(&store, root)?;
        json::stringify_indented(&Value::Array(rows), "  ").map(Option::unwrap_or_default).map_err(|error| Error::internal(error.0))
    })();
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", copy.display()));
    }
    outcome
}

fn runtime_stamp() -> u64 {
    mm_core::runtime::now_ms() as u64
}

/// Run the comparison over a temporary copy of the database and print the rows as indented JSON.
///
/// # Arguments
/// * `root` - The repository root: the recordings and the default `data/mymind.db`.
/// * `args` - The arguments after the subcommand name: `[--db path]`.
///
/// # Returns
/// The exit status: 0, or 1 for a failure.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    let parsed = match parse(args, &[Opt { name: "db", kind: Kind::Text }], false) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    let db = parsed.text("db").map_or_else(|| root.join("data/mymind.db"), |path| resolve(&cwd, path));
    match work(root, &db) {
        Ok(text) => {
            println!("{text}");
            0
        }
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}
