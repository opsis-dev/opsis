#[repr(C)]
pub struct OpsisFilament {
    _private: [u8; 0],
}

#[repr(C)]
pub struct OpsisUniform {
    pub name: *const std::ffi::c_char,
    pub values: *const f32,
    pub count: usize,
    pub columns: usize,
}

#[repr(C)]
pub struct OpsisInput {
    pub pixels: *const u8,
    pub width: u32,
    pub height: u32,
}

unsafe extern "C" {
    pub fn opsis_filament_new(backend: i32) -> *mut OpsisFilament;
    pub fn opsis_filament_free(filament: *mut OpsisFilament);
    pub fn opsis_filament_backend(filament: *const OpsisFilament) -> i32;
    pub fn opsis_filament_material(
        filament: *mut OpsisFilament,
        bytes: *const u8,
        size: usize,
    ) -> i32;
    pub fn opsis_filament_pass(
        filament: *mut OpsisFilament,
        material: i32,
        inputs: *const OpsisInput,
        input_count: usize,
        width: u32,
        height: u32,
        uniforms: *const OpsisUniform,
        uniform_count: usize,
        out: *mut u8,
    ) -> i32;
}

#[repr(C)]
pub struct OpsisMesh {
    pub positions: *const f32,
    pub normals: *const f32,
    pub uvs: *const f32,
    pub vertex_count: u32,
    pub indices: *const u32,
    pub index_count: u32,
    pub transform: [f32; 16],
    pub material: i32,
    pub uniforms: *const OpsisUniform,
    pub uniform_count: usize,
    pub textures: *const OpsisInput,
    pub texture_count: usize,
}

#[repr(C)]
pub struct OpsisLight {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C)]
pub struct OpsisScene {
    pub fov_degrees: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub meshes: *const OpsisMesh,
    pub mesh_count: usize,
    pub lights: *const OpsisLight,
    pub light_count: usize,
    pub reflections: *const u8,
    pub reflections_size: usize,
    pub irradiance: *const f32,
    pub irradiance_bands: u32,
    pub environment_intensity: f32,
    pub samples: u32,
}

unsafe extern "C" {
    pub fn opsis_filament_scene(
        filament: *mut OpsisFilament,
        scene: *const OpsisScene,
        width: u32,
        height: u32,
        out: *mut u8,
    ) -> i32;
}
