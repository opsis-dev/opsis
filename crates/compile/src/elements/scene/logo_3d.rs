use opsis_geometry::mesh_of;
use opsis_geometry::outline::{path_points, shapes_of};
use opsis_macros::settings;
use opsis_scene::node::{Bevel3d, Node, Object3d, Shape3d, Stage3d, Texture3d, div};
use opsis_scene::paint::AssetRef;
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;
use opsis_scene::vector::Vector;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::scene::card_3d::{CardLook, card_objects, picture_size};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::three::motion::{MotionTime, element_pose};
use crate::elements::three::{
    Matrix, SUBJECT_RADIUS, compose, fit_distance, fitted_size, lighting_of, multiply, object,
    pose_matrix, staged, standard, translation, two_sided,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Logo3d {
    #[setting(
        "Logo file",
        widget = "asset",
        accept = ".svg,.png,.jpg,.jpeg,.webp",
        zero_label = "Sample logo",
        help = "A drawn logo is given depth; a picture of a logo goes on a card."
    )]
    pub src: String,
    #[setting("Depth", default = 0.25, range(min = 0.05, max = 1), step = 0.05)]
    pub depth: f64,
    #[setting("Colour", widget = "color", zero_label = "The logo's own colours")]
    pub color: String,
    #[setting("Metal", default = 0.3, range(min = 0, max = 1), step = 0.05, advanced)]
    pub metalness: f64,
    #[setting(
        "Motion",
        default = Motion::Spin
    )]
    pub motion: Motion,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "motion": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3))]
    pub zoom: f64,
    #[setting("Matte", default = 0.35, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting("Bevel", default = 0.08, range(min = 0, max = 0.3), advanced)]
    pub bevel: f64,
    #[setting("Rounded corners", default = 0.12, range(min = 0, max = 0.5), advanced)]
    pub corner_radius: f64,
    #[setting(
        "Tilt",
        default = 52,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        advanced
    )]
    pub tilt: u32,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
    #[setting("Turn the other way", advanced)]
    pub reverse: bool,
    #[setting(
        "Lens",
        default = 30.0,
        range(min = 10, max = 70),
        unit = "°",
        advanced
    )]
    pub lens: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Logo3d>(
        element(
            ElementKind::Component,
            "logo-3d",
            "3D logo",
            "Your logo in 3D, spinning, tilting in or floating, with studio lighting.",
            ElementCategory::Scene,
            150,
        )
        .look(NodeLook::of("lucide:rotate-3d").moves(NodeMotion::Turn)),
    )
}

const SAMPLE_LOGO: &str = "app:placeholders/logo-mark.svg";
const LOGO_SIZE: f64 = 2.0;
const CURVE_SEGMENTS: u32 = 24;
const OUTLINE_DIVISIONS: u32 = 12;
const TUBE_SIDES: u32 = 12;
const BEVEL_SEGMENTS: u32 = 3;
const CARD_DEPTH: f64 = 0.4;
const FACE_ROUGHNESS: f64 = 0.6;
const SEED: f64 = 1.0;

fn is_svg(src: &str) -> bool {
    src.starts_with("data:image/svg")
        || src.split(['?', '#']).next().is_some_and(|path| {
            std::path::Path::new(path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
        })
}

fn svg_extent(svg: &Vector) -> f64 {
    let points = svg
        .paths
        .iter()
        .flat_map(|p| &p.sub_paths)
        .flat_map(|sub| path_points(sub, OUTLINE_DIVISIONS));
    let (low, high) = points.fold(
        ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]),
        |(low, high), p| {
            (
                [low[0].min(p[0]), low[1].min(p[1])],
                [high[0].max(p[0]), high[1].max(p[1])],
            )
        },
    );
    let size = (high[0] - low[0]).max(0.0).max((high[1] - low[1]).max(0.0));
    if size > 0.0 { size } else { 1.0 }
}

struct Part {
    shape: Shape3d,
    local: Matrix,
    color: String,
    bounds: ([f64; 3], [f64; 3]),
}

fn part(shape: Shape3d, z: f64, color: String) -> Option<Part> {
    let (low, high) = mesh_of(&shape).bounds()?;
    Some(Part {
        shape,
        local: translation(0.0, 0.0, z),
        color,
        bounds: (
            [low[0], low[1], low[2] + z],
            [high[0], high[1], high[2] + z],
        ),
    })
}

