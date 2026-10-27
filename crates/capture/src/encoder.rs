use std::path::{Path, PathBuf};

use opsis_media::codecs::{H264, h264_args, strings};
use opsis_media::ffmpeg::{FramePipe, MediaTools, ToolError};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const ENCODE_CRF: f64 = 16.0;
const ALREADY_FINISHED: &str = "The video encoder already finished.";

#[derive(Debug, Clone, thiserror::Error)]
pub enum EncodeError {
    #[error("{0}")]
    Failed(String),
    #[error("Cancelled")]
    Cancelled,
}

impl From<ToolError> for EncodeError {
    fn from(error: ToolError) -> Self {
        match error {
            ToolError::Cancelled => Self::Cancelled,
            ToolError::Failed { output } => Self::Failed(output),
            other => Self::Failed(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FrameImage {
    #[default]
    Jpeg,
    Png,
}

impl FrameImage {
    fn decoder(self) -> &'static str {
        match self {
            Self::Jpeg => "mjpeg",
            Self::Png => "png",
        }
    }
}

pub fn encoder_args(fps: u32, image: FrameImage, output: &Path) -> Vec<String> {
    let mut args = strings(&[
        "-f",
        "image2pipe",
        "-framerate",
        &fps.to_string(),
        "-c:v",
        image.decoder(),
        "-i",
        "-",
        "-an",
        "-vf",
        "pad=ceil(iw/2)*2:ceil(ih/2)*2,format=yuv420p",
    ]);
    args.extend(h264_args(&H264 {
        crf: ENCODE_CRF,
        preset: Some("medium"),
        keyframe_every: None,
        faststart: true,
    }));
    args.extend([
        "-r".to_owned(),
        fps.to_string(),
        output.to_string_lossy().into_owned(),
    ]);
    args
}

enum Message {
    Frame(Vec<u8>, f64),
    Origin(f64),
    Finish(f64),
}

pub struct Encoded {
    pub frames: u64,
    pub duration_ms: f64,
    pub origin_ms: Option<f64>,
}

#[derive(Clone)]
pub struct FrameFeed(mpsc::UnboundedSender<Message>);

impl FrameFeed {
    pub fn offer(&self, jpeg: Vec<u8>, t: f64) {
        self.send(Message::Frame(jpeg, t));
    }

    pub fn set_origin(&self, t: f64) {
        self.send(Message::Origin(t));
    }

    fn send(&self, message: Message) {
        if self.0.send(message).is_err() {
            log::debug!("{ALREADY_FINISHED}");
        }
    }
}

pub struct FrameEncoder {
    pub output: PathBuf,
    feed: FrameFeed,
    writer: JoinHandle<Result<Encoded, EncodeError>>,
    cancel: CancellationToken,
}

struct Writer {
    pipe: FramePipe,
    frame_ms: f64,
    origin: Option<f64>,
    tick: u64,
    last: Option<Vec<u8>>,
}

impl Writer {
    async fn advance(&mut self, t: f64, inclusive: bool) -> Result<(), EncodeError> {
        let (Some(origin), Some(last)) = (self.origin, self.last.as_ref()) else {
            return Ok(());
        };
        loop {
            let at = origin + self.tick as f64 * self.frame_ms;
            if !(if inclusive { at <= t } else { at < t }) {
                return Ok(());
            }
            self.pipe
                .stdin
                .write_all(last)
                .await
                .map_err(|e| EncodeError::Failed(format!("The video encoder stopped: {e}")))?;
            self.tick += 1;
        }
    }

    async fn run(
        mut self,
        mut receiver: mpsc::UnboundedReceiver<Message>,
        cancel: CancellationToken,
    ) -> Result<Encoded, EncodeError> {
        let mut failed = None;
        while let Some(message) = tokio::select! {
            message = receiver.recv() => message,
            () = cancel.cancelled() => None,
        } {
            match message {
                Message::Origin(t) => self.origin = Some(t),
                Message::Frame(jpeg, t) => {
                    if failed.is_none() {
                        failed = self.advance(t, false).await.err();
                    }
                    self.last = Some(jpeg);
                }
                Message::Finish(t) => {
                    if let Some(error) = failed {
                        return Err(self.pipe.done.await.err().map_or(error, EncodeError::from));
                    }
                    self.advance(t, true).await?;
                    let FramePipe { mut stdin, done } = self.pipe;
                    stdin.flush().await.map_err(|e| {
                        EncodeError::Failed(format!("The video encoder stopped: {e}"))
                    })?;
                    drop(stdin);
                    done.await?;
                    return Ok(Encoded {
                        frames: self.tick,
                        duration_ms: self.tick as f64 * self.frame_ms,
                        origin_ms: self.origin,
                    });
                }
            }
        }
        Err(EncodeError::Cancelled)
    }
}

impl FrameEncoder {
    pub fn start(
        tools: &dyn MediaTools,
        output: &Path,
        fps: u32,
        image: FrameImage,
    ) -> Result<Self, EncodeError> {
        let cancel = CancellationToken::new();
        let pipe = tools.pipe(&encoder_args(fps, image, output), Some(cancel.clone()))?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let writer = Writer {
            pipe,
            frame_ms: 1000.0 / f64::from(fps),
            origin: None,
            tick: 0,
            last: None,
        };
        Ok(Self {
            output: output.to_path_buf(),
            feed: FrameFeed(sender),
            writer: tokio::spawn(writer.run(receiver, cancel.clone())),
            cancel,
        })
    }

    pub fn feed(&self) -> FrameFeed {
        self.feed.clone()
    }

    pub fn set_origin(&self, t: f64) {
        self.feed.set_origin(t);
    }

    pub fn offer(&self, jpeg: Vec<u8>, t: f64) {
        self.feed.offer(jpeg, t);
    }

    pub async fn finish(self, t: f64) -> Result<Encoded, EncodeError> {
        if self.feed.0.send(Message::Finish(t)).is_err() {
            return Err(EncodeError::Failed(ALREADY_FINISHED.to_owned()));
        }
        match self.writer.await {
            Ok(done) => done,
            Err(error) => Err(EncodeError::Failed(error.to_string())),
        }
    }

    pub async fn abort(self) {
        self.cancel.cancel();
        drop(self.feed);
        if let Err(error) = self.writer.await {
            log::debug!("stopping the video encoder: {error}");
        }
    }
}
