//! Compare media, downloads, tags and query behavior with frozen pre-switch expectations.

mod common;

use common::{Random, assert_same, calls};
use mm_core::ai_tags::{curate_tags, relevant_tag, tag_key};
use mm_core::download_page::download_page;
use mm_core::media_files::{MIME, byte_range, cover, mime, parse_rendition};
use mm_core::search_query::{Scores, Term, evaluate, parse_query, type_slugs};
use mm_core::{Value, arr, obj};

// Return a historical frozen namespace, never a filesystem input.
fn driver(name: &str) -> String {
    format!("driver:core:{name}")
}

fn same<T: Into<Value>>(rust: mm_core::Result<T>, js: &common::Answer, context: &str) {
    assert_same(&rust.map(Into::into), js, context);
}

fn strings(values: &[String]) -> Value {
    Value::Array(values.iter().map(|value| Value::from(value.as_str())).collect())
}

#[test]
fn mime_table_matches_node() {
    let _goldens = common::test_guard();
    let js = common::call(&driver("render"), "mimeTable", vec![]);
    let rust: Value = MIME.iter().map(|(key, value)| arr![*key, *value]).collect::<Vec<_>>().into();
    common::assert_value(&rust, &js, "MIME table");
    assert_eq!(mime("png"), Some("image/png"));
    assert_eq!(mime("constructor"), None);
}

#[test]
fn rendition_matches_node() {
    let _goldens = common::test_guard();
    let pieces = [
        "a/b.png", ";", ";", "original.png", "original.PNG", "original.", "100x200.webp", "100x200-t.jpg", "100x200-c.jpeg", "10x10/hevc.mp4",
        "0x5.png", "4096x4096.png", "4097x1.png", "1x4097.png", "/", "background(fff)", "page(2)", "page(0)", "page(1001)", "page(1000)",
        "page(x)", "page(", "hevc.mp4", ".mp4", "-t", "99999999999999999999x1.png", "1x1.mp4", "1x1.mp4/hevc.mp4", "x", "٣x٣.png",
        "12x12-x.png", "12x12-t.gif", "12x12.webp/hevc.mp4", "/64x64.png", "a64x64.png", "%20", "é", "007x08.png", "9x9/hevc.mp4/hevc.mp4",
    ];
    let mut random = Random(41);
    let mut paths: Vec<String> = pieces.iter().map(|piece| (*piece).to_string()).collect();
    for piece in pieces {
        paths.push(format!("p;{piece}"));
        paths.push(format!("p;1x1.png/{piece}"));
    }
    for _ in 0..4000 {
        let length = random.below(6) as usize + 1;
        paths.push(random.text(&pieces, length));
    }
    let js = calls("module:media-files", &paths.iter().map(|path| ("parseRendition", vec![Value::from(path.as_str())])).collect::<Vec<_>>());
    let mut ok = 0;
    for (path, js) in paths.iter().zip(&js) {
        let rust = parse_rendition(path).map(|rendition| rendition.to_value());
        ok += usize::from(rust.is_ok());
        same(rust, js, &format!("parseRendition({path:?})"));
    }
    assert!(ok > 300, "{ok}");
}

#[test]
fn byte_range_matches_node() {
    let _goldens = common::test_guard();
    let pieces = ["bytes=", "bytes=", "0", "1", "5", "9", "-", "-", "abc", "007", "9007199254740991", "9007199254740993", "99999999999999999999", " ", "\n", "bytes=0-", "bytes=-0", "bytes=-"];
    let mut random = Random(5);
    let mut ranges: Vec<Option<String>> = vec![None];
    for piece in pieces {
        ranges.push(Some(piece.to_string()));
    }
    for _ in 0..3000 {
        let length = random.below(5) as usize + 1;
        ranges.push(Some(random.text(&pieces, length)));
    }
    let sizes = [0.0, 1.0, 10.0, 100.0, 9_007_199_254_740_991.0];
    let mut requests = Vec::new();
    for range in &ranges {
        for size in sizes {
            requests.push(("byteRange", vec![range.as_deref().map_or(Value::Undefined, Value::from), size.into()]));
        }
    }
    let js = calls("module:media-files", &requests);
    let mut at = 0;
    let mut partial = 0;
    for range in &ranges {
        for size in sizes {
            let answer = byte_range(range.as_deref(), size);
            partial += usize::from(answer.status == 206);
            same(Ok(answer.to_value()), &js[at], &format!("byteRange({range:?}, {size})"));
            at += 1;
        }
    }
    assert!(partial > 100);
}

