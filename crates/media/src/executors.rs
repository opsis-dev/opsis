use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use futures::future::try_join_all;
use opsis_compile::asset_ref::{kind_from_path, project_ref};
use opsis_compile::nodes::audio::{
    END_HIT_FILE, GENERATE_TYPE, GenerateSound, RISER_FILE, RISER_TYPE, ROOM_TONE_TYPE, Riser,
    RoomTone, TONES_TYPE, TRANSITION_SOUNDS_TYPE, Tones, TransitionSoundFrom, TransitionSounds,
    sound_shape, transition_cues, transition_sound_file,
};
use opsis_compile::nodes::grade::{LOOK_TYPE, LookFile, grade_needs_video};
use opsis_compile::nodes::media::{
    APPLY_GRADE_TYPE, CLEAN_SPEECH_TYPE, CONCAT_TYPE, CONVERT_TYPE, ChangePitch, ChangeSpeed,
    CleanSpeech, EXTRACT_AUDIO_TYPE, Encoding, EvenOutVolume, ExtractSound, FASTSTART_TYPE,
    FIT_MUSIC_TYPE, FitMusic, FitPlan, FixClicks, GIF_TYPE, GrabFrame, HUM_TYPE, IMPORT_TYPE,
    JoinFiles, JoinSizeFrom, LOUDNORM_TYPE, MakeGif, MediaFile, NoiseMethod, PAUSES_TYPE,
    PITCH_TYPE, REPAIR_TYPE, REVERSE_TYPE, RemoveHum, Resize, ReverseFile, SCALE_PAD_TYPE,
    SOUND_PICTURE_PORT, SOUND_PICTURE_TYPE, SPEED_TYPE, ShortenPauses, SoundPicture,
    THUMBNAIL_TYPE, TRIM_TYPE, TrimFile, VOICE_TYPE, VoiceEffects, fit_plan, joined_files,
    kept_parts,
};
use opsis_compile::registry::{node_def, node_name};
use opsis_compile::text::{format_duration, single_quoted};
use opsis_probe::Probe;
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::numbers;
use opsis_types::values::{
    ArtifactKind, CompileValue, ImageValue, Measured, MediaKind, MediaValue, PortValues, StyleEntry,
};

use crate::codecs::{
    EVEN_FRAME, EditedFile, FASTSTART, FrameGrab, PCM_16, audio_format_args, audio_format_for,
    codec_info, convert_args, convert_codec, edit_audio, edit_video, edited_file, even_down,
    extract_args, ffmpeg_seconds, frame_grab_args, grade_sound_args, grade_video_args, serde_name,
    sound_file, strings,
};
use crate::denoise::{DENOISE_RATE, denoised};
use crate::ffmpeg::{MediaTools, OnProgress, RunOptions, ToolError, can_stream_copy};
use crate::filters::{
    GifLook, JoinPart, JoinSize, fit_filter, fit_graph, fit_inputs, gif_filter, hum_filter,
    hum_listen_filter, join_filter, joined_parts_graph, loudnorm_filter, noise_level,
    noise_level_listen_filter, pitch_filter, repair_filter, rms_level, silence_listen_filter,
    silences, sound_picture_graph, spectral_floor, speech_filter, speed_args, speed_sound_args,
    voice_filter,
};
use crate::generate::{
    Beeps, beeps, length_ms, room_response, room_tone, write_stereo_wav, write_wav,
};
use crate::grade::{GradeError, GradeSize, grade_filters};
use crate::reverse::{
    PieceBudget, REVERSE_BUDGET_BYTES, ReversePiece, ReversedPart, WHOLE_FILE, frame_bytes,
    join_reversed_args, piece_length_ms, reverse_args, reverse_pieces, reverse_sound_args,
    reversed_parts_list,
};

const CONNECT_MEDIA: &str = "Connect a media input (and run the node that produces it).";
const MEDIA_PORT: &str = "media";
const SOUND_PORT: &str = "audio";
const MP4_PICTURES: [&str; 5] = ["h264", "hevc", "av1", "vp9", "mpeg4"];
const LAST_FRAME_MARGIN_MS: f64 = 100.0;
const PARTS_SHARE: f64 = 0.9;

pub type Tools = Arc<dyn MediaTools>;

