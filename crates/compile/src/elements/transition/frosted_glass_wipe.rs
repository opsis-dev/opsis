use opsis_macros::{choice, settings};
use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div};
use opsis_scene::paint::{ClipPath, ColorStop, Filter, Gradient, LinearDirection, Shadow, Side};
use opsis_scene::style::{BorderSide, BorderStyle, Position, Style};
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::transition::present::{filled, transition_phase};
use crate::elements::units::to_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum WipeFrom {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

#[settings(export)]
pub struct FrostedGlassWipe {
    #[setting("Blur", default = 20.0, help = "Frost panel blur radius.")]
    pub blur: f64,
    #[setting(
        "Panel width",
        default = 0.14,
        step = 0.01,
        help = "Sweep panel width as fraction of frame.",
        advanced
    )]
    pub panel_width: f64,
    #[setting("Direction", default = WipeFrom::FromLeft, help = "Sweep direction.")]
    pub direction: WipeFrom,
    #[setting(
        "Frost color",
        default = "rgba(255,255,255,0.12)",
        help = "Tint of the glass panel.",
        check = colour,
        advanced
    )]
    pub frost_color: String,
    #[setting("Edge colour", default = "rgba(255,255,255,0.22)", widget = "color", check = colour, advanced)]
    pub edge_color: String,
    #[setting(
        "Glow",
        default = 28,
        range(min = 0, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub glow: u32,
}

pub fn definition() -> ElementMeta {
    meta::<FrostedGlassWipe>(
        element(
            ElementKind::Presentation,
            "frosted-glass-wipe",
            "Frosted glass wipe",
            "A pane of frosted glass crosses the frame and leaves the next scene behind it.",
            ElementCategory::Transition,
            24,
        )
        .look(NodeLook::of("tabler:blur")),
    )
}

pub fn present(
    props: &FrostedGlassWipe,
    side: Transition,
    cx: &Cx<'_>,
) -> Result<Node, DrawFailure> {
    let phase = transition_phase(side.progress, side.entering, 1.0, false)?;
    if !phase.entering {
        return Ok(filled(&Style::default(), side.children));
    }
    let swept = 1.0 - phase.displace;
    let vertical = matches!(props.direction, WipeFrom::FromTop | WipeFrom::FromBottom);
    let hidden = Length::percent((1.0 - swept) * 100.0);
    let zero = Length::ZERO;
    let edges = match props.direction {
        WipeFrom::FromTop => Edges {
            top: zero.clone(),
            right: zero.clone(),
            bottom: hidden,
            left: zero,
        },
        WipeFrom::FromBottom => Edges {
            top: hidden,
            right: zero.clone(),
            bottom: zero.clone(),
            left: zero,
        },
        WipeFrom::FromLeft => Edges {
            top: zero.clone(),
            right: hidden,
            bottom: zero.clone(),
            left: zero,
        },
        WipeFrom::FromRight => Edges {
            top: zero.clone(),
            right: zero.clone(),
            bottom: zero,
            left: hidden,
        },
    };
    let reveal = Style::default().clip_path(ClipPath::Inset { edges, round: None });
    let width = props.panel_width * 100.0;
    let offset = swept.mul_add(100.0 + width, -width);
    let glow = interpolate(
        swept,
        &[0.0, 0.15, 0.85, 1.0],
        &[0.0, 1.0, 1.0, 0.0],
        &Interpolation::clamped(),
    )?;
    let edge_color = Color::mix(
        ColorInterpolation {
            space: ColorSpace::Srgb,
            ..ColorInterpolation::DEFAULT
        },
        (cx.color(&props.edge_color), Some(to_fixed(glow * 100.0, 1))),
        (Color::TRANSPARENT, None),
    );
    let line = BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(edge_color),
    };
    let none = BorderSide::default();
    let (mut panel, side_of_glass) = if vertical {
        (
            Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .right(Length::ZERO)
                .top(Length::percent(offset))
                .height(Length::percent(width)),
            if props.direction == WipeFrom::FromTop {
                Side::Bottom
            } else {
                Side::Top
            },
        )
    } else {
        (
            Style::default()
                .position(Position::Absolute)
                .top(Length::ZERO)
                .bottom(Length::ZERO)
                .left(Length::percent(offset))
                .width(Length::percent(width)),
            if props.direction == WipeFrom::FromLeft {
                Side::Right
            } else {
                Side::Left
            },
        )
    };
    let mut borders = Edges::all(BorderSide {
        width: Length::ZERO,
        ..none
    });
    match side_of_glass {
        Side::Top => borders.top = line,
        Side::Bottom => borders.bottom = line,
        Side::Left => borders.left = line,
        Side::Right => borders.right = line,
    }
    let toward = match props.direction {
        WipeFrom::FromLeft => Side::Right,
        WipeFrom::FromRight => Side::Left,
        WipeFrom::FromTop => Side::Bottom,
        WipeFrom::FromBottom => Side::Top,
    };
    let (horizontal, vertical_side) = if vertical {
        (None, Some(toward))
    } else {
        (Some(toward), None)
    };
    let frost = Gradient::Linear {
        direction: LinearDirection::ToSide {
            horizontal,
            vertical: vertical_side,
        },
        stops: vec![
            ColorStop::new(Color::TRANSPARENT),
            ColorStop::new(cx.color(&props.frost_color)),
            ColorStop::new(Color::TRANSPARENT),
        ],
        repeating: false,
        interpolation: ColorInterpolation::DEFAULT,
    };
    panel = panel
        .border(borders)
        .backdrop_filter(vec![Filter::Blur(Length::px(props.blur))])
        .background(vec![frost.layer()])
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::px(to_fixed(f64::from(props.glow) * glow, 1)),
            spread: Length::ZERO,
            color: Color::srgb_alpha(1.0, 1.0, 1.0, to_fixed(0.18 * glow, 3)),
        }])
        .opacity(glow);
    Ok(filled(
        &Style::default(),
        vec![filled(&reveal, side.children), div(panel).into()],
    ))
}
