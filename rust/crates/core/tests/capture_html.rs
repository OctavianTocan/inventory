//! `capture_html` against `lib/capture-html.mjs` on recorded pages and generated markup.

mod common;

use common::{Random, assert_same, calls, root};
use mm_core::capture_html::{capture_title, detect_type, page_metadata, plain_text, profile_image, sanitize_post, source_url};
use mm_core::{Value, arr, obj};

const PIECES: &[&str] = &[
    "<meta property=\"og:title\" content=\"A &amp; B\">", "<meta name=\"description\" content='d'>", "<meta name=Author content=x>",
    "<meta property=\"og:image\" content=\"/i.png\">", "<meta property=\"og:image:secure_url\" content=\"https://a.b/c.png\">",
    "<meta name=\"twitter:image\" content=\"//cdn.x/y.jpg\">", "<meta property=\"og:type\" content=\"article\">",
    "<meta property=\"og:url\" content=\"https://e.com/u\">", "<meta property=\"og:site_name\" content=\"Site\">",
    "<link rel=\"canonical\" href=\"/canon\">", "<link rel=\"alternate canonical\" href=\"https://e.com/c2\">",
    "<link type=\"application/json+oembed\" href=\"https://e.com/o.json\">", "<title> T  &lt; U </title>",
    "<script type=\"application/ld+json\">{\"@type\":\"Article\",\"headline\":\"H\",\"image\":[\"/a.png\",{\"url\":\"/b.png\"}],\"author\":{\"@type\":\"Person\",\"name\":\"N\",\"image\":\"/p.png\"}}</script>",
    "<script type=\"application/ld+json\">[{\"@type\":[\"Product\"],\"name\":\"P\"},{\"@graph\":[{\"@type\":\"Person\",\"name\":\"Q\"}]}]</script>",
    "<script type=\"application/ld+json\">{bad json</script>", "<script>var a = '<p>x</p>';</script>", "<style>p{}</style>",
    "<article><p>Para <b>one</b></p><p>two &amp; three</p></article>", "<main><p>main</p></main>", "<nav>menu</nav>", "<p>loose</p>",
    "<p>", "</p>", "<!-- c -->", "<blockquote><p>q1</p><p>q2</p></blockquote>", "<a href=\"/rel\">l</a>", "<a href=\"javascript:x\">j</a>",
    "<a href=\"http://user:pw@h.com/\">c</a>", "<a href=\"https://access.mymind.com/x\">m</a>", "<a href=\"http://1.2.3.4/p\">ip</a>",
    "<a href=\"//1.2.3.4:80/p\">ip2</a>", "<a href=\"http://0x7f.1/\">hex</a>", "<br>", "<br/>", "<span>s</span>", "<strong>b", "</strong>",
    "<iframe>x</iframe>", "<svg><p>no</p></svg>", "<", ">", "&", "&amp;", "&#x41;", "&nbsp;", " ", "\n", "\t", "text", "é", "😀", "\u{2028}",
    "<p class=\"a\" id=b>attr</p>", "<img src=x>", "<A HREF='/Up'>U</A>", "<cite>c</cite><footer>f</footer>",
];

const URLS: &[&str] = &[
    "https://example.com/a/b", "http://www.example.co.uk/x?y=1#z", "https://x.com/u/status/123", "https://twitter.com/u/status/9",
    "https://github.com/o/r", "https://github.com/o/r.git", "https://github.com/o/r/issues", "https://www.youtube.com/watch?v=1",
    "https://youtu.be/abc", "https://threads.net/@u/post/1", "https://u.substack.com/note/c-1", "https://www.instagram.com/reel/abc/",
    "https://old.reddit.com/r/x/comments/1/t", "https://apps.apple.com/us/app/x/id1", "https://play.google.com/store/apps/details?id=x",
    "https://example.com/doc.PDF", "https://shop.example.com/p", "https://example.org/", "https://a.b.c.com.au/x", "https://pbs.twimg.com/profile_images/1/a.jpg",
    "https://avatars.githubusercontent.com/u/1", "https://lh3.googleusercontent.com/a/abc", "https://lh3.googleusercontent.com/a-/abc",
    "https://scontent.cdninstagram.com/v/t51.2885-19/1.jpg", "https://unavatar.io/x", "https://example.com/%E0%A4%A", "https://example.com/a%20b/avatar.png",
    "https://example.com/user_profile_pic.jpg", "https://example.com/profilepic", "not a url", "", "//x", "ftp://x.com/a", "https://access.mymind.com/x",
    "https://EXAMPLE.com./a", "http://1.2.3.4/x", "http://[::1]/x", "https://xn--e1afmkfd.xn--p1ai/", "https://ex ample.com/", "https://e.com/%zz",
];

