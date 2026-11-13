#[repr(C)]
pub struct OpsisFilament {
    _private: [u8; 0],
}

#[repr(C)]
pub struct OpsisUniform {
    pub name: *const std::ffi::c_char,
    pub values: *const f32,
    pub count: usize,
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
        inputs: *const *const u8,
        input_count: usize,
        width: u32,
        height: u32,
        uniforms: *const OpsisUniform,
        uniform_count: usize,
        out: *mut u8,
    ) -> i32;
}
