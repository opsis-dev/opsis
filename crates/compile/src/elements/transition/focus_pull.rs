use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::node::Node;
use opsis_scene::paint::Filter;
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::anim::{clamped, clamped_eased};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct FocusPull {
    #[setting("Blur", default = 16, range(min = 4, max = 32), step = 1)]
    pub blur: u32,
    #[setting(
        "Glow",
        default = 30.0,
        range(min = 0, max = 100),
        unit = "%",
        advanced
    )]
    pub glow: f64,
    #[setting("Zoom", default = 5.0, range(min = 0, max = 20), unit = "%", advanced)]
    pub zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<FocusPull>(
            element(
                ElementKind::Presentation,
                "focus-pull",
                "Focus pull",
                "A rack-focus transition that defocuses the outgoing scene into a bright blur and resolves the next one into focus.",
                ElementCategory::Transition,
                46,
            )
                .look(NodeLook::of("lucide:focus")),
        )
}

const ENTER_GLOW_SHARE: f64 = 25.0 / 30.0;
const ENTER_ZOOM_SHARE: f64 = 0.6;

const fn smooth() -> Easing {
    Easing::Bezier(0.42, 0.0, 0.58, 1.0)
}

pub fn present(props: &FocusPull, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    let blur = f64::from(props.blur);
    let out_glow = 1.0 + props.glow / 100.0;
    let in_glow = (props.glow / 100.0).mul_add(ENTER_GLOW_SHARE, 1.0);
    let style = if side.entering {
        let scale = clamped_eased(
            p,
            [0.35, 1.0],
            [(props.zoom / 100.0).mul_add(-ENTER_ZOOM_SHARE, 1.0), 1.0],
            Easing::ease_out(Easing::Cubic),
        )?;
        Style::default()
            .opacity(clamped(p, [0.32, 0.52], [0.0, 1.0])?)
            .scale(scale, scale)
            .filter(vec![
                Filter::Blur(Length::px(clamped_eased(
                    p,
                    [0.35, 0.9],
                    [blur, 0.0],
                    Easing::ease_out(Easing::Quad),
                )?)),
                Filter::Brightness(Num(clamped(p, [0.4, 0.85], [in_glow, 1.0])?)),
            ])
    } else {
        let scale = clamped_eased(p, [0.0, 0.6], [1.0, 1.0 + props.zoom / 100.0], smooth())?;
        Style::default()
            .opacity(clamped_eased(p, [0.42, 0.68], [1.0, 0.0], smooth())?)
            .scale(scale, scale)
            .filter(vec![
                Filter::Blur(Length::px(clamped_eased(
                    p,
                    [0.05, 0.55],
                    [0.0, blur],
                    Easing::ease_in(Easing::Quad),
                )?)),
                Filter::Brightness(Num(clamped(p, [0.1, 0.55], [1.0, out_glow])?)),
            ])
    };
    Ok(filled(&style, side.children))
}
