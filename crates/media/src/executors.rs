use opsis_compile::nodes::media::{
    APPLY_GRADE_TYPE, CONCAT_TYPE, CONVERT_TYPE, EXTRACT_AUDIO_TYPE, FASTSTART_TYPE, GIF_TYPE,
    IMPORT_TYPE, LOUDNORM_TYPE, PROBE_TYPE, REVERSE_TYPE, SCALE_PAD_TYPE, SPEED_TYPE,
    THUMBNAIL_TYPE, TRIM_TYPE,
};
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use futures::future::try_join_all;
use opsis_compile::asset_ref::{kind_from_path, project_ref};
use opsis_compile::nodes::grade::{LOOK_TYPE, LookFile, grade_needs_video};
use opsis_compile::nodes::media::{
    CONCAT_PORTS, ChangeSpeed, Encoding, EvenOutVolume, ExtractSound, GrabFrame, JoinFiles,
    MakeGif, MediaFile, Resize, TrimFile,
};
use opsis_compile::registry::{node_def, node_name};
use opsis_compile::text::{format_duration, format_precise_duration, single_quoted};
use opsis_runner::context::{ExecError, ExecutorContext, ExecutorFn, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::values::{
    Artifact, ArtifactKind, CompileValue, ImageValue, MediaKind, MediaValue, PortValues, SrcValue,
    StyleEntry,
};
use opsis_types::workflow::DEFAULT_STILL_MS;

use crate::codecs::{
    EVEN_FRAME, EditedFile, FASTSTART, FrameGrab, PCM_16, audio_format_args, audio_format_for,
    codec_info, convert_args, convert_codec, edit_audio, edit_video, edited_file, even_down,
    ffmpeg_seconds, frame_grab_args, grade_sound_args, grade_video_args, serde_name, sound_file,
    strings,
};
use crate::ffmpeg::{MediaTools, OnProgress, ProbeResult, RunOptions, ToolError, can_stream_copy};
use crate::filters::{
    JoinPart, JoinSize, fit_filter, gif_filter, join_filter, loudnorm_filter, speed_args,
    speed_sound_args,
};
use crate::grade::{GradeError, GradeSize, grade_filters};
use crate::reverse::{
    PieceBudget, REVERSE_BUDGET_BYTES, ReversePiece, ReversedPart, WHOLE_FILE, frame_bytes,
    join_reversed_args, piece_length_ms, reverse_args, reverse_pieces, reverse_sound_args,
    reversed_parts_list,
};

const CONNECT_MEDIA: &str = "Connect a media input (and run the node that produces it).";
const MEDIA_PORT: &str = "media";
const MP4_PICTURES: [&str; 5] = ["h264", "hevc", "av1", "vp9", "mpeg4"];
const LAST_FRAME_MARGIN_MS: f64 = 100.0;
const PARTS_SHARE: f64 = 0.9;

pub type Tools = Arc<dyn MediaTools>;

impl From<GradeError> for ExecError {
    fn from(error: GradeError) -> Self {
        ExecError::new(error.to_string())
    }
}

pub fn tool_error(error: ToolError) -> ExecError {
    match error {
        ToolError::Cancelled => ExecError::Cancelled,
        ToolError::Failed { output } => ExecError::Tool {
            message: "The video engine couldn't process a file.".into(),
            output,
        },
        missing @ ToolError::Missing(_) => ExecError::new(missing.to_string()),
    }
}

fn run_options<'a>(
    ctx: &'a ExecutorContext,
    duration_ms: Option<f64>,
    progress: Option<OnProgress<'a>>,
) -> RunOptions<'a> {
    RunOptions {
        cancel: Some(&ctx.cancel),
        duration_ms,
        on_progress: progress,
        on_log: None,
    }
}

pub struct LinkedMedia {
    pub value: MediaValue,
    pub src: String,
}

