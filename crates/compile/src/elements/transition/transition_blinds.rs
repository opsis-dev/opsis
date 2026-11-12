use opsis_macros::settings;
use opsis_scene::color::{Color, Interpolation as ColorInterpolation};
use opsis_scene::node::Node;
use opsis_scene::paint::{
    ColorStop, Gradient, LayerSize, LinearDirection, MaskLayer, MaskMode, Repeat, Side,
};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Position2};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::whole;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::{filled, transition_phase};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::units::to_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct TransitionBlinds {
    #[setting(
        "Orientation",
        default = Orientation::Horizontal,
        help = "Horizontal slats stack top to bottom and sweep sideways."
    )]
    pub orientation: Orientation,
    #[setting(
        "Slats",
        default = 12,
        range(min = 2, max = 48),
        step = 1,
        help = "Number of slats. Above ~24 it stops reading as slats at 1080p."
    )]
    pub slats: u32,
    #[setting(
        "Stagger",
        default = 0.45,
        range(min = 0, max = 1),
        step = 0.01,
        help = "Share of the window the cascade spans. 0 opens every slat together.",
        advanced
    )]
    pub stagger: f64,
    #[setting(
        "Alternate",
        help = "Reverse alternate slats for a woven counter-sweep."
    )]
    pub alternate: bool,
    #[setting(
        "Edge softness",
        default = 0.06,
        range(min = 0, max = 0.5),
        step = 0.01,
        help = "Soft leading edge per slat, as a share of its length.",
        advanced
    )]
    pub edge_softness: f64,
    #[setting("Reverse sweep", advanced)]
    pub reverse: bool,
}

pub fn definition() -> ElementMeta {
    meta::<TransitionBlinds>(
        element(
            ElementKind::Presentation,
            "transition-blinds",
            "Blinds",
            "Slats open one after another to reveal the next scene.",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("lucide:blinds")),
    )
}

fn slat_progress(coverage: f64, index: f64, count: f64, stagger: f64) -> f64 {
    if count <= 1.0 || stagger <= 0.0 {
        return coverage.clamp(0.0, 1.0);
    }
    let spread = stagger.clamp(0.0, 1.0);
    let start = index / (count - 1.0) * spread;
    let span = 1.0 - spread;
    ((coverage - start) / if span > 0.0 { span } else { 1.0 }).clamp(0.0, 1.0)
}

pub fn present(props: &TransitionBlinds, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let phase = transition_phase(side.progress, side.entering, 1.0, false)?;
    let coverage = 1.0 - phase.displace;
    if !phase.entering || coverage >= 1.0 {
        return Ok(filled(Style::default(), side.children));
    }
    let count = whole(f64::from(props.slats)).max(1);
    let counted = f64::from(count);
    let horizontal = props.orientation == Orientation::Horizontal;
    let feather = props.edge_softness.max(0.0) * 100.0;
    let band = to_fixed(100.0 / counted * 1.04, 4);
    let layers: Vec<MaskLayer> = (0..count)
        .map(|index| {
            let i = f64::from(index);
            let progress = slat_progress(coverage, i, counted, props.stagger);
            let reversed = (props.alternate && index % 2 == 1) != props.reverse;
            let edge = progress.mul_add(feather.mul_add(2.0, 100.0), -feather);
            let side = match (reversed, horizontal) {
                (true, true) => Side::Left,
                (true, false) => Side::Top,
                (false, true) => Side::Right,
                (false, false) => Side::Bottom,
            };
            let (horizontal_side, vertical_side) = if horizontal { (Some(side), None) } else { (None, Some(side)) };
            let mut layer = Gradient::Linear {
                direction: LinearDirection::ToSide {
                    horizontal: horizontal_side,
                    vertical: vertical_side,
                },
                stops: vec![
                    ColorStop::at(Color::BLACK, Length::percent(to_fixed(edge - feather, 2))),
                    ColorStop::at(Color::TRANSPARENT, Length::percent(to_fixed(edge + feather, 2))),
                ],
                repeating: false,
                interpolation: ColorInterpolation::DEFAULT,
            }
            .layer();
            let offset = if count == 1 { 0.0 } else { to_fixed(i / (counted - 1.0) * 100.0, 3) };
            if horizontal {
                layer.size = LayerSize::Explicit(Length::percent(100.0), Length::percent(band));
                layer.position = Position2 { x: Length::percent(0.0), y: Length::percent(offset) };
            } else {
                layer.size = LayerSize::Explicit(Length::percent(band), Length::percent(100.0));
                layer.position = Position2 { x: Length::percent(offset), y: Length::percent(0.0) };
            }
            layer.repeat = (Repeat::NoRepeat, Repeat::NoRepeat);
            MaskLayer {
                layer,
                mode: MaskMode::Alpha,
            }
        })
        .collect();
    Ok(filled(Style::default().mask(layers), side.children))
}
