use cxx::UniquePtr;

use crate::bridge::ffi;
use crate::values::{Matrix33, PointF, RectF, RoundRect};

pub struct Path {
    pub(crate) inner: UniquePtr<ffi::Path>,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Path")
            .field("bounds", &self.bounds())
            .finish_non_exhaustive()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Path {
    pub fn new() -> Self {
        Self {
            inner: ffi::make_path(),
        }
    }

    pub fn move_to(&mut self, point: PointF) {
        self.inner.pin_mut().move_to(point);
    }

    pub fn line_to(&mut self, point: PointF) {
        self.inner.pin_mut().line_to(point);
    }

    pub fn quad_to(&mut self, control: PointF, point: PointF) {
        self.inner.pin_mut().quad_to(control, point);
    }

    pub fn cubic_to(&mut self, first: PointF, second: PointF, point: PointF) {
        self.inner.pin_mut().cubic_to(first, second, point);
    }

    pub fn close(&mut self) {
        self.inner.pin_mut().close();
    }

    pub fn add_rect(&mut self, rect: RectF) {
        self.inner.pin_mut().add_rect(rect);
    }

    pub fn add_round_rect(&mut self, rect: RoundRect) {
        self.inner.pin_mut().add_round_rect(rect);
    }

    pub fn add_oval(&mut self, rect: RectF) {
        self.inner.pin_mut().add_oval(rect);
    }

    pub fn add_path(&mut self, other: &Self, matrix: Matrix33) {
        self.inner.pin_mut().add_path(&other.inner, matrix);
    }

    pub fn set_even_odd(&mut self, even_odd: bool) {
        self.inner.pin_mut().set_even_odd(even_odd);
    }

    pub fn bounds(&self) -> RectF {
        self.inner.bounds()
    }
}
