mod support;

use std::path::Path;
use std::sync::Arc;

use image::RgbImage;
use opsis_capture::encoder::{FrameEncoder, FrameImage};
use opsis_capture::take::{Recording, play_take};
use opsis_compile::nodes::capture::{FramePicture, Record, Timing};
use opsis_types::actions::{ActionKind, ActionLog};
use opsis_types::numbers;
use opsis_types::values::{BrowserAction, BrowserSessionValue, SessionStep};
use serde_json::{Value, json};
use support::{FakeTools, NoSignIns, serve, session, step, take_env};

const FPS: u32 = 30;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

const STATE_PAGE: &str = r#"<!doctype html><body style="margin:0;height:3000px;font:16px sans-serif">
<div id="out" style="position:fixed;top:0;left:0">waiting</div>
<button id="near" style="position:absolute;top:200px;left:20px">Near button</button>
<button id="far" style="position:absolute;top:2400px;left:20px">Far button</button>
<button id="load" style="position:absolute;top:100px;left:20px">Load data</button>
<script>
const out = document.getElementById("out");
const seen = new Set();
addEventListener("scroll", () => seen.add(Math.round(scrollY)));
const report = (name) => {
  out.textContent = `${name} after ${seen.size === 0 ? "no scroll" : seen.size > 3 ? "a smooth scroll" : "a jump"}`;
  seen.clear();
};
document.getElementById("near").addEventListener("click", () => report("near"));
document.getElementById("far").addEventListener("click", () => report("far"));
document.getElementById("load").addEventListener("click", async () => {
  const from = performance.now();
  const reply = await (await fetch("/api/data")).json();
  const waited = performance.now() - from;
  out.textContent = `ok ${reply.ok} ${waited >= 900 ? "after a second" : "at once"}`;
});
out.textContent = `motion ${matchMedia("(prefers-reduced-motion: reduce)").matches ? "reduced" : "full"} · ${innerWidth > document.documentElement.clientWidth ? "scrollbar" : "no scrollbar"}`;
</script></body>"#;

const STRIPED_PAGE: &str = r#"<!doctype html><body style="margin:0;height:3000px;background:repeating-linear-gradient(90deg,#000 0 2px,#fff 2px 4px)"></body>"#;

const SHADED_PAGE: &str = r#"<!doctype html><html><body style="margin:0">
<div style="position:absolute;left:0;top:0;width:100%;height:3000px;background:linear-gradient(#000,#fff)"></div>
<button style="position:absolute;left:300px;top:1400px;width:200px;height:40px">Far target</button>
</body></html>"#;

fn shows(text: &str) -> SessionStep {
    step(
        BrowserAction::Assert,
        "check",
        json!({ "text": text, "exact": true, "timeoutMs": 3000 }),
    )
}

fn opened(port: u16, browser: Value, steps: Vec<SessionStep>) -> BrowserSessionValue {
    let mut params = json!({ "viewportWidth": WIDTH, "viewportHeight": HEIGHT, "dpr": 1.0 });
    params
        .as_object_mut()
        .unwrap()
        .extend(browser.as_object().cloned().unwrap_or_default());
    let navigate = step(
        BrowserAction::Navigate,
        "go",
        json!({ "url": format!("http://127.0.0.1:{port}/"), "after": 0 }),
    );
    session(params, [navigate].into_iter().chain(steps).collect())
}

async fn play(page: &'static str, browser: Value, steps: Vec<SessionStep>) -> ActionLog {
    let port = serve(page).await;
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), Arc::new(NoSignIns));
    let played = play_take(&env, &opened(port, browser, steps), None, |_| {
        Box::pin(async { Ok(()) })
    })
    .await;
    match played.outcome {
        Ok(took) => took.log,
        Err(error) => panic!("the take failed: {error}"),
    }
}

fn jpegs(stream: &[u8]) -> Vec<RgbImage> {
    let mut frames = Vec::new();
    let mut start = 0;
    for i in 1..stream.len() {
        if stream[i - 1] == 0xFF && stream[i] == 0xD9 {
            frames.push(
                image::load_from_memory(&stream[start..=i])
                    .unwrap()
                    .to_rgb8(),
            );
            start = i + 1;
        }
    }
    frames
}

