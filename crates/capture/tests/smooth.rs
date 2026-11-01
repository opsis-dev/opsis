mod support;

use std::path::Path;
use std::sync::Arc;

use image::RgbImage;
use opsis_capture::encoder::{Encoded, FrameEncoder, FrameImage};
use opsis_capture::take::{Recording, record_take};
use opsis_compile::nodes::capture::{FramePicture, Record, Timing};
use opsis_types::actions::{ActionKind, ActionLog};
use opsis_types::numbers;
use opsis_types::values::{BrowserAction, SessionStep};
use serde_json::json;
use support::{FakeTools, NoSignIns, serve, session, step, take_env};

const FPS: u32 = 30;
const PX_PER_FRAME: i64 = 10;
const ROWS: [(&str, u32); 5] = [
    ("transition", 20),
    ("keyframes", 60),
    ("web", 100),
    ("frames", 140),
    ("timer", 180),
];
const PAGE: &str = r#"<!doctype html><html><body style="margin:0;background:#fff">
<style>
.box { position: absolute; left: 0; width: 20px; height: 20px; }
#transition { top: 10px; background: #f00; transition: transform 1s linear; }
#transition.go { transform: translateX(300px); }
#keyframes { top: 50px; background: #00f; animation: slide 1s linear infinite paused; }
#keyframes.go { animation-play-state: running; }
#web { top: 90px; background: #0a0; }
#frames { top: 130px; background: #f80; }
#timer { top: 170px; background: #000; display: none; }
#field { position: absolute; left: 0; top: 220px; width: 300px; font-size: 20px; }
@keyframes slide { from { transform: translateX(0) } to { transform: translateX(300px) } }
</style>
<div class="box" id="transition"></div><div class="box" id="keyframes"></div><div class="box" id="web"></div><div class="box" id="frames"></div><div class="box" id="timer"></div>
<input id="field" aria-label="Search">
<script>
let started = false;
document.getElementById("field").addEventListener("input", () => {
  if (started) return;
  started = true;
  document.getElementById("transition").classList.add("go");
  document.getElementById("keyframes").classList.add("go");
  document.getElementById("web").animate([{ transform: "translateX(0)" }, { transform: "translateX(300px)" }], { duration: 1000 });
  const frames = document.getElementById("frames");
  const from = performance.now();
  const move = () => {
    frames.style.transform = `translateX(${Math.min(300, (performance.now() - from) * 0.3)}px)`;
    requestAnimationFrame(move);
  };
  move();
  setTimeout(() => { document.getElementById("timer").style.display = "block"; }, 500);
});
</script></body></html>"#;

fn jpegs(stream: &[u8]) -> Vec<RgbImage> {
    let mut frames = Vec::new();
    let mut start = 0;
    for i in 1..stream.len() {
        if stream[i - 1] == 0xFF && stream[i] == 0xD9 {
            let decoded = image::load_from_memory(&stream[start..=i]).unwrap();
            frames.push(decoded.to_rgb8());
            start = i + 1;
        }
    }
    frames
}

fn left_edge(frame: &RgbImage, y: u32) -> Option<i64> {
    (0..frame.width())
        .find(|&x| {
            let [r, g, b] = frame.get_pixel(x, y).0;
            u32::from(r) + u32::from(g) + u32::from(b) < 600
        })
        .map(i64::from)
}

struct Smoothly {
    frames: Vec<RgbImage>,
    encoded: Encoded,
    log: ActionLog,
}

