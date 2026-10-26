use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageReader};
use moxcms::{ColorProfile, Layout, TransformOptions};

use crate::ImageError;

fn decoded(bytes: &[u8]) -> Result<(DynamicImage, Option<Vec<u8>>), ImageError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| ImageError::Unreadable(e.to_string()))?;
    if reader.format().is_none() {
        return Err(ImageError::NotAnImage);
    }
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| ImageError::Unreadable(e.to_string()))?;
    let orientation = decoder.orientation().ok();
    let icc = decoder.icc_profile().ok().flatten();
    let mut image =
        DynamicImage::from_decoder(decoder).map_err(|e| ImageError::Unreadable(e.to_string()))?;
    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }
    Ok((image, icc))
}

fn is_srgb(profile: &ColorProfile) -> bool {
    let srgb = ColorProfile::new_srgb();
    profile.red_colorant == srgb.red_colorant
        && profile.green_colorant == srgb.green_colorant
        && profile.blue_colorant == srgb.blue_colorant
        && profile.color_space == srgb.color_space
}

fn to_srgb(image: DynamicImage, icc: &[u8]) -> Result<DynamicImage, ImageError> {
    let Ok(source) = ColorProfile::new_from_slice(icc) else {
        return Ok(image);
    };
    if is_srgb(&source) {
        return Ok(image);
    }
    let srgb = ColorProfile::new_srgb();
    let fail = |e: moxcms::CmsError| ImageError::Unreadable(e.to_string());
    if image.color().bytes_per_pixel() / image.color().channel_count() > 1 {
        let mut pixels = image.to_rgba16();
        let transform = source
            .create_transform_16bit(
                Layout::Rgba,
                &srgb,
                Layout::Rgba,
                TransformOptions::default(),
            )
            .map_err(fail)?;
        let input = pixels.as_raw().clone();
        transform.transform(&input, &mut pixels).map_err(fail)?;
        return Ok(DynamicImage::ImageRgba16(pixels));
    }
    let alpha = image.color().has_alpha();
    let mut pixels = image.to_rgba8();
    let transform = source
        .create_transform_8bit(
            Layout::Rgba,
            &srgb,
            Layout::Rgba,
            TransformOptions::default(),
        )
        .map_err(fail)?;
    let input = pixels.as_raw().clone();
    transform.transform(&input, &mut pixels).map_err(fail)?;
    let converted = DynamicImage::ImageRgba8(pixels);
    Ok(if alpha {
        converted
    } else {
        DynamicImage::ImageRgb8(converted.to_rgb8())
    })
}

pub fn load_image(bytes: &[u8]) -> Result<DynamicImage, ImageError> {
    let (image, icc) = decoded(bytes)?;
    match icc {
        Some(icc) => to_srgb(image, &icc),
        None => Ok(image),
    }
}
