//! Back today and the sky (`GET /app/v1/sky`, the push's `visit` change): each day up to three old finds, picked by
//! meaning near what was saved lately; the constellation they light, one at a time; and the visits that light them.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use mm_js::{Value, date, json, obj, text};

use crate::ai::{Ai, cosine};
use crate::app_api::prose_text;
use crate::host::Fetch;
use crate::search_semantic::decode_vector;
use crate::sql::{Sql, SqlExt, SqlValue};
use crate::store::{CardRow, GRID_HIDDEN_FLAGS, Store, iso_from_ns};
use crate::{Error, Result};

/// Most finds Back today offers in a day.
pub const BACK_PER_DAY: usize = 3;

/// Old finds a constellation joins to the recent save it forms around: six stars in all.
pub const CONSTELLATION_FINDS: usize = 5;

/// The figures a constellation can take, as `(key, name, what it stands for)`. The app draws each key; a figure is
/// used once until every one has been.
pub const FIGURES: [(&str, &str, &str); 14] = [
    ("ship", "The Voyager", "a ship: travel, places to go, journeys"),
    ("cup", "The Hearth", "a steaming cup: home, food, drink, slow mornings"),
    ("book", "The Reader", "an open book: reading, writing, ideas, learning"),
    ("mountain", "The Climber", "a mountain: the outdoors, sport, nature, big goals"),
    ("lantern", "The Lantern", "a lantern: advice, wisdom, quotes, finding your way"),
    ("camera", "The Watcher", "a camera: photography, film, design, things to look at"),
    ("harp", "The Singer", "a harp: music, sound, podcasts, voices"),
    ("tree", "The Gardener", "a tree: plants, health, growth, living things"),
    ("house", "The Builder", "a house: architecture, interiors, homes, making places"),
    ("compass", "The Seeker", "a compass: exploring, maps, research, discovery"),
    ("anvil", "The Smith", "an anvil: tools, code, crafts, how things are made"),
    ("bird", "The Dreamer", "a bird in flight: art, imagination, freedom, what could be"),
    ("wave", "The Tide", "a wave: the sea, water, calm, the mind"),
    ("coin", "The Merchant", "a coin: money, shopping, products, work"),
];

const DAY_MS: f64 = 86_400_000.0;
// A find is old once it was saved, and last saved again, this long ago.
const OLD_DAYS: f64 = 90.0;
// A find opened this recently is not offered again.
const RESTED_DAYS: f64 = 180.0;
// Saves this recent steer the picks and seed constellations.
const RECENT_DAYS: f64 = 14.0;
// A find offered and left alone is not offered again for this long, unless its constellation still needs it.
const OFFERED_DAYS: f64 = 30.0;
// Newest saves the picks are measured against, and tried as a constellation's seed.
const NEWEST: i64 = 20;
// Most of a day's finds that come from the active constellation.
const FROM_CONSTELLATION: usize = 2;

const TABLES: &str = "CREATE TABLE IF NOT EXISTS sky_days (day TEXT PRIMARY KEY, card_ids TEXT NOT NULL CHECK (json_valid(card_ids)), made_ms INTEGER NOT NULL) STRICT;
CREATE TABLE IF NOT EXISTS sky_visits (card_id TEXT PRIMARY KEY, visited_ms INTEGER NOT NULL) STRICT;
CREATE TABLE IF NOT EXISTS sky_constellations (id INTEGER PRIMARY KEY AUTOINCREMENT, figure TEXT NOT NULL, name TEXT NOT NULL, seed TEXT NOT NULL, card_ids TEXT NOT NULL CHECK (json_valid(card_ids)), created_ms INTEGER NOT NULL, completed_ms INTEGER) STRICT";

// A card that may be picked or steer the picks, with its embedding when it has one.
struct Candidate {
    id: String,
    vector: Option<Vec<f32>>,
}

struct Constellation {
    id: i64,
    figure: String,
    name: String,
    seed: String,
    cards: Vec<String>,
    created_ms: f64,
    completed_ms: Option<f64>,
}

/// Create the sky's tables when they are missing.
///
/// # Errors
/// The database's error.
pub fn install(db: &dyn Sql) -> Result<()> {
    db.exec_batch(TABLES)
}

