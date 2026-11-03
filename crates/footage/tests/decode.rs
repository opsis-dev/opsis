use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use image::RgbImage;
use opsis_compile::asset_ref::FOOTAGE_FOLDER;
use opsis_compile::nodes::effects::Area;
use opsis_compile::nodes::image::{RedactImage, RedactStyle};
use opsis_footage::chain::{FootageSource, TimedRedaction};
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::serve::footage_routes;
use opsis_golden::{ffmpeg, home};
use opsis_image::ops::redact;
use opsis_media::grade::GradeChain;
use tower::ServiceExt;

const RATE: i64 = 30;
const SECONDS: i64 = 4;
const EVEN: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

fn samples() -> PathBuf {
    let dir = home().join(".cache/opsis-footage-samples");
    std::fs::create_dir_all(dir.join("tmp")).unwrap();
    dir
}

fn run(args: &[&str]) -> Vec<u8> {
    let out = Command::new(ffmpeg())
        .args(["-hide_banner", "-loglevel", "error", "-nostdin"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

static MAKING: Mutex<()> = Mutex::new(());

fn sample(name: &str, size: &str) -> PathBuf {
    let _making = MAKING.lock().unwrap();
    let path = samples().join(name);
    if !path.exists() {
        let partial = samples().join(format!("partial-{name}"));
        let source = format!("testsrc2=size={size}:rate={RATE}");
        let length = SECONDS.to_string();
        run(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            &source,
            "-t",
            &length,
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-g",
            "30",
            &partial.to_string_lossy(),
        ]);
        std::fs::rename(&partial, &path).unwrap();
    }
    path
}

fn tools() -> FootageTools {
    FootageTools {
        ffmpeg: ffmpeg(),
        temp: samples().join("tmp"),
    }
}

fn plain(file: &Path) -> FootageSource {
    FootageSource {
        file: file.to_owned(),
        grade: GradeChain {
            chain: format!("[0:v]{EVEN}[out]"),
            extra_inputs: Vec::new(),
        },
        redactions: Vec::new(),
        trim_us: 0,
        loop_us: None,
    }
}

fn footage(sources: Vec<(&str, FootageSource)>) -> Footage {
    Footage::new(
        tools(),
        sources
            .into_iter()
            .map(|(id, source)| (id.to_owned(), source)),
    )
}

const fn at(frame: i64) -> i64 {
    (frame * 2_000_000 + RATE) / (2 * RATE)
}

fn reference(file: &Path, frame: i64, filters: &str) -> RgbImage {
    let graph = format!("select=eq(n\\,{frame}),{filters}format=rgb24");
    let png = run(&[
        "-i",
        &file.to_string_lossy(),
        "-vf",
        &graph,
        "-frames:v",
        "1",
        "-f",
        "image2pipe",
        "-c:v",
        "png",
        "pipe:1",
    ]);
    image::load_from_memory(&png).unwrap().into_rgb8()
}

fn difference(a: &RgbImage, b: &RgbImage) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());
    let total: u64 = a
        .as_raw()
        .iter()
        .zip(b.as_raw())
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    let count = u32::try_from(a.as_raw().len()).unwrap();
    opsis_types::numbers::float_wide(total) / f64::from(count)
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn shows_the_frame_ffmpeg_decodes_at_each_time() {
    let video = sample("count.mp4", "320x240");
    let film = footage(vec![("take", plain(&video))]);
    for frame in [0, 1, 29, 30, 31, 64, 7, 100, 119, 45] {
        let shown = film.shown("take", at(frame)).unwrap();
        assert_eq!(shown.pts_us, at(frame), "frame {frame}");
        let own = difference(&shown.image, &reference(&video, frame, ""));
        let next = difference(&shown.image, &reference(&video, (frame + 1).min(119), ""));
        assert!(own < 0.5, "frame {frame} differs by {own}");
        assert!(
            frame == 119 || next > own,
            "frame {frame} looks like the next one"
        );
    }
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn shows_the_last_frame_at_or_before_the_time() {
    let video = sample("count.mp4", "320x240");
    let film = footage(vec![("take", plain(&video))]);
    for frame in [0, 10, 50, 118] {
        let between = at(frame) + 1_000_000 / RATE / 2;
        assert_eq!(film.shown("take", between).unwrap().pts_us, at(frame));
        assert_eq!(
            film.shown("take", at(frame + 1) - 600).unwrap().pts_us,
            at(frame)
        );
    }
    assert_eq!(film.shown("take", -5_000).unwrap().pts_us, 0);
    assert_eq!(film.shown("take", at(500)).unwrap().pts_us, at(119));
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn frames_asked_in_order_start_the_decoder_once() {
    let video = sample("count.mp4", "320x240");
    let film = footage(vec![("take", plain(&video))]);
    for frame in 0..RATE * SECONDS {
        assert_eq!(film.shown("take", at(frame)).unwrap().pts_us, at(frame));
        assert!(!film.picture("take", at(frame)).unwrap().is_empty());
    }
    assert_eq!(film.decoder_starts(), 1);
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn a_step_back_past_the_kept_frames_or_a_long_jump_starts_the_decoder_again() {
    let video = sample("count.mp4", "320x240");
    let film = footage(vec![("take", plain(&video))]);
    film.shown("take", at(60)).unwrap();
    assert_eq!(film.decoder_starts(), 1);
    film.shown("take", at(61)).unwrap();
    film.shown("take", at(61)).unwrap();
    assert_eq!(film.decoder_starts(), 1);
    for frame in [61, 60, 59, 58, 61, 62] {
        assert_eq!(film.shown("take", at(frame)).unwrap().pts_us, at(frame));
    }
    assert_eq!(film.decoder_starts(), 1);
    film.shown("take", at(15)).unwrap();
    assert_eq!(film.decoder_starts(), 2);
    assert_eq!(film.shown("take", at(115)).unwrap().pts_us, at(115));
    assert_eq!(film.decoder_starts(), 3);
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn the_grade_chain_and_its_extra_inputs_run_on_each_frame() {
    let video = sample("count.mp4", "320x240");
    let identity = samples().join("identity-look.png");
    let making = MAKING.lock().unwrap();
    if !identity.exists() {
        run(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "haldclutsrc=8",
            "-frames:v",
            "1",
            &identity.to_string_lossy(),
        ]);
    }
    drop(making);
    let negated = FootageSource {
        grade: GradeChain {
            chain: format!("[0:v]negate,{EVEN}[out]"),
            extra_inputs: Vec::new(),
        },
        ..plain(&video)
    };
    let looked = FootageSource {
        grade: GradeChain {
            chain: format!("[0:v][1:v]haldclut,{EVEN}[out]"),
            extra_inputs: vec![identity.to_string_lossy().into_owned()],
        },
        ..plain(&video)
    };
    let film = footage(vec![("negated", negated), ("looked", looked)]);
    let shown = film.shown("negated", at(40)).unwrap();
    assert!(difference(&shown.image, &reference(&video, 40, "negate,")) < 0.5);
    let looked = film.shown("looked", at(40)).unwrap();
    assert!(difference(&looked.image, &reference(&video, 40, "")) < 2.0);
}

fn hidden(style: RedactStyle) -> RedactImage {
    RedactImage {
        areas: vec![Area {
            x: 0.25,
            y: 0.25,
            width: 0.5,
            height: 0.4,
        }],
        style,
        ..RedactImage::default()
    }
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn redaction_is_the_image_redact_on_the_frames_in_its_time_range() {
    let video = sample("count.mp4", "320x240");
    let mut redacted = plain(&video);
    redacted.redactions = vec![
        TimedRedaction {
            from_us: at(30),
            until_us: Some(at(60)),
            redact: hidden(RedactStyle::Pixelate),
        },
        TimedRedaction {
            from_us: at(50),
            until_us: None,
            redact: hidden(RedactStyle::Solid),
        },
    ];
    let film = footage(vec![
        ("plain", plain(&video)),
        ("redacted", redacted.clone()),
    ]);
    for frame in [10, 30, 45, 55, 59, 60, 90] {
        let plain_frame = film.shown("plain", at(frame)).unwrap().image;
        let expected = redacted
            .redactions
            .iter()
            .filter(|r| r.covers(at(frame)))
            .fold(image::DynamicImage::ImageRgb8(plain_frame), |p, r| {
                redact(&p, &r.redact)
            })
            .into_rgb8();
        assert_eq!(
            film.shown("redacted", at(frame)).unwrap().image,
            expected,
            "frame {frame}"
        );
    }
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn a_trimmed_looping_source_plays_its_part_of_the_file_again_and_again() {
    let video = sample("count.mp4", "320x240");
    let looped = FootageSource {
        trim_us: at(30),
        loop_us: Some(at(60)),
        ..plain(&video)
    };
    let film = footage(vec![("looped", looped)]);
    assert_eq!(film.shown("looped", 0).unwrap().pts_us, at(30));
    assert_eq!(film.shown("looped", at(59)).unwrap().pts_us, at(89));
    assert_eq!(film.shown("looped", at(60)).unwrap().pts_us, at(30));
    assert_eq!(film.shown("looped", at(70)).unwrap().pts_us, at(40));
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn odd_sized_sources_come_out_even() {
    let video = sample("odd.mp4", "321x241");
    let film = footage(vec![("odd", plain(&video))]);
    assert_eq!(
        film.shown("odd", at(3)).unwrap().image.dimensions(),
        (320, 240)
    );
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn idle_decoders_end_and_start_again_when_asked() {
    let video = sample("count.mp4", "320x240");
    let film = footage(vec![("take", plain(&video))]);
    film.shown("take", at(5)).unwrap();
    assert_eq!(film.running_decoders(), 1);
    film.end_idle(Duration::ZERO);
    assert_eq!(film.running_decoders(), 0);
    assert_eq!(film.shown("take", at(6)).unwrap().pts_us, at(6));
    assert_eq!(film.decoder_starts(), 2);
}

#[test]
#[ignore = "needs the staged FFmpeg"]
fn a_file_that_isnt_a_video_fails_instead_of_showing_nothing() {
    let video = samples().join("not-a-video.mp4");
    std::fs::write(&video, b"just text").unwrap();
    let film = footage(vec![("broken", plain(&video))]);
    assert!(film.shown("broken", 0).is_err());
    assert!(film.shown("missing", 0).is_err());
}

#[tokio::test]
#[ignore = "needs the staged FFmpeg"]
async fn serves_each_frame_as_a_jpeg_of_its_size() {
    let video = sample("count.mp4", "320x240");
    let routes = footage_routes(Arc::new(footage(vec![("take", plain(&video))])));
    let get = |path: String| {
        routes
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
    };
    let response = get(format!("/{FOOTAGE_FOLDER}/take/{}", at(12)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/jpeg");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let picture = image::load_from_memory(&bytes).unwrap().into_rgb8();
    assert_eq!(picture.dimensions(), (320, 240));
    let missing = get(format!("/{FOOTAGE_FOLDER}/nothing/0")).await.unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let bad = get(format!("/{FOOTAGE_FOLDER}/take/soon")).await.unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn grades_a_terminal_swatch_through_one_filter_graph() {
    let grade = GradeChain {
        chain: "[0:v]hue=s=0[out]".to_owned(),
        extra_inputs: vec!["look.png".to_owned()],
    };
    let swatch = opsis_terminal::colours::swatch(&[0xff_0000, 0x00_ff00, 0x00_00ff]);
    assert_eq!(
        opsis_footage::terminal::swatch_args(&grade, &swatch).join(" "),
        "-nostdin -hide_banner -nostats -loglevel error -f rawvideo -pix_fmt rgb24 -s 3x1 -i pipe:0 -i look.png -filter_complex [0:v]hue=s=0[out];[out]format=rgb24[graded] -map [graded] -frames:v 1 -f rawvideo -pix_fmt rgb24 pipe:1"
    );
}

#[test]
#[ignore = "needs FFmpeg"]
fn a_black_and_white_grade_turns_every_terminal_colour_grey() {
    let dir = samples();
    let cast = dir.join("grade.cast");
    std::fs::write(
        &cast,
        "{\"version\":3,\"term\":{\"cols\":20,\"rows\":3}}\n[0.1,\"o\",\"\\u001b[31mred \\u001b[44mblue\\u001b[0m\"]\n[0.5,\"o\",\"!\"]\n",
    )
    .unwrap();
    let footage = Footage::new(
        FootageTools {
            ffmpeg: ffmpeg(),
            temp: dir.join("tmp"),
        },
        Vec::new(),
    )
    .with_terminals([(
        "t".to_owned(),
        opsis_footage::terminal::TerminalFootage {
            cast,
            theme: opsis_compile::terminal::default_theme(),
            masks: Vec::new(),
            fps: 10,
            grade: Some(GradeChain {
                chain: "[0:v]hue=s=0[out]".to_owned(),
                extra_inputs: Vec::new(),
            }),
        },
    )]);
    let bytes = footage.terminal_frames("t").unwrap();
    let frames = opsis_terminal::frames::read(&bytes).unwrap();
    let colours = frames.colours();
    assert!(colours.len() > 2);
    for colour in colours {
        let [_, r, g, b] = colour.to_be_bytes();
        assert!(r.abs_diff(g) <= 1 && g.abs_diff(b) <= 1, "{colour:06x}");
    }
}
