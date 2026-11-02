use std::io::Cursor;

use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{
    DynamicImage, GrayImage, ImageBuffer, Luma, Pixel, Primitive, Rgba, Rgba32FImage, RgbaImage,
    imageops,
};
use imageproc::distance_transform::{Norm, euclidean_squared_distance_transform};
use imageproc::filter::gaussian_blur_f32;
use imageproc::geometric_transformations::{
    Border, Interpolation, Projection, rotate_about_center, warp_into,
};
use imageproc::morphology::dilate;
use imageproc::region_labelling::{Connectivity, connected_components};
use num_traits::ToPrimitive;
use opsis_compile::gradient::{Gradient, GradientKind};
use opsis_compile::nodes::image::{
    AdjustImage, AutoLevels, BeforeAfter, BeforeAfterLayout, BeforeAfterPlan, BlurImage,
    ClearColour, CompareScreenshots, CropImage, FrameScreenshot, ImageFormat, ImageGrid, LabelEdge,
    LevelsMethod, Magnify, MagnifyShape, OutlineImage, PlaceImage, Rect, RedactImage, RedactStyle,
    Resample, ResizeBy, ResizeFit, ResizeImage, RotateImage, SpotlightImage, StillFileFormat,
    TiltImage, TrimEdges, Turn, area_rect, before_after_plan, compare_threshold, crop_rect,
    framed_placement, grid_plan, label_size, magnify_plan, outline_padding, resized_size,
    rotated_size, tilt_corners, tilted_canvas, tilted_size,
};
use opsis_types::numbers;
use opsis_types::numbers::whole;

use crate::ImageError;
use crate::text::{draw_text, measure};

fn failed<E: std::fmt::Display>(error: E) -> ImageError {
    ImageError::Failed(error.to_string())
}

pub(crate) fn is_deep(image: &DynamicImage) -> bool {
    image.color().bytes_per_pixel() / image.color().channel_count() > 1
}

const fn algorithm(quality: Resample) -> ResizeAlg {
    match quality {
        Resample::Fast => ResizeAlg::Convolution(FilterType::Bilinear),
        Resample::Smooth => ResizeAlg::Convolution(FilterType::CatmullRom),
        Resample::Sharp => ResizeAlg::Convolution(FilterType::Lanczos3),
    }
}

fn resized_to(
    image: &DynamicImage,
    width: u32,
    height: u32,
    options: &ResizeOptions,
) -> Result<DynamicImage, ImageError> {
    let mut out = DynamicImage::new(width.max(1), height.max(1), image.color());
    Resizer::new()
        .resize(image, &mut out, options)
        .map_err(failed)?;
    Ok(out)
}

fn size_of(image: &DynamicImage) -> (f64, f64) {
    (f64::from(image.width()), f64::from(image.height()))
}

pub fn resize(image: &DynamicImage, p: &ResizeImage) -> Result<DynamicImage, ImageError> {
    let (w, h) = resized_size(size_of(image), p);
    let mut options = ResizeOptions::new().resize_alg(algorithm(p.quality));
    if p.by == ResizeBy::Pixels && p.width > 0 && p.height > 0 && p.fit == ResizeFit::Cover {
        options = options.fit_into_destination(Some((p.crop_at_x, p.crop_at_y)));
    }
    resized_to(
        image,
        numbers::truncate::<u32>(w),
        numbers::truncate::<u32>(h),
        &options,
    )
}

pub fn crop(image: &DynamicImage, p: &CropImage) -> DynamicImage {
    let r = crop_rect(size_of(image), p);
    image.crop_imm(
        numbers::truncate::<u32>(r.x),
        numbers::truncate::<u32>(r.y),
        numbers::truncate::<u32>(r.width),
        numbers::truncate::<u32>(r.height),
    )
}

