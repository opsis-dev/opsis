use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_compile::asset_ref::{FILM_FOLDER, project_ref};
use opsis_compile::hosted::hosted;
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::{
    ExportImages, ExportVideo, Gl, RENDER_TYPE, STILL_TYPE, StillFormat, Variant, export_base_name,
    still_file_names, variant_size, video_file_names,
};
use opsis_compile::time::{frames_to_ms, lerp, ms_to_frames_from};
use opsis_compile::timeline::chapters::{
    FilmChapter, MIN_LISTED_CHAPTERS, chapter_list, chapter_metadata, film_chapters,
};
use opsis_engine::files::write_via_temp;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_footage::serve::footage_routes;
use opsis_media::codecs::{
    FilmEncoding, codec_info, finish_args, has_sound_track, is_image_codec, mux_args,
};
use opsis_media::executors::{Tools, tool_error};
use opsis_media::ffmpeg::RunOptions;
use opsis_prepare::serve::{PreparedDirs, prepared_routes};
use opsis_project::paths::to_posix_rel;
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::numbers;
use opsis_types::numbers::whole;
use opsis_types::timeline::{ReframeMode, SpecAssets, TimelineSpec};
use opsis_types::values::{
    Artifact, ArtifactKind, CompileValue, ImageValue, MediaKind, MediaValue, PortValues,
};
use serde_json::Value;

use crate::film::{Stage, mixed, spec_for};
use crate::footage::film_footage;
use crate::frames::{FilmError, FilmRender, FilmRun, StillRender, render_film, render_still};
use crate::picture::{PictureOptions, cached_picture, picture_key};

const MIX_STAGE: Stage = (0.0, 0.05);
const RENDER_STAGE: Stage = (0.2, 1.0);

pub struct RenderEngine {
    pub tools: Tools,
    pub film_bundle: PathBuf,
}

impl fmt::Debug for RenderEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderEngine")
            .field("film_bundle", &self.film_bundle)
            .finish_non_exhaustive()
    }
}

fn film_error(error: FilmError) -> ExecError {
    match error {
        FilmError::Cancelled => ExecError::Cancelled,
        other => ExecError::new(other.to_string()),
    }
}

fn browser(ctx: &ExecutorContext) -> Result<&Path, ExecError> {
    ctx.env
        .engine
        .browser()
        .map_err(|e| ExecError::new(e.to_string()))
}

fn driver(ctx: &ExecutorContext, gl: Gl) -> Result<DriverOptions<'_>, ExecError> {
    Ok(DriverOptions {
        executable: browser(ctx)?,
        temp_dir: &ctx.env.temp_dir,
        gl,
        capture: Capture::for_this_computer(),
    })
}

struct HostedFilm {
    url: String,
    spec: Value,
    width: u32,
    height: u32,
}

fn hosted_film(spec: &TimelineSpec, server: &str) -> Result<HostedFilm, ExecError> {
    Ok(HostedFilm {
        url: format!("{server}{FILM_FOLDER}/index.html"),
        spec: serde_json::to_value(hosted(spec)).map_err(|e| ExecError::new(e.to_string()))?,
        width: spec.width,
        height: spec.height,
    })
}

impl HostedFilm {
    fn page(&self, scale: f64) -> FilmPage<'_> {
        FilmPage {
            url: &self.url,
            spec: &self.spec,
            width: self.width,
            height: self.height,
            scale,
        }
    }
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
    width: u32,
    height: u32,
    reframe: ReframeMode,
    project_base: &str,
) -> TimelineSpec {
    TimelineSpec {
        width,
        height,
        reframe: Some(reframe),
        assets: Some(SpecAssets {
            project_base: Some(project_base.to_owned()),
        }),
        ..spec.clone()
    }
}

struct VariantJob<'a> {
    ctx: &'a ExecutorContext,
    engine: &'a RenderEngine,
    server: &'a str,
    spec: TimelineSpec,
    p: &'a ExportVideo,
    progress: &'a (dyn Fn(f64) + Send + Sync),
}

async fn render_to(job: &VariantJob<'_>, output: &Path) -> Result<(), ExecError> {
    let p = job.p;
    let film = hosted_film(&job.spec, job.server)?;
    let render = FilmRender {
        driver: driver(job.ctx, p.gl)?,
        page: film.page(p.scale),
        encoding: FilmEncoding {
            codec: p.codec,
            fps: f64::from(job.spec.fps),
            png_frames: false,
            crf: codec_info(p.codec).crf.then_some(f64::from(p.crf)),
            x264_preset: p.x264_preset(),
            pixel_format: p.pixel_format(),
            prores: p.prores(),
            gif_loops: p.gif_loops(),
        },
        every_nth_frame: p.every_nth_frame().unwrap_or(1),
        output,
    };
    let note = |message: String| job.ctx.log(LogLevel::Info, message);
    let run = FilmRun {
        tools: &*job.engine.tools,
        cancel: &job.ctx.cancel,
        progress: job.progress,
        note: &note,
    };
    render_film(&render, &run).await.map_err(film_error)
}

