use opsis_macros::{choice, settings};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::node::{Bevel3d, Node, Object3d, Outline2d, Shape3d, Stage3d, div};
use opsis_scene::paint::Transform;
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::timeline::{ChildAt, absolute_fill};
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Lighting;
use crate::elements::three::{
    Matrix, Pose, STAGE_FOV, compose, lighting_of, multiply, object, pose_matrix, rounded_rect,
    staged_with, standard, translation, view_pose,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};
use crate::timeline::camera_path::camera_at;

#[choice(export)]
pub enum ScreenDevice {
    #[label("Laptop")]
    Laptop,
    #[label("Monitor")]
    Monitor,
    #[label("Phone")]
    Phone,
    #[label("Floating screen")]
    Floating,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum ScreenMove {
    PushIn,
    Orbit,
    TiltReveal,
    Rise,
    Keyframes,
}

#[settings(export)]
pub struct Screen3d {
    #[setting(
        "Device",
        default = ScreenDevice::Laptop
    )]
    pub device: ScreenDevice,
    #[setting("Device colour", default = "#c9ced6", check = colour)]
    pub device_color: String,
    #[setting(
        "Camera move",
        default = ScreenMove::PushIn,
        labels = {
            "push-in": "Push in",
            "orbit": "Orbit",
            "tilt-reveal": "Tilt reveal",
            "rise": "Rise",
            "keyframes": "Keyframes"
        }
    )]
    pub r#move: ScreenMove,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "move": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting(
        "End facing the camera",
        default = true,
        help = "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device.",
        advanced
    )]
    pub end_face_on: bool,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Background", default = "#0b0b10", check = colour)]
    pub background: String,
    #[setting("Bezel colour", default = "#0a0a0c", check = colour, advanced)]
    pub bezel_color: String,
    #[setting(
        "Metalness",
        default = 0.6,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub metalness: f64,
    #[setting(
        "Roughness",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub roughness: f64,
    #[setting(
        "Glare",
        default = 0.16,
        range(min = 0, max = 1),
        step = 0.01,
        help = "How bright the light sweeping over the screen gets as it turns.",
        advanced
    )]
    pub glare: f64,
    #[setting(
        "Move ends at",
        default = 0.85,
        range(min = 0.1, max = 1),
        step = 0.01,
        help = "Share of the element's length the move takes; it holds still after.",
        show_if = { "move": ["push-in", "orbit", "tilt-reveal", "rise"] },
        advanced
    )]
    pub move_ends_at: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Screen3d>(element(ElementKind::Wrapper, "screen-3d",
            "3D screen",
            "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
            ElementCategory::Device,
            150).look(NodeLook::of("iconoir:perspective-view")))
}

const SCREEN_WIDTH: f64 = 2.0;
const GLARE_ANGLE: f64 = 115.0;
const GLARE_TRAVEL: f64 = 120.0;
const NEAR: f64 = 0.05;
const BEZEL_ROUGHNESS: f64 = 0.15;

struct Keyed {
    rotation: [f64; 3],
    position: [f64; 3],
}

const FACE_ON: Keyed = Keyed {
    rotation: [0.0, 0.0, 0.0],
    position: [0.0, 0.0, 0.0],
};
const ANGLED: Keyed = Keyed {
    rotation: [0.1, -0.42, 0.0],
    position: [0.15, -0.05, -2.4],
};

const fn start_of(screen_move: ScreenMove) -> Keyed {
    match screen_move {
        ScreenMove::PushIn | ScreenMove::Keyframes => Keyed {
            rotation: [0.08, -0.25, 0.0],
            position: [0.0, -0.1, -6.0],
        },
        ScreenMove::Orbit => Keyed {
            rotation: [0.15, -1.1, 0.0],
            position: [0.0, 0.0, -3.2],
        },
        ScreenMove::TiltReveal => Keyed {
            rotation: [-0.9, 0.0, 0.0],
            position: [0.0, -0.6, -3.2],
        },
        ScreenMove::Rise => Keyed {
            rotation: [0.35, 0.0, 0.0],
            position: [0.0, -1.3, -3.6],
        },
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the screen has to sit where it puts it"
)]
fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn screen_pose(props: &Screen3d, cx: &Cx<'_>) -> Result<Pose, DrawFailure> {
    let frame = cx.frame();
    if props.r#move == ScreenMove::Keyframes {
        return Ok(view_pose(&camera_at(
            Some(&props.camera),
            frame,
            props.end_face_on,
        )));
    }
    let t = clamped_eased(
        frame,
        [0.0, (cx.duration() * props.move_ends_at).max(1.0)],
        [0.0, 1.0],
        Easing::ease_in_out(Easing::Cubic),
    )?;
    let start = start_of(props.r#move);
    let rest = if props.end_face_on { FACE_ON } else { ANGLED };
    Ok(Pose {
        rotation: mix(start.rotation, rest.rotation, t),
        position: mix(start.position, rest.position, t),
        scale: 1.0,
    })
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page projects the screen's corners this way, and the page has to land on them"
)]
fn screen_quad(
    matrix: &Matrix,
    distance: f64,
    (width, height): (f64, f64),
    screen: (f64, f64),
) -> Option<[[f64; 2]; 4]> {
    let tan_half = (STAGE_FOV * std::f64::consts::PI / 360.0).tan();
    let aspect = width / height;
    let (half_w, half_h) = (screen.0 / 2.0, screen.1 / 2.0);
    let corners = [
        [-half_w, half_h],
        [half_w, half_h],
        [half_w, -half_h],
        [-half_w, -half_h],
    ];
    let mut quad = [[0.0; 2]; 4];
    for (out, [x, y]) in quad.iter_mut().zip(corners) {
        let m = matrix;
        let w = 1.0 / (m[3] * x + m[7] * y + m[15]);
        let px = (m[0] * x + m[4] * y + m[12]) * w;
        let py = (m[1] * x + m[5] * y + m[13]) * w;
        let pz = (m[2] * x + m[6] * y + m[14]) * w;
        let depth = distance - pz;
        if depth <= NEAR {
            return None;
        }
        let ndc_x = px / depth / (tan_half * aspect);
        let ndc_y = py / depth / tan_half;
        *out = [
            f64::midpoint(ndc_x, 1.0) * width,
            ((1.0 - ndc_y) / 2.0) * height,
        ];
    }
    Some(quad)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page measures the quad's winding this way"
)]
fn faces_camera(quad: &[[f64; 2]; 4]) -> bool {
    let mut area = 0.0;
    for i in 0..4 {
        let (a, b) = (quad[i], quad[(i + 1) % 4]);
        area += a[0] * b[1] - b[0] * a[1];
    }
    area > 0.0
}