fn tilted<P>(
    image: &ImageBuffer<P, Vec<P::Subpixel>>,
    size: (f64, f64),
    angle: f64,
    background: P,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + Send + Sync,
    P::Subpixel: Send + Sync + Into<f32> + imageproc::definitions::Clamp<f32>,
{
    let (w, h) = (
        numbers::truncate::<u32>(size.0),
        numbers::truncate::<u32>(size.1),
    );
    let mut canvas = ImageBuffer::from_pixel(w, h, background);
    let x = i64::from(w.saturating_sub(image.width()) / 2);
    let y = i64::from(h.saturating_sub(image.height()) / 2);
    imageops::replace(&mut canvas, image, x, y);
    rotate_about_center(
        &canvas,
        numbers::narrow(angle).to_radians(),
        Interpolation::Bicubic,
        Border::Constant(background),
    )
}

pub fn background_rgba(color: &str) -> [u8; 4] {
    csscolorparser::parse(color.trim()).map_or([0, 0, 0, 0], |c| c.to_rgba8())
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn rotate(image: &DynamicImage, p: &RotateImage) -> DynamicImage {
    let mut out = match p.turn {
        Turn::None => image.clone(),
        Turn::Right => image.rotate90(),
        Turn::Half => image.rotate180(),
        Turn::Left => image.rotate270(),
    };
    if p.flip_horizontal {
        out = out.fliph();
    }
    if p.flip_vertical {
        out = out.flipv();
    }
    if p.angle == 0.0 {
        return out;
    }
    let size = tilted_canvas(size_of(image), p);
    let [r, g, b, a] = background_rgba(&p.background);
    let turned = if is_deep(&out) {
        let wide = |v: u8| u16::from(v) * 257;
        let background = Rgba([wide(r), wide(g), wide(b), wide(a)]);
        DynamicImage::ImageRgba16(tilted(&out.to_rgba16(), size, p.angle, background))
    } else {
        DynamicImage::ImageRgba8(tilted(&out.to_rgba8(), size, p.angle, Rgba([r, g, b, a])))
    };
    if !p.crop_corners {
        return turned;
    }
    let (w, h) = rotated_size(size_of(image), p);
    let (cw, ch) = size_of(&turned);
    turned.crop_imm(
        numbers::truncate::<u32>(((cw - w) / 2.0).max(0.0)),
        numbers::truncate::<u32>(((ch - h) / 2.0).max(0.0)),
        numbers::truncate::<u32>(w),
        numbers::truncate::<u32>(h),
    )
}

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BRIGHTNESS_SPAN: f32 = 0.4;
const CONTRAST_GAIN: f32 = 3.0;

fn hue_matrix(degrees: f32) -> [[f32; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        [
            f32::mul_add(sin, -0.213, f32::mul_add(cos, 0.787, 0.213)),
            f32::mul_add(sin, -0.715, f32::mul_add(cos, -0.715, 0.715)),
            f32::mul_add(sin, 0.928, f32::mul_add(cos, -0.072, 0.072)),
        ],
        [
            f32::mul_add(sin, 0.143, f32::mul_add(cos, -0.213, 0.213)),
            f32::mul_add(sin, 0.140, f32::mul_add(cos, 0.285, 0.715)),
            f32::mul_add(sin, -0.283, f32::mul_add(cos, -0.072, 0.072)),
        ],
        [
            f32::mul_add(sin, -0.787, f32::mul_add(cos, -0.213, 0.213)),
            f32::mul_add(sin, 0.715, f32::mul_add(cos, -0.715, 0.715)),
            f32::mul_add(sin, 0.072, f32::mul_add(cos, 0.928, 0.072)),
        ],
    ]
}

fn adjust_pixel(rgb: [f32; 3], p: &AdjustImage, hue: &[[f32; 3]; 3]) -> [f32; 3] {
    let contrast = if p.contrast >= 0.0 {
        numbers::narrow(p.contrast).mul_add(CONTRAST_GAIN, 1.0)
    } else {
        1.0 + numbers::narrow(p.contrast)
    };
    let mut c = rgb.map(|v| {
        numbers::narrow(p.brightness).mul_add(BRIGHTNESS_SPAN, f32::mul_add(v - 0.5, contrast, 0.5))
    });
    if p.hue != 0.0 {
        c = [0, 1, 2].map(|i| hue[i][2].mul_add(c[2], hue[i][1].mul_add(c[1], hue[i][0] * c[0])));
    }
    let luma = LUMA[2].mul_add(c[2], LUMA[1].mul_add(c[1], LUMA[0] * c[0]));
    let saturation = if p.grayscale {
        0.0
    } else {
        1.0 + numbers::narrow(p.saturation)
    };
    c = c.map(|v| (v - luma).mul_add(saturation, luma));
    if p.invert {
        c = c.map(|v| 1.0 - v);
    }
    c.map(|v| v.clamp(0.0, 1.0))
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn adjust(image: &DynamicImage, p: &AdjustImage) -> DynamicImage {
    let hue = hue_matrix(numbers::narrow(p.hue));
    let deep = is_deep(image);
    let mut pixels = image.to_rgba32f();
    for pixel in pixels.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let [r, g, b] = adjust_pixel([r, g, b], p, &hue);
        pixel.0 = [r, g, b, a];
    }
    let adjusted = DynamicImage::ImageRgba32F(pixels);
    match (deep, image.color().has_alpha()) {
        (true, _) => DynamicImage::ImageRgba16(adjusted.to_rgba16()),
        (false, true) => DynamicImage::ImageRgba8(adjusted.to_rgba8()),
        (false, false) => DynamicImage::ImageRgb8(adjusted.to_rgb8()),
    }
}

pub fn blur(image: &DynamicImage, p: &BlurImage) -> DynamicImage {
    let mut out = if p.blur > 0.0 {
        image.fast_blur(numbers::narrow(p.blur))
    } else {
        image.clone()
    };
    if p.sharpen > 0.0 && p.sharpen_amount > 0 {
        let sharpened = out.unsharpen(numbers::narrow(p.sharpen), numbers::signed(p.threshold));
        out = if p.sharpen_amount == 100 {
            sharpened
        } else {
            toward(
                &out,
                &sharpened,
                numbers::narrow(f64::from(p.sharpen_amount) / 100.0),
            )
        };
    }
    out
}

fn toward(base: &DynamicImage, target: &DynamicImage, amount: f32) -> DynamicImage {
    let mut pixels = base.to_rgba32f();
    let goal = target.to_rgba32f();
    for (pixel, aim) in pixels.pixels_mut().zip(goal.pixels()) {
        for (v, t) in pixel.0.iter_mut().zip(aim.0) {
            *v = f32::mul_add(t - *v, amount, *v).clamp(0.0, 1.0);
        }
    }
    let mixed = DynamicImage::ImageRgba32F(pixels);
    match (is_deep(base), base.color().has_alpha()) {
        (true, _) => DynamicImage::ImageRgba16(mixed.to_rgba16()),
        (false, true) => DynamicImage::ImageRgba8(mixed.to_rgba8()),
        (false, false) => DynamicImage::ImageRgb8(mixed.to_rgb8()),
    }
}

pub fn place(
    image: &DynamicImage,
    overlay: &DynamicImage,
    p: &PlaceImage,
) -> Result<DynamicImage, ImageError> {
    let width = (f64::from(image.width()) * p.size / 100.0).round().max(1.0);
    let height = (width * f64::from(overlay.height()) / f64::from(overlay.width()))
        .round()
        .max(1.0);
    let options = ResizeOptions::new().resize_alg(algorithm(Resample::Sharp));
    let sized = resized_to(
        &DynamicImage::ImageRgba8(overlay.to_rgba8()),
        numbers::truncate::<u32>(width),
        numbers::truncate::<u32>(height),
        &options,
    )?;
    let mut top: RgbaImage = sized.to_rgba8();
    let opacity = p.opacity.clamp(0.0, 1.0);
    for pixel in top.pixels_mut() {
        pixel.0[3] = numbers::truncate::<u8>((f64::from(pixel.0[3]) * opacity).round());
    }
    let x = numbers::truncate::<i64>(
        f64::mul_add(f64::from(image.width()), p.x, -(width / 2.0)).round(),
    );
    let y = numbers::truncate::<i64>(
        f64::mul_add(f64::from(image.height()), p.y, -(height / 2.0)).round(),
    );
    let mut base = image.to_rgba8();
    imageops::overlay(&mut base, &top, x, y);
    Ok(DynamicImage::ImageRgba8(base))
}

fn flattened(image: &DynamicImage, color: &str) -> DynamicImage {
    if !image.color().has_alpha() {
        return DynamicImage::ImageRgb8(image.to_rgb8());
    }
    let [r, g, b, _] = background_rgba(color);
    let mut base = RgbaImage::from_pixel(image.width(), image.height(), Rgba([r, g, b, 255]));
    imageops::overlay(&mut base, &image.to_rgba8(), 0, 0);
    DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(base).to_rgb8())
}

pub fn encode(image: &DynamicImage, p: Option<&ImageFormat>) -> Result<Vec<u8>, ImageError> {
    let mut out = Cursor::new(Vec::new());
    let format = p.map_or(StillFileFormat::Png, |p| p.format);
    match format {
        StillFileFormat::Png => image
            .write_with_encoder(PngEncoder::new(&mut out))
            .map_err(failed)?,
        StillFileFormat::Jpeg => {
            let p = p.ok_or_else(|| ImageError::Failed("no settings".into()))?;
            let quality = p.quality.clamp(1, 100) as u8;
            flattened(image, &p.background)
                .write_with_encoder(JpegEncoder::new_with_quality(&mut out, quality))
                .map_err(failed)?;
        }
        StillFileFormat::Webp => {
            let pixels = if image.color().has_alpha() {
                DynamicImage::ImageRgba8(image.to_rgba8())
            } else {
                DynamicImage::ImageRgb8(image.to_rgb8())
            };
            pixels
                .write_with_encoder(WebPEncoder::new_lossless(&mut out))
                .map_err(failed)?;
        }
        StillFileFormat::Avif => {
            let p = p.ok_or_else(|| ImageError::Failed("no settings".into()))?;
            let encoder = AvifEncoder::new_with_speed_quality(
                &mut out,
                p.speed.clamp(1, 10) as u8,
                p.quality.clamp(1, 100) as u8,
            );
            DynamicImage::ImageRgba8(image.to_rgba8())
                .write_with_encoder(encoder)
                .map_err(failed)?;
        }
    }
    Ok(out.into_inner())
}

type Picture<P> = ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>;

fn in_depth(image: &DynamicImage, deep: Picture<Rgba<u16>>, shallow: RgbaImage) -> DynamicImage {
    if is_deep(image) {
        DynamicImage::ImageRgba16(deep)
    } else if image.color().has_alpha() {
        DynamicImage::ImageRgba8(shallow)
    } else {
        DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(shallow).to_rgb8())
    }
}