pub fn require_media(value: Option<&CompileValue>) -> Result<LinkedMedia, ExecError> {
    let media = value
        .and_then(CompileValue::as_media)
        .ok_or_else(|| ExecError::new(CONNECT_MEDIA))?;
    let src = media
        .src
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ExecError::new(CONNECT_MEDIA))?;
    Ok(LinkedMedia {
        value: media.clone(),
        src,
    })
}

pub async fn media_has_sound(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    media: &LinkedMedia,
) -> Result<bool, ExecError> {
    if let Some(has) = media.value.has_audio {
        return Ok(has);
    }
    Ok(tools
        .probe(&ctx.ref_to_path(&media.src)?)
        .await
        .map_err(tool_error)?
        .audio
        .is_some())
}

fn ext_of(src: &str) -> String {
    Path::new(src).extension().map_or_else(
        || ".mp4".to_owned(),
        |e| format!(".{}", e.to_string_lossy().to_lowercase()),
    )
}

fn duration_ms(probe: &ProbeResult) -> f64 {
    probe.duration_ms.map_or(0.0, f64::from)
}

fn media_kind(probe: &ProbeResult, file: &str) -> MediaKind {
    match kind_from_path(file) {
        MediaKind::Video if probe.video.is_none() => MediaKind::Audio,
        named => named,
    }
}

fn media_value_from(probe: &ProbeResult, src: &str, kind: MediaKind) -> MediaValue {
    MediaValue {
        src: Some(src.to_owned()),
        kind,
        duration_ms: probe.duration_ms,
        width: probe.video.as_ref().map(|v| v.width),
        height: probe.video.as_ref().map(|v| v.height),
        has_audio: Some(probe.audio.is_some()),
    }
}

fn artifact_kind(kind: MediaKind) -> ArtifactKind {
    match kind {
        MediaKind::Video => ArtifactKind::Video,
        MediaKind::Image => ArtifactKind::Image,
        MediaKind::Audio => ArtifactKind::Audio,
    }
}

fn output(port: &str, value: CompileValue) -> PortValues {
    [(port.to_owned(), value)].into_iter().collect()
}

fn artifact(ctx: &ExecutorContext, kind: ArtifactKind, file: &str, label: &str) -> Artifact {
    Artifact {
        kind,
        path: ctx.rel(file),
        label: Some(label.to_owned()),
        bytes: None,
        stamp: None,
    }
}

async fn media_result(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    file: &str,
    label: &str,
) -> Result<ExecutorOutput, ExecError> {
    let probe = tools.probe(&ctx.dir.join(file)).await.map_err(tool_error)?;
    let src = ctx.project_file_ref(file);
    let media = media_value_from(&probe, &src, media_kind(&probe, &src));
    let artifacts = vec![artifact(ctx, artifact_kind(media.kind), file, label)];
    Ok(ExecutorOutput {
        outputs: output(MEDIA_PORT, CompileValue::Media(media)),
        artifacts,
        trace_id: None,
    })
}

struct Transcode<'a> {
    file: String,
    label: &'a str,
    duration_ms: Option<f64>,
}

async fn transcode(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    input: &LinkedMedia,
    job: Transcode<'_>,
    args: impl FnOnce(&str, &str) -> Vec<String>,
) -> Result<ExecutorOutput, ExecError> {
    let source = ctx.ref_to_path(&input.src)?;
    let args = args(
        &source.to_string_lossy(),
        &ctx.dir.join(&job.file).to_string_lossy(),
    );
    let progress = |p: f64| ctx.progress(p, None);
    tools
        .run(
            &args,
            run_options(
                ctx,
                job.duration_ms.or(input.value.duration_ms.map(f64::from)),
                Some(&progress),
            ),
        )
        .await
        .map_err(tool_error)?;
    media_result(ctx, tools, &job.file, job.label).await
}

