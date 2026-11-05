//! The sound a video sends for transcription: ffmpeg pulls it out as mono Ogg Opus, and a silent video has none.

use std::path::PathBuf;
use std::process::Command;

use mm_core::host::{MediaProcessor, MediaUser};
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

fn make_audio(file: &PathBuf, codec: &[&str]) {
    let mut args = vec!["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=440:duration=2"];
    args.extend(codec);
    let status = Command::new("ffmpeg").args(&args).arg(file).status().expect("ffmpeg runs");
    assert!(status.success(), "ffmpeg made the test audio");
}

#[test]
fn audio_files_become_audio_cards_with_their_length() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("skipped: ffmpeg is not installed");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("media-audio-files-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cases: [(&str, &[&str], &str, &str); 4] = [
        ("memo.m4a", &["-c:a", "aac"], "m4a", "audio/mp4"),
        ("song.mp3", &["-c:a", "libmp3lame"], "mp3", "audio/mpeg"),
        ("voice.ogg", &["-c:a", "libopus"], "ogg", "audio/ogg"),
        ("take.wav", &[], "wav", "audio/wav"),
    ];
    let processor = LocalProcessor::new(Some(dir.clone()));
    let user = MediaUser { account_uid: "acct".to_owned(), id_prefix: "card-".to_owned() };
    for (name, codec, extension, content_type) in cases {
        let file = dir.join(name);
        make_audio(&file, codec);
        let bytes = std::fs::read(&file).unwrap();
        mymind::runtime::run(async {
            let prepared = processor.prepare(Body::Bytes(bytes.clone()), "application/octet-stream", name, &user, "card-1").await.unwrap();
            let blob = prepared.data.get("blob");
            assert_eq!(prepared.data.get("type").as_str(), Some("Audio"), "{name}");
            assert_eq!(prepared.data.get("title").as_str(), Some(name));
            assert!(blob.get("path").as_str().unwrap().ends_with(&format!(".{extension}")), "{name}: {}", blob.get("path").to_js_string());
            assert_eq!(blob.get("type").as_str(), Some(content_type), "{name}");
            let length = blob.get("duration").to_number();
            assert!((1.9..2.2).contains(&length), "{name} lasts about two seconds: {length}");
            assert!(blob.get("width").is_nullish() && prepared.extras.get("cover").is_nullish(), "{name} has no picture");
            let sound = processor.audio(Body::Bytes(bytes.clone())).await.unwrap().expect("an audio file has sound");
            assert_eq!(&sound[..4], b"OggS");
        });
    }
    let video = dir.join("loud.mp4");
    make_video(&video, true);
    mymind::runtime::run(async {
        let prepared = processor.prepare(Body::Bytes(std::fs::read(&video).unwrap()), "video/mp4", "loud.mp4", &user, "card-2").await.unwrap();
        assert_eq!(prepared.data.get("type").as_str(), Some("Video"), "a video with sound stays a video");
        assert!(prepared.data.get("blob").get("duration").to_number() > 0.0);
    });
    let _ = std::fs::remove_dir_all(&dir);
}