fn on_each_depth(
    image: &DynamicImage,
    deep: impl FnOnce(Picture<Rgba<u16>>) -> Picture<Rgba<u16>>,
    shallow: impl FnOnce(RgbaImage) -> RgbaImage,
) -> DynamicImage {
    if is_deep(image) {
        in_depth(image, deep(image.to_rgba16()), RgbaImage::new(0, 0))
    } else {
        in_depth(image, ImageBuffer::new(0, 0), shallow(image.to_rgba8()))
    }
}

fn channel<S: Primitive>(value: f64) -> S {
    num_traits::NumCast::from(value.round()).unwrap_or(S::DEFAULT_MAX_VALUE)
}

fn colour_in<P: Pixel>(rgba: [u8; 4]) -> P {
    let max = P::Subpixel::DEFAULT_MAX_VALUE.to_f64().unwrap_or(255.0);
    let channels: Vec<P::Subpixel> = rgba
        .iter()
        .take(usize::from(P::CHANNEL_COUNT))
        .map(|&v| channel(f64::from(v) / 255.0 * max))
        .collect();
    *P::from_slice(&channels)
}

fn pixel_bounds(rect: Rect) -> (u32, u32, u32, u32) {
    (
        numbers::truncate::<u32>(rect.x),
        numbers::truncate::<u32>(rect.y),
        numbers::truncate::<u32>(rect.x + rect.width),
        numbers::truncate::<u32>(rect.y + rect.height),
    )
}

fn pixelate<P: Pixel>(picture: &mut Picture<P>, rect: Rect, block: u32) {
    let (left, top, right, bottom) = pixel_bounds(rect);
    let block = block.max(1);
    let channels = usize::from(P::CHANNEL_COUNT);
    for y0 in (top..bottom).step_by(block as usize) {
        for x0 in (left..right).step_by(block as usize) {
            let (x1, y1) = ((x0 + block).min(right), (y0 + block).min(bottom));
            let mut sum = vec![0.0f64; channels];
            for y in y0..y1 {
                for x in x0..x1 {
                    for (total, value) in sum.iter_mut().zip(picture.get_pixel(x, y).channels()) {
                        *total += value.to_f64().unwrap_or(0.0);
                    }
                }
            }
            let count = f64::from((x1 - x0) * (y1 - y0));
            let average: Vec<P::Subpixel> =
                sum.iter().map(|total| channel(total / count)).collect();
            let average = *P::from_slice(&average);
            for y in y0..y1 {
                for x in x0..x1 {
                    picture.put_pixel(x, y, average);
                }
            }
        }
    }
}

fn blur_inside<P>(picture: &mut Picture<P>, rect: Rect, sigma: f64)
where
    P: Pixel + 'static,
    P::Subpixel: Into<f32> + imageproc::definitions::Clamp<f32>,
{
    let (left, top, right, bottom) = pixel_bounds(rect);
    let part = imageops::crop_imm(picture, left, top, right - left, bottom - top).to_image();
    let blurred = gaussian_blur_f32(&part, numbers::narrow(sigma.max(0.1)));
    imageops::replace(picture, &blurred, i64::from(left), i64::from(top));
}

fn fill<P: Pixel>(picture: &mut Picture<P>, rect: Rect, colour: P, radius: f64) {
    let (left, top, right, bottom) = pixel_bounds(rect);
    for y in top..bottom {
        for x in left..right {
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            if radius <= 0.0 || in_rounded(px, py, rect, radius) {
                picture.put_pixel(x, y, colour);
            }
        }
    }
}

fn redacted<P>(mut picture: Picture<P>, rects: &[Rect], p: &RedactImage) -> Picture<P>
where
    P: Pixel + 'static,
    P::Subpixel: Into<f32> + imageproc::definitions::Clamp<f32>,
{
    let colour = colour_in::<P>(background_rgba(&p.color));
    for &rect in rects {
        match p.style {
            RedactStyle::Pixelate => pixelate(&mut picture, rect, p.block_size),
            RedactStyle::Blur => blur_inside(&mut picture, rect, p.blur),
            RedactStyle::Solid => fill(&mut picture, rect, colour, p.radius),
        }
    }
    picture
}

pub fn redact(image: &DynamicImage, p: &RedactImage) -> DynamicImage {
    let rects: Vec<Rect> = p
        .areas
        .iter()
        .map(|area| area_rect(size_of(image), area))
        .collect();
    on_each_depth(
        image,
        |deep| redacted(deep, &rects, p),
        |shallow| redacted(shallow, &rects, p),
    )
}

fn from_float(image: &DynamicImage, out: Rgba32FImage) -> DynamicImage {
    let out = DynamicImage::ImageRgba32F(out);
    in_depth(image, out.to_rgba16(), out.to_rgba8())
}

fn in_rounded(px: f64, py: f64, rect: Rect, radius: f64) -> bool {
    let r = radius.min(rect.width / 2.0).min(rect.height / 2.0).max(0.0);
    let dx = (rect.x + r - px)
        .max(px - (rect.x + rect.width - r))
        .max(0.0);
    let dy = (rect.y + r - py)
        .max(py - (rect.y + rect.height - r))
        .max(0.0);
    px >= rect.x
        && px <= rect.x + rect.width
        && py >= rect.y
        && py <= rect.y + rect.height
        && dy.mul_add(dy, dx * dx) <= r * r
}

