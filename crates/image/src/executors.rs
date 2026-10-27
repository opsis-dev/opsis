use opsis_compile::nodes::image::{
    ADJUST_TYPE, BLUR_TYPE, CLEAR_COLOUR_TYPE, CROP_TYPE, FORMAT_TYPE, OUTLINE_TYPE, PLACE_TYPE,
    REDACT_TYPE, RESIZE_TYPE, ROTATE_TYPE, SPOTLIGHT_TYPE, TILT_TYPE, TRIM_TYPE,
};
use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use opsis_compile::nodes::image::{
    AdjustImage, BlurImage, ClearColour, CropImage, IMAGE_PORT, ImageFormat, OVERLAY_PORT,
    OutlineImage, PlaceImage, RedactImage, ResizeImage, RotateImage, SOFT_UPSCALE, SpotlightImage,
    TiltImage, TrimEdges, resized_size, upscale,
};
use opsis_engine::files::atomic_write;
use opsis_media::executors::require_media;
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::values::{Artifact, ArtifactKind, CompileValue, ImageValue, PortValues};

use crate::load::load_image;
use crate::ops;

fn exec_error(error: crate::ImageError) -> ExecError {
    ExecError::new(error.to_string())
}

async fn input(
    ctx: &ExecutorContext,
    inputs: &PortValues,
    port: &str,
) -> Result<DynamicImage, ExecError> {
    let media = require_media(inputs.get(port))?;
    let path = ctx.ref_to_path(&media.src)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| ExecError::new(format!("Couldn't read {}: {e}", path.display())))?;
    tokio::task::spawn_blocking(move || load_image(&bytes))
        .await
        .map_err(|e| ExecError::new(e.to_string()))?
        .map_err(exec_error)
}

fn saved(
    ctx: &ExecutorContext,
    image: &DynamicImage,
    format: Option<&ImageFormat>,
) -> Result<ExecutorOutput, ExecError> {
    let extension = format.map_or("png", |f| f.format.extension());
    let file = format!("image.{extension}");
    let bytes = ops::encode(image, format).map_err(exec_error)?;
    write(&ctx.dir.join(&file), &bytes)?;
    let value = ImageValue {
        src: Some(ctx.project_file_ref(&file)),
        width: Some(image.width()),
        height: Some(image.height()),
        scale: None,
        body: None,
    };
    Ok(ExecutorOutput {
        outputs: [(IMAGE_PORT.to_owned(), CompileValue::Image(value))]
            .into_iter()
            .collect(),
        artifacts: vec![Artifact {
            kind: ArtifactKind::Image,
            path: ctx.rel(&file),
            label: Some("Image".to_owned()),
            bytes: None,
            stamp: None,
        }],
        trace_id: None,
    })
}

fn write(file: &Path, bytes: &[u8]) -> Result<(), ExecError> {
    let saved = match file.parent() {
        Some(folder) => std::fs::create_dir_all(folder).and_then(|()| atomic_write(file, bytes)),
        None => atomic_write(file, bytes),
    };
    saved.map_err(|e| ExecError::new(format!("Couldn't save the image: {e}")))
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, crate::ImageError> + Send + 'static,
) -> Result<T, ExecError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| ExecError::new(e.to_string()))?
        .map_err(exec_error)
}

async fn resize(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: ResizeImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let source = (f64::from(image.width()), f64::from(image.height()));
    let factor = upscale(source, resized_size(source, &p));
    if factor > SOFT_UPSCALE {
        ctx.log(
            LogLevel::Warn,
            format!("Enlarging {factor:.1}×, so the image may look soft."),
        );
    }
    let out = blocking(move || ops::resize(&image, &p)).await?;
    saved(&ctx, &out, None)
}

async fn crop(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: CropImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::crop(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn rotate(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: RotateImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::rotate(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn adjust(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: AdjustImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::adjust(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn blur(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: BlurImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::blur(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn place(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: PlaceImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let overlay = input(&ctx, &inputs, OVERLAY_PORT).await?;
    let out = blocking(move || ops::place(&image, &overlay, &p)).await?;
    saved(&ctx, &out, None)
}

async fn format(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: ImageFormat = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    saved(&ctx, &image, Some(&p))
}

async fn redact(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: RedactImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::redact(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn spotlight(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: SpotlightImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::spotlight(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn trim(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: TrimEdges = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::trim(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn clear_colour(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: ClearColour = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::clear_colour(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn outline(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: OutlineImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    if !ops::has_see_through(&image) {
        ctx.log(
            LogLevel::Warn,
            "The image has no see-through parts, so the outline only goes around its edges."
                .to_owned(),
        );
    }
    let out = blocking(move || Ok(ops::outline(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn tilt(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: TiltImage = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || ops::tilt(&image, &p)).await?;
    saved(&ctx, &out, None)
}

pub fn image_executors() -> Vec<(&'static str, Executor)> {
    vec![
        (RESIZE_TYPE, executor(resize)),
        (CROP_TYPE, executor(crop)),
        (ROTATE_TYPE, executor(rotate)),
        (ADJUST_TYPE, executor(adjust)),
        (BLUR_TYPE, executor(blur)),
        (PLACE_TYPE, executor(place)),
        (FORMAT_TYPE, executor(format)),
        (REDACT_TYPE, executor(redact)),
        (SPOTLIGHT_TYPE, executor(spotlight)),
        (TRIM_TYPE, executor(trim)),
        (CLEAR_COLOUR_TYPE, executor(clear_colour)),
        (OUTLINE_TYPE, executor(outline)),
        (TILT_TYPE, executor(tilt)),
    ]
}