#[test]
fn cover_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(9);
    let numbers = [0.0, 1.0, 3.0, 640.0, 800.0, 801.0, 1000.0, 1999.0, 12345.0, 0.5, -3.0, 1e9];
    let mut cases: Vec<(Value, f64, f64)> = Vec::new();
    for _ in 0..800 {
        let pick = |random: &mut Random| numbers[random.below(numbers.len() as u64) as usize];
        let (width, height) = (pick(&mut random), pick(&mut random));
        let mut blob = obj! { "path" => "a/b c.png", "width" => width, "height" => height };
        if random.below(2) == 0 {
            blob.insert("dominantColor", "#fa0");
        }
        cases.push((blob.into(), pick(&mut random), pick(&mut random)));
    }
    cases.push((obj! { "path" => "p", "width" => "30", "height" => "x" }.into(), 800.0, 1000.0));
    cases.push((obj! { "width" => 5, "height" => 5 }.into(), 800.0, 1000.0));
    cases.push((Value::Null, 800.0, 1000.0));
    cases.push((Value::Undefined, 800.0, 1000.0));
    let mut requests: Vec<_> = cases.iter().map(|(blob, w, h)| ("cover", vec![blob.clone(), (*w).into(), (*h).into()])).collect();
    requests.push(("cover", vec![obj! { "path" => "d", "width" => 4000, "height" => 3000 }.into()]));
    let js = calls("module:media-files", &requests);
    for ((blob, w, h), js) in cases.iter().zip(&js) {
        same(cover(blob, *w, *h), js, &format!("cover({}, {w}, {h})", common::text(blob)));
    }
    let defaults = cover(&obj! { "path" => "d", "width" => 4000, "height" => 3000 }.into(), 800.0, 1000.0);
    same(defaults, js.last().unwrap(), "cover defaults");
}

#[test]
fn download_page_matches_node() {
    let _goldens = common::test_guard();
    let labels = ["Linux <x86> & \"64\"", "é😀", "", "plain"];
    let sizes: [Value; 8] = [0.into(), 1_048_576.into(), 1_572_864.into(), "5000000".into(), Value::Undefined, Value::Null, 1e22.into(), (-3).into()];
    let built: [Value; 7] = ["2026-10-16T12:00:00Z".into(), "2026".into(), arr!["a", 2, Value::Null, arr![1, 2]], 5.into(), Value::Null, Value::Undefined, "😀😀😀😀😀😀😀😀😀😀😀".into()];
    let mut manifests: Vec<Value> = vec![
        obj! { "files" => arr![] }.into(),
        obj! { "library" => "https://x.test/\"q\"", "files" => arr![] }.into(),
        obj! { "library" => Value::Null, "files" => arr![] }.into(),
        obj! { "library" => 5, "files" => arr![] }.into(),
        obj! {}.into(),
        obj! { "files" => "abc" }.into(),
        obj! { "files" => obj! { "length" => 0 } }.into(),
        obj! { "files" => arr![Value::Null] }.into(),
        obj! { "files" => arr![obj! {}] }.into(),
        Value::Null,
        Value::Undefined,
    ];
    let mut random = Random(3);
    for index in 0..60 {
        let count = random.below(4) as usize;
        let files: Vec<Value> = (0..count)
            .map(|_| {
                obj! {
                    "name" => format!("mymind-{index} é/#?.tar.gz"),
                    "label" => labels[random.below(4) as usize],
                    "bytes" => sizes[random.below(8) as usize].clone(),
                    "built" => built[random.below(7) as usize].clone(),
                    "sha256" => format!("ab{index}<>\"&"),
                }
                .into()
            })
            .collect();
        manifests.push(obj! { "library" => "https://l.test", "files" => Value::Array(files) }.into());
    }
    let users = ["me@x.test", "<b>\"&", "é"];
    let mut requests = Vec::new();
    for manifest in &manifests {
        for user in users {
            requests.push(("downloadPage", vec![manifest.clone(), user.into()]));
        }
    }
    let js = calls("module:download-page", &requests);
    let mut at = 0;
    for manifest in &manifests {
        for user in users {
            same(download_page(manifest, user), &js[at], &format!("downloadPage({}, {user})", common::text(manifest)));
            at += 1;
        }
    }
}

