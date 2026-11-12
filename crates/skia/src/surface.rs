use cxx::UniquePtr;

use crate::bridge::ffi;
use crate::image::Image;
use crate::paint::{ImageFilter, Paint};
use crate::path::Path;
use crate::text::Typeface;
use crate::values::{FontSpec, Matrix44, PointF, RectF, Rgba, RoundRect, Sampling};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SurfaceError {
    #[error("a {width}×{height} picture can't be drawn")]
    Size { width: u32, height: u32 },
}

pub struct Surface {
    inner: UniquePtr<ffi::Surface>,
    base: Option<Matrix44>,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("width", &self.inner.width())
            .field("height", &self.inner.height())
            .finish()
    }
}

impl Surface {
    pub fn new(width: u32, height: u32) -> Result<Self, SurfaceError> {
        let fail = SurfaceError::Size { width, height };
        let (Ok(w), Ok(h)) = (i32::try_from(width), i32::try_from(height)) else {
            return Err(fail);
        };
        let inner = ffi::make_surface(w, h);
        if inner.is_null() {
            return Err(fail);
        }
        Ok(Self { inner, base: None })
    }

    pub fn with_base(mut self, base: Matrix44) -> Self {
        self.base = Some(base);
        self.reset_matrix();
        self
    }

    pub fn width(&self) -> u32 {
        u32::try_from(self.inner.width()).unwrap_or(0)
    }

    pub fn height(&self) -> u32 {
        u32::try_from(self.inner.height()).unwrap_or(0)
    }

    pub fn clear(&mut self, color: Rgba) {
        self.inner.pin_mut().clear(color);
    }

    pub fn fill_rect(&mut self, rect: RectF, color: Rgba) {
        self.inner.pin_mut().fill_rect(rect, color);
    }

    pub fn rgba(&self) -> Vec<u8> {
        self.inner.read_rgba()
    }

    pub fn png(&self) -> Vec<u8> {
        self.inner.encode_png()
    }

    pub fn snapshot(&self) -> Option<Image> {
        let inner = self.inner.snapshot();
        (!inner.is_null()).then_some(Image { inner })
    }

    pub fn save(&mut self) -> i32 {
        self.inner.pin_mut().save()
    }

    pub fn restore(&mut self) {
        self.inner.pin_mut().restore();
    }

    pub fn restore_to(&mut self, count: i32) {
        self.inner.pin_mut().restore_to(count);
    }

    pub fn save_layer(&mut self, bounds: Option<RectF>, paint: &Paint) -> i32 {
        let (rect, bounded) = bounds.map_or((RectF::new(0.0, 0.0, 0.0, 0.0), false), |b| (b, true));
        self.inner.pin_mut().save_layer(rect, bounded, &paint.inner)
    }

    pub fn save_layer_backdrop(
        &mut self,
        bounds: Option<RectF>,
        paint: &Paint,
        backdrop: &ImageFilter,
    ) -> i32 {
        let (rect, bounded) = bounds.map_or((RectF::new(0.0, 0.0, 0.0, 0.0), false), |b| (b, true));
        self.inner
            .pin_mut()
            .save_layer_backdrop(rect, bounded, &paint.inner, &backdrop.inner)
    }

    pub fn concat(&mut self, matrix: Matrix44) {
        self.inner.pin_mut().concat(matrix);
    }

    pub fn reset_matrix(&mut self) {
        self.inner.pin_mut().reset_matrix();
        if let Some(base) = self.base {
            self.inner.pin_mut().concat(base);
        }
    }

    pub fn clip_rect(&mut self, rect: RectF, anti_alias: bool) {
        self.inner.pin_mut().clip_rect(rect, anti_alias, false);
    }

    pub fn clip_outside_rect(&mut self, rect: RectF, anti_alias: bool) {
        self.inner.pin_mut().clip_rect(rect, anti_alias, true);
    }

    pub fn clip_round_rect(&mut self, rect: RoundRect, anti_alias: bool) {
        self.inner.pin_mut().clip_round_rect(rect, anti_alias);
    }

    pub fn clip_path(&mut self, path: &Path, anti_alias: bool) {
        self.inner.pin_mut().clip_path(&path.inner, anti_alias);
    }

    pub fn draw_rect(&mut self, rect: RectF, paint: &Paint) {
        self.inner.pin_mut().draw_rect(rect, &paint.inner);
    }

    pub fn draw_round_rect(&mut self, rect: RoundRect, paint: &Paint) {
        self.inner.pin_mut().draw_round_rect(rect, &paint.inner);
    }

    pub fn draw_nested_round_rect(&mut self, outer: RoundRect, inner: RoundRect, paint: &Paint) {
        self.inner
            .pin_mut()
            .draw_nested_round_rect(outer, inner, &paint.inner);
    }

    pub fn draw_path(&mut self, path: &Path, paint: &Paint) {
        self.inner.pin_mut().draw_path(&path.inner, &paint.inner);
    }

    pub fn draw_paint(&mut self, paint: &Paint) {
        self.inner.pin_mut().draw_paint(&paint.inner);
    }

    pub fn draw_glyphs(
        &mut self,
        typeface: &Typeface,
        font: FontSpec,
        (glyphs, positions): (&[u16], &[PointF]),
        origin: PointF,
        paint: &Paint,
    ) {
        self.inner.pin_mut().draw_glyphs(
            &typeface.inner,
            font,
            glyphs,
            positions,
            origin,
            &paint.inner,
        );
    }

    pub fn draw_lottie(&mut self, lottie: &crate::lottie::Lottie, target: RectF) {
        self.inner.pin_mut().draw_lottie(&lottie.inner, target);
    }

    pub fn draw_image_rect(
        &mut self,
        image: &Image,
        (source, target): (RectF, RectF),
        sampling: Sampling,
        paint: &Paint,
    ) {
        self.inner
            .pin_mut()
            .draw_image_rect(&image.inner, source, target, sampling, &paint.inner);
    }
}
