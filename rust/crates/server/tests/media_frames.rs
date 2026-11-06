//! Stills from inside a video (`time(s)` renditions), which Inventory's MCP tools show agents.

use std::path::{Path, PathBuf};
use std::process::Command;

use mm_core::host::{MediaProcessor, RenditionSource};
use mm_core::http::Body;
use mymind::media_processor::LocalProcessor;

/// The red, green and blue levels of a 1x1 picture.
fn pixel(picture: &Path) -> [u8; 3] {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(picture)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .expect("ffmpeg runs");
    [output.stdout[0], output.stdout[1], output.stdout[2]]
}

#[test]
fn a_time_rendition_is_the_frame_at_that_moment() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("skipped: ffmpeg is not installed");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("media-frames-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let video = dir.join("two.mp4");
    // One second of red, then one of blue.
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=red:s=64x64:d=1:r=10", "-f", "lavfi", "-i", "color=c=blue:s=64x64:d=1:r=10"])
        .args(["-filter_complex", "[0][1]concat=n=2:v=1", "-pix_fmt", "yuv420p"])
        .arg(&video)
        .status()
        .expect("ffmpeg runs");
    assert!(status.success(), "ffmpeg made the test video");
    let bytes = std::fs::read(&video).unwrap();
    let processor = LocalProcessor::new(Some(dir.clone()));
    let still = |suffix: &str| {
        let source = RenditionSource { body: Body::Bytes(bytes.clone()), size: bytes.len() as u64, name: "acct/card/two.mp4".to_owned() };
        let relative = format!("acct/card/two.mp4;{suffix}");
        let made = mymind::runtime::run(async { processor.rendition(source, &relative).await.unwrap().expect("a rendition") });
        let Body::Bytes(data) = made.body else { panic!("a rendition has bytes") };
        let file = dir.join(format!("{}.png", suffix.replace(['/', '(', ')'], "_")));
        std::fs::write(&file, data).unwrap();
        pixel(&file)
    };
    let [red, _, blue] = still("1x1.png");
    assert!(red > 200 && blue < 60, "the plain still is the first frame (red): {red} {blue}");
    let [red, _, blue] = still("time(1.5)/1x1.png");
    assert!(blue > 200 && red < 60, "time(1.5) is in the blue second: {red} {blue}");
    let [red, _, blue] = still("time(0.2)/1x1.png");
    assert!(red > 200 && blue < 60, "time(0.2) is in the red second: {red} {blue}");
    let _ = std::fs::remove_dir_all(&dir);
}