pub fn thumbnail_time(at_ms: f64, duration_ms: f64) -> f64 {
    if duration_ms <= LAST_FRAME_MARGIN_MS {
        0.0
    } else {
        at_ms.min(duration_ms - LAST_FRAME_MARGIN_MS)
    }
}

fn sound_only_file(probe: &ProbeResult, src: &str, node: &str) -> Result<EditedFile, ExecError> {
    if probe.audio.is_none() {
        return Err(ExecError::new(format!(
            "{node} works on video and sound, and this file has neither."
        )));
    }
    Ok(sound_file(&ext_of(src)))
}

fn concat_list(paths: &[String]) -> String {
    paths
        .iter()
        .map(|f| format!("file {}", single_quoted(f)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn probe_summary(probe: &ProbeResult) -> String {
    let picture = probe.video.as_ref().map_or_else(
        || "no video".to_owned(),
        |v| {
            let rate = v
                .fps
                .map_or_else(|| "unknown".to_owned(), |f| format!("{f:.2}"));
            format!("{} {}×{} @ {rate}fps", v.codec, v.width, v.height)
        },
    );
    let sound = probe.audio.as_ref().map_or_else(
        || "no audio".to_owned(),
        |a| match a.sample_rate {
            Some(rate) => format!("{} {rate}Hz", a.codec),
            None => a.codec.clone(),
        },
    );
    format!(
        "{} · {} · {picture} · {sound}",
        probe.format,
        probe.duration_ms.map_or_else(
            || "unknown length".to_owned(),
            |ms| format_precise_duration(f64::from(ms))
        )
    )
}

async fn import_media(
    ctx: Arc<ExecutorContext>,
    _: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: MediaFile = ctx.params()?;
    if p.path.is_empty() {
        return Err(ExecError::new("Choose a file."));
    }
    let probe = tools
        .probe(&ctx.ref_to_path(&p.path)?)
        .await
        .map_err(tool_error)?;
    let src = project_ref(&p.path);
    let mut media = media_value_from(&probe, &src, media_kind(&probe, &src));
    if media.kind == MediaKind::Image {
        media.duration_ms = None;
    }
    Ok(ExecutorOutput {
        outputs: output(MEDIA_PORT, CompileValue::Media(media)),
        ..ExecutorOutput::default()
    })
}

async fn trim(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: TrimFile = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let moving = probe.video.is_some() && probe.duration_ms.is_some();
    let out = edited_file(&ext_of(&input.src), moving, probe.audio.is_some()).ok_or_else(|| {
        ExecError::new("Trim works on video and sound. A still image has nothing to trim.")
    })?;
    let (start, end) = (f64::from(p.start_ms), f64::from(p.end_ms));
    let duration_ms = if p.end_ms != 0 {
        Some(end - start)
    } else {
        input.value.duration_ms.map(f64::from)
    };
    let job = Transcode {
        file: format!("trimmed{}", out.ext),
        label: "Trimmed",
        duration_ms,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = vec!["-ss".to_owned(), ffmpeg_seconds(start)];
        if p.end_ms != 0 {
            args.extend(["-to".to_owned(), ffmpeg_seconds(end)]);
        }
        args.extend(["-i".to_owned(), i.to_owned()]);
        args.extend(out.codecs);
        args.push(o.to_owned());
        args
    })
    .await
}

fn concat_parts(items: &[LinkedMedia], probes: &[ProbeResult]) -> Vec<(bool, JoinPart)> {
    items
        .iter()
        .zip(probes)
        .map(|(m, probe)| {
            let still = m.value.kind == MediaKind::Image;
            let duration_ms = if still {
                m.value.duration_ms.map_or(DEFAULT_STILL_MS, f64::from)
            } else {
                probe.duration_ms.map_or(0.0, f64::from)
            };
            (
                still,
                JoinPart {
                    video: probe.video.is_some(),
                    audio: probe.audio.is_some(),
                    duration_ms,
                },
            )
        })
        .collect()
}

fn reencode_args(
    ctx: &ExecutorContext,
    paths: &[String],
    probes: &[ProbeResult],
    parts: &[(bool, JoinPart)],
    out: String,
) -> Vec<String> {
    let first = probes.iter().find_map(|x| x.video.as_ref());
    let settings = &ctx.workflow.settings;
    let to = JoinSize {
        width: first.map_or(settings.width, |v| v.width),
        height: first.map_or(settings.height, |v| v.height),
        fps: first.and_then(|v| v.fps).unwrap_or(f64::from(settings.fps)),
    };
    let join = join_filter(&parts.iter().map(|(_, p)| *p).collect::<Vec<_>>(), to);
    let mut args = Vec::new();
    for (path, (still, part)) in paths.iter().zip(parts) {
        if *still {
            args.extend(strings(&["-loop", "1", "-t"]));
            args.push(ffmpeg_seconds(part.duration_ms));
        }
        args.extend(["-i".to_owned(), path.clone()]);
    }
    args.extend(["-filter_complex".to_owned(), join.filter]);
    if join.video {
        args.extend(strings(&["-map", "[v]"]));
        args.extend(edit_video());
    }
    if join.audio {
        args.extend(strings(&["-map", "[a]"]));
        args.extend(edit_audio());
    }
    args.extend(strings(&FASTSTART));
    args.push(out);
    args
}

async fn concat(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: JoinFiles = ctx.params()?;
    let items: Vec<LinkedMedia> = CONCAT_PORTS
        .iter()
        .filter_map(|port| inputs.get(*port))
        .map(|v| require_media(Some(v)))
        .collect::<Result<_, _>>()?;
    if items.len() < 2 {
        return Err(ExecError::new("Connect at least two files."));
    }
    let paths: Vec<String> = items
        .iter()
        .map(|m| {
            ctx.ref_to_path(&m.src)
                .map(|p| p.to_string_lossy().into_owned())
        })
        .collect::<Result<_, _>>()?;
    let probes = try_join_all(paths.iter().map(|f| tools.probe(Path::new(f))))
        .await
        .map_err(tool_error)?;
    let parts = concat_parts(&items, &probes);
    let total: f64 = parts.iter().map(|(_, p)| p.duration_ms).sum();
    let any_video = parts.iter().any(|(_, p)| p.video);
    let file = if any_video {
        "concat.mp4"
    } else {
        "concat.m4a"
    };
    let out = ctx.dir.join(file).to_string_lossy().into_owned();
    let progress = |v: f64| ctx.progress(v, None);
    let args = if !p.force_reencode
        && any_video
        && probes.iter().all(|x| can_stream_copy(&probes[0], x))
    {
        let list = ctx.dir.join("concat.txt");
        tokio::fs::write(&list, concat_list(&paths)).await?;
        ctx.log(
            LogLevel::Info,
            "The files match, joining them without converting.",
        );
        let mut args = strings(&[
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &list.to_string_lossy(),
            "-c",
            "copy",
        ]);
        args.extend(strings(&FASTSTART));
        args.push(out);
        args
    } else {
        ctx.log(
            LogLevel::Info,
            "The files differ, converting them to match the first one.",
        );
        reencode_args(&ctx, &paths, &probes, &parts, out)
    };
    tools
        .run(&args, run_options(&ctx, Some(total), Some(&progress)))
        .await
        .map_err(tool_error)?;
    media_result(&ctx, &*tools, file, "Concatenated").await
}

async fn speed(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: ChangeSpeed = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let label = format!("Speed {}×", p.rate);
    let duration_ms = input.value.duration_ms.map(|d| f64::from(d) / p.rate);
    let Some(video) = &probe.video else {
        let out = sound_only_file(&probe, &input.src, "Change speed")?;
        if !p.keep_audio {
            return Err(ExecError::new(
                "This file is only sound, so turning off Keep the sound leaves nothing.",
            ));
        }
        let job = Transcode {
            file: format!("speed{}", out.ext),
            label: &label,
            duration_ms,
        };
        return transcode(&ctx, &*tools, &input, job, |i, o| {
            speed_sound_args(i, o, p.rate, &out.codecs)
        })
        .await;
    };
    let fps = video.fps.unwrap_or(f64::from(ctx.workflow.settings.fps));
    let sound = p.keep_audio && media_has_sound(&ctx, &*tools, &input).await?;
    let job = Transcode {
        file: "speed.mp4".into(),
        label: &label,
        duration_ms,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        speed_args(i, o, p.rate, fps, sound)
    })
    .await
}

struct ReverseParts<'a> {
    pieces: &'a [ReversePiece],
    sound: bool,
    duration_ms: f64,
}

