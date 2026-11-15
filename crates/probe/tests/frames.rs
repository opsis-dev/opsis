use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use opsis_golden::{dev_cache, ffmpeg};
use opsis_probe::video_frames;
use opsis_types::numbers;

const VIDEO: &str = "testsrc2=size=320x240:rate=30:duration=2";
const SOUND: &str = "sine=frequency=440:sample_rate=48000:duration=2";
const US_PER_S: f64 = 1_000_000.0;

const SAMPLES: &[(&str, &[&str], &[&str])] = &[
    (
        "frames-h264-aac.mp4",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac"],
    ),
    (
        "frames-h264-only.mp4",
        &[VIDEO],
        &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-bf", "3"],
    ),
    (
        "frames-fragmented.mp4",
        &[VIDEO, SOUND],
        &[
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-movflags",
            "frag_keyframe+empty_moov",
        ],
    ),
    (
        "frames-uneven.mp4",
        &["testsrc2=size=320x240:rate=30:duration=2,setpts='N/30/TB+0.011*mod(N,3)/TB'"],
        &[
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-fps_mode",
            "passthrough",
        ],
    ),
    (
        "frames-h264-opus.mkv",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "libopus"],
    ),
    (
        "frames-vp9.webm",
        &[VIDEO],
        &["-c:v", "libvpx-vp9", "-pix_fmt", "yuv420p"],
    ),
];

static MAKING: Mutex<()> = Mutex::new(());

fn sample(name: &str, inputs: &[&str], codec: &[&str]) -> PathBuf {
    let _making = MAKING.lock().unwrap();
    let dir = dev_cache("probe-frames");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    if !path.exists() {
        let partial = dir.join(format!("partial-{name}"));
        let mut args = vec!["-hide_banner", "-loglevel", "error", "-nostdin", "-y"];
        for input in inputs {
            args.extend(["-f", "lavfi", "-i", input]);
        }
        args.extend_from_slice(codec);
        let target = partial.to_string_lossy().into_owned();
        args.push(&target);
        let out = Command::new(ffmpeg()).args(&args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        std::fs::rename(&partial, &path).unwrap();
    }
    path
}

fn decoded_times(file: &PathBuf) -> Vec<i64> {
    let out = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-nostdin",
            "-copyts",
            "-start_at_zero",
            "-i",
        ])
        .arg(file)
        .args(["-map", "0:v:0", "-an", "-vf", "showinfo", "-f", "null", "-"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.contains("Parsed_showinfo") && l.contains(" pts_time:"))
        .filter_map(|l| {
            let seconds: f64 = l
                .split("pts_time:")
                .nth(1)?
                .split_whitespace()
                .next()?
                .parse()
                .ok()?;
            Some(numbers::truncate::<i64>((seconds * US_PER_S).round()))
        })
        .collect()
}

#[test]
#[ignore = "needs setup's FFmpeg (bun run setup)"]
fn every_frame_time_read_from_the_file_is_the_one_the_decoder_shows() {
    let mut failures = Vec::new();
    for (name, inputs, codec) in SAMPLES {
        let file = sample(name, inputs, codec);
        let read = video_frames(&file).unwrap().unwrap();
        let decoded = decoded_times(&file);
        assert_eq!((read.width, read.height), (320, 240), "{name}");
        if read.times_us.len() != decoded.len() {
            failures.push(format!(
                "{name}: {} frames read, {} decoded",
                read.times_us.len(),
                decoded.len()
            ));
            continue;
        }
        let worst = read
            .times_us
            .iter()
            .zip(&decoded)
            .map(|(a, b)| (a - b).abs())
            .max()
            .unwrap_or(0);
        if worst > 1 {
            failures.push(format!("{name}: a frame time differs by {worst} µs"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "needs setup's FFmpeg (bun run setup)"]
fn a_picture_that_isnt_a_video_has_no_frame_times() {
    let dir = dev_cache("probe-frames");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("still.png");
    let out = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .args(["color=red:size=16x16", "-frames:v", "1"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(video_frames(&file).unwrap(), None);
}
