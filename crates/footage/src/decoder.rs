use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use image::RgbImage;

use crate::FootageError;
use crate::chain::{
    FootageSource, FrameInfo, decoder_args, frame_bytes, parse_frame, parse_time_base,
};

const FRAMES_AHEAD: usize = 3;
const KEPT_LOG_LINES: usize = 12;
const EXIT_POLLS: u32 = 200;
const EXIT_POLL: std::time::Duration = std::time::Duration::from_millis(10);
const STALL_LIMIT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct FootageTools {
    pub ffmpeg: PathBuf,
    pub temp: PathBuf,
}

#[derive(Debug)]
pub struct RawFrame {
    pub pts_us: i64,
    pub image: RgbImage,
}

type Log = Arc<Mutex<VecDeque<String>>>;

#[derive(Debug)]
pub struct Decoder {
    child: Child,
    frames: Receiver<RawFrame>,
    log: Log,
    pub start_us: i64,
}

fn keep_line(log: &Log, line: String) {
    if let Ok(mut lines) = log.lock() {
        if lines.len() == KEPT_LOG_LINES {
            lines.pop_front();
        }
        lines.push_back(line);
    }
}

fn read_log(stderr: ChildStderr, infos: &mpsc::Sender<FrameInfo>, log: &Log) {
    let mut base = None;
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        if let Some(found) = parse_time_base(&line) {
            base = Some(found);
        } else if let Some(info) = base.and_then(|b| parse_frame(&line, b)) {
            if infos.send(info).is_err() {
                return;
            }
        } else if !line.contains("Parsed_showinfo") {
            keep_line(log, line);
        }
    }
}

fn read_frames(
    mut stdout: ChildStdout,
    infos: &Receiver<FrameInfo>,
    frames: &SyncSender<RawFrame>,
) {
    for info in infos {
        let Some(bytes) = frame_bytes(info) else {
            return;
        };
        let mut buffer = vec![0; bytes];
        if stdout.read_exact(&mut buffer).is_err() {
            return;
        }
        let Some(image) = RgbImage::from_raw(info.width, info.height, buffer) else {
            return;
        };
        let frame = RawFrame {
            pts_us: info.pts_us,
            image,
        };
        if frames.send(frame).is_err() {
            return;
        }
    }
}

impl Decoder {
    pub fn start(
        tools: &FootageTools,
        source: &FootageSource,
        start_us: i64,
    ) -> Result<Self, FootageError> {
        let mut child = Command::new(&tools.ffmpeg)
            .args(decoder_args(source, start_us))
            .env("TMPDIR", &tools.temp)
            .env("TEMP", &tools.temp)
            .env("TMP", &tools.temp)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(FootageError::Start)?;
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            return Err(FootageError::Decode(
                "The decoder has no output.".to_owned(),
            ));
        };
        let log: Log = Arc::default();
        let (info_tx, info_rx) = mpsc::channel();
        let (frame_tx, frames) = sync_channel(FRAMES_AHEAD);
        let log_for_reader = Arc::clone(&log);
        thread::spawn(move || read_log(stderr, &info_tx, &log_for_reader));
        thread::spawn(move || read_frames(stdout, &info_rx, &frame_tx));
        Ok(Self {
            child,
            frames,
            log,
            start_us,
        })
    }

    fn exit_status(&mut self) -> Result<std::process::ExitStatus, FootageError> {
        for _ in 0..EXIT_POLLS {
            if let Some(status) = self.child.try_wait().map_err(FootageError::Start)? {
                return Ok(status);
            }
            thread::sleep(EXIT_POLL);
        }
        Err(FootageError::Decode(
            "The decoder stopped sending frames but kept running.".to_owned(),
        ))
    }

    pub fn next_frame(&mut self) -> Result<Option<RawFrame>, FootageError> {
        match self.frames.recv_timeout(STALL_LIMIT) {
            Ok(frame) => return Ok(Some(frame)),
            Err(mpsc::RecvTimeoutError::Timeout) => return Err(FootageError::Stalled),
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
        if self.exit_status()?.success() {
            return Ok(None);
        }
        let tail = self
            .log
            .lock()
            .map_or_default(|lines| lines.iter().cloned().collect::<Vec<_>>().join("\n"));
        Err(FootageError::Decode(tail))
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        if let Err(error) = self.child.kill()
            && error.kind() != std::io::ErrorKind::InvalidInput
        {
            log::warn!("ending a footage decoder: {error}");
        }
        if let Err(error) = self.child.wait() {
            log::warn!("waiting for a footage decoder: {error}");
        }
    }
}
