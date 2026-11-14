#![allow(
    unsafe_code,
    reason = "opsis-filament is the crate that drives Filament, through the C functions of its sealed C++ wrapper"
)]

mod ffi;

pub const TINT_MATERIAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tint.filamat"));
pub const STANDARD_MATERIAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/standard.filamat"));

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
    #[error(
        "the setting {name} has {count} numbers, which isn't a whole number of {columns}-number values"
    )]
    Uniform {
        name: String,
        count: usize,
        columns: usize,
    },
    #[error("the pass couldn't be drawn")]
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uniform<'a> {
    pub name: &'a str,
    pub values: &'a [f32],
    pub columns: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input<'a> {
    pub pixels: &'a [u8],
    pub width: u32,
    pub height: u32,
}

const ARRAY_COLUMNS: [usize; 3] = [1, 2, 4];

fn shaped_well(uniform: &Uniform<'_>) -> bool {
    let (count, columns) = (uniform.values.len(), uniform.columns);
    if count == 0 || columns == 0 || count % columns != 0 {
        return false;
    }
    count == columns && columns <= MAX_UNIFORM_SIZE || ARRAY_COLUMNS.contains(&columns)
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
        inputs: &[Input<'_>],
        (width, height): (u32, u32),
        uniforms: &[Uniform<'_>],
    ) -> Result<Vec<u8>, FilamentError> {
        let expected = picture_size(width, height);
        if let Some(wrong) = inputs
            .iter()
            .find(|input| input.pixels.len() != picture_size(input.width, input.height))
        {
            return Err(FilamentError::Picture {
                width: wrong.width,
                height: wrong.height,
                expected: picture_size(wrong.width, wrong.height),
                given: wrong.pixels.len(),
            });
        }
        if let Some(wrong) = uniforms.iter().find(|u| !shaped_well(u)) {
            return Err(FilamentError::Uniform {
                name: wrong.name.to_owned(),
                count: wrong.values.len(),
                columns: wrong.columns,
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
                columns: u.columns,
            })
            .collect();
        let pointers: Vec<ffi::OpsisInput> = inputs
            .iter()
            .map(|input| ffi::OpsisInput {
                pixels: input.pixels.as_ptr(),
                width: input.width,
                height: input.height,
            })
            .collect();
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mesh<'a> {
    pub positions: &'a [f32],
    pub normals: &'a [f32],
    pub uvs: Option<&'a [f32]>,
    pub indices: &'a [u32],
    pub transform: [f32; 16],
    pub material: MaterialId,
    pub uniforms: &'a [Uniform<'a>],
    pub textures: &'a [Input<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub fov_degrees: f32,
    pub near: f32,
    pub far: f32,
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment<'a> {
    pub reflections: &'a [u8],
    pub irradiance: &'a [f32],
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scene<'a> {
    pub camera: Camera,
    pub meshes: &'a [Mesh<'a>],
    pub lights: &'a [Light],
    pub environment: Option<Environment<'a>>,
    pub samples: u32,
}

const SH_COLUMNS: usize = 3;

impl Filament {
    pub fn scene(
        &mut self,
        scene: &Scene<'_>,
        (width, height): (u32, u32),
    ) -> Result<Vec<u8>, FilamentError> {
        let mut names = Vec::new();
        for mesh in scene.meshes {
            if mesh.positions.len() != mesh.normals.len() || mesh.positions.len() % 3 != 0 {
                return Err(FilamentError::Pass);
            }
            for u in mesh.uniforms {
                if !shaped_well(u) {
                    return Err(FilamentError::Uniform {
                        name: u.name.to_owned(),
                        count: u.values.len(),
                        columns: u.columns,
                    });
                }
                names.push(
                    std::ffi::CString::new(u.name)
                        .map_err(|_| FilamentError::Name(u.name.to_owned()))?,
                );
            }
        }
        let mut owned_names = names.iter();
        let uniforms: Vec<Vec<ffi::OpsisUniform>> = scene
            .meshes
            .iter()
            .map(|mesh| {
                mesh.uniforms
                    .iter()
                    .map(|u| ffi::OpsisUniform {
                        name: owned_names.next().map_or(std::ptr::null(), |n| n.as_ptr()),
                        values: u.values.as_ptr(),
                        count: u.values.len(),
                        columns: u.columns,
                    })
                    .collect()
            })
            .collect();
        let textures: Vec<Vec<ffi::OpsisInput>> = scene
            .meshes
            .iter()
            .map(|mesh| {
                mesh.textures
                    .iter()
                    .map(|t| ffi::OpsisInput {
                        pixels: t.pixels.as_ptr(),
                        width: t.width,
                        height: t.height,
                    })
                    .collect()
            })
            .collect();
        let meshes: Vec<ffi::OpsisMesh> = scene
            .meshes
            .iter()
            .zip(uniforms.iter().zip(&textures))
            .map(|(mesh, (u, t))| ffi::OpsisMesh {
                positions: mesh.positions.as_ptr(),
                normals: mesh.normals.as_ptr(),
                uvs: mesh.uvs.map_or(std::ptr::null(), <[f32]>::as_ptr),
                vertex_count: u32::try_from(mesh.positions.len() / 3).unwrap_or(0),
                indices: mesh.indices.as_ptr(),
                index_count: u32::try_from(mesh.indices.len()).unwrap_or(0),
                transform: mesh.transform,
                material: mesh.material.0,
                uniforms: u.as_ptr(),
                uniform_count: u.len(),
                textures: t.as_ptr(),
                texture_count: t.len(),
            })
            .collect();
        let lights: Vec<ffi::OpsisLight> = scene
            .lights
            .iter()
            .map(|l| ffi::OpsisLight {
                direction: l.direction,
                color: l.color,
                intensity: l.intensity,
            })
            .collect();
        let (reflections, reflections_size, irradiance, bands, intensity) = match scene.environment
        {
            Some(e) => {
                let coefficients = e.irradiance.len() / SH_COLUMNS;
                let bands = (1..=3u32)
                    .rev()
                    .find(|b| usize::try_from(b * b).unwrap_or(usize::MAX) <= coefficients)
                    .unwrap_or(0);
                (
                    e.reflections.as_ptr(),
                    e.reflections.len(),
                    e.irradiance.as_ptr(),
                    bands,
                    e.intensity,
                )
            }
            None => (std::ptr::null(), 0, std::ptr::null(), 0, 0.0),
        };
        let described = ffi::OpsisScene {
            fov_degrees: scene.camera.fov_degrees,
            near_plane: scene.camera.near,
            far_plane: scene.camera.far,
            eye: scene.camera.eye,
            target: scene.camera.target,
            up: scene.camera.up,
            meshes: meshes.as_ptr(),
            mesh_count: meshes.len(),
            lights: lights.as_ptr(),
            light_count: lights.len(),
            reflections,
            reflections_size,
            irradiance,
            irradiance_bands: bands,
            environment_intensity: intensity,
            samples: scene.samples,
        };
        let mut out = vec![0u8; picture_size(width, height)];
        let drawn = unsafe {
            ffi::opsis_filament_scene(
                self.engine.as_ptr(),
                &raw const described,
                width,
                height,
                out.as_mut_ptr(),
            )
        };
        if drawn < 0 {
            return Err(FilamentError::Pass);
        }
        Ok(out)
    }
}
