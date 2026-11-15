use std::collections::HashMap;
use std::time::Duration;

use opsis_compile::asset_ref::project_ref;
use opsis_compile::footage::{FootageUse, footage_uses, item_footage};
use opsis_compile::nodes::effects::RedactTake;
use opsis_compile::nodes::image::RedactStyle;
use opsis_engine::ffmpeg::FfmpegMissing;
use opsis_footage::chain::{FootageSource, TimedRedaction};
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_footage::terminal::TerminalFootage;
use opsis_media::grade::{GradeSize, colour_steps, grade_filters};
use opsis_runner::context::{ExecError, ExecutorContext};
use opsis_terminal::colours::parse_hex;
use opsis_terminal::prepare::{Mask, MaskArea, MaskLook};
use opsis_types::terminal::TerminalTheme;
use opsis_types::timeline::{TimelineItem, TimelineSpec, TimelineTrack};
use opsis_types::values::StyleEntry;
use regex::Regex;
use serde_json::Value;

use crate::seekable::seekable;

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

fn speed_of(item: &TimelineItem) -> f64 {
    match item {
        TimelineItem::Recording { props, .. } => props.speed,
        TimelineItem::Element { props, .. } => props
            .get("playbackRate")
            .and_then(Value::as_f64)
            .unwrap_or(1.0),
        TimelineItem::Group { .. } => 1.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseRate {
    pub speed: f64,
    pub places: u32,
}

pub fn remap_rate(remap: &[(f64, f64)]) -> f64 {
    remap
        .windows(2)
        .filter_map(|w| {
            let outer = w[1].0 - w[0].0;
            (outer > 0.0).then(|| ((w[1].1 - w[0].1) / outer).abs())
        })
        .fold(1.0, f64::max)
}

fn gather_rates(
    tracks: &[TimelineTrack],
    rate: f64,
    loops: u32,
    into: &mut HashMap<String, UseRate>,
) {
    for item in tracks.iter().flat_map(|t| &t.items) {
        let speed = speed_of(item).abs() * rate;
        for footage in item_footage(item) {
            let places = 1 + loops + u32::from(footage.used.looping);
            let known = into
                .entry(footage.used.id)
                .or_insert(UseRate { speed, places: 0 });
            known.speed = known.speed.max(speed);
            known.places += places;
        }
        if let TimelineItem::Group { props, .. } = item {
            let inner = rate * props.remap.as_deref().map_or(1.0, remap_rate);
            let looped = loops + u32::from(props.loop_every.is_some_and(|n| n > 0));
            gather_rates(&props.tracks, inner, looped, into);
        }
    }
}

pub fn use_rates(tracks: &[TimelineTrack]) -> HashMap<String, UseRate> {
    let mut rates = HashMap::new();
    gather_rates(tracks, 1.0, 0, &mut rates);
    rates
}

const fn even(size: u32) -> u32 {
    size - size % 2
}

async fn source_of(
    ctx: &ExecutorContext,
    used: &FootageUse,
    rate: UseRate,
) -> Result<FootageSource, ExecError> {
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
    let decoded = seekable(ctx, &file, &probe).await?;
    let frames = opsis_probe::video_frames(&decoded).unwrap_or_else(|e| {
        log::warn!("reading the frame times of {}: {e}", decoded.display());
        None
    });
    let (width, height) = frames
        .as_ref()
        .map_or((video.width, video.height), |f| (f.width, f.height));
    Ok(FootageSource {
        file: decoded,
        grade,
        redactions,
        trim_us,
        loop_us,
        frame_times: frames.map(|f| f.times_us.into()),
        size: Some((even(width), even(height))),
        speed: rate.speed,
        places: rate.places.max(1),
    })
}

fn mask_of(entry: &StyleEntry) -> Result<Mask, ExecError> {
    let p: RedactTake = serde_json::from_value(Value::Object(entry.params.clone()))
        .map_err(|e| ExecError::new(format!("Couldn't read a Redact take: {e}")))?;
    let patterns = p
        .text_to_hide
        .iter()
        .filter(|t| !t.trim().is_empty())
        .map(|t| Regex::new(t).map_err(|e| ExecError::new(format!("“{t}” in Text to hide: {e}"))))
        .collect::<Result<Vec<_>, _>>()?;
    let look = match p.style {
        RedactStyle::Solid => MaskLook::Solid(parse_hex(&p.color).unwrap_or(0)),
        RedactStyle::Blur => MaskLook::Blur,
        RedactStyle::Pixelate => MaskLook::Pixels,
    };
    Ok(Mask {
        areas: p
            .areas
            .iter()
            .map(|a| MaskArea {
                x: a.x,
                y: a.y,
                width: a.width,
                height: a.height,
            })
            .collect(),
        patterns,
        from: Duration::from_millis(u64::from(p.from_ms.unwrap_or(0))),
        until: p.until_ms.map(|ms| Duration::from_millis(u64::from(ms))),
        look,
    })
}

fn terminal_of(
    ctx: &ExecutorContext,
    used: &FootageUse,
    theme: TerminalTheme,
    fps: u32,
) -> Result<TerminalFootage, ExecError> {
    let steps = colour_steps(&used.chain.grade);
    let resolve = |name: &str| {
        ctx.ref_to_path(&project_ref(name))
            .map(|p| p.to_string_lossy().into_owned())
    };
    let grade = if steps.is_empty() {
        None
    } else {
        Some(grade_filters::<ExecError>(
            &steps,
            GradeSize {
                width: 1,
                height: 1,
                still: true,
            },
            &resolve,
        )?)
    };
    Ok(TerminalFootage {
        cast: ctx.ref_to_path(&used.src)?,
        theme,
        grade,
        masks: used
            .chain
            .redact
            .iter()
            .map(mask_of)
            .collect::<Result<Vec<_>, _>>()?,
        fps,
    })
}

pub async fn film_footage(
    ctx: &ExecutorContext,
    spec: &TimelineSpec,
) -> Result<Footage, ExecError> {
    let ffmpeg = ctx
        .env
        .engine
        .ffmpeg
        .clone()
        .ok_or_else(|| ExecError::new(FfmpegMissing.to_string()))?;
    let uses = footage_uses(&spec.tracks);
    let rates = use_rates(&spec.tracks);
    let mut sources = Vec::new();
    for used in uses.iter().filter(|used| used.terminal.is_none()) {
        let rate = rates.get(&used.id).copied().unwrap_or(UseRate {
            speed: 1.0,
            places: 1,
        });
        sources.push((used.id.clone(), source_of(ctx, used, rate).await?));
    }
    let terminals = uses
        .iter()
        .filter_map(|used| {
            let theme = used.terminal.clone()?;
            Some(terminal_of(ctx, used, theme, spec.fps).map(|t| (used.id.clone(), t)))
        })
        .collect::<Result<Vec<_>, ExecError>>()?;
    Ok(Footage::new(
        FootageTools {
            ffmpeg,
            temp: ctx.env.temp_dir.clone(),
        },
        sources,
    )
    .with_terminals(terminals))
}
