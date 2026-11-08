use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use opsis_probe::{AudioStream, Probe, VideoStream, probe};
use opsis_types::numbers;
use serde_json::Value;

const VIDEO: &str = "testsrc2=size=320x240:rate=30:duration=2";
const VIDEO_25: &str = "testsrc2=size=320x240:rate=25:duration=2";
const SMALL: &str = "testsrc2=size=160x120:rate=10:duration=1";
const SOUND: &str = "sine=frequency=440:sample_rate=48000:duration=2";
const SOUND_44: &str = "sine=frequency=440:sample_rate=44100:duration=2";

const SAMPLES: &[(&str, &[&str], &[&str])] = &[
    (
        "h264.mp4",
        &[VIDEO, SOUND],
        &[
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-ac", "2",
        ],
    ),
    (
        "h264-444.mp4",
        &[VIDEO],
        &["-c:v", "libx264", "-pix_fmt", "yuv444p", "-crf", "30"],
    ),
    (
        "h264-full.mp4",
        &[VIDEO],
        &["-c:v", "libx264", "-pix_fmt", "yuvj420p"],
    ),
    (
        "h264-10.mp4",
        &[VIDEO],
        &["-c:v", "libx264", "-pix_fmt", "yuv420p10le"],
    ),
    (
        "hevc10.mp4",
        &[VIDEO],
        &[
            "-c:v",
            "libx265",
            "-x265-params",
            "log-level=error",
            "-pix_fmt",
            "yuv420p10le",
            "-tag:v",
            "hvc1",
        ],
    ),
    (
        "hevc-full.mp4",
        &[VIDEO],
        &[
            "-c:v",
            "libx265",
            "-x265-params",
            "log-level=error",
            "-pix_fmt",
            "yuvj420p",
        ],
    ),
    (
        "vp9.webm",
        &[VIDEO_25, SOUND],
        &["-c:v", "libvpx-vp9", "-b:v", "200k", "-c:a", "libopus"],
    ),
    (
        "vp9-444.webm",
        &[VIDEO],
        &["-c:v", "libvpx-vp9", "-b:v", "200k", "-pix_fmt", "yuv444p"],
    ),
    ("vp9.mp4", &[VIDEO], &["-c:v", "libvpx-vp9", "-b:v", "200k"]),
    (
        "vp8.webm",
        &[VIDEO, SOUND],
        &["-c:v", "libvpx", "-b:v", "200k", "-c:a", "libvorbis"],
    ),
    (
        "av1.mp4",
        &[VIDEO],
        &[
            "-c:v",
            "libaom-av1",
            "-cpu-used",
            "8",
            "-crf",
            "50",
            "-t",
            "1",
        ],
    ),
    (
        "prores-hq.mov",
        &[VIDEO],
        &["-c:v", "prores_ks", "-profile:v", "3"],
    ),
    (
        "prores-4444.mov",
        &[VIDEO],
        &[
            "-c:v",
            "prores_ks",
            "-profile:v",
            "4",
            "-pix_fmt",
            "yuv444p10le",
        ],
    ),
    (
        "prores-4444a.mov",
        &["testsrc2=size=320x240:rate=30:duration=2,format=rgba"],
        &[
            "-c:v",
            "prores_ks",
            "-profile:v",
            "4",
            "-pix_fmt",
            "yuva444p10le",
        ],
    ),
    ("mpeg4.mp4", &[VIDEO], &["-c:v", "mpeg4"]),
    (
        "h264-mp3.mkv",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-c:a", "libmp3lame"],
    ),
    (
        "h264-aac.mkv",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-c:a", "aac", "-ac", "2"],
    ),
    ("mjpeg.avi", &[VIDEO], &["-c:v", "mjpeg"]),
    (
        "mpeg4-mp3.avi",
        &[VIDEO, SOUND],
        &["-c:v", "mpeg4", "-c:a", "libmp3lame"],
    ),
    (
        "frag.mp4",
        &[VIDEO, SOUND],
        &[
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-movflags",
            "frag_keyframe+empty_moov",
        ],
    ),
    (
        "faststart.mp4",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-c:a", "aac", "-movflags", "+faststart"],
    ),
    (
        "live.webm",
        &[VIDEO],
        &["-c:v", "libvpx-vp9", "-b:v", "200k", "-live", "1"],
    ),
    (
        "h264.mov",
        &[VIDEO, SOUND],
        &["-c:v", "libx264", "-c:a", "pcm_s16le"],
    ),
    ("anim.gif", &[SMALL], &[]),
    ("still.png", &[VIDEO], &["-frames:v", "1"]),
    (
        "alpha.png",
        &["testsrc2=size=320x240:rate=30:duration=2,format=rgba"],
        &["-frames:v", "1"],
    ),
    (
        "gray.png",
        &[VIDEO],
        &["-frames:v", "1", "-pix_fmt", "gray"],
    ),
    (
        "deep.png",
        &[VIDEO],
        &["-frames:v", "1", "-pix_fmt", "rgb48be"],
    ),
    ("photo.jpg", &[VIDEO], &["-frames:v", "1"]),
    (
        "photo444.jpg",
        &[VIDEO],
        &["-frames:v", "1", "-pix_fmt", "yuvj444p"],
    ),
    (
        "photo422.jpg",
        &[VIDEO],
        &["-frames:v", "1", "-pix_fmt", "yuvj422p"],
    ),
    (
        "lossy.webp",
        &[VIDEO],
        &["-frames:v", "1", "-c:v", "libwebp"],
    ),
    (
        "lossless.webp",
        &["testsrc2=size=320x240:rate=30:duration=2,format=rgba"],
        &["-frames:v", "1", "-c:v", "libwebp", "-lossless", "1"],
    ),
    (
        "alpha.webp",
        &["testsrc2=size=320x240:rate=30:duration=2,format=rgba"],
        &["-frames:v", "1", "-c:v", "libwebp"],
    ),
    ("s16.wav", &[SOUND], &["-c:a", "pcm_s16le"]),
    ("f32.wav", &[SOUND], &["-c:a", "pcm_f32le"]),
    ("s24.wav", &[SOUND], &["-c:a", "pcm_s24le", "-ac", "2"]),
    ("u8.wav", &[SOUND], &["-c:a", "pcm_u8"]),
    ("aiff.aif", &[SOUND], &[]),
    ("cbr.mp3", &[SOUND], &["-c:a", "libmp3lame", "-b:a", "128k"]),
    ("vbr.mp3", &[SOUND], &["-c:a", "libmp3lame", "-q:a", "4"]),
    (
        "tagged.mp3",
        &[SOUND_44],
        &[
            "-c:a",
            "libmp3lame",
            "-b:a",
            "96k",
            "-metadata",
            "title=Probe",
            "-id3v2_version",
            "3",
        ],
    ),
    ("adts.aac", &[SOUND], &["-c:a", "aac"]),
    ("m4a.m4a", &[SOUND], &["-c:a", "aac"]),
    ("flac.flac", &[SOUND], &[]),
    ("vorbis.ogg", &[SOUND], &["-c:a", "libvorbis"]),
    ("opus.opus", &[SOUND], &["-c:a", "libopus"]),
    ("flac.oga", &[SOUND], &["-c:a", "flac"]),
    ("opus.mka", &[SOUND], &["-c:a", "libopus"]),
    ("caf.caf", &[SOUND_44], &["-c:a", "pcm_s16be"]),
    ("alac.caf", &[SOUND_44], &["-c:a", "alac"]),
    ("wma.wma", &[SOUND], &["-c:a", "wmav2"]),
    ("alac.m4a", &[SOUND_44], &["-c:a", "alac"]),
];

