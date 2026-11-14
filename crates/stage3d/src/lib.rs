pub mod geometry;

use opsis_filament::{
    Camera, Environment as SceneEnvironment, Filament, FilamentError, Light, MaterialId, Mesh,
    STANDARD_MATERIAL, Scene, Uniform,
};
use opsis_scene::node::{Lighting3d, Shape3d, Stage3d};
use opsis_types::numbers::narrow;

use crate::geometry::{Geometry, rounded_box, sphere, torus};

const NEAR: f64 = 0.1;
const FAR_SHARE: f64 = 10.0;
const SAMPLES: u32 = 4;
const SH_COLUMNS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Look {
    ambient: f64,
    sky: f64,
    lights: &'static [([f64; 3], f64)],
    reflections: f64,
}

const fn look_of(lighting: Lighting3d) -> Look {
    match lighting {
        Lighting3d::Studio => Look {
            ambient: 0.35,
            sky: 0.0,
            lights: &[
                ([4.0, 5.0, 6.0], 2.2),
                ([-5.0, 2.0, 4.0], 0.7),
                ([0.0, 4.0, -6.0], 1.2),
            ],
            reflections: 0.8,
        },
        Lighting3d::Soft => Look {
            ambient: 0.6,
            sky: 0.9,
            lights: &[([2.0, 4.0, 6.0], 0.9)],
            reflections: 0.5,
        },
        Lighting3d::Dramatic => Look {
            ambient: 0.08,
            sky: 0.0,
            lights: &[([5.0, 6.0, 3.0], 3.2), ([-4.0, 1.0, -5.0], 1.6)],
            reflections: 0.35,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment<'a> {
    pub reflections: &'a [u8],
    pub irradiance: &'a [f32],
}

#[derive(Debug, Default)]
pub struct Materials {
    standard: Option<MaterialId>,
}

impl Materials {
    fn standard(&mut self, filament: &mut Filament) -> Result<MaterialId, FilamentError> {
        if let Some(id) = self.standard {
            return Ok(id);
        }
        let id = filament.material(STANDARD_MATERIAL)?;
        self.standard = Some(id);
        Ok(id)
    }
}

fn geometry_of(shape: &Shape3d) -> Geometry {
    match shape {
        Shape3d::Sphere {
            radius,
            width_segments,
            height_segments,
        } => sphere(radius.0, *width_segments, *height_segments),
        Shape3d::Torus {
            radius,
            tube,
            radial_segments,
            tubular_segments,
        } => torus(radius.0, tube.0, *radial_segments, *tubular_segments),
        Shape3d::RoundedBox {
            size,
            segments,
            radius,
        } => rounded_box([size[0].0, size[1].0, size[2].0], *segments, radius.0),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js normalises the light's position this way, and the light has to match it"
)]
fn direction_from(position: [f64; 3]) -> [f32; 3] {
    let length =
        (position[0] * position[0] + position[1] * position[1] + position[2] * position[2]).sqrt();
    position.map(|c| narrow(-c / length))
}

fn ambient_irradiance(base: &[f32], look: &Look) -> Vec<f32> {
    let mut sh = base.to_vec();
    let added = (look.ambient + look.sky) / (std::f64::consts::PI * look.reflections);
    for channel in sh.iter_mut().take(SH_COLUMNS) {
        *channel += narrow(added);
    }
    sh
}

pub fn render(
    filament: &mut Filament,
    materials: &mut Materials,
    stage: &Stage3d,
    size: (u32, u32),
    environment: Option<Environment<'_>>,
) -> Result<Vec<u8>, FilamentError> {
    let standard = materials.standard(filament)?;
    let look = look_of(stage.lighting);
    let geometries: Vec<Geometry> = stage
        .objects
        .iter()
        .map(|o| geometry_of(&o.shape))
        .collect();
    let settings: Vec<[Vec<f32>; 3]> = stage
        .objects
        .iter()
        .map(|o| {
            [
                o.material.color.iter().map(|c| narrow(c.0)).collect(),
                vec![narrow(o.material.metalness.0)],
                vec![narrow(o.material.roughness.0)],
            ]
        })
        .collect();
    let uniforms: Vec<[Uniform<'_>; 3]> = settings
        .iter()
        .map(|[color, metallic, roughness]| {
            [
                Uniform {
                    name: "baseColor",
                    values: color,
                    columns: 4,
                },
                Uniform {
                    name: "metallic",
                    values: metallic,
                    columns: 1,
                },
                Uniform {
                    name: "roughness",
                    values: roughness,
                    columns: 1,
                },
            ]
        })
        .collect();
    let meshes: Vec<Mesh<'_>> = stage
        .objects
        .iter()
        .zip(geometries.iter().zip(&uniforms))
        .map(|(object, (geometry, uniforms))| Mesh {
            positions: &geometry.positions,
            normals: &geometry.normals,
            uvs: Some(&geometry.uvs),
            indices: &geometry.indices,
            transform: object.matrix.map(|n| narrow(n.0)),
            material: standard,
            uniforms,
            textures: &[],
        })
        .collect();
    let lights: Vec<Light> = look
        .lights
        .iter()
        .map(|(position, intensity)| Light {
            direction: direction_from(*position),
            color: [1.0, 1.0, 1.0],
            intensity: narrow(*intensity),
        })
        .collect();
    let irradiance = environment.map(|e| ambient_irradiance(e.irradiance, &look));
    let scene_environment =
        environment
            .zip(irradiance.as_deref())
            .map(|(e, sh)| SceneEnvironment {
                reflections: e.reflections,
                irradiance: sh,
                intensity: narrow(look.reflections),
            });
    let distance = stage.distance.0;
    let scene = Scene {
        camera: Camera {
            fov_degrees: narrow(stage.fov.0),
            near: narrow(NEAR),
            far: narrow(distance * FAR_SHARE),
            eye: [0.0, 0.0, narrow(distance)],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        },
        meshes: &meshes,
        lights: &lights,
        environment: scene_environment,
        samples: SAMPLES,
    };
    filament.scene(&scene, size)
}
