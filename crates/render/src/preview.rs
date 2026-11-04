use std::fmt;
use std::sync::Arc;

use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::Gl;
use opsis_compile::terminal::preview::{PREVIEW_FILE, preview_spec};
use opsis_engine::files::write_via_temp;
use opsis_media::codecs::{FilmEncoding, finish_args};
use opsis_runner::context::{BoxFuture, ExecError, ExecutorContext};
use opsis_terminal::executors::TakePreview;
use opsis_types::events::LogLevel;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::RecordingValue;

use crate::executors::{RenderEngine, asset_server, driver, film_error, hosted_film, run_tool};
use crate::frames::{FilmRender, FilmRun, render_film};

const PREVIEW_CRF: f64 = 23.0;
const DRAWING: &str = "Drawing the preview";

pub struct TakePreviews {
    engine: Arc<RenderEngine>,
}

impl TakePreviews {
    pub const fn new(engine: Arc<RenderEngine>) -> Self {
        Self { engine }
    }
}

impl fmt::Debug for TakePreviews {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakePreviews")
            .field("engine", &self.engine)
            .finish()
    }
}

impl TakePreview for TakePreviews {
    fn render(
        &self,
        ctx: Arc<ExecutorContext>,
        take: RecordingValue,
    ) -> BoxFuture<Result<String, ExecError>> {
        let engine = Arc::clone(&self.engine);
        Box::pin(async move { render_preview(&ctx, &engine, &take).await })
    }
}

pub async fn render_preview(
    ctx: &ExecutorContext,
    engine: &RenderEngine,
    take: &RecordingValue,
) -> Result<String, ExecError> {
    let spec = preview_spec(take)
        .map_err(|e| ExecError::new(e.to_string()))?
        .ok_or_else(|| ExecError::new("The take has nothing to show."))?;
    ctx.progress(0.0, Some(DRAWING));
    let server = asset_server(ctx, engine, &spec).await?;
    let spec = TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(server.url.clone()),
        }),
        ..spec
    };
    let film = hosted_film(&spec, &server.url)?;
    let progress = |x: f64| ctx.progress(x, Some(DRAWING));
    let note = |message: String| ctx.log(LogLevel::Info, message);
    let run = FilmRun {
        tools: &*engine.tools,
        cancel: &ctx.cancel,
        progress: &progress,
        note: &note,
    };
    let encoding = FilmEncoding {
        codec: Codec::H264,
        fps: f64::from(spec.fps),
        png_frames: false,
        crf: Some(PREVIEW_CRF),
        x264_preset: None,
        pixel_format: None,
        prores: None,
        gif_loops: None,
    };
    let output = ctx.dir.join(PREVIEW_FILE);
    write_via_temp(&output, |temp| async move {
        let render = FilmRender {
            driver: driver(ctx, Gl::Angle)?,
            page: film.page(1.0),
            encoding,
            every_nth_frame: 1,
            output: &temp,
        };
        render_film(&render, &run).await.map_err(film_error)
    })
    .await?;
    let rendered = output.to_string_lossy().into_owned();
    write_via_temp(&output, |temp| {
        run_tool(
            ctx,
            engine,
            finish_args(&rendered, &temp.to_string_lossy(), true, None),
        )
    })
    .await?;
    Ok(ctx.project_file_ref(PREVIEW_FILE))
}