async fn record(
    page: &'static str,
    browser: Value,
    steps: Vec<SessionStep>,
) -> (Vec<RgbImage>, ActionLog) {
    let port = serve(page).await;
    let temp = tempfile::tempdir().unwrap();
    let env = take_env(temp.path(), Arc::new(NoSignIns));
    let tools = FakeTools::new(1 << 20, None);
    let written = Arc::clone(&tools.written);
    let encoder =
        FrameEncoder::start(&tools, Path::new("take.mp4"), FPS, FrameImage::Jpeg).unwrap();
    let record = Record {
        fps: FPS,
        timing: Timing::Smooth,
        pictures: FramePicture::Standard,
        establish_ms: 0,
        end_tail_ms: 0,
        ..Record::default()
    };
    let steps = [step(BrowserAction::StartRecording, "roll", json!({}))]
        .into_iter()
        .chain(steps)
        .collect();
    let played = play_take(
        &env,
        &opened(port, browser, steps),
        Some(Recording { encoder, record }),
        |_| Box::pin(async { Ok(()) }),
    )
    .await;
    let log = match played.outcome {
        Ok(took) => took.log,
        Err(error) => panic!("the take failed: {error}"),
    };
    let frames = jpegs(&written.lock().unwrap());
    assert!(
        frames
            .iter()
            .all(|frame| frame.dimensions() == (WIDTH, HEIGHT))
    );
    (frames, log)
}