async fn record_smoothly(
    page: &'static str,
    steps: Vec<SessionStep>,
    pictures: FramePicture,
) -> Smoothly {
    let port = serve(page).await;
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), Arc::new(NoSignIns));
    let steps = [
        step(
            BrowserAction::Navigate,
            "go",
            json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
        ),
        step(BrowserAction::StartRecording, "roll", json!({})),
    ]
    .into_iter()
    .chain(steps)
    .collect();
    let session = session(
        json!({ "viewportWidth": 640, "viewportHeight": 360, "dpr": 2.0 }),
        steps,
    );
    let tools = FakeTools::new(1 << 20, None);
    let written = Arc::clone(&tools.written);
    let encoder =
        FrameEncoder::start(&tools, Path::new("take.mp4"), FPS, FrameImage::Jpeg).unwrap();
    let record = Record {
        fps: FPS,
        timing: Timing::Smooth,
        pictures,
        establish_ms: 0,
        end_tail_ms: 0,
        ..Record::default()
    };
    let played = record_take(&env, &session, Recording { encoder, record }).await;
    let video = played.encoded.expect("the take was encoded");
    let log = played.outcome.expect("the take played").log;
    let frames = jpegs(&written.lock().unwrap());
    let size = match pictures {
        FramePicture::FullDetail => (1280, 720),
        FramePicture::Standard | FramePicture::Lossless => (640, 360),
    };
    assert!(
        frames.iter().all(|frame| frame.dimensions() == size),
        "every frame is the window's size, not {:?}",
        frames.first().map(RgbImage::dimensions)
    );
    Smoothly {
        frames,
        encoded: video,
        log,
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn gives_every_frame_its_own_picture_of_the_page_one_frame_on() {
    let Smoothly {
        frames,
        encoded,
        log,
    } = record_smoothly(
        PAGE,
        vec![
            step(
                BrowserAction::Type,
                "type",
                json!({ "target": "Search", "text": "hello", "after": 0, "hoverMs": 0 }),
            ),
            step(
                BrowserAction::Wait,
                "hold",
                json!({ "mode": "ms", "ms": 1200 }),
            ),
        ],
        FramePicture::Standard,
    )
    .await;
    let frame_ms = 1000.0 / f64::from(FPS);

    assert_eq!(frames.len() as u64, encoded.frames);
    assert!((encoded.duration_ms - log.duration_ms).abs() < 1e-6);
    assert_eq!(
        encoded.duration_ms,
        numbers::float_wide(encoded.frames) * frame_ms
    );

    let typed = log
        .actions
        .iter()
        .find(|a| a.kind == ActionKind::Type)
        .expect("the typing is logged");
    let key_times = typed.key_times.clone().expect("each key has a time");
    assert_eq!(key_times.len(), 5);
    for t in key_times
        .iter()
        .chain([typed.t, typed.t_end.unwrap()].iter())
    {
        let frames_in = t / frame_ms;
        assert!(
            (frames_in - frames_in.round()).abs() < 1e-3,
            "{t} isn't on a frame"
        );
    }

    let first_key = numbers::truncate::<usize>((key_times[0] / frame_ms).round());
    let edges: Vec<Vec<Option<i64>>> = frames
        .iter()
        .map(|frame| ROWS.iter().map(|&(_, y)| left_edge(frame, y)).collect())
        .collect();
    let moving = (first_key + 2)..(first_key + 25);
    let moved = |row: usize, frame: usize| {
        let (Some(before), Some(after)) = (edges[frame - 1][row], edges[frame][row]) else {
            panic!("{} is missing at frame {frame}", ROWS[row].0);
        };
        after - before
    };
    for (row, (name, _)) in ROWS.iter().enumerate().take(3) {
        for frame in moving.clone() {
            let px = moved(row, frame);
            assert!(
                (px - PX_PER_FRAME).abs() <= 1,
                "{name} moved {px} px from frame {} to {frame}",
                frame - 1
            );
        }
    }
    let on_animation_frames: Vec<i64> = moving.map(|frame| moved(3, frame)).collect();
    assert!(
        on_animation_frames.iter().all(|&px| px > 0),
        "{on_animation_frames:?}"
    );
    let total: i64 = on_animation_frames.iter().sum();
    assert!(
        (total - PX_PER_FRAME * i64::try_from(on_animation_frames.len()).unwrap()).abs()
            <= 2 * PX_PER_FRAME,
        "{on_animation_frames:?}"
    );
    let shown = edges
        .iter()
        .position(|row| row[4].is_some())
        .expect("the timer fired");
    assert!(
        (first_key + 15..=first_key + 16).contains(&shown),
        "the timer showed at frame {shown}, {} frames after the first key",
        shown - first_key
    );
}

const LINED_PAGE: &str = r#"<!doctype html><html><body style="margin:0;height:3000px;background:repeating-linear-gradient(#fff 0 196px, #000 196px 200px)">
<div id="target" style="position:absolute;left:0;top:1400px;width:200px;height:40px;background:#888">Target</div>
</body></html>"#;
const LINE_GAP: i64 = 200;
const SCROLL_TO: i64 = 1400 + 20 - 180;

fn first_line(frame: &RgbImage) -> i64 {
    (0..frame.height())
        .find(|&y| {
            let [r, g, b] = frame.get_pixel(frame.width() - 10, y).0;
            u32::from(r) + u32::from(g) + u32::from(b) < 200
        })
        .map(i64::from)
        .expect("a line is on screen")
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn scrolls_to_an_element_a_frame_at_a_time() {
    let Smoothly { frames, .. } = record_smoothly(
        LINED_PAGE,
        vec![
            step(
                BrowserAction::Scroll,
                "scroll",
                json!({ "mode": "element", "target": "css=#target", "smooth": true, "durationMs": 3000, "after": 0 }),
            ),
            step(BrowserAction::Wait, "hold", json!({ "mode": "ms", "ms": 300 })),
        ],
        FramePicture::Standard,
    )
    .await;
    let lines: Vec<i64> = frames.iter().map(first_line).collect();
    let moves: Vec<i64> = lines
        .windows(2)
        .map(|pair| (pair[0] - pair[1]).rem_euclid(LINE_GAP))
        .collect();
    let scrolled: i64 = moves.iter().sum();
    assert_eq!(scrolled, SCROLL_TO, "{moves:?}");
    let moving = moves.iter().filter(|&&px| px > 0).count();
    assert!(
        (85..=91).contains(&moving),
        "{moving} frames moved: {moves:?}"
    );
    assert!(moves.iter().all(|&px| px < LINE_GAP / 2), "{moves:?}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn takes_full_detail_frames_at_the_pages_pixel_ratio() {
    let Smoothly { frames, .. } = record_smoothly(
        LINED_PAGE,
        vec![step(
            BrowserAction::Wait,
            "hold",
            json!({ "mode": "ms", "ms": 200 }),
        )],
        FramePicture::FullDetail,
    )
    .await;
    assert!(frames.len() >= 6);
}