pub fn lit_mask(
    size: (u32, u32),
    rects: &[Rect],
    radius: f64,
    softness: f64,
) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let mask = ImageBuffer::from_fn(size.0, size.1, |x, y| {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        Luma([if rects.iter().any(|&r| in_rounded(px, py, r, radius)) {
            1.0
        } else {
            0.0
        }])
    });
    if softness > 0.0 {
        gaussian_blur_f32(&mask, numbers::narrow(softness / 2.0))
    } else {
        mask
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn spotlight(image: &DynamicImage, p: &SpotlightImage) -> DynamicImage {
    let rects: Vec<Rect> = p
        .areas
        .iter()
        .map(|area| area_rect(size_of(image), area))
        .collect();
    let mask = lit_mask(
        (image.width(), image.height()),
        &rects,
        p.radius,
        p.softness,
    );
    let source = image.to_rgba32f();
    let rest = if p.blur > 0.0 {
        DynamicImage::ImageRgba32F(source.clone())
            .fast_blur(numbers::narrow(p.blur))
            .to_rgba32f()
    } else {
        source.clone()
    };
    let keep = 1.0 - numbers::narrow(p.dim.clamp(0.0, 1.0));
    let [dr, dg, db, _] = background_rgba(&p.dim_color).map(|v| f32::from(v) / 255.0);
    let toward = |value: f32, dim: f32| dim.mul_add(1.0 - keep, value * keep);
    let out = ImageBuffer::from_fn(source.width(), source.height(), |x, y| {
        let lit = mask.get_pixel(x, y).0[0].clamp(0.0, 1.0);
        let [r, g, b, a] = source.get_pixel(x, y).0;
        let [rr, rg, rb, ra] = rest.get_pixel(x, y).0;
        let mix = |near: f32, far: f32| far.mul_add(1.0 - lit, near * lit);
        Rgba([
            mix(r, toward(rr, dr)),
            mix(g, toward(rg, dg)),
            mix(b, toward(rb, db)),
            mix(a, ra),
        ])
    });
    from_float(image, out)
}

fn differs(a: [u8; 4], b: [u8; 4], tolerance: u32) -> bool {
    a.iter()
        .zip(b)
        .any(|(x, y)| u32::from(x.abs_diff(y)) > tolerance)
}

pub fn content_rect(image: &RgbaImage, tolerance: u32, edge: Option<[u8; 4]>) -> Option<Rect> {
    let edge = edge.unwrap_or_else(|| image.get_pixel(0, 0).0);
    let (w, h) = image.dimensions();
    let row_full = |y: u32| (0..w).any(|x| differs(image.get_pixel(x, y).0, edge, tolerance));
    let column_full = |x: u32, top: u32, bottom: u32| {
        (top..=bottom).any(|y| differs(image.get_pixel(x, y).0, edge, tolerance))
    };
    let top = (0..h).find(|&y| row_full(y))?;
    let bottom = (0..h).rev().find(|&y| row_full(y))?;
    let left = (0..w).find(|&x| column_full(x, top, bottom))?;
    let right = (0..w).rev().find(|&x| column_full(x, top, bottom))?;
    Some(Rect {
        x: f64::from(left),
        y: f64::from(top),
        width: f64::from(right - left + 1),
        height: f64::from(bottom - top + 1),
    })
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn trim(image: &DynamicImage, p: &TrimEdges) -> DynamicImage {
    let edge = (!p.edge_color.trim().is_empty()).then(|| background_rgba(&p.edge_color));
    let Some(content) = content_rect(&image.to_rgba8(), p.tolerance, edge) else {
        return image.clone();
    };
    let pad = f64::from(p.padding);
    let (w, h) = size_of(image);
    let x = (content.x - pad).max(0.0);
    let y = (content.y - pad).max(0.0);
    let right = (content.x + content.width + pad).min(w);
    let bottom = (content.y + content.height + pad).min(h);
    image.crop_imm(
        numbers::truncate::<u32>(x),
        numbers::truncate::<u32>(y),
        numbers::truncate::<u32>(right - x),
        numbers::truncate::<u32>(bottom - y),
    )
}

fn rgb_distance(a: [f32; 4], b: [f32; 4]) -> f32 {
    let d: f32 = (0..3).map(|i| (a[i] - b[i]).powi(2)).sum();
    (d / 3.0).sqrt() * 100.0
}

fn kept(distance: f32, tolerance: f32, softness: f32) -> f32 {
    if distance <= tolerance {
        0.0
    } else if softness <= 0.0 || distance >= tolerance + softness {
        1.0
    } else {
        (distance - tolerance) / softness
    }
}

fn touching_edges(width: u32, height: u32, near: impl Fn(u32, u32) -> bool) -> Vec<bool> {
    let index = |x: u32, y: u32| (y * width + x) as usize;
    let mut reached = vec![false; (width * height) as usize];
    let mut queue = std::collections::VecDeque::new();
    for x in 0..width {
        queue.push_back((x, 0));
        queue.push_back((x, height - 1));
    }
    for y in 0..height {
        queue.push_back((0, y));
        queue.push_back((width - 1, y));
    }
    while let Some((x, y)) = queue.pop_front() {
        if reached[index(x, y)] || !near(x, y) {
            continue;
        }
        reached[index(x, y)] = true;
        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if x + 1 < width {
            queue.push_back((x + 1, y));
        }
        if y + 1 < height {
            queue.push_back((x, y + 1));
        }
    }
    reached
}

pub fn clear_colour(image: &DynamicImage, p: &ClearColour) -> DynamicImage {
    let mut pixels = image.to_rgba32f();
    let (width, height) = pixels.dimensions();
    let target = if p.color.trim().is_empty() {
        pixels.get_pixel(0, 0).0
    } else {
        background_rgba(&p.color).map(|v| f32::from(v) / 255.0)
    };
    let tolerance = numbers::narrow(p.tolerance.clamp(0.0, 100.0));
    let softness = numbers::narrow(p.softness.clamp(0.0, 100.0));
    let distance = |x: u32, y: u32| rgb_distance(pixels.get_pixel(x, y).0, target);
    let reached = if p.edges_only {
        Some(touching_edges(width, height, |x, y| {
            distance(x, y) < tolerance + softness
        }))
    } else {
        None
    };
    let alphas: Vec<f32> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let open = reached
                .as_ref()
                .is_none_or(|reached| reached[(y * width + x) as usize]);
            if open {
                kept(distance(x, y), tolerance, softness)
            } else {
                1.0
            }
        })
        .collect();
    for (pixel, keep) in pixels.pixels_mut().zip(alphas) {
        pixel.0[3] *= keep;
    }
    let out = DynamicImage::ImageRgba32F(pixels);
    if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    }
}

pub(crate) fn over(top: [f32; 4], bottom: [f32; 4]) -> [f32; 4] {
    let a = bottom[3].mul_add(1.0 - top[3], top[3]);
    if a <= 0.0 {
        return [0.0; 4];
    }
    let mix = |i: usize| (bottom[i] * bottom[3]).mul_add(1.0 - top[3], top[i] * top[3]) / a;
    [mix(0), mix(1), mix(2), a]
}

pub fn has_see_through(image: &DynamicImage) -> bool {
    image.color().has_alpha() && image.to_rgba32f().pixels().any(|p| p.0[3] < 1.0)
}