/// Record that the person opened a find (the push's `visit` change, `cardId`), and finish the active constellation
/// when that lights its last star.
///
/// # Arguments
/// * `store` - The library.
/// * `card_id` - The find opened.
/// * `now_ms` - The time of the visit.
///
/// # Errors
/// Status 404 for a card that is missing or in the trash; the database's error.
pub fn visit(store: &Store, card_id: &str, now_ms: f64) -> Result<()> {
    let db = store.db();
    install(&**db)?;
    if !alive(&**db, card_id)? {
        return Err(Error::status(404, "not-found"));
    }
    db.execute(
        "INSERT INTO sky_visits (card_id, visited_ms) VALUES (?, ?) ON CONFLICT(card_id) DO UPDATE SET visited_ms = excluded.visited_ms",
        &[card_id.into(), (now_ms as i64).into()],
    )?;
    settle(&**db, now_ms)
}

/// The sky for one day, the body of `GET /app/v1/sky`.
///
/// The first call of a day keeps that day's picks, so later calls the same day answer the same finds. When no
/// constellation is active, it forms one around the save from the last two weeks whose five nearest old finds are
/// nearest of all, and asks OpenAI Decisions which of [`FIGURES`] fits them (an unused figure picked by the seed's id
/// when there is no transport or the call fails).
///
/// # Arguments
/// * `store` - The library.
/// * `fetch` - Outbound HTTP for the Decisions call, `None` to never call it.
/// * `day` - The phone's date, `YYYY-MM-DD`.
/// * `now_ms` - The current time.
///
/// # Returns
/// `{day, back, active, done, stars}`: today's finds by id (at most [`BACK_PER_DAY`]: old finds, preferring up to two
/// unlit stars of the active constellation, then those nearest in meaning to recent saves); the active constellation
/// or null; finished constellations, oldest first; and every find visited, `{id, visited}`. A constellation is `{id,
/// figure, name, seed, cards, lit, created, completed}`, its `cards` the seed and then its finds, `lit` the seed and
/// the finds visited since it formed.
///
/// # Errors
/// The database's error.
pub async fn today(store: &Store, fetch: Option<&dyn Fetch>, day: &str, now_ms: f64) -> Result<Value> {
    let db = store.db();
    install(&**db)?;
    settle(&**db, now_ms)?;
    if active(&**db)?.is_none() {
        form(store, fetch, now_ms).await?;
    }
    let current = active(&**db)?;
    let back = picks(store, day, now_ms, current.as_ref())?;
    let done = constellations(&**db, "completed_ms IS NOT NULL ORDER BY completed_ms, id")?
        .iter()
        .map(|constellation| constellation_json(&**db, constellation))
        .collect::<Result<Vec<_>>>()?;
    let stars = db
        .query(
            "SELECT v.card_id, v.visited_ms FROM sky_visits v JOIN cards c ON c.id = v.card_id WHERE c.deleted_ns IS NULL ORDER BY v.visited_ms, v.card_id",
            &[],
        )?
        .iter()
        .map(|row| Ok(obj! { "id" => row.text("card_id").unwrap_or(""), "visited" => iso(row.number("visited_ms").unwrap_or(0.0))? }.into()))
        .collect::<Result<Vec<Value>>>()?;
    let active = match &current {
        Some(constellation) => constellation_json(&**db, constellation)?,
        None => Value::Null,
    };
    Ok(obj! {
        "day" => day,
        "back" => Value::Array(back.into_iter().map(Value::String).collect()),
        "active" => active,
        "done" => Value::Array(done),
        "stars" => Value::Array(stars),
    }
    .into())
}

/// Whether `day` is a calendar date written `YYYY-MM-DD`.
pub fn valid_day(day: &str) -> bool {
    let parts: Vec<&str> = day.split('-').collect();
    let [year, month, date] = parts.as_slice() else { return false };
    if year.len() != 4 || month.len() != 2 || date.len() != 2 || !day.bytes().all(|byte| byte.is_ascii_digit() || byte == b'-') {
        return false;
    }
    let (Ok(year), Ok(month), Ok(date)) = (year.parse::<i64>(), month.parse::<i64>(), date.parse::<i64>()) else { return false };
    (1..=12).contains(&month) && date >= 1 && date::civil_from_days(date::days_from_civil(year, month, date)) == (year, month, date)
}

