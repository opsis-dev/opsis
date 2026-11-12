use cxx::UniquePtr;

use crate::bridge::ffi;
use crate::path::Path;
use crate::values::{Axis, FontSpec, PointF};

pub struct Typeface {
    pub(crate) inner: UniquePtr<ffi::Typeface>,
}

impl std::fmt::Debug for Typeface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Typeface").finish_non_exhaustive()
    }
}

impl Typeface {
    pub fn from_bytes(bytes: &[u8], index: u32, axes: &[Axis]) -> Option<Self> {
        let inner = ffi::make_typeface(bytes, index, axes);
        (!inner.is_null()).then_some(Self { inner })
    }

    pub fn outlines(
        &self,
        font: FontSpec,
        glyphs: &[u16],
        positions: &[PointF],
        origin: PointF,
    ) -> Path {
        Path {
            inner: ffi::glyph_outlines(&self.inner, font, glyphs, positions, origin),
        }
    }
}