fn generated(seed: u64, count: usize) -> Vec<String> {
    let mut random = Random(seed);
    let mut texts: Vec<String> = PIECES.iter().map(|piece| (*piece).to_owned()).collect();
    texts.push(String::new());
    texts.extend((0..count).map(|_| {
        let length = random.below(14) as usize + 1;
        random.text(PIECES, length)
    }));
    texts
}

fn recorded_pages() -> Vec<String> {
    let mut paths: Vec<_> = std::fs::read_dir(root().join("capture/details")).unwrap().map(|entry| entry.unwrap().path()).collect();
    paths.sort();
    paths.iter().step_by(15).filter_map(|path| std::fs::read_to_string(path).ok()).collect()
}

const MODULE: &str = "lib/capture-html.mjs";

#[test]
fn source_url_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(11);
    let mut cases: Vec<(Value, String)> = Vec::new();
    for raw in URLS.iter().chain(["/rel", "rel/x", "?q", "#f", "http://u:p@h/", "HTTPS://H.COM", "\\\\h.com\\p", "//h.com", "http:h.com", "mailto:a@b"].iter()) {
        for base in ["https://example.com/dir/page", "http://b.org/", "garbage", "https://access.mymind.com/"] {
            cases.push((Value::from(*raw), base.to_owned()));
        }
    }
    for _ in 0..1500 {
        let head = ["", "//", "http://", "https://", "/", "HTTP://", "\\\\"][random.below(7) as usize];
        let length = random.below(10) as usize;
        let raw = format!("{head}{}", random.text(&["1", ".", "2", "a", ":", "80", "@", "/", "?", "#", "x", "0x7f", "\\", "%41", "[", "]"], length));
        cases.push((Value::from(raw), "https://example.com/dir/page".to_owned()));
    }
    for value in [Value::Null, Value::Undefined, Value::from(0), Value::from(true), obj! { "url" => "/a" }.into(), obj! { "contentUrl" => "https://c.com/x" }.into(), obj! { "url" => Value::Null }.into(), obj! {}.into(), arr!["a"]] {
        cases.push((value, "https://example.com/".to_owned()));
    }
    let js = calls(MODULE, &cases.iter().map(|(value, base)| ("sourceUrl", vec![value.clone(), Value::from(base.as_str())])).collect::<Vec<_>>());
    for ((value, base), answer) in cases.iter().zip(&js) {
        assert_same(&Ok(source_url(value, base).map_or(Value::Undefined, Value::from)), answer, &format!("sourceUrl({value:?}, {base:?})"));
    }
}

#[test]
fn plain_text_matches_node() {
    let _goldens = common::test_guard();
    let mut texts = generated(3, 1500);
    texts.extend(recorded_pages());
    let js = calls(MODULE, &texts.iter().map(|text| ("plainText", vec![Value::from(text.as_str())])).collect::<Vec<_>>());
    for (text, answer) in texts.iter().zip(&js) {
        assert_same(&Ok(plain_text(text).into()), answer, &format!("plainText({:?})", text.chars().take(200).collect::<String>()));
    }
}

#[test]
fn sanitize_post_matches_node() {
    let _goldens = common::test_guard();
    let mut texts = generated(5, 2500);
    texts.extend(recorded_pages().into_iter().take(40));
    let js = calls(MODULE, &texts.iter().map(|text| ("sanitizePost", vec![Value::from(text.as_str()), Value::from("https://example.com/dir/")])).collect::<Vec<_>>());
    for (text, answer) in texts.iter().zip(&js) {
        assert_same(&Ok(sanitize_post(text, "https://example.com/dir/").into()), answer, &format!("sanitizePost({:?})", text.chars().take(200).collect::<String>()));
    }
}

#[test]
fn page_metadata_matches_node() {
    let _goldens = common::test_guard();
    let mut cases: Vec<(String, &str)> = generated(9, 2500).into_iter().map(|text| (text, "https://www.example.com/dir/page")).collect();
    cases.extend(recorded_pages().into_iter().map(|text| (text, "https://example.org/a/b?c=d")));
    cases.push((String::new(), "not a url"));
    cases.push(("<meta property=\"og:image\" content=\"/i.png\">".to_owned(), "garbage"));
    let js = calls(MODULE, &cases.iter().map(|(html, url)| ("pageMetadata", vec![Value::from(html.as_str()), Value::from(*url)])).collect::<Vec<_>>());
    for ((html, url), answer) in cases.iter().zip(&js) {
        assert_same(&page_metadata(html, url).map(|metadata| metadata.to_value()), answer, &format!("pageMetadata({:?}, {url})", html.chars().take(200).collect::<String>()));
    }
}

