use opsis_compile::asset_ref::project_ref;
use opsis_compile::footage::{FootageUse, footage_uses};
use opsis_compile::nodes::effects::RedactTake;
use opsis_engine::paths::EngineMissing;
use opsis_footage::chain::{FootageSource, TimedRedaction};
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_media::grade::{GradeSize, grade_filters};
use opsis_runner::context::{ExecError, ExecutorContext};
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::StyleEntry;
use serde_json::Value;

const US_PER_MS: i64 = 1000;

fn redaction(entry: &StyleEntry) -> Result<TimedRedaction, ExecError> {
    let p: RedactTake = serde_json::from_value(Value::Object(entry.params.clone()))
        .map_err(|e| ExecError::new(format!("Couldn't read a Redact take: {e}")))?;
    Ok(TimedRedaction {
        from_us: i64::from(p.from_ms.unwrap_or(0)) * US_PER_MS,
        until_us: p.until_ms.map(|ms| i64::from(ms) * US_PER_MS),
        redact: p.picture(),
    })
}

fn source_of(ctx: &ExecutorContext, used: &FootageUse) -> Result<FootageSource, ExecError> {
    let file = ctx.ref_to_path(&used.src)?;
    let probe = opsis_probe::probe(&file)
        .map_err(|e| ExecError::new(format!("Couldn't read {}: {e}", file.display())))?;
    let video = probe
        .video
        .as_ref()
        .ok_or_else(|| ExecError::new(format!("{} has no picture.", file.display())))?;
    let resolve = |name: &str| {
        ctx.ref_to_path(&project_ref(name))
            .map(|p| p.to_string_lossy().into_owned())
    };
    let grade = grade_filters::<ExecError>(
        &used.chain.grade,
        GradeSize {
            width: video.width,
            height: video.height,
            still: false,
        },
        &resolve,
    )?;
    let redactions = used
        .chain
        .redact
        .iter()
        .map(redaction)
        .collect::<Result<Vec<_>, _>>()?;
    let trim_us = i64::from(used.trim_ms) * US_PER_MS;
    let loop_us = if used.looping {
        probe
            .duration_ms
            .map(|ms| (i64::from(ms) * US_PER_MS - trim_us).max(1))
    } else {
        None
    };
    Ok(FootageSource {
        file,
        grade,
        redactions,
        trim_us,
        loop_us,
    })
}

pub fn film_footage(ctx: &ExecutorContext, spec: &TimelineSpec) -> Result<Footage, ExecError> {
    let ffmpeg = ctx
        .env
        .engine
        .ffmpeg
        .clone()
        .ok_or_else(|| ExecError::new(EngineMissing.to_string()))?;
    let sources = footage_uses(&spec.tracks)
        .iter()
        .map(|used| Ok((used.id.clone(), source_of(ctx, used)?)))
        .collect::<Result<Vec<_>, ExecError>>()?;
    Ok(Footage::new(
        FootageTools {
            ffmpeg,
            temp: ctx.env.temp_dir.clone(),
        },
        sources,
    ))
}
