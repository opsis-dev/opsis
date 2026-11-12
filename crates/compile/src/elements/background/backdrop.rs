use opsis_macros::settings;
use opsis_scene::node::{Node, div, img};
use opsis_scene::style::{ObjectFit, Style};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::backdrops::Backdrop;
use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct StudioBackdrop {
    #[setting("Backdrop", default = Backdrop::Glaze)]
    pub name: Backdrop,
}

pub fn definition() -> ElementMeta {
    meta::<StudioBackdrop>(
        element(
            ElementKind::Component,
            "backdrop",
            "Studio backdrop",
            "A ready-made studio background image.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:wallpaper")),
    )
}

pub fn draw(props: &StudioBackdrop, _: &Cx<'_>) -> Result<Node, DrawFailure> {
    let picture = img(
        format!("app:backdrops/{}.jpg", props.name.id()),
        Style::default()
            .width(Length::percent(100.0))
            .height(Length::percent(100.0))
            .object_fit(ObjectFit::Cover),
    );
    Ok(div(absolute_fill()).child(picture).into())
}
