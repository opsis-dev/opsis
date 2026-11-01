use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opsis_capture::live::FrameSink;
use opsis_engine::paths::downloaded_browser;
use opsis_stream::host::ViewEnv;
use opsis_stream::view::{StreamView, ViewEvent, ViewInput, ViewSink, ViewSize};
use opsis_types::actions::MouseButton;
use tokio::sync::Notify;

const FLASH: &str = "data:text/html,<body style='margin:0;background:white' onmousedown=\"document.body.style.background = document.body.style.background === 'black' ? 'white' : 'black'\"></body>";

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
    _temp: tempfile::TempDir,
}

fn setup() -> Setup {
    let temp = tempfile::tempdir().unwrap();
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    Setup {
        env: ViewEnv {
            executable: downloaded_browser(&engine).unwrap(),
            temp_dir: temp.path().to_path_buf(),
        },
        _temp: temp,
    }
}

const SIZES: [(u32, u32); 2] = [(960, 540), (1920, 1080)];

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
