//! `mm_js::url` against Node's `URL`.

mod common;

use mm_core::{Value, arr, obj};
use mm_js::url;

const DRIVER: &str = "rust/crates/core/tests/drivers/url.mjs";

fn parsed(input: &str, base: Option<&str>) -> Value {
    let Some(url) = url::parse(input, base) else { return Value::Null };
    let params: Vec<Value> = url.query_pairs().map(|(key, value)| arr![key.into_owned(), value.into_owned()]).collect();
    obj! {
        "href" => url.as_str(),
        "origin" => url::origin(&url),
        "protocol" => format!("{}:", url.scheme()),
        "host" => url::host(&url),
        "hostname" => url.host_str().unwrap_or(""),
        "port" => url.port().map(|port| port.to_string()).unwrap_or_default(),
        "pathname" => url.path(),
        "search" => url::search(&url),
        "hash" => url::hash(&url),
        "params" => params,
    }
    .into()
}

#[test]
fn urls_parse_as_node_parses_them() {
    let bases = [None, Some("http://localhost:4321"), Some("http://localhost:4321/a/b?c#d"), Some("https://access.mymind.com/"), Some("http://undefined"), Some("http://[::1]:80"), Some("http://Example.COM"), Some("http://a:b:c"), Some("http://localhost:99999")];
    let parts = [
        "/", "a", "%", "2", "F", "e9", "^", "|", "{", "}", "`", "\"", "'", "<", ">", " ", "?", "#", "&", "=", "+", ".", "..", "\\", "é",
        "\t", "\n", ":", "@", "[", "]", "~", "%zz", "%2e", "%2E%2e", "😀", "\u{0}", "\u{7f}", ";", ",", "*", "!", "$", "(", ")",
    ];
    let starts = ["", "/", "//", "http://", "https://", "HTTP://x.Y/", "https://media.mymind.host/", "file:///", "mailto:", "data:", "javascript:", "x:", "http://user:pw@h/", "http://1.2.3.4/", "http://0x7f.1/", "http://[::ffff:1.2.3.4]/", "http://xn--caf-dma.com/", "http://café.com/"];
    let mut random = common::Random(0x0072_6c75_7261);
    let mut cases: Vec<(String, Option<&str>)> = Vec::new();
    for _ in 0..30_000 {
        let start = starts[random.below(starts.len() as u64) as usize];
        let length = random.below(12) as usize;
        let input = format!("{start}{}", random.text(&parts, length));
        cases.push((input, bases[random.below(bases.len() as u64) as usize]));
    }
    let calls: Vec<(&str, Vec<Value>)> = cases.iter().map(|(input, base)| ("parse", vec![Value::from(input.as_str()), base.map_or(Value::Null, Value::from)])).collect();
    let mut differences = Vec::new();
    for ((input, base), answer) in cases.iter().zip(common::calls(DRIVER, &calls)) {
        let rust = parsed(input, *base);
        let node = answer.as_ref().unwrap();
        // Known difference: the `url` crate drops a `file:` path's leading empty segment (`file:////x`), which the
        // backend never parses.
        if input.starts_with("file:") && node.get("pathname").as_str().is_some_and(|path| path.starts_with("//")) {
            continue;
        }
        if common::text(&rust) != common::text(node) {
            differences.push(format!("{input:?} on {base:?}\n  node {}\n  rust {}", common::text(node), common::text(&rust)));
        }
    }
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.iter().take(25).cloned().collect::<Vec<_>>().join("\n"));
}
