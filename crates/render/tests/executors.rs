use std::path::PathBuf;
use std::sync::Arc;

use opsis_capture::executors::{CaptureDeps, Login, LoginRequest, SignInStore, capture_executors};
use opsis_compile::registry::{node_def, registry};
use opsis_engine::paths::{ShellPlatform, engine_paths};
use opsis_media::executors::{Tools, media_executors};
use opsis_media::ffmpeg::Ffmpeg;
use opsis_render::executors::{RenderEngine, output_executors};
use opsis_render::export_project::ExportSources;
use opsis_render::sidecar::{Sidecar, SidecarCommand};
use opsis_runner::context::{BoxFuture, ExecError, ExecutorFn, Executors};
use tokio_util::sync::CancellationToken;

struct NoSignIns;

impl SignInStore for NoSignIns {
    fn read(&self, _: &std::path::Path, _: &str) -> Result<Option<String>, String> {
        Ok(None)
    }

    fn write(&self, _: &std::path::Path, _: &str, _: &str) -> Result<(), String> {
        Ok(())
    }
}

impl Login for NoSignIns {
    fn login(&self, _: LoginRequest, _: CancellationToken) -> BoxFuture<Result<String, ExecError>> {
        Box::pin(async { Err(ExecError::new("No sign-in window in tests.")) })
    }
}

fn all_executors() -> Vec<(&'static str, ExecutorFn)> {
    let tools: Tools = Arc::new(Ffmpeg::new(&engine_paths(
        &PathBuf::from("/nowhere"),
        ShellPlatform::current().unwrap(),
    )));
    let command = SidecarCommand {
        program: "bun".into(),
        args: Vec::new(),
        env: Vec::new(),
        cwd: None,
    };
    let engine = Arc::new(RenderEngine {
        sidecar: Arc::new(Sidecar::new(command, Arc::new(|_| {}))),
        tools: Arc::clone(&tools),
        serve_url: String::new(),
        binaries_directory: None,
        sources: ExportSources {
            root: PathBuf::from("/nowhere"),
            packaged: false,
        },
    });
    let capture = Arc::new(CaptureDeps {
        tools: Arc::clone(&tools),
        sign_ins: Arc::new(NoSignIns),
        login: Arc::new(NoSignIns),
        live: None,
    });
    let mut all = media_executors(&tools);
    all.extend(output_executors(&engine));
    all.extend(opsis_image::executors::image_executors());
    all.extend(capture_executors(&capture));
    all
}

#[test]
fn exist_for_every_node_that_executes() {
    let executors = all_executors();
    let missing: Vec<&str> = registry()
        .all()
        .iter()
        .filter(|d| d.executes && !executors.iter().any(|(kind, _)| *kind == d.kind))
        .map(|d| &*d.kind)
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn are_each_for_a_node_that_executes_with_no_node_twice() {
    let executors = all_executors();
    let not_executing: Vec<&str> = executors
        .iter()
        .map(|(kind, _)| *kind)
        .filter(|kind| !node_def(kind).is_some_and(|d| d.executes))
        .collect();
    assert!(not_executing.is_empty(), "{not_executing:?}");
    assert!(Executors::default().extend(executors).is_ok());
}

#[test]
fn refuse_a_second_executor_for_the_same_node() {
    let mut executors = all_executors();
    let (kind, run) = executors.remove(0);
    let mut table = Executors::default();
    table.add(kind, Arc::clone(&run)).unwrap();
    assert_eq!(
        table.add(kind, run).unwrap_err().to_string(),
        format!("Two executors for {kind}")
    );
}
