use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::asset_ref::project_ref;
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::{
    ExportImages, ExportVideo, Gl, Variant, export_base_name, still_file_names, variant_size,
    video_file_names,
};
use opsis_compile::text::number;
use opsis_compile::time::{frames_to_ms, lerp, ms_to_frames_from};
use opsis_compile::timeline::chapters::{
    FilmChapter, MIN_LISTED_CHAPTERS, chapter_list, chapter_metadata, film_chapters,
};
use opsis_engine::files::write_via_temp;
use opsis_engine::http_files::FileServer;
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell, require_engine_file};
use opsis_media::codecs::{codec_info, finish_args, has_sound_track, is_image_codec, mux_args};
use opsis_media::executors::{Tools, tool_error};
use opsis_media::ffmpeg::RunOptions;
use opsis_project::paths::to_posix_rel;
use opsis_runner::context::{ExecError, ExecutorContext, ExecutorFn, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::timeline::{ReframeMode, SpecAssets, TimelineSpec};
use opsis_types::values::{
    Artifact, ArtifactKind, CompileValue, ImageValue, MediaKind, MediaValue, PortValues,
};

use crate::export_project::{ExportSources, export_code};
use crate::film::{Stage, mixed, spec_for};
use crate::picture::{PictureOptions, cached_picture, picture_key};
use crate::protocol::{COMPOSITION_ID, MediaJob, RenderTarget, SidecarCall, StillJob};
use crate::sidecar::{Sidecar, SidecarError};

const MIX_STAGE: Stage = (0.0, 0.05);
const RENDER_STAGE: Stage = (0.2, 1.0);
const FREE_LICENSE: &str = "free-license";

pub struct RenderEngine {
    pub sidecar: Arc<Sidecar>,
    pub tools: Tools,
    pub serve_url: String,
    pub binaries_directory: Option<String>,
    pub sources: ExportSources,
}

fn sidecar_error(error: SidecarError) -> ExecError {
    match error {
        SidecarError::Cancelled => ExecError::Cancelled,
        other => ExecError::new(other.to_string()),
    }
}

fn license_key(ctx: &ExecutorContext) -> String {
    let key = ctx.env.settings.render.license_key.trim();
    if key.is_empty() {
        FREE_LICENSE.to_owned()
    } else {
        key.to_owned()
    }
}

fn render_target(
    ctx: &ExecutorContext,
    engine: &RenderEngine,
    spec: TimelineSpec,
    gl: Gl,
) -> Result<RenderTarget, ExecError> {
    let platform = ShellPlatform::current().map_err(|e| ExecError::new(e.to_string()))?;
    let paths = engine_paths(&ctx.env.engine_dir, platform);
    let shell = require_engine_file(find_headless_shell(&paths))
        .map_err(|e| ExecError::new(e.to_string()))?;
    Ok(RenderTarget {
        serve_url: engine.serve_url.clone(),
        browser_executable: shell.to_string_lossy().into_owned(),
        binaries_directory: engine.binaries_directory.clone(),
        license_key: license_key(ctx),
        composition: COMPOSITION_ID.to_owned(),
        spec,
        gl,
    })
}

fn project_path(ctx: &ExecutorContext, file: &Path) -> Result<String, ExecError> {
    to_posix_rel(&ctx.project_dir, file)
        .ok_or_else(|| ExecError::new(format!("{} isn't in the project.", file.display())))
}

fn first_artifact(artifacts: &[Artifact]) -> Result<&Artifact, ExecError> {
    artifacts
        .first()
        .ok_or_else(|| ExecError::new("Nothing was exported. Pick at least one size."))
}

fn kind_of_codec(codec: Codec) -> MediaKind {
    if is_image_codec(codec) {
        MediaKind::Image
    } else {
        MediaKind::Video
    }
}

fn artifact_kind(codec: Codec) -> ArtifactKind {
    if is_image_codec(codec) {
        ArtifactKind::Image
    } else {
        ArtifactKind::Video
    }
}

fn sized(
    spec: &TimelineSpec,
    width: f64,
    height: f64,
    reframe: ReframeMode,
    project_base: &str,
) -> TimelineSpec {
    TimelineSpec {
        width,
        height,
        reframe: Some(reframe),
        assets: Some(SpecAssets {
            project_base: Some(project_base.to_owned()),
            versions: None,
        }),
        ..spec.clone()
    }
}

struct VariantJob<'a> {
    ctx: &'a ExecutorContext,
    engine: &'a RenderEngine,
    spec: TimelineSpec,
    p: &'a ExportVideo,
    progress: &'a (dyn Fn(f64) + Send + Sync),
}