const WORDS: &[&str] = &[
    "a", "B", "cats", "Cat", "ies", "Berries", "berry", "boxes", "Box", "bus", "gas", "class", "glass", "kiss", "this", "its", "is", "us",
    "watches", "dishes", "quizzes", "buzzes", "ox", "-", "_", " ", "  ", "\t", "\u{a0}", "\u{feff}", "highlight", "Private", "hidden", "NSFW",
    "website", "link", "page", "image", "Images", "screenshot", "video", "Videos", "pdf", "document", "file", "web page", "webpage", "app",
    "applications", "mobile app", "tweets", "post", "posts", "İ", "é", "😀", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "web  page", "e-mail", "to_do",
];

const TYPES: &[&str] = &[
    "Image", "Screenshot", "Video", "YouTubeVideo", "Document", "WebPage", "SoftwareApplication", "XPost", "ThreadsPost", "SubstackNote",
    "RedditPost", "Article", "", "constructor", "__proto__", "toString", "hasOwnProperty", "valueOf", "x", "Note",
];

#[test]
fn tag_keys_match_node() {
    let _goldens = common::test_guard();
    let mut random = Random(77);
    let mut names: Vec<String> = WORDS.iter().map(|word| (*word).to_string()).collect();
    for _ in 0..4000 {
        let length = random.below(5) as usize;
        names.push(random.text(WORDS, length));
    }
    let js = calls("module:ai-tags", &names.iter().map(|name| ("tagKey", vec![Value::from(name.as_str())])).collect::<Vec<_>>());
    for (name, js) in names.iter().zip(&js) {
        same(Ok(tag_key(name)), js, &format!("tagKey({name:?})"));
    }
    let js = calls("module:ai-tags", &[("tagKey", vec![Value::Null]), ("tagKey", vec![Value::Undefined]), ("tagKey", vec![5.into()])]);
    assert_same(&Ok(tag_key("").into()), &js[0], "tagKey(null)");
    assert_same(&Ok(tag_key("").into()), &js[1], "tagKey(undefined)");
    assert_same(&Ok(tag_key("5").into()), &js[2], "tagKey(5)");
}

#[test]
fn relevant_tags_match_node() {
    let _goldens = common::test_guard();
    let mut random = Random(78);
    let mut cases: Vec<(String, &str)> = Vec::new();
    for word in WORDS {
        for kind in TYPES {
            cases.push(((*word).to_string(), kind));
        }
    }
    for _ in 0..3000 {
        let length = random.below(3) as usize + 1;
        cases.push((random.text(WORDS, length), TYPES[random.below(TYPES.len() as u64) as usize]));
    }
    let js = calls("module:ai-tags", &cases.iter().map(|(name, kind)| ("relevantTag", vec![Value::from(name.as_str()), Value::from(*kind)])).collect::<Vec<_>>());
    for ((name, kind), js) in cases.iter().zip(&js) {
        same(relevant_tag(name, kind).map(Value::Bool), js, &format!("relevantTag({name:?}, {kind:?})"));
    }
}