fn frame_at(t: f64) -> usize {
    numbers::truncate::<usize>((t * f64::from(FPS) / 1000.0).round())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn tells_the_page_about_motion_and_scrollbars_as_set() {
    play(
        STATE_PAGE,
        json!({}),
        vec![shows("motion full · no scrollbar")],
    )
    .await;
    play(
        STATE_PAGE,
        json!({ "reduceMotion": true, "showScrollbars": true }),
        vec![shows("motion reduced · scrollbar")],
    )
    .await;
}

fn scrollbar_columns(frame: &RgbImage) -> usize {
    (WIDTH - 20..WIDTH)
        .filter(|&x| {
            (0..HEIGHT).any(|y| {
                let [r, g, b] = frame.get_pixel(x, y).0;
                u32::from(r) + u32::from(g) + u32::from(b) < 450
            })
        })
        .count()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn draws_scrollbars_only_when_asked() {
    let wait = || {
        step(
            BrowserAction::Wait,
            "hold",
            json!({ "mode": "ms", "ms": 200 }),
        )
    };
    let (hidden, _) = record(STATE_PAGE, json!({}), vec![wait()]).await;
    let (shown, _) = record(STATE_PAGE, json!({ "showScrollbars": true }), vec![wait()]).await;
    let hidden = scrollbar_columns(hidden.last().unwrap());
    let shown = scrollbar_columns(shown.last().unwrap());
    assert_eq!(hidden, 0);
    assert!(shown >= 4, "{shown} columns at the right edge");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn scrolls_smoothly_to_a_click_only_when_the_element_is_off_screen() {
    let click = |id: &str, target: &str, smoothly: bool| {
        step(
            BrowserAction::Click,
            id,
            json!({ "target": target, "after": 0, "hoverMs": 0, "scrollSmoothly": smoothly }),
        )
    };
    play(
        STATE_PAGE,
        json!({}),
        vec![
            click("near", "Near button", true),
            shows("near after no scroll"),
            click("far", "Far button", true),
            shows("far after a smooth scroll"),
        ],
    )
    .await;
    play(
        STATE_PAGE,
        json!({}),
        vec![
            click("near", "Near button", false),
            shows("near after no scroll"),
            click("far", "Far button", false),
            shows("far after a jump"),
        ],
    )
    .await;
}

fn shade(frame: &RgbImage) -> i64 {
    i64::from(frame.get_pixel(10, 10).0[0])
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn scrolls_to_a_click_a_frame_at_a_time_in_a_smooth_take() {
    let (frames, _) = record(
        SHADED_PAGE,
        json!({}),
        vec![step(
            BrowserAction::Click,
            "far",
            json!({ "target": "Far target", "after": 0, "hoverMs": 0, "scrollSmoothly": true }),
        )],
    )
    .await;
    let shades: Vec<i64> = frames.iter().map(shade).collect();
    let moves: Vec<i64> = shades.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let moving = moves.iter().filter(|&&step| step > 0).count();
    assert!(moving >= 8, "{moving} frames moved: {shades:?}");
    assert!(moves.iter().all(|&step| step >= -2), "{shades:?}");
    assert!(
        moves.iter().all(|&step| step < 40),
        "no frame jumps: {shades:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn holds_a_mocked_reply_back_for_its_delay() {
    let mock = |delay: u32| {
        step(
            BrowserAction::MockRoute,
            "mock",
            json!({ "url": "**/api/**", "body": r#"{"ok":true}"#, "delayMs": delay }),
        )
    };
    let load = || {
        step(
            BrowserAction::Click,
            "load",
            json!({ "target": "Load data", "after": 0, "hoverMs": 0 }),
        )
    };
    play(
        STATE_PAGE,
        json!({}),
        vec![mock(0), load(), shows("ok true at once")],
    )
    .await;
    play(
        STATE_PAGE,
        json!({}),
        vec![mock(1000), load(), shows("ok true after a second")],
    )
    .await;
}

#[allow(clippy::many_single_char_names)]
fn is_red(frame: &RgbImage, x: u32, y: u32) -> bool {
    let [r, g, b] = frame.get_pixel(x, y).0;
    r > 200 && g < 60 && b < 60
}

fn red_box(frame: &RgbImage) -> Option<(u32, u32, u32, u32)> {
    let mut found: Option<(u32, u32, u32, u32)> = None;
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            if is_red(frame, x, y) {
                found = Some(match found {
                    None => (x, y, x, y),
                    Some((l, t, r, b)) => (l.min(x), t.min(y), r.max(x), b.max(y)),
                });
            }
        }
    }
    found
}

fn redness(frame: &RgbImage, y: u32) -> f64 {
    let xs = WIDTH / 2 - 20..WIDTH / 2 + 20;
    let count = f64::from(xs.end - xs.start);
    xs.map(|x| {
        let [r, g, b] = frame.get_pixel(x, y).0;
        f64::from(r) - f64::midpoint(f64::from(g), f64::from(b))
    })
    .sum::<f64>()
        / count
        / 255.0
}

fn greys_above(frame: &RgbImage, y: u32) -> usize {
    (0..WIDTH)
        .filter(|&x| {
            let [r, _, _] = frame.get_pixel(x, y).0;
            (60..196).contains(&r)
        })
        .count()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn draws_the_chapter_card_as_set() {
    let chapter = |id: &str, position: &str, blur: u32| {
        step(
            BrowserAction::Chapter,
            id,
            json!({
                "title": "A chapter title long enough to fill the whole width of its card",
                "showInVideo": true,
                "durationMs": 1000,
                "afterMs": 0,
                "cardColor": "#ff0000",
                "edgeColor": "#ff0000",
                "textColor": "#0000ff",
                "titleSize": 20,
                "cardWidth": 400,
                "position": position,
                "blur": blur,
                "fadeMs": 300,
            }),
        )
    };
    let (frames, log) = record(
        STRIPED_PAGE,
        json!({}),
        vec![chapter("top", "top", 8), chapter("bottom", "bottom", 0)],
    )
    .await;
    let starts: Vec<usize> = log
        .actions
        .iter()
        .filter(|a| a.kind == ActionKind::Chapter)
        .map(|a| frame_at(a.t))
        .collect();
    assert_eq!(starts.len(), 2);
    let held = |start: usize| &frames[start + 20];

    let (left, top, right, _) = red_box(held(starts[0])).expect("the top card shows");
    assert_eq!(top, 48, "the top card starts at the padding");
    assert_eq!(right - left + 1, 400, "the card is as wide as set");
    assert!(
        greys_above(held(starts[0]), 20) > 400,
        "the page behind is blurred"
    );

    let (_, _, _, bottom) = red_box(held(starts[1])).expect("the bottom card shows");
    assert_eq!(
        bottom,
        HEIGHT - 48 - 1,
        "the bottom card ends at the padding"
    );
    assert!(
        greys_above(held(starts[1]), 20) < 20,
        "nothing is blurred with no blur"
    );

    let row = top + 10;
    let rising: Vec<f64> = (0..=12)
        .map(|k| redness(&frames[starts[0] + k], row))
        .collect();
    assert!(rising[0] < 0.2, "{rising:?}");
    assert!(
        rising.iter().any(|&a| (0.3..0.8).contains(&a)),
        "{rising:?}"
    );
    assert!(rising[12] > 0.95, "{rising:?}");
    assert!(rising.windows(2).all(|w| w[1] >= w[0] - 0.02), "{rising:?}");

    assert!(
        (39..=40).contains(&(starts[1] - starts[0])),
        "the card shows for a second, then fades out over 300 ms: {starts:?}"
    );
    let falling: Vec<f64> = (starts[0] + 28..starts[1])
        .map(|k| redness(&frames[k], row))
        .collect();
    assert!(falling[0] > 0.95, "{falling:?}");
    assert!(
        falling.iter().any(|&a| (0.2..0.8).contains(&a)),
        "{falling:?}"
    );
    assert!(
        falling.windows(2).all(|w| w[1] <= w[0] + 0.02),
        "{falling:?}"
    );
}