async fn render_to(job: &VariantJob<'_>, output: &Path, muted: bool) -> Result<(), ExecError> {
    let p = job.p;
    let media = MediaJob {
        target: render_target(job.ctx, job.engine, job.spec.clone(), p.gl)?,
        output: output.to_string_lossy().into_owned(),
        codec: p.codec,
        crf: codec_info(p.codec).crf.then_some(p.crf as f64),
        scale: p.scale,
        muted,
    };
    job.engine
        .sidecar
        .call(
            SidecarCall::RenderMedia(media),
            &job.ctx.cancel,
            Some(job.progress),
        )
        .await
        .map_err(sidecar_error)
}

async fn picture_for(job: &VariantJob<'_>) -> Result<PathBuf, ExecError> {
    let p = job.p;
    let opts = PictureOptions {
        codec: p.codec,
        crf: p.crf as f64,
        scale: p.scale,
        gl: p.gl,
    };
    let key = picture_key(&job.spec, &opts, &job.ctx.env.app_version);
    let name = format!("{key}.{}", codec_info(p.codec).ext);
    let picture = cached_picture(&job.ctx.project_dir, &name, |temp| async move {
        render_to(job, &temp, true).await
    })
    .await?;
    if picture.reused {
        job.ctx.log(
            LogLevel::Info,
            "The picture hasn't changed, so only the sound is added again.",
        );
        (job.progress)(1.0);
    }
    Ok(picture.file)
}

struct Chapters {
    chapters: Vec<FilmChapter>,
    metadata: Option<PathBuf>,
    listed: bool,
}

async fn chapters_for(
    ctx: &ExecutorContext,
    spec: &TimelineSpec,
    p: &ExportVideo,
) -> Result<Chapters, ExecError> {
    let chapters = if p.codec == Codec::Gif {
        Vec::new()
    } else {
        film_chapters(spec)
    };
    let metadata = (p.chapters && !chapters.is_empty()).then(|| ctx.dir.join("chapters.txt"));
    if let Some(file) = &metadata {
        tokio::fs::write(file, chapter_metadata(&chapters)).await?;
    }
    let listed = p.chapter_list && chapters.len() >= MIN_LISTED_CHAPTERS;
    if !chapters.is_empty() {
        let titles: Vec<&str> = chapters.iter().map(|c| c.title.as_str()).collect();
        ctx.log(LogLevel::Info, format!("Chapters: {}", titles.join(", ")));
    }
    if p.chapter_list && !chapters.is_empty() && !listed {
        ctx.log(LogLevel::Warn, format!("The chapter list needs at least {MIN_LISTED_CHAPTERS} chapters of ten seconds or more, so it wasn't saved."));
    }
    Ok(Chapters {
        chapters,
        metadata,
        listed,
    })
}

async fn run_tool(
    ctx: &ExecutorContext,
    engine: &RenderEngine,
    args: Vec<String>,
) -> Result<(), ExecError> {
    engine
        .tools
        .run(
            &args,
            RunOptions {
                cancel: Some(&ctx.cancel),
                ..RunOptions::default()
            },
        )
        .await
        .map_err(tool_error)
}

async fn render_variant(
    job: &VariantJob<'_>,
    variant: Variant,
    out: &Path,
    chapters: &Chapters,
) -> Result<Vec<Artifact>, ExecError> {
    let (ctx, codec) = (job.ctx, job.p.codec);
    let shown = |path: &Path| path.to_string_lossy().into_owned();
    match &job.spec.mix {
        Some(mix) if has_sound_track(codec) => {
            let sound = ctx.ref_to_path(&mix.src)?;
            let picture = picture_for(job).await?;
            write_via_temp(out, |temp| {
                run_tool(
                    ctx,
                    job.engine,
                    mux_args(codec, &shown(&picture), &shown(&sound), &shown(&temp)),
                )
            })
            .await?;
        }
        _ => {
            write_via_temp(
                out,
                |temp| async move { render_to(job, &temp, false).await },
            )
            .await?
        }
    }
    let faststart = codec_info(codec).faststart;
    if faststart || chapters.metadata.is_some() {
        let metadata = chapters.metadata.as_deref().map(shown);
        write_via_temp(out, |temp| {
            run_tool(
                ctx,
                job.engine,
                finish_args(&shown(out), &shown(&temp), faststart, metadata.as_deref()),
            )
        })
        .await?;
    }
    let mut artifacts = vec![Artifact {
        kind: artifact_kind(codec),
        path: project_path(ctx, out)?,
        label: Some(variant.id().to_owned()),
        bytes: None,
        stamp: None,
    }];
    if chapters.listed {
        let stem = out
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let list = out.with_file_name(format!("{stem}.youtube-chapters.txt"));
        tokio::fs::write(&list, chapter_list(&chapters.chapters)).await?;
        artifacts.push(Artifact {
            kind: ArtifactKind::File,
            path: project_path(ctx, &list)?,
            label: Some(format!("{} chapter list", variant.id())),
            bytes: None,
            stamp: None,
        });
    }
    Ok(artifacts)
}

