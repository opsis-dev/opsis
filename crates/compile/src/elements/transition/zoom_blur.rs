use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::Style;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::filled;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct ZoomBlur {
    #[setting("Blur", default = 16, range(min = 0, max = 40), step = 2, unit = "px")]
    pub blur: u32,
    #[setting("Rise", range(min = 0, max = 160), step = 10, unit = "px")]
    pub rise: u32,
    #[setting("Zoom", default = 1.12, range(min = 1, max = 2), unit = "×", advanced)]
    pub zoom: f64,
}

pub fn definition() -> ElementMeta {
    meta::<ZoomBlur>(
            element(
                ElementKind::Presentation,
                "zoom-blur",
                "Zoom blur",
                "A depth punch-in transition that pushes the outgoing scene through the viewer while the incoming one resolves out of blur.",
                ElementCategory::Transition,
                18,
            )
                .look(NodeLook::of("lucide:expand")),
        )
}

pub fn present(props: &ZoomBlur, side: Transition, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let p = side.progress;
    let (blur, rise) = (f64::from(props.blur), f64::from(props.rise));
    let style = if side.entering {
        let scale = p.mul_add(0.1, 0.9);
        let mut moves = vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))];
        if rise > 0.0 {
            moves.push(Transform::Translate(Length::ZERO, Length::px((1.0 - p) * rise), Length::ZERO));
        }
        let style = Style::default().opacity(p).transform(moves);
        if p < 1.0 {
            style.filter(vec![Filter::Blur(Length::px((1.0 - p) * blur))])
        } else {
            style
        }
    } else {
        let scale = p.mul_add(props.zoom - 1.0, 1.0);
        let mut moves = vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))];
        if rise > 0.0 {
            moves.push(Transform::Translate(Length::ZERO, Length::px(-p * rise), Length::ZERO));
        }
        let style = Style::default().transform(moves);
        if p > 0.0 {
            style.filter(vec![Filter::Blur(Length::px(p * blur))])
        } else {
            style
        }
    };
    Ok(filled(style, side.children))
}