/// The UTC date of a time, `YYYY-MM-DD`, for a request that names no day.
pub fn utc_day(now_ms: f64) -> String {
    date::iso_from_ms(now_ms).map(|iso| text::slice(&iso, 0, Some(10)).to_owned()).unwrap_or_default()
}

fn iso(ms: f64) -> Result<String> {
    iso_from_ns((ms as i128) * 1_000_000, 0)
}

fn ns(ms: f64) -> SqlValue {
    SqlValue::big((ms as i128) * 1_000_000)
}

// FNV-1a over the day and an id: the order cards without a usable embedding are taken in, the same all day.
fn shuffle_key(day: &str, id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in day.bytes().chain([0]).chain(id.bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn alive(db: &dyn Sql, id: &str) -> Result<bool> {
    Ok(db.one(&format!("SELECT 1 FROM cards WHERE id = ? AND deleted_ns IS NULL AND (flags & {GRID_HIDDEN_FLAGS}) = 0"), &[id.into()])?.is_some())
}

fn visited_ms(db: &dyn Sql, id: &str) -> Result<Option<f64>> {
    Ok(db.one("SELECT visited_ms FROM sky_visits WHERE card_id = ?", &[id.into()])?.and_then(|row| row.number("visited_ms")))
}

fn ids(raw: &str) -> Vec<String> {
    json::parse(raw).ok().and_then(|value| value.as_array().map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect())).unwrap_or_default()
}

fn ids_json(ids: &[String]) -> Result<String> {
    Ok(json::stringify(&Value::Array(ids.iter().cloned().map(Value::String).collect()))?.unwrap_or_default())
}

fn constellations(db: &dyn Sql, filter: &str) -> Result<Vec<Constellation>> {
    Ok(db
        .query(&format!("SELECT id, figure, name, seed, card_ids, created_ms, completed_ms FROM sky_constellations WHERE {filter}"), &[])?
        .iter()
        .map(|row| Constellation {
            id: row.int("id").unwrap_or(0),
            figure: row.text("figure").unwrap_or("").to_owned(),
            name: row.text("name").unwrap_or("").to_owned(),
            seed: row.text("seed").unwrap_or("").to_owned(),
            cards: ids(row.text("card_ids").unwrap_or("[]")),
            created_ms: row.number("created_ms").unwrap_or(0.0),
            completed_ms: row.number("completed_ms"),
        })
        .collect())
}

fn active(db: &dyn Sql) -> Result<Option<Constellation>> {
    Ok(constellations(db, "completed_ms IS NULL ORDER BY id DESC LIMIT 1")?.into_iter().next())
}

// The seed, which is lit from the start, and each find opened since the constellation formed.
fn lit(db: &dyn Sql, constellation: &Constellation) -> Result<Vec<String>> {
    let mut lit = Vec::new();
    for id in &constellation.cards {
        if *id == constellation.seed || visited_ms(db, id)?.is_some_and(|at| at >= constellation.created_ms) {
            lit.push(id.clone());
        }
    }
    Ok(lit)
}

// The finds still dark, in the constellation's order; a find in the trash no longer counts.
fn unlit(db: &dyn Sql, constellation: &Constellation) -> Result<Vec<String>> {
    let lit: HashSet<String> = lit(db, constellation)?.into_iter().collect();
    let mut dark = Vec::new();
    for id in &constellation.cards {
        if !lit.contains(id) && alive(db, id)? {
            dark.push(id.clone());
        }
    }
    Ok(dark)
}

// Finish the active constellation once nothing in it is dark.
fn settle(db: &dyn Sql, now_ms: f64) -> Result<()> {
    if let Some(constellation) = active(db)?
        && unlit(db, &constellation)?.is_empty()
    {
        db.execute("UPDATE sky_constellations SET completed_ms = ? WHERE id = ?", &[(now_ms as i64).into(), constellation.id.into()])?;
    }
    Ok(())
}

fn constellation_json(db: &dyn Sql, constellation: &Constellation) -> Result<Value> {
    let strings = |ids: Vec<String>| Value::Array(ids.into_iter().map(Value::String).collect());
    Ok(obj! {
        "id" => constellation.id as f64,
        "figure" => constellation.figure.as_str(),
        "name" => constellation.name.as_str(),
        "seed" => constellation.seed.as_str(),
        "cards" => strings(constellation.cards.clone()),
        "lit" => strings(lit(db, constellation)?),
        "created" => iso(constellation.created_ms)?,
        "completed" => match constellation.completed_ms {
            Some(ms) => Value::String(iso(ms)?),
            None => Value::Null,
        },
    }
    .into())
}

fn candidates(db: &dyn Sql, sql: &str, params: &[SqlValue]) -> Result<Vec<Candidate>> {
    Ok(db
        .query(sql, params)?
        .iter()
        .map(|row| Candidate { id: row.text("id").unwrap_or("").to_owned(), vector: row.blob("vector").and_then(decode_vector) })
        .collect())
}

// Old finds that may be offered: saved and last saved again over three months ago, not opened in six, visible.
fn old_finds(store: &Store, now_ms: f64) -> Result<Vec<Candidate>> {
    let old = ns(now_ms - OLD_DAYS * DAY_MS);
    candidates(
        &**store.db(),
        &format!(
            "SELECT c.id, e.vector FROM cards c LEFT JOIN embeddings e ON e.card_id = c.id
      WHERE c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0 AND c.created_ns < ? AND c.bumped_ns < ?
        AND NOT EXISTS (SELECT 1 FROM sky_visits v WHERE v.card_id = c.id AND v.visited_ms >= ?)
      ORDER BY c.id"
        ),
        &[store.default_user_id()?.into(), old.clone(), old, ((now_ms - RESTED_DAYS * DAY_MS) as i64).into()],
    )
}

// The newest saves that have an embedding, newest first.
fn newest(store: &Store) -> Result<Vec<(Candidate, f64)>> {
    Ok(store
        .db()
        .query(
            &format!(
                "SELECT c.id, c.created_ns, e.vector FROM cards c JOIN embeddings e ON e.card_id = c.id
      WHERE c.user_id = ? AND c.deleted_ns IS NULL AND (c.flags & {GRID_HIDDEN_FLAGS}) = 0 ORDER BY c.created_ns DESC, c.id DESC LIMIT {NEWEST}"
            ),
            &[store.default_user_id()?.into()],
        )?
        .iter()
        .filter_map(|row| {
            let vector = row.blob("vector").and_then(decode_vector)?;
            Some((Candidate { id: row.text("id").unwrap_or("").to_owned(), vector: Some(vector) }, row.number("created_ns").unwrap_or(0.0) / 1e6))
        })
        .collect())
}

// Every card any constellation has used.
fn in_constellations(db: &dyn Sql) -> Result<HashSet<String>> {
    Ok(db.query("SELECT card_ids FROM sky_constellations", &[])?.iter().flat_map(|row| ids(row.text("card_ids").unwrap_or("[]"))).collect())
}

// Old finds nearest in meaning to the steering saves first (each scored by its nearest one), then those without an
// embedding in the day's own order.
fn by_meaning(pool: Vec<Candidate>, steering: &[Candidate], day: &str) -> Vec<String> {
    let mut scored: Vec<(Option<f64>, u64, String)> = pool
        .into_iter()
        .map(|candidate| {
            let score = candidate.vector.as_deref().and_then(|vector| {
                steering
                    .iter()
                    .filter(|steer| steer.id != candidate.id)
                    .filter_map(|steer| steer.vector.as_deref().map(|other| cosine(vector, other)))
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            });
            (score, shuffle_key(day, &candidate.id), candidate.id)
        })
        .collect();
    scored.sort_by(|a, b| match (a.0, b.0) {
        (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(Ordering::Equal).then_with(|| a.2.cmp(&b.2)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)),
    });
    scored.into_iter().map(|(_, _, id)| id).collect()
}

// The day's finds: kept from the day's first call, or chosen now and kept.
fn picks(store: &Store, day: &str, now_ms: f64, constellation: Option<&Constellation>) -> Result<Vec<String>> {
    let db = store.db();
    let kept = |db: &dyn Sql| -> Result<Option<Vec<String>>> {
        Ok(db.one("SELECT card_ids FROM sky_days WHERE day = ?", &[day.into()])?.map(|row| ids(row.text("card_ids").unwrap_or("[]"))))
    };
    if let Some(found) = kept(&**db)? {
        let mut alive_ids = Vec::new();
        for id in found {
            if alive(&**db, &id)? {
                alive_ids.push(id);
            }
        }
        return Ok(alive_ids);
    }
    let mut offered: HashMap<String, f64> = HashMap::new();
    for row in db.query("SELECT card_ids, made_ms FROM sky_days WHERE day != ?", &[day.into()])? {
        let made = row.number("made_ms").unwrap_or(0.0);
        for id in ids(row.text("card_ids").unwrap_or("[]")) {
            let last = offered.entry(id).or_insert(made);
            *last = last.max(made);
        }
    }
    let mut dark = match constellation {
        Some(constellation) => unlit(&**db, constellation)?,
        None => Vec::new(),
    };
    dark.sort_by(|a, b| offered.get(a).copied().unwrap_or(f64::MIN).partial_cmp(&offered.get(b).copied().unwrap_or(f64::MIN)).unwrap_or(Ordering::Equal));
    let mut chosen: Vec<String> = dark.iter().take(FROM_CONSTELLATION).cloned().collect();
    let skip: HashSet<&String> = constellation.map(|constellation| constellation.cards.iter().collect()).unwrap_or_default();
    let rested = now_ms - OFFERED_DAYS * DAY_MS;
    let pool: Vec<Candidate> = old_finds(store, now_ms)?
        .into_iter()
        .filter(|candidate| !skip.contains(&candidate.id) && offered.get(&candidate.id).is_none_or(|made| *made < rested))
        .collect();
    let newest = newest(store)?;
    let recent: Vec<Candidate> = newest.iter().filter(|(_, created)| *created >= now_ms - RECENT_DAYS * DAY_MS).map(|(candidate, _)| copy(candidate)).collect();
    let steering = if recent.is_empty() { newest.iter().map(|(candidate, _)| copy(candidate)).collect() } else { recent };
    for id in by_meaning(pool, &steering, day) {
        if chosen.len() >= BACK_PER_DAY {
            break;
        }
        chosen.push(id);
    }
    for id in dark.into_iter().skip(FROM_CONSTELLATION) {
        if chosen.len() >= BACK_PER_DAY {
            break;
        }
        chosen.push(id);
    }
    db.execute("INSERT OR IGNORE INTO sky_days (day, card_ids, made_ms) VALUES (?, ?, ?)", &[day.into(), ids_json(&chosen)?.into(), (now_ms as i64).into()])?;
    Ok(kept(&**db)?.unwrap_or(chosen))
}

fn copy(candidate: &Candidate) -> Candidate {
    Candidate { id: candidate.id.clone(), vector: candidate.vector.clone() }
}

// The seed and its finds: of the saves from the last two weeks, the one whose five nearest old finds are nearest on
// average, newest first on a tie.
fn pattern(seeds: &[Candidate], pool: &[Candidate]) -> Option<(String, Vec<String>)> {
    let mut best: Option<(f64, String, Vec<String>)> = None;
    for seed in seeds {
        let Some(vector) = seed.vector.as_deref() else { continue };
        let mut near: Vec<(f64, &str)> = pool
            .iter()
            .filter(|candidate| candidate.id != seed.id)
            .filter_map(|candidate| candidate.vector.as_deref().map(|other| (cosine(vector, other), candidate.id.as_str())))
            .collect();
        if near.len() < CONSTELLATION_FINDS {
            continue;
        }
        near.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal).then_with(|| a.1.cmp(b.1)));
        near.truncate(CONSTELLATION_FINDS);
        let mean = near.iter().map(|(score, _)| score).sum::<f64>() / near.len() as f64;
        if best.as_ref().is_none_or(|(top, _, _)| mean > *top) {
            best = Some((mean, seed.id.clone(), near.into_iter().map(|(_, id)| id.to_owned()).collect()));
        }
    }
    best.map(|(_, seed, finds)| (seed, finds))
}