fn soften(mask: ImageBuffer<Luma<f32>, Vec<f32>>, sigma: f64) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    if sigma > 0.0 {
        gaussian_blur_f32(&mask, numbers::narrow(sigma))
    } else {
        mask
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn outline(image: &DynamicImage, p: &OutlineImage) -> DynamicImage {
    let source = image.to_rgba32f();
    let pad = numbers::truncate::<u32>(outline_padding(p));
    let (w, h) = (source.width() + 2 * pad, source.height() + 2 * pad);
    let at = |x: u32, y: u32| {
        if x >= pad && y >= pad && x < pad + source.width() && y < pad + source.height() {
            source.get_pixel(x - pad, y - pad).0
        } else {
            [0.0; 4]
        }
    };
    let solid = ImageBuffer::from_fn(w, h, |x, y| Luma([u8::from(at(x, y)[3] > 0.5) * 255]));
    let distance = euclidean_squared_distance_transform(&solid);
    let width = p.width.max(0.0);
    let ring = ImageBuffer::from_fn(w, h, |x, y| {
        let d = distance.get_pixel(x, y).0[0].sqrt();
        Luma([numbers::narrow((width + 0.5 - d).clamp(0.0, 1.0))])
    });
    let ring = soften(ring, p.softness / 2.0);
    let shadow = p.shadow.then(|| soften(ring.clone(), p.shadow_softness()));
    let drop = numbers::whole(p.shadow_drop());
    let shade = background_rgba(&p.shadow_color).map(|v| f32::from(v) / 255.0);
    let shadow_alpha = numbers::narrow(p.shadow_opacity);
    let [r, g, b, a] = background_rgba(&p.color).map(|v| f32::from(v) / 255.0);
    let out = ImageBuffer::from_fn(w, h, |x, y| {
        let shadowed = if let Some(shadow) = &shadow
            && y >= drop
        {
            [
                shade[0],
                shade[1],
                shade[2],
                shadow.get_pixel(x, y - drop).0[0] * shadow_alpha * shade[3],
            ]
        } else {
            [0.0; 4]
        };
        let ringed = over([r, g, b, a * ring.get_pixel(x, y).0[0]], shadowed);
        Rgba(over(at(x, y), ringed))
    });
    let out = DynamicImage::ImageRgba32F(out);
    if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    }
}

pub fn tilt(image: &DynamicImage, p: &TiltImage) -> Result<DynamicImage, ImageError> {
    let size = size_of(image);
    let (w, h) = tilted_size(size, p);
    let from = [(0.0, 0.0), (size.0, 0.0), size, (0.0, size.1)]
        .map(|(x, y)| (numbers::narrow(x), numbers::narrow(y)));
    let to = tilt_corners(size, p).map(|(x, y)| (numbers::narrow(x), numbers::narrow(y)));
    let projection = Projection::from_control_points(from, to)
        .ok_or_else(|| ImageError::Failed("That angle folds the picture away.".to_owned()))?;
    let mut warped = ImageBuffer::new(numbers::truncate::<u32>(w), numbers::truncate::<u32>(h));
    warp_into(
        &image.to_rgba16(),
        projection,
        Interpolation::Bilinear,
        Border::Constant(Rgba([0, 0, 0, 0])),
        &mut warped,
    );
    let warped = DynamicImage::ImageRgba16(warped).to_rgba32f();
    let long = size.0.max(size.1);
    let soft = p.shadow_softness / 100.0 * long;
    let shadow = p.shadow.then(|| {
        let alpha = ImageBuffer::from_fn(warped.width(), warped.height(), |x, y| {
            Luma([warped.get_pixel(x, y).0[3]])
        });
        soften(alpha, soft)
    });
    let drop = numbers::whole(p.shadow_drop / 100.0 * long);
    let shade = background_rgba(&p.shadow_color).map(|v| f32::from(v) / 255.0);
    let shadow_alpha = numbers::narrow(p.shadow_opacity);
    let background = background_rgba(&p.background).map(|v| f32::from(v) / 255.0);
    let out = ImageBuffer::from_fn(warped.width(), warped.height(), |x, y| {
        let mut pixel = background;
        if let Some(shadow) = &shadow
            && y >= drop
        {
            let a = shadow.get_pixel(x, y - drop).0[0] * shadow_alpha * shade[3];
            pixel = over([shade[0], shade[1], shade[2], a], pixel);
        }
        Rgba(over(warped.get_pixel(x, y).0, pixel))
    });
    let out = DynamicImage::ImageRgba32F(out);
    Ok(if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    })
}

const LEVEL_BINS: usize = 1024;

fn bin(v: f32) -> usize {
    numbers::truncate::<usize>(f64::from(
        (v.clamp(0.0, 1.0) * numbers::narrow(numbers::float(LEVEL_BINS - 1))).round(),
    ))
}

fn histogram(values: impl Iterator<Item = f32>) -> Vec<f64> {
    let mut counts = vec![0.0; LEVEL_BINS];
    for v in values {
        counts[bin(v)] += 1.0;
    }
    counts
}

fn level_at(counts: &[f64], reached: impl Fn(f64) -> bool) -> f32 {
    let mut seen = 0.0;
    for (i, count) in counts.iter().enumerate() {
        seen += count;
        if reached(seen) {
            return numbers::narrow(numbers::float(i))
                / numbers::narrow(numbers::float(LEVEL_BINS - 1));
        }
    }
    1.0
}

fn stretch_map(counts: &[f64], clip: f64) -> impl Fn(f32) -> f32 + use<> {
    let share = clip.clamp(0.0, 5.0) / 100.0;
    let total: f64 = counts.iter().sum();
    let low = level_at(counts, |seen| seen > total * share);
    let high = level_at(counts, |seen| seen >= total * (1.0 - share));
    let span = (high - low).max(1.0 / 255.0);
    move |v| ((v - low) / span).clamp(0.0, 1.0)
}

fn equalize_map(counts: &[f64]) -> impl Fn(f32) -> f32 + use<> {
    let total: f64 = counts.iter().sum::<f64>().max(1.0);
    let mut running = 0.0;
    let cdf: Vec<f32> = counts
        .iter()
        .map(|c| {
            running += c;
            numbers::narrow(running / total)
        })
        .collect();
    move |v| cdf[bin(v)]
}

pub fn auto_levels(image: &DynamicImage, p: &AutoLevels) -> DynamicImage {
    let mut pixels = image.to_rgba32f();
    let strength = numbers::narrow(p.strength.clamp(0.0, 1.0));
    let maps: Vec<Box<dyn Fn(f32) -> f32>> = (0..3)
        .map(|channel| {
            let values: Vec<f32> = if p.per_channel {
                pixels.pixels().map(|px| px.0[channel]).collect()
            } else {
                pixels
                    .pixels()
                    .flat_map(|px| [px.0[0], px.0[1], px.0[2]])
                    .collect()
            };
            let counts = histogram(values.into_iter());
            match p.method {
                LevelsMethod::Stretch => {
                    Box::new(stretch_map(&counts, p.clip)) as Box<dyn Fn(f32) -> f32>
                }
                LevelsMethod::Equalize => Box::new(equalize_map(&counts)),
            }
        })
        .collect();
    for pixel in pixels.pixels_mut() {
        for (channel, map) in maps.iter().enumerate() {
            let v = pixel.0[channel];
            pixel.0[channel] = f32::mul_add(map(v) - v, strength, v);
        }
    }
    from_float(image, pixels)
}

