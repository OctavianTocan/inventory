//! Recording HTML rules and link lifecycle assertions formerly run by `tools/check-render.mjs`.

#[path = "support/recordings.rs"]
mod recordings;

use mm_core::render_card::render_card;
use mm_core::{Value, obj};
use mm_js::json;
use recordings::{Recording, recording_extras};

const TYPE_COUNTS: &[(&str, usize)] = &[
    ("WebPage", 341),
    ("XPost", 934),
    ("Article", 174),
    ("YouTubeVideo", 26),
    ("Repository", 281),
    ("Video", 77),
    ("Note", 31),
    ("Image", 141),
    ("Screenshot", 215),
    ("InstagramReel", 36),
    ("SoftwareApplication", 3),
    ("Product", 4),
    ("Document", 3),
    ("ThreadsPost", 4),
    ("Placeholder", 2),
    ("SubstackNote", 1),
    ("Content", 2),
    ("RedditPost", 1),
];

#[derive(Debug, PartialEq)]
struct Options {
    kind: Option<String>,
    show: usize,
}

// Parse the original supported flags, rejecting missing values and unknown options.
fn options(args: &[&str]) -> Result<Options, &'static str> {
    let mut result = Options {
        kind: None,
        show: 3,
    };
    let mut at = 0;
    while at < args.len() {
        match args[at] {
            "--type"
                if args
                    .get(at + 1)
                    .is_some_and(|kind| !kind.is_empty() && !kind.starts_with("--")) =>
            {
                at += 1;
                result.kind = Some(args[at].into());
            }
            "--show"
                if args.get(at + 1).is_some_and(|value| {
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                }) =>
            {
                at += 1;
                result.show = args[at].parse().unwrap_or(usize::MAX);
            }
            _ => return Err("Usage: check_render [--type X] [--show N]"),
        }
        at += 1;
    }
    Ok(result)
}

struct Report {
    counts: Vec<(String, usize, usize)>,
    failures: Vec<String>,
}

