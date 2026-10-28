use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::protocol::{CancelJob, SidecarCall, SidecarMessage, SidecarRequest};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone)]
pub struct SidecarCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}

pub type OnLine = Arc<dyn Fn(&str) + Send + Sync>;
pub type OnProgress<'a> = &'a (dyn Fn(f64) + Send + Sync);

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    #[error("The render engine couldn't start: {0}")]
    Start(std::io::Error),
    #[error("The render engine stopped unexpectedly.")]
    Stopped,
    #[error("Cancelled")]
    Cancelled,
    #[error("{0}")]
    Failed(String),
}

type Reply = Result<(), SidecarError>;

struct Waiting {
    reply: oneshot::Sender<Reply>,
    progress: mpsc::UnboundedSender<f64>,
}

type Pending = Arc<Mutex<HashMap<u32, Waiting>>>;

struct Running {
    _child: Child,
    stdin: ChildStdin,
    pending: Pending,
    stopped: CancellationToken,
}

pub struct Sidecar {
    command: SidecarCommand,
    on_log: OnLine,
    running: tokio::sync::Mutex<Option<Running>>,
    next_id: AtomicU32,
}

async fn read_lines(stream: impl AsyncRead + Unpin, mut each: impl FnMut(&str)) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        each(&line);
    }
}

fn hand_over<T>(sent: Result<(), T>, id: u32) {
    if sent.is_err() {
        log::debug!("render job {id} finished before the render engine answered");
    }
}

fn deliver(pending: &Pending, line: &str, on_log: &OnLine) {
    let Ok(message) = serde_json::from_str::<SidecarMessage>(line) else {
        on_log(line);
        return;
    };
    let mut waiting = pending.lock().expect("pending replies");
    match message {
        SidecarMessage::Progress { params, .. } => {
            if let Some(job) = waiting.get(&params.id) {
                hand_over(job.progress.send(params.progress), params.id);
            }
        }
        SidecarMessage::Failed { id, error } => {
            if let Some(job) = waiting.remove(&id) {
                hand_over(job.reply.send(Err(SidecarError::Failed(error.message))), id);
            }
        }
        SidecarMessage::Done { id, .. } => {
            if let Some(job) = waiting.remove(&id) {
                hand_over(job.reply.send(Ok(())), id);
            }
        }
    }
}

impl Sidecar {
    pub fn new(command: SidecarCommand, on_log: OnLine) -> Self {
        Self {
            command,
            on_log,
            running: tokio::sync::Mutex::new(None),
            next_id: AtomicU32::new(1),
        }
    }

    fn spawn(&self) -> Result<Running, SidecarError> {
        let mut command = Command::new(&self.command.program);
        command
            .args(&self.command.args)
            .envs(self.command.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(cwd) = &self.command.cwd {
            command.current_dir(cwd);
        }
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        let mut child = command.spawn().map_err(SidecarError::Start)?;
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            return Err(SidecarError::Start(std::io::Error::other(
                "no pipes to the render engine",
            )));
        };
        let pending: Pending = Arc::default();
        let stopped = CancellationToken::new();
        let replies = Arc::clone(&pending);
        let on_log = Arc::clone(&self.on_log);
        let ended = stopped.clone();
        tokio::spawn(async move {
            read_lines(stdout, |line| deliver(&replies, line, &on_log)).await;
            ended.cancel();
            replies.lock().expect("pending replies").clear();
        });
        let on_log = Arc::clone(&self.on_log);
        tokio::spawn(read_lines(stderr, move |line| on_log(line)));
        Ok(Running {
            _child: child,
            stdin,
            pending,
            stopped,
        })
    }

    async fn send(
        &self,
        request: &SidecarRequest,
        waiting: Option<Waiting>,
    ) -> Result<(), SidecarError> {
        let mut running = self.running.lock().await;
        if running.as_ref().is_none_or(|r| r.stopped.is_cancelled()) {
            *running = Some(self.spawn()?);
        }
        let sidecar = running.as_mut().ok_or(SidecarError::Stopped)?;
        if let Some(waiting) = waiting {
            sidecar
                .pending
                .lock()
                .expect("pending replies")
                .insert(request.id, waiting);
        }
        let mut line =
            serde_json::to_vec(request).map_err(|e| SidecarError::Failed(e.to_string()))?;
        line.push(b'\n');
        let written = async {
            sidecar.stdin.write_all(&line).await?;
            sidecar.stdin.flush().await
        }
        .await;
        written.map_err(|error| {
            log::warn!("writing to the render engine: {error}");
            sidecar
                .pending
                .lock()
                .expect("pending replies")
                .remove(&request.id);
            SidecarError::Stopped
        })
    }

    fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn call(
        &self,
        call: SidecarCall,
        cancel: &CancellationToken,
        on_progress: Option<OnProgress<'_>>,
    ) -> Result<(), SidecarError> {
        let id = self.next_id();
        let (reply, mut replied) = oneshot::channel();
        let (progress, mut progressed) = mpsc::unbounded_channel();
        self.send(
            &SidecarRequest { id, call },
            Some(Waiting { reply, progress }),
        )
        .await?;
        let mut cancelled = false;
        loop {
            tokio::select! {
                Some(p) = progressed.recv() => {
                    if let Some(report) = on_progress {
                        report(p);
                    }
                }
                () = cancel.cancelled(), if !cancelled => {
                    cancelled = true;
                    self.send(&SidecarRequest { id: self.next_id(), call: SidecarCall::Cancel(CancelJob { target: id }) }, None).await?;
                }
                answer = &mut replied => {
                    return match answer {
                        _ if cancelled => Err(SidecarError::Cancelled),
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(error)) => Err(error),
                        Err(_) => Err(SidecarError::Stopped),
                    };
                }
            }
        }
    }

    pub async fn ping(&self) -> Result<(), SidecarError> {
        self.call(SidecarCall::Ping, &CancellationToken::new(), None)
            .await
    }
}
