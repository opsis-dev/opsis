mod glb;
mod model;

use opsis_filament::{
    BASIC_MATERIAL, Camera, Environment as SceneEnvironment, Filament, FilamentError, Input, Light,
    MaterialId, Mesh, STANDARD_DOUBLE_MATERIAL, STANDARD_MATERIAL, Scene, Uniform,
};
use opsis_geometry::mesh_of;
use opsis_scene::node::{Lighting3d, Shading3d, Stage3d, Texture3d};
use opsis_scene::values::Num;
use opsis_types::numbers::narrow;

pub use crate::model::{Files, Models};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picture<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default)]
pub struct Materials {
    models: Models,
    standard: Option<MaterialId>,
    standard_double: Option<MaterialId>,
    basic: Option<MaterialId>,
}

fn loaded(
    slot: &mut Option<MaterialId>,
    filament: &mut Filament,
    package: &[u8],
) -> Result<MaterialId, FilamentError> {
    if let Some(id) = *slot {
        return Ok(id);
    }
    let id = filament.material(package)?;
    *slot = Some(id);
    Ok(id)
}

impl Materials {
    fn of(
        &mut self,
        filament: &mut Filament,
        shading: Shading3d,
        double_sided: bool,
    ) -> Result<MaterialId, FilamentError> {
        match (shading, double_sided) {
            (Shading3d::Lit, false) => loaded(&mut self.standard, filament, STANDARD_MATERIAL),
            (Shading3d::Lit, true) => loaded(
                &mut self.standard_double,
                filament,
                STANDARD_DOUBLE_MATERIAL,
            ),
            (Shading3d::Unlit, _) => loaded(&mut self.basic, filament, BASIC_MATERIAL),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pixels {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

fn copied(picture: &Picture<'_>) -> Pixels {
    Pixels {
        rgba: picture.rgba.to_vec(),
        width: picture.width,
        height: picture.height,
    }
}

const WHITE: Input<'static> = Input {
    pixels: &[255, 255, 255, 255],
    width: 1,
    height: 1,
};

#[expect(
    clippy::imprecise_flops,
    reason = "the glow is a gradient stop by stop, and the recorded goldens hold what that draws"
)]
fn glow(side: u32, alpha: f64) -> Pixels {
    let half = f64::from(side) / 2.0;
    let mut rgba = Vec::with_capacity(usize::try_from(side).unwrap_or(0).pow(2) * 4);
    for y in 0..side {
        for x in 0..side {
            let dx = f64::from(x) + 0.5 - half;
            let dy = f64::from(y) + 0.5 - half;
            let t = ((dx * dx + dy * dy).sqrt() / half).min(1.0);
            let a = (alpha * (1.0 - t) * 255.0).round();
            rgba.extend([
                0,
                0,
                0,
                u8::try_from(opsis_types::numbers::whole(a)).unwrap_or(255),
            ]);
        }
    }
    Pixels {
        rgba,
        width: side,
        height: side,
    }
}

fn texture_pixels(texture: &Texture3d, picture: Option<&Picture<'_>>) -> Option<Pixels> {
    match texture {
        Texture3d::Picture(_) => picture.map(copied),
        Texture3d::Glow { side, alpha } => Some(glow(*side, alpha.0)),
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

const SHADOW_SIDE: u32 = 256;
const SHADOW_ALPHA: f64 = 0.55;

pub fn render(
    filament: &mut Filament,
    materials: &mut Materials,
    stage: &Stage3d,
    size: (u32, u32),
    pictures: &[Option<Picture<'_>>],
    files: Files<'_>,
    environment: Option<Environment<'_>>,
) -> Result<Vec<u8>, FilamentError> {
    let look = look_of(stage.lighting);
    let mut objects = stage.objects.clone();
    let mut texture_list = stage.textures.clone();
    let glow = u32::try_from(texture_list.len()).unwrap_or(u32::MAX);
    let mut uses = Vec::new();
    for model in &stage.models {
        let Some(placed) = materials.models.place(filament, model, files, glow) else {
            continue;
        };
        uses.push(placed.model);
        objects.extend(placed.shadow);
    }
    if objects.len() > stage.objects.len() {
        texture_list.push(Texture3d::Glow {
            side: SHADOW_SIDE,
            alpha: Num(SHADOW_ALPHA),
        });
    }
    let textures: Vec<Option<Pixels>> = texture_list
        .iter()
        .enumerate()
        .map(|(i, texture)| texture_pixels(texture, pictures.get(i).and_then(Option::as_ref)))
        .collect();
    let mut drawn = Vec::new();
    for object in &objects {
        let texture = match object.material.texture {
            None => None,
            Some(index) => {
                let found = usize::try_from(index)
                    .ok()
                    .and_then(|i| textures.get(i))
                    .and_then(Option::as_ref);
                if found.is_none() {
                    continue;
                }
                found
            }
        };
        let material = materials.of(
            filament,
            object.material.shading,
            object.material.double_sided,
        )?;
        drawn.push((object, mesh_of(&object.shape), texture, material));
    }
    let settings: Vec<[Vec<f32>; 5]> = drawn
        .iter()
        .map(|(o, _, texture, _)| {
            [
                o.material.color.iter().map(|c| narrow(c.0)).collect(),
                vec![narrow(o.material.metalness.0)],
                vec![narrow(o.material.roughness.0)],
                vec![narrow(o.material.opacity.0)],
                vec![if texture.is_some() { 1.0 } else { 0.0 }],
            ]
        })
        .collect();
    let uniforms: Vec<[Uniform<'_>; 5]> = settings
        .iter()
        .map(|[color, metallic, roughness, opacity, textured]| {
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
                Uniform {
                    name: "opacity",
                    values: opacity,
                    columns: 1,
                },
                Uniform {
                    name: "textured",
                    values: textured,
                    columns: 1,
                },
            ]
        })
        .collect();
    let inputs: Vec<Vec<Input<'_>>> = drawn
        .iter()
        .map(|(_, _, texture, _)| {
            vec![texture.map_or(WHITE, |pixels| Input {
                pixels: &pixels.rgba,
                width: pixels.width,
                height: pixels.height,
            })]
        })
        .collect();
    let meshes: Vec<Mesh<'_>> = drawn
        .iter()
        .zip(uniforms.iter().zip(&inputs))
        .map(
            |((object, geometry, _, material), (uniforms, inputs))| Mesh {
                positions: &geometry.positions,
                normals: &geometry.normals,
                uvs: Some(&geometry.uvs),
                indices: &geometry.indices,
                transform: object.matrix.map(|n| narrow(n.0)),
                material: *material,
                uniforms,
                textures: inputs,
            },
        )
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
        models: &uses,
        lights: &lights,
        environment: scene_environment,
        samples: SAMPLES,
    };
    filament.scene(&scene, size)
}
