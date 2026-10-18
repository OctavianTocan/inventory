//! `mymind seed-db`: builds a clean synthetic SQLite library database without reading recordings.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use mm_core::render_card::render_card;
use mm_core::sql::{Sql, SqlExt, SqlValue};
use mm_core::store::{Store, iso_from_ns};
use mm_core::{Error, Result, params};
use mm_js::{Value, json, obj, text};
use mm_sqlite::Sqlite;

use crate::node_path::{join, resolve};
use crate::tools::embedding_item;

const USAGE: &str = "Usage: mymind seed-db [--db data/mymind.db]";
const BASE_NS: i128 = 1_760_000_000_000_000_000;
const STEP_NS: i128 = 1_000_000_000;
const SYNTHETIC_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
    0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn message(error: &Error) -> String {
    let text = error.to_string();
    for prefix in ["SyntaxError: ", "TypeError: ", "RangeError: ", "Error: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return rest.to_owned();
        }
    }
    text
}

struct Previous {
    vector: Vec<u8>,
    text: String,
}

fn previous_embeddings(path: &Path) -> Result<HashMap<String, Previous>> {
    let mut previous = HashMap::new();
    if !path.exists() {
        return Ok(previous);
    }
    let db = Sqlite::open(path, true)?;
    if db.one("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'embeddings'", &[])?.is_none() {
        return Ok(previous);
    }
    for row in db.query(
        "SELECT e.card_id, e.vector, c.type, c.data, c.extras, b.body FROM embeddings e
      JOIN cards c ON c.id = e.card_id LEFT JOIN card_content b ON b.card_id = e.card_id",
        &[],
    )? {
        let item = embedding_item(row.text("data").unwrap_or("null"), row.text("extras").unwrap_or("null"), row.text("body"))?;
        let vector = row.blob("vector").map(<[u8]>::to_vec).ok_or_else(|| Error::internal("vector is not a blob"))?;
        previous.insert(row.text("card_id").unwrap_or_default().to_owned(), Previous { vector, text: mm_core::ai::document_text(&item) });
    }
    Ok(previous)
}

fn restore_embeddings(store: &Store, previous: &HashMap<String, Previous>) -> Result<i64> {
    let db = store.db();
    store.transaction(|| {
        let mut kept = 0;
        for card in db.query("SELECT id, user_id, type, data, extras FROM cards", &[])? {
            let id = card.text("id").unwrap_or_default();
            let Some(old) = previous.get(id) else { continue };
            let body = db.one("SELECT body FROM card_content WHERE card_id = ?", params![id])?;
            let item = embedding_item(card.text("data").unwrap_or("null"), card.text("extras").unwrap_or("null"), body.as_ref().and_then(|row| row.text("body")))?;
            if mm_core::ai::document_text(&item) != old.text {
                continue;
            }
            db.execute("INSERT INTO embeddings (card_id, user_id, dimensions, vector) VALUES (?, ?, 768, ?)", &[id.into(), card.get("user_id").clone(), old.vector.clone().into()])?;
            kept += 1;
        }
        Ok(kept)
    })
}

fn prose_doc(paragraph: &str) -> Value {
    obj! {
        "type" => "doc",
        "content" => vec![Value::from(obj! {
            "type" => "paragraph",
            "content" => vec![Value::from(obj! {
                "type" => "text",
                "text" => paragraph,
            })],
        })],
    }
    .into()
}