#[expect(
    clippy::many_single_char_names,
    reason = "r, g, b, a are colour channels and w, h, x, y the picture's geometry"
)]
pub fn grid(images: &[DynamicImage], p: &ImageGrid) -> Result<DynamicImage, ImageError> {
    let sizes: Vec<(f64, f64)> = images.iter().map(size_of).collect();
    let plan = grid_plan(&sizes, p);
    let [r, g, b, a] = background_rgba(&p.background);
    let wide = |v: u8| u16::from(v) * 257;
    let mut canvas: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_pixel(
        numbers::truncate::<u32>(plan.canvas.0),
        numbers::truncate::<u32>(plan.canvas.1),
        Rgba([wide(r), wide(g), wide(b), wide(a)]),
    );
    let options = ResizeOptions::new().resize_alg(algorithm(Resample::Sharp));
    for (image, cell) in images.iter().zip(&plan.cells) {
        let fitted = if (cell.width, cell.height) == size_of(image) {
            image.clone()
        } else {
            resized_to(
                &DynamicImage::ImageRgba16(image.to_rgba16()),
                numbers::truncate::<u32>(cell.width),
                numbers::truncate::<u32>(cell.height),
                &options,
            )?
        };
        let mut fitted = fitted.to_rgba16();
        if p.radius > 0.0 {
            let shape = Rect {
                x: 0.0,
                y: 0.0,
                width: f64::from(fitted.width()),
                height: f64::from(fitted.height()),
            };
            for (x, y, pixel) in fitted.enumerate_pixels_mut() {
                let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
                if !in_rounded(px, py, shape, p.radius) {
                    pixel.0[3] = 0;
                }
            }
        }
        imageops::overlay(
            &mut canvas,
            &fitted,
            numbers::truncate::<i64>(cell.x),
            numbers::truncate::<i64>(cell.y),
        );
    }
    let deep = images.iter().any(is_deep);
    Ok(if deep {
        DynamicImage::ImageRgba16(canvas)
    } else {
        DynamicImage::ImageRgba8(DynamicImage::ImageRgba16(canvas).to_rgba8())
    })
}

#[derive(Debug)]
pub enum Paint {
    Colour([u8; 4]),
    Gradient(Gradient),
    Picture(DynamicImage),
}

fn stop_colour(colour: &str) -> [f32; 4] {
    background_rgba(colour).map(|v| f32::from(v) / 255.0)
}

fn along(gradient: &Gradient, t: f64) -> [f32; 4] {
    let stops: Vec<(f64, [f32; 4])> = gradient
        .stops
        .iter()
        .map(|s| (s.at / 100.0, stop_colour(&s.color)))
        .collect();
    let first = stops.first().map_or([0.0; 4], |s| s.1);
    if t <= stops.first().map_or(0.0, |s| s.0) {
        return first;
    }
    for pair in stops.windows(2) {
        let ((a, ca), (b, cb)) = (pair[0], pair[1]);
        if t <= b {
            let k = if b > a {
                numbers::narrow((t - a) / (b - a))
            } else {
                1.0
            };
            return [0, 1, 2, 3].map(|i| f32::mul_add(cb[i] - ca[i], k, ca[i]));
        }
    }
    stops.last().map_or(first, |s| s.1)
}

pub fn paint_gradient(size: (u32, u32), gradient: &Gradient) -> Rgba32FImage {
    let (w, h) = (f64::from(size.0), f64::from(size.1));
    let (sin, cos) = gradient.angle.to_radians().sin_cos();
    let length = (w * sin).abs() + (h * cos).abs();
    ImageBuffer::from_fn(size.0, size.1, |x, y| {
        let (dx, dy) = (f64::from(x) + 0.5 - w / 2.0, f64::from(y) + 0.5 - h / 2.0);
        let t = match gradient.kind {
            GradientKind::Linear => f64::mul_add(dy, -cos, dx * sin) / length.max(1.0) + 0.5,
            GradientKind::Radial => {
                let (rx, ry) = (
                    w / 2.0 * std::f64::consts::SQRT_2,
                    h / 2.0 * std::f64::consts::SQRT_2,
                );
                (dx / rx).hypot(dy / ry)
            }
        };
        Rgba(along(gradient, t))
    })
}

fn covering(picture: &DynamicImage, size: (u32, u32)) -> Result<Rgba32FImage, ImageError> {
    let options = ResizeOptions::new()
        .resize_alg(algorithm(Resample::Smooth))
        .fit_into_destination(Some((0.5, 0.5)));
    Ok(resized_to(
        &DynamicImage::ImageRgba16(picture.to_rgba16()),
        size.0,
        size.1,
        &options,
    )?
    .to_rgba32f())
}

fn rounded_distance(px: f64, py: f64, rect: Rect, radius: f64) -> f64 {
    let r = radius.min(rect.width / 2.0).min(rect.height / 2.0).max(0.0);
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let qx = (px - cx).abs() - (rect.width / 2.0 - r);
    let qy = (py - cy).abs() - (rect.height / 2.0 - r);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0) - r
}

fn rounded_cover(px: f64, py: f64, rect: Rect, radius: f64) -> f32 {
    numbers::narrow((0.5 - rounded_distance(px, py, rect, radius)).clamp(0.0, 1.0))
}

pub fn frame(
    image: &DynamicImage,
    paint: &Paint,
    p: &FrameScreenshot,
) -> Result<DynamicImage, ImageError> {
    let source = size_of(image);
    let ((w, h), (x0, y0)) = framed_placement(source, p);
    let size = (numbers::truncate::<u32>(w), numbers::truncate::<u32>(h));
    let base = match paint {
        Paint::Colour(rgba) => {
            ImageBuffer::from_pixel(size.0, size.1, Rgba(rgba.map(|v| f32::from(v) / 255.0)))
        }
        Paint::Gradient(gradient) => paint_gradient(size, gradient),
        Paint::Picture(picture) => covering(picture, size)?,
    };
    let picture = image.to_rgba32f();
    let rect = Rect {
        x: x0,
        y: y0,
        width: source.0,
        height: source.1,
    };
    let drop = p.shadow_drop.max(0.0);
    let shadow = (p.shadow_opacity > 0.0).then(|| {
        let mask = ImageBuffer::from_fn(size.0, size.1, |x, y| {
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5 - drop);
            Luma([rounded_cover(px, py, rect, p.radius)])
        });
        soften(mask, p.shadow_softness / 2.0)
    });
    let opacity = numbers::narrow(p.shadow_opacity.clamp(0.0, 1.0));
    let shade = background_rgba(&p.shadow_color).map(|v| f32::from(v) / 255.0);
    let out = ImageBuffer::from_fn(size.0, size.1, |x, y| {
        let mut pixel = base.get_pixel(x, y).0;
        if let Some(shadow) = &shadow {
            pixel = over(
                [
                    shade[0],
                    shade[1],
                    shade[2],
                    shadow.get_pixel(x, y).0[0] * opacity * shade[3],
                ],
                pixel,
            );
        }
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        let cover = rounded_cover(px, py, rect, p.radius);
        if cover > 0.0 {
            let (ix, iy) = (
                numbers::truncate::<u32>(f64::from(x) - x0),
                numbers::truncate::<u32>(f64::from(y) - y0),
            );
            if ix < picture.width() && iy < picture.height() {
                let mut top = picture.get_pixel(ix, iy).0;
                top[3] *= cover;
                pixel = over(top, pixel);
            }
        }
        Rgba(pixel)
    });
    let out = DynamicImage::ImageRgba32F(out);
    Ok(if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    })
}

