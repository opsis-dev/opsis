use kurbo::Point;
use opsis_macros::{choice, settings};
use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::paint::{ClipPath, FillRule, Filter};
use opsis_scene::style::{Position, Style};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::transition::path_morph::{js_round, morphed, path_data, prepare_morph};
use crate::elements::transition::present::{filled, transition_phase};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MorphShape {
    Circle,
    Square,
    Squircle,
    Triangle,
    Diamond,
    Blob,
}

#[settings(export)]
pub struct TransitionMorphShape {
    #[setting(
        "From",
        default = MorphShape::Circle,
        help = "Shape the reveal starts as, at zero size. A raw path d also works."
    )]
    pub from: MorphShape,
    #[setting(
        "To",
        default = MorphShape::Squircle,
        help = "Shape it has become once it covers the frame."
    )]
    pub to: MorphShape,
    #[setting(
        "Horizontal origin",
        default = 0.5,
        step = 0.01,
        help = "Where the shape grows from, 0→1 of the frame width.",
        advanced
    )]
    pub origin_x: f64,
    #[setting(
        "Vertical origin",
        default = 0.5,
        step = 0.01,
        help = "Where the shape grows from, 0→1 of the frame height.",
        advanced
    )]
    pub origin_y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Overshoot",
        range(min = 0.8, max = 3),
        zero_label = "Fit the shape",
        step = 0.05,
        help = "Final size as a multiple of the frame diagonal. Leave empty to grow each shape until it covers the frame.",
        advanced,
        "type" = "number"
    )]
    pub overshoot: Option<f64>,
    #[setting(
        "Warp",
        range(min = 0, max = 100),
        step = 1,
        unit = "px",
        help = "0 keeps the silhouette crisp.",
        advanced
    )]
    pub warp: f64,
    #[setting(
        "Warp detail",
        default = 0.008,
        range(min = 0.001, max = 0.05),
        advanced
    )]
    pub warp_detail: f64,
    #[setting(
        "Variation",
        default = 7,
        range(min = 0, max = 100),
        step = 1,
        advanced
    )]
    pub seed: u32,
}

pub fn definition() -> ElementMeta {
    meta::<TransitionMorphShape>(
            element(
                ElementKind::Presentation,
                "transition-morph-shape",
                "Shape morph",
                "The next scene shows through a shape that grows and changes form until it fills the frame.",
                ElementCategory::Transition,
                24,
            )
                .look(NodeLook::of("lucide:square").morphs("lucide:circle")),
        )
}

const WARP_FLOOR: f64 = 0.5;
const CHURN: f64 = 6.0;
const OCTAVES: u32 = 2;
const MAX_OVERSCAN: f64 = 3.0;
const WARP_ID: &str = "shape-morph-warp";

const fn overshoot_for(shape: MorphShape) -> f64 {
    match shape {
        MorphShape::Circle | MorphShape::Square => 1.06,
        MorphShape::Squircle => 1.1,
        MorphShape::Blob => 1.28,
        MorphShape::Diamond => 1.42,
        MorphShape::Triangle => 2.1,
    }
}

fn overscan_for(scale: f64, frame_size: f64) -> f64 {
    let drag = scale / 2.0;
    if drag <= 0.0 || frame_size <= 0.0 {
        return 1.0;
    }
    let clean = 2.0f64.mul_add(-(drag + 2.0), frame_size);
    if clean <= 0.0 {
        return MAX_OVERSCAN;
    }
    MAX_OVERSCAN.min(frame_size / clean)
}

fn warp_filter(frequency: f64, seed: f64, scale: f64) -> SvgElement {
    SvgElement::new("filter")
        .attr("id", WARP_ID)
        .attr("x", "-30%")
        .attr("y", "-30%")
        .attr("width", "160%")
        .attr("height", "160%")
        .child(
            SvgElement::new("feTurbulence")
                .attr("type", "fractalNoise")
                .attr("baseFrequency", frequency)
                .attr("numOctaves", OCTAVES)
                .attr("seed", seed)
                .attr("result", "noise"),
        )
        .child(
            SvgElement::new("feDisplacementMap")
                .attr("in", "SourceGraphic")
                .attr("in2", "noise")
                .attr("scale", scale)
                .attr("xChannelSelector", "R")
                .attr("yChannelSelector", "G"),
        )
}

pub fn present(
    props: &TransitionMorphShape,
    side: Transition,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let phase = transition_phase(side.progress, side.entering, 1.0, false)?;
    if !phase.entering {
        return Ok(filled(&Style::default(), side.children));
    }
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let displace = phase.displace.clamp(0.0, 1.0);
    let scale = props.warp * 4.0 * displace * (1.0 - displace);
    let coverage = 1.0 - displace;
    let wants_filter = scale >= WARP_FLOOR;
    let wants_mask = coverage < 1.0;
    if !wants_filter && !wants_mask {
        return Ok(filled(&Style::default(), side.children));
    }
    let mut outer = Style::default();
    if wants_mask {
        let overshoot = props.overshoot.unwrap_or_else(|| overshoot_for(props.to));
        let reach = width.hypot(height) * overshoot * coverage;
        let (center_x, center_y) = (width * props.origin_x, height * props.origin_y);
        let pair = prepare_morph(props.from, props.to);
        let data = path_data(&morphed(coverage, &pair), pair.closed, |p| {
            Point::new(
                (p.x - 0.5).mul_add(reach, center_x),
                (p.y - 0.5).mul_add(reach, center_y),
            )
        });
        outer = outer.clip_path(ClipPath::Path {
            fill: FillRule::Nonzero,
            data,
        });
    }
    let mut layers: Vec<Node> = Vec::new();
    let mut inner = Style::default();
    if wants_filter {
        let seed = js_round(CHURN.mul_add(displace, f64::from(props.seed)));
        layers.push(
            svg(
                SvgElement::new("svg")
                    .attr("width", 0)
                    .attr("height", 0)
                    .child(SvgElement::new("defs").child(warp_filter(
                        props.warp_detail,
                        seed,
                        scale,
                    ))),
                Style::default().position(Position::Absolute),
            )
            .into(),
        );
        let overscan = overscan_for(scale, width.min(height));
        inner = inner
            .filter(vec![Filter::Url(WARP_ID.to_owned())])
            .scale(overscan, overscan);
    }
    layers.push(filled(&inner, side.children));
    Ok(filled(&outer, layers))
}
