//! Inventory's own `time(s)` rendition modifier: a still taken that far into a video.

use mm_core::media_files::{Rendition, parse_rendition};

fn time_of(path: &str) -> Option<f64> {
    match parse_rendition(path).expect(path) {
        Rendition::Derived { time, .. } => time,
        other => panic!("{path} is not a still: {other:?}"),
    }
}

#[test]
fn time_names_a_moment_of_a_video() {
    assert_eq!(time_of("u/c/v.mp4;time(12.5)/768x768-t.jpg"), Some(12.5));
    assert_eq!(time_of("u/c/v.mp4;time(0)/128x128.png"), Some(0.0));
    assert_eq!(time_of("u/c/v.mp4;time(86400)/128x128.png"), Some(86_400.0));
    assert_eq!(time_of("u/c/v.mp4;768x768-t.jpg"), None);
    let value = parse_rendition("u/c/v.mp4;time(3.25)/64x64.jpg").unwrap().to_value();
    assert_eq!(value.get("time").to_number(), 3.25);
    assert!(parse_rendition("u/c/v.mp4;64x64.jpg").unwrap().to_value().get("time").is_undefined());
}

#[test]
fn time_out_of_bounds_or_on_a_video_rendition_is_refused() {
    for path in ["v.mp4;time(86400.5)/64x64.jpg", "v.mp4;time(1)/64x64/hevc.mp4", "v.mp4;time(1)"] {
        assert!(parse_rendition(path).is_err(), "{path}");
    }
    // Malformed times are not read as times, so the still is the first frame.
    for path in ["v.mp4;time(-1)/64x64.jpg", "v.mp4;time(1.2345)/64x64.jpg", "v.mp4;time(abc)/64x64.jpg"] {
        assert_eq!(time_of(path), None, "{path}");
    }
}

#[test]
fn info_names_a_files_length_and_size() {
    assert_eq!(parse_rendition("u/c/v.mp4;info.json").unwrap(), Rendition::Info { source: "u/c/v.mp4".to_owned() });
    assert!(parse_rendition("u/c/v.mp4;info.png").is_err());
}
