use std::collections::HashMap;

use opsis_filament::{Animation, Filament, ModelId, ModelUse};
use opsis_scene::node::{Material3d, Model3d, Object3d, Shading3d, Shape3d};
use opsis_scene::values::Num;
use opsis_types::numbers::narrow;

const BOUNDS_SAMPLES: u32 = 12;
const SHADOW_SPREAD: f64 = 1.25;
const SHADOW_FLOOR_DEPTH: f64 = 0.6;
const SHADOW_FLOOR_GAP: f64 = 0.01;

#[derive(Debug, Clone, PartialEq)]
struct Loaded {
    id: ModelId,
    animations: Vec<Animation>,
}

#[derive(Debug, Default)]
pub struct Models {
    loaded: HashMap<String, Option<Loaded>>,
}

pub type Files<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub model: ModelUse,
    pub shadow: Option<Object3d>,
}

fn clip_of(animations: &[Animation], wanted: &str) -> Option<(u32, f64)> {
    let wanted = wanted.trim().to_lowercase();
    let index = animations
        .iter()
        .position(|a| a.name.trim().to_lowercase() == wanted)
        .or_else(|| (!animations.is_empty()).then_some(0))?;
    Some((
        u32::try_from(index).ok()?,
        f64::from(animations[index].duration),
    ))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js wraps a looping clip's time this way, and the pose has to match it"
)]
fn clip_time(time: f64, duration: f64, looped: bool) -> f64 {
    if duration <= 0.0 {
        return 0.0;
    }
    if looped {
        if time >= duration {
            time - duration * (time / duration).floor()
        } else {
            time
        }
    } else {
        time.min(duration)
    }
}

type Bounds = ([f64; 3], [f64; 3]);

fn animated_bounds(
    filament: &mut Filament,
    id: ModelId,
    clip: Option<(u32, f64)>,
    looped: bool,
) -> Option<Bounds> {
    let samples = match clip {
        Some((_, duration)) if duration > 0.0 => BOUNDS_SAMPLES,
        _ => 1,
    };
    let mut union: Option<Bounds> = None;
    for i in 0..samples {
        let (animation, time) = clip.map_or((None, 0.0), |(index, duration)| {
            let at = duration * f64::from(i) / f64::from((samples - 1).max(1));
            (Some(index), clip_time(at, duration, looped))
        });
        let Some((low, high)) = filament.model_bounds(id, animation, narrow(time)) else {
            continue;
        };
        let (low, high) = (low.map(f64::from), high.map(f64::from));
        union = Some(union.map_or((low, high), |(a, b)| {
            (
                [a[0].min(low[0]), a[1].min(low[1]), a[2].min(low[2])],
                [b[0].max(high[0]), b[1].max(high[1]), b[2].max(high[2])],
            )
        }));
    }
    union
}

fn multiplied(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            out[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    out
}

fn shadow(floor: f64, width: f64, opacity: f64, texture: u32) -> Object3d {
    let angle = -std::f64::consts::FRAC_PI_2;
    let (c, s) = (angle.cos(), angle.sin());
    let matrix = [
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        c,
        s,
        0.0,
        0.0,
        -s,
        c,
        0.0,
        0.0,
        floor - SHADOW_FLOOR_GAP,
        0.0,
        1.0,
    ];
    Object3d {
        shape: Shape3d::Plane {
            width: Num(width * SHADOW_SPREAD),
            height: Num(width * SHADOW_FLOOR_DEPTH * SHADOW_SPREAD),
        },
        matrix: matrix.map(Num),
        material: Material3d {
            shading: Shading3d::Unlit,
            double_sided: false,
            color: [Num(1.0); 4],
            metalness: Num(0.0),
            roughness: Num(1.0),
            opacity: Num(opacity),
            texture: Some(texture),
        },
    }
}

impl Models {
    fn load(&mut self, filament: &mut Filament, source: &str, files: Files<'_>) -> Option<Loaded> {
        self.loaded
            .entry(source.to_owned())
            .or_insert_with(|| {
                let bytes = files(source)?;
                let bytes = crate::glb::with_unit_bounds(&bytes).unwrap_or(bytes);
                let id = filament
                    .model(&bytes)
                    .map_err(|e| log::warn!("the 3D model {source}: {e}"))
                    .ok()?;
                Some(Loaded {
                    id,
                    animations: filament.animations(id),
                })
            })
            .clone()
    }

    pub fn place(
        &mut self,
        filament: &mut Filament,
        model: &Model3d,
        files: Files<'_>,
        glow: u32,
    ) -> Option<Placed> {
        let loaded = self.load(filament, &model.source.0, files)?;
        let clip = clip_of(&loaded.animations, &model.animation);
        let (low, high) = animated_bounds(filament, loaded.id, clip, model.looped)?;
        let center = [0, 1, 2].map(|k| f64::midpoint(low[k], high[k]));
        let extent = [0, 1, 2].map(|k| high[k] - low[k]);
        let largest = extent[0].max(extent[1]).max(extent[2]);
        let s = model.size.0 / if largest > 0.0 { largest } else { 1.0 };
        let fit = [
            s,
            0.0,
            0.0,
            0.0,
            0.0,
            s,
            0.0,
            0.0,
            0.0,
            0.0,
            s,
            0.0,
            -center[0] * s,
            -center[1] * s,
            -center[2] * s,
            1.0,
        ];
        let root = multiplied(&model.matrix.map(|n| n.0), &fit);
        let (animation, time) = clip.map_or((None, 0.0), |(index, duration)| {
            (Some(index), clip_time(model.time.0, duration, model.looped))
        });
        let floor = (low[1] - center[1]) * s;
        let width = (extent[0] * s).max(extent[2] * s);
        Some(Placed {
            model: ModelUse {
                model: loaded.id,
                transform: root.map(narrow),
                animation,
                time: narrow(time),
            },
            shadow: model
                .shadow
                .map(|opacity| shadow(floor, width, opacity.0, glow)),
        })
    }
}
