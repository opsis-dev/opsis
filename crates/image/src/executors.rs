use opsis_compile::nodes::image::{
    ADJUST_TYPE, BEFORE_AFTER_TYPE, BLUR_TYPE, CLEAR_COLOUR_TYPE, CROP_TYPE, FORMAT_TYPE,
    FRAME_TYPE, GRID_TYPE, LEVELS_TYPE, OUTLINE_TYPE, PLACE_TYPE, REDACT_TYPE, RESIZE_TYPE,
    ROTATE_TYPE, SPOTLIGHT_TYPE, TILT_TYPE, TRIM_TYPE,
};
use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use opsis_compile::nodes::image::{
    AFTER_PORT, AdjustImage, AutoLevels, BeforeAfter, BlurImage, ClearColour, CropImage,
    FrameScreenshot, IMAGE_PORT, ImageFormat, ImageGrid, OVERLAY_PORT, OutlineImage, PlaceImage,
    RedactImage, ResizeImage, RotateImage, SOFT_UPSCALE, SpotlightImage, TiltImage, TrimEdges,
    resized_size, upscale,
};
use opsis_engine::files::atomic_write;
use opsis_media::executors::require_media;
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::values::{Artifact, ArtifactKind, CompileValue, ImageValue, PortValues};

use crate::load::load_image;
use crate::ops;
use crate::text::load_font;
use opsis_compile::elements::prepared::face_for;
use opsis_compile::fonts::family_for;
use opsis_compile::gradient::parse_gradient;
use opsis_compile::nodes::camera::BackdropKind;
use opsis_types::fonts::FontRole;

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
        measured: Vec::new(),
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

async fn levels(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: AutoLevels = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || Ok(ops::auto_levels(&image, &p))).await?;
    saved(&ctx, &out, None)
}

async fn grid(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: ImageGrid = ctx.params()?;
    let mut ports: Vec<(usize, &String)> = inputs
        .keys()
        .filter_map(|port| Some((opsis_compile::ports::in_index(port)?, port)))
        .collect();
    ports.sort_by_key(|(i, _)| *i);
    if ports.len() < 2 {
        return Err(ExecError::new("Connect at least two images."));
    }
    let mut images = Vec::with_capacity(ports.len());
    for (_, port) in ports {
        images.push(input(&ctx, &inputs, port).await?);
    }
    let out = blocking(move || ops::grid(&images, &p)).await?;
    saved(&ctx, &out, None)
}

async fn picture_at(ctx: &ExecutorContext, reference: &str) -> Result<DynamicImage, ExecError> {
    let path = ctx.ref_to_path(reference)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| ExecError::new(format!("Couldn't read {}: {e}", path.display())))?;
    blocking(move || load_image(&bytes)).await
}

async fn paint_of(ctx: &ExecutorContext, p: &FrameScreenshot) -> Result<ops::Paint, ExecError> {
    let background = &p.background;
    Ok(match background.kind {
        BackdropKind::Image => ops::Paint::Picture(
            picture_at(ctx, &format!("app:backdrops/{}.jpg", background.name.id())).await?,
        ),
        BackdropKind::Asset => ops::Paint::Picture(picture_at(ctx, &background.asset).await?),
        BackdropKind::Solid => ops::Paint::Colour(ops::background_rgba(&background.color)),
        BackdropKind::Gradient => ops::Paint::Gradient(
            parse_gradient(&background.gradient)
                .ok_or_else(|| ExecError::new("The gradient can't be read. Pick it again."))?,
        ),
    })
}

async fn frame(ctx: Arc<ExecutorContext>, inputs: PortValues) -> Result<ExecutorOutput, ExecError> {
    let p: FrameScreenshot = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let paint = paint_of(&ctx, &p).await?;
    let out = blocking(move || ops::frame(&image, &paint, &p)).await?;
    saved(&ctx, &out, None)
}

async fn label_font(
    ctx: &ExecutorContext,
    p: &BeforeAfter,
) -> Result<Option<ab_glyph::FontVec>, ExecError> {
    if !p.labels {
        return Ok(None);
    }
    let name = match p.font.trim() {
        "" => FontRole::Main.family(),
        name => name,
    };
    let weight = f64::from(p.label_weight);
    let face = family_for(name, &ctx.workflow.settings.fonts)
        .and_then(|def| face_for(def, weight, false))
        .ok_or_else(|| {
            ExecError::new(format!(
                "The font “{name}” isn't in this project. Pick another."
            ))
        })?;
    let path = ctx.ref_to_path(&face.file)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| ExecError::new(format!("Couldn't read {}: {e}", path.display())))?;
    blocking(move || load_font(bytes, weight)).await.map(Some)
}

async fn before_after(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: BeforeAfter = ctx.params()?;
    let before = input(&ctx, &inputs, IMAGE_PORT).await?;
    let after = input(&ctx, &inputs, AFTER_PORT).await?;
    let font = label_font(&ctx, &p).await?;
    let out = blocking(move || ops::before_after(&before, &after, font.as_ref(), &p)).await?;
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
        (BEFORE_AFTER_TYPE, executor(before_after)),
        (FORMAT_TYPE, executor(format)),
        (REDACT_TYPE, executor(redact)),
        (SPOTLIGHT_TYPE, executor(spotlight)),
        (TRIM_TYPE, executor(trim)),
        (CLEAR_COLOUR_TYPE, executor(clear_colour)),
        (OUTLINE_TYPE, executor(outline)),
        (TILT_TYPE, executor(tilt)),
        (LEVELS_TYPE, executor(levels)),
        (GRID_TYPE, executor(grid)),
        (FRAME_TYPE, executor(frame)),
    ]
}
