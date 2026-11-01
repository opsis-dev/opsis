use std::io::Cursor;

use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};

use crate::PrepareError;

pub const LONG_SIDE_CAP: u32 = 4096;
const JPEG_QUALITY: u8 = 90;

fn detail<E: std::fmt::Display>(error: E) -> PrepareError {
    PrepareError::Detail(error.to_string())
}

pub fn texture(bytes: &[u8]) -> Result<Vec<u8>, PrepareError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(detail)?;
    let Some(format) = reader.format() else {
        return Ok(bytes.to_vec());
    };
    let (width, height) = reader.into_dimensions().map_err(detail)?;
    if width.max(height) <= LONG_SIDE_CAP {
        return Ok(bytes.to_vec());
    }
    let image = image::load_from_memory_with_format(bytes, format).map_err(detail)?;
    let smaller = image.resize(LONG_SIDE_CAP, LONG_SIDE_CAP, FilterType::Lanczos3);
    let mut out = Cursor::new(Vec::new());
    if format == ImageFormat::Jpeg {
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
        smaller
            .to_rgb8()
            .write_with_encoder(encoder)
            .map_err(detail)?;
    } else {
        smaller
            .write_to(&mut out, ImageFormat::Png)
            .map_err(detail)?;
    }
    Ok(out.into_inner())
}
