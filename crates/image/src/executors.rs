use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use opsis_compile::elements::prepared::face_for;
use opsis_compile::fonts::family_for;
use opsis_compile::gradient::parse_gradient;
use opsis_compile::nodes::camera::BackdropKind;
use opsis_compile::nodes::image::{
    ADJUST_TYPE, AFTER_PORT, ANNOTATE_TYPE, AdjustImage, Annotate, AutoLevels, BEFORE_AFTER_TYPE,
    BLUR_TYPE, BeforeAfter, BlurImage, CLEAR_COLOUR_TYPE, COMPARE_TYPE, CROP_TYPE, ClearColour,
    CompareScreenshots, CropImage, DIFFERENCES_PORT, FORMAT_TYPE, FRAME_TYPE, FrameScreenshot,
    GRID_TYPE, IMAGE_PORT, ImageFormat, ImageGrid, LEVELS_TYPE, MAGNIFY_TYPE, Magnify,
    OUTLINE_TYPE, OVERLAY_PORT, OutlineImage, PLACE_TYPE, PlaceImage, REDACT_TYPE, RESIZE_TYPE,
    ROTATE_TYPE, RedactImage, ResizeImage, RotateImage, SOFT_UPSCALE, SPOTLIGHT_TYPE,
    SpotlightImage, TILT_TYPE, TRIM_TYPE, TiltImage, TrimEdges, resized_size, upscale,
};
use opsis_engine::files::atomic_write;
use opsis_media::executors::require_media;
use opsis_runner::context::{ExecError, Executor, ExecutorContext, ExecutorOutput, executor};
use opsis_types::events::LogLevel;
use opsis_types::fonts::FontRole;
use opsis_types::values::{Artifact, ArtifactKind, CompileValue, ImageValue, Measured, PortValues};

use crate::load::load_image;
use crate::ops;
use crate::text::load_font;

fn exec_error(error: &crate::ImageError) -> ExecError {
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
        .map_err(|e| exec_error(&e))
}

fn saved(
    ctx: &ExecutorContext,
    image: &DynamicImage,
    format: Option<&ImageFormat>,
) -> Result<ExecutorOutput, ExecError> {
    let extension = format.map_or("png", |f| f.format.extension());
    let file = format!("image.{extension}");
    let bytes = ops::encode(image, format).map_err(|e| exec_error(&e))?;
    write(&ctx.dir.join(&file), &bytes)?;
    let value = ImageValue {
        src: Some(ctx.project_file_ref(&file)),
        width: Some(image.width()),
        height: Some(image.height()),
        scale: None,
        body: None,
    };
    Ok(ExecutorOutput {
        outputs: PortValues::from([(IMAGE_PORT.to_owned(), CompileValue::Image(value))]),
        artifacts: vec![ctx.artifact(ArtifactKind::Image, &file, "Image")],
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
        .map_err(|e| exec_error(&e))
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
    font_named(ctx, &p.font, p.label_weight).await.map(Some)
}

async fn font_named(
    ctx: &ExecutorContext,
    font: &str,
    weight: u32,
) -> Result<ab_glyph::FontVec, ExecError> {
    let name = match font.trim() {
        "" => FontRole::Main.family(),
        name => name,
    };
    let weight = f64::from(weight);
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
    blocking(move || load_font(bytes, weight)).await
}

async fn annotate(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: Annotate = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let mut fonts = crate::marks::MarkFonts::new();
    for mark in p.marks.iter().filter(|m| m.kind.has_text()) {
        if let std::collections::hash_map::Entry::Vacant(slot) =
            fonts.entry((mark.font.clone(), mark.weight))
        {
            slot.insert(font_named(&ctx, &mark.font, mark.weight).await?);
        }
    }
    let out = blocking(move || {
        Ok(crate::marks::annotate(
            &image,
            &p.marks,
            p.first_step,
            &fonts,
        ))
    })
    .await?;
    saved(&ctx, &out, None)
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

async fn magnify(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: Magnify = ctx.params()?;
    let image = input(&ctx, &inputs, IMAGE_PORT).await?;
    let out = blocking(move || ops::magnify(&image, &p)).await?;
    saved(&ctx, &out, None)
}

fn saved_png(
    ctx: &ExecutorContext,
    image: &DynamicImage,
    file: &str,
    label: &str,
) -> Result<(CompileValue, Artifact), ExecError> {
    let bytes = ops::encode(image, None).map_err(|e| exec_error(&e))?;
    write(&ctx.dir.join(file), &bytes)?;
    let value = CompileValue::Image(ImageValue {
        src: Some(ctx.project_file_ref(file)),
        width: Some(image.width()),
        height: Some(image.height()),
        scale: None,
        body: None,
    });
    let artifact = ctx.artifact(ArtifactKind::Image, file, label);
    Ok((value, artifact))
}

async fn compare(
    ctx: Arc<ExecutorContext>,
    inputs: PortValues,
) -> Result<ExecutorOutput, ExecError> {
    let p: CompareScreenshots = ctx.params()?;
    let before = input(&ctx, &inputs, IMAGE_PORT).await?;
    let after = input(&ctx, &inputs, AFTER_PORT).await?;
    if (before.width(), before.height()) != (after.width(), after.height()) {
        return Err(ExecError::new(format!(
            "The two pictures are different sizes ({} × {} and {} × {}). Compare two captures of the same size.",
            before.width(),
            before.height(),
            after.width(),
            after.height()
        )));
    }
    let found = blocking(move || ops::compare(&before, &after, &p)).await?;
    let count = u32::try_from(found.boxes.len()).unwrap_or(u32::MAX);
    ctx.log(
        LogLevel::Info,
        match count {
            0 => "No change found.".to_owned(),
            1 => "Found 1 change.".to_owned(),
            n => format!("Found {n} changes."),
        },
    );
    let (marked, marked_file) = saved_png(&ctx, &found.marked, "marked.png", "Marked")?;
    let (differences, differences_file) =
        saved_png(&ctx, &found.differences, "differences.png", "Differences")?;
    Ok(ExecutorOutput {
        outputs: [
            (IMAGE_PORT.to_owned(), marked),
            (DIFFERENCES_PORT.to_owned(), differences),
        ]
        .into_iter()
        .collect(),
        artifacts: vec![marked_file, differences_file],
        trace_id: None,
        measured: vec![Measured::Changes { count }],
    })
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
        (MAGNIFY_TYPE, executor(magnify)),
        (COMPARE_TYPE, executor(compare)),
        (ANNOTATE_TYPE, executor(annotate)),
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
