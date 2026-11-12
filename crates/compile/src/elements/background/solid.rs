use opsis_macros::settings;
use opsis_scene::node::{Node, div};
use opsis_scene::timeline::absolute_fill;
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct Solid {
    #[setting("Colour", default = "#09090b", check = colour)]
    pub color: String,
}

pub fn definition() -> ElementMeta {
    meta::<Solid>(
        element(
            ElementKind::Component,
            "solid",
            "Solid colour",
            "A plain colour background.",
            ElementCategory::Background,
            90,
        )
        .look(NodeLook::of("lucide:paint-bucket")),
    )
}

pub fn draw(props: &Solid, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    Ok(div(absolute_fill().background_color(cx.color(&props.color))).into())
}
