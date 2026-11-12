use cxx::UniquePtr;

use crate::bridge::ffi;

pub struct Image {
    pub(crate) inner: UniquePtr<ffi::Image>,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl Image {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let inner = ffi::decode_image(bytes);
        (!inner.is_null()).then_some(Self { inner })
    }

    pub fn from_rgba(width: u32, height: u32, pixels: &[u8]) -> Option<Self> {
        let inner = ffi::image_from_rgba(
            i32::try_from(width).ok()?,
            i32::try_from(height).ok()?,
            pixels,
        );
        (!inner.is_null()).then_some(Self { inner })
    }

    pub fn width(&self) -> u32 {
        u32::try_from(self.inner.image_width()).unwrap_or(0)
    }

    pub fn height(&self) -> u32 {
        u32::try_from(self.inner.image_height()).unwrap_or(0)
    }
}