async fn reverse_pieces_into(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    src: &str,
    out: &str,
    dir: &Path,
    job: &ReverseParts<'_>,
) -> Result<(), ExecError> {
    let parts: Vec<ReversedPart> = job
        .pieces
        .iter()
        .enumerate()
        .map(|(n, piece)| ReversedPart {
            file: dir
                .join(format!("part-{n}.mkv"))
                .to_string_lossy()
                .into_owned(),
            length_ms: piece_length_ms(piece, job.duration_ms),
        })
        .collect();
    let pcm = strings(&PCM_16);
    let mut done = 0.0;
    for (n, piece) in job.pieces.iter().enumerate() {
        let share = PARTS_SHARE * parts[n].length_ms / job.duration_ms;
        let message = format!("Reversing part {} of {}", n + 1, job.pieces.len());
        let start = done;
        ctx.progress(start, Some(&message));
        let progress = |p: f64| ctx.progress(start + p * share, Some(&message));
        let args = reverse_args(src, &parts[n].file, piece, job.sound, &pcm);
        tools
            .run(
                &args,
                run_options(ctx, Some(parts[n].length_ms), Some(&progress)),
            )
            .await
            .map_err(tool_error)?;
        done += share;
    }
    let list = dir.join("parts.txt");
    tokio::fs::write(&list, reversed_parts_list(&parts)).await?;
    let progress = |p: f64| {
        ctx.progress(
            PARTS_SHARE + p * (1.0 - PARTS_SHARE),
            Some("Joining the parts"),
        )
    };
    let args = join_reversed_args(&list.to_string_lossy(), out, job.sound);
    tools
        .run(
            &args,
            run_options(ctx, Some(job.duration_ms), Some(&progress)),
        )
        .await
        .map_err(tool_error)
}