fn nine_places(v: f64) -> f64 {
    let rounded = (v * 1e9).round() / 1e9;
    if rounded == 0.0 { 0.0 } else { rounded }
}

#[expect(
    clippy::suboptimal_flops,
    clippy::many_single_char_names,
    reason = "the film page solves the homography this way and names its terms so; the page has to land where it puts it"
)]
fn quad_matrix(width: f64, height: f64, quad: &[[f64; 2]; 4]) -> [Num; 16] {
    let [q0, q1, q2, q3] = *quad;
    let dx1 = q1[0] - q2[0];
    let dx2 = q3[0] - q2[0];
    let dx3 = q0[0] - q1[0] + q2[0] - q3[0];
    let dy1 = q1[1] - q2[1];
    let dy2 = q3[1] - q2[1];
    let dy3 = q0[1] - q1[1] + q2[1] - q3[1];
    let den = dx1 * dy2 - dx2 * dy1;
    let solved = den != 0.0;
    let g = if solved {
        (dx3 * dy2 - dx2 * dy3) / den
    } else {
        0.0
    };
    let h = if solved {
        (dx1 * dy3 - dx3 * dy1) / den
    } else {
        0.0
    };
    let a = q1[0] - q0[0] + g * q1[0];
    let b = q3[0] - q0[0] + h * q3[0];
    let d = q1[1] - q0[1] + g * q1[1];
    let e = q3[1] - q0[1] + h * q3[1];
    [
        a / width,
        d / width,
        0.0,
        g / width,
        b / height,
        e / height,
        0.0,
        h / height,
        0.0,
        0.0,
        1.0,
        0.0,
        q0[0],
        q0[1],
        0.0,
        1.0,
    ]
    .map(|v| Num(nine_places(v)))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page slides the glare this way, and it has to sit where it puts it"
)]
fn glare(turn: f64, strength: f64, width: f64) -> Node {
    let position = (50.0 + turn * GLARE_TRAVEL) / 100.0;
    let tile = width * 2.0;
    let offset = (width - tile) * position;
    let gradient = format!(
        "linear-gradient({GLARE_ANGLE}deg, rgba(255,255,255,{strength}) 0%, rgba(255,255,255,0) 45%)"
    );
    let mut layer = div(absolute_fill().opacity((turn.abs() * 3.0).min(1.0)));
    for k in -3..=3 {
        let left = offset + tile * f64::from(k);
        if left >= width || left + tile <= 0.0 {
            continue;
        }
        layer = layer.child(div(with_background(
            Style::default()
                .position(Position::Absolute)
                .top(Length::ZERO)
                .left(Length::px(left))
                .width(Length::px(tile))
                .height(Length::percent(100.0)),
            &gradient,
        )));
    }
    layer.into()
}

struct Part {
    shape: Shape3d,
    at: [f64; 3],
    bezel: bool,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page builds the device this way, and its parts have to sit where it puts them"
)]
fn slab(width: f64, height: f64, radius: f64, depth: f64) -> Part {
    Part {
        shape: Shape3d::Extruded {
            outlines: vec![Outline2d {
                shape: rounded_rect(width, height, radius),
                holes: Vec::new(),
            }],
            curve_segments: 12,
            depth: Num(depth),
            bevel: Some(Bevel3d {
                thickness: Num(depth * 0.15),
                size: Num(radius.min(depth) * 0.3),
                offset: Num(0.0),
                segments: 3,
            }),
        },
        at: [0.0, 0.0, -0.003 - depth * 1.15],
        bezel: false,
    }
}