fn unit_colour(rgba: [u8; 4]) -> [f32; 4] {
    rgba.map(|v| f32::from(v) / 255.0)
}

fn lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [0, 1, 2, 3].map(|i| (b[i] - a[i]).mul_add(t, a[i]))
}

fn split_pictures(
    before: &Rgba32FImage,
    after: &Rgba32FImage,
    divider: [f32; 4],
    p: &BeforeAfter,
) -> Rgba32FImage {
    let (w, h) = before.dimensions();
    let centre = (
        p.position.clamp(0.0, 1.0) * f64::from(w),
        f64::from(h) / 2.0,
    );
    let (sin, cos) = p.angle.to_radians().sin_cos();
    let half = f64::from(p.divider_width) / 2.0;
    ImageBuffer::from_fn(w, h, |x, y| {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        let d = f64::mul_add(py - centre.1, sin, (px - centre.0) * cos);
        let share = numbers::narrow((d + 0.5).clamp(0.0, 1.0));
        let mut pixel = lerp(before.get_pixel(x, y).0, after.get_pixel(x, y).0, share);
        if half > 0.0 {
            let cover = numbers::narrow((half - d.abs() + 0.5).clamp(0.0, 1.0));
            let mut line = divider;
            line[3] *= cover;
            pixel = over(line, pixel);
        }
        Rgba(pixel)
    })
}

fn paste(canvas: &mut Rgba32FImage, picture: &Rgba32FImage, at: Rect) {
    let (x0, y0) = (whole(at.x), whole(at.y));
    for (x, y, pixel) in picture.enumerate_pixels() {
        if x0 + x < canvas.width() && y0 + y < canvas.height() {
            canvas.put_pixel(x0 + x, y0 + y, *pixel);
        }
    }
}

fn draw_label(
    canvas: &mut Rgba32FImage,
    font: &ab_glyph::FontVec,
    size: f64,
    corner: (f64, f64, bool, bool),
    text: &str,
    p: &BeforeAfter,
) {
    if text.trim().is_empty() {
        return;
    }
    let measured = measure(font, size, text);
    let (pad_x, pad_y) = (size * 0.55, size * 0.3);
    let rect = Rect {
        x: if corner.2 {
            2.0f64.mul_add(-pad_x, corner.0 - measured.width)
        } else {
            corner.0
        },
        y: if corner.3 {
            2.0f64.mul_add(-pad_y, corner.1 - measured.height)
        } else {
            corner.1
        },
        width: 2.0f64.mul_add(pad_x, measured.width),
        height: 2.0f64.mul_add(pad_y, measured.height),
    };
    let background = unit_colour(background_rgba(&p.label_background));
    let radius = rect.height * f64::from(p.label_radius) / 100.0;
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let cover = rounded_cover(f64::from(x) + 0.5, f64::from(y) + 0.5, rect, radius);
        if cover > 0.0 {
            let mut top = background;
            top[3] *= cover;
            *pixel = Rgba(over(top, pixel.0));
        }
    }
    let colour = unit_colour(background_rgba(&p.label_color));
    draw_text(
        canvas,
        font,
        size,
        (rect.x + pad_x, rect.y + pad_y),
        text,
        colour,
    );
}

pub fn before_after(
    before: &DynamicImage,
    after: &DynamicImage,
    font: Option<&ab_glyph::FontVec>,
    p: &BeforeAfter,
) -> Result<DynamicImage, ImageError> {
    let plan: BeforeAfterPlan = before_after_plan(size_of(before), size_of(after), p);
    let (w, h) = plan.size;
    let divider = unit_colour(background_rgba(&p.divider_color));
    let pixels_of = |rect: Rect| (whole(rect.width), whole(rect.height));
    let before_pixels = covering(before, pixels_of(plan.before))?;
    let after_pixels = covering(after, pixels_of(plan.after))?;
    let mut out = match p.layout {
        BeforeAfterLayout::Split => split_pictures(&before_pixels, &after_pixels, divider, p),
        BeforeAfterLayout::SideBySide | BeforeAfterLayout::Stacked => {
            let mut canvas = ImageBuffer::from_pixel(w, h, Rgba(divider));
            paste(&mut canvas, &before_pixels, plan.before);
            paste(&mut canvas, &after_pixels, plan.after);
            canvas
        }
    };
    if p.labels
        && let Some(font) = font
    {
        let size = label_size(&plan, p);
        let margin = size * 0.8;
        let split = p.layout == BeforeAfterLayout::Split;
        let bottom = p.label_position == LabelEdge::Bottom;
        let edge = |rect: Rect| {
            if bottom {
                rect.y + rect.height - margin
            } else {
                rect.y + margin
            }
        };
        let before_corner = (plan.before.x + margin, edge(plan.before), false, bottom);
        let after_corner = if split {
            (
                plan.after.x + plan.after.width - margin,
                edge(plan.after),
                true,
                bottom,
            )
        } else {
            (plan.after.x + margin, edge(plan.after), false, bottom)
        };
        draw_label(&mut out, font, size, before_corner, &p.before_label, p);
        draw_label(&mut out, font, size, after_corner, &p.after_label, p);
    }
    let out = DynamicImage::ImageRgba32F(out);
    Ok(if is_deep(before) || is_deep(after) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    })
}

fn shape_distance(px: f64, py: f64, rect: Rect, p: &Magnify) -> f64 {
    let corner = match p.shape {
        MagnifyShape::Circle => rect.width.min(rect.height) / 2.0,
        MagnifyShape::Rounded => f64::from(p.radius),
    };
    rounded_distance(px, py, rect, corner)
}