async fn remove_dir(dir: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_dir_all(dir).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        done => done,
    }
}

async fn reverse_in_parts(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    src: &str,
    out: &str,
    job: ReverseParts<'_>,
) -> Result<(), ExecError> {
    let dir = ctx.dir.join("reverse-parts");
    remove_dir(&dir).await?;
    tokio::fs::create_dir_all(&dir).await?;
    let reversed = reverse_pieces_into(ctx, tools, src, out, &dir, &job).await;
    remove_dir(&dir).await?;
    reversed
}

async fn reverse(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let src = ctx.ref_to_path(&input.src)?;
    let probe = tools.probe(&src).await.map_err(tool_error)?;
    let label = "Reversed";
    let Some(video) = &probe.video else {
        let out = sound_only_file(&probe, &input.src, "Reverse")?;
        let job = Transcode {
            file: format!("reversed{}", out.ext),
            label,
            duration_ms: None,
        };
        return transcode(&ctx, &*tools, &input, job, |i, o| {
            reverse_sound_args(i, o, &out.codecs)
        })
        .await;
    };
    let sound = media_has_sound(&ctx, &*tools, &input).await?;
    let pieces = reverse_pieces(PieceBudget {
        duration_ms: duration_ms(&probe),
        fps: video.fps.unwrap_or(f64::from(ctx.workflow.settings.fps)),
        frame_bytes: frame_bytes(
            f64::from(video.width),
            f64::from(video.height),
            &video.pix_fmt,
        ),
        budget_bytes: REVERSE_BUDGET_BYTES,
    });
    let file = "reversed.mp4";
    if pieces.len() == 1 {
        let job = Transcode {
            file: file.into(),
            label,
            duration_ms: None,
        };
        return transcode(&ctx, &*tools, &input, job, |i, o| {
            reverse_args(i, o, &WHOLE_FILE, sound, &[])
        })
        .await;
    }
    ctx.log(
        LogLevel::Info,
        format!("Reversing in {} parts to keep memory low.", pieces.len()),
    );
    let out = ctx.dir.join(file).to_string_lossy().into_owned();
    reverse_in_parts(
        &ctx,
        &*tools,
        &src.to_string_lossy(),
        &out,
        ReverseParts {
            pieces: &pieces,
            sound,
            duration_ms: duration_ms(&probe),
        },
    )
    .await?;
    media_result(&ctx, &*tools, file, label).await
}

