use opsis_macros::settings;
use opsis_scene::node::Node;
use opsis_scene::timeline::ChildAt;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shader::{filtered, rgb_uniform};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct HalftonePrint {
    #[setting("Dot size", default = 10, range(min = 4, max = 32), step = 1)]
    pub dot_size: u32,
    #[setting("Screen angle", range(min = 0, max = 90), step = 1, advanced)]
    pub angle: u32,
    #[setting(
        "Misregistration",
        default = 1.2,
        range(min = 0, max = 6),
        step = 0.2,
        advanced
    )]
    pub misregistration: f64,
    #[setting("Paper", default = "#f4efe4", check = colour)]
    pub paper_tint: String,
    #[setting("Cyan ink", default = "#14e0fa", widget = "color", check = colour, advanced)]
    pub cyan_ink: String,
    #[setting("Magenta ink", default = "#e61ff0", widget = "color", check = colour, advanced)]
    pub magenta_ink: String,
    #[setting("Yellow ink", default = "#f7e61a", widget = "color", check = colour, advanced)]
    pub yellow_ink: String,
}

pub fn definition() -> ElementMeta {
    meta::<HalftonePrint>(
            element(
                ElementKind::Wrapper,
                "halftone-print",
                "Halftone print",
                "Print a scene onto paper: each colour separates onto its own rotated screen of dots sized by the ink under them, slightly out of register the way a real press runs.",
                ElementCategory::Effect,
                90,
            )
                .look(NodeLook::of("lucide:newspaper")),
        )
}

const PROGRAM: &str = "halftone-print";

fn absorb(ink: &str) -> Vec<f64> {
    rgb_uniform(ink).into_iter().map(|c| 1.0 - c).collect()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn wrap(props: &HalftonePrint, child: ChildAt<'_>, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let inner = child(cx.at.stage)?;
    let height = cx.at.stage.height;
    Ok(filtered(
        PROGRAM,
        vec![
            ("cell", vec![f64::from(props.dot_size.max(1)) / height]),
            (
                "angle",
                vec![f64::from(props.angle) * std::f64::consts::PI / 180.0],
            ),
            ("shift", vec![props.misregistration / height]),
            ("paper", rgb_uniform(&props.paper_tint)),
            ("cyanAbsorb", absorb(&props.cyan_ink)),
            ("magentaAbsorb", absorb(&props.magenta_ink)),
            ("yellowAbsorb", absorb(&props.yellow_ink)),
        ],
        inner,
    ))
}
