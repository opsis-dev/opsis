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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
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
    #[error("the setting name {0} can't be passed on")]
    Name(String),
    #[error("the setting {name} has {count} numbers, and a setting holds one to four")]
    Uniform { name: String, count: usize },
    #[error("the pass couldn't be drawn")]
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uniform<'a> {
    pub name: &'a str,
    pub values: &'a [f32],
}

const MAX_UNIFORM_SIZE: usize = 4;

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
        inputs: &[&[u8]],
        (width, height): (u32, u32),
        uniforms: &[Uniform<'_>],
    ) -> Result<Vec<u8>, FilamentError> {
        let expected = picture_size(width, height);
        if let Some(wrong) = inputs.iter().find(|input| input.len() != expected) {
            return Err(FilamentError::Picture {
                width,
                height,
                expected,
                given: wrong.len(),
            });
        }
        if let Some(wrong) = uniforms
            .iter()
            .find(|u| u.values.is_empty() || u.values.len() > MAX_UNIFORM_SIZE)
        {
            return Err(FilamentError::Uniform {
                name: wrong.name.to_owned(),
                count: wrong.values.len(),
            });
        }
        let names = uniforms
            .iter()
            .map(|u| {
                std::ffi::CString::new(u.name).map_err(|_| FilamentError::Name(u.name.to_owned()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let shaped: Vec<ffi::OpsisUniform> = uniforms
            .iter()
            .zip(&names)
            .map(|(u, name)| ffi::OpsisUniform {
                name: name.as_ptr(),
                values: u.values.as_ptr(),
                count: u.values.len(),
            })
            .collect();
        let pointers: Vec<*const u8> = inputs.iter().map(|input| input.as_ptr()).collect();
        let mut out = vec![0u8; expected];
        let drawn = unsafe {
            ffi::opsis_filament_pass(
                self.engine.as_ptr(),
                material.0,
                pointers.as_ptr(),
                pointers.len(),
                width,
                height,
                shaped.as_ptr(),
                shaped.len(),
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