async fn scale_pad(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: Resize = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    if probe.video.is_none() {
        return Err(ExecError::new(
            "Resize works on video and images, and this file is only sound.",
        ));
    }
    let image = input.value.kind == MediaKind::Image;
    let (w, h) = (f64::from(p.width), f64::from(p.height));
    let filter = if image {
        fit_filter(p.mode, w, h, &p.background)
    } else {
        fit_filter(p.mode, even_down(w), even_down(h), &p.background)
    };
    let label = format!("{}×{}", p.width, p.height);
    let job = Transcode {
        file: if image { "scaled.png" } else { "scaled.mp4" }.into(),
        label: &label,
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = vec!["-i".to_owned(), i.to_owned(), "-vf".to_owned(), filter];
        if !image {
            args.extend(edit_video());
            args.extend(strings(&["-c:a", "copy"]));
        }
        args.push(o.to_owned());
        args
    })
    .await
}

async fn convert(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: Encoding = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let label = serde_name(&p.codec);
    let job = Transcode {
        file: format!("converted.{}", codec_info(convert_codec(p.codec)).ext),
        label: &label,
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = vec!["-i".to_owned(), i.to_owned()];
        args.extend(convert_args(p.codec, f64::from(p.crf), p.preset));
        args.push(o.to_owned());
        args
    })
    .await
}

fn check_look_files(ctx: &ExecutorContext, entries: &[StyleEntry]) -> Result<(), ExecError> {
    let look_def = node_def(LOOK_TYPE).ok_or_else(|| GradeError::NotAStep(LOOK_TYPE.to_owned()))?;
    for entry in entries.iter().filter(|e| e.kind == LOOK_TYPE) {
        let look: LookFile = look_def.typed_params(&entry.params)?;
        if look.strength == 0.0 {
            continue;
        }
        let name = ctx
            .workflow
            .nodes
            .iter()
            .find(|n| n.id == entry.node_id)
            .map_or_else(|| look_def.title.clone(), node_name);
        if look.file.is_empty() {
            return Err(ExecError::new(format!("Choose a look file for “{name}”.")));
        }
        if !ctx.ref_to_path(&project_ref(&look.file))?.exists() {
            return Err(ExecError::new(format!(
                "The look file for “{name}” isn't in the project."
            )));
        }
    }
    Ok(())
}