// What a find is, for the figure question: its title, type, site and description.
fn find_text(store: &Store, row: &CardRow) -> Result<String> {
    let data = json::parse(&row.data)?;
    let extras = json::parse(&row.extras)?;
    let title = [extras.get("title"), data.get("title")].into_iter().find_map(|value| value.as_str().filter(|title| !title.is_empty())).unwrap_or("");
    let mut line = if title.is_empty() {
        match store.db().one("SELECT body FROM card_content WHERE card_id = ?", &[row.id.as_str().into()])? {
            Some(found) => prose_text(&json::parse(found.text("body").unwrap_or("null"))?).unwrap_or_default(),
            None => String::new(),
        }
    } else {
        title.to_owned()
    };
    line = text::slice(&line.replace('\n', " "), 0, Some(200)).to_owned();
    let site = data.get("domain").as_str().unwrap_or("");
    let about = text::slice(data.get("description").as_str().unwrap_or(""), 0, Some(200)).to_owned();
    Ok(format!("{line} ({}{}){}", row.kind, if site.is_empty() { String::new() } else { format!(", {site}") }, if about.is_empty() { String::new() } else { format!(": {about}") }))
}

// The figure for a new constellation: the one Decisions picks among those not yet used, or one picked by the seed.
async fn figure(store: &Store, fetch: Option<&dyn Fetch>, seed: &str, cards: &[String]) -> Result<usize> {
    let used: Vec<String> = store.db().query("SELECT figure FROM sky_constellations ORDER BY id", &[])?.iter().map(|row| row.text("figure").unwrap_or("").to_owned()).collect();
    let mut offered: Vec<usize> = (0..FIGURES.len()).filter(|index| !used.iter().any(|key| key == FIGURES[*index].0)).collect();
    if offered.is_empty() {
        offered = (0..FIGURES.len()).filter(|index| used.last().is_none_or(|last| last != FIGURES[*index].0)).collect();
    }
    let fallback = offered[(shuffle_key("", seed) % offered.len() as u64) as usize];
    let Some(fetch) = fetch else { return Ok(fallback) };
    let rows = store.cards_by_ids(cards, None)?;
    let lines: Vec<String> = rows.iter().enumerate().map(|(at, row)| Ok(format!("{}. {}", at + 1, find_text(store, row)?))).collect::<Result<_>>()?;
    let input = format!("Things one person saved to their library, which will join as a constellation in their night sky:\n{}", lines.join("\n"));
    let choices: Vec<Value> = offered.iter().map(|index| obj! { "value" => FIGURES[*index].0, "description" => format!("{}, drawn as {}", FIGURES[*index].1, FIGURES[*index].2) }.into()).collect();
    let question = obj! {
        "type" => "choice",
        "name" => "figure",
        "instructions" => "Which figure best stands for what these saved things have in common?",
        "choices" => Value::Array(choices),
    };
    let Ok(answers) = Ai::new(fetch).decide(Value::String(input), vec![question.into()]).await else { return Ok(fallback) };
    let picked = answers
        .iter()
        .find(|answer| answer.get("name").as_str() == Some("figure") && answer.get("type").as_str() == Some("choice"))
        .and_then(|answer| answer.get("choice").as_str().map(str::to_owned))
        .and_then(|key| offered.iter().copied().find(|index| FIGURES[*index].0 == key));
    Ok(picked.unwrap_or(fallback))
}

