//! The text modules (`render/encoding`, `capture-entities`, `device`) against the JS backend on generated input.

mod common;

use common::{Random, assert_same, calls};
use mm_core::render::encoding::{escape_html, unescape_html};
use mm_core::{Value, capture_entities::decode_entities, device::device_variant};

const PIECES: &[&str] = &[
    "a", "Z", "0", "9", " ", "&", ";", "#", "x", "X", "<", ">", "\"", "'", "+", "\n", "\t", "\u{7f}", "é", "😀", "\u{a0}",
    "amp", "AMP", "lt", "Gt", "quot", "apos", "nbsp", "NBSP", "copy", "hellip", "QUOT", "notin", "zwj", "#x41", "#X1F600",
    "#65", "#0", "#xD800", "#x110000", "#1114111", "#99999999999", "#x", "&#", "&amp;", "&#x27;", "&nbsp;", "&eacute;",
    "&Eacute;", "&foo;", "&a;", "&ab;", "&#xffff;", "&#12a;", "f", "1",
];

fn generated(seed: u64, count: usize) -> Vec<String> {
    let mut random = Random(seed);
    let mut texts: Vec<String> = PIECES.iter().map(|piece| (*piece).to_owned()).collect();
    texts.extend((0..count).map(|_| {
        let length = random.below(12) as usize;
        random.text(PIECES, length)
    }));
    texts
}

#[test]
fn encoding_matches_node() {
    let _goldens = common::test_guard();
    let texts = generated(7, 4000);
    let js = calls(
        "lib/render/encoding.mjs",
        &texts.iter().flat_map(|text| [("escapeHtml", vec![Value::from(text)]), ("unescapeHtml", vec![Value::from(text)])]).collect::<Vec<_>>(),
    );
    for (index, text) in texts.iter().enumerate() {
        assert_same(&Ok(escape_html(text).into()), &js[index * 2], &format!("escapeHtml({text:?})"));
        assert_same(&unescape_html(text).map(Value::from), &js[index * 2 + 1], &format!("unescapeHtml({text:?})"));
    }
}

#[test]
fn entities_match_node() {
    let _goldens = common::test_guard();
    let texts = generated(11, 4000);
    let js = calls("lib/capture-entities.mjs", &texts.iter().map(|text| ("decodeEntities", vec![Value::from(text)])).collect::<Vec<_>>());
    for (text, js) in texts.iter().zip(&js) {
        assert_same(&Ok(decode_entities(text).into()), js, &format!("decodeEntities({text:?})"));
    }
}

#[test]
fn device_variants_match_node() {
    let _goldens = common::test_guard();
    let agents = [
        "", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)", "iPod touch", "Mozilla/5.0 (iPad; CPU OS 17_0)",
        "Mozilla/5.0 (Linux; Android 14; Pixel 8) Mobile Safari", "Mozilla/5.0 (Linux; Android 14; Tab)", "Android\nMobile",
        "Android\u{2028}Mobile", "Mobile Android", "Android Android\nMobile Mobile", "Mozilla/5.0 (X11; Linux x86_64)",
    ];
    let js = calls("lib/device.mjs", &agents.iter().map(|agent| ("deviceVariant", vec![Value::from(*agent)])).collect::<Vec<_>>());
    for (agent, js) in agents.iter().zip(&js) {
        assert_same(&Ok(device_variant(agent).into()), js, &format!("deviceVariant({agent:?})"));
    }
}