async fn apply_grade(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: Encoding = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let entries = inputs
        .get("grade")
        .and_then(CompileValue::as_grade)
        .map(|g| g.entries.clone())
        .ok_or_else(|| ExecError::new("Connect a grade."))?;
    check_look_files(&ctx, &entries)?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let video = probe
        .video
        .as_ref()
        .ok_or_else(|| ExecError::new("This file has no picture to grade."))?;
    let still = input.value.kind == MediaKind::Image;
    let resolve = |file: &str| {
        ctx.ref_to_path(&project_ref(file))
            .map(|p| p.to_string_lossy().into_owned())
    };
    let chain = grade_filters::<ExecError>(
        &entries,
        GradeSize {
            width: video.width,
            height: video.height,
            still,
        },
        &resolve,
    )?;
    if still && grade_needs_video(&entries) {
        ctx.log(
            LogLevel::Warn,
            "Motion trails and film grain are left out of a still image.",
        );
    }
    let codec = codec_info(convert_codec(p.codec));
    let file = if still {
        "graded.png".to_owned()
    } else {
        format!("graded.{}", codec.ext)
    };
    let sound = probe.audio.as_ref().map(|a| a.codec.as_str());
    let job = Transcode {
        file,
        label: "Graded file",
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = vec!["-i".to_owned(), i.to_owned()];
        for extra in chain.extra_inputs {
            args.extend(["-i".to_owned(), extra]);
        }
        args.extend([
            "-filter_complex".to_owned(),
            chain.chain,
            "-map".to_owned(),
            "[out]".to_owned(),
        ]);
        if still {
            args.extend(strings(&["-frames:v", "1"]));
        } else {
            if sound.is_some() {
                args.extend(strings(&["-map", "0:a?"]));
            }
            args.extend(grade_video_args(p.codec, f64::from(p.crf), p.preset));
            args.extend(grade_sound_args(p.codec, sound));
            if codec.faststart {
                args.extend(strings(&FASTSTART));
            }
        }
        args.push(o.to_owned());
        args
    })
    .await
}

async fn gif(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: MakeGif = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let filter = gif_filter(f64::from(p.fps), f64::from(p.width), p.dither);
    let looped = if p.r#loop { "0" } else { "-1" };
    let job = Transcode {
        file: "animation.gif".into(),
        label: "GIF",
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        strings(&["-i", i, "-filter_complex", &filter, "-loop", looped, o])
    })
    .await
}

async fn extract_audio(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: ExtractSound = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    if !media_has_sound(&ctx, &*tools, &input).await? {
        return Err(ExecError::new("This file has no sound to take out."));
    }
    let job = Transcode {
        file: format!("audio.{}", serde_name(&p.format)),
        label: "Audio",
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = strings(&["-i", i, "-vn"]);
        args.extend(audio_format_args(p.format));
        args.push(o.to_owned());
        args
    })
    .await
}

async fn loudnorm(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: EvenOutVolume = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let format = (input.value.kind == MediaKind::Audio)
        .then(|| audio_format_for(ext_of(&input.src).trim_start_matches('.')));
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let audio = probe
        .audio
        .as_ref()
        .ok_or_else(|| ExecError::new("This file has no sound to even out."))?;
    let picture = match &probe.video {
        Some(v) if MP4_PICTURES.contains(&v.codec.as_str()) => strings(&["-c:v", "copy"]),
        _ => {
            let mut args = strings(&["-vf", EVEN_FRAME]);
            args.extend(edit_video());
            args
        }
    };
    let ext = format.map_or_else(|| "mp4".to_owned(), |f| serde_name(&f));
    let job = Transcode {
        file: format!("loudnorm.{ext}"),
        label: "Loudness normalised",
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = vec![
            "-i".to_owned(),
            i.to_owned(),
            "-af".to_owned(),
            loudnorm_filter(p.integrated, p.true_peak, p.lra),
        ];
        if let Some(rate) = audio.sample_rate {
            args.extend(["-ar".to_owned(), rate.to_string()]);
        }
        match format {
            Some(f) => args.extend(audio_format_args(f)),
            None => {
                args.extend(picture);
                args.extend(edit_audio());
            }
        }
        args.push(o.to_owned());
        args
    })
    .await
}