// Form the next constellation when the library has a recent save and five old finds near it.
async fn form(store: &Store, fetch: Option<&dyn Fetch>, now_ms: f64) -> Result<()> {
    let db = store.db();
    let used = in_constellations(&**db)?;
    let seeds: Vec<Candidate> = newest(store)?
        .into_iter()
        .filter(|(candidate, created)| *created >= now_ms - RECENT_DAYS * DAY_MS && !used.contains(&candidate.id))
        .map(|(candidate, _)| candidate)
        .collect();
    if seeds.is_empty() {
        return Ok(());
    }
    let pool: Vec<Candidate> = old_finds(store, now_ms)?.into_iter().filter(|candidate| candidate.vector.is_some() && !used.contains(&candidate.id)).collect();
    let Some((seed, finds)) = pattern(&seeds, &pool) else { return Ok(()) };
    let cards: Vec<String> = std::iter::once(seed.clone()).chain(finds).collect();
    let (key, name, _) = FIGURES[figure(store, fetch, &seed, &cards).await?];
    // Another request may have formed one while the figure was being chosen.
    db.execute(
        "INSERT INTO sky_constellations (figure, name, seed, card_ids, created_ms) SELECT ?, ?, ?, ?, ? WHERE NOT EXISTS (SELECT 1 FROM sky_constellations WHERE completed_ms IS NULL)",
        &[key.into(), name.into(), seed.as_str().into(), ids_json(&cards)?.into(), (now_ms as i64).into()],
    )?;
    Ok(())
}