const DURATION_SLACK_MS: f64 = 25.0;

fn ffmpeg() -> PathBuf {
    PathBuf::from(std::env::var_os("OPSIS_FFMPEG").expect(
        "OPSIS_FFMPEG names the staged FFmpeg: run cargo through bun run test:rust, or set it to what bun run ffmpeg:stage prints",
    ))
}

fn ffprobe_program() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").expect("a home folder"))
        .join(".cache/opsis-dev/ffmpeg")
        .join(format!("ffprobe{}", std::env::consts::EXE_SUFFIX))
}

static MADE: OnceLock<PathBuf> = OnceLock::new();

fn samples() -> PathBuf {
    MADE.get_or_init(make_samples).clone()
}

fn make_samples() -> PathBuf {
    let dir = PathBuf::from(std::env::var_os("HOME").expect("a home folder"))
        .join(".cache/opsis-dev/probe-samples");
    std::fs::create_dir_all(&dir).expect("the sample folder");
    for (name, sources, args) in SAMPLES {
        let file = dir.join(name);
        if file.exists() {
            continue;
        }
        let mut command = Command::new(ffmpeg());
        command.args(["-y", "-loglevel", "error"]);
        for source in *sources {
            command.args(["-f", "lavfi", "-i", source]);
        }
        let status = command
            .args(*args)
            .arg(&file)
            .status()
            .expect("ffmpeg runs");
        assert!(status.success(), "ffmpeg couldn't make {name}");
    }
    dir
}

fn rate(text: Option<&Value>) -> Option<f64> {
    let (count, per) = text?.as_str()?.split_once('/')?;
    let fps = count.parse::<f64>().ok()? / per.parse::<f64>().ok()?;
    (fps.is_finite() && fps > 0.0).then_some(fps)
}