impl From<GradeError> for ExecError {
    fn from(error: GradeError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileCheckError {
    #[error("This file isn't in the project folder any more.")]
    Missing,
    #[error("This is a folder, not a file.")]
    Folder,
    #[error("Can't open this file: {0}.")]
    Unopenable(std::io::ErrorKind),
    #[error(transparent)]
    Tool(#[from] ToolError),
}

impl From<FileCheckError> for ExecError {
    fn from(error: FileCheckError) -> Self {
        match error {
            FileCheckError::Tool(tool) => tool_error(tool),
            other => Self::new(other.to_string()),
        }
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

#[derive(Debug)]
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

fn duration_ms(probe: &Probe) -> f64 {
    probe.duration_ms.map_or(0.0, f64::from)
}

fn media_kind(probe: &Probe, file: &str) -> MediaKind {
    match kind_from_path(file) {
        MediaKind::Video if probe.video.is_none() => MediaKind::Audio,
        named => named,
    }
}

fn media_value_from(probe: &Probe, src: &str, kind: MediaKind) -> MediaValue {
    MediaValue {
        src: Some(src.to_owned()),
        kind,
        duration_ms: probe.duration_ms,
        width: probe.video.as_ref().map(|v| v.width),
        height: probe.video.as_ref().map(|v| v.height),
        has_audio: Some(probe.audio.is_some()),
    }
}

const fn artifact_kind(kind: MediaKind) -> ArtifactKind {
    match kind {
        MediaKind::Video => ArtifactKind::Video,
        MediaKind::Image => ArtifactKind::Image,
        MediaKind::Audio => ArtifactKind::Audio,
        MediaKind::TerminalTake => ArtifactKind::File,
    }
}

fn output(port: &str, value: CompileValue) -> PortValues {
    PortValues::from([(port.to_owned(), value)])
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
    let artifacts = vec![ctx.artifact(artifact_kind(media.kind), file, label)];
    Ok(ExecutorOutput {
        outputs: output(MEDIA_PORT, CompileValue::Media(media)),
        artifacts,
        trace_id: None,
        measured: Vec::new(),
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
                job.duration_ms
                    .or_else(|| input.value.duration_ms.map(f64::from)),
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

fn sound_only_file(probe: &Probe, src: &str, node: &str) -> Result<EditedFile, ExecError> {
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

async fn import_media(
    ctx: Arc<ExecutorContext>,
    _: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: MediaFile = ctx.params()?;
    if p.path.is_empty() {
        return Err(ExecError::new("Choose a file."));
    }
    let src = project_ref(&p.path);
    let checked = check_media_file(&*tools, &ctx.ref_to_path(&src)?, &src).await?;
    Ok(ExecutorOutput {
        outputs: output(MEDIA_PORT, CompileValue::Media(checked.media)),
        measured: vec![checked.measured],
        ..ExecutorOutput::default()
    })
}

#[derive(Debug)]
pub struct CheckedFile {
    pub media: MediaValue,
    pub measured: Measured,
}

fn opening_error(error: &std::io::Error) -> FileCheckError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileCheckError::Missing,
        std::io::ErrorKind::IsADirectory => FileCheckError::Folder,
        kind => FileCheckError::Unopenable(kind),
    }
}

pub async fn check_media_file(
    tools: &dyn MediaTools,
    file: &Path,
    src: &str,
) -> Result<CheckedFile, FileCheckError> {
    let meta = tokio::fs::metadata(file)
        .await
        .map_err(|e| opening_error(&e))?;
    if !meta.is_file() {
        return Err(FileCheckError::Folder);
    }
    tokio::fs::File::open(file)
        .await
        .map_err(|e| opening_error(&e))?;
    let bytes = meta.len();
    let probe = tools.probe(file).await?;
    let mut media = media_value_from(&probe, src, media_kind(&probe, src));
    if media.kind == MediaKind::Image {
        media.duration_ms = None;
    }
    let fps = probe
        .video
        .as_ref()
        .and_then(|v| v.fps)
        .filter(|_| media.kind == MediaKind::Video);
    Ok(CheckedFile {
        media,
        measured: Measured::File { bytes, fps },
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
    if p.start_ms == 0 && p.end_ms == 0 {
        return Ok(ExecutorOutput {
            outputs: output(MEDIA_PORT, CompileValue::Media(input.value)),
            ..ExecutorOutput::default()
        });
    }
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

fn concat_parts(items: &[LinkedMedia], probes: &[Probe], still_ms: f64) -> Vec<(bool, JoinPart)> {
    items
        .iter()
        .zip(probes)
        .map(|(m, probe)| {
            let still = m.value.kind == MediaKind::Image;
            let duration_ms = if still {
                m.value.duration_ms.map_or(still_ms, f64::from)
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
    p: &JoinFiles,
    paths: &[String],
    probes: &[Probe],
    parts: &[(bool, JoinPart)],
    out: String,
) -> Vec<String> {
    let first = probes
        .iter()
        .find_map(|x| x.video.as_ref())
        .filter(|_| p.match_to == JoinSizeFrom::First);
    let settings = &ctx.workflow.settings;
    let to = JoinSize {
        width: first.map_or(settings.width, |v| v.width),
        height: first.map_or(settings.height, |v| v.height),
        fps: first
            .and_then(|v| v.fps)
            .unwrap_or_else(|| f64::from(settings.fps)),
    };
    let join = join_filter(
        &parts.iter().map(|(_, part)| *part).collect::<Vec<_>>(),
        to,
        &p.background,
    );
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
    let items: Vec<LinkedMedia> = joined_files(&inputs)
        .into_iter()
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
    let parts = concat_parts(&items, &probes, f64::from(p.still_ms));
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
        reencode_args(&ctx, &p, &paths, &probes, &parts, out)
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
            speed_sound_args(i, o, p.rate, p.keep_pitch, &out.codecs)
        })
        .await;
    };
    let fps = video
        .fps
        .unwrap_or_else(|| f64::from(ctx.workflow.settings.fps));
    let sound = p.keep_audio && media_has_sound(&ctx, &*tools, &input).await?;
    let job = Transcode {
        file: "speed.mp4".into(),
        label: &label,
        duration_ms,
    };
    transcode(&ctx, &*tools, &input, job, |i, o| {
        speed_args(i, o, p.rate, fps, sound.then_some(p.keep_pitch))
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
        let progress = |p: f64| ctx.progress(p.mul_add(share, start), Some(&message));
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
            p.mul_add(1.0 - PARTS_SHARE, PARTS_SHARE),
            Some("Joining the parts"),
        );
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
    let keep_audio = ctx.params::<ReverseFile>()?.keep_audio;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let src = ctx.ref_to_path(&input.src)?;
    let probe = tools.probe(&src).await.map_err(tool_error)?;
    let label = "Reversed";
    let Some(video) = &probe.video else {
        let out = sound_only_file(&probe, &input.src, "Reverse")?;
        if !keep_audio {
            return Err(ExecError::new(
                "This file is only sound, so turning off Keep the sound leaves nothing.",
            ));
        }
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
    let sound = keep_audio && media_has_sound(&ctx, &*tools, &input).await?;
    let pieces = reverse_pieces(PieceBudget {
        duration_ms: duration_ms(&probe),
        fps: video
            .fps
            .unwrap_or_else(|| f64::from(ctx.workflow.settings.fps)),
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
        fit_filter(p.mode, w, h, &p.background, p.anchor)
    } else {
        fit_filter(p.mode, even_down(w), even_down(h), &p.background, p.anchor)
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
        args.extend(convert_args(&p));
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
            args.extend(grade_video_args(&p));
            args.extend(grade_sound_args(&p, sound));
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
    let filter = gif_filter(
        f64::from(p.fps),
        f64::from(p.width),
        &GifLook {
            dither: p.dither,
            colours: p.colours,
            palette_from: p.palette_from,
        },
    );
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
        args.extend(extract_args(p.format, p.bitrate));
        args.push(o.to_owned());
        args
    })
    .await
}

struct SoundFilter<'a> {
    stem: &'a str,
    label: &'a str,
    silent: &'a str,
    filter: String,
    sound: Option<&'a Path>,
}

async fn filter_sound(
    ctx: &ExecutorContext,
    inputs: &PortValues,
    tools: &dyn MediaTools,
    job: SoundFilter<'_>,
) -> Result<ExecutorOutput, ExecError> {
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
        .ok_or_else(|| ExecError::new(job.silent))?;
    let picture = match &probe.video {
        Some(v) if MP4_PICTURES.contains(&v.codec.as_str()) => strings(&["-c:v", "copy"]),
        _ => {
            let mut args = strings(&["-vf", EVEN_FRAME]);
            args.extend(edit_video());
            args
        }
    };
    let ext = format.map_or_else(|| "mp4".to_owned(), |f| serde_name(&f));
    let transcoded = Transcode {
        file: format!("{}.{ext}", job.stem),
        label: job.label,
        duration_ms: None,
    };
    let filter = job.filter;
    let sound = job.sound.map(|path| path.to_string_lossy().into_owned());
    transcode(ctx, tools, &input, transcoded, |i, o| {
        let mut args = vec!["-i".to_owned(), i.to_owned()];
        if let Some(sound) = sound {
            args.extend(strings(&["-i", &sound, "-map", "0:v?", "-map", "1:a"]));
            if let Some(channels) = audio.channels {
                args.extend(["-ac".to_owned(), channels.to_string()]);
            }
        }
        args.extend(["-af".to_owned(), filter]);
        if let Some(rate) = audio.sample_rate {
            args.extend(["-ar".to_owned(), rate.to_string()]);
        }
        if let Some(f) = format {
            args.extend(audio_format_args(f));
        } else {
            args.extend(picture);
            args.extend(edit_audio());
        }
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
    let job = SoundFilter {
        stem: "loudnorm",
        label: "Loudness normalised",
        silent: "This file has no sound to even out.",
        filter: loudnorm_filter(p.integrated, p.true_peak, p.lra),
        sound: None,
    };
    filter_sound(&ctx, &inputs, &*tools, job).await
}

async fn hum_level(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    source: &Path,
    hertz: f64,
) -> Result<Option<f64>, ExecError> {
    let lines = std::sync::Mutex::new(Vec::new());
    let log = |line: &str| lines.lock().expect("log lines").push(line.to_owned());
    let args = vec![
        "-i".to_owned(),
        source.to_string_lossy().into_owned(),
        "-af".to_owned(),
        hum_listen_filter(hertz),
        "-f".to_owned(),
        "null".to_owned(),
        "-".to_owned(),
    ];
    let mut options = run_options(ctx, None, None);
    options.on_log = Some(&log);
    tools.run(&args, options).await.map_err(tool_error)?;
    Ok(rms_level(&lines.into_inner().expect("log lines")))
}

async fn hum(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: RemoveHum = ctx.params()?;
    let hertz = if let Some(hertz) = p.mains.hertz() {
        hertz
    } else {
        let input = require_media(inputs.get(MEDIA_PORT))?;
        let source = ctx.ref_to_path(&input.src)?;
        let fifty = hum_level(&ctx, &*tools, &source, 50.0).await?;
        let sixty = hum_level(&ctx, &*tools, &source, 60.0).await?;
        let hertz = if sixty.unwrap_or(f64::NEG_INFINITY) > fifty.unwrap_or(f64::NEG_INFINITY) {
            60.0
        } else {
            50.0
        };
        ctx.log(LogLevel::Info, format!("Found the hum at {hertz} Hz."));
        hertz
    };
    let job = SoundFilter {
        stem: "hum",
        label: "Hum removed",
        silent: "This file has no sound to clean.",
        filter: hum_filter(hertz, p.harmonics, p.width, p.depth),
        sound: None,
    };
    filter_sound(&ctx, &inputs, &*tools, job).await
}

async fn sound_picture(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: SoundPicture = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let source = ctx.ref_to_path(&input.src)?;
    let probe = tools.probe(&source).await.map_err(tool_error)?;
    if probe.audio.is_none() {
        return Err(ExecError::new("This file has no sound to draw."));
    }
    let file = "picture.png";
    let args = vec![
        "-i".to_owned(),
        source.to_string_lossy().into_owned(),
        "-filter_complex".to_owned(),
        sound_picture_graph(&p),
        "-map".to_owned(),
        "[out]".to_owned(),
        "-frames:v".to_owned(),
        "1".to_owned(),
        ctx.dir.join(file).to_string_lossy().into_owned(),
    ];
    tools
        .run(&args, run_options(&ctx, None, None))
        .await
        .map_err(tool_error)?;
    let value = ImageValue {
        src: Some(ctx.project_file_ref(file)),
        width: Some(p.width),
        height: Some(p.height),
        scale: None,
        body: None,
    };
    Ok(ExecutorOutput {
        outputs: output(SOUND_PICTURE_PORT, CompileValue::Image(value)),
        artifacts: vec![ctx.artifact(ArtifactKind::Image, file, "Sound picture")],
        trace_id: None,
        measured: Vec::new(),
    })
}

fn seed_of(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn made_sound(
    ctx: &ExecutorContext,
    file: &str,
    label: &str,
    samples: &[f32],
) -> Result<ExecutorOutput, ExecError> {
    write_wav(&ctx.dir.join(file), samples).map_err(|e| ExecError::new(e.to_string()))?;
    let value = MediaValue {
        src: Some(ctx.project_file_ref(file)),
        kind: MediaKind::Audio,
        duration_ms: Some(length_ms(samples)),
        width: None,
        height: None,
        has_audio: Some(true),
    };
    Ok(ExecutorOutput {
        outputs: output(SOUND_PORT, CompileValue::Media(value)),
        artifacts: vec![ctx.artifact(ArtifactKind::Audio, file, label)],
        trace_id: None,
        measured: Vec::new(),
    })
}

async fn make_room_tone(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    _tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: RoomTone = ctx.params()?;
    let samples = room_tone(p.colour, p.level, seed_of(&ctx.node.id));
    made_sound(&ctx, "room-tone.wav", "Room tone", &samples)
}

async fn make_tones(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    _tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: Tones = ctx.params()?;
    let samples = beeps(&Beeps {
        kind: p.kind,
        pitch_hz: p.pitch_hz,
        length_ms: f64::from(p.length_ms),
        count: p.count,
        spacing_ms: f64::from(p.spacing_ms),
        wave: p.wave,
        final_length: p.final_length,
        final_pitch: p.final_pitch,
    });
    made_sound(&ctx, "tones.wav", "Tones", &samples)
}

const NO_SPEECH: &str = "This file has no sound to clean.";

async fn noise_level_of(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    file: &Path,
) -> Result<Option<f64>, ExecError> {
    let args = vec![
        "-i".to_owned(),
        file.to_string_lossy().into_owned(),
        "-map".to_owned(),
        "0:a:0".to_owned(),
        "-af".to_owned(),
        noise_level_listen_filter().to_owned(),
        "-f".to_owned(),
        "null".to_owned(),
        "-".to_owned(),
    ];
    Ok(noise_level(&logged(ctx, tools, &args).await?))
}

pub fn speech_decode_args(source: &Path, out: &Path) -> Vec<String> {
    let mut args = vec!["-i".to_owned(), source.to_string_lossy().into_owned()];
    args.extend(strings(&["-map", "0:a:0", "-ac", "1", "-ar"]));
    args.push(DENOISE_RATE.to_string());
    args.extend(strings(&PCM_16));
    args.push(out.to_string_lossy().into_owned());
    args
}

fn wav_error(error: &hound::Error) -> ExecError {
    ExecError::new(format!("The sound couldn't be read or written: {error}"))
}

fn denoise_file(from: &Path, to: &Path, strength: f64) -> Result<(), ExecError> {
    let mut reader = hound::WavReader::open(from).map_err(|e| wav_error(&e))?;
    let samples = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| wav_error(&e))?;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: DENOISE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(to, spec).map_err(|e| wav_error(&e))?;
    for sample in denoised(&samples, strength) {
        writer.write_sample(sample).map_err(|e| wav_error(&e))?;
    }
    writer.finalize().map_err(|e| wav_error(&e))
}

async fn clean_speech(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: CleanSpeech = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let source = ctx.ref_to_path(&input.src)?;
    let probe = tools.probe(&source).await.map_err(tool_error)?;
    if probe.audio.is_none() {
        return Err(ExecError::new(NO_SPEECH));
    }
    let before = noise_level_of(&ctx, &*tools, &source).await?;
    let learned = p.method == NoiseMethod::Learned && p.noise > 0.0;
    let voice = ctx.dir.join("voice.wav");
    if learned {
        let decoded = ctx.dir.join("voice-in.wav");
        tools
            .run(
                &speech_decode_args(&source, &decoded),
                run_options(&ctx, None, None),
            )
            .await
            .map_err(tool_error)?;
        let (from, to, strength) = (decoded.clone(), voice.clone(), p.noise);
        tokio::task::spawn_blocking(move || denoise_file(&from, &to, strength))
            .await
            .map_err(|e| ExecError::new(e.to_string()))??;
        if let Err(error) = std::fs::remove_file(&decoded) {
            log::debug!("kept {}: {error}", decoded.display());
        }
    }
    let job = SoundFilter {
        stem: "speech",
        label: "Speech cleaned",
        silent: NO_SPEECH,
        filter: speech_filter(&p, (!learned).then(|| spectral_floor(before))),
        sound: learned.then_some(voice.as_path()),
    };
    let mut out = filter_sound(&ctx, &inputs, &*tools, job).await?;
    if learned && let Err(error) = std::fs::remove_file(&voice) {
        log::debug!("kept {}: {error}", voice.display());
    }
    let made = out
        .outputs
        .get(MEDIA_PORT)
        .and_then(CompileValue::as_media)
        .and_then(|m| m.src.clone());
    let after = match made {
        Some(src) => noise_level_of(&ctx, &*tools, &ctx.ref_to_path(&src)?).await?,
        None => None,
    };
    if let (Some(before_db), Some(after_db)) = (before, after) {
        out.measured.push(Measured::NoiseLevel {
            before_db,
            after_db,
        });
    }
    Ok(out)
}

async fn make_generated(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    _tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: GenerateSound = ctx.params()?;
    let shape = sound_shape(&p);
    let channels = tokio::task::spawn_blocking(move || crate::sound_maker::make(&shape))
        .await
        .map_err(|e| ExecError::new(e.to_string()))?;
    let file = "generated.wav";
    crate::generate::write_stereo_pcm(&ctx.dir.join(file), &channels)
        .map_err(|e| ExecError::new(e.to_string()))?;
    let value = MediaValue {
        src: Some(ctx.project_file_ref(file)),
        kind: MediaKind::Audio,
        duration_ms: Some(length_ms(&channels[0])),
        width: None,
        height: None,
        has_audio: Some(true),
    };
    Ok(ExecutorOutput {
        outputs: output(SOUND_PORT, CompileValue::Media(value)),
        artifacts: vec![ctx.artifact(ArtifactKind::Audio, file, "Generated sound")],
        trace_id: None,
        measured: Vec::new(),
    })
}

async fn make_transition_sounds(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    _tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: TransitionSounds = ctx.params()?;
    if p.sound_from == TransitionSoundFrom::File {
        return Ok(ExecutorOutput::default());
    }
    let clip = inputs
        .get(opsis_compile::node::CLIP_PORT)
        .and_then(CompileValue::as_clip)
        .ok_or_else(|| ExecError::new("Connect a clip with transitions."))?;
    let settings = &ctx.workflow.settings;
    let lengths: std::collections::BTreeSet<u32> = transition_cues(clip, settings, p.at_cuts)
        .iter()
        .map(|cue| p.length_ms(cue, settings.fps))
        .collect();
    let base = p.shape();
    let mut artifacts = Vec::new();
    for ms in lengths {
        let shape = opsis_compile::nodes::audio::SoundShape {
            seconds: f64::from(ms) / 1000.0,
            ..base
        };
        let channels = tokio::task::spawn_blocking(move || crate::sound_maker::make(&shape))
            .await
            .map_err(|e| ExecError::new(e.to_string()))?;
        let file = transition_sound_file(ms);
        crate::generate::write_stereo_pcm(&ctx.dir.join(&file), &channels)
            .map_err(|e| ExecError::new(e.to_string()))?;
        artifacts.push(ctx.artifact(ArtifactKind::Audio, &file, "Transition sound"));
    }
    Ok(ExecutorOutput {
        artifacts,
        ..ExecutorOutput::default()
    })
}

async fn make_riser(
    ctx: Arc<ExecutorContext>,
    _inputs: PortValues,
    _tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: Riser = ctx.params()?;
    let mut artifacts = Vec::new();
    for (file, shape) in [(RISER_FILE, Some(p.shape())), (END_HIT_FILE, p.hit_shape())] {
        let Some(shape) = shape else {
            continue;
        };
        let channels = tokio::task::spawn_blocking(move || crate::sound_maker::make(&shape))
            .await
            .map_err(|e| ExecError::new(e.to_string()))?;
        crate::generate::write_stereo_pcm(&ctx.dir.join(file), &channels)
            .map_err(|e| ExecError::new(e.to_string()))?;
        artifacts.push(ctx.artifact(ArtifactKind::Audio, file, "Riser"));
    }
    Ok(ExecutorOutput {
        artifacts,
        ..ExecutorOutput::default()
    })
}

async fn logged(
    ctx: &ExecutorContext,
    tools: &dyn MediaTools,
    args: &[String],
) -> Result<Vec<String>, ExecError> {
    let lines = std::sync::Mutex::new(Vec::new());
    let log = |line: &str| lines.lock().expect("log lines").push(line.to_owned());
    let mut options = run_options(ctx, None, None);
    options.on_log = Some(&log);
    tools.run(args, options).await.map_err(tool_error)?;
    Ok(lines.into_inner().expect("log lines"))
}

async fn change_pitch(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: ChangePitch = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    let probe = tools
        .probe(&ctx.ref_to_path(&input.src)?)
        .await
        .map_err(tool_error)?;
    let job = SoundFilter {
        stem: "pitch",
        label: "Pitch changed",
        silent: "This file has no sound to change.",
        filter: pitch_filter(&p, probe.duration_ms),
        sound: None,
    };
    filter_sound(&ctx, &inputs, &*tools, job).await
}

async fn voice_effects(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: VoiceEffects = ctx.params()?;
    let room = match room_response(p.effect, seed_of(&ctx.node.id), p.reverb_ms) {
        Some(channels) => {
            let file = ctx.dir.join("room.wav");
            write_stereo_wav(&file, &channels).map_err(|e| ExecError::new(e.to_string()))?;
            Some(crate::grade::filter_path(&file.to_string_lossy()))
        }
        None => None,
    };
    let job = SoundFilter {
        stem: "voice",
        label: "Voice effect",
        silent: "This file has no sound to change.",
        filter: voice_filter(&p, room.as_deref()),
        sound: None,
    };
    filter_sound(&ctx, &inputs, &*tools, job).await
}

async fn fit_music(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: FitMusic = ctx.params()?;
    let clip = inputs
        .get(opsis_compile::node::CLIP_PORT)
        .and_then(CompileValue::as_clip)
        .ok_or_else(|| ExecError::new("Connect the film, so the song knows how long to be."))?;
    let target_ms =
        f64::from(clip.duration_in_frames) * 1000.0 / f64::from(ctx.workflow.settings.fps.max(1));
    let src = match inputs
        .get(MEDIA_PORT)
        .and_then(CompileValue::as_media)
        .and_then(|m| m.src.clone())
    {
        Some(src) => src,
        None if !p.src.is_empty() => project_ref(&p.src),
        None => return Err(ExecError::new("Choose a song or connect a sound file.")),
    };
    let source = ctx.ref_to_path(&src)?;
    let probe = tools.probe(&source).await.map_err(tool_error)?;
    if probe.audio.is_none() {
        return Err(ExecError::new("This file has no sound to fit."));
    }
    let song_ms = probe
        .duration_ms
        .map(f64::from)
        .ok_or_else(|| ExecError::new("This file's length can't be read."))?;
    let plan = fit_plan(song_ms, target_ms, &p);
    let (tempo, loops) = match plan {
        FitPlan::Stretch { tempo } => (Some(tempo), 0),
        FitPlan::Loop { repeats, .. } => (None, repeats),
        FitPlan::Cut => (None, 0),
    };
    let file = sound_file(&ext_of(&src));
    let out = format!("fitted{}", file.ext);
    let mut args = Vec::new();
    for _ in 0..fit_inputs(plan) {
        args.extend(["-i".to_owned(), source.to_string_lossy().into_owned()]);
    }
    args.extend([
        "-filter_complex".to_owned(),
        fit_graph(plan, target_ms, &p),
        "-map".to_owned(),
        "[out]".to_owned(),
    ]);
    args.extend(file.codecs);
    args.push(ctx.dir.join(&out).to_string_lossy().into_owned());
    let progress = |done: f64| ctx.progress(done, None);
    tools
        .run(&args, run_options(&ctx, Some(target_ms), Some(&progress)))
        .await
        .map_err(tool_error)?;
    let mut result = media_result(&ctx, &*tools, &out, "Fitted music").await?;
    result.measured.push(Measured::Fitted { tempo, loops });
    Ok(result)
}

async fn shorten_pauses(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: ShortenPauses = ctx.params()?;
    let input = require_media(inputs.get(MEDIA_PORT))?;
    if input.value.kind != MediaKind::Audio {
        return Err(ExecError::new(
            "Shorten pauses works on sound files, so a video doesn't lose its sync. Take the sound out with Extract sound first.",
        ));
    }
    let source = ctx.ref_to_path(&input.src)?;
    let probe = tools.probe(&source).await.map_err(tool_error)?;
    let duration_ms = probe
        .duration_ms
        .map(f64::from)
        .ok_or_else(|| ExecError::new("This file's length can't be read."))?;
    let source_text = source.to_string_lossy().into_owned();
    let listen = vec![
        "-i".to_owned(),
        source_text.clone(),
        "-af".to_owned(),
        silence_listen_filter(p.threshold, p.longer_than_ms),
        "-f".to_owned(),
        "null".to_owned(),
        "-".to_owned(),
    ];
    let pauses = silences(&logged(&ctx, &*tools, &listen).await?);
    let parts = kept_parts(duration_ms, &pauses, f64::from(p.keep_ms));
    let kept: f64 = parts.iter().map(|(a, b)| b - a).sum();
    let shortened = pauses
        .iter()
        .filter(|pause| pause.end_ms - pause.start_ms > f64::from(p.keep_ms))
        .count();
    ctx.log(
        LogLevel::Info,
        format!(
            "Saved {} from {}.",
            format_duration(duration_ms - kept),
            opsis_compile::text::plural(numbers::float(shortened), "pause")
        ),
    );
    let format = audio_format_for(ext_of(&input.src).trim_start_matches('.'));
    let job = Transcode {
        file: format!("shortened.{}", serde_name(&format)),
        label: "Pauses shortened",
        duration_ms: Some(kept),
    };
    let graph = joined_parts_graph(&parts);
    transcode(&ctx, &*tools, &input, job, |i, o| {
        let mut args = strings(&["-i", i, "-filter_complex", &graph, "-map", "[out]"]);
        args.extend(audio_format_args(format));
        args.push(o.to_owned());
        args
    })
    .await
}

async fn repair(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
    tools: Tools,
) -> Result<ExecutorOutput, ExecError> {
    let p: FixClicks = ctx.params()?;
    let filter = repair_filter(p.clicks, p.clipping, p.strength)
        .ok_or_else(|| ExecError::new("Turn on clicks or clipping to fix."))?;
    let job = SoundFilter {
        stem: "repaired",
        label: "Clicks and clipping fixed",
        silent: "This file has no sound to fix.",
        filter,
        sound: None,
    };
    filter_sound(&ctx, &inputs, &*tools, job).await
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
    let artifacts = vec![ctx.artifact(ArtifactKind::Image, file, "Thumbnail")];
    Ok(ExecutorOutput {
        outputs: output("image", CompileValue::Image(image)),
        artifacts,
        trace_id: None,
        measured: Vec::new(),
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

fn bind<F, Fut>(tools: &Tools, run: F) -> Executor
where
    F: Fn(Arc<ExecutorContext>, PortValues, Tools) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    let tools = Arc::clone(tools);
    executor(move |ctx, inputs| run(ctx, inputs, Arc::clone(&tools)))
}

pub fn media_executors(tools: &Tools) -> Vec<(&'static str, Executor)> {
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
        (REPAIR_TYPE, bind(tools, repair)),
        (HUM_TYPE, bind(tools, hum)),
        (PAUSES_TYPE, bind(tools, shorten_pauses)),
        (PITCH_TYPE, bind(tools, change_pitch)),
        (VOICE_TYPE, bind(tools, voice_effects)),
        (FIT_MUSIC_TYPE, bind(tools, fit_music)),
        (CLEAN_SPEECH_TYPE, bind(tools, clean_speech)),
        (SOUND_PICTURE_TYPE, bind(tools, sound_picture)),
        (THUMBNAIL_TYPE, bind(tools, thumbnail)),
        (FASTSTART_TYPE, bind(tools, faststart)),
        (ROOM_TONE_TYPE, bind(tools, make_room_tone)),
        (TONES_TYPE, bind(tools, make_tones)),
        (GENERATE_TYPE, bind(tools, make_generated)),
        (TRANSITION_SOUNDS_TYPE, bind(tools, make_transition_sounds)),
        (RISER_TYPE, bind(tools, make_riser)),
    ]
}