// Name the first unequal line and byte without logging private markup or prose.
fn difference(expected: &str, actual: &str) -> String {
    let offset = expected
        .bytes()
        .zip(actual.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(expected.len().min(actual.len()));
    let expected_lines = expected.split('\n').collect::<Vec<_>>();
    let actual_lines = actual.split('\n').collect::<Vec<_>>();
    let line = (0..expected_lines.len().max(actual_lines.len()))
        .find(|index| expected_lines.get(*index) != actual_lines.get(*index))
        .map_or(0, |index| index + 1);
    format!(
        "line {line}, byte {offset}, expected {} bytes, actual {} bytes",
        expected.len(),
        actual.len()
    )
}

// Generate each selected card independently and preserve literal recorded HTML equality.
fn check_cards(cards: &[Recording], config: &Options) -> Result<Report, String> {
    let mut report = Report {
        counts: Vec::new(),
        failures: Vec::new(),
    };
    for (index, recording) in cards.iter().enumerate() {
        let data = json::parse(&recording.json).map_err(|_| "invalid recorded card JSON")?;
        let kind = data
            .get("type")
            .as_str()
            .ok_or("recorded card type missing")?;
        if config
            .kind
            .as_deref()
            .is_some_and(|selected| selected != kind)
        {
            continue;
        }
        let at = report
            .counts
            .iter()
            .position(|(name, ..)| name == kind)
            .unwrap_or_else(|| {
                report.counts.push((kind.into(), 0, 0));
                report.counts.len() - 1
            });
        report.counts[at].2 += 1;
        let rendered = recording_extras(&recording.html, &recording.json)
            .and_then(|extras| render_card(&data, &extras));
        match rendered {
            Ok(actual) if actual == recording.html => report.counts[at].1 += 1,
            Ok(actual) => report.failures.push(format!(
                "{kind} recording #{}: {}",
                index + 1,
                difference(&recording.html, &actual)
            )),
            Err(_) => report
                .failures
                .push(format!("{kind} recording #{}: rendering threw", index + 1)),
        }
    }
    if config.kind.is_none() {
        let recording = recordings::link_save_recording();
        let changes = recording
            .get("changes")
            .as_array()
            .expect("link lifecycle changes");
        let mut passed = 0;
        for (stage, change) in [1, 2, 4].iter().zip(changes) {
            let data = json::parse(
                change
                    .get("cardData")
                    .as_str()
                    .expect("lifecycle card JSON"),
            )
            .expect("valid lifecycle JSON");
            let extras: Value = obj! { "id" => change.get("object").get("id").clone(), "capturePending" => true, "screenshotPending" => data.get("type").as_str() == Some("WebPage") }.into();
            let expected = change.get("cardHtml").as_str().expect("lifecycle markup");
            match render_card(&data, &extras) {
                Ok(actual) if actual == expected => passed += 1,
                Ok(actual) => report.failures.push(format!(
                    "link save stage {stage}: {}",
                    difference(expected, &actual)
                )),
                Err(_) => report
                    .failures
                    .push(format!("link save stage {stage}: rendering threw")),
            }
        }
        report
            .counts
            .push(("Link save lifecycle".into(), passed, changes.len()));
    }
    if report.counts.is_empty() {
        return Err("No recorded cards have selected type".into());
    }
    Ok(report)
}

// Fail the test even when diagnostic output is hidden by the original `--show 0` option.
fn passed(report: &Report, show: usize) {
    assert!(
        report.failures.is_empty(),
        "{}/{} render assertions failed: {}",
        report.failures.len(),
        report
            .counts
            .iter()
            .map(|(_, _, total)| total)
            .sum::<usize>(),
        report
            .failures
            .iter()
            .take(show)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ")
    );
}

#[test]
fn every_recorded_card_and_each_link_save_stage_have_exact_original_html() {
    let cards = recordings::captured_cards();
    let report = check_cards(&cards, &options(&[]).expect("default options"))
        .expect("recorded render cases");
    assert_eq!(cards.len(), 2276, "complete original captured corpus");
    assert_eq!(
        report.counts.len(),
        TYPE_COUNTS.len() + 1,
        "all original types and lifecycle"
    );
    for (kind, expected) in TYPE_COUNTS {
        assert_eq!(
            report
                .counts
                .iter()
                .find(|(name, ..)| name == kind)
                .expect("recorded type")
                .2,
            *expected,
            "complete {kind} corpus"
        );
    }
    assert_eq!(
        report
            .counts
            .iter()
            .map(|(_, _, count)| count)
            .sum::<usize>(),
        2279,
        "2276 cards plus all three lifecycle stages"
    );
    passed(&report, 3);
}

#[test]
fn recorded_link_save_stages_preserve_exact_bytes_flags_and_pending_rendering() {
    let report = check_cards(&[], &options(&[]).expect("default options"))
        .expect("recorded lifecycle cases");
    assert_eq!(report.counts.len(), 1, "only lifecycle requested");
    assert_eq!(report.counts[0].2, 3, "all three recorded changes");
    passed(&report, 3);
}

// Keep Cargo's per-type test filters equivalent to the original `--type` selection.
macro_rules! recorded_type {
    ($name:ident, $kind:literal, $count:literal) => {
        #[test]
        fn $name() {
            let config = options(&["--type", $kind]).expect("type option");
            let report = check_cards(&recordings::captured_cards(), &config)
                .expect("selected recording type");
            assert_eq!(
                report.counts.len(),
                1,
                "type filter excludes lifecycle and other types"
            );
            assert_eq!(report.counts[0].2, $count, "complete selected corpus");
            passed(&report, config.show);
        }
    };
}

recorded_type!(recorded_type_webpage, "WebPage", 341);
recorded_type!(recorded_type_xpost, "XPost", 934);
recorded_type!(recorded_type_article, "Article", 174);
recorded_type!(recorded_type_youtube_video, "YouTubeVideo", 26);
recorded_type!(recorded_type_repository, "Repository", 281);
recorded_type!(recorded_type_video, "Video", 77);
recorded_type!(recorded_type_note, "Note", 31);
recorded_type!(recorded_type_image, "Image", 141);
recorded_type!(recorded_type_screenshot, "Screenshot", 215);
recorded_type!(recorded_type_instagram_reel, "InstagramReel", 36);
recorded_type!(recorded_type_software_application, "SoftwareApplication", 3);
recorded_type!(recorded_type_product, "Product", 4);
recorded_type!(recorded_type_document, "Document", 3);
recorded_type!(recorded_type_threads_post, "ThreadsPost", 4);
recorded_type!(recorded_type_placeholder, "Placeholder", 2);
recorded_type!(recorded_type_substack_note, "SubstackNote", 1);
recorded_type!(recorded_type_content, "Content", 2);
recorded_type!(recorded_type_reddit_post, "RedditPost", 1);

#[test]
fn original_option_rejections_and_unknown_type_failure_are_preserved() {
    assert_eq!(
        options(&[]).unwrap(),
        Options {
            kind: None,
            show: 3
        }
    );
    assert_eq!(
        options(&["--show", "0", "--type", "Image"]).unwrap(),
        Options {
            kind: Some("Image".into()),
            show: 0
        }
    );
    for args in [
        vec!["--type"],
        vec!["--show"],
        vec!["--type", "--show", "1"],
        vec!["--show", "-1"],
        vec!["--show", "1.5"],
        vec!["--type=Image"],
        vec!["--typo"],
    ] {
        assert!(
            options(&args).is_err(),
            "invalid or missing option value must fail"
        );
    }
    let unknown = options(&["--type", "UnrecordedType"]).unwrap();
    assert!(
        check_cards(&recordings::captured_cards(), &unknown).is_err(),
        "unrecorded type must fail instead of checking nothing"
    );
    let bad = Recording {
        html: "<mymind-card data-id=\"synthetic\">wrong</mymind-card>".into(),
        json: "{\"type\":\"Note\"}".into(),
    };
    let report = check_cards(
        &[bad],
        &options(&["--type", "Note", "--show", "0"]).unwrap(),
    )
    .unwrap();
    assert_eq!(
        report.failures.len(),
        1,
        "zero diagnostic limit still records failed equality"
    );
}

#[test]
fn exact_html_comparison_keeps_whitespace_and_trailing_bytes() {
    assert!(difference("a\n b", "a\nb").starts_with("line 2, byte 2"));
    assert!(difference("a", "a\n").starts_with("line 2, byte 1"));
    let card = Recording {
        html: "<mymind-card data-id=\"synthetic\"></mymind-card>\n".into(),
        json: "{\"type\":\"Note\"}".into(),
    };
    let mut exact = card.clone();
    exact.html = render_card(
        &json::parse(&card.json).unwrap(),
        &recording_extras(&card.html, &card.json).unwrap(),
    )
    .unwrap();
    let config = options(&["--type", "Note"]).unwrap();
    assert!(
        check_cards(&[exact.clone()], &config)
            .unwrap()
            .failures
            .is_empty()
    );
    exact.html.push('\n');
    assert_eq!(
        check_cards(&[exact], &config).unwrap().failures.len(),
        1,
        "recorded HTML stays byte-exact"
    );
}

#[test]
fn extras_recover_entity_metadata_without_copying_layout_or_unproven_titles() {
    let clipped = "a".repeat(200);
    let data = json::to_string(&obj! { "type" => "Article", "title" => clipped.as_str(), "source" => obj! { "url" => "https://example.invalid/original" } }.into());
    let html = format!(
        "<mymind-card data-id=\"a&amp;b\"><a class=\"link-bubble\" href=\"https://example.invalid/other?a=1&amp;b=2\"><h3>{clipped} beyond</h3><mymind-image class=\"svg\" style=\"aspect-ratio:8/5;background-color:#abc;\"><img data-src=\"/media;400x250.webp\" width=\"400\" height=\"250\"></mymind-image><div class=\"recorded-layout\">private layout</div>"
    );
    let extras = recording_extras(&html, &data).unwrap();
    assert_eq!(extras.get("id").as_str(), Some("a&b"));
    assert_eq!(
        extras.get("linkUrl").as_str(),
        Some("https://example.invalid/other?a=1&b=2")
    );
    assert_eq!(
        extras.get("title").as_str(),
        Some(format!("{clipped} beyond").as_str())
    );
    assert_eq!(extras.get("cover").get("width").as_f64(), Some(400.0));
    assert_eq!(extras.get("cover").get("height").as_f64(), Some(250.0));
    assert_eq!(
        extras.get("cover").get("dominantColor").as_str(),
        Some("#abc")
    );
    assert_eq!(extras.get("cover").get("svg").as_bool(), Some(true));
    assert!(!json::to_string(&extras).contains("recorded-layout"));
    let unmatched = html.replace(
        &format!("<h3>{clipped} beyond</h3>"),
        "<h3>unmatched title</h3>",
    );
    assert!(
        recording_extras(&unmatched, &data)
            .unwrap()
            .get("title")
            .is_undefined()
    );
    let short = json::to_string(&obj! { "type" => "Article", "title" => "a".repeat(199) }.into());
    assert!(
        recording_extras(&html, &short)
            .unwrap()
            .get("title")
            .is_undefined()
    );
    for kind in ["Video", "Document", "Note", "Content", "Repository"] {
        assert!(
            recording_extras(&html, &format!("{{\"type\":\"{kind}\"}}"))
                .unwrap()
                .get("cover")
                .is_undefined(),
            "{kind} does not reuse layout cover"
        );
    }
}

#[test]
fn extras_cover_image_thumbnail_social_app_product_and_reddit_rules() {
    let image = "<mymind-card data-id=\"synthetic\"><mymind-image style=\"aspect-ratio:8/5;background-color:#abc;\"><img data-src=\"/image.webp\"></mymind-image>";
    let data = "{\"type\":\"Image\",\"blob\":{\"dominantColor\":\"#abc\"}}";
    assert!(
        recording_extras(image, data)
            .unwrap()
            .get("cover")
            .is_undefined(),
        "matching Image blob owns its rendition"
    );
    assert!(
        recording_extras(&image.replace("/image.webp", "/image-t.webp"), data)
            .unwrap()
            .get("cover")
            .is_object(),
        "thumbnail uses selected entity cover"
    );
    assert!(
        recording_extras(image, &data.replace("#abc", "#def"))
            .unwrap()
            .get("cover")
            .is_object(),
        "different selected cover color overrides blob"
    );
    assert!(
        recording_extras("<mymind-card>", data).is_err(),
        "missing Image cover preserves thrown-error behavior"
    );
    let social = format!(
        "{image}<i class=\"platform\"></i>\n\n    <b>source rich body</b>\n\n    <p class=\"author\">A &amp; B</p><div class=\"fake-player\"></div><div class=\"gallery\"><img class=\"lazy\" data-srcset=\"/a&amp;b 2x\" width=\"20\" height=\"10\"/></div></div>"
    );
    for kind in ["XPost", "ThreadsPost", "SubstackNote"] {
        let extras = recording_extras(&social, &format!("{{\"type\":\"{kind}\"}}")).unwrap();
        assert_eq!(
            extras.get("post").get("bodyHtml").as_str(),
            Some("<b>source rich body</b>")
        );
        assert_eq!(extras.get("post").get("author").as_str(), Some("A & B"));
        assert_eq!(extras.get("post").get("hasVideo").as_bool(), Some(true));
        assert_eq!(
            extras
                .get("post")
                .get("gallery")
                .at(0)
                .get("srcset")
                .as_str(),
            Some("/a&b 2x")
        );
    }
    let reel = "<img class=\"lazy main-img\" data-src=\"/reel\" width=\"20\" height=\"30\">";
    let extras = recording_extras(reel, "{\"type\":\"InstagramReel\"}").unwrap();
    assert_eq!(extras.get("cover").get("isThumbnail").as_bool(), Some(true));
    assert_eq!(
        recording_extras(
            &format!("instagram-reel not-thumbnail{reel}"),
            "{\"type\":\"InstagramReel\"}"
        )
        .unwrap()
        .get("cover")
        .get("isThumbnail")
        .as_bool(),
        Some(false)
    );
    let app = recording_extras("<div class=\"inner distributor_apple\"><img srcset=\"/a&amp;b 2x\" width=\"20\" height=\"30\">", "{\"type\":\"SoftwareApplication\"}").unwrap();
    assert_eq!(app.get("app").get("distributor").as_str(), Some("apple"));
    assert_eq!(app.get("app").get("srcset").as_str(), Some("/a&b 2x"));
    assert_eq!(app.get("app").get("width").as_f64(), Some(20.0));
    assert_eq!(
        recording_extras(
            "<span class=\"price\">&pound; &amp; 10</span>",
            "{\"type\":\"Product\"}"
        )
        .unwrap()
        .get("product")
        .get("price")
        .as_str(),
        Some("&pound; & 10")
    );
    let reddit = recording_extras(
        "<h3>A &amp; B</h3><div class=\"label\">r/example</div>",
        "{\"type\":\"RedditPost\"}",
    )
    .unwrap();
    assert_eq!(reddit.get("reddit").get("title").as_str(), Some("A & B"));
    assert_eq!(
        reddit.get("reddit").get("subreddit").as_str(),
        Some("r/example")
    );
}