async fn picture_for(job: &VariantJob<'_>) -> Result<PathBuf, ExecError> {
    let p = job.p;
    let opts = PictureOptions {
        codec: p.codec,
        crf: f64::from(p.crf),
        scale: p.scale,
        gl: p.gl,
        x264_preset: p.x264_preset(),
        pixel_format: p.pixel_format(),
        pro_res_profile: p.prores(),
        number_of_gif_loops: p.gif_loops(),
        every_nth_frame: p.every_nth_frame(),
    };
    let key = picture_key(&job.spec, &opts, &job.ctx.env.app_version);
    let name = format!("{key}.{}", codec_info(p.codec).ext);
    let picture = cached_picture(&job.ctx.project_dir, &name, |temp| async move {
        render_to(job, &temp).await
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
    list: Vec<FilmChapter>,
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
        film_chapters(spec, &p.first_chapter)
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
        list: chapters,
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
                    mux_args(
                        codec,
                        &shown(&picture),
                        &shown(&sound),
                        &shown(&temp),
                        job.p.sound_kbps,
                    ),
                )
            })
            .await?;
        }
        _ => write_via_temp(out, |temp| async move { render_to(job, &temp).await }).await?,
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
    let mut artifacts = vec![Artifact::labelled(
        artifact_kind(codec),
        project_path(ctx, out)?,
        variant.id(),
    )];
    if chapters.listed {
        let stem = out
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let list = out.with_file_name(format!("{stem}.youtube-chapters.txt"));
        tokio::fs::write(&list, chapter_list(&chapters.list)).await?;
        artifacts.push(Artifact::labelled(
            ArtifactKind::File,
            project_path(ctx, &list)?,
            format!("{} chapter list", variant.id()),
        ));
    }
    Ok(artifacts)
}

async fn asset_server(
    ctx: &ExecutorContext,
    engine: &RenderEngine,
    spec: &TimelineSpec,
) -> Result<FileServer, ExecError> {
    let dirs = PreparedDirs {
        project: ctx.project_dir.clone(),
        public: ctx.env.public_dir.clone(),
    };
    let footage = Arc::new(film_footage(ctx, spec)?);
    let routes = prepared_routes(dirs)
        .merge(footage_routes(footage))
        .merge(folder_routes(FILM_FOLDER, &engine.film_bundle));
    Ok(FileServer::start_with(&ctx.project_dir, routes).await?)
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
    let server = asset_server(&ctx, &engine, &spec).await?;
    let chapters = chapters_for(&ctx, &spec, &p).await?;
    let count = numbers::float(p.variants.len());
    let mut artifacts = Vec::new();
    for (i, (variant, name)) in p.variants.iter().zip(&names).enumerate() {
        let size = variant_size(*variant, &ctx.workflow.settings);
        let out = ctx.out_dir.join(name);
        ctx.log(
            LogLevel::Info,
            format!(
                "Exporting {} ({}×{}) as {name}",
                variant.id(),
                size.width,
                size.height
            ),
        );
        let message = format!("Rendering {}", variant.id());
        let progress = |x: f64| {
            ctx.progress(
                lerp(
                    RENDER_STAGE.0,
                    RENDER_STAGE.1,
                    (numbers::float(i) + x) / count,
                ),
                Some(&message),
            );
        };
        let job = VariantJob {
            ctx: &ctx,
            engine: &engine,
            server: &server.url,
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
        duration_ms: Some(whole(frames_to_ms(spec.duration_in_frames, spec.fps))),
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
        measured: Vec::new(),
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
    let server = asset_server(&ctx, &engine, &spec).await?;
    let count = numbers::float(p.presets.len());
    let mut artifacts = Vec::new();
    for (i, (preset, name)) in p.presets.iter().zip(&names).enumerate() {
        let size = preset.size();
        let out = ctx.out_dir.join(name);
        ctx.progress(
            lerp(RENDER_STAGE.0, RENDER_STAGE.1, numbers::float(i) / count),
            Some(&format!("Still {}", preset.id())),
        );
        let film = hosted_film(
            &sized(&spec, size.width, size.height, p.reframe, &server.url),
            &server.url,
        )?;
        render_still(&StillRender {
            driver: driver(&ctx, p.gl)?,
            page: film.page(p.scale),
            frame: ms_to_frames_from(f64::from(p.at_ms), spec.fps, 0),
            format: p.format,
            jpeg_quality: (p.format == StillFormat::Jpeg).then_some(f64::from(p.jpeg_quality)),
            output: &out,
        })
        .await
        .map_err(film_error)?;
        artifacts.push(Artifact::labelled(
            ArtifactKind::Image,
            project_path(&ctx, &out)?,
            preset.id(),
        ));
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
        measured: Vec::new(),
    })
}

fn bind<F, Fut>(engine: &Arc<RenderEngine>, run: F) -> Executor
where
    F: Fn(Arc<ExecutorContext>, PortValues, Arc<RenderEngine>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ExecutorOutput, ExecError>> + Send + 'static,
{
    let engine = Arc::clone(engine);
    executor(move |ctx, inputs| run(ctx, inputs, Arc::clone(&engine)))
}

pub fn output_executors(engine: &Arc<RenderEngine>) -> Vec<(&'static str, Executor)> {
    vec![
        (RENDER_TYPE, bind(engine, export_video)),
        (STILL_TYPE, bind(engine, export_images)),
    ]
}
