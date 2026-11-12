#![allow(
    unsafe_code,
    reason = "opsis-filament is the crate that drives Filament, through the C functions of its sealed C++ wrapper"
)]

mod ffi;

pub const TINT_MATERIAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tint.filamat"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Vulkan,
    OpenGl,
    Metal,
}

impl Backend {
    const fn code(self) -> i32 {
        match self {
            Self::Vulkan => 0,
            Self::OpenGl => 1,
            Self::Metal => 2,
        }
    }

    const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::OpenGl,
            2 => Self::Metal,
            _ => Self::Vulkan,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FilamentError {
    #[error("the {0:?} drawing backend couldn't start")]
    Engine(Backend),
    #[error("the material couldn't be read")]
    Material,
    #[error("a {width}×{height} pass needs {expected} bytes of picture, not {given}")]
    Picture {
        width: u32,
        height: u32,
        expected: usize,
        given: usize,
    },
    #[error("the pass couldn't be drawn")]
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialId(i32);

pub struct Filament {
    engine: std::ptr::NonNull<ffi::OpsisFilament>,
}

impl std::fmt::Debug for Filament {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filament")
            .field("backend", &self.backend())
            .finish_non_exhaustive()
    }
}

fn top_down(bottom_up: &[u8], width: u32) -> Vec<u8> {
    let row = picture_size(width, 1).max(1);
    bottom_up
        .chunks_exact(row)
        .rev()
        .flatten()
        .copied()
        .collect()
}

fn picture_size(width: u32, height: u32) -> usize {
    usize::try_from(u64::from(width) * u64::from(height) * 4).unwrap_or(usize::MAX)
}

impl Filament {
    pub fn new(backend: Backend) -> Result<Self, FilamentError> {
        let made = unsafe { ffi::opsis_filament_new(backend.code()) };
        std::ptr::NonNull::new(made)
            .map(|engine| Self { engine })
            .ok_or(FilamentError::Engine(backend))
    }

    pub fn backend(&self) -> Backend {
        Backend::from_code(unsafe { ffi::opsis_filament_backend(self.engine.as_ptr()) })
    }

    pub fn material(&mut self, package: &[u8]) -> Result<MaterialId, FilamentError> {
        let id = unsafe {
            ffi::opsis_filament_material(self.engine.as_ptr(), package.as_ptr(), package.len())
        };
        if id < 0 {
            return Err(FilamentError::Material);
        }
        Ok(MaterialId(id))
    }

    pub fn pass(
        &mut self,
        material: MaterialId,
        source: &[u8],
        width: u32,
        height: u32,
        values: &[f32],
    ) -> Result<Vec<u8>, FilamentError> {
        let expected = picture_size(width, height);
        if source.len() != expected {
            return Err(FilamentError::Picture {
                width,
                height,
                expected,
                given: source.len(),
            });
        }
        let mut out = vec![0u8; expected];
        let drawn = unsafe {
            ffi::opsis_filament_pass(
                self.engine.as_ptr(),
                material.0,
                source.as_ptr(),
                width,
                height,
                values.as_ptr(),
                values.len(),
                out.as_mut_ptr(),
            )
        };
        if drawn < 0 {
            return Err(FilamentError::Pass);
        }
        Ok(top_down(&out, width))
    }
}

impl Drop for Filament {
    fn drop(&mut self) {
        unsafe { ffi::opsis_filament_free(self.engine.as_ptr()) };
    }
}