fn rounded_box(size: [f64; 3], radius: f64, at: [f64; 3]) -> Part {
    Part {
        shape: Shape3d::RoundedBox {
            size: size.map(Num),
            segments: 4,
            radius: Num(radius),
        },
        at,
        bezel: false,
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page builds the device this way, and its parts have to sit where it puts them"
)]
fn device_parts(device: ScreenDevice, (w, h): (f64, f64)) -> Vec<Part> {
    let unit = w.min(h);
    let (front, mut parts) = match device {
        ScreenDevice::Floating => {
            let edge = unit * 0.015;
            let size = (w + edge * 2.0, h + edge * 2.0, unit * 0.04);
            (size, vec![slab(size.0, size.1, size.2, unit * 0.03)])
        }
        ScreenDevice::Phone => {
            let edge = unit * 0.05;
            let size = (w + edge * 2.0, h + edge * 2.0, unit * 0.12);
            (size, vec![slab(size.0, size.1, size.2, unit * 0.07)])
        }
        ScreenDevice::Monitor => {
            let bezel = unit * 0.05;
            let neck = h * 0.32;
            let bottom = -h / 2.0 - bezel;
            let size = (w + bezel * 2.0, h + bezel * 2.0, unit * 0.03);
            (
                size,
                vec![
                    slab(size.0, size.1, size.2, unit * 0.06),
                    Part {
                        shape: Shape3d::Box {
                            size: [Num(w * 0.12), Num(neck), Num(unit * 0.05)],
                        },
                        at: [0.0, bottom - neck / 2.0 + unit * 0.02, -unit * 0.12],
                        bezel: false,
                    },
                    rounded_box(
                        [w * 0.36, unit * 0.04, unit * 0.45],
                        unit * 0.015,
                        [0.0, bottom - neck + unit * 0.02, -unit * 0.1],
                    ),
                ],
            )
        }
        ScreenDevice::Laptop => {
            let bezel = unit * 0.045;
            let deck = h * 0.72;
            let thick = unit * 0.035;
            let hinge = -h / 2.0 - bezel;
            let size = (w + bezel * 2.0, h + bezel * 2.0, unit * 0.03);
            (
                size,
                vec![
                    slab(size.0, size.1, size.2, unit * 0.025),
                    rounded_box(
                        [w + bezel * 2.4, thick, deck],
                        thick * 0.4,
                        [0.0, hinge - thick / 2.0, deck / 2.0 - unit * 0.03],
                    ),
                ],
            )
        }
    };
    let faces = [
        Part {
            shape: Shape3d::Plane {
                width: Num(w),
                height: Num(h),
            },
            at: [0.0, 0.0, -0.001],
            bezel: true,
        },
        Part {
            shape: Shape3d::Flat {
                outline: Outline2d {
                    shape: rounded_rect(front.0, front.1, front.2),
                    holes: Vec::new(),
                },
                curve_segments: 12,
                uv_span: [Num(1.0), Num(1.0)],
                uv_offset: [Num(0.0), Num(0.0)],
            },
            at: [0.0, 0.0, -0.002],
            bezel: true,
        },
    ];
    parts.splice(0..0, faces);
    parts
}

fn device_objects(props: &Screen3d, screen: (f64, f64), posed: &Matrix) -> Vec<Object3d> {
    device_parts(props.device, screen)
        .into_iter()
        .map(|part| {
            let material = if part.bezel {
                standard(&props.bezel_color, BEZEL_ROUGHNESS, 0.0)
            } else {
                standard(&props.device_color, props.roughness, props.metalness)
            };
            object(
                part.shape,
                &multiply(posed, &translation(part.at[0], part.at[1], part.at[2])),
                material,
            )
        })
        .collect()
}

pub fn wrap(props: &Screen3d, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let screen = (SCREEN_WIDTH, SCREEN_WIDTH / (width / height));
    let distance = screen.1 / 2.0 / (STAGE_FOV * std::f64::consts::PI / 360.0).tan();
    let pose = screen_pose(props, cx)?;
    let posed = compose(pose.position, pose.rotation, [pose.scale; 3]);
    let overlay = match screen_quad(&posed, distance, (width, height), screen) {
        Some(quad) if faces_camera(&quad) => Some(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .width(Length::px(width))
                .height(Length::px(height))
                .overflow(Overflow::Hidden)
                .transform_origin((Length::ZERO, Length::ZERO, Length::ZERO))
                .transform(vec![Transform::Matrix3d(quad_matrix(width, height, &quad))]))
            .child(child(cx.at.stage)?)
            .child(glare(pose.rotation[1], props.glare, width))
            .into(),
        ),
        _ => None,
    };
    let objects = device_objects(props, screen, &pose_matrix(&pose));
    Ok(staged_with(
        Stage3d {
            fov: Num(STAGE_FOV),
            distance: Num(distance),
            lighting: lighting_of(props.lighting),
            objects,
            textures: Vec::new(),
            models: Vec::new(),
        },
        &props.background,
        overlay,
    ))
}
