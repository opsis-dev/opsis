use opsis_scene::css::with_background;
use opsis_scene::node::{
    Element, Kind, Lighting3d, Material3d, Node, Object3d, Shape3d, Stage3d, div,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;

use crate::elements::oklch_mix::parsed_or_black;
use crate::elements::shared::Lighting;
use crate::timeline::camera_path::CameraView;

pub const STAGE_FOV: f64 = 30.0;
pub const SUBJECT_RADIUS: f64 = 1.15;
const DEG: f64 = std::f64::consts::PI / 180.0;
const DISTANCE_UNITS: f64 = 3.0;

pub type Matrix = [f64; 16];

pub const IDENTITY: Matrix = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub rotation: [f64; 3],
    pub position: [f64; 3],
    pub scale: f64,
}

#[expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "three.js composes an Euler XYZ rotation this way and names its terms so; the pose has to match it"
)]
pub fn compose(position: [f64; 3], rotation: [f64; 3], scale: [f64; 3]) -> Matrix {
    let (a, b) = (rotation[0].cos(), rotation[0].sin());
    let (c, d) = (rotation[1].cos(), rotation[1].sin());
    let (e, f) = (rotation[2].cos(), rotation[2].sin());
    let (ae, af, be, bf) = (a * e, a * f, b * e, b * f);
    let [sx, sy, sz] = scale;
    [
        c * e * sx,
        (af + be * d) * sx,
        (bf - ae * d) * sx,
        0.0,
        -c * f * sy,
        (ae - bf * d) * sy,
        (be + af * d) * sy,
        0.0,
        d * sz,
        -b * c * sz,
        a * c * sz,
        0.0,
        position[0],
        position[1],
        position[2],
        1.0,
    ]
}

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js multiplies matrices term by term, and the pose has to match it"
)]
pub fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            out[column * 4 + row] = a[row] * b[column * 4]
                + a[4 + row] * b[column * 4 + 1]
                + a[8 + row] * b[column * 4 + 2]
                + a[12 + row] * b[column * 4 + 3];
        }
    }
    out
}

pub fn pose_matrix(pose: &Pose) -> Matrix {
    compose(pose.position, pose.rotation, [pose.scale; 3])
}

pub fn view_pose(view: &CameraView) -> Pose {
    Pose {
        rotation: [view.pitch * DEG, -view.yaw * DEG, 0.0],
        position: [0.0, -view.height, -(view.distance - 1.0) * DISTANCE_UNITS],
        scale: 1.0,
    }
}

pub fn fit_distance(radius: f64, fov: f64, aspect: f64) -> f64 {
    let vertical = fov * std::f64::consts::PI / 360.0;
    let horizontal = (vertical.tan() * aspect).atan();
    radius / vertical.min(horizontal).sin()
}

fn linear(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_color(css: &str) -> [Num; 4] {
    let rgb = parsed_or_black(css);
    [
        Num(linear(rgb.r)),
        Num(linear(rgb.g)),
        Num(linear(rgb.b)),
        Num(rgb.alpha.unwrap_or(1.0)),
    ]
}

pub fn standard(css: &str, roughness: f64, metalness: f64) -> Material3d {
    Material3d {
        color: linear_color(css),
        metalness: Num(metalness),
        roughness: Num(roughness),
    }
}

pub fn object(shape: Shape3d, matrix: &Matrix, material: Material3d) -> Object3d {
    Object3d {
        shape,
        matrix: matrix.map(Num),
        material,
    }
}

pub const fn lighting_of(lighting: Lighting) -> Lighting3d {
    match lighting {
        Lighting::Studio => Lighting3d::Studio,
        Lighting::Soft => Lighting3d::Soft,
        Lighting::Dramatic => Lighting3d::Dramatic,
    }
}

pub fn stage(lighting: Lighting, background: &str, distance: f64, objects: Vec<Object3d>) -> Node {
    let canvas = Element::new(
        Kind::Stage3d(Box::new(Stage3d {
            fov: Num(STAGE_FOV),
            distance: Num(distance),
            lighting: lighting_of(lighting),
            objects,
        })),
        absolute_fill(),
    );
    let fill = if background.trim().is_empty() || background.trim() == "transparent" {
        absolute_fill()
    } else {
        with_background(absolute_fill(), background)
    };
    div(fill).child(canvas).into()
}
