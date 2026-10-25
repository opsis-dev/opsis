use opsis_compile::mix::levels::{apply_mix_levels, with_master_stem};
use opsis_compile::mix::plan::build_mix_plan;
use opsis_compile::text::number;
use opsis_compile::time::lerp;
use opsis_compile::timeline::flatten::flatten_spec;
use opsis_media::ffmpeg::MediaTools;
use opsis_media::mix::render::{MixDeps, MixError, render_mix};
use opsis_runner::context::{ExecError, ExecutorContext};
use opsis_types::events::LogLevel;
use opsis_types::timeline::TimelineSpec;

pub type Stage = (f64, f64);

pub fn spec_for(ctx: &ExecutorContext) -> Result<TimelineSpec, ExecError> {
    let result = ctx.compile();
    if let Some(error) = result.errors.get(&ctx.node.id) {
        return Err(ExecError::new(error.clone()));
    }
    result
        .specs
        .get(&ctx.node.id)
        .cloned()
        .ok_or_else(|| ExecError::new("Connect a clip to this node."))
}

fn mix_error(error: MixError) -> ExecError {
    match error {
        MixError::Cancelled => ExecError::Cancelled,
        other => ExecError::new(other.to_string()),
    }
}

pub async fn mixed(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    spec: TimelineSpec,
    stage: Stage,
) -> Result<TimelineSpec, ExecError> {
    let flat = flatten_spec(&spec);
    let mix = ctx.workflow.mix.as_ref();
    let plan = build_mix_plan(&spec, mix, &flat);
    let resolve = |reference: &str| ctx.ref_to_path(reference).map_err(|_| MixError::Failed);
    let progress = |p: f64, message: &str| ctx.progress(lerp(stage.0, stage.1, p), Some(message));
    let deps = MixDeps {
        project_dir: &ctx.project_dir,
        resolve: &resolve,
        ffmpeg: tools,
        cancel: Some(&ctx.cancel),
        on_progress: Some(&progress),
    };
    let result = render_mix(&plan, &deps).await.map_err(mix_error)?;
    let Some(master) = result.master else {
        return Ok(apply_mix_levels(&spec, mix, &flat));
    };
    if let Some(lufs) = result.loudness.as_ref().and_then(|l| l.integrated) {
        let peak = result
            .loudness
            .as_ref()
            .and_then(|l| l.true_peak)
            .map(|p| format!(", peak {} dBTP", number(p)))
            .unwrap_or_default();
        ctx.log(
            LogLevel::Info,
            format!("Sound: {} LUFS{peak}", number(lufs)),
        );
    }
    Ok(with_master_stem(&spec, &master.src, &master.hash))
}
