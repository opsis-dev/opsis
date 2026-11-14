use opsis_macros::settings;
use opsis_scene::node::{Node, Shape3d};
use opsis_scene::noise::NoiseField;
use opsis_scene::random::random;
use opsis_scene::values::Num;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, Lighting};
use crate::elements::three::{
    STAGE_FOV, SUBJECT_RADIUS, compose, fit_distance, multiply, object, pose_matrix, stage,
    standard, view_pose,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};
use crate::timeline::camera_path::camera_at;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Sphere,
    Ring,
    Box,
}

#[settings(export)]
pub struct Shapes3d {
    #[setting("Shapes", default = 14, range(min = 3, max = 40), step = 1)]
    pub count: u32,
    #[setting(
        "Colours",
        default = vec!["#6366f1".into(), "#a78bfa".into(), "#f472b6".into(), "#38bdf8".into()],
        length(min = 1, max = 8)
    )]
    pub palette: Vec<Colour>,
    #[setting(
        "Arrangement",
        default = 1,
        range(min = 1, max = 100),
        step = 1,
        advanced
    )]
    pub seed: u32,
    #[setting("Speed", default = 1.0, range(min = 0, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        advanced
    )]
    pub camera: CameraPath,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b1020", check = colour)]
    pub background: String,
    #[setting("Depth", default = 1.0, range(min = 0, max = 2), unit = "x", advanced)]
    pub depth: f64,
    #[setting("Shape size", default = 1.0, range(min = 0.25, max = 3), unit = "x")]
    pub size: f64,
    #[setting("Drift", default = 0.45, range(min = 0, max = 1.5), advanced)]
    pub drift: f64,
    #[setting("Spin", default = 1.0, range(min = 0, max = 4), unit = "x", advanced)]
    pub spin: f64,
    #[setting("Roughness", default = 0.4, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting("Metalness", default = 0.1, range(min = 0, max = 1), advanced)]
    pub metalness: f64,
    #[setting(
        "Kinds",
        default = vec![ShapeKind::Sphere, ShapeKind::Ring, ShapeKind::Box],
        length(min = 1, max = 3),
        widget = "multiselect",
        labels = { "sphere": "Spheres", "ring": "Rings", "box": "Boxes" },
        check = { "rule": "unique", "message": "Pick each shape once" },
        advanced
    )]
    pub kinds: Vec<ShapeKind>,
}

pub fn definition() -> ElementMeta {
    meta::<Shapes3d>(
        element(
            ElementKind::Component,
            "shapes-3d",
            "Floating shapes",
            "Soft 3D spheres, rings and rounded boxes drifting slowly.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:torus")),
    )
}

const FOV_HALF: f64 = 30.0 * std::f64::consts::PI / 360.0;
const DEPTH_NEAR: f64 = 0.6;
const DEPTH_FAR: f64 = -3.5;
const SIZE_MIN: f64 = 0.14;
const SIZE_MAX: f64 = 0.42;
const DRIFT_PACE: f64 = 0.006;
const DRIFT_SPIN: f64 = 0.012;
const SPREAD: f64 = 1.05;

struct Placed {
    kind: ShapeKind,
    color: usize,
    at: [f64; 3],
    size: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn place(props: &Shapes3d, aspect: f64) -> Vec<Placed> {
    let stage_distance = SUBJECT_RADIUS / FOV_HALF.sin();
    let colors = props.palette.len();
    let kinds = props.kinds.len().max(1);
    (0..props.count)
        .map(|i| {
            let r = |key: &str| random(format!("{}:{i}:{key}", props.seed));
            let z = (DEPTH_FAR + r("z") * (DEPTH_NEAR - DEPTH_FAR)) * props.depth;
            let half_h = (stage_distance - z) * FOV_HALF.tan() * SPREAD;
            let pick = (r("kind") * opsis_types::numbers::float(kinds)).floor();
            let kind_index =
                usize::try_from(opsis_types::numbers::whole(pick)).unwrap_or(0) % kinds;
            Placed {
                kind: props
                    .kinds
                    .get(kind_index)
                    .copied()
                    .unwrap_or(ShapeKind::Sphere),
                color: if colors == 0 {
                    0
                } else {
                    usize::try_from(i).unwrap_or(0) % colors
                },
                at: [
                    (r("x") * 2.0 - 1.0) * half_h * aspect,
                    (r("y") * 2.0 - 1.0) * half_h,
                    z,
                ],
                size: (SIZE_MIN + r("size") * (SIZE_MAX - SIZE_MIN)) * props.size,
            }
        })
        .collect()
}

const fn shape_of(kind: ShapeKind) -> Shape3d {
    match kind {
        ShapeKind::Sphere => Shape3d::Sphere {
            radius: Num(1.0),
            width_segments: 48,
            height_segments: 32,
        },
        ShapeKind::Ring => Shape3d::Torus {
            radius: Num(0.8),
            tube: Num(0.3),
            radial_segments: 32,
            tubular_segments: 96,
        },
        ShapeKind::Box => Shape3d::RoundedBox {
            size: [Num(1.4), Num(1.4), Num(1.4)],
            segments: 6,
            radius: Num(0.3),
        },
    }
}

pub fn draw(props: &Shapes3d, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let aspect = width / height;
    let frame = cx.frame();
    let t = frame * props.speed;
    let posed = pose_matrix(&view_pose(&camera_at(Some(&props.camera), frame, false)));
    let objects = place(props, aspect)
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let color = props.palette.get(p.color)?;
            let fi = opsis_types::numbers::float(i);
            let drift = |axis: &str| {
                NoiseField::new(&format!("{}:{axis}", props.seed)).at(fi * 7.3, 0.0, t * DRIFT_PACE)
                    * props.drift
            };
            let turn = |axis: &str| {
                NoiseField::new(&format!("{}:r{axis}", props.seed)).at(
                    fi * 3.1,
                    0.0,
                    t * DRIFT_SPIN * props.spin,
                ) * std::f64::consts::PI
            };
            let local = compose(
                [
                    p.at[0] + drift("x"),
                    p.at[1] + drift("y"),
                    p.at[2] + drift("z"),
                ],
                [turn("x"), turn("y"), turn("z")],
                [p.size; 3],
            );
            Some(object(
                shape_of(p.kind),
                &multiply(&posed, &local),
                standard(&color.0, props.roughness, props.metalness),
            ))
        })
        .collect();
    Ok(stage(
        props.lighting,
        &props.background,
        fit_distance(SUBJECT_RADIUS, STAGE_FOV, aspect),
        objects,
    ))
}