#[test]
fn capture_title_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(21);
    let titles = ["Hello - Site", "Hello | Example", "Hello \u{2014} Example Com", "Hello - Other", "Plain", "A - B - Example", "Hello  -  example", " - Example", "Foo - Apps on Google Play", "Bar on the App Store ", "", "T | www.example.com", "X \u{2013} Example"];
    let sites = ["Site", "Example", "example.com", "Exa-mple", "", "\u{c9}xample", "\u{1d4d0}"];
    let types = ["Repository", "SoftwareApplication", "Article", "WebPage", "Product", "XPost", "Document"];
    let entity_sets: Vec<Value> = vec![
        arr![],
        arr![obj! { "@type" => "SoftwareApplication", "name" => "Cool App - Apps on Google Play" }],
        arr![obj! { "@type" => arr!["Thing", "SoftwareApplication"], "name" => "Cool App on the App Store" }],
        arr![obj! { "@type" => "SoftwareApplication" }],
        arr![obj! { "@type" => "Other", "name" => "n" }],
    ];
    let mut cases = Vec::new();
    for _ in 0..2500 {
        let canonical = URLS[random.below(URLS.len() as u64) as usize];
        let metadata = obj! {
            "canonical" => canonical,
            "siteName" => sites[random.below(sites.len() as u64) as usize],
            "entities" => entity_sets[random.below(entity_sets.len() as u64) as usize].clone(),
        };
        cases.push((titles[random.below(titles.len() as u64) as usize].to_owned(), types[random.below(types.len() as u64) as usize], Value::from(metadata)));
    }
    cases.push(("t".to_owned(), "Article", Value::from(obj! { "canonical" => "https://e.com/" })));
    cases.push(("t".to_owned(), "Document", Value::from(obj! {})));
    cases.push(("t".to_owned(), "Repository", Value::from(obj! { "canonical" => "https://github.com/", "entities" => arr![] })));
    let js = calls(MODULE, &cases.iter().map(|(title, kind, metadata)| ("captureTitle", vec![Value::from(title.as_str()), Value::from(*kind), metadata.clone()])).collect::<Vec<_>>());
    for ((title, kind, metadata), answer) in cases.iter().zip(&js) {
        assert_same(&capture_title(title, kind, metadata).map(Value::from), answer, &format!("captureTitle({title:?}, {kind}, {metadata:?})"));
    }
}

#[test]
fn profile_image_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(31);
    let mut values: Vec<String> = URLS.iter().map(|url| (*url).to_owned()).collect();
    for _ in 0..2000 {
        let host = ["example.com", "avatar.cdn.com", "www.unavatar.io", "x.googleusercontent.com", "pbs.twimg.com"][random.below(5) as usize];
        let length = random.below(9) as usize;
        values.push(format!("https://{host}/{}", random.text(&["profile", "_", "-", ".", "/", "images", "image", "pic", "s", "avatar", "a", "t51.1-19", "%41", "%E0", "banner", "x"], length)));
    }
    let js = calls(MODULE, &values.iter().map(|value| ("profileImage", vec![Value::from(value.as_str())])).collect::<Vec<_>>());
    for (value, answer) in values.iter().zip(&js) {
        assert_same(&profile_image(value).map(Value::from), answer, &format!("profileImage({value:?})"));
    }
}

#[test]
fn detect_type_matches_node() {
    let _goldens = common::test_guard();
    let mut random = Random(41);
    let og = [Value::Undefined, arr!["article"], arr!["website"], arr!["x", "article:foo"]];
    let entities = [
        arr![],
        arr![obj! { "@type" => "Product" }],
        arr![obj! { "@type" => "https://schema.org/Product" }],
        arr![obj! { "@type" => arr!["NewsArticle"] }],
        arr![obj! { "@type" => "BlogPosting" }],
        arr![obj! { "@type" => "Thing" }, obj! { "@type" => "SoftwareApplication" }],
    ];
    let mimes = ["", "application/pdf", "text/html", "x/application/pdf;y"];
    let mut cases = Vec::new();
    for url in URLS {
        for _ in 0..6 {
            let mut metadata = obj! { "entities" => entities[random.below(entities.len() as u64) as usize].clone() };
            let og_type = og[random.below(og.len() as u64) as usize].clone();
            if !matches!(og_type, Value::Undefined) {
                metadata.insert("meta", Value::from(obj! { "og:type" => og_type }));
            }
            cases.push((*url, Value::from(metadata), mimes[random.below(mimes.len() as u64) as usize]));
        }
        cases.push((*url, Value::from(obj! {}), ""));
    }
    let js = calls(MODULE, &cases.iter().map(|(url, metadata, mime)| ("detectType", vec![Value::from(*url), metadata.clone(), Value::from(*mime)])).collect::<Vec<_>>());
    for ((url, metadata, mime), answer) in cases.iter().zip(&js) {
        assert_same(&detect_type(url, metadata, mime).map(Value::from), answer, &format!("detectType({url:?}, {metadata:?}, {mime:?})"));
    }
}