fn ffprobe(file: &Path) -> Option<Probe> {
    let out = Command::new(ffprobe_program())
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(file)
        .output()
        .expect("ffprobe runs");
    let json: Value = serde_json::from_slice(&out.stdout).ok()?;
    let streams = json["streams"].as_array().cloned().unwrap_or_default();
    let video = streams.iter().find(|s| {
        s["codec_type"] == "video" && s["disposition"]["attached_pic"].as_u64() != Some(1)
    });
    let audio = streams.iter().find(|s| s["codec_type"] == "audio");
    let duration = [Some(&json["format"]), video, audio]
        .into_iter()
        .flatten()
        .find_map(|s| s["duration"].as_str()?.parse::<f64>().ok());
    let text = |v: &Value| v.as_str().unwrap_or_default().to_owned();
    let number = |v: &Value| v.as_u64().and_then(|n| u32::try_from(n).ok());
    Some(Probe {
        duration_ms: duration.map(|s| numbers::whole(s * 1000.0)),
        format: text(&json["format"]["format_name"]),
        video: video.map(|v| VideoStream {
            codec: text(&v["codec_name"]),
            width: number(&v["width"]).unwrap_or(0),
            height: number(&v["height"]).unwrap_or(0),
            fps: rate(v.get("avg_frame_rate")).or_else(|| rate(v.get("r_frame_rate"))),
            pix_fmt: text(&v["pix_fmt"]),
        }),
        audio: audio.map(|a| AudioStream {
            codec: text(&a["codec_name"]),
            sample_rate: a["sample_rate"].as_str().and_then(|r| r.parse().ok()),
            channels: number(&a["channels"]),
        }),
    })
}

fn differences(name: &str, ours: &Probe, theirs: &Probe) -> Vec<String> {
    let mut out = Vec::new();
    let mut differ = |what: &str, a: String, b: String| {
        if a != b {
            out.push(format!("{name} {what}: ours {a}, ffprobe {b}"));
        }
    };
    differ("format", ours.format.clone(), theirs.format.clone());
    match (ours.duration_ms, theirs.duration_ms) {
        (Some(a), Some(b)) => {
            let slack = DURATION_SLACK_MS.max(f64::from(b) * 0.01);
            if (f64::from(a) - f64::from(b)).abs() > slack {
                differ("duration", a.to_string(), b.to_string());
            }
        }
        (_, None) => {}
        (a, b) => differ("duration", format!("{a:?}"), format!("{b:?}")),
    }
    match (&ours.video, &theirs.video) {
        (Some(a), Some(b)) => {
            differ("video codec", a.codec.clone(), b.codec.clone());
            differ(
                "size",
                format!("{}x{}", a.width, a.height),
                format!("{}x{}", b.width, b.height),
            );
            differ("pixel format", a.pix_fmt.clone(), b.pix_fmt.clone());
            let close = match (a.fps, b.fps) {
                (Some(x), Some(y)) => (x - y).abs() < 0.01,
                (x, y) => x == y,
            };
            if !close {
                differ("frame rate", format!("{:?}", a.fps), format!("{:?}", b.fps));
            }
        }
        (a, b) => differ(
            "video",
            format!("{:?}", a.is_some()),
            format!("{:?}", b.is_some()),
        ),
    }
    match (&ours.audio, &theirs.audio) {
        (Some(a), Some(b)) => {
            differ("audio codec", a.codec.clone(), b.codec.clone());
            differ(
                "sample rate",
                format!("{:?}", a.sample_rate),
                format!("{:?}", b.sample_rate),
            );
            differ(
                "channels",
                format!("{:?}", a.channels),
                format!("{:?}", b.channels),
            );
        }
        (a, b) => differ(
            "audio",
            format!("{:?}", a.is_some()),
            format!("{:?}", b.is_some()),
        ),
    }
    out
}

#[test]
#[ignore = "needs the staged FFmpeg and ffprobe (bun run ffprobe:stage)"]
fn reads_every_file_as_ffprobe_does() {
    let dir = samples();
    let mut found = Vec::new();
    for (name, _, _) in SAMPLES {
        let file = dir.join(name);
        let theirs = ffprobe(&file).unwrap_or_else(|| panic!("ffprobe couldn't read {name}"));
        match probe(&file) {
            Ok(ours) => found.extend(differences(name, &ours, &theirs)),
            Err(e) => found.push(format!("{name}: {e}")),
        }
    }
    assert!(found.is_empty(), "\n{}", found.join("\n"));
}

#[test]
#[ignore = "needs the staged FFmpeg and ffprobe (bun run ffprobe:stage)"]
fn knows_the_length_of_a_stream_written_without_one() {
    let dir = samples();
    let live = probe(&dir.join("live.webm")).expect("a probe");
    let length = live
        .duration_ms
        .expect("a length worked out from the frames");
    assert!(length.abs_diff(2000) <= 40, "{length} ms");
}

#[test]
#[ignore = "needs the staged FFmpeg and ffprobe (bun run ffprobe:stage)"]
fn answers_files_cut_short_without_panicking() {
    let dir = samples();
    let cut = tempfile::NamedTempFile::new().expect("a temp file");
    for (name, _, _) in SAMPLES {
        let bytes = std::fs::read(dir.join(name)).expect("a sample");
        for tenth in 1..10 {
            std::fs::write(cut.path(), &bytes[..bytes.len() * tenth / 10]).expect("written");
            let _ = probe(cut.path());
        }
    }
}
