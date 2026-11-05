//! The sound a video sends for transcription: ffmpeg pulls it out as mono Ogg Opus, and a silent video has none.

use std::path::PathBuf;
use std::process::Command;

use mm_core::host::MediaProcessor;
use mm_core::http::Body;
use mymind::media_processor::LocalProcessor;

fn make_video(file: &PathBuf, sound: bool) {
    let mut args = vec!["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=black:s=64x64:d=1"];
    if sound {
        args.extend(["-f", "lavfi", "-i", "sine=frequency=440:duration=1", "-shortest"]);
    }
    args.extend(["-pix_fmt", "yuv420p"]);
    let status = Command::new("ffmpeg").args(&args).arg(file).status().expect("ffmpeg runs");
    assert!(status.success(), "ffmpeg made the test video");
}

#[test]
fn audio_is_extracted_from_videos_with_sound_only() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("skipped: ffmpeg is not installed");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("media-audio-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (loud, mute) = (dir.join("loud.mp4"), dir.join("mute.mp4"));
    make_video(&loud, true);
    make_video(&mute, false);
    let processor = LocalProcessor::new(Some(dir.clone()));
    mymind::runtime::run(async {
        let audio = processor.audio(Body::Bytes(std::fs::read(&loud).unwrap())).await.unwrap().expect("a video with sound has audio");
        assert_eq!(&audio[..4], b"OggS");
        assert!(audio.len() < 20_000, "one second of speech-quality Opus stays small: {} bytes", audio.len());
        let silent = processor.audio(Body::Bytes(std::fs::read(&mute).unwrap())).await.unwrap();
        assert!(silent.is_none(), "a video without a sound track has no audio");
    });
    let _ = std::fs::remove_dir_all(&dir);
}
