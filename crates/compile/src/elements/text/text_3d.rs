use opsis_geometry::mesh_of;
use opsis_geometry::outline::shapes_of;
use opsis_macros::settings;
use opsis_scene::node::{Bevel3d, Node, Outline2d, Segment2d, Shape3d, Stage3d, div};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::typeface::{GlyphCommand, Typeface};
use opsis_scene::values::Num;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::narrow;
use opsis_types::timeline::CameraPath;
use serde::{Deserialize, Serialize};

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion, TextAlign};
use crate::elements::three::motion::{MotionTime, element_pose};
use crate::elements::three::{
    Matrix, SUBJECT_RADIUS, box_distance, fit_distance, lighting_of, multiply, object, pose_matrix,
    scaling, staged, standard, translation,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreparedTypeface {
    pub typeface: String,
    pub typeface_italic: bool,
}

#[settings(export)]
pub struct Text3d {
    #[setting("Text", default = "Ship faster", widget = "textarea")]
    pub text: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Empty uses the project's heading font."
    )]
    pub font_family: String,
    #[setting("Font weight", default = 800, range(min = 100, max = 900), step = 10)]
    pub font_weight: u32,
    #[setting("Italic")]
    pub italic: bool,
    #[setting("Depth", default = 0.35, range(min = 0.05, max = 1), step = 0.05)]
    pub depth: f64,
    #[setting(
        "Rounded edges",
        default = 0.4,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub bevel: f64,
    #[setting("Colour", default = "#fafafa", check = colour)]
    pub color: String,
    #[setting("Metal", default = 0.2, range(min = 0, max = 1), step = 0.05, advanced)]
    pub metalness: f64,
    #[setting("Alignment", default = TextAlign::Center)]
    pub align: TextAlign,
    #[setting(
        "Line length",
        default = 16,
        range(min = 4, max = 60),
        step = 1,
        unit = "letters",
        help = "Long text wraps onto a new line after about this many letters.",
        advanced
    )]
    pub line_width: u32,
    #[setting(
        "Motion",
        default = Motion::TiltIn
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
    #[setting("Background", default = "transparent")]
    pub background: String,
    #[setting(
        "Size in frame",
        default = 1.0,
        range(min = 0.3, max = 1.5),
        unit = "×",
        advanced
    )]
    pub size: f64,
    #[setting(
        "Line spacing",
        default = 1.15,
        range(min = 0.8, max = 2),
        unit = "em",
        advanced
    )]
    pub line_height: f64,
    #[setting("Roughness", default = 0.3, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting(
        "Lens",
        default = 30.0,
        range(min = 10, max = 70),
        unit = "°",
        advanced
    )]
    pub lens: f64,
    #[setting(
        "Tilt",
        default = 52,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        advanced
    )]
    pub tilt: u32,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("Turn the other way", advanced)]
    pub reverse: bool,
}

pub fn definition() -> ElementMeta {
    meta::<Text3d>(
        element(
            ElementKind::Component,
            "text-3d",
            "3D text",
            "Words in solid 3D letters with rounded edges, in any project font.",
            ElementCategory::Text,
            120,
        )
        .look(NodeLook::of("lucide:case-sensitive").moves(NodeMotion::Turn)),
    )
}

const TEXT_SIZE: f64 = 2.6;
const LETTER_EM: f64 = 0.6;
const DEPTH_EM: f64 = 0.6;
const BEVEL_THICKNESS: f64 = 0.05;
const BEVEL_SIZE: f64 = 0.035;
const BEVEL_SEGMENTS: u32 = 4;
const CURVE_SEGMENTS: u32 = 10;
const ITALIC_SHEAR: f64 = 0.2;
const SEED: f64 = 1.0;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Typefaced {
    #[serde(flatten)]
    pub settings: Text3d,
    #[serde(default)]
    pub typeface: String,
    #[serde(default)]
    pub typeface_italic: bool,
}

pub fn wrap_lines(text: &str, max_width: f64, width_of: impl Fn(&str) -> f64) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let next = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && width_of(&next) > max_width {
                lines.push(std::mem::replace(&mut line, word.to_owned()));
            } else {
                line = next;
            }
        }
        lines.push(line);
    }
    lines
}

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js places each glyph point this way, and the letters have to land where it puts them"
)]
fn at(p: [f64; 2], scale: f64, offset: f64) -> [Num; 2] {
    [Num(p[0] * scale + offset), Num(p[1] * scale)]
}

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js moves the pen by each advance this way, and the letters have to land where it puts them"
)]
fn line_outlines(face: &Typeface, line: &str) -> Vec<Outline2d> {
    let scale = 1.0 / face.resolution;
    let mut offset = 0.0;
    let mut outlines = Vec::new();
    for ch in line.chars() {
        let Some(glyph) = face.glyph(ch) else {
            continue;
        };
        let mut sub_paths: Vec<Vec<Segment2d>> = Vec::new();
        for command in &glyph.commands {
            match *command {
                GlyphCommand::Move(p) => {
                    sub_paths.push(vec![Segment2d::Move(at(p, scale, offset))]);
                }
                GlyphCommand::Line(p) => {
                    if let Some(path) = sub_paths.last_mut() {
                        path.push(Segment2d::Line(at(p, scale, offset)));
                    }
                }
                GlyphCommand::Quad { to, control } => {
                    if let Some(path) = sub_paths.last_mut() {
                        path.push(Segment2d::Quad(
                            at(control, scale, offset),
                            at(to, scale, offset),
                        ));
                    }
                }
                GlyphCommand::Cubic { to, first, second } => {
                    if let Some(path) = sub_paths.last_mut() {
                        path.push(Segment2d::Cubic(
                            at(first, scale, offset),
                            at(second, scale, offset),
                            at(to, scale, offset),
                        ));
                    }
                }
            }
        }
        outlines.extend(shapes_of(&sub_paths, false));
        offset += glyph.advance * scale;
    }
    outlines
}

