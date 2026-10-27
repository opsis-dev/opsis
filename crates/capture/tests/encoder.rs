use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opsis_capture::encoder::{FrameEncoder, FrameImage};
use opsis_media::ffmpeg::{FramePipe, MediaTools, ProbeResult, RunOptions, ToolError, ToolFuture};
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

struct FakeTools {
    written: Arc<Mutex<Vec<u8>>>,
    buffer: usize,
    busy_for: Duration,
    crash: Option<&'static str>,
}

impl FakeTools {
    fn new(buffer: usize, crash: Option<&'static str>) -> Self {
        Self {
            written: Arc::default(),
            buffer,
            busy_for: Duration::ZERO,
            crash,
        }
    }

    fn text(&self) -> String {
        String::from_utf8(self.written.lock().unwrap().clone()).unwrap()
    }
}

impl MediaTools for FakeTools {
    fn run<'a>(&'a self, _: &'a [String], _: RunOptions<'a>) -> ToolFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }

    fn probe<'a>(&'a self, _: &'a Path) -> ToolFuture<'a, ProbeResult> {
        Box::pin(async {
            Err(ToolError::Failed {
                output: "no probe in this test".into(),
            })
        })
    }

    fn pipe(&self, _: &[String], _: Option<CancellationToken>) -> Result<FramePipe, ToolError> {
        let (stdin, mut reader) = tokio::io::duplex(self.buffer);
        let written = Arc::clone(&self.written);
        let (crash, busy_for) = (self.crash, self.busy_for);
        let reading = tokio::spawn(async move {
            tokio::time::sleep(busy_for).await;
            let mut all = Vec::new();
            let read = reader.read_to_end(&mut all).await;
            written.lock().unwrap().extend(all);
            read
        });
        let done = Box::pin(async move {
            let read = reading.await.map_err(|e| ToolError::Failed {
                output: e.to_string(),
            })?;
            if let Some(message) = crash {
                return Err(ToolError::Failed {
                    output: message.into(),
                });
            }
            read.map(|_| ()).map_err(|e| ToolError::Failed {
                output: e.to_string(),
            })
        });
        Ok(FramePipe {
            stdin: Box::pin(stdin),
            done,
        })
    }
}

#[tokio::test]
async fn holds_each_frame_until_the_next_one_arrives_at_constant_frame_rate_from_the_origin() {
    let tools = FakeTools::new(1024, None);
    let encoder = FrameEncoder::start(&tools, Path::new("out.mp4"), 10, FrameImage::Jpeg).unwrap();
    encoder.offer(b"A".to_vec(), 900.0);
    encoder.set_origin(1000.0);
    encoder.offer(b"B".to_vec(), 1250.0);
    encoder.offer(b"C".to_vec(), 1400.0);
    let done = encoder.finish(1600.0).await.unwrap();
    assert_eq!(tools.text(), "AAABCCC");
    assert_eq!((done.frames, done.duration_ms), (7, 700.0));
}

#[tokio::test]
async fn drops_frames_from_before_the_origin_except_the_latest_one() {
    let tools = FakeTools::new(1024, None);
    let encoder = FrameEncoder::start(&tools, Path::new("out.mp4"), 10, FrameImage::Jpeg).unwrap();
    encoder.offer(b"X".to_vec(), 100.0);
    encoder.offer(b"Y".to_vec(), 200.0);
    encoder.set_origin(500.0);
    encoder.finish(600.0).await.unwrap();
    assert_eq!(tools.text(), "YY");
}

#[tokio::test]
async fn takes_frames_at_once_while_the_encoder_is_still_busy_and_writes_them_all_later() {
    let tools = FakeTools {
        busy_for: Duration::from_millis(300),
        ..FakeTools::new(1, None)
    };
    let encoder = FrameEncoder::start(&tools, Path::new("out.mp4"), 10, FrameImage::Jpeg).unwrap();
    encoder.set_origin(1000.0);
    let offered = Instant::now();
    for (frame, t) in [(b"A", 1000.0), (b"B", 1250.0), (b"C", 1400.0)] {
        encoder.offer(frame.to_vec(), t);
    }
    assert!(offered.elapsed() < Duration::from_millis(50));
    encoder.finish(1600.0).await.unwrap();
    assert_eq!(tools.text(), "AAABCCC");
}

#[tokio::test]
async fn reports_a_failed_encoder_when_the_take_finishes() {
    let tools = FakeTools::new(1024, Some("The encoder crashed."));
    let encoder = FrameEncoder::start(&tools, Path::new("out.mp4"), 10, FrameImage::Jpeg).unwrap();
    encoder.set_origin(1000.0);
    encoder.offer(b"A".to_vec(), 1000.0);
    encoder.offer(b"B".to_vec(), 1250.0);
    let failed = encoder.finish(1600.0).await.err().map(|e| e.to_string());
    assert_eq!(failed.as_deref(), Some("The encoder crashed."));
}
