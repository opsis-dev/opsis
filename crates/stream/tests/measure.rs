use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use opsis_capture::browser::shell_in;
use opsis_capture::live::FrameSink;
use opsis_compile::compile::compile_workflow;
use opsis_project::last_run::read_last_run;
use opsis_stream::host::ViewEnv;
use opsis_stream::preview::{Preview, PreviewDirs};
use opsis_stream::view::{MouseButton, StreamView, ViewEvent, ViewInput, ViewSink, ViewSize};
use opsis_types::timeline::{PreviewQuality, TimelineSpec};
use opsis_types::workflow::Workflow;
use serde_json::Value;
use tokio::sync::Notify;

const CLOCK_TICKS: f64 = 100.0;
const INTRO_END: f64 = 75.0;
const FLASH: &str = "data:text/html,<body style='margin:0;background:white' onmousedown=\"document.body.style.background = document.body.style.background === 'black' ? 'white' : 'black'\"></body>";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn film(project: &Path, title_background: Option<&str>) -> TimelineSpec {
    let mut json: Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("examples/todomvc/project.opsis.json")).unwrap(),
    )
    .unwrap();
    if let Some(background) = title_background {
        for node in json["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter(|n| n["id"] == "intro-title")
        {
            node["params"]["background"] = background.into();
        }
    }
    let workflow: Workflow = serde_json::from_value(json).unwrap();
    let results = read_last_run(project).unwrap().expect("a saved run").nodes;
    TimelineSpec {
        quality: Some(PreviewQuality::Fast),
        ..compile_workflow(&workflow, Some(&results))
            .specs
            .get("render")
            .cloned()
            .unwrap()
    }
}

#[derive(Default)]
struct Meter {
    frames: Mutex<Vec<(Instant, usize)>>,
    arrived: Notify,
}

impl FrameSink for Meter {
    fn frame(&self, jpeg: &[u8], _: f64, _: f64, _: f64) {
        self.frames
            .lock()
            .unwrap()
            .push((Instant::now(), jpeg.len()));
        self.arrived.notify_waiters();
    }
}

impl ViewSink for Meter {
    fn event(&self, _: ViewEvent) {}
}

impl Meter {
    fn count(&self) -> usize {
        self.frames.lock().unwrap().len()
    }

    async fn next_after(&self, after: Instant) -> Instant {
        loop {
            let waiting = self.arrived.notified();
            if let Some((at, _)) = self
                .frames
                .lock()
                .unwrap()
                .iter()
                .find(|(at, _)| *at > after)
            {
                return *at;
            }
            waiting.await;
        }
    }
}

fn cpu_ticks(marker: &Path) -> f64 {
    let marker = marker.to_string_lossy().into_owned();
    std::fs::read_dir("/proc")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_digit())
        })
        .filter(|e| {
            std::fs::read(e.path().join("cmdline"))
                .is_ok_and(|c| String::from_utf8_lossy(&c).contains(&marker))
        })
        .filter_map(|e| std::fs::read_to_string(e.path().join("stat")).ok())
        .map(|stat| {
            let fields: Vec<&str> = stat
                .rsplit_once(") ")
                .map(|(_, rest)| rest)
                .unwrap_or_default()
                .split_whitespace()
                .collect();
            fields
                .get(11)
                .and_then(|u| u.parse::<f64>().ok())
                .unwrap_or(0.0)
                + fields
                    .get(12)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
        })
        .sum()
}