pub(crate) fn segment_distance(point: (f64, f64), from: (f64, f64), to: (f64, f64)) -> f64 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = f64::mul_add(dy, dy, dx * dx);
    let t = if length > 0.0 {
        ((point.1 - from.1).mul_add(dy, (point.0 - from.0) * dx) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    f64::mul_add(t, -dx, point.0 - from.0).hypot(f64::mul_add(t, -dy, point.1 - from.1))
}

fn ring(distance: f64, half: f64) -> f32 {
    numbers::narrow((half - distance.abs() + 0.5).clamp(0.0, 1.0))
}

fn tinted(pixel: [f32; 4], colour: [f32; 4], cover: f32) -> [f32; 4] {
    let mut top = colour;
    top[3] *= cover;
    over(top, pixel)
}

const MAGNIFY_SHADOW_DROP: f64 = 0.04;
const MAGNIFY_SHADOW_ALPHA: f32 = 0.35;

pub fn magnify(image: &DynamicImage, p: &Magnify) -> Result<DynamicImage, ImageError> {
    let plan = magnify_plan(size_of(image), p);
    let (area, shown) = (plan.area, plan.shown);
    let cut = image.crop_imm(
        whole(area.x),
        whole(area.y),
        whole(area.width).max(1),
        whole(area.height).max(1),
    );
    let options = ResizeOptions::new().resize_alg(algorithm(p.quality));
    let view = resized_to(
        &DynamicImage::ImageRgba16(cut.to_rgba16()),
        whole(shown.width).max(1),
        whole(shown.height).max(1),
        &options,
    )?
    .to_rgba32f();
    let picture = image.to_rgba32f();
    let border = unit_colour(background_rgba(&p.border_color));
    let half = f64::from(p.border_width) / 2.0;
    let thin = p
        .line_width
        .map_or_else(|| (half / 2.0).max(0.5), |width| f64::from(width) / 2.0);
    let drop = (shown.width.min(shown.height) * MAGNIFY_SHADOW_DROP).max(1.0);
    let ends = (
        (area.x + area.width / 2.0, area.y + area.height / 2.0),
        (shown.x + shown.width / 2.0, shown.y + shown.height / 2.0),
    );
    let out = ImageBuffer::from_fn(picture.width(), picture.height(), |x, y| {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        let mut pixel = picture.get_pixel(x, y).0;
        let to_area = shape_distance(px, py, area, p);
        let to_shown = shape_distance(px, py, shown, p);
        if p.line && (p.line_width.is_some() || p.border_width > 0) {
            let along = segment_distance((px, py), ends.0, ends.1);
            let outside = numbers::narrow((0.5 + to_area).clamp(0.0, 1.0));
            pixel = tinted(pixel, border, ring(along, thin) * outside);
            pixel = tinted(pixel, border, ring(to_area, thin));
        }
        if p.shadow {
            let below = shape_distance(px, py - drop, shown, p);
            let cover = numbers::narrow(((drop - below) / (2.0 * drop)).clamp(0.0, 1.0));
            pixel = over([0.0, 0.0, 0.0, MAGNIFY_SHADOW_ALPHA * cover], pixel);
        }
        let inside = numbers::narrow((0.5 - to_shown).clamp(0.0, 1.0));
        if inside > 0.0 {
            let vx = whole(px - shown.x - 0.5).min(view.width() - 1);
            let vy = whole(py - shown.y - 0.5).min(view.height() - 1);
            let mut top = view.get_pixel(vx, vy).0;
            top[3] *= inside;
            pixel = over(top, pixel);
        }
        if p.border_width > 0 {
            pixel = tinted(pixel, border, ring(to_shown, half));
        }
        Rgba(pixel)
    });
    let out = DynamicImage::ImageRgba32F(out);
    Ok(if is_deep(image) {
        DynamicImage::ImageRgba16(out.to_rgba16())
    } else {
        DynamicImage::ImageRgba8(out.to_rgba8())
    })
}

#[derive(Debug)]
pub struct Comparison {
    pub boxes: Vec<Rect>,
    pub marked: DynamicImage,
    pub differences: DynamicImage,
}

fn changed_mask(before: &RgbaImage, after: &RgbaImage, threshold: u8) -> GrayImage {
    GrayImage::from_fn(after.width(), after.height(), |x, y| {
        let (a, b) = (before.get_pixel(x, y).0, after.get_pixel(x, y).0);
        let most = (0..4).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0);
        Luma([if most > threshold { 255 } else { 0 }])
    })
}

pub fn changed_boxes(mask: &GrayImage, p: &CompareScreenshots) -> Vec<Rect> {
    let padding = p.padding.min(255);
    let joined = if padding > 0 {
        dilate(mask, Norm::LInf, u8::try_from(padding).unwrap_or(u8::MAX))
    } else {
        mask.clone()
    };
    let labels = connected_components(&joined, Connectivity::Eight, Luma([0u8]));
    let mut found: std::collections::BTreeMap<u32, (u32, u32, u32, u32)> =
        std::collections::BTreeMap::new();
    for (x, y, pixel) in mask.enumerate_pixels() {
        if pixel.0[0] == 0 {
            continue;
        }
        let label = labels.get_pixel(x, y).0[0];
        let entry = found.entry(label).or_insert((x, y, x, y));
        *entry = (
            entry.0.min(x),
            entry.1.min(y),
            entry.2.max(x),
            entry.3.max(y),
        );
    }
    let (w, h) = (f64::from(mask.width()), f64::from(mask.height()));
    let pad = f64::from(padding);
    let mut boxes: Vec<Rect> = found
        .into_values()
        .filter(|(x0, y0, x1, y1)| (x1 - x0 + 1).max(y1 - y0 + 1) >= p.smallest)
        .map(|(x0, y0, x1, y1)| {
            let left = (f64::from(x0) - pad).max(0.0);
            let top = (f64::from(y0) - pad).max(0.0);
            let right = (f64::from(x1) + 1.0 + pad).min(w);
            let bottom = (f64::from(y1) + 1.0 + pad).min(h);
            Rect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
            }
        })
        .collect();
    boxes.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    boxes
}

fn in_band(x: u32, y: u32, rect: Rect, width: f64) -> bool {
    let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
    let inside =
        px >= rect.x && px < rect.x + rect.width && py >= rect.y && py < rect.y + rect.height;
    let core = px >= rect.x + width
        && px < rect.x + rect.width - width
        && py >= rect.y + width
        && py < rect.y + rect.height - width;
    inside && !core
}

pub fn compare(
    before: &DynamicImage,
    after: &DynamicImage,
    p: &CompareScreenshots,
) -> Result<Comparison, ImageError> {
    let mask = changed_mask(
        &before.to_rgba8(),
        &after.to_rgba8(),
        compare_threshold(p.sensitivity),
    );
    let boxes = changed_boxes(&mask, p);
    let colour = unit_colour(background_rgba(&p.box_color));
    let width = f64::from(p.box_width);
    let picture = after.to_rgba32f();
    let marked = ImageBuffer::from_fn(picture.width(), picture.height(), |x, y| {
        let pixel = picture.get_pixel(x, y).0;
        if boxes.iter().any(|b| in_band(x, y, *b, width)) {
            Rgba(over(colour, pixel))
        } else {
            Rgba(pixel)
        }
    });
    let unchanged = unit_colour(background_rgba(&p.differences_background));
    let differences = ImageBuffer::from_fn(picture.width(), picture.height(), |x, y| {
        if mask.get_pixel(x, y).0[0] == 0 {
            Rgba(unchanged)
        } else {
            *picture.get_pixel(x, y)
        }
    });
    let depth = |out: Rgba32FImage| {
        let out = DynamicImage::ImageRgba32F(out);
        if is_deep(after) {
            DynamicImage::ImageRgba16(out.to_rgba16())
        } else {
            DynamicImage::ImageRgba8(out.to_rgba8())
        }
    };
    Ok(Comparison {
        boxes,
        marked: depth(marked),
        differences: depth(differences),
    })
}
