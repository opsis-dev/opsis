use std::io::Cursor;

use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{
    DynamicImage, ImageBuffer, Luma, Pixel, Primitive, Rgba, Rgba32FImage, RgbaImage, imageops,
};
use imageproc::filter::gaussian_blur_f32;
use imageproc::geometric_transformations::{Border, Interpolation, rotate_about_center};
use num_traits::ToPrimitive;
use opsis_compile::nodes::image::{
    AdjustImage, BlurImage, CropImage, ImageFormat, PlaceImage, Rect, RedactImage, RedactStyle,
    Resample, ResizeFit, ResizeImage, RotateImage, SpotlightImage, StillFileFormat, Turn,
    area_rect, crop_rect, resized_size, rotated_size,
};

use crate::ImageError;

fn failed<E: std::fmt::Display>(error: E) -> ImageError {
    ImageError::Failed(error.to_string())
}

fn is_deep(image: &DynamicImage) -> bool {
    image.color().bytes_per_pixel() / image.color().channel_count() > 1
}

fn algorithm(quality: Resample) -> ResizeAlg {
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
    if p.width > 0 && p.height > 0 && p.fit == ResizeFit::Cover {
        options = options.fit_into_destination(Some((0.5, 0.5)));
    }
    resized_to(image, w as u32, h as u32, &options)
}

pub fn crop(image: &DynamicImage, p: &CropImage) -> DynamicImage {
    let r = crop_rect(size_of(image), p);
    image.crop_imm(r.x as u32, r.y as u32, r.width as u32, r.height as u32)
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
    let (w, h) = (size.0 as u32, size.1 as u32);
    let mut canvas = ImageBuffer::from_pixel(w, h, background);
    let x = i64::from(w.saturating_sub(image.width()) / 2);
    let y = i64::from(h.saturating_sub(image.height()) / 2);
    imageops::replace(&mut canvas, image, x, y);
    rotate_about_center(
        &canvas,
        (angle as f32).to_radians(),
        Interpolation::Bicubic,
        Border::Constant(background),
    )
}

pub fn background_rgba(color: &str) -> [u8; 4] {
    csscolorparser::parse(color.trim())
        .map(|c| c.to_rgba8())
        .unwrap_or([0, 0, 0, 0])
}

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
    let size = rotated_size(size_of(image), p);
    let [r, g, b, a] = background_rgba(&p.background);
    if is_deep(&out) {
        let wide = |v: u8| u16::from(v) * 257;
        let background = Rgba([wide(r), wide(g), wide(b), wide(a)]);
        DynamicImage::ImageRgba16(tilted(&out.to_rgba16(), size, p.angle, background))
    } else {
        DynamicImage::ImageRgba8(tilted(&out.to_rgba8(), size, p.angle, Rgba([r, g, b, a])))
    }
}

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BRIGHTNESS_SPAN: f32 = 0.4;
const CONTRAST_GAIN: f32 = 3.0;

fn hue_matrix(degrees: f32) -> [[f32; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        [
            0.213 + cos * 0.787 - sin * 0.213,
            0.715 - cos * 0.715 - sin * 0.715,
            0.072 - cos * 0.072 + sin * 0.928,
        ],
        [
            0.213 - cos * 0.213 + sin * 0.143,
            0.715 + cos * 0.285 + sin * 0.140,
            0.072 - cos * 0.072 - sin * 0.283,
        ],
        [
            0.213 - cos * 0.213 - sin * 0.787,
            0.715 - cos * 0.715 + sin * 0.715,
            0.072 + cos * 0.928 + sin * 0.072,
        ],
    ]
}

fn adjust_pixel(rgb: [f32; 3], p: &AdjustImage, hue: &[[f32; 3]; 3]) -> [f32; 3] {
    let contrast = if p.contrast >= 0.0 {
        1.0 + p.contrast as f32 * CONTRAST_GAIN
    } else {
        1.0 + p.contrast as f32
    };
    let mut c = rgb.map(|v| (v - 0.5) * contrast + 0.5 + p.brightness as f32 * BRIGHTNESS_SPAN);
    if p.hue != 0.0 {
        c = [0, 1, 2].map(|i| hue[i][0] * c[0] + hue[i][1] * c[1] + hue[i][2] * c[2]);
    }
    let luma = LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2];
    let saturation = if p.grayscale {
        0.0
    } else {
        1.0 + p.saturation as f32
    };
    c = c.map(|v| luma + (v - luma) * saturation);
    if p.invert {
        c = c.map(|v| 1.0 - v);
    }
    c.map(|v| v.clamp(0.0, 1.0))
}

pub fn adjust(image: &DynamicImage, p: &AdjustImage) -> DynamicImage {
    let hue = hue_matrix(p.hue as f32);
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
        image.fast_blur(p.blur as f32)
    } else {
        image.clone()
    };
    if p.sharpen > 0.0 {
        out = out.unsharpen(p.sharpen as f32, p.threshold as i32);
    }
    out
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
        width as u32,
        height as u32,
        &options,
    )?;
    let mut top: RgbaImage = sized.to_rgba8();
    let opacity = p.opacity.clamp(0.0, 1.0);
    for pixel in top.pixels_mut() {
        pixel.0[3] = (f64::from(pixel.0[3]) * opacity).round() as u8;
    }
    let x = (f64::from(image.width()) * p.x - width / 2.0).round() as i64;
    let y = (f64::from(image.height()) * p.y - height / 2.0).round() as i64;
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
        rect.x as u32,
        rect.y as u32,
        (rect.x + rect.width) as u32,
        (rect.y + rect.height) as u32,
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
    let blurred = gaussian_blur_f32(&part, sigma.max(0.1) as f32);
    imageops::replace(picture, &blurred, i64::from(left), i64::from(top));
}

fn fill<P: Pixel>(picture: &mut Picture<P>, rect: Rect, colour: P) {
    let (left, top, right, bottom) = pixel_bounds(rect);
    for y in top..bottom {
        for x in left..right {
            picture.put_pixel(x, y, colour);
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
            RedactStyle::Solid => fill(&mut picture, rect, colour),
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
        && dx * dx + dy * dy <= r * r
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
        gaussian_blur_f32(&mask, (softness / 2.0) as f32)
    } else {
        mask
    }
}

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
            .fast_blur(p.blur as f32)
            .to_rgba32f()
    } else {
        source.clone()
    };
    let keep = 1.0 - p.dim.clamp(0.0, 1.0) as f32;
    let out = ImageBuffer::from_fn(source.width(), source.height(), |x, y| {
        let lit = mask.get_pixel(x, y).0[0].clamp(0.0, 1.0);
        let [r, g, b, a] = source.get_pixel(x, y).0;
        let [rr, rg, rb, ra] = rest.get_pixel(x, y).0;
        let mix = |near: f32, far: f32| near * lit + far * (1.0 - lit);
        Rgba([
            mix(r, rr * keep),
            mix(g, rg * keep),
            mix(b, rb * keep),
            mix(a, ra),
        ])
    });
    from_float(image, out)
}