#[test]
fn curated_tags_match_node() {
    let _goldens = common::test_guard();
    let mut random = Random(79);
    type Case<'a> = (Vec<Value>, &'a str, Vec<String>, Vec<String>);
    let mut requests: Vec<Case> = Vec::new();
    for _ in 0..3000 {
        let make = |random: &mut Random, count: usize| -> Vec<String> {
            (0..count).map(|_| { let length = random.below(4) as usize + 1; random.text(WORDS, length) }).collect()
        };
        let count = random.below(16) as usize;
        let tags: Vec<Value> = make(&mut random, count)
            .into_iter()
            .map(|name| obj! { "name" => name, "score" => random.below(100) as f64 / 100.0, "provider" => "p" }.into())
            .collect();
        let vocabulary_count = random.below(8) as usize;
        let existing_count = random.below(4) as usize;
        let vocabulary = make(&mut random, vocabulary_count);
        let existing = make(&mut random, existing_count);
        requests.push((tags, TYPES[random.below(TYPES.len() as u64) as usize], vocabulary, existing));
    }
    requests.push((vec![Value::Null], "Image", vec![], vec![]));
    requests.push((vec![obj! {}.into(), obj! { "name" => 5 }.into(), "str".into(), obj! { "name" => "ok", "z" => 1, "a" => 2 }.into()], "Image", vec![], vec![]));
    requests.push((vec![obj! { "name" => "  padded name  " }.into(), obj! { "name" => "a b c d e" }.into(), obj! { "name" => "a b c d" }.into()], "Note", vec!["padded Name".into()], vec![]));
    requests.push(((0..14).map(|index| obj! { "name" => format!("tag{index}x") }.into()).collect(), "Note", vec![], vec![]));
    let js = calls("module:ai-tags", &requests.iter().map(|(tags, kind, vocabulary, existing)| ("curateTags", vec![Value::Array(tags.clone()), Value::from(*kind), strings(vocabulary), strings(existing)])).collect::<Vec<_>>());
    let mut kept = 0;
    for ((tags, kind, vocabulary, existing), js) in requests.iter().zip(&js) {
        let rust = curate_tags(tags, kind, vocabulary, existing);
        kept += rust.as_ref().map_or(0, Vec::len);
        same(rust.map(Value::Array), js, &format!("curateTags({}, {kind}, {vocabulary:?}, {existing:?})", common::text(&Value::Array(tags.clone()))));
    }
    assert!(kept > 1000, "{kept}");
}

const QUERY: &[&str] = &[
    "cat", "dog", "Cat*", "*", "a", "b", "c", "AND", "OR", "NOT", "&&", "||", "(", ")", "-", "!", "+", " ", " ", "  ", "tag:", "tag:x", "label:y",
    "#tag", "#a", "type:image", "type:", "Type:Note", "date:[2020 TO 2021]", "date:[2020-01-01", "TO", "2021]", "\"quoted phrase\"", "\"half", "phrase\"",
    "\"a\"~3", "\"a\\\"b\"", "\\", "\\(", "\\\\", "^2", "cat^2", "cat^2.5", "cat^1000", "\"q\"^3", "\u{201c}smart\u{201d}", "\u{ab}fr\u{bb}", "é", "😀",
    "tag:\"x y\"", "tag:[a", "a:b:c", "http://x.y", "\n", "\t", "\u{a0}", "[", "]", "a]b", "x[y z]w", "\"~", "\"x\"~12", "\"x\"~", "\\\n",
];

#[test]
fn type_slugs_match_node() {
    let _goldens = common::test_guard();
    let mut random = Random(13);
    let mut kinds: Vec<String> = TYPES.iter().map(|kind| (*kind).to_string()).collect();
    let extra = ["XPost", "YouTubeVideo", "InstagramReel", "TikTokPost", "TedTalk", "RealProperty", "Palette", "Music", "Recording", "URL", "HTMLElement", "ABCdef", "aBC", "A", "a1B2c", "iOSApp", "WebPage", "Quote", "Quotation", "VideoGame", "Embed", "BlueskyPost", "x-post", "É", ""];
    kinds.extend(extra.iter().map(|kind| (*kind).to_string()));
    for _ in 0..2000 {
        let length = random.below(5) as usize + 1;
        kinds.push(random.text(&extra, length));
    }
    let js = calls("module:search-query", &kinds.iter().map(|kind| ("typeSlugs", vec![Value::from(kind.as_str())])).collect::<Vec<_>>());
    for (kind, js) in kinds.iter().zip(&js) {
        same(type_slugs(kind).map(|slugs| strings(&slugs)), js, &format!("typeSlugs({kind:?})"));
    }
}

fn queries() -> Vec<String> {
    let mut random = Random(21);
    let mut queries: Vec<String> = QUERY.iter().map(|piece| (*piece).to_string()).collect();
    queries.push(String::new());
    queries.push("x".repeat(8192));
    queries.push("x".repeat(8193));
    queries.push("😀".repeat(4096));
    queries.push("😀".repeat(4097));
    queries.push(vec!["a"; 256].join(" "));
    queries.push(vec!["a"; 257].join(" "));
    queries.push("(".repeat(300));
    for _ in 0..6000 {
        let length = random.below(9) as usize + 1;
        let joiner = [" ", "", " ", "  "][random.below(4) as usize];
        queries.push((0..length).map(|_| QUERY[random.below(QUERY.len() as u64) as usize]).collect::<Vec<_>>().join(joiner));
    }
    queries
}

