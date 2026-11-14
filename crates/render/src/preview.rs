use std::fmt;
use std::sync::Arc;

use opsis_compile::hosted::hosted;
use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::Gl;
use opsis_compile::terminal::preview::{PREVIEW_FILE, preview_spec};
use opsis_engine::files::write_via_temp;
use opsis_media::codecs::FilmEncoding;
use opsis_runner::context::{BoxFuture, ExecError, ExecutorContext};
use opsis_terminal::executors::TakePreview;
use opsis_types::events::LogLevel;
use opsis_types::timeline::{SpecAssets, TimelineSpec};
use opsis_types::values::RecordingValue;

use crate::executors::{RenderEngine, film_error, native_files};
use crate::frames::FilmRun;
use crate::native::{NATIVE_BASE, NativeRender, render_native};

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
    let spec = hosted(&TimelineSpec {
        assets: Some(SpecAssets {
            project_base: Some(NATIVE_BASE.to_owned()),
        }),
        ..spec
    });
    let files = native_files(ctx, &spec, Gl::Card)?;
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
        crf: Some(PREVIEW_CRF),
        x264_preset: None,
        pixel_format: None,
        prores: None,
        gif_loops: None,
    };
    write_via_temp(&ctx.dir.join(PREVIEW_FILE), |temp| async move {
        let render = NativeRender {
            spec,
            files,
            encoding,
            every_nth_frame: 1,
            scale: 1.0,
            output: temp,
        };
        render_native(render, &run).await.map_err(film_error)
    })
    .await?;
    Ok(ctx.project_file_ref(PREVIEW_FILE))
}