fn median(mut list: Vec<f64>) -> (f64, f64) {
    list.sort_by(f64::total_cmp);
    (
        list[list.len() / 2],
        list[(list.len() * 9 / 10).min(list.len() - 1)],
    )
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

struct Setup {
    env: ViewEnv,
    project: PathBuf,
    _temp: tempfile::TempDir,
}

fn setup() -> Setup {
    let temp = tempfile::tempdir().unwrap();
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    let project = std::env::var_os("OPSIS_PREVIEW_PROJECT")
        .map(PathBuf::from)
        .expect("OPSIS_PREVIEW_PROJECT");
    Setup {
        env: ViewEnv {
            executable: shell_in(&engine).unwrap(),
            temp_dir: temp.path().to_path_buf(),
        },
        project,
        _temp: temp,
    }
}

async fn open_preview(
    setup: &Setup,
    size: ViewSize,
    quality: i64,
    spec: &TimelineSpec,
) -> (Preview, Arc<Meter>) {
    let (page, public) = (root().join("apps/preview/dist"), root().join("public"));
    let dirs = PreviewDirs {
        page: &page,
        public: &public,
        project: &setup.project,
    };
    let preview = Preview::open(&setup.env, &dirs, size, quality)
        .await
        .unwrap();
    let meter = Arc::new(Meter::default());
    preview.view().attach(meter.clone());
    preview.load(spec).await.unwrap();
    (preview, meter)
}

const SIZES: [(f64, f64); 2] = [(960.0, 540.0), (1920.0, 1080.0)];

#[tokio::test(flavor = "multi_thread")]
#[ignore = "a measurement: needs the dev engine, a built preview page and a recorded take"]
async fn stream_rate_while_playing() {
    let setup = setup();
    let spec = film(&setup.project, None);
    for (width, height) in SIZES {
        for quality in [70, 85] {
            let (preview, meter) = open_preview(
                &setup,
                ViewSize {
                    width,
                    height,
                    dpr: 1.0,
                },
                quality,
                &spec,
            )
            .await;
            let mut frames = preview.frames();
            preview.seek(120.0).await.unwrap();
            frames.wait_for(|f| *f == 120.0).await.unwrap();
            tokio::time::sleep(Duration::from_millis(500)).await;
            let (before, ticks, started) = (
                meter.count(),
                cpu_ticks(&setup.env.temp_dir),
                Instant::now(),
            );
            preview.play().await.unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
            let elapsed = started.elapsed().as_secs_f64();
            let cpu = (cpu_ticks(&setup.env.temp_dir) - ticks) / CLOCK_TICKS / elapsed * 100.0;
            let delivered = meter.frames.lock().unwrap()[before..].to_vec();
            let bytes = delivered.iter().map(|(_, b)| *b as f64).sum::<f64>()
                / delivered.len().max(1) as f64;
            eprintln!(
                "RATE {width}x{height} q{quality}: {:.1} frames/s, {:.0} KB a frame, browser CPU {cpu:.0}% of one core, film frame {}",
                delivered.len() as f64 / elapsed,
                bytes / 1024.0,
                *preview.frames().borrow()
            );
            preview.close().await;
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "a measurement: needs the dev engine"]
async fn input_to_frame_latency() {
    let setup = setup();
    for (width, height) in SIZES {
        let view = StreamView::open(
            &setup.env.executable,
            &setup.env.temp_dir,
            FLASH,
            ViewSize {
                width,
                height,
                dpr: 1.0,
            },
            80,
            &[],
        )
        .await
        .unwrap();
        let meter = Arc::new(Meter::default());
        view.attach(meter.clone());
        tokio::time::sleep(Duration::from_millis(800)).await;
        let mut samples = Vec::new();
        for _ in 0..20 {
            let sent = Instant::now();
            view.input(&ViewInput::MouseDown {
                x: 10.0,
                y: 10.0,
                button: MouseButton::Left,
                buttons: 1,
                click_count: 1,
                modifiers: 0,
            })
            .await
            .unwrap();
            let shown = meter.next_after(sent).await;
            samples.push(ms(shown - sent));
            view.input(&ViewInput::MouseUp {
                x: 10.0,
                y: 10.0,
                button: MouseButton::Left,
                buttons: 0,
                click_count: 1,
                modifiers: 0,
            })
            .await
            .unwrap();
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let (mid, p90) = median(samples);
        eprintln!("INPUT {width}x{height}: input to frame {mid:.0} ms median, {p90:.0} ms p90");
        view.close().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "a measurement: needs the dev engine, a built preview page and a recorded take"]
async fn seek_to_frame() {
    let setup = setup();
    let spec = film(&setup.project, Some("transparent"));
    for (width, height) in SIZES {
        let (preview, meter) = open_preview(
            &setup,
            ViewSize {
                width,
                height,
                dpr: 1.0,
            },
            80,
            &spec,
        )
        .await;
        let mut frames = preview.frames();
        for (what, targets) in [
            ("recording", [150.0, 330.0, 210.0, 390.0, 270.0]),
            ("shader", [12.0, 60.0, 24.0, 48.0, 36.0]),
        ] {
            let (mut reported, mut shown) = (Vec::new(), Vec::new());
            for at in targets {
                let sent = Instant::now();
                preview.seek(at).await.unwrap();
                frames.wait_for(|f| *f == at).await.unwrap();
                let told = Instant::now();
                reported.push(ms(told - sent));
                shown.push(ms(meter.next_after(told).await - sent));
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            let ((report, _), (show, _)) = (median(reported), median(shown));
            eprintln!(
                "SEEK {width}x{height} {what}: frame reported after {report:.0} ms, next frame on screen after {show:.0} ms (medians of 5)"
            );
        }
        preview.seek(36.0).await.unwrap();
        frames.wait_for(|f| *f == 36.0).await.unwrap();
        tokio::time::sleep(Duration::from_millis(800)).await;
        let still = preview.still().await.unwrap();
        if let Some(dir) = std::env::var_os("OPSIS_MEASURE_STILLS") {
            std::fs::write(
                PathBuf::from(dir).join(format!("shader-{width}.png")),
                &still,
            )
            .unwrap();
        }
        preview.close().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "a measurement: needs the dev engine, a built preview page and a recorded take"]
async fn clock_drift_over_two_minutes() {
    let setup = setup();
    let spec = film(&setup.project, None);
    let (preview, _) = open_preview(
        &setup,
        ViewSize {
            width: 960.0,
            height: 540.0,
            dpr: 1.0,
        },
        80,
        &spec,
    )
    .await;
    let fps = spec.fps;
    let last = spec.duration_in_frames - 1.0;
    let mut frames = preview.frames();
    let started = Instant::now();
    let (mut startups, mut after_intro, mut at_end) = (Vec::new(), Vec::new(), Vec::new());
    while started.elapsed() < Duration::from_secs(120) {
        preview.seek(0.0).await.unwrap();
        frames.wait_for(|f| *f == 0.0).await.unwrap();
        preview.play().await.unwrap();
        let run = Instant::now();
        frames.wait_for(|f| *f > 0.0).await.unwrap();
        let first = *frames.borrow();
        let startup = ms(run.elapsed()) - first / fps * 1000.0;
        startups.push(startup);
        let (mut intro, mut end) = (None, 0.0);
        loop {
            if tokio::time::timeout(Duration::from_secs(2), frames.changed())
                .await
                .is_err()
            {
                break;
            }
            let frame = *frames.borrow();
            let lag = ms(run.elapsed()) - frame / fps * 1000.0 - startup;
            if intro.is_none() && frame >= INTRO_END {
                intro = Some(lag);
            }
            end = lag;
            if frame >= last {
                break;
            }
        }
        after_intro.push(intro.unwrap_or(end));
        at_end.push(end);
    }
    let plays = at_end.len();
    let ((startup, _), (intro, _), (end, worst)) =
        (median(startups), median(after_intro), median(at_end));
    eprintln!(
        "CLOCK {plays} plays in 2 min: start-up {startup:.0} ms; after the start-up the reported frame falls behind wall time by {intro:.0} ms over the shader intro and {end:.0} ms by the end of the film (p90 {worst:.0} ms)"
    );
    preview.close().await;
}
