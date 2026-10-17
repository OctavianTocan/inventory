//! `search_document` against `lib/search-document.mjs` on every card of both fixtures and generated documents.

mod common;

use common::{Random, assert_same, calls, hosted_db, test_db};
use mm_core::search_document::{analysis_item, prose_text, search_document};
use mm_core::sql::Sql;
use mm_core::{Value, obj};
use mm_js::json;
use mm_sqlite::Sqlite;

const MODULE: &str = "lib/search-document.mjs";

struct CardText {
    data: String,
    extras: String,
    tags: Option<String>,
}

fn fixture_cards() -> Vec<CardText> {
    let mut cards = Vec::new();
    for path in [test_db(), hosted_db()] {
        let db = Sqlite::open(common::temp_copy(&path), false).unwrap();
        for row in db
            .query("SELECT c.data, c.extras, (SELECT group_concat(name, char(10)) FROM annotations WHERE card_id = c.id) AS tags FROM cards c ORDER BY c.id", &[])
            .unwrap()
        {
            cards.push(CardText {
                data: row.text("data").unwrap().to_owned(),
                extras: row.text("extras").unwrap().to_owned(),
                tags: row.text("tags").map(str::to_owned),
            });
        }
    }
    cards
}

fn node(random: &mut Random, depth: u32) -> String {
    let texts = ["\"\"", "\"a\"", "\"hello world\"", "\"\\u00e9\"", "0", "5", "null", "true", "\"  \"", "[]", "{}"];
    let mut parts = Vec::new();
    if random.below(4) != 0 {
        parts.push(format!("\"text\":{}", texts[random.below(texts.len() as u64) as usize]));
    }
    if depth > 0 && random.below(3) != 0 {
        let count = random.below(4);
        let children: Vec<String> = (0..count).map(|_| node(random, depth - 1)).collect();
        let content = match random.below(12) {
            0 => "null".to_owned(),
            1 => "\"str\"".to_owned(),
            2 => "{}".to_owned(),
            3 => format!("[{}]", [vec!["null".to_owned()], children].concat().join(",")),
            _ => format!("[{}]", children.join(",")),
        };
        parts.push(format!("\"content\":{content}"));
    }
    if random.below(5) == 0 {
        parts.push("\"attrs\":{\"text\":\"hidden\"}".to_owned());
    }
    format!("{{{}}}", parts.join(","))
}

fn pick(random: &mut Random, options: &[&str]) -> String {
    options[random.below(options.len() as u64) as usize].to_owned()
}

fn document(random: &mut Random) -> (String, String) {
    let mut data = Vec::new();
    for (key, options) in [
        ("type", &["\"Article\"", "\"XPost\"", "\"WebPage\"", "null", "5"][..]),
        ("title", &["\"T\"", "null", "\"\"", "0", "\"Title &amp; more\""][..]),
        ("description", &["\"D <b>bold</b> &amp; x\"", "null", "\"\"", "5", "[]"][..]),
        ("summary", &["\"S\"", "null", "\"\"", "0"][..]),
        ("domain", &["\"d.com\"", "null", "\"\""][..]),
        ("siteName", &["\"Site\"", "null", "\"\""][..]),
        ("ocr", &["\"text in image\"", "null", "\"\"", "3"][..]),
        ("source", &["{\"url\":\"https://s.com/a\"}", "{}", "null", "{\"url\":null}", "5"][..]),
    ] {
        if random.below(5) != 0 {
            data.push(format!("\"{key}\":{}", pick(random, options)));
        }
    }
    if random.below(3) != 0 {
        data.push(format!("\"prose\":{}", node(random, 3)));
    }
    if random.below(3) == 0 {
        let note = node(random, 2);
        data.push(format!("\"note\":{{\"id\":\"n\",\"prose\":{note}}}"));
    }
    let mut extras = Vec::new();
    for (key, options) in [
        ("title", &["\"ET\"", "null", "\"\"", "7"][..]),
        ("linkUrl", &["\"https://l.com\"", "null", "\"\""][..]),
        ("captureText", &["\"captured text\"", "null", "\"\"", "\"dup\""][..]),
        ("post", &["{\"bodyHtml\":\"<p>post &amp; <script>x</script></p>\"}", "{}", "null", "{\"bodyHtml\":null}"][..]),
    ] {
        if random.below(4) != 0 {
            extras.push(format!("\"{key}\":{}", pick(random, options)));
        }
    }
    (format!("{{{}}}", data.join(",")), format!("{{{}}}", extras.join(",")))
}

