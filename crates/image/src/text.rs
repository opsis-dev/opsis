use ab_glyph::{Font, FontVec, PxScale, ScaleFont, VariableFont, point};
use image::{Rgba, Rgba32FImage};

use crate::ImageError;

const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";
const WOFF_SIGNATURE: &[u8; 4] = b"wOFF";

fn unreadable<E>(_: E) -> ImageError {
    ImageError::Unreadable("the font file can't be read".to_owned())
}

pub fn load_font(bytes: Vec<u8>, weight: f64) -> Result<FontVec, ImageError> {
    let sfnt = match bytes.get(..4) {
        Some(signature) if signature == WOFF2_SIGNATURE => {
            wuff::decompress_woff2(&bytes).map_err(unreadable)?
        }
        Some(signature) if signature == WOFF_SIGNATURE => {
            wuff::decompress_woff1(&bytes).map_err(unreadable)?
        }
        _ => bytes,
    };
    let mut font = FontVec::try_from_vec(sfnt).map_err(unreadable)?;
    font.set_variation(b"wght", weight as f32);
    Ok(font)
}

pub struct TextBox {
    pub width: f64,
    pub ascent: f64,
    pub height: f64,
}

pub fn measure(font: &FontVec, size: f64, text: &str) -> TextBox {
    let scaled = font.as_scaled(PxScale::from(size as f32));
    let mut width = 0.0f32;
    let mut previous = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(before) = previous {
            width += scaled.kern(before, id);
        }
        width += scaled.h_advance(id);
        previous = Some(id);
    }
    TextBox {
        width: f64::from(width),
        ascent: f64::from(scaled.ascent()),
        height: f64::from(scaled.ascent() - scaled.descent()),
    }
}

pub fn draw_text(
    canvas: &mut Rgba32FImage,
    font: &FontVec,
    size: f64,
    origin: (f64, f64),
    text: &str,
    colour: [f32; 4],
) {
    let scaled = font.as_scaled(PxScale::from(size as f32));
    let baseline = origin.1 as f32 + scaled.ascent();
    let mut caret = origin.0 as f32;
    let mut previous = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(before) = previous {
            caret += scaled.kern(before, id);
        }
        let glyph = id.with_scale_and_position(scaled.scale(), point(caret, baseline));
        caret += scaled.h_advance(id);
        previous = Some(id);
        let Some(outlined) = font.outline_glyph(glyph) else {
            continue;
        };
        let bounds = outlined.px_bounds();
        outlined.draw(|gx, gy, coverage| {
            let x = i64::from(gx) + bounds.min.x as i64;
            let y = i64::from(gy) + bounds.min.y as i64;
            let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
                return;
            };
            if x >= canvas.width() || y >= canvas.height() {
                return;
            }
            let pixel = canvas.get_pixel_mut(x, y);
            let mut top = colour;
            top[3] *= coverage.clamp(0.0, 1.0);
            *pixel = Rgba(crate::ops::over(top, pixel.0));
        });
    }
}