fn tube_part(points: &[[f64; 2]], radius: f64, color: &str) -> Option<Part> {
    if points.len() < 2 {
        return None;
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let closed = (first[0] - last[0]).hypot(first[1] - last[1]) < radius * 0.5;
    let ends = if closed {
        &points[..points.len() - 1]
    } else {
        points
    };
    part(
        Shape3d::Tube {
            points: ends
                .iter()
                .map(|p| [Num(p[0]), Num(p[1]), Num(0.0)])
                .collect(),
            closed,
            tubular_segments: u32::try_from(ends.len() * 8).unwrap_or(u32::MAX).max(8),
            radius: Num(radius),
            radial_segments: TUBE_SIDES,
        },
        0.0,
        color.to_owned(),
    )
}

fn svg_parts(svg: &Vector, props: &Logo3d) -> Vec<Part> {
    let depth = props.depth * svg_extent(svg) * 0.5;
    let mut parts = Vec::new();
    for path in &svg.paths {
        if let Some(fill) = &path.fill {
            parts.extend(part(
                Shape3d::Extruded {
                    outlines: shapes_of(&path.sub_paths, path.even_odd),
                    curve_segments: CURVE_SEGMENTS,
                    depth: Num(depth),
                    bevel: Some(Bevel3d {
                        thickness: Num(depth * props.bevel),
                        size: Num(depth * props.bevel),
                        offset: Num(0.0),
                        segments: BEVEL_SEGMENTS,
                    }),
                },
                -depth / 2.0,
                fill.clone(),
            ));
        }
        if let Some(stroke) = &path.stroke {
            let width = if stroke.width > 0.0 {
                stroke.width
            } else {
                1.0
            };
            for sub in &path.sub_paths {
                parts.extend(tube_part(
                    &path_points(sub, CURVE_SEGMENTS),
                    (width / 2.0).max(depth / 2.0),
                    &stroke.color,
                ));
            }
        }
    }
    parts
}

fn svg_objects(svg: &Vector, props: &Logo3d, posed: &Matrix) -> Vec<Object3d> {
    let parts = svg_parts(svg, props);
    let Some((low, high)) = parts
        .iter()
        .map(|p| {
            let (low, high) = p.bounds;
            ([low[0], -high[1], low[2]], [high[0], -low[1], high[2]])
        })
        .reduce(|(a, b), (c, d)| {
            (
                [a[0].min(c[0]), a[1].min(c[1]), a[2].min(c[2])],
                [b[0].max(d[0]), b[1].max(d[1]), b[2].max(d[2])],
            )
        })
    else {
        return Vec::new();
    };
    let center = [0, 1, 2].map(|k| f64::midpoint(low[k], high[k]));
    let largest = (0..3).map(|k| high[k] - low[k]).fold(0.0, f64::max);
    let s = LOGO_SIZE * props.zoom / if largest > 0.0 { largest } else { 1.0 };
    let flipped = multiply(
        posed,
        &compose(
            [-center[0] * s, -center[1] * s, -center[2] * s],
            [0.0; 3],
            [s, -s, s],
        ),
    );
    parts
        .into_iter()
        .map(|p| {
            let color = if props.color.is_empty() {
                p.color.as_str()
            } else {
                props.color.as_str()
            };
            object(
                p.shape,
                &multiply(&flipped, &p.local),
                two_sided(standard(color, props.roughness, props.metalness)),
            )
        })
        .collect()
}

pub fn draw(props: &Logo3d, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let src = if props.src.is_empty() {
        SAMPLE_LOGO
    } else {
        props.src.as_str()
    };
    let pose = element_pose(
        props.motion,
        Some(&props.camera),
        &MotionTime {
            frame: cx.frame(),
            fps: cx.fps(),
            speed: props.speed,
            seed: SEED,
            tilt_deg: f64::from(props.tilt),
            end_facing: props.end_facing,
            amount: props.motion_amount,
            reverse: props.reverse,
            grow_in: props.grow_in,
        },
    );
    let posed = pose_matrix(&pose);
    let (objects, textures) = if is_svg(src) {
        let Some(svg) = cx.measure.vector(src) else {
            return Ok(div(absolute_fill()).into());
        };
        (svg_objects(&svg, props, &posed), Vec::new())
    } else {
        let Some((w, h)) = picture_size(cx, src) else {
            return Ok(div(absolute_fill()).into());
        };
        let longest = LOGO_SIZE * props.zoom;
        (
            card_objects(
                fitted_size(w, h, longest),
                longest,
                props.corner_radius,
                props.depth * CARD_DEPTH,
                &CardLook {
                    edge_color: &props.color,
                    metalness: props.metalness,
                    face_roughness: FACE_ROUGHNESS,
                },
                &posed,
            )
            .to_vec(),
            vec![Texture3d::Picture(AssetRef(src.to_owned()))],
        )
    };
    let aspect = cx.at.stage.width / cx.at.stage.height;
    Ok(staged(
        Stage3d {
            fov: Num(props.lens),
            distance: Num(fit_distance(SUBJECT_RADIUS, props.lens, aspect)),
            lighting: lighting_of(props.lighting),
            objects,
            textures,
        },
        &props.background,
    ))
}
