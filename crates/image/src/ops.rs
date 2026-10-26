use std::io::Cursor;

use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{Border, Interpolation, rotate_about_center};
use opsis_compile::nodes::image::{
    AdjustImage, BlurImage, CropImage, ImageFormat, PlaceImage, Resample, ResizeFit, ResizeImage,
    RotateImage, StillFileFormat, Turn, crop_rect, resized_size, rotated_size,
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
