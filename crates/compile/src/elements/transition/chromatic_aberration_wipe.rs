use opsis_macros::settings;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::paint::{BlendMode, Filter};
use opsis_scene::style::{Position, Style};
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::transition::present::{filled, transition_phase};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ChromaticAberrationWipe {
    #[setting("Intensity", default = 12.0)]
    pub intensity: f64,
    #[setting(
        "Axis",
        default = Orientation::Horizontal,
        help = "Axis the scenes slide along and the channels separate on."
    )]
    pub axis: Orientation,
    #[setting(
        "Slide distance",
        default = 1.0,
        help = "Share of the frame the scenes travel. Below 1 shows background.",
        advanced
    )]
    pub slide: f64,
    #[setting("Fade while sliding", advanced)]
    pub fade: bool,
}

pub fn definition() -> ElementMeta {
    meta::<ChromaticAberrationWipe>(
            element(
                ElementKind::Presentation,
                "chromatic-aberration-wipe",
                "Colour split slide",
                "A fast slide that pulls the colours apart mid-move and brings them back together as it lands.",
                ElementCategory::Transition,
                14,
            )
                .look(NodeLook::of("tabler:color-filter")),
        )
}

const CHANNELS: [(&str, &str, f64); 3] = [
    ("red", "1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0", 1.0),
    ("green", "0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0", 0.0),
    ("blue", "0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0", -1.0),
];
const FILTER_ID: &str = "chromatic-aberration";

pub fn present(
    props: &ChromaticAberrationWipe,
    side: Transition,
    _: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let phase = transition_phase(side.progress, side.entering, 1.0, props.fade)?;
    let separation = interpolate(
        phase.displace,
        &[0.0, 0.5, 1.0],
        &[0.0, props.intensity, 0.0],
        &Interpolation::clamped(),
    )?;
    let travel = phase.displace * if phase.entering { 100.0 } else { -100.0 } * props.slide;
    let horizontal = props.axis == Orientation::Horizontal;
    let id = format!("{FILTER_ID}-{}", if side.entering { "in" } else { "out" });
    let defs = CHANNELS
        .iter()
        .fold(SvgElement::new("defs"), |defs, (channel, matrix, _)| {
            defs.child(
                SvgElement::new("filter")
                    .attr("id", format!("{id}-{channel}"))
                    .child(
                        SvgElement::new("feColorMatrix")
                            .attr("type", "matrix")
                            .attr("values", *matrix),
                    ),
            )
        });
    let mut layers: Vec<Node> = vec![
        svg(
            SvgElement::new("svg")
                .attr("width", 0)
                .attr("height", 0)
                .child(defs),
            Style::default().position(Position::Absolute),
        )
        .into(),
    ];
    if separation >= 0.1 {
        for (channel, _, direction) in CHANNELS {
            let offset = Length::px(direction * separation);
            let shift = if horizontal {
                (offset, Length::ZERO)
            } else {
                (Length::ZERO, offset)
            };
            layers.push(filled(
                &Style::default()
                    .filter(vec![Filter::Url(format!("{id}-{channel}"))])
                    .mix_blend_mode(BlendMode::Screen)
                    .translate(shift.0, shift.1),
                side.children.clone(),
            ));
        }
    } else {
        layers.push(filled(&Style::default(), side.children));
    }
    let shift = if horizontal {
        (Length::percent(travel), Length::percent(0.0))
    } else {
        (Length::percent(0.0), Length::percent(travel))
    };
    Ok(filled(
        &Style::default()
            .opacity(phase.opacity)
            .translate(shift.0, shift.1)
            .isolation(true),
        layers,
    ))
}