const fn shear(amount: f64) -> Matrix {
    [
        1.0, 0.0, 0.0, 0.0, amount, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

type Bounds = ([f64; 3], [f64; 3]);

#[expect(
    clippy::suboptimal_flops,
    reason = "three.js shears the text's points this way, and its box has to match it"
)]
fn sheared_bounds(positions: &[f32], amount: f64) -> Option<Bounds> {
    let mut points = positions.as_chunks::<3>().0.iter().map(|p| {
        let [x, y, z] = p.map(f64::from);
        [f64::from(narrow(x + amount * y)), y, z]
    });
    let first = points.next()?;
    Some(points.fold((first, first), |(low, high), p| {
        (
            [low[0].min(p[0]), low[1].min(p[1]), low[2].min(p[2])],
            [high[0].max(p[0]), high[1].max(p[1]), high[2].max(p[2])],
        )
    }))
}

struct Line {
    shape: Shape3d,
    bounds: Bounds,
}

fn line_shape(face: &Typeface, line: &str, props: &Text3d, slant: bool) -> Option<Line> {
    if line.trim().is_empty() {
        return None;
    }
    let shape = Shape3d::Extruded {
        outlines: line_outlines(face, line),
        curve_segments: CURVE_SEGMENTS,
        depth: Num(props.depth * DEPTH_EM),
        bevel: (props.bevel > 0.0).then_some(Bevel3d {
            thickness: Num(props.bevel * BEVEL_THICKNESS),
            size: Num(props.bevel * BEVEL_SIZE),
            offset: Num(0.0),
            segments: BEVEL_SEGMENTS,
        }),
    };
    let geometry = mesh_of(&shape);
    let bounds = sheared_bounds(&geometry.positions, if slant { ITALIC_SHEAR } else { 0.0 })
        .unwrap_or(([0.0; 3], [0.0; 3]));
    Some(Line { shape, bounds })
}

pub fn draw(drawn: &Typefaced, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let props = &drawn.settings;
    let Some(face) = cx.measure.typeface(&drawn.typeface) else {
        return Ok(div(absolute_fill()).into());
    };
    let slant = props.italic && !drawn.typeface_italic;
    let lines = wrap_lines(&props.text, f64::from(props.line_width) * LETTER_EM, |s| {
        s.chars().map(|c| face.advance(c)).sum::<f64>() / face.resolution
    });
    let shapes: Vec<Option<Line>> = lines
        .iter()
        .map(|line| line_shape(&face, line, props, slant))
        .collect();
    let widest = shapes
        .iter()
        .map(|l| l.as_ref().map_or(0.0, |l| l.bounds.1[0] - l.bounds.0[0]))
        .fold(0.0, f64::max);
    let placed: Vec<(Line, [f64; 3])> = shapes
        .into_iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line?;
            let (low, high) = line.bounds;
            let x = match props.align {
                TextAlign::Left => -widest / 2.0 - low[0],
                TextAlign::Right => widest / 2.0 - high[0],
                TextAlign::Center => -(low[0] + high[0]) / 2.0,
            };
            Some((
                line,
                [x, -opsis_types::numbers::float(i) * props.line_height, 0.0],
            ))
        })
        .collect();
    let union = placed
        .iter()
        .map(|(line, at)| {
            let (low, high) = line.bounds;
            (
                [low[0] + at[0], low[1] + at[1], low[2] + at[2]],
                [high[0] + at[0], high[1] + at[1], high[2] + at[2]],
            )
        })
        .reduce(|(a_low, a_high), (b_low, b_high)| {
            (
                [
                    a_low[0].min(b_low[0]),
                    a_low[1].min(b_low[1]),
                    a_low[2].min(b_low[2]),
                ],
                [
                    a_high[0].max(b_high[0]),
                    a_high[1].max(b_high[1]),
                    a_high[2].max(b_high[2]),
                ],
            )
        });
    let Some((low, high)) = union else {
        return Ok(div(absolute_fill()).into());
    };
    let center = [0, 1, 2].map(|k| f64::midpoint(low[k], high[k]));
    let extent = [0, 1, 2].map(|k| high[k] - low[k]);
    let largest = extent[0].max(extent[1]).max(extent[2]);
    let fit = TEXT_SIZE / if largest > 0.0 { largest } else { 1.0 };
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
    let text_matrix = multiply(
        &pose_matrix(&pose),
        &multiply(
            &translation(-center[0] * fit, -center[1] * fit, -center[2] * fit),
            &scaling(fit),
        ),
    );
    let slanted = shear(if slant { ITALIC_SHEAR } else { 0.0 });
    let material = standard(&props.color, props.roughness, props.metalness);
    let objects = placed
        .into_iter()
        .map(|(line, at)| {
            object(
                line.shape,
                &multiply(
                    &text_matrix,
                    &multiply(&translation(at[0], at[1], at[2]), &slanted),
                ),
                material.clone(),
            )
        })
        .collect();
    let aspect = cx.at.stage.width / cx.at.stage.height;
    let distance = fit_distance(SUBJECT_RADIUS, props.lens, aspect).max(box_distance(
        extent[0] * fit / props.size,
        extent[1] * fit / props.size,
        props.lens,
        aspect,
    ));
    Ok(staged(
        Stage3d {
            fov: Num(props.lens),
            distance: Num(distance),
            lighting: lighting_of(props.lighting),
            objects,
            textures: Vec::new(),
            models: Vec::new(),
        },
        &props.background,
    ))
}
