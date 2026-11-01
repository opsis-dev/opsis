#![allow(
    dead_code,
    reason = "each test file uses only some of the shared helpers"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use opsis_capture::take::{SignInError, SignIns, TakeEnv};
use opsis_compile::nodes::browser::OPEN_BROWSER_TYPE;
use opsis_compile::registry::default_params;
use opsis_engine::paths::downloaded_browser;
use opsis_media::ffmpeg::{FramePipe, MediaTools, RunOptions, ToolError, ToolFuture};
use opsis_probe::Probe;
use opsis_types::values::{BrowserAction, BrowserSessionValue, SessionBrowser, SessionStep};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

pub struct NoSignIns;

impl SignIns for NoSignIns {
    fn read(&self, _: &str) -> Result<Option<String>, SignInError> {
        Ok(None)
    }
}

pub fn engine_dir() -> PathBuf {
    std::env::var_os("OPSIS_ENGINE_DIR").map_or_else(
        || opsis_golden::home().join(".config/Opsis/engine"),
        PathBuf::from,
    )
}

pub fn take_env(dir: &Path, sign_ins: Arc<dyn SignIns>) -> TakeEnv {
    TakeEnv {
        project_dir: dir.to_path_buf(),
        executable: downloaded_browser(&engine_dir()).unwrap(),
        temp_dir: dir.to_path_buf(),
        live: None,
        trace: false,
        sign_ins,
        cancel: CancellationToken::new(),
        log: Arc::new(|_, _| {}),
        step: Arc::new(|_, _, _| {}),
    }
}

pub fn step(kind: BrowserAction, id: &str, params: Value) -> SessionStep {
    let mut all = default_params(kind.kind());
    if let Value::Object(params) = params {
        all.extend(params);
    }
    SessionStep {
        id: id.to_owned(),
        kind,
        label: None,
        params: all,
    }
}

pub fn session(browser: Value, steps: Vec<SessionStep>) -> BrowserSessionValue {
    let mut params = default_params(OPEN_BROWSER_TYPE);
    if let Value::Object(browser) = browser {
        params.extend(browser);
    }
    BrowserSessionValue {
        browser: SessionBrowser {
            node_id: "browser".into(),
            params,
        },
        steps,
        address: None,
    }
}

pub async fn serve(page: &'static str) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut request = [0; 4096];
            if socket.read(&mut request).await.is_ok() {
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{page}",
                    page.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
    });
    port
}

pub struct FakeTools {
    pub written: Arc<Mutex<Vec<u8>>>,
    pub buffer: usize,
    pub busy_for: Duration,
    pub crash: Option<&'static str>,
}

impl FakeTools {
    pub fn new(buffer: usize, crash: Option<&'static str>) -> Self {
        Self {
            written: Arc::default(),
            buffer,
            busy_for: Duration::ZERO,
            crash,
        }
    }

    pub fn text(&self) -> String {
        String::from_utf8(self.written.lock().unwrap().clone()).unwrap()
    }
}

impl MediaTools for FakeTools {
    fn run<'a>(&'a self, _: &'a [String], _: RunOptions<'a>) -> ToolFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }

    fn probe<'a>(&'a self, _: &'a Path) -> ToolFuture<'a, Probe> {
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