#[test]
fn parse_query_matches_node() {
    let _goldens = common::test_guard();
    let queries = queries();
    let mut requests = Vec::new();
    for query in &queries {
        requests.push(("parseQuery", vec![Value::from(query.as_str()), Value::Bool(false)]));
        requests.push(("parseQuery", vec![Value::from(query.as_str()), Value::Bool(true)]));
    }
    requests.push(("parseQuery", vec![Value::Null]));
    requests.push(("parseQuery", vec![Value::Undefined, Value::Bool(true)]));
    let js = calls("module:search-query", &requests);
    let mut at = 0;
    let mut appended = 0;
    for query in &queries {
        for wildcard in [false, true] {
            let parsed = parse_query(query, wildcard);
            appended += usize::from(wildcard && parsed.as_ref().is_ok_and(|parsed| parsed.query.len() > query.trim().len() && parsed.query.ends_with('*') && !query.trim().ends_with('*')));
            same(parsed.map(|parsed| parsed.to_value()), &js[at], &format!("parseQuery({query:?}, {wildcard})"));
            at += 1;
        }
    }
    assert!(appended > 50, "{appended}");
    assert_same(&Ok(parse_query("", false).unwrap().to_value()), &js[at], "parseQuery(null)");
    assert_same(&Ok(parse_query("", true).unwrap().to_value()), &js[at + 1], "parseQuery(undefined)");
}

fn hash(text: &str) -> u64 {
    text.chars().fold(7u64, |n, c| (n * 31 + c as u64) % 1_000_003)
}

#[test]
fn evaluate_matches_node() {
    let _goldens = common::test_guard();
    let ids: Vec<String> = (0..24).map(|index| format!("id{index}é")).collect();
    let queries: Vec<String> = queries().into_iter().filter(|query| query.len() < 400).take(2500).collect();
    let mut requests = Vec::new();
    for (index, query) in queries.iter().enumerate() {
        requests.push(("run", vec![Value::from(query.as_str()), Value::Bool(index % 2 == 0), strings(&ids), Value::Bool(index % 3 == 0)]));
    }
    let js = calls(&driver("search-query"), &requests);
    let universe: Scores = ids.iter().map(|id| (id.clone(), 1.0)).collect();
    let mut compared = 0;
    for (index, (query, js)) in queries.iter().zip(&js).enumerate() {
        let (wildcard, negative_only) = (index % 2 == 0, index % 3 == 0);
        let parsed = match parse_query(query, wildcard) {
            Ok(parsed) => parsed,
            Err(error) => {
                common::assert_throws(&Err(error), js, &format!("parse query {query:?}"));
                continue;
            }
        };
        let mut trace: Vec<Value> = Vec::new();
        let mut resolve = |term: &Term, negative: bool| -> mm_core::Result<Scores> {
            trace.push(arr![term.field.as_str(), term.value.as_str(), term.quoted, term.prefix, term.boost, negative]);
            let seed = hash(&format!("{}:{}:{}", term.field, term.value, term.prefix));
            let mut out = Scores::new();
            for (position, id) in ids.iter().enumerate() {
                if (seed + hash(id) + position as u64).is_multiple_of(3) {
                    out.insert(id.clone(), term.boost + (seed % 5) as f64 / 4.0);
                }
            }
            if negative_only && negative {
                out.insert("extra".into(), 0.5);
            }
            Ok(out)
        };
        let result = evaluate(&parsed.tree, &parsed.terms, &universe, &mut resolve, false).unwrap();
        let rust = obj! {
            "result" => Value::Array(result.iter().map(|(id, score)| arr![id.as_str(), *score]).collect()),
            "trace" => Value::Array(trace),
        };
        compared += 1;
        assert_same(&Ok(rust.into()), js, &format!("evaluate({query:?}, {wildcard}, {negative_only})"));
    }
    assert!(compared > 2000);
}
