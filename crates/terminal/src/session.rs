use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use opsis_engine::children::{forget_child, remember_child};
use opsis_vt::{Event as VtEvent, Terminal, VtError};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::shell::ShellCommand;

const READ_BYTES: usize = 16 * 1024;
const SCROLLBACK: u32 = 10_000;
const EXIT_WAIT: Duration = Duration::from_secs(2);
const EXIT_POLL: Duration = Duration::from_millis(20);
pub const CELL_WIDTH_PX: u32 = 10;
pub const CELL_HEIGHT_PX: u32 = 20;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Couldn't open a terminal: {0}")]
    Open(String),
    #[error("Couldn't start the shell: {0}")]
    Spawn(String),
    #[error("Couldn't write to the terminal: {0}")]
    Write(#[from] std::io::Error),
    #[error(transparent)]
    Terminal(#[from] VtError),
}

#[derive(Debug)]
pub struct Chunk {
    pub at: Duration,
    pub bytes: Vec<u8>,
    pub events: Vec<VtEvent>,
}

type Shared<T> = Arc<Mutex<T>>;

fn lock<T>(shared: &Mutex<T>) -> MutexGuard<'_, T> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct Session {
    start: Instant,
    terminal: Shared<Terminal>,
    writer: Shared<Box<dyn Write + Send>>,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Box<dyn Child + Send + Sync>,
    pid: Option<u32>,
    reader: Option<JoinHandle<()>>,
    chunks: Receiver<Chunk>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("start", &self.start)
            .field("pid", &self.pid)
            .finish_non_exhaustive()
    }
}

fn read_loop(
    mut reader: Box<dyn Read + Send>,
    terminal: &Shared<Terminal>,
    writer: &Shared<Box<dyn Write + Send>>,
    sender: &Sender<Chunk>,
    start: Instant,
) {
    let mut buffer = vec![0; READ_BYTES];
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let at = start.elapsed();
        let bytes = buffer[..n].to_vec();
        let (replies, events) = {
            let mut t = lock(terminal);
            t.feed(&bytes);
            (t.take_replies(), t.take_events())
        };
        if !replies.is_empty() {
            let mut w = lock(writer);
            if w.write_all(&replies).and_then(|()| w.flush()).is_err() {
                break;
            }
        }
        if sender.send(Chunk { at, bytes, events }).is_err() {
            break;
        }
    }
}

impl Session {
    pub fn start(command: &ShellCommand, cols: u16, rows: u16) -> Result<Self, SessionError> {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: u16::try_from(u32::from(cols) * CELL_WIDTH_PX).unwrap_or(u16::MAX),
                pixel_height: u16::try_from(u32::from(rows) * CELL_HEIGHT_PX).unwrap_or(u16::MAX),
            })
            .map_err(|e| SessionError::Open(e.to_string()))?;
        let mut builder = CommandBuilder::new(&command.program);
        builder.args(&command.args);
        for (name, value) in &command.env {
            builder.env(name, value);
        }
        builder.cwd(&command.cwd);
        let child = pair
            .slave
            .spawn_command(builder)
            .map_err(|e| SessionError::Spawn(e.to_string()))?;
        drop(pair.slave);
        let pid = child.process_id();
        if let Some(pid) = pid {
            remember_child(pid);
        }
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| SessionError::Open(e.to_string()))?;
        let writer: Shared<Box<dyn Write + Send>> = Arc::new(Mutex::new(
            pair.master
                .take_writer()
                .map_err(|e| SessionError::Open(e.to_string()))?,
        ));
        let mut terminal = Terminal::new(cols, rows, SCROLLBACK)?;
        terminal.set_cell_size(CELL_WIDTH_PX, CELL_HEIGHT_PX);
        let terminal = Arc::new(Mutex::new(terminal));
        let start = Instant::now();
        let (sender, chunks) = channel();
        let thread = {
            let terminal = Arc::clone(&terminal);
            let writer = Arc::clone(&writer);
            std::thread::Builder::new()
                .name("opsis-terminal-reader".to_owned())
                .spawn(move || read_loop(reader, &terminal, &writer, &sender, start))
                .map_err(SessionError::Write)?
        };
        Ok(Self {
            start,
            terminal,
            writer,
            master: Some(pair.master),
            child,
            pid,
            reader: Some(thread),
            chunks,
        })
    }

    pub fn now(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn write(&self, bytes: &[u8]) -> Result<Duration, SessionError> {
        let at = self.now();
        let mut w = lock(&self.writer);
        w.write_all(bytes)?;
        w.flush()?;
        Ok(at)
    }

    pub fn terminal(&self) -> MutexGuard<'_, Terminal> {
        lock(&self.terminal)
    }

    pub fn next_chunk(&self, wait: Duration) -> Option<Chunk> {
        self.chunks.recv_timeout(wait).ok()
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), SessionError> {
        if let Some(master) = &self.master {
            master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: u16::try_from(u32::from(cols) * CELL_WIDTH_PX).unwrap_or(u16::MAX),
                    pixel_height: u16::try_from(u32::from(rows) * CELL_HEIGHT_PX)
                        .unwrap_or(u16::MAX),
                })
                .map_err(|e| SessionError::Open(e.to_string()))?;
        }
        self.terminal().resize(cols, rows)?;
        Ok(())
    }

    pub fn finish(mut self, exit_line: &[u8]) -> Finished {
        let _ = self.write(exit_line);
        let deadline = Instant::now() + EXIT_WAIT;
        let mut status = None;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(done)) => {
                    status = Some(done);
                    break;
                }
                Ok(None) => std::thread::sleep(EXIT_POLL),
                Err(_) => break,
            }
        }
        if status.is_none() {
            let _ = self.child.kill();
            status = self.child.wait().ok();
        }
        if let Some(pid) = self.pid {
            forget_child(pid);
        }
        let ended = self.now();
        drop(self.master.take());
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        let rest = self.chunks.try_iter().collect();
        Finished {
            ended,
            exit: status.map(|s| i32::try_from(s.exit_code()).unwrap_or(i32::MAX)),
            rest,
        }
    }
}

#[derive(Debug)]
pub struct Finished {
    pub ended: Duration,
    pub exit: Option<i32>,
    pub rest: Vec<Chunk>,
}
