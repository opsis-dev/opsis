use cxx::UniquePtr;

use crate::bridge::ffi;

pub struct Lottie {
    pub(crate) inner: UniquePtr<ffi::Lottie>,
}

impl std::fmt::Debug for Lottie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lottie")
            .field("size", &self.size())
            .field("fps", &self.fps())
            .finish_non_exhaustive()
    }
}

impl Lottie {
    pub fn from_json(json: &[u8]) -> Option<Self> {
        let inner = ffi::make_lottie(json);
        (!inner.is_null()).then_some(Self { inner })
    }

    pub fn size(&self) -> (f32, f32) {
        (self.inner.lottie_width(), self.inner.lottie_height())
    }

    pub fn fps(&self) -> f64 {
        self.inner.lottie_fps()
    }

    pub fn frames(&self) -> (f64, f64) {
        (self.inner.lottie_in_point(), self.inner.lottie_out_point())
    }

    pub fn seek(&mut self, frame: f64) {
        self.inner.pin_mut().seek(frame);
    }
}
