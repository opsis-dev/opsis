use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Rgb,
    Jpeg,
}

#[derive(Clone)]
pub struct FrameRef {
    bytes: Arc<Vec<u8>>,
    width: u32,
    height: u32,
    serial: u64,
    encoding: Encoding,
}

impl fmt::Debug for FrameRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameRef")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("serial", &self.serial)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

impl FrameRef {
    pub fn empty() -> Self {
        Self {
            bytes: Arc::default(),
            width: 0,
            height: 0,
            serial: 0,
            encoding: Encoding::Rgb,
        }
    }

    pub fn rgb(bytes: Arc<Vec<u8>>, width: u32, height: u32, serial: u64) -> Self {
        let complete =
            u64::try_from(bytes.len()).ok() == Some(u64::from(width) * u64::from(height) * 3);
        if complete {
            Self {
                bytes,
                width,
                height,
                serial,
                encoding: Encoding::Rgb,
            }
        } else {
            Self {
                serial,
                ..Self::empty()
            }
        }
    }

    pub const fn jpeg(bytes: Arc<Vec<u8>>, width: u32, height: u32, serial: u64) -> Self {
        Self {
            bytes,
            width,
            height,
            serial,
            encoding: Encoding::Jpeg,
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) const fn width(&self) -> u32 {
        self.width
    }

    pub(crate) const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) const fn stride(&self) -> u32 {
        match self.encoding {
            Encoding::Rgb => self.width.saturating_mul(3),
            Encoding::Jpeg => 0,
        }
    }

    pub(crate) const fn serial(&self) -> u64 {
        self.serial
    }

    pub(crate) fn is_jpeg(&self) -> bool {
        self.encoding == Encoding::Jpeg
    }
}
