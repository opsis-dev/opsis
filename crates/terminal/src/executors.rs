use std::sync::Arc;

use opsis_compile::node::RECORDING_PORT;
use opsis_compile::nodes::terminal::{RECORD_TERMINAL_TYPE, RecordTerminal, TERMINAL_PORT};
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::numbers::millis;
use opsis_types::values::{
    ArtifactKind, CompileValue, PortValues, RecordingValue, TakeSource, TerminalSource,
};

use crate::take::{CAST_FILE, LOG_FILE, TakeError, TakeOptions, record_take, save_take};

fn failed(error: &TakeError) -> ExecError {
    match error {
        TakeError::Driver(crate::driver::DriverError::Cancelled) => ExecError::Cancelled,
        other => ExecError::new(other.to_string()),
    }
}

async fn record(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: RecordTerminal = ctx.params()?;
    let Some(CompileValue::TerminalSession(session)) = inputs.get(TERMINAL_PORT).cloned() else {
        return Err(ExecError::new("Connect an Open terminal and its actions."));
    };
    let job = Arc::clone(&ctx);
    let take = tokio::task::spawn_blocking(move || {
        let cancelled = || job.cancel.is_cancelled();
        let mut log = |line: String| job.log(LogLevel::Warn, line);
        let take = record_take(&mut TakeOptions {
            session: &session,
            project_dir: &job.project_dir,
            temp: &job.env.temp_dir,
            name: &p.name,
            pause_before_ms: p.pause_before_ms,
            keep_after_ms: p.keep_after_ms,
            cancelled: &cancelled,
            log: &mut log,
        })?;
        save_take(&job.dir, &take)?;
        Ok::<_, TakeError>((take, session))
    })
    .await
    .map_err(|e| ExecError::new(e.to_string()))?;
    let (take, session) = take.map_err(|e| failed(&e))?;
    ctx.log(
        LogLevel::Info,
        format!(
            "Recorded {} s in a {} × {} terminal",
            millis(take.length) / 1000.0,
            session.cols,
            session.rows
        ),
    );
    let recording = RecordingValue {
        duration_ms: millis(take.length),
        log: Some(take.log),
        node_id: ctx.node.id.clone(),
        edits: Vec::new(),
        source: TakeSource::Terminal(TerminalSource {
            cast: Some(ctx.project_file_ref(CAST_FILE)),
            cols: session.cols,
            rows: session.rows,
            theme: None,
        }),
    };
    Ok(ExecutorOutput {
        outputs: PortValues::from([(
            RECORDING_PORT.to_owned(),
            CompileValue::Recording(recording),
        )]),
        artifacts: vec![
            ctx.artifact(ArtifactKind::File, CAST_FILE, "Terminal take"),
            ctx.artifact(ArtifactKind::Json, LOG_FILE, "Action log"),
        ],
        trace_id: None,
        measured: Vec::new(),
    })
}

pub fn terminal_executors() -> Vec<(&'static str, Executor)> {
    vec![(RECORD_TERMINAL_TYPE, executor(record))]
}