async fn thumbnail(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: GrabFrame = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let src = ctx.ref_to_path(&input.src)?;
    let source = tools.probe(&src).await.map_err(tool_error)?;
    if source.video.is_none() {
        return Err(ExecError::new(
            "This file has no picture to take a thumbnail from.",
        ));
    }
    let at_ms = thumbnail_time(f64::from(p.at_ms), duration_ms(&source));
    if at_ms < f64::from(p.at_ms) && input.value.kind != MediaKind::Image {
        ctx.log(
            LogLevel::Warn,
            format!(
                "The file is only {} long, so the thumbnail is from {}.",
                format_duration(duration_ms(&source)),
                format_duration(at_ms)
            ),
        );
    }
    let file = "thumbnail.png";
    let target = ctx.dir.join(file);
    let args = frame_grab_args(
        &src.to_string_lossy(),
        &target.to_string_lossy(),
        &FrameGrab {
            at_ms,
            width: f64::from(p.width),
            ..FrameGrab::default()
        },
    );
    tools
        .run(&args, run_options(&ctx, None, None))
        .await
        .map_err(tool_error)?;
    let probe = tools.probe(&target).await.map_err(tool_error)?;
    let image = ImageValue {
        src: Some(ctx.project_file_ref(file)),
        width: probe.video.as_ref().map(|v| v.width),
        height: probe.video.as_ref().map(|v| v.height),
        scale: None,
        body: None,
    };
    let artifacts = vec![artifact(&ctx, ArtifactKind::Image, file, "Thumbnail")];
    Ok(ExecutorOutput {
        outputs: output("image", CompileValue::Image(image)),
        artifacts,
        trace_id: None,
    })
}

async fn faststart(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let job = Transcode {
        file: format!("faststart{}", ext_of(&input.src)),
        label: "Web-ready",
        duration_ms: None,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = strings(&["-i", i, "-c", "copy"]);
        args.extend(strings(&FASTSTART));
        args.push(o.to_owned());
        args
    })
    .await
}

async fn probe(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let file = "probe.json";
    opsis_engine::files::write_json(&ctx.dir.join(file), &probe.raw)?;
    ctx.log(LogLevel::Info, probe_summary(&probe));
    let mut outputs = output(
        "info",
        CompileValue::FileInfo(SrcValue {
            src: Some(ctx.project_file_ref(file)),
        }),
    );
    outputs.insert(
        MEDIA_PORT.to_owned(),
        CompileValue::Media(media_value_from(&probe, &input.src, input.value.kind)),
    );
    Ok(ExecutorOutput {
        outputs,
        artifacts: vec![artifact(&ctx, ArtifactKind::Json, file, "Probe")],
        trace_id: None,
    })
}

fn bind<F, Fut>(tools: &Tools, run: F) -> ExecutorFn
where
    F: Fn(Arc<ExecutorContext>, PortValues, Tools) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    let tools = Arc::clone(tools);
    executor(move |ctx, inputs| run(ctx, inputs, Arc::clone(&tools)))
}

pub fn media_executors(tools: &Tools) -> Vec<(&'static str, ExecutorFn)> {
    vec![
        (IMPORT_TYPE, bind(tools, import_media)),
        (TRIM_TYPE, bind(tools, trim)),
        (CONCAT_TYPE, bind(tools, concat)),
        (SPEED_TYPE, bind(tools, speed)),
        (REVERSE_TYPE, bind(tools, reverse)),
        (SCALE_PAD_TYPE, bind(tools, scale_pad)),
        (CONVERT_TYPE, bind(tools, convert)),
        (APPLY_GRADE_TYPE, bind(tools, apply_grade)),
        (GIF_TYPE, bind(tools, gif)),
        (EXTRACT_AUDIO_TYPE, bind(tools, extract_audio)),
        (LOUDNORM_TYPE, bind(tools, loudnorm)),
        (THUMBNAIL_TYPE, bind(tools, thumbnail)),
        (FASTSTART_TYPE, bind(tools, faststart)),
        (PROBE_TYPE, bind(tools, probe)),
    ]
}