#[test]
fn prose_text_matches_node() {
    let mut random = Random(5);
    let mut nodes: Vec<Value> = ["null", "{}", "\"x\"", "5", "[]", "{\"text\":\"a\"}", "{\"content\":[]}", "{\"text\":0,\"content\":[{\"text\":\"b\"}]}", "{\"content\":{\"length\":1}}", "{\"content\":5}"]
        .iter()
        .map(|text| json::parse(text).unwrap())
        .collect();
    nodes.push(Value::Undefined);
    nodes.extend((0..2500).map(|_| json::parse(&node(&mut random, 4)).unwrap()));
    let js = calls(MODULE, &nodes.iter().map(|item| ("proseText", vec![item.clone()])).collect::<Vec<_>>());
    for (item, answer) in nodes.iter().zip(&js) {
        assert_same(&prose_text(item).map(Value::from), answer, &format!("proseText({item:?})"));
    }
}

#[test]
fn search_document_matches_node_on_every_card() {
    let cards = fixture_cards();
    let js = calls(
        MODULE,
        &cards
            .iter()
            .map(|card| ("searchDocument", vec![Value::from(card.data.as_str()), Value::from(card.extras.as_str()), Value::Null, card.tags.as_deref().map_or(Value::Null, Value::from)]))
            .collect::<Vec<_>>(),
    );
    for (card, answer) in cards.iter().zip(&js) {
        assert_same(&search_document(&card.data, &card.extras, None, card.tags.as_deref()).map(Value::from), answer, &format!("searchDocument({})", &card.data[..card.data.len().min(120)]));
    }
}

#[test]
fn analysis_item_matches_node_on_every_card() {
    let cards = fixture_cards();
    let js = calls(MODULE, &cards.iter().map(|card| ("analysisItem", vec![obj! { "data" => card.data.as_str(), "extras" => card.extras.as_str() }.into()])).collect::<Vec<_>>());
    for (card, answer) in cards.iter().zip(&js) {
        assert_same(&analysis_item(&card.data, &card.extras, None).map(|item| item.to_value()), answer, &format!("analysisItem({})", &card.data[..card.data.len().min(120)]));
    }
}

#[test]
fn generated_documents_match_node() {
    let mut random = Random(17);
    type Case = (String, String, Option<String>, Option<String>, Option<Value>);
    let mut cases: Vec<Case> = Vec::new();
    for _ in 0..3000 {
        let (data, extras) = document(&mut random);
        let content = match random.below(4) {
            0 => Some(node(&mut random, 3)),
            1 => Some("null".to_owned()),
            2 => Some(String::new()),
            _ => None,
        };
        let tags = match random.below(3) {
            0 => Some("a\nb".to_owned()),
            1 => Some(String::new()),
            _ => None,
        };
        let parsed = content.as_deref().filter(|text| !text.is_empty()).map(|text| json::parse(text).unwrap());
        cases.push((data, extras, content, tags, parsed));
    }
    for (data, extras) in [("null", "{}"), ("{}", "null"), ("{", "{}"), ("{}", ""), ("5", "{}"), ("[]", "[]"), ("\"s\"", "{}"), ("{}", "{}")] {
        cases.push((data.to_owned(), extras.to_owned(), None, None, None));
    }
    let js = calls(
        MODULE,
        &cases
            .iter()
            .map(|(data, extras, content, tags, _)| {
                ("searchDocument", vec![Value::from(data.as_str()), Value::from(extras.as_str()), content.as_deref().map_or(Value::Null, Value::from), tags.as_deref().map_or(Value::Null, Value::from)])
            })
            .collect::<Vec<_>>(),
    );
    for ((data, extras, content, tags, _), answer) in cases.iter().zip(&js) {
        let content = content.as_deref().filter(|text| !text.is_empty());
        assert_same(&search_document(data, extras, content, tags.as_deref()).map(Value::from), answer, &format!("searchDocument({data}, {extras}, {content:?}, {tags:?})"));
    }
    let js = calls(
        MODULE,
        &cases
            .iter()
            .map(|(data, extras, _, _, parsed)| {
                ("analysisItem", vec![obj! { "data" => data.as_str(), "extras" => extras.as_str() }.into(), parsed.clone().unwrap_or(Value::Null)])
            })
            .collect::<Vec<_>>(),
    );
    for ((data, extras, _, _, parsed), answer) in cases.iter().zip(&js) {
        assert_same(&analysis_item(data, extras, parsed.as_ref()).map(|item| item.to_value()), answer, &format!("analysisItem({data}, {extras}, {parsed:?})"));
    }
}