async fn export_video(
    ctx: Arc<ExecutorContext>,
    _: PortValues,
    engine: Arc<RenderEngine>,
) -> Result<ExecutorOutput, ExecError> {
    let p: ExportVideo = ctx.params()?;
    let spec = mixed(&ctx, &*engine.tools, spec_for(&ctx)?, MIX_STAGE).await?;
    let names = video_file_names(&export_base_name(&p.file_name), p.codec, &p.variants);
    ctx.progress(0.0, Some("Preparing render engine"));
    let server = FileServer::start(&ctx.project_dir).await?;
    let chapters = chapters_for(&ctx, &spec, &p).await?;
    let count = p.variants.len() as f64;
    let mut artifacts = Vec::new();
    for (i, (variant, name)) in p.variants.iter().zip(&names).enumerate() {
        let size = variant_size(*variant, &ctx.workflow.settings);
        let out = ctx.out_dir.join(name);
        ctx.log(
            LogLevel::Info,
            format!(
                "Exporting {} ({}×{}) as {name}",
                variant.id(),
                number(size.width),
                number(size.height)
            ),
        );
        let message = format!("Rendering {}", variant.id());
        let progress = |x: f64| {
            ctx.progress(
                lerp(RENDER_STAGE.0, RENDER_STAGE.1, (i as f64 + x) / count),
                Some(&message),
            )
        };
        let job = VariantJob {
            ctx: &ctx,
            engine: &engine,
            spec: sized(&spec, size.width, size.height, p.reframe, &server.url),
            p: &p,
            progress: &progress,
        };
        artifacts.extend(render_variant(&job, *variant, &out, &chapters).await?);
    }
    drop(server);
    let exported = first_artifact(&artifacts)?.path.clone();
    let probe = engine
        .tools
        .probe(&ctx.project_dir.join(&exported))
        .await
        .map_err(tool_error)?;
    let media = MediaValue {
        src: Some(project_ref(&exported)),
        kind: kind_of_codec(p.codec),
        duration_ms: Some(frames_to_ms(spec.duration_in_frames, spec.fps)),
        width: None,
        height: None,
        has_audio: Some(probe.audio.is_some()),
    };
    Ok(ExecutorOutput {
        outputs: [("media".to_owned(), CompileValue::Media(media))]
            .into_iter()
            .collect(),
        artifacts,
        trace_id: None,
    })
}

async fn export_images(
    ctx: Arc<ExecutorContext>,
    _: PortValues,
    engine: Arc<RenderEngine>,
) -> Result<ExecutorOutput, ExecError> {
    let p: ExportImages = ctx.params()?;
    let spec = spec_for(&ctx)?;
    let names = still_file_names(&export_base_name(&p.file_name), &p.presets, p.format);
    ctx.progress(0.0, Some("Preparing render engine"));
    let server = FileServer::start(&ctx.project_dir).await?;
    let count = p.presets.len() as f64;
    let mut artifacts = Vec::new();
    for (i, (preset, name)) in p.presets.iter().zip(&names).enumerate() {
        let size = preset.size();
        let out = ctx.out_dir.join(name);
        ctx.progress(
            lerp(RENDER_STAGE.0, RENDER_STAGE.1, i as f64 / count),
            Some(&format!("Still {}", preset.id())),
        );
        let still = StillJob {
            target: render_target(
                &ctx,
                &engine,
                sized(
                    &spec,
                    size.width,
                    size.height,
                    ReframeMode::Follow,
                    &server.url,
                ),
                p.gl,
            )?,
            frame: ms_to_frames_from(p.at_ms as f64, spec.fps, 0.0),
            output: out.to_string_lossy().into_owned(),
            format: p.format,
            scale: None,
            jpeg_quality: None,
        };
        engine
            .sidecar
            .call(SidecarCall::RenderStill(still), &ctx.cancel, None)
            .await
            .map_err(sidecar_error)?;
        artifacts.push(Artifact {
            kind: ArtifactKind::Image,
            path: project_path(&ctx, &out)?,
            label: Some(preset.id().to_owned()),
            bytes: None,
            stamp: None,
        });
    }
    drop(server);
    let image = ImageValue {
        src: Some(project_ref(&first_artifact(&artifacts)?.path)),
        width: None,
        height: None,
        scale: None,
        body: None,
    };
    Ok(ExecutorOutput {
        outputs: [("image".to_owned(), CompileValue::Image(image))]
            .into_iter()
            .collect(),
        artifacts,
        trace_id: None,
    })
}

fn bind<F, Fut>(engine: &Arc<RenderEngine>, run: F) -> ExecutorFn
where
    F: Fn(Arc<ExecutorContext>, PortValues, Arc<RenderEngine>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    let engine = Arc::clone(engine);
    executor(move |ctx, inputs| run(ctx, inputs, Arc::clone(&engine)))
}

pub fn output_executors(engine: &Arc<RenderEngine>) -> Vec<(&'static str, ExecutorFn)> {
    vec![
        ("output.render", bind(engine, export_video)),
        ("output.still", bind(engine, export_images)),
        ("output.exportProject", bind(engine, export_code)),
    ]
}