/// Build a synthetic SQLite library database, replacing any existing file (`seedDatabase`).
pub async fn seed_database(db_path: &Path, _root: &Path) -> Result<Vec<(String, i64)>> {
    let embeddings = previous_embeddings(db_path)?;
    if let Some(parent) = db_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| Error::internal(format!("{error}: {}", parent.display())))?;
    }
    for suffix in ["", "-wal", "-shm"] {
        let mut file = db_path.as_os_str().to_owned();
        file.push(suffix);
        let _ = std::fs::remove_file(file);
    }
    let media_dir = db_path.parent().unwrap_or_else(|| Path::new(".")).join("media/synthetic");
    let _ = std::fs::create_dir_all(&media_dir);
    let _ = std::fs::write(media_dir.join("app.png"), SYNTHETIC_PNG);

    let db: std::rc::Rc<dyn Sql> = std::rc::Rc::new(Sqlite::open(db_path, false)?);
    let store = Store::new(db.clone())?;

    let user_id = "00000002UnE0O2ZiM1aBcD";
    let account_uid = "00000002UnE0O2ZiM1aBcA";
    let id_prefix = "00000002UnE";
    let settings = obj! {
        "colorScheme" => "dark",
        "timeZoneId" => "Europe/Madrid",
        "locale" => "en-US",
        "currency" => "USD",
        "aiLevel" => 3.0,
        "onboardFlags" => 0.0,
        "objectLimit" => 50_000.0,
    };

    let kinds = ["Note", "Image", "Article", "WebPage", "XPost", "SoftwareApplication"];
    let manual_tags = ["design", "architecture", "reference", "rust", "testing"];
    let mut cards_count = 0i64;
    let mut trash_count = 0i64;
    let mut annotations_count = 0i64;
    let mut changes_count = 0i64;
    let mut space_cards_count = 0i64;

    store.transaction(|| {
        db.execute(
            "INSERT INTO users (id, account_uid, id_prefix, flags, settings) VALUES (?, ?, ?, 0, ?)",
            &[user_id.into(), account_uid.into(), id_prefix.into(), json::to_string(&settings.into()).into()],
        )?;

        for (rank, tag) in manual_tags.iter().enumerate() {
            db.execute(
                "INSERT INTO tag_names (user_id, name, modified_ms, manual_rank) VALUES (?, ?, ?, ?)",
                &[user_id.into(), (*tag).into(), (1_760_000_000_000.0 + rank as f64 * 1000.0).into(), (rank as i64).into()],
            )?;
        }

        let space_static = "00000002UnEspace000001";
        let space_smart = "00000002UnEspace000002";
        let query_id = "00000002UnEquery000001";
        db.execute(
            "INSERT INTO spaces (id, user_id, name, color, position, created, query) VALUES (?, ?, 'Design References', '#166ff4', 0, '2026-10-15T10:00:00Z', NULL)",
            &[space_static.into(), user_id.into()],
        )?;
        let smart_query = json::to_string(&obj! { "id" => query_id, "filters" => vec![Value::from("type:image")], "checked" => "2026-10-16T06:20:22Z" }.into());
        db.execute(
            "INSERT INTO spaces (id, user_id, name, color, position, created, query) VALUES (?, ?, 'All Images', '#17c37b', 1, '2026-10-15T10:00:00Z', ?)",
            &[space_smart.into(), user_id.into(), smart_query.into()],
        )?;

        let mut static_rank = 0i64;
        let mut smart_rank = 0i64;

        for index in 1..=110usize {
            let id = if index == 110 { "00000002UnE0O4zunrVB9S".to_owned() } else { format!("{id_prefix}0O{index:09}") };
            let kind = kinds[(index - 1) % kinds.len()];
            let trashed = index > 108;
            let ts_ns = BASE_NS + (index as i128) * STEP_NS;
            let iso = iso_from_ns(ts_ns, 0)?;
            let title = format!("Synthetic {kind} Card {index}");
            let desc = format!("Description for synthetic {kind} card {index} in the library.");
            let tag_name = manual_tags[(index - 1) % manual_tags.len()];
            let tag_id = format!("{id_prefix}tag{index:08}");
            let tag_obj = obj! { "id" => tag_id.as_str(), "name" => tag_name, "flags" => 8.0 };

            let (mut data, extras, content_doc) = match kind {
                "Note" => {
                    let doc = prose_doc(&desc);
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "Note",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "prose" => doc,
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! { "id" => id.as_str(), "title" => title.as_str() },
                        None,
                    )
                }
                "Image" => {
                    let rel_path = format!("media/synthetic/{id}.png");
                    let _ = std::fs::write(media_dir.join(format!("{id}.png")), SYNTHETIC_PNG);
                    let note_doc = prose_doc(&format!("Note on image {index}"));
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "Image",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "blob" => obj! {
                                "path" => rel_path.as_str(),
                                "type" => "image/png",
                                "width" => 800.0,
                                "height" => 600.0,
                                "dominantColor" => "#224466",
                            },
                            "note" => obj! {
                                "id" => format!("{id_prefix}note{index:07}"),
                                "prose" => note_doc,
                            },
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! {
                            "id" => id.as_str(),
                            "title" => title.as_str(),
                            "cover" => obj! {
                                "src" => format!("/{rel_path}"),
                                "w" => 800.0,
                                "h" => 600.0,
                                "ar" => "800/600",
                                "bg" => "#224466",
                            },
                        },
                        None,
                    )
                }
                "Article" => {
                    let url = format!("https://example.test/articles/{index}");
                    let doc = prose_doc(&desc);
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "Article",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "domain" => "example.test",
                            "siteName" => "Example Journal",
                            "source" => obj! { "url" => url.as_str() },
                            "prose" => doc.clone(),
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! {
                            "id" => id.as_str(),
                            "title" => title.as_str(),
                            "linkUrl" => url.as_str(),
                            "reader" => true,
                        },
                        Some(doc),
                    )
                }
                "WebPage" => {
                    let url = format!("https://example.test/pages/{index}");
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "WebPage",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "domain" => "example.test",
                            "siteName" => "Example Web",
                            "source" => obj! { "url" => url.as_str() },
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! {
                            "id" => id.as_str(),
                            "title" => title.as_str(),
                            "linkUrl" => url.as_str(),
                        },
                        None,
                    )
                }
                "XPost" => {
                    let url = format!("https://x.com/example/status/{index}");
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "XPost",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "domain" => "x.com",
                            "siteName" => "X",
                            "source" => obj! { "url" => url.as_str() },
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! {
                            "id" => id.as_str(),
                            "title" => title.as_str(),
                            "linkUrl" => url.as_str(),
                            "post" => obj! { "body" => desc.as_str(), "author" => "Example Author" },
                        },
                        None,
                    )
                }
                _ => {
                    let url = format!("https://github.com/example/repo-{index}");
                    (
                        obj! {
                            "id" => id.as_str(),
                            "type" => "SoftwareApplication",
                            "title" => title.as_str(),
                            "description" => desc.as_str(),
                            "domain" => "github.com",
                            "siteName" => "GitHub",
                            "source" => obj! { "url" => url.as_str() },
                            "tags" => vec![Value::from(tag_obj)],
                            "created" => iso.as_str(),
                            "modified" => iso.as_str(),
                            "bumped" => iso.as_str(),
                        },
                        obj! {
                            "id" => id.as_str(),
                            "title" => title.as_str(),
                            "linkUrl" => url.as_str(),
                            "app" => obj! { "src" => "/media/synthetic/app.png", "srcset" => "/media/synthetic/app.png" },
                        },
                        None,
                    )
                }
            };

            data.insert("flags", 0.0);
            if trashed {
                data.insert("deleted", iso.as_str());
            }
            let data_val = Value::Object(data);
            let extras_val = Value::Object(extras);
            let html = render_card(&data_val, &extras_val)?;
            let deleted_sql = if trashed { SqlValue::big(ts_ns) } else { SqlValue::Null };

            db.execute(
                "INSERT INTO cards (id, user_id, type, flags, data, html, extras, created_ns, bumped_ns, modified_ns, deleted_ns)
                 VALUES (?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    id.as_str().into(),
                    user_id.into(),
                    kind.into(),
                    json::to_string(&data_val).into(),
                    html.into(),
                    json::to_string(&extras_val).into(),
                    SqlValue::big(ts_ns),
                    SqlValue::big(ts_ns),
                    SqlValue::big(ts_ns),
                    deleted_sql,
                ],
            )?;

            if let Some(doc) = content_doc {
                db.execute(
                    "INSERT INTO card_content (card_id, body) VALUES (?, ?)",
                    &[id.as_str().into(), json::to_string(&doc).into()],
                )?;
            }

            db.execute(
                "INSERT INTO annotations (id, user_id, card_id, name, flags, position) VALUES (?, ?, ?, ?, 8, 0)",
                &[tag_id.as_str().into(), user_id.into(), id.as_str().into(), tag_name.into()],
            )?;
            annotations_count += 1;

            db.execute(
                "INSERT INTO changes (user_id, ts_ns, object_type, object_id, type) VALUES (?, ?, 9, ?, ?)",
                &[user_id.into(), SqlValue::big(ts_ns), id.as_str().into(), (if trashed { 2.0 } else { 1.0 }).into()],
            )?;
            changes_count += 1;

            if !trashed {
                cards_count += 1;
                if index <= 6 {
                    db.execute(
                        "INSERT INTO space_cards (space_id, card_id, user_id, rank, score, query_id, excluded) VALUES (?, ?, ?, ?, 1.0, NULL, 0)",
                        &[space_static.into(), id.as_str().into(), user_id.into(), static_rank.into()],
                    )?;
                    static_rank += 1;
                    space_cards_count += 1;
                }
                if kind == "Image" {
                    db.execute(
                        "INSERT INTO space_cards (space_id, card_id, user_id, rank, score, query_id, excluded) VALUES (?, ?, ?, ?, 1.0, ?, 0)",
                        &[space_smart.into(), id.as_str().into(), user_id.into(), smart_rank.into(), query_id.into()],
                    )?;
                    smart_rank += 1;
                    space_cards_count += 1;
                }
                if index <= 2 {
                    db.execute(
                        "INSERT INTO pins (card_id, position) VALUES (?, ?)",
                        &[id.as_str().into(), ((index - 1) as i64).into()],
                    )?;
                }
            } else {
                trash_count += 1;
            }
        }
        Ok(())
    })?;

    let kept = restore_embeddings(&store, &embeddings)?;
    drop(store);
    Ok(vec![
        ("cards".to_owned(), cards_count),
        ("trash".to_owned(), trash_count),
        ("annotations".to_owned(), annotations_count),
        ("spaces".to_owned(), 2),
        ("spaceCards".to_owned(), space_cards_count),
        ("tagNames".to_owned(), manual_tags.len() as i64),
        ("changes".to_owned(), changes_count),
        ("embeddingsKept".to_owned(), kept),
    ])
}

fn parse_args(root: &Path, args: &[String]) -> Result<PathBuf> {
    let mut db = join(root, "data/mymind.db");
    let cwd = std::env::current_dir().map_err(|error| Error::internal(error.to_string()))?;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--db" && args.get(index + 1).is_some_and(|next| !next.is_empty()) {
            index += 1;
            db = resolve(&cwd, &args[index]);
        } else if let Some(path) = args[index].strip_prefix("--db=") {
            if path.is_empty() {
                return Err(Error::internal(USAGE));
            }
            db = resolve(&cwd, path);
        } else {
            return Err(Error::internal(USAGE));
        }
        index += 1;
    }
    Ok(db)
}

/// Run `mymind seed-db [--db path | --db=path]`.
pub async fn run(root: &Path, args: &[String]) -> u8 {
    mm_core::runtime::set_env(std::env::vars().collect());
    let result = async {
        let path = parse_args(root, args)?;
        let summary = seed_database(&path, root).await?;
        println!("Seeded {}", path.display());
        for (key, value) in summary {
            println!("  {} {value}", text::pad_end(&key, 22, " "));
        }
        Ok::<(), Error>(())
    }
    .await;
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", message(&error));
            1
        }
    }
}
